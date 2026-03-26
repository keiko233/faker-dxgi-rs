use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::System::SystemServices::*;

#[no_mangle]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        #[cfg(feature = "debug-log")]
        {
            faker_core::init_tracing("faker-dxvk.log");
            tracing::info!("faker-dxvk loaded (DLL_PROCESS_ATTACH)");
        }
        faker_core::install_hook_via_dxgi();
    }
    TRUE
}
