use spdlog::warn;
use win32_ecoqos::windows_result;
use windows::Win32::{
    Foundation::{HANDLE, LUID},
    Security::{
        AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_DEBUG_NAME,
        SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

pub fn try_enable_se_debug_privilege() -> windows_result::Result<()> {
    unsafe {
        let processhandle = GetCurrentProcess();
        let mut tokenhandle = HANDLE(std::ptr::null_mut());
        let desiredaccess = TOKEN_ADJUST_PRIVILEGES;
        OpenProcessToken(processhandle, desiredaccess, &mut tokenhandle as _)?;

        if tokenhandle.is_invalid() {
            warn!("failed to open access token!");
        }

        let mut luid = LUID::default();
        let lpname = SE_DEBUG_NAME;
        LookupPrivilegeValueW(None, Some(&lpname), &mut luid)?;

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
        AdjustTokenPrivileges(
            tokenhandle,
            disableallprivileges,
            Some(&newstate),
            0,
            None,
            None,
        )?;
    }
    Ok(())
}
