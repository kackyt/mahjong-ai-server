use anyhow::bail;
use libc::c_void;
#[cfg(feature = "load-dll")]
use loadlibrary::{win_dlopen, win_dlsym};
use std::path::Path;

pub fn load_ai<P: AsRef<Path>>(path: &P) -> anyhow::Result<*mut c_void> {
    #[cfg(feature = "load-dll")]
    {
        return win_dlopen(path);
    }
    bail!("not implemented")
}

pub unsafe fn get_ai_symbol(handle: *mut c_void, sym: &str) -> anyhow::Result<*const c_void> {
    #[cfg(feature = "load-dll")]
    {
        return win_dlsym(handle, sym);
    }
    bail!("not implemented")
}
