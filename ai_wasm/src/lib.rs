use mahjong_core::{mahjong_generated::open_mahjong::PaiT, shanten::PaiState};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_discard(tehai: &[u8]) -> usize {
    let mut min_shanten = 999;
    let mut best_discard = 0;

    // Iterate over each tile in tehai to try discarding it
    for (i, &discard_pai) in tehai.iter().enumerate() {
        if discard_pai >= 34 {
            continue;
        }

        // Construct a temporary tehai without the discarded tile
        let mut temp_tehai_nums = tehai.to_vec();
        temp_tehai_nums.remove(i);

        // Create PaiT vector for PaiState
        let mut pai_list: Vec<PaiT> = Vec::new();
        for &pai_num in &temp_tehai_nums {
            let mut pai = PaiT::default();
            pai.pai_num = pai_num;
            pai_list.push(pai);
        }

        // Calculate shanten
        let mut state = PaiState::from(&pai_list);
        let shanten = state.get_shanten(0);

        // Update best discard if this one is better
        if shanten < min_shanten {
            min_shanten = shanten;
            best_discard = i;
        }
    }

    best_discard
}

#[wasm_bindgen]
pub fn get_shanten(tehai: &[u8]) -> i32 {
    let mut pai_list: Vec<PaiT> = Vec::new();
    for &pai_num in tehai {
        let mut pai = PaiT::default();
        pai.pai_num = pai_num;
        pai_list.push(pai);
    }
    let mut state = PaiState::from(&pai_list);
    state.get_shanten(0)
}
