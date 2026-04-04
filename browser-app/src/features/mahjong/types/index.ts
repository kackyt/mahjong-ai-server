export type PaiNum = number; // 0-33

// 0-8: Manzu (1-9)
// 9-17: Pinzu (1-9)
// 18-26: Souzu (1-9)
// 27-33: Zihai (Ton, Nan, Sha, Pei, Haku, Hatsu, Chun)

export interface Pai {
  paiNum: PaiNum;
  id: number; // 0-3 (same tile identifier)
  isTsumogiri: boolean;
  isRiichi: boolean;
  isNakare: boolean;
}

export const MentsuType = {
  Shuntsu: 0,
  Koutsu: 1,
  Minkan: 2,
  Ankan: 3,
  Atama: 4,
} as const;
export type MentsuType = (typeof MentsuType)[keyof typeof MentsuType];

export interface Mentsu {
  type: MentsuType;
  paiList: Pai[];
}

export interface Player {
  score: number;
  tehai: Pai[]; // 手牌
  tsumohai: Pai | null; // 自摸牌
  kawahai: Pai[]; // 河牌
  isRiichi: boolean;
  isDoubleRiichi: boolean;
  isIppatsu: boolean;
  isTsumo: boolean;
  // Win Flags (set by game process/state)
  isHaitei?: boolean; // Haitei / Houtei
  isRinshan?: boolean;
  isChankan?: boolean;
  // mentsu: Mentsu[]; // Fuuro count etc. if needed
  fuuro: Mentsu[]; // 副露
  wind: number; // 0: Ton, 1: Nan, 2: Sha, 3: Pei (Zikaze)
  shanten: number;
}

export interface GameState {
  players: Player[];
  yama: Pai[];
  dora: Pai[];
  uraDora: Pai[];
  currentTurn: number; // Player index (0 for single player main)
  turnCount: number; // For debugging or limits
  bakaze: number; // 0: Ton, 1: Nan
  kyoku: number; // 1-4 (e.g. East 1 = bakaze 0, kyoku 1)
  honba: number; // Number of Honba sticks (100 pts each usually, 300 for calculation)
  kyoutaku: number; // Riichi sticks on table (1000 pts each)
  oya: number; // Dealer index (0-3)

  isGameOver: boolean;
  resultMessage: string | null;
  lastHandResult?: HandResult;
}

export interface HandResult {
  type: 'Agari' | 'Ryukyoku';
  winner?: number; // Player index (multiple possible implies array, but let's stick to single winner or headbump for now)
  loser?: number | null; // Player index (null for Tsumo)
  isTsumo?: boolean;
  tenpai?: boolean[]; // For Ryuukyoku
  scoreDiffs: number[]; // Score changes for each player
  yakuList?: string[]; // For display
  han?: number;
  fu?: number;
  score?: number; // Basic score (e.g. 8000)
}

// Fixed constants
export const PAI_COUNT = 136;
export const MANZU_OFFSET = 0;
export const PINZU_OFFSET = 9;
export const SOUZU_OFFSET = 18;
export const ZIHAI_OFFSET = 27;
