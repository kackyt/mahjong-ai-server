use iced::widget::{container, row};
use iced::{
    color,
    widget::{button, column, text, Checkbox, Row},
    Background, Element, Length,
};

use crate::{
    components::{dora, fulo, kawahai, tehai},
    types::{AppState, Message},
};
use ai_bridge::interface::G_STATE;

pub fn view<'a>(
    state: AppState,
    turns: u32,
    riichi_intent: bool,
    can_ron_flag: bool,
    can_pon_flag: bool,
    can_chi_flag: bool,
    can_kan_flag: bool,
) -> Element<'a, Message> {
    unsafe {
        let core_state = &G_STATE;

        let isnt_riichi = !core_state.players[0].is_riichi;
        let shanten = {
            // tehaiから有効な牌のみ抽出し、副露数を計算
            let valid_tehai: Vec<_> = core_state.players[0]
                .tehai
                .iter()
                .filter(|p| p.pai_num < 34)
                .cloned()
                .collect();
            let mut all_pai = valid_tehai.clone();
            if core_state.players[0].is_tsumo && core_state.players[0].tsumohai.pai_num < 34 {
                all_pai.push(core_state.players[0].tsumohai.clone());
            }
            let open_meld_count = core_state.players[0].mentsu_len as usize;
            mahjong_core::shanten::PaiState::from(&all_pai).get_shanten(open_meld_count)
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
                                get_wind_name((*i as u32 + 4 - oya) % 4)
                            )),
                            oya_marker(oya == (*i) as u32),
                            score_style(&format!("{}点", player.score))
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center),
                        kawahai::view(&player.kawahai, player.kawahai_len as usize),
                        row![
                            fulo::view(&player.mentsu[0..player.mentsu_len as usize]),
                            tehai::view(
                                &player.tehai,
                                player.tehai_len as usize,
                                &player.tsumohai,
                                player.is_tsumo,
                                false,
                                true,
                                match *i {
                                    1 => 270,
                                    2 => 180,
                                    3 => 90,
                                    _ => 0,
                                },
                            )
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center),
                    ]
                    .spacing(10)
                })
                .collect::<Vec<_>>();

            let p0_kawahai = kawahai::view(&p0.kawahai, p0.kawahai_len as usize);

            let p0_tehai_elem = tehai::view(
                &p0.tehai,
                p0.tehai_len as usize,
                &p0.tsumohai,
                p0.is_tsumo,
                state == AppState::Started,
                false,
                0,
            );
            // Fulou (Melds)
            let p0_fulo = fulo::view(&p0.mentsu[0..p0.mentsu_len as usize]);

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
                        if can_ron_flag {
                            r = r.push(
                                button("ロン")
                                    .on_press(Message::Ron)
                                    .style(iced::theme::Button::Primary),
                            );
                        }
                        if can_pon_flag {
                            r = r.push(button("ポン").on_press(Message::Pon));
                        }
                        if can_chi_flag {
                            r = r.push(button("チー").on_press(Message::Chi));
                        }
                        if can_kan_flag {
                            r = r.push(button("カン").on_press(Message::Kan));
                        }
                        if can_ron_flag || can_pon_flag || can_chi_flag || can_kan_flag {
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
            );
            let tehai_elem = tehai::view(
                &core_state.players[0].tehai,
                core_state.players[0].tehai_len as usize,
                &core_state.players[0].tsumohai,
                core_state.players[0].is_tsumo,
                state == AppState::Started,
                false,
                0,
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
