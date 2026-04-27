## ADDED Requirements

### Requirement: Monte Carlo Wall Estimation
MUST: 可視情報から、山牌および他家の手牌をモンテカルロ法によりサンプリングし、残り牌の期待値を算出する。

#### Scenario: Estimating remaining tiles
- **WHEN** AIの思考フェーズにおいて
- **THEN** 公開情報を除いた136枚の牌からランダムなシミュレーションを実行し、各牌の残り枚数期待値を更新する

### Requirement: Expected Value Discard Evaluation
SHALL: 算出された残り牌期待値に基づき、各候補牌を捨てた場合の和了確率と得点の期待値を計算し、最も期待値の高い牌を選択する。

#### Scenario: Discard selection
- **WHEN** 打牌候補が複数あるとき
- **THEN** シャンテン数、役（翻数）、フリテンの状態を考慮した期待値計算を行い、最適な牌を選択する

### Requirement: Parallel Score Calculation
MUST: `rayon` を使用して、各打牌候補の期待値計算を並列に実行し、思考時間を短縮する。

#### Scenario: Parallel thinking
- **WHEN** 手牌が14枚あり、14通りの打牌候補を評価するとき
- **THEN** 各候補の探索が並列化され、結果を迅速に返す
