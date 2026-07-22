use netstitch_shared::models::{
    ExportProfileAdvancedSettingsRequestDto, ExportProfilePlanDto, ExportProfileRequestDto,
    IntegrationModuleDto, IntegrationModuleUiActionClientRequestDto,
    IntegrationModuleUiActionResponseDto, IntegrationProviderDto, MonitoringCsvImportRequestDto,
    MonitoringCsvImportResultDto, ProfileExportUiStateDto,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotResponse {
    pub tracked_apps: Vec<TrackedAppDto>,
    pub observations: Vec<ObservationDto>,
    pub ignored_addresses: Vec<IgnoredAddressDto>,
    pub integration: IntegrationIntegrationDto,
    pub integration_modules: Vec<IntegrationModuleDto>,
    pub integration_providers: Vec<IntegrationProviderDto>,
    pub ui: UiStatusDto,
    pub app_settings: AppSettingsDto,
    pub runtime_status: RuntimeStatusDto,
    pub filters: FiltersDto,
    pub pending_exe_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSettingsDto {
    pub language_code: Option<String>,
    pub enable_all_overlay: bool,
    pub remember_window_placement: bool,
    pub hide_when_minimized: bool,
    pub module_order: Vec<String>,
    pub web_access_localhost: bool,
    pub domain_capture_enabled: bool,
    pub update_check_interval_minutes: u64,
    pub profile_export_ui_state: Option<ProfileExportUiStateDto>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RuntimeStatusDto {
    pub connector_apps_detected: usize,
    pub unavailable_tracked_apps: Vec<TrackedAppAvailabilityDto>,
    pub integration_modules: Vec<IntegrationModuleRuntimeStatusDto>,
    pub endpoint_probe: EndpointProbeStatusDto,
    pub tool_available: bool,
    pub domain_capture: DomainCaptureStatusDto,
    pub flow_capture: FlowCaptureStatusDto,
    pub domain_capture_admin_disabled: bool,
    pub is_elevated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationModuleRuntimeStatusDto {
    pub id: String,
    pub display_name: String,
    pub manifest_path: String,
    pub connected: bool,
    pub enabled: bool,
    pub error: Option<String>,
    pub background_active: bool,
    pub background_subscriptions: Vec<String>,
    pub background_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EndpointProbeStatusDto {
    pub is_checking: bool,
    pub first_successful_target: Option<String>,
    pub probes: Vec<EndpointProbeTargetDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointProbeTargetDto {
    pub target: String,
    pub available: Option<bool>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
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

#[derive(Clone, Debug, PartialEq, Default)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct TrackedAppAvailabilityDto {
    pub tracked_app_id: u64,
    pub display_name: String,
    pub exe_path: String,
    pub connector_managed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackedAppDto {
    pub id: u64,
    pub connector_id: Option<String>,
    pub cloud_app_id: Option<String>,
    pub display_name: String,
    pub icon_key: String,
    pub icon_path: Option<String>,
    pub current_tag: Option<String>,
    pub exe_path: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolDto {
    Tcp,
    Udp,
    Other,
}

impl ProtocolDto {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Other => "OTHER",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStateDto {
    Unknown,
    Attempting,
    Established,
    Closing,
    Failed,
}

impl ConnectionStateDto {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Attempting => "Attempting",
            Self::Established => "Established",
            Self::Closing => "Closing",
            Self::Failed => "Failed",
        }
    }

    pub fn is_failure_like(&self) -> bool {
        matches!(self, Self::Attempting | Self::Failed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationDto {
    pub id: u64,
    pub tracked_app_id: u64,
    pub cloud_app_id: Option<String>,
    pub app_signature_key: Option<String>,
    pub app_signature_subject: Option<String>,
    pub app_signature_issuer: Option<String>,
    pub app_signature_source: Option<String>,
    pub process_name: String,
    pub remote_ip: String,
    pub remote_port: u16,
    pub protocol: ProtocolDto,
    pub first_seen_ms: u64,
    pub first_seen: String,
    pub last_seen_ms: u64,
    pub last_seen: String,
    pub hits: u32,
    pub connection_state: ConnectionStateDto,
    pub failed_hits: u32,
    pub successful_hits: u32,
    pub is_confirmed: bool,
    pub is_exported: bool,
    pub tags: Vec<String>,
    pub enrichment: Option<IpEnrichmentDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredAddressDto {
    pub id: u64,
    pub address_pattern: String,
    pub created_at: String,
    pub enrichment: Option<IpEnrichmentDto>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct IpEnrichmentDto {
    pub domain_name: Option<String>,
    pub domain_source: Option<String>,
    pub owner_name: Option<String>,
    pub owner_range: Option<String>,
    pub registry: Option<String>,
    pub country: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationIntegrationDto {
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
pub struct UiStatusDto {
    pub monitoring: bool,
    pub watcher_connected: bool,
    pub snapshot_loaded: bool,
    pub status_text: String,
    pub error_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObservationFilterDto {
    All,
    Unconfirmed,
    Confirmed,
    Success,
    Exported,
    Failed,
}

impl ObservationFilterDto {
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

#[derive(Clone, Debug, PartialEq)]
pub struct FiltersDto {
    pub app_search: String,
    pub search_text: String,
    pub domain_search: String,
    pub port_search: String,
    pub protocol: String,
    pub public_ip: bool,
    pub observation_filter: ObservationFilterDto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddTrackedAppRequest {
    pub exe_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToggleTrackedAppRequest {
    pub app_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteTrackedAppRequest {
    pub app_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetTrackedAppTagRequest {
    pub app_id: u64,
    pub tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetAllTrackedAppsEnabledRequest {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmObservationsRequest {
    pub observation_ids: Vec<u64>,
    pub confirmed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct MarkObservationsExportedRequest {
    pub observation_ids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteObservationRequest {
    pub observation_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteObservationsRequest {
    pub observation_ids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClearObservationTagsRequest {
    pub observation_ids: Vec<u64>,
    pub tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteLocalTagRequest {
    pub tag: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IgnoreAddressRequest {
    pub address_pattern: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteIgnoredAddressRequest {
    pub ignored_address_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DownloadIntegrationRequest {
    pub module_id: Option<String>,
    pub provider_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetFilterRequest {
    pub app_search: String,
    pub search_text: String,
    pub domain_search: String,
    pub port_search: String,
    pub protocol: String,
    pub public_ip: bool,
    pub observation_filter: ObservationFilterDto,
}

pub trait WatcherApiClient {
    fn snapshot(&self) -> SnapshotResponse;
    fn set_pending_exe_path(&mut self, pending_exe_path: String);
    fn add_tracked_app(&mut self, request: AddTrackedAppRequest);
    fn toggle_tracked_app(&mut self, request: ToggleTrackedAppRequest);
    fn delete_tracked_app(&mut self, request: DeleteTrackedAppRequest);
    fn set_tracked_app_tag(&mut self, request: SetTrackedAppTagRequest) -> Result<(), String>;
    fn set_all_tracked_apps_enabled(&mut self, request: SetAllTrackedAppsEnabledRequest);
    fn start_monitoring(&mut self);
    fn stop_monitoring(&mut self);
    fn confirm_observations(&mut self, request: ConfirmObservationsRequest);
    #[allow(dead_code)]
    fn mark_observations_exported(
        &mut self,
        request: MarkObservationsExportedRequest,
    ) -> Result<usize, String>;
    fn delete_observation(&mut self, request: DeleteObservationRequest) -> Result<usize, String>;
    fn delete_observations(&mut self, request: DeleteObservationsRequest) -> Result<usize, String>;
    fn clear_observation_tags(
        &mut self,
        request: ClearObservationTagsRequest,
    ) -> Result<usize, String>;
    fn delete_local_tag(&mut self, request: DeleteLocalTagRequest) -> Result<usize, String>;
    fn import_monitoring_csv(
        &mut self,
        request: MonitoringCsvImportRequestDto,
    ) -> Result<MonitoringCsvImportResultDto, String>;
    fn preview_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn analyze_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn analyze_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn apply_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn apply_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn backup_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn revert_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String>;
    fn set_profile_export_ui_state(
        &mut self,
        state: netstitch_shared::models::ProfileExportUiStateDto,
    );
    fn run_integration_module_ui_action(
        &mut self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto, String>;
    fn stop_integration_module_background(&mut self, module_id: String) -> Result<bool, String>;
    fn send_integration_module_dialog_result(
        &mut self,
        module_id: String,
        dialog_id: String,
        result: String,
    ) -> Result<IntegrationModuleUiActionResponseDto, String>;
    fn set_filters(&mut self, request: SetFilterRequest);
    fn set_language_code(&mut self, language_code: String);
    fn set_module_order(&mut self, order: Vec<String>);
    fn set_web_access_localhost(&mut self, enabled: bool);
    fn set_domain_capture_enabled(&mut self, enabled: bool);
    fn set_remember_window_placement(&mut self, enabled: bool);
    fn set_hide_when_minimized(&mut self, enabled: bool);
    fn ignore_address(&mut self, request: IgnoreAddressRequest);
    fn delete_ignored_address(&mut self, request: DeleteIgnoredAddressRequest);
    fn download_integration(&mut self, request: DownloadIntegrationRequest);
}
