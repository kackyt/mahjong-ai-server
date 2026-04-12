use std::{env, ffi::c_void, path::PathBuf};

use anyhow::ensure;
use clap::Parser;
use mahjong_core::{play_log, shanten::PaiState};

use ai_bridge::{
    ai_loader::{get_ai_symbol, load_ai},
    bindings::{
        MJEK_RYUKYOKU, MJPIR_REACH, MJPIR_SUTEHAI, MJPIR_TSUMO, MJPI_BASHOGIME,
        MJPI_CREATEINSTANCE, MJPI_ENDKYOKU, MJPI_INITIALIZE, MJPI_STARTGAME, MJPI_STARTKYOKU,
        MJPI_SUTEHAI,
    },
    interface::{mjsend_message, G_STATE},
};

extern crate libc;

type MJPInterfaceFuncP = unsafe extern "system" fn(*mut c_void, usize, usize, usize) -> usize;

#[derive(Parser, Debug)]
#[command(author, about, version)]
struct Command {
    #[arg(short, long)]
    log_path: String,
    #[arg(short, long)]
    input_dll: String,
    #[arg(short, long)]
    paiyama_path: Option<String>,
}

unsafe fn experiment(func: MJPInterfaceFuncP, inst: *mut c_void, play_log: &mut play_log::PlayLog) {
    {
        let state = unsafe { &mut *std::ptr::addr_of_mut!(G_STATE) };
        state.start(play_log);
    }
    func(inst, MJPI_STARTKYOKU as usize, 0, 0);
    println!("start kyoku end");

    let mut is_agari = false;

    for _i in 0..18 {
        let tsumohai_num: usize;
        {
            let state = unsafe { &mut *std::ptr::addr_of_mut!(G_STATE) };
            let _ = state.tsumo(play_log);
            tsumohai_num = state.players[state.teban as usize].tsumohai.pai_num.into();
        }

        let ret: u32 = func(inst, MJPI_SUTEHAI as usize, tsumohai_num, 0) as u32;
        let index = ret & 0x3F;
        let flag = ret & 0xFF80;
        println!("ret = {} flag = {:04x}", index, flag);

        {
            let state = unsafe { &mut *std::ptr::addr_of_mut!(G_STATE) };
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
                let _ = state.sutehai(play_log, index as usize, false);
            } else if flag == MJPIR_REACH {
                let _ = state.sutehai(play_log, index as usize, true);
            } else if flag == MJPIR_TSUMO {
                let score: [i32; 4] = [0, 0, 0, 0];
                println!("agari!!!");
                let agari_r = state.tsumo_agari(play_log);

                match agari_r {
                    Ok(agari) => {
                        println!("{:?}", agari.yaku);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }

                is_agari = true;
                func(
                    inst,
                    MJPI_ENDKYOKU as usize,
                    MJEK_RYUKYOKU as usize,
                    score.as_ptr() as usize,
                );
                break;
            }
        }
    }

    if !is_agari {
        println!("流局(;;)");
        let score: [i32; 4] = [-3000, 0, 0, 0];
        func(
            inst,
            MJPI_ENDKYOKU as usize,
            MJEK_RYUKYOKU as usize,
            score.as_ptr() as usize,
        );
        let state = unsafe { &mut *std::ptr::addr_of_mut!(G_STATE) };
        state.nagare(play_log);
    }
}

fn cmd(args: &Command) -> anyhow::Result<()> {
    let sendmes_ptr = mjsend_message as *const ();
    let mut play_log = play_log::PlayLog::new();

    let path = PathBuf::from(&args.input_dll);

    let handle = load_ai(&path)?;

    unsafe {
        let func: MJPInterfaceFuncP = std::mem::transmute::<*const c_void, MJPInterfaceFuncP>(
            get_ai_symbol(handle, "MJPInterfaceFunc")?,
        );
        println!("MJPInterfaceFunc :{:p}", func);
        println!("test create instance");

        let size = func(std::ptr::null_mut(), MJPI_CREATEINSTANCE as usize, 0, 0);

        println!("size = {}", size);

        if size > 0 {
            let inst = libc::malloc(size as usize);
            let dummy: [i32; 4] = [4, 5, 6, 7];

            libc::memset(inst, 0, size as usize);

            ensure!(!inst.is_null(), "cannot allocate AI memory.");

            let init_success = func(inst, MJPI_INITIALIZE as usize, 0, sendmes_ptr as usize);

            println!("init end {} {:p}", init_success, inst);

            {
                let state = &mut *std::ptr::addr_of_mut!(G_STATE);
                state.create(b"test", 1, &mut play_log);
            }

            func(inst, MJPI_STARTGAME as usize, 0, 0);
            println!("start game end");
            func(inst, MJPI_BASHOGIME as usize, dummy.as_ptr() as usize, 0);
            println!("bashogime end");

            if let Some(paiyama_path) = &args.paiyama_path {
                let paiyama_batch = play_log::PaiyamaBatch::new(paiyama_path);

                for paiyama_r in paiyama_batch {
                    let paiyama = paiyama_r?;

                    {
                        let state = &mut *std::ptr::addr_of_mut!(G_STATE);
                        state.load(&paiyama.1);
                    }
                    experiment(func, inst, &mut play_log);
                }
            } else {
                // 1回きりの実行
                {
                    let state = &mut *std::ptr::addr_of_mut!(G_STATE);
                    state.shuffle();
                }

                experiment(func, inst, &mut play_log);
            }

            play_log.write_to_parquet(&args.log_path)?;

            libc::free(inst);
        }
    }

    println!("end.");

    Ok(())
}

fn main() {
    let _ = dotenv::dotenv();
    let args = Command::parse();
    let _guard = sentry::init((
        env::var("SENTRY_DSN").unwrap_or(String::from("")),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    cmd(&args).unwrap();
}
