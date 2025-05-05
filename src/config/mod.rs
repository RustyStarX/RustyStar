use std::{error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use spdlog::warn;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ListenForegroundEvents {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ListenNewProcessMode {
    #[default]
    Normal,
    BlacklistOnly,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ListenNewProcess {
    pub enabled: bool,
    pub mode: ListenNewProcessMode,
    pub blacklist: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub listen_new_process: ListenNewProcess,
    pub listen_foreground_events: ListenForegroundEvents,
    pub throttle_all_startup: bool,
    pub system_process: bool,
    pub whitelist: Vec<String>,
}

impl Config {
    pub fn from_profile() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_dir = directories::ProjectDirs::from("io", "RustyStarX", "RustyStar")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or(PathBuf::from("."));
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        if config_path.exists() {
            Ok(toml::from_str(&fs::read_to_string(&config_path)?)?)
        } else {
            warn!("config not existing! falling back to default...");
            let config = Self::default();
            let serialized = toml::to_string_pretty(&config)?;
            _ = fs::write(config_path, serialized).inspect_err(|e| {
                warn!("failed to write default config: {e}");
            });
            Ok(config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
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
