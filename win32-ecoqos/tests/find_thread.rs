use std::{ffi::OsString, sync::mpsc::channel, thread::sleep, time::Duration};

use win32_ecoqos::utils::{Thread, Threads};
use windows::Win32::System::Threading::GetCurrentThreadId;

#[test]
fn find_thread_by_name_snap() -> windows_result::Result<()> {
    let (tx, rx) = channel();
    let _ = std::thread::Builder::new()
        .name("mythread-snap".to_owned())
        .spawn(move || {
            let tid = unsafe { GetCurrentThreadId() };
            let _ = tx.send(tid);
            loop {
                sleep(Duration::from_secs(5));
            }
        });

    let snapshot = Threads::try_new()?;
    let thread_id = rx.recv().expect("failed to retrieve thread id");
    let thread_name = OsString::from("mythread-snap");
    let threads = snapshot.find_thread_by_name(&thread_name, true);

    assert_eq!(
        threads.collect::<Vec<_>>(),
        vec![Thread {
            thread_id,
            owner_process_id: std::process::id()
        }]
    );

    Ok(())
}
