import { type GameState, type Pai, type Player, MentsuType } from "../../mahjong/types";
import { checkYaku } from "../../mahjong/utils/agari";
import { PaiState } from "../../mahjong/utils/shanten";

// ゲームの進行ロジック (game_process.rs の移植)
// 一人麻雀 / ソリティアモード用

export class GameProcess {
  state: GameState;

  constructor() {
    this.state = this.createEmptyState();
  }

  private createEmptyState(
    bakaze = 0,
    kyoku = 1,
    honba = 0,
    kyoutaku = 0,
    oya = 0
  ): GameState {
    return {
      players: [],
      yama: [],
      dora: [],
      uraDora: [],
      currentTurn: 0, // Starts with Oya (0) usually, gets overwritten in setup
      turnCount: 0,
      bakaze,
      kyoku,
      honba,
      kyoutaku,
      oya,
      isGameOver: false,
      resultMessage: null,
    };
  }

  // ゲーム開始 (初期化、洗牌、配牌) -> Full Reset
  startGame() {
    // 25000 * 4
    this.setupGame([25000, 25000, 25000, 25000], 0, 1, 0, 0, 0);
  }

  // 次の局へ
  nextHand() {
    if (!this.state.lastHandResult) {
      // Should not happen, but fallback
      this.startGame();
      return;
    }

    const { type, winner, tenpai } = this.state.lastHandResult;
    let { bakaze, kyoku, honba, kyoutaku, oya } = this.state;
    let renchan = false;

    if (type === 'Agari') {
      // If Winner is Oya, Renchan
      if (winner === oya) {
        renchan = true;
        honba++;
      } else {
        renchan = false;
        honba = 0;
        const prevOya = oya;
        oya = (oya + 1) % 4;
        // Detect Round Change (South)
        if (oya < prevOya) { // Wrapped 3->0
          bakaze++;
        }
        kyoku = oya + 1;
      }
    } else {
      // Ryukyoku
      // If Oya is Tenpai OR (Rules: some rules Oya listens = Renchan)
      // Standard: Oya Tenpai = Renchan. Noten = Nagare.
      const oyaTenpai = tenpai && tenpai[oya];
      if (oyaTenpai) {
        renchan = true;
        honba++;
      } else {
        renchan = false;
        honba++;
        const prevOya = oya;
        oya = (oya + 1) % 4;
        if (oya < prevOya) {
          bakaze++;
        }
        kyoku = oya + 1;
      }
    }

    // Check Game End
    // End if Bakaze > 1 (West Round reached)
    // Or other conditions (Tobi? not implemented yet)
    if (bakaze > 1) {
      this.finishGame();
      return;
    }

    // South 4 Agari Yame check?
    // If Bakaze=1 (South), Kyoku=4 (South 4).
    // If renchan (Oya won/tenpai) AND Oya is Top -> End.
    if (bakaze === 1 && kyoku === 4 && renchan) {
      // Check scores
      const scores = this.state.players.map((p, i) => ({ score: p.score, index: i }));
      scores.sort((a, b) => b.score - a.score);
      if (scores[0].index === oya) {
        this.finishGame();
        return;
      }
    }

    const currentScores = this.state.players.map((p) => p.score);
    this.setupGame(
      currentScores,
      bakaze,
      kyoku,
      honba,
      kyoutaku,
      oya
    );
  }

  private finishGame() {
    // 1. Calculate Oka
    // Origin 30000, Start 25000. Oka = (30000 - 25000) * 4 = 20000.
    // Top player gets +20000.
    const scores = this.state.players.map((p, i) => ({ score: p.score, index: i }));
    // Sort: Descending. Tie-breaker: initial seat order? (0-3).
    // Standard: East(Start) > South > ...
    // In current logic, Index 0 is Start East?
    // setupGame winds are relative.
    // Let's assume Index 0 is the "User" and initial priority.
    // Using stable sort or explicit index check.
    scores.sort((a, b) => {
      if (b.score !== a.score) return b.score - a.score;
      return a.index - b.index; // Tie-break by index (Seat order)
    });

    // Apply Oka to Top
    this.state.players[scores[0].index].score += 20000;
    scores[0].score += 20000;

    // Format Result
    let msg = "Game Over\n\nRanking:\n";
    scores.forEach((s, rank) => {
      msg += `${rank + 1}位: Player ${s.index} (${s.score})\n`;
    });

    this.state.isGameOver = true;
    this.state.resultMessage = msg;
    // Don't setup new game.
  }

