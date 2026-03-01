#![allow(unused_imports)]

use ai_bridge::bindings::{
    MJITehai, MJMI_GETDORA, MJMI_GETKAWA, MJMI_GETTEHAI, MJPI_CREATEINSTANCE, MJPI_INITIALIZE,
    MJPI_SUTEHAI, MJPI_YOURNAME,
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

// Windows (32-bit/64-bit) expects system/stdcall for WINAPI callbacks.
type MJSendMessage = extern "system" fn(*const c_void, u32, usize, usize) -> usize;

static mut MESSAGE_FUNC: Option<MJSendMessage> = None;
// Global state to avoid Box and avoid extending MahjongAIState beyond host's fixed buffer size (legacy compatibility).
static mut G_STATE: Option<GameStateT> = None;

unsafe fn sync_game_state(
    inst: *mut MahjongAIState,
    callback: MJSendMessage,
) -> anyhow::Result<()> {
    let state_ref = &mut *inst;

    if G_STATE.is_none() {
        G_STATE = Some(GameStateT::default());
    }
    let game_state = G_STATE.as_mut().unwrap();

    // Set basic info
    game_state.bakaze = state_ref.kyoku / 4;
    game_state.teban = state_ref.cha;

    // 1. Get Tehai (My Hand)
    // Use MJITehai1 to ensure sufficient buffer size (372 bytes) as some hosts write extended data.
    // MJITehai (148 bytes) causes stack corruption if host writes MJITehai1.
    let mut tehai_struct: ai_bridge::bindings::MJITehai1 = std::mem::zeroed();

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
        let p1 = MentsuPaiT {
            pai_num: pai,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p2 = MentsuPaiT {
            pai_num: pai,
            id: 1,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p3 = MentsuPaiT {
            pai_num: pai,
            id: 2,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p4 = MentsuPaiT {
            pai_num: pai,
            id: 3,
            flag: MentsuFlag::FLAG_NONE,
        };

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
        let p1 = MentsuPaiT {
            pai_num: pai,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p2 = MentsuPaiT {
            pai_num: pai,
            id: 1,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p3 = MentsuPaiT {
            pai_num: pai,
            id: 2,
            flag: MentsuFlag::FLAG_NONE,
        };
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
        let p1 = MentsuPaiT {
            pai_num: pai,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p2 = MentsuPaiT {
            pai_num: pai + 1,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p3 = MentsuPaiT {
            pai_num: pai + 2,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
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
        let p1 = MentsuPaiT {
            pai_num: pai,
            id: 0,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p2 = MentsuPaiT {
            pai_num: pai,
            id: 1,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p3 = MentsuPaiT {
            pai_num: pai,
            id: 2,
            flag: MentsuFlag::FLAG_NONE,
        };
        let p4 = MentsuPaiT {
            pai_num: pai,
            id: 3,
            flag: MentsuFlag::FLAG_NONE,
        };

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
        ) as u32;

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
    ) as u32;

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

    Ok(())
}

#[no_mangle]
pub extern "system" fn MJPInterfaceFunc(
    inst: *mut MahjongAIState,
    message: u32,
    param1: usize,
    param2: usize,
) -> u32 {
    let name: &'static str = "MahjongAI Type4 Rust\0";
    let name_ptr = name.as_ptr();

    use mahjong_ai::evaluator::eval_sutehai;

    let _ = param1; // Suppress unused warning

    // Check message value directly
    match message {
        MJPI_CREATEINSTANCE => std::mem::size_of::<MahjongAIState>() as u32,
        MJPI_INITIALIZE => {
            unsafe {
                if !inst.is_null() {
                    // Initialize memory space given by host
                    // Host allocates size returned by MJPI_CREATEINSTANCE (288 bytes)
                    // We must initialize it to avoid garbage values if host doesn't write to it immediately
                    // or if test app allocates without init.
                    let state = &mut *inst;
                    state.te_cnt = [0; 34];
                    state.sute_cnt = [0; 34];
                    state.kyoku = 0;
                    state.cha = 0;
                    state.kaze = 0;
                    state.tsumohai = -1; // No tile drawn yet
                }

                if G_STATE.is_none() {
                    G_STATE = Some(GameStateT::default());
                }
                MESSAGE_FUNC = Some(std::mem::transmute(param2));
            }
            0
        }
        MJPI_SUTEHAI => {
            unsafe {
                if let Some(func) = MESSAGE_FUNC {
                    match sync_game_state(inst, func) {
                        Ok(_) => {
                            if let Some(state) = &G_STATE {
                                match eval_sutehai(state) {
                                    Ok((pai, _score)) => {
                                        let player = &state.players[state.teban as usize];
                                        // Try to find the tile in hand (tsumohai or tehai)
                                        // Prioritize Tsumogiri (index 13) if tsumohai matches
                                        if player.is_tsumo
                                            && player.tsumohai.pai_num as usize == pai
                                        {
                                            return consts::MJPIR_SUTEHAI | 13;
                                        }

                                        // Find in tehai
                                        for i in 0..player.tehai_len as usize {
                                            if player.tehai[i].pai_num as usize == pai {
                                                return consts::MJPIR_SUTEHAI | (i as u32);
                                            }
                                        }

                                        // Fallback: If not found (shouldn't happen), try tsumohai index 13 just in case
                                        return consts::MJPIR_SUTEHAI | 13;
                                    }
                                    Err(_) => {
                                        // Fallback
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            consts::MJPIR_SUTEHAI | 13
        }
        MJPI_YOURNAME => name_ptr as u32,
        MJPI_DESTROY => {
            unsafe {
                G_STATE = None;
            }
            0
        }
        _ => 0,
    }
}
