## ADDED Requirements

### Requirement: ゲームフロー操作が独立した System 関数として実装される
Game flow operations SHALL be implemented as independent System functions that accept typed Context Views, not `MahjongWorld` directly.
ツモ・捨て牌・副露・和了の各ゲーム操作は、`MahjongWorld` 全体ではなく、その System が必要とするコンポーネントのみを束ねた型付き View（`TsumoView` 等）と値型の Input を受け取る独立した System 関数として実装されなければなりません。各 System は他者が担当するコンポーネントを直接書き換えてはなりません。

#### Scenario: tsumo System がツモ牌を Hand コンポーネントに設定する
- **WHEN** `run_tsumo(TsumoView { hand, cursol }, &TsumoInput { taku_pai, .. })` が呼ばれる
- **THEN** `hand.tsumohai` に山から取得した牌が設定され、`TsumoEvent { tsumohai, kyoku_id, teban, seq }` が返される。`MahjongWorld` や `PlayLog` は引数として渡されない

#### Scenario: sutehai System が Hand から牌を除去して DiscardPile に追記する
- **WHEN** `run_sutehai(SutehaiView { hand, discard, riichi }, &SutehaiInput { index, is_riichi, .. })` が呼ばれる
- **THEN** `hand` の該当牌が除去され、`discard.tiles` に追記される。`SutehaiEvent { kawahai, kyoku_id, teban, seq }` が返される

#### Scenario: agari System がスコアを更新する
- **WHEN** `run_tsumo_agari(AgariView { hand, score, .. }, &AgariInput { .. })` が呼ばれる
- **THEN** 点数計算が実行され、`AgariEvent` に点数移動結果が含まれて返される。呼び出し元が `Score` コンポーネントに反映する

### Requirement: System が PlayLog への書き込みを行わない
Each System SHALL NOT write to `PlayLog` directly. Systems SHALL return typed event structs, and callers are responsible for recording events to `PlayLog`.
各 System は `PlayLog` への書き込みを直接行ってはならず、イベント型（`TsumoEvent` 等）を返す値のみとしなければなりません。`PlayLog` への記録は呼び出し側（`game_process.rs` 等）の責務とします。これにより将来の非同期化も呼び出し側の変更のみで対応できます。

#### Scenario: PlayLog への書き込みが呼び出し側で行われる
- **WHEN** `run_tsumo(view, &input)` が呼ばれ `TsumoEvent` が返される
- **THEN** 呼び出し側が `play_log.record(&event.into())` を呼び出す。System の内部では `PlayLog` への参照は一切保持しない

### Requirement: System が副作用を局所化し、他 System を破壊しない
Each System SHALL document its modified components via its View type and explicitly declare dependencies.
各 System の変更対象コンポーネントは View 型の定義として自己文書化されていなければならず、他の System が所有するコンポーネントを変更する場合は View 型に明示的に含めなければなりません。

#### Scenario: fulo System が相手の DiscardPile を変更する
- **WHEN** `run_fulo(FuloView { hand, fulo, opponent_discard }, &FuloInput { .. })` が呼ばれる
- **THEN** `opponent_discard` の最後の牌に `is_nakare` フラグが設定され、`hand` と `fulo` コンポーネントが更新される。`FuloEvent` が返される

### Requirement: 点数計算は Entity ID 非依存の純粋関数として維持される
The `agari.rs` algorithms SHALL be maintained as pure functions to allow sharing between the engine and DLLs.
`agari.rs` の点数計算ロジックは値型を入力とする純粋関数のままでなければならず、System はこれらの結果をイベント型に含めて返す役割のみを担います。

#### Scenario: 点数計算関数が World のクエリなしに実行できる
- **WHEN** `calculate_score(pai_state, fulo, wind, rule)` を直接呼び出す
- **THEN** ECS World への参照なしに点数が返される

### Requirement: MahjongWorld が System 向け View ファクトリを提供する
`MahjongWorld` SHALL provide factory methods that extract typed Views for each System, encapsulating direct `hecs::World` access.
`MahjongWorld` は各 System に対応する View を返すファクトリメソッド（`tsumo_view(teban)` 等）を提供しなければなりません。System は `hecs::World` に直接アクセスしてはなりません。

#### Scenario: MahjongWorld から TsumoView が切り出せる
- **WHEN** `world.tsumo_view(teban)` が呼ばれる
- **THEN** 該当プレイヤー Entity の `Hand`・`Cursol` への可変参照を含む `TsumoView` が返される。呼び出し元はこれを `run_tsumo` に渡せる
