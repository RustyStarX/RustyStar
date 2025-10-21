use win32_ecoqos::thread::{ecoqos_enabled, toggle_efficiency_mode_handle};
use windows::Win32::System::Threading::GetCurrentThread;

#[test]
pub fn test_thread_ecoqos() -> Result<(), windows_result::Error> {
    unsafe {
        let hthread = GetCurrentThread();

        toggle_efficiency_mode_handle(hthread, Some(true))?;
        assert!(ecoqos_enabled(hthread)?);
        toggle_efficiency_mode_handle(hthread, Some(false))?;
        assert!(!ecoqos_enabled(hthread)?);
    }

    Ok(())
}
