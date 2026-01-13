use std::env::current_exe;
use std::error::Error;

use spdlog::{error, info};

pub fn setup_auto_launch(autostart_on_boot: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("configuring autostart...");
    let auto_launch = auto_launch::AutoLaunchBuilder::new()
        .set_app_name(env!("CARGO_PKG_NAME"))
        .set_app_path(&current_exe()?.to_string_lossy())
        .build()?;
    if auto_launch.is_enabled()? != autostart_on_boot {
        let re = if autostart_on_boot {
            auto_launch.enable()
        } else {
            auto_launch.disable()
        };

        let action = if autostart_on_boot {
            "enable"
        } else {
            "disable"
        };

        if let Err(e) = re {
            error!("failed to {action} auto-start: {e}");
        } else {
            info!("auto-start {action}d successfully",);
        }
    }

    Ok(())
}
