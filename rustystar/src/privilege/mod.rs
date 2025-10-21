use std::os::windows::io::{FromRawHandle, OwnedHandle};

use win32_ecoqos::windows_result;
use windows::Win32::{
    Foundation::{HANDLE, LUID},
    Security::{
        AdjustTokenPrivileges, GetTokenInformation, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
        SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION,
        TOKEN_PRIVILEGES, TOKEN_QUERY, TokenElevation,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

fn is_elevated(tokenhandle: HANDLE) -> windows_result::Result<bool> {
    let mut tokeninformation = TOKEN_ELEVATION::default();
    let mut needed = 0_u32;

    unsafe {
        GetTokenInformation(
            tokenhandle,
            TokenElevation,
            Some(&mut tokeninformation as *mut _ as _),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut needed as _,
        )
        .map(|_| tokeninformation.TokenIsElevated != 0)
    }
}

fn enable_se_debug(tokenhandle: HANDLE) -> windows_result::Result<()> {
    let mut luid = LUID::default();
    let lpname = SE_DEBUG_NAME;
    unsafe {
        LookupPrivilegeValueW(None, Some(&lpname), &mut luid)?;
    }

    let attributes = SE_PRIVILEGE_ENABLED;
    let privilege = LUID_AND_ATTRIBUTES {
        Luid: luid,
        Attributes: attributes,
    };

    let disableallprivileges = false;
    let newstate = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [privilege],
    };

    unsafe {
        AdjustTokenPrivileges(
            tokenhandle,
            disableallprivileges,
            Some(&newstate),
            0,
            None,
            None,
        )
    }
}

pub fn try_enable_se_debug_privilege() -> windows_result::Result<bool> {
    unsafe {
        let processhandle = GetCurrentProcess();
        let mut tokenhandle = HANDLE(std::ptr::null_mut());
        let desiredaccess = TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY;

        OpenProcessToken(processhandle, desiredaccess, &mut tokenhandle as _)?;

        let _defer = OwnedHandle::from_raw_handle(tokenhandle.0);
        enable_se_debug(tokenhandle)?;
        is_elevated(tokenhandle)
    }
}
