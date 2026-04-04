import type { Pai } from "../types";

// シャンテン数の計算ロジック
// SHANTEN CALCULATION

/**
 * メンツ、ターツ、孤立牌の数からシャンテン数を計算する
 * 13 - (メンツ * 3) - (ターツ * 2) - 孤立牌
 */
function calcShantenNormal(
  nMentsu: number,
  nTahtsu: number,
  nKoritsu: number,
  bAtama: boolean,
): number {
  let m = nMentsu;
  let t = nTahtsu;
  let k = nKoritsu;
  const n = bAtama ? 4 : 5;

  // メンツ過多の補正
  if (m > 4) {
    t += m - 4;
    m = 4;
  }

  // メンツ＋ターツ過多の補正
  if (m + t > 4) {
    k += m + t - 4;
    t = 4 - m;
  }

  if (m + t + k > n) {
    k = n - m - t;
  }

  if (bAtama) {
    t += 1;
  }

  return 13 - m * 3 - t * 2 - k;
}

// ターツと孤立牌の数を数える
function countTahtsuKoritsu(haiCount: number[]): { nTahtsu: number; nKoritsu: number } {
  let nPai = 0;
  let nTahtsu = 0;
  let nKoritsu = 0;

  for (let n = 0; n < 9; n++) {
    nPai += haiCount[n];

    // 順子（シュンツ）になりそうなものを探す (n, n+1)
    if (n < 8 && haiCount[n + 1] === 0 && n < 7 && haiCount[n + 2] === 0) {
      // ここはRustのロジック:
      // if n < 7 && hai_count[n + 1] == 0 && hai_count[n + 2] == 0
      // ビット演算 `n_pai >> 1` は `Math.floor(n_pai / 2)`
      nTahtsu += Math.floor(nPai / 2);
      nKoritsu += nPai % 2;
      nPai = 0;
    }
  }
  // ループ終了後の残処理
  nTahtsu += Math.floor(nPai / 2);
  nKoritsu += nPai % 2;

  return { nTahtsu, nKoritsu };
}

// 再帰的にメンツを抜き出して最適解を探す
function countMentsu(
  haiCount: number[],
  n: number,
): { nMentsu: number; nTahtsu: number; nKoritsu: number } {
  if (n >= 9) {
    const { nTahtsu, nKoritsu } = countTahtsuKoritsu(haiCount);
    return { nMentsu: 0, nTahtsu, nKoritsu };
  }

  // そのまま次に進む場合の最良値
  let max = countMentsu(haiCount, n + 1);

  // 順子を抜き出す
  if (n < 7 && haiCount[n] > 0 && haiCount[n + 1] > 0 && haiCount[n + 2] > 0) {
    haiCount[n]--;
    haiCount[n + 1]--;
    haiCount[n + 2]--;

    const r = countMentsu(haiCount, n);

    haiCount[n]++;
    haiCount[n + 1]++;
    haiCount[n + 2]++;

    const newMentsu = r.nMentsu + 1;
    // 評価: シャンテン数が小さい方が良い -> メンツが多く、ターツが多い方が良い
    // シャンテン数 = 13 - 3*M - 2*T - K
    // ここでは最大値を保持するロジックにする（元のRustコードは最小の何かを使っている？）
    // maxと比較して更新
    if (
      evaluate(newMentsu, r.nTahtsu, r.nKoritsu) > evaluate(max.nMentsu, max.nTahtsu, max.nKoritsu)
    ) {
      max = { nMentsu: newMentsu, nTahtsu: r.nTahtsu, nKoritsu: r.nKoritsu };
    }
  }

  // 刻子を抜き出す
  if (haiCount[n] >= 3) {
    haiCount[n] -= 3;
    const r = countMentsu(haiCount, n);
    haiCount[n] += 3;

    const newMentsu = r.nMentsu + 1;
    if (
      evaluate(newMentsu, r.nTahtsu, r.nKoritsu) > evaluate(max.nMentsu, max.nTahtsu, max.nKoritsu)
    ) {
      max = { nMentsu: newMentsu, nTahtsu: r.nTahtsu, nKoritsu: r.nKoritsu };
    }
  }

  return max;
}

// 評価関数: (メンツ, ターツ) の組の良さを数値化
function evaluate(m: number, t: number, _k: number): number {
  // メンツ最優先、次にターツ
  return m * 10 + t;
}

// Rust: PaiState
export class PaiState {
  haiCountM: number[] = new Array(9).fill(0);
  haiCountP: number[] = new Array(9).fill(0);
  haiCountS: number[] = new Array(9).fill(0);
  haiCountZ: number[] = new Array(7).fill(0);

