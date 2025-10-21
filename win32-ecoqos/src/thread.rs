use std::ffi::c_void;

use crate::preset::{THREAD_RESTORE, THREAD_THROTTLE, THREAD_UNTHROTTLE};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        GetThreadInformation, OpenThread, SetThreadInformation, SetThreadPriority,
        ThreadPowerThrottling, THREAD_INFORMATION_CLASS, THREAD_POWER_THROTTLING_CURRENT_VERSION,
        THREAD_POWER_THROTTLING_EXECUTION_SPEED, THREAD_POWER_THROTTLING_STATE, THREAD_PRIORITY,
        THREAD_PRIORITY_IDLE, THREAD_PRIORITY_NORMAL, THREAD_SET_INFORMATION,
    },
};

unsafe fn toggle_efficiency_mode_impl(
    hthread: HANDLE,
    threadinformation: *const c_void,
    threadinformationclass: THREAD_INFORMATION_CLASS,
    threadinformationsize: u32,
    npriority: THREAD_PRIORITY,
) -> Result<(), windows_result::Error> {
    unsafe {
        SetThreadInformation(
            hthread,
            threadinformationclass,
            threadinformation,
            threadinformationsize,
        )?;
        SetThreadPriority(hthread, npriority)?;
    }

    Ok(())
}

/// Toggle efficiency mode of a thread, by a thread_id.
///
/// WARN: [`Thread::id()`](https://doc.rust-lang.org/std/thread/struct.Thread.html#method.id)
/// is entirely unrelated to underlying thread ID.
///
/// To receive win32 thread id with ease,
/// see [retrieve_thread.rs](https://github.com/mokurin000/fitgirl-ecoqos/blob/master/win32-ecoqos/examples/retrieve_thread.rs)
pub fn toggle_efficiency_mode(
    thread_id: u32,
    enable: Option<bool>,
) -> Result<(), windows_result::Error> {
    let hthread = unsafe { OpenThread(THREAD_SET_INFORMATION, false, thread_id)? };
    let result = unsafe { toggle_efficiency_mode_handle(hthread, enable) };
    let _ = unsafe { CloseHandle(hthread) };

    result
}

/// Toggle efficiency mode of a thread, by a [`HANDLE`](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Foundation/struct.HANDLE.html).
///
/// [`GetCurrentThread`]: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/fn.GetCurrentThread.html
/// The handle returned by [`GetCurrentThread`] have a `THREAD_ALL_ACCESS`.
///
/// You must enable [`THREAD_SET_INFORMATION`](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/constant.THREAD_SET_INFORMATION.html)
/// access flag on the handle to apply EcoQoS throttle.
///
/// SAFETY: `hthread` must be a valid process handle. DO NOT pass null ptr, e.g.
pub unsafe fn toggle_efficiency_mode_handle(
    hthread: HANDLE,
    enable: Option<bool>,
) -> Result<(), windows_result::Error> {
    let new_state = match enable {
        Some(true) => THREAD_THROTTLE,
        Some(false) => THREAD_UNTHROTTLE,
        None => THREAD_RESTORE,
    };

    let threadinformationclass = ThreadPowerThrottling;
    let threadinformation = &new_state as *const _ as *const c_void;
    let threadinformationsize = size_of::<THREAD_POWER_THROTTLING_STATE>() as u32;

    let npriority = if let Some(true) = enable {
        THREAD_PRIORITY_IDLE
    } else {
        THREAD_PRIORITY_NORMAL
    };

    unsafe {
        toggle_efficiency_mode_impl(
            hthread,
            threadinformation,
            threadinformationclass,
            threadinformationsize,
            npriority,
        )
    }
}

/// check whether EcoQoS is enabled on a thread.
///
/// `hprocess` must have `THREAD_QUERY_INFORMATION` access right.
///
/// SAFETY: `hthread` must be a valid process handle. DO NOT pass null ptr, e.g.
pub unsafe fn ecoqos_enabled(hthread: HANDLE) -> Result<bool, windows_result::Error> {
    let mut thread_info = THREAD_POWER_THROTTLING_STATE {
        Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
        ..Default::default()
    };

    unsafe {
        GetThreadInformation(
            hthread,
            ThreadPowerThrottling,
            &mut thread_info as *mut _ as *mut _,
            size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
        )?;
    }

    Ok(
        thread_info.StateMask & THREAD_POWER_THROTTLING_EXECUTION_SPEED
            == THREAD_POWER_THROTTLING_EXECUTION_SPEED,
    )
}
