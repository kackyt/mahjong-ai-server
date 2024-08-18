use std::env;

use ai_bridge::{
    ai_loader::{get_ai_symbol, load_ai},
    bindings::{
        MJEK_RYUKYOKU, MJPIR_REACH, MJPIR_SUTEHAI, MJPIR_TSUMO, MJPI_BASHOGIME,
        MJPI_CREATEINSTANCE, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_STARTGAME, MJPI_SUTEHAI,
    },
    interface::{mjsend_message, MJPInterfaceFuncP, G_STATE},
};
use iced::{
    color, executor, theme,
    widget::{button, column, combo_box, container, image, row, text, Checkbox, Row, Space},
    Application, Background, Command, Element,
};
use log::{debug, info};
use mahjong_core::{mahjong_generated::open_mahjong::PaiT, play_log, shanten::PaiState};
use modal::Modal;
pub mod modal;

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

        sleep(Duration::from_millis(10));

        (self.symbol)(self.inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0)
            .try_into()
            .unwrap()
    }
}

enum AppState {
    Created,
    Started,
    Ended,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
    Tsumo,
    ToggleRiichi(bool),
    FontLoaded,
    ShowModal(String),
    HideModal,
    SelectAI(String),
    AICommand(u32),
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

fn painum2path(painum: u32) -> String {
    if painum < 9 {
        return format!(
            "{}/images/haiga/man{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum + 1
        );
    }

    if painum < 18 {
        return format!(
            "{}/images/haiga/pin{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum - 8
        );
    }

    if painum < 27 {
        return format!(
            "{}/images/haiga/sou{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            painum - 17
        );
    }

    if painum < 34 {
        let zihai = ["ton", "nan", "sha", "pei", "haku", "hatu", "tyun"];
        return format!(
            "{}/images/haiga/{}.gif",
            env!("CARGO_MANIFEST_DIR"),
            zihai[(painum - 27) as usize]
        );
    }

    format!("{}/images/haiga/ura.gif", env!("CARGO_MANIFEST_DIR"))
}

unsafe fn player_shanten(player_num: usize) -> i32 {
    // シャンテン数を計算
    let state = &G_STATE;
    let mut tehai: Vec<PaiT> = state.players[player_num].tehai.iter().cloned().collect();
    tehai.push(state.players[player_num].tsumohai.clone());

    PaiState::from(&tehai).get_shanten(0)
}

unsafe fn player_is_riichi(player_num: usize) -> bool {
    let state = &G_STATE;

    state.players[player_num].is_riichi
}

fn yaku_to_string(arr: &Vec<(String, i32)>) -> String {
    arr.iter()
        .map(|(yaku, han)| format!("{} {}翻", yaku, han))
        .collect::<Vec<String>>()
        .join("\n")
}

impl App {
    unsafe fn dora<'a>(&self) -> Vec<Element<'a, Message>> {
        match self.state {
            AppState::Created => {
                vec![]
            }
            AppState::Started => {
                let state = &G_STATE;
                let dora = state.get_dora();

                dora.iter()
                    .map(|pai| container(image(painum2path(pai.pai_num as u32))).into())
                    .collect()
            }
            AppState::Ended => {
                let state = &G_STATE;
                let dora = state.get_dora();
                let uradora = state.get_uradora();

                let mut arr = dora
                    .iter()
                    .map(|pai| container(image(painum2path(pai.pai_num as u32))).into())
                    .collect::<Vec<Element<'a, Message>>>();

                arr.push(Space::new(10, 0).into());

                arr.extend(
                    uradora
                        .iter()
                        .map(|pai| container(image(painum2path(pai.pai_num as u32))).into()),
                );

                arr
            }
        }
    }
    unsafe fn kawahai<'a>(&self) -> Vec<Element<'a, Message>> {
        match self.state {
            AppState::Created => {
                vec![]
            }
            AppState::Started | AppState::Ended => {
                let state = &G_STATE;
                let kawahai = &state.players[0].kawahai;
                let kawahai_num = state.players[0].kawahai_len;

                kawahai[0..kawahai_num as usize]
                    .iter()
                    .map(|pai| {
                        if pai.is_riichi {
                            container(image(painum2path(pai.pai_num as u32)))
                                .style(move |_: &_| container::Appearance {
                                    background: Some(Background::Color(color!(0, 0, 255))),
                                    ..Default::default()
                                })
                                .padding([0, 0, 4, 0])
                                .into()
                        } else {
                            container(image(painum2path(pai.pai_num as u32))).into()
                        }
                    })
                    .collect()
            }
        }
    }

    unsafe fn tehai<'a>(&self) -> Vec<Element<'a, Message>> {
        match self.state {
            AppState::Created => {
                vec![]
            }
            AppState::Started => {
                let state = &G_STATE;
                let tehai = &state.players[0].tehai;
                let tehai_num = state.players[0].tehai_len;
                debug!("tehai_num = {}", tehai_num);

                let mut ui_tehai: Vec<Element<'a, Message>> = tehai[0..tehai_num as usize]
                    .iter()
                    .enumerate()
                    .map(|(index, pai)| {
                        button(image(painum2path(pai.pai_num as u32)))
                            .on_press(Message::Dahai(index))
                            .padding(0)
                            .into()
                    })
                    .collect();
                ui_tehai.push(Space::new(10, 0).into());

                ui_tehai.push(
                    button(image(painum2path(state.players[0].tsumohai.pai_num as u32)))
                        .on_press(Message::Dahai(13))
                        .padding(0)
                        .into(),
                );

                ui_tehai
            }
            AppState::Ended => {
                let state = &G_STATE;
                let tehai = &state.players[0].tehai;
                let tehai_num = state.players[0].tehai_len;
                debug!("tehai_num = {}", tehai_num);

                let mut ui_tehai: Vec<Element<'a, Message>> = tehai[0..tehai_num as usize]
                    .iter()
                    .map(|pai| image(painum2path(pai.pai_num as u32)).into())
                    .collect();

                if state.players[0].is_tsumo {
                    ui_tehai.push(Space::new(10, 0).into());
                    ui_tehai
                        .push(image(painum2path(state.players[0].tsumohai.pai_num as u32)).into());
                }

                ui_tehai
            }
        }
    }

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
                        if flag == MJPIR_SUTEHAI {
                            state.sutehai(&mut self.play_log, index as usize, false);
                        } else if flag == MJPIR_REACH {
                            state.sutehai(&mut self.play_log, index as usize, true);
                        }
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
                            Command::perform(ai.ai_next(tsumohai_num), |r| Message::AICommand(r))
                        }
                    }
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        unsafe {
            let isnt_riichi = !player_is_riichi(0);
            let shanten = player_shanten(0);
            let content: Element<_> = column![
                button("Start").on_press(Message::Start),
                text("ドラ"),
                Row::from_vec(self.dora()),
                row![
                    text("AI"),
                    combo_box(
                        &self.ai_files,
                        "AIファイル(.dll)",
                        self.ai_path.as_ref(),
                        Message::SelectAI
                    ),
                ]
                .spacing(10),
                text(format!("{} シャンテン", shanten)),
                Row::from_vec(self.kawahai()),
                Row::from_vec(self.tehai()),
                row![
                    button("ツモ").on_press(Message::Tsumo),
                    Checkbox::new("リーチ", self.is_riichi)
                        .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                ]
                .spacing(10)
            ]
            .spacing(10)
            .padding(10)
            .into();

            let containered_content = container(content);

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
