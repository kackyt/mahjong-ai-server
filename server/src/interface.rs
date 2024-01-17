use std::ffi::{c_void, c_char, CStr};
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
static mut G_TAKU: Lazy<Taku> = Lazy::new(|| Default::default());

pub unsafe extern "stdcall" fn mjsend_message(inst: *mut c_void, message: u32, param1: u32, param2: u32) -> u32
{
    let taku: &mut Taku = &mut G_TAKU;

    println!("message flag = {:08x} param1 = {:08x} param2 = {:08x}", message, param1, param2);

    match message {
        MJMI_GETTEHAI => {
            let tehai: &mut MJITehai = std::mem::transmute(param2);

            if param1 == 0 {

            }

            1
        },
        MJMI_GETMACHI => {
            let p: *mut libc::c_void = std::mem::transmute(param2);

            libc::memset(p, 0, 34 * 4);

            0
        }
        MJMI_GETAGARITEN => {
            // dumyy
            if param1 != 0 {
                100
            } else {
                0
            }
        },
        MJMI_GETKAWA => {
            let idx = (param1 & 0xFFFF) as usize;
            let p: *mut libc::c_void = std::mem::transmute(param2);

            libc::memcpy(
                p,
                &taku.sutehai[idx] as *const Sutehai as *const libc::c_void,
                std::mem::size_of::<Sutehai>()
            );

            taku.sutehai[idx].num as u32
        },
        MJMI_GETKAWAEX => {
            let idx = (param1 & 0xFFFF) as usize;
            let mut p: *mut MJIKawahai = std::mem::transmute(param2);
            let ps = &taku.sutehai[idx];

            for i in 0..ps.num as usize {
                let kawa_ref = &mut *p;

                kawa_ref.hai = ps.hai[i] as u16;
                kawa_ref.state = 0;
                p = p.add(1);
            }

            ps.num as u32
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
