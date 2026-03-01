import { type GameState, type Mentsu, MentsuType, type Pai, type Player } from "../types";

// 和了（あがり）情報
export interface AgariResult {
  score: number;
  han: number;
  fu: number;
  yaku: string[];
  yakuman: string[];
  isYakuman: boolean;
}

/**
 * 点数計算
 */
export function calculateScore(han: number, fu: number, isOya: boolean, _isTsumo: boolean): number {
  if (han === 0) return 0;

  let basePoint = fu * 2 ** (2 + han);

  // 満貫以上固定
  if (han >= 5) {
    // 満貫〜
    if (han >= 13)
      basePoint = 8000; // 数え役満
    else if (han >= 11)
      basePoint = 6000; // 三倍満
    else if (han >= 8)
      basePoint = 4000; // 倍満
    else if (han >= 6)
      basePoint = 3000; // 跳満
    else basePoint = 2000; // 満貫
  } else {
    if (basePoint > 2000) basePoint = 2000; // 満貫打ち切り
  }

  const oyaRate = isOya ? 6 : 4; // 親は6倍、子は4倍
  // 点数は100点単位切り上げ
  const score = Math.ceil((basePoint * oyaRate) / 100) * 100;

  return score;
}

// 符計算
export function calculateFu(
  mentsuList: Mentsu[],
  _head: Pai,
  isTsumo: boolean,
  menzen: boolean,
  bakaze: number,
  zikaze: number,
): number {
  let fu = 20; // 符底

  // 門前ロンなら+10
  if (menzen && !isTsumo) {
    fu += 10;
  }
  // ツモなら+2 (ピンフツモを除くが、それは役判定で処理されることが多い。ここでは単純加算し、ピンフ判定で調整)
  if (isTsumo) {
    fu += 2;
  }

  // 各面子の符
  for (const m of mentsuList) {
    if (m.type === MentsuType.Koutsu) {
      let val = 2; // 中張牌・明刻
      const p = m.paiList[0];
      if (isYaochu(p.paiNum)) val *= 2; // 幺九牌
      // 明刻/暗刻の判別:
      // decomposeから渡されるHandPatternの場合は「手牌にある」= 暗刻扱い
      // 副露(Minkan)の場合は Minkan
      // 副露(Koutsu) -> 明刻 (should be Minkan usually, but check type)
      // ここでは簡易的に、inputのtypeで判断する。
      val *= 2; // Assume Ankou by default for Koutsu type in logic?
      // Better logic relies on caller knowing types.
      fu += val;
    }
    if (m.type === MentsuType.Ankan) {
      let val = 16;
      const p = m.paiList[0];
      if (isYaochu(p.paiNum)) val *= 2;
      fu += val;
    }
    if (m.type === MentsuType.Minkan) {
      let val = 8;
      const p = m.paiList[0];
      if (isYaochu(p.paiNum)) val *= 2;
      fu += val;
    }
    if (m.type === MentsuType.Atama) {
      // 役牌の頭は+2
      const n = m.paiList[0].paiNum;
      if (n >= 27) {
        if (n === 27 + bakaze) fu += 2; // 場風
        if (n === 27 + zikaze) fu += 2; // 自風
        if (n >= 31) fu += 2; // 三元牌
      }
    }
  }

  return Math.ceil(fu / 10) * 10;
}

function isYaochu(n: number): boolean {
  return (
    (n < 9 && (n === 0 || n === 8)) ||
    (n >= 9 && n < 18 && (n === 9 || n === 17)) ||
    (n >= 18 && n < 27 && (n === 18 || n === 26)) ||
    n >= 27
  );
}

// 手牌構成分析用
class HandPattern {
  mentsuList: Mentsu[] = [];
  head: Pai | null = null;
}

