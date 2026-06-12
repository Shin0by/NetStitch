use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

pub mod modules;

mod icons;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    Macos,
    Linux,
}

impl TargetOs {
    pub fn current() -> Option<Self> {
        if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "macos") {
            Some(Self::Macos)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else {
            None
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessAlias {
    pub os: TargetOs,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscoverySource {
    pub os: TargetOs,
    pub kind: &'static str,
    pub detail: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorManifest {
    pub id: &'static str,
    pub display_name: &'static str,
    pub icon_key: &'static str,
    pub icon_ico: &'static [u8],
    pub icon_svg: &'static [u8],
    pub process_names: &'static [&'static str],
    pub process_aliases: &'static [ProcessAlias],
    pub discovery_sources: &'static [DiscoverySource],
    pub manual_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedApp {
    pub connector_id: &'static str,
    pub display_name: &'static str,
    pub icon_key: &'static str,
    pub icon_ico: &'static [u8],
    pub icon_svg: &'static [u8],
    pub exe_path: PathBuf,
    pub process_name: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppFileMetadata {
    pub company_name: Option<String>,
    pub product_name: Option<String>,
}

pub trait AppConnector {
    fn manifest(&self) -> ConnectorManifest;
    fn discover(&self) -> Vec<DetectedApp>;
}

pub fn manifests() -> Vec<ConnectorManifest> {
    active_connectors()
        .into_iter()
        .map(|connector| connector.manifest())
        .collect()
}

pub fn discover_installed_apps() -> Vec<DetectedApp> {
    discover_from_connectors(active_connectors())
}

pub fn discover_installed_apps_from_dir(module_dir: impl AsRef<Path>) -> Vec<DetectedApp> {
    let mut connectors = external_modules::load_connectors(module_dir.as_ref());
    connectors.push(Box::new(modules::manual::ManualPathConnector));
    discover_from_connectors(connectors)
}

fn discover_from_connectors(connectors: Vec<Box<dyn AppConnector>>) -> Vec<DetectedApp> {
    let mut seen_paths = BTreeSet::new();
    let mut apps = Vec::new();

    for connector in connectors {
        for app in connector.discover() {
            let normalized = normalize_path_key(&app.exe_path);
            if seen_paths.insert(normalized) {
                apps.push(app);
            }
        }
    }

    apps
}

pub fn default_icon_svg() -> &'static [u8] {
    icons::DEFAULT_SVG
}

pub fn app_file_metadata(exe_path: &Path) -> AppFileMetadata {
    platform_app_file_metadata(exe_path)
}

fn active_connectors() -> Vec<Box<dyn AppConnector>> {
    if let Some(module_dir) = runtime_module_dir() {
        let mut connectors = external_modules::load_connectors(&module_dir);
        connectors.push(Box::new(modules::manual::ManualPathConnector));
        return connectors;
    }

    modules::all_connectors()
}

fn runtime_module_dir() -> Option<PathBuf> {
    if let Some(explicit) = env_path("NETSTITCH__CONNECTORS_DIR") {
        return explicit.is_dir().then_some(explicit);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join("apps");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    env::current_dir()
        .ok()
        .map(|cwd| cwd.join("apps"))
        .filter(|candidate| candidate.is_dir())
}

fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

pub(crate) fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(crate) fn existing_file(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

#[cfg(windows)]
fn platform_app_file_metadata(exe_path: &Path) -> AppFileMetadata {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW};

    let wide_path = exe_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let mut handle = 0u32;
    let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
    if size == 0 {
        return AppFileMetadata::default();
    }
    let mut data = vec![0u8; size as usize];
    let ok = unsafe {
        GetFileVersionInfoW(
            wide_path.as_ptr(),
            0,
            size,
            data.as_mut_ptr().cast::<std::ffi::c_void>(),
        )
    };
    if ok == 0 {
        return AppFileMetadata::default();
    }

    for lang_codepage in version_translations(&data)
        .into_iter()
        .chain(["040904b0".to_string(), "040904e4".to_string()])
    {
        let company_name = version_string(&data, &lang_codepage, "CompanyName");
        let product_name = version_string(&data, &lang_codepage, "ProductName");
        if company_name.is_some() || product_name.is_some() {
            return AppFileMetadata {
                company_name,
                product_name,
            };
        }
    }

    AppFileMetadata::default()
}

#[cfg(not(windows))]
fn platform_app_file_metadata(_exe_path: &Path) -> AppFileMetadata {
    AppFileMetadata::default()
}

#[cfg(windows)]
fn version_translations(data: &[u8]) -> Vec<String> {
    use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

    let sub_block = version_wide_null("\\VarFileInfo\\Translation");
    let mut buffer = std::ptr::null_mut::<std::ffi::c_void>();
    let mut len = 0u32;
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast::<std::ffi::c_void>(),
            sub_block.as_ptr(),
            &mut buffer,
            &mut len,
        )
    };
    if ok == 0 || buffer.is_null() || len < 4 {
        return Vec::new();
    }
    let words = unsafe { std::slice::from_raw_parts(buffer.cast::<u16>(), (len as usize) / 2) };
    words
        .chunks_exact(2)
        .map(|pair| format!("{:04x}{:04x}", pair[0], pair[1]))
        .collect()
}

#[cfg(windows)]
fn version_string(data: &[u8], lang_codepage: &str, key: &str) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

    let sub_block = version_wide_null(&format!("\\StringFileInfo\\{lang_codepage}\\{key}"));
    let mut buffer = std::ptr::null_mut::<std::ffi::c_void>();
    let mut len = 0u32;
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast::<std::ffi::c_void>(),
            sub_block.as_ptr(),
            &mut buffer,
            &mut len,
        )
    };
    if ok == 0 || buffer.is_null() || len == 0 {
        return None;
    }
    let raw = unsafe { std::slice::from_raw_parts(buffer.cast::<u16>(), len as usize) };
    let end = raw
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(raw.len());
    let value = String::from_utf16_lossy(&raw[..end]).trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(windows)]
fn version_wide_null(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn app_path_candidates_from_values(
    exe_name: &str,
    default_value: Option<&str>,
    path_value: Option<&str>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let default_path = default_value
        .and_then(normalized_registry_value)
        .map(PathBuf::from);

    if let Some(path) = default_path.as_ref() {
        candidates.push(path.clone());
    }

    let fallback_exe_name = default_path
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(exe_name);

    if let Some(path_value) = path_value.and_then(normalized_registry_value) {
        for entry in path_value.split(';').filter_map(normalized_registry_value) {
            candidates.push(PathBuf::from(entry).join(fallback_exe_name));
        }
    }

    dedup_paths(candidates)
}

pub(crate) fn registry_app_path_candidates(exe_name: &str) -> Vec<PathBuf> {
    registry::app_path_candidates(exe_name)
}

pub(crate) fn windows_store_package_candidates(
    package_prefixes: &[&str],
    exe_names: &[&str],
) -> Vec<PathBuf> {
    let Some(program_files) = env_path("ProgramFiles") else {
        return Vec::new();
    };

    discover_package_exes(
        Some(program_files.join("WindowsApps")),
        package_prefixes,
        exe_names,
    )
}

pub(crate) fn macos_application_bundle_candidates(
    bundle_names: &[&str],
    executable_names: &[&str],
) -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(home) = env_path("HOME") {
        roots.push(home.join("Applications"));
    }

    let candidates = roots.into_iter().flat_map(|root| {
        discover_macos_bundles(Some(root), bundle_names, executable_names).into_iter()
    });

    dedup_paths(candidates)
}

pub(crate) fn linux_application_candidates(
    desktop_ids: &[&str],
    executable_names: &[&str],
) -> Vec<PathBuf> {
    let desktop_candidates = linux_desktop_entry_candidates(desktop_ids, executable_names);
    let path_candidates = linux_command_candidates(executable_names);
    dedup_paths(desktop_candidates.into_iter().chain(path_candidates))
}

pub(crate) fn discover_first_existing(
    candidates: impl IntoIterator<Item = PathBuf>,
) -> Vec<PathBuf> {
    candidates
        .into_iter()
        .filter_map(existing_file)
        .take(1)
        .collect()
}

pub(crate) fn discover_versioned_child(
    root: Option<PathBuf>,
    child_prefix: &str,
    exe_name: &str,
) -> Vec<PathBuf> {
    let Some(root) = root else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    let mut candidates = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(child_prefix))
        })
        .map(|path| path.join(exe_name))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.reverse();
    candidates.into_iter().take(1).collect()
}

