use iced::{
    color,
    widget::{container, image, Row},
    Background, Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{utils::painum2path, Message};

pub fn view<'a>(kawahai: &[PaiT], kawahai_len: usize) -> Element<'a, Message> {
    let elems = kawahai[0..kawahai_len]
        .iter()
        .map(|pai| {
            if pai.is_riichi {
                container(image(painum2path(pai.pai_num as u32)))
                    .style(move |_: &_| container::Appearance {
                        background: Some(Background::Color(color!(0, 0, 255))),
                        ..Default::default()
                    })
                    .padding([0, 0, 4, 0])
                    .into()
            } else {
                container(image(painum2path(pai.pai_num as u32))).into()
            }
        })
        .collect();

    Row::from_vec(elems).into()
}
