## Context

`mahjong_core` は FlatBuffers 生成型の `GameStateT` にゲームロジックが直接実装されており、
`ai_bridge` はグローバルな `static mut G_STATE: Lazy<GameStateT>` を介してシングルトンの対局状態を参照している。
この構造はマルチ対局・並行テスト・AIコーディングアシスタントによる安全な部分修正を困難にしている。

本設計は ECS（Entity Component System）アーキテクチャを段階的に導入し、
「内部ロジック（ECS World）」と「外部インターフェース（FlatBuffers/DLL）」を完全分離することを目標とする。
移行は 5 Phase に分割され、各 Phase は独立した論理的単位としてコミットし、段階的に検証します。

## Goals / Non-Goals

**Goals:**
- `hecs` を利用した軽量 ECS World の導入（Phase 0–1）
- `static mut G_STATE` を `Mutex<GameRegistry>` で置き換え、グローバル可変状態を排除（Phase 2–4）
- `mjsend_message` の外部シグネチャを変更せず DLL 後方互換を維持
- マルチ対局サポート（`inst` ポインタを Key とした Registry）
- `game_process.rs` の 1364 行ロジックを独立した System 関数に分割（Phase 3）
- 点数計算（`agari.rs`）・シャンテン数計算（`shanten.rs`）は Entity ID 非依存の純粋関数として維持

**Non-Goals:**
- FlatBuffers スキーマ自体の変更
- `ai_wasm` の no_std 環境での `hecs::World` 使用（`feature = "ecs"` で opt-in のみ）
- `browser-app` / `app` の UI 層変更
- ゲームルール変更

## Decisions

### 決定1: ECS ライブラリは `hecs`

**選択**: `hecs`（v0.10）  
**理由**:
- 依存コストが最小（proc-macros 不要でも動作）
- `alloc` feature で no_std 対応があり将来の WASM 展開に対応可能
- `shipyard` / `bevy_ecs` に比べてコードの読みやすさが高い

**代替**: `shipyard` — System スケジューリングが充実しているが、本プロジェクトの単純な順次実行には過剰

---

### 決定2: Registry 方式（`inst` ポインタ → `MahjongWorld`）

**選択**: `HashMap<usize, MahjongWorld>` を `Mutex` で包み `once_cell::Lazy` で初期化

```rust
static G_REGISTRY: Lazy<Mutex<GameRegistry>> = Lazy::new(|| Mutex::new(GameRegistry::new()));
```

**理由**:
- DLL の `inst` ポインタをグローバル外部ポインタとして ECS の Key に使うことで、内部ポインタを漏らさずに正しい対局を特定できる
- `Mutex` によりスレッドセーフ性を保証（`static mut` の排除）

**代替**: `thread_local!` — シングルスレッド DLL 環境では有効だが、将来のマルチスレッド対応を妨げる

---

### 決定3: Phase 4 完了まで `GameStateT` の互換シムを維持

**選択**: Phase 3 終了後も `GameStateT` の `impl` メソッドを残し、ECS System への委譲 wrapper として機能させる

**理由**:
- `server` / `app` / `sample` クレートへの波及を Phase 4 でまとめて行う
- 各 Phase を独立してレビュー・ロールバック可能にする

---

### 決定4: ロジックの Value Level / Entity Level 分離

- **Value Level**: `PaiState` を入力とする点数計算・シャンテン計算は Entity ID 非依存の純粋関数のまま維持
- **Entity Level**: 山牌の追跡・特定牌の移動は `MahjongWorld` のクエリに移行

## Risks / Trade-offs

- **`Mutex` のロック競合**: 対局数が増えると Registry のロックがボトルネックになる可能性 → 当面は単一スレッドで運用するため問題ない。マルチスレッド対応が必要になった時点で `DashMap` 等に移行
- **`unsafe` ブロック残存**: FFI 境界の `unsafe` は排除できないが、`static mut` を削除することで範囲を大幅に縮小できる
- **Phase 間の中間状態**: `G_STATE` と `G_REGISTRY` が Phase 2–4 で一時的に共存する → 段階的移行の代償として許容

## Migration Plan

| Phase | コミットメッセージの例 | 主な変更 |
|---|---|---|
| 0 | `feat(core): setup ecs base` | `hecs` 依存追加・`components/`, `systems/` 等の骨格 |
| 1 | `feat(core): define world` | コンポーネント定義・`MahjongWorld` |
| 2 | `feat(bridge): add registry` | `GameRegistry` 実装・`interface.rs` 移行 |
| 3 | `feat(core): implement systems` | System 化（tsumo / sutehai / fulo / agari） |
| 4 | `refactor: kill G_STATE` | `G_STATE` 完全廃止・互換シム削除 |

**ロールバック**: 必要に応じて特定のコミットまで `git revert` または `reset` を行います。`G_STATE` は Phase 4 まで保存されるため、任意の時点で以前のロジックへ戻すことが可能です。

## Open Questions

- `ai_wasm` クレートで将来的に ECS を使用するか（`alloc` feature 有効化）？
- `server` / `app` / `sample` の ECS API への直接移行は Phase 4 に含めるか、別 change として切り出すか？
