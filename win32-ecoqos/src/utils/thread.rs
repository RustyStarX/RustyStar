use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::OsStringExt,
};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        Threading::{GetThreadDescription, OpenThread, THREAD_QUERY_LIMITED_INFORMATION},
    },
};

#[derive(Debug, PartialEq, Eq)]
/// process information from snapshot.
pub struct Thread {
    /// win32 thread id
    pub thread_id: u32,
    /// owner win32 process id of this thread
    pub owner_process_id: u32,
}

/// Snapshot of win32 threads.
///
/// ```rust
/// use std::ffi::OsString;
/// use win32_ecoqos::utils::Threads;
///
/// let _ = std::thread::Builder::new()
///     .name("hello".into())
///     .spawn(|| loop {
///         std::thread::sleep(std::time::Duration::from_secs(60));
///     });
///
/// let snapshot = Threads::try_new().unwrap();
/// assert!(
///     snapshot
///         .find_thread_by_name(&OsString::from("hello"), true)
///         .next()
///         .is_some()
/// );
/// ```
#[derive(Debug)]
pub struct Threads {
    snapshot: HANDLE,
    last_entry: Option<THREADENTRY32>,
}

impl Drop for Threads {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.snapshot) };
    }
}

impl Thread {
    pub fn get_name(&self) -> windows_result::Result<OsString> {
        unsafe {
            let hthread = OpenThread(THREAD_QUERY_LIMITED_INFORMATION, false, self.thread_id)?;
            let description = GetThreadDescription(hthread)?;

            Ok(OsString::from_wide(description.as_wide()))
        }
    }
}

impl Threads {
    /// create a new snapshop to find threads
    pub fn try_new() -> windows_result::Result<Self> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, std::process::id()) }?;
        Ok(Self {
            snapshot,
            last_entry: None,
        })
    }

    /// find a thread of current process, by it's name
    pub fn find_thread_by_name<'a>(
        self,
        thread_name: &'a OsStr,
        full_match: bool,
    ) -> impl Iterator<Item = Thread> + 'a {
        let process_id = std::process::id();
        self.filter(move |t| t.owner_process_id == process_id)
            .filter(move |t| {
                t.get_name().is_ok_and(|name| {
                    if full_match {
                        &name == thread_name
                    } else {
                        name.to_string_lossy()
                            .contains(thread_name.to_string_lossy().as_ref())
                    }
                })
            })
    }
}

impl Iterator for Threads {
    type Item = Thread;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.last_entry.is_none();
        let mut entry = self.last_entry.take().unwrap_or(THREADENTRY32 {
            dwSize: size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        });

        unsafe {
            if first {
                Thread32First(self.snapshot, &mut entry as *mut _)
            } else {
                Thread32Next(self.snapshot, &mut entry as *mut _)
            }
        }
        .ok()
        .map(|_| entry)
        .inspect(|entry| self.last_entry = Some(*entry))
        .map(
            |THREADENTRY32 {
                 th32ThreadID,
                 th32OwnerProcessID,
                 ..
             }| {
                Thread {
                    thread_id: th32ThreadID,
                    owner_process_id: th32OwnerProcessID,
                }
            },
        )
    }
}
