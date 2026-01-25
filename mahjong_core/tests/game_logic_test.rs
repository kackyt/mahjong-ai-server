#[cfg(test)]
mod tests {
    use mahjong_core::{
        mahjong_generated::open_mahjong::{GameStateT, PaiT},
        play_log::PlayLog,
    };

    fn create_state() -> GameStateT {
        let mut state = GameStateT::default();
        let mut play_log = PlayLog::new();
        state.create(b"test", 4, &mut play_log);
        state.start(&mut play_log);
        state
    }

    fn create_pai(pai_num: u8, id: u32) -> PaiT {
        PaiT {
            pai_num,
            id: id as u8,
            is_tsumogiri: false,
            is_riichi: false,
            is_nakare: false,
        }
    }

    #[test]
    fn test_pon_check_logic() {
        let mut state = create_state();
        let mut play_log = PlayLog::new();

        // Setup P0 Hand: 1m, 1m, 2m, 3m...
        // P0 has pair of 1m.
        let mut p0 = state.players[0].clone();
        p0.tehai[0] = create_pai(0, 0); // 1m
        p0.tehai[1] = create_pai(0, 1); // 1m
        p0.tehai_len = 13;
        state.players[0] = p0;

        // P3 Discards 1m (id 2)
        state.teban = 3;
        let mut p3 = state.players[3].clone();
        p3.tsumohai = create_pai(0, 2);
        p3.is_tsumo = true;
        state.players[3] = p3;

        // P3 discards 1m
        state.sutehai(&mut play_log, 13, false).unwrap();

        // Now teban is 0. Discarder should be 3.
        assert_eq!(state.teban, 0);

        let discarder_idx = (state.teban + 4 - 1) % 4;
        assert_eq!(discarder_idx, 3);

        let tile = state.players[3].kawahai[0].clone();
        assert_eq!(tile.pai_num, 0);

        // Check Pon for P0
        let cands = state.check_pon(0, &tile);
        assert!(!cands.is_empty(), "P0 should be able to Pon 1m");

        // Scenario: Self Naki (P0 discards 1m, checks Pon)
        // Set P0 turn
        state.teban = 0;
        let mut p0 = state.players[0].clone();
        p0.tsumohai = create_pai(0, 3); // 1m
        p0.is_tsumo = true;
        state.players[0] = p0;

        state.sutehai(&mut play_log, 13, false).unwrap(); // P0 discards 1m

        // Teban is now 1. Discarder is 0.
        assert_eq!(state.teban, 1);
        let discarder_idx = (state.teban + 4 - 1) % 4;
        assert_eq!(discarder_idx, 0);

        let tile = state.players[0].kawahai.last().unwrap().clone();

        // Check Pon for P0 (Self Naki check)
        let cands = state.check_pon(0, &tile);
        assert!(cands.is_empty(), "P0 should NOT be able to Pon own discard");
    }

    #[test]
    fn test_chi_check_logic() {
        let mut state = create_state();
        let mut play_log = PlayLog::new();

        // Setup P0 Hand: 2m, 3m...
        let mut p0 = state.players[0].clone();
        p0.tehai[0] = create_pai(1, 0); // 2m
        p0.tehai[1] = create_pai(2, 0); // 3m
        state.players[0] = p0;

        // P3 discards 1m
        state.teban = 3;
        let mut p3 = state.players[3].clone();
        p3.tsumohai = create_pai(0, 0); // 1m
        p3.is_tsumo = true;
        state.players[3] = p3;

        state.sutehai(&mut play_log, 13, false).unwrap();

        // Teban is 0. Discarder is 3 (Left of 0).
        let tile = state.players[3].kawahai[0].clone();

        // Check Chi for P0
        let cands = state.check_chii(0, &tile);
        assert!(
            !cands.is_empty(),
            "P0 should be able to Chi 1m from P3 (Left)"
        );

        // Setup P2 discards 1m (Kamicha of Kamicha -> Toimen of P0)
        state.teban = 2;
        let mut p2 = state.players[2].clone();
        p2.tsumohai = create_pai(0, 1); // 1m
        p2.is_tsumo = true;
        state.players[2] = p2;

        state.sutehai(&mut play_log, 13, false).unwrap();
        // Teban is 3. Discarder is 2.

        let tile = state.players[2].kawahai[0].clone();

        // Check Chi for P0 (from P2) -> Should fail because Chi only from Left (P3)
        // Wait, check_chii implementation checks if player_idx == teban.
        // If P2 discarded, teban is P3. P0 != P3. So checks first condition.
        // But `check_chii`:
        // if player_idx != self.teban as usize { return res; }
        // So P0 cannot Chi if it's not P0's turn (next turn).
        // If P2 discards, next is P3. P0 is not next. So returns empty. Correct.

        let cands = state.check_chii(0, &tile);
        assert!(cands.is_empty(), "P0 cannot Chi from P2");
    }
}
