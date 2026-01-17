use iced::{
    color,
    widget::{container, image, Row, Column},
    Background, Element,
};
use mahjong_core::mahjong_generated::open_mahjong::PaiT;

use crate::{images::ImageCache, Message};

pub fn view<'a>(kawahai: &[PaiT], kawahai_len: usize, angle: u16, image_cache: &ImageCache) -> Element<'a, Message> {
    let valid_kawahai = &kawahai[0..kawahai_len];
    let mut chunks_vec: Vec<Vec<Element<'a, Message>>> = Vec::new();

    for chunk_slice in valid_kawahai.chunks(6) {
        let mut chunk_elements = Vec::new();
        for pai in chunk_slice {
            let handle = image_cache.get(pai.pai_num as u32, angle, pai.is_nakare);
            let img = image(handle);
            
            let container_widget = container(img).padding(1);

            let styled = if pai.is_riichi {
                 container_widget.style(move |_: &_| container::Appearance {
                        background: Some(Background::Color(color!(0, 0, 255))),
                        ..Default::default()
                    })
            } else {
                container_widget
            };
            
            chunk_elements.push(styled.into());
        }
        chunks_vec.push(chunk_elements);
    }

    if angle == 180 {
        // Top Player
        // Items: Right->Left (Screen) -> Reverse Items
        // Chunks: Bottom->Top (Screen) -> Reverse Chunks
        for c in &mut chunks_vec {
            c.reverse();
        }
        chunks_vec.reverse();
    } else if angle == 270 {
        // Right Player
        // Items: Top->Bottom (Screen) to flow Down (Clockwise). Natural is T->B. -> No Reverse
        // Chunks: Left->Right (Screen) -> No Reverse
    } else if angle == 90 {
        // Left Player
        // Items: Bottom->Top (Screen) to flow Up (Clockwise). Natural is T->B. -> Reverse Items
        // Chunks: Right->Left (Screen) -> Reverse Chunks
        for c in &mut chunks_vec {
             c.reverse();
        }
        chunks_vec.reverse();
    }

    if angle == 90 || angle == 270 {
        // Vertical Flow (Side Players)
        let mut main_row = Row::new();
        for c in chunks_vec {
            main_row = main_row.push(Column::with_children(c));
        }
        main_row.into()
    } else {
        // Horizontal Flow (Top/Bottom Players)
        let mut main_col = Column::new();
        for c in chunks_vec {
            main_col = main_col.push(Row::with_children(c));
        }
        main_col.into()
    }
}
