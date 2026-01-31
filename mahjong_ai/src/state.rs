use mahjong_core::mahjong_generated::open_mahjong::{GameStateT};

pub struct AIStateWrapper<'a> {
    pub game_state: &'a GameStateT,
    pub visible_counts: [u8; 34],
    pub remain_counts: [u8; 34],
    pub my_tehai_counts: [u8; 34],
}

impl<'a> AIStateWrapper<'a> {
    pub fn new(game_state: &'a GameStateT) -> Self {
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

        // Count tsumohai if valid
        if myself.tsumohai.pai_num < 34 {
             visible_counts[myself.tsumohai.pai_num as usize] += 1;
             my_tehai_counts[myself.tsumohai.pai_num as usize] += 1;
        }

        // 2. Count discards (kawahai) of all players
        for player in &game_state.players {
            for i in 0..player.kawahai_len as usize {
                let pai = &player.kawahai[i];
                if pai.pai_num < 34 {
                    visible_counts[pai.pai_num as usize] += 1;
                }
            }
        }

        // 3. Count melds (mentsu) of all players
        for player in &game_state.players {
            for i in 0..player.mentsu_len as usize {
                let mentsu = &player.mentsu[i];
                // MentsuT has pai_list which is [MentsuPaiT; 4]
                // We assume valid tiles are those with pai_num < 34 (and maybe valid flag/id?)
                // Usually we just count valid pai_num.
                // But pai_list has fixed size 4.
                // Mentsu len is in mentsu.pai_len? No, MentsuT has `pai_len`.
                // Actually MentsuT has `pai_len: u32`.
                // So we iterate up to pai_len.

                for j in 0..mentsu.pai_len as usize {
                    let p = &mentsu.pai_list[j];
                    if p.pai_num < 34 {
                        visible_counts[p.pai_num as usize] += 1;
                    }
                }
            }
        }

        // 4. Doras
        // Dora indicators are in taku.n5
        // We use dora_len to know how many.
        for i in 0..game_state.dora_len as usize {
            let dora = &game_state.taku.n5[i];
            if dora.pai_num < 34 {
                visible_counts[dora.pai_num as usize] += 1;
            }
        }

        let mut remain_counts = [0; 34];
        for i in 0..34 {
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