// 再帰的に面子分解を行う
function decompose(
  counts: number[],
  remaining: number,
  current: HandPattern,
  patterns: HandPattern[],
) {
  if (remaining === 0) {
    // 完成
    if (current.head && current.mentsuList.length === 4) {
      // Deep copy structure
      const pat = new HandPattern();
      pat.head = current.head;
      pat.mentsuList = current.mentsuList.map((m) => ({ ...m, paiList: [...m.paiList] }));
      patterns.push(pat);
    }
    return;
  }

  // 面子を探す (インデックス順)
  for (let i = 0; i < 34; i++) {
    if (counts[i] > 0) {
      // 1. 雀頭 (未確定の場合)
      if (!current.head && counts[i] >= 2) {
        counts[i] -= 2;
        current.head = { paiNum: i } as Pai;
        decompose(counts, remaining - 2, current, patterns);
        current.head = null;
        counts[i] += 2;
      }

      // 2. 刻子
      if (counts[i] >= 3) {
        counts[i] -= 3;
        current.mentsuList.push({
          type: MentsuType.Koutsu,
          paiList: [{ paiNum: i }, { paiNum: i }, { paiNum: i }] as Pai[],
        });
        decompose(counts, remaining - 3, current, patterns);
        current.mentsuList.pop();
        counts[i] += 3;
      }

      // 3. 順子 (数牌かつi < 27)
      if (i < 27 && i % 9 <= 6) {
        if (counts[i] >= 1 && counts[i + 1] >= 1 && counts[i + 2] >= 1) {
          counts[i]--;
          counts[i + 1]--;
          counts[i + 2]--;
          current.mentsuList.push({
            type: MentsuType.Shuntsu,
            paiList: [{ paiNum: i }, { paiNum: i + 1 }, { paiNum: i + 2 }] as Pai[],
          });
          decompose(counts, remaining - 3, current, patterns);
          current.mentsuList.pop();
          counts[i]++;
          counts[i + 1]++;
          counts[i + 2]++;
        }
      }
      return;
    }
  }
}

// ドラ表示牌からドラ牌番号を取得
function getDoraNext(indicator: number): number {
  if (indicator < 27) {
    if (indicator % 9 === 8) return indicator - 8;
    return indicator + 1;
  }
  if (indicator < 31) {
    if (indicator === 30) return 27;
    return indicator + 1;
  }
  if (indicator === 33) return 31;
  return indicator + 1;
}

// --- Yaku Helpers ---
function isSanshoku(mentsuList: Mentsu[]): boolean {
  const shuntsu = mentsuList.filter((m) => m.type === MentsuType.Shuntsu);
  if (shuntsu.length < 3) return false;
  const byNum: Mentsu[][] = Array.from({ length: 9 }, () => []);
  for (const m of shuntsu) {
    const start = m.paiList[0].paiNum;
    if (start >= 27) continue;
    byNum[start % 9].push(m);
  }
  for (const group of byNum) {
    if (group.length >= 3) {
      const suits = new Set(group.map((m) => Math.floor(m.paiList[0].paiNum / 9)));
      if (suits.size >= 3) return true;
    }
  }
  return false;
}

function isItsu(mentsuList: Mentsu[]): boolean {
  const shuntsu = mentsuList.filter((m) => m.type === MentsuType.Shuntsu);
  if (shuntsu.length < 3) return false;
  const bySuit: number[][] = [[], [], []];
  for (const m of shuntsu) {
    const start = m.paiList[0].paiNum;
    if (start >= 27) continue;
    const suit = Math.floor(start / 9);
    const num = start % 9;
    bySuit[suit].push(num);
  }
  for (const suitNums of bySuit) {
    if (suitNums.includes(0) && suitNums.includes(3) && suitNums.includes(6)) return true;
  }
  return false;
}

function isToitoi(mentsuList: Mentsu[], fuuro: Mentsu[]): boolean {
  const all = [...mentsuList, ...fuuro];
  if (all.length !== 4) return false;
  return all.every(
    (m) =>
      m.type === MentsuType.Koutsu || m.type === MentsuType.Ankan || m.type === MentsuType.Minkan,
  );
}

function isSanankou(
  patMentsu: Mentsu[],
  fuuro: Mentsu[],
  isTsumo: boolean,
  agariPai: Pai,
): boolean {
  let ankouCount = fuuro.filter((m) => m.type === MentsuType.Ankan).length;
  const patKoutsu = patMentsu.filter((m) => m.type === MentsuType.Koutsu);

  if (isTsumo) {
    ankouCount += patKoutsu.length;
  } else {
    let hit = false;
    for (const k of patKoutsu) {
      if (!hit && k.paiList[0].paiNum === agariPai.paiNum) {
        hit = true;
      } else {
        ankouCount++;
      }
    }
    if (!hit) ankouCount += patKoutsu.length;
  }
  return ankouCount >= 3;
}

