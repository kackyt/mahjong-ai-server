import { describe, expect, it } from "vitest";
import { GameProcess } from "./game_process";

describe("GameProcess Flow", () => {
    it("initializes 4 players with 25000 points", () => {
        const process = new GameProcess();
        process.startGame();
        const state = process.state;
        expect(state.players.length).toBe(4);
        expect(state.players[0].score).toBe(25000);
        expect(state.players[3].score).toBe(25000);
        expect(state.bakaze).toBe(0);
        expect(state.kyoku).toBe(1);
        expect(state.honba).toBe(0);
        expect(state.kyoutaku).toBe(0);
        expect(state.oya).toBe(0);
    });

    it("handles Ryuukyoku and score exchange", () => {
        const process = new GameProcess();
        process.startGame();

        // Force Tenpai for Player 0
        process.state.players[0].tehai = []; // Empty hand is not tenpai, but let's mock the check or use implemented logic
        // ryukyoku() checks state. So we must set up a valid tenpai state or mock the check.
        // Or just manually set score diffs via internal check if accessible?
        // ryukyoku() uses new PaiState(tehai).getShanten().
        // Let's verify 'Ryukyoku' logic simply:
        // If we call ryukyoku("Time out") with empty hands, everyone Noten -> 0 pts.
        process.ryukyoku("Time out");
        expect(process.state.isGameOver).toBe(true);

        // Check nextHand logic
        // Everyone Noten -> Oya rotates. Honba increments.
        process.nextHand(); // Should start East 2
        expect(process.state.bakaze).toBe(0);
        expect(process.state.kyoku).toBe(2);
        expect(process.state.honba).toBe(1);
        expect(process.state.oya).toBe(1);
    });

    it("finishes game after South 4", () => {
        const process = new GameProcess();
        process.startGame();

        // Fast forward counting
        process.state.bakaze = 1; // South
        process.state.kyoku = 4;  // South 4
        process.state.honba = 0;
        process.state.oya = 3;    // Player 3 is Dealer
        process.state.currentTurn = 3;

        // Simulate Agari by Non-Dealer (Player 0)
        // Manually trigger agari steps or mock lastHandResult.
        // nextHand() relies on lastHandResult.
        process.state.lastHandResult = {
            type: 'Agari',
            winner: 0,
            loser: 3, // Ron
            scoreDiffs: [8000, 0, 0, -8000],
            isTsumo: false
        };
        process.state.players[0].score = 33000;
        process.state.players[3].score = 17000;

        process.nextHand();

        // Should be Game Over (West 1 is invalid)
        expect(process.state.isGameOver).toBe(true);
        expect(process.state.resultMessage).toContain("Game Over");
        // Check Oka: Top player (0) should have +20000 -> 53000
        // But logic applies score diffs via lastHandResult in Agari? No nextHand doesn't apply scores.
        // Agari applies scores check.
        // finishGame applies Oka.
        // players[0] was 33000. +20000 = 53000.
        const topScore = process.state.players[0].score;
        expect(topScore).toBe(53000);
    });
});
