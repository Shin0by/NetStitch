fn main() {
    println!("cargo:rerun-if-env-changed=NETSTITCH__BUILD_FULL_VERSION");
    let full_version = std::env::var("NETSTITCH__BUILD_FULL_VERSION")
        .unwrap_or_else(|_| format!("{}.1", env!("CARGO_PKG_VERSION")));
    println!("cargo:rustc-env=NETSTITCH__BUILD_FULL_VERSION={full_version}");
}
