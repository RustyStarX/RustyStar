use std::collections::BTreeMap;

use rustc_hash::FxHashSet;
use spdlog::{debug, info, warn};
use win32_ecoqos::{
    process::toggle_efficiency_mode,
    utils::{Process, Processes},
    windows_result,
};

use crate::bypass::whitelisted;

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

        info!("[{action:^10}] process {main_pid:6}: {process_name:?}");
    } else {
        info!("[{action:^10}] process {main_pid:6}");
    }

    let relations = BTreeMap::from_iter(procs.iter().map(
        |&Process {
             process_id,
             process_parent_id,
             ..
         }| (process_id, process_parent_id),
    ));
    let in_process_tree = move |mut pid: u32| {
        // first case: it self is root process
        if pid == main_pid {
            return true;
        }

        let mut met = FxHashSet::default();
        while let Some(&parent_pid) = relations.get(&pid) {
            if parent_pid == 0 || met.contains(&parent_pid) {
                return false;
            }
            if parent_pid == main_pid {
                return true;
            }

            pid = parent_pid;
            met.insert(pid);
        }

        false
    };

    for Process {
        process_id,
        process_name,
        ..
    } in &procs
    {
        if !in_process_tree(*process_id) {
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
