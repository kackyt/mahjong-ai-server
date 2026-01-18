use iced::{
    widget::{container, image, Row, Column},
    Element, Length,
};
use mahjong_core::mahjong_generated::open_mahjong::{MentsuT, MentsuFlag, MentsuType};
use crate::{images::ImageCache, Message};

pub fn view<'a>(mentsu_list: &[MentsuT], image_cache: &ImageCache, is_vertical: bool) -> Element<'a, Message> {
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    for mentsu in mentsu_list {
        let mut meld_row = Row::new().align_items(iced::Alignment::End); 

        let tiles = &mentsu.pai_list;
        let len = mentsu.pai_len as usize;
        
        // Arrange based on flag
        let mut normal_tiles = Vec::new();
        let mut called_tile = None;
        let mut called_flag = MentsuFlag::FLAG_NONE;

        for i in 0..len {
            let p = &tiles[i];
            if p.flag != MentsuFlag::FLAG_NONE && p.flag != MentsuFlag::FLAG_AGARI {
                called_tile = Some(p);
                called_flag = p.flag;
            } else {
                normal_tiles.push(p);
            }
        }

        let mk_img = |pai_num: u32, angle: u16, cache: &ImageCache| {
             let handle = cache.get(pai_num, angle, false);
             image(handle)
        };
        
        if mentsu.mentsu_type == MentsuType::TYPE_ANKAN {
             for p in tiles.iter().take(len) {
                 meld_row = meld_row.push(mk_img(p.pai_num as u32, 0, image_cache));
             }
        } else {
            match called_flag {
                MentsuFlag::FLAG_KAMICHA => {
                    if let Some(p) = called_tile {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 90, image_cache));
                    }
                    for p in normal_tiles {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 0, image_cache));
                    }
                },
                MentsuFlag::FLAG_TOIMEN => {
                     if !normal_tiles.is_empty() {
                         meld_row = meld_row.push(mk_img(normal_tiles[0].pai_num as u32, 0, image_cache));
                     }
                     if let Some(p) = called_tile {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 90, image_cache));
                     }
                     for i in 1..normal_tiles.len() {
                         meld_row = meld_row.push(mk_img(normal_tiles[i].pai_num as u32, 0, image_cache));
                     }
                },
                MentsuFlag::FLAG_SIMOCHA => {
                    for p in normal_tiles {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 0, image_cache));
                    }
                    if let Some(p) = called_tile {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 90, image_cache));
                    }
                },
                _ => {
                    for p in tiles.iter().take(len) {
                        meld_row = meld_row.push(mk_img(p.pai_num as u32, 0, image_cache));
                    }
                }
            }
        }
        
        elements.push(meld_row.into());
    }

    if is_vertical {
        let mut col = Column::new().spacing(10);
        for child in elements {
            col = col.push(child);
        }
        col.into()
    } else {
        let mut row = Row::new().spacing(10);
        for child in elements {
            row = row.push(child);
        }
        row.into()
    }
}
