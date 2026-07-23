#[derive(Clone, Debug, PartialEq)]
pub struct TrackedApp {
    pub id: u64,
    pub connector_id: Option<String>,
    pub cloud_app_id: Option<String>,
    pub display_name: String,
    pub icon_key: String,
    pub exe_path: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Unknown,
    Attempting,
    Established,
    Closing,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationRow {
    pub id: u64,
    pub tracked_app_id: u64,
    pub process_name: String,
    pub remote_ip: String,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub first_seen: String,
    pub last_seen: String,
    pub hits: u32,
    pub connection_state: ConnectionState,
    pub failed_hits: u32,
    pub successful_hits: u32,
    pub is_confirmed: bool,
    pub is_exported: bool,
    pub enrichment: Option<crate::watcher_api::IpEnrichmentDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredAddress {
    pub id: u64,
    pub address_pattern: String,
    pub created_at: String,
    pub enrichment: Option<crate::watcher_api::IpEnrichmentDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationIntegration {
    pub repo_path: String,
    pub export_path: String,
    pub reference_data_path: String,
    pub profile_paths: Vec<String>,
    pub ready: bool,
    pub status_text: String,
    pub last_export_text: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub repository_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiState {
    pub monitoring: bool,
    pub status_text: String,
    pub error_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSettings {
    pub language_code: Option<String>,
    pub enable_all_overlay: bool,
    pub remember_window_placement: bool,
    pub hide_when_minimized: bool,
    pub module_order: Vec<String>,
    pub monitoring_hide_tags: bool,
    pub monitoring_hide_connection_count: bool,
    pub web_access_localhost: bool,
    pub domain_capture_enabled: bool,
    pub update_check_interval_minutes: u64,
    pub profile_export_ui_state: Option<netstitch_shared::models::ProfileExportUiStateDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObservationFilter {
    All,
    Unconfirmed,
    Confirmed,
    Success,
    Exported,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filters {
    pub app_search: String,
    pub search_text: String,
    pub domain_search: String,
    pub port_search: String,
    pub protocol: String,
    pub public_ip: bool,
    pub observation_filter: ObservationFilter,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceState {
    pub tracked_apps: Vec<TrackedApp>,
    pub observations: Vec<ObservationRow>,
    pub ignored_addresses: Vec<IgnoredAddress>,
    pub integration: IntegrationIntegration,
    pub ui: UiState,
    pub app_settings: AppSettings,
    pub filters: Filters,
    pub pending_exe_path: String,
    pub next_app_id: u64,
    pub next_observation_id: u64,
    pub next_ignored_address_id: u64,
}
