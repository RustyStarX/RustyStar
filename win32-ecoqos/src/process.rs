use std::ffi::c_void;

use crate::preset::{PROCESS_RESTORE, PROCESS_THROTTLE, PROCESS_UNTHROTTLE};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        GetProcessInformation, OpenProcess, ProcessPowerThrottling, SetPriorityClass,
        SetProcessInformation, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
        PROCESS_INFORMATION_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
        PROCESS_SET_INFORMATION,
    },
};

unsafe fn toggle_efficiency_mode_impl(
    hprocess: HANDLE,
    processinformation: *const c_void,
    processinformationclass: PROCESS_INFORMATION_CLASS,
    processinformationsize: u32,
    dwpriorityclass: PROCESS_CREATION_FLAGS,
) -> Result<(), windows_result::Error> {
    unsafe {
        SetProcessInformation(
            hprocess,
            processinformationclass,
            processinformation,
            processinformationsize,
        )?;
        SetPriorityClass(hprocess, dwpriorityclass)?;
    }

    Ok(())
}

/// Toggle efficiency mode of a process, by it's PID.
///
/// ```rust
/// use win32_ecoqos::process::toggle_efficiency_mode;
///
/// let pid = std::process::id();
///
/// // Enable EcoQoS
/// toggle_efficiency_mode(pid, Some(true));
/// // Enable HighQoS
/// toggle_efficiency_mode(pid, Some(false));
/// // Let system decide
/// toggle_efficiency_mode(pid, None);
/// ```
pub fn toggle_efficiency_mode(pid: u32, enable: Option<bool>) -> Result<(), windows_result::Error> {
    let hprocess = unsafe { OpenProcess(PROCESS_SET_INFORMATION, false, pid)? };
    let result = unsafe { toggle_efficiency_mode_handle(hprocess, enable) };
    let _ = unsafe { CloseHandle(hprocess) };

    result
}

/// Toggle efficiency mode of a process, by a [`HANDLE`](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Foundation/struct.HANDLE.html).
///
/// [`GetCurrentProcess`]: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/fn.GetCurrentProcess.html
/// The handle returned by [`GetCurrentProcess`] have a `PROCESS_ALL_ACCESS`.
///
/// You must enable [`PROCESS_SET_INFORMATION`](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/constant.PROCESS_SET_INFORMATION.html)
/// access flag on your handle to apply EcoQoS throttle.
///
/// SAFETY: `hprocess` must be a valid process handle. DO NOT pass null ptr, e.g.
///
/// ```rust
/// use win32_ecoqos::process::toggle_efficiency_mode_handle;
/// use windows::Win32::Foundation::CloseHandle;
/// use windows::Win32::System::Threading::GetCurrentProcess;
///
/// unsafe {
///     let hprocess = GetCurrentProcess();
///
///     // Enable EcoQos
///     toggle_efficiency_mode_handle(hprocess, Some(true));
///     // Enable HighQos
///     toggle_efficiency_mode_handle(hprocess, Some(false));
///     // Let system decide
///     toggle_efficiency_mode_handle(hprocess, None);
///
///     // Avoid resource leak
///     CloseHandle(hprocess);
/// }
/// ```
pub unsafe fn toggle_efficiency_mode_handle(
    hprocess: HANDLE,
    enable: Option<bool>,
) -> Result<(), windows_result::Error> {
    let new_state = match enable {
        Some(true) => PROCESS_THROTTLE,
        Some(false) => PROCESS_UNTHROTTLE,
        None => PROCESS_RESTORE,
    };

    let processinformationclass = ProcessPowerThrottling;
    let processinformation = &new_state as *const _ as *const c_void;
    let processinformationsize = size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32;

    let dwpriorityclass = if let Some(true) = enable {
        IDLE_PRIORITY_CLASS
    } else {
        NORMAL_PRIORITY_CLASS
    };

    unsafe {
        toggle_efficiency_mode_impl(
            hprocess,
            processinformation,
            processinformationclass,
            processinformationsize,
            dwpriorityclass,
        )
    }
}

/// check whether EcoQoS is enabled on a process.
///
/// `hprocess` must have `PROCESS_QUERY_INFORMATION` access right.
///
/// SAFETY: `hprocess` must be a valid process handle. DO NOT pass null ptr, e.g.
pub unsafe fn ecoqos_enabled(hprocess: HANDLE) -> Result<bool, windows_result::Error> {
    let mut process_info = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: 0,
        StateMask: 0,
    };

    unsafe {
        GetProcessInformation(
            hprocess,
            ProcessPowerThrottling,
            &mut process_info as *mut _ as *mut _,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )?;
    }

    Ok(
        process_info.StateMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED
            == PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    )
}
