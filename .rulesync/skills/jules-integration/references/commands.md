# Jules CLI コマンドリファレンス

Julesは、分離されたクラウド環境でタスクを実行する自律型AIコーディングエージェントです。

## 基本コマンド

### プロジェクトへの追加
プロジェクトで `pnpm jules` を使用できるようにするために、以下のコマンドが事前に実行されている必要があります。
```bash
pnpm add -D @google/jules
```

### 認証
Julesを使用するにはGoogleアカウントでのログインが必要です。
```bash
pnpm jules login
pnpm jules logout
```

## リモートセッション管理

### セッションの新規作成
Julesにタスクを依頼します。
```bash
pnpm jules remote new --session "タスクの内容"
```
例:
```bash
pnpm jules remote new --session "UnitIdをNewtypeパターンで実装し、ドメインロジックをリファクタリングしてください"
```

### セッション一覧の表示
現在進行中または完了したセッションを確認します。
```bash
pnpm jules remote list --session
```

## 成果物の取り込み

### 方法A: Teleport (推奨)
既存のリポジトリにセッションのパッチを直接適用します。
```bash
pnpm jules teleport <SESSION_ID>
```

### 方法B: Remote Pull
セッションの結果を取得し、適用します。
```bash
pnpm jules remote pull --session <SESSION_ID> --apply
```

## セッションステータスの見方
`pnpm jules remote list --session` で表示されるステータスの意味：
- `Planning`: 計画作成中または承認待ち。成果物（パッチ）はまだありません。
- `Planned` / `In Progress`: 実行フェーズ. コード変更が行われています。
- `Complete`: 正常終了。成果物が取り込み可能です。
- `Failed`: 失敗。

## トラブルシューティング：ENOENTエラー
`spawn ... jules.exe ENOENT` が発生した場合は、以下の直接パスを試してください。
`C:\Users\t_kak\AppData\Local\Temp\jules_tmp\jules.exe`

また、 `pnpm install --force @google/jules` でパッケージを強制再インストールすることで、パス設定が更新される場合があります。

## プルリクエストについて
Julesはタスクを完了すると、自動的にフィーチャーブランチを作成してプルリクエストをオープンします。CLIから明示的に「PRを作成する」コマンドを叩く必要はありません。
セッション完了後、GitHub上でPRを確認してください。
