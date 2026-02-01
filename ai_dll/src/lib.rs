#![allow(unused_imports)]

use ai_bridge::bindings::{
    MJMI_GETDORA, MJMI_GETKAWA, MJMI_GETTEHAI, MJITehai,
    MJPI_CREATEINSTANCE, MJPI_INITIALIZE, MJPI_SUTEHAI, MJPI_YOURNAME,
};
use mahjong_core::mahjong_generated::open_mahjong::{
    GameStateT, MentsuFlag, MentsuPaiT, MentsuT, MentsuType, PaiT,
};
use std::ffi::c_void;

mod consts;

#[repr(C)]
pub struct MahjongAIState {
    te_cnt: [u32; 34],
    sute_cnt: [u32; 34],
    kyoku: u32,
    cha: u32,
    kaze: u32,
    tsumohai: i32,
}

// Updated to use usize for pointers/params to support 64-bit and match ai_bridge
type MJSendMessage = extern "system" fn(*const c_void, u32, usize, usize) -> usize;

static mut MESSAGE_FUNC: Option<MJSendMessage> = None;

unsafe fn sync_game_state(
    inst: *mut MahjongAIState,
    callback: MJSendMessage,
) -> anyhow::Result<GameStateT> {
    let mut game_state = GameStateT::default();
    let state_ref = &*inst;

    // Set basic info
    game_state.bakaze = state_ref.kyoku / 4;
    game_state.teban = state_ref.cha;

    // 1. Get Tehai (My Hand)
    let mut tehai_struct: MJITehai = std::mem::zeroed();
    callback(
        inst as *const c_void,
        MJMI_GETTEHAI,
        0,
        &mut tehai_struct as *mut _ as usize,
    );

    let me_idx = game_state.teban as usize;

    // Tehai
    for i in 0..tehai_struct.tehai_max as usize {
        let pai = tehai_struct.tehai[i] as u8;
        if i < 13 {
             game_state.players[me_idx].tehai[i] = PaiT {
                pai_num: pai,
                id: 0,
                is_tsumogiri: false,
                is_riichi: false,
                is_nakare: false,
            };
        }
    }
    game_state.players[me_idx].tehai_len = tehai_struct.tehai_max;

    if state_ref.tsumohai >= 0 && state_ref.tsumohai < 34 {
        game_state.players[me_idx].tsumohai = PaiT {
            pai_num: state_ref.tsumohai as u8,
            id: 0,
            is_tsumogiri: false,
            is_riichi: false,
            is_nakare: false,
        };
        game_state.players[me_idx].is_tsumo = true;
    }

    // Melds
    let mut mentsu_idx = 0;
    // Minkan
    for i in 0..tehai_struct.minkan_max as usize {
        let pai = tehai_struct.minkan[i] as u8;
        let p1 = MentsuPaiT { pai_num: pai, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p2 = MentsuPaiT { pai_num: pai, id: 1, flag: MentsuFlag::FLAG_NONE };
        let p3 = MentsuPaiT { pai_num: pai, id: 2, flag: MentsuFlag::FLAG_NONE };
        let p4 = MentsuPaiT { pai_num: pai, id: 3, flag: MentsuFlag::FLAG_NONE };

        if mentsu_idx < 4 {
            game_state.players[me_idx].mentsu[mentsu_idx] = MentsuT {
                 pai_list: [p1, p2, p3, p4],
                 pai_len: 4,
                 mentsu_type: MentsuType::TYPE_MINKAN,
            };
            mentsu_idx += 1;
        }
    }
    // Minkou
    for i in 0..tehai_struct.minkou_max as usize {
        let pai = tehai_struct.minkou[i] as u8;
        let p1 = MentsuPaiT { pai_num: pai, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p2 = MentsuPaiT { pai_num: pai, id: 1, flag: MentsuFlag::FLAG_NONE };
        let p3 = MentsuPaiT { pai_num: pai, id: 2, flag: MentsuFlag::FLAG_NONE };
        let p4 = MentsuPaiT::default();

        if mentsu_idx < 4 {
            game_state.players[me_idx].mentsu[mentsu_idx] = MentsuT {
                 pai_list: [p1, p2, p3, p4],
                 pai_len: 3,
                 mentsu_type: MentsuType::TYPE_KOUTSU,
            };
            mentsu_idx += 1;
        }
    }
    // Minshun
    for i in 0..tehai_struct.minshun_max as usize {
        let pai = tehai_struct.minshun[i] as u8;
        let p1 = MentsuPaiT { pai_num: pai, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p2 = MentsuPaiT { pai_num: pai+1, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p3 = MentsuPaiT { pai_num: pai+2, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p4 = MentsuPaiT::default();

        if mentsu_idx < 4 {
            game_state.players[me_idx].mentsu[mentsu_idx] = MentsuT {
                 pai_list: [p1, p2, p3, p4],
                 pai_len: 3,
                 mentsu_type: MentsuType::TYPE_SHUNTSU,
            };
            mentsu_idx += 1;
        }
    }
    // Ankan
    for i in 0..tehai_struct.ankan_max as usize {
        let pai = tehai_struct.ankan[i] as u8;
        let p1 = MentsuPaiT { pai_num: pai, id: 0, flag: MentsuFlag::FLAG_NONE };
        let p2 = MentsuPaiT { pai_num: pai, id: 1, flag: MentsuFlag::FLAG_NONE };
        let p3 = MentsuPaiT { pai_num: pai, id: 2, flag: MentsuFlag::FLAG_NONE };
        let p4 = MentsuPaiT { pai_num: pai, id: 3, flag: MentsuFlag::FLAG_NONE };

        if mentsu_idx < 4 {
            game_state.players[me_idx].mentsu[mentsu_idx] = MentsuT {
                 pai_list: [p1, p2, p3, p4],
                 pai_len: 4,
                 mentsu_type: MentsuType::TYPE_ANKAN,
            };
            mentsu_idx += 1;
        }
    }
    game_state.players[me_idx].mentsu_len = mentsu_idx as u32;

    // 2. Get Kawa
    for i in 0..4 {
        let mut kawahai_buf = [0u32; 256];
        let count = callback(
            inst as *const c_void,
            MJMI_GETKAWA,
            i as usize,
            kawahai_buf.as_mut_ptr() as usize,
        );

        let player = &mut game_state.players[i];
        for k in 0..count as usize {
             if k < 20 {
                 player.kawahai[k] = PaiT {
                     pai_num: kawahai_buf[k] as u8,
                     id: 0,
                     is_tsumogiri: false,
                     is_riichi: false,
                     is_nakare: false,
                 };
             }
        }
        player.kawahai_len = std::cmp::min(count, 20) as u32;
    }

    // 3. Get Dora
    let mut dora_buf = [0u32; 8];
    let dora_count = callback(
        inst as *const c_void,
        MJMI_GETDORA,
        dora_buf.as_mut_ptr() as usize,
        0,
    );

    for i in 0..dora_count as usize {
        if i < 8 {
            game_state.taku.n5[i] = PaiT {
                 pai_num: dora_buf[i] as u8,
                 id: 0,
                 is_tsumogiri: false,
                 is_riichi: false,
                 is_nakare: false,
            };
        }
    }
    game_state.dora_len = dora_count as u32;

    Ok(game_state)
}

#[cfg(windows)]
#[no_mangle]
pub extern "system" fn MJPInterfaceFunc(
    inst: *mut MahjongAIState,
    message: usize,
    param1: usize,
    param2: usize,
) -> usize {
    let name: &'static str = "MahjongAI Type4 Rust\0";
    let name_ptr = name.as_ptr();

    use mahjong_ai::evaluator::eval_sutehai;

    // Check message value directly
    match message as u32 {
        MJPI_CREATEINSTANCE => std::mem::size_of::<MahjongAIState>() as usize,
        MJPI_INITIALIZE => {
            unsafe {
                MESSAGE_FUNC = Some(std::mem::transmute(param2));
            }
            0
        }
        MJPI_SUTEHAI => {
            unsafe {
                if let Some(func) = MESSAGE_FUNC {
                    match sync_game_state(inst, func) {
                        Ok(game_state) => {
                            match eval_sutehai(&game_state) {
                                Ok((pai, _score)) => {
                                    return (consts::MJPIR_SUTEHAI | (pai as u32)) as usize;
                                },
                                Err(_) => {
                                    // Fallback
                                }
                            }
                        },
                        Err(_) => {}
                    }
                }
            }
            (consts::MJPIR_SUTEHAI | 13) as usize
        }
        MJPI_YOURNAME => name_ptr as usize,
        _ => 0,
    }
}
