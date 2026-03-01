import { create } from "zustand";
import type { GameState } from "../../mahjong/types";
import { GameProcess } from "../utils/game_process";

interface GameStore {
  gameProcess: GameProcess;
  gameState: GameState;
  initGame: () => void;
  dahai: (index: number) => void;
  tsumo: () => void; // Usually auto, but for debugging?
  agari: () => void;
  nextTurn: () => void;
  nextHand: () => void;
  riichiMode: boolean;
  riichi: () => void;
  tryRiichiDiscard: (index: number) => void;
}

export const useGameStore = create<GameStore>((set, _get) => {
  const process = new GameProcess();

  return {
    gameProcess: process,
    gameState: process.state,
    riichiMode: false,

    initGame: () => {
      process.initGame();
      set({ gameState: { ...process.state }, riichiMode: false });
    },

    dahai: (index: number) => {
      set((state) => {
        if (state.riichiMode) return {}; // Prevent normal dahai in riichi mode if clicked directly? handled in UI
        return {};
      });
      process.dahai(index);
      set({ gameState: { ...process.state } });
    },

    tsumo: () => {
      process.tsumo();
      set({ gameState: { ...process.state } });
    },

    riichi: () => {
      set((state) => ({ riichiMode: !state.riichiMode }));
    },

    tryRiichiDiscard: (index: number) => {
      if (process.checkTenpaiAfterDiscard(index)) {
        process.riichi(); // Deduct score, set flag
        process.dahai(index); // Discard
        set({ gameState: { ...process.state }, riichiMode: false });
      } else {
        // Invalid discard for Riichi
        alert("聴牌にならない牌は切れません");
      }
    },

    agari: () => {
      process.agari();
      set({ gameState: { ...process.state } });
    },

    nextTurn: () => {
      process.nextTurn();
      set({ gameState: { ...process.state } });
    },

    nextHand: () => {
      process.nextHand();
      set({ gameState: { ...process.state }, riichiMode: false });
    },
  };
});
