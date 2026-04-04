# Project Context

## Purpose
オープンソースの麻雀アプリケーション開発プロジェクト。
Rust製の麻雀AIおよびゲーム進行ロジックとReact製のモダンなフロントエンドを組み合わせ、ブラウザやOSを問わず動作する高品質な麻雀対戦プラットフォームを提供することを目指しています。


## Tech Stack
- Frontend
  - TypeScript
  - React 19
  - Vite
  - Zustand (State Management)
  - Panda CSS (Zero-Runtime CSS-in-JS)
  - React Router
  - MUI Base (Headless UI)
- Backend / AI
  - Rust
  - WebAssembly (ai_wasm)
- Tools & Runtime
  - Bun (Package Manager, Script Runner)
  - Biome (Formatter, Linter)

## Project Conventions

### Code Style

#### Frontend (TypeScript/React)
- **Formatter & Linter**: Biomeを使用 (`bun format`, `bun lint`)。
- **Component**: Functional Componentを使用する。
- **Type Definitions**: `interface` ではなく `type` を使用する。
- **Styles**: Panda CSSを使用し、インラインスタイルは避ける。

#### Backend (Rust)
- **Naming**: Rust標準 (RFC 430) に準拠 (`snake_case` for crates/modules/funcs/vars, `UpperCamelCase` for types)。
- **Formatter**: `rustfmt` を使用 (4 spaces indent, max 100 chars)。`cargo fmt` で適用。
- **Linter**: `clippy` を使用。clippyのwarningをなくすこと。
- **Documentation**: Publicな関数/型には `///` ドキュメントコメントを記述する。
- **Error Handling**: `Result` 型を使用し、`panic!` は避ける。App層は `anyhow`, Lib層は `thiserror` を推奨。Error型を体系化し、app層でDialogを出すなりエラーレスポンスを返すなど適切に処理することを目指す。
- **Safety**: `unsafe` は極力避け、使用する場合は理由をコメントする。
- エラーは推測するのではなく、補足してUIに表示するなりしてデバッグすること。

### Architecture Patterns

#### Frontend
- **Directory Structure**: [Bulletproof React](https://github.com/alan2207/bulletproof-react) を参考にした構成。
  - `src/app`: アプリケーションのページコンポーネント。Feature --> App の依存のみ許可（逆は不可）。
  - `src/features`: 機能ごとのモジュール。Hooks, Components, Typesなどを含む。Feature間の依存は許容。
  - `src/components`: プレゼンテーションのみを担当する共通UIコンポーネント。Global Storeには依存しない。
  - `src/hooks`: 共通のカスタムHooks。ビジネスロジックはHooksとして切り出す。
  - `src/store`: Zustandによるグローバルステート。

#### Backend (Rust Workspace)
- **Core**: `mahjong_core` (麻雀のコアロジック: 牌、役、点数計算)。
- **Libraries**:
  - `game_lib`: ゲーム進行ライブラリ。
  - `ai_bridge`: AI (DLL) 通信ブリッジ。
  - `loadlibrary`: DLLロードユーティリティ。
- **Applications**:
  - `server`: 麻雀AIサーバー。
  - `ai_wasm`: WebAssembly用バインディング。
  - `app`: Iced使用のGUIアプリ。

### Testing Strategy
- **Frontend**: Vitest (`bun test`)。
- **Backend**: `cargo test`。各モジュール内に `mod tests` を配置して単体テストを記述。
- **Requirement**: コード変更時は、Lint, Format, Check, Test を全てパスさせること。

### Git Workflow
- 一般的なFeature Branchフローを想定。

## Domain Context
- 日本のリーチ麻雀（特に最高位戦ルールなどを参照）の仕様に基づく。
- 牌譜形式やAIの思考ロジック（Rust実装）との連携が重要。

## Important Constraints
- AIロジックはRust/Wasmで動作するため、Wasmのロードや非同期処理の扱いに注意が必要。

## External Dependencies
- 無し（現状はローカル完結またはWasmバンドル）
