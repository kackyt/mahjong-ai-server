use crate::{
    agari::{add_machi_to_mentsu, Agari, AgariBehavior},
    fbs_utils::TakuControl,
    mahjong_generated::open_mahjong::{
        ActionType, GameStateT, MentsuFlag, MentsuPaiT, MentsuT, MentsuType, PaiT, PlayerT, RuleT,
        TakuT,
    },
    play_log::PlayLog,
    shanten::{all_of_chiitoitsu, all_of_kokushi, all_of_mentsu, PaiState},
};
use anyhow::{bail, ensure};
use chrono::Utc;
use itertools::Itertools;
use rand::seq::SliceRandom;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum PostDrawAction {
    TsumoAgari,
    Ryuukyoku,
    Nothing,
}

#[derive(Error, Debug)]
pub enum GameProcessError {
    #[error("リーチ後はツモ切りのみです")]
    IllegalSutehaiAfterRiichi,
    /// 面前ではない場合のエラー
    #[error("面前ではありません")]
    NotMenzen,
    /// ツモっていない場合のエラー
    #[error("ツモしていません")]
    NotTsumo,
    /// テンパイではない場合のエラー
    #[error("テンパイではありません")]
    NotTenpai,
    /// ECS世界（MahjongWorld）におけるエラー
    #[error("World error: {0}")]
    WorldError(#[from] crate::components::world::WorldError),
    /// 各システム（ツモ・打牌等）におけるエラー
    #[error("System error: {0}")]
    SystemError(#[from] crate::systems::types::SystemError),
    /// その他のエラー（anyhow等）
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

const DORA_START_INDEX: usize = 0;
const URADORA_START_INDEX: usize = 5;
const START_OF_YAMA: [u32; 4] = [14, 45, 75, 105];

impl RuleT {
    pub fn update_to_default(&mut self) {
        self.enable_kuitan = true;
        self.enable_kansaki = false;
        self.enable_pao = false;
        self.initial_score = 25000;
        self.enable_tobi = true;
        self.enable_wareme = false;
        self.aka_type = 0;
        self.shanyu_score = 0;
        self.nannyu_score = -1;
        self.enable_kuinaoshi = true;
        self.uradora_type = 2;
        self.enable_minus_riichi = true;
        self.enable_ryanhan_shibari = false;
        self.enable_keiten = true;
        self.oyanagare_type = 0x0f;
        self.kan_in_riichi = 1;
        self.enable_kiriage = false;
    }
}

impl GameStateT {
    /// ゲームの状態を初期化します。
    ///
    /// # Arguments
    /// * `title` - ゲームのタイトル
    /// * `player_len` - プレイヤー人数（通常4人、または1人など）
    /// * `play_log` - ログ記録用オブジェクト
    pub fn create(&mut self, title: &[u8], player_len: u32, play_log: &mut PlayLog) {
        self.player_len = player_len;
        self.rule.update_to_default();
        self.title = title.into();
        let uuid = Uuid::new_v4();
        self.game_id = uuid.into_bytes();
        let dt = Utc::now();

        for idx in 0..self.player_len {
            let player = &mut self.players[idx as usize];
            player.score = self.rule.initial_score as i32;
        }

        play_log.append_game_log(uuid.hyphenated().to_string(), dt.timestamp() as u64);
    }

    /// プレイヤーをゲームに登録します。
    ///
    /// # Arguments
    /// * `name` - プレイヤー名
    /// * `play_log` - ログ記録用オブジェクト
    ///
    /// # Returns
    /// 登録されたプレイヤーのインデックスを返します。満員の場合はエラーになります。
    pub fn register_player(
        &mut self,
        name: &[u8],
        play_log: &mut PlayLog,
    ) -> anyhow::Result<usize> {
        // registered == falseなplayerのindexのリストを作る
        let unregistered_index = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, x)| !x.is_registered())
            .map(|(i, _)| i)
            .collect_vec();
        ensure!(!unregistered_index.is_empty(), "player is full");

        // unregistered_indexからランダムに選ぶ
        let mut rng = rand::thread_rng();
        let chosen_index = unregistered_index.choose(&mut rng);

        // chosen_indexがNoneならば、エラー
        ensure!(chosen_index.is_some(), "player index choose error");

        let index = chosen_index.unwrap();
        let uuid = Uuid::from_bytes_ref(&self.game_id);

        self.players[*index].name = name.into();

        play_log.append_game_player_log(
            uuid.hyphenated().to_string(),
            String::from_utf8(name.to_vec())?,
            *index as i32,
        );

        Ok(*index)
    }

    pub fn are_players_all_registered(&self) -> bool {
        self.players[..self.player_len as usize]
            .iter()
            .all(|x| x.is_registered())
    }

    pub fn shuffle(&mut self) {
        self.taku = TakuT::create_shuffled()
    }

    pub fn load(&mut self, hai_ids: &[u32]) {
        self.taku = TakuT::load(hai_ids);
    }

    pub fn next_cursol(&mut self) {
        if self.is_non_duplicate {
            self.taku_cursol += 1;
        } else {
            self.players[self.teban as usize].cursol += 1;
        }
    }

    pub fn get_zikaze(&self, who: usize) -> u32 {
        let diff = (who as i32) - (self.oya as i32);

        if diff < 0 {
            (diff + self.player_len as i32) as u32
        } else {
            diff as u32
        }
    }

    pub fn remain(&self) -> u32 {
        if self.is_non_duplicate {
            136 - self.taku_cursol
        } else {
            136 - 14
                - self.players[0..self.player_len as usize]
                    .iter()
                    .enumerate()
                    .map(|(idx, x)| x.cursol - START_OF_YAMA[idx])
                    .sum::<u32>()
        }
    }

    /// 局を開始します。配牌やドラの決定、各種状態のリセットを行います。
    pub fn start(&mut self, play_log: &mut PlayLog) {
        // 配牌
        self.taku_cursol = 14;
        self.dora_len = 1;
        self.uradora_len = 1;
        self.seq = 0;
        let dt = Utc::now();
        self.kyoku_id = (dt.timestamp() / (24 * 3600) * 100000) as u64;
        let mut kazes = [Some(0), Some(0), Some(0), Some(0)];
        self.teban = self.oya;

        for idx in 0..self.player_len {
            kazes[idx as usize] = Some(self.get_zikaze(idx as usize) as i32);
        }

        let uuid = Uuid::from_bytes_ref(&self.game_id);

        play_log.append_kyoku_log(
            self.kyoku_id,
            uuid.hyphenated().to_string(),
            0,
            self.tsumobou as i32,
            self.riichibou as i32,
            &self
                .players
                .iter()
                .map(|p| Some(p.score))
                .collect::<Vec<Option<i32>>>(),
            &kazes,
        );

        for idx in 0..self.player_len {
            let player = &mut self.players[idx as usize];
            player.cursol = 14 + (idx * if idx < 2 { 31 } else { 30 });
            player.kawahai_len = 0;
            player.is_ippatsu = false;
            player.is_riichi = false;
            let cursol: &mut u32 = if self.is_non_duplicate {
                &mut self.taku_cursol
            } else {
                &mut player.cursol
            };
            let r = self
                .taku
                .get_range((*cursol as usize)..(*cursol + 13) as usize);

            if let Ok(mut v) = r {
                v.sort_unstable();
                player.tehai.clone_from_slice(&v);
                player.tehai_len = 13;
            }

            play_log.append_haipais_log(
                self.kyoku_id,
                idx as i32,
                &player.tehai[..player.tehai_len as usize]
                    .iter()
                    .map(|x| Some(x.get_pai_id()))
                    .collect::<Vec<Option<u32>>>(),
            );

            *cursol += 13;
        }
    }

    pub fn get_player(&self, index: usize) -> PlayerT {
        self.players[index].clone()
    }

    /// 現在の有効な山カーソルを取得します（重複山モードに対応）。
    pub fn get_taku_cursor(&self) -> u32 {
        if self.is_non_duplicate {
            self.taku_cursol
        } else {
            self.players[self.teban as usize].cursol
        }
    }

    /// 現在の手番プレイヤーがツモ和了の形（4面子1雀頭など）になっているか判定します。
    /// 役の有無は考慮しません。
    pub fn is_tsumo_agari_form(&self) -> bool {
        let player = &self.players[self.teban as usize];
        if !player.is_tsumo {
            return false;
        }
        let mut tehai: Vec<PaiT> = player.tehai[..player.tehai_len as usize].to_vec();
        tehai.push(player.tsumohai.clone());
        let shanten = PaiState::from(&tehai).get_shanten(player.mentsu_len as usize);
        shanten == -1
    }

    /// 現在のプレイヤーがツモ和了した場合のスコアを計算します。
    /// 和了していない場合や役がない場合は 0 を返します。
    pub fn evaluate_tsumo_agari_score(&self) -> i32 {
        let player = &self.players[self.teban as usize];
        if !player.is_tsumo {
            return 0;
        }
        let mut tehai: Vec<PaiT> = player.tehai[..player.tehai_len as usize].to_vec();
        tehai.push(player.tsumohai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let mut all_mentsu = all_of_mentsu(&mut state, fulo.len());
        if fulo.is_empty() {
            all_mentsu.extend(all_of_chiitoitsu(&state));
            all_mentsu.extend(all_of_kokushi(&state));
        }
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &player.tsumohai.pack());

        if all_mentsu_w_machi.is_empty() {
            return 0;
        }

        let best_agari =
            self.get_best_agari(self.teban as usize, &all_mentsu_w_machi, &fulo, 0, false);
        match best_agari {
            Ok(agari) => agari.score,
            Err(_) => 0,
        }
    }

    /// ツモ直後の状態を評価し、和了、流局、または継続を判定します。
    pub fn evaluate_post_draw_status(&self) -> PostDrawAction {
        // 1. 和了判定 (形 + 役)
        if self.evaluate_tsumo_agari_score() > 0 {
            return PostDrawAction::TsumoAgari;
        }

        // 2. 流局判定
        if self.get_taku_cursor() >= self.taku.length {
            return PostDrawAction::Ryuukyoku;
        }

        PostDrawAction::Nothing
    }

    /// ツモを行います。
    #[cfg(feature = "ecs")]
    pub fn tsumo(&mut self, play_log: &mut PlayLog) -> Result<(), GameProcessError> {
        let mut world = crate::components::world::MahjongWorld::from_game_state(self);

        let teban = self.teban as usize;
        let taku_cursol = if self.is_non_duplicate {
            self.taku_cursol as usize
        } else {
            self.players[teban].cursol as usize
        };
        let tsumohai = self.taku.get(taku_cursol)?;

        let tsumo_input = crate::systems::tsumo::TsumoInput {
            teban,
            seq: crate::components::SeqCount(self.seq),
            kyoku_id: crate::components::KyokuId(self.kyoku_id),
            is_non_duplicate: self.is_non_duplicate,
            taku_cursol: crate::components::TakuCursolPos(taku_cursol as u32),
            tsumohai: tsumohai.clone(),
        };

        let entity = world
            .query_player(teban)
            .ok_or(crate::components::world::WorldError::PlayerNotFound(teban))?;

        let event = {
            // hecsのBorrow制約のため、特定のスコープでクエリを実行する
            let mut q = world
                .world
                .query_one::<(&mut crate::components::Hand, &mut crate::components::Cursol)>(entity)
                .map_err(crate::components::world::WorldError::EntityError)?;
            let (hand, cursol) = q
                .get()
                .ok_or(crate::components::world::WorldError::ComponentsNotFound)?;

            let view = crate::systems::tsumo::TsumoView { hand, cursol };
            crate::systems::tsumo::run_tsumo(view, &tsumo_input)?
        };

        play_log.append_actions_log(
            event.kyoku_id.0,
            event.teban as i32,
            event.seq.0 as i32,
            String::from("tsumo"),
            event.tsumohai.get_pai_id(),
        );

        world.context.seq.0 += 1;

        world.to_game_state(self);
        self.next_cursol();
        Ok(())
    }

    /// 牌を捨てます。立直判定や一発の解除、河への追加を行います。
    #[cfg(feature = "ecs")]
    pub fn sutehai(
        &mut self,
        play_log: &mut PlayLog,
        index: usize,
        is_riichi: bool,
    ) -> Result<PaiT, GameProcessError> {
        // バリデーション（非ECSのレガシー構造を参照）
        if is_riichi {
            let player = &self.players[self.teban as usize];
            if player.mentsu_len != 0 {
                return Err(GameProcessError::NotMenzen);
            }
            if !player.is_tsumo {
                return Err(GameProcessError::NotTsumo);
            }

            // 立直が可能かチェック（シャンテン数）
            let mut tehai_check: Vec<PaiT> = player
                .tehai
                .iter()
                .take(player.tehai_len as usize)
                .cloned()
                .collect();
            if index < player.tehai_len as usize {
                tehai_check.remove(index);
                tehai_check.push(player.tsumohai.clone());
                tehai_check.sort_unstable();
            }
            let mut state = PaiState::from(&tehai_check);
            let shanten = state.get_shanten(player.mentsu_len as usize);
            if shanten != 0 {
                return Err(GameProcessError::NotTenpai);
            }
        }

        let mut world = crate::components::world::MahjongWorld::from_game_state(self);

        let teban = self.teban as usize;
        let sutehai_input = crate::systems::sutehai::SutehaiInput {
            kyoku_id: crate::components::KyokuId(self.kyoku_id),
            teban,
            seq: crate::components::SeqCount(self.seq),
            index,
            is_riichi,
        };

        let entity = world
            .query_player(teban)
            .ok_or(crate::components::world::WorldError::PlayerNotFound(teban))?;

        let event = {
            let mut q = world
                .world
                .query_one::<(
                    &mut crate::components::Hand,
                    &mut crate::components::DiscardPile,
                    &mut crate::components::RiichiStatus,
                )>(entity)
                .map_err(crate::components::world::WorldError::EntityError)?;
            let (hand, discard_pile, riichi_status) = q
                .get()
                .ok_or(crate::components::world::WorldError::ComponentsNotFound)?;

            let view = crate::systems::sutehai::SutehaiView {
                hand,
                discard_pile,
                riichi_status,
            };
            crate::systems::sutehai::run_sutehai(view, &sutehai_input)?
        };

        play_log.append_actions_log(
            event.kyoku_id.0,
            event.teban as i32,
            event.seq.0 as i32,
            String::from("sutehai"),
            event.kawahai.get_pai_id(),
        );

        world.context.seq.0 += 1;
        world.context.teban = (world.context.teban + 1) % world.players.len() as u32;

        world.to_game_state(self);

        Ok(event.kawahai)
    }

    /// ツモ和了の処理を行います。点数計算、スコア移動を適用し、結果を返します。
    pub fn tsumo_agari(&mut self, play_log: &mut PlayLog) -> anyhow::Result<Agari> {
        let player = &self.players[self.teban as usize];
        let mut tehai: Vec<PaiT> = player.tehai.to_vec();
        let machipai = player.tsumohai.clone();

        tehai.push(machipai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let mut all_mentsu = all_of_mentsu(&mut state, fulo.len());
        if fulo.is_empty() {
            all_mentsu.extend(all_of_chiitoitsu(&state));
            all_mentsu.extend(all_of_kokushi(&state));
        }
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &player.tsumohai.pack());

        ensure!(!all_mentsu_w_machi.is_empty(), "和了ではありません");

        let mut best_agari =
            self.get_best_agari(self.teban as usize, &all_mentsu_w_machi, &fulo, 0, false)?;

        let is_oya = self.teban == self.oya;
        if is_oya {
            best_agari.score = ((best_agari.score as f32 * 1.5).ceil() as i32 + 99) / 100 * 100;
        }

        let mut scores = [0; 4];
        let mut score_diffs = [Some(0); 4];

        if is_oya {
            let payment = ((best_agari.score as f32 / 3.0).ceil() as i32 + 99) / 100 * 100;
            for i in 0..self.player_len as usize {
                if i == self.teban as usize {
                    scores[i] = best_agari.score
                        + self.riichibou as i32 * 1000
                        + self.tsumobou as i32 * 300;
                    score_diffs[i] = Some(scores[i]);
                } else {
                    scores[i] = -(payment + self.tsumobou as i32 * 100);
                    score_diffs[i] = Some(scores[i]);
                }
            }
        } else {
            let oya_payment = ((best_agari.score as f32 / 2.0).ceil() as i32 + 99) / 100 * 100;
            let ko_payment = ((best_agari.score as f32 / 4.0).ceil() as i32 + 99) / 100 * 100;
            for i in 0..self.player_len as usize {
                if i == self.teban as usize {
                    scores[i] = best_agari.score
                        + self.riichibou as i32 * 1000
                        + self.tsumobou as i32 * 300;
                    score_diffs[i] = Some(scores[i]);
                } else if i == self.oya as usize {
                    scores[i] = -(oya_payment + self.tsumobou as i32 * 100);
                    score_diffs[i] = Some(scores[i]);
                } else {
                    scores[i] = -(ko_payment + self.tsumobou as i32 * 100);
                    score_diffs[i] = Some(scores[i]);
                }
            }
        }

        for (i, &score) in scores.iter().enumerate().take(self.player_len as usize) {
            self.players[i].score += score;
        }
        self.riichibou = 0;
        self.tsumobou = 0;

        let dora_orig = self
            .get_dora()
            .iter()
            .map(|x| Some(x.get_pai_id()))
            .collect_vec();
        let uradora_orig = self
            .get_uradora()
            .iter()
            .map(|x| Some(x.get_pai_id()))
            .collect_vec();

        play_log.append_agaris_log(
            self.kyoku_id,
            machipai.get_pai_id(),
            best_agari.score,
            best_agari.fu,
            best_agari.han,
            &tehai.iter().map(|x| Some(x.get_pai_id())).collect_vec(),
            &best_agari.yaku,
            &dora_orig,
            &uradora_orig,
            &dora_orig,
            &uradora_orig,
            self.teban as i32,
            self.teban as i32,
            &score_diffs,
            false,
            0,
        );

        Ok(best_agari)
    }

    /// ロン和了の処理を行います。点数計算、スコア移動（放銃者払い）を適用します。
    pub fn ron_agari(
        &mut self,
        play_log: &mut PlayLog,
        winner_idx: usize,
        loser_idx: usize,
        pai: &PaiT,
    ) -> anyhow::Result<Agari> {
        let player = &self.players[winner_idx];
        let mut tehai: Vec<PaiT> = player.tehai.to_vec();
        let machipai = pai.clone();

        tehai.push(machipai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let mut all_mentsu = all_of_mentsu(&mut state, fulo.len());
        if fulo.is_empty() {
            all_mentsu.extend(all_of_chiitoitsu(&state));
            all_mentsu.extend(all_of_kokushi(&state));
        }
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &pai.pack());

        ensure!(!all_mentsu_w_machi.is_empty(), "和了ではありません");

        let mut best_agari =
            self.get_best_agari(winner_idx, &all_mentsu_w_machi, &fulo, 0, true)?;

        let is_oya = winner_idx as u32 == self.oya;
        if is_oya {
            best_agari.score = ((best_agari.score as f32 * 1.5).ceil() as i32 + 99) / 100 * 100;
        }

        let mut scores = [0; 4];
        let mut score_diffs = [Some(0); 4];

        let total_score =
            best_agari.score + self.riichibou as i32 * 1000 + self.tsumobou as i32 * 300;

        for i in 0..self.player_len as usize {
            if i == winner_idx {
                scores[i] = total_score;
                score_diffs[i] = Some(scores[i]);
            } else if i == loser_idx {
                scores[i] = -(best_agari.score + self.tsumobou as i32 * 300);
                score_diffs[i] = Some(scores[i]);
            } else {
                scores[i] = 0;
                score_diffs[i] = Some(0);
            }
        }

        for (i, &score) in scores.iter().enumerate().take(self.player_len as usize) {
            self.players[i].score += score;
        }
        self.riichibou = 0;
        self.tsumobou = 0;

        let dora_orig = self
            .get_dora()
            .iter()
            .map(|x| Some(x.get_pai_id()))
            .collect_vec();
        let uradora_orig = self
            .get_uradora()
            .iter()
            .map(|x| Some(x.get_pai_id()))
            .collect_vec();

        play_log.append_agaris_log(
            self.kyoku_id,
            machipai.get_pai_id(),
            best_agari.score,
            best_agari.fu,
            best_agari.han,
            &tehai.iter().map(|x| Some(x.get_pai_id())).collect_vec(),
            &best_agari.yaku,
            &dora_orig,
            &uradora_orig,
            &dora_orig,
            &uradora_orig,
            winner_idx as i32,
            loser_idx as i32,
            &score_diffs,
            false,
            0,
        );

        Ok(best_agari)
    }

    /// ロンが可能かどうかを判定します。フリテンチェックや役の確認を行います。
    pub fn check_ron(&self, winner_idx: usize, pai: &PaiT) -> Option<Agari> {
        let player = &self.players[winner_idx];
        let mut tehai: Vec<PaiT> = player.tehai.to_vec();
        tehai.push(pai.clone());

        // Genbutsu Furiten Check
        for k in 0..player.kawahai_len as usize {
            if player.kawahai[k].pai_num == pai.pai_num {
                return None;
            }
        }

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let mut all_mentsu = all_of_mentsu(&mut state, fulo.len());
        if fulo.is_empty() {
            all_mentsu.extend(all_of_chiitoitsu(&state));
            all_mentsu.extend(all_of_kokushi(&state));
        }
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &pai.pack());

        if all_mentsu_w_machi.is_empty() {
            return None;
        }

        let best_agari = self.get_best_agari(winner_idx, &all_mentsu_w_machi, &fulo, 0, true);

        match best_agari {
            Ok(mut agari) if agari.score > 0 => {
                let is_oya = winner_idx as u32 == self.oya;
                if is_oya {
                    agari.score = ((agari.score as f32 * 1.5).ceil() as i32 + 99) / 100 * 100;
                }
                Some(agari)
            }
            _ => None,
        }
    }

