use iced::{
    widget::{button, text, Column, Container, Row},
    Element, Length,
};
use crate::types::Message;
use ai_bridge::interface::G_STATE;

pub fn view<'a>() -> Element<'a, Message> {
    unsafe {
        let state = &G_STATE;

        let mut rows = Column::new().spacing(10);
        rows = rows.push(text("Game Result").size(30));

        // Simple ranking display
        // Assuming state.players[i].score is updated

        let mut players: Vec<(usize, i32)> = (0..state.player_len as usize)
            .map(|i| (i, state.players[i].score))
            .collect();

        // Sort by score descending
        players.sort_by(|a, b| b.1.cmp(&a.1));

        for (rank, (idx, score)) in players.iter().enumerate() {
            rows = rows.push(
                Row::new()
                    .push(text(format!("{}. Player {}: ", rank + 1, idx)))
                    .push(text(format!("{}", score)))
                    .spacing(20)
            );
        }

        rows = rows.push(button("Return to Title").on_press(Message::HideModal)); // Or reset state logic

        Container::new(rows)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
