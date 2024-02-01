use std::path::Path;
use anyhow::anyhow;
use libc::c_void;
#[cfg(feature = "load-dll")]
use loadlibrary::win_dlopen;

pub fn load_ai<P: AsRef<Path>>(path: &P) {
    #[cfg(feature = "load-dll")]
    win_dlopen(path);
}

pub unsafe fn get_ai_symbol(sym: &str) -> anyhow::Result<*const c_void> {
    #[cfg(feature = "load-dll")]
    {
        return win_dlsym(sym)
    }
    Err(anyhow!("not implemented"))
}
