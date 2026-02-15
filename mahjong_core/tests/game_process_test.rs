use mahjong_core::{
    mahjong_generated::open_mahjong::{GameStateT, PaiT},
    play_log,
};

#[test]
fn game_start_test() {
    let mut state = GameStateT::default();
    let mut play_log = play_log::PlayLog::new();

    // 1人プレイのテスト
    state.create("test".as_bytes(), 1, &mut play_log);
    state.shuffle();

    state.start(&mut play_log);

    assert!(state.tsumo(&mut play_log).is_ok(), "ツモ失敗");

    let mut player = state.get_player(0);

    for item in &player.tehai {
        print!("{}", item);
    }
    println!();
    println!("{}", player.tsumohai);

    assert_eq!(player.tehai_len, 13);
    assert_ne!(player.tsumohai, PaiT::default());

    state.sutehai(&mut play_log, 10, false);
    assert!(state.tsumo(&mut play_log).is_ok(), "ツモ失敗");

    player = state.get_player(0);

    for item in &player.tehai {
        print!("{}", item);
    }
    println!();
    println!("{}", player.tsumohai);

    assert_eq!(player.tehai_len, 13);
    assert_ne!(player.tsumohai, PaiT::default());
}

#[test]
fn test_fulo_sutehai() {
    let mut state = GameStateT::default();
    let mut play_log = play_log::PlayLog::new();
    state.create("test".as_bytes(), 1, &mut play_log);
    state.shuffle();
    state.start(&mut play_log);

    // 強制的に副露後の状態を作る
    // is_tsumo = false, tehai_len = 10 (3枚鳴いた想定)
    {
        let player = &mut state.players[0];
        player.is_tsumo = false;
        player.tehai_len = 10;
        player.tsumohai = PaiT::default(); // ツモ牌なし
    }

    // 手出し (index 0) は成功するはず
    // tehai_len が 10 -> 9 になるはず
    let result = state.sutehai(&mut play_log, 0, false);
    assert!(result.is_ok(), "副露後の手出しに失敗: {:?}", result.err());

    let player = state.get_player(0);
    assert_eq!(player.tehai_len, 9, "手牌の枚数が減っていません");
    assert!(!player.is_tsumo, "is_tsumoが変わっています");

    // 不正なインデックス (index 9 = tehai_len) はエラーになるはず（ツモ切り扱いだがツモ牌がない）
    // 状態をリセット
    {
        let player = &mut state.players[0];
        player.tehai_len = 9;
    }
    let result = state.sutehai(&mut play_log, 9, false);
    assert!(
        result.is_err(),
        "副露後の無効なインデックスでエラーになっていません"
    );
}

#[test]
fn test_normal_tsumogiri_with_tehai_len() {
    let mut state = GameStateT::default();
    let mut play_log = play_log::PlayLog::new();
    state.create("test".as_bytes(), 1, &mut play_log);
    state.shuffle();
    state.start(&mut play_log);
    state.tsumo(&mut play_log).unwrap();

    // 通常のツモ切り (index 13)
    // tehai_len は 13 なので、 index == tehai_len となる
    let result = state.sutehai(&mut play_log, 13, false);
    assert!(result.is_ok(), "通常のツモ切りに失敗: {:?}", result.err());

    let player = state.get_player(0);
    assert_eq!(player.tehai_len, 13, "ツモ切り後も tehai_len は変わらないはず（次は他家だがテストでは更新されず? sutehai内でteban更新される）");
}
