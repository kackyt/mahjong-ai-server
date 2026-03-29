## Context
現在 `ts-mahjong` は純粋なReactフロントエンドアプリケーションであり、Wasmを使ってRustロジックと通信しています。このプロジェクトを `mahjong-ai-server` 内にTauriアプリとして移植することで、ネイティブデスクトップアプリ化を図ります。

## Goals / Non-Goals
- Goals:
  - `ts-mahjong` のUI要素をそのままTauriのフロントエンドとして動作させる。
  - RustのバックエンドとTauri IPCを利用して連携できる土台を作る。
  - `mahjong-ai-server` のRustワークスペースビルドの一部としてTauriのバックエンドを統合する。
- Non-Goals:
  - ゲームのコアロジックやAIアルゴリズム自体の変更。
  - UIデザイン・機能セットの根本的な再実装（既存コンポーネントを流用する）。

## Decisions
- Decision: 新しいパッケージ `browser-app` を `mahjong-ai-server` 内に作成する。
  - `browser-app/src-tauri` にRustのTauriバックエンドを配置。
  - `browser-app` ルートおよび `src` に従来の `ts-mahjong` フロントエンドコードを配置（Vite環境）。
- Alternatives considered: 既存のGUI実装である `app`（Iced製）を上書きする案。しかし、Iced版とTauri版の共存や段階的移行の可能性を考慮し、別パッケージ名で分離する方が安全。

## Risks / Trade-offs
- Wasmを経由したロジック呼び出しから、Tauri IPCを経由した呼び出しへの移行における非同期処理アーキテクチャの手戻り。
- **Mitigation**: まずはTauriのフロントエンドとして既存のコード（Wasm活用）をそのまま動かせる最小構成を確立させ、その後IPCへの徐々な移行を検討する。

## Migration Plan
1. `mahjong-ai-server` 内に `browser-app` のTauriボイラープレートを生成する。
2. `ts-mahjong` のソースおよび依存・設定をコピーし統合する。
3. Tauriネイティブウィンドウでアプリが正常に起動・描画されることを確認する。
4. 最終的にユーザー合意の上で、レガシーとなった `ts-mahjong` フォルダを削除する。
