# Design: Implement Full Game Cycle

## Context
Currently, `GameProcess` manages a single-player state where `nextHand` just resets the tiles but keeps the score of the single player. To support a proper game, we need to track 4 players, manage the "Oya" (dealer), and progress through East 1-4 and South 1-4.

## Architecture
- **State Management**:
    - `GameState` will hold `players: Player[4]`.
    - `kyoku` (1-4) and `bakaze` (0=Ton, 1=Nan) will track progress.
    - `honba` and `riichi_sticks` (kyoutaku) will be tracked.
    - `oya` index (0-3) determines dealer.
- **UI Display**: Must show current Round (Kyoku), Honba sticks, Riichi sticks, and Oya badge.
- **Game Loop**:
    - **Init**: Randomize piles, deal 13 tiles to all 4 players.
    - **Turn**: Rotate 0->1->2->3, but must support interruption for Chi/Pon/Kan (Fuuro).
    - **Agari**:
        - Calculate basic points (Fu/Han).
        - Determine payments:
            - Tsumo: All others pay (split by Oya/Ko).
            - Ron: Discarder pays all.
        - Add/Subtract from `Player.score`.
        - Handle `riichi_sticks` (winner takes).
        - Check Oya consistency (Renchan if Oya wins).
    - **Ryukyoku**:
        - Check Tenpai/Noten.
        - Exchange 3000 points accordingly.
        - Handle `honba` (+1).
        - Oya rotation check (Tenpai = Renchan? Rules dependent, usually Oya Tenpai = Renchan).
- **Ranking**:
    - After Game Over, sort `players` by `score`.
    - **Scoring Rule**: 25,000 start / 30,000 return.
    - **Oka**: The difference ((30,000 - 25,000) * 4 = 20,000) is added to the top player's score.
    - No "Uma" (placement bonus like 10-30) required, just Oka.

## Trade-offs
- **AI**: 
    - AI must support **Riichi** decisions (checking Shanten/Tenpai).
    - AI must support **Fuuro** (Chi/Pon/Kan) checks (interrupting turn).
    - Reuse existing Wasm AI or implement basic random/heurestic for these new actions if Wasm AI is stateless/limited.

## Backend (Rust) Design
- **State**: `App` struct (main.rs) acts as the high-level Game Controller.
    - needs `bakaze`, `kyoku` fields.
- **Flow**:
    - `Agari` / `Ryukyoku` -> Show Modal -> "Next Hand" Button.
    - "Next Hand" Action:
        - Check logic (Oya rotation/Renchan).
        - Update `G_STATE.oya`, `tsumobou`, `riichibou`.
        - `G_STATE.shuffle()` -> `G_STATE.start()`.
        - If Game Over -> Show Ranking Modal.
- **Rules**: Simplifications might be made (e.g., no Chankan, no Double Ron handling yet) to focus on the loop.

## Data Structures
```typescript
interface GameState {
  // ... existing ...
  bakaze: number; // 0, 1
  kyoku: number; // 1-4
  honba: number;
  kyoutaku: number;
  oya: number; // 0-3
}
```
