use mahjong_core::{
    agari::{add_machi_to_mentsu, AgariBehavior}, fbs_utils::GetTsumo, mahjong_generated::open_mahjong::{
        GameStateT, Mentsu, MentsuFlag, MentsuPai, MentsuType, Pai, PaiT,
    }, shanten::{all_of_mentsu, PaiState}
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, ffi::{c_char, c_void, CStr}};

use crate::bindings::{
    MJIKawahai, MJITehai, MJITehai1, MJMI_FUKIDASHI, MJMI_GETAGARITEN, MJMI_GETDORA, MJMI_GETHAIREMAIN, MJMI_GETKAWA, MJMI_GETKAWAEX, MJMI_GETMACHI, MJMI_GETRULE, MJMI_GETSCORE, MJMI_GETTEHAI, MJMI_GETVERSION, MJMI_GETVISIBLEHAIS, MJMI_SETSTRUCTTYPE, MJRL_77MANGAN, MJRL_AKA5, MJRL_AKA5S, MJRL_BUTTOBI, MJRL_DBLRONCHONBO, MJRL_DORAPLUS, MJRL_FURITENREACH, MJRL_KANINREACH, MJRL_KANSAKI, MJRL_KARATEN, MJRL_KUINAOSHI, MJRL_KUITAN, MJRL_MOCHITEN, MJRL_NANNYU, MJRL_NANNYU_SCORE, MJRL_NOTENOYANAGARE, MJRL_PAO, MJRL_PINZUMO, MJRL_RYANSHIBA, MJRL_SCORE0REACH, MJRL_SHANYU, MJRL_SHANYU_SCORE, MJRL_TOPOYAAGARIEND, MJRL_URADORA, MJRL_WAREME, MJR_NOTCARED
};

extern crate libc;

// スレッドセーフではない
pub static mut G_STATE: Lazy<GameStateT> = Lazy::new(Default::default);
pub static mut G_STRUCTURE_TYPE: Lazy<HashMap<*mut c_void, usize>> = Lazy::new(HashMap::new);

fn get_rule(state: &GameStateT, idx: u32) -> u32 {
    match idx {
        MJRL_KUITAN => state.rule.enable_kuitan as u32,
        MJRL_KANSAKI => state.rule.enable_kansaki as u32,
        MJRL_PAO => state.rule.enable_pao as u32,
        MJRL_MOCHITEN => state.rule.initial_score,
        MJRL_BUTTOBI => state.rule.enable_tobi as u32,
        MJRL_WAREME => state.rule.enable_wareme as u32,
        MJRL_AKA5 => if state.rule.aka_type != 0 { 1 } else { 0 },
        MJRL_AKA5S => state.rule.aka_type as u32,
        MJRL_SHANYU => {
            if state.rule.shanyu_score < 0 { 1 }
            else if state.rule.shanyu_score == 0 { 0 }
            else { 2 }
        },
        MJRL_SHANYU_SCORE => state.rule.shanyu_score as u32,
        MJRL_NANNYU => {
            if state.rule.nannyu_score < 0 { 1 }
            else if state.rule.nannyu_score == 0 { 0 }
            else { 2 }
        },
        MJRL_NANNYU_SCORE => state.rule.nannyu_score as u32,
        MJRL_KUINAOSHI => state.rule.enable_kuinaoshi as u32,
        MJRL_URADORA => state.rule.uradora_type as u32,
        MJRL_SCORE0REACH => state.rule.enable_minus_riichi as u32,
        MJRL_RYANSHIBA => state.rule.enable_ryanhan_shibari as u32,
        MJRL_DORAPLUS => 0,
        MJRL_FURITENREACH => state.rule.furiten_riichi_type,
        MJRL_KARATEN => state.rule.enable_keiten as u32,
        MJRL_PINZUMO => 1,
        MJRL_NOTENOYANAGARE => state.rule.oyanagare_type as u32,
        MJRL_KANINREACH => state.rule.kan_in_riichi as u32,
        MJRL_TOPOYAAGARIEND => state.rule.enable_agariyame as u32,
        MJRL_77MANGAN => state.rule.enable_kiriage as u32,
        MJRL_DBLRONCHONBO => 0,
        _ => MJR_NOTCARED,
    }
}

