use iced::{
    color,
    widget::{container, image, Row, Column},
    Background, Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{utils::painum2path, Message};

pub fn view<'a>(kawahai: &[PaiT], kawahai_len: usize) -> Element<'a, Message> {
    let mut rows = Column::new();
    let mut chunk = vec![];

    for (i, pai) in kawahai[0..kawahai_len].iter().enumerate() {
        if i > 0 && i % 6 == 0 {
            rows = rows.push(Row::from_vec(chunk));
            chunk = vec![];
        }

        let img = if pai.is_riichi {
            container(image(painum2path(pai.pai_num as u32)))
                .style(move |_: &_| container::Appearance {
                    background: Some(Background::Color(color!(0, 0, 255))),
                    ..Default::default()
                })
                .padding([0, 0, 4, 0])
                .into()
        } else {
            container(image(painum2path(pai.pai_num as u32))).into()
        };
        chunk.push(img);
    }
    if !chunk.is_empty() {
        rows = rows.push(Row::from_vec(chunk));
    }

    rows.into()
}
