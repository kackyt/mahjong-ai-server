## 1. Phase 0: hecs 導入・ECS モジュール骨格

- [x] 1.1 `mahjong_core/Cargo.toml` に `hecs = { version = "0.10", features = ["macros"] }` を追加し、`[features]` に `ecs = ["dep:hecs"]` を定義する
- [x] 1.2 `mahjong_core/src/components/mod.rs` および `src/systems/mod.rs` を作成する
- [x] 1.3 `mahjong_core/src/lib.rs` に `#[cfg(feature = "ecs")] pub mod components;`, `pub mod systems;` 等を追加する
- [x] 1.4 `cargo build --features ecs -p mahjong_core` が通ることを確認する

## 2. Phase 1: ECS コンポーネントの定義

- [x] 2.1 `mahjong_core/src/components/` 配下に `Hand`・`DiscardPile`・`Score`・`Wind` 等のコンポーネントを定義する
- [x] 2.2 `mahjong_core/src/lib.rs` または適切な場所に `MahjongWorld` 構造体（`hecs::World` ラッパー）を作成し、`new(player_len: usize)` でプレイヤー Entity を生成する
- [x] 2.3 `MahjongWorld::query_player(idx: usize)` 等のアクセサを実装する
- [x] 2.4 `MahjongWorld::from_game_state(state: &GameStateT)` 互換変換メソッドを実装する
- [x] 2.5 `components/` 内にテストを追加し、`cargo test --features ecs -p mahjong_core` が通ることを確認する

## 3. Phase 2: GameRegistry（Context Lookup）実装

- [x] 3.1 `ai_bridge/src/registry.rs` を新規作成し、`GameRegistry` 構造体（`HashMap<usize, MahjongWorld>`）を実装する
- [x] 3.2 `GameRegistry::insert` / `get` / `get_mut` / `remove` メソッドを実装する
- [x] 3.3 `ai_bridge/src/interface.rs` の `G_STATE: Lazy<GameStateT>` を `G_REGISTRY: Lazy<Mutex<GameRegistry>>` に置き換える
- [x] 3.4 `interface.rs` の `mjsend_message_impl` 内で `G_STATE` 参照を `registry.get(inst)` 経由に変更する（全 message ハンドラ対象）
- [ ] 3.5 `Registry` の `insert/get/remove` 単体テストを追加する
- [x] 3.6 `cargo clippy --all-targets --all-features -- -D warnings` が通ることを確認する

## 4. Phase 3: ゲームロジックの System 化

### 4a. View / Input / Event 型の定義

- [ ] 4.1 `mahjong_core/src/systems/types.rs` を新規作成し、各 System 共通のエラー型 `SystemError` を `thiserror` で定義する
- [ ] 4.2 `systems/tsumo.rs` に `TsumoView<'w>` / `TsumoInput` / `TsumoEvent` 型を定義する
- [ ] 4.3 `systems/sutehai.rs` に `SutehaiView<'w>` / `SutehaiInput` / `SutehaiEvent` 型を定義する
- [ ] 4.4 `systems/fulo.rs` に `FuloView<'w>` / `FuloInput` / `FuloEvent` 型を定義する
- [ ] 4.5 `systems/agari.rs` に `AgariView<'w>` / `AgariInput` / `AgariEvent` 型を定義する

### 4b. MahjongWorld に View ファクトリを追加

- [ ] 4.6 `MahjongWorld::tsumo_view(&mut self, teban: usize) -> TsumoView<'_>` を実装する
- [ ] 4.7 `MahjongWorld::tsumo_input(&self) -> TsumoInput` を実装する
- [ ] 4.8 sutehai / fulo / agari の対応ファクトリメソッドを実装する

### 4c. System 関数のシグネチャ置き換え

- [x] 4.9 `run_tsumo(world, play_log)` → `run_tsumo(view: TsumoView<'_>, input: &TsumoInput) -> Result<TsumoEvent, SystemError>` に置き換える
- [x] 4.10 `run_sutehai(world, play_log, ...)` → `run_sutehai(view: SutehaiView<'_>, input: &SutehaiInput) -> Result<SutehaiEvent, SystemError>` に置き換える
- [ ] 4.11 `run_fulo` / `run_ankan` / `run_kakan` を同様の View/Input/Event パターンで実装する
- [ ] 4.12 `run_tsumo_agari` / `run_ron_agari` / `run_check_ron` を同様の View/Input/Event パターンで実装する
- [ ] 4.13 `systems/scoring.rs` に Entity ID 非依存の点数計算ユーティリティをまとめる

### 4d. PlayLog の分離

- [ ] 4.14 `game_process.rs` の各呼び出し箇所を「`run_XXX(view, &input)?` を呼び、返った Event を `play_log.record()` に渡す」形に写き直す
- [ ] 4.15 `PlayLog::record(&mut self, event: &GameEvent)` を定義し、各 Event 型 `From<TsumoEvent>` 等の変換実装を追加する

### 4e. 単体テスト

- [ ] 4.16 `systems/tsumo.rs` に `MahjongWorld` 不要の単体テスト（ツモ牌が手牌に入るか、cursor が進むか等）を追加する
- [ ] 4.17 `systems/sutehai.rs` に単体テスト（リーチ後のツモ切り制約、河牌への追加等）を追加する
- [ ] 4.18 ツモ → 打牌 → 和了のシーケンス結合テスト（View を連鎖して呼び出す）を `systems/mod.rs` に追加する
- [ ] 4.19 `cargo test --features ecs --workspace` が通ることを確認する

## 5. Phase 4: static mut G_STATE の完全廃止

- [ ] 5.1 `ai_bridge/src/interface.rs` から `pub static mut G_STATE` を削除する
- [ ] 5.2 `game_process.rs` の `GameStateT` impl を ECS System への委譲 wrapper に変更する（`server` / `app` / `sample` への互換 API を維持）
- [ ] 5.3 `MahjongWorld` の `pub world: World` フィールドをプライベート化し、System が直接クエリを発行できない構造にする
- [ ] 5.4 残存する `unsafe` ブロックを FFI 境界のみに限定し、コメントで理由を記載する
- [ ] 5.5 `cargo clippy --all-targets --all-features -- -D warnings` / `cargo fmt --all -- --check` / `cargo test --workspace` が全て通ることを確認する

## 6. 最終検証

- [ ] 6.1 DLL AI（Akagi_1.0.dll 等）を用いたエンドツーエンドテストを実行し、従来通り対局が進行することを確認する
- [ ] 6.2 マルチ対局シナリオ（2つの `inst` が同時に Registry に登録）を単体テストで検証する
- [ ] 6.3 一時ファイル（`tmp_*.json`）を削除してブランチをクリーンアップする