unsafe fn mjsend_message_impl(
    inst: *mut c_void,
    message: usize,
    param1: usize,
    param2: usize,
) -> usize {
    let taku: &GameStateT = &G_STATE;

    println!(
        "message flag = {:08x} param1 = {:08x} param2 = {:08x}",
        message, param1, param2
    );

    match message as u32 {
        MJMI_GETRULE => get_rule(taku, param1 as u32).try_into().unwrap(),
        MJMI_GETTEHAI => {
            let map = &G_STRUCTURE_TYPE;

            let v = map.get(&inst);

            if let Some(typ) = v {
                if *typ == 1 {
                    let tehai: &mut MJITehai1 = std::mem::transmute(param2);

                    if param1 == 0 {
                        let player = &taku.players[taku.teban as usize];
        
                        for i in 0..player.tehai_len as usize {
                            tehai.tehai[i] = player.tehai[i].pai_num as u32;
                        }
                        tehai.tehai_max = player.tehai_len as u32;
                        tehai.minkan_max = 0;
                        tehai.minkou_max = 0;
                        tehai.minshun_max = 0;
                        tehai.ankan_max = 0;
                    }
        
                    return 1;        
                }
            }
            let tehai: &mut MJITehai = std::mem::transmute(param2);

            if param1 == 0 {
                let player = &taku.players[taku.teban as usize];

                for i in 0..player.tehai_len as usize {
                    tehai.tehai[i] = player.tehai[i].pai_num as u32;
                }
                tehai.tehai_max = player.tehai_len as u32;
                tehai.minkan_max = 0;
                tehai.minkou_max = 0;
                tehai.minshun_max = 0;
                tehai.ankan_max = 0;
            }

            1
        }
        MJMI_GETMACHI => {
            let p: *const MJITehai = std::mem::transmute(param1);
            let mut p2: *mut u32 = std::mem::transmute(param2);

            let mut pstate: PaiState;
            let mut v_fulo: Vec<Mentsu> = Vec::new();
            let mut num = 0;

            if p == std::ptr::null() {
                let player = &taku.players[taku.teban as usize];

                pstate = PaiState::from(&player.tehai[0..player.tehai_len as usize]);

                v_fulo = player.mentsu[0..player.mentsu_len as usize]
                    .iter()
                    .map(|x| x.pack())
                    .collect();
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
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_MINKAN,
                    ));
                }
                for i in 0..(*p).minkou_max as usize {
                    let n = (*p).minkou[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_KOUTSU,
                    ));
                }
                for i in 0..(*p).minshun_max as usize {
                    let n = (*p).minshun[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n + 1, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n + 2, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_SHUNTSU,
                    ));
                }
                for i in 0..(*p).ankan_max as usize {
                    let n = (*p).ankan[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_ANKAN,
                    ));
                }
            }

            for i in 0..34 {
                if i >= 27 {
                    pstate.hai_count_z[i - 27] += 1;
                } else if i >= 18 {
                    pstate.hai_count_p[i - 18] += 1;
                } else if i >= 9 {
                    pstate.hai_count_s[i - 9] += 1;
                } else {
                    pstate.hai_count_m[i] += 1;
                }
                let all_mentsu = all_of_mentsu(&mut pstate, v_fulo.len());
                if i >= 27 {
                    pstate.hai_count_z[i - 27] -= 1;
                } else if i >= 18 {
                    pstate.hai_count_p[i - 18] -= 1;
                } else if i >= 9 {
                    pstate.hai_count_s[i - 9] -= 1;
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
            let agari_pai = Pai::new(param2 as u8, 0, false, false, false);

            let mut pstate: PaiState;
            let mut v_fulo: Vec<Mentsu> = Vec::new();

            if p == std::ptr::null_mut() {
                let player = &taku.players[taku.teban as usize];

                pstate = PaiState::from(&player.tehai[0..player.tehai_len as usize]);

                v_fulo = player.mentsu[0..player.mentsu_len as usize]
                    .iter()
                    .map(|x| x.pack())
                    .collect();
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
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_MINKAN,
                    ));
                }
                for i in 0..(*p).minkou_max as usize {
                    let n = (*p).minkou[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_KOUTSU,
                    ));
                }
                for i in 0..(*p).minshun_max as usize {
                    let n = (*p).minshun[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n + 1, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n + 2, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(0, 0, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_SHUNTSU,
                    ));
                }
                for i in 0..(*p).ankan_max as usize {
                    let n = (*p).ankan[i] as u8;
                    v_fulo.push(Mentsu::new(
                        &[
                            MentsuPai::new(n, 0, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 1, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 2, MentsuFlag::FLAG_NONE),
                            MentsuPai::new(n, 3, MentsuFlag::FLAG_NONE),
                        ],
                        4,
                        MentsuType::TYPE_ANKAN,
                    ));
                }
            }

            pstate.append(&agari_pai.unpack());

            let all_mentsu = all_of_mentsu(&mut pstate, v_fulo.len());
            let all_of_mentsu_with_agari = add_machi_to_mentsu(&all_mentsu, &agari_pai);

            let result =
                taku.get_best_agari(taku.teban as usize, &all_of_mentsu_with_agari, &v_fulo, 0);

            if let Ok(agari) = result {
                agari.score.try_into().unwrap()
            } else {
                0
            }
        }
        MJMI_GETKAWA => {
            let idx = (param1 & 0xFFFF) as usize;
            let player = &taku.players[idx];
            let mut p: *mut u32 = std::mem::transmute(param2);

            for i in 0..player.kawahai_len as usize {
                *p = player.kawahai[i].pai_num as u32;
                p = p.add(1);
            }

            player.kawahai_len.try_into().unwrap()
        }
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

            player.kawahai_len.try_into().unwrap()
        }
        MJMI_FUKIDASHI => {
            let p_cstr: *const c_char = std::mem::transmute(param1);
            let c_str: &CStr = CStr::from_ptr(p_cstr);
            println!("fukidashi");

            match c_str.to_str() {
                Ok(str_slice) => {
                    // println!("{}", str_slice);
                }
                _ => {}
            }

            0
        }
        MJMI_GETDORA => {
            let mut p: *mut u32 = std::mem::transmute(param1);
            let dora = taku.get_dora();
            for i in 0..dora.len() as usize {
                *p = dora[i].pai_num as u32;
                p = p.add(1);
            }
            dora.len().try_into().unwrap()
        }
        MJMI_GETHAIREMAIN => taku.remain().try_into().unwrap(),
        MJMI_GETVISIBLEHAIS => {
            let player = &taku.players[taku.teban as usize];

            player.tehai[0..player.tehai_len as usize]
                .into_iter()
                .chain(player.kawahai[0..player.kawahai_len as usize].into_iter())
                .chain(taku.get_dora().into_iter())
                .chain(player.get_tsumohai().iter())
                .filter(|x| x.pai_num == param1 as u8)
                .count()
        }
        MJMI_SETSTRUCTTYPE => {
            let map = &mut G_STRUCTURE_TYPE;
            map.insert(inst, param1);
            0
        },
        MJMI_GETSCORE => 25000,
        MJMI_GETVERSION => 12,
        _ => 0,
    }
}

#[cfg(target_os = "linux")]
pub unsafe extern "stdcall" fn mjsend_message(
    inst: *mut c_void,
    message: usize,
    param1: usize,
    param2: usize,
) -> usize {
    mjsend_message_impl(inst, message, param1, param2)
}

#[cfg(not(target_os = "linux"))]
pub unsafe fn mjsend_message(
    inst: *mut c_void,
    message: usize,
    param1: usize,
    param2: usize,
) -> usize {
    mjsend_message_impl(inst, message, param1, param2)
}
