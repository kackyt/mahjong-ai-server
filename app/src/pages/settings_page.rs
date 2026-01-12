use iced::{
    widget::{button, combo_box, row, text, Column, Container, Checkbox},
    Element, Length,
};

use crate::types::{Message, Settings};

pub fn view<'a>(
    settings: &Settings,
    ai_files: &'a combo_box::State<String>,
) -> Element<'a, Message> {
    let mode_select = row![
        text("Mode:"),
        Checkbox::new("1-Player (vs 3 AI)", settings.is_1p_mode)
            .on_toggle(|b| Message::SelectMode(b)),
    ].spacing(20);

    let mut ai_selectors = Column::new().spacing(10);

    // Player 0 is usually human, but let's allow AI selection if we want 4 AI demo?
    // User request: "自分以外は組込みのAIとdll経由のAIを選べるように" (Allow selecting built-in or DLL AI for others, implying Player 0 is Self/Human).

    // So Player 1, 2, 3 selection.
    for i in 1..4 {
        ai_selectors = ai_selectors.push(
            row![
                text(format!("Player {}", i)),
                combo_box(
                    ai_files,
                    "Select AI",
                    settings.ai_names[i].as_ref(),
                    move |s| Message::SelectAI(i, s)
                )
            ].spacing(10)
        );
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
