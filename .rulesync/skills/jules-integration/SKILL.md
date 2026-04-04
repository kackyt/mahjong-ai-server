---
name: jules-integration
description: Jules AI（Googleの自律型コーディングエージェント）を専用のCLI（@google/jules）を通じて操作し、タスクの委譲、リモートセッション（Session）の管理、作業結果の取り込みを自律的に行います。Julesに対してコードのリファクタリング、バグ修正、テスト実装などを指示し、完了後にプルリクエストとして結果を確認する際に使用します。
---

# Jules Integration

Jules AI（Googleの自律型コーディングエージェント）をコマンドラインから操作するためのスキルです。

## 基本的なワークフロー

### 0. 事前チェック (Diagnostic)
Julesを実行する前に、以下のコマンドで環境が整っているか確認してください。
```bash
# バイナリと認証の確認
pnpm jules --help
```
> [!IMPORTANT]
> **ENOENTエラーが発生する場合**: `spawn ... jules.exe ENOENT` というエラーが出る場合は、Node.jsラッパーがバイナリを見失っています。
> その場合、 `C:\Users\t_kak\AppData\Local\Temp\jules_tmp\jules.exe` を直接実行するか、 `pnpm install --force @google/jules` を試行してください。

> [!NOTE]
> **認証の必須**: `jules login` が未完了の場合、コマンドはハングしたり失敗したりします。

## 基本的なワークフロー

### 1. タスクの委譲 (Remote New)
Julesに新しい作業を依頼します。
```bash
pnpm jules remote new --session "指示内容"
```

### 2. セッションの監視 (Remote List - 重要)
進行中のタスクの状態を確認します。
```bash
pnpm jules remote list --session
```
> [!TIP]
> Statusが `Planning` の場合、Julesはまだ計画を作成中か、計画の承認待ちです。
> `Completed` または `In Progress` (実行フェーズ) になっていることを確認してから成果物を取り込んでください。

### 3. 成果物の取り込み (Teleport / Pull)
作業結果を現在のローカルリポジトリに適用します。推奨される方法は以下の2つです。

#### 方法A: Teleport (推奨)
既存のリポジトリにセッションのパッチを直接適用します。
```bash
pnpm jules teleport <SESSION_ID>
```

#### 方法B: Remote Pull
明示的に成果物（パッチ）をダウンロードして適用します。
```bash
pnpm jules remote pull --session <SESSION_ID> --apply
```

> [!CAUTION]
> **取り込み前の注意**: 不意な変更の上書きを防ぐため、取り込み前にJulesのセッションを **Pause** しておくことが推奨されます。

## トラブルシューティング

### コマンドがハングする
- `jules login` が完了しているか確認してください。
- インターネット接続を確認してください。
- セッションの状態が `Planning` のままの場合、成果物がないため待機状態になることがあります。

### 成果物が適用されない
- `git status` で変更が未コミット状態で残っていないか確認してください。
- `teleport` と `pull --apply` の両方を試してください。

## Julesへの指示のコツ
- **具体性**: 具体的なファイルパスや型名、アーキテクチャパターン（例: Newtypeパターン, HECS, DDD）を指示に含めると、Julesがより正確に実装を行います。
- **コンテキストの提供**: 必要に応じて、特定ファイルの出力をパイプで渡すことも検討してください。
  `cat src/lib.rs | pnpm jules remote new --session "このファイルのリファクタリングをお願いします"`
