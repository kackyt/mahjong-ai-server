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

- [x] 4.1 `mahjong_core/src/systems/` ディレクトリに必要な System（tsumo.rs 等）を追加する
- [x] 4.2 `systems/tsumo.rs` に `run_tsumo(world: &mut MahjongWorld, play_log: &mut PlayLog) -> Result<()>` を実装する
- [x] 4.3 `systems/sutehai.rs` に `run_sutehai(world: &mut MahjongWorld, play_log: &mut PlayLog, index: usize, is_riichi: bool) -> Result<PaiT>` を実装する
- [ ] 4.4 `systems/fulo.rs` に `run_fulo` / `run_ankan` / `run_kakan` を実装する
- [ ] 4.5 `systems/agari.rs` に `run_tsumo_agari` / `run_ron_agari` / `run_check_ron` を実装する
- [ ] 4.6 `systems/scoring.rs` に Entity ID 非依存の点数計算ユーティリティをまとめる
- [ ] 4.7 各 System の単体テスト（tsumo → sutehai → agari のシーケンス）を追加する
- [ ] 4.8 `cargo test --features ecs --workspace` が通ることを確認する

## 5. Phase 4: static mut G_STATE の完全廃止

- [ ] 5.1 `ai_bridge/src/interface.rs` から `pub static mut G_STATE` を削除する
- [ ] 5.2 `game_process.rs` の `GameStateT` impl を ECS System への委譲 wrapper に変更する（`server` / `app` / `sample` への互換 API を維持）
- [ ] 5.3 残存する `unsafe` ブロックを FFI 境界のみに限定し、コメントで理由を記載する
- [ ] 5.4 `cargo clippy --all-targets --all-features -- -D warnings` / `cargo fmt --all -- --check` / `cargo test --workspace` が全て通ることを確認する

## 6. 最終検証

- [ ] 6.1 DLL AI（Akagi_1.0.dll 等）を用いたエンドツーエンドテストを実行し、従来通り対局が進行することを確認する
- [ ] 6.2 マルチ対局シナリオ（2つの `inst` が同時に Registry に登録）を単体テストで検証する
- [ ] 6.3 一時ファイル（`tmp_*.json`）を削除してブランチをクリーンアップする
