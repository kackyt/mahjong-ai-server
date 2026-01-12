# Rust コーディング規約

このプロジェクトでは、以下のコーディング規約に従います。

## 1. 命名規則 (Naming Conventions)

基本的にはRustの標準的な命名規則（[RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-naming-conventions.md)）に準拠します。

| 項目 | ケース | 例 |
| :--- | :--- | :--- |
| パッケージ / クレート | `snake_case` | `mahjong_core` |
| モジュール | `snake_case` | `game_process` |
| 型 (struct, enum, trait) | `UpperCamelCase` | `Player`, `GameState` |
| 関数 / メソッド | `snake_case` | `calculate_score` |
| 変数 (ローカル / メンバー) | `snake_case` | `player_index` |
| 定数 (const / static) | `SCREAMING_SNAKE_CASE` | `MAX_PLAYERS` |

### 補足
- **略語の扱い**: `TCP` や `UUID` などの略語も、キャメルケースの一部として扱います（例: `TcpConnection`, `UuidGen`）。ただし、`FBI` のように一般的すぎるものは要検討ですが、基本は `UpperCamelCase` を優先します。

## 2. コードフォーマット (Code Formatting)

`rustfmt` を使用して自動整形を行います。コミット前には必ずフォーマットを適用してください。

```bash
cargo fmt
```

### 設定
標準の設定を使用しますが、可読性のために以下の点に注意してください。
- インデントはスペース4つ。
- 行の最大長は100文字（`rustfmt.toml` で設定されている場合を除く）。

## 3. コメントとドキュメント (Comments & Documentation)

### ドキュメントコメント (`///`)
公開されている関数(`pub fn`)、構造体(`pub struct`)、列挙型(`pub enum`)には、必ずドキュメントコメントを記述してください。

```rust
/// プレイヤーの点数を計算します。
///
/// # Arguments
/// * `han` - 翻数
/// * `fu` - 符数
///
/// # Returns
/// 計算された点数を返しますが、エラーの場合は `None` を返します。
pub fn calculate_score(han: i32, fu: i32) -> Option<i32> {
    // ...
}
```

### 実装コメント (`//`)
複雑なロジックや、なぜそのように実装したかの意図（Why）を記述する場合に使用します。コードを見ればわかること（What）は書かないようにします。

## 4. エラー処理 (Error Handling)

- **`Result` 型**: 失敗する可能性のある処理は必ず `Result` を返し、`panic!` は避けてください。
- **`anyhow` / `thiserror`**: アプリケーション層では `anyhow`、ライブラリ層では `thiserror` の使用を推奨します。
- **`unwrap()` / `expect()`**: プロダクションコードでは原則禁止です。テストコードや、論理的に絶対に失敗しないことが証明できる場合のみ許可されます（その場合もコメントで理由を書いてください）。

## 5. テスト (Testing)

- ユニットテストは、対象のモジュール内に `mod tests` を作成して記述します。
- テスト関数名は `test_` で始める必要はありませんが、何をするテストかわかる名前（例: `score_calculation_should_return_correct_value`）にします。

## 6. その他

- **`unsafe`**: `unsafe` ブロックを使用する場合は、なぜ安全なのか(`Safety`)コメントを必ず記述してください。FFI連携以外での使用は極力避けます。
- **Clippy**: `cargo clippy` を実行し、警告が出ないようにします。
