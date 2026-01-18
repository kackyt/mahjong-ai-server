use iced::{
    widget::{button, text, Column, Container, Row, scrollable},
    Element, Length, alignment,
};
use crate::types::Message;
use ai_bridge::interface::G_STATE;

pub fn view<'a>(result: Option<&mahjong_core::agari::Agari>) -> Element<'a, Message> {
    unsafe {
        let state = &G_STATE;

        let mut rows = Column::new().spacing(10);
        rows = rows.push(text("Game Result").size(30));

        if let Some(agari) = result {
             let score_text = format!("{} 翻 {} 符  {} 点", agari.han, agari.fu, agari.score);
             rows = rows.push(text(score_text).size(25));
             
             let mut yaku_col = Column::new().spacing(5);
             for (name, han) in &agari.yaku {
                 let yaku_text = if *han < 0 {
                     format!("{}", name) // Yakuman
                 } else {
                     format!("{} {}翻", name, han)
                 };
                 yaku_col = yaku_col.push(text(yaku_text).size(20));
             }
             rows = rows.push(yaku_col);
        }

        // Simple ranking display
        let mut players: Vec<(usize, i32)> = (0..state.player_len as usize)
            .map(|i| (i, state.players[i].score))
            .collect();

        // Sort by score descending
        players.sort_by(|a, b| b.1.cmp(&a.1));

        let mut ranking_col = Column::new().spacing(5);
        for (rank, (idx, score)) in players.iter().enumerate() {
            ranking_col = ranking_col.push(
                Row::new()
                    .push(text(format!("{}. Player {}: ", rank + 1, idx)))
                    .push(text(format!("{}", score)))
                    .spacing(20)
            );
        }
        
        rows = rows.push(text("Ranking").size(20));
        rows = rows.push(ranking_col);

        rows = rows.push(button("Return to Title").on_press(Message::HideModal)); 

        Container::new(scrollable(rows))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
    }
}
