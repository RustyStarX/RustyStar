let rev_count = (git rev-list --count HEAD) | str trim;
let current_version = (cargo metadata --format-version 1 --no-deps | from json).packages.0.version;
let install_version = $current_version + "-" + $rev_count;

for $target in [aarch64-pc-windows-msvc x86_64-pc-windows-msvc] {
    (
        cargo wix
            --bin-path "C:/Program Files (x86)/WiX Toolset v3.14/bin"
            --name RustyStar
            --target $target
            --install-version $install_version
    )
}