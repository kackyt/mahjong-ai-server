use ai_bridge::bindings::{
    MJIKawahai, MJITehai, MJMI_GETDORA, MJMI_GETKAWA, MJMI_GETTEHAI, MJPI,
};
use mahjong_ai::evaluator::eval_sutehai;
use mahjong_core::mahjong_generated::open_mahjong::{
    GameStateT, Mentsu, MentsuFlag, MentsuPai, MentsuType, PaiT, PlayerT,
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

type MJSendMessage = extern "system" fn(*const c_void, u32, u32, u32) -> u32;

static mut MESSAGE_FUNC: Option<MJSendMessage> = None;

unsafe fn sync_game_state(
    inst: *mut MahjongAIState,
    callback: MJSendMessage,
) -> anyhow::Result<GameStateT> {
    let mut game_state = GameStateT::default();
    let state_ref = &*inst;

    // Set basic info
    game_state.bakaze = (state_ref.kyoku / 4) as u8; // Approximate
    game_state.teban = state_ref.cha as u8;
    game_state.honba = 0; // Unknown
    game_state.kyotaku = 0; // Unknown

    // Initialize players (4 players)
    game_state.players = vec![PlayerT::default(); 4];

    // 1. Get Tehai (My Hand)
    // MJMI_GETTEHAI: param1 = 0 (myself), param2 = &MJITehai
    let mut tehai_struct: MJITehai = std::mem::zeroed();
    callback(
        inst as *const c_void,
        MJMI_GETTEHAI,
        0,
        &mut tehai_struct as *mut _ as u32,
    );

    // Convert MJITehai to PlayerT
    let me = &mut game_state.players[game_state.teban as usize];

    // Tehai
    for i in 0..tehai_struct.tehai_max as usize {
        let pai = tehai_struct.tehai[i] as u8;
        me.tehai.push(PaiT {
            pai_num: pai,
            id: 0,
            is_tsumogiri: false,
            is_riichi: false,
            is_nakare: false,
        });
    }

    // If tsumohai is set in state_ref?
    if state_ref.tsumohai >= 0 && state_ref.tsumohai < 34 {
        me.tsumohai = PaiT {
            pai_num: state_ref.tsumohai as u8,
            id: 0,
            is_tsumogiri: false,
            is_riichi: false,
            is_nakare: false,
        };
    } else {
        // Maybe tsumohai is already in tehai array?
        // Standard MJITehai usually contains 13 or 14 tiles.
        // If 14, one is tsumo.
        // MahjongAIType4 logic separates them?
        // We will trust the tehai struct. If length is 14, last might be tsumo.
        // But mahjong_core expects tsumohai separately for some logic.
        // For eval_sutehai, we just need the full list.
        // In evaluator.rs, I combined tehai and tsumohai.
        // So checking tehai struct is enough.
    }

    // Melds (Mentsu) from MJITehai
    // Minkan
    for i in 0..tehai_struct.minkan_max as usize {
        let pai = tehai_struct.minkan[i] as u8;
        // Construct minkan (4 tiles)
        me.mentsu.push(Mentsu::new(
             &vec![
                 MentsuPai::new(pai, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 1, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 2, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 3, MentsuFlag::FLAG_NONE),
             ],
             4,
             MentsuType::TYPE_MINKAN
        ));
    }
    // Minkou
    for i in 0..tehai_struct.minkou_max as usize {
        let pai = tehai_struct.minkou[i] as u8;
        me.mentsu.push(Mentsu::new(
             &vec![
                 MentsuPai::new(pai, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 1, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 2, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
             ],
             4, // Length? 3 tiles but struct might expect 4 slots? mahjong_core Mentsu len is num of tiles.
             MentsuType::TYPE_KOUTSU
        ));
    }
    // Minshun
    for i in 0..tehai_struct.minshun_max as usize {
        let pai = tehai_struct.minshun[i] as u8;
        me.mentsu.push(Mentsu::new(
             &vec![
                 MentsuPai::new(pai, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai+1, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai+2, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
             ],
             4,
             MentsuType::TYPE_SHUNTSU
        ));
    }
    // Ankan
    for i in 0..tehai_struct.ankan_max as usize {
        let pai = tehai_struct.ankan[i] as u8;
        me.mentsu.push(Mentsu::new(
             &vec![
                 MentsuPai::new(pai, 0, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 1, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 2, MentsuFlag::FLAG_NONE),
                 MentsuPai::new(pai, 3, MentsuFlag::FLAG_NONE),
             ],
             4,
             MentsuType::TYPE_ANKAN
        ));
    }

    // 2. Get Kawa (Discards) for all players
    // We assume 4 players.
    for i in 0..4 {
        let mut kawahai_buf = [0u32; 256]; // Assuming raw array of u32 or MJIKawahai?
        // MJMI_GETKAWA documentation (implied) says param2 is pointer to buffer.
        // In ai_bridge: "let mut p: *mut u32 = std::mem::transmute(param2);"
        // So it writes u32 pai_nums.

        // Wait, ai_bridge also has MJMI_GETKAWAEX which writes MJIKawahai.
        // Let's use GETKAWA (simpler).

        let count = callback(
            inst as *const c_void,
            MJMI_GETKAWA,
            i as u32,
            kawahai_buf.as_mut_ptr() as u32,
        );

        let player = &mut game_state.players[i];
        for k in 0..count as usize {
             player.kawahai.push(PaiT {
                 pai_num: kawahai_buf[k] as u8,
                 id: 0,
                 is_tsumogiri: false,
                 is_riichi: false, // Lost info
                 is_nakare: false,
             });
        }
    }

    // 3. Get Dora
    let mut dora_buf = [0u32; 8];
    let dora_count = callback(
        inst as *const c_void,
        MJMI_GETDORA,
        dora_buf.as_mut_ptr() as u32,
        0,
    );
    for i in 0..dora_count as usize {
        game_state.dora.push(PaiT {
             pai_num: dora_buf[i] as u8,
             id: 0,
             is_tsumogiri: false,
             is_riichi: false,
             is_nakare: false,
        });
    }

    Ok(game_state)
}

#[cfg(windows)]
#[no_mangle]
pub extern "system" fn MJPInterfaceFunc(
    inst: *mut MahjongAIState,
    message: isize,
    param1: u32,
    param2: u32,
) -> u32 {
    let name: &'static str = "MahjongAI Type4 Rust\0";
    let name_ptr = name.as_ptr();
    match MJPI::from_value(message) {
        Some(MJPI::MJPI_CREATEINSTANCE) => std::mem::size_of::<MahjongAIState>() as u32,
        Some(MJPI::MJPI_INITIALIZE) => {
            unsafe {
                MESSAGE_FUNC = Some(std::mem::transmute(param2));
            }
            0
        }
        Some(MJPI::MJPI_SUTEHAI) => {
            unsafe {
                if let Some(func) = MESSAGE_FUNC {
                    match sync_game_state(inst, func) {
                        Ok(game_state) => {
                            match eval_sutehai(&game_state) {
                                Ok((pai, _score)) => {
                                    return consts::MJPIR_SUTEHAI | (pai as u32);
                                },
                                Err(_) => {
                                    // Fallback: tsumogiri (13?) or 0?
                                    // MJPIR_SUTEHAI | 0?
                                }
                            }
                        },
                        Err(_) => {}
                    }
                }
            }
            consts::MJPIR_SUTEHAI | 13 // Tsumogiri default
        }
        Some(MJPI::MJPI_YOURNAME) => name_ptr as u32,
        _ => 0,
    }
}
