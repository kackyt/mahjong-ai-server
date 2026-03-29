# Change: Integrate Rust AI WASM into ts-mahjong

## Summary
Integrated Rust-based Mahjong AI into the frontend using WebAssembly.

## What Changed
- Created build scripts for WASM.
- Updated crate type to `cdylib`.
- Implemented `useAi` hook.

## Impact
- **Affected specs**:
    - `app`
- **Affected code**:
    - `mahjong-ai-server/ai_wasm/*`
    - `ts-mahjong/src/features/mahjong/ai/*`
    - `ts-mahjong/vite.config.ts`
