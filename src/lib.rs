use std::ffi::c_void;
use std::sync::{Once, OnceLock};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::SystemServices::*;

use retour::GenericDetour;

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

const FAKE_GPU_NAME: &str = concat!(env!("FAKE_GPU_NAME"), "\0");

type GetDescFn = unsafe extern "system" fn(*mut c_void, *mut DXGI_ADAPTER_DESC) -> HRESULT;

static GET_DESC_HOOK: OnceLock<GenericDetour<GetDescFn>> = OnceLock::new();

unsafe extern "system" fn hooked_getdesc(
    this: *mut c_void,
    desc: *mut DXGI_ADAPTER_DESC,
) -> HRESULT {
    let hook = GET_DESC_HOOK.get().unwrap();
    let hr = hook.call(this, desc);

    if hr.is_ok() && !desc.is_null() {
        #[cfg(feature = "debug-log")]
        {
            let desc_arr = &(*desc).Description;
            let name_len = desc_arr.iter().position(|&c| c == 0).unwrap_or(128);
            let real_name = String::from_utf16_lossy(&desc_arr[..name_len]);
            tracing::info!(real_gpu = %real_name, "GetDesc hooked");
        }

        let fake: Vec<u16> = FAKE_GPU_NAME.encode_utf16().collect();
        let desc_arr = &mut (*desc).Description;
        desc_arr[..fake.len()].copy_from_slice(&fake);
        log_info!(
            fake_gpu = FAKE_GPU_NAME.trim_end_matches('\0'),
            "GetDesc faked"
        );
    } else {
        log_info!(hr = format_args!("0x{:08X}", hr.0), "GetDesc call failed");
    }

    hr
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

static HOOK_INIT: Once = Once::new();

/// Install the GetDesc hook via the factory's first adapter.
/// Uses `Once` so it only runs once regardless of which CreateDXGIFactory* is called.
unsafe fn try_install_hook(pp_factory: *mut *mut c_void) {
    HOOK_INIT.call_once(|| {
        let factory_ptr = *pp_factory;
        if factory_ptr.is_null() {
            log_error!("Factory pointer is null, cannot install hook");
            return;
        }

        // IDXGIFactory vtable:
        //   IUnknown:    [0] QueryInterface  [1] AddRef  [2] Release
        //   IDXGIObject: [3] SetPrivateData  [4] SetPrivateDataInterface
        //                [5] GetPrivateData  [6] GetParent
        //   IDXGIFactory:[7] EnumAdapters ...
        let vtable = *(factory_ptr as *const *const usize);

        type EnumAdaptersFn =
            unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> HRESULT;
        let enum_adapters: EnumAdaptersFn = std::mem::transmute(*vtable.add(7));

        let mut adapter_ptr: *mut c_void = std::ptr::null_mut();
        let hr = enum_adapters(factory_ptr, 0, &mut adapter_ptr);
        if hr.is_err() || adapter_ptr.is_null() {
            log_error!("EnumAdapters(0) failed: 0x{:08X}", hr.0);
            return;
        }

        // IDXGIAdapter vtable:
        //   IUnknown:     [0..2]
        //   IDXGIObject:  [3..6]
        //   IDXGIAdapter: [7] EnumOutputs  [8] GetDesc  [9] CheckInterfaceSupport
        let adapter_vtable = *(adapter_ptr as *const *const usize);
        let get_desc_fn: GetDescFn = std::mem::transmute(*adapter_vtable.add(8));

        match GenericDetour::new(get_desc_fn, hooked_getdesc) {
            Ok(hook) => match hook.enable() {
                Ok(()) => {
                    log_info!("GetDesc hook installed successfully");
                    GET_DESC_HOOK.set(hook).ok();
                }
                Err(_e) => {
                    log_error!("Failed to enable GetDesc hook: {}", _e);
                }
            },
            Err(_e) => {
                log_error!("Failed to create GetDesc hook: {}", _e);
            }
        }

        // Release the adapter
        type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
        let release: ReleaseFn = std::mem::transmute(*adapter_vtable.add(2));
        release(adapter_ptr);
    });
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
        try_install_hook(pp_factory);
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
        try_install_hook(pp_factory);
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
        try_install_hook(pp_factory);
    }
    hr
}

#[cfg(feature = "debug-log")]
fn init_tracing() {
    use tracing_appender::rolling;
    use tracing_subscriber::fmt;

    let exe_path = std::env::current_exe().unwrap_or_default();
    let log_dir = exe_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let file_appender = rolling::never(&log_dir, "faker-dxgi.log");

    let _ = fmt().with_writer(file_appender).with_ansi(false).try_init();
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        #[cfg(feature = "debug-log")]
        {
            init_tracing();
            tracing::info!("faker-dxgi loaded (DLL_PROCESS_ATTACH)");
        }
        ensure_real_dxgi();
    }

    TRUE
}
