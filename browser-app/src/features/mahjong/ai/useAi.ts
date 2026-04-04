import { useCallback } from "react";
import { get_discard } from "ai_wasm";

export interface AiGameState {
    tehai: number[];
}

export interface AiResult {
    discard: number;
}

export const useAi = () => {
    const computeMove = useCallback((gameState: AiGameState): AiResult | null => {
        try {
            const jsonStr = JSON.stringify(gameState);
            const resultJson = get_discard(jsonStr);
            return JSON.parse(resultJson) as AiResult;
        } catch (e) {
            console.error("AI execution failed:", e);
            return null;
        }
    }, []);

    return { isReady: true, computeMove };
};