    fn get_kan_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.player_len as usize {
            for j in 0..self.players[i].mentsu_len as usize {
                let m = &self.players[i].mentsu[j];
                if m.mentsu_type == MentsuType::TYPE_ANKAN
                    || m.mentsu_type == MentsuType::TYPE_MINKAN
                {
                    count += 1;
                }
            }
        }
        count
    }

    fn rinshan_tsumo(&mut self, play_log: &mut PlayLog, player_idx: usize) -> anyhow::Result<()> {
        let kan_count = self.get_kan_count();
        let index = 14 - kan_count;

        let player = &mut self.players[player_idx];
        player.tsumohai = self.taku.get(index)?;
        player.is_tsumo = true;

        play_log.append_actions_log(
            self.kyoku_id,
            player_idx as i32,
            self.seq as i32,
            String::from("rinshan"),
            player.tsumohai.get_pai_id(),
        );
        self.seq += 1;
        Ok(())
    }

    /// 副露（ポン、チー、明カン）を実行します。河から牌を取得し、面子を構築し、手番を更新します。
    pub fn operate_fulo(
        &mut self,
        play_log: &mut PlayLog,
        player_idx: usize,
        mentsu: MentsuT,
    ) -> anyhow::Result<()> {
        let discarder_idx =
            (self.teban as usize + self.player_len as usize - 1) % self.player_len as usize;
        let discarder = &mut self.players[discarder_idx];
        if discarder.kawahai_len > 0 {
            discarder.kawahai[discarder.kawahai_len as usize - 1].is_nakare = true;
        }

        let player = &mut self.players[player_idx];
        let mut tiles_to_remove = Vec::new();

        for i in 0..mentsu.pai_len as usize {
            let mp = &mentsu.pai_list[i];
            if mp.flag == MentsuFlag::FLAG_NONE && mp.pai_num != 0 {
                tiles_to_remove.push(mp.pai_num);
            }
        }

        for &t in &tiles_to_remove {
            if let Some(pos) = player.tehai[0..player.tehai_len as usize]
                .iter()
                .position(|p| p.pai_num == t)
            {
                for j in pos..(player.tehai_len as usize - 1) {
                    player.tehai[j] = player.tehai[j + 1].clone();
                }
                player.tehai_len -= 1;
            } else {
                bail!("Tile not found in hand for fulo");
            }
        }

        player.mentsu[player.mentsu_len as usize] = mentsu.clone();
        player.mentsu_len += 1;

        self.teban = player_idx as u32;

        if mentsu.mentsu_type == MentsuType::TYPE_MINKAN {
            self.rinshan_tsumo(play_log, player_idx)?;
        } else {
            self.players[player_idx].is_tsumo = false;
        }

        play_log.append_actions_log(
            self.kyoku_id,
            player_idx as i32,
            self.seq as i32,
            String::from("fulo"),
            0,
        );
        self.seq += 1;

        Ok(())
    }

    /// 暗カンを実行します。手牌から4枚を除去し、暗カン面子を作成します。ドラも増やします。
    pub fn operate_ankan(
        &mut self,
        play_log: &mut PlayLog,
        player_idx: usize,
        mentsu: MentsuT,
    ) -> anyhow::Result<()> {
        let player = &mut self.players[player_idx];
        let mut tiles_to_remove = Vec::new();
        for i in 0..4 {
            tiles_to_remove.push(mentsu.pai_list[i].pai_num);
        }

        for &t in &tiles_to_remove {
            if player.is_tsumo && player.tsumohai.pai_num == t {
                player.is_tsumo = false;
                player.tsumohai = Default::default();
            } else if let Some(pos) = player.tehai[0..player.tehai_len as usize]
                .iter()
                .position(|p| p.pai_num == t)
            {
                for j in pos..(player.tehai_len as usize - 1) {
                    player.tehai[j] = player.tehai[j + 1].clone();
                }
                player.tehai_len -= 1;
            } else {
                bail!("Tile not found in hand for ankan");
            }
        }

        player.mentsu[player.mentsu_len as usize] = mentsu.clone();
        player.mentsu_len += 1;

        if self.dora_len < 5 {
            self.dora_len += 1;
        }

        play_log.append_actions_log(
            self.kyoku_id,
            player_idx as i32,
            self.seq as i32,
            String::from("ankan"),
            0,
        );
        self.seq += 1;

        self.rinshan_tsumo(play_log, player_idx)?;
        Ok(())
    }

    /// 加カンを実行します。既存のポン面子に牌を加え、加カン面子に変更します。
    pub fn operate_kakan(
        &mut self,
        play_log: &mut PlayLog,
        player_idx: usize,
        mentsu: MentsuT,
    ) -> anyhow::Result<()> {
        let player = &mut self.players[player_idx];
        let added_tile = mentsu.pai_list[3].pai_num;

        if player.is_tsumo && player.tsumohai.pai_num == added_tile {
            player.is_tsumo = false;
            player.tsumohai = Default::default();
        } else if let Some(pos) = player.tehai[0..player.tehai_len as usize]
            .iter()
            .position(|p| p.pai_num == added_tile)
        {
            for j in pos..(player.tehai_len as usize - 1) {
                player.tehai[j] = player.tehai[j + 1].clone();
            }
            player.tehai_len -= 1;
        } else {
            bail!("Tile not found for kakan");
        }

        let mut found = false;
        for i in 0..player.mentsu_len as usize {
            if player.mentsu[i].mentsu_type == MentsuType::TYPE_KOUTSU
                && player.mentsu[i].pai_list[0].pai_num == mentsu.pai_list[0].pai_num
            {
                player.mentsu[i] = mentsu.clone();
                found = true;
                break;
            }
        }

        if !found {
            bail!("Original Pon not found for Kakan");
        }

        if self.dora_len < 5 {
            self.dora_len += 1;
        }

        play_log.append_actions_log(
            self.kyoku_id,
            player_idx as i32,
            self.seq as i32,
            String::from("kakan"),
            0,
        );
        self.seq += 1;

        self.rinshan_tsumo(play_log, player_idx)?;
        Ok(())
    }

    /// チーが可能かどうかを判定し、可能な面子候補のリストを返します。
    pub fn check_chii(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        // Since teban has advanced (after sutehai), teban indicates the next player's turn.
        // Chii can only be done by the next player (kamicha discarded).
        if player_idx != self.teban as usize {
            return res;
        }
        if pai.pai_num >= 27 {
            return res;
        }

        let player = &self.players[player_idx];
        let n = pai.pai_num;
        let num = n % 9;

        if player.is_riichi {
            return res;
        }

        let find = |target: u8| -> Option<usize> {
            player.tehai[0..player.tehai_len as usize]
                .iter()
                .position(|p| p.pai_num == target)
        };

        if num >= 2 {
            if let (Some(i1), Some(i2)) = (find(n - 2), find(n - 1)) {
                let p1 = MentsuPaiT {
                    pai_num: player.tehai[i1].pai_num,
                    id: player.tehai[i1].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p2 = MentsuPaiT {
                    pai_num: player.tehai[i2].pai_num,
                    id: player.tehai[i2].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p3 = MentsuPaiT {
                    pai_num: pai.pai_num,
                    id: pai.id,
                    flag: MentsuFlag::FLAG_KAMICHA,
                };
                let p4 = MentsuPaiT {
                    pai_num: 0,
                    id: 0,
                    flag: MentsuFlag::FLAG_NONE,
                };
                res.push(MentsuT {
                    pai_list: [p1, p2, p3, p4],
                    pai_len: 3,
                    mentsu_type: MentsuType::TYPE_SHUNTSU,
                });
            }
        }

        if (1..=7).contains(&num) {
            if let (Some(i1), Some(i2)) = (find(n - 1), find(n + 1)) {
                let p1 = MentsuPaiT {
                    pai_num: player.tehai[i1].pai_num,
                    id: player.tehai[i1].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p2 = MentsuPaiT {
                    pai_num: player.tehai[i2].pai_num,
                    id: player.tehai[i2].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p3 = MentsuPaiT {
                    pai_num: pai.pai_num,
                    id: pai.id,
                    flag: MentsuFlag::FLAG_KAMICHA,
                };
                let p4 = MentsuPaiT {
                    pai_num: 0,
                    id: 0,
                    flag: MentsuFlag::FLAG_NONE,
                };
                res.push(MentsuT {
                    pai_list: [p1, p2, p3, p4],
                    pai_len: 3,
                    mentsu_type: MentsuType::TYPE_SHUNTSU,
                });
            }
        }

        if num <= 6 {
            if let (Some(i1), Some(i2)) = (find(n + 1), find(n + 2)) {
                let p1 = MentsuPaiT {
                    pai_num: player.tehai[i1].pai_num,
                    id: player.tehai[i1].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p2 = MentsuPaiT {
                    pai_num: player.tehai[i2].pai_num,
                    id: player.tehai[i2].id,
                    flag: MentsuFlag::FLAG_NONE,
                };
                let p3 = MentsuPaiT {
                    pai_num: pai.pai_num,
                    id: pai.id,
                    flag: MentsuFlag::FLAG_KAMICHA,
                };
                let p4 = MentsuPaiT {
                    pai_num: 0,
                    id: 0,
                    flag: MentsuFlag::FLAG_NONE,
                };
                res.push(MentsuT {
                    pai_list: [p1, p2, p3, p4],
                    pai_len: 3,
                    mentsu_type: MentsuType::TYPE_SHUNTSU,
                });
            }
        }

        res
    }

    /// ポンが可能かどうかを判定し、可能な面子候補のリストを返します。
    pub fn check_pon(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        // Updated logic: Teban has advanced, so player CAN be teban (actually must be for Pon usually? No, Pon can be anyone usually except discarder)
        // Discarder = teban-1. Player != Discarder.
        let discarder =
            (self.teban as usize + self.player_len as usize - 1) % self.player_len as usize;
        if player_idx == discarder {
            return res;
        }

        let player = &self.players[player_idx];
        let mut count = 0;
        let mut idxs = Vec::new();

        if player.is_riichi {
            return res;
        }

        for (i, p) in player.tehai[0..player.tehai_len as usize]
            .iter()
            .enumerate()
        {
            if p.pai_num == pai.pai_num {
                count += 1;
                idxs.push(i);
            }
        }

        if count >= 2 {
            let diff = (self.teban as i32 - player_idx as i32 + 4) % 4;
            let flag = match diff {
                1 => MentsuFlag::FLAG_SIMOCHA,
                2 => MentsuFlag::FLAG_TOIMEN,
                3 => MentsuFlag::FLAG_KAMICHA,
                _ => MentsuFlag::FLAG_NONE,
            };

            let p1 = MentsuPaiT {
                pai_num: player.tehai[idxs[0]].pai_num,
                id: player.tehai[idxs[0]].id,
                flag: MentsuFlag::FLAG_NONE,
            };
            let p2 = MentsuPaiT {
                pai_num: player.tehai[idxs[1]].pai_num,
                id: player.tehai[idxs[1]].id,
                flag: MentsuFlag::FLAG_NONE,
            };
            let p3 = MentsuPaiT {
                pai_num: pai.pai_num,
                id: pai.id,
                flag,
            };
            let p4 = MentsuPaiT {
                pai_num: 0,
                id: 0,
                flag: MentsuFlag::FLAG_NONE,
            };
            res.push(MentsuT {
                pai_list: [p1, p2, p3, p4],
                pai_len: 3,
                mentsu_type: MentsuType::TYPE_KOUTSU,
            });
        }

        res
    }

    /// 明カンが可能かどうかを判定し、可能な面子候補のリストを返します。
    pub fn check_minkan(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        let discarder =
            (self.teban as usize + self.player_len as usize - 1) % self.player_len as usize;
        if player_idx == discarder {
            return res;
        }

        let player = &self.players[player_idx];
        let mut count = 0;
        let mut idxs = Vec::new();

        if player.is_riichi {
            return res;
        }

        for (i, p) in player.tehai[0..player.tehai_len as usize]
            .iter()
            .enumerate()
        {
            if p.pai_num == pai.pai_num {
                count += 1;
                idxs.push(i);
            }
        }

        if count >= 3 {
            let diff = (self.teban as i32 - player_idx as i32 + 4) % 4;
            let flag = match diff {
                1 => MentsuFlag::FLAG_SIMOCHA,
                2 => MentsuFlag::FLAG_TOIMEN,
                3 => MentsuFlag::FLAG_KAMICHA,
                _ => MentsuFlag::FLAG_NONE,
            };

            let p1 = MentsuPaiT {
                pai_num: player.tehai[idxs[0]].pai_num,
                id: player.tehai[idxs[0]].id,
                flag: MentsuFlag::FLAG_NONE,
            };
            let p2 = MentsuPaiT {
                pai_num: player.tehai[idxs[1]].pai_num,
                id: player.tehai[idxs[1]].id,
                flag: MentsuFlag::FLAG_NONE,
            };
            let p3 = MentsuPaiT {
                pai_num: player.tehai[idxs[2]].pai_num,
                id: player.tehai[idxs[2]].id,
                flag: MentsuFlag::FLAG_NONE,
            };
            let p4 = MentsuPaiT {
                pai_num: pai.pai_num,
                id: pai.id,
                flag,
            };
            res.push(MentsuT {
                pai_list: [p1, p2, p3, p4],
                pai_len: 4,
                mentsu_type: MentsuType::TYPE_MINKAN,
            });
        }

        res
    }

    /// 暗カンが可能かどうかを判定し、可能な面子候補のリストを返します。
    pub fn check_ankan(&self, player_idx: usize) -> Vec<MentsuT> {
        let mut res = Vec::new();
        let player = &self.players[player_idx];

        let mut counts = [0; 34];
        for p in player.tehai[0..player.tehai_len as usize].iter() {
            counts[p.pai_num as usize] += 1;
        }
        if player.is_tsumo {
            counts[player.tsumohai.pai_num as usize] += 1;
        }

        for (i, &c) in counts.iter().enumerate() {
            if c == 4 {
                let mut pais = Vec::new();
                for p in player.tehai[0..player.tehai_len as usize].iter() {
                    if p.pai_num as usize == i {
                        pais.push(p.clone());
                    }
                }
                if player.is_tsumo && player.tsumohai.pai_num as usize == i {
                    pais.push(player.tsumohai.clone());
                }

                if pais.len() == 4 {
                    let p1 = MentsuPaiT {
                        pai_num: pais[0].pai_num,
                        id: pais[0].id,
                        flag: MentsuFlag::FLAG_NONE,
                    };
                    let p2 = MentsuPaiT {
                        pai_num: pais[1].pai_num,
                        id: pais[1].id,
                        flag: MentsuFlag::FLAG_NONE,
                    };
                    let p3 = MentsuPaiT {
                        pai_num: pais[2].pai_num,
                        id: pais[2].id,
                        flag: MentsuFlag::FLAG_NONE,
                    };
                    let p4 = MentsuPaiT {
                        pai_num: pais[3].pai_num,
                        id: pais[3].id,
                        flag: MentsuFlag::FLAG_NONE,
                    };
                    res.push(MentsuT {
                        pai_list: [p1, p2, p3, p4],
                        pai_len: 4,
                        mentsu_type: MentsuType::TYPE_ANKAN,
                    });
                }
            }
        }
        res
    }

    /// 加カンが可能かどうかを判定し、可能な面子候補のリストを返します。
    pub fn check_kakan(&self, player_idx: usize) -> Vec<MentsuT> {
        let mut res = Vec::new();
        let player = &self.players[player_idx];

        if !player.is_tsumo {
            return res;
        }

        let check_tile = |pai: &PaiT| {
            for m in player.mentsu[0..player.mentsu_len as usize].iter() {
                if m.mentsu_type == MentsuType::TYPE_KOUTSU && m.pai_list[0].pai_num == pai.pai_num
                {
                    return Some((m.clone(), pai.clone()));
                }
            }
            None
        };

        for p in player.tehai[0..player.tehai_len as usize].iter() {
            if let Some((m, tile)) = check_tile(p) {
                let mut list = [
                    MentsuPaiT::default(),
                    MentsuPaiT::default(),
                    MentsuPaiT::default(),
                    MentsuPaiT::default(),
                ];
                list[..3].clone_from_slice(&m.pai_list[..3]);
                list[3] = MentsuPaiT {
                    pai_num: tile.pai_num,
                    id: tile.id,
                    flag: MentsuFlag::FLAG_NONE,
                };

                res.push(MentsuT {
                    pai_list: list,
                    pai_len: 4,
                    mentsu_type: MentsuType::TYPE_MINKAN,
                });
            }
        }

        if let Some((m, tile)) = check_tile(&player.tsumohai) {
            let mut list = [
                MentsuPaiT::default(),
                MentsuPaiT::default(),
                MentsuPaiT::default(),
                MentsuPaiT::default(),
            ];
            list[..3].clone_from_slice(&m.pai_list[..3]);
            list[3] = MentsuPaiT {
                pai_num: tile.pai_num,
                id: tile.id,
                flag: MentsuFlag::FLAG_NONE,
            };
            res.push(MentsuT {
                pai_list: list,
                pai_len: 4,
                mentsu_type: MentsuType::TYPE_MINKAN,
            });
        }

        res
    }

    pub fn nagare(&mut self, play_log: &mut PlayLog) {
        let score = [Some(-3000), Some(0), Some(0), Some(0)];
        play_log.append_nagare_log(self.kyoku_id, String::from("流局"), &score);
    }

    /// 和了者リストと流局状況から、次局への設定・親更新を行います。
    pub fn next_kyoku(&mut self, agari_players: &[usize], is_ryuukyoku: bool) {
        if is_ryuukyoku {
            // 流局時の聴牌判定等は一旦すべて親聴牌（またはノーテン流局）として引数で受け取る
            // ここでは簡易的に(テンパイ判定が無い場合)親流れとするか、agari_players に oya がいれば連荘とみなす
            let is_renchan = agari_players.contains(&(self.oya as usize));
            if is_renchan {
                self.tsumobou += 1;
            } else {
                let prev_oya = self.oya;
                self.oya = (self.oya + 1) % 4;
                if self.oya < prev_oya {
                    self.bakaze += 1;
                }
                self.tsumobou += 1; // 流局による親流れは本場を引き継いで加算
            }
        } else {
            // 和了した場合
            let is_renchan = agari_players.contains(&(self.oya as usize));
            if is_renchan {
                self.tsumobou += 1;
            } else {
                let prev_oya = self.oya;
                self.oya = (self.oya + 1) % 4;
                if self.oya < prev_oya {
                    self.bakaze += 1;
                }
                self.tsumobou = 0; // 和了での親流れは本場リセット
            }
        }
        self.teban = self.oya;
    }

    /// クライアントからのアクションリクエストを処理し、適切なメソッドを呼び出します。
    #[allow(clippy::too_many_arguments)]
    pub fn action(
        &mut self,
        play_log: &mut PlayLog,
        action_type: ActionType,
        player_index: usize,
        param: u32,
    ) -> anyhow::Result<()> {
        match action_type {
            ActionType::ACTION_RIICHI => {
                if player_index == self.teban as usize {
                    let _ = self.sutehai(play_log, param as usize, true);
                    Ok(())
                } else {
                    bail!("not teban")
                }
            }
            ActionType::ACTION_SYNC => {
                if player_index == self.teban as usize {
                    self.tsumo(play_log)?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            ActionType::ACTION_SUTEHAI => {
                if player_index == self.teban as usize {
                    let _ = self.sutehai(play_log, param as usize, false);
                    Ok(())
                } else {
                    bail!("not teban")
                }
            }
            ActionType::ACTION_CHII => {
                let discarder =
                    (self.teban as usize + self.player_len as usize - 1) % self.player_len as usize;
                if self.players[discarder].kawahai_len == 0 {
                    bail!("No discard to Chii");
                }
                let discard = &self.players[discarder].kawahai
                    [self.players[discarder].kawahai_len as usize - 1];
                let cands = self.check_chii(player_index, discard);
                if (param as usize) < cands.len() {
                    self.operate_fulo(play_log, player_index, cands[param as usize].clone())?;
                } else {
                    bail!("Invalid chii param");
                }
                Ok(())
            }
            ActionType::ACTION_PON => {
                let discarder =
                    (self.teban as usize + self.player_len as usize - 1) % self.player_len as usize;
                if self.players[discarder].kawahai_len == 0 {
                    bail!("No discard to Pon");
                }
                let discard = &self.players[discarder].kawahai
                    [self.players[discarder].kawahai_len as usize - 1];
                let cands = self.check_pon(player_index, discard);
                if (param as usize) < cands.len() {
                    self.operate_fulo(play_log, player_index, cands[param as usize].clone())?;
                } else {
                    bail!("Invalid pon param");
                }
                Ok(())
            }
            ActionType::ACTION_KAN => {
                if player_index == self.teban as usize {
                    // Ankan or Kakan
                    let ankans = self.check_ankan(player_index);
                    let kakans = self.check_kakan(player_index);
                    if (param as usize) < ankans.len() {
                        self.operate_ankan(play_log, player_index, ankans[param as usize].clone())?;
                    } else if (param as usize) < ankans.len() + kakans.len() {
                        self.operate_kakan(
                            play_log,
                            player_index,
                            kakans[param as usize - ankans.len()].clone(),
                        )?;
                    } else {
                        bail!("Invalid kan param (self)");
                    }
                } else {
                    // Minkan
                    let discarder = (self.teban as usize + self.player_len as usize - 1)
                        % self.player_len as usize;
                    if self.players[discarder].kawahai_len == 0 {
                        bail!("No discard to Kan");
                    }
                    let discard = &self.players[discarder].kawahai
                        [self.players[discarder].kawahai_len as usize - 1];
                    let cands = self.check_minkan(player_index, discard);
                    if (param as usize) < cands.len() {
                        self.operate_fulo(play_log, player_index, cands[param as usize].clone())?;
                    } else {
                        bail!("Invalid kan param (other)");
                    }
                }
                Ok(())
            }
            ActionType::ACTION_TSUMO => {
                if player_index == self.teban as usize {
                    self.tsumo_agari(play_log)?;
                    Ok(())
                } else {
                    bail!("not teban")
                }
            }
            ActionType::ACTION_NAGASHI => todo!(),
            _ => todo!(),
        }
    }

    pub fn copy_dora(&mut self, dora: &[PaiT]) {
        self.dora_len = dora.len() as u32;
        for (i, item) in dora.iter().enumerate() {
            self.taku.n1[DORA_START_INDEX + i] = item.clone();
        }
    }

    pub fn copy_uradora(&mut self, uradora: &[PaiT]) {
        self.uradora_len = uradora.len() as u32;
        for (i, item) in uradora.iter().enumerate() {
            self.taku.n1[URADORA_START_INDEX + i] = item.clone();
        }
    }

    pub fn get_dora(&self) -> &[PaiT] {
        &self.taku.n1[DORA_START_INDEX..(DORA_START_INDEX + self.dora_len as usize)]
    }

    pub fn get_uradora(&self) -> &[PaiT] {
        &self.taku.n1[URADORA_START_INDEX..(URADORA_START_INDEX + self.uradora_len as usize)]
    }
}
