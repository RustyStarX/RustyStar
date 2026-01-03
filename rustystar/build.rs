use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    windows_exe_info::versioninfo::VersionInfo::from_cargo_env_ex(
        None,
        Some("RustyStar"),
        Some("RustyStar"),
        None,
    )
    .link()?;
    windows_exe_info::icon::icon_ico("res/rustystar.ico");

    Ok(())
}
