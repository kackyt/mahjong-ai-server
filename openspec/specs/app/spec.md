# app Specification

## Purpose
TBD - created by archiving change integrate-wasm-ai. Update Purpose after archive.
## Requirements
### Requirement: WebAssembly AI Integration
The application SHALL integrate the Rust-based AI module via WebAssembly.

#### Scenario: AI Loading
- **WHEN** the Mahjong feature is initialized
- **THEN** the AI WASM module is loaded
- **AND** the AI interface is available for `useAi` hook

#### Scenario: AI Decision
- **WHEN** the `computeMove` function is called with a game state
- **THEN** the AI returns a valid discard or action

### Requirement: Tauri Desktop Architecture
The application SHALL be structured as a native desktop application using Tauri, integrating the React frontend with the Rust backend workspace.

#### Scenario: Desktop Application Launch
- **WHEN** the user launches the Tauri application binary
- **THEN** a native OS window opens rendering the React-based frontend
- **AND** the frontend successfully initializes and interacts with the underlying game logic

