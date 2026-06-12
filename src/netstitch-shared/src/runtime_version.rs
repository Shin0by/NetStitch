use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

const VERSION_MANIFEST_RELATIVE_PATH: &str = "config/version-manifest.json";
const PACKAGE_REVISION_RELATIVE_PATH: &str = "config/package-revision.txt";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RuntimeVersionManifest {
    #[serde(default)]
    pub package_version: String,
    #[serde(default)]
    pub modules: BTreeMap<String, RuntimeVersionModule>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RuntimeVersionModule {
    #[serde(default)]
    pub artifact: String,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub full_version: String,
}

static RUNTIME_VERSION_MANIFEST: LazyLock<Option<RuntimeVersionManifest>> =
    LazyLock::new(load_runtime_version_manifest);

pub fn runtime_build_version() -> String {
    runtime_module_version("app")
}

pub fn runtime_module_version(module_key: &str) -> String {
    let package_version = RUNTIME_VERSION_MANIFEST
        .as_ref()
        .map(|manifest| manifest.package_version.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    if let Some(version) = RUNTIME_VERSION_MANIFEST
        .as_ref()
        .and_then(|manifest| manifest.modules.get(module_key))
        .map(|module| module.full_version.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return version;
    }

    let package_revision = load_package_revision();
    if let Some(revision) = package_revision {
        return format!("{package_version}.{revision}");
    }

    compile_time_build_version().unwrap_or_else(fallback_full_version)
}

fn load_runtime_version_manifest() -> Option<RuntimeVersionManifest> {
    let exe_dir = env::current_exe().ok()?.parent()?.to_path_buf();
    let manifest_path = exe_dir.join(VERSION_MANIFEST_RELATIVE_PATH);
    let manifest_text = fs::read_to_string(manifest_path).ok()?;
    serde_json::from_str(&manifest_text).ok()
}

fn load_package_revision() -> Option<u64> {
    let exe_dir = env::current_exe().ok()?.parent()?.to_path_buf();
    let revision_path = exe_dir.join(PACKAGE_REVISION_RELATIVE_PATH);
    let revision_text = fs::read_to_string(revision_path).ok()?;
    revision_text.trim().parse::<u64>().ok()
}

fn fallback_full_version() -> String {
    format!("{}.1", env!("CARGO_PKG_VERSION"))
}

fn compile_time_build_version() -> Option<String> {
    option_env!("NETSTITCH__BUILD_FULL_VERSION")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{RuntimeVersionManifest, RuntimeVersionModule, fallback_full_version};
    use std::collections::BTreeMap;

    #[test]
    fn fallback_version_includes_first_revision() {
        assert_eq!(
            fallback_full_version(),
            format!("{}.1", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn manifest_schema_keeps_full_versions_per_module() {
        let manifest = RuntimeVersionManifest {
            package_version: "1.1.0".to_string(),
            modules: BTreeMap::from([(
                "watcher".to_string(),
                RuntimeVersionModule {
                    artifact: "netstitch_watcher.dll".to_string(),
                    revision: 3,
                    full_version: "1.1.0.3".to_string(),
                },
            )]),
        };

        assert_eq!(
            manifest.modules["watcher"].full_version,
            "1.1.0.3".to_string()
        );
    }
}
