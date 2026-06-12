use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, env_path, icons, linux_application_candidates,
    macos_application_bundle_candidates, registry_app_path_candidates,
    windows_store_package_candidates,
};

const PROCESS_NAMES: &[&str] = &[
    "chrome.exe",
    "Google Chrome",
    "google-chrome",
    "google-chrome-stable",
    "chrome",
];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "chrome.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "Google Chrome",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "chrome",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "google-chrome",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "google-chrome-stable",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "chrome.exe",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "Google.Chrome_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "ProgramFiles/LOCALAPPDATA Google Chrome",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/Google Chrome.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "google-chrome*.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "google-chrome/google-chrome-stable/chrome",
    },
];

pub struct ChromeConnector;

impl AppConnector for ChromeConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "chrome",
            display_name: "Google Chrome",
            icon_key: "chrome",
            icon_ico: icons::CHROME,
            icon_svg: icons::CHROME_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("chrome.exe");
        candidates.extend(windows_store_package_candidates(
            &["Google.Chrome_", "GoogleLLC.Chrome_"],
            &["chrome.exe"],
        ));
        if let Some(program_files) = env_path("ProgramFiles") {
            candidates.push(
                program_files
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
        if let Some(program_files_x86) = env_path("ProgramFiles(x86)") {
            candidates.push(
                program_files_x86
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
        if let Some(local) = env_path("LOCALAPPDATA") {
            candidates.push(
                local
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe"),
            );
        }
        candidates.extend(macos_application_bundle_candidates(
            &["Google Chrome"],
            &["Google Chrome"],
        ));
        candidates.extend(linux_application_candidates(
            &[
                "google-chrome.desktop",
                "google-chrome-stable.desktop",
                "com.google.Chrome.desktop",
            ],
            &["google-chrome", "google-chrome-stable", "chrome"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux") {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some("google-chrome-stable") => "google-chrome-stable",
                        Some("chrome") => "chrome",
                        _ => "google-chrome",
                    }
                } else if cfg!(target_os = "macos") {
                    "Google Chrome"
                } else {
                    "chrome.exe"
                };
                detected(
                    "chrome",
                    "Google Chrome",
                    "chrome",
                    icons::CHROME,
                    icons::CHROME_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
