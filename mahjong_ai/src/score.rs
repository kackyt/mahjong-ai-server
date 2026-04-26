use crate::state::AIStateWrapper;
use crate::utils::{get_dist_coef, get_kind_coef, paidistance};
use dashmap::DashMap;
use mahjong_core::agari::AgariBehavior;
use mahjong_core::mahjong_generated::open_mahjong::{Mentsu, MentsuFlag, MentsuPai, MentsuType};
use mahjong_core::shanten::PaiState;

use std::sync::Arc;

/// 探索コンテキスト
/// DashMap を使うことでスレッドセーフにキャッシュを共有できる
#[derive(Clone)]
pub struct SearchContext<'a> {
    pub wrapper: &'a AIStateWrapper<'a>,
    pub shanten_base: i32,
    pub nokori_sum: f64,
    pub hand_counts: [u8; 34],
    pub machi_cache: Arc<DashMap<([u8; 34], usize), f64>>,
}

/// 待ち係数の計算（シャンテン0の受入枚数ベースの評価）
pub fn calc_machi_coef(ctx: &SearchContext, current_counts: &[u8; 34], machi_hai: usize) -> f64 {
    // DashMap でキャッシュを参照（並列安全）
    if let Some(cached) = ctx.machi_cache.get(&(*current_counts, machi_hai)) {
        return *cached;
    }

    let mut temp_counts = *current_counts;
    if temp_counts[machi_hai] > 0 {
        temp_counts[machi_hai] -= 1;
    } else {
        return 0.0;
    }

    let mut pstate = PaiState::default();
    for (i, &count) in temp_counts.iter().enumerate().take(34) {
        match i {
            0..=8 => pstate.hai_count_m[i] = count as i32,
            9..=17 => pstate.hai_count_p[i - 9] = count as i32,
            18..=26 => pstate.hai_count_s[i - 18] = count as i32,
            27..=33 => pstate.hai_count_z[i - 27] = count as i32,
            _ => {}
        }
    }

    let player = &ctx.wrapper.game_state.players[ctx.wrapper.game_state.teban as usize];
    let n_fulo = player.mentsu_len as usize;

    let mut num = 0.0;
    let mut furiten = false;
    let my_kawa = &player.kawahai;
    let kawa_len = player.kawahai_len as usize;

    for (i, &count) in temp_counts.iter().enumerate().take(34) {
        // 残り牌がゼロならスキップ
        let avail_in_wall = ctx.wrapper.nokorihai[i];

        let used_from_wall = count.saturating_sub(ctx.hand_counts[i]);

        let left = avail_in_wall - (used_from_wall as f64);

        if left <= 0.0 {
            continue;
        }

        match i {
            0..=8 => pstate.hai_count_m[i] += 1,
            9..=17 => pstate.hai_count_p[i - 9] += 1,
            18..=26 => pstate.hai_count_s[i - 18] += 1,
            27..=33 => pstate.hai_count_z[i - 27] += 1,
            _ => {}
        }

        let s = pstate.get_standard_shanten(n_fulo);

        match i {
            0..=8 => pstate.hai_count_m[i] -= 1,
            9..=17 => pstate.hai_count_p[i - 9] -= 1,
            18..=26 => pstate.hai_count_s[i - 18] -= 1,
            27..=33 => pstate.hai_count_z[i - 27] -= 1,
            _ => {}
        }

        if s == -1 {
            if my_kawa[0..kawa_len].iter().any(|p| p.pai_num as usize == i) {
                furiten = true;
            }
            num += left;
        }
    }

    let ret = num / 5.0;
    let mut final_ret = ret;
    if furiten {
        final_ret *= 0.33;
    }
    // DashMap にキャッシュ挿入（並列安全）
    ctx.machi_cache
        .insert((*current_counts, machi_hai), final_ret);
    final_ret
}

pub fn calc_score(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    mentsu_list: &[Mentsu],
    diff: i32,
) -> f64 {
    calc_score_inner(ctx, current_counts, mentsu_list, diff)
}

