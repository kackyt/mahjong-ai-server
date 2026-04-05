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
- **各 System を `MahjongWorld` 全体に依存させず、型付き Context View（`TsumoView` 等）のみを受け取る形にして単体テストを容易にする（Phase 3）**
- **PlayLog への書き込みを System の副作用から切り離し、System はイベント型（`TsumoEvent` 等）を返す形にする（Phase 3）**

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

---

### 決定5: System の依存を型付き Context View に絞る

**背景**: 当初の実装では `run_tsumo(world: &mut MahjongWorld, ...)` のように World 全体を受け取るため、
単体テストに4人分のEntity構築が必要になり、テスタビリティが著しく低かった。

**選択**: 各 System はそれぞれ専用の View 型とInput 型を受け取り、World を直接受け取らない。

```rust
// System ごとに専用の型を定義する（共通基底型は作らない）
pub struct TsumoView<'w> {
    pub hand:   &'w mut Hand,
    pub cursol: &'w mut Cursol,
}

pub struct TsumoInput {
    pub taku_pai:    PaiT,   // コピー渡し。Entityへの参照なし
    pub taku_cursol: u32,
    pub kyoku_id:    u64,
    pub teban:       u32,
    pub seq:         u32,
}

pub fn run_tsumo(
    view:  TsumoView<'_>,
    input: &TsumoInput,
) -> Result<TsumoEvent, SystemError>
```

`MahjongWorld` 側には View を切り出すファクトリメソッドを置く：

```rust
impl MahjongWorld {
    pub fn tsumo_view(&mut self, teban: usize) -> TsumoView<'_> { ... }
    pub fn tsumo_input(&self) -> TsumoInput { ... }
}
```

`world.world`（`hecs::World`）フィールドは将来的にプライベート化し、
System が直接クエリを発行できない構造にする。

**テストへの効果**:
```rust
#[test]
fn test_tsumo_sets_tsumohai() {
    // MahjongWorld 不要。Component を直接組み立てる
    let mut hand   = Hand { tiles: vec![], tsumohai: None, is_tsumo: false };
    let mut cursol = Cursol { cursol: 0 };
    let input = TsumoInput { taku_pai: PaiT::one_man(), .. };

    let event = run_tsumo(
        TsumoView { hand: &mut hand, cursol: &mut cursol },
        &input,
    ).unwrap();

    assert_eq!(hand.tsumohai, Some(PaiT::one_man()));
}
```

**代替**: Pure Function（引数を直接並べる）— より単純だが引数が増えると可読性が低下する。
View 型は「依存の宣言」として機能し、System の責任範囲をドキュメント化できる。

---

### 決定6: PlayLog への書き込みを System から分離（Event Output）

**背景**: 従来 `run_tsumo` の中で `play_log.append_actions_log(...)` を呼んでいたため、
System のテストに `PlayLog` の構築も必要だった。また、将来の非同期化の妨げになる。

**選択**: System はイベント型を返すだけとし、PlayLog への書き込みは呼び出し側の責任とする。

```rust
// System が返すイベント
pub struct TsumoEvent {
    pub kyoku_id: u64,
    pub teban:    u32,
    pub seq:      u32,
    pub tsumohai: PaiT,
}

// 呼び出し側（game_process.rs 等）
let event = run_tsumo(view, &input)?;
play_log.record(&event.into());  // ← PlayLog への書き込みはここで行う
```

これにより PlayLog の実装（同期/非同期/ノーオペ）を呼び出し側で差し替え可能になる。

**将来の非同期化**: `mpsc::Sender<GameEvent>` を渡す形への移行も、呼び出し側だけの変更で済む。

## Risks / Trade-offs

- **`Mutex` のロック競合**: 対局数が増えると Registry のロックがボトルネックになる可能性 → 当面は単一スレッドで運用するため問題ない。マルチスレッド対応が必要になった時点で `DashMap` 等に移行
- **`unsafe` ブロック残存**: FFI 境界の `unsafe` は排除できないが、`static mut` を削除することで範囲を大幅に縮小できる
- **Phase 間の中間状態**: `G_STATE` と `G_REGISTRY` が Phase 2–4 で一時的に共存する → 段階的移行の代償として許容
- **Context View の型数増加**: System ごとに View 型・Input 型・Event 型の3種が必要になり、型の総数が増える。ただし各型の責任が明確になるため複雑度は下がる
- **既存 System の破壊的変更**: Phase 3 では `run_tsumo` / `run_sutehai` のシグネチャが変わるため、呼び出し元（`game_process.rs`）の修正が必要になる

## Migration Plan

| Phase | コミットメッセージの例 | 主な変更 |
|---|---|---|
| 0 | `feat(core): setup ecs base` | `hecs` 依存追加・`components/`, `systems/` 等の骨格 |
| 1 | `feat(core): define world` | コンポーネント定義・`MahjongWorld` |
| 2 | `feat(bridge): add registry` | `GameRegistry` 実装・`interface.rs` 移行 |
| 3 | `feat(core): implement systems` | View/Input/Event 型定義・System 化（tsumo / sutehai / fulo / agari）・PlayLog 分離 |
| 4 | `refactor: kill G_STATE` | `G_STATE` 完全廃止・互換シム削除・`world.world` プライベート化 |

**ロールバック**: 必要に応じて特定のコミットまで `git revert` または `reset` を行います。`G_STATE` は Phase 4 まで保存されるため、任意の時点で以前のロジックへ戻すことが可能です。

## Open Questions

- `ai_wasm` クレートで将来的に ECS を使用するか（`alloc` feature 有効化）？
- `server` / `app` / `sample` の ECS API への直接移行は Phase 4 に含めるか、別 change として切り出すか？
