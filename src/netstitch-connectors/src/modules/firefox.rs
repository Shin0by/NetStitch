use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, env_path, icons, linux_application_candidates,
    macos_application_bundle_candidates, registry_app_path_candidates,
    windows_store_package_candidates,
};

const PROCESS_NAMES: &[&str] = &["firefox.exe", "firefox", "firefox-esr"];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "firefox.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "firefox",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "firefox",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "firefox-esr",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "firefox.exe",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "Mozilla.Firefox_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "ProgramFiles Mozilla Firefox",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/Firefox.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "firefox.desktop/firefox-esr.desktop/org.mozilla.firefox.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "firefox/firefox-esr",
    },
];

pub struct FirefoxConnector;

impl AppConnector for FirefoxConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "firefox",
            display_name: "Mozilla Firefox",
            icon_key: "firefox",
            icon_ico: icons::FIREFOX,
            icon_svg: icons::FIREFOX_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("firefox.exe");
        candidates.extend(windows_store_package_candidates(
            &["Mozilla.Firefox_"],
            &["firefox.exe"],
        ));
        if let Some(program_files) = env_path("ProgramFiles") {
            candidates.push(program_files.join("Mozilla Firefox").join("firefox.exe"));
        }
        if let Some(program_files_x86) = env_path("ProgramFiles(x86)") {
            candidates.push(
                program_files_x86
                    .join("Mozilla Firefox")
                    .join("firefox.exe"),
            );
        }
        candidates.extend(macos_application_bundle_candidates(
            &["Firefox"],
            &["firefox"],
        ));
        candidates.extend(linux_application_candidates(
            &[
                "firefox.desktop",
                "firefox-esr.desktop",
                "org.mozilla.firefox.desktop",
            ],
            &["firefox", "firefox-esr"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name == "firefox-esr")
                {
                    "firefox-esr"
                } else if cfg!(target_os = "linux") {
                    "firefox"
                } else {
                    "firefox.exe"
                };
                detected(
                    "firefox",
                    "Mozilla Firefox",
                    "firefox",
                    icons::FIREFOX,
                    icons::FIREFOX_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
