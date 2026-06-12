#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rerun-if-env-changed=NETSTITCH__BUILD_FULL_VERSION");
    let mut resource = winresource::WindowsResource::new();
    let full_version =
        std::env::var("NETSTITCH__BUILD_FULL_VERSION").unwrap_or_else(|_| fallback_full_version());
    let numeric_version = parse_numeric_version(&full_version);
    resource
        .set("CompanyName", "NetStitch")
        .set("ProductName", "NetStitch")
        .set("FileDescription", "NetStitch network tool module")
        .set("InternalName", "netstitch_tool")
        .set("OriginalFilename", "netstitch_tool.dll")
        .set("FileVersion", &full_version)
        .set("ProductVersion", &full_version)
        .set_version_info(winresource::VersionInfo::FILEVERSION, numeric_version)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric_version)
        .set("LegalCopyright", "Copyright (c) NetStitch");
    let _ = resource.compile();
}

#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn fallback_full_version() -> String {
    format!("{}.1", env!("CARGO_PKG_VERSION"))
}

#[cfg(target_os = "windows")]
fn parse_numeric_version(version: &str) -> u64 {
    let mut parts = version
        .split('.')
        .take(4)
        .map(|segment| segment.trim().parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>();
    while parts.len() < 4 {
        parts.push(0);
    }
    (parts[0] << 48) | (parts[1] << 32) | (parts[2] << 16) | parts[3]
}