  private setupGame(
    scores: number[],
    bakaze: number,
    kyoku: number,
    honba: number,
    kyoutaku: number,
    oya: number
  ) {
    this.state = this.createEmptyState(bakaze, kyoku, honba, kyoutaku, oya);
    this.state.yama = this.createShuffledYama();
    this.state.currentTurn = oya; // Oya starts

    // プレイヤー作成 (4人)
    for (let i = 0; i < 4; i++) {
      // Wind: (Seat - Oya + 4) % 4 ? No, standard mapping:
      // Index 0 = East (if Oya=0), 1=South.. relative to Oya?
      // Usually fixed index 0..3, and Oya rotates.
      // Let's say Index 0 is ALWAYS the User for UI convenience?
      // Or Index is strictly seat (Ton, Nan, Sha, Pei).
      // Let's assume Index 0 = Ton (Initial), 1 = Nan...
      // Player's Jikaze depends on Oya.
      // Jikaze = (PlayerIndex - Oya + 4) % 4. (0=Ton, 1=Nan...)
      const wind = (i - oya + 4) % 4; // This might need verification
      this.state.players.push(this.createPlayer(wind, scores[i] ?? 25000));
    }

    // 配牌 (Deal 13 tiles to each player)
    // Detailed dealing (4 tiles at a time) is skippable for now, just loop
    for (let pIdx = 0; pIdx < 4; pIdx++) {
      for (let i = 0; i < 13; i++) {
        const p = this.drawFromYama();
        if (p) this.state.players[pIdx].tehai.push(p);
      }
      this.sortTehai(pIdx);
    }

    // ドラ表示牌
    const doraInd = this.drawFromYama();
    if (doraInd) this.state.dora.push(doraInd);

    // 裏ドラ (確保のみ、ゲーム終了時に表示)
    const uraInd = this.drawFromYama();
    if (uraInd) this.state.uraDora.push(uraInd);

    // 最初のツモ (Current Turn Dealer)
    this.tsumo();
  }

  // Alias for compatibility if needed, currently unused
  initGame() {
    this.startGame();
  }

  // 山の生成とシャッフル
  private createShuffledYama(): Pai[] {
    const yama: Pai[] = [];

    // 全種類の牌を4枚ずつ生成
    // 数牌 0-26 (9x3), 字牌 27-33 (7)
    for (let i = 0; i < 34; i++) {
      for (let j = 0; j < 4; j++) {
        yama.push({
          paiNum: i,
          id: j,
          isTsumogiri: false,
          isRiichi: false,
          isNakare: false,
        });
      }
    }

    // Fisher-Yates Shuffle
    for (let i = yama.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [yama[i], yama[j]] = [yama[j], yama[i]];
    }

    return yama;
  }

  private createPlayer(wind: number, score: number): Player {
    return {
      score,
      tehai: [],
      tsumohai: null,
      kawahai: [],
      isRiichi: false,
      isDoubleRiichi: false,
      isIppatsu: false,
      isTsumo: false,
      fuuro: [],
      wind,
      shanten: 8,
    };
  }

  private drawFromYama(): Pai | undefined {
    return this.state.yama.pop();
  }

  // ツモ
  tsumo() {
    if (this.state.isGameOver) return;

    // Remove 18 turns limit, check Yama length only (Dead wall is kept in yama? usually yama has 14 dead tiles excluded or included?
    // In this simplified logic, let's assume yama includes only drawable tiles OR we check a count.
    // The previous implementation had 136 tiles and popped.
    // Standard game: 136 - 14 = 122 drawable.
    // Let's assume we stop when yama.length <= 14 (Dead wall simulation)
    if (this.state.yama.length <= 14) {
      this.ryukyoku("山切れ (流局)");
      return;
    }

    const p = this.drawFromYama();
    if (!p) {
      // Should catch above, but safety
      this.ryukyoku("山切れ (流局)");
      return;
    }

    const player = this.state.players[this.state.currentTurn];
    player.tsumohai = p;
    player.isTsumo = true;

    // シャンテン数更新
    player.shanten = this.getShanten(); // 自分の手牌+ツモで計算

    // AI Turn Handling
    if (this.state.currentTurn !== 0) {
      // Simple AI Logic
      // Check Tsumo
      const tsumoResult = checkYaku(player, this.state, p);
      if (tsumoResult.han > 0) {
        this.agari();
        return;
      }

      // Check Riichi
      // If Tenpai and !Riichi and Score >= 1000
      if (!player.isRiichi && player.score >= 1000 && player.shanten <= 0) {
        this.riichi();
        // Discard Tsumohai (simple)
        this.dahai(player.tehai.length);
        return;
      }

      // Discard
      // Simple: Discard Tsumohai (Tsumogiri)
      // Or discard unuseful tile?
      // Random discard for variety?
      // Let's just Tsumogiri for flow testing
      this.dahai(player.tehai.length);
    }
  }

