use mahjong_ai::evaluator::eval_sutehai;
use mahjong_ai::strategy::mj0::mj0_simulate;
use mahjong_core::mahjong_generated::open_mahjong::{GameStateT, PaiT, PlayerT, TakuT};

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
    let mut game_state = GameStateT {
        players: [
            PlayerT::default(),
            PlayerT::default(),
            PlayerT::default(),
            PlayerT::default(),
        ],
        teban: 0,
        taku: TakuT::default(),
        dora_len: 1,
        ..Default::default()
    };
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

    assert!(
        nokorihai[0] < 0.001,
        "1m should be 0 remaining, got {}",
        nokorihai[0]
    );

    // 2m (1) count:
    // Visible = 0.
    // Nokorihai[1] should be around 4. (It's average of wall counts).
    // MJ0 initializes wall with 4.
    // Since opponents might use 2m in their hands during simulation, it can be less than 4.
    // But it should be > 2.0 (unlikely everyone uses 2m).
    assert!(
        nokorihai[1] > 2.0 && nokorihai[1] <= 4.0,
        "2m should be reasonable remaining, got {}",
        nokorihai[1]
    );
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
        0, 1, 2, // 123m
        9, 10, 11, // 456p
        18, 19, 20, // 789s
        27, 27, // East pair
        28, 28, // South pair
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
        0, 1, // 12m
        13, 14, // 56p
        26, 26, // 99s
        29, 30, 31, 32, 33, // Winds/Dragons
        3, 21, // 4m, 4s (Isolated)
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
    assert!(
        pai >= 29 || pai == 10 || pai == 21,
        "Should discard isolated honor or useless tile, got {}",
        pai
    );
}

#[test]
fn test_ai_logic_yaku_priority_tanyao() {
    let mut game_state = setup_game_state();
    let player = &mut game_state.players[0];
    game_state.rule.enable_kuitan = true; // Ensure Kuitan is enabled if relevant (though this is menzen)

    // Setup:
    // 23m 456p 456s 66z (White Dragon pair)
    // Tsumo: 8m

    // Hand:
    // 2m(1), 3m(2) -> Wait 1m, 4m.
    // 456p (Completed)
    // 456s (Completed)
    // 66z (Pair - White Dragon) -> Yaku: Yakuhai (if triplet)

    // If we have 23m and draw 8m... wait, let's construct a choice.
    // Choice between 1m discard and 2m discard?

    // Let's try:
    // 234m (Completed)
    // 67p (Ryanmen 58p)
    // 23s (Ryanmen 14s)
    // 88s (Pair)
    // Tsumo: 5s

    // If I have 234m, 67p, 23s, 88s, 5s.
    // 23s + 5s is useless.

    // Let's try "Tanyao vs Pinfu vs Nothing".
    // 23m, 78m.
    // Discard 1m or 9m?

    // Better example: Tanyao choice.
    // Hand: 234m 234p 234s 67s 5s(Tsumo)
    // 67s + 5s -> 567s.
    // So 4 mentsu complete. Need pair.
    // Hand was 13 tiles + Tsumo = 14.
    // 234m, 234p, 234s, 67s.
    // Tsumo 5s. -> 567s.
    // Now we have 4 completed shuntsu: 234m, 234p, 234s, 567s.
    // No pair. This is "Hadaka Tanki" state if we discard one?
    // No, we have 14 tiles. 4*3 = 12. 2 tiles left.

    // Let's construct a Tenpai choice.
    // 23m 456p 456s 66z(White) 8m(Tsumo)
    // 23m + 8m -> Useless.
    // Hand: 23m, 456p, 456s, 66z.
    // We are Tenpai on 1m, 4m.
    // If we discard 8m, we stay Tenpai.
    // This doesn't test Yaku choice.

    // Test case: Choice between 1m and 4m?
    // 23456m.
    // If we cut 1m -> 23456m left.
    // If we cut 6m -> 12345m left.

    // Let's try:
    // Hand: 23m 456p 789s 88p(Pair) + 1m(Tsumo)
    // 123m (Pinfu/Tanyao?) 1m is terminal. No Tanyao.
    // If we had 4m instead of 1m?

    // Scenario:
    // Hand: 34m 456p 456s 88p
    // Tsumo: 2m
    // Hand becomes: 234m (No Tanyao).
    // Tsumo: 5m
    // Hand becomes: 345m (Tanyao).

    // We need a discard choice.
    // Hand: 2m 3m 4m 5m 6m 7m ...
    // Let's say we have 2m, 5m, 8m.
    // And other completed sets.
    // 234m, 567m.

    // Specific Tanyao test:
    // Hand: 1m 2m 3m 4m 5m 6m 456p 456s 55z
    // Discard 1m -> 23456m ... wait 147m?
    // Discard 1m -> 234m 56m ... wait 47m. Tanyao confirmed (if 55z is non-honor? 55z is White? No 27+4 = 31. White is 31).
    // 55z is 5z? 5z is 27+4 = 31?
    // Indices: 0-8 (m), 9-17 (p), 18-26 (s), 27-33 (z).
    // z0=E, z1=S, z2=W, z3=N, z4=Haku, z5=Hatsu, z6=Chun.
    // Tanyao requires no 1,9,z.

    // Setup:
    // 23m 456p 456s 22s(Pair)
    // Tsumo: 1m
    // Hand: 123m(1,9 mixed), 456p, 456s, 22s.
    // Discard 1m: Tenpai on 1m/4m? No.

    // Let's try structure where we have 1m and 4m as floating tiles, and we must cut one.
    // 23m + 1m -> 123m (No Tanyao)
    // 23m + 4m -> 234m (Tanyao)
    // We hold 1m, 2m, 3m, 4m.
    // We must discard 1m or 4m.
    // Rest of hand is Tanyao safe.
    // 456p 456s 22p.

    let tiles = vec![
        0, 1, 2, 3, // 1m, 2m, 3m, 4m
        12, 13, 14, // 456p (Indices 9+3, 9+4, 9+5) -> 12, 13, 14
        21, 22, 23, // 456s (Indices 18+3...) -> 21, 22, 23
        10, 10, // 2p pair (Indices 9+1) -> 10, 10
    ];
    // 1m(0), 2m(1), 3m(2), 4m(3)
    // 4p(12), 5p(13), 6p(14)
    // 4s(21), 5s(22), 6s(23)
    // 2p(10), 2p(10)
    // Total 13 tiles.
    // Tsumo: Let's say we just drew the 4m(3).

    // If discard 1m(0): Hand has 234m (Tanyao possible).
    // If discard 4m(3): Hand has 123m (No Tanyao).

    // The evaluator should prefer Tanyao (higher score).

    for (i, &t) in tiles.iter().enumerate() {
        // Leave last one for tsumo logic simulation if needed, but here we fill tehai.
        if i < 13 {
            player.tehai[i] = create_pai(t);
        }
    }
    player.tehai_len = 13;
    player.tsumohai = create_pai(3); // Draw 4m
                                     // But wait, 1m is in hand (index 0).
                                     // So we have 1,2,3,4m.

    let result = eval_sutehai(&game_state);
    assert!(result.is_ok());
    let (pai, _score) = result.unwrap();

    // Expect discard 1m (0) to keep Tanyao potential with 234m.
    assert_eq!(
        pai, 0,
        "Should discard 1m (0) to aim for Tanyao over 4m (3)"
    );
}
