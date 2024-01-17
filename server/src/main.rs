use std::{env, path::PathBuf, ffi::c_void};

use loadlibrary::{win_dlopen, win_dlsym};
use anyhow::ensure;

use crate::{bindings::{MJPI_CREATEINSTANCE, MJPI_ONEXCHANGE, MJST_INKYOKU, MJPI_SUTEHAI, MJPI_INITIALIZE}, interface::mjsend_message};

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
            
            /* 途中参加でエミュレート */
            func(inst, MJPI_ONEXCHANGE, MJST_INKYOKU, 0);

            let ret = func(inst, MJPI_SUTEHAI, 10, 0);

            println!("ret = {} flag = {:04x}", ret & 0x3F, ret & 0xFF80);

            libc::free(inst);
        }
    }

    println!("end.");

    Ok(())
}