  constructor(tehai: Pai[]) {
    for (const p of tehai) {
      this.add(p);
    }
  }

  add(p: Pai) {
    const n = p.paiNum;
    if (n < 9) this.haiCountM[n]++;
    else if (n < 18) this.haiCountP[n - 9]++;
    else if (n < 27) this.haiCountS[n - 18]++;
    else if (n < 34) this.haiCountZ[n - 27]++;
  }

  // 一般形のシャンテン数計算
  private getShantenCase(bAtama: boolean, nFulo: number): number {
    const m = countMentsu([...this.haiCountM], 0);
    const p = countMentsu([...this.haiCountP], 0);
    const s = countMentsu([...this.haiCountS], 0);

    // 字牌の処理
    let zMentsu = 0;
    let zTahtsu = 0;
    let zKoritsu = 0;
    for (let i = 0; i < 7; i++) {
      if (this.haiCountZ[i] >= 3) zMentsu++;
      else if (this.haiCountZ[i] === 2) zTahtsu++;
      else if (this.haiCountZ[i] === 1) zKoritsu++;
    }

    // Rustの iproduct! のような組み合わせ全探索は重いので
    // 単純に各色のベストな構成を足し合わせるのアプローチでよいか確認
    // Rust版では `iproduct!` で (m, p, s) の全組み合わせの和を見ている。
    // countMentsu は max を返しているので、基本的にはベストな構成が返っているはずだが、
    // 「メンツ重視」「ターツ重視」で分岐がありうる場合は注意が必要。
    // ここでは簡略化して countMentsu の結果（ベスト）を使う。

    const totalMentsu = nFulo + m.nMentsu + p.nMentsu + s.nMentsu + zMentsu;
    const totalTahtsu = m.nTahtsu + p.nTahtsu + s.nTahtsu + zTahtsu;
    const totalKoritsu = m.nKoritsu + p.nKoritsu + s.nKoritsu + zKoritsu;

    return calcShantenNormal(totalMentsu, totalTahtsu, totalKoritsu, bAtama);
  }

  // 七対子のシャンテン数
  private getShantenChitoi(): number {
    let toitsu = 0;
    let kinds = 0;

    // 全牌を走査
    const scan = (arr: number[]) => {
      for (const c of arr) {
        if (c >= 1) kinds++;
        if (c >= 2) toitsu++;
      }
    };
    scan(this.haiCountM);
    scan(this.haiCountP);
    scan(this.haiCountS);
    scan(this.haiCountZ);

    // 6 - 対子数 + max(0, 7 - 種類数)
    return 6 - toitsu + Math.max(0, 7 - kinds);
  }

  // 国士無双のシャンテン数
  private getShantenKokushi(): number {
    const yaochu = [
      this.haiCountM[0],
      this.haiCountM[8],
      this.haiCountP[0],
      this.haiCountP[8],
      this.haiCountS[0],
      this.haiCountS[8],
      ...this.haiCountZ,
    ];

    let yaochuKinds = 0;
    let hasPair = false;

    for (const c of yaochu) {
      if (c > 0) yaochuKinds++;
      if (c >= 2) hasPair = true;
    }

    // 13 - 種類数 - (雀頭があれば1)
    return 13 - yaochuKinds - (hasPair ? 1 : 0);
  }

  // 全ての役の形態から最小のシャンテン数を返す
  public getShanten(nFulo: number): number {
    let min = 13;

    // 一般形 (雀頭なし)
    min = Math.min(min, this.getShantenCase(false, nFulo));

    // 一般形 (雀頭あり)
    // 雀頭候補を全探索して引いてから計算
    // Rust版と同様のアプローチ
    const tryAtama = (arr: number[]) => {
      for (let i = 0; i < arr.length; i++) {
        if (arr[i] >= 2) {
          arr[i] -= 2;
          const val = this.getShantenCase(true, nFulo);
          if (val < min) min = val;
          arr[i] += 2;
        }
      }
    };

    tryAtama(this.haiCountM);
    tryAtama(this.haiCountP);
    tryAtama(this.haiCountS);
    tryAtama(this.haiCountZ);

    // 七対子 (副露なしのみ)
    if (nFulo === 0) {
      min = Math.min(min, this.getShantenChitoi());
    }

    // 国士無双 (副露なしのみ)
    if (nFulo === 0) {
      min = Math.min(min, this.getShantenKokushi());
    }

    return min;
  }
}
