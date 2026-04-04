## ADDED Requirements
### Requirement: WebAssembly AI Integration
The application SHALL integrate the Rust-based AI module via WebAssembly.

#### Scenario: AI Loading
- **WHEN** the Mahjong feature is initialized
- **THEN** the AI WASM module is loaded
- **AND** the AI interface is available for `useAi` hook

#### Scenario: AI Decision
- **WHEN** the `computeMove` function is called with a game state
- **THEN** the AI returns a valid discard or action
