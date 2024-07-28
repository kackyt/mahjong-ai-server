use ai_bridge::interface::G_STATE;
use iced::{
    widget::{button, column, container, image, Row, Space},
    Element, Sandbox
};
use log::{debug, info};
use mahjong_core::play_log;

struct App {
    play_log: play_log::PlayLog,
    state: AppState,
}

enum AppState {
    Created,
    Started,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Dahai(usize),
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

impl Sandbox for App {
    fn title(&self) -> String {
        String::from("openmahjong sample app")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::Start => unsafe {
                let state = &mut G_STATE;

                info!("Start");
                state.create(b"test", 1, &mut self.play_log);
                state.shuffle();
                state.start(&mut self.play_log);
                state.tsumo(&mut self.play_log);
                self.state = AppState::Started;
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
            },
        }
    }

    fn view(&self) -> Element<Message> {
        unsafe {
            let content: Element<_> = column![
                button("Start").on_press(Message::Start),
                Space::new(0, 10),
                Row::from_vec(self.kawahai()),
                Space::new(0, 10),
                Row::from_vec(self.tehai())
            ]
            .padding(10)
            .into();

            container(content).into()
        }
    }

    type Message = Message;

    fn new() -> App {
        App {
            play_log: play_log::PlayLog::new(),
            state: AppState::Created,
        }
    }
}

fn main() -> iced::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    App::run(iced::Settings::default())
}
