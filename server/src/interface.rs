use std::ffi::{c_void, c_char, CStr};
use mahjong_core::{agari::{add_machi_to_mentsu, AgariBehavior}, mahjong_generated::open_mahjong::{GameStateT, MentsuFlag, Mentsu, MentsuPai, MentsuType, Pai, PaiT}, shanten::{all_of_mentsu, PaiState}};
use once_cell::sync::Lazy;

use crate::bindings::{MJMI_GETTEHAI, MJITehai, MJMI_GETVERSION, MJMI_GETSCORE, MJMI_FUKIDASHI, MJMI_GETMACHI, MJMI_GETAGARITEN, MJMI_GETKAWA, MJMI_GETKAWAEX, MJIKawahai};

extern crate libc;

// (仮) 麻雀の状態管理
#[repr(C)]
#[derive(Default)]
struct Sutehai {
    hai: [u32; 18],
    num: i32
}

#[derive(Default)]
struct Taku {
    tehai: [u32; 13],
    sutehai: [Sutehai; 4],
    dora: [u32; 4],
    dora_num: i32,
    tehai_max: i32,
    tsumohai: u32,
    kyoku: i32,
    zikaze: i32
}

// スレッドセーフではない
pub static mut G_STATE: Lazy<GameStateT> = Lazy::new(|| Default::default());

