# Change: Support Variable Player Counts

## Why
Currently, the `mahjong-ai-server` app hardcodes the player count to 4. The user has requested support for both 4-player Mahjong and 1-player Mahjong (Solo/Efficiency practice).

## What Changes
- **UI**: Replace the "1-Player (vs 3 AI)" checkbox in `Settings` with a "Game Mode" dropdown (combobox).
    - Options: "4-Player (Vs AI)", "4-Player (Manual)", "1-Player (Solo)".
    - **Behavior**: The AI selection options for players 1-3 SHALL ONLY be visible when "4-Player (Vs AI)" is selected.
- **Logic**: 
    - Update `App::init_agents` and `Message::Start` to respect the selected player count.
    - Initialize `GameState` with `player_count = 1` for Solo mode.
    - Initialize `GameState` with `player_count = 4` for 4-Player modes.
    - Ensure AI only runs in "4-Player (Vs AI)" mode.

## Impact
- Affected specs: []
- Affected code:
    - `mahjong-ai-server/app/src/types.rs`: Add `GameMode` enum.
    - `mahjong-ai-server/app/src/pages/settings_page.rs`: Update UI.
    - `mahjong-ai-server/app/src/main.rs`: Handle game initialization and loop adaptation.
