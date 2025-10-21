use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
};

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
/// process information from snapshot.
pub struct Process {
    /// win32 process id
    pub process_id: u32,
    /// win32 process id of it's parent
    pub process_parent_id: u32,
    /// win32 process name, null bytes trimmed.
    pub process_name: OsString,
}

/// snapshot of processes
#[derive(Debug)]
pub struct Processes {
    snapshot: HANDLE,
    last_entry: Option<PROCESSENTRY32W>,
}

impl Drop for Processes {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.snapshot) };
    }
}

impl Processes {
    /// try to capture a snapshot of processes.
    pub fn try_new() -> windows_result::Result<Self> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;
        Ok(Self {
            snapshot,
            last_entry: None,
        })
    }
}

impl Iterator for Processes {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.last_entry.is_none();
        let mut entry = self.last_entry.take().unwrap_or(PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        });

        unsafe {
            if first {
                Process32FirstW(self.snapshot, &mut entry as *mut _)
            } else {
                Process32NextW(self.snapshot, &mut entry as *mut _)
            }
        }
        .ok()
        .map(|_| entry)
        .inspect(|entry| self.last_entry = Some(*entry))
        .map(
            |PROCESSENTRY32W {
                 th32ProcessID,
                 th32ParentProcessID,
                 szExeFile,
                 ..
             }| {
                let null = szExeFile
                    .iter()
                    .enumerate()
                    .find_map(|(idx, ch)| if ch == &0 { Some(idx) } else { None })
                    .unwrap_or(szExeFile.len());

                let process_name = OsString::from_wide(&szExeFile[..null]);
                Process {
                    process_id: th32ProcessID,
                    process_parent_id: th32ParentProcessID,
                    process_name,
                }
            },
        )
    }
}