/// スコア計算の内部実装
/// 頭候補のフィルタリング強化・確率閾値による早期リターンで高速化
fn calc_score_inner(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    mentsu_list: &[Mentsu],
    diff: i32,
) -> f64 {
    let mut max_val = 0.0;

    let player = &ctx.wrapper.game_state.players[ctx.wrapper.game_state.teban as usize];
    let open_mentsu: Vec<Mentsu> = player
        .mentsu
        .iter()
        .take(player.mentsu_len as usize)
        .map(|m| m.pack())
        .collect();

    for head_pai in 0..34 {
        // 最適化: 手牌にも残り牌にもない牌はスキップ
        if ctx.wrapper.remain_counts[head_pai] == 0 && ctx.hand_counts[head_pai] == 0 {
            continue;
        }

        // 最適化: 手牌0枚かつ残り1枚以下 → 頭にする確率が極めて低いためスキップ
        if ctx.hand_counts[head_pai] == 0 && ctx.wrapper.remain_counts[head_pai] <= 1 {
            continue;
        }

        if current_counts[head_pai] + 2 > 4 {
            continue;
        }

        current_counts[head_pai] += 2;

        let mut probability = 1.0;
        let mut rest = ctx.nokori_sum;
        let mut possible = true;

        for (i, &count) in current_counts.iter().enumerate().take(34) {
            if count > ctx.hand_counts[i] {
                let needed = (count - ctx.hand_counts[i]) as i32;

                let avail_in_wall = ctx.wrapper.nokorihai[i];

                if (needed as u8) > ctx.wrapper.remain_counts[i] {
                    possible = false;
                    break;
                }

                let dist = paidistance(&ctx.hand_counts, i);
                let dist_c = get_dist_coef(dist);
                let kind_c = get_kind_coef(ctx.wrapper.game_state, i);

                for _ in 0..needed {
                    if rest <= 0.0 {
                        probability = 0.0;
                        break;
                    }
                    probability *= (avail_in_wall) / rest;
                    probability *= dist_c;
                    probability *= kind_c;
                    rest -= 1.0;
                }
                if probability == 0.0 {
                    possible = false;
                    break;
                }
            }
        }

        // 最適化: 確率が極めて低い場合は和了判定をスキップ
        if !possible || probability <= 1e-10 {
            current_counts[head_pai] -= 2;
            continue;
        }

        let head_pai_obj = MentsuPai::new(head_pai as u8, 0, MentsuFlag::FLAG_NONE);
        let dummy = MentsuPai::default();
        let head_mentsu = Mentsu::new(
            &[head_pai_obj, head_pai_obj, dummy, dummy],
            2,
            MentsuType::TYPE_ATAMA,
        );

        let mut full_mentsu = mentsu_list.to_owned();
        full_mentsu.push(head_mentsu);

        for (idx, m) in full_mentsu.iter().enumerate() {
            let first_pai = m.pai_list().get(0).pai_num() as usize;
            let mut is_machi_candidate = false;
            let mut machi_hai_candidate = 0;

            match m.mentsu_type() {
                MentsuType::TYPE_KOUTSU | MentsuType::TYPE_ATAMA
                    if current_counts[first_pai] > ctx.hand_counts[first_pai] =>
                {
                    is_machi_candidate = true;
                    machi_hai_candidate = first_pai;
                }
                MentsuType::TYPE_SHUNTSU => {
                    if current_counts[first_pai] > ctx.hand_counts[first_pai] {
                        is_machi_candidate = true;
                        machi_hai_candidate = first_pai;
                    } else if current_counts[first_pai + 1] > ctx.hand_counts[first_pai + 1] {
                        is_machi_candidate = true;
                        machi_hai_candidate = first_pai + 1;
                    } else if current_counts[first_pai + 2] > ctx.hand_counts[first_pai + 2] {
                        is_machi_candidate = true;
                        machi_hai_candidate = first_pai + 2;
                    }
                }
                _ => {}
            }

            if is_machi_candidate {
                let mut test_mentsu = full_mentsu.clone();
                let mut m_mod = test_mentsu[idx].unpack();
                for p in &mut m_mod.pai_list {
                    if p.pai_num as usize == machi_hai_candidate {
                        p.flag = MentsuFlag::FLAG_AGARI;
                        break;
                    }
                }
                test_mentsu[idx] = m_mod.pack();

                let agari_state = ctx.wrapper.game_state.get_agari(
                    ctx.wrapper.game_state.teban as usize,
                    &test_mentsu,
                    &open_mentsu,
                    false,
                );

                let mut yakus = ctx
                    .wrapper
                    .game_state
                    .get_condition_yaku(ctx.wrapper.game_state.teban as usize, &agari_state);
                yakus.extend(agari_state.get_yaku_list());
                yakus.extend(ctx.wrapper.game_state.get_dora_yaku(
                    ctx.wrapper.game_state.teban as usize,
                    &test_mentsu,
                    &open_mentsu,
                    0,
                ));

                let agari_res = agari_state.get_agari(&yakus);
                let score = agari_res.score as f64;
                if score > 0.0 {
                    let machi_coef = calc_machi_coef(ctx, current_counts, machi_hai_candidate);
                    let term = 0.8 * probability * probability * score + 0.2 * probability * score;
                    let val = machi_coef * term / (diff as f64).max(1.0);
                    max_val += val;
                }
            }
        }

        current_counts[head_pai] -= 2;
    }

    max_val
}

