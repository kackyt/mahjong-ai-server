## ADDED Requirements

### Requirement: AI State Wrapping
MUST: エンジンの全情報（`GameStateT`）から、AIが意思決定に使用する情報（自分の手牌、公開情報、推定された残り牌情報など）を適切に抽出し、`AIStateWrapper` として提供しなければならない。

#### Scenario: Creating a state wrapper
- **WHEN** AIの思考を開始するとき
- **THEN** `AIStateWrapper::new(game_state)` により、その時点の盤面情報がカプセル化される

### Requirement: Single Source of Truth Integration
MUST: AIおよびシミュレーターは、エンジンの `GameStateT` を唯一の状態源として利用し、独自の冗長な状態管理を行ってはならない。

#### Scenario: Referencing engine state
- **WHEN** AIが盤面情報を参照するとき
- **THEN** `AIStateWrapper` を通じて `GameStateT` の各フィールド（players, taku, etc）にアクセスする
