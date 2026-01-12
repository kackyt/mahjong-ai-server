use crate::{
    agari::{add_machi_to_mentsu, Agari, AgariBehavior},
    fbs_utils::TakuControl,
    mahjong_generated::open_mahjong::{
        ActionType, GameStateT, MentsuFlag, MentsuPaiT, MentsuT, MentsuType, PaiT, PlayerT, RuleT,
        TakuT,
    },
    play_log::PlayLog,
    shanten::{all_of_mentsu, PaiState},
};
use anyhow::{bail, ensure};
use chrono::Utc;
use itertools::Itertools;
use rand::seq::SliceRandom;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum GameProcessError {
    #[error("リーチ後はツモ切りのみです")]
    IllegalSutehaiAfterRiichi,
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
        ensure!(unregistered_index.len() != 0, "player is full");

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

    pub fn start(&mut self, play_log: &mut PlayLog) {
        // 配牌
        self.taku_cursol = 14;
        self.dora_len = 1;
        self.uradora_len = 1;
        self.seq = 0;
        let dt = Utc::now();
        self.kyoku_id = (dt.timestamp() / (24 * 3600) * 100000) as u64;
        let mut kazes = [Some(0), Some(0), Some(0), Some(0)];

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
            let cursol: &mut u32;

            player.cursol = 14 + (idx * if idx < 2 { 31 } else { 30 });
            player.kawahai_len = 0;
            player.is_ippatsu = false;
            player.is_riichi = false;

            if self.is_non_duplicate {
                cursol = &mut self.taku_cursol;
            } else {
                cursol = &mut player.cursol;
            }
            let r = self
                .taku
                .get_range((*cursol as usize)..(*cursol + 13) as usize);

            if let Ok(mut v) = r {
                v.sort_unstable();
                for (i, item) in v.into_iter().enumerate() {
                    player.tehai[i] = item;
                }
                player.tehai_len = 13;
            }

            play_log.append_haipais_log(
                self.kyoku_id,
                idx as i32,
                &player.tehai[..player.tehai_len as usize]
                    .into_iter()
                    .map(|x| Some(x.get_pai_id()))
                    .collect::<Vec<Option<u32>>>(),
            );

            *cursol += 13;
        }
    }

    pub fn get_player(&self, index: usize) -> PlayerT {
        self.players[index].clone()
    }

    pub fn tsumo(&mut self, play_log: &mut PlayLog) -> anyhow::Result<()> {
        let player = &mut self.players[self.teban as usize];
        player.is_tsumo = true;

        if self.is_non_duplicate {
            player.tsumohai = self.taku.get(self.taku_cursol as usize)?;
        } else {
            player.tsumohai = self.taku.get(player.cursol as usize)?;
        }

        play_log.append_actions_log(
            self.kyoku_id,
            self.teban as i32,
            self.seq as i32,
            String::from("tsumo"),
            player.tsumohai.get_pai_id(),
        );
        self.seq += 1;

        self.next_cursol();

        Ok(())
    }

    pub fn sutehai(
        &mut self,
        play_log: &mut PlayLog,
        index: usize,
        is_riichi: bool,
    ) -> anyhow::Result<()> {
        let player = &mut self.players[self.teban as usize];
        let mut tehai: Vec<PaiT> = player.tehai.iter().cloned().collect();
        let mut kawahai = match index {
            13 => player.tsumohai.clone(),
            _ => {
                let p = tehai.remove(index);
                tehai.push(player.tsumohai.clone());
                tehai.sort_unstable();
                p
            }
        };

        ensure!(
            !(player.is_riichi && index != 13),
            GameProcessError::IllegalSutehaiAfterRiichi
        );

        if is_riichi {
            ensure!(!player.is_riichi, "すでにリーチしています");
            ensure!(player.mentsu_len == 0, "面前ではありません");
            // シャンテン数チェック
            let mut state = PaiState::from(&tehai);
            let shanten = state.get_shanten(player.mentsu_len as usize);
            ensure!(shanten == 0, "テンパイではありません");

            player.is_riichi = true;
            player.is_ippatsu = true;
            player.score -= 1000;
            kawahai.is_riichi = true;
        } else {
            player.is_ippatsu = false;
        }

        if index != 13 {
            for (i, item) in tehai.into_iter().enumerate() {
                player.tehai[i] = item;
            }
        }

        player.kawahai[player.kawahai_len as usize] = kawahai;

        play_log.append_actions_log(
            self.kyoku_id,
            self.teban as i32,
            self.seq as i32,
            String::from("sutehai"),
            player.kawahai[player.kawahai_len as usize].get_pai_id(),
        );
        self.seq += 1;

        player.kawahai_len += 1;
        player.tsumohai = Default::default();

        player.is_tsumo = false;
        self.teban += 1;
        if self.teban == self.player_len {
            self.teban = 0;
        }

        Ok(())
    }

    pub fn tsumo_agari(&mut self, play_log: &mut PlayLog) -> anyhow::Result<Agari> {
        let player = &self.players[self.teban as usize];
        let mut tehai: Vec<PaiT> = player.tehai.iter().cloned().collect();
        let machipai = player.tsumohai.clone();

        tehai.push(machipai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let all_mentsu = all_of_mentsu(&mut state, fulo.len());
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &player.tsumohai.pack());

        ensure!(all_mentsu_w_machi.len() > 0, "和了ではありません");

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
                    scores[i] = best_agari.score + self.riichibou as i32 * 1000 + self.tsumobou as i32 * 300;
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
                    scores[i] = best_agari.score + self.riichibou as i32 * 1000 + self.tsumobou as i32 * 300;
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

        for i in 0..self.player_len as usize {
            self.players[i].score += scores[i];
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

    pub fn ron_agari(
        &mut self,
        play_log: &mut PlayLog,
        winner_idx: usize,
        loser_idx: usize,
        pai: &PaiT,
    ) -> anyhow::Result<Agari> {
        let player = &self.players[winner_idx];
        let mut tehai: Vec<PaiT> = player.tehai.iter().cloned().collect();
        let machipai = pai.clone();

        tehai.push(machipai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let all_mentsu = all_of_mentsu(&mut state, fulo.len());
        let all_mentsu_w_machi = add_machi_to_mentsu(&all_mentsu, &pai.pack());

        ensure!(all_mentsu_w_machi.len() > 0, "和了ではありません");

        let mut best_agari =
            self.get_best_agari(winner_idx, &all_mentsu_w_machi, &fulo, 0, true)?;

        let is_oya = winner_idx as u32 == self.oya;
        if is_oya {
            best_agari.score = ((best_agari.score as f32 * 1.5).ceil() as i32 + 99) / 100 * 100;
        }

        let mut scores = [0; 4];
        let mut score_diffs = [Some(0); 4];

        let total_score = best_agari.score + self.riichibou as i32 * 1000 + self.tsumobou as i32 * 300;

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

        for i in 0..self.player_len as usize {
            self.players[i].score += scores[i];
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

    pub fn check_ron(&self, winner_idx: usize, pai: &PaiT) -> Option<Agari> {
        let player = &self.players[winner_idx];
        let mut tehai: Vec<PaiT> = player.tehai.iter().cloned().collect();
        tehai.push(pai.clone());

        let mut state = PaiState::from(&tehai);
        let fulo: Vec<crate::mahjong_generated::open_mahjong::Mentsu> = player.mentsu
            [0..player.mentsu_len as usize]
            .iter()
            .map(|m| m.pack())
            .collect();

        let all_mentsu = all_of_mentsu(&mut state, fulo.len());
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
            },
            _ => None,
        }
    }

    fn get_kan_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.player_len as usize {
            for j in 0..self.players[i].mentsu_len as usize {
                let m = &self.players[i].mentsu[j];
                if m.mentsu_type == MentsuType::TYPE_ANKAN ||
                   m.mentsu_type == MentsuType::TYPE_MINKAN {
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

    pub fn operate_fulo(&mut self, play_log: &mut PlayLog, player_idx: usize, mentsu: MentsuT) -> anyhow::Result<()> {
        let discarder_idx = self.teban as usize;
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
            if let Some(pos) = player.tehai[0..player.tehai_len as usize].iter().position(|p| p.pai_num == t) {
                for j in pos..(player.tehai_len as usize - 1) {
                    player.tehai[j] = player.tehai[j+1].clone();
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

    pub fn operate_ankan(&mut self, play_log: &mut PlayLog, player_idx: usize, mentsu: MentsuT) -> anyhow::Result<()> {
        let player = &mut self.players[player_idx];
        let mut tiles_to_remove = Vec::new();
        for i in 0..4 {
             tiles_to_remove.push(mentsu.pai_list[i].pai_num);
        }

        for &t in &tiles_to_remove {
            if player.is_tsumo && player.tsumohai.pai_num == t {
                player.is_tsumo = false;
                player.tsumohai = Default::default();
            } else if let Some(pos) = player.tehai[0..player.tehai_len as usize].iter().position(|p| p.pai_num == t) {
                for j in pos..(player.tehai_len as usize - 1) {
                    player.tehai[j] = player.tehai[j+1].clone();
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

    pub fn operate_kakan(&mut self, play_log: &mut PlayLog, player_idx: usize, mentsu: MentsuT) -> anyhow::Result<()> {
        let player = &mut self.players[player_idx];
        let added_tile = mentsu.pai_list[3].pai_num;

        if player.is_tsumo && player.tsumohai.pai_num == added_tile {
            player.is_tsumo = false;
            player.tsumohai = Default::default();
        } else if let Some(pos) = player.tehai[0..player.tehai_len as usize].iter().position(|p| p.pai_num == added_tile) {
            for j in pos..(player.tehai_len as usize - 1) {
                player.tehai[j] = player.tehai[j+1].clone();
            }
            player.tehai_len -= 1;
        } else {
            bail!("Tile not found for kakan");
        }

        let mut found = false;
        for i in 0..player.mentsu_len as usize {
            if player.mentsu[i].mentsu_type == MentsuType::TYPE_KOUTSU {
                if player.mentsu[i].pai_list[0].pai_num == mentsu.pai_list[0].pai_num {
                    player.mentsu[i] = mentsu.clone();
                    found = true;
                    break;
                }
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

    pub fn check_chii(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        if player_idx != (self.teban as usize + 1) % self.player_len as usize {
            return res;
        }
        if pai.pai_num >= 27 {
            return res;
        }

        let player = &self.players[player_idx];
        let n = pai.pai_num;
        let num = n % 9;

        let find = |target: u8| -> Option<usize> {
            player.tehai[0..player.tehai_len as usize].iter().position(|p| p.pai_num == target)
        };

        if num >= 2 {
            if let (Some(i1), Some(i2)) = (find(n - 2), find(n - 1)) {
                let p1 = MentsuPaiT { pai_num: player.tehai[i1].pai_num, id: player.tehai[i1].id, flag: MentsuFlag::FLAG_NONE };
                let p2 = MentsuPaiT { pai_num: player.tehai[i2].pai_num, id: player.tehai[i2].id, flag: MentsuFlag::FLAG_NONE };
                let p3 = MentsuPaiT { pai_num: pai.pai_num, id: pai.id, flag: MentsuFlag::FLAG_KAMICHA };
                let p4 = MentsuPaiT { pai_num: 0, id: 0, flag: MentsuFlag::FLAG_NONE };
                res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 3, mentsu_type: MentsuType::TYPE_SHUNTSU });
            }
        }

        if num >= 1 && num <= 7 {
            if let (Some(i1), Some(i2)) = (find(n - 1), find(n + 1)) {
                let p1 = MentsuPaiT { pai_num: player.tehai[i1].pai_num, id: player.tehai[i1].id, flag: MentsuFlag::FLAG_NONE };
                let p2 = MentsuPaiT { pai_num: player.tehai[i2].pai_num, id: player.tehai[i2].id, flag: MentsuFlag::FLAG_NONE };
                let p3 = MentsuPaiT { pai_num: pai.pai_num, id: pai.id, flag: MentsuFlag::FLAG_KAMICHA };
                let p4 = MentsuPaiT { pai_num: 0, id: 0, flag: MentsuFlag::FLAG_NONE };
                res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 3, mentsu_type: MentsuType::TYPE_SHUNTSU });
            }
        }

        if num <= 6 {
            if let (Some(i1), Some(i2)) = (find(n + 1), find(n + 2)) {
                let p1 = MentsuPaiT { pai_num: player.tehai[i1].pai_num, id: player.tehai[i1].id, flag: MentsuFlag::FLAG_NONE };
                let p2 = MentsuPaiT { pai_num: player.tehai[i2].pai_num, id: player.tehai[i2].id, flag: MentsuFlag::FLAG_NONE };
                let p3 = MentsuPaiT { pai_num: pai.pai_num, id: pai.id, flag: MentsuFlag::FLAG_KAMICHA };
                let p4 = MentsuPaiT { pai_num: 0, id: 0, flag: MentsuFlag::FLAG_NONE };
                res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 3, mentsu_type: MentsuType::TYPE_SHUNTSU });
            }
        }

        res
    }

    pub fn check_pon(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        if player_idx == self.teban as usize {
            return res;
        }

        let player = &self.players[player_idx];
        let mut count = 0;
        let mut idxs = Vec::new();

        for (i, p) in player.tehai[0..player.tehai_len as usize].iter().enumerate() {
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

            let p1 = MentsuPaiT { pai_num: player.tehai[idxs[0]].pai_num, id: player.tehai[idxs[0]].id, flag: MentsuFlag::FLAG_NONE };
            let p2 = MentsuPaiT { pai_num: player.tehai[idxs[1]].pai_num, id: player.tehai[idxs[1]].id, flag: MentsuFlag::FLAG_NONE };
            let p3 = MentsuPaiT { pai_num: pai.pai_num, id: pai.id, flag: flag };
            let p4 = MentsuPaiT { pai_num: 0, id: 0, flag: MentsuFlag::FLAG_NONE };
            res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 3, mentsu_type: MentsuType::TYPE_KOUTSU });
        }

        res
    }

    pub fn check_minkan(&self, player_idx: usize, pai: &PaiT) -> Vec<MentsuT> {
        let mut res = Vec::new();
        if player_idx == self.teban as usize {
            return res;
        }

        let player = &self.players[player_idx];
        let mut count = 0;
        let mut idxs = Vec::new();

        for (i, p) in player.tehai[0..player.tehai_len as usize].iter().enumerate() {
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

            let p1 = MentsuPaiT { pai_num: player.tehai[idxs[0]].pai_num, id: player.tehai[idxs[0]].id, flag: MentsuFlag::FLAG_NONE };
            let p2 = MentsuPaiT { pai_num: player.tehai[idxs[1]].pai_num, id: player.tehai[idxs[1]].id, flag: MentsuFlag::FLAG_NONE };
            let p3 = MentsuPaiT { pai_num: player.tehai[idxs[2]].pai_num, id: player.tehai[idxs[2]].id, flag: MentsuFlag::FLAG_NONE };
            let p4 = MentsuPaiT { pai_num: pai.pai_num, id: pai.id, flag: flag };
            res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 4, mentsu_type: MentsuType::TYPE_MINKAN });
        }

        res
    }

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
                    let p1 = MentsuPaiT { pai_num: pais[0].pai_num, id: pais[0].id, flag: MentsuFlag::FLAG_NONE };
                    let p2 = MentsuPaiT { pai_num: pais[1].pai_num, id: pais[1].id, flag: MentsuFlag::FLAG_NONE };
                    let p3 = MentsuPaiT { pai_num: pais[2].pai_num, id: pais[2].id, flag: MentsuFlag::FLAG_NONE };
                    let p4 = MentsuPaiT { pai_num: pais[3].pai_num, id: pais[3].id, flag: MentsuFlag::FLAG_NONE };
                    res.push(MentsuT { pai_list: [p1, p2, p3, p4], pai_len: 4, mentsu_type: MentsuType::TYPE_ANKAN });
                }
            }
        }
        res
    }

    pub fn check_kakan(&self, player_idx: usize) -> Vec<MentsuT> {
        let mut res = Vec::new();
        let player = &self.players[player_idx];

        if !player.is_tsumo {
            return res;
        }

        let check_tile = |pai: &PaiT| {
            for m in player.mentsu[0..player.mentsu_len as usize].iter() {
                if m.mentsu_type == MentsuType::TYPE_KOUTSU {
                    if m.pai_list[0].pai_num == pai.pai_num {
                        return Some((m.clone(), pai.clone()));
                    }
                }
            }
            None
        };

        for p in player.tehai[0..player.tehai_len as usize].iter() {
            if let Some((m, tile)) = check_tile(p) {
                let mut list = [MentsuPaiT::default(), MentsuPaiT::default(), MentsuPaiT::default(), MentsuPaiT::default()];
                for i in 0..3 {
                    list[i] = m.pai_list[i].clone();
                }
                list[3] = MentsuPaiT { pai_num: tile.pai_num, id: tile.id, flag: MentsuFlag::FLAG_NONE };

                res.push(MentsuT { pai_list: list, pai_len: 4, mentsu_type: MentsuType::TYPE_MINKAN });
            }
        }

        if let Some((m, tile)) = check_tile(&player.tsumohai) {
             let mut list = [MentsuPaiT::default(), MentsuPaiT::default(), MentsuPaiT::default(), MentsuPaiT::default()];
                for i in 0..3 {
                    list[i] = m.pai_list[i].clone();
                }
                list[3] = MentsuPaiT { pai_num: tile.pai_num, id: tile.id, flag: MentsuFlag::FLAG_NONE };
                res.push(MentsuT { pai_list: list, pai_len: 4, mentsu_type: MentsuType::TYPE_MINKAN });
        }

        res
    }

    pub fn nagare(&mut self, play_log: &mut PlayLog) {
        let score = [Some(-3000), Some(0), Some(0), Some(0)];
        play_log.append_nagare_log(self.kyoku_id, String::from("流局"), &score);
    }

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
                    self.sutehai(play_log, param as usize, true);
                    Ok(())
                } else {
                    bail!("not teban")
                }
            }
            ActionType::ACTION_SYNC => {
                if player_index == self.teban as usize {
                    self.tsumo(play_log)
                } else {
                    Ok(())
                }
            }
            ActionType::ACTION_SUTEHAI => {
                if player_index == self.teban as usize {
                    self.sutehai(play_log, param as usize, false);
                    Ok(())
                } else {
                    bail!("not teban")
                }
            }
            ActionType::ACTION_CHII => {
                let discarder = self.teban as usize;
                if self.players[discarder].kawahai_len == 0 {
                    bail!("No discard to Chii");
                }
                let discard = &self.players[discarder].kawahai[self.players[discarder].kawahai_len as usize - 1];
                let cands = self.check_chii(player_index, discard);
                if (param as usize) < cands.len() {
                    self.operate_fulo(play_log, player_index, cands[param as usize].clone())?;
                } else {
                    bail!("Invalid chii param");
                }
                Ok(())
            }
            ActionType::ACTION_PON => {
                let discarder = self.teban as usize;
                if self.players[discarder].kawahai_len == 0 {
                    bail!("No discard to Pon");
                }
                let discard = &self.players[discarder].kawahai[self.players[discarder].kawahai_len as usize - 1];
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
                        self.operate_kakan(play_log, player_index, kakans[param as usize - ankans.len()].clone())?;
                    } else {
                        bail!("Invalid kan param (self)");
                    }
                } else {
                    // Minkan
                    let discarder = self.teban as usize;
                    if self.players[discarder].kawahai_len == 0 {
                        bail!("No discard to Kan");
                    }
                    let discard = &self.players[discarder].kawahai[self.players[discarder].kawahai_len as usize - 1];
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

    pub fn copy_dora(&mut self, dora: &Vec<PaiT>) {
        self.dora_len = dora.len() as u32;
        for (i, item) in dora.iter().enumerate() {
            self.taku.n1[(DORA_START_INDEX + i) as usize] = item.clone();
        }
    }

    pub fn copy_uradora(&mut self, uradora: &Vec<PaiT>) {
        self.uradora_len = uradora.len() as u32;
        for (i, item) in uradora.iter().enumerate() {
            self.taku.n1[(URADORA_START_INDEX + i) as usize] = item.clone();
        }
    }

    pub fn get_dora(&self) -> &[PaiT] {
        &self.taku.n1[DORA_START_INDEX..(DORA_START_INDEX + self.dora_len as usize)]
    }

    pub fn get_uradora(&self) -> &[PaiT] {
        &self.taku.n1[URADORA_START_INDEX..(URADORA_START_INDEX + self.uradora_len as usize)]
    }
}
