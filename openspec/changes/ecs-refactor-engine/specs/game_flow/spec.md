## MODIFIED Requirements

### Requirement: Four Player Support
The game MUST support 4 players in the state, each with their own hand, discards, and score.
The game state SHALL be managed via Entity/Component data, with each player represented as an entity.
ゲーム状態は ECS World の Entity/Component として管理され、各プレイヤーは `Hand`・`DiscardPile`・`Score`・`Wind` コンポーネントを持つ Entity として表現されなければなりません。

#### Scenario: Initialization
- **WHEN** `MahjongWorld::new(player_len)` が呼ばれる
- **THEN** `player_len` 個のプレイヤー Entity が生成され、各 Entity に `Hand`・`DiscardPile`・`Score`・`Wind` コンポーネントが付与される

### Requirement: Round Progression
The game MUST progress through rounds (kyoku) from East 1 to South 4.
The game logic SHALL be implemented as ECS System functions to decouple rules from data.
ゲーム進行ロジック（ツモ・捨て牌・副露・和了）は ECS System 関数として実装されなければなりません。

#### Scenario: Next Hand
- **WHEN** 局が終了して次局へ進む
- **THEN** `CurrentTurn` タグが次の手番プレイヤー Entity に移動し、各プレイヤーの `Hand`・`DiscardPile` がリセットされる

### Requirement: Turn Interruption
The game flow MUST support turn variations for Fuuro (Chi, Pon, Kan).

#### Scenario: Call Actions
- **WHEN** プレイヤーが捨て牌を行い、他プレイヤーが副露を宣言する
- **THEN** `run_fulo(FuloView { .. }, &FuloInput { .. })` System が呼ばれ、`FuloEvent` が返り、呼び出し側が `CurrentTurn` タグを副露プレイヤーの Entity に移動する

### Requirement: AI Capabilities
AI players MUST be able to perform standard actions including Riichi and Fuuro.

#### Scenario: AI Riichi
- **WHEN** AI プレイヤーのターンで `RiichiStatus.is_riichi` が false の場合にリーチ条件が成立する
- **THEN** `run_sutehai(SutehaiView { .. }, &SutehaiInput { is_riichi: true, .. })` が呼ばれ、`SutehaiEvent` が返り、呼び出し側が `RiichiStatus.is_riichi` を true に更新する

#### Scenario: AI Fuuro
- **WHEN** 他プレイヤーの捨て牌が副露可能な状態のとき、AI の副露ロジックが呼ばれる
- **THEN** `run_fulo(FuloView { .. }, &FuloInput { .. })` System が呼ばれ、`FuloEvent` が返り、呼び出し側が `Hand` と `Fulo` コンポーネントを更新する

### Requirement: Score Exchange
Points MUST be exchanged between players based on Agari (Win) or Ryuukyoku (Draw).

#### Scenario: Ron Agari
- **WHEN** `run_ron_agari(AgariView { .. }, &AgariInput { winner_idx, loser_idx, pai, .. })` が呼ばれる
- **THEN** 点数計算（純粋関数）の結果を含む `AgariEvent` が返り、呼び出し側が winner / loser の `Score` コンポーネントを更新する

#### Scenario: Ryuukyoku
- **WHEN** 山が枯れた状態で局終了処理が行われる
- **THEN** テンパイ判定が各 `Hand` コンポーネントに対して実行され、点数が `Score` コンポーネントに反映される

### Requirement: Game Information Display
The UI MUST display critical game state information including Round, Dealer, and Sticks.

#### Scenario: Display Elements
- **WHEN** ゲームが進行中でボードがレンダリングされる
- **THEN** `MahjongWorld` から局風・本場・リーチ棒・各プレイヤーの `Score` コンポーネントをクエリして表示できる

### Requirement: Game End and Ranking
The game MUST declare a winner and rank players at the end.

#### Scenario: Game Over
- **WHEN** ゲームが終了条件に達する
- **THEN** 全プレイヤーの `Score` コンポーネント値を集計し、ランキングを出力する
