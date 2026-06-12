use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, env_path, icons, linux_application_candidates,
    macos_application_bundle_candidates, registry_app_path_candidates,
    windows_store_package_candidates,
};

const PROCESS_NAMES: &[&str] = &["Telegram.exe", "Telegram", "telegram-desktop"];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "Telegram.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "Telegram",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "Telegram",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "telegram-desktop",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "Telegram.exe",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "TelegramMessengerLLP.TelegramDesktop_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "APPDATA/Telegram Desktop",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/Telegram.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "org.telegram.desktop.desktop/telegramdesktop.desktop/telegram-desktop.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "telegram-desktop/Telegram",
    },
];

pub struct TelegramConnector;

impl AppConnector for TelegramConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "telegram",
            display_name: "Telegram",
            icon_key: "telegram",
            icon_ico: icons::TELEGRAM,
            icon_svg: icons::TELEGRAM_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("Telegram.exe");
        candidates.extend(windows_store_package_candidates(
            &["TelegramMessengerLLP.TelegramDesktop_"],
            &["Telegram.exe"],
        ));
        if let Some(appdata) = env_path("APPDATA") {
            candidates.push(appdata.join("Telegram Desktop").join("Telegram.exe"));
        }
        if let Some(local) = env_path("LOCALAPPDATA") {
            candidates.push(
                local
                    .join("Programs")
                    .join("Telegram Desktop")
                    .join("Telegram.exe"),
            );
        }
        candidates.extend(macos_application_bundle_candidates(
            &["Telegram"],
            &["Telegram"],
        ));
        candidates.extend(linux_application_candidates(
            &[
                "org.telegram.desktop.desktop",
                "telegramdesktop.desktop",
                "telegram-desktop.desktop",
            ],
            &["telegram-desktop", "Telegram"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux") {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some("Telegram") => "Telegram",
                        _ => "telegram-desktop",
                    }
                } else if cfg!(target_os = "macos") {
                    "Telegram"
                } else {
                    "Telegram.exe"
                };
                detected(
                    "telegram",
                    "Telegram",
                    "telegram",
                    icons::TELEGRAM,
                    icons::TELEGRAM_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
