use crate::strategy::mj0::mj0_simulate;
use mahjong_core::mahjong_generated::open_mahjong::GameStateT;

pub struct AIStateWrapper<'a> {
    pub game_state: &'a GameStateT,
    pub visible_counts: [u8; 34],
    pub remain_counts: [u8; 34],
    pub my_tehai_counts: [u8; 34],
    pub kikenhai: [f64; 34],
    pub nokorihai: [f64; 34],
}

impl<'a> AIStateWrapper<'a> {
    pub fn new(game_state: &'a GameStateT) -> Self {
        // Run MJ0 Simulation
        let (nokorihai, kikenhai, _, _, _) = mj0_simulate(game_state);

        let mut visible_counts = [0; 34];
        let mut my_tehai_counts = [0; 34];

        // 1. Count tiles in my hand (tehai)
        let myself = &game_state.players[game_state.teban as usize];
        for i in 0..myself.tehai_len as usize {
            let pai = &myself.tehai[i];
            if pai.pai_num < 34 {
                visible_counts[pai.pai_num as usize] += 1;
                my_tehai_counts[pai.pai_num as usize] += 1;
            }
        }

        if myself.tsumohai.pai_num < 34 {
            visible_counts[myself.tsumohai.pai_num as usize] += 1;
            my_tehai_counts[myself.tsumohai.pai_num as usize] += 1;
        }

        // 2. Count discards (kawahai)
        for player in &game_state.players {
            for i in 0..player.kawahai_len as usize {
                let pai = &player.kawahai[i];
                if pai.pai_num < 34 {
                    visible_counts[pai.pai_num as usize] += 1;
                }
            }
        }

        // 3. Melds
        for player in &game_state.players {
            for i in 0..player.mentsu_len as usize {
                let mentsu = &player.mentsu[i];
                for j in 0..mentsu.pai_len as usize {
                    let p = &mentsu.pai_list[j];
                    if p.pai_num < 34 {
                        visible_counts[p.pai_num as usize] += 1;
                    }
                }
            }
        }

        // 4. Doras
        for i in 0..game_state.dora_len as usize {
            let dora = &game_state.taku.n5[i];
            if dora.pai_num < 34 {
                visible_counts[dora.pai_num as usize] += 1;
            }
        }

        let mut remain_counts = [0; 34];
        for i in 0..34 {
            // MJ0 returns estimated remaining count in `nokorihai`.
            // Should we use MJ0's estimate or exact visibility?
            // `remain_counts` in score logic is usually (4 - visible).
            // But MJ0 estimation provides "Tiles not held by opponents and not in dead wall".
            // Actually MJ0 `nokorihai` is `average(wall_counts)`.
            // So it includes tiles in the wall.
            // My score logic (shuntsu_point etc) iterates:
            // "probability *= (avail_in_wall) / rest"
            // So we need "Expected number of tile i in wall".
            // `nokorihai[i]` IS exactly that!

            // So we can use `nokorihai` (rounded or as f64).
            // My previous code used `remain_counts` (u8) and cast to f64.
            // I should update `remain_counts` to use MJ0 if possible, or provide f64 accessor.
            // But `AIStateWrapper` struct has `remain_counts: [u8; 34]`.
            // I will keep `remain_counts` as (4 - visible) for strict checks,
            // but add `nokorihai` array for probability calc.

            if visible_counts[i] > 4 {
                remain_counts[i] = 0;
            } else {
                remain_counts[i] = 4 - visible_counts[i];
            }
        }

        Self {
            game_state,
            visible_counts,
            remain_counts,
            my_tehai_counts,
            kikenhai,
            nokorihai,
        }
    }

    pub fn get_remain_count(&self, pai: usize) -> u8 {
        if pai < 34 {
            self.remain_counts[pai]
        } else {
            0
        }
    }
}
