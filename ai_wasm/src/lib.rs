use mahjong_core::{mahjong_generated::open_mahjong::PaiT, shanten::PaiState};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct GameState {
    tehai: Vec<u8>,
}

#[derive(Serialize)]
struct AIResult {
    discard: Option<usize>,
}

#[wasm_bindgen]
pub fn get_discard(json_state: String) -> String {
    let state: GameState = match serde_json::from_str(&json_state) {
        Ok(s) => s,
        Err(_) => return "{}".to_string(),
    };

    let best_discard = calculate_discard(&state.tehai);

    let result = AIResult {
        discard: best_discard,
    };
    serde_json::to_string(&result).unwrap_or("{}".to_string())
}

fn calculate_discard(tehai: &[u8]) -> Option<usize> {
    let mut min_shanten = 999;
    let mut best_discard = None;

    // Iterate over each tile in tehai to try discarding it
    for (i, &discard_pai) in tehai.iter().enumerate() {
        if discard_pai >= 34 {
            continue;
        }

        // Construct a temporary tehai without the discarded tile using an iterator
        let temp_tehai_nums: Vec<u8> = tehai
            .iter()
            .enumerate()
            .filter_map(|(idx, &pai)| if idx == i { None } else { Some(pai) })
            .collect();

        // Create PaiT vector for PaiState
        let mut pai_list: Vec<PaiT> = Vec::with_capacity(temp_tehai_nums.len());
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
            best_discard = Some(i);
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
