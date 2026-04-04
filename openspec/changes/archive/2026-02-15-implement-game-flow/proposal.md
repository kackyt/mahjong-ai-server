# Change: Implement Game Flow (South 4 Rounds & Ranking)

## Why
The current application already has a basic 4-player mode, but it resets after each hand (Solitaire style). To support a standard Mahjong game (Hanchan), we need to implement the progression logic up to South 4, manage player scores across rounds, and calculate final rankings. The user explicitly requested "South 4 progression, total score calculation, and ranking".

## What Changes
- **Frontend (TS)**:
    - **Game State**: Expand `GameState` to support 4 players, round tracking (`bakaze`, `kyoku`), stick counts (`honba`, `kyoutaku`), and dealer (`oya`) rotation.
    - **Logic**: Implement `nextHand` in `GameProcess` to handle Oya rotation, Honba accumulation, and Game Over conditions (after South 4).
    - **Scoring**: Implement proper `agari` score calculation (Han/Fu based) and `ryukyoku` point exchange.
    - **Ranking**: Sort players by score after Game Over (Oka rule).
    - **UI**: Update `GamePage` header to show Round info (e.g. East 1), sticks, and Oya.
- **Backend (Rust)**:
    - **App Logic**: Update `mahjong-ai-server/app` to support the same Hanchan flow.
    - **Game Loop**: Refactor `main.rs` to handle "Next Hand" transitions instead of just ending.
    - **State Management**: Track `bakaze`, `kyoku` in `App` and update `G_STATE` (oya, sticks) accordingly.yers.
    - Implement game termination condition (End of South 4).
- **Ranking**: Implement final score sorting and result display.
- **UI**: Update `GamePage` to display current round (e.g., East 1), wind, and final results table.

## Impact
- **Affected specs**:
    - `game_flow` (New/Modify)
- **Affected code**:
    - `ts-mahjong/src/features/mahjong/types/index.ts` (State definitions)
    - `ts-mahjong/src/features/game/utils/game_process.ts` (Core logic)
    - `ts-mahjong/src/features/game/store/gameStore.ts` (Store wrapper)
    - `ts-mahjong/src/app/pages/GamePage.tsx` (UI)
