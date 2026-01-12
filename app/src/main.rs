
use std::{any, env};

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
    executor, theme,
    widget::{combo_box, container, text, column, button, row},
    Application, Command, Element,
};
use log::{debug, info};
use mahjong_core::{
    game_process::GameProcessError, play_log,
    mahjong_generated::open_mahjong::{ActionType, MentsuT},
};

use modal::Modal;
pub mod modal;

pub mod agent;
pub mod components;
pub mod pages;
pub mod types;
pub mod utils;

use types::{ActionState, AppState, Message, Settings};
use pages::{game_page, settings_page, result_page};
use agent::{Agent, DllAgent, BuiltInAgent};

extern crate libc;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
    is_riichi: bool,
    turns: u32,
    is_show_modal: bool,
    modal_message: String,

    settings: Settings,

    ai_files: combo_box::State<String>,

    agents: Vec<Box<dyn Agent>>,

    action_state: ActionState,
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

    fn init_agents(&mut self) {
        self.agents.clear();
        for i in 1..4 {
            if let Some(name) = &self.settings.ai_names[i] {
                let mut cur = env::current_dir().unwrap();
                cur.push(format!("{}.dll", name));

                if let Ok(handle) = ai_bridge::ai_loader::load_ai(&cur) {
                    if let Ok(symbol) = ai_bridge::ai_loader::get_ai_symbol(handle, "MJPInterfaceFunc") {
                        let ai_symbol: MJPInterfaceFuncP = unsafe { std::mem::transmute(symbol) };
                        unsafe {
                            let size = (ai_symbol)(std::ptr::null_mut(), MJPI_CREATEINSTANCE.try_into().unwrap(), 0, 0);
                            let inst = libc::malloc(size as usize);
                            libc::memset(inst, 0, size as usize);

                            let sendmes_ptr = mjsend_message as *const ();
                            (ai_symbol)(inst, MJPI_INITIALIZE.try_into().unwrap(), 0, std::mem::transmute(sendmes_ptr));

                            self.agents.push(Box::new(DllAgent {
                                name: name.clone(),
                                symbol: ai_symbol,
                                inst,
                            }));
                        }
                        continue;
                    }
                }
            }
            self.agents.push(Box::new(BuiltInAgent));
        }
    }

    unsafe fn check_human_actions(&mut self, discarder_idx: usize, discard: &mahjong_core::mahjong_generated::open_mahjong::PaiT) -> bool {
        let state = &mut G_STATE;
        let human_idx = 0;

        // Skip if self discard
        if discarder_idx == human_idx {
            return false;
        }

        self.action_state = ActionState::default();

        // Check Ron
        if let Some(agari) = state.check_ron(human_idx, discard) {
            self.action_state.ron_candidate = Some(agari);
        }

        // Check Pon
        let pons = state.check_pon(human_idx, discard);
        if !pons.is_empty() {
            self.action_state.pon_candidates = pons;
        }

        // Check Kan (Minkan)
        let kans = state.check_minkan(human_idx, discard);
        if !kans.is_empty() {
            self.action_state.kan_candidates = kans;
        }

        // Check Chii
        let chiis = state.check_chii(human_idx, discard);
        if !chiis.is_empty() {
            self.action_state.chii_candidates = chiis;
        }

        self.action_state.has_any()
    }
}

const FONT_BYTES: &'static [u8] = include_bytes!("../fonts/Mamelon-5-Hi-Regular.otf");

