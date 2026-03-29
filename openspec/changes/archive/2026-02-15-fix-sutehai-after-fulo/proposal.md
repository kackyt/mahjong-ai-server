# Fix Sutehai Logic after Fulo

## Why
Current implementation lacks logic for discarding after calling (fulo) and incorrectly relies on a hardcoded index (13) for tsumogiri. This fails when the hand size varies (e.g., after calls or in 3-player variants), leading to invalid state transitions.
現在、`game_process.rs`の`sutehai`関数はツモ切り判定に固定値（13）を使用しており、副露後や手牌枚数が変化した場合に対応できていません。
また、副露直後の打牌時にはツモ牌が存在しないにもかかわらず、誤ってツモ牌を手牌にマージするロジックが動く可能性があります。

## What Changes
Update `sutehai` to determine tsumogiri based on `tehai_len` instead of hardcoded 13, and ensure valid discards after fulo.
ツモ切り判定を `index == tehai_len`（手牌枚数と等しいか）で行うように変更し、固定値13への依存を排除します。
また、`is_tsumo` フラグを確認し、副露直後（ツモ無し状態）でのツモ切りや不正なインデックス指定を防止します。

## How
Refactor `sutehai` in `game_process.rs`:
1. Use `player.tehai_len` to validate the discard index.
2. If `index == player.tehai_len`, treat as tsumogiri (only allowed if `is_tsumo` is true).
3. If `index < player.tehai_len`, treat as hand discard (tedashi).
4. If `is_tsumo` is false (post-fulo), ensure `index < player.tehai_len` and skip tsumohai merging.
