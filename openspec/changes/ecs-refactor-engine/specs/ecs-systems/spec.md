## ADDED Requirements

### Requirement: ゲームフロー操作が独立した System 関数として実装される
Game flow operations SHALL be implemented as independent System functions that take `MahjongWorld` as an argument.
ツモ・捨て牌・副露・和了の各ゲーム操作は、`MahjongWorld` を引数に取る独立した System 関数として実装されなければなりません。各 System は他者が担当するコンポーネントを直接書き換えてはなりません。

#### Scenario: tsumo System がツモ牌を TsumoHai コンポーネントとして付与する
- **WHEN** `run_tsumo(world, play_log)` が呼ばれる
- **THEN** 現在の `CurrentTurn` Entity に `TsumoHai` コンポーネントが追加され、山から1枚の牌IDが記録される

#### Scenario: sutehai System が TsumoHai を除去して DiscardPile に追記する
- **WHEN** `run_sutehai(world, play_log, index, is_riichi)` が呼ばれる
- **THEN** `CurrentTurn` Entity の `Hand` または `TsumoHai` から該当牌が除去され、`DiscardPile` に追記される。`TsumoHai` コンポーネントが Entity から削除される

#### Scenario: agari System がスコアを更新する
- **WHEN** `run_tsumo_agari(world, play_log)` が呼ばれる
- **THEN** 点数計算が実行され、全プレイヤーの `Score` コンポーネントが更新される

### Requirement: System が副作用を局所化し、他 System を破壊しない
Each System SHALL document its modified components and explicitly declare dependencies.
各 System の変更対象コンポーネントは事前に文書化されていなければならず、他の System が所有するコンポーネントを変更する場合は明示的な依存として宣言しなければなりません。

#### Scenario: fulo System が相手の DiscardPile を変更する
- **WHEN** `run_fulo(world, play_log, player_idx, mentsu)` が呼ばれる
- **THEN** 副露元プレイヤーの `DiscardPile` の最後の牌に `is_nakare` フラグが設定され、副露プレイヤーの `Hand` と `Mentsu` コンポーネントが更新される

### Requirement: 点数計算は Entity ID 非依存の純粋関数として維持される
The `agari.rs` algorithms SHALL be maintained as pure functions to allow sharing between the engine and DLLs.
`agari.rs` の点数計算ロジックは値型を入力とする純粋関数のままでなければならず、System はこれらの結果を `Score` コンポーネントに反映する役割のみを担います。

#### Scenario: 点数計算関数が World のクエリなしに実行できる
- **WHEN** `calculate_score(pai_state, fulo, wind, rule)` を直接呼び出す
- **THEN** ECS World への参照なしに点数が返される
