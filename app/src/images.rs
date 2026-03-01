use iced::widget::image;
use std::cell::RefCell;
use std::collections::HashMap;

pub const BACK_TILE_NUM: u32 = 99;

thread_local! {
    static CACHE: RefCell<HashMap<(u32, u16), image::Handle>> = RefCell::new(HashMap::new());
}

pub fn get(pai_num: u32, angle: u16) -> image::Handle {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(handle) = cache.get(&(pai_num, angle)) {
            return handle.clone();
        }

        let handle = if pai_num == BACK_TILE_NUM {
            image::Handle::from_path(format!(
                "{}/images/haiga/transparent.gif",
                env!("CARGO_MANIFEST_DIR")
            ))
        } else {
            let prefix = match angle {
                90 => "ty",
                180 => "t",
                270 => "y",
                _ => "",
            };

            let name = get_tile_name(pai_num);
            let filename = format!("{}{}.gif", prefix, name);
            let path = format!("{}/images/haiga/{}", env!("CARGO_MANIFEST_DIR"), filename);

            image::Handle::from_path(path)
        };

        cache.insert((pai_num, angle), handle.clone());
        handle
    })
}

fn get_tile_name(pai_num: u32) -> String {
    if pai_num < 9 {
        return format!("man{}", pai_num + 1);
    }
    if pai_num < 18 {
        return format!("pin{}", pai_num - 8);
    }
    if pai_num < 27 {
        return format!("sou{}", pai_num - 17);
    }
    if pai_num < 34 {
        let zihai = ["ton", "nan", "sha", "pei", "haku", "hatu", "tyun"];
        return zihai[(pai_num - 27) as usize].to_string();
    }
    "ura".to_string()
}
