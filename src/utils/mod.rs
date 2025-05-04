use spdlog::info;
use win32_ecoqos::{
    process::toggle_efficiency_mode,
    utils::{Process, Processes},
    windows_result,
};

use crate::{bypass::should_bypass, logging::log_warn};

pub fn process_child_process(enable: Option<bool>, pid: u32) -> windows_result::Result<()> {
    let action = match enable {
        Some(true) => "throtting",
        Some(false) => "boosting ",
        None => "recovering",
    };

    let procs = Processes::try_new()?
        .filter(
            |&Process {
                 process_id,
                 process_parent_id,
                 ..
             }| { process_id == pid || process_parent_id == pid },
        )
        .collect::<Vec<_>>();

    if let Some(Process { process_name, .. }) = procs
        .iter()
        .find(|Process { process_id, .. }| process_id == &pid)
    {
        if should_bypass(process_name) {
            info!("skipping whitelisted process: {process_name:?}");
            return Ok(());
        }

        info!("{action} process {pid:6}: {process_name:?}");
    } else {
        info!("{action} process {pid:6}");
    }

    for Process { process_id, .. } in procs {
        _ = toggle_efficiency_mode(process_id, enable).inspect_err(log_warn);
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
        if should_bypass(process_name) {
            continue;
        }
        _ = toggle_efficiency_mode(pid, enable);
    }

    Ok(())
}
