use iced::widget::image;
use std::cell::RefCell;
use std::collections::HashMap;

/// 牌画像の識別子を表すNewtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileImageId(pub u32);

/// 相手の手牌など、牌の裏面を表示するときに使う定数
pub const BACK_TILE_NUM: TileImageId = TileImageId(99);
/// UIレイアウト調整用の透明プレースホルダー定数
pub const TRANSPARENT_TILE_NUM: TileImageId = TileImageId(100);

thread_local! {
    static CACHE: RefCell<HashMap<(TileImageId, u16), image::Handle>> = RefCell::new(HashMap::new());
}

/// 指定された識別子と角度に対応する画像ハンドルを取得する
pub fn get(id: TileImageId, angle: u16) -> image::Handle {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(handle) = cache.get(&(id, angle)) {
            return handle.clone();
        }

        let handle = if id == BACK_TILE_NUM {
            // 相手の手牌は裏面（ura.gif）で表示する
            image::Handle::from_path(format!(
                "{}/images/haiga/ura.gif",
                env!("CARGO_MANIFEST_DIR")
            ))
        } else if id == TRANSPARENT_TILE_NUM {
            // UIレイアウト調整用の透明プレースホルダー
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

            let name = get_tile_name(id);
            let filename = format!("{}{}.gif", prefix, name);
            let path = format!("{}/images/haiga/{}", env!("CARGO_MANIFEST_DIR"), filename);

            image::Handle::from_path(path)
        };

        cache.insert((id, angle), handle.clone());
        handle
    })
}

fn get_tile_name(id: TileImageId) -> String {
    let num = id.0;
    if num < 9 {
        return format!("man{}", num + 1);
    }
    if num < 18 {
        return format!("pin{}", num - 8);
    }
    if num < 27 {
        return format!("sou{}", num - 17);
    }
    if num < 34 {
        let zihai = ["ton", "nan", "sha", "pei", "haku", "hatu", "tyun"];
        return zihai[(num - 27) as usize].to_string();
    }
    "ura".to_string()
}
