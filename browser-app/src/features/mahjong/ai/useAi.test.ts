import { describe, it, expect } from 'vitest';
import { get_discard } from 'ai_wasm';

describe('WASM AI', () => {
    it('should calculate a discard for a given hand', () => {
        // A simple hand: 11122233344455 m
        // pids: 0,0,0, 1,1,1, 2,2,2, 3,3,3, 4,4
        const tehai = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4];
        const state = { tehai };
        const jsonStr = JSON.stringify(state);

        console.log("Input JSON:", jsonStr);
        const resultStr = get_discard(jsonStr);
        console.log("Output JSON:", resultStr);

        const result = JSON.parse(resultStr);

        expect(result).toHaveProperty('discard');
        expect(typeof result.discard).toBe('number');
        expect(result.discard).toBeGreaterThanOrEqual(0);
        expect(result.discard).toBeLessThan(14);
    });
});
