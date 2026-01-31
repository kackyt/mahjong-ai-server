use crate::state::AIStateWrapper;
use crate::utils::{get_dist_coef, get_kind_coef, paidistance};
use mahjong_core::agari::AgariBehavior;
use mahjong_core::mahjong_generated::open_mahjong::{
    Mentsu, MentsuFlag, MentsuPai, MentsuType,
};
use mahjong_core::shanten::PaiState;

#[derive(Clone)]
pub struct SearchContext<'a> {
    pub wrapper: &'a AIStateWrapper<'a>,
    pub shanten_base: i32,
    pub nokori_sum: f64,
    pub hand_counts: [u8; 34],
}

pub fn calc_machi_coef(
    ctx: &SearchContext,
    current_counts: &[u8; 34],
    machi_hai: usize,
) -> f64 {
    let mut temp_counts = *current_counts;
    if temp_counts[machi_hai] > 0 {
        temp_counts[machi_hai] -= 1;
    } else {
        return 0.0;
    }

    let mut pstate = PaiState::default();
    for i in 0..34 {
        match i {
            0..=8 => pstate.hai_count_m[i] = temp_counts[i] as i32,
            9..=17 => pstate.hai_count_p[i - 9] = temp_counts[i] as i32,
            18..=26 => pstate.hai_count_s[i - 18] = temp_counts[i] as i32,
            27..=33 => pstate.hai_count_z[i - 27] = temp_counts[i] as i32,
            _ => {}
        }
    }

    let player = &ctx.wrapper.game_state.players[ctx.wrapper.game_state.teban as usize];
    let n_fulo = player.mentsu_len as usize;

    let mut num = 0.0;
    let mut furiten = false;
    let my_kawa = &player.kawahai;
    let kawa_len = player.kawahai_len as usize;

    for i in 0..34 {
        match i {
            0..=8 => pstate.hai_count_m[i] += 1,
            9..=17 => pstate.hai_count_p[i - 9] += 1,
            18..=26 => pstate.hai_count_s[i - 18] += 1,
            27..=33 => pstate.hai_count_z[i - 27] += 1,
            _ => {}
        }

        let s = pstate.get_shanten(n_fulo);

        if s == -1 {
            if my_kawa[0..kawa_len].iter().any(|p| p.pai_num as usize == i) {
                furiten = true;
            }

            let used_from_wall = if temp_counts[i] > ctx.hand_counts[i] {
                temp_counts[i] - ctx.hand_counts[i]
            } else { 0 };

            let left = ctx.wrapper.remain_counts[i] as i32 - used_from_wall as i32;

            if left > 0 {
                 num += left as f64;
            }
        }

        match i {
            0..=8 => pstate.hai_count_m[i] -= 1,
            9..=17 => pstate.hai_count_p[i - 9] -= 1,
            18..=26 => pstate.hai_count_s[i - 18] -= 1,
            27..=33 => pstate.hai_count_z[i - 27] -= 1,
            _ => {}
        }
    }

    let mut ret = num / 5.0;
    if furiten {
        ret *= 0.33;
    }
    ret
}

