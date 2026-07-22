use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

pub type TimestampMillis = u64;
pub type RequestId = u64;
pub type TrackedAppId = u64;
pub type ObservedEndpointId = u64;
pub type ExportRunId = u64;
pub type IgnoredAddressId = u64;

pub const SETTING_UI_LANGUAGE: &str = "ui.language";
pub const SETTING_UI_ENABLE_ALL_OVERLAY: &str = "ui.enable_all_overlay";
pub const SETTING_UI_REMEMBER_WINDOW_PLACEMENT: &str = "ui.remember_window_placement";
pub const SETTING_UI_HIDE_WHEN_MINIMIZED: &str = "ui.hide_when_minimized";
pub const SETTING_UI_MODULE_ORDER: &str = "ui.modules.order";
pub const SETTING_UI_MONITORING_PUBLIC_IP: &str = "ui.monitoring.public_ip";
pub const SETTING_UI_WINDOW_X: &str = "ui.window_x";
pub const SETTING_UI_WINDOW_Y: &str = "ui.window_y";
pub const SETTING_UI_WINDOW_WIDTH: &str = "ui.window_width";
pub const SETTING_UI_WINDOW_HEIGHT: &str = "ui.window_height";
pub const SETTING_UI_WINDOW_HIDDEN: &str = "ui.window_hidden";
pub const SETTING_WEB_ACCESS_LOCALHOST: &str = "web.localhost_enabled";
pub const SETTING_DOMAIN_CAPTURE_ENABLED: &str = "domain_capture.enabled";
pub const SETTING_UPDATE_CHECK_INTERVAL_MINUTES: &str = "updates.check_interval_minutes";
pub const CLIENT_HEADER_NAME: &str = "x-netstitch-client";
pub const CLIENT_HEADER_DESKTOP_UI: &str = "desktop-ui";
pub const SETTING_IP_ENRICHMENT_ENABLED: &str = "ip_enrichment.enabled";
pub const SETTING_IGNORE_DEFAULTS_SEEDED: &str = "system.ignore_defaults_seeded";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Other,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
            Self::Other => "other",
        }
    }

    pub fn from_name(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "tcp" => Self::Tcp,
            "udp" => Self::Udp,
            _ => Self::Other,
        }
    }

    pub fn from_db_value(value: impl AsRef<str>) -> Self {
        Self::from_name(value.as_ref())
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_name(&value))
    }
}

pub type TransportProtocol = Protocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectionState {
    Unknown,
    Attempting,
    Established,
    Closing,
    Failed,
}

