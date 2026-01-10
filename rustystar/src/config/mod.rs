use std::error::Error;
use std::path::PathBuf;
use std::sync::LazyLock;

use compio::fs::{self, File};
use compio::io::AsyncReadAtExt;
use directories::ProjectDirs;
use documented::DocumentedFields;
use serde::{Deserialize, Serialize};
use spdlog::warn;

use crate::config::merge::append_comments;

mod merge;

#[derive(Debug, Serialize, Deserialize, DocumentedFields)]
#[serde(default)]
pub struct ListenForegroundEvents {
    /// listen foreground window change events
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ListenNewProcessMode {
    #[default]
    Normal,
    BlacklistOnly,
}

#[derive(Debug, Serialize, Deserialize, DocumentedFields)]
#[serde(default)]
pub struct ListenNewProcess {
    /// listen new process creation
    pub enabled: bool,
    /// blacklist_only: only throttle blacklisted
    /// normal: throttle all new process
    pub mode: ListenNewProcessMode,
    /// blacklist
    pub blacklist: Vec<String>,
}

pub static PROJECT_DIR: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| directories::ProjectDirs::from("io", "RustyStarX", "RustyStar"));

#[derive(Debug, Serialize, Deserialize, DocumentedFields)]
#[serde(default)]
pub struct Config {
    /// setup auto-start (enable/disable)
    ///
    /// Only taking effect if `auto-launch` feature enabled
    pub autostart_on_boot: bool,
    /// monitor new processes and toggle EcoQoS
    pub listen_new_process: ListenNewProcess,
    /// monitor foreground change events
    pub listen_foreground_events: ListenForegroundEvents,
    /// on startup, throttle all processes to EcoQoS
    pub throttle_all_startup: bool,
    /// also taking effect on some `SYSTEM` priviledged process
    pub system_process: bool,
    /// whitelisted process will not be throttled
    pub whitelist: Vec<String>,
}

impl Config {
    pub async fn config_path() -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
        let config_dir = PROJECT_DIR
            .as_ref()
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or(PathBuf::from("."));
        fs::create_dir_all(&config_dir).await?;

        Ok(config_dir.join("config.toml"))
    }

    pub async fn from_profile() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_path = Self::config_path().await?;
        let config = if config_path.exists() {
            let file = File::open(&config_path).await?;
            let result = file.read_to_end_at(Vec::with_capacity(4096), 0).await;
            if !result.is_ok() {
                Err("Failed to read configuration")?;
            }
            toml::from_str(&String::from_utf8_lossy(&result.1))?
        } else {
            warn!("config not existing! falling back to default...");
            Self::default()
        };

        let serialized = append_comments(&toml::to_string_pretty(&config)?)?;
        _ = fs::write(config_path, serialized.trim_start().to_string())
            .await
            .0
            .inspect_err(|e| {
                warn!("failed to write default config: {e}");
            });

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autostart_on_boot: false,
            listen_new_process: ListenNewProcess::default(),
            listen_foreground_events: ListenForegroundEvents::default(),
            throttle_all_startup: true,
            system_process: true,
            whitelist: [
                // ourself
                "RustyStar.exe",
                // System processes
                "explorer.exe",
                // Windows Manager of Windows
                "dwm.exe",
                // CSRSS core process
                "csrss.exe",
                // Windows services process
                "svchost.exe",
                // Task Manager
                "Taskmgr.exe",
                // Session Manager Subsystem
                "smss.exe",
                // Chinese input method
                "ChsIME.exe",
                // Speech-To-Text, Screen keyboard, handwrite input, e.g.
                "ctfmon.exe",
                // Windows User Mode Driver Framework
                "WUDFRd.exe",
                "WUDFHost.exe",
                // Edge is energy aware
                "msedge.exe",
                // UWP special handle
                "ApplicationFrameHost.exe",
                // system itself
                "[System Process]",
                "System",
                "Registry",
                // parent of "services.exe"
                "wininit.exe",
                // parent of "svchost.exe", "wudfhost.exe", e.g.
                "services.exe",
                // Local Security Authority Subsystem Service
                "lsass.exe",
                // part of the Windows Security Center,
                // responsible for monitoring and reporting the security status of your system
                "SecurityHealthService.exe",
            ]
            .map(str::to_string)
            .to_vec(),
        }
    }
}

impl Default for ListenForegroundEvents {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for ListenNewProcess {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ListenNewProcessMode::default(),
            blacklist: vec![],
        }
    }
}
