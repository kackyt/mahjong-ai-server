import { describe, expect, it } from "vitest";
import type { Pai } from "../types";
import { PaiState } from "../utils/shanten";

// Helper to create mocked Pai
function createPai(num: number): Pai {
  return {
    paiNum: num,
    id: 0,
    isTsumogiri: false,
    isRiichi: false,
    isNakare: false,
  };
}

// Convert shorthand string to Pai[]
// 1m, 2p, 3s, 1z (Ton), etc.
function parseHand(str: string): Pai[] {
  const paiList: Pai[] = [];
  let nums: number[] = [];

  for (let i = 0; i < str.length; i++) {
    const c = str[i];
    if (c >= "0" && c <= "9") {
      nums.push(Number.parseInt(c, 10));
    } else {
      let offset = 0;
      if (c === "m") offset = 0;
      else if (c === "p") offset = 9;
      else if (c === "s") offset = 18;
      else if (c === "z") offset = 27;

      for (const n of nums) {
        // Adjust for 0-index: 1m -> 0, 9m -> 8
        // EXCEPT for Zihai: 1z -> 27 (Ton), 7z -> 33 (Chun)
        // My PaiNum definition:
        // 0-8: Manzu 1-9
        // 9-17: Pinzu 1-9
        // 18-26: Souzu 1-9
        // 27-33: Zihai

        let val = -1;
        if (offset === 27) {
          val = offset + n - 1;
        } else {
          val = offset + n - 1;
        }
        paiList.push(createPai(val));
      }
      nums = [];
    }
  }
  return paiList;
}

describe("Shanten Calculator", () => {
  it("Calculates normal tempai (0-shanten)", () => {
    // 123m 456p 789s 11z 22z -> 13 tiles (waiting for 1z or 2z)
    // 0-shanten (Tenpai)
    const hand = parseHand("123m456p789s11z22z");
    const state = new PaiState(hand);
    expect(state.getShanten(0)).toBe(0);
  });

  it("Calculates 1-shanten", () => {
    // 123m 456p 789s 11z 2z 3z -> 13 tiles.
    // M=3. Pair=1(11z). Koritsu=2(2z,3z).
    // Remove Pair 11z -> Rem: 123m456p789s2z3z. M=3, K=2.
    // Shanten = 1.
    const hand = parseHand("123m456p789s11z2z3z");
    const state = new PaiState(hand);
    expect(state.getShanten(0)).toBe(1);
  });

  it("Calculates Chitoitsu Tempai", () => {
    // 11 22 33m 44 55p 66s 7z -> 13 tiles
    const hand = parseHand("112233m4455p66s7z");
    const state = new PaiState(hand);
    expect(state.getShanten(0)).toBe(0);
  });

  it("Calculates Kokushi Musou Tempai", () => {
    // 19m 19p 19s 1234567z (13 tiles, waiting for pair)
    const hand = parseHand("19m19p19s1234567z");
    const state = new PaiState(hand);
    expect(state.getShanten(0)).toBe(0);
  });

  it("Calculates Kokushi 13-wait (Agari state basically)", () => {
    // 19m 19p 19s 1234567z + 1m (14 tiles) -> -1 shanten (Agari)
    const hand = parseHand("19m19p19s1234567z1m");
    const state = new PaiState(hand);
    expect(state.getShanten(0)).toBe(-1);
  });
});
