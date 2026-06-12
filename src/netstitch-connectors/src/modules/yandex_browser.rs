use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, env_path, icons, linux_application_candidates,
    macos_application_bundle_candidates, registry_app_path_candidates,
    windows_store_package_candidates,
};

pub struct YandexBrowserConnector;

const PROCESS_NAMES: &[&str] = &[
    "browser.exe",
    "Yandex",
    "yandex-browser",
    "yandex-browser-stable",
    "browser",
];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "browser.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "Yandex",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "yandex-browser",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "yandex-browser-stable",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "browser",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "browser.exe filtered to YandexBrowser",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "Yandex.YandexBrowser_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "Yandex/YandexBrowser/Application/browser.exe",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/Yandex.app or Yandex Browser.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "yandex-browser*.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "yandex-browser/yandex-browser-stable",
    },
];

impl AppConnector for YandexBrowserConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "yandex_browser",
            display_name: "Yandex Browser",
            icon_key: "yandex_browser",
            icon_ico: icons::YANDEX_BROWSER,
            icon_svg: icons::YANDEX_BROWSER_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("browser.exe")
            .into_iter()
            .filter(|path| {
                path.to_string_lossy()
                    .replace('/', "\\")
                    .to_ascii_lowercase()
                    .contains(r"\yandex\yandexbrowser\")
            })
            .collect::<Vec<_>>();
        candidates.extend(windows_store_package_candidates(
            &["Yandex.YandexBrowser_", "YANDEX.YANDEXBROWSER_"],
            &["browser.exe"],
        ));
        if let Some(local) = env_path("LOCALAPPDATA") {
            candidates.push(
                local
                    .join("Yandex")
                    .join("YandexBrowser")
                    .join("Application")
                    .join("browser.exe"),
            );
        }
        if let Some(program_files) = env_path("ProgramFiles") {
            candidates.push(
                program_files
                    .join("Yandex")
                    .join("YandexBrowser")
                    .join("Application")
                    .join("browser.exe"),
            );
        }
        candidates.extend(macos_application_bundle_candidates(
            &["Yandex", "Yandex Browser"],
            &["Yandex", "Yandex Browser"],
        ));
        candidates.extend(linux_application_candidates(
            &[
                "yandex-browser.desktop",
                "yandex-browser-stable.desktop",
                "ru.yandex.Browser.desktop",
                "ru.yandex.desktop.browser.desktop",
            ],
            &["yandex-browser", "yandex-browser-stable", "browser"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux") {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some("yandex-browser-stable") => "yandex-browser-stable",
                        Some("browser") => "browser",
                        _ => "yandex-browser",
                    }
                } else if cfg!(target_os = "macos") {
                    "Yandex"
                } else {
                    "browser.exe"
                };
                detected(
                    "yandex_browser",
                    "Yandex Browser",
                    "yandex_browser",
                    icons::YANDEX_BROWSER,
                    icons::YANDEX_BROWSER_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
