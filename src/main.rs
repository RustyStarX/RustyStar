use std::error::Error;
use std::ffi::OsString;

use ahash::AHashSet;
use rustystar::bypass::should_bypass;
use rustystar::logging::log_error;
use rustystar::utils::{process_child_process, toggle_all};
use rustystar::{PID_SENDER, WHITELIST};
use spdlog::{Level, LevelFilter, debug, info};

use rustystar::events::enter_event_loop;
use win32_ecoqos::process::toggle_efficiency_mode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    spdlog::default_logger().set_level_filter(LevelFilter::MoreSevereEqual(Level::Debug));

    let _ = WHITELIST.set(AHashSet::from_iter(
        [
            // System processes
            "explorer.exe",
            "dwm.exe",
            "csrss.exe",
            "svchost.exe",
            "Taskmgr.exe",
            "smss.exe",
            "ChsIME.exe",
            "ctfmon.exe",
            "WUDFRd.exe",
            // Edge is energy aware
            "msedge.exe",
            // UWP special handle
            "ApplicationFrameHost.exe",
        ]
        .map(OsString::from),
    ));

    ctrlc::set_handler(|| {
        info!("received ctrl-c, recovering...");
        _ = toggle_all(None);
        std::process::exit(0);
    })?;

    info!("throtting all processes...");
    tokio::task::spawn_blocking(|| toggle_all(Some(true))).await??;

    let (tx, rx) = kanal::bounded_async(64);
    let _ = PID_SENDER.set(tx.to_sync());

    tokio::task::spawn_blocking(|| {
        let _ = enter_event_loop().inspect_err(log_error);
    });

    info!("listening foreground events...");
    tokio::task::spawn(async move {
        let mut last_pid = None;

        while let Ok(pid) = rx.recv().await {
            debug!("received: {pid}");

            match last_pid {
                // skip boosting
                Some(last) if last == pid => {
                    continue;
                }
                Some(last_pid) => {
                    _ = tokio::task::spawn_blocking(move || {
                        process_child_process(Some(true), last_pid)
                    })
                    .await?;
                }
                None => {}
            }

            _ = tokio::task::spawn_blocking(move || process_child_process(Some(false), pid))
                .await?;
            last_pid = Some(pid);
        }

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    info!("listening new processes...");
    listen_new_proc::listen_process_creation(|listen_new_proc::Process { process_id, name }| {
        let proc = OsString::from(name);
        if should_bypass(proc) {
            return;
        }

        _ = toggle_efficiency_mode(process_id, Some(true));
    })
    .await?;

    Ok(())
}