pub fn calc_score(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    mentsu_list: &Vec<Mentsu>,
    diff: i32,
) -> f64 {
    let mut max_val = 0.0;

    let player = &ctx.wrapper.game_state.players[ctx.wrapper.game_state.teban as usize];
    let open_mentsu: Vec<Mentsu> = player.mentsu.iter()
        .take(player.mentsu_len as usize)
        .map(|m| m.pack())
        .collect();

    for head_pai in 0..34 {
        if current_counts[head_pai] + 2 > 4 {
            continue;
        }

        current_counts[head_pai] += 2;

        let mut probability = 1.0;
        let mut rest = ctx.nokori_sum;
        let mut possible = true;

        for i in 0..34 {
            if current_counts[i] > ctx.hand_counts[i] {
                let needed = (current_counts[i] - ctx.hand_counts[i]) as i32;
                let avail_in_wall = ctx.wrapper.remain_counts[i] as i32;

                if needed > avail_in_wall {
                    possible = false;
                    break;
                }

                let dist = paidistance(&ctx.hand_counts, i);
                let dist_c = get_dist_coef(dist);
                let kind_c = get_kind_coef(ctx.wrapper.game_state, i);

                for _ in 0..needed {
                    probability *= (avail_in_wall as f64) / rest;
                    probability *= dist_c;
                    probability *= kind_c;
                    rest -= 1.0;
                }
            }
        }

        if possible {
             let head_pai_obj = MentsuPai::new(head_pai as u8, 0, MentsuFlag::FLAG_NONE);
             let dummy = MentsuPai::default();
             let head_mentsu = Mentsu::new(
                &[head_pai_obj, head_pai_obj, dummy, dummy],
                2,
                MentsuType::TYPE_ATAMA
             );

             let mut full_mentsu = mentsu_list.clone();
             full_mentsu.push(head_mentsu);

             for (idx, m) in full_mentsu.iter().enumerate() {
                 let first_pai = m.pai_list().get(0).pai_num() as usize;
                 let mut is_machi_candidate = false;
                 let mut machi_hai_candidate = 0;

                 match m.mentsu_type() {
                     MentsuType::TYPE_KOUTSU | MentsuType::TYPE_ATAMA => {
                         if current_counts[first_pai] > ctx.hand_counts[first_pai] {
                             is_machi_candidate = true;
                             machi_hai_candidate = first_pai;
                         }
                     },
                     MentsuType::TYPE_SHUNTSU => {
                         if current_counts[first_pai] > ctx.hand_counts[first_pai] {
                             is_machi_candidate = true;
                             machi_hai_candidate = first_pai;
                         } else if current_counts[first_pai+1] > ctx.hand_counts[first_pai+1] {
                             is_machi_candidate = true;
                             machi_hai_candidate = first_pai + 1;
                         } else if current_counts[first_pai+2] > ctx.hand_counts[first_pai+2] {
                             is_machi_candidate = true;
                             machi_hai_candidate = first_pai + 2;
                         }
                     },
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
                         false
                     );

                     let mut yakus = ctx.wrapper.game_state.get_condition_yaku(
                         ctx.wrapper.game_state.teban as usize,
                         &agari_state
                     );
                     yakus.extend(agari_state.get_yaku_list());
                     yakus.extend(ctx.wrapper.game_state.get_dora_yaku(
                         ctx.wrapper.game_state.teban as usize,
                         &test_mentsu,
                         &open_mentsu,
                         0
                     ));

                     let agari_res = agari_state.get_agari(&yakus);
                     let score = agari_res.score as f64;
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

pub fn shuntsu_point(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    current_mentsu: &mut Vec<Mentsu>,
    koutsu_num: i32,
    shuntsu_num: i32,
    koutsu_pos: usize,
    shuntsu_pos: usize,
) -> f64 {
    let mut diff = 0;
    for i in 0..34 {
        if current_counts[i] > ctx.hand_counts[i] {
            diff += (current_counts[i] - ctx.hand_counts[i]) as i32;
        }
    }

    if diff > ctx.shanten_base + 2 || diff >= 7 {
        return 0.0;
    }

    if shuntsu_num <= 0 {
        return calc_score(ctx, current_counts, current_mentsu, diff);
    }

    let mut ret = 0.0;

    for i in shuntsu_pos..21 {
        let pai = (i / 7) * 9 + (i % 7);
        if ctx.hand_counts[pai] == 0
           && ctx.hand_counts[pai+1] == 0
           && ctx.hand_counts[pai+2] == 0 {
               continue;
        }

        let check_avail = |p: usize| -> bool {
             let used = current_counts[p] + 1;
             let have = ctx.hand_counts[p];
             let remain = ctx.wrapper.remain_counts[p];
             if used > have {
                 let need = used - have;
                 if need > remain { return false; }
             }
             true
        };

        if check_avail(pai) && check_avail(pai+1) && check_avail(pai+2) {
             current_counts[pai] += 1;
             current_counts[pai+1] += 1;
             current_counts[pai+2] += 1;

             let p1 = MentsuPai::new(pai as u8, 0, MentsuFlag::FLAG_NONE);
             let p2 = MentsuPai::new((pai+1) as u8, 0, MentsuFlag::FLAG_NONE);
             let p3 = MentsuPai::new((pai+2) as u8, 0, MentsuFlag::FLAG_NONE);
             let dummy = MentsuPai::default();

             let m = Mentsu::new(
                 &[p1, p2, p3, dummy],
                 3,
                 MentsuType::TYPE_SHUNTSU
             );
             current_mentsu.push(m);

             ret += shuntsu_point(ctx, current_counts, current_mentsu, koutsu_num, shuntsu_num - 1, koutsu_pos, i);

             current_mentsu.pop();
             current_counts[pai] -= 1;
             current_counts[pai+1] -= 1;
             current_counts[pai+2] -= 1;
        }
    }

    ret
}

pub fn koutsu_point(
    ctx: &SearchContext,
    current_counts: &mut [u8; 34],
    current_mentsu: &mut Vec<Mentsu>,
    koutsu_num: i32,
    shuntsu_num: i32,
    koutsu_pos: usize,
    shuntsu_pos: usize,
) -> f64 {
    let mut diff = 0;
    for i in 0..34 {
        if current_counts[i] > ctx.hand_counts[i] {
            diff += (current_counts[i] - ctx.hand_counts[i]) as i32;
        }
    }

    if diff > ctx.shanten_base + 2 || diff >= 7 {
        return 0.0;
    }

    let mut ret = 0.0;

    for i in koutsu_pos..34 {
        if ctx.hand_counts[i] == 0 { continue; }

        let check_avail = |p: usize| -> bool {
             let used = current_counts[p] + 3;
             let have = ctx.hand_counts[p];
             let remain = ctx.wrapper.remain_counts[p];
             if used > have {
                 let need = used - have;
                 if need > remain { return false; }
             }
             true
        };

        if check_avail(i) {
             current_counts[i] += 3;
             let p = MentsuPai::new(i as u8, 0, MentsuFlag::FLAG_NONE);
             let dummy = MentsuPai::default();
             let m = Mentsu::new(
                 &[p, p, p, dummy],
                 3,
                 MentsuType::TYPE_KOUTSU
             );
             current_mentsu.push(m);

             if koutsu_num - 1 > 0 {
                 ret += koutsu_point(ctx, current_counts, current_mentsu, koutsu_num - 1, shuntsu_num, i + 1, shuntsu_pos);
             } else {
                 ret += shuntsu_point(ctx, current_counts, current_mentsu, 0, shuntsu_num, i + 1, shuntsu_pos);
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

        for i in 0..34 {
             if current_counts[i] > ctx.hand_counts[i] {
                let needed = (current_counts[i] - ctx.hand_counts[i]) as i32;
                let avail_in_wall = ctx.wrapper.remain_counts[i] as i32;
                if needed > avail_in_wall { return 0.0; }

                let dist = paidistance(&ctx.hand_counts, i);
                let dist_c = get_dist_coef(dist);
                let kind_c = get_kind_coef(ctx.wrapper.game_state, i);

                for _ in 0..needed {
                    probability *= (avail_in_wall as f64) / rest;
                    probability *= dist_c;
                    probability *= kind_c;
                    rest -= 1.0;
                }
            }
        }

        if probability <= 0.0 { return 0.0; }

        let mut mentsu_vec = Vec::new();
        for i in 0..34 {
            if current_counts[i] >= 2 {
                let p = MentsuPai::new(i as u8, 0, MentsuFlag::FLAG_NONE);
                let dummy = MentsuPai::default();
                mentsu_vec.push(Mentsu::new(
                    &[p, p, dummy, dummy],
                    2,
                    MentsuType::TYPE_ATAMA
                ));
            }
        }

        if let Some(_) = mentsu_vec.first() {
             let mut test_mentsu = mentsu_vec.clone();
             let mut m = test_mentsu[0].unpack();
             m.pai_list[0].flag = MentsuFlag::FLAG_AGARI;
             test_mentsu[0] = m.pack();

             let agari_state = ctx.wrapper.game_state.get_agari(
                 ctx.wrapper.game_state.teban as usize,
                 &test_mentsu,
                 &vec![], // Chiitoi: no open melds
                 false
             );
             let mut yakus = ctx.wrapper.game_state.get_condition_yaku(
                 ctx.wrapper.game_state.teban as usize,
                 &agari_state
             );
             yakus.extend(agari_state.get_yaku_list());
             yakus.extend(ctx.wrapper.game_state.get_dora_yaku(
                 ctx.wrapper.game_state.teban as usize,
                 &test_mentsu,
                 &vec![],
                 0
             ));
             let agari_res = agari_state.get_agari(&yakus);
             let score = agari_res.score as f64;

             return 0.8 * probability * probability * score + 0.2 * probability * score;
        }
        return 0.0;
    }

    let mut sum = 0.0;

    for i in pos..34 {
        let have = ctx.hand_counts[i];
        if have >= 2 {
             if current_counts[i] == 0 {
                 current_counts[i] = 2;
                 sum += chiitoi_point(ctx, current_counts, cnt + 1, i + 1);
                 current_counts[i] = 0;
             }
        } else {
             let need = 2 - have;
             if ctx.wrapper.remain_counts[i] >= need {
                 current_counts[i] = 2;
                 sum += chiitoi_point(ctx, current_counts, cnt + 1, i + 1);
                 current_counts[i] = 0;
             }
        }
    }
    sum
}
