use std::{env, path::PathBuf, ffi::c_void};

use loadlibrary::{win_dlopen, win_dlsym, pe_image};
use anyhow::ensure;
use core::fmt::Error;

mod bindings;

type MJPInterfaceFuncP = extern "stdcall" fn(*mut c_void, u32, u32, u32) -> u32;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut image: pe_image = Default::default();

    ensure!(args.len() >= 2, "usage: {} DLLPath", args[0]);

    let path = PathBuf::from(&args[1]);

    let _ = win_dlopen(&mut image, &path);

    unsafe {
        let func: MJPInterfaceFuncP = std::mem::transmute(win_dlsym("MJPInterfaceFunc")?);
        println!("MJPInterfaceFunc :{:p}", func);
        println!("test create instance");

        let size = func(std::ptr::null_mut(), bindings::MJPI_INITIALIZE, 0, 0);
    }

    println!("Hello, world!");

    Ok(())
}
