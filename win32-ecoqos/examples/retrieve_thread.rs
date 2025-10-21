use std::{error::Error, sync::mpsc::channel, thread::sleep, time::Duration};

use win32_ecoqos::thread::{ecoqos_enabled, toggle_efficiency_mode_handle};
use windows::Win32::{
    Foundation::CloseHandle,
    System::Threading::{
        GetCurrentThreadId, OpenThread, THREAD_QUERY_INFORMATION, THREAD_SET_INFORMATION,
    },
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (tx, rx) = channel();
    let _ = std::thread::spawn(move || {
        let thread_id = unsafe { GetCurrentThreadId() };
        let _ = tx.send(thread_id);
        loop {
            println!("Hello world!");
            sleep(Duration::from_secs(5));
        }
    });

    let thread_id = rx.recv().unwrap();

    unsafe {
        let hthread = OpenThread(
            THREAD_SET_INFORMATION | THREAD_QUERY_INFORMATION,
            false,
            thread_id,
        )?;

        toggle_efficiency_mode_handle(hthread, Some(true))?;
        assert!(ecoqos_enabled(hthread)?);
        toggle_efficiency_mode_handle(hthread, Some(false))?;
        assert!(!ecoqos_enabled(hthread)?);

        CloseHandle(hthread)?;
    }

    Ok(())
}
