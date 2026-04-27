## Why

現在のシミュレーターおよびAIの実装を刷新し、4人打ちのゲームルールへの対応と、より高度な思考ルーチンを持つAIの評価基盤を構築しました。
これにより、ECSベースの新エンジン（`mahjong_core`）上でのAIの動作検証と、統計的な評価が可能になります。

## What Changes

- `simulator` クレートを新規作成し、4人打ちに対応したコマンドラインベースの評価ツールとして実装しました。
- `mahjong_ai` クレートにおいて、モンテカルロ法を用いた残り牌推定および危険度推定を導入した「MJ0」戦略を実装しました。
- AIの評価ロジックを `rayon` により並列化し、`DashMap` によるキャッシュ共有を行うことで、高速な期待値計算を実現しました。
- 既存のプロトタイプ（sample, server）を削除し、新しいシミュレーターに機能を統合しました。

## Capabilities

### New Capabilities
- `simulator-4player`: 4人のAIによる全自動対局シミュレーション機能。JSONL形式でのログ出力に対応。
- `ai-strategy-mj0`: モンテカルロ法による他家手牌推定と、シャンテン数・役・フリテンを考慮した期待値計算ロジック。
- `ai-infrastructure`: `AIStateWrapper` による盤面情報の集約と、エンジンとの統合基盤。
- `sim-metrics`: 和了・放銃・流局などのアクションログを出力するための基盤。

### Modified Capabilities
- `mahjong-core-integration`: ECSエンジンの `G_STATE` (GameStateT) を直接参照し、アクションを発行する形式への統合。

## Impact

- `mahjong_core`: ゲーム進行と状態管理のシングルソース（Single Source of Truth）として機能します。
- `mahjong_ai`: 高度な統計的推定と並列計算を用いた意思決定エンジンとなります。
- `simulator`: 4人打ちの対局を高速に実行し、AIの性能を定量的に評価するツールとなります。
