use std::error::Error;
use std::ffi::OsString;

use ahash::AHashSet;
use spdlog::{Level, LevelFilter, debug, error, info, warn};
use win32_ecoqos::process::toggle_efficiency_mode;

use rustystar::bypass::whitelisted;
use rustystar::config::Config;
use rustystar::events::enter_event_loop;
use rustystar::logging::log_error;
use rustystar::privilege::try_enable_se_debug_privilege;
use rustystar::utils::{process_child_process, toggle_all};
use rustystar::{PID_SENDER, WHITELIST};

#[compio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    spdlog::default_logger().set_level_filter(LevelFilter::MoreSevereEqual(
        if cfg!(debug_assertions) {
            Level::Debug
        } else {
            Level::Info
        },
    ));

    let os_version = windows_version::OsVersion::current().build;
    match () {
        _ if os_version < 21359 => {
            error!("EcoQoS is not supported on your system, found {os_version} < 21359");
            return Ok(());
        }
        _ if os_version < 22621 => {
            warn!("EcoQoS needs Windows 11 22H2 or newer to be most effective");
        }
        _ => {
            info!("Congratulations! Your system will make best result");
        }
    }

    let config = Config::from_profile()
        .await
        .expect("failed to load configuration!");
    info!("loaded configuration: {config:#?}");
    let Config {
        listen_new_process,
        listen_foreground_events,
        throttle_all_startup,
        system_process,
        whitelist,
    } = config;

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
                debug!("received: {pid}");

                match last_pid {
                    // skip boosting
                    Some(last) if last == pid => {
                        continue;
                    }
                    Some(last_pid) => {
                        _ = compio::runtime::spawn_blocking(move || {
                            process_child_process(Some(true), last_pid)
                        })
                        .await;
                    }
                    None => {}
                }

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
                        if whitelisted(proc_name) {
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
