use std::error::Error;
use std::iter::once;
use std::os::windows::ffi::OsStrExt as _;
use std::path::PathBuf;
use std::time::Duration;

use spdlog::{debug, info};
use tray_item::{IconSource, TrayItem};
use windows::Win32::UI::Shell::{SEE_MASK_INVOKEIDLIST, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use crate::config::Config;
use crate::utils::toggle_all;

fn encode_path(path: &PathBuf) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<u16>>()
}

pub async fn start_tray_service(log_file: PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let icon = IconSource::Resource("icon0");
    let mut tray = TrayItem::new("RustyStar", icon.clone());

    // FIXME: wait for Shell_TrayWnd -> TrayNotifyWnd
    let mut tray = loop {
        debug!("trying to spawn tray icon...");
        if let Ok(tray) = tray {
            break tray;
        }
        tray = TrayItem::new("RustyStar", icon.clone());
        compio::time::sleep(Duration::from_millis(200)).await;
    };

    let config_file = Config::config_path().await?;
    tray.add_menu_item("Open config", move || unsafe {
        let lpfile = encode_path(&config_file);
        let mut execute_info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as _,
            fMask: SEE_MASK_INVOKEIDLIST,
            lpVerb: w!("openas"),
            lpParameters: PCWSTR(std::ptr::null()),
            nShow: SW_SHOWNORMAL.0,
            lpFile: PCWSTR(lpfile.as_ptr()),
            ..Default::default()
        };
        _ = ShellExecuteExW((&mut execute_info) as *mut _);
    })?;
    tray.add_menu_item("Open log", move || unsafe {
        let lpfile = encode_path(&log_file);
        let mut execute_info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as _,
            fMask: SEE_MASK_INVOKEIDLIST,
            lpVerb: w!("openas"),
            lpParameters: PCWSTR(std::ptr::null()),
            nShow: SW_SHOWNORMAL.0,
            lpFile: PCWSTR(lpfile.as_ptr()),
            ..Default::default()
        };
        _ = ShellExecuteExW((&mut execute_info) as *mut _);
    })?;
    tray.add_menu_item("Quit", || {
        info!("received quit signal, recovering...");
        _ = toggle_all(None);
        std::process::exit(0);
    })?;

    std::mem::forget(tray);

    Ok(())
}
