use iced::widget::{container, row};
use iced::{
    color,
    widget::{button, column, text, Checkbox, Row},
    Background, Element, Length,
};

use crate::{
    components::{dora, kawahai, tehai, fulo}, 
    types::{AppState, Message},
    images::ImageCache, 
};
use ai_bridge::interface::G_STATE;

pub fn view<'a>(state: AppState, turns: u32, is_riichi: bool, image_cache: &ImageCache, can_ron: bool, can_pon: bool, can_chi: bool, can_kan: bool) -> Element<'a, Message> {
    unsafe {
        let core_state = &G_STATE;
        
        let isnt_riichi = !core_state.players[0].is_riichi;
        let shanten = {
             let mut tehai: Vec<mahjong_core::mahjong_generated::open_mahjong::PaiT> = core_state.players[0].tehai.iter().cloned().collect();
             tehai.push(core_state.players[0].tsumohai.clone());
             mahjong_core::shanten::PaiState::from(&tehai).get_shanten(0)
        };

        let dora_elem = dora::view(
            &core_state.get_dora(),
            &core_state.get_uradora(), 
            state == AppState::Ended,
        );

        let player_len = core_state.player_len as usize;

        if player_len == 4 {
             let p0 = &core_state.players[0];
             let p1 = &core_state.players[1];
             let p2 = &core_state.players[2];
             let p3 = &core_state.players[3];

             // Rotation conventions:
             // P0 (Bottom): 0
             // P1 (Right/Shimocha): 270 (Vertical)
             // P2 (Top/Toimen): 180 (Inverted).
             // P3 (Left/Kamicha): 90 (Top pointing Right).

             let p2_kawahai = kawahai::view(&p2.kawahai, p2.kawahai_len as usize, image_cache, 180, false);
             let p3_kawahai = kawahai::view(&p3.kawahai, p3.kawahai_len as usize, image_cache, 90, true);
             let p1_kawahai = kawahai::view(&p1.kawahai, p1.kawahai_len as usize, image_cache, 270, true);
             let p0_kawahai = kawahai::view(&p0.kawahai, p0.kawahai_len as usize, image_cache, 0, false);

             let p0_tehai_elem = tehai::view(
                &p0.tehai,
                p0.tehai_len as usize,
                &p0.tsumohai,
                p0.is_tsumo,
                state == AppState::Started,
                image_cache,
                0,
                false,
                false,
             );
             
             // Opponent Tehais (Face down)
             let p1_tehai = tehai::view(&p1.tehai, p1.tehai_len as usize, &p1.tsumohai, p1.is_tsumo, false, image_cache, 270, true, true);
             let p2_tehai = tehai::view(&p2.tehai, p2.tehai_len as usize, &p2.tsumohai, p2.is_tsumo, false, image_cache, 180, true, false);
             let p3_tehai = tehai::view(&p3.tehai, p3.tehai_len as usize, &p3.tsumohai, p3.is_tsumo, false, image_cache, 90, true, true);

             // Fulou (Melds)
             let p0_fulo = fulo::view(&p0.mentsu[0..p0.mentsu_len as usize], image_cache, false);
             let p1_fulo = fulo::view(&p1.mentsu[0..p1.mentsu_len as usize], image_cache, true);
             let p2_fulo = fulo::view(&p2.mentsu[0..p2.mentsu_len as usize], image_cache, false);
             let p3_fulo = fulo::view(&p3.mentsu[0..p3.mentsu_len as usize], image_cache, true);

             // Styles
             let text_style = |t: &str| text(t).style(color!(255, 255, 255)).size(20);

             // Fixed Layout Construction
             
             // 1. Top Bar (P2 Hand) - Fixed Height
             let top_bar = container(
                 column![
                     text_style("Player 2 (North)"),
                     row![p2_fulo, p2_tehai].spacing(5).align_items(iced::Alignment::Center),
                 ].spacing(5).align_items(iced::Alignment::Center)
             )
             .height(Length::Fixed(120.0))
             .width(Length::Fill)
             .align_y(iced::alignment::Vertical::Bottom)
             .center_x();

             // 2. Bottom Bar (P0 Hand) - Fixed Height
             let bottom_bar = container(
                 column![
                     text_style("Player 0 (You)"),
                     p0_tehai_elem, // Tehai
                     p0_fulo,       // Fulo
                     {
                         let mut r = row![
                            button("ツモ").on_press(Message::Tsumo),
                            Checkbox::new("リーチ", is_riichi)
                                .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                        ].spacing(10);
                        if can_ron {
                            r = r.push(button("ロン").on_press(Message::Ron).style(iced::theme::Button::Primary));
                        }
                        if can_pon {
                            r = r.push(button("ポン").on_press(Message::Pon));
                        }
                        if can_chi {
                            r = r.push(button("チー").on_press(Message::Chi));
                        }
                        if can_kan {
                            r = r.push(button("カン").on_press(Message::Kan));
                        }
                        if can_ron || can_pon || can_chi || can_kan {
                            r = r.push(button("パス").on_press(Message::Pass).style(iced::theme::Button::Secondary));
                        }
                        r
                     }
                 ].spacing(5).align_items(iced::Alignment::Center)
             )
             .height(Length::Fixed(140.0))
             .width(Length::Fill)
             .align_y(iced::alignment::Vertical::Top)
             .center_x();

             // 3. Middle Section
             
             // Left Bar (P3 Hand) - Fixed Width
             let left_bar = container(
                 column![
                     text_style("Player 3 (West)"),
                     row![p3_tehai, p3_fulo].spacing(5).align_items(iced::Alignment::Center), 
                 ].spacing(5).align_items(iced::Alignment::Center)
             )
             .width(Length::Fixed(120.0))
             .height(Length::Fill)
             .align_x(iced::alignment::Horizontal::Right)
             .center_y();

             // Right Bar (P1 Hand) - Fixed Width
             let right_bar = container(
                 column![
                      text_style("Player 1 (East/South)"),
                      row![p1_fulo, p1_tehai].spacing(5).align_items(iced::Alignment::Center)
                 ].spacing(5).align_items(iced::Alignment::Center)
             )
             .width(Length::Fixed(120.0))
             .height(Length::Fill)
             .align_x(iced::alignment::Horizontal::Left)
             .center_y();

             // Center Table (Rivers + Info)
             let center_info = column![
                 text_style("ドラ"),
                 dora_elem,
                 text_style(&format!("残り {} 枚", core_state.remain())),
                 text_style(&format!("{} シャンテン", shanten)),
             ].spacing(5).padding(10).align_items(iced::Alignment::Center);

             let center_table = container(
                 column![
                     // P2 River (Top Center)
                     p2_kawahai,
                     
                     iced::widget::Space::with_height(Length::Fill),
                     
                     // Middle Row (P3 River | Info | P1 River)
                     row![
                         p3_kawahai,
                         iced::widget::Space::with_width(Length::Fill),
                         center_info,
                         iced::widget::Space::with_width(Length::Fill),
                         p1_kawahai
                     ].align_items(iced::Alignment::Center),

                     iced::widget::Space::with_height(Length::Fill),

                     // P0 River (Bottom Center)
                     p0_kawahai
                 ].align_items(iced::Alignment::Center)
             )
             .width(Length::Fill)
             .height(Length::Fill)
             .padding(10); 

             let middle_row = row![
                 left_bar,
                 center_table,
                 right_bar
             ].height(Length::Fill);

             let content = column![
                 top_bar,
                 middle_row,
                 bottom_bar
             ];

             container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(move |_: &_| container::Appearance {
                        background: Some(Background::Color(color!(42, 126, 25))), 
                        ..Default::default()
                })
                .into()

        } else {
             // 1-Player
             let kawahai_elem = kawahai::view(&core_state.players[0].kawahai, core_state.players[0].kawahai_len as usize, image_cache, 0, false);
             let tehai_elem = tehai::view(
                &core_state.players[0].tehai,
                core_state.players[0].tehai_len as usize,
                &core_state.players[0].tsumohai,
                core_state.players[0].is_tsumo,
                state == AppState::Started,
                image_cache,
                0,
                false,
                false
            );

            column![
                text("ドラ"),
                dora_elem,
                text(format!("turn {}", turns)),
                text(format!("{} シャンテン", shanten)),
                kawahai_elem,
                tehai_elem,
                {
                     let mut r = row![
                        button("ツモ").on_press(Message::Tsumo),
                        Checkbox::new("リーチ", is_riichi)
                            .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                    ].spacing(10);
                        if can_ron {
                            r = r.push(button("ロン").on_press(Message::Ron));
                        }
                        if can_pon {
                            r = r.push(button("ポン").on_press(Message::Pon));
                        }
                        if can_chi {
                            r = r.push(button("チー").on_press(Message::Chi));
                        }
                        if can_kan {
                            r = r.push(button("カン").on_press(Message::Kan));
                        }
                        if can_ron || can_pon || can_chi || can_kan {
                            r = r.push(button("パス").on_press(Message::Pass));
                        }
                        r
                     }
            ]
            .spacing(10)
            .into()
        }
    }
}
