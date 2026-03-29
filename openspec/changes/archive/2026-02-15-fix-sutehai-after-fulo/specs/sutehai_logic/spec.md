# Sutehai Logic Spec

## ADDED Requirements

### Requirement: Proper discard after fulo
The system MUST handle discards correctly when a player has performed a call (fulo) and does not have a tsumo tile.

#### Scenario: Discard handling after fulo
- **Given** プレイヤーがポン、チー、または大明槓を行った直後である（`is_tsumo` is false）
- **When** `sutehai` 関数が呼び出される
- **Then** インデックスが `tehai_len` 未満の場合、手牌からその牌が捨てられること
- **And** `tsumohai` が手牌にマージされないこと
- **And** インデックスが `tehai_len` 以上の場合（ツモ切り含む）はエラーとなること

### Requirement: Normal discard processing
The system MUST handle discards normally when a player has drawn a tile (tsumo), using dynamic hand size checks.

#### Scenario: Normal discard dealing (after tsumo)
- **Given** プレイヤーがツモを行っている（`is_tsumo` is true）
- **When** `sutehai` 関数が呼び出される
- **Then** インデックスが `tehai_len` と等しい場合、ツモ切りとして `tsumohai` が捨てられること
- **And** インデックスが `tehai_len` 未満の場合、手出しとして処理され、`tsumohai` が手牌にマージされソートされること