  // 打牌
  dahai(index: number) {
    if (this.state.isGameOver) return;

    const player = this.state.players[this.state.currentTurn];
    let p: Pai;

    // ツモ切り
    if (index === player.tehai.length) {
      if (!player.tsumohai) return;
      p = player.tsumohai;
      p.isTsumogiri = true;
      player.tsumohai = null;
    } else {
      // 手出し
      p = player.tehai[index];
      if (player.tsumohai) {
        player.tehai[index] = player.tsumohai;
        player.tsumohai = null;
        this.sortTehai(this.state.currentTurn);
      } else {
        player.tehai.splice(index, 1);
      }
    }

    // 立直中ならツモ切り強制チェックなどはUI側で制御するか、ここで弾く
    if (player.isRiichi && !p.isTsumogiri) {
      // 立直後の手出し禁止（今回は簡易実装）
    }

    if (player.isRiichi) {
      // 既に立直宣言牌が河にあるかチェック
      const alreadyDeclared = player.kawahai.some((k) => k.isRiichi);
      if (!alreadyDeclared) {
        // これが宣言牌
        p.isRiichi = true;
        // 一発は維持する (次巡のツモまで有効)
      } else {
        // 宣言後の打牌 -> 一発消滅
        player.isIppatsu = false;
      }
    } else {
      player.isIppatsu = false;
    }

    player.kawahai.push(p);

    this.state.turnCount++;

    // Check Reactions (Ron / Pon / Chi)
    // If Reaction -> Execute and divert
    // Else
    if (!this.checkReactions(p, this.state.currentTurn)) {
      this.nextTurn();
    }
  }

  // Check reactions from other players
  // Returns true if flow interrupted
  private checkReactions(discard: Pai, discarderIdx: number): boolean {
    // Order: Ron (Priority), then Pon/Kan, then Chi (Left player only)
    // Check Ron in turn order
    for (let i = 1; i <= 3; i++) {
      const idx = (discarderIdx + i) % 4;
      const player = this.state.players[idx];
      // Check Ron
      // Need to clone hand + discard
      const result = checkYaku(player, this.state, discard);
      if (result.han > 0) {
        // AI decides to Ron? Always Yes for now.
        // If User (0), we theoretically should ask.
        // But for this "Game Flow" scope, let's auto-Ron for AI, skip User for manual button?
        // User has "Agari" button in UI.
        // But checkReactions is called synchronously.
        // If User can Ron, we should PAUSE.
        // HANDLING PAUSE IS COMPLEX.
        // Fallback: This logic is for AI-only reactions to keep flow moving.
        // User reactions are handled by UI detecting state change.
        // But if Process auto-advances, User misses chance.
        // Solution: If User can Ron, we DO NOT call nextTurn(). We return true (interrupted).
        // And wait for User action.
        if (idx === 0) {
          // User can Ron.
          // TODO: Set state "WaitingForUser"?
          // For now, let's just NOT auto-advance if User is involved.
          // But if we return true, who resumes?
          // User clicks "Ron" -> agari().
          // User clicks "Skip" -> nextTurn(). (Need Skip button).
          return false; // Let's simplify: Auto-skip User for now unless we add phase.
        }

        // AI Ron
        this.agari(idx, discarderIdx);
        return true;
      }
    }

    // Check Pon (Any player)
    for (let i = 1; i <= 3; i++) {
      const idx = (discarderIdx + i) % 4;
      if (idx === 0) continue; // Skip User
      const player = this.state.players[idx];
      if (player.isRiichi) continue;

      // Check Pair
      const count = player.tehai.filter(p => p.paiNum === discard.paiNum).length;
      if (count >= 2) {
        // Simple AI: 50% chance to Pon
        if (Math.random() > 0.5) {
          this.pon(idx, discarderIdx, discard);
          return true;
        }
      }
    }

    // Check Chi (Right player only)
    // const nextIdx = (discarderIdx + 1) % 4;
    // ... implementation similar ...

    return false;
  }

  pon(who: number, _from: number, pai: Pai) {
    const player = this.state.players[who];
    // Remove 2 matching tiles
    let removed = 0;
    for (let i = player.tehai.length - 1; i >= 0; i--) {
      if (player.tehai[i].paiNum === pai.paiNum && removed < 2) {
        player.tehai.splice(i, 1);
        removed++;
      }
    }
    // Add to Fuuro
    player.fuuro.push({
      type: MentsuType.Koutsu, // Should be Minkan effectively
      paiList: [pai, pai, pai] // Simplified
    });
    // Move turn
    this.state.currentTurn = who;
    // Player needs to discard now. 
    // If AI, auto-discard.
    if (who !== 0) {
      // Discard right-most (simple)
      this.dahai(player.tehai.length - 1);
    }
  }

