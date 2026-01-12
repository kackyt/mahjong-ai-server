use iced::{
    widget::{container, image, Row, Space},
    Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{utils::painum2path, Message};

pub fn view<'a>(dora: &[PaiT], uradora: &[PaiT], is_ended: bool) -> Element<'a, Message> {
    let mut arr = dora
        .iter()
        .map(|pai| container(image(painum2path(pai.pai_num as u32))).into())
        .collect::<Vec<Element<'a, Message>>>();

    if is_ended {
        arr.push(Space::new(10, 0).into());
        arr.extend(
            uradora
                .iter()
                .map(|pai| container(image(painum2path(pai.pai_num as u32))).into()),
        );
    }

    Row::from_vec(arr).into()
}
