use std::ffi::OsStr;

use crate::WHITELIST;

pub fn should_bypass(proc_name: impl AsRef<OsStr>) -> bool {
    WHITELIST
        .get()
        .is_some_and(|bypass| bypass.contains(proc_name.as_ref()))
}