  // 次のターンへ
  nextTurn() {
    this.state.currentTurn = (this.state.currentTurn + 1) % 4;
    this.tsumo();
  }

  // 理牌 (ソート)
  sortTehai(playerIdx: number) {
    this.state.players[playerIdx].tehai.sort((a, b) => a.paiNum - b.paiNum);
  }

  // 立直
  riichi() {
    const player = this.state.players[this.state.currentTurn];
    if (player.isRiichi) return;

    player.isRiichi = true;
    player.isIppatsu = true;
    player.score -= 1000;
    this.state.kyoutaku++; // Add stick
  }

  // 和了判定 (ツモ / ロン)
  // winnerIndex: defaults to currentTurn (Tsumo)
  // loserIndex: if set, it is Ron from this player
  agari(winnerIndex?: number, loserIndex?: number) {
    const winner = winnerIndex ?? this.state.currentTurn;
    const isTsumo = loserIndex === undefined || loserIndex === null;
    const player = this.state.players[winner];

    // Check Shanten? If handling Ron, we assume check was done or we do it here.
    // For Tsumo, we check Shanten is -1.
    // For Ron, we should check if tile gives -1.
    // Simplifying: Assume valid win condition checked by caller or basic check here.

    let agariPai: Pai | null = null;
    if (isTsumo) {
      agariPai = player.tsumohai;
    } else {
      // Ron: Get last discarded tile from loser
      const loser = this.state.players[loserIndex!];
      if (loser.kawahai.length > 0) {
        agariPai = loser.kawahai[loser.kawahai.length - 1]; // Only reference?
      }
    }

    if (!agariPai) return;

    // TODO: For Ron, we technically need to add agariPai to player's hand pattern for checkYaku
    // checkYaku takes (player, state, agariPai). It adds agariPai to list.
    // So distinct handling not strictly needed for checkYaku, but we need strictly correct AgariPai obj.

    const result = checkYaku(player, this.state, agariPai);

    if (result.han > 0 || result.yakuman.length > 0) {
      // Calculate Score Distribution
      const { diffs, basicScore: _basicScore } = this.calculateScoreDistribution(result, winner, loserIndex ?? null);

      // Apply Scores
      for (let i = 0; i < 4; i++) {
        this.state.players[i].score += diffs[i];
      }

      // Add Kyoutaku to Winner
      this.state.players[winner].score += this.state.kyoutaku * 1000;
      const _acquiredKyoutaku = this.state.kyoutaku;
      this.state.kyoutaku = 0; // Reset after win

      this.state.resultMessage = `和了! ${isTsumo ? "ツモ" : "ロン"}\n${result.yaku.join(", ")}\n${result.score}点`;

      this.state.lastHandResult = {
        type: 'Agari',
        winner,
        loser: loserIndex,
        isTsumo,
        scoreDiffs: diffs,
        yakuList: result.yaku,
        han: result.han,
        fu: result.fu,
        score: result.score
      };

      this.state.isGameOver = true;
      // Note: isGameOver just stops the loop. nextHand() will check if game continues.
    } else {
      this.state.resultMessage = "役なし (チョンボ?)";
    }
  }

