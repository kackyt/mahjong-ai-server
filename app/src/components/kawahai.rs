use iced::{
    color,
    widget::{column, container, image, Row},
    Background, Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images::ImageCache, Message};

pub fn view<'a>(
    kawahai: &[PaiT],
    kawahai_len: usize,
    image_cache: &ImageCache,
    angle: u16,
    is_vertical: bool,
) -> Element<'a, Message> {
    if is_vertical {
        // Vertical: Row of Columns
        // Each Column has max 6 tiles (Top to Bottom)
        // Columns added Left to Right
        let mut cols = Row::new().spacing(0);
        let mut current_col = column![].spacing(0);
        let mut count = 0;

        for pai in kawahai.iter().take(kawahai_len) {
            let handle = image_cache.get(pai.pai_num as u32, angle, false);
            let img: Element<'a, Message> = if pai.is_riichi {
                container(image(handle))
                    .style(move |_: &_| container::Appearance {
                        background: Some(Background::Color(color!(0, 0, 255))),
                        ..Default::default()
                    })
                    .padding([0, 0, 4, 0])
                    .into()
            } else {
                container(image(handle)).into()
            };

            current_col = current_col.push(img);
            count += 1;

            if count % 6 == 0 {
                cols = cols.push(current_col);
                current_col = column![].spacing(0);
            }
        }
        if count % 6 != 0 {
            cols = cols.push(current_col);
        }
        cols.into()
    } else {
        // Horizontal: Column of Rows (Standard)
        // Each Row has max 6 tiles (Left to Right)
        // Rows added Top to Bottom
        let mut rows = column![].spacing(0);
        let mut current_row = Row::new().spacing(0);
        let mut count = 0;

        for pai in kawahai.iter().take(kawahai_len) {
            let handle = image_cache.get(pai.pai_num as u32, angle, false);
            let img: Element<'a, Message> = if pai.is_riichi {
                container(image(handle))
                    .style(move |_: &_| container::Appearance {
                        background: Some(Background::Color(color!(0, 0, 255))),
                        ..Default::default()
                    })
                    .padding([0, 0, 4, 0])
                    .into()
            } else {
                container(image(handle)).into()
            };

            current_row = current_row.push(img);
            count += 1;

            if count % 6 == 0 {
                rows = rows.push(current_row);
                current_row = Row::new().spacing(0);
            }
        }

        if count % 6 != 0 {
            rows = rows.push(current_row);
        }

        rows.into()
    }
}
