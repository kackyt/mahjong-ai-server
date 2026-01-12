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
        let core_state = &G_STATE;
        
        let isnt_riichi = !core_state.players[0].is_riichi;
        let shanten = {
             let mut tehai: Vec<mahjong_core::mahjong_generated::open_mahjong::PaiT> = core_state.players[0].tehai.iter().cloned().collect();
             if core_state.players[0].is_tsumo {
                tehai.push(core_state.players[0].tsumohai.clone());
             }
             mahjong_core::shanten::PaiState::from(&tehai).get_shanten(0)
        };

        let dora_elem = dora::view(
            &core_state.get_dora(),
            &core_state.get_uradora(),
            state == AppState::Ended,
        );

        let kawahai_0 = kawahai::view(&core_state.players[0].kawahai, core_state.players[0].kawahai_len as usize);
        let kawahai_1 = kawahai::view(&core_state.players[1].kawahai, core_state.players[1].kawahai_len as usize);
        let kawahai_2 = kawahai::view(&core_state.players[2].kawahai, core_state.players[2].kawahai_len as usize);
        let kawahai_3 = kawahai::view(&core_state.players[3].kawahai, core_state.players[3].kawahai_len as usize);

        let tehai_elem = tehai::view(
            &core_state.players[0].tehai,
            core_state.players[0].tehai_len as usize,
            &core_state.players[0].tsumohai,
            core_state.players[0].is_tsumo,
            state == AppState::Started,
        );

        let center_area = column![
            text("ドラ").style(Color::WHITE),
            dora_elem,
            text(format!("turn {}", turns)).style(Color::WHITE),
            text(format!("{} シャンテン", shanten)).style(Color::WHITE),
        ].spacing(5).align_items(iced::Alignment::Center);

        let middle_row = row![
            column![text("Player 3 (West)").size(20).style(Color::WHITE), kawahai_3].spacing(5).align_items(iced::Alignment::Center),
            center_area,
            column![text("Player 1 (East/South)").size(20).style(Color::WHITE), kawahai_1].spacing(5).align_items(iced::Alignment::Center),
        ].spacing(40).align_items(iced::Alignment::Center);

        use iced::{Color, Background, widget::container};

        container(
            column![
                // Top (Player 2)
                column![text("Player 2 (North)").size(20).style(Color::WHITE), kawahai_2].spacing(5).align_items(iced::Alignment::Center),
                
                middle_row,
                
                // Bottom (Player 0)
                column![
                    text("Player 0 (You)").size(20).style(Color::WHITE),
                    kawahai_0,
                    tehai_elem,
                    row![
                        button("ツモ").on_press(Message::Tsumo),
                        row![
                            Checkbox::new("", is_riichi)
                                .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                            text("リーチ").style(Color::WHITE),
                        ].spacing(5)
                    ].spacing(10)
                ].spacing(10).align_items(iced::Alignment::Center)
            ]
            .spacing(40)
            .padding(20)
            .align_items(iced::Alignment::Center)
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x()
        .center_y()
        .style(|_: &_| container::Appearance {
            background: Some(Background::Color(Color::from_rgb8(34, 139, 34))), // Forest Green
            ..Default::default()
        })
        .into()
    }
}
