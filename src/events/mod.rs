use std::error::Error;
use std::slice;

use spdlog::info;
use win32_ecoqos::utils::Processes;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EVENT_SYSTEM_FOREGROUND, EnumChildWindows, GetMessageW,
    GetWindowThreadProcessId, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

use windows::core::BOOL;

use crate::PID_SENDER;
pub fn enter_event_loop() -> Result<(), Box<dyn Error + Send + Sync>> {
    unsafe extern "system" fn hook(
        _hwineventhook: HWINEVENTHOOK,
        _event: u32,
        window_thread: HWND,
        _idobject: i32,
        _idchild: i32,
        _ideventthread: u32,
        _dwmseventtime: u32,
    ) {
        unsafe {
            let mut process_id = 0_u32;

            if GetWindowThreadProcessId(window_thread, Some(&mut process_id as _)) == 0 {
                return;
            }

            let is_uwp = {
                Processes::try_new().is_ok_and(|mut procs| {
                    procs
                        .find(|p| p.process_id == process_id)
                        .is_some_and(|p| p.process_name == "ApplicationFrameHost.exe")
                })
            };
            if is_uwp {
                let real_pid = try_find_uwp_process(process_id, window_thread);
                if real_pid != 0 {
                    process_id = real_pid
                }
            }

            if let Some(tx) = PID_SENDER.get() {
                let _ = tx.send(process_id);
            }
        }
    }

    let eventmin = EVENT_SYSTEM_FOREGROUND;
    let eventmax = EVENT_SYSTEM_FOREGROUND;
    let hmodwineventproc = None;
    let pfnwineventproc =
        Some(hook as unsafe extern "system" fn(HWINEVENTHOOK, u32, HWND, i32, i32, u32, u32));
    let idprocess = 0;
    let idthread = 0;
    let dwflags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;

    info!("setup WinEventHook...");

    unsafe {
        SetWinEventHook(
            eventmin,
            eventmax,
            hmodwineventproc,
            pfnwineventproc,
            idprocess,
            idthread,
            dwflags,
        );
    };

    let mut msg = MSG::default();

    info!("entering event loop...");

    unsafe {
        GetMessageW(&mut msg as *mut _, None, 0, 0).ok()?;
        DispatchMessageW(&msg as _);
    }

    Ok(())
}

fn try_find_uwp_process(process_id: u32, window_thread: HWND) -> u32 {
    unsafe extern "system" fn find_match(hwnd: HWND, ctx_ptr: LPARAM) -> BOOL {
        let ctx = unsafe { slice::from_raw_parts_mut(ctx_ptr.0 as _, 2) };
        let mut lpdwprocessid = 0_u32;
        if unsafe { GetWindowThreadProcessId(hwnd, Some(&mut lpdwprocessid as _)) } != 0 {
            if lpdwprocessid != ctx[1] {
                ctx[0] = lpdwprocessid;
            }
        }
        true.into()
    }

    let mut ctx = [0_u32, process_id];
    let lparam = LPARAM(ctx.as_mut_ptr() as isize);

    unsafe {
        _ = EnumChildWindows(Some(window_thread), Some(find_match), lparam);
        ctx[0]
    }
}
