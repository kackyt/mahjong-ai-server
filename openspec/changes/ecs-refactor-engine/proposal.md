## Why

現在の `mahjong_core` は FlatBuffers 生成型 (`GameStateT`) にロジックが密結合し、グローバルな `static mut G_STATE` がシングルトンとして存在するため、マルチ対局・テスト容易性・AI可読性のいずれも致命的に低い。  
ECS（Entity Component System）アーキテクチャへ移行することで「Logical Layer（ECS）」と「Interface Layer（FlatBuffers/DLL）」を分離し、副作用を局所化された System として記述できる "AI Ready" な状態を実現する。

## What Changes

- `mahjong_core` に `hecs` ベースの ECS 構造（`components/`, `systems/` モジュール等）を追加
- 牌・プレイヤー・河・点数などを ECS コンポーネントとして再定義 (`Hand`, `DiscardPile`, `Score`, `Wind`, `TsumoHai`, `InRiichi` など)
- `MahjongWorld` 構造体を導入し、`hecs::World` を使ってゲーム状態を管理
- `ai_bridge` に `GameRegistry`（`inst` ポインタ → `MahjongWorld` のルックアップ）を導入
- **BREAKING**: `ai_bridge/src/interface.rs` の `pub static mut G_STATE` を `Mutex<GameRegistry>` に置き換え
- `game_process.rs` の巨大ロジック（1364行）を独立した System に分割（`tsumo`, `sutehai`, `fulo`, `agari`, `scoring`）
- 点数計算・シャンテン数計算は `PaiState` ベースの純粋関数として ECS 外に維持（Value Level）

### Phase 構成（段階的コミット）

| Phase | 内容 |コミットの目安 |
|---|---|---|
| **Phase 0** | `hecs` 導入・src 直下のモジュール骨格作成 | `feat(core): setup ecs base with hecs` |
| **Phase 1** | ECS コンポーネント定義・World 構造体 | `feat(core): define ecs components and world` |
| **Phase 2** | `GameRegistry` 実装・`interface.rs` 移行 | `feat(bridge): implement registry and swap G_STATE` |
| **Phase 3** | `game_process.rs` ロジックの System 化 | `feat(core): refactor game logic into ecs systems` |
| **Phase 4** | `static mut G_STATE` の完全廃止 | `refactor: remove legacy G_STATE and sync api` |

## Capabilities

### New Capabilities

- `ecs-world`: `hecs::World` を包む `MahjongWorld` 構造体とコンポーネント定義群（`Hand`, `DiscardPile`, `Score`, `Wind`, `TsumoHai`, `InRiichi`, `CurrentTurn`）
- `ecs-registry`: DLL `inst` ポインタをキーに `MahjongWorld` を管理する `GameRegistry`（マルチ対局サポート）
- `ecs-systems`: tsumo / sutehai / fulo / agari の各ゲームフロー処理を独立した System として分離

### Modified Capabilities

- `game_flow`: `GameStateT` 中心のゲーム進行ロジックが、ECS System ベースに移行する（状態遷移の責務が変わる）

## Impact

- **`mahjong_core`**: `Cargo.toml` に `hecs` 追加、`components/`, `systems/` 等のモジュール新設
- **`ai_bridge`**: `interface.rs` の `G_STATE` 参照を Registry 経由に変更、`registry.rs` 新設
- **`server` / `app` / `sample`**: `game_process.rs` の API が変わるため互換シムを一時提供（Phase 4 完了後に整理）
- **DLL 互換性**: `mjsend_message` のシグネチャは変更なし。Registry 経由の検索追加のみで後方互換を維持
- **null 破壊リスク**: `static mut G_STATE` 削除後はデータ競合の可能性が解消される
