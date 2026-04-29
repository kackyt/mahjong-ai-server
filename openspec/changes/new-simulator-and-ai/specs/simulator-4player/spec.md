## ADDED Requirements

### Requirement: CLI-based 4-Player Simulation
MUST: 4人のAIプレイヤーを配置し、`GameStateT` に基づく対局を自動的に実行するCLIツールを提供する。

#### Scenario: Running a 4-player session
- **WHEN** シミュレーターを起動し、`--players 4` を指定したとき
- **THEN** 4人のAIが順次ツモ・打牌を行い、和了または流局まで自動的に進行する

### Requirement: Reproducible Simulation
SHALL: `--pai-list-file` および `--index` 引数により、特定の牌山構成からの対局を再現できる。

#### Scenario: Loading a specific wall
- **WHEN** 有効な牌山リストファイルを指定してシミュレーターを実行したとき
- **THEN** 指定されたインデックスの牌山データがロードされ、決定論的な対局が開始される

### Requirement: Action Logging
MUST: 対局中のすべての打牌および主要なイベントを JSONL 形式でファイルに出力する。

#### Scenario: Log generation
- **WHEN** シミュレーションが完了したとき
- **THEN** 指定されたログファイル（デフォルト: `simulator_log.jsonl`）に各ターンの状態とアクションが記録されている
