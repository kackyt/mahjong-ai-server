use iced::{
    widget::{button, combo_box, pick_list, row, text, Column},
    Element,
};

use crate::types::{GameMode, Message};

pub fn view<'a>(
    ai_files: &'a [combo_box::State<String>],
    ai_paths: &[Option<String>],
    game_mode: GameMode,
) -> Element<'a, Message> {
    let mut content = Column::new().spacing(10);

    // Game Mode Selection
    content = content.push(
        row![
            text("Mode:"),
            pick_list(&GameMode::ALL[..], Some(game_mode), Message::SelectMode),
        ]
        .spacing(10),
    );

    // AI Selection (Only for 4-Player Vs AI)
    if game_mode == GameMode::FourPlayerVsAI {
        for i in 1..4 {
            content = content.push(
                row![
                    text(format!("Player {} AI:", i)),
                    combo_box(
                        &ai_files[i],
                        "Select AI",
                        ai_paths[i].as_ref(),
                        move |s| Message::SelectAI(i, s)
                    ),
                ]
                .spacing(10),
            );
        }
    }

    content.push(button("Start").on_press(Message::Start)).into()
}
