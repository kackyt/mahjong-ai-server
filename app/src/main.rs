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
    widget::{combo_box, container, text, column, button}, // Added column/text/button for modal fallback if needed
    Application, Command, Element,
};
use log::{debug, info};
use mahjong_core::{
    game_process::GameProcessError, play_log,
};

use modal::Modal;
pub mod modal;

pub mod components;
pub mod pages;
pub mod types;
pub mod utils;

use types::{AppState, Message};
use pages::{game_page, title_page};

extern crate libc;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
    is_riichi: bool,
    turns: u32,
    is_show_modal: bool,
    modal_message: String,
    ai_path: Option<String>,
    ai_files: combo_box::State<String>,
    ai_symbol: MJPInterfaceFuncP,
    ai_inst: *mut std::ffi::c_void,
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
        use std::thread::sleep;
        use std::time::Duration;

        sleep(Duration::from_millis(100));

        (self.symbol)(self.inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0)
            .try_into()
            .unwrap()
    }
}

extern "stdcall" fn dummy_func(
    _inst: *mut std::ffi::c_void,
    _message: usize,
    _param1: usize,
    _param2: usize,
) -> usize {
    0
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
            Message::Start => unsafe {
                let state = &mut G_STATE;
                let sendmes_ptr = mjsend_message as *const ();
                let dummy: [i32; 4] = [4, 5, 6, 7];

                info!("Start");

                // AIファイルをロードする
                if let Some(ai_path) = &self.ai_path {
                    let mut cur = env::current_dir().unwrap();
                    cur.push(format!("{}.dll", ai_path));
                    let res = load_ai(&cur);
                    if let Ok(handle) = res {
                        let symbol = get_ai_symbol(handle, "MJPInterfaceFunc");

                        if let Ok(s) = symbol {
                            self.ai_symbol = std::mem::transmute(s);
                            // メモリの確保
                            let size = (self.ai_symbol)(
                                std::ptr::null_mut(),
                                MJPI_CREATEINSTANCE.try_into().unwrap(),
                                0,
                                0,
                            );

                            let inst = libc::malloc(size as usize);

                            libc::memset(inst, 0, size as usize);

                            (self.ai_symbol)(
                                inst,
                                MJPI_INITIALIZE.try_into().unwrap(),
                                0,
                                std::mem::transmute(sendmes_ptr),
                            );
                            (self.ai_symbol)(inst, MJPI_STARTGAME.try_into().unwrap(), 0, 0);
                            (self.ai_symbol)(
                                inst,
                                MJPI_BASHOGIME.try_into().unwrap(),
                                std::mem::transmute(dummy.as_ptr()),
                                0,
                            );

                            self.ai_inst = inst;
                        }
                    }
                }
                state.create(b"test", 1, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                state.tsumo(&mut self.play_log);
                let tsumohai_num: usize = state.players[state.teban as usize]
                    .tsumohai
                    .pai_num
                    .try_into()
                    .unwrap();

                self.state = AppState::Started;
                self.turns = 0;
                self.is_riichi = false;

                if self.ai_inst.is_null() {
                    Command::none()
                } else {
                    let ai = AI {
                        symbol: self.ai_symbol,
                        inst: self.ai_inst,
                    };
                    Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r))
                }
            },
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
                let result =
                    state.sutehai(&mut self.play_log, index, !state_riichi && self.is_riichi);

                match result {
                    Ok(_) => {
                        self.turns += 1;

                        if self.turns > 18 {
                            self.state = AppState::Ended;
                            self.show_modal("流局");
                        } else {
                            state.tsumo(&mut self.play_log);
                        }
                    }
                    Err(m) => {
                        self.show_modal(&format!("{:?}", m));
                        self.is_riichi = state_riichi;
                    }
                }
                Command::none()
            },
            Message::Tsumo => {
                unsafe {
                    let state = &mut G_STATE;
                    let result = state.tsumo_agari(&mut self.play_log);

                    match result {
                        Ok(agari) => {
                            self.state = AppState::Ended;

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
                self.is_riichi = r;
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
            Message::SelectAI(ai_path) => {
                self.ai_path = Some(ai_path);
                Command::none()
            }
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
                                self.state = AppState::Ended;

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

                        (self.ai_symbol)(
                            self.ai_inst,
                            MJPI_ENDKYOKU.try_into().unwrap(),
                            MJEK_RYUKYOKU.try_into().unwrap(),
                            std::mem::transmute(score.as_ptr()),
                        );
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
                            Ok(_) => {
                                self.turns += 1;
                                if self.turns > 18 {
                                    self.state = AppState::Ended;
                                    self.show_modal("流局");
                                    Command::none()
                                } else {
                                    state.tsumo(&mut self.play_log);
                                    let tsumohai_num: usize = state.players[state.teban as usize]
                                        .tsumohai
                                        .pai_num
                                        .try_into()
                                        .unwrap();
                                    let ai = AI {
                                        symbol: self.ai_symbol,
                                        inst: self.ai_inst,
                                    };
                                    Command::perform(ai.ai_next(tsumohai_num), |r| {
                                        Message::AICommand(r)
                                    })
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
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self.state {
            AppState::Created => {
                title_page::view(&self.ai_files, self.ai_path.as_ref())
            },
            AppState::Started | AppState::Ended => {
                game_page::view(self.state, self.turns, self.is_riichi)
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