impl Application for App {
    fn title(&self) -> String {
        String::from("openmahjong sample app")
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            Message::SelectMode(is_1p) => {
                self.settings.is_1p_mode = is_1p;
                Command::none()
            }
            Message::SelectAI(idx, name) => {
                if idx < 4 {
                    self.settings.ai_names[idx] = Some(name);
                }
                Command::none()
            }
            Message::Start => unsafe {
                let state = &mut G_STATE;
                let dummy: [i32; 4] = [4, 5, 6, 7];

                info!("Start");

                self.init_agents();

                for agent in &self.agents {
                    if let Some(dll_agent) = agent.as_any().downcast_ref::<DllAgent>() {
                         let symbol = dll_agent.symbol;
                         let inst = dll_agent.inst;
                         (symbol)(inst, MJPI_STARTGAME.try_into().unwrap(), 0, 0);
                         (symbol)(inst, MJPI_BASHOGIME.try_into().unwrap(), std::mem::transmute(dummy.as_ptr()), 0);
                    }
                }

                let player_count = if self.settings.is_1p_mode { 1 } else { 4 };
                state.create(b"test", player_count, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                state.tsumo(&mut self.play_log);

                self.state = AppState::Started;
                self.turns = 0;
                self.is_riichi = false;
                self.action_state = ActionState::default();

                let teban = state.teban as usize;
                if teban != 0 && !self.settings.is_1p_mode {
                    if teban - 1 < self.agents.len() {
                        return self.agents[teban - 1].decide(teban);
                    }
                }

                Command::none()
            },
            Message::Dahai(index) => unsafe {
                let state = &mut G_STATE;
                let state_riichi = player_is_riichi(0);

                let result = state.sutehai(&mut self.play_log, index, !state_riichi && self.is_riichi);

                match result {
                    Ok(_) => {
                        self.turns += 1;
                        if self.turns > 70 {
                             // Check Ryuukyoku logic
                        }

                        // Check if other players can Ron/Pon/Chi/Kan on Human Discard
                        // (Simplified: AI priority Ron logic implementation skipped for brevity,
                        // assuming Human vs AI doesn't strictly require AI interruptions immediately
                        // or we implement simple "AI Ron" check here)

                        // Proceed to next
                        state.tsumo(&mut self.play_log);

                        let teban = state.teban as usize;
                        if teban != 0 {
                             if teban - 1 < self.agents.len() {
                                return self.agents[teban - 1].decide(teban);
                            }
                        }
                    }
                    Err(m) => {
                        self.show_modal(&format!("{:?}", m));
                        self.is_riichi = state_riichi;
                    }
                }
                Command::none()
            },
            Message::AICommand(ret) => unsafe {
                let index = ret & 0x3F;
                let flag = ret & 0xFF80;
                let state = &mut G_STATE;
                let teban = state.teban as usize;

                if flag == MJPIR_TSUMO {
                     let _ = state.tsumo_agari(&mut self.play_log);
                     self.state = AppState::Ended;
                } else if flag == MJPIR_SUTEHAI {
                     let _ = state.sutehai(&mut self.play_log, index as usize, false);

                     // Check Human Actions on AI Discard
                     let discarder_idx = teban;
                     let discard = &state.players[discarder_idx].kawahai[state.players[discarder_idx].kawahai_len as usize - 1];

                     if self.check_human_actions(discarder_idx, discard) {
                         // Stop flow, wait for Human input
                         return Command::none();
                     }

                     state.tsumo(&mut self.play_log);

                     let next_teban = state.teban as usize;
                     if next_teban != 0 {
                         if next_teban - 1 < self.agents.len() {
                            return self.agents[next_teban - 1].decide(next_teban);
                        }
                     }
                }
                Command::none()
            },
            Message::Tsumo => {
                unsafe {
                    let state = &mut G_STATE;
                    let result = state.tsumo_agari(&mut self.play_log);
                    if result.is_ok() {
                        self.state = AppState::Ended;
                    }
                }
                Command::none()
            },
            Message::ToggleRiichi(r) => {
                self.is_riichi = r;
                Command::none()
            },
            Message::FontLoaded => Command::none(),
            Message::HideModal => {
                self.is_show_modal = false;
                Command::none()
            },
            Message::ShowModal(mes) => {
                self.is_show_modal = true;
                self.modal_message = mes;
                Command::none()
            },
            Message::Pass => unsafe {
                // Human passed on action
                self.action_state = ActionState::default();
                let state = &mut G_STATE;

                // Continue game flow
                state.tsumo(&mut self.play_log);
                let next_teban = state.teban as usize;
                if next_teban != 0 {
                     if next_teban - 1 < self.agents.len() {
                        return self.agents[next_teban - 1].decide(next_teban);
                    }
                }
                Command::none()
            },
            Message::Ron => unsafe {
                let state = &mut G_STATE;
                let discarder_idx = state.teban as usize; // Who discarded
                let discard = &state.players[discarder_idx].kawahai[state.players[discarder_idx].kawahai_len as usize - 1];
                let _ = state.ron_agari(&mut self.play_log, 0, discarder_idx, discard);
                self.state = AppState::Ended;
                self.action_state = ActionState::default();
                Command::none()
            },
            Message::Chii(idx) => unsafe {
                let state = &mut G_STATE;
                let _ = state.action(&mut self.play_log, ActionType::ACTION_CHII, 0, idx as u32);
                self.action_state = ActionState::default();
                Command::none()
            },
            Message::Pon(idx) => unsafe {
                let state = &mut G_STATE;
                let _ = state.action(&mut self.play_log, ActionType::ACTION_PON, 0, idx as u32);
                self.action_state = ActionState::default();
                Command::none()
            },
            Message::Kan(idx) => unsafe {
                let state = &mut G_STATE;
                let _ = state.action(&mut self.play_log, ActionType::ACTION_KAN, 0, idx as u32);
                self.action_state = ActionState::default();
                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self.state {
            AppState::Created => {
                settings_page::view(&self.settings, &self.ai_files)
            },
            AppState::Started => {
                let game = game_page::view(self.state, self.turns, self.is_riichi);

                if self.action_state.has_any() {
                    let mut buttons = row![];
                    if self.action_state.ron_candidate.is_some() {
                        buttons = buttons.push(button("Ron").on_press(Message::Ron));
                    }
                    if !self.action_state.pon_candidates.is_empty() {
                        buttons = buttons.push(button("Pon").on_press(Message::Pon(0)));
                    }
                    if !self.action_state.chii_candidates.is_empty() {
                        buttons = buttons.push(button("Chii").on_press(Message::Chii(0)));
                    }
                    if !self.action_state.kan_candidates.is_empty() {
                        buttons = buttons.push(button("Kan").on_press(Message::Kan(0)));
                    }
                    buttons = buttons.push(button("Pass").on_press(Message::Pass));

                    container(
                        column![
                            game,
                            container(buttons).padding(20).style(theme::Container::Box)
                        ]
                    ).into()
                } else {
                    game.into()
                }
            }
            AppState::Ended => {
                result_page::view()
            }
        };

        let containered_content = container(content).padding(10);

        if self.is_show_modal {
            let modal = container(
                column![
                    text(self.modal_message.clone()),
                    button("Close").on_press(Message::HideModal),
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
                is_riichi: false,
                turns: 0,
                is_show_modal: false,
                modal_message: String::new(),
                ai_files: combo_box::State::new(find_dll_files()),
                ai_path: None,
                ai_symbol: dummy_func,
                ai_inst: std::ptr::null_mut(),
                settings: Settings::default(),
                agents: Vec::new(),
                action_state: ActionState::default(),
            },
            load_font,
        )
    }

    type Executor = executor::Default;

    type Theme = iced::Theme;

    type Flags = ();
}

fn main() -> iced::Result {
    env_logger::init();

    App::run(iced::Settings {
        antialiasing: true,
        default_font: iced::Font::with_name("マメロン"),
        ..iced::Settings::default()
    })
}
