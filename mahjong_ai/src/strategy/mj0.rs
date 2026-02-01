use mahjong_core::mahjong_generated::open_mahjong::{GameStateT, MentsuType, PaiT};
use rand::Rng;

const SIMU_SIZE: usize = 5000;

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

pub fn mj0_simulate(
    game_state: &GameStateT,
) -> ([f64; 34], [f64; 34], [f64; 34], [f64; 34], [f64; 34]) {
    // Outputs
    let mut nokorihai = [0.0; 34];
    let mut kikenhai = [0.0; 34];
    let mut mentsu_simo = [0.0; 34];
    let mut mentsu_toimen = [0.0; 34];
    let mut mentsu_kami = [0.0; 34];

    // Initialize wall assumption (4 of each)
    let mut wall_counts = [4u8; 34];

    // 1. Remove my hand and melds from wall counts
    let myself = &game_state.players[game_state.teban as usize];

    // My Hand
    for i in 0..myself.tehai_len as usize {
        let pai = &myself.tehai[i];
        if (pai.pai_num as usize) < 34 {
            if wall_counts[pai.pai_num as usize] > 0 {
                wall_counts[pai.pai_num as usize] -= 1;
            }
        }
    }
    if myself.tsumohai.pai_num < 34 {
        if wall_counts[myself.tsumohai.pai_num as usize] > 0 {
            wall_counts[myself.tsumohai.pai_num as usize] -= 1;
        }
    }

    // My Melds
    for i in 0..myself.mentsu_len as usize {
        let m = &myself.mentsu[i];
        for j in 0..m.pai_len as usize {
            let p = &m.pai_list[j];
            if (p.pai_num as usize) < 34 {
                if wall_counts[p.pai_num as usize] > 0 {
                    wall_counts[p.pai_num as usize] -= 1;
                }
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
                    if (p.pai_num as usize) < 34 {
                        if wall_counts[p.pai_num as usize] > 0 {
                            wall_counts[p.pai_num as usize] -= 1;
                        }
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
        if ind < 34 {
            if wall_counts[ind] > 0 {
                wall_counts[ind] -= 1;
            }
        }
    }

    // 4. Monte Carlo Simulation (SIMU_SIZE)
    let mut rng = rand::rng(); // Use new rand syntax if updated, or thread_rng.
    // rand 0.9 uses rand::rng() probably?
    // Let's use old style or check docs?
    // Error said "use of unresolved module rand".
    // I added rand.

    for _ in 0..SIMU_SIZE {
        let mut sim_wall = wall_counts.clone();

        let mut opponent_hands_mentsu = [Vec::new(), Vec::new(), Vec::new()];

        let mut initial_mentsu_count = [0; 3];
        for i in 0..3 {
            let target_seat = (game_state.teban as usize + i + 1) % 4;
            initial_mentsu_count[i] = game_state.players[target_seat].mentsu_len;
        }

        let mut current_mentsu_count = initial_mentsu_count;

        let mut attempts = 0;
        loop {
            attempts += 1;
            if attempts > 100 { break; }

            let mut done = true;
            for j in 0..3 {
                if current_mentsu_count[j] < 4 {
                    done = false;

                    let mut candidates = Vec::new();

                    // Shuntsu
                    for k in 0..21 {
                        let suit = k / 7;
                        let num = k % 7;
                        let p1 = suit * 9 + num;
                        let p2 = p1 + 1;
                        let p3 = p1 + 2;

                        let c = (sim_wall[p1] as u32) * (sim_wall[p2] as u32) * (sim_wall[p3] as u32);
                        if c > 0 {
                            candidates.push(MentsuCandidate {
                                tiles: [p1 as u8, p2 as u8, p3 as u8],
                                is_shuntsu: true,
                                count: c,
                                sum: 0,
                            });
                        }
                    }

                    // Koutsu
                    for k in 0..34 {
                        if sim_wall[k] >= 3 {
                            candidates.push(MentsuCandidate {
                                tiles: [k as u8, k as u8, k as u8],
                                is_shuntsu: false,
                                count: 1,
                                sum: 0,
                            });
                        }
                    }

                    if candidates.is_empty() {
                        current_mentsu_count[j] = 4; // Force skip
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

                    opponent_hands_mentsu[j].push(selected.clone());
                    current_mentsu_count[j] += 1;
                }
            }
            if done { break; }
        }

        for j in 0..3 {
            let mut pairs = Vec::new();
            for k in 0..34 {
                if sim_wall[k] >= 2 {
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
                } else if comp_idx < opponent_hands_mentsu[j].len() {
                    let m = &opponent_hands_mentsu[j][comp_idx];
                    if m.is_shuntsu {
                        let remove_idx = rng.random_range(0..3);
                        let p = m.tiles[0] as usize;
                        match remove_idx {
                            0 => { // Ryanmen/Penchan
                                machi[p] = true;
                                if (p % 9) < 6 { machi[p+3] = true; }
                            },
                            1 => { // Kanchan
                                machi[p+1] = true;
                            },
                            2 => { // Ryanmen/Penchan
                                machi[p+2] = true;
                                if (p % 9) > 0 { machi[p-1] = true; }
                            },
                            _ => {}
                        }
                    } else {
                        machi[m.tiles[0] as usize] = true;
                    }
                }

                let mut furiten = false;
                let op_seat = (game_state.teban as usize + j + 1) % 4;
                for k in 0..34 {
                    if machi[k] && anpai[op_seat][k] {
                        furiten = true;
                        break;
                    }
                }

                if !furiten {
                    for k in 0..34 {
                        if machi[k] {
                            kikenhai[k] += 1.0;
                        }
                    }
                }
            }
        }

        for k in 0..34 {
            nokorihai[k] += sim_wall[k] as f64;
        }
    }

    // Normalize
    for i in 0..34 {
        nokorihai[i] /= SIMU_SIZE as f64;
        kikenhai[i] /= SIMU_SIZE as f64;
        mentsu_simo[i] /= SIMU_SIZE as f64;
        mentsu_toimen[i] /= SIMU_SIZE as f64;
        mentsu_kami[i] /= SIMU_SIZE as f64;
    }

    (nokorihai, kikenhai, mentsu_simo, mentsu_toimen, mentsu_kami)
}
