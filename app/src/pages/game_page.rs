use ai_bridge::interface::G_STATE;
use iced::widget::row;
use iced::{
    widget::{button, column, text, Checkbox, Row},
    Element,
};

use crate::{
    components::{dora, kawahai, tehai},
    types::{AppState, Message},
    utils::painum2path,
};

pub fn view<'a>(state: AppState, turns: u32, is_riichi: bool) -> Element<'a, Message> {
    unsafe {
        // Accessing global G_STATE here as done in original main.rs
        // Ideally this should be passed in, but strictly following refactor of view logic first.
        let core_state = &G_STATE;
        
        let isnt_riichi = !core_state.players[0].is_riichi;
        let shanten = {
             let mut tehai: Vec<mahjong_core::mahjong_generated::open_mahjong::PaiT> = core_state.players[0].tehai.iter().cloned().collect();
             tehai.push(core_state.players[0].tsumohai.clone());
             mahjong_core::shanten::PaiState::from(&tehai).get_shanten(0)
        };

        // We need to implement player_shanten logic here or import it.
        // Let's just create the layout first.

        let dora_elem = dora::view(
            &core_state.get_dora(),
            &core_state.get_uradora(), // Logic for showing uradora depends on state
            state == AppState::Ended,
        );

        let kawahai_elem = kawahai::view(
            &core_state.players[0].kawahai,
            core_state.players[0].kawahai_len as usize,
        );

        let tehai_elem = tehai::view(
            &core_state.players[0].tehai,
            core_state.players[0].tehai_len as usize,
            &core_state.players[0].tsumohai,
            core_state.players[0].is_tsumo,
            state == AppState::Started, // Interactive only if started
        );

        column![
            text("ドラ"),
            dora_elem,
            text(format!("turn {}", turns)),
            text(format!("{} シャンテン", shanten)),
            kawahai_elem,
            tehai_elem,
            row![
                button("ツモ").on_press(Message::Tsumo),
                Checkbox::new("リーチ", is_riichi)
                    .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
            ]
            .spacing(10)
        ]
        .spacing(10)
        .into()
    }
}