pub unsafe extern "stdcall" fn mjsend_message(inst: *mut c_void, message: u32, param1: u32, param2: u32) -> u32 {
    let taku: &GameStateT = &G_STATE;

    println!("message flag = {:08x} param1 = {:08x} param2 = {:08x}", message, param1, param2);

    match message {
        MJMI_GETTEHAI => {
            let tehai: &mut MJITehai = std::mem::transmute(param2);

            if param1 == 0 {
                let player = &taku.players[taku.teban as usize];

                for i in 0..player.tehai_len as usize {
                    tehai.tehai[i] = player.tehai[i].pai_num as u32;
                    tehai.tehai_max = player.tehai_len as u32;
                }
            }

            1
        },
        MJMI_GETMACHI => {
            let p: *const MJITehai = std::mem::transmute(param1);
            let mut p2: *mut u32 = std::mem::transmute(param2);

            let mut pstate: PaiState;
            let mut v_fulo: Vec<Mentsu> = Vec::new();
            let mut num = 0;

            if p == std::ptr::null_mut() {
                let player = &taku.players[taku.teban as usize];

                pstate = PaiState::from(&player.tehai[0..player.tehai_len as usize]);

                v_fulo = player.mentsu[0..player.mentsu_len as usize].iter().map(|x| x.pack()).collect();
            } else {
                let mut tehai: Vec<PaiT> = Vec::new();

                for i in 0..(*p).tehai_max as usize {
                    tehai.push(PaiT {
                        pai_num: (*p).tehai[i] as u8,
                        id: 0,
                        is_tsumogiri: false,
                        is_riichi: false,
                        is_nakare: false,
                    });
                }

                pstate = PaiState::from(&tehai);

                for i in 0..(*p).minkan_max as usize {
                    let n = (*p).minkan[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_MINKAN));
                }
                for i in 0..(*p).minkou_max as usize {
                    let n = (*p).minkou[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_KOUTSU));
                }
                for i in 0..(*p).minshun_max as usize {
                    let n = (*p).minshun[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n+1, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n+2, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_SHUNTSU));
                }
                for i in 0..(*p).ankan_max as usize {
                    let n = (*p).ankan[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_ANKAN));
                }
            }

            for i in 0..34 {
                if i >= 27 {
                    pstate.hai_count_z[i-27] += 1;
                } else if i >= 18 {
                    pstate.hai_count_p[i-18] += 1;
                } else if i >= 9 {
                    pstate.hai_count_s[i-9] += 1;
                } else {
                    pstate.hai_count_m[i] += 1;
                }
                let all_mentsu = all_of_mentsu(&mut pstate, v_fulo.len());
                if i >= 27 {
                    pstate.hai_count_z[i-27] -= 1;
                } else if i >= 18 {
                    pstate.hai_count_p[i-18] -= 1;
                } else if i >= 9 {
                    pstate.hai_count_s[i-9] -= 1;
                } else {
                    pstate.hai_count_m[i] -= 1;
                }

                if all_mentsu.len() > 0 {
                    *p2 = 1;
                    num += 1;
                } else {
                    *p2 = 0;
                }
                p2 = p2.add(1);
            }

            num
        }
        MJMI_GETAGARITEN => {
            let p: *const MJITehai = std::mem::transmute(param1);
            let agari_pai = Pai::new(
                param2 as u8,
                0,
                false,
                false,
                false);

            let mut pstate : PaiState;
            let mut v_fulo: Vec<Mentsu> = Vec::new();

            if p == std::ptr::null_mut() {
                let player = &taku.players[taku.teban as usize];

                pstate = PaiState::from(&player.tehai[0..player.tehai_len as usize]);

                v_fulo = player.mentsu[0..player.mentsu_len as usize].iter().map(|x| x.pack()).collect();
            } else {
                let mut tehai: Vec<PaiT> = Vec::new();

                for i in 0..(*p).tehai_max as usize {
                    tehai.push(PaiT {
                        pai_num: (*p).tehai[i] as u8,
                        id: 0,
                        is_tsumogiri: false,
                        is_riichi: false,
                        is_nakare: false,
                    });
                }

                pstate = PaiState::from(&tehai);
                for i in 0..(*p).minkan_max as usize {
                    let n = (*p).minkan[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_MINKAN));
                }
                for i in 0..(*p).minkou_max as usize {
                    let n = (*p).minkou[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_KOUTSU));
                }
                for i in 0..(*p).minshun_max as usize {
                    let n = (*p).minshun[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n+1, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n+2, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_SHUNTSU));
                }
                for i in 0..(*p).ankan_max as usize {
                    let n = (*p).ankan[i] as u8;
                    v_fulo.push(Mentsu::new(&[
                        MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                        MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                    ], 4, MentsuType::TYPE_ANKAN));
                }
            }

            let all_mentsu = all_of_mentsu(&mut pstate, v_fulo.len());
            let all_of_mentsu_with_agari = add_machi_to_mentsu(&all_mentsu, &agari_pai);

            let result = taku.get_best_agari(
                taku.teban as usize,
                &all_of_mentsu_with_agari,
                &v_fulo,
                0);

            if let Ok(agari) = result {
                agari.score as u32
            } else {
                0
            }
        },
        MJMI_GETKAWA => {
            let idx = (param1 & 0xFFFF) as usize;
            let player = &taku.players[idx];
            let mut p: *mut u32 = std::mem::transmute(param2);

            for i in 0..player.kawahai_len as usize {
                *p = player.kawahai[i].pai_num as u32;
                p = p.add(1);
            }

            player.kawahai_len as u32
        },
        MJMI_GETKAWAEX => {
            let idx = (param1 & 0xFFFF) as usize;
            let player = &taku.players[idx];
            let mut p: *mut MJIKawahai = std::mem::transmute(param2);

            for i in 0..player.kawahai_len as usize {
                let kawa_ref = &mut *p;

                kawa_ref.hai = player.kawahai[i].pai_num as u16;
                kawa_ref.state = 0;
                p = p.add(1);
            }

            player.kawahai_len as u32
        },
        MJMI_FUKIDASHI => {
            let p_cstr: *const c_char = std::mem::transmute(param1);
            let c_str: &CStr = CStr::from_ptr(p_cstr);

            match c_str.to_str() {
                Ok(str_slice) => {
                    println!("{}", str_slice);
                },
                _ => {}
            }

            0
        },
        MJMI_GETSCORE => 25000,
        MJMI_GETVERSION => 12,
        _ => 0
    }
}
