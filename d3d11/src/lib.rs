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

static mut REAL_D3D11: HMODULE = HMODULE(0);

fn ensure_real_d3d11() {
    unsafe {
        if REAL_D3D11.0 == 0 {
            match LoadLibraryA(s!("C:\\Windows\\System32\\d3d11.dll")) {
                Ok(h) => {
                    REAL_D3D11 = h;
                    log_info!("Loaded real d3d11.dll: {:?}", h);
                }
                Err(_e) => {
                    log_error!("Failed to load real d3d11.dll: {}", _e);
                }
            }
        }
    }
}

// D3D11CreateDevice(pAdapter, DriverType, Software, Flags,
//                  pFeatureLevels, FeatureLevels, SDKVersion,
//                  ppDevice, pFeatureLevel, ppImmediateContext)
type D3D11CreateDeviceFn = unsafe extern "system" fn(
    *mut c_void,
    u32,
    isize,
    u32,
    *const u32,
    u32,
    u32,
    *mut *mut c_void,
    *mut u32,
    *mut *mut c_void,
) -> HRESULT;

// D3D11CreateDeviceAndSwapChain adds pSwapChainDesc and ppSwapChain
type D3D11CreateDeviceAndSwapChainFn = unsafe extern "system" fn(
    *mut c_void,
    u32,
    isize,
    u32,
    *const u32,
    u32,
    u32,
    *const c_void,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut u32,
    *mut *mut c_void,
) -> HRESULT;

#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDevice(
    p_adapter: *mut c_void,
    driver_type: u32,
    software: isize,
    flags: u32,
    p_feature_levels: *const u32,
    feature_levels: u32,
    sdk_version: u32,
    pp_device: *mut *mut c_void,
    p_feature_level: *mut u32,
    pp_immediate_context: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d11();
    let proc = GetProcAddress(REAL_D3D11, s!("D3D11CreateDevice"));
    let func: D3D11CreateDeviceFn = std::mem::transmute(proc);
    let hr = func(
        p_adapter,
        driver_type,
        software,
        flags,
        p_feature_levels,
        feature_levels,
        sdk_version,
        pp_device,
        p_feature_level,
        pp_immediate_context,
    );
    log_info!(hr = format_args!("0x{:08X}", hr.0), "D3D11CreateDevice called");
    hr
}

#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDeviceAndSwapChain(
    p_adapter: *mut c_void,
    driver_type: u32,
    software: isize,
    flags: u32,
    p_feature_levels: *const u32,
    feature_levels: u32,
    sdk_version: u32,
    p_swap_chain_desc: *const c_void,
    pp_swap_chain: *mut *mut c_void,
    pp_device: *mut *mut c_void,
    p_feature_level: *mut u32,
    pp_immediate_context: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d11();
    let proc = GetProcAddress(REAL_D3D11, s!("D3D11CreateDeviceAndSwapChain"));
    let func: D3D11CreateDeviceAndSwapChainFn = std::mem::transmute(proc);
    let hr = func(
        p_adapter,
        driver_type,
        software,
        flags,
        p_feature_levels,
        feature_levels,
        sdk_version,
        p_swap_chain_desc,
        pp_swap_chain,
        pp_device,
        p_feature_level,
        pp_immediate_context,
    );
    log_info!(
        hr = format_args!("0x{:08X}", hr.0),
        "D3D11CreateDeviceAndSwapChain called"
    );
    hr
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        #[cfg(feature = "debug-log")]
        {
            faker_core::init_tracing("faker-d3d11.log");
            tracing::info!("faker-d3d11 loaded (DLL_PROCESS_ATTACH)");
        }
        ensure_real_d3d11();
        faker_core::install_hook_via_dxgi();
    }
    TRUE
}
