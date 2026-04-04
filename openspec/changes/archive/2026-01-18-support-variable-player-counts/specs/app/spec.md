## ADDED Requirements
### Requirement: Game Mode Selection
The application SHALL allow the user to select the game mode, including player count and opponent type.

#### Scenario: Select 1-Player Mode
- **WHEN** the user selects "1-Player" mode in settings
- **THEN** the game starts with 1 player (Solo)
- **AND** no AI opponents are initialized

#### Scenario: Select 4-Player Vs AI Mode
- **WHEN** the user selects "4-Player (Vs AI)" mode
- **THEN** the game starts with 4 players
- **AND** 3 AI opponents are initialized
- **AND** the AI selection dropdowns for Players 1-3 are visible in the settings UI

#### Scenario: Select 4-Player Manual Mode
- **WHEN** the user selects "4-Player (Manual)" mode
- **THEN** the game starts with 4 players
- **AND** no AI opponents are initialized (manual control for all seats)
- **AND** the AI selection dropdowns are hidden in the settings UI

#### Scenario: AI Selection Visibility
- **WHEN** the "1-Player (Solo)" or "4-Player (Manual)" mode is selected
- **THEN** the AI selection dropdowns are hidden from the user interface

## MODIFIED Requirements
### Requirement: Game Initialization
The application SHALL initialize the game with the configured number of players.

#### Scenario: Initialize 1-Player Game
- **WHEN** starting a 1-Player game
- **THEN** `GameState` is created with `player_len = 1`

#### Scenario: Initialize 4-Player Game
- **WHEN** starting a 4-Player game
- **THEN** `GameState` is created with `player_len = 4`