pub(crate) fn discover_package_exes(
    root: Option<PathBuf>,
    package_prefixes: &[&str],
    exe_names: &[&str],
) -> Vec<PathBuf> {
    let Some(root) = root else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    let normalized_prefixes = package_prefixes
        .iter()
        .map(|prefix| prefix.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let normalized_exe_names = exe_names
        .iter()
        .map(|exe_name| exe_name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let mut package_dirs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_ascii_lowercase())
                .is_some_and(|name| {
                    normalized_prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
                })
        })
        .collect::<Vec<_>>();

    package_dirs.sort();
    package_dirs.reverse();

    let mut candidates = Vec::new();
    for package_dir in package_dirs {
        for exe_name in &normalized_exe_names {
            if let Some(path) = find_exe_in_package_dir(&package_dir, exe_name, 4) {
                candidates.push(path);
                break;
            }
        }
    }

    dedup_paths(candidates)
}

pub(crate) fn discover_macos_bundles(
    root: Option<PathBuf>,
    bundle_names: &[&str],
    executable_names: &[&str],
) -> Vec<PathBuf> {
    let Some(root) = root else {
        return Vec::new();
    };

    let candidates = bundle_names.iter().flat_map(|bundle_name| {
        let bundle = root.join(format!("{bundle_name}.app"));
        executable_names
            .iter()
            .map(move |exe_name| bundle.join("Contents").join("MacOS").join(exe_name))
    });

    discover_first_existing(candidates)
}

pub(crate) fn linux_desktop_entry_candidates(
    desktop_ids: &[&str],
    executable_names: &[&str],
) -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from("/var/lib/snapd/desktop/applications"),
    ];
    if let Some(home) = env_path("HOME") {
        roots.push(home.join(".local").join("share").join("applications"));
        roots.push(
            home.join(".local")
                .join("share")
                .join("flatpak")
                .join("exports")
                .join("share")
                .join("applications"),
        );
    }

    let mut candidates = Vec::new();
    for root in roots {
        for desktop_id in desktop_ids {
            let desktop_file = root.join(desktop_id);
            if !desktop_file.is_file() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&desktop_file) {
                candidates.extend(desktop_entry_exec_candidates(
                    &content,
                    executable_names,
                    &path_dirs(),
                ));
            }
        }
    }

    dedup_paths(candidates)
}

pub(crate) fn linux_command_candidates(executable_names: &[&str]) -> Vec<PathBuf> {
    let path_dirs = path_dirs();
    dedup_paths(
        executable_names
            .iter()
            .filter_map(|name| resolve_command(name, &path_dirs))
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn desktop_entry_exec_candidates(
    content: &str,
    fallback_executable_names: &[&str],
    path_dirs: &[PathBuf],
) -> Vec<PathBuf> {
    desktop_entry_exec_candidates_with_flatpak_roots(
        content,
        fallback_executable_names,
        path_dirs,
        &flatpak_app_roots(),
    )
}

fn desktop_entry_exec_candidates_with_flatpak_roots(
    content: &str,
    fallback_executable_names: &[&str],
    path_dirs: &[PathBuf],
    flatpak_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for key in ["TryExec", "Exec"] {
        for value in desktop_entry_values(content, key) {
            if let Some(flatpak) = parse_flatpak_exec(value) {
                candidates.extend(flatpak_app_binary_candidates(
                    &flatpak.app_id,
                    &flatpak.command,
                    flatpak_roots,
                ));
                if let Some(path) = resolve_command(&flatpak.command, path_dirs) {
                    candidates.push(path);
                }
                continue;
            }

            let Some(command) = first_exec_token(value) else {
                continue;
            };
            if Path::new(command).is_absolute() {
                candidates.push(PathBuf::from(command));
            } else if let Some(path) = resolve_command(command, path_dirs) {
                candidates.push(path);
            }
        }
    }

    if candidates.is_empty() {
        for name in fallback_executable_names {
            if let Some(path) = resolve_command(name, path_dirs) {
                candidates.push(path);
            }
        }
    }

    discover_first_existing(candidates)
}

#[derive(Debug, PartialEq, Eq)]
struct FlatpakExec {
    app_id: String,
    command: String,
}

fn parse_flatpak_exec(value: &str) -> Option<FlatpakExec> {
    let tokens = desktop_exec_tokens(value);
    let flatpak_index = tokens.iter().position(|token| {
        Path::new(token)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "flatpak")
    })?;
    if tokens.get(flatpak_index + 1).map(String::as_str) != Some("run") {
        return None;
    }

    let mut command = None;
    let mut app_id = None;
    let mut index = flatpak_index + 2;
    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(value) = token.strip_prefix("--command=") {
            command = Some(value.to_string());
        } else if token == "--command" {
            if let Some(value) = tokens.get(index + 1) {
                command = Some(value.clone());
                index += 1;
            }
        } else if !token.starts_with('-')
            && !token.starts_with('%')
            && !token.starts_with("@@")
            && token.contains('.')
        {
            app_id = Some(token.clone());
        }
        index += 1;
    }

    Some(FlatpakExec {
        app_id: app_id?,
        command: command?,
    })
}

