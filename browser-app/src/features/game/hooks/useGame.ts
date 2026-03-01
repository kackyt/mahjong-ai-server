import { useGameStore } from "../store/gameStore";

export const useGame = () => {
  return useGameStore();
};
