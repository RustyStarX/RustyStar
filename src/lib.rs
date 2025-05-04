use std::sync::OnceLock;

use kanal::Sender;

pub mod events;
pub mod logging;
pub mod utils;

pub static PID_SENDER: OnceLock<Sender<u32>> = OnceLock::new();