fn desktop_exec_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in value.trim().chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn flatpak_app_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/var/lib/flatpak/app")];
    if let Some(home) = env_path("HOME") {
        roots.push(
            home.join(".local")
                .join("share")
                .join("flatpak")
                .join("app"),
        );
    }
    roots
}

fn flatpak_app_binary_candidates(
    app_id: &str,
    command: &str,
    flatpak_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for root in flatpak_roots {
        let app_root = root.join(app_id);
        for arch_dir in child_dirs(&app_root) {
            for branch_dir in child_dirs(&arch_dir) {
                for commit_dir in child_dirs(&branch_dir) {
                    candidates.push(commit_dir.join("files").join("bin").join(command));
                }
            }
        }
    }
    candidates.sort();
    candidates.reverse();
    discover_first_existing(candidates)
}

fn child_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = std::fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            path.is_dir().then_some(path)
        })
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn desktop_entry_values<'a>(content: &'a str, key: &str) -> impl Iterator<Item = &'a str> {
    let prefix = format!("{key}=");
    content
        .lines()
        .map(str::trim)
        .filter_map(move |line| line.strip_prefix(&prefix).map(str::trim))
}

fn first_exec_token(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix('"') {
        return rest.split('"').next().filter(|token| !token.is_empty());
    }

    trimmed
        .split_whitespace()
        .next()
        .filter(|token| !token.starts_with('%'))
}

fn resolve_command(command: &str, path_dirs: &[PathBuf]) -> Option<PathBuf> {
    let command_path = Path::new(command);
    if command_path.is_absolute() {
        return command_path.is_file().then(|| command_path.to_path_buf());
    }

    path_dirs
        .iter()
        .map(|dir| dir.join(command))
        .find(|path| path.is_file())
}

fn path_dirs() -> Vec<PathBuf> {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default()
}

fn find_exe_in_package_dir(
    root: &Path,
    normalized_exe_name: &str,
    max_depth: usize,
) -> Option<PathBuf> {
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case(normalized_exe_name))
            {
                return Some(path);
            }

            if depth < max_depth && path.is_dir() {
                stack.push((path, depth + 1));
            }
        }
    }

    None
}

pub(crate) fn detected(
    connector_id: &'static str,
    display_name: &'static str,
    icon_key: &'static str,
    icon_ico: &'static [u8],
    icon_svg: &'static [u8],
    exe_path: PathBuf,
    process_name: &'static str,
) -> DetectedApp {
    DetectedApp {
        connector_id,
        display_name,
        icon_key,
        icon_ico,
        icon_svg,
        exe_path,
        process_name,
    }
}

fn dedup_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();

    for path in paths {
        if seen.insert(normalize_path_key(&path)) {
            deduped.push(path);
        }
    }

    deduped
}

