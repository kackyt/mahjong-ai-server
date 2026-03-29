# Tasks: Integrate WASM AI

- [x] [Script] Create `scripts/build-wasm.ps1` to run `wasm-pack` and copy files <!-- id: 0 -->
- [x] [Rust] Update `ai_wasm/Cargo.toml` to include `cdylib` crate type <!-- id: 1 -->
- [x] [Rust] Implement/Export `get_discard` in `ai_wasm/src/lib.rs` <!-- id: 2 -->
- [x] [Frontend] Add `vite-plugin-wasm` (if needed) or configure Vite to serve wasm <!-- id: 3 -->
- [x] [Frontend] Create `useAi` hook in `ts-mahjong` that loads the WASM module <!-- id: 4 -->
- [x] [Test] Verify AI makes a move in the browser <!-- id: 5 -->
