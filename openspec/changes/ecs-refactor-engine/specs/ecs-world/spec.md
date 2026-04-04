## ADDED Requirements

### Requirement: MahjongWorld がゲーム状態を ECS として管理する
The `MahjongWorld` SHALL manage the game state (tiles, players, scores, winds) as Entity/Component data using the `hecs` engine.
`MahjongWorld` 構造体は `hecs::World` をラップし、麻雀の全ゲーム状態を Entity/Component として保持しなければなりません。

#### Scenario: ゲーム開始時にプレイヤー Entity が生成される
- **WHEN** `MahjongWorld::new(player_len)` が呼ばれる
- **THEN** `player_len` 個のプレイヤー Entity が World に生成され、`Hand`・`DiscardPile`・`Score`・`Wind` コンポーネントが付与される

#### Scenario: ツモ番のプレイヤーにのみ TsumoHai コンポーネントが付与される
- **WHEN** ツモ操作後に World を問い合わせる
- **THEN** `TsumoHai` コンポーネントを持つ Entity はちょうど 1 つであり、その Entity が `CurrentTurn` タグも持つ

### Requirement: コンポーネントが Entity ID に依存しない純粋データを保持する
Components SHALL NOT store Entity IDs internally, and scoring algorithms SHALL operate as pure functions.
全コンポーネントは Entity の ID を内部状態として保持してはならず、点数計算・シャンテン計算アルゴリズムは値型を入力とする純粋関数として実装しなければなりません。

#### Scenario: Hand コンポーネントへのアクセスが Entity ID なしで行える
- **WHEN** `world.query::<&Hand>()` を実行する
- **THEN** 結果の `Hand` 構造体のフィールドには Entity ID が含まれず、牌番号の Vec のみが含まれる

### Requirement: MahjongWorld が GameStateT から構築できる（移行互換）
The `MahjongWorld` SHALL provide a conversion method from `GameStateT` for legacy compatibility.
Phase 4 完了まで、`MahjongWorld::from_game_state(state: &GameStateT)` が提供されなければなりません。

#### Scenario: 既存の GameStateT から MahjongWorld を生成する
- **WHEN** `MahjongWorld::from_game_state(&state)` が呼ばれる
- **THEN** `state` の全プレイヤー・スコア・手牌が対応するコンポーネントとして World に登録される
