---
targets: ["*"]
description: "Mahjong app for Rust programming guidelines"
globs: ["**/*"]
---

# はじめに

このプロジェクトは麻雀AIおよび4人うち麻雀アプリを実現するためのプロジェクトです。
Rustを使用して開発します。


# 技術スタック

- Rust
- cargo
- anyhow
- flatbuffers
- iced

# ディレクトリ構成

- app (icedを使ったネイティブアプリ dll検証用)
- browser-app (Tauriを使ったネイティブアプリ)
- loadlibrary (Windows dllを読み込むcrate)
- ai_bridge (まうじゃんプラグイン実装)
- ai_dll (ai dll作成基盤)
- ai_wasm (ai wasm作成基盤)
- fbs (flatbuffers 定義ファイル)
- game_lib (mahjong_coreとdllのbridge)
- mahjong_ai (AIロジック本体)
- mahjong_core (麻雀ゲーム進行基盤)
- proc_macros (マクロ)
- sample (自動うちサンプル)
- server (コマンドベースcliサンプル)


# コーディング規約

- `cargo clippy --all-targets --all-features -- -D warnings` がエラーなく通ること
- `cargo fmt --all -- --check` がエラーなく通ること
- `cargo test` が正常に動作すること
- エラーハンドリングを適切に行うこと
- mahjong_coreのG_STATEをsingle source of truthにすること。他のモジュールやアプリケーションなどで重複したStateを持たないようにする。
- appのstructの変数はViewModelであり、mahjong_coreのG_STATE(Model)からViewを表現するための写像である。このViewModelはコマンドが実行されたときにModelを更新したのちにViewModelを更新し、Viewを再描画する。Viewのレンダリング時(view)ではViewModelおよび、Modelの更新は決して行わないこと。