#[allow(clippy::too_many_arguments)]
pub fn shuntsu_point(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    current_mentsu: &mut Vec<Mentsu>,
    _koutsu_num: i32,
    shuntsu_num: i32,
    _koutsu_pos: usize,
    shuntsu_pos: usize,
    diff: i32,
) -> f64 {
    if diff > ctx.shanten_base + 2 || diff >= 7 {
        return 0.0;
    }

    if shuntsu_num <= 0 {
        return calc_score(ctx, current_counts, current_mentsu, diff);
    }

    let mut ret = 0.0;

    // 字牌は順子を構成できないためスキップ
    for i in shuntsu_pos..21 {
        let pai = (i / 7) * 9 + (i % 7);
        // 手牌か残り牌にあるかの簡易チェック
        let p1_ok = ctx.hand_counts[pai] > 0 || ctx.wrapper.remain_counts[pai] > 0;
        let p2_ok = ctx.hand_counts[pai + 1] > 0 || ctx.wrapper.remain_counts[pai + 1] > 0;
        let p3_ok = ctx.hand_counts[pai + 2] > 0 || ctx.wrapper.remain_counts[pai + 2] > 0;

        if !p1_ok || !p2_ok || !p3_ok {
            continue;
        }

        let check_avail = |p: usize| -> bool {
            let used = current_counts[p] + 1;
            let have = ctx.hand_counts[p];
            let remain = ctx.wrapper.remain_counts[p];
            if used > have {
                let need = used - have;
                if need > remain {
                    return false;
                }
            }
            true
        };

        if check_avail(pai) && check_avail(pai + 1) && check_avail(pai + 2) {
            current_counts[pai] += 1;
            current_counts[pai + 1] += 1;
            current_counts[pai + 2] += 1;

            let p1 = MentsuPai::new(pai as u8, 0, MentsuFlag::FLAG_NONE);
            let p2 = MentsuPai::new((pai + 1) as u8, 0, MentsuFlag::FLAG_NONE);
            let p3 = MentsuPai::new((pai + 2) as u8, 0, MentsuFlag::FLAG_NONE);
            let dummy = MentsuPai::default();

            let m = Mentsu::new(&[p1, p2, p3, dummy], 3, MentsuType::TYPE_SHUNTSU);
            current_mentsu.push(m);

            let mut added_diff = 0;
            if current_counts[pai] > ctx.hand_counts[pai] {
                added_diff += 1;
            }
            if current_counts[pai + 1] > ctx.hand_counts[pai + 1] {
                added_diff += 1;
            }
            if current_counts[pai + 2] > ctx.hand_counts[pai + 2] {
                added_diff += 1;
            }

            ret += shuntsu_point(
                ctx,
                current_counts,
                current_mentsu,
                _koutsu_num,
                shuntsu_num - 1,
                _koutsu_pos,
                i,
                diff + added_diff,
            );

            current_mentsu.pop();
            current_counts[pai] -= 1;
            current_counts[pai + 1] -= 1;
            current_counts[pai + 2] -= 1;
        }
    }

    ret
}