function isSankantsu(fuuro: Mentsu[]): boolean {
  const kanCount = fuuro.filter(
    (m) => m.type === MentsuType.Ankan || m.type === MentsuType.Minkan,
  ).length;
  return kanCount >= 3;
}

function getTerminalsAndHonorsState(allPai: Pai[]) {
  const isAllYaochu = allPai.every((p) => isYaochu(p.paiNum));
  const hasYaochu = allPai.some((p) => isYaochu(p.paiNum));
  return { isAllYaochu, hasYaochu };
}

function checkTerminalBlocks(allMentsu: Mentsu[], head: Pai): boolean {
  if (!isYaochu(head.paiNum)) return false;
  for (const m of allMentsu) {
    if (!m.paiList.some((p) => isYaochu(p.paiNum))) return false;
  }
  return true;
}

function checkColors(allPai: Pai[]) {
  let hasMan = false;
  let hasPin = false;
  let hasSou = false;
  let hasZi = false;
  for (const p of allPai) {
    const n = p.paiNum;
    if (n < 9) hasMan = true;
    else if (n < 18) hasPin = true;
    else if (n < 27) hasSou = true;
    else hasZi = true;
  }
  const suits = (hasMan ? 1 : 0) + (hasPin ? 1 : 0) + (hasSou ? 1 : 0);
  const isChinitsu = suits === 1 && !hasZi;
  const isHonitsu = suits === 1 && hasZi;
  return { isHonitsu, isChinitsu };
}

function checkYakuman(
  player: Player,
  agariPai: Pai,
  allPai: Pai[],
  patterns: HandPattern[],
  _gameState: GameState,
  isMenzen: boolean,
): AgariResult | null {
  const yakumanList: string[] = [];

  // 1. Kokushi Musou (Special check, ignores patterns)
  // Condition: 13 distinct Yaochu + 1 duplicate Yaochu
  if (isMenzen) {
    const yaochu = allPai.filter((p) => isYaochu(p.paiNum));
    if (yaochu.length === 14) {
      const unique = new Set(yaochu.map((p) => p.paiNum));
      if (unique.size === 13) {
        yakumanList.push("国士無双");
        // Double? (13 wait) - logic requires knowing wait.
        // If agariPai was the duplicate (pair), it's 13-wait?
        // Simplifying: Just Kokushi.
      }
    }
  }

  if (yakumanList.length > 0) {
    return {
      score: calculateScore(13, 0, false, player.isTsumo),
      han: 13,
      fu: 0,
      yaku: [],
      yakuman: yakumanList,
      isYakuman: true,
    };
  }

  // 2. Pattern-based Yakuman
  let maxHan = 0;
  let maxYakuman: string[] = [];

  for (const pat of patterns) {
    const y: string[] = [];
    const allMentsu = [...pat.mentsuList, ...player.fuuro];

    // Daisangen
    let dragonKoutsu = 0;
    for (const m of allMentsu) {
      if (m.type !== MentsuType.Shuntsu && m.paiList[0].paiNum >= 31) dragonKoutsu++;
    }
    if (dragonKoutsu === 3) y.push("大三元");

    // Four Winds
    let windKoutsu = 0;
    let windHead = false;
    if (pat.head && pat.head.paiNum >= 27 && pat.head.paiNum <= 30) windHead = true;
    for (const m of allMentsu) {
      if (m.type !== MentsuType.Shuntsu && m.paiList[0].paiNum >= 27 && m.paiList[0].paiNum <= 30)
        windKoutsu++;
    }
    if (windKoutsu === 4) y.push("大四喜");
    else if (windKoutsu === 3 && windHead) y.push("小四喜");

    // Tsuuiisou
    if (allPai.every((p) => p.paiNum >= 27)) y.push("字一色");

    // Chinroutou
    if (allPai.every((p) => isYaochu(p.paiNum) && p.paiNum < 27)) y.push("清老頭");

    // Ryuuiisou (Sou 2,3,4,6,8 + 32(Hats))
    const green = [19, 20, 21, 23, 25, 32];
    if (allPai.every((p) => green.includes(p.paiNum))) y.push("緑一色");

    // Suukantsu
    if (isSankantsu(player.fuuro)) {
      // Logic in helper is >=3. Need 4.
      const kans = player.fuuro.filter(
        (m) => m.type === MentsuType.Ankan || m.type === MentsuType.Minkan,
      ).length;
      if (kans === 4) y.push("四槓子");
    }

    // Suuankou (Menzen Only)
    if (isMenzen) {
      const patKoutsu = pat.mentsuList.filter((m) => m.type === MentsuType.Koutsu);
      // Fuuro must be empty for Menzen (or Ankan).
      // If Menzen, Fuuro is Ankan only.
      const fuuroAnkou = player.fuuro.filter((m) => m.type === MentsuType.Ankan).length;

      if (player.isTsumo) {
        // Tsumo: All patKoutsu are Ankou
        if (patKoutsu.length + fuuroAnkou === 4) y.push("四暗刻");
      } else {
        // Ron: Only if Tanki wait (Head wait)
        // If AgariPai makes the Head, then all Koutsu are Ankou.
        if (pat.head && pat.head.paiNum === agariPai.paiNum) {
          // Check if head was formed by AgariPai.
          // agariPai is included in allPai and handCounts.
          // decompose picked it.
          // Logic: If pattern has 4 Koutsu + Head.
          // And we are Ronning the Head.
          // Then 4 Koutsu are preserved as Ankou.
          if (patKoutsu.length + fuuroAnkou === 4) y.push("四暗刻単騎");
        }
      }
    }

    // Chuuren Poutou (Menzen Chinitsu + 1112345678999 shape)
    // Checking shape is hard on decompose patterns.
    // But valid Chuuren is always a Chinitsu.
    // Check histogram of numbers.
    if (isMenzen) {
      const { isChinitsu } = checkColors(allPai);
      if (isChinitsu) {
        // Get counts of the suit
        const counts = new Array(9).fill(0);
        for (const p of allPai) counts[p.paiNum % 9]++;

        // Requirement: 1->3+, 9->3+, 2-8->1+ (Total 13) + 1 extra
        // Base: 3,1,1,1,1,1,1,1,3 = 13 tiles.
        // Check if we meet base
        if (counts[0] >= 3 && counts[8] >= 3 && counts.slice(1, 8).every((c) => c >= 1)) {
          // Yes
          y.push("九蓮宝燈"); // Or Pure if 9-way wait?
        }
      }
    }

    if (y.length > 0) {
      const han = y.length * 13; // Simple calc
      if (han > maxHan) {
        maxHan = han;
        maxYakuman = y;
      }
    }
  }

  if (maxYakuman.length > 0) {
    return {
      score: calculateScore(maxHan, 0, false, player.isTsumo),
      han: maxHan,
      fu: 0,
      yaku: [],
      yakuman: maxYakuman,
      isYakuman: true,
    };
  }
  return null;
}

