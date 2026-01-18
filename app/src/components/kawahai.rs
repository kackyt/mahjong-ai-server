use iced::{
    color,
    widget::{column, container, image, Row},
    Background, Element, Length,
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
    // Determine reversal flags based on angle
    // reverse_flow: Reverse order of tiles within a chunk (Row/Column)
    // reverse_stack: Reverse order of chunks (Rows/Columns)
    let (reverse_flow, reverse_stack) = match angle {
        0 => (false, false),  // P0: L->R, T->B (Stack Std)
        90 => (false, true),  // P3: T->B (Flow Std), R->L (Stack Rev)
        180 => (true, true),  // P2: R->L (Flow Rev), B->T (Stack Rev)
        270 => (true, false), // P1: B->T (Flow Rev), L->R (Stack Std)
        _ => (false, false),
    };

    // Helper to create element
    let create_elem = |pai: &PaiT| {
            let handle = image_cache.get(pai.pai_num as u32, angle, false);
            
            // Scale tiles: 
            // 0/180 (Vertical/Portrait): Height ~38px
            // 90/270 (Horizontal/Landscape): Height ~28px
            // (Assuming approx 3:4 aspect ratio)
            let img_height = match angle {
                90 | 270 => 29.0,
                _ => 38.0,
            };

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

    // Manual chunking
    let mut chunks: Vec<Vec<Element<'a, Message>>> = Vec::new();
    let mut current_chunk = Vec::new();
    
    for pai in kawahai.iter().take(kawahai_len) {
        current_chunk.push(create_elem(pai));
        if current_chunk.len() == 6 {
            if reverse_flow {
                current_chunk.reverse();
            }
            chunks.push(current_chunk);
            current_chunk = Vec::new();
        }
    }
    if !current_chunk.is_empty() {
        if reverse_flow {
            current_chunk.reverse();
        }
        chunks.push(current_chunk);
    }

    if reverse_stack {
        chunks.reverse();
    }

    if is_vertical {
        // Vertical: Row of Columns
        let cols_vec: Vec<_> = chunks.into_iter()
            .map(|items| column(items).spacing(0).into())
            .collect();
        
        Row::with_children(cols_vec).spacing(0).into()
    } else {
        // Horizontal: Column of Rows
        let rows_vec: Vec<_> = chunks.into_iter()
            .map(|items| Row::with_children(items).spacing(0).into())
            .collect();

        column(rows_vec).spacing(0).into()
    }
}
