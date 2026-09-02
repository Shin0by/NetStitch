use std::cell::RefCell;
use std::env;
use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Local, Utc};
use netstitch_core::NetstitchCore;
use netstitch_shared::CloudUserSessionResponse;
use netstitch_shared::ipc::{
    AddIgnoredAddressRequest as SharedAddIgnoredAddressRequest,
    AddTrackedAppRequest as SharedAddTrackedAppRequest,
    ClearObservationTagsRequest as SharedClearObservationTagsRequest, ConfirmEndpointsRequest,
    DeleteIgnoredAddressRequest as SharedDeleteIgnoredAddressRequest,
    DeleteLocalTagRequest as SharedDeleteLocalTagRequest,
    DeleteLocalTagResponse as SharedDeleteLocalTagResponse,
    DeleteObservationsRequest as SharedDeleteObservationsRequest,
    DeleteTrackedAppRequest as SharedDeleteTrackedAppRequest,
    DownloadIntegrationProviderRequest as SharedDownloadIntegrationProviderRequest,
    MarkEndpointsExportedRequest, MonitorCommandResponse, SetAppSettingRequest,
    SetTrackedAppEnabledRequest, SetTrackedAppTagRequest as SharedSetTrackedAppTagRequest,
};
use netstitch_shared::models::{
    CLIENT_HEADER_DESKTOP_UI, CLIENT_HEADER_NAME, ConnectionState as SharedConnectionState,
    ExportProfileAdvancedSettingsRequestDto, ExportProfilePlanDto, ExportProfileRequestDto,
    IntegrationModuleUiActionClientRequestDto, IntegrationModuleUiActionEventsResponseDto,
    IntegrationModuleUiActionResponseDto, MonitorStatus, MonitoringCsvImportRequestDto,
    MonitoringCsvImportResultDto, MonitoringImportSourceDto, NetworkDiagnosticProgressDto,
    NetworkDiagnosticRequestDto, NetworkDiagnosticResultDto, NetworkDiagnosticStartDto,
    ObservedEndpoint, ProfileExportUiStateDto, Protocol as SharedProtocol,
    SETTING_DOMAIN_CAPTURE_ENABLED, SETTING_UI_ENABLE_ALL_OVERLAY, SETTING_UI_HIDE_WHEN_MINIMIZED,
    SETTING_UI_LANGUAGE, SETTING_UI_MODULE_ORDER, SETTING_UI_MONITORING_PUBLIC_IP,
    SETTING_UI_MONITORING_SHOW_CONNECTION_COUNT, SETTING_UI_MONITORING_SHOW_TAGS,
    SETTING_UI_REMEMBER_WINDOW_PLACEMENT, SETTING_UI_WINDOW_HEIGHT, SETTING_UI_WINDOW_HIDDEN,
    SETTING_UI_WINDOW_WIDTH, SETTING_UI_WINDOW_X, SETTING_UI_WINDOW_Y,
    SETTING_UPDATE_CHECK_INTERVAL_MINUTES, SETTING_WEB_ACCESS_LOCALHOST,
    SnapshotResponse as SharedSnapshotResponse, UiFiltersDto as SharedUiFiltersDto,
    UiObservationFilterDto as SharedUiObservationFilterDto, default_update_check_interval_minutes,
};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use rusqlite::OptionalExtension;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::mock::MockWatcherApi;
use crate::watcher_api::{
    AddTrackedAppRequest, AppSettingsDto, ClearObservationTagsRequest, ConfirmObservationsRequest,
    ConnectionStateDto, DeleteIgnoredAddressRequest, DeleteLocalTagRequest,
    DeleteObservationRequest, DeleteObservationsRequest, DeleteTrackedAppRequest,
    DownloadIntegrationRequest, FiltersDto, IgnoreAddressRequest, IgnoredAddressDto,
    IntegrationIntegrationDto, IntegrationModuleRuntimeStatusDto, MarkObservationsExportedRequest,
    ObservationDto, ObservationFilterDto, ProtocolDto, RuntimeStatusDto,
    SetAllTrackedAppsEnabledRequest, SetFilterRequest, SetTrackedAppTagRequest, SnapshotResponse,
    ToggleTrackedAppRequest, TrackedAppAvailabilityDto, TrackedAppDto, UiStatusDto,
    WatcherApiClient,
};

const DEFAULT_WATCHER_PORT: u16 = 46473;
const APP_ENV_WATCHER_ADDR: &str = "NETSTITCH__WATCHER_ADDR";
const APP_ENV_WEB_SCHEME: &str = "NETSTITCH__WEB_SCHEME";
const APP_ENV_UI_USE_MOCK: &str = "NETSTITCH__UI_USE_MOCK";
const APP_ENV_INTEGRATION_DIR: &str = "NETSTITCH__INTEGRATION_DIR";
const APP_ENV_ENDPOINT_PROBE_TARGETS: &str = "NETSTITCH__ENDPOINT_PROBE_TARGETS";
const APP_ENV_ENDPOINT_PROBE_TARGETS_FILE: &str = "NETSTITCH__ENDPOINT_PROBE_TARGETS_FILE";
const ENDPOINT_PROBE_TARGETS_FILE: &str = "endpoint_probe_targets.txt";

#[derive(Debug, Deserialize)]
struct DeleteObservationsResponse {
    deleted: usize,
}

#[derive(Debug, Deserialize)]
struct ClearObservationTagsResponse {
    cleared: usize,
}

const DEFAULT_WATCHER_HOST: &str = "127.0.0.1";
const SETTING_OBSOLETE_CLOUD_LOGIN: &str = "cloud.auth.login";
const SETTING_OBSOLETE_CLOUD_EMAIL: &str = "cloud.auth.email";
const SETTING_CLOUD_EMAIL_DPAPI: &str = "cloud.auth.email.dpapi.v1";
const SETTING_CLOUD_PROVIDER: &str = "cloud.auth.provider";
const SETTING_CLOUD_PROVIDER_SUBJECT_HASH: &str = "cloud.auth.provider_subject_hash";
const SETTING_OBSOLETE_CLOUD_DISPLAY_NAME: &str = "cloud.auth.display_name";
const SETTING_CLOUD_DISPLAY_NAME_DPAPI: &str = "cloud.auth.display_name.dpapi.v1";
const SETTING_OBSOLETE_CLOUD_PASSWORD_DPAPI: &str = "cloud.auth.password.dpapi.v1";
const SETTING_CLOUD_USER_ID: &str = "cloud.auth.user_id";
const SETTING_CLOUD_CLIENT_ID: &str = "cloud.auth.client_id";
const SETTING_CLOUD_KEY_ID: &str = "cloud.auth.key_id";
const SETTING_CLOUD_SESSION_TOKEN_DPAPI: &str = "cloud.auth.session_token.dpapi.v1";
const SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI: &str = "cloud.auth.client_private_key.dpapi.v1";
const SETTING_CLOUD_SESSION_EXPIRES_AT_MS: &str = "cloud.auth.session_expires_at_ms";
const SETTING_OBSOLETE_CLOUD_STAY_SIGNED_IN: &str = "cloud.auth.stay_signed_in";
const SETTING_CLOUD_UPLOAD_NICKNAME: &str = "cloud.upload.nickname";

#[derive(Serialize)]
struct ScopedProfileExportRequest<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    module_id: Option<String>,
    request: T,
}

fn desktop_client_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static(CLIENT_HEADER_NAME),
        HeaderValue::from_static(CLIENT_HEADER_DESKTOP_UI),
    );
    headers
}
pub(crate) struct SnapshotRefreshJob {
    receiver: mpsc::Receiver<SnapshotRefreshResult>,
    epoch: u64,
}

pub(crate) struct ModuleUiActionJob {
    receiver: mpsc::Receiver<Result<IntegrationModuleUiActionResponseDto, String>>,
}

pub(crate) struct NetworkDiagnosticJob {
    receiver: mpsc::Receiver<NetworkDiagnosticJobEvent>,
}

pub(crate) enum NetworkDiagnosticJobEvent {
    Progress(String),
    Finished(NetworkDiagnosticResultDto),
    Failed(String),
}

pub(crate) struct SnapshotRefreshResult {
    snapshot: Result<SharedSnapshotResponse, String>,
    watcher_connected: bool,
    status_note: Option<String>,
    failure_message: Option<String>,
    epoch: u64,
}

impl SnapshotRefreshJob {
    pub(crate) fn try_finish(&self) -> Option<SnapshotRefreshResult> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(SnapshotRefreshResult {
                snapshot: Err("background snapshot refresh disconnected".to_string()),
                watcher_connected: false,
                status_note: None,
                failure_message: None,
                epoch: self.epoch,
            }),
        }
    }
}

impl ModuleUiActionJob {
    pub(crate) fn try_finish(
        &self,
    ) -> Option<Result<IntegrationModuleUiActionResponseDto, String>> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("module UI action worker disconnected".to_string()))
            }
        }
    }
}

impl NetworkDiagnosticJob {
    pub(crate) fn try_event(&self) -> Option<NetworkDiagnosticJobEvent> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(NetworkDiagnosticJobEvent::Failed(
                "network diagnostic worker disconnected".to_string(),
            )),
        }
    }
}

#[derive(Clone)]
pub struct AppWatcherApi {
    inner: WatcherApiKind,
}

#[derive(Clone)]
enum WatcherApiKind {
    Live(LiveWatcherApi),
    Mock(MockWatcherApi),
}

#[derive(Clone)]
struct LiveWatcherApi {
    client: Client,
    base_url: String,
    state: Rc<RefCell<LiveState>>,
}

struct LiveState {
    snapshot: SnapshotResponse,
    filters: FiltersDto,
    pending_exe_path: String,
    integration_repo_root: Option<PathBuf>,
    last_export_text: String,
    last_status_note: Option<String>,
    last_refresh_at: Option<Instant>,
    snapshot_epoch: u64,
    refresh_in_progress: bool,
}

fn import_monitoring_status_note(
    import_source: MonitoringImportSourceDto,
    result: &MonitoringCsvImportResultDto,
) -> String {
    let source_label = match import_source {
        MonitoringImportSourceDto::Csv => "CSV imported",
        MonitoringImportSourceDto::CloudDownload => "Cloud rows added",
    };
    format!(
        "{source_label}: {}/{} row(s), {} skipped",
        result.imported_count, result.requested_count, result.skipped_count
    )
}

#[derive(Serialize)]
struct ConfigureIntegrationRootRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    module_id: Option<String>,
    repo_root: Option<String>,
}

#[derive(Serialize)]
struct StopIntegrationModuleBackgroundRequest {
    module_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PersistedWindowPlacement {
    pub x: i32,
    pub y: i32,
    pub width: f64,
    pub height: f64,
    pub hidden: bool,
}

impl AppWatcherApi {
    pub fn cold_start() -> Self {
        if should_use_mock() {
            return Self {
                inner: WatcherApiKind::Mock(MockWatcherApi::seeded()),
            };
        }

        let live = LiveWatcherApi::cold_start();
        Self {
            inner: WatcherApiKind::Live(live),
        }
    }

    pub(crate) fn start_snapshot_refresh_job(&self) -> Option<SnapshotRefreshJob> {
        match &self.inner {
            WatcherApiKind::Live(api) => api.start_snapshot_refresh_job(),
            WatcherApiKind::Mock(_) => None,
        }
    }

    pub(crate) fn apply_snapshot_refresh_result(&self, result: SnapshotRefreshResult) -> bool {
        if let WatcherApiKind::Live(api) = &self.inner {
            api.apply_snapshot_refresh_result(result)
        } else {
            false
        }
    }

    pub(crate) fn start_integration_module_ui_action_job(
        &self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> ModuleUiActionJob {
        match &self.inner {
            WatcherApiKind::Live(api) => api.start_integration_module_ui_action_job(request),
            WatcherApiKind::Mock(api) => {
                let (sender, receiver) = mpsc::channel();
                let mut api = api.clone();
                std::thread::spawn(move || {
                    let _ = sender.send(api.run_integration_module_ui_action(request));
                });
                ModuleUiActionJob { receiver }
            }
        }
    }

    pub(crate) fn start_network_diagnostic_job(
        &self,
        request: NetworkDiagnosticRequestDto,
    ) -> NetworkDiagnosticJob {
        let (sender, receiver) = mpsc::channel();
        match &self.inner {
            WatcherApiKind::Live(api) => {
                let base_url = api.base_url.clone();
                std::thread::spawn(move || {
                    if let Err(error) =
                        send_network_diagnostic_in_background(base_url, request, &sender)
                    {
                        let _ = sender.send(NetworkDiagnosticJobEvent::Failed(error));
                    }
                });
            }
            WatcherApiKind::Mock(_) => {
                std::thread::spawn(move || {
                    let target = request.target.trim().to_string();
                    let output = format!("Mock {} result for {target}", request.kind.as_str());
                    let _ = sender.send(NetworkDiagnosticJobEvent::Progress(output.clone()));
                    std::thread::sleep(Duration::from_millis(25));
                    let _ = sender.send(NetworkDiagnosticJobEvent::Finished(
                        NetworkDiagnosticResultDto {
                            kind: request.kind,
                            target: target.clone(),
                            dns_server: request.dns_server,
                            success: true,
                            output,
                            exit_code: Some(0),
                            duration_ms: 1,
                        },
                    ));
                });
            }
        }
        NetworkDiagnosticJob { receiver }
    }

    pub(crate) fn apply_integration_module_ui_action_response(
        &self,
        response: &IntegrationModuleUiActionResponseDto,
    ) {
        if let WatcherApiKind::Live(api) = &self.inner {
            api.apply_integration_module_ui_action_response(response);
        }
    }

    pub(crate) fn integration_module_ui_action_events(
        &self,
        module_id: &str,
        ui_action_token: &str,
        after: u64,
    ) -> Result<IntegrationModuleUiActionEventsResponseDto, String> {
        match &self.inner {
            WatcherApiKind::Live(api) => {
                api.integration_module_ui_action_events(module_id, ui_action_token, after)
            }
            WatcherApiKind::Mock(_) => Ok(IntegrationModuleUiActionEventsResponseDto::default()),
        }
    }

    pub fn configure_integration_folder(
        &mut self,
        module_id: Option<String>,
        candidate: &str,
    ) -> Result<(), String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.configure_integration_folder(module_id, candidate),
            WatcherApiKind::Mock(api) => api.configure_integration_folder(module_id, candidate),
        }
    }

    pub fn shutdown_embedded_watcher(&mut self) {
        if let WatcherApiKind::Live(api) = &mut self.inner {
            api.shutdown_embedded_watcher();
        }
    }

    pub(crate) fn live_base_url(&self) -> Option<String> {
        match &self.inner {
            WatcherApiKind::Live(api) => Some(api.base_url.clone()),
            WatcherApiKind::Mock(_) => None,
        }
    }