#[allow(clippy::too_many_arguments)]
pub fn koutsu_point(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    current_mentsu: &mut Vec<Mentsu>,
    koutsu_num: i32,
    shuntsu_num: i32,
    koutsu_pos: usize,
    shuntsu_pos: usize,
    diff: i32,
) -> f64 {
    if diff > ctx.shanten_base + 2 || diff >= 7 {
        return 0.0;
    }

    let mut ret = 0.0;

    for i in koutsu_pos..34 {
        if ctx.hand_counts[i] == 0 {
            continue;
        }

        let check_avail = |p: usize| -> bool {
            let used = current_counts[p] + 3;
            let have = ctx.hand_counts[p];
            let remain = ctx.wrapper.remain_counts[p];
            if used > have {
                let need = used - have;
                if need > remain {
                    return false;
                }
            }
            true
        };

        if check_avail(i) {
            current_counts[i] += 3;
            let p = MentsuPai::new(i as u8, 0, MentsuFlag::FLAG_NONE);
            let dummy = MentsuPai::default();
            let m = Mentsu::new(&[p, p, p, dummy], 3, MentsuType::TYPE_KOUTSU);
            current_mentsu.push(m);

            // added_diff: 枝刈り用の差分計算。
            // current_counts[i]は既に+3済みの値。さらに将来的な利用を
            // 仮定してk枚追加した場合に hand_counts を超えるかを確認する。
            // この計算は意図的に保守的（大きめ）にすることで
            // 枝刈りを積極的に働かせる設計になっている。
            let added_diff = if current_counts[i] > ctx.hand_counts[i] {
                (current_counts[i] - ctx.hand_counts[i]) as i32
            } else {
                0
            };

            if koutsu_num - 1 > 0 {
                ret += koutsu_point(
                    ctx,
                    current_counts,
                    current_mentsu,
                    koutsu_num - 1,
                    shuntsu_num,
                    i + 1,
                    shuntsu_pos,
                    diff + added_diff,
                );
            } else {
                ret += shuntsu_point(
                    ctx,
                    current_counts,
                    current_mentsu,
                    0,
                    shuntsu_num,
                    i + 1,
                    shuntsu_pos,
                    diff + added_diff,
                );
            }

            current_mentsu.pop();
            current_counts[i] -= 3;
        }
    }

    ret
}

