use crate::{images, Message};
use iced::{
    widget::{image, Row},
    Element,
};
use mahjong_core::mahjong_generated::open_mahjong::MentsuT;

pub fn view<'a>(mentsu_list: &[MentsuT]) -> Element<'a, Message> {
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    for mentsu in mentsu_list {
        let mut meld_row = Row::new().align_items(iced::Alignment::End);

        let tiles = &mentsu.pai_list;
        let len = mentsu.pai_len as usize;

        let mk_img = |pai_num: u32| {
            let handle = images::get(pai_num, 0);
            image(handle)
        };

        for i in 0..len {
            meld_row = meld_row.push(mk_img(tiles[i].pai_num as u32));
        }

        elements.push(meld_row.into());
    }

    let mut row = Row::new().spacing(10);
    for child in elements {
        row = row.push(child);
    }
    row.into()
}