    pub(crate) fn tool_available(&self) -> bool {
        match &self.inner {
            WatcherApiKind::Live(api) => api.tool_available(),
            WatcherApiKind::Mock(api) => api.snapshot().runtime_status.tool_available,
        }
    }

    pub(crate) fn confirm_observations_deferred(&mut self, request: ConfirmObservationsRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.confirm_observations_deferred(request),
            WatcherApiKind::Mock(api) => api.confirm_observations(request),
        }
    }
}

impl WatcherApiClient for AppWatcherApi {
    fn snapshot(&self) -> SnapshotResponse {
        match &self.inner {
            WatcherApiKind::Live(api) => api.snapshot(),
            WatcherApiKind::Mock(api) => api.snapshot(),
        }
    }

    fn set_pending_exe_path(&mut self, pending_exe_path: String) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_pending_exe_path(pending_exe_path),
            WatcherApiKind::Mock(api) => api.set_pending_exe_path(pending_exe_path),
        }
    }

    fn add_tracked_app(&mut self, request: AddTrackedAppRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.add_tracked_app(request),
            WatcherApiKind::Mock(api) => api.add_tracked_app(request),
        }
    }

    fn toggle_tracked_app(&mut self, request: ToggleTrackedAppRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.toggle_tracked_app(request),
            WatcherApiKind::Mock(api) => api.toggle_tracked_app(request),
        }
    }

    fn delete_tracked_app(&mut self, request: DeleteTrackedAppRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.delete_tracked_app(request),
            WatcherApiKind::Mock(api) => api.delete_tracked_app(request),
        }
    }

    fn set_tracked_app_tag(&mut self, request: SetTrackedAppTagRequest) -> Result<(), String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_tracked_app_tag(request),
            WatcherApiKind::Mock(api) => api.set_tracked_app_tag(request),
        }
    }

    fn set_all_tracked_apps_enabled(&mut self, request: SetAllTrackedAppsEnabledRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_all_tracked_apps_enabled(request),
            WatcherApiKind::Mock(api) => api.set_all_tracked_apps_enabled(request),
        }
    }

    fn start_monitoring(&mut self) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.start_monitoring(),
            WatcherApiKind::Mock(api) => api.start_monitoring(),
        }
    }

    fn stop_monitoring(&mut self) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.stop_monitoring(),
            WatcherApiKind::Mock(api) => api.stop_monitoring(),
        }
    }

    fn confirm_observations(&mut self, request: ConfirmObservationsRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.confirm_observations(request),
            WatcherApiKind::Mock(api) => api.confirm_observations(request),
        }
    }

    fn mark_observations_exported(
        &mut self,
        request: MarkObservationsExportedRequest,
    ) -> Result<usize, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.mark_observations_exported(request),
            WatcherApiKind::Mock(api) => api.mark_observations_exported(request),
        }
    }

    fn delete_observation(&mut self, request: DeleteObservationRequest) -> Result<usize, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.delete_observation(request),
            WatcherApiKind::Mock(api) => api.delete_observation(request),
        }
    }

    fn delete_observations(&mut self, request: DeleteObservationsRequest) -> Result<usize, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.delete_observations(request),
            WatcherApiKind::Mock(api) => api.delete_observations(request),
        }
    }

    fn clear_observation_tags(
        &mut self,
        request: ClearObservationTagsRequest,
    ) -> Result<usize, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.clear_observation_tags(request),
            WatcherApiKind::Mock(api) => api.clear_observation_tags(request),
        }
    }

    fn delete_local_tag(&mut self, request: DeleteLocalTagRequest) -> Result<usize, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.delete_local_tag(request),
            WatcherApiKind::Mock(api) => api.delete_local_tag(request),
        }
    }

    fn import_monitoring_csv(
        &mut self,
        request: MonitoringCsvImportRequestDto,
    ) -> Result<MonitoringCsvImportResultDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.import_monitoring_csv(request),
            WatcherApiKind::Mock(api) => api.import_monitoring_csv(request),
        }
    }

    fn preview_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.preview_profile_export(module_id, request),
            WatcherApiKind::Mock(api) => api.preview_profile_export(module_id, request),
        }
    }

    fn analyze_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.analyze_profile_export(module_id, request),
            WatcherApiKind::Mock(api) => api.analyze_profile_export(module_id, request),
        }
    }

    fn analyze_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => {
                api.analyze_profile_export_advanced_settings(module_id, request)
            }
            WatcherApiKind::Mock(api) => {
                api.analyze_profile_export_advanced_settings(module_id, request)
            }
        }
    }

    fn apply_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.apply_profile_export(module_id, request),
            WatcherApiKind::Mock(api) => api.apply_profile_export(module_id, request),
        }
    }

    fn apply_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => {
                api.apply_profile_export_advanced_settings(module_id, request)
            }
            WatcherApiKind::Mock(api) => {
                api.apply_profile_export_advanced_settings(module_id, request)
            }
        }
    }

    fn backup_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.backup_profile_export(module_id, request),
            WatcherApiKind::Mock(api) => api.backup_profile_export(module_id, request),
        }
    }

    fn revert_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.revert_profile_export(module_id, request),
            WatcherApiKind::Mock(api) => api.revert_profile_export(module_id, request),
        }
    }

    fn set_profile_export_ui_state(&mut self, state: ProfileExportUiStateDto) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_profile_export_ui_state(state),
            WatcherApiKind::Mock(api) => api.set_profile_export_ui_state(state),
        }
    }

    fn run_integration_module_ui_action(
        &mut self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.run_integration_module_ui_action(request),
            WatcherApiKind::Mock(api) => api.run_integration_module_ui_action(request),
        }
    }

    fn stop_integration_module_background(&mut self, module_id: String) -> Result<bool, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.stop_integration_module_background(module_id),
            WatcherApiKind::Mock(api) => api.stop_integration_module_background(module_id),
        }
    }

    fn send_integration_module_dialog_result(
        &mut self,
        module_id: String,
        dialog_id: String,
        result: String,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        match &mut self.inner {
            WatcherApiKind::Live(api) => {
                api.send_integration_module_dialog_result(module_id, dialog_id, result)
            }
            WatcherApiKind::Mock(api) => {
                api.send_integration_module_dialog_result(module_id, dialog_id, result)
            }
        }
    }

    fn set_filters(&mut self, request: SetFilterRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_filters(request),
            WatcherApiKind::Mock(api) => api.set_filters(request),
        }
    }

    fn set_language_code(&mut self, language_code: String) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_language_code(language_code),
            WatcherApiKind::Mock(api) => api.set_language_code(language_code),
        }
    }

    fn set_module_order(&mut self, order: Vec<String>) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_module_order(order),
            WatcherApiKind::Mock(api) => api.set_module_order(order),
        }
    }

    fn set_monitoring_show_tags(&mut self, visible: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_monitoring_show_tags(visible),
            WatcherApiKind::Mock(api) => api.set_monitoring_show_tags(visible),
        }
    }

    fn set_monitoring_show_connection_count(&mut self, visible: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_monitoring_show_connection_count(visible),
            WatcherApiKind::Mock(api) => api.set_monitoring_show_connection_count(visible),
        }
    }

    fn set_web_access_localhost(&mut self, enabled: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_web_access_localhost(enabled),
            WatcherApiKind::Mock(api) => api.set_web_access_localhost(enabled),
        }
    }

    fn set_domain_capture_enabled(&mut self, enabled: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_domain_capture_enabled(enabled),
            WatcherApiKind::Mock(api) => api.set_domain_capture_enabled(enabled),
        }
    }

    fn set_remember_window_placement(&mut self, enabled: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_remember_window_placement(enabled),
            WatcherApiKind::Mock(api) => api.set_remember_window_placement(enabled),
        }
    }

    fn set_hide_when_minimized(&mut self, enabled: bool) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.set_hide_when_minimized(enabled),
            WatcherApiKind::Mock(api) => api.set_hide_when_minimized(enabled),
        }
    }

    fn ignore_address(&mut self, request: IgnoreAddressRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.ignore_address(request),
            WatcherApiKind::Mock(api) => api.ignore_address(request),
        }
    }

    fn delete_ignored_address(&mut self, request: DeleteIgnoredAddressRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.delete_ignored_address(request),
            WatcherApiKind::Mock(api) => api.delete_ignored_address(request),
        }
    }

    fn download_integration(&mut self, request: DownloadIntegrationRequest) {
        match &mut self.inner {
            WatcherApiKind::Live(api) => api.download_integration(request),
            WatcherApiKind::Mock(api) => api.download_integration(request),
        }
    }
}

impl LiveWatcherApi {
    fn cold_start() -> Self {
        let explicit_watcher_addr = env::var(APP_ENV_WATCHER_ADDR)
            .ok()
            .filter(|value| !value.trim().is_empty());
        let raw_addr = explicit_watcher_addr
            .clone()
            .unwrap_or_else(default_watcher_addr);
        let base_url = if raw_addr.starts_with("http://") || raw_addr.starts_with("https://") {
            raw_addr
        } else {
            format!("{}://{raw_addr}", default_web_scheme())
        };
        let integration_repo_root = env::var(APP_ENV_INTEGRATION_DIR)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);

        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .danger_accept_invalid_certs(true)
            .default_headers(desktop_client_headers())
            .build()
            .expect("failed to construct reqwest blocking client");

