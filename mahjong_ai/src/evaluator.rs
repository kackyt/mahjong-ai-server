use crate::score::{chiitoi_point, koutsu_point, shuntsu_point, SearchContext};
use crate::state::AIStateWrapper;
use anyhow::Result;
use dashmap::DashMap;
use itertools::Itertools;
use mahjong_core::mahjong_generated::open_mahjong::GameStateT;
use mahjong_core::shanten::PaiState;
use rayon::prelude::*;
use std::sync::Arc;

/// 残り牌の合計を計算
fn calculate_nokori_sum(wrapper: &AIStateWrapper) -> f64 {
    wrapper.nokorihai.iter().sum()
}

/// 捨て牌の評価
/// 各候補牌について、捨てた場合の期待値を並列に計算し、最も高い期待値の牌を選択
pub fn eval_sutehai(game_state: &GameStateT) -> Result<(usize, f64)> {
    let wrapper = AIStateWrapper::new(game_state);
    let myself = &game_state.players[game_state.teban as usize];

    // 候補牌の収集（重複除外）
    let mut candidates: Vec<usize> = myself
        .tehai
        .iter()
        .take(myself.tehai_len as usize)
        .map(|p| p.pai_num as usize)
        .filter(|&p| p < 34)
        .collect();

    if myself.tsumohai.pai_num < 34 {
        candidates.push(myself.tsumohai.pai_num as usize);
    }

    let unique_candidates: Vec<usize> = candidates.into_iter().unique().collect();
    let nokori_sum = calculate_nokori_sum(&wrapper);

    // 各候補牌の評価を並列実行（rayon）
    let results: Vec<(usize, f64)> = unique_candidates
        .par_iter()
        .map(|&pai| {
            let mut hand_counts = wrapper.my_tehai_counts;
            if hand_counts[pai] > 0 {
                hand_counts[pai] -= 1;
            } else {
                return (pai, -1.0);
            }

            let mut pstate = PaiState::default();
            for (i, &count) in hand_counts.iter().enumerate() {
                match i {
                    0..=8 => pstate.hai_count_m[i] = count as i32,
                    9..=17 => pstate.hai_count_p[i - 9] = count as i32,
                    18..=26 => pstate.hai_count_s[i - 18] = count as i32,
                    27..=33 => pstate.hai_count_z[i - 27] = count as i32,
                    _ => {}
                }
            }

            let n_naki =
                wrapper.game_state.players[wrapper.game_state.teban as usize].mentsu_len as i32;
            let shanten = pstate.get_shanten(n_naki as usize);

            // DashMap を使ったスレッドセーフなキャッシュ
            let machi_cache = Arc::new(DashMap::new());

            let ctx = SearchContext {
                wrapper: &wrapper,
                shanten_base: shanten,
                nokori_sum,
                hand_counts,
                machi_cache,
            };

            let mut total_score = 0.0;

            let needed = 4 - n_naki;
            if needed >= 0 {
                // 刻子・順子の組み合わせ探索
                // 各 (k, s) の組み合わせは独立しているため逐次でも十分高速
                // （外側のpar_iterで候補牌ごとに並列化済み）
                for k in 0..=needed {
                    let s = needed - k;
                    let mut current_counts = [0u8; 34];
                    let mut current_mentsu = Vec::new();

                    let score = if k > 0 {
                        koutsu_point(
                            &ctx,
                            &mut current_counts,
                            &mut current_mentsu,
                            k,
                            s,
                            0,
                            0,
                            0,
                        )
                    } else {
                        shuntsu_point(
                            &ctx,
                            &mut current_counts,
                            &mut current_mentsu,
                            0,
                            s,
                            0,
                            0,
                            0,
                        )
                    };
                    total_score += score;
                }
            }

            // 七対子の評価（門前のみ）
            if n_naki == 0 {
                let mut current_counts = [0u8; 34];
                total_score += chiitoi_point(&ctx, &mut current_counts, 0, 0);
            }

            (pai, total_score)
        })
        .collect();

    results
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or_else(|| anyhow::anyhow!("No results"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahjong_core::mahjong_generated::open_mahjong::{PaiT, PlayerT};

    #[test]
    #[ignore]
    fn test_eval_sutehai_basic() {
        let mut game_state = GameStateT {
            players: [
                PlayerT::default(),
                PlayerT::default(),
                PlayerT::default(),
                PlayerT::default(),
            ],
            teban: 0,
            ..Default::default()
        };

        let player = &mut game_state.players[0];
        let tiles = vec![0, 1, 2, 3, 4, 5, 9, 10, 11, 18, 19, 20, 27];
        for (i, t) in tiles.iter().enumerate() {
            player.tehai[i] = PaiT {
                pai_num: *t as u8,
                id: 0,
                is_tsumogiri: false,
                is_riichi: false,
                is_nakare: false,
            };
        }
        player.tehai_len = tiles.len() as u32;

        player.tsumohai = PaiT {
            pai_num: 28,
            id: 0,
            is_tsumogiri: false,
            is_riichi: false,
            is_nakare: false,
        };

        // We need to set up minimal valid game state for MJ0
        // (dora_len, players, etc).
        game_state.dora_len = 1;
        // Default taku has zeros.

        let result = eval_sutehai(&game_state);
        assert!(result.is_ok());
        let (pai, score) = result.unwrap();
        println!("Best discard: {}, Score: {}", pai, score);

        assert!(pai == 27 || pai == 28);
    }
}
