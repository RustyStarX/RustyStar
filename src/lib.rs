use std::{ffi::OsString, sync::OnceLock};

use ahash::AHashSet;
use kanal::Sender;

pub mod bypass;
pub mod config;
pub mod events;
pub mod logging;
pub mod privilege;
pub mod utils;

pub static PID_SENDER: OnceLock<Sender<u32>> = OnceLock::new();

/// don't touch processes in whitelist
pub static WHITELIST: OnceLock<AHashSet<OsString>> = OnceLock::new();
