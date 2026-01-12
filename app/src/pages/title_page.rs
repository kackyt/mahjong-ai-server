use iced::{
    widget::{button, combo_box, row, text, Column},
    Element,
};

use crate::types::Message;

pub fn view<'a>(
    ai_files: &'a combo_box::State<String>,
    ai_path: Option<&String>,
) -> Element<'a, Message> {
    Column::new()
        .spacing(10)
        .push(button("Start").on_press(Message::Start))
        .push(
            row![
                text("AI"),
                combo_box(ai_files, "AIファイル(.dll)", ai_path, Message::SelectAI),
            ]
            .spacing(10),
        )
        .into()
}