        let public_ip_filter = read_app_setting_from_sqlite(SETTING_UI_MONITORING_PUBLIC_IP)
            .as_deref()
            .map(setting_truthy)
            .unwrap_or(true);
        let mut state = LiveState {
            snapshot: initial_snapshot(&base_url, cold_start_settings()),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: public_ip_filter,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
            integration_repo_root,
            last_export_text: "No export yet".to_string(),
            last_status_note: Some("Connecting to embedded monitoring runtime...".to_string()),
            last_refresh_at: Some(Instant::now()),
            snapshot_epoch: 0,
            refresh_in_progress: false,
        };
        state.snapshot.pending_exe_path = state.pending_exe_path.clone();
        state.snapshot.filters = state.filters.clone();
        Self {
            client,
            base_url,
            state: Rc::new(RefCell::new(state)),
        }
    }

    fn persist_monitoring_visibility_setting(&mut self, key: &str, hidden: bool) {
        let payload = SetAppSettingRequest {
            key: key.to_string(),
            value: hidden.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            let _ = persist_app_setting_to_sqlite(&payload.key, &payload.value);
            self.set_request_error(&message);
        }
    }

    fn configure_integration_folder(
        &mut self,
        module_id: Option<String>,
        candidate: &str,
    ) -> Result<(), String> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            self.persist_integration_root(module_id, None)?;
            let mut state = self.state.borrow_mut();
            state.integration_repo_root = None;
            state.last_status_note = Some("Integration root cleared".to_string());
            state.snapshot.integration.repo_path = "Not configured".to_string();
            state.snapshot.integration.export_path =
                "Configure an integration module before exporting.".to_string();
            state.snapshot.integration.reference_data_path =
                "The module reference data path appears after validation.".to_string();
            state.snapshot.integration.ready = false;
            state.snapshot.integration.status_text =
                "Integration root is not configured yet".to_string();
            state.snapshot.integration.provider_id = None;
            state.snapshot.integration.provider_name = None;
            state.snapshot.integration.repository_url = None;
            state.snapshot.ui.error_text = None;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            drop(state);
            self.refresh_now();
            return Ok(());
        }

        let status = self.persist_integration_root(module_id, Some(Path::new(candidate)))?;
        let status =
            status.ok_or_else(|| "Integration module did not return status".to_string())?;

        {
            let mut state = self.state.borrow_mut();
            state.integration_repo_root = status.repo_root.clone();
            let repo_root = status
                .repo_root
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "Not configured".to_string());
            state.last_status_note = Some(format!("Integration root resolved to {repo_root}"));
            state.snapshot.integration.repo_path = repo_root;
            state.snapshot.integration.export_path = status.export_path.display().to_string();
            state.snapshot.integration.reference_data_path = status
                .reference_data_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "Not detected".to_string());
            state.snapshot.integration.ready = status.is_available;
            state.snapshot.integration.status_text = status
                .details
                .clone()
                .unwrap_or_else(|| "Integration root configured".to_string());
            state.snapshot.integration.provider_id = status.provider_id;
            state.snapshot.integration.provider_name = status.provider_name;
            state.snapshot.integration.repository_url = status.repository_url;
            state.snapshot.ui.error_text = None;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }
        self.refresh_now();
        Ok(())
    }

    fn refresh_now(&self) {
        let snapshot_result = self.request_snapshot();
        match snapshot_result {
            Ok(shared) => {
                self.apply_shared_snapshot(shared, true);
            }
            Err(message) => {
                self.set_watcher_unavailable(&message);
            }
        }
    }

    #[cfg(test)]
    fn watcher_available(&self) -> bool {
        let state = self.state.borrow();
        state.snapshot.ui.watcher_connected
    }

    fn tool_available(&self) -> bool {
        let state = self.state.borrow();
        state.snapshot.runtime_status.tool_available
    }

    fn start_snapshot_refresh_job(&self) -> Option<SnapshotRefreshJob> {
        let (base_url, integration_repo_root, epoch) = {
            let mut state = self.state.borrow_mut();
            if state.refresh_in_progress {
                return None;
            }
            state.refresh_in_progress = true;
            (
                self.base_url.clone(),
                state.integration_repo_root.clone(),
                state.snapshot_epoch,
            )
        };
        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            let result = request_snapshot_in_background(base_url, integration_repo_root, epoch);
            let _ = sender.send(result);
        });

        Some(SnapshotRefreshJob { receiver, epoch })
    }

    fn start_integration_module_ui_action_job(
        &self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> ModuleUiActionJob {
        let base_url = self.base_url.clone();
        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            let result = send_integration_module_ui_action_in_background(base_url, request);
            let _ = sender.send(result);
        });

        ModuleUiActionJob { receiver }
    }

    fn apply_integration_module_ui_action_response(
        &self,
        response: &IntegrationModuleUiActionResponseDto,
    ) {
        let mut state = self.state.borrow_mut();
        state.last_status_note = response.message.clone();
        state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        if response.refresh || !response.commands.is_empty() {
            state.last_refresh_at = None;
        }
    }

    fn integration_module_ui_action_events(
        &self,
        module_id: &str,
        ui_action_token: &str,
        after: u64,
    ) -> Result<IntegrationModuleUiActionEventsResponseDto, String> {
        let url = format!("{}/v1/integrations/ui-action-events", self.base_url);
        let after_string = after.to_string();
        let response = self
            .client
            .get(url)
            .query(&[
                ("module_id", module_id),
                ("ui_action_token", ui_action_token),
                ("after", after_string.as_str()),
            ])
            .send()
            .map_err(|error| format!("request failed: {error}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let details = response.text().unwrap_or_default();
            return Err(format!(
                "/v1/integrations/ui-action-events failed with {status}: {details}"
            ));
        }

        response
            .json::<IntegrationModuleUiActionEventsResponseDto>()
            .map_err(|error| format!("invalid module UI action events response: {error}"))
    }

    fn apply_snapshot_refresh_result(&self, result: SnapshotRefreshResult) -> bool {
        let is_stale = {
            let mut state = self.state.borrow_mut();
            state.refresh_in_progress = false;
            if let Some(note) = result.status_note {
                state.last_status_note = Some(note);
            }
            result.epoch < state.snapshot_epoch
        };

        if is_stale {
            return false;
        }

        match result.snapshot {
            Ok(shared) => {
                let mut changed = self.apply_shared_snapshot(shared, result.watcher_connected);
                if !result.watcher_connected {
                    if let Some(message) = result.failure_message.as_deref() {
                        let mut state = self.state.borrow_mut();
                        let next_message = message.to_string();
                        if state.snapshot.ui.error_text.as_ref() != Some(&next_message)
                            || state.snapshot.ui.status_text != next_message
                        {
                            state.snapshot.ui.error_text = Some(next_message.clone());
                            state.snapshot.ui.status_text = next_message;
                            changed = true;
                        }
                    }
                }
                changed
            }
            Err(message) => self.set_watcher_unavailable(&message),
        }
    }

    fn apply_shared_snapshot(
        &self,
        shared: SharedSnapshotResponse,
        watcher_connected: bool,
    ) -> bool {
        let (pending_exe_path, integration_repo_root, last_export_text, last_status_note) = {
            let state = self.state.borrow();
            (
                state.pending_exe_path.clone(),
                state.integration_repo_root.clone(),
                state.last_export_text.clone(),
                state
                    .last_status_note
                    .clone()
                    .filter(|note| !watcher_connected || !is_transient_watcher_startup_note(note)),
            )
        };

        let mut mapped = map_snapshot(
            &shared,
            &pending_exe_path,
            integration_repo_root.as_deref(),
            &last_export_text,
            last_status_note.as_deref(),
        );
        mapped.ui.watcher_connected = watcher_connected;

        let mut state = self.state.borrow_mut();
        let next_filters = mapped.filters.clone();
        mapped.pending_exe_path = state.pending_exe_path.clone();
        mapped.filters = next_filters.clone();
        let changed = state.snapshot != mapped || state.filters != next_filters;
        if changed {
            state.filters = next_filters;
            state.snapshot = mapped;
        }
        state.last_refresh_at = Some(Instant::now());
        state.refresh_in_progress = false;
        changed
    }

    fn request_snapshot(&self) -> Result<SharedSnapshotResponse, String> {
        let url = format!("{}/v1/snapshot", self.base_url);
        let integration_repo_root = self.state.borrow().integration_repo_root.clone();
        let mut request = self.client.get(&url);
        if let Some(path) = integration_repo_root {
            let path_string = path.display().to_string();
            request = request.query(&[("integration_dir", &path_string)]);
        }
        let endpoint_probe_targets = endpoint_probe_query_params();
        if !endpoint_probe_targets.is_empty() {
            request = request.query(&endpoint_probe_targets);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                return Err(format!("failed to reach local watcher: {error}"));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let details = response.text().unwrap_or_default();
            return Err(format!(
                "watcher responded with {status} while loading snapshot: {details}"
            ));
        }

        response
            .json::<SharedSnapshotResponse>()
            .map_err(|error| format!("failed to decode snapshot payload: {error}"))
    }

    fn shutdown_embedded_watcher(&self) {
        let _ =
            self.post_json::<serde_json::Value, serde_json::Value>("/v1/watcher/shutdown", None);
    }

    fn set_local_error(&self, message: &str) {
        let mut state = self.state.borrow_mut();
        state.snapshot.ui.error_text = Some(message.to_string());
        state.snapshot.ui.status_text = message.to_string();
        state.snapshot.ui.monitoring = false;
        state.snapshot.integration.last_export_text = state.last_export_text.clone();
        state.snapshot.pending_exe_path = state.pending_exe_path.clone();
        state.snapshot.filters = state.filters.clone();
        state.last_refresh_at = Some(Instant::now());
        state.refresh_in_progress = false;
    }

    fn set_watcher_unavailable(&self, message: &str) -> bool {
        let mut state = self.state.borrow_mut();
        let mut next = state.snapshot.clone();
        next.ui.error_text = Some(message.to_string());
        next.ui.status_text = format!("Local watcher unavailable at {}", self.base_url);
        next.ui.monitoring = false;
        next.ui.watcher_connected = false;
        next.integration.last_export_text = state.last_export_text.clone();
        next.pending_exe_path = state.pending_exe_path.clone();
        next.filters = state.filters.clone();
        let changed = state.snapshot != next;
        if changed {
            state.snapshot = next;
        }
        state.last_refresh_at = Some(Instant::now());
        state.refresh_in_progress = false;
        changed
    }

    fn set_request_error(&self, message: &str) {
        if message.starts_with("request failed:") {
            self.set_watcher_unavailable(message);
        } else {
            self.set_local_error(message);
        }
    }

    fn persist_integration_root(
        &self,
        module_id: Option<String>,
        repo_root: Option<&Path>,
    ) -> Result<Option<netstitch_shared::models::IntegrationStatusDto>, String> {
        let payload = ConfigureIntegrationRootRequest {
            module_id,
            repo_root: repo_root.map(|path| path.display().to_string()),
        };

        self.post_json::<_, Option<netstitch_shared::models::IntegrationStatusDto>>(
            "/v1/integrations/configure",
            Some(&payload),
        )
    }

    fn post_json<Req, Resp>(&self, route: &str, body: Option<&Req>) -> Result<Resp, String>
    where
        Req: Serialize + ?Sized,
        Resp: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, route);
        let builder = self.client.post(url);
        let response = match body {
            Some(payload) => builder.json(payload).send(),
            None => builder.send(),
        }
        .map_err(|error| format!("request failed: {error}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let details = response.text().unwrap_or_default();
            return Err(format!("{route} failed with {status}: {details}"));
        }

        response
            .json::<Resp>()
            .map_err(|error| format!("invalid response payload for {route}: {error}"))
    }

    fn apply_monitor_command_result(&self, response: MonitorCommandResponse) {
        let mut state = self.state.borrow_mut();
        state.snapshot.ui.monitoring = matches!(
            response.status,
            MonitorStatus::Starting | MonitorStatus::Running
        );
        state.last_status_note = response.message;
        state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        state.last_refresh_at = None;
    }

    fn confirm_observations_deferred(&self, request: ConfirmObservationsRequest) {
        if request.observation_ids.is_empty() {
            return;
        }

        let snapshot = self.snapshot();
        let confirmed = request.confirmed.unwrap_or_else(|| {
            if request.observation_ids.len() == 1 {
                snapshot
                    .observations
                    .iter()
                    .find(|item| item.id == request.observation_ids[0])
                    .is_none_or(|item| !(item.is_confirmed && !item.is_exported))
            } else {
                true
            }
        });
        let payload = ConfirmEndpointsRequest {
            endpoint_ids: request.observation_ids,
            confirmed,
        };
        let client = self.client.clone();
        let url = format!("{}{}", self.base_url, "/v1/observations/confirm");
        thread::spawn(move || {
            let _ = client.post(url).json(&payload).send();
        });
    }
}

impl WatcherApiClient for LiveWatcherApi {
    fn snapshot(&self) -> SnapshotResponse {
        self.state.borrow().snapshot.clone()
    }

    fn set_pending_exe_path(&mut self, pending_exe_path: String) {
        let mut state = self.state.borrow_mut();
        state.pending_exe_path = pending_exe_path.clone();
        state.snapshot.pending_exe_path = pending_exe_path;
    }

