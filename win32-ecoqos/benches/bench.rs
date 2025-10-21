#![feature(test)]

use win32_ecoqos::process::{toggle_efficiency_mode, toggle_efficiency_mode_handle};
use windows::Win32::{
    Foundation::CloseHandle,
    System::Threading::{GetCurrentProcess, GetCurrentThreadId},
};

extern crate test;

#[bench]
fn disable_ecoqos_pid(b: &mut test::Bencher) {
    let pid = std::process::id();
    b.iter(|| toggle_efficiency_mode(pid, Some(false)));
}

#[bench]
fn disable_ecoqos_handle(b: &mut test::Bencher) {
    let hprocess = unsafe { GetCurrentProcess() };
    b.iter(|| unsafe { toggle_efficiency_mode_handle(hprocess, Some(false)) });
    let _ = unsafe { CloseHandle(hprocess) };
}

#[cfg(feature = "find_process")]
#[bench]
fn list_process(b: &mut test::Bencher) {
    use win32_ecoqos::utils::Processes;
    b.iter(|| Processes::try_new().unwrap().count());
}

#[cfg(feature = "find_thread")]
#[bench]
fn list_thread(b: &mut test::Bencher) {
    use win32_ecoqos::utils::Threads;

    b.iter(|| Threads::try_new().unwrap().count());
}

#[cfg(feature = "find_thread")]
#[bench]
fn find_thread_by_name(b: &mut test::Bencher) {
    use std::time::Duration;

    use win32_ecoqos::utils::Threads;

    b.iter(|| {
        let _ = std::thread::Builder::new()
            .name("bench-find".into())
            .spawn(|| loop {
                std::thread::sleep(Duration::from_secs(60));
            });

        Threads::try_new()
            .unwrap()
            .find_thread_by_name("bench-find".as_ref(), true)
            .next()
    });
}

#[bench]
fn send_thread_id_back(b: &mut test::Bencher) {
    use std::time::Duration;

    b.iter(|| {
        let (tx, rx) = oneshot::channel();
        let _ = std::thread::Builder::new().spawn(|| {
            let thread_id = unsafe { GetCurrentThreadId() };
            let _ = tx.send(thread_id);
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        });

        rx.recv().unwrap()
    });
}
