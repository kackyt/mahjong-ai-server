use std::env;

use ai_bridge::{
    ai_loader::{get_ai_symbol, load_ai},
    bindings::{
        MJEK_RYUKYOKU, MJPIR_REACH, MJPIR_SUTEHAI, MJPIR_TSUMO, MJPI_BASHOGIME,
        MJPI_CREATEINSTANCE, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_STARTGAME, MJPI_SUTEHAI,
    },
    interface::{mjsend_message, MJPInterfaceFuncP, G_STATE},
};
use anyhow::anyhow;
use iced::{
    executor,
    theme,
    widget::{button, column, combo_box, container, text}, // Added column/text/button for modal fallback if needed
    Application,
    Command,
    Element,
};
use log::{debug, info};
use mahjong_core::{
    game_process::GameProcessError, mahjong_generated::open_mahjong::PaiT, play_log,
};

use modal::Modal;
pub mod modal;

pub mod components;
pub mod images;
pub mod pages;
pub mod types;
pub mod utils;

use pages::{game_page, title_page};
use types::{AppState, Message};

extern crate libc;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
    riichi_intent: bool,
    turns: u32,
    is_show_modal: bool,
    modal_message: String,
    game_mode: crate::types::GameMode,
    ai_paths: [Option<String>; 4],
    ai_files: Vec<combo_box::State<String>>,
    ai_instances: Vec<AI>,
    can_ron_flag: bool,
    can_pon_flag: bool,
    can_chi_flag: bool,
    can_kan_flag: bool,
    sutehai: PaiT,
    last_agari_players: Vec<usize>,
    is_ryuukyoku: bool,
}

#[derive(Clone)]
struct AI {
    symbol: MJPInterfaceFuncP,
    inst: *mut std::ffi::c_void,
}

unsafe impl Send for AI {}
unsafe impl Sync for AI {}

impl AI {
    async fn ai_next(self, tsumohai_num: usize) -> u32 {
        // use std::thread::sleep;
        // use std::time::Duration;
        // sleep(Duration::from_millis(100));
        debug!("AI thinking...");

        (self.symbol)(self.inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0)
            .try_into()
            .unwrap()
    }
}

impl Drop for AI {
    fn drop(&mut self) {
        if !self.inst.is_null() {
            unsafe {
                libc::free(self.inst);
            }
        }
    }
}

extern "system" fn dummy_func(
    _inst: *mut std::ffi::c_void,
    _message: usize,
    _param1: usize,
    _param2: usize,
) -> usize {
    // println!("Dummy AI Sutehai");
    (MJPIR_SUTEHAI | 13) as usize
}

