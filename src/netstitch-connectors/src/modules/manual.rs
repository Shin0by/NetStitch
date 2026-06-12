use crate::{AppConnector, ConnectorManifest, DetectedApp, icons};

pub struct ManualPathConnector;

impl AppConnector for ManualPathConnector {
    fn manifest(&self) -> ConnectorManifest {
        ConnectorManifest {
            id: "manual_path",
            display_name: "Manual path",
            icon_key: "manual",
            icon_ico: icons::MANUAL,
            icon_svg: icons::DEFAULT_SVG,
            process_names: &[],
            process_aliases: &[],
            discovery_sources: &[],
            manual_only: true,
        }
    }

    fn discover(&self) -> Vec<DetectedApp> {
        Vec::new()
    }
}
