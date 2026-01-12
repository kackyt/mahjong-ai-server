use iced::{
    color,
    widget::{container, image, Row, Column},
    Background, Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images::ImageCache, Message};

pub fn view<'a>(kawahai: &[PaiT], kawahai_len: usize, angle: u16, image_cache: &ImageCache) -> Element<'a, Message> {
    let mut rows = Column::new();
    let mut chunk = vec![];

    for (i, pai) in kawahai[0..kawahai_len].iter().enumerate() {
        if i > 0 && i % 6 == 0 {
            rows = rows.push(Row::from_vec(chunk));
            chunk = vec![];
        }

        let handle = image_cache.get(pai.pai_num as u32, angle, pai.is_nakare);
        let img = image(handle);
        
        let container_widget = container(img)
            .padding(1);

        let styled = if pai.is_riichi {
             container_widget.style(move |_: &_| container::Appearance {
                    background: Some(Background::Color(color!(0, 0, 255))),
                    ..Default::default()
                })
        } else {
            container_widget
        };
        
        chunk.push(styled.into());
    }
    if !chunk.is_empty() {
        rows = rows.push(Row::from_vec(chunk));
    }

    rows.into()
}
