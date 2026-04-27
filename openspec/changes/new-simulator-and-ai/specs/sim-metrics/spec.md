## ADDED Requirements

### Requirement: JSONL Action Logging
MUST: シミュレーターは、各ターンの情報を `ActionLog` 構造体として JSONL 形式で出力しなければならない。

#### Scenario: Logging turn information
- **WHEN** AIが打牌を選択したとき
- **THEN** ターン数、プレイヤーID、手牌、ツモ牌、シャンテン数、捨て牌、アクション種別が1行のJSONとしてファイルに追記される

### Requirement: Game End Logging
MUST: 和了または流局時に、その結果を特定のアクション種別（`TSUMO_AGARI` または `RYUUKYOKU`）として記録しなければならない。

#### Scenario: Logging game end
- **WHEN** ゲームが和了または流局で終了したとき
- **THEN** 最終ターンの情報として、和了牌や終了理由がログに出力される

### Requirement: Metrics Calculation (Future)
SHALL: 将来的に、出力された JSONL ログを解析して和了率や放銃率などの統計指標を算出する独立したツールまたはモジュールを提供する。

#### Scenario: Calculating aggregate stats
- **WHEN** 100局分の JSONL ログを入力として集計スクリプトを実行したとき
- **THEN** 各プレイヤーごとの和了率、放銃率、平均順位が算出される
