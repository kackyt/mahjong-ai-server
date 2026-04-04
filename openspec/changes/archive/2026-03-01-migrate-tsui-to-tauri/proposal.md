# Change: migrate-tsui-to-tauri

## Why
現在、UI層はブラウザ向けのReactアプリケーション（`ts-mahjong`）として分離して管理されています。これをTauriを用いて `mahjong-ai-server` のRustワークスペース内に統合し、ネイティブデスクトップアプリ化を図ります。これによりRustバックエンド・AIモジュールとの親和性が高まり、将来的なパフォーマンス向上と配布の容易化が見込めます。

## What Changes
- `mahjong-ai-server` ワークスペース内にTauriプロジェクト（`browser-app`）を新規作成します。
- 既存の `ts-mahjong`（React/Vite/Panda CSS）のコードベースをTauriのフロントエンドとして移植します。
- Rustワークスペースと協調してビルドできるよう `Cargo.toml` や構成ファイルを調整します。

## Impact
- Affected specs: `app`
- Affected code:
  - `ts-mahjong` (Tauriフロントエンドとして移行)
  - `mahjong-ai-server/browser-app` (新規構築)
  - `mahjong-ai-server/Cargo.toml` (ワークスペースメンバ追加)
