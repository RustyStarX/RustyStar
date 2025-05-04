use std::error::Error;

use spdlog::info;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::HWINEVENTHOOK,
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GetMessageW, GetWindowThreadProcessId, MSG,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

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
            GetWindowThreadProcessId(window_thread, Some(&mut process_id as _));
            if let Some(tx) = PID_SENDER.get() {
                let _ = tx.send(process_id);
            }
        }
    }

    let eventmin = EVENT_SYSTEM_FOREGROUND;
    let eventmax = EVENT_SYSTEM_FOREGROUND;
    let hmodwineventproc = None;
    let pfnwineventproc = Some(
        hook as unsafe extern "system" fn(
            hwineventhook: HWINEVENTHOOK,
            event: u32,
            hwnd: HWND,
            idobject: i32,
            idchild: i32,
            ideventthread: u32,
            dwmseventtime: u32,
        ),
    );
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
