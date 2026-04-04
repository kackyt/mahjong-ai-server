use libc::c_void;
#[cfg(feature = "load-dll")]
use loadlibrary::{win_dlopen, win_dlsym};
use std::path::Path;

pub fn load_ai<P: AsRef<Path>>(_path: &P) -> anyhow::Result<*mut c_void> {
    #[cfg(feature = "load-dll")]
    return win_dlopen(_path);

    #[cfg(not(feature = "load-dll"))]
    anyhow::bail!("not implemented")
}

/// # Safety
///
/// この関数は、有効なDLLハンドルと正しいシンボル名を前提としています。
pub unsafe fn get_ai_symbol(_handle: *mut c_void, _sym: &str) -> anyhow::Result<*const c_void> {
    #[cfg(feature = "load-dll")]
    return win_dlsym(_handle, _sym);

    #[cfg(not(feature = "load-dll"))]
    anyhow::bail!("not implemented")
}
