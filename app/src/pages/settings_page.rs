use crate::types::{GameMode, Message, Settings};
use iced::{
    widget::{button, combo_box, pick_list, row, text, Column, Container},
    Element, Length,
};

pub fn view<'a>(
    settings: &Settings,
    ai_files: &'a [combo_box::State<String>],
) -> Element<'a, Message> {
    let mode_select = row![
        text("Mode:"),
        pick_list(
            &GameMode::ALL[..],
            Some(settings.game_mode),
            Message::SelectMode
        ),
    ]
    .spacing(20);

    let mut ai_selectors = Column::new().spacing(10);

    // Player 0 is usually human
    // AI selection for Player 1, 2, 3 only visible in FourPlayerVsAI mode
    if settings.game_mode == GameMode::FourPlayerVsAI {
        for i in 1..4 {
            ai_selectors = ai_selectors.push(
                row![
                    text(format!("Player {}", i)),
                    combo_box(
                        &ai_files[i],
                        "Select AI",
                        settings.ai_names[i].as_ref(),
                        move |s| Message::SelectAI(i, s)
                    )
                ]
                .spacing(10),
            );
        }
    }

    Container::new(
        Column::new()
            .spacing(20)
            .push(text("Mahjong Settings").size(30))
            .push(mode_select)
            .push(ai_selectors)
            .push(button("Start Game").on_press(Message::Start).padding(10))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
