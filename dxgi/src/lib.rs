use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::SystemServices::*;

#[cfg(feature = "debug-log")]
macro_rules! log_info {
    ($($arg:tt)*) => { tracing::info!($($arg)*) };
}
#[cfg(not(feature = "debug-log"))]
macro_rules! log_info {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "debug-log")]
macro_rules! log_error {
    ($($arg:tt)*) => { tracing::error!($($arg)*) };
}
#[cfg(not(feature = "debug-log"))]
macro_rules! log_error {
    ($($arg:tt)*) => {};
}

static mut REAL_DXGI: HMODULE = HMODULE(0);

fn ensure_real_dxgi() {
    unsafe {
        if REAL_DXGI.0 == 0 {
            match LoadLibraryA(s!("C:\\Windows\\System32\\dxgi.dll")) {
                Ok(h) => {
                    REAL_DXGI = h;
                    log_info!("Loaded real dxgi.dll: {:?}", h);
                }
                Err(_e) => {
                    log_error!("Failed to load real dxgi.dll: {}", _e);
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory(
    riid: *const GUID,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    ensure_real_dxgi();

    let proc = GetProcAddress(REAL_DXGI, s!("CreateDXGIFactory"));
    let func: extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT =
        std::mem::transmute(proc);

    let hr = func(riid, pp_factory);
    log_info!(
        hr = format_args!("0x{:08X}", hr.0),
        "CreateDXGIFactory called"
    );
    if hr.is_ok() {
        faker_core::try_install_hook(pp_factory);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory1(
    riid: *const GUID,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    ensure_real_dxgi();

    let proc = GetProcAddress(REAL_DXGI, s!("CreateDXGIFactory1"));
    let func: extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT =
        std::mem::transmute(proc);

    let hr = func(riid, pp_factory);
    log_info!(
        hr = format_args!("0x{:08X}", hr.0),
        "CreateDXGIFactory1 called"
    );
    if hr.is_ok() {
        faker_core::try_install_hook(pp_factory);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory2(
    flags: u32,
    riid: *const GUID,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    ensure_real_dxgi();

    let proc = GetProcAddress(REAL_DXGI, s!("CreateDXGIFactory2"));
    let func: extern "system" fn(u32, *const GUID, *mut *mut c_void) -> HRESULT =
        std::mem::transmute(proc);

    let hr = func(flags, riid, pp_factory);
    log_info!(
        flags,
        hr = format_args!("0x{:08X}", hr.0),
        "CreateDXGIFactory2 called"
    );
    if hr.is_ok() {
        faker_core::try_install_hook(pp_factory);
    }
    hr
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        #[cfg(feature = "debug-log")]
        {
            faker_core::init_tracing("faker-dxgi.log");
            tracing::info!("faker-dxgi loaded (DLL_PROCESS_ATTACH)");
        }
        ensure_real_dxgi();
    }

    TRUE
}
