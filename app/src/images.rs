use iced::widget::image;
use std::collections::HashMap;
use std::env;

pub struct ImageCache {
    cache: HashMap<String, image::Handle>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, pai_num: u32, angle: u16, _is_red: bool) -> image::Handle {
        // Red tile logic if needed (passing is_red, but file naming might be different)
        // For now, let's assume standard names or handle 'red' suffix if passed.
        // The original utils::painum2path handled standard lookup.
        // We need rotated variants.
        
        let prefix = match angle {
            90 => "ty",
            180 => "t",
            270 => "y",
            _ => "",
        };

        // Reuse strict logic from utils, but adapted for rotation prefixes
        let name = get_tile_name(pai_num);
        let filename = format!("{}{}.gif", prefix, name);
        let path = format!("{}/images/haiga/{}", env!("CARGO_MANIFEST_DIR"), filename);

        image::Handle::from_path(path)
    }
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
