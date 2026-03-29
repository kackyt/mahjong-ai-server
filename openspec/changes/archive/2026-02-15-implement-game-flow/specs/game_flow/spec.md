# Spec: Game Flow

## ADDED Requirements

### Requirement: Four Player Support
The game MUST support 4 players in the state, each with their own hand, discards, and score.

#### Scenario: Initialization
- **Given** a new game starts
- **When** `startGame` or `initGame` is called
- **Then** the state should contain 4 players
- **And** each player should be initialized with 25000 points (or custom amount)
- **And** the dealer (Oya) should be the player with index 0 (Ton)

### Requirement: Round Progression
The game MUST progress through rounds (kyoku) from East 1 to South 4.

#### Scenario: Next Hand
- **Given** a hand ends
- **When** `nextHand` is called
- **Then** the game should setup the next round
- **And** if the dealer (Oya) did not win or tenpai, the dealer should rotate to the next player
- **And** if the round was South 4 and the game is over, it should transition to Game Over state

### Requirement: Turn Interruption
The game flow MUST support turn variations for Fuuro (Chi, Pon, Kan).

#### Scenario: Call Actions
- **Given** a player discards a tile
- **When** another player calls Chi, Pon, or Kan
- **Then** the turn should jump to the calling player effectively skipping others

### Requirement: AI Capabilities
AI players MUST be able to perform standard actions including Riichi and Fuuro.

#### Scenario: AI Riichi
- **Given** an AI player is Tenpai and meets Riichi conditions
- **When** it is their turn
- **Then** the AI should be able to declare Riichi

#### Scenario: AI Fuuro
- **Given** a tile is discarded that completes a set for an AI
- **When** the AI logic triggers
- **Then** the AI should be able to interrupt and call Chi/Pon/Kan

### Requirement: Score Exchange
Points MUST be exchanged between players based on Agari (Win) or Ryuukyoku (Draw).

#### Scenario: Ron Agari
- **Given** Player A deals into Player B's Ron
- **When** score is calculated
- **Then** Player A's score determines the payment
- **And** Player A loses the points
- **And** Player B gains the points + any Riichi sticks

#### Scenario: Ryuukyoku
- **Given** the wall is exhausted (Ryukyoku)
- **When** the hand ends
- **Then** players who are Tenpai receive points from players who are Noten (3000 total exchanged)
- **And** Honba count should increase

### Requirement: Game Information Display
The UI MUST display critical game state information including Round, Dealer, and Sticks.

#### Scenario: Display Elements
- **Given** the game is in progress
- **When** the board is rendered
- **Then** it must show the current Round (e.g., East 1)
- **And** it must show the current Dealer (Oya)
- **And** it must show the number of Honba sticks
- **And** it must show the number of Riichi sticks

### Requirement: Game End and Ranking
The game MUST declare a winner and rank players at the end.

#### Scenario: Game Over
- **Given** the game reaches the end condition (e.g. after South 4)
- **When** the game ends
- **Then** the players should be sorted by their final score
- **And** the "Oka" (bonus from 25000 start / 30000 return) should be added to the top player
- **And** the result (rankings) should be displayed
