use mahjong_ai::strategy::mj0::mj0_simulate;
use mahjong_ai::evaluator::eval_sutehai;
use mahjong_core::mahjong_generated::open_mahjong::{
    GameStateT, PaiT, PlayerT, TakuT
};

fn create_pai(pai_num: u8) -> PaiT {
    PaiT {
        pai_num,
        id: 0,
        is_tsumogiri: false,
        is_riichi: false,
        is_nakare: false,
    }
}

fn setup_game_state() -> GameStateT {
    let mut game_state = GameStateT::default();
    // 4 Players
    game_state.players = [
        PlayerT::default(),
        PlayerT::default(),
        PlayerT::default(),
        PlayerT::default(),
    ];
    game_state.teban = 0;
    // Taku
    game_state.taku = TakuT::default();
    game_state.dora_len = 1;
    game_state.taku.n5[0] = create_pai(0); // 1m dora indicator -> 2m dora

    game_state
}

#[test]
fn test_mj0_wall_reading_basic() {
    let mut game_state = setup_game_state();
    let player = &mut game_state.players[0];

    // My hand: 1m, 1m, 1m, 2m, 3m...
    player.tehai[0] = create_pai(0);
    player.tehai[1] = create_pai(0);
    player.tehai[2] = create_pai(0);
    player.tehai_len = 3;

    // Discards: 1m
    player.kawahai[0] = create_pai(0);
    player.kawahai_len = 1;

    // Run MJ0
    let (nokorihai, _kikenhai, _, _, _) = mj0_simulate(&game_state);

    // 1m (0) count:
    // Total 4.
    // My hand has 3.
    // My discard has 1.
    // Visible = 4.
    // Nokorihai[0] should be 0.

    assert!(nokorihai[0] < 0.001, "1m should be 0 remaining, got {}", nokorihai[0]);

    // 2m (1) count:
    // Visible = 0.
    // Nokorihai[1] should be around 4. (It's average of wall counts).
    // MJ0 initializes wall with 4.
    // Since opponents might use 2m in their hands during simulation, it can be less than 4.
    // But it should be > 2.0 (unlikely everyone uses 2m).
    assert!(nokorihai[1] > 2.0 && nokorihai[1] <= 4.0, "2m should be reasonable remaining, got {}", nokorihai[1]);
}

#[test]
fn test_ai_logic_tenpai_priority() {
    let mut game_state = setup_game_state();
    let player = &mut game_state.players[0];

    // Hand: 123m 456p 789s 11z 22z + 3z (South)
    // 1m, 2m, 3m, 9p, 10p, 11p, 18s, 19s, 20s, 27z, 27z, 28z, 28z
    // Tsumo: 29z (West)

    // Wait: 27z (East) and 28z (South) are pairs (Shanpon).
    // Or maybe discard West.

    let tiles = vec![
        0, 1, 2,        // 123m
        9, 10, 11,      // 456p
        18, 19, 20,     // 789s
        27, 27,         // East pair
        28, 28,         // South pair
    ];

    for (i, &t) in tiles.iter().enumerate() {
        player.tehai[i] = create_pai(t);
    }
    player.tehai_len = 13;
    player.tsumohai = create_pai(29); // West

    let result = eval_sutehai(&game_state);
    assert!(result.is_ok());
    let (pai, _score) = result.unwrap();

    // Should discard West (29) to keep Tenpai (Shanpon wait on East/South)
    // Or maybe it prefers keeping West if it's safe?
    // But this is attack logic. Keeping West breaks Tenpai?
    // Wait, 13 tiles + West = 14 tiles.
    // Hand structure:
    // 123m (Completed)
    // 456p (Completed)
    // 789s (Completed)
    // 27z, 27z (Pair)
    // 28z, 28z (Pair)
    // 29z (Isolated)
    // Discarding 29z leaves 2 pairs. This is Tenpai (Shanpon).

    assert_eq!(pai, 29, "Should discard West (29) to reach Tenpai");
}

#[test]
fn test_ai_logic_shanten_progress() {
    let mut game_state = setup_game_state();
    let player = &mut game_state.players[0];

    // Hand: 12m 56p 99s ... isolated tiles
    // 1m, 2m, 13p, 14p, 26s, 26s, 30z, 31z, 32z, 33z, 4m, 8p, 2s
    // Tsumo: 5z

    // Let's make it clearer.
    // 1m, 2m (Penchan)
    // 5p, 6p (Ryanmen)
    // 9s, 9s (Pair)
    // Isolated: West(29), North(30), White(31), Green(32), Red(33)
    // We should discard isolated winds/dragons.

    let tiles = vec![
        0, 1,       // 12m
        13, 14,     // 56p
        26, 26,     // 99s
        29, 30, 31, 32, 33, // Winds/Dragons
        3, 21,      // 4m, 4s (Isolated)
    ];

    for (i, &t) in tiles.iter().enumerate() {
        player.tehai[i] = create_pai(t);
    }
    player.tehai_len = 13;
    player.tsumohai = create_pai(10); // 2p. Useful? No.

    let result = eval_sutehai(&game_state);
    assert!(result.is_ok());
    let (pai, _score) = result.unwrap();

    // Should discard one of the honors (29..33) or maybe 2p if it's useless.
    // Honors are usually discarded first.
    // 21 (4s) is also isolated.
    assert!(pai >= 29 || pai == 10 || pai == 21, "Should discard isolated honor or useless tile, got {}", pai);
}
