use std::{env, path::PathBuf, ffi::c_void};

use loadlibrary::{win_dlopen, win_dlsym};
use anyhow::ensure;

use crate::{bindings::{MJEK_RYUKYOKU, MJPIR_TSUMO, MJPI_CREATEINSTANCE, MJPI_ENDGAME, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_ONEXCHANGE, MJPI_SUTEHAI, MJST_INKYOKU}, interface::mjsend_message};

extern crate libc;

mod bindings;
mod interface;

type MJPInterfaceFuncP = extern "stdcall" fn(*mut c_void, u32, u32, u32) -> u32;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let sendmes_ptr = mjsend_message as *const();

    ensure!(args.len() >= 2, "usage: {} DLLPath", args[0]);

    let path = PathBuf::from(&args[1]);

    let _ = win_dlopen(&path);

    unsafe {
        let func: MJPInterfaceFuncP = std::mem::transmute(win_dlsym("MJPInterfaceFunc")?);
        println!("MJPInterfaceFunc :{:p}", func);
        println!("test create instance");

        let size = func(std::ptr::null_mut(), MJPI_CREATEINSTANCE, 0, 0);

        println!("size = {}", size);

        if size > 0 {
            let inst = libc::malloc(size as usize);

            ensure!(!inst.is_null(), "cannot allocate AI memory.");

            func(inst, MJPI_INITIALIZE, 0, std::mem::transmute(sendmes_ptr));
            {
                let state = &mut interface::G_STATE;
                state.shuffle();
                state.start();
            }
        
            /* 途中参加でエミュレート */
            func(inst, MJPI_ONEXCHANGE, MJST_INKYOKU, 0);
            let mut is_agari = false;

            for i in 0..18 {
                let mut tsumohai_num: u32;
                {
                    let state = &mut interface::G_STATE;
                    state.tsumo();
                    tsumohai_num = state.players[state.teban as usize].tsumohai.pai_num as u32;
                }

                let ret = func(inst, MJPI_SUTEHAI, tsumohai_num, 0);
                let index = ret & 0x3F;
                let flag = ret & 0xFF80;
                println!("ret = {} flag = {:04x}", index, flag);

                {
                    let state = &mut interface::G_STATE;

                    if flag == MJPI_SUTEHAI {
                        state.sutehai(index as usize);                        
                    } else if flag == MJPIR_TSUMO {
                        let score: [i32; 4] = [0, 0, 0, 0];
                        println!("agari!!!");
                        state.tsumo_agari();
                        is_agari = true;
                        func(inst, MJPI_ENDKYOKU, MJEK_RYUKYOKU, std::mem::transmute(score.as_ptr()));
                        break;
                    }
                }
            }

            if !is_agari {
                println!("流局(;;)");
                let score: [i32; 4] = [-3000, 0, 0, 0];
                func(inst, MJPI_ENDKYOKU, MJEK_RYUKYOKU, std::mem::transmute(score.as_ptr()));
            }

            libc::free(inst);
        }
    }

    println!("end.");

    Ok(())
}
