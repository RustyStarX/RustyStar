#![cfg_attr(feature = "hide-to-tray", windows_subsystem = "windows")]

use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use ahash::AHashSet;
use spdlog::sink::FileSink;
use spdlog::{Level, LevelFilter, debug, error, info, trace, warn};
use win32_ecoqos::process::toggle_efficiency_mode;

use rustystar::bypass::whitelisted;
use rustystar::config::{Config, PROJECT_DIR};
use rustystar::events::enter_event_loop;
use rustystar::logging::log_error;
use rustystar::privilege::try_enable_se_debug_privilege;
use rustystar::utils::{ProcTree, process_child_process, toggle_all};
use rustystar::{CURRENT_FOREGROUND_PID, PID_SENDER, WHITELIST};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Shell::{
    QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
};
use windows::core::w;

fn singleton_check() -> Result<bool, Box<dyn Error + Send + Sync>> {
    unsafe {
        CreateMutexW(None, true, w!("RustyStar"))?;
        // According to MSDN, the `CreateMutexW` will not fail, but you have
        // to call `GetLastError` for checking instance
        if GetLastError() == ERROR_ALREADY_EXISTS {
            info!("found existing instance, exiting silently...");
            return Ok(false);
        }
    };
    Ok(true)
}

#[compio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let log_path = PROJECT_DIR
        .as_ref()
        .map(|proj| proj.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let log_file = log_path.join("rustystar.log");

    let logger = spdlog::default_logger().fork_with(|logger| {
        logger.set_level_filter(LevelFilter::MoreSevereEqual(if cfg!(debug_assertions) {
            Level::Debug
        } else {
            Level::Info
        }));

        _ = std::fs::create_dir_all(&log_path);

        let file_level = LevelFilter::MoreSevereEqual(Level::Info);
        logger.sinks_mut().push(Arc::new(
            FileSink::builder()
                .level_filter(file_level)
                .path(&log_file)
                .build()?,
        ));
        logger.set_flush_level_filter(file_level);
        Ok(())
    })?;
    spdlog::set_default_logger(logger);

    if let Ok(false) = singleton_check() {
        return Ok(());
    }

    let os_version = windows_version::OsVersion::current().build;
    match os_version {
        ..21359 => {
            error!("EcoQoS is not supported on your system, found {os_version} < 21359");
            return Ok(());
        }
        21359..22621 => {
            warn!("EcoQoS needs Windows 11 22H2 or newer to be most effective");
        }
        22621.. => {
            info!("Congratulations! Your system will make best result");
        }
    }

    let config = Config::from_profile()
        .await
        .expect("failed to load configuration!");
    info!("loaded configuration: {config:#?}");
    let Config {
        #[cfg_attr(not(feature = "auto-launch"), allow(unused))]
        autostart_on_boot,
        listen_new_process,
        listen_foreground_events,
        throttle_all_startup,
        system_process,
        whitelist,
    } = config;

    #[cfg(feature = "auto-launch")]
    let _ = rustystar::auto_launch::setup_auto_launch(autostart_on_boot);

    info!("initializing whitelist...");
    let _ = WHITELIST.set(AHashSet::from_iter(
        whitelist.into_iter().map(OsString::from),
    ));

    info!("registering Ctrl-C handler...");
    ctrlc::set_handler(|| {
        info!("received ctrl-c, recovering...");
        _ = toggle_all(None);
        std::process::exit(0);
    })?;

    #[cfg(feature = "hide-to-tray")]
    let _ = rustystar::tray::start_tray_service(log_file);

    if system_process {
        match try_enable_se_debug_privilege() {
            Ok(true) => {
                info!("SeDebugPrivilege enabled!");
            }
            Ok(false) => {
                warn!("SeDebugPrivilege enabled, but RustyStar wasn't evelated!");
            }
            Err(e) => {
                warn!("SeDebugPrivilege enable failed: {e}");
            }
        }
    } else {
        info!("skip to enable SeDebugPrivilege");
    }

    if throttle_all_startup {
        info!("throtting all processes...");
        _ = compio::runtime::spawn_blocking(|| toggle_all(Some(true))).await;
    }

    let mut taskset = Vec::new();
    if listen_foreground_events.enabled {
        let (tx, rx) = kanal::bounded_async(64);
        let _ = PID_SENDER.set(tx.to_sync());

        taskset.push(compio::runtime::spawn_blocking(|| {
            let _ = enter_event_loop().inspect_err(log_error);
            Ok(())
        }));

        info!("listening foreground events...");
        taskset.push(compio::runtime::spawn(async move {
            let mut last_pid = None;

            while let Ok(pid) = rx.recv().await {
                trace!("received: {pid}");

                match last_pid {
                    // skip boosting
                    Some(last) if last == pid => {
                        continue;
                    }
                    Some(last_pid) => match unsafe { SHQueryUserNotificationState() } {
                        Ok(QUNS_BUSY) | Ok(QUNS_RUNNING_D3D_FULL_SCREEN) => {
                            debug!("detected full screen app! skip throttling");
                        }
                        _ => {
                            _ = compio::runtime::spawn_blocking(move || {
                                process_child_process(Some(true), last_pid)
                            })
                            .await;
                        }
                    },

                    None => {}
                }

                CURRENT_FOREGROUND_PID.store(pid, Ordering::Release);
                _ = compio::runtime::spawn_blocking(move || {
                    process_child_process(Some(false), pid)
                })
                .await;
                last_pid = Some(pid);
            }

            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }));
    }

    if listen_new_process.enabled {
        let blacklist =
            AHashSet::from_iter(listen_new_process.blacklist.iter().map(OsString::from));
        info!("listening new processes...");
        listen_new_proc::listen_process_creation(
            move |listen_new_proc::Process {
                      process_id, name, ..
                  }| {
                let proc_name = OsString::from(name);
                match listen_new_process.mode {
                    rustystar::config::ListenNewProcessMode::Normal => {
                        if whitelisted(&proc_name) {
                            return;
                        }

                        let current_fg = CURRENT_FOREGROUND_PID.load(Ordering::Acquire);
                        if current_fg != 0
                            && ProcTree::new()
                                .is_ok_and(|proc_tree| proc_tree.is_in_tree(current_fg, process_id))
                        {
                            debug!("skipping {proc_name:?}: foreground process child");
                            return;
                        }
                    }
                    rustystar::config::ListenNewProcessMode::BlacklistOnly => {
                        if !blacklist.contains(&proc_name) {
                            return;
                        }
                    }
                }

                _ = toggle_efficiency_mode(process_id, Some(true));
            },
        )
        .await?;
    }

    if !taskset.is_empty() {
        for task in taskset {
            _ = task.await;
        }
    } else {
        info!("one-shot mode detected! will leave processes throttled");
    }
    Ok(())
}
