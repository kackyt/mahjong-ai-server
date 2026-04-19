# はじめに

このプロジェクトはコンピュータ麻雀およびAIを実現するためのプロジェクトです。
Rustを使用して開発します。

# 技術スタック

- Rust
- cargo
- anyhow
- hecs

## Project Conventions

### Code Style
- ソースコードにはロジックの内容がわかるように日本語のコメントをいれること

- 値オブジェクト（Value Object）としての Newtype パターン
Rustのタプル構造体を使って、型安全性を担保します。プリミティブ型（i32やString）の直接利用を避けます。

Rust
// domain/model/unit.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitPoint {
    current: u32,
    max: u32,
}

impl HitPoint {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }
    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }
}

- エンティティの参照は「ポインタ」ではなく「ID」で行う
戦略ゲームでは、「マップ上のこの位置に、このユニットがいる」という関係性が発生します。ここで Unit の実体（参照）を Map に持たせると、借用チェッカーとの終わりのない戦いが始まります。必ずIDで関連付けを行ってください。

Rust
// domain/model/map.rs
pub struct HexMap {
    pub id: MapId,
    // &Unit ではなく UnitId を保持する
    cells: HashMap<Position, Option<UnitId>>, 
}

- 依存性の注入（DI）にはジェネリクスか dyn Trait を使う
アプリケーション層がインフラ層に依存しないよう、ドメイン層で定義した trait を使います。

静的ディスパッチ（ジェネリクス）: パフォーマンス重視。コンパイル時間は伸びる。(推奨)

動的ディスパッチ（Arc<dyn Trait> または Box<dyn Trait>）: 記述がシンプルになり、モックの差し替えが容易。

Rust
// domain/repository/unit_repository.rs
pub trait UnitRepository: Send + Sync {
    fn find_by_id(&self, id: &UnitId) -> Result<Option<Unit>, DomainError>;
    fn save(&self, unit: &Unit) -> Result<(), DomainError>;
}

// application/usecase/move_unit.rs
pub struct MoveUnitUseCase<R: UnitRepository> {
    unit_repo: R,
}

impl<R: UnitRepository> MoveUnitUseCase<R> {
    // 依存を注入してユースケースを初期化
    pub fn new(unit_repo: R) -> Self { Self { unit_repo } }

    pub fn execute(&self, unit_id: UnitId, dest: Position) -> Result<(), AppError> {
        // 1. リポジトリから再構築
        let mut unit = self.unit_repo.find_by_id(&unit_id)?.unwrap();
        // 2. ドメインのロジックを実行
        unit.move_to(dest)?;
        // 3. 結果を永続化
        self.unit_repo.save(&unit)?;
        Ok(())
    }
}

- エラーハンドリングに thiserror と anyhow を活用する
ドメイン層・インフラ層: thiserror クレートを使って、型安全で明確なカスタムエラー（DomainError, InfraError）を定義します。

アプリケーション層・プレゼンテーション層: 最終的なエラーの集約として anyhow を使うと、スタックトレースやコンテキストの付与が簡単になります。

- `cargo clippy --all-targets --all-features -- -D warnings` がエラーなく通ること
- `cargo fmt --all -- --check` がエラーなく通ること
- `cargo test` が正常に動作すること
- SOLID原則、DRY原則を意識すること
- 依存性の注入(DI)を適切に行うこと
- メモリのライフサイクルを意識すること。無駄なコピーやcloneは避ける。解放タイミングが不明なBoxを定義しない。

### Architecture Patterns
Reactはbullet proof reactでディレクトリは配置

Rustはworkspaceを使ったマルチクレート構成にする
coreクレートにはECS(Entity Component System)を採用する

#### ドメインロジックの非漏出（責務の分離）
- ゲームのルールや制約（例: 「首都から3マス以内でしか生産できない」「攻撃可能な範囲の計算」など）の判定ロジックを、プレゼンテーション層（`cli` や `gui` クレート）に直接記述してはなりません。
- これらのビジネスロジックは必ず `engine` クレート内の System や純粋なドメイン関数として実装し、プレゼンテーション層はそれらを呼び出して結果（表示の切り替えや警告文の表示など）を利用するのみに留めてください。

