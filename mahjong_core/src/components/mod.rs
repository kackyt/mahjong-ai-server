pub mod world;

use crate::mahjong_generated::open_mahjong::{PaiT, MentsuT};

pub struct Hand {
    pub tiles: Vec<PaiT>,
    pub tsumohai: Option<PaiT>,
    pub is_tsumo: bool,
}

pub struct DiscardPile {
    pub tiles: Vec<PaiT>,
}

pub struct Fulo {
    pub mentsu: Vec<MentsuT>,
}

pub struct Score {
    pub score: i32,
}

pub struct Wind {
    pub wind: u32,
}
