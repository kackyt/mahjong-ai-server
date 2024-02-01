use std::{env, path::PathBuf, ffi::c_void};

use anyhow::ensure;
use mahjong_core::shanten::PaiState;

use ai_bridge::{ai_loader::{get_ai_symbol, load_ai}, bindings::{MJEK_RYUKYOKU, MJPIR_SUTEHAI, MJPIR_TSUMO, MJPI_BASHOGIME, MJPI_CREATEINSTANCE, MJPI_ENDGAME, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_ONEXCHANGE, MJPI_STARTGAME, MJPI_STARTKYOKU, MJPI_SUTEHAI, MJST_INKYOKU}, interface::{mjsend_message, G_STATE}};

extern crate libc;

type MJPInterfaceFuncP = extern "stdcall" fn(*mut c_void, usize, usize, usize) -> usize;

fn main() -> anyhow::Result<()> {
    let _guard = sentry::init((env::var("SENTRY_ENV").unwrap_or(String::from("")), sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));
    let args: Vec<String> = env::args().collect();
    let sendmes_ptr = mjsend_message as *const();

    ensure!(args.len() >= 2, "usage: {} DLLPath", args[0]);

    let path = PathBuf::from(&args[1]);

    load_ai(&path);

    unsafe {
        let func: MJPInterfaceFuncP = std::mem::transmute(get_ai_symbol("MJPInterfaceFunc")?);
        println!("MJPInterfaceFunc :{:p}", func);
        println!("test create instance");

        let size = func(std::ptr::null_mut(), MJPI_CREATEINSTANCE.try_into().unwrap(), 0, 0);

        println!("size = {}", size);

        if size > 0 {
            let inst = libc::malloc(size as usize);
            let dummy: [i32; 4] = [4, 5, 6, 7];

            libc::memset(inst, 0, size as usize);

            ensure!(!inst.is_null(), "cannot allocate AI memory.");

            let init_success = func(inst, MJPI_INITIALIZE.try_into().unwrap(), 0, std::mem::transmute(sendmes_ptr));

            println!("init end {} {:p}", init_success, inst);

            // ensure!(init_success == 0, "cannot initialize AI.");

            {
                let state = &mut G_STATE;
                state.create(b"test", 1);
                state.shuffle();
                state.start();
            }

            /* 途中参加でエミュレート
            func(inst, MJPI_ONEXCHANGE, MJST_INKYOKU, 0);
            */
            func(inst, MJPI_STARTGAME.try_into().unwrap(), 0, 0);
            println!("start game end");
            func(inst, MJPI_BASHOGIME.try_into().unwrap(), std::mem::transmute(dummy.as_ptr()), 0);
            println!("bashogime end");
            func(inst, MJPI_STARTKYOKU.try_into().unwrap(), 0, 0);
            println!("start kyoku end");
        
            let mut is_agari = false;

            for i in 0..18 {
                let mut tsumohai_num: usize;
                {
                    let state = &mut G_STATE;
                    state.tsumo();
                    tsumohai_num = state.players[state.teban as usize].tsumohai.pai_num.try_into().unwrap();
                }

                let ret: u32 = func(inst, MJPI_SUTEHAI.try_into().unwrap(), tsumohai_num, 0).try_into().unwrap();
                let index = ret & 0x3F;
                let flag = ret & 0xFF80;
                println!("ret = {} flag = {:04x}", index, flag);

                {
                    let state = &mut G_STATE;
                    {
                        let player = &state.players[state.teban as usize];
                        for p in &player.tehai {
                            print!("{}", p);
                        }

                        print!("{}", player.tsumohai);
                        let shanten = PaiState::from(&player.tehai).get_shanten(0);
                        println!(" シャンテン数 {}\r", shanten);
                    }

                    if flag == MJPIR_SUTEHAI {
                        state.sutehai(index as usize);                        
                    } else if flag == MJPIR_TSUMO {
                        let score: [i32; 4] = [0, 0, 0, 0];
                        println!("agari!!!");
                        let agari = state.tsumo_agari()?;
                        println!("{:?}", agari.yaku);
                        is_agari = true;
                        func(inst, MJPI_ENDKYOKU.try_into().unwrap(), MJEK_RYUKYOKU.try_into().unwrap(), std::mem::transmute(score.as_ptr()));
                        break;
                    }
                }
            }

            if !is_agari {
                println!("流局(;;)");
                let score: [i32; 4] = [-3000, 0, 0, 0];
                func(inst, MJPI_ENDKYOKU.try_into().unwrap(), MJEK_RYUKYOKU.try_into().unwrap(), std::mem::transmute(score.as_ptr()));
            }

            libc::free(inst);
        }
    }

    println!("end.");

    Ok(())
}
