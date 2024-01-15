use std::{env, path::PathBuf, ffi::c_void};

use loadlibrary::{win_dlopen, win_dlsym};

type MJPInterfaceFuncP = extern "stdcall" fn(*mut c_void, u32, u32, u32) -> u32;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("usage: {} DLLPath", args[0]);
    }

    let path = PathBuf::from(args[1]);

    win_dlopen(&path);

    unsafe {
        let func: MJPInterfaceFuncP = win_dlsym("MJPInterfaceFunc") as MJPInterfaceFuncP;
        println!("MJPInterfaceFunc :{:p}", func);
        println!("test create instance");

        let size = func(std::ptr::null_mut(), MJPI_INITIALIZE, 0, 0);
    }

    println!("Hello, world!");
}
