use crate::{
    AppConnector, ConnectorManifest, DetectedApp, DiscoverySource, ProcessAlias, TargetOs,
    detected, discover_first_existing, discover_versioned_child, env_path, icons,
    linux_application_candidates, macos_application_bundle_candidates,
    registry_app_path_candidates, windows_store_package_candidates,
};

const PROCESS_NAMES: &[&str] = &["Discord.exe", "Discord", "discord"];
const PROCESS_ALIASES: &[ProcessAlias] = &[
    ProcessAlias {
        os: TargetOs::Windows,
        name: "Discord.exe",
    },
    ProcessAlias {
        os: TargetOs::Macos,
        name: "Discord",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "Discord",
    },
    ProcessAlias {
        os: TargetOs::Linux,
        name: "discord",
    },
];
const DISCOVERY_SOURCES: &[DiscoverySource] = &[
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "app_paths",
        detail: "Discord.exe",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "store_msix",
        detail: "DiscordInc.Discord_*",
    },
    DiscoverySource {
        os: TargetOs::Windows,
        kind: "known_paths",
        detail: "LOCALAPPDATA/Discord/app-*",
    },
    DiscoverySource {
        os: TargetOs::Macos,
        kind: "app_bundle",
        detail: "/Applications/Discord.app",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "desktop_entry",
        detail: "discord.desktop/com.discordapp.Discord.desktop",
    },
    DiscoverySource {
        os: TargetOs::Linux,
        kind: "path",
        detail: "discord/Discord",
    },
];

pub struct DiscordConnector;

impl AppConnector for DiscordConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "discord",
            display_name: "Discord",
            icon_key: "discord",
            icon_ico: icons::DISCORD,
            icon_svg: icons::DISCORD_SVG,
            process_names: PROCESS_NAMES,
            process_aliases: PROCESS_ALIASES,
            discovery_sources: DISCOVERY_SOURCES,
            manual_only: false,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        let mut candidates = registry_app_path_candidates("Discord.exe");
        candidates.extend(windows_store_package_candidates(
            &["DiscordInc.Discord_", "Discord.Discord_"],
            &["Discord.exe"],
        ));
        candidates.extend(discover_versioned_child(
            env_path("LOCALAPPDATA").map(|path| path.join("Discord")),
            "app-",
            "Discord.exe",
        ));
        candidates.extend(macos_application_bundle_candidates(
            &["Discord"],
            &["Discord"],
        ));
        candidates.extend(linux_application_candidates(
            &["discord.desktop", "com.discordapp.Discord.desktop"],
            &["discord", "Discord"],
        ));

        discover_first_existing(candidates)
            .into_iter()
            .map(|path| {
                let process_name = if cfg!(target_os = "linux") {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some("Discord") => "Discord",
                        _ => "discord",
                    }
                } else if cfg!(target_os = "macos") {
                    "Discord"
                } else {
                    "Discord.exe"
                };
                detected(
                    "discord",
                    "Discord",
                    "discord",
                    icons::DISCORD,
                    icons::DISCORD_SVG,
                    path,
                    process_name,
                )
            })
            .collect()
    }
}
