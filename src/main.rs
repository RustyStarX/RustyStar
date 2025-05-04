#![feature(let_chains)]

use std::error::Error;

use rustystar::PID_SENDER;
use rustystar::logging::log_error;
use rustystar::utils::{process_child_process, toggle_all};
use spdlog::{Level, LevelFilter, debug, info};

use rustystar::events::enter_event_loop;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    spdlog::default_logger().set_level_filter(LevelFilter::MoreSevereEqual(Level::Debug));

    info!("throtting all processes...");
    tokio::task::spawn_blocking(|| toggle_all(Some(true))).await??;

    let (tx, rx) = kanal::bounded_async(64);
    let _ = PID_SENDER.set(tx.to_sync());

    tokio::task::spawn_blocking(|| {
        let _ = enter_event_loop().inspect_err(log_error);
    });

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

        _ = tokio::task::spawn_blocking(move || process_child_process(Some(false), pid)).await?;
        last_pid = Some(pid);
    }

    Ok(())
}
