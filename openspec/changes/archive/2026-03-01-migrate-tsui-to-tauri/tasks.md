## 1. Scaffold Tauri Project
- [x] 1.1 `mahjong-ai-server` ディレクトリ内でTauriプロジェクトを `browser-app` として新規作成・初期化
- [x] 1.2 `mahjong-ai-server/Cargo.toml` の `members` に `"browser-app/src-tauri"` を追加

## 2. Migrate Frontend Code
- [x] 2.1 `ts-mahjong` プロジェクトの `src`, `public`, `index.html` および設定ファイル群を `browser-app` にコピー
- [x] 2.2 `ts-mahjong` の依存パッケージ（`package.json` の dependencies/devDependencies）を `browser-app` にマージし、インストールを行う
- [x] 2.3 ViteおよびPanda CSSの設定がTauriプロジェクトの構成で正しく動作するようパス等を修正

## 3. Integration & Testing
- [x] 3.1 `tauri.conf.json` での `buildCommand` と `devPath` セッティングを適正化
- [x] 3.2 既存のWasmブリッジなどがTauriデスクトップ環境下でも正しく解決・ロードされて動作するか確認
- [x] 3.3 ビルド(`npm run tauri dev` 相当)を通し、麻雀のゲーム画面が正常にネイティブウィンドウで起動することを確認

## 4. Cleanup
- [x] 4.1 古い `ts-mahjong` ディレクトリの削除または移行完了のREADME案内への置き換え
