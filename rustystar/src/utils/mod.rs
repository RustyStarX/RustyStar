use std::error::Error;

use spdlog::{debug, warn};
use win32_ecoqos::process::toggle_efficiency_mode;
use win32_ecoqos::utils::{Process, Processes};
use win32_ecoqos::windows_result;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::w;

use crate::bypass::whitelisted;

mod proc_tree;
pub use proc_tree::ProcTree;

pub fn process_child_process(enable: Option<bool>, main_pid: u32) -> windows_result::Result<()> {
    let action = match enable {
        Some(true) => "throtting",
        Some(false) => "boosting",
        None => "recovering",
    };

    let procs = Processes::try_new()?.collect::<Vec<_>>();
    if let Some(Process { process_name, .. }) = procs
        .iter()
        .find(|Process { process_id, .. }| process_id == &main_pid)
    {
        if whitelisted(process_name) {
            debug!("[{action:^10}] skipping {process_name:?}");
            return Ok(());
        }

        debug!("[{action:^10}] process {main_pid:6}: {process_name:?}");
    } else {
        debug!("[{action:^10}] process {main_pid:6}");
    }

    let relations = ProcTree::from(procs.iter());

    for Process {
        process_id,
        process_name,
        ..
    } in &procs
    {
        if !relations.is_in_tree(main_pid, *process_id) {
            continue;
        }
        if whitelisted(process_name) {
            continue;
        }
        if let Err(e) = toggle_efficiency_mode(*process_id, enable) {
            warn!("failed to toggle {process_name:?}: {e}");
        }
    }

    Ok(())
}

pub fn toggle_all(enable: Option<bool>) -> windows_result::Result<()> {
    for Process {
        process_id: pid,
        process_name,
        ..
    } in Processes::try_new()?
    {
        if whitelisted(&process_name) {
            continue;
        }
        if let Err(e) = toggle_efficiency_mode(pid, enable) {
            warn!("failed to toggle {process_name:?}: {e}");
        }
    }

    Ok(())
}

pub fn singleton_check() -> Result<bool, Box<dyn Error + Send + Sync>> {
    unsafe {
        CreateMutexW(None, true, w!("RustyStar"))?;
        // According to MSDN, the `CreateMutexW` will not fail, but you have
        // to call `GetLastError` for checking instance
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return Ok(false);
        }
    };
    Ok(true)
}
