use std::env;
use windows::core::{HSTRING, PCSTR, PCWSTR};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_dll>", args[0]);
        std::process::exit(1);
    }

    let dll_path = &args[1];
    println!("Attempting to load DLL: {}", dll_path);

    let path_wide: Vec<u16> = HSTRING::from(dll_path).as_wide().to_vec();
    // Add null terminator if HSTRING doesn't guarantee it for as_wide() slice (it usually does for internal buffer but as_wide returns slice)
    // Actually HSTRING::as_wide() returns a slice without null terminator?
    // Wait, PCWSTR expects null-terminated string.
    // HSTRING itself manages null termination internally, but as_wide() returns the slice.
    // The safe way is to append 0.
    let mut path_wide_null = path_wide.clone();
    path_wide_null.push(0);
    let path_pcwstr = PCWSTR(path_wide_null.as_ptr());

    unsafe {
        match LoadLibraryW(path_pcwstr) {
            Ok(h_module) => {
                println!("Successfully loaded DLL. Handle: {:?}", h_module);

                let proc_name = std::ffi::CString::new("MJPInterfaceFunc").unwrap();
                let proc_name_pcstr = PCSTR(proc_name.as_ptr() as *const u8);

                let proc_addr = GetProcAddress(h_module, proc_name_pcstr);

                if let Some(addr) = proc_addr {
                    println!(
                        "Successfully found 'MJPInterfaceFunc' at address: {:?}",
                        addr
                    );
                } else {
                    use windows::Win32::Foundation::GetLastError;
                    let error = GetLastError();
                    eprintln!("Failed to find 'MJPInterfaceFunc'. Error code: {:?}", error);
                }
            }
            Err(e) => {
                eprintln!("Failed to load DLL.");
                eprintln!("Error: {:?}", e);
                eprintln!("Common causes: Architecture mismatch (32-bit vs 64-bit), missing dependencies, or invalid path.");
                std::process::exit(1);
            }
        }
    }
}
