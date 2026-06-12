#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rerun-if-changed=assets/favicon.ico");
    println!("cargo:rerun-if-env-changed=NETSTITCH__BUILD_FULL_VERSION");

    let mut resource = winresource::WindowsResource::new();
    apply_netstitch_metadata(&mut resource, "NetStitch.exe", "NetStitch Desktop");

    if std::path::Path::new("assets/favicon.ico").exists() {
        resource.set_icon("assets/favicon.ico");
    }

    if let Err(error) = resource.compile() {
        panic!("failed to embed windows resource: {error}");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn apply_netstitch_metadata(
    resource: &mut winresource::WindowsResource,
    original_filename: &str,
    file_description: &str,
) {
    let full_version =
        std::env::var("NETSTITCH__BUILD_FULL_VERSION").unwrap_or_else(|_| fallback_full_version());
    let numeric_version = parse_numeric_version(&full_version);
    resource
        .set("CompanyName", "NetStitch")
        .set("ProductName", "NetStitch")
        .set("FileDescription", file_description)
        .set("InternalName", original_filename)
        .set("OriginalFilename", original_filename)
        .set("FileVersion", &full_version)
        .set("ProductVersion", &full_version)
        .set_version_info(winresource::VersionInfo::FILEVERSION, numeric_version)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric_version)
        .set("LegalCopyright", "Copyright (c) 2026 NetStitch")
        .set("Comments", "NetStitch local network endpoint observer");
}

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
