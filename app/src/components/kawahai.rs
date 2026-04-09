use iced::{
    color,
    widget::{container, image, Row},
    Background, Element, Length,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images, Message};

pub fn view<'a>(kawahai: &[PaiT], kawahai_len: usize) -> Element<'a, Message> {
    // Helper to create element
    let create_elem = |pai: &PaiT| {
        let handle = images::get(images::TileImageId(pai.pai_num as u32), 0);

        // Scale tiles:
        // 0/180 (Vertical/Portrait): Height ~38px
        let img_height = 38.0;

        let img = image(handle).height(Length::Fixed(img_height));

        if pai.is_riichi {
            container(img)
                .style(move |_: &_| container::Appearance {
                    background: Some(Background::Color(color!(0, 0, 255))),
                    ..Default::default()
                })
                .padding([0, 0, 4, 0])
                .into()
        } else {
            container(img).into()
        }
    };

    let mut tile_elements: Vec<Element<'a, Message>> = Vec::new();

    // UIレイアウトのズレを防ぐため、先頭に透明なプレースホルダー画像を配置
    tile_elements.push(
        container(image(images::get(images::TRANSPARENT_TILE_NUM, 0)))
            .height(Length::Fixed(38.0))
            .into(),
    );

    for pai in kawahai.iter().take(kawahai_len) {
        tile_elements.push(create_elem(pai));
    }

    Row::with_children(tile_elements).spacing(2).into()
}
