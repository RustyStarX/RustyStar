use thiserror::Error;

pub mod listen;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    IOError(#[from] std::io::Error),
    #[error("wmi: {0}")]
    WMIError(#[from] wmi::WMIError),
}

pub use listen::{Process, listen_process_creation};
