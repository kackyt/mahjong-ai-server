# 設計: 鳴き後の捨て牌ロジック修正 (Dynamic Index Support)

## 現状の課題
- `sutehai` 関数が `index == 13` を特別扱いしてツモ切りと判定している。
- 副露（ポン・チー・カン）後は手牌枚数が減る（例: 13枚→10枚）ため、`index 13` は無効、かつ `index 10` が本来の「リスト外」にあたるが、現状のロジックではこれが考慮されていない。
- ツモ無し状態（`is_tsumo == false`）でも `tsumohai` のマージ処理が走る可能性がある。

## 変更内容
`sutehai` 関数内の判定ロジックを以下のように刷新します。

1. **ツモ切り判定の動的化**
   - 固定値 `13` の代わりに `player.tehai_len` を使用します。
   - `index == player.tehai_len` の場合、ツモ切り（Tsumogiri）とみなします。

2. **打牌バリデーション**
   - **Case A: `is_tsumo` が true (ツモ番)**
     - `index == player.tehai_len`: ツモ切り。`player.tsumohai` を捨てます。
     - `index < player.tehai_len`: 手出し。手牌の `index` を捨て、`tsumohai` を手牌に入れ、ソートします。
     - `index > player.tehai_len`: エラー（範囲外）。

   - **Case B: `is_tsumo` が false (副露直後など)**
     - `index < player.tehai_len`: 手出し。手牌の `index` を捨てます。**`tsumohai` のマージは行いません。**
     - `index >= player.tehai_len`: エラー。副露後はツモ牌がないため、ツモ切りは存在しません。

## 影響範囲
- `mahjong_core::game_process::GameStateT::sutehai`
- 本修正により、手牌枚数が13枚以外のルール（3人麻雀や少牌/多牌状態の許容など）への拡張性も向上します。
