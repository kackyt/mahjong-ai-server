use iced::{
    widget::{button, image, Row, Space},
    Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images::ImageCache, Message};

pub fn view<'a>(
    tehai: &[PaiT],
    tehai_len: usize,
    tsumohai: &PaiT,
    is_tsumo: bool,
    is_interactive: bool,
    image_cache: &ImageCache,
    angle: u16,
    is_opponent: bool, 
    is_vertical: bool,
) -> Element<'a, Message> {
    let back_idx = 99; 

    let mk_img = |pai_num: u32| {
        image_cache.get(pai_num, angle, false)
    };

    let mut ui_tehai: Vec<Element<'a, Message>> = tehai[0..tehai_len]
        .iter()
        .enumerate()
        .map(|(index, pai)| {
            if is_opponent {
                 image(mk_img(back_idx)).into()
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
             image(mk_img(back_idx)).into()
         } else if is_interactive {
             button(image(mk_img(tsumohai.pai_num as u32)))
                .on_press(Message::Dahai(13)) 
                .padding(0)
                .into()
         } else {
             image(mk_img(tsumohai.pai_num as u32)).into()
         };
         ui_tehai.push(img);
    }

    if is_vertical {
        // Use column for vertical
        iced::widget::Column::from_vec(ui_tehai).into()
    } else {
        Row::from_vec(ui_tehai).into()
    }
}