#[cfg(any(target_os = "windows", test))]
fn normalized_registry_value(value: &str) -> Option<&str> {
    let trimmed = value.trim().trim_matches('"').trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

mod external_modules {
    use crate::{
        AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
        detected, discover_first_existing, icons, linux_application_candidates,
        macos_application_bundle_candidates, registry_app_path_candidates,
        windows_store_package_candidates,
    };
    use std::collections::BTreeMap;
    use std::env;
    use std::path::{Path, PathBuf};

    pub(super) fn load_connectors(module_dir: &Path) -> Vec<Box<dyn AppConnector>> {
        let Ok(entries) = std::fs::read_dir(module_dir) else {
            return Vec::new();
        };

        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
            })
            .collect::<Vec<_>>();
        paths.sort();

        paths
            .into_iter()
            .filter_map(|path| match RuntimeModule::from_file(&path) {
                Ok(module) => Some(module),
                Err(error) => {
                    eprintln!("NetStitch connector skipped: {error}");
                    None
                }
            })
            .filter(|module| module.enabled)
            .map(|module| Box::new(RuntimeConnector::new(module)) as Box<dyn AppConnector>)
            .collect()
    }

    #[derive(Clone, Debug, Default)]
    struct RuntimeModule {
        enabled: bool,
        id: String,
        display_name: String,
        icon_key: String,
        process_names: Vec<String>,
        aliases: Vec<RuntimeAlias>,
        discovery_sources: Vec<RuntimeDiscoverySource>,
        discovery_rules: Vec<RuntimeDiscoveryRule>,
        manual_only: bool,
    }

    #[derive(Clone, Debug)]
    struct RuntimeAlias {
        os: TargetOs,
        name: String,
    }

    #[derive(Clone, Debug)]
    struct RuntimeDiscoverySource {
        os: TargetOs,
        kind: String,
        detail: String,
    }

    #[derive(Clone, Debug, Default)]
    struct RuntimeDiscoveryRule {
        os: Option<TargetOs>,
        kind: String,
        value: Option<String>,
        root_env: Option<String>,
        relative_path: Option<String>,
        package_prefixes: Vec<String>,
        exe_names: Vec<String>,
        bundle_names: Vec<String>,
        executable_names: Vec<String>,
        desktop_ids: Vec<String>,
    }

    #[derive(Clone)]
    struct RuntimeConnector {
        manifest: ConnectorManifest,
        discovery_rules: Vec<RuntimeDiscoveryRule>,
    }

    impl RuntimeConnector {
        fn new(module: RuntimeModule) -> Self {
            let icon_ico = built_in_icon_for_key(&module.icon_key);
            let icon_svg = built_in_icon_svg_for_key(&module.icon_key);
            let process_aliases = module
                .aliases
                .into_iter()
                .map(|alias| ProcessAlias {
                    os: alias.os,
                    name: leak_string(alias.name),
                })
                .collect::<Vec<_>>();
            let discovery_sources = module
                .discovery_sources
                .into_iter()
                .map(|source| DiscoverySource {
                    os: source.os,
                    kind: leak_string(source.kind),
                    detail: leak_string(source.detail),
                })
                .collect::<Vec<_>>();

            Self {
                manifest: ConnectorManifest {
                    id: leak_string(module.id),
                    display_name: leak_string(module.display_name),
                    icon_key: leak_string(module.icon_key),
                    icon_ico,
                    icon_svg,
                    process_names: leak_slice(
                        module.process_names.into_iter().map(leak_string).collect(),
                    ),
                    process_aliases: leak_slice(process_aliases),
                    discovery_sources: leak_slice(discovery_sources),
                    manual_only: module.manual_only,
                },
                discovery_rules: module.discovery_rules,
            }
        }

        fn process_name_for_current_os(&self) -> &'static str {
            if let Some(current_os) = TargetOs::current() {
                if let Some(alias) = self
                    .manifest
                    .process_aliases
                    .iter()
                    .find(|alias| alias.os == current_os)
                {
                    return alias.name;
                }
            }

            self.manifest
                .process_names
                .first()
                .copied()
                .unwrap_or(self.manifest.display_name)
        }

        fn process_name_for_path(&self, path: &Path) -> &'static str {
            let Some(current_os) = TargetOs::current() else {
                return self.process_name_for_current_os();
            };
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                return self.process_name_for_current_os();
            };

            self.manifest
                .process_aliases
                .iter()
                .find(|alias| alias.os == current_os && alias.name == file_name)
                .map(|alias| alias.name)
                .unwrap_or_else(|| self.process_name_for_current_os())
        }
    }

    impl AppConnector for RuntimeConnector {
        fn manifest(&self) -> ConnectorManifest {
            self.manifest.clone()
        }

        fn discover(&self) -> Vec<DetectedApp> {
            if self.manifest.manual_only {
                return Vec::new();
            }

            self.discovery_rules
                .iter()
                .flat_map(discover_rule)
                .map(|path| {
                    let process_name = self.process_name_for_path(&path);
                    detected(
                        self.manifest.id,
                        self.manifest.display_name,
                        self.manifest.icon_key,
                        self.manifest.icon_ico,
                        self.manifest.icon_svg,
                        path,
                        process_name,
                    )
                })
                .collect()
        }
    }

    impl RuntimeModule {
        fn from_file(path: &Path) -> Result<Self, String> {
            let content = std::fs::read_to_string(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            parse_module(&content)
        }
    }

    fn parse_module(content: &str) -> Result<RuntimeModule, String> {
        let tables = parse_tables(content)?;
        let mut module = RuntimeModule {
            enabled: true,
            ..RuntimeModule::default()
        };

        if let Some(root) = tables.get("root").and_then(|items| items.first()) {
            module.enabled = root.bool("enabled").unwrap_or(true);
            module.id = root.required_string("id")?;
            module.display_name = root.required_string("display_name")?;
            module.icon_key = root
                .string("icon_key")
                .unwrap_or_else(|| module.id.replace('-', "_"));
            module.manual_only = root.bool("manual_only").unwrap_or(false);
            module.process_names = root.string_array("process_names").unwrap_or_default();
        }

        for table in tables.get("process_aliases").into_iter().flatten() {
            module.aliases.push(RuntimeAlias {
                os: table.required_os()?,
                name: table.required_string("name")?,
            });
        }

        for table in tables.get("discovery_sources").into_iter().flatten() {
            module.discovery_sources.push(RuntimeDiscoverySource {
                os: table.required_os()?,
                kind: table.required_string("kind")?,
                detail: table.required_string("detail")?,
            });
        }

        for table in tables.get("discovery").into_iter().flatten() {
            module.discovery_rules.push(RuntimeDiscoveryRule {
                os: table.os(),
                kind: table.required_string("kind")?,
                value: table.value_or_path(),
                root_env: table.string("root_env"),
                relative_path: table.string("relative_path"),
                package_prefixes: table.string_array("package_prefixes").unwrap_or_default(),
                exe_names: table.string_array("exe_names").unwrap_or_default(),
                bundle_names: table.string_array("bundle_names").unwrap_or_default(),
                executable_names: table.string_array("executable_names").unwrap_or_default(),
                desktop_ids: table.string_array("desktop_ids").unwrap_or_default(),
            });
        }
        normalize_discovery_rules(&mut module.discovery_rules);
        if module.discovery_rules.is_empty() {
            module.discovery_rules = module
                .discovery_sources
                .iter()
                .filter_map(discovery_source_to_rule)
                .collect();
            normalize_discovery_rules(&mut module.discovery_rules);
        }

        if module.id.trim().is_empty() {
            return Err("module id is required".to_string());
        }
        if module.display_name.trim().is_empty() {
            return Err("display_name is required".to_string());
        }

        Ok(module)
    }

    fn discover_rule(rule: &RuntimeDiscoveryRule) -> Vec<PathBuf> {
        if !rule_applies_to_current_os(rule.os) {
            return Vec::new();
        }

        match rule.kind.as_str() {
            "app_paths" => rule
                .value
                .as_deref()
                .map(registry_app_path_candidates)
                .unwrap_or_default(),
            "fixed_path" => rule
                .value
                .as_deref()
                .map(expand_known_path)
                .filter(|path| path.is_file())
                .into_iter()
                .collect(),
            "known_path" => {
                let Some(root_env) = rule.root_env.as_deref() else {
                    return Vec::new();
                };
                let Some(relative_path) = rule.relative_path.as_deref() else {
                    return Vec::new();
                };
                let Some(root) = env::var_os(root_env).map(PathBuf::from) else {
                    return Vec::new();
                };
                discover_known_path(&root, relative_path)
            }
            "store_msix" => {
                let prefixes = rule
                    .package_prefixes
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let exe_names = rule
                    .exe_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                windows_store_package_candidates(&prefixes, &exe_names)
            }
            "macos_bundle" => {
                let bundle_names = rule
                    .bundle_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let executable_names = rule
                    .executable_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                macos_application_bundle_candidates(&bundle_names, &executable_names)
            }
            "linux_desktop" => {
                let desktop_ids = rule
                    .desktop_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let executable_names = rule
                    .executable_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                linux_application_candidates(&desktop_ids, &executable_names)
            }
            "linux_path" => {
                let executable_names = rule
                    .executable_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                linux_application_candidates(&[], &executable_names)
            }
            _ => Vec::new(),
        }
    }

    fn normalize_discovery_rules(rules: &mut [RuntimeDiscoveryRule]) {
        for rule in rules {
            let Some(value) = rule.value.as_deref() else {
                continue;
            };
            if rule.kind == "known_path"
                && rule.root_env.is_none()
                && rule.relative_path.is_none()
                && looks_like_direct_path(value)
            {
                rule.kind = "fixed_path".to_string();
            }
        }
    }

    fn discovery_source_to_rule(source: &RuntimeDiscoverySource) -> Option<RuntimeDiscoveryRule> {
        let detail = source.detail.trim();
        if detail.is_empty() {
            return None;
        }

        let mut rule = RuntimeDiscoveryRule {
            os: Some(source.os),
            kind: source.kind.clone(),
            value: Some(detail.to_string()),
            ..RuntimeDiscoveryRule::default()
        };

        match rule.kind.as_str() {
            "app_paths" | "fixed_path" => Some(rule),
            "known_path" if looks_like_direct_path(detail) => {
                rule.kind = "fixed_path".to_string();
                Some(rule)
            }
            _ => None,
        }
    }

    fn looks_like_direct_path(value: &str) -> bool {
        let value = value.trim();
        value.starts_with('%')
            || value.starts_with('/')
            || value.starts_with('~')
            || value
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b':' && (pair[1] == b'\\' || pair[1] == b'/'))
    }

    fn discover_known_path(root: &Path, relative_path: &str) -> Vec<PathBuf> {
        let normalized = relative_path.replace('\\', "/");
        let segments = normalized
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        let candidates = expand_segments(vec![root.to_path_buf()], &segments);
        discover_first_existing(candidates)
    }

    fn expand_segments(roots: Vec<PathBuf>, segments: &[&str]) -> Vec<PathBuf> {
        let Some((segment, rest)) = segments.split_first() else {
            return roots;
        };

        let mut next = Vec::new();
        for root in roots {
            if segment.contains('*') {
                if let Ok(entries) = std::fs::read_dir(&root) {
                    let mut matches = entries
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .filter(|path| {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| wildcard_match(segment, name))
                        })
                        .collect::<Vec<_>>();
                    matches.sort();
                    matches.reverse();
                    next.extend(matches);
                }
            } else {
                next.push(root.join(segment));
            }
        }

        expand_segments(next, rest)
    }

    fn wildcard_match(pattern: &str, value: &str) -> bool {
        let pattern = pattern.to_ascii_lowercase();
        let value = value.to_ascii_lowercase();
        if pattern == "*" {
            return true;
        }
        let Some((prefix, suffix)) = pattern.split_once('*') else {
            return pattern == value;
        };
        value.starts_with(prefix) && value.ends_with(suffix)
    }

    fn expand_known_path(value: &str) -> PathBuf {
        let mut expanded = String::new();
        let mut rest = value;

        while let Some(start) = rest.find('%') {
            expanded.push_str(&rest[..start]);
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('%') else {
                expanded.push('%');
                expanded.push_str(after_start);
                return PathBuf::from(expanded);
            };
            let name = &after_start[..end];
            if let Some(replacement) = env::var_os(name) {
                expanded.push_str(&replacement.to_string_lossy());
            } else {
                expanded.push('%');
                expanded.push_str(name);
                expanded.push('%');
            }
            rest = &after_start[end + 1..];
        }

        expanded.push_str(rest);
        PathBuf::from(expanded)
    }

    fn rule_applies_to_current_os(os: Option<TargetOs>) -> bool {
        os.is_none_or(|os| Some(os) == TargetOs::current())
    }

    #[derive(Clone, Debug, Default)]
    struct Table {
        values: BTreeMap<String, Value>,
    }

    impl Table {
        fn string(&self, key: &str) -> Option<String> {
            match self.values.get(key) {
                Some(Value::String(value)) => Some(value.clone()),
                _ => None,
            }
        }

        fn required_string(&self, key: &str) -> Result<String, String> {
            self.string(key)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| format!("{key} is required"))
        }

        fn value_or_path(&self) -> Option<String> {
            self.string("value").or_else(|| self.string("path"))
        }

        fn string_array(&self, key: &str) -> Option<Vec<String>> {
            match self.values.get(key) {
                Some(Value::Array(values)) => Some(values.clone()),
                _ => None,
            }
        }

        fn bool(&self, key: &str) -> Option<bool> {
            match self.values.get(key) {
                Some(Value::Bool(value)) => Some(*value),
                _ => None,
            }
        }

        fn os(&self) -> Option<TargetOs> {
            self.string("os").and_then(|value| parse_os(&value))
        }

        fn required_os(&self) -> Result<TargetOs, String> {
            self.os()
                .ok_or_else(|| "os must be windows, macos, or linux".to_string())
        }
    }

    #[derive(Clone, Debug)]
    enum Value {
        String(String),
        Array(Vec<String>),
        Bool(bool),
    }

    fn parse_tables(content: &str) -> Result<BTreeMap<String, Vec<Table>>, String> {
        let mut tables = BTreeMap::<String, Vec<Table>>::new();
        tables.insert("root".to_string(), vec![Table::default()]);
        let mut current = "root".to_string();

        for raw_line in content.lines() {
            let stripped_line = strip_comment(raw_line);
            let line = stripped_line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("[[") && line.ends_with("]]") {
                current = line
                    .trim_start_matches("[[")
                    .trim_end_matches("]]")
                    .trim()
                    .to_string();
                tables
                    .entry(current.clone())
                    .or_default()
                    .push(Table::default());
                continue;
            }

            let Some((key, raw_value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().to_string();
            let value = parse_value(raw_value.trim())?;
            let table = tables
                .entry(current.clone())
                .or_default()
                .last_mut()
                .expect("current table should exist");
            table.values.insert(key, value);
        }

        Ok(tables)
    }

    fn parse_value(value: &str) -> Result<Value, String> {
        if value.eq_ignore_ascii_case("true") {
            return Ok(Value::Bool(true));
        }
        if value.eq_ignore_ascii_case("false") {
            return Ok(Value::Bool(false));
        }
        if value.starts_with('[') && value.ends_with(']') {
            let inner = value.trim_start_matches('[').trim_end_matches(']');
            let items = inner
                .split(',')
                .filter_map(|item| parse_string(item.trim()))
                .collect::<Vec<_>>();
            return Ok(Value::Array(items));
        }
        if value.chars().all(|item| item.is_ascii_digit()) {
            return Ok(Value::String(value.to_string()));
        }
        Ok(Value::String(parse_relaxed_string(value)))
    }

    fn parse_string(value: &str) -> Option<String> {
        let trimmed = value.trim();
        trimmed
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| value.replace("\\\\", "\\").replace("\\\"", "\""))
    }

    fn parse_relaxed_string(value: &str) -> String {
        value
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace("\\\\", "\\")
            .replace("\\\"", "\"")
    }

    fn strip_comment(line: &str) -> String {
        let mut in_string = false;
        let mut escaped = false;
        for (index, ch) in line.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '#' if !in_string => return line[..index].to_string(),
                _ => {}
            }
        }
        line.to_string()
    }

    fn parse_os(value: &str) -> Option<TargetOs> {
        match value.trim().to_ascii_lowercase().as_str() {
            "windows" => Some(TargetOs::Windows),
            "macos" => Some(TargetOs::Macos),
            "linux" => Some(TargetOs::Linux),
            _ => None,
        }
    }

    fn built_in_icon_for_key(icon_key: &str) -> &'static [u8] {
        match icon_key {
            "chrome" => icons::CHROME,
            "discord" => icons::DISCORD,
            "firefox" => icons::FIREFOX,
            "telegram" => icons::TELEGRAM,
            "whatsapp" => icons::WHATSAPP,
            "yandex_browser" => icons::YANDEX_BROWSER,
            _ => icons::MANUAL,
        }
    }

    fn built_in_icon_svg_for_key(icon_key: &str) -> &'static [u8] {
        match icon_key {
            "chrome" => icons::CHROME_SVG,
            "discord" => icons::DISCORD_SVG,
            "firefox" => icons::FIREFOX_SVG,
            "telegram" => icons::TELEGRAM_SVG,
            "whatsapp" => icons::WHATSAPP_SVG,
            "yandex_browser" => icons::YANDEX_BROWSER_SVG,
            _ => icons::DEFAULT_SVG,
        }
    }

    fn leak_string(value: String) -> &'static str {
        Box::leak(value.into_boxed_str())
    }

    fn leak_slice<T>(items: Vec<T>) -> &'static [T] {
        Box::leak(items.into_boxed_slice())
    }

    #[cfg(test)]
    mod tests {
        use super::{discover_known_path, load_connectors, parse_module, wildcard_match};
        use crate::AppConnector;
        use std::fs;

        #[test]
        fn parse_module_manifest_keeps_user_connector_contract() {
            let module = parse_module(
                r#"
                version = 1
                enabled = true
                id = "demo"
                display_name = "Demo App"
                icon_key = "demo"
                process_names = ["demo.exe", "demo"]

                [[process_aliases]]
                os = "windows"
                name = "demo.exe"

                [[discovery_sources]]
                os = "windows"
                kind = "known_path"
                detail = "%LOCALAPPDATA%\\Demo\\demo.exe"

                [[discovery]]
                os = "windows"
                kind = "known_path"
                root_env = "LOCALAPPDATA"
                relative_path = "Demo\\demo.exe"
                "#,
            )
            .expect("module should parse");

            assert_eq!(module.id, "demo");
            assert_eq!(module.display_name, "Demo App");
            assert_eq!(module.icon_key, "demo");
            assert_eq!(module.process_names, vec!["demo.exe", "demo"]);
            assert_eq!(module.aliases.len(), 1);
            assert_eq!(module.discovery_rules.len(), 1);
        }

        #[test]
        fn parse_module_accepts_human_friendly_path_aliases() {
            let module = parse_module(
                r#"
                version = 1
                enabled = true
                id = "demo"
                display_name = "Demo App"
                process_names = ["Demo.exe"]

                [[discovery]]
                os = "windows"
                kind = "known_path"
                path = "D:\\Games\\Demo App\\Demo.exe" # inline comments are allowed
                "#,
            )
            .expect("module should parse");

            assert_eq!(module.discovery_rules.len(), 1);
            assert_eq!(module.discovery_rules[0].kind, "fixed_path");
            assert_eq!(
                module.discovery_rules[0].value.as_deref(),
                Some(r"D:\Games\Demo App\Demo.exe")
            );
        }

        #[test]
        fn parse_module_accepts_single_backslash_windows_paths() {
            let module = parse_module(
                r#"

                # User-authored app manifests should be readable like Explorer paths.
                version = 1
                enabled = true
                id = "demo"
                display_name = "Demo App"
                process_names = ["Demo.exe"]

                [[discovery]]
                kind = "fixed_path"
                path = "D:\Games\Demo App\Demo.exe"
                "#,
            )
            .expect("module should parse");

            assert_eq!(module.discovery_rules.len(), 1);
            assert_eq!(
                module.discovery_rules[0].value.as_deref(),
                Some(r"D:\Games\Demo App\Demo.exe")
            );
        }

        #[test]
        fn parse_module_derives_discovery_from_sources_when_rules_are_absent() {
            let module = parse_module(
                r#"
                version = 1
                enabled = true
                id = "demo"
                display_name = "Demo App"
                process_names = ["Demo.exe"]

                [[discovery_sources]]
                os = "windows"
                kind = "known_path"
                detail = D:\\Games\\Demo App\\Demo.exe"
                "#,
            )
            .expect("module should parse");

            assert_eq!(module.discovery_sources.len(), 1);
            assert_eq!(module.discovery_rules.len(), 1);
            assert_eq!(module.discovery_rules[0].kind, "fixed_path");
            assert_eq!(
                module.discovery_rules[0].value.as_deref(),
                Some(r"D:\Games\Demo App\Demo.exe")
            );
        }

        #[test]
        fn runtime_module_discovers_fixed_path_from_path_alias_without_platform_rules() {
            let root = std::env::temp_dir().join(format!(
                "netstitch-runtime-module-path-alias-{}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("runtime module dir");
            let app = root.join("Demo.exe");
            fs::write(&app, b"").expect("demo exe");
            let app_text = app.to_string_lossy();

            let module = parse_module(&format!(
                r#"
                version = 1
                enabled = true
                id = "demo"
                display_name = "Demo App"
                process_names = ["Demo.exe"]

                [[discovery]]
                kind = "fixed_path"
                path = "{app_text}"
                "#
            ))
            .expect("module should parse");
            let connector = super::RuntimeConnector::new(module);
            let detected = connector.discover();
            assert_eq!(detected.len(), 1);
            assert_eq!(detected[0].exe_path, app);

            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn wildcard_segments_match_versioned_app_folders() {
            assert!(wildcard_match("app-*", "app-1.2.3"));
            assert!(wildcard_match("*Browser", "YandexBrowser"));
            assert!(!wildcard_match("app-*", "bin-1.2.3"));
        }

        #[test]
        fn known_path_wildcard_picks_latest_matching_folder() {
            let root = std::env::temp_dir()
                .join(format!("netstitch-runtime-modules-{}", std::process::id()));
            let old = root.join("Demo").join("app-1.0.0");
            let new = root.join("Demo").join("app-2.0.0");
            fs::create_dir_all(&old).expect("old dir");
            fs::create_dir_all(&new).expect("new dir");
            fs::write(old.join("Demo.exe"), b"").expect("old exe");
            fs::write(new.join("Demo.exe"), b"").expect("new exe");

            let found = discover_known_path(&root, "Demo\\app-*\\Demo.exe");
            assert_eq!(found, vec![new.join("Demo.exe")]);

            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn runtime_module_icon_key_uses_embedded_known_icon() {
            let root = std::env::temp_dir().join(format!(
                "netstitch-runtime-module-icons-{}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("runtime module dir");
            fs::write(
                root.join("chrome.app"),
                r#"
                version = 1
                enabled = true
                id = "chrome-runtime"
                display_name = "Chrome Runtime"
                icon_key = "chrome"
                process_names = ["chrome.exe"]
                "#,
            )
            .expect("runtime module fixture");

            let connectors = load_connectors(&root);
            let manifest = connectors.first().expect("runtime connector").manifest();
            assert_eq!(manifest.icon_key, "chrome");
            assert!(!manifest.icon_ico.is_empty());
            assert!(!manifest.icon_svg.is_empty());

            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn runtime_loader_accepts_only_app_extension() {
            let root = std::env::temp_dir().join(format!(
                "netstitch-runtime-module-extensions-{}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("runtime module dir");
            for (file_name, id) in [("alpha.app", "alpha"), ("beta.toml", "beta")] {
                fs::write(
                    root.join(file_name),
                    format!(
                        r#"
                        version = 1
                        enabled = true
                        id = "{id}"
                        display_name = "{id}"
                        process_names = ["{id}.exe"]
                        "#
                    ),
                )
                .expect("runtime module fixture");
            }

            let connectors = load_connectors(&root);
            let ids = connectors
                .iter()
                .map(|connector| connector.manifest().id)
                .collect::<Vec<_>>();
            assert_eq!(ids, vec!["alpha"]);

            fs::remove_dir_all(root).ok();
        }

        #[test]
        fn shipped_app_manifests_load_as_runtime_connectors() {
            let apps_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(std::path::Path::parent)
                .expect("connector crate lives under src/netstitch-connectors")
                .join("resources")
                .join("connectors")
                .join("apps");
            let app_files = fs::read_dir(&apps_dir)
                .unwrap_or_else(|error| {
                    panic!("{} should be readable: {error}", apps_dir.display())
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|error| {
                    panic!("{} entries should be readable: {error}", apps_dir.display())
                })
                .into_iter()
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "app")
                })
                .collect::<Vec<_>>();
            assert!(
                !app_files.is_empty(),
                "{} must contain shipped .app connector manifests",
                apps_dir.display()
            );

            let connectors = load_connectors(&apps_dir);
            let ids = connectors
                .iter()
                .map(|connector| connector.manifest().id)
                .collect::<Vec<_>>();
            assert_eq!(
                ids.len(),
                app_files.len(),
                "all shipped .app manifests must parse and load, got ids: {ids:?}"
            );
        }
    }
}

#[cfg(target_os = "windows")]
mod registry {
    use crate::{app_path_candidates_from_values, dedup_paths};
    use std::ffi::c_void;
    use std::path::PathBuf;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, REG_ROUTINE_FLAGS, RRF_RT_REG_EXPAND_SZ,
        RRF_RT_REG_SZ, RRF_SUBKEY_WOW6432KEY, RRF_SUBKEY_WOW6464KEY, RegGetValueW,
    };
    use windows::core::PCWSTR;

    const APP_PATHS_ROOT: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";

    pub(crate) fn app_path_candidates(exe_name: &str) -> Vec<PathBuf> {
        let subkey = format!(r"{APP_PATHS_ROOT}\{exe_name}");
        let mut candidates = Vec::new();

        for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            for view_flag in [
                REG_ROUTINE_FLAGS(0),
                RRF_SUBKEY_WOW6464KEY,
                RRF_SUBKEY_WOW6432KEY,
            ] {
                let default_value = read_registry_string(root, &subkey, None, view_flag);
                let path_value = read_registry_string(root, &subkey, Some("Path"), view_flag);
                candidates.extend(app_path_candidates_from_values(
                    exe_name,
                    default_value.as_deref(),
                    path_value.as_deref(),
                ));
            }
        }

        dedup_paths(candidates)
    }

    fn read_registry_string(
        root: HKEY,
        subkey: &str,
        value_name: Option<&str>,
        view_flag: REG_ROUTINE_FLAGS,
    ) -> Option<String> {
        let subkey = wide_null(subkey);
        let value_name = value_name.map(wide_null);
        let value_name = value_name
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or_else(PCWSTR::null);
        let flags = RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | view_flag;
        let mut byte_len = 0u32;

        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                value_name,
                flags,
                None,
                None,
                Some(&mut byte_len),
            )
        };
        if status.0 != 0 || byte_len < 2 {
            return None;
        }

        let mut buffer = vec![0u16; (byte_len as usize).div_ceil(2)];
        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                value_name,
                flags,
                None,
                Some(buffer.as_mut_ptr().cast::<c_void>()),
                Some(&mut byte_len),
            )
        };
        if status.0 != 0 {
            return None;
        }

        if let Some(nul_index) = buffer.iter().position(|value| *value == 0) {
            buffer.truncate(nul_index);
        }
        String::from_utf16(&buffer)
            .ok()
            .map(expand_percent_env_vars)
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn expand_percent_env_vars(value: String) -> String {
        let mut expanded = String::new();
        let mut rest = value.as_str();

        while let Some(start) = rest.find('%') {
            expanded.push_str(&rest[..start]);
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('%') else {
                expanded.push('%');
                expanded.push_str(after_start);
                return expanded;
            };

            let name = &after_start[..end];
            if let Some(replacement) = std::env::var_os(name) {
                expanded.push_str(&replacement.to_string_lossy());
            } else {
                expanded.push('%');
                expanded.push_str(name);
                expanded.push('%');
            }
            rest = &after_start[end + 1..];
        }

        expanded.push_str(rest);
        expanded
    }
}

#[cfg(not(target_os = "windows"))]
mod registry {
    use std::path::PathBuf;

    pub(crate) fn app_path_candidates(_exe_name: &str) -> Vec<PathBuf> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        app_path_candidates_from_values, desktop_entry_exec_candidates,
        desktop_entry_exec_candidates_with_flatpak_roots, discover_first_existing,
        discover_macos_bundles, discover_package_exes, discover_versioned_child, icons, modules,
    };
    use std::fs;

    #[test]
    fn discover_first_existing_keeps_first_real_file() {
        let root = unique_temp_dir("first");
        fs::create_dir_all(&root).expect("temp root");
        let missing = root.join("missing.exe");
        let real = root.join("real.exe");
        fs::write(&real, b"").expect("real exe fixture");

        let found = discover_first_existing([missing, real.clone()]);
        assert_eq!(found, vec![real]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn discover_versioned_child_picks_latest_matching_folder() {
        let root = unique_temp_dir("versioned");
        let old = root.join("app-1.0.0");
        let new = root.join("app-2.0.0");
        fs::create_dir_all(&old).expect("old dir");
        fs::create_dir_all(&new).expect("new dir");
        fs::write(old.join("Demo.exe"), b"").expect("old exe");
        fs::write(new.join("Demo.exe"), b"").expect("new exe");

        let found = discover_versioned_child(Some(root.clone()), "app-", "Demo.exe");
        assert_eq!(found, vec![new.join("Demo.exe")]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn app_paths_values_keep_default_exe_and_path_fallbacks() {
        let found = app_path_candidates_from_values(
            "Demo.exe",
            Some(r#""C:\Tools\Demo\Demo.exe""#),
            Some(r"C:\Tools\Demo;D:\Portable\Demo;"),
        );

        assert_eq!(
            found,
            vec![
                std::path::PathBuf::from(r"C:\Tools\Demo\Demo.exe"),
                std::path::PathBuf::from(r"D:\Portable\Demo").join("Demo.exe"),
            ]
        );
    }

    #[test]
    fn app_paths_values_deduplicate_case_insensitively() {
        let found = app_path_candidates_from_values(
            "Demo.exe",
            Some(r"C:\Tools\Demo\Demo.exe"),
            Some(r"c:\tools\demo"),
        );

        assert_eq!(
            found,
            vec![std::path::PathBuf::from(r"C:\Tools\Demo\Demo.exe")]
        );
    }

    #[test]
    fn discover_package_exes_picks_latest_matching_store_package() {
        let root = unique_temp_dir("store");
        let old_package = root.join("Demo.App_1.0.0.0_x64__publisher");
        let new_package = root.join("Demo.App_2.0.0.0_x64__publisher");
        fs::create_dir_all(old_package.join("VFS").join("ProgramFiles").join("Demo"))
            .expect("old package dirs");
        fs::create_dir_all(&new_package).expect("new package dir");
        fs::write(
            old_package
                .join("VFS")
                .join("ProgramFiles")
                .join("Demo")
                .join("Demo.exe"),
            b"",
        )
        .expect("old exe");
        fs::write(new_package.join("Demo.Root.exe"), b"").expect("new exe");

        let found = discover_package_exes(
            Some(root.clone()),
            &["Demo.App_"],
            &["Demo.Root.exe", "Demo.exe"],
        );
        assert_eq!(
            found,
            vec![
                new_package.join("Demo.Root.exe"),
                old_package
                    .join("VFS")
                    .join("ProgramFiles")
                    .join("Demo")
                    .join("Demo.exe")
            ]
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn discover_macos_bundles_resolves_app_bundle_executable() {
        let root = unique_temp_dir("macos-bundle");
        let executable = root
            .join("Demo.app")
            .join("Contents")
            .join("MacOS")
            .join("Demo");
        fs::create_dir_all(executable.parent().expect("bundle parent")).expect("bundle dirs");
        fs::write(&executable, b"").expect("bundle executable");

        let found = discover_macos_bundles(Some(root.clone()), &["Demo"], &["Demo"]);
        assert_eq!(found, vec![executable]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn desktop_entry_exec_candidates_resolve_tryexec_before_exec() {
        let root = unique_temp_dir("linux-desktop");
        let bin = root.join("bin");
        let try_exec = bin.join("demo-stable");
        let exec = bin.join("demo");
        fs::create_dir_all(&bin).expect("bin dir");
        fs::write(&try_exec, b"").expect("tryexec file");
        fs::write(&exec, b"").expect("exec file");
        let desktop_entry = "TryExec=demo-stable\nExec=demo --profile %u\n";

        let found = desktop_entry_exec_candidates(desktop_entry, &["demo"], &[bin]);
        assert_eq!(found, vec![try_exec]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn desktop_entry_exec_candidates_resolve_flatpak_command_binary() {
        let root = unique_temp_dir("linux-flatpak");
        let bin = root.join("bin");
        let flatpak_runner = bin.join("flatpak");
        let app_root = root
            .join("flatpak")
            .join("com.github.demo.App")
            .join("x86_64")
            .join("stable")
            .join("abcdef")
            .join("files")
            .join("bin");
        let app_binary = app_root.join("demo-flatpak");
        fs::create_dir_all(&bin).expect("bin dir");
        fs::create_dir_all(&app_root).expect("flatpak app bin");
        fs::write(&flatpak_runner, b"").expect("flatpak runner");
        fs::write(&app_binary, b"").expect("flatpak app binary");
        let desktop_entry = "Exec=/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=demo-flatpak --file-forwarding com.github.demo.App @@u %u @@\n";

        let found = desktop_entry_exec_candidates_with_flatpak_roots(
            desktop_entry,
            &["demo-flatpak"],
            &[bin],
            &[root.join("flatpak")],
        );
        assert_eq!(found, vec![app_binary]);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn known_app_connectors_embed_icon_bytes() {
        for connector in modules::all_connectors() {
            let manifest = connector.manifest();
            if manifest.manual_only {
                assert!(
                    manifest.icon_ico.is_empty(),
                    "manual connector should use letter fallback"
                );
                assert!(
                    !manifest.icon_svg.is_empty(),
                    "manual connector should embed the default icon"
                );
            } else {
                assert!(
                    !manifest.icon_ico.is_empty(),
                    "{} should embed an application icon",
                    manifest.id
                );
                assert!(
                    !manifest.icon_svg.is_empty(),
                    "{} should embed an application svg icon",
                    manifest.id
                );
            }
        }
    }

    #[test]
    fn small_slot_connector_icons_use_normalized_svg_viewports() {
        let yandex_svg = std::str::from_utf8(icons::YANDEX_BROWSER_SVG).expect("yandex svg utf8");
        assert!(yandex_svg.contains(r#"width="800" height="800""#));
        assert!(yandex_svg.contains(r#"viewBox="0 0 800 800""#));
        assert!(!yandex_svg.contains(r#"viewBox="-120 -200 1040 1200""#));

        let telegram_svg = std::str::from_utf8(icons::TELEGRAM_SVG).expect("telegram svg utf8");
        assert!(telegram_svg.contains(r#"viewBox="0 -4 100 100""#));
        assert!(telegram_svg.contains(r#"transform="translate(-10 0)""#));
        assert!(!telegram_svg.contains("<style"));

        let firefox_svg = std::str::from_utf8(icons::FIREFOX_SVG).expect("firefox svg utf8");
        assert!(firefox_svg.contains(r#"viewBox="0 0 32 32""#));
        assert!(firefox_svg.contains(r#"scale(1.08)"#));
    }

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "netstitch-connectors-{prefix}-{}",
            std::process::id()
        ))
    }
}
