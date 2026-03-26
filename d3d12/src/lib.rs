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

static mut REAL_D3D12: HMODULE = HMODULE(0);

fn ensure_real_d3d12() {
    unsafe {
        if REAL_D3D12.0 == 0 {
            match LoadLibraryA(s!("C:\\Windows\\System32\\d3d12.dll")) {
                Ok(h) => {
                    REAL_D3D12 = h;
                    log_info!("Loaded real d3d12.dll: {:?}", h);
                }
                Err(_e) => {
                    log_error!("Failed to load real d3d12.dll: {}", _e);
                }
            }
        }
    }
}

// D3D12CreateDevice(pAdapter, MinimumFeatureLevel, riid, ppDevice)
type D3D12CreateDeviceFn = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *const GUID,
    *mut *mut c_void,
) -> HRESULT;

// D3D12GetDebugInterface(riid, ppvDebug)
type D3D12GetDebugInterfaceFn =
    unsafe extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT;

// D3D12SerializeRootSignature(pRootSignature, Version, ppBlob, ppErrorBlob)
type D3D12SerializeRootSignatureFn = unsafe extern "system" fn(
    *const c_void,
    u32,
    *mut *mut c_void,
    *mut *mut c_void,
) -> HRESULT;

// D3D12SerializeVersionedRootSignature(pRootSignature, ppBlob, ppErrorBlob)
type D3D12SerializeVersionedRootSignatureFn = unsafe extern "system" fn(
    *const c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> HRESULT;

// D3D12CreateRootSignatureDeserializer(pSrcData, SrcDataSizeInBytes, pRootSignatureDeserializerInterface, ppRootSignatureDeserializer)
type D3D12CreateRootSignatureDeserializerFn = unsafe extern "system" fn(
    *const c_void,
    usize,
    *const GUID,
    *mut *mut c_void,
) -> HRESULT;

// D3D12CreateVersionedRootSignatureDeserializer(pSrcData, SrcDataSizeInBytes, pRootSignatureDeserializerInterface, ppRootSignatureDeserializer)
type D3D12CreateVersionedRootSignatureDeserializerFn = unsafe extern "system" fn(
    *const c_void,
    usize,
    *const GUID,
    *mut *mut c_void,
) -> HRESULT;

#[no_mangle]
pub unsafe extern "system" fn D3D12CreateDevice(
    p_adapter: *mut c_void,
    minimum_feature_level: u32,
    riid: *const GUID,
    pp_device: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(REAL_D3D12, s!("D3D12CreateDevice"));
    let func: D3D12CreateDeviceFn = std::mem::transmute(proc);
    let hr = func(p_adapter, minimum_feature_level, riid, pp_device);
    log_info!(hr = format_args!("0x{:08X}", hr.0), "D3D12CreateDevice called");
    hr
}

#[no_mangle]
pub unsafe extern "system" fn D3D12GetDebugInterface(
    riid: *const GUID,
    ppv_debug: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(REAL_D3D12, s!("D3D12GetDebugInterface"));
    let func: D3D12GetDebugInterfaceFn = std::mem::transmute(proc);
    let hr = func(riid, ppv_debug);
    log_info!(hr = format_args!("0x{:08X}", hr.0), "D3D12GetDebugInterface called");
    hr
}

#[no_mangle]
pub unsafe extern "system" fn D3D12SerializeRootSignature(
    p_root_signature: *const c_void,
    version: u32,
    pp_blob: *mut *mut c_void,
    pp_error_blob: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(REAL_D3D12, s!("D3D12SerializeRootSignature"));
    let func: D3D12SerializeRootSignatureFn = std::mem::transmute(proc);
    func(p_root_signature, version, pp_blob, pp_error_blob)
}

#[no_mangle]
pub unsafe extern "system" fn D3D12SerializeVersionedRootSignature(
    p_root_signature: *const c_void,
    pp_blob: *mut *mut c_void,
    pp_error_blob: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(REAL_D3D12, s!("D3D12SerializeVersionedRootSignature"));
    let func: D3D12SerializeVersionedRootSignatureFn = std::mem::transmute(proc);
    func(p_root_signature, pp_blob, pp_error_blob)
}

#[no_mangle]
pub unsafe extern "system" fn D3D12CreateRootSignatureDeserializer(
    p_src_data: *const c_void,
    src_data_size: usize,
    p_interface: *const GUID,
    pp_deserializer: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(REAL_D3D12, s!("D3D12CreateRootSignatureDeserializer"));
    let func: D3D12CreateRootSignatureDeserializerFn = std::mem::transmute(proc);
    func(p_src_data, src_data_size, p_interface, pp_deserializer)
}

#[no_mangle]
pub unsafe extern "system" fn D3D12CreateVersionedRootSignatureDeserializer(
    p_src_data: *const c_void,
    src_data_size: usize,
    p_interface: *const GUID,
    pp_deserializer: *mut *mut c_void,
) -> HRESULT {
    ensure_real_d3d12();
    let proc = GetProcAddress(
        REAL_D3D12,
        s!("D3D12CreateVersionedRootSignatureDeserializer"),
    );
    let func: D3D12CreateVersionedRootSignatureDeserializerFn = std::mem::transmute(proc);
    func(p_src_data, src_data_size, p_interface, pp_deserializer)
}

#[no_mangle]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        #[cfg(feature = "debug-log")]
        {
            faker_core::init_tracing("faker-d3d12.log");
            tracing::info!("faker-d3d12 loaded (DLL_PROCESS_ATTACH)");
        }
        ensure_real_d3d12();
        faker_core::install_hook_via_dxgi();
    }
    TRUE
}
