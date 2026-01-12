use iced::{
    widget::{button, image, Row, Space},
    Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{utils::painum2path, Message};

pub fn view<'a>(
    tehai: &[PaiT],
    tehai_len: usize,
    tsumohai: &PaiT,
    is_tsumo: bool,
    is_interactive: bool,
) -> Element<'a, Message> {
    let mut ui_tehai: Vec<Element<'a, Message>> = tehai[0..tehai_len]
        .iter()
        .enumerate()
        .map(|(index, pai)| {
            if is_interactive {
                button(image(painum2path(pai.pai_num as u32)))
                    .on_press(Message::Dahai(index))
                    .padding(0)
                    .into()
            } else {
                image(painum2path(pai.pai_num as u32)).into()
            }
        })
        .collect();

    // Tsumohai logic
    // If interactive (Started state), show tsumohai as button if it exists (which main.rs assumed it does in started state)
    // If ended, show tsumohai if is_tsumo is true.

    if is_tsumo {
        ui_tehai.push(Space::new(10, 0).into());
        if is_interactive {
            ui_tehai.push(
                button(image(painum2path(tsumohai.pai_num as u32)))
                    .on_press(Message::Dahai(13))
                    .padding(0)
                    .into(),
            );
        } else {
             ui_tehai.push(image(painum2path(tsumohai.pai_num as u32)).into());
        }
    }

    Row::from_vec(ui_tehai).into()
}
