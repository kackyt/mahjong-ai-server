use iced::{
    widget::{button, image, Row, Space},
    Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images, Message};

pub fn view<'a>(
    tehai: &[PaiT],
    tehai_len: usize,
    tsumohai: &PaiT,
    is_tsumo: bool,
    is_interactive: bool,
    is_opponent: bool,
    angle: u16,
) -> Element<'a, Message> {
    let mk_img = |pai_num: u32| images::get(pai_num, angle);

    let mut ui_tehai: Vec<Element<'a, Message>> = tehai[0..tehai_len]
        .iter()
        .enumerate()
        .map(|(index, pai)| {
            if is_opponent {
                image(mk_img(images::BACK_TILE_NUM)).into()
            } else if is_interactive {
                button(image(mk_img(pai.pai_num as u32)))
                    .on_press(Message::Dahai(index))
                    .padding(0)
                    .into()
            } else {
                image(mk_img(pai.pai_num as u32)).into()
            }
        })
        .collect();

    if is_tsumo {
        ui_tehai.push(Space::new(10, 10).into()); // Generic spacing
        let img = if is_opponent {
            image(mk_img(images::BACK_TILE_NUM)).into()
        } else if is_interactive {
            button(image(mk_img(tsumohai.pai_num as u32)))
                .on_press(Message::Dahai(tehai_len))
                .padding(0)
                .into()
        } else {
            image(mk_img(tsumohai.pai_num as u32)).into()
        };
        ui_tehai.push(img);
    }

    Row::from_vec(ui_tehai).into()
}