mahjong-ai-server/
├── Cargo.toml               # ワークスペース全体の定義
├── mahjong_core/            # 【コア】ECSを採用したゲームロジック
│   ├── Cargo.toml
│   └── src/
│       ├── components/      # エンティティ、値オブジェクト、リポジトリのtrait
│       ├── systems/         # システム
│       ├── events/          # イベント (uiへの通知)
│       ├── resources/       # リソース (マスターデータ等)
│       └── lib.rs
├── cli/                 # 【プレゼン層 1】CUIアプリケーション
│   ├── Cargo.toml       # engineクレートに依存
│   └── src/
│       └── main.rs      # clap,ncurses等を使ったコマンドライン入力の処理
├── app/                 # 【プレゼン層 2】GUIアプリケーション (iced)
│   ├── Cargo.toml       # engineクレートに依存
│   └── src/
│       └── main.rs      # icedを使ったGUIの処理
├── browser-app/                 # 【プレゼン層 3】Tauriアプリケーション
│   ├── Cargo.toml       # engineクレートに依存
│   ├── src-tauri/       # Tauriのバックエンド（Rust）
│   │   ├── src/
│   │   │   ├── main.rs  # tauriのセットアップ、DIの注入
│   │   │   └── cmd.rs   # tauri::command群（コントローラーの役割）
│   │   └── tauri.conf.json
│   └── src/             # Tauriのフロントエンド（TypeScript / React / Vue など）

### engineクレートの構成
#### Components に実装すべきもの
「ゲームの盤面を再現するために最低限必要なデータ」に絞ります。描画用の Sprite や Transform (Bevyの場合) は含めず、純粋な座標やステータスを持たせます。カテゴリコンポーネント例役割位置・配置GridPosition(x, y)マップ上のセル座標。属性・所有Faction(u32), UnitType勢力ID、ユニットの種類（歩兵、戦車など）。ステータスHealth, ActionPointsHP、残り行動回数、移動力など。能力値AttackStat, DefenseStat攻撃力、防御力、射程などの静的/動的パラメータ。状態フラグIsExhausted, IsCapturing行動済みフラグ、占領中フラグなど。


#### Systems に実装すべきもの
「ルールの適用」と「状態の更新」を記述します。移動ロジック: GridPosition の更新。経路探索（A*等）の結果を適用し、移動コストを ActionPoints から差し引く。戦闘解決: 攻撃側と防御側のコンポーネントを参照し、乱数や定数に基づいて Health を減算する。ターン管理: 全ユニットの ActionPoints を回復させる、バフの継続時間を減らす等のバッチ処理。勝敗判定: 拠点の Health や特定のユニット（将軍など）の生存をチェックする。AI思考: 敵勢力のコンポーネントをスキャンし、次の行動を決定してコマンドを発行する。


#### 描画Crate（別Crate）との連携パターンロジックと描画を分離する場合、

イベント駆動方式（推奨）ロジックCrateは、何かが起きたときに Event を発行するだけに留めます。

```
// ロジックCrateで定義
pub struct UnitMovedEvent {
    pub entity: Entity,
    pub from: GridPosition,
    pub to: GridPosition,
}
```
描画Crateはこのイベントを購読し、「移動アニメーション」を開始したり、「足音SE」を鳴らしたりします。これにより、ロジック側は「どう見えるか」を完全に無視できます。

### Testing Strategy

ロジック単体テストは必ず実装する
GUIは後回し

### Git Workflow
GitHub Flowを採用する

### Simulatorについて

simulatorはAIを評価、比較するために使用する自己対戦を行うツールです。
重い処理が動くため、必ずreleaseモードでビルドしてください。

```bash
cargo build --release --bin simulator
```
