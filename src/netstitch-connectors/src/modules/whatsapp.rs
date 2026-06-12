use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, env_path, icons, linux_application_candidates,
    macos_application_bundle_candidates, registry_app_path_candidates,
    windows_store_package_candidates,
};

pub struct WhatsAppConnector;

const PROCESS_NAMES: &[&str] = &[
    "WhatsApp.exe",
    "WhatsApp.Root.exe",
    "WhatsApp",
    "whatsapp",
    "whatsapp-for-linux",
];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "WhatsApp.exe",
    },
    ProcessAlias {
        os: TargetOs::Windows,
        name: "WhatsApp.Root.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "WhatsApp",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "whatsapp",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "whatsapp-for-linux",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "WhatsApp.exe/WhatsApp.Root.exe",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "5319275A.WhatsAppDesktop_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "LOCALAPPDATA WhatsApp",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/WhatsApp.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "whatsapp*.desktop/com.github.eneshecan.WhatsAppForLinux.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "whatsapp/whatsapp-for-linux",
    },
];

impl AppConnector for WhatsAppConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "whatsapp",
            display_name: "WhatsApp",
            icon_key: "whatsapp",
            icon_ico: icons::WHATSAPP,
            icon_svg: icons::WHATSAPP_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("WhatsApp.exe");
        candidates.extend(registry_app_path_candidates("WhatsApp.Root.exe"));
        candidates.extend(windows_store_package_candidates(
            &["5319275A.WhatsAppDesktop_"],
            &["WhatsApp.Root.exe", "WhatsApp.exe"],
        ));
        if let Some(local) = env_path("LOCALAPPDATA") {
            candidates.push(local.join("WhatsApp").join("WhatsApp.exe"));
            candidates.push(local.join("Programs").join("WhatsApp").join("WhatsApp.exe"));
        }
        candidates.extend(macos_application_bundle_candidates(
            &["WhatsApp"],
            &["WhatsApp"],
        ));
        candidates.extend(linux_application_candidates(
            &[
                "whatsapp.desktop",
                "whatsapp-for-linux.desktop",
                "com.whatsapp.WhatsApp.desktop",
                "com.github.eneshecan.WhatsAppForLinux.desktop",
            ],
            &["whatsapp", "whatsapp-for-linux"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux") {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some("whatsapp") => "whatsapp",
                        _ => "whatsapp-for-linux",
                    }
                } else if cfg!(target_os = "macos") {
                    "WhatsApp"
                } else if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case("WhatsApp.Root.exe"))
                {
                    "WhatsApp.Root.exe"
                } else {
                    "WhatsApp.exe"
                };
                detected(
                    "whatsapp",
                    "WhatsApp",
                    "whatsapp",
                    icons::WHATSAPP,
                    icons::WHATSAPP_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