fn find_dll_files() -> Vec<String> {
    let mut files = vec![];
    if let Ok(entries) = std::fs::read_dir(env::current_dir().unwrap()) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "dll" {
                                if let Some(file_name) = entry.path().file_stem() {
                                    if let Some(file_name) = file_name.to_str() {
                                        files.push(file_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    files
}

fn yaku_to_string(arr: &Vec<(String, i32)>) -> String {
    arr.iter()
        .map(|(yaku, han)| format!("{} {}翻", yaku, han))
        .collect::<Vec<String>>()
        .join("\n")
}

unsafe fn player_is_riichi(player_num: usize) -> bool {
    let state = &G_STATE;
    state.players[player_num].is_riichi
}

impl App {
    fn show_modal(&mut self, message: &str) {
        self.is_show_modal = true;
        self.modal_message = String::from(message);
    }
}

const FONT_BYTES: &'static [u8] = include_bytes!("../fonts/Mamelon-5-Hi-Regular.otf");

impl Application for App {
    fn title(&self) -> String {
        String::from("openmahjong sample app")
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            // ゲームを開始します。
            Message::Start => unsafe {
                let state = &mut G_STATE;
                let sendmes_ptr = mjsend_message as *const ();
                let dummy: [i32; 4] = [4, 5, 6, 7];

                info!("Start Game Mode: {:?}", self.game_mode);

                self.ai_instances.clear();

                // Initialize AIs if in VsAI mode
                let mut status_messages = Vec::new();
                if self.game_mode == crate::types::GameMode::FourPlayerVsAI {
                    for i in 1..4 {
                        if let Some(ai_path) = &self.ai_paths[i] {
                            let mut cur = env::current_dir().unwrap();
                            cur.push(format!("{}.dll", ai_path));
                            // AI loading logic...
                            let res = load_ai(&cur);
                            if let Ok(handle) = res {
                                let symbol = get_ai_symbol(handle, "MJPInterfaceFunc");
                                if let Ok(s) = symbol {
                                    let ai_symbol: MJPInterfaceFuncP = std::mem::transmute(s);
                                    let size = (ai_symbol)(
                                        std::ptr::null_mut(),
                                        MJPI_CREATEINSTANCE.try_into().unwrap(),
                                        0,
                                        0,
                                    );
                                    let inst = libc::malloc(size as usize);
                                    libc::memset(inst, 0, size as usize);

                                    (ai_symbol)(
                                        inst,
                                        MJPI_INITIALIZE.try_into().unwrap(),
                                        0,
                                        std::mem::transmute(sendmes_ptr),
                                    );
                                    (ai_symbol)(inst, MJPI_STARTGAME.try_into().unwrap(), 0, 0);
                                    (ai_symbol)(
                                        inst,
                                        MJPI_BASHOGIME.try_into().unwrap(),
                                        std::mem::transmute(dummy.as_ptr()),
                                        0,
                                    );

                                    self.ai_instances.push(AI {
                                        symbol: ai_symbol,
                                        inst,
                                    });
                                } else {
                                    status_messages
                                        .push(format!("P{}: Symbol not found in {}", i, ai_path));
                                    self.ai_instances.push(AI {
                                        symbol: dummy_func,
                                        inst: std::ptr::null_mut(),
                                    });
                                }
                            } else {
                                status_messages.push(format!(
                                    "P{}: Load failed for {}: {:?}",
                                    i,
                                    ai_path,
                                    res.err()
                                ));
                                self.ai_instances.push(AI {
                                    symbol: dummy_func,
                                    inst: std::ptr::null_mut(),
                                });
                            }
                        } else {
                            status_messages
                                .push(format!("P{}: No AI selected. Dummy AI will play.", i));
                            self.ai_instances.push(AI {
                                symbol: dummy_func,
                                inst: std::ptr::null_mut(),
                            });
                        }
                    }
                }

                if !status_messages.is_empty() {
                    self.show_modal(&status_messages.join("\n"));
                }

                let player_len = if self.game_mode == crate::types::GameMode::OnePlayerSolo {
                    1
                } else {
                    4
                };
                state.create(b"test", player_len, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                let _ = state.tsumo(&mut self.play_log);

                self.state = AppState::Started;
                self.turns = 0;
                self.riichi_intent = false;

                // Initialize Round Info
                // We should probably initialize G_STATE vals if this is a NEW game.
                state.bakaze = 0;
                state.kyoku_id = 1;
                state.tsumobou = 0;
                state.riichibou = 0;
                state.oya = 0;

                // グローバル待ちフラグクリア
                self.can_ron_flag = false;
                self.can_pon_flag = false;
                self.can_chi_flag = false;
                self.can_kan_flag = false;

                // Trigger AI if it's AI's turn (only in 4P Vs AI)
                let teban = state.teban as usize;
                if self.game_mode == crate::types::GameMode::FourPlayerVsAI && teban != 0 {
                    if teban - 1 < self.ai_instances.len() {
                        let ai = self.ai_instances[teban - 1].clone();
                        let tsumohai_num: usize =
                            state.players[teban].tsumohai.pai_num.try_into().unwrap();
                        return Command::perform(ai.ai_next(tsumohai_num), |r| {
                            Message::AICommand(r)
                        });
                    }
                }

                Command::none()
            },
            // プレイヤー（人間）が打牌した時の処理です。
            Message::Dahai(index) => unsafe {
                let state = &mut G_STATE;
                let state_riichi = player_is_riichi(0);
                if index < state.players[0].tehai_len as usize {
                    let pai = &state.players[0].tehai[index];
                    debug!("Dahai {}", pai.pai_num);
                } else {
                    let pai = &state.players[0].tsumohai;
                    debug!("Dahai {}", pai.pai_num);
                }
                let result = state.sutehai(
                    &mut self.play_log,
                    index,
                    !state_riichi && self.riichi_intent,
                );

                match result {
                    Ok(_) => {
                        self.turns += 1;
                        // 18 turns is for 1-player. 4-player is about 70.
                        let is_game_over =
                            if self.game_mode == crate::types::GameMode::OnePlayerSolo {
                                self.turns > 18
                            } else {
                                state.remain() == 0
                            };

                        if is_game_over {
                            self.state = AppState::HandEnded;
                            self.is_ryuukyoku = true;
                            self.last_agari_players.clear();
                            self.show_modal("流局");
                        } else {
                            let _ = state.tsumo(&mut self.play_log);

                            // Check if next player is AI
                            let next_teban = state.teban as usize;
                            if self.game_mode == crate::types::GameMode::FourPlayerVsAI
                                && next_teban != 0
                            {
                                if next_teban - 1 < self.ai_instances.len() {
                                    debug!("Triggering AI for P{}", next_teban);
                                    let ai = self.ai_instances[next_teban - 1].clone();
                                    let tsumohai_num: usize = state.players[next_teban]
                                        .tsumohai
                                        .pai_num
                                        .try_into()
                                        .unwrap();
                                    return Command::perform(ai.ai_next(tsumohai_num), |r| {
                                        Message::AICommand(r)
                                    });
                                }
                            }
                        }
                    }
                    Err(m) => {
                        self.show_modal(&format!("{:?}", m));
                        self.riichi_intent = state_riichi;
                    }
                }
                Command::none()
            },
            // ツモ和了ボタンが満たされた時の処理です。
            Message::Tsumo => {
                unsafe {
                    let state = &mut G_STATE;
                    let result = state.tsumo_agari(&mut self.play_log);

                    match result {
                        Ok(agari) => {
                            self.state = AppState::HandEnded;
                            self.is_ryuukyoku = false;
                            self.last_agari_players = vec![state.teban as usize];

                            self.show_modal(&format!(
                                "{}\n{}翻\n{}符\n{}点",
                                yaku_to_string(&agari.yaku),
                                agari.han,
                                agari.fu,
                                agari.score
                            ));
                        }
                        Err(m) => {
                            self.show_modal(&format!("{:?}", m));
                        }
                    }
                }
                Command::none()
            }
            Message::ToggleRiichi(r) => {
                self.riichi_intent = r;
                Command::none()
            }
            Message::FontLoaded => Command::none(),
            Message::HideModal => {
                self.is_show_modal = false;
                Command::none()
            }
            Message::ShowModal(mes) => {
                self.is_show_modal = true;
                self.modal_message = mes;
                Command::none()
            }
            Message::SelectMode(mode) => {
                self.game_mode = mode;
                Command::none()
            }
            Message::SelectAI(idx, name) => {
                if idx < 4 {
                    self.ai_paths[idx] = Some(name);
                }
                Command::none()
            }
            // AIからのコマンド（打牌、リーチ、ツモなど）を受信した時の処理です。
            Message::AICommand(ret) => unsafe {
                let index = ret & 0x3F;
                let flag = ret & 0xFF80;

                {
                    let state = &mut G_STATE;

                    if flag == MJPIR_TSUMO {
                        let score: [i32; 4] = [0, 0, 0, 0];
                        info!("agari!!!");
                        let agari_r = state.tsumo_agari(&mut self.play_log);

                        match agari_r {
                            Ok(agari) => {
                                self.state = AppState::HandEnded;
                                self.is_ryuukyoku = false;
                                self.last_agari_players = vec![state.teban as usize];

                                self.show_modal(&format!(
                                    "{}\n{}翻\n{}符\n{}点",
                                    yaku_to_string(&agari.yaku),
                                    agari.han,
                                    agari.fu,
                                    agari.score
                                ));
                            }
                            Err(m) => {
                                self.show_modal(&format!("{:?}", m));
                            }
                        }

                        // Notify all AIs
                        for ai in &self.ai_instances {
                            (ai.symbol)(
                                ai.inst,
                                MJPI_ENDKYOKU.try_into().unwrap(),
                                MJEK_RYUKYOKU.try_into().unwrap(),
                                std::mem::transmute(score.as_ptr()),
                            );
                        }
                        Command::none()
                    } else {
                        let result = match flag {
                            MJPIR_SUTEHAI => {
                                state.sutehai(&mut self.play_log, index as usize, false)
                            }
                            MJPIR_REACH => state.sutehai(&mut self.play_log, index as usize, true),
                            _ => Err(anyhow!("unknown flag {}", flag)),
                        };

                        match result {
                            Ok(sutehai) => {
                                self.sutehai = sutehai;
                                self.turns += 1;
                                let is_game_over =
                                    if self.game_mode == crate::types::GameMode::OnePlayerSolo {
                                        self.turns > 18
                                    } else {
                                        state.remain() == 0
                                    };

                                if is_game_over {
                                    self.state = AppState::HandEnded;
                                    self.is_ryuukyoku = true;
                                    self.last_agari_players.clear();
                                    self.show_modal("流局");
                                    Command::none()
                                } else {
                                    // state.tsumo removed from here

                                    // Reset flags
                                    self.can_ron_flag = false;
                                    self.can_pon_flag = false;
                                    self.can_chi_flag = false;
                                    self.can_kan_flag = false;

                                    let discarder_idx =
                                        (state.teban as usize + state.player_len as usize - 1)
                                            % state.player_len as usize;

                                    // Cannot call on own discard
                                    if discarder_idx != 0 {
                                        // 1. Check RON
                                        debug!("DISCARDER: {}", discarder_idx);
                                        debug!("DISCARD: {}", self.sutehai);

                                        if let Some(_) = state.check_ron(0, &self.sutehai) {
                                            self.can_ron_flag = true;
                                        }

                                        // 2. Check PON/KAN
                                        if !state.check_pon(0, &self.sutehai).is_empty() {
                                            self.can_pon_flag = true;
                                        }
                                        // check_minkan roughly corresponds to Kan on discard (Dai-minkan)
                                        if !state.check_minkan(0, &self.sutehai).is_empty() {
                                            self.can_kan_flag = true;
                                        }
                                        // 3. Check CHI (Only from Kamicha/Left)
                                        // discarder_idx == 3 means Left relative to P0
                                        if discarder_idx == 3 {
                                            if !state.check_chii(0, &self.sutehai).is_empty() {
                                                self.can_chi_flag = true;
                                            }
                                        }

                                        debug!("can_ron_flag: {}, can_pon_flag: {}, can_chi_flag: {}, can_kan_flag: {}", self.can_ron_flag, self.can_pon_flag, self.can_chi_flag, self.can_kan_flag);

                                        if self.can_ron_flag
                                            || self.can_pon_flag
                                            || self.can_chi_flag
                                            || self.can_kan_flag
                                        {
                                            debug!("Pause for Human Action: Ron={}, Pon={}, Chi={}, Kan={}", self.can_ron_flag, self.can_pon_flag, self.can_chi_flag, self.can_kan_flag);
                                            // Pausing by returning none. View will calculate and show buttons.
                                            return Command::none();
                                        }
                                    }

                                    let _ = state.tsumo(&mut self.play_log);

                                    let next_teban = state.teban as usize;

                                    // Check if next player is AI
                                    if self.game_mode == crate::types::GameMode::FourPlayerVsAI
                                        && next_teban != 0
                                    {
                                        if next_teban - 1 < self.ai_instances.len() {
                                            let ai = self.ai_instances[next_teban - 1].clone();
                                            let tsumohai_num: usize = state.players[next_teban]
                                                .tsumohai
                                                .pai_num
                                                .try_into()
                                                .unwrap();
                                            return Command::perform(
                                                ai.ai_next(tsumohai_num),
                                                |r| Message::AICommand(r),
                                            );
                                        }
                                    }
                                    Command::none()
                                }
                            }
                            Err(m) => {
                                if let Some(gp_err) = m.downcast_ref::<GameProcessError>() {
                                    match gp_err {
                                        GameProcessError::IllegalSutehaiAfterRiichi => {}
                                        GameProcessError::Other(e) => {
                                            self.show_modal(&format!("{:?}", e));
                                        }
                                    }
                                } else {
                                    self.show_modal(&format!("{:?}", m));
                                }
                                Command::none()
                            }
                        }
                    }
                }
            },
            // ロンボタンが押された時の処理です。
            Message::Ron => unsafe {
                // Execute Ron
                let state = &mut G_STATE;

                if let Ok(agari) = state.ron_agari(&mut self.play_log, 0, 0, &self.sutehai) {
                    self.state = AppState::HandEnded;
                    self.is_ryuukyoku = false;
                    self.last_agari_players = vec![0];
                    self.show_modal(&format!(
                        "RON!\n{}\n{}翻\n{}符\n{}点",
                        yaku_to_string(&agari.yaku),
                        agari.han,
                        agari.fu,
                        agari.score
                    ));
                } else {
                    self.show_modal("チョンボ！！！");
                }
                Command::none()
            },
            Message::Pass => unsafe {
                // 即座に関連フラグをクリア
                self.can_ron_flag = false;
                self.can_pon_flag = false;
                self.can_chi_flag = false;
                self.can_kan_flag = false;

                // Proceed to next turn
                let state = &mut G_STATE;
                let _ = state.tsumo(&mut self.play_log);
                let next_teban = state.teban as usize;

                // Check if next player is AI
                if self.game_mode == crate::types::GameMode::FourPlayerVsAI && next_teban != 0 {
                    if next_teban - 1 < self.ai_instances.len() {
                        let ai = self.ai_instances[next_teban - 1].clone();
                        let tsumohai_num: usize = state.players[next_teban]
                            .tsumohai
                            .pai_num
                            .try_into()
                            .unwrap();
                        return Command::perform(ai.ai_next(tsumohai_num), |r| {
                            Message::AICommand(r)
                        });
                    }
                }
                Command::none()
            },
            // ポンボタンが押された時の処理です。
            Message::Pon => {
                // 即座にフラグをクリア
                self.can_ron_flag = false;
                self.can_pon_flag = false;
                self.can_chi_flag = false;
                self.can_kan_flag = false;

                unsafe {
                    let state = &mut G_STATE;
                    let cands = state.check_pon(0, &self.sutehai);
                    if let Some(mentsu) = cands.first() {
                        if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                            self.show_modal(&format!("Pon Error: {:?}", e));
                        }
                        // After fulo, it is P0's turn (set in operate_fulo).
                        // Wait for Dahai.
                    }
                }
                Command::none()
            }
            // チーボタンが押された時の処理です。
            Message::Chi => {
                // 即座にフラグをクリア
                self.can_ron_flag = false;
                self.can_pon_flag = false;
                self.can_chi_flag = false;
                self.can_kan_flag = false;

                unsafe {
                    let state = &mut G_STATE;
                    let cands = state.check_chii(0, &self.sutehai);
                    // TODO: Select which Chi if multiple. Default to first.
                    if let Some(mentsu) = cands.first() {
                        if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                            self.show_modal(&format!("Chi Error: {:?}", e));
                        }
                    }
                }
                Command::none()
            }
            // カンボタンが押された時の処理です。
            Message::Kan => {
                // 即座にフラグをクリア
                self.can_ron_flag = false;
                self.can_pon_flag = false;
                self.can_chi_flag = false;
                self.can_kan_flag = false;

                unsafe {
                    let state = &mut G_STATE;
                    let cands = state.check_minkan(0, &self.sutehai);
                    if let Some(mentsu) = cands.first() {
                        if let Err(e) = state.operate_fulo(&mut self.play_log, 0, mentsu.clone()) {
                            self.show_modal(&format!("Kan Error: {:?}", e));
                        }
                    }
                }
                Command::none()
            }
            // 次の局へ進む処理（Next Handボタン）です。親の交代やゲーム終了判定も行います。
            Message::NextHand => unsafe {
                let state = &mut G_STATE;

                if self.is_ryuukyoku {
                    state.next_kyoku(&self.last_agari_players, true);
                } else {
                    state.next_kyoku(&self.last_agari_players, false);
                }

                // Check Game Over
                if state.bakaze > 1 {
                    // Game Over
                    self.state = AppState::GameFinished;
                    // Show Ranking
                    let mut scores: Vec<(usize, i32)> = state
                        .players
                        .iter()
                        .enumerate()
                        .take(4)
                        .map(|(i, p)| (i, p.score))
                        .collect();
                    scores.sort_by(|a, b| b.1.cmp(&a.1));

                    // Oka
                    // オカを1位に加算（実スコアを更新し、表示用もそれに合わせる）
                    state.players[scores[0].0].score += 20000;
                    scores[0].1 = state.players[scores[0].0].score;

                    let msg = scores
                        .iter()
                        .enumerate()
                        .map(|(rank, (idx, s))| format!("{}位: P{} {}", rank + 1, idx, s))
                        .collect::<Vec<String>>()
                        .join("\n");
                    self.show_modal(&format!("Game Over\n\n{}", msg));
                    Command::none()
                } else {
                    // Start Next Hand
                    self.can_ron_flag = false;
                    self.can_pon_flag = false;
                    self.can_chi_flag = false;
                    self.can_kan_flag = false;

                    state.shuffle();
                    state.start(&mut self.play_log);
                    // Oya needs to draw the 14th tile
                    let _ = state.tsumo(&mut self.play_log);

                    self.state = AppState::Started;
                    self.turns = 0;
                    self.riichi_intent = false;
                    Command::none()
                }
            },
            Message::BackToTitle => {
                self.state = AppState::Created;
                self.is_show_modal = false;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self.state {
            AppState::Created => title_page::view(&self.ai_files, &self.ai_paths, self.game_mode),
            AppState::Started | AppState::HandEnded | AppState::GameFinished => game_page::view(
                self.state,
                self.turns,
                self.riichi_intent,
                self.can_ron_flag,
                self.can_pon_flag,
                self.can_chi_flag,
                self.can_kan_flag,
            ),
        };

        let containered_content = container(content).padding(10);

        if self.is_show_modal {
            let modal = container(
                column![
                    text(self.modal_message.clone()),
                    match self.state {
                        AppState::HandEnded => button("Next Hand").on_press(Message::NextHand),
                        AppState::GameFinished =>
                            button("Back to Title").on_press(Message::BackToTitle),
                        _ => button("Close").on_press(Message::HideModal),
                    }
                ]
                .spacing(10)
                .padding(10),
            )
            .style(theme::Container::Box);

            Modal::new(containered_content, modal).into()
        } else {
            containered_content.into()
        }
    }

    type Message = Message;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let load_font = iced::font::load(FONT_BYTES).map(|_| Message::FontLoaded);
        (
            App {
                play_log: play_log::PlayLog::new(),
                state: AppState::Created,
                riichi_intent: false,
                turns: 0,
                is_show_modal: false,
                modal_message: String::new(),
                ai_paths: [None, None, None, None],
                ai_files: {
                    let files = find_dll_files();
                    (0..4)
                        .map(|_| combo_box::State::new(files.clone()))
                        .collect()
                },
                game_mode: crate::types::GameMode::default(),
                // The user code used `ai_symbol` and `ai_inst` (singular).
                // We will need a vector of these for 4-player mode.
                ai_instances: vec![],
                can_ron_flag: false,
                can_pon_flag: false,
                can_chi_flag: false,
                can_kan_flag: false,
                sutehai: PaiT::default(),
                last_agari_players: Vec::new(),
                is_ryuukyoku: false,
            },
            load_font,
        )
    }

    type Executor = executor::Default;

    type Theme = iced::Theme;

    type Flags = ();
}

fn main() -> iced::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    App::run(iced::Settings {
        antialiasing: true,
        default_font: iced::Font::with_name("マメロン"),
        ..iced::Settings::default()
    })
}