impl ConnectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Attempting => "attempting",
            Self::Established => "established",
            Self::Closing => "closing",
            Self::Failed => "failed",
        }
    }

    pub fn from_name(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "attempting" => Self::Attempting,
            "established" => Self::Established,
            "closing" => Self::Closing,
            "failed" => Self::Failed,
            _ => Self::Unknown,
        }
    }

    pub fn from_db_value(value: impl AsRef<str>) -> Self {
        Self::from_name(value.as_ref())
    }

    pub fn is_failure_like(self) -> bool {
        matches!(self, Self::Attempting | Self::Failed)
    }

    pub fn is_success_like(self) -> bool {
        matches!(self, Self::Established | Self::Closing)
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedApp {
    pub id: Option<TrackedAppId>,
    pub exe_path: PathBuf,
    #[serde(default)]
    pub connector_id: Option<String>,
    #[serde(default)]
    pub cloud_app_id: Option<String>,
    pub process_name: Option<String>,
    pub display_name: Option<String>,
    pub icon_key: Option<String>,
    pub icon_path: Option<PathBuf>,
    #[serde(default)]
    pub current_tag: Option<String>,
    pub enabled: bool,
    pub created_at_ms: TimestampMillis,
}

impl TrackedApp {
    pub fn new(exe_path: impl Into<PathBuf>, created_at_ms: TimestampMillis) -> Self {
        Self {
            id: None,
            exe_path: exe_path.into(),
            connector_id: None,
            cloud_app_id: None,
            process_name: None,
            display_name: None,
            icon_key: None,
            icon_path: None,
            current_tag: None,
            enabled: true,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedEndpoint {
    pub id: Option<ObservedEndpointId>,
    pub tracked_app_id: TrackedAppId,
    #[serde(default)]
    pub cloud_app_id: Option<String>,
    #[serde(default)]
    pub app_signature_key: Option<String>,
    #[serde(default)]
    pub app_signature_subject: Option<String>,
    #[serde(default)]
    pub app_signature_issuer: Option<String>,
    #[serde(default)]
    pub app_signature_source: Option<String>,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub first_seen_ms: TimestampMillis,
    pub last_seen_ms: TimestampMillis,
    pub hits: u64,
    pub connection_state: ConnectionState,
    pub failed_hits: u64,
    pub successful_hits: u64,
    pub is_confirmed: bool,
    pub is_exported: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub enrichment: Option<IpEnrichmentDto>,
}

impl ObservedEndpoint {
    pub fn new(
        tracked_app_id: TrackedAppId,
        remote_ip: IpAddr,
        remote_port: u16,
        protocol: Protocol,
        first_seen_ms: TimestampMillis,
    ) -> Self {
        Self {
            id: None,
            tracked_app_id,
            cloud_app_id: None,
            app_signature_key: None,
            app_signature_subject: None,
            app_signature_issuer: None,
            app_signature_source: None,
            process_id: None,
            process_name: None,
            remote_ip,
            remote_port,
            protocol,
            first_seen_ms,
            last_seen_ms: first_seen_ms,
            hits: 1,
            connection_state: ConnectionState::Unknown,
            failed_hits: 0,
            successful_hits: 0,
            is_confirmed: false,
            is_exported: false,
            tags: Vec::new(),
            enrichment: None,
        }
    }

    pub fn touch(&mut self, seen_at_ms: TimestampMillis) {
        self.last_seen_ms = seen_at_ms;
        self.hits = self.hits.saturating_add(1);
    }

    pub fn mark_confirmed(&mut self, confirmed: bool) {
        self.is_confirmed = confirmed;
    }

    pub fn mark_exported(&mut self, exported: bool) {
        self.is_exported = exported;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringCsvImportRowDto {
    pub application: String,
    #[serde(default)]
    pub app_connector_id: Option<String>,
    #[serde(default)]
    pub cloud_app_id: Option<String>,
    #[serde(default)]
    pub app_signature_key: Option<String>,
    #[serde(default)]
    pub app_signature_subject: Option<String>,
    #[serde(default)]
    pub app_signature_issuer: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub remote_ip: IpAddr,
    pub domain: Option<String>,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub connection_state: ConnectionState,
    pub first_seen_ms: TimestampMillis,
    pub last_seen_ms: TimestampMillis,
    pub hits: u64,
    pub failed_hits: u64,
    pub successful_hits: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitoringImportSourceDto {
    Csv,
    CloudDownload,
}

impl Default for MonitoringImportSourceDto {
    fn default() -> Self {
        Self::Csv
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MonitoringCsvImportRequestDto {
    #[serde(default)]
    pub import_source: MonitoringImportSourceDto,
    pub rows: Vec<MonitoringCsvImportRowDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringCsvImportResultDto {
    pub requested_count: usize,
    pub imported_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregatedIpDto {
    pub remote_ip: IpAddr,
    pub tracked_app_ids: Vec<TrackedAppId>,
    pub process_names: Vec<String>,
    pub process_ids: Vec<u32>,
    pub remote_ports: Vec<u16>,
    pub protocols: Vec<Protocol>,
    pub first_seen_ms: TimestampMillis,
    pub last_seen_ms: TimestampMillis,
    pub hits: u64,
    pub failed_hits: u64,
    pub successful_hits: u64,
    pub is_confirmed: bool,
    pub is_exported: bool,
}

impl AggregatedIpDto {
    pub fn new(remote_ip: IpAddr) -> Self {
        Self {
            remote_ip,
            tracked_app_ids: Vec::new(),
            process_names: Vec::new(),
            process_ids: Vec::new(),
            remote_ports: Vec::new(),
            protocols: Vec::new(),
            first_seen_ms: 0,
            last_seen_ms: 0,
            hits: 0,
            failed_hits: 0,
            successful_hits: 0,
            is_confirmed: false,
            is_exported: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportRun {
    pub id: Option<ExportRunId>,
    pub target_path: PathBuf,
    pub created_at_ms: TimestampMillis,
    pub exported_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoredAddressRule {
    pub id: Option<IgnoredAddressId>,
    pub address_pattern: String,
    pub created_at_ms: TimestampMillis,
    #[serde(default)]
    pub enrichment: Option<IpEnrichmentDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IpEnrichmentDto {
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_source: Option<String>,
    #[serde(default)]
    pub owner_name: Option<String>,
    #[serde(default)]
    pub owner_range: Option<String>,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub updated_at_ms: Option<TimestampMillis>,
}

impl IpEnrichmentDto {
    pub fn is_empty(&self) -> bool {
        self.domain_name.is_none()
            && self.domain_source.is_none()
            && self.owner_name.is_none()
            && self.owner_range.is_none()
            && self.registry.is_none()
            && self.country.is_none()
            && self.source.is_none()
    }
}

impl ExportRun {
    pub fn new(
        target_path: impl Into<PathBuf>,
        created_at_ms: TimestampMillis,
        exported_count: u64,
    ) -> Self {
        Self {
            id: None,
            target_path: target_path.into(),
            created_at_ms,
            exported_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserExportTarget {
    pub path: PathBuf,
}

impl UserExportTarget {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportResultDto {
    pub target_path: PathBuf,
    pub exported_ips: Vec<IpAddr>,
    pub exported_count: usize,
    pub skipped_unconfirmed: usize,
    pub skipped_duplicates: usize,
    #[serde(default)]
    pub changed_files: Vec<ExportFileChangeDto>,
    #[serde(default)]
    pub warnings: Vec<ExportPlanWarningDto>,
    pub created_at_ms: TimestampMillis,
}

impl ExportResultDto {
    pub fn new(
        target_path: impl Into<PathBuf>,
        exported_ips: Vec<IpAddr>,
        skipped_unconfirmed: usize,
        skipped_duplicates: usize,
        created_at_ms: TimestampMillis,
    ) -> Self {
        let exported_count = exported_ips.len();
        Self {
            target_path: target_path.into(),
            exported_ips,
            exported_count,
            skipped_unconfirmed,
            skipped_duplicates,
            changed_files: Vec::new(),
            warnings: Vec::new(),
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportModeDto {
    AttachNetstitchLists,
    PatchSelectedProfile,
    MergeIntoExistingLists,
}

impl Default for ExportModeDto {
    fn default() -> Self {
        Self::AttachNetstitchLists
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFileOperationDto {
    Create,
    Update,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportFileChangeDto {
    pub path: PathBuf,
    pub operation: ExportFileOperationDto,
    #[serde(default)]
    pub bytes_written: Option<usize>,
}

impl ExportFileChangeDto {
    pub fn new(
        path: impl Into<PathBuf>,
        operation: ExportFileOperationDto,
        bytes_written: Option<usize>,
    ) -> Self {
        Self {
            path: path.into(),
            operation,
            bytes_written,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPlanWarningDto {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub protocol: Option<Protocol>,
    #[serde(default)]
    pub port: Option<u16>,
}

impl ExportPlanWarningDto {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            protocol: None,
            port: None,
        }
    }

    pub fn for_endpoint(
        code: impl Into<String>,
        message: impl Into<String>,
        protocol: Protocol,
        port: u16,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            protocol: Some(protocol),
            port: Some(port),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfileRuleDto {
    pub protocol: Protocol,
    pub ports: Vec<u16>,
    #[serde(default)]
    pub ports_display: String,
    pub list_refs: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfileUncoveredTargetDto {
    pub protocol: Protocol,
    pub port: u16,
    pub ips: Vec<IpAddr>,
    pub ip_count: usize,
}

impl ExportProfileUncoveredTargetDto {
    pub fn new(protocol: Protocol, port: u16, ips: Vec<IpAddr>) -> Self {
        let ip_count = ips.len();
        Self {
            protocol,
            port,
            ips,
            ip_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExportProfileAdvancedSettingsDto {
    #[serde(default)]
    pub create_rules_for_uncovered: bool,
    #[serde(default)]
    pub add_ips_to_exclude: bool,
    #[serde(default)]
    pub use_whois_ranges_for_export: bool,
    #[serde(default)]
    pub add_detected_domains: bool,
    #[serde(default)]
    pub remove_detected_domains: bool,
    #[serde(default)]
    pub manual_domains: Vec<String>,
    #[serde(default)]
    pub template_rule_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfileAdvancedSettingsRequestDto {
    pub export: ExportProfileRequestDto,
    #[serde(default)]
    pub settings: ExportProfileAdvancedSettingsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfileRequestDto {
    pub provider_id: String,
    pub repo_root: PathBuf,
    #[serde(default)]
    pub selected_profile_path: Option<PathBuf>,
    #[serde(default)]
    pub generated_profile_name: Option<String>,
    #[serde(default)]
    pub mode: ExportModeDto,
    #[serde(default)]
    pub dangerous_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfilePlanDto {
    pub provider_id: String,
    pub repo_root: PathBuf,
    pub mode: ExportModeDto,
    pub selected_profile_path: Option<PathBuf>,
    pub generated_profile_path: Option<PathBuf>,
    pub exported_ips: Vec<IpAddr>,
    pub exported_count: usize,
    #[serde(default)]
    pub covered_ips: Vec<IpAddr>,
    #[serde(default)]
    pub covered_count: usize,
    #[serde(default)]
    pub covered_targets: Vec<String>,
    #[serde(default)]
    pub covered_target_count: usize,
    #[serde(default)]
    pub use_whois_ranges_for_export: bool,
    #[serde(default)]
    pub covered_range_count: usize,
    #[serde(default)]
    pub covered_range_address_count: String,
    #[serde(default)]
    pub uncovered_targets: Vec<ExportProfileUncoveredTargetDto>,
    #[serde(default)]
    pub uncovered_count: usize,
    #[serde(default)]
    pub analysis_performed: bool,
    #[serde(default)]
    pub existing_ips: Vec<IpAddr>,
    #[serde(default)]
    pub existing_count: usize,
    #[serde(default)]
    pub existing_targets: Vec<String>,
    #[serde(default)]
    pub existing_target_count: usize,
    #[serde(default)]
    pub existing_range_count: usize,
    #[serde(default)]
    pub existing_range_address_count: String,
    #[serde(default)]
    pub new_ips: Vec<IpAddr>,
    #[serde(default)]
    pub new_count: usize,
    #[serde(default)]
    pub new_targets: Vec<String>,
    #[serde(default)]
    pub new_target_count: usize,
    #[serde(default)]
    pub new_range_count: usize,
    #[serde(default)]
    pub new_range_address_count: String,
    #[serde(default)]
    pub advanced_domain_count: usize,
    #[serde(default)]
    pub advanced_existing_domain_count: usize,
    #[serde(default)]
    pub advanced_new_domain_count: usize,
    pub skipped_unconfirmed: usize,
    pub skipped_exported: usize,
    pub skipped_duplicates: usize,
    pub matched_rules: Vec<ExportProfileRuleDto>,
    pub warnings: Vec<ExportPlanWarningDto>,
    pub file_changes: Vec<ExportFileChangeDto>,
    #[serde(default)]
    pub advanced_file_changes: Vec<ExportFileChangeDto>,
    pub created_at_ms: TimestampMillis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileExportUiStateDto {
    pub repo_root_key: String,
    #[serde(default)]
    pub mode: ExportModeDto,
    #[serde(default)]
    pub attach_profile_path: String,
    #[serde(default)]
    pub patch_profile_path: String,
    #[serde(default)]
    pub merge_profile_path: String,
    #[serde(default)]
    pub generated_profile_name: String,
    #[serde(default)]
    pub dangerous_confirmed: bool,
    #[serde(default)]
    pub advanced_create_rules_for_uncovered: bool,
    #[serde(default)]
    pub advanced_add_ips_to_exclude: bool,
    #[serde(default)]
    pub advanced_use_whois_ranges_for_export: bool,
    #[serde(default)]
    pub advanced_add_detected_domains: bool,
    #[serde(default)]
    pub advanced_manual_domains: String,
    #[serde(default)]
    pub advanced_template_rule_source: String,
    #[serde(default)]
    pub advanced_ready_for_export: bool,
    #[serde(default)]
    pub advanced_domain_cleanup_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    PermissionDenied,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub monitoring_supported: bool,
    pub export_supported: bool,
    pub external_integrations_supported: bool,
    pub requires_admin: bool,
    pub ipc_supported: bool,
}

impl CapabilityReport {
    pub fn baseline() -> Self {
        Self {
            monitoring_supported: true,
            export_supported: true,
            external_integrations_supported: true,
            requires_admin: true,
            ipc_supported: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleDto {
    pub id: String,
    pub display_name: String,
    pub tooltip: String,
    pub icon_label: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub display_name_key: Option<String>,
    #[serde(default)]
    pub tooltip_key: Option<String>,
    #[serde(default)]
    pub provider_title: Option<String>,
    #[serde(default)]
    pub provider_title_key: Option<String>,
    #[serde(default)]
    pub export_title: Option<String>,
    #[serde(default)]
    pub export_title_key: Option<String>,
    #[serde(default)]
    pub button_color: Option<String>,
    #[serde(default)]
    pub icon_svg: Option<String>,
    #[serde(default)]
    pub icon_path: Option<PathBuf>,
    #[serde(default)]
    pub icon_data_uri: Option<String>,
    #[serde(default)]
    pub header_actions: Vec<IntegrationModuleActionDto>,
    #[serde(default)]
    pub ui_schema: Vec<IntegrationUiEntityDto>,
    #[serde(default)]
    pub background_active: bool,
    #[serde(default)]
    pub background_subscriptions: Vec<String>,
    #[serde(default)]
    pub background_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleActionDto {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_key: Option<String>,
    #[serde(default)]
    pub tooltip: Option<String>,
    #[serde(default)]
    pub tooltip_key: Option<String>,
    #[serde(default)]
    pub icon_svg: Option<String>,
    #[serde(default)]
    pub icon_path: Option<PathBuf>,
    #[serde(default)]
    pub icon_data_uri: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub pulse: bool,
    #[serde(default)]
    pub pulse_when_background_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiEntityDto {
    pub id: String,
    pub entity_type: String,
    #[serde(default)]
    pub page: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_key: Option<String>,
    #[serde(default)]
    pub tooltip: Option<String>,
    #[serde(default)]
    pub tooltip_key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub value_key: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub placeholder_key: Option<String>,
    #[serde(default)]
    pub options: Vec<IntegrationUiOptionDto>,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub clear_button: bool,
    #[serde(default)]
    pub commit_on_enter: bool,
    #[serde(default)]
    pub compact: bool,
    #[serde(default)]
    pub hide_label: bool,
    #[serde(default)]
    pub hide_host_back_button: bool,
    #[serde(default)]
    pub progress_stages: Vec<IntegrationUiProgressStageDto>,
    #[serde(default)]
    pub scroll: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub height: Option<String>,
    #[serde(default)]
    pub min_width: Option<String>,
    #[serde(default)]
    pub min_height: Option<String>,
    #[serde(default)]
    pub max_width: Option<String>,
    #[serde(default)]
    pub max_height: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub justify: Option<String>,
    #[serde(default)]
    pub button_layout: Option<String>,
    #[serde(default)]
    pub margin: Option<String>,
    #[serde(default)]
    pub padding: Option<String>,
    #[serde(default)]
    pub columns: Option<String>,
    #[serde(default)]
    pub table_columns: Vec<IntegrationUiTableColumnDto>,
    #[serde(default)]
    pub rows: Option<String>,
    #[serde(default)]
    pub gap: Option<String>,
    #[serde(default)]
    pub grid_column: Option<String>,
    #[serde(default)]
    pub grid_row: Option<String>,
    #[serde(default = "default_ui_opacity")]
    pub opacity: String,
    #[serde(default)]
    pub children: Vec<IntegrationUiEntityDto>,
    #[serde(default)]
    pub actions: Vec<IntegrationModuleActionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiProgressStageDto {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub percent: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub name_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiTableColumnDto {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub text_field: bool,
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub min_width: Option<String>,
    #[serde(default)]
    pub max_width: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiOptionDto {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub label_key: Option<String>,
}

fn default_ui_opacity() -> String {
    "100%".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleTableContextDto {
    pub id: String,
    pub total_rows: usize,
    pub displayed_rows: usize,
    pub selected_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleHostContextDto {
    pub app_version: String,
    pub language_code: Option<String>,
    pub monitoring_active: bool,
    #[serde(default)]
    pub module_background_active: bool,
    pub filters: UiFiltersDto,
    #[serde(default)]
    pub tables: Vec<IntegrationModuleTableContextDto>,
    #[serde(default)]
    pub selected_monitoring_row_ids: Vec<ObservedEndpointId>,
    #[serde(default)]
    pub displayed_monitoring_row_ids: Vec<ObservedEndpointId>,
    pub integration_module_count: usize,
    pub tracked_app_count: usize,
    pub enabled_tracked_app_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleBackgroundEventDto {
    pub event_type: String,
    pub created_at_ms: TimestampMillis,
    pub context: IntegrationModuleHostContextDto,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleUiActionClientRequestDto {
    pub module_id: String,
    pub action_id: String,
    #[serde(default)]
    pub ui_action_token: String,
    #[serde(default)]
    pub selected_monitoring_row_ids: Vec<ObservedEndpointId>,
    #[serde(default)]
    pub displayed_monitoring_row_ids: Vec<ObservedEndpointId>,
    #[serde(default)]
    pub filters: UiFiltersDto,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleUiActionRequestDto {
    pub action_id: String,
    pub context: IntegrationModuleHostContextDto,
    #[serde(default)]
    pub monitoring_rows: Vec<ObservedEndpoint>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleUiActionResponseDto {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default = "default_system_event_severity")]
    pub severity: String,
    #[serde(default)]
    pub refresh: bool,
    #[serde(default)]
    pub commands: Vec<IntegrationModuleHostCommandDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleHostCommandDto {
    pub command_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationModuleUiActionEventDto {
    pub seq: u64,
    pub module_id: String,
    pub ui_action_token: String,
    pub event_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IntegrationModuleUiActionEventsResponseDto {
    #[serde(default)]
    pub events: Vec<IntegrationModuleUiActionEventDto>,
}

#[repr(C)]
pub struct IntegrationAbiBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

pub type IntegrationAbiEventCallback =
    Option<unsafe extern "C" fn(*const u8, usize, *mut std::ffi::c_void)>;
pub type IntegrationAbiCall = unsafe extern "C" fn(
    *const u8,
    usize,
    IntegrationAbiEventCallback,
    *mut std::ffi::c_void,
    *mut IntegrationAbiBuffer,
) -> i32;
pub type IntegrationAbiFree = unsafe extern "C" fn(*mut u8, usize);

pub const INTEGRATION_ABI_VERSION: u32 = 1;
pub const INTEGRATION_ABI_CALL_EXPORT: &[u8] = b"netstitch_integration_call\0";
pub const INTEGRATION_ABI_FREE_EXPORT: &[u8] = b"netstitch_integration_free\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationHostRequest {
    pub abi_version: u32,
    pub module_id: String,
    pub storage_dir: PathBuf,
    pub action: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationHostResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationHostEvent {
    pub event: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationRootRequestDto {
    #[serde(default)]
    pub repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationProfileExportRequestDto<T> {
    pub request: T,
    #[serde(default)]
    pub endpoints: Vec<ObservedEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationStatusDto {
    pub repo_root: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub reference_data_path: Option<PathBuf>,
    pub export_path: PathBuf,
    #[serde(default)]
    pub profile_paths: Vec<PathBuf>,
    pub is_available: bool,
    pub details: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub provider_name: Option<String>,
    #[serde(default)]
    pub repository_url: Option<String>,
    #[serde(default)]
    pub profile_export_ui_state: Option<ProfileExportUiStateDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationDownloadProgressDto {
    pub active: bool,
    pub provider_id: Option<String>,
    pub stage: String,
    pub percent: Option<u8>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub extracted_entries: Option<u64>,
    pub total_entries: Option<u64>,
    pub message: Option<String>,
    pub repo_root: Option<PathBuf>,
}

impl IntegrationDownloadProgressDto {
    pub fn idle() -> Self {
        Self {
            active: false,
            provider_id: None,
            stage: "idle".to_string(),
            percent: None,
            downloaded_bytes: None,
            total_bytes: None,
            extracted_entries: None,
            total_entries: None,
            message: None,
            repo_root: None,
        }
    }

    pub fn stage(provider_id: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            active: true,
            provider_id: Some(provider_id.into()),
            stage: stage.into(),
            percent: None,
            downloaded_bytes: None,
            total_bytes: None,
            extracted_entries: None,
            total_entries: None,
            message: None,
            repo_root: None,
        }
    }
}

impl IntegrationStatusDto {
    pub fn unavailable(export_path: impl Into<PathBuf>, details: impl Into<String>) -> Self {
        Self {
            repo_root: None,
            data_dir: None,
            reference_data_path: None,
            export_path: export_path.into(),
            profile_paths: Vec::new(),
            is_available: false,
            details: Some(details.into()),
            provider_id: None,
            provider_name: None,
            repository_url: None,
            profile_export_ui_state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationProviderDto {
    pub id: String,
    pub display_name: String,
    pub repository_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceDto {
    pub schema: String,
    pub module_id: String,
    pub surface: UiSurfaceKind,
    #[serde(default)]
    pub entities: Vec<UiEntityDto>,
    #[serde(default)]
    pub actions: Vec<UiActionDto>,
}

impl UiSurfaceDto {
    pub const SCHEMA: &'static str = "netstitch.ui.v1";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSurfaceKind {
    MainPanel,
    Overlay,
    Subpanel,
    HeaderAction,
    Footer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiEntityDto {
    pub stable_id: String,
    pub kind: UiEntityKind,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub i18n_key: Option<String>,
    #[serde(default)]
    pub tooltip_i18n_key: Option<String>,
    #[serde(default)]
    pub state: UiStateDto,
    #[serde(default)]
    pub layout: UiLayoutDto,
    #[serde(default)]
    pub data: UiEntityDataDto,
    #[serde(default)]
    pub children: Vec<UiEntityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiEntityKind {
    Panel,
    PanelHeader,
    PanelFooter,
    Subpanel,
    Row,
    Table,
    TableColumn,
    PathField,
    Modal,
    Progress,
    Tooltip,
    Button,
    ActionButton,
    SegmentedModes,
    Input,
    Textarea,
    Select,
    Switch,
    StatusLabel,
    ValueLabel,
    HelpText,
    Separator,
    FilePicker,
}

impl UiEntityKind {
    pub const fn as_contract_name(&self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::PanelHeader => "panel-header",
            Self::PanelFooter => "panel-footer",
            Self::Subpanel => "subpanel",
            Self::Row => "row",
            Self::Table => "table",
            Self::TableColumn => "table-column",
            Self::PathField => "path-field",
            Self::Modal => "modal",
            Self::Progress => "progress-bar",
            Self::Tooltip => "tooltip",
            Self::Button => "button",
            Self::ActionButton => "action_button",
            Self::SegmentedModes => "segmented-modes",
            Self::Input => "text_input",
            Self::Textarea => "textarea",
            Self::Select => "select",
            Self::Switch => "switch",
            Self::StatusLabel => "status-label",
            Self::ValueLabel => "value-label",
            Self::HelpText => "help-text",
            Self::Separator => "separator",
            Self::FilePicker => "file-picker",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStateDto {
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub busy: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub invalid: bool,
    #[serde(default)]
    pub tone: Option<UiToneDto>,
}

impl Default for UiStateDto {
    fn default() -> Self {
        Self {
            visible: true,
            disabled: false,
            busy: false,
            selected: false,
            invalid: false,
            tone: None,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiToneDto {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UiLayoutDto {
    #[serde(default)]
    pub density: Option<UiDensityDto>,
    #[serde(default)]
    pub align: Option<UiAlignDto>,
    #[serde(default)]
    pub width: Option<UiWidthDto>,
    #[serde(default)]
    pub scroll: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDensityDto {
    Compact,
    Normal,
    Wide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAlignDto {
    Start,
    Center,
    End,
    Between,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidthDto {
    Auto,
    Fill,
    Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UiEntityDataDto {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub label_i18n_key: Option<String>,
    #[serde(default)]
    pub title_i18n_key: Option<String>,
    #[serde(default)]
    pub help_i18n_key: Option<String>,
    #[serde(default)]
    pub placeholder_i18n_key: Option<String>,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub percent: Option<u8>,
    #[serde(default)]
    pub rows: Vec<UiTableRowDto>,
    #[serde(default)]
    pub columns: Vec<UiTableColumnDto>,
    #[serde(default)]
    pub options: Vec<UiOptionDto>,
    #[serde(default)]
    pub footer_actions: Vec<String>,
    #[serde(default)]
    pub meta: Vec<UiKeyValueDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTableColumnDto {
    pub id: String,
    pub label_i18n_key: String,
    #[serde(default)]
    pub sortable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTableRowDto {
    pub key: String,
    #[serde(default)]
    pub cells: Vec<UiTableCellDto>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub tone: Option<UiToneDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTableCellDto {
    pub column_id: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOptionDto {
    pub value: String,
    pub label_i18n_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiKeyValueDto {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionDto {
    pub action_id: String,
    pub kind: UiActionKind,
    #[serde(default)]
    pub target_entity_id: Option<String>,
    #[serde(default)]
    pub payload_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub side_effect: UiActionSideEffect,
    #[serde(default)]
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiActionKind {
    Command,
    Toggle,
    SetValue,
    OpenModal,
    CloseModal,
    BrowseFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiActionSideEffect {
    #[default]
    None,
    LocalFs,
    Network,
    Storage,
    Cloud,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettingsDto {
    pub ui_language_code: Option<String>,
    pub ui_enable_all_overlay: bool,
    pub ui_remember_window_placement: bool,
    pub ui_hide_when_minimized: bool,
    #[serde(default)]
    pub ui_module_order: Vec<String>,
    pub web_access_localhost: bool,
    #[serde(default)]
    pub domain_capture_enabled: bool,
    #[serde(default = "default_update_check_interval_minutes")]
    pub update_check_interval_minutes: u64,
}

impl AppSettingsDto {
    pub fn empty() -> Self {
        Self {
            ui_language_code: None,
            ui_enable_all_overlay: false,
            ui_remember_window_placement: false,
            ui_hide_when_minimized: true,
            ui_module_order: Vec::new(),
            web_access_localhost: false,
            domain_capture_enabled: false,
            update_check_interval_minutes: default_update_check_interval_minutes(),
        }
    }
}

pub fn default_update_check_interval_minutes() -> u64 {
    10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UiObservationFilterDto {
    #[default]
    All,
    Unconfirmed,
    Confirmed,
    Success,
    Exported,
    Failed,
}

impl UiObservationFilterDto {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Unconfirmed => "Unconfirmed",
            Self::Confirmed => "Confirmed",
            Self::Success => "Success",
            Self::Exported => "Exported",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UiFiltersDto {
    #[serde(default)]
    pub app_search: String,
    #[serde(default)]
    pub ip_search: String,
    #[serde(default)]
    pub domain_search: String,
    #[serde(default)]
    pub port_search: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub public_ip: bool,
    #[serde(default)]
    pub observation_filter: UiObservationFilterDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RuntimeStatusDto {
    pub connector_apps_detected: usize,
    pub unavailable_tracked_apps: Vec<TrackedAppAvailabilityDto>,
    #[serde(default)]
    pub integration_modules: Vec<IntegrationModuleRuntimeStatusDto>,
    #[serde(default)]
    pub endpoint_probe: EndpointProbeStatusDto,
    #[serde(default)]
    pub tool_available: bool,
    #[serde(default)]
    pub domain_capture: DomainCaptureStatusDto,
    #[serde(default)]
    pub flow_capture: FlowCaptureStatusDto,
    #[serde(default)]
    pub domain_capture_admin_disabled: bool,
    #[serde(default)]
    pub is_elevated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemEventDto {
    pub event_id: u64,
    pub created_at_ms: u64,
    #[serde(default = "default_system_event_repeat_count")]
    pub repeat_count: u64,
    #[serde(default = "default_system_event_source")]
    pub source: String,
    pub component: String,
    pub action_type: String,
    pub severity: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemEventRequestDto {
    #[serde(default)]
    pub source: Option<String>,
    pub component: String,
    pub action_type: String,
    #[serde(default = "default_system_event_severity")]
    pub severity: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

fn default_system_event_severity() -> String {
    "info".to_string()
}

fn default_system_event_repeat_count() -> u64 {
    1
}

fn default_system_event_source() -> String {
    "core".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleRuntimeStatusDto {
    pub id: String,
    pub display_name: String,
    pub manifest_path: PathBuf,
    pub connected: bool,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub background_active: bool,
    #[serde(default)]
    pub background_subscriptions: Vec<String>,
    #[serde(default)]
    pub background_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EndpointProbeStatusDto {
    pub is_checking: bool,
    pub first_successful_target: Option<String>,
    pub probes: Vec<EndpointProbeTargetDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointProbeTargetDto {
    pub target: String,
    pub available: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DomainCaptureStatusDto {
    pub backend_started: bool,
    pub backend_error: Option<String>,
    pub packets_seen: u64,
    pub tcp_payload_packets: u64,
    pub domains_detected: u64,
    pub domains_matched: u64,
    pub pending_packets: u64,
    pub dropped_no_owner: u64,
    pub dropped_no_process: u64,
    pub dropped_untracked_process: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FlowCaptureStatusDto {
    pub flow_backend_started: bool,
    pub packet_backend_started: bool,
    pub backend_error: Option<String>,
    pub flow_events: u64,
    pub udp_flow_events: u64,
    pub packet_events: u64,
    pub udp_packet_events: u64,
    pub observations_emitted: u64,
    pub matched_tracked_app: u64,
    pub dropped_no_owner: u64,
    pub dropped_no_process: u64,
    pub dropped_untracked_process: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedAppAvailabilityDto {
    pub tracked_app_id: Option<TrackedAppId>,
    pub display_name: Option<String>,
    pub exe_path: PathBuf,
    pub connector_managed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub monitor_status: MonitorStatus,
    pub tracked_apps: Vec<TrackedApp>,
    pub observed_endpoints: Vec<ObservedEndpoint>,
    pub aggregated_ips: Vec<AggregatedIpDto>,
    pub ignored_addresses: Vec<IgnoredAddressRule>,
    pub capability_report: CapabilityReport,
    #[serde(default)]
    pub integration_modules: Vec<IntegrationModuleDto>,
    #[serde(default)]
    pub integration_providers: Vec<IntegrationProviderDto>,
    pub integration_status: Option<IntegrationStatusDto>,
    pub app_settings: AppSettingsDto,
    #[serde(default)]
    pub filters: UiFiltersDto,
    #[serde(default)]
    pub runtime_status: RuntimeStatusDto,
}

impl SnapshotResponse {
    pub fn empty(monitor_status: MonitorStatus) -> Self {
        Self {
            monitor_status,
            tracked_apps: Vec::new(),
            observed_endpoints: Vec::new(),
            aggregated_ips: Vec::new(),
            ignored_addresses: Vec::new(),
            capability_report: CapabilityReport::baseline(),
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            integration_status: None,
            app_settings: AppSettingsDto::empty(),
            filters: UiFiltersDto::default(),
            runtime_status: RuntimeStatusDto::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationOverlayTarget {
    pub repo_root: PathBuf,
    pub data_dir: PathBuf,
    pub reference_data_path: PathBuf,
    pub export_path: PathBuf,
}

impl IntegrationOverlayTarget {
    pub fn new(
        repo_root: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        reference_data_path: impl Into<PathBuf>,
        export_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            repo_root: repo_root.into(),
            data_dir: data_dir.into(),
            reference_data_path: reference_data_path.into(),
            export_path: export_path.into(),
        }
    }
}
