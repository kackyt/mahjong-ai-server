import { expose } from "comlink";
import init, { get_discard, get_shanten } from "ai_wasm";

let isInitialized = false;

const api = {
  async init() {
    if (!isInitialized) {
      await init();
      isInitialized = true;
    }
  },
  getDiscard(tehai: number[]) {
    if (!isInitialized) throw new Error("AI not initialized");
    // Ensure tehai are u8
    const tehaiU8 = new Uint8Array(tehai);
    return get_discard(tehaiU8);
  },
  getShanten(tehai: number[]) {
    if (!isInitialized) throw new Error("AI not initialized");
    const tehaiU8 = new Uint8Array(tehai);
    return get_shanten(tehaiU8);
  }
};

expose(api);

export type AiWorkerApi = typeof api;