pub fn chiitoi_point(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    cnt: i32,
    pos: usize,
) -> f64 {
    if cnt == 7 {
        let mut probability = 1.0;
        let mut rest = ctx.nokori_sum;

        for (i, &count) in current_counts.iter().enumerate().take(34) {
            if count > ctx.hand_counts[i] {
                let needed = (count - ctx.hand_counts[i]) as i32;
                let avail_in_wall = ctx.wrapper.nokorihai[i];
                if (needed as u8) > ctx.wrapper.remain_counts[i] {
                    return 0.0;
                }

                let dist = paidistance(&ctx.hand_counts, i);
                let dist_c = get_dist_coef(dist);
                let kind_c = get_kind_coef(ctx.wrapper.game_state, i);

                for _ in 0..needed {
                    if rest <= 0.0 {
                        probability = 0.0;
                        break;
                    }
                    probability *= (avail_in_wall) / rest;
                    probability *= dist_c;
                    probability *= kind_c;
                    rest -= 1.0;
                }
                if probability == 0.0 {
                    return 0.0;
                }
            }
        }

        if probability <= 0.0 {
            return 0.0;
        }

        let mut mentsu_vec = Vec::with_capacity(7);
        for (i, &count) in current_counts.iter().enumerate().take(34) {
            if count >= 2 {
                let p = MentsuPai::new(i as u8, 0, MentsuFlag::FLAG_NONE);
                let dummy = MentsuPai::default();
                mentsu_vec.push(Mentsu::new(
                    &[p, p, dummy, dummy],
                    2,
                    MentsuType::TYPE_ATAMA,
                ));
            }
        }

        if !mentsu_vec.is_empty() {
            let mut test_mentsu = mentsu_vec.clone();
            let mut m = test_mentsu[0].unpack();
            m.pai_list[0].flag = MentsuFlag::FLAG_AGARI;
            test_mentsu[0] = m.pack();

            let agari_state = ctx.wrapper.game_state.get_agari(
                ctx.wrapper.game_state.teban as usize,
                &test_mentsu,
                &[],
                false,
            );
            let mut yakus = ctx
                .wrapper
                .game_state
                .get_condition_yaku(ctx.wrapper.game_state.teban as usize, &agari_state);
            yakus.extend(agari_state.get_yaku_list());
            yakus.extend(ctx.wrapper.game_state.get_dora_yaku(
                ctx.wrapper.game_state.teban as usize,
                &test_mentsu,
                &[],
                0,
            ));
            let agari_res = agari_state.get_agari(&yakus);
            let score = agari_res.score as f64;

            return 0.8 * probability * probability * score + 0.2 * probability * score;
        }
        return 0.0;
    }

    let mut sum = 0.0;

    for i in pos..34 {
        // 手牌に0枚で2枚引く必要がある場合は確率が極めて低いためスキップ
        // 1枚以上持っている場合のみ七対子候補として考慮

        let have = ctx.hand_counts[i];
        if have >= 2 {
            if current_counts[i] == 0 {
                current_counts[i] = 2;
                sum += chiitoi_point(ctx, current_counts, cnt + 1, i + 1);
                current_counts[i] = 0;
            }
        } else if have == 1 {
            let need = 1;
            if ctx.wrapper.remain_counts[i] >= need {
                current_counts[i] = 2;
                sum += chiitoi_point(ctx, current_counts, cnt + 1, i + 1);
                current_counts[i] = 0;
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AIStateWrapper;
    use mahjong_core::mahjong_generated::open_mahjong::GameStateT;
    use mahjong_core::play_log::PlayLog;

    #[test]
    fn test_calc_score_boundary_conditions() {
        let mut game_state = GameStateT::default();
        let mut play_log = PlayLog::new();
        game_state.create(b"test", 1, &mut play_log);
        game_state.shuffle();
        game_state.start(&mut play_log);

        let mut hand_counts = [0; 34];
        for item in hand_counts.iter_mut().take(4) {
            *item = 3;
        }

        let mut wrapper = AIStateWrapper::new(&game_state);
        let mentsu_list = Vec::new();

        // 1. rest == 0 の場合 (nokori_sum = 0)
        {
            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: 0,
                nokori_sum: 0.0,
                hand_counts,
                machi_cache: Arc::new(DashMap::new()),
            };
            let mut current_counts = hand_counts;
            let score = calc_score(&ctx, &mut current_counts, &mentsu_list, 0);
            assert_eq!(score, 0.0, "nokori_sum が 0 の時はスコア 0.0 であるべき");
        }

        // 2. rest < 0 の場合 (通常は発生しないが、ロジックとして検証)
        {
            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: 0,
                nokori_sum: -1.0,
                hand_counts,
                machi_cache: Arc::new(DashMap::new()),
            };
            let mut current_counts = hand_counts;
            let score = calc_score(&ctx, &mut current_counts, &mentsu_list, 0);
            assert_eq!(score, 0.0, "nokori_sum が負の時はスコア 0.0 であるべき");
        }

        // 3. remain_counts 不足ケース
        {
            for i in 0..34 {
                wrapper.remain_counts[i] = 0;
            }
            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: 0,
                nokori_sum: 100.0,
                hand_counts,
                machi_cache: Arc::new(DashMap::new()),
            };
            let mut current_counts = [0; 34];
            let score = calc_score(&ctx, &mut current_counts, &mentsu_list, 0);
            assert_eq!(
                score, 0.0,
                "remain_counts がすべて 0 の時はスコア 0.0 であるべき"
            );
        }
    }

    #[test]
    fn test_chiitoi_point_boundary_conditions() {
        let mut game_state = GameStateT::default();
        let mut play_log = PlayLog::new();
        game_state.create(b"test", 1, &mut play_log);
        game_state.start(&mut play_log);

        let hand_counts = [0; 34];
        let mut wrapper = AIStateWrapper::new(&game_state);

        // 1. rest == 0 の場合
        {
            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: 0,
                nokori_sum: 0.0,
                hand_counts,
                machi_cache: Arc::new(DashMap::new()),
            };
            let mut current_counts = [0; 34];
            for item in current_counts.iter_mut().take(7) {
                *item = 2;
            }
            let score = chiitoi_point(&ctx, &mut current_counts, 7, 0);
            assert_eq!(score, 0.0, "rest が 0 の時は chiitoi スコア 0.0 であるべき");
        }

        // 2. probability == 0.0 のケース
        {
            for i in 0..34 {
                wrapper.remain_counts[i] = 0;
                wrapper.nokorihai[i] = 0.0;
            }
            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: 0,
                nokori_sum: 100.0,
                hand_counts,
                machi_cache: Arc::new(DashMap::new()),
            };
            let mut current_counts = [0; 34];
            current_counts[0] = 2;
            let score = chiitoi_point(&ctx, &mut current_counts, 7, 0);
            assert_eq!(
                score, 0.0,
                "牌が足りない場合は chiitoi スコア 0.0 であるべき"
            );
        }
    }
}
