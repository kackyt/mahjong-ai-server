---
targets: ["*"]
description: "Mahjong app for React programming guidelines"
globs: ["browser-app/**/*"]
---


# 技術スタック

- React 19
- TypeScript
- Vite
- Zustand
- Panda CSS (Zero-Runtime CSS-in-JS)
- React Router
- MUI base
- biome (formatter and linter)
- bun

# コード規約

- appにアプリケーションのもとになるページコンポーネントを配置する。
- featuresは機能で区切られたモジュールであり、必要に応じてhooksやcomponentsを配置する。
- 2つ以上のfeaturesで共通で使われるようなhooksやcomponentsはルートに配置することを検討する
- featuresからfeaturesへの依存は持って良い。
- features --> app の依存関係を持つ。逆の向きは避ける。
- componentsにはプレゼンテーションコンポーネントのみを配置する。グローバルなstoreを利用しない。
- biomeを使用してコードをフォーマットする
- biomeを使用してコードをチェックする
- functional componentを使用する
- interfaceではなくtypeを使用する
- ビジネスロジックはhooksを使って公開する
- コンポーネントはディレクトリをわけて、index.tsxを配置する。またコンポーネントのスタイルはPanda CSSを使用して定義する。
- inline styleは避ける
- コードをコミットするときは必ずlint, format, typecheck, testを実行してpassすることを確認する

# ディレクトリ構成

※ ディレクトリの戦略は [bullet react](https://github.com/alan2207/bulletproof-react/blob/master/docs/project-structure.md) を参考にしている

- docs
  - specification.md (仕様書)
- public
  - images
- src
  - components (presentational components)
  - hooks (global custom hooks)
  - app (page components)
    - title
    - single_player
  - features (feature modules)
    - ...
      - hooks
      - components
      - types
  - store (zustand store)
  - theme (panda css theme)
  - types (typescript types)
  - utils (utility functions)

# コマンド

## lint

```sh
bun lint
```

## format

```sh
bun format
```

## typecheck

```sh
bun typecheck
```

## dev server

```sh
bun dev
```

## test

```sh
bun test
```