// 役判定ロジック
export function checkYaku(player: Player, gameState: GameState, agariPai: Pai): AgariResult {
  // 1. 全牌リスト
  let allPai = [...player.tehai, agariPai];
  for (const m of player.fuuro) {
    allPai = [...allPai, ...m.paiList];
  }
  const handCounts = new Array(34).fill(0);
  const handTiles = [...player.tehai, agariPai];
  for (const p of handTiles) {
    handCounts[p.paiNum]++;
  }

  // 2. 分解
  const patterns: HandPattern[] = [];
  const remainingExposed = 14 - player.fuuro.length * 3;
  decompose(handCounts, remainingExposed, new HandPattern(), patterns);

  const isMenzen =
    player.fuuro.length === 0 || player.fuuro.every((m) => m.type === MentsuType.Ankan);

  // Check Yakuman First
  const yakumanResult = checkYakuman(player, agariPai, allPai, patterns, gameState, isMenzen);
  if (yakumanResult) return yakumanResult;

  // ドラ
  let doraHan = 0;
  const doraYaku: string[] = [];
  const realDoraNums = gameState.dora.map((p) => getDoraNext(p.paiNum));
  let normalDora = 0;
  for (const p of allPai) {
    if (realDoraNums.includes(p.paiNum)) {
      normalDora += realDoraNums.filter((d) => d === p.paiNum).length;
    }
  }
  if (normalDora > 0) {
    doraHan += normalDora;
    doraYaku.push(`ドラ ${normalDora}`);
  }

  if (player.isRiichi) {
    const realUraNums = gameState.uraDora
      .slice(0, gameState.dora.length)
      .map((p) => getDoraNext(p.paiNum));
    let ura = 0;
    for (const p of allPai) {
      if (realUraNums.includes(p.paiNum)) {
        ura += realUraNums.filter((d) => d === p.paiNum).length;
      }
    }
    if (ura > 0) {
      doraHan += ura;
      doraYaku.push(`裏ドラ ${ura}`);
    }
  }

  // Special: Chiitoitsu
  if (patterns.length === 0) {
    const pairs = handCounts.filter((c) => c === 2).length;
    if (pairs === 7 && remainingExposed === 14) {
      const fu = 25;
      const yaku = ["七対子"];
      let han = 2;

      if (player.isDoubleRiichi) {
        yaku.push("ダブル立直");
        han += 2;
      } else if (player.isRiichi) {
        yaku.push("立直");
        han++;
      }

      if (player.isIppatsu) {
        yaku.push("一発");
        han++;
      }
      if (player.isTsumo) {
        yaku.push("門前清自摸和");
        han++;
      }
      if (player.isHaitei) {
        yaku.push(player.isTsumo ? "海底摸月" : "河底撈魚");
        han++;
      }

      if (allPai.every((p) => !isYaochu(p.paiNum))) {
        yaku.push("断么九");
        han++;
      }

      const { isHonitsu, isChinitsu } = checkColors(allPai);
      if (isChinitsu) {
        yaku.push("清一色");
        han += 6;
      } else if (isHonitsu) {
        yaku.push("混一色");
        han += 3;
      }

      // Add Dora
      han += doraHan;
      yaku.push(...doraYaku);

      const score = calculateScore(han, fu, false, player.isTsumo);
      return { score, han, fu, yaku, yakuman: [], isYakuman: false };
    }
    return { score: 0, han: 0, fu: 0, yaku: [], yakuman: [], isYakuman: false };
  }

  // 3. Max Result Loop
  let maxResult: AgariResult = { score: 0, han: 0, fu: 0, yaku: [], yakuman: [], isYakuman: false };

  for (const pat of patterns) {
    const yaku: string[] = [];
    let han = 0;
    let fu = 0;

    // --- 1 Han ---
    if (isMenzen) {
      if (player.isDoubleRiichi) {
        yaku.push("ダブル立直");
        han += 2;
      } else if (player.isRiichi) {
        yaku.push("立直");
        han++;
      }

      if (player.isIppatsu) {
        yaku.push("一発");
        han++;
      }
      if (player.isTsumo) {
        yaku.push("門前清自摸和");
        han++;
      }
    }

    // Tanyao
    if (allPai.every((p) => !isYaochu(p.paiNum))) {
      yaku.push("断么九");
      han++;
    }

    // Yakuhai
    const allMentsu = [...pat.mentsuList, ...player.fuuro];
    let yakuhaiHan = 0;
    for (const m of allMentsu) {
      if (
        m.type === MentsuType.Koutsu ||
        m.type === MentsuType.Minkan ||
        m.type === MentsuType.Ankan
      ) {
        const n = m.paiList[0].paiNum;
        if (n >= 31) {
          yaku.push("役牌");
          yakuhaiHan++;
        } else if (n === 27 + gameState.bakaze) {
          yaku.push("場風");
          yakuhaiHan++;
        } else if (n === 27 + player.wind) {
          yaku.push("自風");
          yakuhaiHan++;
        }
      }
    }
    han += yakuhaiHan;

    // Pinfu (Menzen Only)
    let isPinfu = false;
    if (isMenzen) {
      let isPinfuHead = true;
      if (pat.head) {
        const h = pat.head.paiNum;
        if (h >= 31) isPinfuHead = false;
        if (h === 27 + gameState.bakaze) isPinfuHead = false;
        if (h === 27 + player.wind) isPinfuHead = false;
      }
      const isAllShuntsu = pat.mentsuList.every((m) => m.type === MentsuType.Shuntsu);
      let isRyamen = false;
      if (isAllShuntsu && isPinfuHead) {
        for (const m of pat.mentsuList) {
          if (m.type === MentsuType.Shuntsu) {
            const start = m.paiList[0].paiNum;
            if (agariPai.paiNum === start && start % 9 < 6) isRyamen = true;
            else if (agariPai.paiNum === start + 2 && start % 9 > 0) isRyamen = true;
            if (isRyamen) break;
          }
        }
      }
      if (isAllShuntsu && isPinfuHead && isRyamen) {
        isPinfu = true;
        yaku.push("平和");
        han++;
      }
    }

    if (player.isHaitei) {
      yaku.push(player.isTsumo ? "海底摸月" : "河底撈魚");
      han++;
    }
    if (player.isRinshan) {
      yaku.push("嶺上開花");
      han++;
    }
    if (player.isChankan) {
      yaku.push("槍槓");
      han++;
    }

    // --- 2 Han ---
    if (isSanshoku(allMentsu)) {
      yaku.push("三色同順");
      han += isMenzen ? 2 : 1;
    }
    if (isItsu(allMentsu)) {
      yaku.push("一気通貫");
      han += isMenzen ? 2 : 1;
    }
    if (isToitoi(pat.mentsuList, player.fuuro)) {
      yaku.push("対々和");
      han += 2;
    }
    if (isSanankou(pat.mentsuList, player.fuuro, player.isTsumo, agariPai)) {
      yaku.push("三暗刻");
      han += 2;
    }
    if (isSankantsu(player.fuuro)) {
      yaku.push("三槓子");
      han += 2;
    }
    {
      let dragonKoutsu = 0;
      let dragonHead = false;
      for (const m of allMentsu) {
        if (
          (m.type === MentsuType.Koutsu ||
            m.type === MentsuType.Minkan ||
            m.type === MentsuType.Ankan) &&
          m.paiList[0].paiNum >= 31
        ) {
          dragonKoutsu++;
        }
      }
      if (pat.head && pat.head.paiNum >= 31) dragonHead = true;

      if (dragonKoutsu === 2 && dragonHead) {
        yaku.push("小三元");
        han += 2;
      }
    }

    // Chanta / Junchan / Honroutou
    if (pat.head) {
      if (checkTerminalBlocks(allMentsu, pat.head)) {
        const { isAllYaochu } = getTerminalsAndHonorsState(allPai);
        if (isAllYaochu) {
          yaku.push("混老頭");
          han += 2;
        } else {
          const hasZi = allPai.some((p) => p.paiNum >= 27);
          if (hasZi) {
            yaku.push("混全帯幺九");
            han += isMenzen ? 2 : 1;
          } else {
            yaku.push("純全帯幺九");
            han += isMenzen ? 3 : 2;
          }
        }
      }
    }

    // --- 3 Han ---
    if (isMenzen) {
      const shuntsu = pat.mentsuList.filter((m) => m.type === MentsuType.Shuntsu);
      const counts: Record<number, number> = {};
      for (const s of shuntsu) {
        const start = s.paiList[0].paiNum;
        counts[start] = (counts[start] || 0) + 1;
      }
      let pairCount = 0;
      for (const k in counts) {
        if (counts[k] >= 2) pairCount++;
        if (counts[k] >= 4) pairCount++;
      }

      if (pairCount === 2) {
        yaku.push("二盃口");
        han += 3;
      } else if (pairCount === 1) {
        yaku.push("一盃口");
        han += 1;
      }
    }

    // Honitsu / Chinitsu
    const { isHonitsu, isChinitsu } = checkColors(allPai);
    if (isChinitsu) {
      yaku.push("清一色");
      han += isMenzen ? 6 : 5;
    } else if (isHonitsu) {
      yaku.push("混一色");
      han += isMenzen ? 3 : 2;
    }

    han += doraHan;
    yaku.push(...doraYaku);

    if (isPinfu && player.isTsumo) {
      fu = 20;
    } else {
      if (pat.head) {
        fu = calculateFu(
          allMentsu,
          pat.head,
          player.isTsumo,
          isMenzen,
          gameState.bakaze,
          player.wind,
        );
      } else {
        // Should not happen for normal hands if we have yaku?
        // But maybe no head error? Default to 30?
        fu = 30;
      }
    }

    const score = calculateScore(han, fu, false, player.isTsumo);
    if (score > maxResult.score || (score === maxResult.score && han > maxResult.han)) {
      maxResult = { score, han, fu, yaku, yakuman: [], isYakuman: false };
    }
  }

  return maxResult;
}
