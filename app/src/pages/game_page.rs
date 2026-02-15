use iced::widget::{container, row};
use iced::{
    color,
    widget::{button, column, text, Checkbox, Row},
    Background, Element, Length,
};

use crate::{
    components::{dora, fulo, kawahai, tehai},
    images::ImageCache,
    types::{AppState, Message},
};
use ai_bridge::interface::G_STATE;

pub fn view<'a>(
    state: AppState,
    turns: u32,
    riichi_intent: bool,
    image_cache: &ImageCache,
) -> Element<'a, Message> {
    unsafe {
        let core_state = &G_STATE;

        let isnt_riichi = !core_state.players[0].is_riichi;
        let shanten = {
            let mut tehai: Vec<mahjong_core::mahjong_generated::open_mahjong::PaiT> =
                core_state.players[0].tehai.iter().cloned().collect();
            tehai.push(core_state.players[0].tsumohai.clone());
            mahjong_core::shanten::PaiState::from(&tehai).get_shanten(0)
        };

        let dora_elem = dora::view(
            &core_state.get_dora(),
            &core_state.get_uradora(),
            state == AppState::HandEnded || state == AppState::GameFinished,
        );

        let player_len = core_state.player_len as usize;

        if player_len == 4 {
            let p0 = &core_state.players[0];
            let bakaze = core_state.bakaze;
            let kyoku = core_state.kyoku_id; // Using kyoku_id as Kyoku number? No, kyoku_id is timestamp.
                                             // core_state doesn't seem to track 'East 1 Bureau' as a simple number pair in exposed fields easily?
                                             // Actually 'bakaze' is there. 'kyoku_id' is NOT the bureau number.
                                             // Looking at main.rs previously check:
                                             // self.kyoku = 1; ... self.kyoku = self.oya + 1;
                                             // G_STATE doesn't have 'kyoku' (bureau number). It has 'oya' and 'bakaze'.
                                             // Bureau number is roughly oya + 1?
                                             // In 4-player, Oya rotates 0->1->2->3.
                                             // If Oya is 0, it's East 1. If 1, East 2.
            let kyoku_display = core_state.oya + 1;
            let honba = core_state.tsumobou;
            let riichibou = core_state.riichibou;
            let oya = core_state.oya;

            let text_style = |t: &str| text(t).style(color!(255, 255, 255)).size(20);
            let score_style = |t: &str| text(t).style(color!(200, 200, 200)).size(16);
            let oya_marker = |is_oya: bool| {
                if is_oya {
                    text(" [親]").style(color!(255, 100, 100)).size(20)
                } else {
                    text("").size(20)
                }
            };
            let get_wind_name = |w: u32| match w {
                0 => "東",
                1 => "南",
                2 => "西",
                3 => "北",
                _ => "?",
            };

            let players_view = [1, 2, 3]
                .iter()
                .map(|i| {
                    let player = &core_state.players[*i];
                    column![
                        row![
                            text_style(&format!(
                                "Player {} ({})",
                                *i + 1,
                                get_wind_name(*i as u32)
                            )),
                            oya_marker(oya == (*i) as u32),
                            score_style(&format!("{}点", player.score))
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center),
                        kawahai::view(
                            &player.kawahai,
                            player.kawahai_len as usize,
                            image_cache,
                            0,
                            false,
                        ),
                        row![
                            fulo::view(
                                &player.mentsu[0..player.mentsu_len as usize],
                                image_cache,
                                false,
                            ),
                            tehai::view(
                                &player.tehai,
                                player.tehai_len as usize,
                                &player.tsumohai,
                                player.is_tsumo,
                                state == AppState::Started,
                                image_cache,
                                0,
                                false,
                                false,
                            )
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center),
                    ]
                    .spacing(10)
                })
                .collect::<Vec<_>>();

            let p0_kawahai =
                kawahai::view(&p0.kawahai, p0.kawahai_len as usize, image_cache, 0, false);

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
            // Fulou (Melds)
            let p0_fulo = fulo::view(&p0.mentsu[0..p0.mentsu_len as usize], image_cache, false);

            // Derive derived flags using G_STATE
            let mut can_ron = false;
            let mut can_pon = false;
            let mut can_chi = false;
            let mut can_kan = false;

            if core_state.teban as usize != 0 {
                let discarder_idx = (core_state.teban as usize + core_state.player_len as usize)
                    % core_state.player_len as usize;
                if let Some(tile) = core_state.players[discarder_idx].kawahai.iter().last() {
                    println!("DISCARDER: {}", discarder_idx);
                    println!("DISCARD: {}", tile);
                    // 1. Ron
                    let t = mahjong_core::mahjong_generated::open_mahjong::PaiT {
                        pai_num: tile.pai_num,
                        id: 0,
                        is_tsumogiri: false,
                        is_riichi: false,
                        is_nakare: false,
                    };
                    if let Some(_) = core_state.check_ron(0, &t) {
                        can_ron = true;
                        println!("CAN RON");
                    }
                    // 2. Pon/Kan
                    if !core_state.players[0].is_riichi {
                        // Assuming P0
                        if !core_state.check_pon(0, &t).is_empty() {
                            println!("CAN PON");
                            can_pon = true;
                        }
                        // check_minkan roughly corresponds to Kan on discard (Dai-minkan)
                        if !core_state.check_minkan(0, &t).is_empty() {
                            println!("CAN KAN");
                            can_kan = true;
                        }

                        // 3. Chi
                        let from_left = discarder_idx == 3; // P3 is left of P0
                        if from_left {
                            if !core_state.check_chii(0, &t).is_empty() {
                                println!("CAN CHI");
                                can_chi = true;
                            }
                        }
                    }
                }
            }

            // Styles

            let bakaze_text = format!("{} {}局", get_wind_name(bakaze), kyoku_display);
            let honba_text = format!("{}本場 供託{}", honba, riichibou);

            // Fixed Layout Construction

            // 2. Bottom Bar (P0 Hand) - Fixed Height
            let bottom_bar = container(
                column![
                    row![
                        text_style("Player 0 (You)"),
                        oya_marker(oya == 0),
                        score_style(&format!("{}点", p0.score))
                    ]
                    .spacing(10)
                    .align_items(iced::Alignment::Center),
                    p0_kawahai, // Kawahai
                    row![
                        p0_tehai_elem, // Tehai
                        p0_fulo,       // Fulo
                    ]
                    .spacing(10)
                    .align_items(iced::Alignment::Center),
                    {
                        let mut r = Row::new();
                        if p0.is_tsumo {
                            r = r.push(button("ツモ").on_press(Message::Tsumo));
                        }
                        r = r
                            .push(
                                Checkbox::new("リーチ", riichi_intent)
                                    .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                            )
                            .spacing(10);
                        if can_ron {
                            r = r.push(
                                button("ロン")
                                    .on_press(Message::Ron)
                                    .style(iced::theme::Button::Primary),
                            );
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
                            r = r.push(
                                button("パス")
                                    .on_press(Message::Pass)
                                    .style(iced::theme::Button::Secondary),
                            );
                        }
                        r
                    }
                ]
                .spacing(5),
            );

            // Center Table (Rivers + Info)
            let center_info = column![
                text_style("ドラ"),
                dora_elem,
                text_style(&bakaze_text),
                text_style(&honba_text),
                text_style(&format!("残り {} 枚", core_state.remain())),
                text_style(&format!("{} シャンテン", shanten)),
            ]
            .spacing(5)
            .padding(10);

            let mut content = column![center_info];
            for p_view in players_view {
                content = content.push(p_view);
            }
            let content = content.push(bottom_bar);

            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
                .style(move |_: &_| container::Appearance {
                    background: Some(Background::Color(color!(42, 126, 25))),
                    ..Default::default()
                })
                .into()
        } else {
            // 1-Player
            let kawahai_elem = kawahai::view(
                &core_state.players[0].kawahai,
                core_state.players[0].kawahai_len as usize,
                image_cache,
                0,
                false,
            );
            let tehai_elem = tehai::view(
                &core_state.players[0].tehai,
                core_state.players[0].tehai_len as usize,
                &core_state.players[0].tsumohai,
                core_state.players[0].is_tsumo,
                state == AppState::Started,
                image_cache,
                0,
                false,
                false,
            );

            column![
                text("ドラ"),
                dora_elem,
                text(format!("turn {}", turns)),
                text(format!("{} シャンテン", shanten)),
                kawahai_elem,
                tehai_elem,
                {
                    let mut r = Row::new();
                    if core_state.players[0].is_tsumo {
                        r = r.push(button("ツモ").on_press(Message::Tsumo));
                    }
                    r = r
                        .push(
                            Checkbox::new("リーチ", riichi_intent)
                                .on_toggle_maybe(isnt_riichi.then_some(Message::ToggleRiichi)),
                        )
                        .spacing(10);
                    r
                }
            ]
            .spacing(10)
            .into()
        }
    }
}