    fn add_tracked_app(&mut self, request: AddTrackedAppRequest) {
        let exe_path = request.exe_path.trim().to_string();
        if exe_path.is_empty() {
            self.set_local_error("Enter an exe path before adding it.");
            return;
        }

        let payload = SharedAddTrackedAppRequest {
            exe_path: PathBuf::from(exe_path),
            enabled: false,
        };
        if let Err(message) =
            self.post_json::<_, serde_json::Value>("/v1/tracked-apps", Some(&payload))
        {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state.pending_exe_path.clear();
            state.snapshot.pending_exe_path.clear();
            state.snapshot.ui.error_text = None;
            state.snapshot.ui.watcher_connected = true;
            state.last_status_note = Some("Tracked executable added".to_string());
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn set_tracked_app_tag(&mut self, request: SetTrackedAppTagRequest) -> Result<(), String> {
        let payload = SharedSetTrackedAppTagRequest {
            tracked_app_id: request.app_id,
            tag: request.tag.clone(),
        };
        self.post_json::<_, serde_json::Value>("/v1/tracked-apps/tag", Some(&payload))
            .map_err(|message| {
                self.set_request_error(&message);
                message
            })?;
        {
            let mut state = self.state.borrow_mut();
            if let Some(app) = state
                .snapshot
                .tracked_apps
                .iter_mut()
                .find(|app| app.id == request.app_id)
            {
                app.current_tag = request.tag;
            }
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
        Ok(())
    }

    fn toggle_tracked_app(&mut self, request: ToggleTrackedAppRequest) {
        let snapshot = self.snapshot();
        let Some(app) = snapshot
            .tracked_apps
            .iter()
            .find(|app| app.id == request.app_id)
        else {
            self.set_local_error("Tracked app not found for toggle.");
            return;
        };

        let payload = SetTrackedAppEnabledRequest {
            tracked_app_id: app.id,
            enabled: !app.enabled,
        };
        if let Err(message) =
            self.post_json::<_, serde_json::Value>("/v1/tracked-apps/toggle", Some(&payload))
        {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            if let Some(current_app) = state
                .snapshot
                .tracked_apps
                .iter_mut()
                .find(|current| current.id == request.app_id)
            {
                current_app.enabled = !app.enabled;
            }
            state.snapshot.ui.error_text = None;
            state.snapshot.ui.watcher_connected = true;
            state.last_status_note = Some(format!(
                "{} is now {}",
                app.display_name,
                if app.enabled { "disabled" } else { "enabled" }
            ));
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn delete_tracked_app(&mut self, request: DeleteTrackedAppRequest) {
        let snapshot = self.snapshot();
        let Some(app) = snapshot
            .tracked_apps
            .iter()
            .find(|app| app.id == request.app_id)
        else {
            self.set_local_error("Tracked app not found for delete.");
            return;
        };

        let payload = SharedDeleteTrackedAppRequest {
            tracked_app_id: request.app_id,
        };
        if let Err(message) =
            self.post_json::<_, serde_json::Value>("/v1/tracked-apps/delete", Some(&payload))
        {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state
                .snapshot
                .tracked_apps
                .retain(|current| current.id != request.app_id);
            state.snapshot.ui.error_text = None;
            state.snapshot.ui.watcher_connected = true;
            state.last_status_note = Some(format!(
                "{} was removed from tracked apps",
                app.display_name
            ));
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn set_all_tracked_apps_enabled(&mut self, request: SetAllTrackedAppsEnabledRequest) {
        let payload = SetAppSettingRequest {
            key: SETTING_UI_ENABLE_ALL_OVERLAY.to_string(),
            value: request.enabled.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.enable_all_overlay = request.enabled;
            state.snapshot.ui.error_text = None;
            state.snapshot.ui.watcher_connected = true;
            state.last_status_note = Some(if request.enabled {
                "Enable-all overlay is active".to_string()
            } else {
                "Enable-all overlay is inactive".to_string()
            });
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn start_monitoring(&mut self) {
        match self.post_json::<serde_json::Value, MonitorCommandResponse>("/v1/monitor/start", None)
        {
            Ok(response) => {
                self.apply_monitor_command_result(response);
                self.refresh_now();
            }
            Err(message) => self.set_request_error(&message),
        }
    }

    fn stop_monitoring(&mut self) {
        match self.post_json::<serde_json::Value, MonitorCommandResponse>("/v1/monitor/stop", None)
        {
            Ok(response) => {
                self.apply_monitor_command_result(response);
                self.refresh_now();
            }
            Err(message) => self.set_request_error(&message),
        }
    }

    fn confirm_observations(&mut self, request: ConfirmObservationsRequest) {
        if request.observation_ids.is_empty() {
            return;
        }

        let snapshot = self.snapshot();
        let confirmed = request.confirmed.unwrap_or_else(|| {
            if request.observation_ids.len() == 1 {
                snapshot
                    .observations
                    .iter()
                    .find(|item| item.id == request.observation_ids[0])
                    .is_none_or(|item| !(item.is_confirmed && !item.is_exported))
            } else {
                true
            }
        });

        let payload = ConfirmEndpointsRequest {
            endpoint_ids: request.observation_ids,
            confirmed,
        };
        if let Err(message) =
            self.post_json::<_, serde_json::Value>("/v1/observations/confirm", Some(&payload))
        {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state.last_status_note = Some(if confirmed {
                "Selected observations were marked for export".to_string()
            } else {
                "Selected observations were unconfirmed".to_string()
            });
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn mark_observations_exported(
        &mut self,
        request: MarkObservationsExportedRequest,
    ) -> Result<usize, String> {
        if request.observation_ids.is_empty() {
            return Ok(0);
        }
        let ids = request
            .observation_ids
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let payload = MarkEndpointsExportedRequest {
            endpoint_ids: ids.iter().copied().collect(),
        };
        let response = self
            .post_json::<_, serde_json::Value>("/v1/observations/mark-exported", Some(&payload))?;
        let updated = response
            .get("updated")
            .and_then(|value| value.as_u64())
            .unwrap_or(ids.len() as u64) as usize;
        {
            let mut state = self.state.borrow_mut();
            for observation in &mut state.snapshot.observations {
                if ids.contains(&observation.id) {
                    observation.is_exported = true;
                }
            }
            state.last_status_note = Some(format!("Cloud uploaded observations: {updated}"));
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
        Ok(updated)
    }

    fn delete_observation(&mut self, request: DeleteObservationRequest) -> Result<usize, String> {
        self.delete_observations(DeleteObservationsRequest {
            observation_ids: vec![request.observation_id],
        })
    }

    fn delete_observations(&mut self, request: DeleteObservationsRequest) -> Result<usize, String> {
        if request.observation_ids.is_empty() {
            return Ok(0);
        }

        let ids = request
            .observation_ids
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let payload = SharedDeleteObservationsRequest {
            endpoint_ids: ids.iter().copied().collect(),
        };
        let response = match self.post_json::<_, DeleteObservationsResponse>(
            "/v1/observations/delete-batch",
            Some(&payload),
        ) {
            Ok(response) => response,
            Err(message) => {
                self.set_request_error(&message);
                return Err(message);
            }
        };

        {
            let mut state = self.state.borrow_mut();
            state
                .snapshot
                .observations
                .retain(|observation| !ids.contains(&observation.id));
            state.snapshot.ui.error_text = None;
            state.snapshot.ui.watcher_connected = true;
            state.last_status_note = Some(format!("Observations deleted: {}", response.deleted));
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
        Ok(response.deleted)
    }

    fn clear_observation_tags(
        &mut self,
        request: ClearObservationTagsRequest,
    ) -> Result<usize, String> {
        if request.observation_ids.is_empty() {
            return Ok(0);
        }
        let tag = request.tag.clone();
        let ids = request
            .observation_ids
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let payload = SharedClearObservationTagsRequest {
            endpoint_ids: ids.iter().copied().collect(),
            tag: tag.clone(),
        };
        let response = self
            .post_json::<_, ClearObservationTagsResponse>(
                "/v1/observations/clear-tags",
                Some(&payload),
            )
            .map_err(|message| {
                self.set_request_error(&message);
                message
            })?;
        {
            let mut state = self.state.borrow_mut();
            for observation in &mut state.snapshot.observations {
                if ids.contains(&observation.id) {
                    if let Some(tag) = tag.as_deref() {
                        observation.tags.retain(|value| value != tag);
                    } else {
                        observation.tags.clear();
                    }
                }
            }
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
        Ok(response.cleared)
    }

    fn delete_local_tag(&mut self, request: DeleteLocalTagRequest) -> Result<usize, String> {
        let tag = netstitch_shared::normalize_cloud_tag(&request.tag)
            .ok_or_else(|| "invalid_tag".to_string())?;
        let payload = SharedDeleteLocalTagRequest { tag: tag.clone() };
        let response = self
            .post_json::<_, SharedDeleteLocalTagResponse>("/v1/tags/delete-local", Some(&payload))
            .map_err(|message| {
                self.set_request_error(&message);
                message
            })?;
        {
            let mut state = self.state.borrow_mut();
            for app in &mut state.snapshot.tracked_apps {
                if app.current_tag.as_deref() == Some(tag.as_str()) {
                    app.current_tag = None;
                }
            }
            for observation in &mut state.snapshot.observations {
                observation.tags.retain(|value| value != &tag);
            }
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
        Ok(response.cleared_tracked_apps + response.removed_observation_links)
    }

    fn import_monitoring_csv(
        &mut self,
        request: MonitoringCsvImportRequestDto,
    ) -> Result<MonitoringCsvImportResultDto, String> {
        let import_source = request.import_source.clone();
        match self.post_json::<_, MonitoringCsvImportResultDto>(
            "/v1/observations/import-csv",
            Some(&request),
        ) {
            Ok(result) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.last_status_note =
                        Some(import_monitoring_status_note(import_source, &result));
                    state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                    state.last_refresh_at = None;
                }
                self.refresh_now();
                Ok(result)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn preview_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        self.post_json::<_, ExportProfilePlanDto>("/v1/export/profile/preview", Some(&payload))
    }

    fn analyze_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        self.post_json::<_, ExportProfilePlanDto>("/v1/export/profile/analyze", Some(&payload))
    }

    fn analyze_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        self.post_json::<_, ExportProfilePlanDto>(
            "/v1/export/profile/advanced-settings/analyze",
            Some(&payload),
        )
    }

    fn apply_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        match self.post_json::<_, ExportProfilePlanDto>("/v1/export/profile/apply", Some(&payload))
        {
            Ok(plan) => {
                let mut state = self.state.borrow_mut();
                state.last_export_text = export_profile_result_text(&plan);
                state.last_status_note = Some(state.last_export_text.clone());
                state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                state.last_refresh_at = None;
                drop(state);
                self.refresh_now();
                Ok(plan)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn apply_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        self.post_json::<_, ExportProfilePlanDto>(
            "/v1/export/profile/advanced-settings/apply",
            Some(&payload),
        )
    }

    fn backup_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        match self.post_json::<_, ExportProfilePlanDto>("/v1/export/profile/backup", Some(&payload))
        {
            Ok(plan) => {
                let mut state = self.state.borrow_mut();
                state.last_export_text = export_profile_result_text(&plan);
                state.last_status_note = Some(state.last_export_text.clone());
                state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                Ok(plan)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn revert_profile_export(
        &mut self,
        module_id: Option<String>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto, String> {
        let payload = ScopedProfileExportRequest { module_id, request };
        match self.post_json::<_, ExportProfilePlanDto>("/v1/export/profile/revert", Some(&payload))
        {
            Ok(plan) => {
                let mut state = self.state.borrow_mut();
                state.last_export_text = export_profile_result_text(&plan);
                state.last_status_note = Some(state.last_export_text.clone());
                state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                state.last_refresh_at = None;
                drop(state);
                self.refresh_now();
                Ok(plan)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn set_profile_export_ui_state(&mut self, state: ProfileExportUiStateDto) {
        {
            let mut current = self.state.borrow_mut();
            current.snapshot.app_settings.profile_export_ui_state = Some(state.clone());
            current.snapshot_epoch = current.snapshot_epoch.wrapping_add(1);
        }

        if let Err(message) = self.post_json::<_, serde_json::Value>(
            "/v1/integrations/profile-export-ui-state",
            Some(&state),
        ) {
            self.set_request_error(&message);
        }
    }

    fn run_integration_module_ui_action(
        &mut self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        match self.post_json::<_, IntegrationModuleUiActionResponseDto>(
            "/v1/integrations/ui-action",
            Some(&request),
        ) {
            Ok(response) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.last_status_note = response.message.clone();
                    state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                    if response.refresh || !response.commands.is_empty() {
                        state.last_refresh_at = None;
                    }
                }
                if response.refresh || !response.commands.is_empty() {
                    self.refresh_now();
                }
                Ok(response)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn stop_integration_module_background(&mut self, module_id: String) -> Result<bool, String> {
        let payload = StopIntegrationModuleBackgroundRequest {
            module_id: module_id.clone(),
        };
        match self
            .post_json::<_, serde_json::Value>("/v1/integrations/background/stop", Some(&payload))
        {
            Ok(value) => {
                let stopped = value
                    .get("stopped")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                {
                    let mut state = self.state.borrow_mut();
                    for module in &mut state.snapshot.integration_modules {
                        if module.id == module_id {
                            module.background_active = false;
                            module.background_subscriptions.clear();
                            module.background_status = Some("stopped".to_string());
                        }
                    }
                    state.last_status_note = Some(if stopped {
                        "Module listener stopped".to_string()
                    } else {
                        "Module listener was already stopped".to_string()
                    });
                    state.last_refresh_at = None;
                    state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                }
                self.refresh_now();
                Ok(stopped)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn send_integration_module_dialog_result(
        &mut self,
        module_id: String,
        dialog_id: String,
        result: String,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        let payload = serde_json::json!({
            "module_id": module_id,
            "dialog_id": dialog_id,
            "result": result,
        });
        match self.post_json::<_, IntegrationModuleUiActionResponseDto>(
            "/v1/integrations/dialog-result",
            Some(&payload),
        ) {
            Ok(response) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.last_status_note = response.message.clone();
                    state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                    if response.refresh || !response.commands.is_empty() {
                        state.last_refresh_at = None;
                    }
                }
                if response.refresh || !response.commands.is_empty() {
                    self.refresh_now();
                }
                Ok(response)
            }
            Err(message) => {
                self.set_request_error(&message);
                Err(message)
            }
        }
    }

    fn set_filters(&mut self, request: SetFilterRequest) {
        {
            let mut state = self.state.borrow_mut();
            state.filters = FiltersDto {
                app_search: request.app_search.clone(),
                search_text: request.search_text.clone(),
                domain_search: request.domain_search.clone(),
                port_search: request.port_search.clone(),
                protocol: request.protocol.clone(),
                public_ip: request.public_ip,
                observation_filter: request.observation_filter.clone(),
            };
            state.snapshot.filters = state.filters.clone();
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SharedUiFiltersDto {
            app_search: request.app_search,
            ip_search: request.search_text,
            domain_search: request.domain_search,
            port_search: request.port_search,
            protocol: request.protocol,
            public_ip: request.public_ip,
            observation_filter: match request.observation_filter {
                ObservationFilterDto::All => SharedUiObservationFilterDto::All,
                ObservationFilterDto::Unconfirmed => SharedUiObservationFilterDto::Unconfirmed,
                ObservationFilterDto::Confirmed => SharedUiObservationFilterDto::Confirmed,
                ObservationFilterDto::Success => SharedUiObservationFilterDto::Success,
                ObservationFilterDto::Exported => SharedUiObservationFilterDto::Exported,
                ObservationFilterDto::Failed => SharedUiObservationFilterDto::Failed,
            },
        };
        if let Err(message) = self.post_json::<_, SharedUiFiltersDto>("/v1/filters", Some(&payload))
        {
            self.set_request_error(&message);
        }
    }

    fn set_language_code(&mut self, language_code: String) {
        let language_code = language_code.trim().to_ascii_lowercase();
        if language_code.is_empty() {
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.language_code = Some(language_code.clone());
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_UI_LANGUAGE.to_string(),
            value: language_code,
        };
        match self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            Ok(_) => self.refresh_now(),
            Err(message) => {
                let _ = persist_app_setting_to_sqlite(&payload.key, &payload.value);
                self.set_request_error(&message);
            }
        }
    }

    fn set_module_order(&mut self, order: Vec<String>) {
        let order = normalized_module_order(order);
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.module_order = order.clone();
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_UI_MODULE_ORDER.to_string(),
            value: serde_json::to_string(&order).unwrap_or_else(|_| "[]".to_string()),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            let _ = persist_app_setting_to_sqlite(&payload.key, &payload.value);
            self.set_request_error(&message);
        }
    }

    fn set_monitoring_show_tags(&mut self, visible: bool) {
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.monitoring_show_tags = visible;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }
        self.persist_monitoring_visibility_setting(SETTING_UI_MONITORING_SHOW_TAGS, visible);
    }

    fn set_monitoring_show_connection_count(&mut self, visible: bool) {
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.monitoring_show_connection_count = visible;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }
        self.persist_monitoring_visibility_setting(
            SETTING_UI_MONITORING_SHOW_CONNECTION_COUNT,
            visible,
        );
    }

    fn set_web_access_localhost(&mut self, enabled: bool) {
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.web_access_localhost = enabled;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_WEB_ACCESS_LOCALHOST.to_string(),
            value: enabled.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            self.set_request_error(&message);
        }
    }

    fn set_domain_capture_enabled(&mut self, enabled: bool) {
        if enabled {
            let snapshot = self.state.borrow().snapshot.clone();
            if snapshot.ui.snapshot_loaded && !snapshot.runtime_status.is_elevated {
                self.set_local_error("Run NetStitch as administrator to use advanced monitoring.");
                return;
            }
        }
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.domain_capture_enabled = enabled;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_DOMAIN_CAPTURE_ENABLED.to_string(),
            value: enabled.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            {
                let mut state = self.state.borrow_mut();
                state.snapshot.app_settings.domain_capture_enabled = !enabled;
                state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            }
            self.set_request_error(&message);
        }
    }

    fn set_remember_window_placement(&mut self, enabled: bool) {
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.remember_window_placement = enabled;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_UI_REMEMBER_WINDOW_PLACEMENT.to_string(),
            value: enabled.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            let _ = persist_app_setting_to_sqlite(&payload.key, &payload.value);
            self.set_request_error(&message);
        }
    }

    fn set_hide_when_minimized(&mut self, enabled: bool) {
        {
            let mut state = self.state.borrow_mut();
            state.snapshot.app_settings.hide_when_minimized = enabled;
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
        }

        let payload = SetAppSettingRequest {
            key: SETTING_UI_HIDE_WHEN_MINIMIZED.to_string(),
            value: enabled.to_string(),
        };
        if let Err(message) = self.post_json::<_, netstitch_shared::models::AppSettingsDto>(
            "/v1/settings",
            Some(&payload),
        ) {
            let _ = persist_app_setting_to_sqlite(&payload.key, &payload.value);
            self.set_request_error(&message);
        }
    }

    fn ignore_address(&mut self, request: IgnoreAddressRequest) {
        let address_pattern = request.address_pattern.trim().to_string();
        if address_pattern.is_empty() {
            return;
        }

        let payload = SharedAddIgnoredAddressRequest { address_pattern };
        match self.post_json::<_, netstitch_shared::models::IgnoredAddressRule>(
            "/v1/ignored-addresses",
            Some(&payload),
        ) {
            Ok(rule) => {
                let mut state = self.state.borrow_mut();
                let mapped = map_ignored_address_rule(&rule);
                if !state
                    .snapshot
                    .ignored_addresses
                    .iter()
                    .any(|existing| existing.address_pattern == mapped.address_pattern)
                {
                    state.snapshot.ignored_addresses.push(mapped);
                }
                state.last_status_note = Some("Address was added to the ignore list".to_string());
                state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                state.last_refresh_at = None;
                drop(state);
                self.refresh_now();
            }
            Err(message) => self.set_request_error(&message),
        }
    }

    fn delete_ignored_address(&mut self, request: DeleteIgnoredAddressRequest) {
        let payload = SharedDeleteIgnoredAddressRequest {
            ignored_address_id: request.ignored_address_id,
        };
        if let Err(message) =
            self.post_json::<_, serde_json::Value>("/v1/ignored-addresses/delete", Some(&payload))
        {
            self.set_request_error(&message);
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            state
                .snapshot
                .ignored_addresses
                .retain(|rule| rule.id != request.ignored_address_id);
            state.last_status_note = Some("Address was removed from the ignore list".to_string());
            state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
            state.last_refresh_at = None;
        }
        self.refresh_now();
    }

    fn download_integration(&mut self, request: DownloadIntegrationRequest) {
        let payload = SharedDownloadIntegrationProviderRequest {
            module_id: request.module_id,
            provider_id: request.provider_id,
        };
        match self.post_json::<_, netstitch_shared::models::IntegrationStatusDto>(
            "/v1/integrations/download",
            Some(&payload),
        ) {
            Ok(status) => {
                let repo_root = status
                    .repo_root
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "Not configured".to_string());
                {
                    let mut state = self.state.borrow_mut();
                    state.integration_repo_root = status.repo_root.clone();
                    state.last_status_note =
                        Some(format!("Integration provider downloaded: {repo_root}"));
                    state.snapshot.integration.repo_path = repo_root;
                    state.snapshot.integration.export_path =
                        status.export_path.display().to_string();
                    state.snapshot.integration.reference_data_path = status
                        .reference_data_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "Reference data path not detected".to_string());
                    state.snapshot.integration.ready = status.is_available;
                    state.snapshot.integration.provider_id = status.provider_id;
                    state.snapshot.integration.provider_name = status.provider_name;
                    state.snapshot.integration.repository_url = status.repository_url;
                    state.snapshot_epoch = state.snapshot_epoch.wrapping_add(1);
                    state.last_refresh_at = None;
                }
                self.refresh_now();
            }
            Err(message) => self.set_request_error(&message),
        }
    }
}

fn should_use_mock() -> bool {
    env::var(APP_ENV_UI_USE_MOCK)
        .map(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "1" || value == "true" || value == "yes" || value == "on"
        })
        .unwrap_or(false)
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for path in paths {
        if !deduped.contains(&path) {
            deduped.push(path);
        }
    }
    deduped
}

fn default_watcher_addr() -> String {
    format!("{DEFAULT_WATCHER_HOST}:{DEFAULT_WATCHER_PORT}")
}

fn default_web_scheme() -> &'static str {
    match env::var(APP_ENV_WEB_SCHEME) {
        Ok(value) if value.trim().eq_ignore_ascii_case("http") => "http",
        _ => "https",
    }
}

pub(crate) fn local_machine_ip_for_watcher() -> Option<String> {
    for target in load_endpoint_probe_targets() {
        let target = target
            .strip_prefix("tcp://")
            .or_else(|| target.strip_prefix("udp://"))
            .unwrap_or(target.as_str());
        let Ok(remote) = target.parse::<SocketAddr>() else {
            continue;
        };
        let bind_addr = if remote.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let Ok(socket) = UdpSocket::bind(bind_addr) else {
            continue;
        };
        if socket.connect(remote).is_err() {
            continue;
        }
        let Ok(local_addr) = socket.local_addr() else {
            continue;
        };
        let ip = local_addr.ip();
        if ip.is_loopback() || ip.is_unspecified() {
            continue;
        }
        return Some(ip.to_string());
    }
    None
}

fn endpoint_probe_query_params() -> Vec<(String, String)> {
    load_endpoint_probe_targets()
        .into_iter()
        .flat_map(|target| {
            let (target, protocol) = endpoint_probe_request_spec(&target, None);
            if target.is_empty() {
                return Vec::new();
            }
            [
                ("endpoint_probe_target".to_string(), target),
                (
                    "endpoint_probe_protocol".to_string(),
                    protocol.as_str().to_string(),
                ),
            ]
            .to_vec()
        })
        .collect()
}

fn load_endpoint_probe_targets() -> Vec<String> {
    if let Ok(raw_targets) = env::var(APP_ENV_ENDPOINT_PROBE_TARGETS) {
        let targets = parse_endpoint_probe_targets(&raw_targets);
        if !targets.is_empty() {
            return targets;
        }
    }

    for path in candidate_endpoint_probe_target_files() {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let targets = parse_endpoint_probe_targets(&content);
        if !targets.is_empty() {
            return targets;
        }
    }

    Vec::new()
}

fn candidate_endpoint_probe_target_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(explicit) = env::var(APP_ENV_ENDPOINT_PROBE_TARGETS_FILE) {
        files.push(PathBuf::from(explicit));
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            files.push(parent.join("config").join(ENDPOINT_PROBE_TARGETS_FILE));
        }
    }
    if let Ok(current_dir) = env::current_dir() {
        files.push(current_dir.join("config").join(ENDPOINT_PROBE_TARGETS_FILE));
    }
    dedupe_paths(files)
}

fn parse_endpoint_probe_targets(content: &str) -> Vec<String> {
    content
        .lines()
        .flat_map(|line| line.split([',', ';']))
        .map(|item| item.split('#').next().unwrap_or("").trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn endpoint_probe_request_spec(
    target: &str,
    protocol_override: Option<&str>,
) -> (String, SharedProtocol) {
    let trimmed = target.trim();
    let (prefixed_protocol, normalized_target) = if let Some(value) = trimmed.strip_prefix("tcp://")
    {
        (SharedProtocol::Tcp, value)
    } else if let Some(value) = trimmed.strip_prefix("udp://") {
        (SharedProtocol::Udp, value)
    } else {
        (default_probe_protocol(trimmed), trimmed)
    };

    let protocol = protocol_override
        .map(SharedProtocol::from_name)
        .filter(|value| *value != SharedProtocol::Other)
        .unwrap_or(prefixed_protocol);

    let normalized_target = normalized_target.trim();
    if normalized_target.is_empty() {
        return (String::new(), protocol);
    }

    (normalized_target.to_string(), protocol)
}

fn default_probe_protocol(target: &str) -> SharedProtocol {
    match target
        .rsplit(':')
        .next()
        .and_then(|port| port.parse::<u16>().ok())
    {
        Some(53) => SharedProtocol::Udp,
        _ => SharedProtocol::Tcp,
    }
}

fn request_snapshot_in_background(
    base_url: String,
    integration_repo_root: Option<PathBuf>,
    epoch: u64,
) -> SnapshotRefreshResult {
    let client = match Client::builder()
        .timeout(Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .default_headers(desktop_client_headers())
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return SnapshotRefreshResult {
                snapshot: Err(format!(
                    "failed to construct background HTTP client: {error}"
                )),
                watcher_connected: false,
                status_note: None,
                failure_message: None,
                epoch,
            };
        }
    };

    match send_snapshot_request(&client, &base_url, integration_repo_root.as_deref()) {
        Ok(snapshot) => SnapshotRefreshResult {
            snapshot: Ok(snapshot),
            watcher_connected: true,
            status_note: None,
            failure_message: None,
            epoch,
        },
        Err(error) if error.starts_with("connect:") => {
            let connect_error = error.replacen("connect:", "failed to reach local watcher:", 1);
            let local_snapshot =
                local_core_snapshot(integration_repo_root.as_deref()).map_err(|local_error| {
                    format!("{connect_error}; local snapshot fallback failed: {local_error}",)
                });
            SnapshotRefreshResult {
                snapshot: local_snapshot,
                watcher_connected: false,
                status_note: Some(connect_error.clone()),
                failure_message: Some(connect_error),
                epoch,
            }
        }
        Err(error) => SnapshotRefreshResult {
            snapshot: Err(error.replacen("connect:", "failed to reach local watcher:", 1)),
            watcher_connected: false,
            status_note: None,
            failure_message: None,
            epoch,
        },
    }
}

fn send_integration_module_ui_action_in_background(
    base_url: String,
    request: IntegrationModuleUiActionClientRequestDto,
) -> Result<IntegrationModuleUiActionResponseDto, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .danger_accept_invalid_certs(true)
        .default_headers(desktop_client_headers())
        .build()
        .map_err(|error| format!("failed to construct module action HTTP client: {error}"))?;

    post_json_with_client(
        &client,
        &base_url,
        "/v1/integrations/ui-action",
        Some(&request),
    )
}

fn send_network_diagnostic_in_background(
    base_url: String,
    request: NetworkDiagnosticRequestDto,
    sender: &mpsc::Sender<NetworkDiagnosticJobEvent>,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(125))
        .danger_accept_invalid_certs(true)
        .default_headers(desktop_client_headers())
        .build()
        .map_err(|error| format!("failed to construct network diagnostic HTTP client: {error}"))?;

    let start = post_json_with_client::<_, NetworkDiagnosticStartDto>(
        &client,
        &base_url,
        "/v1/network-diagnostics",
        Some(&request),
    )?;
    let route = format!("/v1/network-diagnostics/{}", start.job_id);
    let started_at = Instant::now();
    let mut last_output = String::new();
    loop {
        if started_at.elapsed() >= Duration::from_secs(125) {
            return Err("network diagnostic progress timed out".to_string());
        }
        let progress =
            get_json_with_client::<NetworkDiagnosticProgressDto>(&client, &base_url, &route)?;
        if progress.output != last_output {
            last_output = progress.output.clone();
            if sender
                .send(NetworkDiagnosticJobEvent::Progress(progress.output))
                .is_err()
            {
                return Ok(());
            }
        }
        if !progress.running {
            if let Some(result) = progress.result {
                let _ = sender.send(NetworkDiagnosticJobEvent::Finished(result));
                return Ok(());
            }
            return Err(progress
                .error
                .unwrap_or_else(|| "network diagnostic finished without a result".to_string()));
        }
        std::thread::sleep(Duration::from_millis(75));
    }
}

fn send_snapshot_request(
    client: &Client,
    base_url: &str,
    integration_repo_root: Option<&Path>,
) -> Result<SharedSnapshotResponse, String> {
    let url = format!("{base_url}/v1/snapshot");
    let mut request = client.get(&url);
    if let Some(path) = integration_repo_root {
        let path_string = path.display().to_string();
        request = request.query(&[("integration_dir", &path_string)]);
    }
    let endpoint_probe_targets = endpoint_probe_query_params();
    if !endpoint_probe_targets.is_empty() {
        request = request.query(&endpoint_probe_targets);
    }

    let response = request.send().map_err(|error| {
        if watcher_transport_error(&error) {
            format!("connect:{error}")
        } else {
            format!("failed to reach local watcher: {error}")
        }
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let details = response.text().unwrap_or_default();
        return Err(format!(
            "watcher responded with {status} while loading snapshot: {details}"
        ));
    }

    response
        .json::<SharedSnapshotResponse>()
        .map_err(|error| format!("failed to decode snapshot payload: {error}"))
}

fn post_json_with_client<Req, Resp>(
    client: &Client,
    base_url: &str,
    route: &str,
    body: Option<&Req>,
) -> Result<Resp, String>
where
    Req: Serialize + ?Sized,
    Resp: DeserializeOwned,
{
    let url = format!("{base_url}{route}");
    let builder = client.post(url);
    let response = match body {
        Some(payload) => builder.json(payload).send(),
        None => builder.send(),
    }
    .map_err(|error| format!("request failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let details = response.text().unwrap_or_default();
        return Err(format!("{route} failed with {status}: {details}"));
    }

    response
        .json::<Resp>()
        .map_err(|error| format!("invalid response payload for {route}: {error}"))
}

fn get_json_with_client<Resp>(client: &Client, base_url: &str, route: &str) -> Result<Resp, String>
where
    Resp: DeserializeOwned,
{
    let response = client
        .get(format!("{base_url}{route}"))
        .send()
        .map_err(|error| format!("request failed: {error}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let details = response.text().unwrap_or_default();
        return Err(format!("{route} failed with {status}: {details}"));
    }
    response
        .json::<Resp>()
        .map_err(|error| format!("invalid response payload for {route}: {error}"))
}

fn local_core_snapshot(
    integration_repo_root: Option<&Path>,
) -> Result<SharedSnapshotResponse, String> {
    let core = NetstitchCore::bootstrap()
        .map_err(|error| format!("local core bootstrap failed: {error}"))?;
    core.snapshot(MonitorStatus::Stopped, integration_repo_root)
        .map_err(|error| format!("local snapshot failed: {error}"))
}

fn watcher_transport_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout()
}

fn initial_snapshot(base_url: &str, app_settings: AppSettingsDto) -> SnapshotResponse {
    SnapshotResponse {
        tracked_apps: Vec::new(),
        observations: Vec::new(),
        ignored_addresses: Vec::new(),
        integration: IntegrationIntegrationDto {
            repo_path: "Not configured".to_string(),
            export_path: "Configure an integration module before exporting.".to_string(),
            reference_data_path: "The module reference data path appears after validation."
                .to_string(),
            profile_paths: Vec::new(),
            ready: false,
            status_text: "Integration folder is not configured yet".to_string(),
            last_export_text: "No export yet".to_string(),
            provider_id: None,
            provider_name: None,
            repository_url: None,
        },
        integration_modules: Vec::new(),
        integration_providers: Vec::new(),
        ui: UiStatusDto {
            monitoring: false,
            watcher_connected: false,
            snapshot_loaded: false,
            status_text: format!("Connecting to local watcher at {base_url}"),
            error_text: None,
        },
        app_settings,
        runtime_status: RuntimeStatusDto {
            tool_available: false,
            ..RuntimeStatusDto::default()
        },
        filters: FiltersDto {
            app_search: String::new(),
            search_text: String::new(),
            domain_search: String::new(),
            port_search: String::new(),
            protocol: "All".to_string(),
            public_ip: true,
            observation_filter: ObservationFilterDto::All,
        },
        pending_exe_path: String::new(),
    }
}

fn map_snapshot(
    shared: &SharedSnapshotResponse,
    pending_exe_path: &str,
    integration_repo_root: Option<&Path>,
    last_export_text: &str,
    status_note: Option<&str>,
) -> SnapshotResponse {
    let tracked_apps: Vec<TrackedAppDto> = shared
        .tracked_apps
        .iter()
        .map(|app| TrackedAppDto {
            id: app.id.unwrap_or_default(),
            connector_id: app.connector_id.clone(),
            cloud_app_id: app.cloud_app_id.clone().or_else(|| {
                app.connector_id
                    .as_deref()
                    .and_then(netstitch_shared::cloud_app_for_connector_id)
                    .map(|item| item.app_id.to_string())
            }),
            display_name: display_name(app),
            icon_key: app.icon_key.clone().unwrap_or_else(|| "manual".to_string()),
            icon_path: app
                .icon_path
                .as_ref()
                .map(|path| path.display().to_string()),
            current_tag: app.current_tag.clone(),
            exe_path: app.exe_path.display().to_string(),
            enabled: app.enabled,
            created_at: format_timestamp(app.created_at_ms),
        })
        .collect();

    let observations: Vec<ObservationDto> = shared
        .observed_endpoints
        .iter()
        .map(map_observation)
        .collect();
    let ignored_addresses = shared
        .ignored_addresses
        .iter()
        .map(map_ignored_address_rule)
        .collect();

    let integration = map_integration(shared, integration_repo_root, last_export_text);
    let ui = UiStatusDto {
        monitoring: matches!(
            shared.monitor_status,
            MonitorStatus::Running | MonitorStatus::Starting
        ),
        watcher_connected: true,
        snapshot_loaded: true,
        status_text: status_note
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| monitor_status_text(shared.monitor_status.clone())),
        error_text: monitor_error_text(shared.monitor_status.clone()),
    };

    SnapshotResponse {
        tracked_apps,
        observations,
        ignored_addresses,
        integration,
        integration_modules: shared.integration_modules.clone(),
        integration_providers: shared.integration_providers.clone(),
        ui,
        app_settings: AppSettingsDto {
            language_code: shared.app_settings.ui_language_code.clone(),
            enable_all_overlay: shared.app_settings.ui_enable_all_overlay,
            remember_window_placement: shared.app_settings.ui_remember_window_placement,
            hide_when_minimized: shared.app_settings.ui_hide_when_minimized,
            module_order: shared.app_settings.ui_module_order.clone(),
            monitoring_show_tags: shared.app_settings.ui_monitoring_show_tags,
            monitoring_show_connection_count: shared
                .app_settings
                .ui_monitoring_show_connection_count,
            web_access_localhost: shared.app_settings.web_access_localhost,
            domain_capture_enabled: shared.app_settings.domain_capture_enabled,
            update_check_interval_minutes: shared.app_settings.update_check_interval_minutes,
            profile_export_ui_state: shared
                .integration_status
                .as_ref()
                .and_then(|status| status.profile_export_ui_state.clone()),
        },
        runtime_status: RuntimeStatusDto {
            connector_apps_detected: shared.runtime_status.connector_apps_detected,
            unavailable_tracked_apps: shared
                .runtime_status
                .unavailable_tracked_apps
                .iter()
                .map(|app| TrackedAppAvailabilityDto {
                    tracked_app_id: app.tracked_app_id.unwrap_or_default(),
                    display_name: app
                        .display_name
                        .clone()
                        .unwrap_or_else(|| app.exe_path.display().to_string()),
                    exe_path: app.exe_path.display().to_string(),
                    connector_managed: app.connector_managed,
                })
                .collect(),
            integration_modules: shared
                .runtime_status
                .integration_modules
                .iter()
                .map(|module| IntegrationModuleRuntimeStatusDto {
                    id: module.id.clone(),
                    display_name: module.display_name.clone(),
                    manifest_path: module.manifest_path.display().to_string(),
                    connected: module.connected,
                    enabled: module.enabled,
                    error: module.error.clone(),
                    background_active: module.background_active,
                    background_subscriptions: module.background_subscriptions.clone(),
                    background_status: module.background_status.clone(),
                })
                .collect(),
            endpoint_probe: crate::watcher_api::EndpointProbeStatusDto {
                is_checking: shared.runtime_status.endpoint_probe.is_checking,
                first_successful_target: shared
                    .runtime_status
                    .endpoint_probe
                    .first_successful_target
                    .clone(),
                probes: shared
                    .runtime_status
                    .endpoint_probe
                    .probes
                    .iter()
                    .map(|probe| crate::watcher_api::EndpointProbeTargetDto {
                        target: probe.target.clone(),
                        available: probe.available,
                        error: probe.error.clone(),
                    })
                    .collect(),
            },
            tool_available: shared.runtime_status.tool_available,
            is_elevated: shared.runtime_status.is_elevated,
            domain_capture_admin_disabled: shared.runtime_status.domain_capture_admin_disabled,
            domain_capture: crate::watcher_api::DomainCaptureStatusDto {
                backend_started: shared.runtime_status.domain_capture.backend_started,
                backend_error: shared.runtime_status.domain_capture.backend_error.clone(),
                packets_seen: shared.runtime_status.domain_capture.packets_seen,
                tcp_payload_packets: shared.runtime_status.domain_capture.tcp_payload_packets,
                domains_detected: shared.runtime_status.domain_capture.domains_detected,
                domains_matched: shared.runtime_status.domain_capture.domains_matched,
                pending_packets: shared.runtime_status.domain_capture.pending_packets,
                dropped_no_owner: shared.runtime_status.domain_capture.dropped_no_owner,
                dropped_no_process: shared.runtime_status.domain_capture.dropped_no_process,
                dropped_untracked_process: shared
                    .runtime_status
                    .domain_capture
                    .dropped_untracked_process,
            },
            flow_capture: crate::watcher_api::FlowCaptureStatusDto {
                flow_backend_started: shared.runtime_status.flow_capture.flow_backend_started,
                packet_backend_started: shared.runtime_status.flow_capture.packet_backend_started,
                backend_error: shared.runtime_status.flow_capture.backend_error.clone(),
                flow_events: shared.runtime_status.flow_capture.flow_events,
                udp_flow_events: shared.runtime_status.flow_capture.udp_flow_events,
                packet_events: shared.runtime_status.flow_capture.packet_events,
                udp_packet_events: shared.runtime_status.flow_capture.udp_packet_events,
                observations_emitted: shared.runtime_status.flow_capture.observations_emitted,
                matched_tracked_app: shared.runtime_status.flow_capture.matched_tracked_app,
                dropped_no_owner: shared.runtime_status.flow_capture.dropped_no_owner,
                dropped_no_process: shared.runtime_status.flow_capture.dropped_no_process,
                dropped_untracked_process: shared
                    .runtime_status
                    .flow_capture
                    .dropped_untracked_process,
            },
        },
        filters: map_filters(&shared.filters),
        pending_exe_path: pending_exe_path.to_string(),
    }
}

fn map_filters(shared: &SharedUiFiltersDto) -> FiltersDto {
    FiltersDto {
        app_search: shared.app_search.clone(),
        search_text: shared.ip_search.clone(),
        domain_search: shared.domain_search.clone(),
        port_search: shared.port_search.clone(),
        protocol: if shared.protocol.trim().is_empty() {
            "All".to_string()
        } else {
            shared.protocol.clone()
        },
        public_ip: shared.public_ip,
        observation_filter: match shared.observation_filter {
            SharedUiObservationFilterDto::All => ObservationFilterDto::All,
            SharedUiObservationFilterDto::Unconfirmed => ObservationFilterDto::Unconfirmed,
            SharedUiObservationFilterDto::Confirmed => ObservationFilterDto::Confirmed,
            SharedUiObservationFilterDto::Success => ObservationFilterDto::Success,
            SharedUiObservationFilterDto::Exported => ObservationFilterDto::Exported,
            SharedUiObservationFilterDto::Failed => ObservationFilterDto::Failed,
        },
    }
}

fn is_transient_watcher_startup_note(note: &str) -> bool {
    let trimmed = note.trim();
    trimmed.eq_ignore_ascii_case("Connecting to embedded monitoring runtime...")
        || trimmed.eq_ignore_ascii_case("Connecting to local monitoring watcher...")
}

fn map_ignored_address_rule(
    rule: &netstitch_shared::models::IgnoredAddressRule,
) -> IgnoredAddressDto {
    IgnoredAddressDto {
        id: rule.id.unwrap_or_default(),
        address_pattern: rule.address_pattern.clone(),
        created_at: format_timestamp(rule.created_at_ms),
        enrichment: rule.enrichment.as_ref().map(map_ip_enrichment),
    }
}

fn cold_start_settings() -> AppSettingsDto {
    let mut settings = AppSettingsDto {
        language_code: None,
        enable_all_overlay: false,
        remember_window_placement: false,
        hide_when_minimized: true,
        module_order: Vec::new(),
        monitoring_show_tags: true,
        monitoring_show_connection_count: true,
        web_access_localhost: false,
        domain_capture_enabled: false,
        update_check_interval_minutes: default_update_check_interval_minutes(),
        profile_export_ui_state: None,
    };

    if let Some(language) = read_app_setting_from_sqlite(SETTING_UI_LANGUAGE) {
        settings.language_code = Some(language);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_ENABLE_ALL_OVERLAY) {
        settings.enable_all_overlay = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_REMEMBER_WINDOW_PLACEMENT) {
        settings.remember_window_placement = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_HIDE_WHEN_MINIMIZED) {
        settings.hide_when_minimized = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_MODULE_ORDER) {
        settings.module_order = parse_module_order_setting(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_MONITORING_SHOW_TAGS) {
        settings.monitoring_show_tags = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UI_MONITORING_SHOW_CONNECTION_COUNT) {
        settings.monitoring_show_connection_count = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_WEB_ACCESS_LOCALHOST) {
        settings.web_access_localhost = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_DOMAIN_CAPTURE_ENABLED) {
        settings.domain_capture_enabled = setting_truthy(&value);
    }
    if let Some(value) = read_app_setting_from_sqlite(SETTING_UPDATE_CHECK_INTERVAL_MINUTES) {
        settings.update_check_interval_minutes = parse_update_check_interval_minutes(&value);
    }
    settings
}

fn read_app_setting_from_sqlite(key: &str) -> Option<String> {
    let database_path = cold_start_database_path()?;
    if !database_path.exists() {
        return None;
    }
    let conn = rusqlite::Connection::open(database_path).ok()?;
    conn.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

pub(crate) fn read_persisted_window_placement() -> Option<PersistedWindowPlacement> {
    if !read_app_setting_from_sqlite(SETTING_UI_REMEMBER_WINDOW_PLACEMENT)
        .is_some_and(|value| setting_truthy(&value))
    {
        return None;
    }

    let x = read_app_setting_from_sqlite(SETTING_UI_WINDOW_X)?
        .trim()
        .parse::<i32>()
        .ok()?;
    let y = read_app_setting_from_sqlite(SETTING_UI_WINDOW_Y)?
        .trim()
        .parse::<i32>()
        .ok()?;
    let width = read_app_setting_from_sqlite(SETTING_UI_WINDOW_WIDTH)?
        .trim()
        .parse::<f64>()
        .ok()?;
    let height = read_app_setting_from_sqlite(SETTING_UI_WINDOW_HEIGHT)?
        .trim()
        .parse::<f64>()
        .ok()?;
    let hidden = read_app_setting_from_sqlite(SETTING_UI_WINDOW_HIDDEN)
        .is_some_and(|value| setting_truthy(&value));

    Some(PersistedWindowPlacement {
        x,
        y,
        width,
        height,
        hidden,
    })
}

pub(crate) fn read_persisted_cloud_session() -> Option<CloudUserSessionResponse> {
    let user_id = read_app_setting_from_sqlite(SETTING_CLOUD_USER_ID)?;
    let provider = read_app_setting_from_sqlite(SETTING_CLOUD_PROVIDER);
    let login = provider
        .as_deref()
        .filter(|value| value.eq_ignore_ascii_case("google"))
        .map(|_| "Google".to_string())
        .unwrap_or_else(|| "Google".to_string());
    let client_id = read_app_setting_from_sqlite(SETTING_CLOUD_CLIENT_ID)?;
    let key_id = read_app_setting_from_sqlite(SETTING_CLOUD_KEY_ID)?;
    let protected_token = read_app_setting_from_sqlite(SETTING_CLOUD_SESSION_TOKEN_DPAPI)?;
    let session_token = crate::secure_store::unprotect_string(&protected_token).ok()?;
    let session_expires_at_ms = read_app_setting_from_sqlite(SETTING_CLOUD_SESSION_EXPIRES_AT_MS)?
        .parse::<u64>()
        .ok()?;
    let display_name = read_app_setting_from_sqlite(SETTING_CLOUD_DISPLAY_NAME_DPAPI)
        .and_then(|value| crate::secure_store::unprotect_string(&value).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let email = read_app_setting_from_sqlite(SETTING_CLOUD_EMAIL_DPAPI)
        .and_then(|value| crate::secure_store::unprotect_string(&value).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    Some(CloudUserSessionResponse {
        user_id,
        login,
        client_id,
        key_id,
        session_token,
        session_expires_at_ms,
        provider,
        display_name,
        email,
    })
}

pub(crate) fn read_persisted_cloud_client_private_key() -> Option<Vec<u8>> {
    let protected_key = read_app_setting_from_sqlite(SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI)?;
    let key_b64 = crate::secure_store::unprotect_string(&protected_key).ok()?;
    URL_SAFE_NO_PAD.decode(key_b64.trim()).ok()
}

pub(crate) fn read_persisted_cloud_upload_nickname() -> String {
    read_app_setting_from_sqlite(SETTING_CLOUD_UPLOAD_NICKNAME)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
}

pub(crate) fn persist_cloud_upload_nickname_to_sqlite(nickname: &str) -> Result<(), String> {
    let nickname = nickname.trim();
    if nickname.is_empty() {
        delete_app_setting_from_sqlite(SETTING_CLOUD_UPLOAD_NICKNAME)
    } else {
        persist_app_setting_to_sqlite(SETTING_CLOUD_UPLOAD_NICKNAME, nickname)
    }
}

pub(crate) fn persist_cloud_session_to_sqlite(
    session: &CloudUserSessionResponse,
    client_private_key_pkcs8_der: &[u8],
) -> Result<(), String> {
    if session.session_token.trim().is_empty() {
        return Err("cloud session token must be non-empty before saving".to_string());
    }
    if client_private_key_pkcs8_der.is_empty() {
        return Err("cloud client private key must be non-empty before saving".to_string());
    }

    let protected_token = crate::secure_store::protect_string(&session.session_token)?;
    let protected_private_key =
        crate::secure_store::protect_string(&URL_SAFE_NO_PAD.encode(client_private_key_pkcs8_der))?;
    persist_app_setting_to_sqlite(SETTING_CLOUD_USER_ID, &session.user_id)?;
    persist_app_setting_to_sqlite(SETTING_CLOUD_CLIENT_ID, &session.client_id)?;
    persist_app_setting_to_sqlite(SETTING_CLOUD_KEY_ID, &session.key_id)?;
    persist_app_setting_to_sqlite(SETTING_CLOUD_SESSION_TOKEN_DPAPI, &protected_token)?;
    persist_app_setting_to_sqlite(
        SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI,
        &protected_private_key,
    )?;
    persist_app_setting_to_sqlite(
        SETTING_CLOUD_SESSION_EXPIRES_AT_MS,
        &session.session_expires_at_ms.to_string(),
    )?;
    if let Some(provider) = session.provider.as_deref() {
        persist_app_setting_to_sqlite(SETTING_CLOUD_PROVIDER, provider)?;
    }
    persist_protected_optional_app_setting(SETTING_CLOUD_EMAIL_DPAPI, session.email.as_deref())?;
    persist_protected_optional_app_setting(
        SETTING_CLOUD_DISPLAY_NAME_DPAPI,
        session.display_name.as_deref(),
    )?;
    delete_app_setting_from_sqlite(SETTING_OBSOLETE_CLOUD_LOGIN)?;
    delete_app_setting_from_sqlite(SETTING_OBSOLETE_CLOUD_EMAIL)?;
    delete_app_setting_from_sqlite(SETTING_OBSOLETE_CLOUD_DISPLAY_NAME)?;
    delete_app_setting_from_sqlite(SETTING_OBSOLETE_CLOUD_PASSWORD_DPAPI)?;
    Ok(())
}

pub(crate) fn clear_persisted_cloud_auth_state() -> Result<(), String> {
    for key in [
        SETTING_OBSOLETE_CLOUD_LOGIN,
        SETTING_OBSOLETE_CLOUD_EMAIL,
        SETTING_CLOUD_EMAIL_DPAPI,
        SETTING_CLOUD_PROVIDER,
        SETTING_CLOUD_PROVIDER_SUBJECT_HASH,
        SETTING_OBSOLETE_CLOUD_DISPLAY_NAME,
        SETTING_CLOUD_DISPLAY_NAME_DPAPI,
        SETTING_OBSOLETE_CLOUD_PASSWORD_DPAPI,
        SETTING_CLOUD_USER_ID,
        SETTING_CLOUD_CLIENT_ID,
        SETTING_CLOUD_KEY_ID,
        SETTING_CLOUD_SESSION_TOKEN_DPAPI,
        SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI,
        SETTING_CLOUD_SESSION_EXPIRES_AT_MS,
        SETTING_OBSOLETE_CLOUD_STAY_SIGNED_IN,
    ] {
        delete_app_setting_from_sqlite(key)?;
    }
    Ok(())
}

fn persist_protected_optional_app_setting(key: &str, value: Option<&str>) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return delete_app_setting_from_sqlite(key);
    };
    let protected = crate::secure_store::protect_string(value)?;
    persist_app_setting_to_sqlite(key, &protected)
}

pub(crate) fn persist_window_placement_to_sqlite(
    placement: &PersistedWindowPlacement,
) -> Result<(), String> {
    persist_app_setting_to_sqlite(SETTING_UI_WINDOW_X, &placement.x.to_string())?;
    persist_app_setting_to_sqlite(SETTING_UI_WINDOW_Y, &placement.y.to_string())?;
    persist_app_setting_to_sqlite(SETTING_UI_WINDOW_WIDTH, &placement.width.to_string())?;
    persist_app_setting_to_sqlite(SETTING_UI_WINDOW_HEIGHT, &placement.height.to_string())?;
    persist_app_setting_to_sqlite(SETTING_UI_WINDOW_HIDDEN, &placement.hidden.to_string())?;
    Ok(())
}

pub(crate) fn append_ui_message_to_sqlite(message: &str, severity: &str) -> Result<(), String> {
    append_ui_message_to_sqlite_with_source(message, severity, "core")
}

pub(crate) fn append_ui_message_to_sqlite_with_source(
    message: &str,
    severity: &str,
    source: &str,
) -> Result<(), String> {
    let message = message.trim();
    if message.is_empty() {
        return Ok(());
    }
    let source = normalize_system_event_source_for_sqlite(source);
    let conn = open_cold_start_sqlite("message logging")?;
    ensure_system_events_schema(&conn)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| format!("failed to start system event transaction: {error}"))?;
    let severity = normalize_system_event_severity_for_sqlite(severity);
    let payload_json = serde_json::json!({ "message": message }).to_string();
    let created_at_ms = current_timestamp_ms_for_sqlite();
    let last_event = tx
        .query_row(
            r#"
            SELECT event_id, severity, source, payload_json
            FROM system_events
            WHERE component = 'ui'
              AND action_type = 'message'
              AND entity_type = 'footer'
            ORDER BY event_id DESC
            LIMIT 1
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("failed to inspect latest UI message event: {error}"))?;
    if let Some((event_id, last_severity, last_source, last_payload_json)) = last_event {
        if last_severity == severity && last_source == source && last_payload_json == payload_json {
            tx.execute(
                r#"
                UPDATE system_events
                SET created_at_ms = ?1,
                    repeat_count = repeat_count + 1
                WHERE event_id = ?2
                "#,
                rusqlite::params![created_at_ms, event_id],
            )
            .map_err(|error| format!("failed to update repeated UI message event: {error}"))?;
            rotate_system_events_for_sqlite(&tx)?;
            tx.commit()
                .map_err(|error| format!("failed to commit UI message event: {error}"))?;
            return Ok(());
        }
    }
    tx.execute(
        r#"
        INSERT INTO system_events
            (created_at_ms, repeat_count, source, component, action_type, severity, entity_type, entity_id, payload_json)
        VALUES (?1, 1, ?2, 'ui', 'message', ?3, 'footer', NULL, ?4)
        "#,
        rusqlite::params![created_at_ms, source, severity, payload_json],
    )
    .map_err(|error| format!("failed to append UI message event: {error}"))?;
    rotate_system_events_for_sqlite(&tx)?;
    tx.commit()
        .map_err(|error| format!("failed to commit UI message event: {error}"))?;
    Ok(())
}

pub(crate) fn read_ui_messages_from_sqlite(limit: usize) -> Vec<(String, String)> {
    let Ok(conn) = open_cold_start_sqlite("message history") else {
        return Vec::new();
    };
    if ensure_system_events_schema(&conn).is_err() {
        return Vec::new();
    }
    let Ok(mut stmt) = conn.prepare(
        r#"
        SELECT severity, payload_json
        FROM system_events
        WHERE component = 'ui'
          AND action_type = 'message'
        ORDER BY event_id DESC
        LIMIT ?1
        "#,
    ) else {
        return Vec::new();
    };
    let limit = limit.clamp(1, 9999);
    let Ok(rows) = stmt.query_map(rusqlite::params![limit as i64], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for (severity, payload_json) in rows.flatten() {
        let message = serde_json::from_str::<serde_json::Value>(&payload_json)
            .ok()
            .as_ref()
            .and_then(|value| {
                value
                    .get("message")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            });
        if let Some(message) = message {
            items.push((severity, message));
        }
    }
    items.reverse();
    items
}

fn persist_app_setting_to_sqlite(key: &str, value: &str) -> Result<(), String> {
    let conn = open_cold_start_sqlite("app setting persistence")?;
    conn.execute(
        r#"
        INSERT INTO app_settings (setting_key, setting_value, updated_at_ms)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(setting_key) DO UPDATE SET
            setting_value = excluded.setting_value,
            updated_at_ms = excluded.updated_at_ms
        "#,
        rusqlite::params![key, value.trim(), current_timestamp_ms_for_sqlite()],
    )
    .map_err(|error| format!("failed to persist app setting in sqlite: {error}"))?;
    Ok(())
}

fn open_cold_start_sqlite(operation: &str) -> Result<rusqlite::Connection, String> {
    let Some(database_path) = cold_start_database_path() else {
        return Err(format!("unable to locate sqlite storage for {operation}"));
    };
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create storage directory: {error}"))?;
    }
    rusqlite::Connection::open(database_path)
        .map_err(|error| format!("failed to open sqlite database for {operation}: {error}"))
}

fn ensure_system_events_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS system_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at_ms INTEGER NOT NULL,
            repeat_count INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL DEFAULT 'core',
            component TEXT NOT NULL,
            action_type TEXT NOT NULL,
            severity TEXT NOT NULL CHECK(severity IN ('debug', 'info', 'success', 'warning', 'error')),
            entity_type TEXT,
            entity_id TEXT,
            payload_json TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_system_events_created
            ON system_events(created_at_ms DESC, event_id DESC);
        CREATE INDEX IF NOT EXISTS idx_system_events_component_action
            ON system_events(component, action_type, event_id DESC);
        "#,
    )
    .map_err(|error| format!("failed to initialize system_events schema: {error}"))?;
    if !sqlite_table_has_column(conn, "system_events", "repeat_count")? {
        conn.execute(
            "ALTER TABLE system_events ADD COLUMN repeat_count INTEGER NOT NULL DEFAULT 1",
            [],
        )
        .map_err(|error| format!("failed to add system_events repeat_count column: {error}"))?;
    }
    if !sqlite_table_has_column(conn, "system_events", "source")? {
        conn.execute(
            "ALTER TABLE system_events ADD COLUMN source TEXT NOT NULL DEFAULT 'core'",
            [],
        )
        .map_err(|error| format!("failed to add system_events source column: {error}"))?;
    }
    if sqlite_system_events_needs_success_severity(conn)? {
        rebuild_sqlite_system_events_with_success_severity(conn)?;
    }
    Ok(())
}

fn sqlite_system_events_needs_success_severity(
    conn: &rusqlite::Connection,
) -> Result<bool, String> {
    let create_sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'system_events'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| format!("failed to inspect system_events schema: {error}"))?
        .unwrap_or_default()
        .to_ascii_lowercase();
    Ok(create_sql.contains("check") && !create_sql.contains("'success'"))
}

fn rebuild_sqlite_system_events_with_success_severity(
    conn: &rusqlite::Connection,
) -> Result<(), String> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;
        ALTER TABLE system_events RENAME TO system_events_rebuild_source;
        CREATE TABLE system_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            created_at_ms INTEGER NOT NULL,
            repeat_count INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL DEFAULT 'core',
            component TEXT NOT NULL,
            action_type TEXT NOT NULL,
            severity TEXT NOT NULL CHECK(severity IN ('debug', 'info', 'success', 'warning', 'error')),
            entity_type TEXT,
            entity_id TEXT,
            payload_json TEXT NOT NULL DEFAULT '{}'
        );
        INSERT INTO system_events (
            event_id,
            created_at_ms,
            repeat_count,
            source,
            component,
            action_type,
            severity,
            entity_type,
            entity_id,
            payload_json
        )
        SELECT
            event_id,
            created_at_ms,
            repeat_count,
            COALESCE(source, 'core'),
            component,
            action_type,
            severity,
            entity_type,
            entity_id,
            payload_json
        FROM system_events_rebuild_source;
        DROP TABLE system_events_rebuild_source;
        CREATE INDEX IF NOT EXISTS idx_system_events_created
            ON system_events(created_at_ms DESC, event_id DESC);
        CREATE INDEX IF NOT EXISTS idx_system_events_component_action
            ON system_events(component, action_type, event_id DESC);
        PRAGMA foreign_keys = ON;
        "#,
    )
    .map_err(|error| format!("failed to rebuild system_events severity schema: {error}"))
}

fn sqlite_table_has_column(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
) -> Result<bool, String> {
    let query = format!("PRAGMA table_info({table})");
    let mut stmt = conn
        .prepare(&query)
        .map_err(|error| format!("failed to inspect sqlite schema: {error}"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("failed to read sqlite schema: {error}"))?;
    for existing in rows.flatten() {
        if existing.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn rotate_system_events_for_sqlite(tx: &rusqlite::Transaction<'_>) -> Result<(), String> {
    tx.execute(
        r#"
        DELETE FROM system_events
        WHERE event_id NOT IN (
            SELECT event_id
            FROM system_events
            ORDER BY event_id DESC
            LIMIT 9999
        )
        "#,
        [],
    )
    .map_err(|error| format!("failed to rotate system events: {error}"))?;
    Ok(())
}

fn normalize_system_event_severity_for_sqlite(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" => "debug",
        "success" => "success",
        "warning" | "warn" => "warning",
        "error" => "error",
        _ => "info",
    }
}

fn normalize_system_event_source_for_sqlite(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "core".to_string()
    } else if trimmed.eq_ignore_ascii_case("core") {
        "core".to_string()
    } else if trimmed.eq_ignore_ascii_case("system") {
        "system".to_string()
    } else {
        trimmed.chars().take(96).collect()
    }
}

fn delete_app_setting_from_sqlite(key: &str) -> Result<(), String> {
    let Some(database_path) = cold_start_database_path() else {
        return Ok(());
    };
    if !database_path.exists() {
        return Ok(());
    }
    let conn = rusqlite::Connection::open(database_path)
        .map_err(|error| format!("failed to open sqlite database: {error}"))?;
    conn.execute(
        "DELETE FROM app_settings WHERE setting_key = ?1",
        rusqlite::params![key],
    )
    .map_err(|error| format!("failed to delete app setting from sqlite: {error}"))?;
    Ok(())
}

fn current_timestamp_ms_for_sqlite() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn cold_start_database_path() -> Option<PathBuf> {
    if let Ok(explicit) = env::var("NETSTITCH__DATA_DIR") {
        let explicit = explicit.trim();
        if !explicit.is_empty() {
            return Some(PathBuf::from(explicit).join("netstitch.sqlite3"));
        }
    }

    let cwd = env::current_dir().ok()?;
    if cwd.join("storage").exists() {
        return Some(cwd.join("storage").join("netstitch.sqlite3"));
    }

    directories::ProjectDirs::from("netstitch", "Netstitch", "Netstitch")
        .map(|dirs| dirs.data_local_dir().join("netstitch.sqlite3"))
        .or_else(|| Some(cwd.join("storage").join("netstitch.sqlite3")))
}

fn setting_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_module_order_setting(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value.trim())
        .map(normalized_module_order)
        .unwrap_or_default()
}

fn parse_update_check_interval_minutes(value: &str) -> u64 {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|minutes| *minutes > 0)
        .unwrap_or_else(default_update_check_interval_minutes)
        .clamp(1, 24 * 60)
}

fn normalized_module_order(order: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for item in order {
        let item = item.trim();
        if item.is_empty() || normalized.iter().any(|existing| existing == item) {
            continue;
        }
        normalized.push(item.to_string());
    }
    normalized
}

fn map_observation(endpoint: &ObservedEndpoint) -> ObservationDto {
    ObservationDto {
        id: endpoint.id.unwrap_or_default(),
        tracked_app_id: endpoint.tracked_app_id,
        cloud_app_id: endpoint.cloud_app_id.clone(),
        app_signature_key: endpoint.app_signature_key.clone(),
        app_signature_subject: endpoint.app_signature_subject.clone(),
        app_signature_issuer: endpoint.app_signature_issuer.clone(),
        app_signature_source: endpoint.app_signature_source.clone(),
        process_name: endpoint
            .process_name
            .clone()
            .unwrap_or_else(|| "Unknown process".to_string()),
        remote_ip: endpoint.remote_ip.to_string(),
        remote_port: endpoint.remote_port,
        protocol: match endpoint.protocol {
            SharedProtocol::Tcp => ProtocolDto::Tcp,
            SharedProtocol::Udp => ProtocolDto::Udp,
            SharedProtocol::Other => ProtocolDto::Other,
        },
        first_seen_ms: endpoint.first_seen_ms,
        first_seen: format_timestamp(endpoint.first_seen_ms),
        last_seen_ms: endpoint.last_seen_ms,
        last_seen: format_timestamp(endpoint.last_seen_ms),
        hits: endpoint.hits.min(u32::MAX as u64) as u32,
        connection_state: match endpoint.connection_state {
            SharedConnectionState::Unknown => ConnectionStateDto::Unknown,
            SharedConnectionState::Attempting => ConnectionStateDto::Attempting,
            SharedConnectionState::Established => ConnectionStateDto::Established,
            SharedConnectionState::Closing => ConnectionStateDto::Closing,
            SharedConnectionState::Failed => ConnectionStateDto::Failed,
        },
        failed_hits: endpoint.failed_hits.min(u32::MAX as u64) as u32,
        successful_hits: endpoint.successful_hits.min(u32::MAX as u64) as u32,
        is_confirmed: endpoint.is_confirmed,
        is_exported: endpoint.is_exported,
        tags: endpoint.tags.clone(),
        cloud_tags: endpoint.cloud_tags.clone(),
        enrichment: endpoint.enrichment.as_ref().map(map_ip_enrichment),
    }
}

fn map_ip_enrichment(
    enrichment: &netstitch_shared::models::IpEnrichmentDto,
) -> crate::watcher_api::IpEnrichmentDto {
    crate::watcher_api::IpEnrichmentDto {
        domain_name: enrichment.domain_name.clone(),
        domain_source: enrichment.domain_source.clone(),
        owner_name: enrichment.owner_name.clone(),
        owner_range: enrichment.owner_range.clone(),
        registry: enrichment.registry.clone(),
        country: enrichment.country.clone(),
        source: enrichment.source.clone(),
    }
}

fn map_integration(
    shared: &SharedSnapshotResponse,
    configured_repo: Option<&Path>,
    last_export_text: &str,
) -> IntegrationIntegrationDto {
    if let Some(integration) = shared.integration_status.as_ref() {
        return IntegrationIntegrationDto {
            repo_path: integration
                .repo_root
                .as_ref()
                .map(|path| path.display().to_string())
                .or_else(|| configured_repo.map(|path| path.display().to_string()))
                .unwrap_or_else(|| "Not configured".to_string()),
            export_path: integration.export_path.display().to_string(),
            reference_data_path: integration
                .reference_data_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "Module reference data not detected".to_string()),
            profile_paths: integration
                .profile_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            ready: integration.is_available,
            status_text: if integration.is_available {
                integration
                    .provider_name
                    .clone()
                    .map(|name| format!("{name} ready"))
                    .unwrap_or_else(|| "Integration integration ready".to_string())
            } else {
                integration
                    .details
                    .clone()
                    .unwrap_or_else(|| "Integration folder not confirmed yet".to_string())
            },
            last_export_text: last_export_text.to_string(),
            provider_id: integration.provider_id.clone(),
            provider_name: integration.provider_name.clone(),
            repository_url: integration.repository_url.clone(),
        };
    }

    IntegrationIntegrationDto {
        repo_path: configured_repo
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Not configured".to_string()),
        export_path: "Configure an integration module before exporting.".to_string(),
        reference_data_path: "The module reference data path appears after validation.".to_string(),
        profile_paths: Vec::new(),
        ready: false,
        status_text: "Integration folder is not configured yet".to_string(),
        last_export_text: last_export_text.to_string(),
        provider_id: None,
        provider_name: None,
        repository_url: None,
    }
}

fn export_profile_result_text(plan: &ExportProfilePlanDto) -> String {
    if plan.exported_count == 0 {
        return "Profile export skipped: no confirmed IPs".to_string();
    }
    let changed = plan
        .file_changes
        .iter()
        .map(|change| change.path.display().to_string())
        .collect::<Vec<_>>();
    if changed.is_empty() {
        format!("Profile export completed: {} IP(s)", plan.exported_count)
    } else {
        format!(
            "Profile export completed: {} IP(s); changed {}",
            plan.exported_count,
            changed.join(", ")
        )
    }
}

fn display_name(app: &netstitch_shared::models::TrackedApp) -> String {
    if let Some(name) = app.display_name.as_ref() {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return capitalize_display_name(trimmed);
        }
    }

    if let Some(name) = app.process_name.as_ref() {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return capitalize_display_name(trimmed.trim_end_matches(".exe"));
        }
    }

    app.exe_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(capitalize_display_name)
        .unwrap_or_else(|| "Tracked app".to_string())
}

fn capitalize_display_name(value: &str) -> String {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return "Tracked app".to_string();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

fn format_timestamp(timestamp_ms: u64) -> String {
    if timestamp_ms == 0 {
        return "n/a".to_string();
    }

    let dt_utc = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64);
    match dt_utc {
        Some(value) => value
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        None => "n/a".to_string(),
    }
}

fn monitor_status_text(status: MonitorStatus) -> String {
    match status {
        MonitorStatus::Stopped => "Monitoring paused".to_string(),
        MonitorStatus::Starting => "Monitoring is starting".to_string(),
        MonitorStatus::Running => "Monitoring is active".to_string(),
        MonitorStatus::Stopping => "Monitoring stop requested".to_string(),
        MonitorStatus::PermissionDenied => {
            "Monitoring requires administrator rights on Windows".to_string()
        }
        MonitorStatus::Unavailable => "Monitoring backend unavailable".to_string(),
    }
}

fn monitor_error_text(status: MonitorStatus) -> Option<String> {
    match status {
        MonitorStatus::PermissionDenied => Some(
            "Monitoring requires elevated rights. Run NetStitch with the required privileges."
                .to_string(),
        ),
        MonitorStatus::Unavailable => {
            Some("Embedded monitoring runtime is unavailable.".to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LiveWatcherApi, SnapshotRefreshResult, display_name, endpoint_probe_request_spec,
        format_timestamp, monitor_error_text, monitor_status_text, parse_endpoint_probe_targets,
    };
    use netstitch_shared::models::{
        MonitorStatus, Protocol as SharedProtocol, SnapshotResponse as SharedSnapshotResponse,
        TrackedApp,
    };
    use std::path::PathBuf;

    #[test]
    fn network_diagnostics_forward_progress_before_completion() {
        let source = include_str!("app_watcher.rs");
        for expected in [
            "NetworkDiagnosticJobEvent::Progress",
            "NetworkDiagnosticJobEvent::Finished",
            "NetworkDiagnosticProgressDto",
            "format!(\"/v1/network-diagnostics/{}\", start.job_id)",
            "Duration::from_millis(75)",
        ] {
            assert!(
                source.contains(expected),
                "network diagnostics adapter must keep token {expected}"
            );
        }
    }

    #[test]
    fn display_name_prefers_process_name() {
        let app = TrackedApp {
            id: Some(10),
            exe_path: PathBuf::from(r"C:\Games\DemoGame.exe"),
            connector_id: None,
            cloud_app_id: None,
            process_name: Some("DemoGame.exe".to_string()),
            display_name: Some("Demo Game".to_string()),
            icon_key: Some("demo_game".to_string()),
            icon_path: None,
            current_tag: None,
            enabled: true,
            created_at_ms: 0,
        };

        assert_eq!(display_name(&app), "Demo Game");
    }

    #[test]
    fn display_name_falls_back_to_file_stem() {
        let app = TrackedApp {
            id: Some(11),
            exe_path: PathBuf::from(r"C:\Games\EasyAntiCheat\launcher.exe"),
            connector_id: None,
            cloud_app_id: None,
            process_name: None,
            display_name: None,
            icon_key: None,
            icon_path: None,
            current_tag: None,
            enabled: true,
            created_at_ms: 0,
        };

        assert_eq!(display_name(&app), "Launcher");
    }

    #[test]
    fn display_name_capitalizes_chrome_process_name() {
        let app = TrackedApp {
            id: Some(12),
            exe_path: PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            connector_id: Some("chrome".to_string()),
            cloud_app_id: Some("netstitch.app.chrome".to_string()),
            process_name: Some("chrome.exe".to_string()),
            display_name: Some("chrome".to_string()),
            icon_key: Some("chrome".to_string()),
            icon_path: None,
            current_tag: None,
            enabled: true,
            created_at_ms: 0,
        };

        assert_eq!(display_name(&app), "Chrome");
    }

    #[test]
    fn timestamp_format_handles_zero_and_valid_values() {
        assert_eq!(format_timestamp(0), "n/a");
        assert_ne!(format_timestamp(1_714_980_800_000), "n/a");
    }

    #[test]
    fn successful_snapshot_clears_transient_watcher_startup_status_note() {
        let api = LiveWatcherApi::cold_start();
        api.state.borrow_mut().last_status_note =
            Some("Connecting to embedded monitoring runtime...".to_string());

        api.apply_shared_snapshot(SharedSnapshotResponse::empty(MonitorStatus::Stopped), true);

        assert_eq!(
            api.state.borrow().snapshot.ui.status_text,
            monitor_status_text(MonitorStatus::Stopped)
        );
    }

    #[test]
    fn monitor_status_mapping_is_stable() {
        assert_eq!(
            monitor_status_text(MonitorStatus::Running),
            "Monitoring is active"
        );
        assert!(
            monitor_error_text(MonitorStatus::PermissionDenied)
                .unwrap()
                .contains("privileges")
        );
        assert!(monitor_error_text(MonitorStatus::Stopped).is_none());
    }

    #[test]
    fn endpoint_probe_targets_parse_external_config_format() {
        let parsed = parse_endpoint_probe_targets(
            r#"
            # comments are allowed
            udp://1.1.1.1:53, 8.8.8.8:53
            [2606:4700:4700::1111]:53 ; 9.9.9.9:53 # inline comment
            "#,
        );

        assert_eq!(
            parsed,
            vec![
                "udp://1.1.1.1:53",
                "8.8.8.8:53",
                "[2606:4700:4700::1111]:53",
                "9.9.9.9:53"
            ]
        );
    }

    #[test]
    fn live_delete_observations_returns_backend_deleted_count() {
        let source = include_str!("app_watcher.rs");
        let method_source = source
            .split("fn delete_observations(")
            .nth(2)
            .expect("live delete_observations implementation");
        let delete_source = method_source
            .split("fn import_monitoring_csv")
            .next()
            .unwrap_or(method_source);

        assert!(source.contains("struct DeleteObservationsResponse"));
        assert!(source.contains("deleted: usize"));
        assert!(delete_source.contains("post_json::<_, DeleteObservationsResponse>"));
        assert!(delete_source.contains("Ok(response.deleted)"));
        assert!(
            !delete_source.contains("post_json::<_, serde_json::Value>"),
            "live delete must not ignore the backend deleted count"
        );
    }

    #[test]
    fn endpoint_probe_query_params_include_protocol() {
        assert_eq!(
            endpoint_probe_request_spec("udp://1.1.1.1:53", None),
            ("1.1.1.1:53".to_string(), SharedProtocol::Udp)
        );
        assert_eq!(
            endpoint_probe_request_spec("example.com:443", None),
            ("example.com:443".to_string(), SharedProtocol::Tcp)
        );
        assert_eq!(
            endpoint_probe_request_spec("1.1.1.1:53", Some("tcp")),
            ("1.1.1.1:53".to_string(), SharedProtocol::Tcp)
        );
    }

    #[test]
    fn default_watcher_addr_uses_loopback() {
        assert_eq!(super::default_watcher_addr(), "127.0.0.1:46473");
    }

    #[test]
    fn snapshot_mapping_carries_saved_ui_language() {
        let mut shared = SharedSnapshotResponse::empty(MonitorStatus::Stopped);
        shared.app_settings.ui_language_code = Some("ru-ru".to_string());
        shared.app_settings.ui_hide_when_minimized = false;
        shared.app_settings.ui_module_order = vec!["module-b".to_string(), "module-a".to_string()];
        shared.app_settings.ui_monitoring_show_tags = true;
        shared.app_settings.ui_monitoring_show_connection_count = true;
        shared.app_settings.web_access_localhost = true;

        let mapped = super::map_snapshot(&shared, "", None, "", None);

        assert_eq!(mapped.app_settings.language_code, Some("ru-ru".to_string()));
        assert!(!mapped.app_settings.hide_when_minimized);
        assert_eq!(
            mapped.app_settings.module_order,
            vec!["module-b".to_string(), "module-a".to_string()]
        );
        assert!(mapped.app_settings.monitoring_show_tags);
        assert!(mapped.app_settings.monitoring_show_connection_count);
        assert!(mapped.app_settings.web_access_localhost);
        assert!(
            mapped.ui.watcher_connected,
            "successful snapshot mapping marks the watcher API as connected"
        );
    }

    #[test]
    fn cold_start_defers_backend_refresh_until_window_can_paint() {
        let api = LiveWatcherApi::cold_start();
        let state = api.state.borrow();

        assert!(state.last_refresh_at.is_some());
        assert!(state.snapshot.tracked_apps.is_empty());
        assert!(!state.snapshot.ui.watcher_connected);
        assert!(state.snapshot.ui.error_text.is_none());
        assert!(state.snapshot.app_settings.monitoring_show_tags);
        assert!(state.snapshot.app_settings.monitoring_show_connection_count);
        assert!(
            !state
                .snapshot
                .ui
                .status_text
                .contains("Watcher endpoint not reachable yet"),
            "cold-start snapshot should not log a transient watcher error before the first refresh"
        );
    }

    #[test]
    fn watcher_connection_flag_tracks_watcher_unavailability_separately_from_local_errors() {
        let api = LiveWatcherApi::cold_start();
        api.apply_shared_snapshot(SharedSnapshotResponse::empty(MonitorStatus::Stopped), true);
        assert!(api.state.borrow().snapshot.ui.watcher_connected);

        api.set_local_error("local validation failed");
        assert!(
            api.state.borrow().snapshot.ui.watcher_connected,
            "local UI validation errors should not mark the watcher connection as down"
        );

        api.set_watcher_unavailable("request failed: connection refused");
        assert!(!api.state.borrow().snapshot.ui.watcher_connected);
    }

    #[test]
    fn watcher_availability_tracks_embedded_runtime_connection() {
        let api = LiveWatcherApi::cold_start();
        api.set_watcher_unavailable("request failed: connection refused");

        assert!(!api.watcher_available());
    }

    #[test]
    fn stale_background_refresh_does_not_overwrite_newer_local_state() {
        let api = LiveWatcherApi::cold_start();
        {
            let mut state = api.state.borrow_mut();
            state.snapshot.ui.status_text = "fresh local state".to_string();
            state.snapshot_epoch = 2;
            state.refresh_in_progress = true;
        }

        api.apply_snapshot_refresh_result(SnapshotRefreshResult {
            snapshot: Ok(SharedSnapshotResponse::empty(MonitorStatus::Running)),
            watcher_connected: true,
            status_note: None,
            failure_message: None,
            epoch: 1,
        });

        let state = api.state.borrow();
        assert!(!state.refresh_in_progress);
        assert_eq!(state.snapshot.ui.status_text, "fresh local state");
        assert_eq!(state.snapshot_epoch, 2);
    }

    #[test]
    fn identical_background_refresh_reports_no_visible_change() {
        let api = LiveWatcherApi::cold_start();
        let shared = SharedSnapshotResponse::empty(MonitorStatus::Stopped);
        api.apply_shared_snapshot(shared.clone(), true);
        let epoch = api.state.borrow().snapshot_epoch;

        let changed = api.apply_snapshot_refresh_result(SnapshotRefreshResult {
            snapshot: Ok(shared),
            watcher_connected: true,
            status_note: None,
            failure_message: None,
            epoch,
        });

        assert!(
            !changed,
            "identical background refresh should not force a desktop UI rerender"
        );
    }

    #[test]
    fn background_fallback_preserves_watcher_failure_message_when_not_connected() {
        let api = LiveWatcherApi::cold_start();

        let changed = api.apply_snapshot_refresh_result(SnapshotRefreshResult {
            snapshot: Ok(SharedSnapshotResponse::empty(MonitorStatus::Stopped)),
            watcher_connected: false,
            status_note: Some("failed to reach local watcher: connection refused".to_string()),
            failure_message: Some(
                "failed to reach local watcher: connection refused (os error 10061)".to_string(),
            ),
            epoch: 1,
        });

        assert!(changed);
        let state = api.state.borrow();
        assert_eq!(
            state.snapshot.ui.status_text,
            "failed to reach local watcher: connection refused (os error 10061)"
        );
        assert_eq!(
            state.snapshot.ui.error_text.as_deref(),
            Some("failed to reach local watcher: connection refused (os error 10061)")
        );
        assert!(!state.snapshot.ui.watcher_connected);
    }
}