  // Calculate score changes (Does NOT apply them, just returns diffs)
  private calculateScoreDistribution(
    result: { han: number; fu: number; yakuman: string[] }, // from checkYaku
    winner: number,
    loser: number | null
  ) {
    const isYakuman = result.yakuman.length > 0;
    const han = result.han;
    const fu = result.fu;
    const isTsumo = loser === null;
    const isWinnerOya = winner === this.state.oya; // Oya index check

    // 1. Calculate Base Point
    let basePoint = 0;
    if (isYakuman) {
      basePoint = 8000 * result.yakuman.length;
    } else {
      if (han >= 13) basePoint = 8000;
      else if (han >= 11) basePoint = 6000;
      else if (han >= 8) basePoint = 4000;
      else if (han >= 6) basePoint = 3000;
      else if (han >= 5) basePoint = 2000;
      else {
        basePoint = fu * Math.pow(2, 2 + han);
        if (basePoint > 2000) basePoint = 2000;
      }
    }

    const diffs = [0, 0, 0, 0];
    const honbaPoints = this.state.honba * 300;

    if (isTsumo) {
      // Tsumo
      if (isWinnerOya) {
        // Dealer Tsumo: All Ko pay 1/3 of total? No, 2*Base each.
        // Total = Base * 6.
        // Each Ko pays ceil(Base * 2 / 100) * 100
        const koPayment = Math.ceil((basePoint * 2) / 100) * 100;
        const totalGain = koPayment * 3 + honbaPoints;

        const honbaPayment = 100 * this.state.honba;

        for (let i = 0; i < 4; i++) {
          if (i === winner) {
            diffs[i] += totalGain; // Gain (from 3 players)
          } else {
            diffs[i] -= (koPayment + honbaPayment);
          }
        }
      } else {
        // Ko Tsumo
        // Dealer pays ceil(Base * 2 / 100) * 100
        // Ko pays ceil(Base * 1 / 100) * 100
        const oyaPayment = Math.ceil((basePoint * 2) / 100) * 100;
        const koPayment = Math.ceil((basePoint * 1) / 100) * 100;
        // Winner valid gain depends on actual rounded sum?
        // Standard: Winner gains sum of payments.
        const honbaPayment = 100 * this.state.honba;

        let totalGain = 0;
        for (let i = 0; i < 4; i++) {
          if (i === winner) continue;
          let pay = 0;
          if (i === this.state.oya) pay = oyaPayment;
          else pay = koPayment;

          pay += honbaPayment;
          diffs[i] -= pay;
          totalGain += pay;
        }
        diffs[winner] += totalGain;
      }
    } else {
      // Ron
      // Loser pays Full Score
      // Full Score = ceil(Base * (isWinnerOya ? 6 : 4) / 100) * 100
      const rate = isWinnerOya ? 6 : 4;
      let ronScore = Math.ceil((basePoint * rate) / 100) * 100;

      // Add Honba (Payer pays all 300 * honba)
      ronScore += honbaPoints;

      diffs[loser!] -= ronScore;
      diffs[winner] += ronScore;
    }

    return { diffs, basicScore: basePoint }; // basicScore not strictly used after this, but nice to have?
  }

  ryukyoku(reason: string) {
    this.state.isGameOver = true;
    this.state.resultMessage = reason;

    // Tenpai Check
    const tenpaiFlags = this.state.players.map(p => {
      // This assumes shanten is up-to-date.
      // Or re-calculate? Better re-calc to be safe.
      // Currently shanten includes tsumohai if exists.
      // Ryuukyoku -> No tsumohai usually (discarded last).
      // So check tehai (13 tiles).
      const shantenCalc = new PaiState(p.tehai);
      const shanten = shantenCalc.getShanten(p.fuuro.length);
      return shanten <= 0; // 0 is Tenpai
    });

    const tenpaiCount = tenpaiFlags.filter(t => t).length;
    const diffs = [0, 0, 0, 0];

    if (tenpaiCount > 0 && tenpaiCount < 4) {
      const pot = 3000;
      const gain = pot / tenpaiCount;
      const loss = pot / (4 - tenpaiCount);

      for (let i = 0; i < 4; i++) {
        if (tenpaiFlags[i]) diffs[i] = gain;
        else diffs[i] = -loss;
      }
    }

    // Apply
    for (let i = 0; i < 4; i++) {
      this.state.players[i].score += diffs[i];
    }

    this.state.lastHandResult = {
      type: 'Ryukyoku',
      tenpai: tenpaiFlags,
      scoreDiffs: diffs
    };
  }

  // シャンテン数取得 (UI表示用)
  getShanten(): number {
    const player = this.state.players[this.state.currentTurn];
    if (!player) return 8; // 初期値

    // 手牌 + ツモ牌を含めて計算
    const all = [...player.tehai];
    if (player.tsumohai) all.push(player.tsumohai);

    const shantenCalc = new PaiState(all);
    return shantenCalc.getShanten(0);
  }

  // 指定した牌を切った後にテンパイ（シャンテン0以下）になるか判定 (立直時の選択制御用)
  checkTenpaiAfterDiscard(index: number): boolean {
    const player = this.state.players[this.state.currentTurn];
    // クローンして操作
    const tehai = [...player.tehai];
    let tsumohai = player.tsumohai;

    // 打牌シミュレーション
    if (index === tehai.length) {
      // ツモ切り
      tsumohai = null;
    } else {
      // 手出し
      if (tsumohai) {
        tehai[index] = tsumohai;
        tsumohai = null;
      } else {
        tehai.splice(index, 1);
      }
    }

    // 13枚の状態でのシャンテン数計算
    const all = [...tehai];
    const shantenCalc = new PaiState(all);
    const shanten = shantenCalc.getShanten(player.fuuro.length);

    return shanten <= 0;
  }
}
