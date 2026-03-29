# Design: WASM AI Integration

## Architecture
The system bridges the Rust-based Mahjong AI (`mahjong-ai-server`) and the React Frontend (`ts-mahjong`) using WebAssembly.

```mermaid
graph LR
    Rust[Rust AI Core] -->|wasm-pack| Wasm[Wasm Module (.wasm + .js)]
    Wasm -->|Copy| Frontend[ts-mahjong /src/libraries]
    Frontend -->|Import| React[React Components]
```

## Build Pipeline
1. **Compilation**: `ai_wasm` is compiled using `wasm-pack build --target web`.
2. **Distribution**: The resulting `pkg` directory is synchronized to `ts-mahjong/src/libraries/ai_wasm`.
3. **Consumption**: The frontend imports the module dynamically or statically (depending on Vite config).

## Interface
The `ai_wasm` crate must expose:
- `init()`: Default export from `wasm-bindgen` to initialize memory.
- `get_discard(json_state: String) -> String`: Main AI function taking a JSON game state and returning a decision.
- (Optional) `eval_hand(tiles: &[u8]) -> Value`: Helper functions for valid move checking.

## Constraints
- **Async Loading**: WASM must be initialized asynchronously. The UI should handle the "Loading AI..." state.
- **Asset Handling**: The `.wasm` file must be served by Vite. Placing it in `src` vs `public` depends on the import strategy. Using `wasm-pack --target web` usually generates JS that fetches the wasm file relative to itself.
