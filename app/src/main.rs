use ai_bridge::interface::G_STATE;
use iced::{
    executor,
    widget::{button, column, container, image, row, text_input, Row, Space},
    Application, Command, Element,
};
use log::{debug, info};
use mahjong_core::play_log;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
    is_riichi: bool,
    font_loaded: bool,
    input_value: String,
}

enum AppState {
    Created,
    Started,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
    Tsumo,
    Riichi,
    FontLoaded,
    InputChanged(String),
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

impl App {
    unsafe fn kawahai<'a>(&self) -> Vec<Element<'a, Message>> {
        match self.state {
            AppState::Created => {
                vec![]
            }
            AppState::Started => {
                let state = &G_STATE;
                let kawahai = &state.players[0].kawahai;
                let kawahai_num = state.players[0].kawahai_len;

                kawahai[0..kawahai_num as usize]
                    .iter()
                    .map(|pai| container(image(painum2path(pai.pai_num as u32))).into())
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
                            .into()
                    })
                    .collect();

                ui_tehai.push(
                    button(image(painum2path(state.players[0].tsumohai.pai_num as u32)))
                        .on_press(Message::Dahai(13))
                        .into(),
                );

                ui_tehai
            }
        }
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

                info!("Start");
                state.create(b"test", 1, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                state.tsumo(&mut self.play_log);
                self.state = AppState::Started;
                Command::none()
            },
            Message::Dahai(index) => unsafe {
                let state = &mut G_STATE;
                if index < state.players[0].tehai_len as usize {
                    let pai = &state.players[0].tehai[index];
                    debug!("Dahai {}", pai.pai_num);
                } else {
                    let pai = &state.players[0].tsumohai;
                    debug!("Dahai {}", pai.pai_num);
                }
                state.sutehai(&mut self.play_log, index, false);
                state.tsumo(&mut self.play_log);
                Command::none()
            },
            Message::Tsumo => Command::none(),
            Message::Riichi => {
                self.is_riichi = !self.is_riichi;
                Command::none()
            }
            Message::FontLoaded => {
                self.font_loaded = true;
                Command::none()
            }
            Message::InputChanged(value) => {
                self.input_value = value;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        unsafe {
            let content: Element<_> = column![
                button("Start").on_press(Message::Start),
                text_input("Type something", &self.input_value).on_input(Message::InputChanged),
                Row::from_vec(self.kawahai()),
                Row::from_vec(self.tehai()),
            ]
            .push_maybe(self.font_loaded.then(|| {
                row![
                    button("ツモ").on_press(Message::Tsumo),
                    button("リーチ").on_press(Message::Riichi)
                ]
            }))
            .spacing(10)
            .padding(10)
            .into();

            container(content).into()
        }
    }

    type Message = Message;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // let load_font = iced::font::load(FONT_BYTES).map(|_| Message::FontLoaded);
        (
            App {
                play_log: play_log::PlayLog::new(),
                state: AppState::Created,
                is_riichi: false,
                font_loaded: false,
                input_value: String::new(),
            },
            Command::none(),
        )
    }

    type Executor = executor::Default;

    type Theme = iced::Theme;

    type Flags = ();
}

fn main() -> iced::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    App::run(iced::Settings::default())
}
