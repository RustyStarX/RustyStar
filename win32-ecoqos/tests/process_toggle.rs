use win32_ecoqos::process::{ecoqos_enabled, toggle_efficiency_mode_handle};
use windows::Win32::System::Threading::GetCurrentProcess;

#[test]
pub fn test_process_ecoqos() -> Result<(), windows_result::Error> {
    unsafe {
        let hprocess = GetCurrentProcess();

        toggle_efficiency_mode_handle(hprocess, Some(true))?;
        assert!(ecoqos_enabled(hprocess)?);
        toggle_efficiency_mode_handle(hprocess, Some(false))?;
        assert!(!ecoqos_enabled(hprocess)?);
    }

    Ok(())
}
