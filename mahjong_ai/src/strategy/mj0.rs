use mahjong_core::mahjong_generated::open_mahjong::{GameStateT, MentsuType, PaiT};
use rand::Rng;
use rayon::prelude::*;

#[cfg(not(test))]
const SIMU_SIZE: usize = 1000;
#[cfg(test)]
const SIMU_SIZE: usize = 100;

#[derive(Clone)]
pub struct MJ0Param {
    pub tehai_counts: [u8; 34],
    pub melds: Vec<(MentsuType, Vec<u8>)>, // Type and tiles
    pub kawahai: Vec<PaiT>,
    pub riichi: bool,
}

impl Default for MJ0Param {
    fn default() -> Self {
        Self {
            tehai_counts: [0; 34],
            melds: Vec::new(),
            kawahai: Vec::new(),
            riichi: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct MentsuCandidate {
    tiles: [u8; 3],
    is_shuntsu: bool,
    count: u32,
    sum: u32,
}

#[allow(clippy::type_complexity)]
pub fn mj0_simulate(
    game_state: &GameStateT,
) -> ([f64; 34], [f64; 34], [f64; 34], [f64; 34], [f64; 34]) {
    // 出力配列
    let mut nokorihai = [0.0; 34];
    let kikenhai = [0.0; 34];
    let mentsu_simo = [0.0; 34];
    let mentsu_toimen = [0.0; 34];
    let mentsu_kami = [0.0; 34];

    // Initialize wall assumption (4 of each)
    let mut wall_counts = [4u8; 34];

    // 1. Remove my hand and melds from wall counts
    let myself = &game_state.players[game_state.teban as usize];

    // My Hand
    for i in 0..myself.tehai_len as usize {
        let pai = &myself.tehai[i];
        if (pai.pai_num as usize) < 34 && wall_counts[pai.pai_num as usize] > 0 {
            wall_counts[pai.pai_num as usize] -= 1;
        }
    }
    if myself.tsumohai.pai_num < 34 && wall_counts[myself.tsumohai.pai_num as usize] > 0 {
        wall_counts[myself.tsumohai.pai_num as usize] -= 1;
    }

    // My Melds
    for i in 0..myself.mentsu_len as usize {
        let m = &myself.mentsu[i];
        for j in 0..m.pai_len as usize {
            let p = &m.pai_list[j];
            if (p.pai_num as usize) < 34 && wall_counts[p.pai_num as usize] > 0 {
                wall_counts[p.pai_num as usize] -= 1;
            }
        }
    }

    // 2. Remove all visible discards and opponent open melds
    // Safe tiles (Genbutsu)
    let mut anpai = [[false; 34]; 4];

    for (pidx, player) in game_state.players.iter().enumerate() {
        // Kawa
        for i in 0..player.kawahai_len as usize {
            let k = &player.kawahai[i];
            if (k.pai_num as usize) < 34 {
                if wall_counts[k.pai_num as usize] > 0 {
                    wall_counts[k.pai_num as usize] -= 1;
                }
                // Mark as safe against this player
                anpai[pidx][k.pai_num as usize] = true;
            }
        }

        // Melds (Opponents)
        if pidx != game_state.teban as usize {
            for i in 0..player.mentsu_len as usize {
                let m = &player.mentsu[i];
                for j in 0..m.pai_len as usize {
                    let p = &m.pai_list[j];
                    if (p.pai_num as usize) < 34 && wall_counts[p.pai_num as usize] > 0 {
                        wall_counts[p.pai_num as usize] -= 1;
                    }
                }
            }
        }
    }

    // Riichi Genbutsu Logic
    let mut is_riichi = [false; 4];
    for (pidx, player) in game_state.players.iter().enumerate() {
        if player.is_riichi {
            is_riichi[pidx] = true;
        }
    }

    // 3. Remove Dora Indicators
    for i in 0..game_state.dora_len as usize {
        let ind = game_state.taku.n5[i].pai_num as usize;
        if ind < 34 && wall_counts[ind] > 0 {
            wall_counts[ind] -= 1;
        }
    }

    if game_state.player_len == 1 {
        for i in 0..34 {
            nokorihai[i] = wall_counts[i] as f64;
        }
        return (nokorihai, kikenhai, mentsu_simo, mentsu_toimen, mentsu_kami);
    }

    // 4. 各相手プレイヤーの面子数を事前計算（並列ループ内で使うため）
    let mut initial_mentsu_count = [0u32; 3];
    let mut active_seat = [false; 3];
    for (i, count) in initial_mentsu_count.iter_mut().enumerate() {
        let target_seat = (game_state.teban as usize + i + 1) % 4;
        if (target_seat as u32) < game_state.player_len {
            *count = game_state.players[target_seat].mentsu_len;
            active_seat[i] = true;
        } else {
            *count = 4; // 不在プレイヤーはスキップ
        }
    }
    let teban = game_state.teban as usize;

    // 5. Monte Carlo Simulation (SIMU_SIZE) — rayon 並列化
    // 各スレッドでローカルなRNG・集計配列を持ち、最後にreduceで集約
    let (nokorihai, kikenhai) = (0..SIMU_SIZE)
        .into_par_iter()
        .fold(
            || ([0.0f64; 34], [0.0f64; 34]),
            |(mut local_nokori, mut local_kiken), _| {
                let mut rng = rand::rng();
                let mut sim_wall = wall_counts;
                let mut opponent_hands_mentsu: [Vec<MentsuCandidate>; 3] =
                    [Vec::new(), Vec::new(), Vec::new()];
                let mut current_mentsu_count = initial_mentsu_count;

                // 相手プレイヤーの面子をランダムに割り当て
                let mut attempts = 0;
                loop {
                    attempts += 1;
                    if attempts > 100 {
                        break;
                    }

                    let mut done = true;
                    for j in 0..3 {
                        if current_mentsu_count[j] < 4 {
                            done = false;

                            let mut candidates = Vec::new();

                            // 順子候補
                            for k in 0..21 {
                                let suit = k / 7;
                                let num = k % 7;
                                let p1 = suit * 9 + num;
                                let p2 = p1 + 1;
                                let p3 = p1 + 2;

                                let c = (sim_wall[p1] as u32)
                                    * (sim_wall[p2] as u32)
                                    * (sim_wall[p3] as u32);
                                if c > 0 {
                                    candidates.push(MentsuCandidate {
                                        tiles: [p1 as u8, p2 as u8, p3 as u8],
                                        is_shuntsu: true,
                                        count: c,
                                        sum: 0,
                                    });
                                }
                            }

                            // 刻子候補
                            for (k, &count) in sim_wall.iter().enumerate() {
                                if count >= 3 {
                                    candidates.push(MentsuCandidate {
                                        tiles: [k as u8, k as u8, k as u8],
                                        is_shuntsu: false,
                                        count: 1,
                                        sum: 0,
                                    });
                                }
                            }

                            if candidates.is_empty() {
                                current_mentsu_count[j] = 4; // 強制スキップ
                                continue;
                            }

                            let mut total_weight = 0;
                            for cand in &mut candidates {
                                cand.sum = total_weight;
                                total_weight += cand.count;
                            }

                            let r = rng.random_range(0..total_weight);
                            let selected = candidates.iter().find(|c| r < c.sum + c.count).unwrap();

                            sim_wall[selected.tiles[0] as usize] -= 1;
                            sim_wall[selected.tiles[1] as usize] -= 1;
                            sim_wall[selected.tiles[2] as usize] -= 1;

                            opponent_hands_mentsu[j].push(*selected);
                            current_mentsu_count[j] += 1;
                        }
                    }
                    if done {
                        break;
                    }
                }

                // 危険牌の推定
                for (j, opp_mentsu) in opponent_hands_mentsu.iter().enumerate() {
                    if !active_seat[j] {
                        continue;
                    }
                    let mut pairs = Vec::new();
                    for (k, &count) in sim_wall.iter().enumerate() {
                        if count >= 2 {
                            pairs.push(k);
                        }
                    }

                    if !pairs.is_empty() {
                        let p_idx = rng.random_range(0..pairs.len());
                        let pair = pairs[p_idx];
                        sim_wall[pair] -= 2;

                        let comp_idx = rng.random_range(0..5);

                        let mut machi = [false; 34];

                        if comp_idx == 4 {
                            machi[pair] = true;
                        } else if comp_idx < opp_mentsu.len() {
                            let m = &opp_mentsu[comp_idx];
                            if m.is_shuntsu {
                                let remove_idx = rng.random_range(0..3);
                                let p = m.tiles[0] as usize;
                                match remove_idx {
                                    0 => {
                                        // 両面/辺張
                                        machi[p] = true;
                                        if (p % 9) < 6 {
                                            machi[p + 3] = true;
                                        }
                                    }
                                    1 => {
                                        // 嵌張
                                        machi[p + 1] = true;
                                    }
                                    2 => {
                                        // 両面/辺張
                                        machi[p + 2] = true;
                                        if !p.is_multiple_of(9) {
                                            machi[p - 1] = true;
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                machi[m.tiles[0] as usize] = true;
                            }
                        }

                        let mut furiten = false;
                        let op_seat = (teban + j + 1) % 4;
                        for k in 0..34 {
                            if machi[k] && anpai[op_seat][k] {
                                furiten = true;
                                break;
                            }
                        }

                        if !furiten {
                            for k in 0..34 {
                                if machi[k] {
                                    local_kiken[k] += 1.0;
                                }
                            }
                        }
                    }
                }

                // 残り牌カウント集計
                for k in 0..34 {
                    local_nokori[k] += sim_wall[k] as f64;
                }

                (local_nokori, local_kiken)
            },
        )
        .reduce(
            || ([0.0f64; 34], [0.0f64; 34]),
            |(mut a_n, mut a_k), (b_n, b_k)| {
                for i in 0..34 {
                    a_n[i] += b_n[i];
                    a_k[i] += b_k[i];
                }
                (a_n, a_k)
            },
        );

    // 正規化
    let mut final_nokori = nokorihai;
    let mut final_kiken = kikenhai;
    for i in 0..34 {
        final_nokori[i] /= SIMU_SIZE as f64;
        final_kiken[i] /= SIMU_SIZE as f64;
    }

    (
        final_nokori,
        final_kiken,
        mentsu_simo,
        mentsu_toimen,
        mentsu_kami,
    )
}
