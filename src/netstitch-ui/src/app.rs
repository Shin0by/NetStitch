use crate::{
    app_watcher::{
        AppWatcherApi, PersistedWindowPlacement, append_ui_message_to_sqlite,
        clear_persisted_cloud_auth_state, local_machine_ip_for_watcher,
        persist_cloud_session_to_sqlite, persist_cloud_upload_nickname_to_sqlite,
        persist_window_placement_to_sqlite, read_persisted_cloud_client_private_key,
        read_persisted_cloud_session, read_persisted_cloud_upload_nickname,
        read_persisted_window_placement, read_ui_messages_from_sqlite,
    },
    cloud_sync::{
        CloudCatalogApp, CloudDownloadedObservation, CloudNicknameCheckStatus, CloudSearchFilters,
        CloudSyncUiState, CloudUserAppSummary, check_author_signature_availability,
        cloud_upload_app_preview_for_observation, default_cloud_base_url, download_observations,
        local_client_identifier, login_cloud_user_with_google, open_browser_url,
        quota_is_exhausted, quota_label, quota_window_label, refresh_cloud_state,
        upload_confirmed_observations, validate_author_signature,
    },
    theme,
    tray::{TrayController, TrayMenuAction, tray_menu_action_from_id},
    ui_entities as ui,
    watcher_api::{
        AddTrackedAppRequest, ConfirmObservationsRequest, DeleteIgnoredAddressRequest,
        DeleteObservationRequest, DeleteObservationsRequest, DeleteTrackedAppRequest,
        DownloadIntegrationRequest, IgnoreAddressRequest, ObservationDto, ObservationFilterDto,
        SetAllTrackedAppsEnabledRequest, SnapshotResponse, ToggleTrackedAppRequest,
        WatcherApiClient,
    },
};
use base64::Engine;
use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use dioxus::desktop::{
    self, DesktopContext, WindowCloseBehaviour,
    tao::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, WindowEvent},
    },
    trayicon::{MouseButton, TrayIconEvent},
};
use dioxus::prelude::*;
use netstitch_shared::{
    CloudObservationVisibility, CloudObservationVisibilityScope, CloudQuotaSnapshot,
    cloud_observation_ip_is_public, derive_web_access_key,
    ipc::DownloadIntegrationProviderRequest as SharedDownloadIntegrationProviderRequest,
    models::{
        ConnectionState as SharedConnectionState, ExportFileChangeDto, ExportFileOperationDto,
        ExportModeDto, ExportProfileAdvancedSettingsDto, ExportProfileAdvancedSettingsRequestDto,
        ExportProfilePlanDto, ExportProfileRequestDto, ExportProfileRuleDto,
        IntegrationDownloadProgressDto, IntegrationModuleActionDto, IntegrationModuleDto,
        IntegrationModuleHostCommandDto, IntegrationModuleUiActionClientRequestDto,
        IntegrationUiEntityDto, MonitoringCsvImportRequestDto, MonitoringCsvImportResultDto,
        MonitoringCsvImportRowDto, MonitoringImportSourceDto, ObservedEndpointId,
        ProfileExportUiStateDto, Protocol as SharedProtocol, SystemEventRequestDto,
        UiFiltersDto as SharedUiFiltersDto, UiObservationFilterDto as SharedUiObservationFilterDto,
    },
    runtime_build_version, runtime_module_version,
};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    net::IpAddr,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering as AtomicOrdering},
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

const CLOSE_TIMES_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/close-times-svgrepo-com.svg");
const MODULE_CLOSE_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/logout-svgrepo-com.svg");
const MODULE_STOP_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/stop-circle-cross.svg");
const MODULE_REORDER_LEFT_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/left-svgrepo-com.svg");
const COPY_ICON_SVG: &str = include_str!("../../../resources/ui/icons/copy.svg");
const CONFIRM_FILTERED_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/filter-add-solid-svgrepo-com.svg");
const UNCONFIRM_FILTERED_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/filter-remove-solid-svgrepo-com.svg");
const IGNORE_ADDRESS_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/eye-close-solid-svgrepo-com.svg");
const HELP_CIRCLE_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/help-circle-solid-svgrepo-com.svg");
const CLEAR_MONITORING_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/trash-solid-svgrepo-com.svg");
const CLOUD_SYNC_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/upload-to-cloud-svgrepo-com.svg");
const CLOUD_IMPORT_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/import-from-cloud-svgrepo-com.svg");
const MY_PUBLICATIONS_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/torso-svgrepo-com.svg");
const IMPORT_CSV_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/csv-import-svgrepo-com.svg");
const EXPORT_CSV_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/csv-export-svgrepo-com.svg");
const HELP_INFO_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/help-info-svgrepo-com.svg");
const APP_ENV_WEB_SCHEME: &str = "NETSTITCH__WEB_SCHEME";
const MONITORING_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/open-eye-svgrepo-com.svg");
const SORT_IDLE_ICON_SVG: &str = include_str!("../../../resources/ui/icons/sort-svgrepo-com.svg");
const SORT_ASC_ICON_SVG: &str = include_str!("../../../resources/ui/icons/sort-up-svgrepo-com.svg");
const SORT_DESC_ICON_SVG: &str =
    include_str!("../../../resources/ui/icons/sort-down-svgrepo-com.svg");
const APP_LOGO_PNG: &[u8] = include_bytes!("../../../resources/branding/images/NetStitch.png");
static MODULE_UI_ACTION_TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);
const DEFAULT_WATCHER_PORT: u16 = 46473;
const OBSERVATION_SELECTION_CONFIRM_DEBOUNCE_MS: u64 = 250;
const APP_AUTHOR: &str = "Shin0by";
const APP_EMAIL: &str = "warfactory@gmail.com";
const APP_EMAIL_HREF: &str = "mailto:warfactory@gmail.com";
const APP_REPOSITORY_URL: &str = "https://github.com/Shin0by/NetStitch";
const APP_RELEASES_URL: &str = "https://github.com/Shin0by/NetStitch/releases";
const APP_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/Shin0by/NetStitch/releases/latest";
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const INPUT_COMMIT_SCRIPT: &str = r#"
(() => {
  if (window.__netstitchInputCommitInstalled) return;
  window.__netstitchInputCommitInstalled = true;
  const hasClearButton = (control) => control?.dataset?.clearButton === 'true';
  const preservesDraft = (control) => control?.dataset?.preserveDraft === 'true';
  const hasCommittedValue = (control) =>
    Object.prototype.hasOwnProperty.call(control?.dataset || {}, 'committedValue');
  const focusedDraftValues = new WeakMap();
  const clearButtonForControl = (control) => {
    if (!hasClearButton(control)) return null;
    const shell = control?.closest?.('.path-input-shell');
    if (!shell) return null;
    return Array.from(shell.children).find((child) =>
      child.classList?.contains('path-input-clear') && child.dataset?.clearButton === 'true'
    ) || null;
  };
  const syncClearButton = (control) => {
    if (!(control instanceof HTMLInputElement || control instanceof HTMLTextAreaElement)) return;
    const button = clearButtonForControl(control);
    if (!button) return;
    button.disabled = control.value.length === 0 || control.disabled || control.readOnly;
  };
  const syncCommittedValue = (control) => {
    if (!(control instanceof HTMLInputElement || control instanceof HTMLTextAreaElement)) return;
    if (!preservesDraft(control) || !hasCommittedValue(control)) {
      syncClearButton(control);
      return;
    }
    const committed = control.dataset.committedValue ?? '';
    if (document.activeElement === control) {
      const draft = focusedDraftValues.get(control);
      if (draft !== undefined && control.value !== draft) {
        control.value = draft;
      }
      syncClearButton(control);
      return;
    }
    if (control.value !== committed) {
      control.value = committed;
    }
    syncClearButton(control);
  };
  const syncTree = (root) => {
    if (root instanceof HTMLInputElement || root instanceof HTMLTextAreaElement) {
      syncCommittedValue(root);
    }
    root?.querySelectorAll?.('input[data-preserve-draft="true"], textarea[data-preserve-draft="true"]')
      .forEach(syncCommittedValue);
  };
  const rememberFocusedDraft = (control) => {
    if (!(control instanceof HTMLInputElement || control instanceof HTMLTextAreaElement) || !preservesDraft(control)) return;
    if (hasCommittedValue(control)) {
      const committed = control.dataset.committedValue ?? '';
      if (control.value !== committed) {
        control.value = committed;
      }
    }
    syncClearButton(control);
    focusedDraftValues.set(control, control.value);
  };
  const committedObserver = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === 'attributes') {
        syncCommittedValue(mutation.target);
        continue;
      }
      for (const node of mutation.addedNodes) {
        syncTree(node);
      }
    }
  });
  committedObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-committed-value', 'disabled', 'readonly'],
    childList: true,
    subtree: true,
  });
  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Enter') return;
    const target = event.target;
    if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) return;
    if (target.dataset.commitOnEnter !== 'true') return;
    target.classList.remove('input--apply-pulse');
    void target.offsetWidth;
    target.classList.add('input--apply-pulse');
    window.setTimeout(() => target.classList.remove('input--apply-pulse'), 220);
    target.dispatchEvent(new Event('change', { bubbles: true }));
    const clickTargetId = target.dataset.enterClickTarget;
    if (clickTargetId) {
      event.preventDefault();
      queueMicrotask(() => document.getElementById(clickTargetId)?.click());
    }
  }, true);
  document.addEventListener('input', (event) => {
    if (preservesDraft(event.target)) {
      focusedDraftValues.set(event.target, event.target.value);
    }
    syncClearButton(event.target);
  }, true);
  document.addEventListener('change', (event) => {
    if (preservesDraft(event.target)) {
      focusedDraftValues.set(event.target, event.target.value);
    }
    syncClearButton(event.target);
  }, true);
  document.addEventListener('focusin', (event) => {
    rememberFocusedDraft(event.target);
  }, true);
  document.addEventListener('focusout', (event) => {
    if (preservesDraft(event.target)) {
      focusedDraftValues.delete(event.target);
    }
  }, true);
  document.addEventListener('click', (event) => {
    const button = event.target?.closest?.('.path-input-clear');
    if (!button || button.disabled || button.dataset?.clearButton !== 'true') return;
    const shell = button.closest('.path-input-shell');
    const control = shell?.querySelector?.('input, textarea');
    if (!(control instanceof HTMLInputElement || control instanceof HTMLTextAreaElement)) return;
    if (!hasClearButton(control)) return;
    if (control.disabled || control.readOnly) return;
    if (control.value.length === 0) return;
    control.value = '';
    if (preservesDraft(control)) {
      focusedDraftValues.set(control, '');
    }
    syncClearButton(control);
    control.dispatchEvent(new Event('input', { bubbles: true }));
  }, true);
  queueMicrotask(() => {
    document.querySelectorAll('input[data-preserve-draft="true"], textarea[data-preserve-draft="true"]')
      .forEach(syncCommittedValue);
    document.querySelectorAll('.path-input-shell input[data-clear-button="true"], .path-input-shell textarea[data-clear-button="true"]')
      .forEach(syncClearButton);
  });
})();
"#;

fn pulse_text_input(mut pulse: Signal<Option<&'static str>>, field: &'static str) {
    pulse.set(Some(field));
    spawn(async move {
        tokio::time::sleep(Duration::from_millis(220)).await;
        if pulse() == Some(field) {
            pulse.set(None);
        }
    });
}
static ICON_IMAGE_SRC_CACHE: LazyLock<Mutex<BTreeMap<String, String>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static APP_LOGO_DATA_URI: LazyLock<String> = LazyLock::new(|| {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(APP_LOGO_PNG)
    )
});
static LOCAL_MACHINE_WEB_IP_CACHE: LazyLock<Option<String>> =
    LazyLock::new(local_machine_ip_for_watcher);

fn desktop_http_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static(netstitch_shared::models::CLIENT_HEADER_NAME),
        HeaderValue::from_static(netstitch_shared::models::CLIENT_HEADER_DESKTOP_UI),
    );
    headers
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HeaderObservationFilters {
    app_search: String,
    domain_search: String,
    port_search: String,
    protocol: String,
    public_ip: bool,
}

impl Default for HeaderObservationFilters {
    fn default() -> Self {
        Self {
            app_search: String::new(),
            domain_search: String::new(),
            port_search: String::new(),
            protocol: String::new(),
            public_ip: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloudOverlayMode {
    Download,
    Upload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservationSortColumn {
    App,
    Ip,
    Domain,
    Port,
    Protocol,
    Connection,
    FirstSeen,
    LastSeen,
    Hits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloudAppSortColumn {
    App,
    Company,
    AvailableRows,
    Authors,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CloudAppSortState {
    column: Option<CloudAppSortColumn>,
    descending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloudRowSortColumn {
    App,
    Source,
    Ip,
    Domain,
    Port,
    Protocol,
    Connection,
    Hits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloudPublicationSortColumn {
    App,
    NewRows,
    NonPublicRows,
    AuthorRows,
    TotalRows,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CloudRowSortState {
    column: Option<CloudRowSortColumn>,
    descending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CloudPublicationSortState {
    column: Option<CloudPublicationSortColumn>,
    descending: bool,
}

impl Default for CloudPublicationSortState {
    fn default() -> Self {
        Self {
            column: Some(CloudPublicationSortColumn::NewRows),
            descending: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ObservationSortState {
    column: Option<ObservationSortColumn>,
    descending: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CsvExportRow {
    application: String,
    app_connector_id: String,
    cloud_app_id: String,
    app_signature_key: String,
    app_signature_subject: String,
    app_signature_issuer: String,
    ip: String,
    domain: String,
    port: String,
    protocol: String,
    connection: String,
    requests: String,
    first_seen: String,
    last_seen: String,
}

#[derive(Clone, Debug)]
struct CloudRefreshRunResult {
    state: CloudSyncUiState,
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct CloudAuthRunResult {
    state: CloudSyncUiState,
    error: Option<String>,
    refresh_error: Option<String>,
}

#[derive(Clone, Debug)]
struct CloudUploadRunResult {
    state: CloudSyncUiState,
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct CloudDownloadRunResult {
    state: CloudSyncUiState,
    error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AppUpdateCheckState {
    latest_version: Option<String>,
    latest_url: Option<String>,
    update_available: bool,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CloudMessageLabels {
    refresh_done: String,
    download: String,
    upload: String,
    rows: String,
    requests: String,
    skipped_non_public: String,
}

#[derive(Default)]
struct PendingObservationSelectionConfirm {
    select_ids: BTreeSet<u64>,
    deselect_ids: BTreeSet<u64>,
    in_flight_select_ids: BTreeSet<u64>,
    in_flight_deselect_ids: BTreeSet<u64>,
    changed_at: Option<Instant>,
}

#[derive(Default)]
struct ObservationSelectionStore {
    selected_ids: BTreeSet<u64>,
    initialized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusHistoryKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StatusHistoryLine {
    text: String,
    kind: StatusHistoryKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModuleHostDialogState {
    module_id: String,
    dialog_id: String,
    title: String,
    message: String,
    show_cancel: bool,
    icon_src: Option<String>,
    icon_fallback: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FooterProgressState {
    label: String,
    percent: u8,
    meta: String,
    stages: Vec<ProgressStage>,
}

impl StatusHistoryLine {
    fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusHistoryKind::Info,
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusHistoryKind::Error,
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusHistoryKind::Success,
        }
    }

    fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: StatusHistoryKind::Warning,
        }
    }

    fn class_suffix(&self) -> &'static str {
        match self.kind {
            StatusHistoryKind::Info => "",
            StatusHistoryKind::Success => " footer-message-text--success",
            StatusHistoryKind::Warning => " footer-message-text--warning",
            StatusHistoryKind::Error => " footer-message-text--error",
        }
    }
}

impl StatusHistoryKind {
    fn severity(self) -> &'static str {
        match self {
            StatusHistoryKind::Info => "info",
            StatusHistoryKind::Success => "success",
            StatusHistoryKind::Warning => "warning",
            StatusHistoryKind::Error => "error",
        }
    }

    fn from_severity(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "success" => StatusHistoryKind::Success,
            "warning" | "warn" => StatusHistoryKind::Warning,
            "error" => StatusHistoryKind::Error,
            _ => StatusHistoryKind::Info,
        }
    }
}

trait FooterHistoryLine {
    fn footer_text(&self) -> &str;
    fn footer_class_suffix(&self) -> &'static str;
}

impl FooterHistoryLine for StatusHistoryLine {
    fn footer_text(&self) -> &str {
        &self.text
    }

    fn footer_class_suffix(&self) -> &'static str {
        self.class_suffix()
    }
}

impl FooterHistoryLine for String {
    fn footer_text(&self) -> &str {
        self
    }

    fn footer_class_suffix(&self) -> &'static str {
        ""
    }
}

impl Default for ObservationSortState {
    fn default() -> Self {
        Self {
            column: Some(ObservationSortColumn::LastSeen),
            descending: true,
        }
    }
}

impl ObservationSortState {
    fn toggled(self, column: ObservationSortColumn) -> Self {
        if self.column == Some(column) {
            if self.descending {
                Self::default()
            } else {
                Self {
                    column: Some(column),
                    descending: true,
                }
            }
        } else {
            Self {
                column: Some(column),
                descending: false,
            }
        }
    }

    fn indicator_state(self, column: ObservationSortColumn) -> &'static str {
        if self.column != Some(column) {
            "idle"
        } else if self.descending {
            "desc"
        } else {
            "asc"
        }
    }
}

impl CloudAppSortState {
    fn toggled(self, column: CloudAppSortColumn) -> Self {
        if self.column == Some(column) {
            if self.descending {
                Self::default()
            } else {
                Self {
                    column: Some(column),
                    descending: true,
                }
            }
        } else {
            Self {
                column: Some(column),
                descending: false,
            }
        }
    }

    fn indicator_state(self, column: CloudAppSortColumn) -> &'static str {
        if self.column != Some(column) {
            "idle"
        } else if self.descending {
            "desc"
        } else {
            "asc"
        }
    }
}

impl CloudRowSortState {
    fn toggled(self, column: CloudRowSortColumn) -> Self {
        if self.column == Some(column) {
            if self.descending {
                Self::default()
            } else {
                Self {
                    column: Some(column),
                    descending: true,
                }
            }
        } else {
            Self {
                column: Some(column),
                descending: false,
            }
        }
    }

    fn indicator_state(self, column: CloudRowSortColumn) -> &'static str {
        if self.column != Some(column) {
            "idle"
        } else if self.descending {
            "desc"
        } else {
            "asc"
        }
    }
}

impl CloudPublicationSortState {
    fn toggled(self, column: CloudPublicationSortColumn) -> Self {
        if self.column == Some(column) {
            if self.descending {
                Self::default()
            } else {
                Self {
                    column: Some(column),
                    descending: true,
                }
            }
        } else {
            Self {
                column: Some(column),
                descending: false,
            }
        }
    }

    fn indicator_state(self, column: CloudPublicationSortColumn) -> &'static str {
        if self.column != Some(column) {
            "idle"
        } else if self.descending {
            "desc"
        } else {
            "asc"
        }
    }
}

#[allow(unused_mut)]
#[component]
pub fn App() -> Element {
    let mut watcher = use_signal(AppWatcherApi::cold_start);
    let window = desktop::use_window();
    let tray = use_hook(TrayController::build);
    let mut remembered_window_placement = use_signal(read_persisted_window_placement);
    let mut show_integration_module_prompt = use_signal(|| false);
    let mut selected_integration_module_id = use_signal(|| None::<String>);
    let mut module_host_dialog = use_signal(|| None::<ModuleHostDialogState>);
    let mut module_ui_page = use_signal(|| "main".to_string());
    let module_ui_values = use_signal(HashMap::<String, serde_json::Value>::new);
    let mut module_ui_action_generation = use_signal(|| 0_u64);
    let mut module_order_editing = use_signal(|| false);
    let saved_module_order = watcher.read().snapshot().app_settings.module_order.clone();
    let mut module_order = use_signal(move || saved_module_order.clone());
    let mut show_integration_prompt = use_signal(|| false);
    let mut return_integration_prompt_to_module = use_signal(|| false);
    let mut show_profile_export_prompt = use_signal(|| false);
    let mut return_profile_export_to_module = use_signal(|| false);
    let mut show_profile_export_advanced_wizard = use_signal(|| false);
    let mut show_cloud_sync_prompt = use_signal(|| false);
    let mut cloud_overlay_mode = use_signal(|| CloudOverlayMode::Download);
    let initial_filter_drafts = watcher.read().snapshot().filters.clone();
    let mut monitoring_ip_filter_draft = use_signal({
        let value = initial_filter_drafts.search_text.clone();
        move || value.clone()
    });
    let mut monitoring_ip_filter_dirty = use_signal(|| false);
    let mut monitoring_domain_filter_draft = use_signal({
        let value = initial_filter_drafts.domain_search.clone();
        move || value.clone()
    });
    let mut monitoring_domain_filter_dirty = use_signal(|| false);
    let mut monitoring_port_filter_draft = use_signal({
        let value = initial_filter_drafts.port_search.clone();
        move || value.clone()
    });
    let mut monitoring_port_filter_dirty = use_signal(|| false);
    let mut cloud_app_search = use_signal(String::new);
    let mut cloud_app_search_draft = use_signal(String::new);
    let mut cloud_publisher_search = use_signal(String::new);
    let mut cloud_publisher_search_draft = use_signal(String::new);
    let mut cloud_ip_search = use_signal(String::new);
    let mut cloud_ip_search_draft = use_signal(String::new);
    let mut cloud_domain_search = use_signal(String::new);
    let mut cloud_domain_search_draft = use_signal(String::new);
    let mut cloud_port_search = use_signal(String::new);
    let mut cloud_port_search_draft = use_signal(String::new);
    let mut cloud_protocol_filter = use_signal(|| "all".to_string());
    let mut cloud_source_search = use_signal(String::new);
    let mut cloud_source_search_draft = use_signal(String::new);
    let mut cloud_row_status_filter = use_signal(|| ObservationFilterDto::All);
    let mut cloud_visibility_scope_filter = use_signal(|| CloudObservationVisibilityScope::All);
    let mut cloud_selected_app_id = use_signal(|| None::<String>);
    let mut cloud_loading_app_id = use_signal(|| None::<String>);
    let mut cloud_scope_mine = use_signal(|| false);
    let persisted_cloud_upload_nickname = read_persisted_cloud_upload_nickname();
    let mut cloud_upload_nickname = use_signal({
        let value = persisted_cloud_upload_nickname.clone();
        move || value.clone()
    });
    let mut cloud_upload_nickname_draft =
        use_signal(move || persisted_cloud_upload_nickname.clone());
    let mut cloud_upload_nickname_dirty = use_signal(|| false);
    let mut cloud_upload_private = use_signal(|| false);
    let mut cloud_nickname_check_status = use_signal(CloudNicknameCheckStatus::default);
    let mut cloud_nickname_check_generation = use_signal(|| 0u64);
    let mut cloud_filter_generation = use_signal(|| 0u64);
    let mut cloud_app_sort = use_signal(CloudAppSortState::default);
    let mut cloud_row_sort = use_signal(CloudRowSortState::default);
    let mut cloud_publication_sort = use_signal(CloudPublicationSortState::default);
    let mut input_apply_pulse = use_signal(|| None::<&'static str>);
    let initial_pending_exe_path = watcher.read().snapshot().pending_exe_path.clone();
    let mut pending_exe_path_draft = use_signal(move || initial_pending_exe_path.clone());
    let mut pending_exe_path_dirty = use_signal(|| false);
    let mut cloud_state = use_signal(|| CloudSyncUiState {
        client_identifier: local_client_identifier().unwrap_or_default(),
        session: read_persisted_cloud_session(),
        client_private_key_pkcs8_der: read_persisted_cloud_client_private_key(),
        ..CloudSyncUiState::default()
    });
    let mut show_information_prompt = use_signal(|| false);
    let mut app_update_check = use_signal(AppUpdateCheckState::default);
    let mut show_domain_capture_admin_prompt = use_signal(|| false);
    let mut integration_path_input = use_signal(String::new);
    let mut integration_path_applied = use_signal(String::new);
    let mut integration_feedback = use_signal(|| None::<String>);
    let mut profile_export_repo_root_key = use_signal(String::new);
    let mut profile_export_mode = use_signal(ExportModeDto::default);
    let mut profile_export_attach_path_input = use_signal(String::new);
    let mut profile_export_attach_path_draft = use_signal(String::new);
    let mut profile_export_patch_path_input = use_signal(String::new);
    let mut profile_export_patch_path_draft = use_signal(String::new);
    let mut profile_export_merge_path_input = use_signal(String::new);
    let mut profile_export_merge_path_draft = use_signal(String::new);
    let mut profile_export_generated_name = use_signal(|| "NetStitch".to_string());
    let mut profile_export_generated_name_draft = use_signal(|| "NetStitch".to_string());
    let mut profile_export_dangerous_confirmed = use_signal(|| false);
    let mut profile_export_preview = use_signal(|| None::<ExportProfilePlanDto>);
    let mut profile_export_manual_plan_key = use_signal(|| None::<String>);
    let mut profile_export_feedback = use_signal(|| None::<String>);
    let mut profile_export_advanced_create_rules = use_signal(|| false);
    let mut profile_export_advanced_exclude_ips = use_signal(|| false);
    let mut profile_export_advanced_whois_ranges = use_signal(|| false);
    let mut profile_export_advanced_domains = use_signal(|| false);
    let mut profile_export_advanced_manual_domains = use_signal(String::new);
    let mut profile_export_advanced_template_rule = use_signal(String::new);
    let mut profile_export_advanced_ready_for_export = use_signal(|| false);
    let mut profile_export_advanced_domain_cleanup_requested = use_signal(|| false);
    let mut integration_download = use_signal(IntegrationDownloadUiState::default);
    let mut integration_prompt_was_open = use_signal(|| false);
    let mut pending_scan_after_setup = use_signal(|| false);
    let mut hide_when_minimized =
        use_signal(|| watcher.read().snapshot().app_settings.hide_when_minimized);
    let mut remember_window_placement = use_signal(|| {
        watcher
            .read()
            .snapshot()
            .app_settings
            .remember_window_placement
    });
    let mut window_visible = use_signal(|| true);
    let mut hidden_window_was_maximized = use_signal(|| false);
    let mut last_tray_menu_event = use_signal(|| None::<(String, Instant)>);
    let mut pending_delete_app = use_signal(|| None::<crate::watcher_api::TrackedAppDto>);
    let mut pending_delete_ignored_address =
        use_signal(|| None::<crate::watcher_api::IgnoredAddressDto>);
    let mut pending_delete_observation = use_signal(|| None::<crate::watcher_api::ObservationDto>);
    let mut show_clear_monitoring_prompt = use_signal(|| false);
    let mut status_history = use_signal(load_footer_message_history);
    let mut cloud_download_progress = use_signal(|| None::<FooterProgressState>);
    let mut cloud_upload_progress = use_signal(|| None::<FooterProgressState>);
    let mut initial_connector_events_seeded = use_signal(|| false);
    let mut module_status_event_keys = use_signal(Vec::<String>::new);
    let mut last_watcher_connected = use_signal(|| None::<bool>);
    let mut last_build_status_event = use_signal(|| None::<String>);
    let mut last_current_status_event = use_signal(|| None::<String>);
    let mut last_app_status_event = use_signal(|| None::<String>);
    let mut last_watcher_status_event = use_signal(|| None::<String>);
    let mut last_tool_status_event = use_signal(|| None::<String>);
    let mut last_web_server_status_event = use_signal(|| None::<String>);
    let mut last_dns_status_event = use_signal(|| None::<String>);
    let mut last_domain_capture_error = use_signal(String::new);
    let mut last_domain_capture_admin_rejected = use_signal(|| false);
    let language_catalog = use_hook(crate::i18n::I18nCatalog::load);
    let saved_language = watcher.read().snapshot().app_settings.language_code.clone();
    let default_language = language_catalog.preferred_language_code(saved_language.as_deref());
    let mut selected_language = use_signal(move || default_language.clone());
    let mut language_menu_open = use_signal(|| false);
    let exe_file_dialog_open = use_signal(|| false);
    let integration_folder_dialog_open = use_signal(|| false);
    let module_path_picker_dialog_open = use_signal(|| false);
    let profile_export_file_dialog_open = use_signal(|| false);
    let csv_import_file_dialog_open = use_signal(|| false);
    let csv_export_file_dialog_open = use_signal(|| false);
    let mut observation_sort = use_signal(ObservationSortState::default);
    let mut monitoring_ui_refresh_nonce = use_signal(|| 0u64);
    let mut last_observation_selection_anchor = use_signal(|| None::<u64>);
    let mut last_cloud_download_selection_anchor = use_signal(|| None::<String>);
    let observation_selection_store =
        use_hook(|| Arc::new(Mutex::new(ObservationSelectionStore::default())));
    let pending_observation_selection_confirm =
        use_hook(|| Arc::new(Mutex::new(PendingObservationSelectionConfirm::default())));

    use_hook({
        let window = window.clone();
        move || {
            window.set_close_behavior(WindowCloseBehaviour::WindowCloses);
        }
    });

    use_effect({
        let window = window.clone();
        let remembered_window_placement = remembered_window_placement;
        move || {
            if let Some(placement) = remembered_window_placement().as_ref() {
                apply_persisted_window_placement(&window, placement);
            }
        }
    });

    use_effect({
        let window = window.clone();
        move || {
            let _ = window.webview.evaluate_script(theme::TOOLTIP_SCRIPT);
            let _ = window.webview.evaluate_script(INPUT_COMMIT_SCRIPT);
        }
    });

    use_future({
        let mut status_history = status_history;
        move || async move {
            for _ in 0..120 {
                if !status_history.read().is_empty() {
                    break;
                }
                let loaded = tokio::task::spawn_blocking(load_footer_message_history)
                    .await
                    .unwrap_or_default();
                if !loaded.is_empty() {
                    let mut write = status_history.write();
                    if merge_persisted_status_history_if_empty(&mut write, loaded) {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    });

    use_future({
        let mut app_update_check = app_update_check;
        let watcher = watcher;
        move || async move {
            loop {
                let current_version = runtime_build_version().to_string();
                let checked_version = current_version.clone();
                let result =
                    tokio::task::spawn_blocking(move || check_latest_app_update(&checked_version))
                        .await
                        .unwrap_or_else(|error| AppUpdateCheckState {
                            error: Some(format!("update check task failed: {error}")),
                            ..AppUpdateCheckState::default()
                        });
                let base_url = watcher.read().live_base_url();
                let event = update_check_system_event(&current_version, &result);
                record_system_event_via_watcher(base_url, event);
                app_update_check.set(result);
                let interval = update_check_interval_duration(
                    watcher
                        .read()
                        .snapshot()
                        .app_settings
                        .update_check_interval_minutes,
                );
                tokio::time::sleep(interval).await;
            }
        }
    });

    use_future({
        let mut watcher = watcher;
        let pending_selection_confirm = pending_observation_selection_confirm.clone();
        move || {
            let pending_selection_confirm = pending_selection_confirm.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let Some((select_ids, deselect_ids)) =
                        drain_ready_observation_selection_confirm(&pending_selection_confirm)
                    else {
                        continue;
                    };
                    if !select_ids.is_empty() {
                        watcher
                            .write()
                            .confirm_observations_deferred(ConfirmObservationsRequest {
                                observation_ids: select_ids,
                                confirmed: Some(true),
                            });
                    }
                    if !deselect_ids.is_empty() {
                        watcher
                            .write()
                            .confirm_observations_deferred(ConfirmObservationsRequest {
                                observation_ids: deselect_ids,
                                confirmed: Some(false),
                            });
                    }
                }
            }
        }
    });

    use_effect({
        let mut watcher = watcher;
        let mut profile_export_repo_root_key = profile_export_repo_root_key;
        let mut profile_export_mode = profile_export_mode;
        let mut profile_export_attach_path_input = profile_export_attach_path_input;
        let mut profile_export_attach_path_draft = profile_export_attach_path_draft;
        let mut profile_export_patch_path_input = profile_export_patch_path_input;
        let mut profile_export_patch_path_draft = profile_export_patch_path_draft;
        let mut profile_export_merge_path_input = profile_export_merge_path_input;
        let mut profile_export_merge_path_draft = profile_export_merge_path_draft;
        let mut profile_export_generated_name = profile_export_generated_name;
        let mut profile_export_generated_name_draft = profile_export_generated_name_draft;
        let mut profile_export_dangerous_confirmed = profile_export_dangerous_confirmed;
        let mut profile_export_advanced_create_rules = profile_export_advanced_create_rules;
        let mut profile_export_advanced_exclude_ips = profile_export_advanced_exclude_ips;
        let mut profile_export_advanced_whois_ranges = profile_export_advanced_whois_ranges;
        let mut profile_export_advanced_domains = profile_export_advanced_domains;
        let mut profile_export_advanced_manual_domains = profile_export_advanced_manual_domains;
        let mut profile_export_advanced_template_rule = profile_export_advanced_template_rule;
        let mut profile_export_advanced_ready_for_export = profile_export_advanced_ready_for_export;
        let mut profile_export_advanced_domain_cleanup_requested =
            profile_export_advanced_domain_cleanup_requested;
        let mut profile_export_preview = profile_export_preview;
        let mut profile_export_manual_plan_key = profile_export_manual_plan_key;
        let mut profile_export_feedback = profile_export_feedback;
        move || {
            let snapshot = watcher.read().snapshot();
            let integration = snapshot.integration.clone();
            let saved_state = snapshot.app_settings.profile_export_ui_state.clone();
            drop(snapshot);
            let repo_key = profile_export_integration_repo_key(&integration);
            if repo_key == profile_export_repo_root_key() {
                return;
            }

            let default_path = default_profile_export_path(&integration);
            let next_state = saved_state
                .filter(|state| state.repo_root_key == repo_key)
                .map(|state| profile_export_ui_state_with_defaults(state, &default_path))
                .unwrap_or_else(|| {
                    default_profile_export_ui_state(repo_key.clone(), default_path.clone())
                });
            profile_export_repo_root_key.set(next_state.repo_root_key.clone());
            profile_export_mode.set(next_state.mode);
            profile_export_attach_path_input.set(next_state.attach_profile_path.clone());
            profile_export_attach_path_draft.set(next_state.attach_profile_path.clone());
            profile_export_patch_path_input.set(next_state.patch_profile_path.clone());
            profile_export_patch_path_draft.set(next_state.patch_profile_path.clone());
            profile_export_merge_path_input.set(next_state.merge_profile_path.clone());
            profile_export_merge_path_draft.set(next_state.merge_profile_path.clone());
            profile_export_generated_name.set(next_state.generated_profile_name.clone());
            profile_export_generated_name_draft.set(next_state.generated_profile_name.clone());
            profile_export_dangerous_confirmed.set(next_state.dangerous_confirmed);
            profile_export_advanced_create_rules
                .set(next_state.advanced_create_rules_for_uncovered);
            profile_export_advanced_exclude_ips.set(next_state.advanced_add_ips_to_exclude);
            profile_export_advanced_whois_ranges
                .set(next_state.advanced_use_whois_ranges_for_export);
            profile_export_advanced_domains.set(next_state.advanced_add_detected_domains);
            profile_export_advanced_manual_domains.set(next_state.advanced_manual_domains.clone());
            profile_export_advanced_template_rule
                .set(next_state.advanced_template_rule_source.clone());
            profile_export_advanced_ready_for_export.set(next_state.advanced_ready_for_export);
            profile_export_advanced_domain_cleanup_requested
                .set(next_state.advanced_domain_cleanup_requested);
            profile_export_preview.set(None);
            profile_export_manual_plan_key.set(None);
            profile_export_feedback.set(None);
            watcher.write().set_profile_export_ui_state(next_state);
        }
    });

    use_effect({
        let mut watcher = watcher;
        move || {
            let repo_key = profile_export_repo_root_key();
            if repo_key.is_empty() {
                return;
            }
            watcher
                .write()
                .set_profile_export_ui_state(ProfileExportUiStateDto {
                    repo_root_key: repo_key,
                    mode: profile_export_mode(),
                    attach_profile_path: profile_export_attach_path_input(),
                    patch_profile_path: profile_export_patch_path_input(),
                    merge_profile_path: profile_export_merge_path_input(),
                    generated_profile_name: profile_export_generated_name(),
                    dangerous_confirmed: profile_export_dangerous_confirmed(),
                    advanced_create_rules_for_uncovered: profile_export_advanced_create_rules(),
                    advanced_add_ips_to_exclude: profile_export_advanced_exclude_ips(),
                    advanced_use_whois_ranges_for_export: profile_export_advanced_whois_ranges(),
                    advanced_add_detected_domains: profile_export_advanced_domains(),
                    advanced_manual_domains: profile_export_advanced_manual_domains(),
                    advanced_template_rule_source: profile_export_advanced_template_rule(),
                    advanced_ready_for_export: profile_export_advanced_ready_for_export(),
                    advanced_domain_cleanup_requested:
                        profile_export_advanced_domain_cleanup_requested(),
                });
        }
    });

    use_future({
        let mut watcher = watcher;
        let show_integration_prompt = show_integration_prompt;
        let mut monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        move || async move {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            loop {
                if show_integration_prompt() {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    continue;
                }

                let refresh_job = watcher.read().start_snapshot_refresh_job();
                if let Some(job) = refresh_job {
                    loop {
                        if let Some(result) = job.try_finish() {
                            let before_snapshot = watcher.read().snapshot();
                            let changed = watcher.read().apply_snapshot_refresh_result(result);
                            let after_snapshot = watcher.read().snapshot();
                            if changed
                                && snapshot_ui_render_relevant_changed(
                                    &before_snapshot,
                                    &after_snapshot,
                                )
                            {
                                monitoring_ui_refresh_nonce
                                    .set(monitoring_ui_refresh_nonce().wrapping_add(1));
                            }
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(650)).await;
            }
        }
    });

    use_future({
        let window = window.clone();
        let mut window_visible = window_visible;
        let show_integration_module_prompt = show_integration_module_prompt;
        let show_integration_prompt = show_integration_prompt;
        let show_cloud_sync_prompt = show_cloud_sync_prompt;
        move || {
            let window = window.clone();
            async move {
                loop {
                    if show_integration_module_prompt()
                        || show_integration_prompt()
                        || show_cloud_sync_prompt()
                    {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }

                    if window_visible() && !window.is_visible() && !window.is_minimized() {
                        window_visible.set(false);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

    let _monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce();
    let snapshot = watcher.read().snapshot();
    let monitoring_active_now = snapshot.ui.monitoring;
    let header_filters = HeaderObservationFilters {
        app_search: snapshot.filters.app_search.clone(),
        domain_search: snapshot.filters.domain_search.clone(),
        port_search: snapshot.filters.port_search.clone(),
        protocol: snapshot.filters.protocol.clone(),
        public_ip: snapshot.filters.public_ip,
    };
    let tool_available = watcher.read().tool_available();
    let ui_controls_disabled = shell_controls_disabled(&snapshot);
    let cloud_overlay_active = show_cloud_sync_prompt();
    let cloud_import_active =
        show_cloud_sync_prompt() && cloud_overlay_mode() == CloudOverlayMode::Download;
    let cloud_import_loading = cloud_import_active && cloud_loading_app_id().is_some();
    let shell_class = match (ui_controls_disabled, cloud_overlay_active) {
        (true, true) => "shell shell--controls-disabled shell--cloud-overlay-active",
        (true, false) => "shell shell--controls-disabled",
        (false, true) => "shell shell--cloud-overlay-active",
        (false, false) => "shell",
    };
    use_effect({
        let committed_value = snapshot.filters.search_text.clone();
        let mut draft = monitoring_ip_filter_draft;
        let dirty = monitoring_ip_filter_dirty;
        move || {
            if !dirty() && draft() != committed_value {
                draft.set(committed_value.clone());
            }
        }
    });
    use_effect({
        let committed_value = snapshot.filters.domain_search.clone();
        let mut draft = monitoring_domain_filter_draft;
        let dirty = monitoring_domain_filter_dirty;
        move || {
            if !dirty() && draft() != committed_value {
                draft.set(committed_value.clone());
            }
        }
    });
    use_effect({
        let committed_value = snapshot.filters.port_search.clone();
        let mut draft = monitoring_port_filter_draft;
        let dirty = monitoring_port_filter_dirty;
        move || {
            if !dirty() && draft() != committed_value {
                draft.set(committed_value.clone());
            }
        }
    });
    use_effect({
        let committed_value = snapshot.pending_exe_path.clone();
        let mut draft = pending_exe_path_draft;
        let dirty = pending_exe_path_dirty;
        move || {
            if !dirty() && draft() != committed_value {
                draft.set(committed_value.clone());
            }
        }
    });
    use_effect({
        let committed_value = cloud_upload_nickname();
        let mut draft = cloud_upload_nickname_draft;
        let dirty = cloud_upload_nickname_dirty;
        move || {
            if !dirty() && draft() != committed_value {
                draft.set(committed_value.clone());
            }
        }
    });
    let visible_observations = sort_observations(
        &snapshot,
        filter_observations_with_header_filters(&snapshot, &header_filters),
        observation_sort(),
    );
    let visible_observations_for_selection = Arc::new(visible_observations.clone());
    sync_observation_selection_store(
        &observation_selection_store,
        &pending_observation_selection_confirm,
        &snapshot.observations,
    );
    let selected_observation_ids = clone_observation_selection_store(&observation_selection_store);
    let monitoring_total_count = snapshot.observations.len();
    let monitoring_displayed_count = visible_observations.len();
    let monitoring_selected_count = selected_observation_ids.len();
    let app_filter_options = tracked_app_filter_options(&snapshot);
    let enabled_tracked_apps_count = effective_enabled_tracked_apps_count(&snapshot);
    let tracked_apps_enabled_meta_class = if enabled_tracked_apps_count > 0 {
        "panel-header-meta panel-header-meta--success"
    } else {
        "panel-header-meta panel-header-meta--danger"
    };
    let enable_all_overlay = snapshot.app_settings.enable_all_overlay;
    let selected_language_code = selected_language();
    let language_options = language_catalog.language_options();
    let selected_language_label = language_options
        .iter()
        .find(|language| language.code == selected_language_code)
        .map(|language| language.label.clone())
        .unwrap_or_else(|| selected_language_code.clone());
    let t = |key: &str| language_catalog.text(&selected_language_code, key);
    let header_web_access_localhost = t("header.web_access.localhost");
    let header_domain_capture = t("header.domain_capture");
    let domain_capture_tooltip = if snapshot.app_settings.domain_capture_enabled {
        t("header.domain_capture.enabled_tooltip")
    } else {
        t("header.domain_capture.disabled_tooltip")
    };
    let web_access_tooltip = if snapshot.app_settings.web_access_localhost {
        t("switch.disable_web_access")
    } else {
        t("switch.enable_web_access")
    };
    let tracked_apps_title = t("tracked_apps.title");
    let tracked_apps_help = t("tracked_apps.help");
    let tracked_apps_path_placeholder = t("tracked_apps.path_placeholder");
    let tracked_apps_add_exe = t("tracked_apps.add_exe");
    let input_clear = t("input.clear");
    let tracked_apps_enable_all = t("tracked_apps.enable_all");
    let tracked_apps_items = t("tracked_apps.items");
    let tracked_apps_enabled_status = t("tracked_apps.enabled_status");
    let ignored_addresses_title = t("ignored_addresses.title");
    let ignored_addresses_help = t("ignored_addresses.help");
    let ignored_addresses_empty = t("ignored_addresses.empty");
    let ignored_addresses_remove = t("ignored_addresses.remove");
    let dialog_delete_ignored_title = t("dialog.delete_ignored.title");
    let dialog_delete_ignored_help = t("dialog.delete_ignored.help");
    let dialog_delete_ignored_address = t("dialog.delete_ignored.address");
    let close_button_src = inline_svg_data_uri(CLOSE_TIMES_ICON_SVG);
    let module_stop_button_src = inline_svg_data_uri(MODULE_STOP_ICON_SVG);
    let module_close_button_src = inline_svg_data_uri(MODULE_CLOSE_ICON_SVG);
    let module_reorder_left_button_src = inline_svg_data_uri(MODULE_REORDER_LEFT_ICON_SVG);
    let help_icon_src = inline_svg_data_uri(HELP_CIRCLE_ICON_SVG);
    let confirm_filtered_button_src = inline_svg_data_uri(CONFIRM_FILTERED_ICON_SVG);
    let unconfirm_filtered_button_src = inline_svg_data_uri(UNCONFIRM_FILTERED_ICON_SVG);
    let ignore_address_button_src = inline_svg_data_uri(IGNORE_ADDRESS_ICON_SVG);
    let clear_monitoring_button_src = inline_svg_data_uri(CLEAR_MONITORING_ICON_SVG);
    let cloud_sync_button_src = inline_svg_data_uri(CLOUD_SYNC_ICON_SVG);
    let cloud_import_button_src = inline_svg_data_uri(CLOUD_IMPORT_ICON_SVG);
    let import_csv_button_src = inline_svg_data_uri(IMPORT_CSV_ICON_SVG);
    let export_csv_button_src = inline_svg_data_uri(EXPORT_CSV_ICON_SVG);
    let information_button_src = inline_svg_data_uri(HELP_INFO_ICON_SVG);
    let monitoring_button_src = inline_svg_data_uri(MONITORING_ICON_SVG);
    let sort_indicator_idle_src = inline_svg_data_uri(SORT_IDLE_ICON_SVG);
    let sort_indicator_asc_src = inline_svg_data_uri(SORT_ASC_ICON_SVG);
    let sort_indicator_desc_src = inline_svg_data_uri(SORT_DESC_ICON_SVG);
    let toggle_all_tooltip = if enable_all_overlay {
        t("switch.disable_all")
    } else {
        t("switch.enable_all")
    };
    let switch_enable_app = t("switch.enable_app");
    let switch_disable_app = t("switch.disable_app");
    let tracked_app_open_folder = t("tracked_app.open_folder");
    let tracked_app_delete = t("tracked_app.delete");
    let filter_all = t("filter.all");
    let filter_unconfirmed = t("filter.unconfirmed");
    let filter_confirmed = t("filter.confirmed");
    let filter_success = t("filter.success");
    let filter_failed = t("filter.failed");
    let filter_app = t("filter.app");
    let filter_port = t("filter.port");
    let filter_port_placeholder = t("filter.port_placeholder");
    let filter_protocol = t("filter.protocol");
    let filter_protocol_all = t("filter.protocol.all");
    let filter_search_placeholder = t("filter.search_placeholder");
    let filter_domain_placeholder = t("filter.domain_placeholder");
    let filter_domain_tooltip = t("filter.domain_tooltip");
    let action_start_monitoring = t("action.start_monitoring");
    let action_stop_monitoring = t("action.stop_monitoring");
    let action_confirm_filtered = t("action.confirm_filtered");
    let action_unconfirm_filtered = t("action.unconfirm_filtered");
    let action_clear_monitoring = t("action.clear_monitoring");
    let action_cloud_import = t("action.cloud_import");
    let action_cloud_export = t("action.cloud_export");
    let action_cloud_import_select_visible = t("action.cloud_import_select_visible");
    let action_cloud_import_clear_selection = t("action.cloud_import_clear_selection");
    let action_cloud_import_delete_rows = t("action.cloud_import_delete_rows");
    let action_import_csv = t("action.import_csv");
    let action_export_csv = t("action.export_csv");
    let action_information = t("action.information");
    let modules_background_running = t("modules.background_running");
    let modules_stop = t("modules.stop");
    let modules_stop_tooltip = t("modules.stop_tooltip");
    let action_clear_monitoring_event = action_clear_monitoring.clone();
    let observations_title = t("observations.title");
    let observations_subtitle = t("observations.subtitle");
    let observations_total = t("observations.total");
    let observations_displayed = t("observations.displayed");
    let observations_selected = t("observations.selected");
    let observations_public_ip = t("observations.public_ip");
    let observations_public_ip_tooltip = t("observations.public_ip_tooltip");
    let table_app = t("table.app");
    let table_ip = t("table.ip");
    let table_domain = t("table.domain");
    let table_domain_tooltip = t("table.domain_tooltip");
    let table_port = t("table.port");
    let table_proto = t("table.proto");
    let table_conn = t("table.conn");
    let table_conn_tooltip = t("table.conn_tooltip");
    let table_first_seen = t("table.first_seen");
    let table_last_seen = t("table.last_seen");
    let table_hits = t("table.hits");
    let table_hits_tooltip = t("table.hits_tooltip");
    let table_state = t("table.state");
    let table_action = t("table.action");
    let action_confirm = t("action.confirm");
    let action_unconfirm = t("action.unconfirm");
    let action_ignore_address = t("action.ignore_address");
    let action_delete_observation = t("action.delete_observation");
    let dialog_delete_observation_title = t("dialog.delete_observation.title");
    let dialog_delete_observation_help = t("dialog.delete_observation.help");
    let dialog_delete_observation_app = t("dialog.delete_observation.app");
    let dialog_delete_observation_endpoint = t("dialog.delete_observation.endpoint");
    let dialog_delete_observation_state = t("dialog.delete_observation.state");
    let dialog_clear_monitoring_title = t("dialog.clear_monitoring.title");
    let dialog_clear_monitoring_help = t("dialog.clear_monitoring.help");
    let dialog_clear_monitoring_all_count = t("dialog.clear_monitoring.all_count");
    let dialog_clear_monitoring_selected_count = t("dialog.clear_monitoring.selected_count");
    let dialog_clear_monitoring_all = t("dialog.clear_monitoring.all");
    let dialog_clear_monitoring_selected = t("dialog.clear_monitoring.selected");
    let dialog_cloud_import_clear_title = t("dialog.cloud_import_clear.title");
    let dialog_cloud_import_clear_help = t("dialog.cloud_import_clear.help");
    let dialog_cloud_import_clear_all_count = t("dialog.cloud_import_clear.all_count");
    let dialog_cloud_import_clear_selected_count = t("dialog.cloud_import_clear.selected_count");
    let dialog_cloud_import_clear_all = t("dialog.cloud_import_clear.all");
    let dialog_cloud_import_clear_selected = t("dialog.cloud_import_clear.selected");
    let enrichment_domain_label = t("enrichment.domain");
    let enrichment_owner_label = t("enrichment.owner");
    let enrichment_range_label = t("enrichment.range");
    let enrichment_registry_label = t("enrichment.registry");
    let enrichment_source_label = t("enrichment.source");
    let enrichment_unknown = t("enrichment.unknown");
    let enrichment_localhost_rule = t("enrichment.localhost_rule");
    let enrichment_local_ip_rule = t("enrichment.local_ip_rule");
    let integration_title = t("integration.title");
    let integration_subtitle = t("integration.subtitle");
    let integration_loaded_template = t("integration.loaded");
    let modules_order_label = t("modules.order");
    let modules_order_move_left = t("modules.order.move_left");
    let modules_order_move_right = t("modules.order.move_right");
    let integration_provider_label = t("integration.provider");
    let dialog_export_title = t("dialog.export.title");
    let dialog_export_help = t("dialog.export.help");
    let dialog_export_mode_attach = t("dialog.profile_export.mode_attach");
    let dialog_export_mode_patch = t("dialog.profile_export.mode_patch");
    let dialog_export_mode_merge = t("dialog.profile_export.mode_merge");
    let dialog_export_note_attach = t("dialog.profile_export.note_attach");
    let dialog_export_note_patch = t("dialog.profile_export.note_patch");
    let dialog_export_note_merge = t("dialog.profile_export.note_merge");
    let dialog_export_profile_path = t("dialog.export.profile_path");
    let dialog_export_generated_name = t("dialog.export.generated_name");
    let dialog_profile_export_available_profiles = t("dialog.profile_export.available_profiles");
    let dialog_profile_export_no_profiles = t("dialog.profile_export.no_profiles");
    let dialog_export_dangerous_confirm = t("dialog.export.dangerous_confirm");
    let dialog_export_analyze = t("dialog.profile_export.analyze");
    let dialog_export_apply = t("dialog.profile_export.apply");
    let dialog_export_backup = t("dialog.profile_export.backup");
    let dialog_export_revert = t("dialog.profile_export.revert");
    let dialog_export_revert_success = t("dialog.profile_export.revert_success");
    let dialog_export_close = t("dialog.profile_export.close");
    let dialog_profile_export_group_addresses = t("dialog.profile_export.group_addresses");
    let dialog_profile_export_group_domains = t("dialog.profile_export.group_domains");
    let dialog_profile_export_group_ranges = t("dialog.profile_export.group_ranges");
    let dialog_profile_export_selected = t("dialog.profile_export.selected");
    let dialog_profile_export_will_add = t("dialog.profile_export.will_add");
    let dialog_profile_export_skipped = t("dialog.profile_export.skipped");
    let dialog_profile_export_uncovered_title = t("dialog.profile_export.uncovered_title");
    let dialog_profile_export_uncovered_help = t("dialog.profile_export.uncovered_help");
    let dialog_profile_export_advanced_wizard = t("dialog.profile_export.advanced_wizard");
    let dialog_profile_export_advanced_wizard_title =
        t("dialog.profile_export.advanced_wizard_title");
    let dialog_profile_export_advanced_wizard_help =
        t("dialog.profile_export.advanced_wizard_help");
    let dialog_profile_export_advanced_wizard_empty =
        t("dialog.profile_export.advanced_wizard_empty");
    let dialog_profile_export_advanced_wizard_step_template =
        t("dialog.profile_export.advanced_wizard_step_template");
    let dialog_profile_export_advanced_create_rules =
        t("dialog.profile_export.advanced_create_rules");
    let dialog_profile_export_advanced_create_rules_help =
        t("dialog.profile_export.advanced_create_rules_help");
    let dialog_profile_export_advanced_exclude_ips =
        t("dialog.profile_export.advanced_exclude_ips");
    let dialog_profile_export_advanced_exclude_ips_help =
        t("dialog.profile_export.advanced_exclude_ips_help");
    let dialog_profile_export_advanced_whois_ranges =
        t("dialog.profile_export.advanced_whois_ranges");
    let dialog_profile_export_advanced_whois_ranges_help =
        t("dialog.profile_export.advanced_whois_ranges_help");
    let dialog_profile_export_advanced_domains = t("dialog.profile_export.advanced_domains");
    let dialog_profile_export_advanced_domains_help =
        t("dialog.profile_export.advanced_domains_help");
    let dialog_profile_export_advanced_manual_domains =
        t("dialog.profile_export.advanced_manual_domains");
    let dialog_profile_export_advanced_manual_domains_placeholder =
        t("dialog.profile_export.advanced_manual_domains_placeholder");
    let dialog_profile_export_advanced_add_domains =
        t("dialog.profile_export.advanced_add_domains");
    let dialog_profile_export_advanced_template_rule =
        t("dialog.profile_export.advanced_template_rule");
    let dialog_profile_export_advanced_apply = t("dialog.profile_export.advanced_apply");
    let dialog_profile_export_advanced_cancel = t("dialog.profile_export.advanced_cancel");
    let dialog_export_files = t("dialog.export.files");
    let dialog_export_warnings = t("dialog.export.warnings");
    let dialog_export_rules = t("dialog.export.rules");
    let dialog_export_no_files = t("dialog.export.no_files");
    let footer_apps = t("footer.apps");
    let footer_watcher_label = t("footer.watcher.label");
    let footer_tool_label = t("footer.tool.label");
    let footer_web_server_label = t("footer.web_server.label");
    let footer_web_server_copied_prefix = t("footer.web_server.copied_prefix");
    let footer_web_server_localhost_fallback = t("footer.web_server.localhost_fallback");
    let footer_network_label = t("footer.network.label");
    let footer_network_available_prefix = t("footer.network.available_prefix");
    let footer_network_unavailable = t("footer.network.unavailable");
    let footer_network_checking = t("footer.network.checking");
    let footer_network_true = t("footer.network.true");
    let footer_network_false = t("footer.network.false");
    let footer_network_unknown = t("footer.network.unknown");
    let status_build_version = t("status.build_version");
    let status_current = t("status.current");
    let status_current_app = t("status.current_app");
    let status_watcher = t("status.watcher");
    let status_tool = t("status.tool");
    let status_web_server = t("status.web_server");
    let status_dns = t("status.dns");
    let status_available = t("status.available");
    let status_unavailable = t("status.unavailable");
    let status_disabled = t("status.disabled");
    let status_waiting_for_watcher = t("status.waiting_for_watcher");
    let footer_message_label = t("footer.message.label");
    let footer_status_copy_tooltip = t("footer.status.copy_tooltip");
    let footer_status_copied = t("footer.status.copied");
    let footer_monitoring_started = t("footer.event.monitoring_started");
    let footer_monitoring_stopped = t("footer.event.monitoring_stopped");
    let footer_domain_capture_enabled = t("footer.event.domain_capture_enabled");
    let footer_domain_capture_disabled = t("footer.event.domain_capture_disabled");
    let footer_domain_capture_admin_required = t("footer.event.domain_capture_admin_required");
    let footer_domain_capture_failed = t("footer.event.domain_capture_failed");
    let footer_confirmed_filtered_prefix = t("footer.event.confirmed_filtered_prefix");
    let footer_export_completed = t("footer.event.export_completed");
    let footer_csv_export_empty = t("footer.event.csv_export_empty");
    let footer_csv_export_completed = t("footer.event.csv_export_completed");
    let footer_csv_export_failed = t("footer.event.csv_export_failed");
    let footer_csv_import_completed = t("footer.event.csv_import_completed");
    let footer_csv_import_failed = t("footer.event.csv_import_failed");
    let footer_csv_import_empty = t("footer.event.csv_import_empty");
    let footer_profile_export_analyzed = t("footer.event.profile_export_analyzed");
    let footer_profile_export_backup_created = t("footer.event.profile_export_backup_created");
    let footer_watcher_connected_event = t("footer.event.watcher_connected");
    let footer_watcher_disconnected_event = t("footer.event.watcher_disconnected");
    let footer_flow_capture = t("footer.flow_capture");
    let footer_flow_capture_flow_events = t("footer.flow_capture.flow_events");
    let footer_flow_capture_packet_events = t("footer.flow_capture.packet_events");
    let footer_flow_capture_emitted = t("footer.flow_capture.emitted");
    let footer_flow_capture_dropped = t("footer.flow_capture.dropped");
    let footer_connectors_loaded_prefix = t("footer.event.connectors_loaded_prefix");
    let footer_connectors_loaded_empty = t("footer.event.connectors_loaded_empty");
    let footer_connector_apps_suffix = t("footer.event.connector_apps_suffix");
    let footer_tracked_app_unavailable_prefix = t("footer.event.tracked_app_unavailable_prefix");
    let footer_module_connected_prefix = t("footer.event.module_connected_prefix");
    let footer_module_failed_prefix = t("footer.event.module_failed_prefix");
    let footer_language_label = t("footer.language.label");
    let dialog_integration_title = t("dialog.integration.title");
    let dialog_integration_help = t("dialog.integration.help");
    let dialog_integration_path_help = t("dialog.integration.path_help");
    let dialog_integration_primary_path = t("dialog.integration.primary_path");
    let dialog_integration_export_path = t("dialog.integration.export_path");
    let dialog_integration_status = t("dialog.integration.status");
    let dialog_integration_status_configured = t("dialog.integration.status_configured");
    let dialog_integration_status_missing = t("dialog.integration.status_missing");
    let dialog_integration_status_invalid = t("dialog.integration.status_invalid");
    let dialog_integration_status_entered = t("dialog.integration.status_entered");
    let dialog_integration_provider_unset = t("dialog.integration.provider_unset");
    let dialog_integration_download_progress = t("dialog.integration.download_progress");
    let dialog_integration_progress_kb = t("dialog.integration.progress_kb");
    let dialog_integration_progress_entries = t("dialog.integration.progress_entries");
    let dialog_integration_stage_preparing = t("dialog.integration.stage_preparing");
    let dialog_integration_stage_downloading = t("dialog.integration.stage_downloading");
    let dialog_integration_stage_extracting = t("dialog.integration.stage_extracting");
    let dialog_integration_stage_detecting = t("dialog.integration.stage_detecting");
    let dialog_integration_stage_installing = t("dialog.integration.stage_installing");
    let dialog_integration_stage_complete = t("dialog.integration.stage_complete");
    let dialog_integration_stage_failed = t("dialog.integration.stage_failed");
    let dialog_integration_stage_idle = t("dialog.integration.stage_idle");
    let dialog_integration_cancel_download = t("dialog.integration.cancel_download");
    let dialog_integration_browse = t("dialog.integration.browse");
    let dialog_integration_validation = t("dialog.integration.validation");
    let dialog_integration_validation_folder_missing =
        t("dialog.integration.validation_folder_missing");
    let dialog_integration_validation_folder_invalid =
        t("dialog.integration.validation_folder_invalid");
    let dialog_cancel = t("dialog.cancel");
    let dialog_ok = t("dialog.ok");
    let dialog_back = t("dialog.back");
    let dialog_close = t("dialog.close");
    let dialog_integration_use_folder = t("dialog.integration.use_folder");
    let dialog_delete_title = t("dialog.delete.title");
    let dialog_delete_help = t("dialog.delete.help");
    let dialog_delete_app = t("dialog.delete.app");
    let dialog_delete_path = t("dialog.delete.path");
    let dialog_no = t("dialog.no");
    let dialog_yes_delete = t("dialog.yes_delete");
    let dialog_info_title = t("dialog.info.title");
    let dialog_info_about_title = t("dialog.info.about_title");
    let dialog_info_info_title = t("dialog.info.info_title");
    let dialog_info_product_name = t("dialog.info.product_name");
    let dialog_info_author = t("dialog.info.author");
    let dialog_info_version = t("dialog.info.version");
    let dialog_info_email = t("dialog.info.email");
    let dialog_info_repository = t("dialog.info.repository");
    let dialog_info_update_program = t("dialog.info.update_program");
    let dialog_info_update_available = t("dialog.info.update_available");
    let dialog_info_info_footer = t("dialog.info.info_footer");
    let dialog_info_group_monitoring = t("dialog.info.group_monitoring");
    let dialog_info_monitoring_1 = t("dialog.info.monitoring_1");
    let dialog_info_monitoring_2 = t("dialog.info.monitoring_2");
    let dialog_info_monitoring_3 = t("dialog.info.monitoring_3");
    let dialog_info_monitoring_4 = t("dialog.info.monitoring_4");
    let dialog_info_monitoring_5 = t("dialog.info.monitoring_5");
    let dialog_info_group_domains = t("dialog.info.group_domains");
    let dialog_info_domains_1 = t("dialog.info.domains_1");
    let dialog_info_domains_2 = t("dialog.info.domains_2");
    let dialog_info_domains_4 = t("dialog.info.domains_4");
    let dialog_info_group_csv = t("dialog.info.group_csv");
    let dialog_info_csv_1 = t("dialog.info.csv_1");
    let dialog_info_csv_2 = t("dialog.info.csv_2");
    let dialog_info_group_tracked_apps = t("dialog.info.group_tracked_apps");
    let dialog_info_tracked_apps_1 = t("dialog.info.tracked_apps_1");
    let dialog_info_group_ignored_addresses = t("dialog.info.group_ignored_addresses");
    let dialog_info_ignored_addresses_1 = t("dialog.info.ignored_addresses_1");
    let dialog_info_ignored_addresses_2 = t("dialog.info.ignored_addresses_2");
    let dialog_info_group_cloud_sync = t("dialog.info.group_cloud_sync");
    let dialog_info_cloud_sync_1 = t("dialog.info.cloud_sync_1");
    let dialog_info_cloud_sync_2 = t("dialog.info.cloud_sync_2");
    let dialog_info_cloud_sync_3 = t("dialog.info.cloud_sync_3");
    let dialog_info_group_web_access = t("dialog.info.group_web_access");
    let dialog_info_web_access_1 = t("dialog.info.web_access_1");
    let dialog_info_web_access_2 = t("dialog.info.web_access_2");
    let dialog_info_web_access_3 = t("dialog.info.web_access_3");
    let dialog_info_group_modules = t("dialog.info.group_modules");
    let dialog_info_modules_1 = t("dialog.info.modules_1");
    let dialog_info_modules_2 = t("dialog.info.modules_2");
    let dialog_info_modules_3 = t("dialog.info.modules_3");
    let dialog_info_close = t("dialog.info.close");
    let dialog_cloud_sync_download_title = t("dialog.cloud_sync.download_title");
    let dialog_cloud_sync_upload_title = t("dialog.cloud_sync.upload_title");
    let dialog_cloud_sync_identifier = t("dialog.cloud_sync.identifier");
    let dialog_cloud_sync_identifier_pending = t("dialog.cloud_sync.identifier_pending");
    let dialog_cloud_sync_identifier_copy_tooltip = t("dialog.cloud_sync.identifier_copy_tooltip");
    let dialog_cloud_sync_identifier_copied = t("dialog.cloud_sync.identifier_copied");
    let dialog_cloud_sync_upload = t("dialog.cloud_sync.upload");
    let dialog_cloud_sync_download = t("dialog.cloud_sync.download");
    let dialog_cloud_sync_upload_action = t("dialog.cloud_sync.upload_action");
    let dialog_cloud_sync_download_action = t("dialog.cloud_sync.download_action");
    let dialog_cloud_sync_upload_limit_tooltip = t("dialog.cloud_sync.upload_limit_tooltip");
    let dialog_cloud_sync_upload_limit_tooltip_fresh =
        t("dialog.cloud_sync.upload_limit_tooltip_fresh");
    let dialog_cloud_sync_download_limit_tooltip = t("dialog.cloud_sync.download_limit_tooltip");
    let dialog_cloud_sync_download_limit_tooltip_fresh =
        t("dialog.cloud_sync.download_limit_tooltip_fresh");
    let dialog_cloud_sync_not_checked = t("dialog.cloud_sync.not_checked");
    let dialog_cloud_sync_app_search = t("dialog.cloud_sync.app_search");
    let dialog_cloud_sync_publisher_search = t("dialog.cloud_sync.publisher_search");
    let _dialog_cloud_sync_ip_search = t("dialog.cloud_sync.ip_search");
    let _dialog_cloud_sync_domain_search = t("dialog.cloud_sync.domain_search");
    let _dialog_cloud_sync_port_search = t("dialog.cloud_sync.port_search");
    let _dialog_cloud_sync_protocol = t("dialog.cloud_sync.protocol");
    let dialog_cloud_sync_source = t("dialog.cloud_sync.source");
    let dialog_cloud_sync_source_search = t("dialog.cloud_sync.source_search");
    let dialog_cloud_sync_company = t("dialog.cloud_sync.company");
    let dialog_cloud_sync_available_rows = t("dialog.cloud_sync.available_rows");
    let dialog_cloud_sync_stats = t("dialog.cloud_sync.stats");
    let dialog_cloud_sync_nickname = t("dialog.cloud_sync.nickname");
    let dialog_cloud_sync_private_upload = t("dialog.cloud_sync.private_upload");
    let dialog_cloud_sync_private_upload_tooltip = t("dialog.cloud_sync.private_upload_tooltip");
    let dialog_cloud_sync_visibility_scope = t("dialog.cloud_sync.visibility_scope");
    let dialog_cloud_sync_visibility_all = t("dialog.cloud_sync.visibility_all");
    let dialog_cloud_sync_visibility_public = t("dialog.cloud_sync.visibility_public");
    let dialog_cloud_sync_visibility_private = t("dialog.cloud_sync.visibility_private");
    let dialog_cloud_sync_privacy_column = t("dialog.cloud_sync.privacy_column");
    let dialog_cloud_sync_privacy_column_tooltip = t("dialog.cloud_sync.privacy_column_tooltip");
    let dialog_cloud_sync_nickname_invalid = t("dialog.cloud_sync.nickname_invalid");
    let dialog_cloud_sync_nickname_taken = t("dialog.cloud_sync.nickname_taken");
    let dialog_cloud_sync_nickname_accepted = t("dialog.cloud_sync.nickname_accepted");
    let _dialog_cloud_sync_select_all_visible = t("dialog.cloud_sync.select_all");
    let _dialog_cloud_sync_clear_selection = t("dialog.cloud_sync.clear_selection");
    let dialog_cloud_sync_select_row = t("dialog.cloud_sync.select_row");
    let dialog_cloud_sync_unselect_row = t("dialog.cloud_sync.unselect_row");
    let dialog_cloud_sync_select_row_tooltip = t("dialog.cloud_sync.select_row_tooltip");
    let dialog_cloud_sync_unselect_row_tooltip = t("dialog.cloud_sync.unselect_row_tooltip");
    let dialog_cloud_sync_add_to_monitoring = t("dialog.cloud_sync.add_to_monitoring");
    let dialog_cloud_sync_import_done_detail = t("dialog.cloud_sync.import_done_detail");
    let dialog_cloud_sync_export_selected = t("dialog.cloud_sync.export_selected");
    let dialog_cloud_sync_no_downloaded_rows = t("dialog.cloud_sync.no_downloaded_rows");
    let dialog_cloud_sync_apps_not_loaded = t("dialog.cloud_sync.apps_not_loaded");
    let dialog_cloud_sync_apps_empty = t("dialog.cloud_sync.apps_empty");
    let dialog_cloud_sync_authorization = t("dialog.cloud_sync.authorization");
    let dialog_cloud_sync_authorization_active = t("dialog.cloud_sync.authorization_active");
    let dialog_cloud_sync_authorization_tooltip = t("dialog.cloud_sync.authorization_tooltip");
    let dialog_cloud_sync_sign_out = t("dialog.cloud_sync.sign_out");
    let dialog_cloud_sync_signed_out = t("dialog.cloud_sync.signed_out");
    let dialog_cloud_sync_sign_in_started = t("dialog.cloud_sync.sign_in_started");
    let dialog_cloud_sync_upload_started = t("dialog.cloud_sync.upload_started");
    let dialog_cloud_sync_download_started = t("dialog.cloud_sync.download_started");
    let dialog_cloud_sync_scope_mine = t("dialog.cloud_sync.scope_mine");
    let dialog_cloud_sync_my_apps_not_loaded = t("dialog.cloud_sync.my_apps_not_loaded");
    let dialog_cloud_sync_publications = t("dialog.cloud_sync.publications");
    let dialog_cloud_sync_new_rows = t("dialog.cloud_sync.new_rows");
    let dialog_cloud_sync_non_public_rows = t("dialog.cloud_sync.non_public_rows");
    let dialog_cloud_sync_non_public_rows_tooltip = t("dialog.cloud_sync.non_public_rows_tooltip");
    let dialog_cloud_sync_author_rows = t("dialog.cloud_sync.author_rows");
    let dialog_cloud_sync_total_rows = t("dialog.cloud_sync.total_rows");
    let dialog_cloud_sync_status_ok = t("dialog.cloud_sync.status_ok");
    let dialog_cloud_sync_status_failed = t("dialog.cloud_sync.status_failed");
    let dialog_cloud_sync_refresh_done = t("dialog.cloud_sync.refresh_done");
    let dialog_cloud_sync_sign_in_done = t("dialog.cloud_sync.sign_in_done");
    let dialog_cloud_sync_credentials_save_failed = t("dialog.cloud_sync.credentials_save_failed");
    let dialog_cloud_sync_rows = t("dialog.cloud_sync.message_rows");
    let dialog_cloud_sync_requests = t("dialog.cloud_sync.message_requests");
    let dialog_cloud_sync_skipped_non_public = t("dialog.cloud_sync.skipped_non_public");
    let cloud_message_labels = CloudMessageLabels {
        refresh_done: dialog_cloud_sync_refresh_done.clone(),
        download: dialog_cloud_sync_download.clone(),
        upload: dialog_cloud_sync_upload.clone(),
        rows: dialog_cloud_sync_rows,
        requests: dialog_cloud_sync_requests,
        skipped_non_public: dialog_cloud_sync_skipped_non_public,
    };
    let dialog_domain_capture_admin_title = t("dialog.domain_capture_admin.title");
    let dialog_domain_capture_admin_help = t("dialog.domain_capture_admin.help");
    let dialog_domain_capture_admin_close = t("dialog.domain_capture_admin.close");
    let app_logo_src = APP_LOGO_DATA_URI.as_str();
    let app_author = APP_AUTHOR;
    let app_email = APP_EMAIL;
    let app_email_href = APP_EMAIL_HREF;
    let repository_url = APP_REPOSITORY_URL;
    let app_update_state = app_update_check();
    let app_update_available = app_update_state.update_available;
    let app_update_url = app_update_state
        .latest_url
        .clone()
        .unwrap_or_else(|| APP_RELEASES_URL.to_string());
    let information_button_class = if app_update_available {
        "input-box button button--icon header-action-button header-action-button--update-available"
    } else {
        "input-box button button--icon header-action-button"
    };
    let information_button_tooltip = if app_update_available {
        if let Some(latest_version) = app_update_state.latest_version.as_ref() {
            format!("{action_information}: {dialog_info_update_available} v{latest_version}")
        } else {
            format!("{action_information}: {dialog_info_update_available}")
        }
    } else {
        action_information.clone()
    };
    let cloud_status = cloud_state();
    let cloud_identifier_value = if cloud_status.client_identifier.is_empty() {
        dialog_cloud_sync_identifier_pending.clone()
    } else {
        cloud_status.client_identifier.clone()
    };
    let cloud_identifier_copy_value = cloud_status.client_identifier.clone();
    let cloud_identifier_tooltip = if cloud_identifier_copy_value.is_empty() {
        dialog_cloud_sync_identifier_pending.clone()
    } else {
        format!(
            "{} {}",
            dialog_cloud_sync_identifier_copy_tooltip, cloud_identifier_copy_value
        )
    };
    let cloud_availability_value = match cloud_status.service_available {
        Some(true) => dialog_cloud_sync_status_ok.clone(),
        Some(false) => dialog_cloud_sync_status_failed.clone(),
        None => dialog_cloud_sync_not_checked.clone(),
    };
    let cloud_availability_indicator_class = match cloud_status.service_available {
        Some(true) => {
            "footer-watcher-status__indicator footer-watcher-status__indicator--connected"
        }
        Some(false) => {
            "footer-watcher-status__indicator footer-watcher-status__indicator--disconnected"
        }
        None => "footer-watcher-status__indicator footer-watcher-status__indicator--inactive",
    };
    let cloud_quota_hours_suffix = if selected_language().starts_with("ru") {
        "ч"
    } else {
        "h"
    };
    let cloud_upload_quota_value = quota_label(cloud_status.upload_quota.as_ref());
    let cloud_download_quota_value = quota_label(cloud_status.download_quota.as_ref());
    let cloud_upload_quota_exhausted = quota_is_exhausted(cloud_status.upload_quota.as_ref());
    let cloud_upload_quota_value_class =
        cloud_quota_value_class(cloud_status.upload_quota.as_ref());
    let cloud_download_quota_value_class =
        cloud_quota_value_class(cloud_status.download_quota.as_ref());
    let cloud_upload_quota_tooltip = cloud_quota_tooltip(
        cloud_status.upload_quota.as_ref(),
        &dialog_cloud_sync_upload_limit_tooltip,
        &dialog_cloud_sync_upload_limit_tooltip_fresh,
        cloud_quota_hours_suffix,
    );
    let cloud_download_quota_tooltip = cloud_quota_tooltip(
        cloud_status.download_quota.as_ref(),
        &dialog_cloud_sync_download_limit_tooltip,
        &dialog_cloud_sync_download_limit_tooltip_fresh,
        cloud_quota_hours_suffix,
    );
    let cloud_author_filter_display = cloud_status
        .my_apps
        .iter()
        .filter_map(|app| app.author_signature.as_deref())
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| validate_author_signature(&cloud_upload_nickname()).ok())
        .unwrap_or_else(|| cloud_source_search().trim().to_string());
    let cloud_upload_nickname_valid = validate_author_signature(&cloud_upload_nickname()).is_ok();
    let cloud_nickname_status = cloud_nickname_check_status();
    let cloud_upload_nickname_blocks_upload = cloud_nickname_status.blocks_upload();
    let cloud_upload_nickname_status_label = match cloud_nickname_status {
        CloudNicknameCheckStatus::Invalid => dialog_cloud_sync_nickname_invalid.clone(),
        CloudNicknameCheckStatus::Taken => dialog_cloud_sync_nickname_taken.clone(),
        CloudNicknameCheckStatus::Accepted => dialog_cloud_sync_nickname_accepted.clone(),
        _ => String::new(),
    };
    let cloud_upload_nickname_status_class = match cloud_nickname_status {
        CloudNicknameCheckStatus::Invalid => {
            "cloud-sync-nickname-status cloud-sync-nickname-status--invalid"
        }
        CloudNicknameCheckStatus::Taken => {
            "cloud-sync-nickname-status cloud-sync-nickname-status--taken"
        }
        CloudNicknameCheckStatus::Accepted => {
            "cloud-sync-nickname-status cloud-sync-nickname-status--accepted"
        }
        _ => "cloud-sync-nickname-status",
    };
    let cloud_author_publication_rows = build_cloud_author_publication_rows(
        &snapshot,
        &cloud_status.my_apps,
        &selected_observation_ids,
        &cloud_status.uploaded_observation_ids,
        if cloud_upload_private() {
            CloudObservationVisibility::Private
        } else {
            CloudObservationVisibility::Public
        },
        cloud_publication_sort(),
    );
    let cloud_upload_has_candidates = cloud_author_publication_rows
        .iter()
        .any(|row| row.new_rows > 0);
    let cloud_session_login = cloud_status
        .session
        .as_ref()
        .map(|session| session.login.clone());
    let cloud_authorization_value = cloud_status
        .session
        .as_ref()
        .and_then(|session| {
            session
                .email
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    session
                        .display_name
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| {
                    let login = session.login.trim();
                    (!login.is_empty() && !login.eq_ignore_ascii_case("google")).then_some(login)
                })
                .map(str::to_string)
        })
        .or_else(|| {
            cloud_status
                .session
                .is_some()
                .then(|| dialog_cloud_sync_authorization_active.clone())
        })
        .unwrap_or_else(|| "-".to_string());
    let cloud_apps = sort_cloud_apps(
        cloud_apps_for_visibility_scope(
            cloud_status.apps.clone(),
            &cloud_status.my_apps,
            cloud_visibility_scope_filter(),
            &cloud_app_search(),
            &cloud_publisher_search(),
            &cloud_source_search(),
            &cloud_author_filter_display,
            cloud_scope_mine(),
        ),
        cloud_app_sort(),
    );
    let cloud_downloaded_rows = sort_cloud_downloaded_rows(
        filter_cloud_downloaded_rows(
            cloud_status.downloaded_rows.clone(),
            &cloud_app_search(),
            &cloud_ip_search(),
            &cloud_domain_search(),
            &cloud_port_search(),
            &cloud_protocol_filter(),
            cloud_row_status_filter(),
            &cloud_status.selected_download_row_ids,
        ),
        cloud_row_sort(),
    );
    let cloud_downloaded_rows_for_selection = Arc::new(cloud_downloaded_rows.clone());
    let cloud_base_url = default_cloud_base_url();
    let cloud_credentials_save_failed_for_google =
        dialog_cloud_sync_credentials_save_failed.clone();
    let integration_progress_labels = IntegrationProgressLabels {
        kb: dialog_integration_progress_kb.clone(),
        entries: dialog_integration_progress_entries.clone(),
        preparing: dialog_integration_stage_preparing.clone(),
        downloading: dialog_integration_stage_downloading.clone(),
        extracting: dialog_integration_stage_extracting.clone(),
        detecting: dialog_integration_stage_detecting.clone(),
        installing: dialog_integration_stage_installing.clone(),
        complete: dialog_integration_stage_complete.clone(),
        failed: dialog_integration_stage_failed.clone(),
        idle: dialog_integration_stage_idle.clone(),
    };
    let integration_repo_path_for_prompt = snapshot.integration.repo_path.clone();
    let web_server_url = browser_ui_url();
    let web_server_log_url_text = web_server_log_url(&web_server_url);
    let monitoring_action_label = if monitoring_active_now {
        action_stop_monitoring.clone()
    } else {
        action_start_monitoring.clone()
    };
    let watcher_connection_tooltip = with_flow_capture_status(
        availability_status_line(
            &status_watcher,
            snapshot.ui.watcher_connected,
            &web_server_log_url_text,
            &status_available,
            &status_unavailable,
        ),
        &footer_flow_capture,
        &footer_flow_capture_flow_events,
        &footer_flow_capture_packet_events,
        &footer_flow_capture_emitted,
        &footer_flow_capture_dropped,
        &snapshot.runtime_status.flow_capture,
    );
    let tool_connection_tooltip = availability_status_line(
        &status_tool,
        tool_available,
        "netstitch-tool native library",
        &status_available,
        &status_unavailable,
    );
    let watcher_module = module_version_detail("watcher", "embedded watcher");
    let web_server_available =
        snapshot.app_settings.web_access_localhost && snapshot.ui.watcher_connected;
    let footer_status_ready = snapshot.ui.snapshot_loaded;
    let web_server_tooltip = web_server_status_line(
        &status_web_server,
        snapshot.app_settings.web_access_localhost,
        snapshot.ui.watcher_connected,
        &module_status_detail(&web_server_log_url_text, &watcher_module),
        &status_available,
        &status_unavailable,
        &status_disabled,
    );
    let network_probe_available = snapshot
        .runtime_status
        .endpoint_probe
        .first_successful_target
        .is_some();
    let network_probe_checking = !network_probe_available
        && (snapshot.runtime_status.endpoint_probe.is_checking || !snapshot.ui.watcher_connected);
    let network_probe_tooltip = endpoint_probe_tooltip(
        &snapshot.runtime_status.endpoint_probe,
        snapshot.ui.watcher_connected,
        &footer_network_available_prefix,
        &footer_network_unavailable,
        &footer_network_checking,
        &status_waiting_for_watcher,
        &footer_network_true,
        &footer_network_false,
        &footer_network_unknown,
    );
    let web_server_copied_message =
        web_server_event_line(&footer_web_server_copied_prefix, &web_server_url);
    let web_server_url_for_footer = web_server_url.clone();
    let footer_web_server_localhost_fallback_for_footer =
        footer_web_server_localhost_fallback.clone();
    let copy_button_src = inline_svg_data_uri(COPY_ICON_SVG);
    let footer_status_history = status_history();
    let footer_status_text = footer_message_text(&footer_status_history);
    let footer_status_tooltip = footer_message_tooltip(&footer_status_history);
    let footer_status_text_class = format!(
        "footer-message-text path-field{}",
        footer_message_class_suffix(&footer_status_history)
    );
    let clipboard_window = window.clone();
    let cloud_identifier_clipboard_window = window.clone();
    let status_history_window = window.clone();
    let integration_dialog_preview = integration_dialog_preview(
        integration_path_applied().trim(),
        &snapshot.integration,
        &dialog_integration_status_configured,
        &dialog_integration_status_missing,
        &dialog_integration_status_invalid,
        &dialog_integration_status_entered,
        &dialog_integration_provider_unset,
    );
    let integration_module_buttons = snapshot
        .integration_modules
        .iter()
        .filter(|module| module.enabled)
        .cloned()
        .collect::<Vec<_>>();
    let integration_module_buttons =
        ordered_integration_modules(integration_module_buttons, &module_order());
    let integration_modules_ready = !integration_module_buttons.is_empty();
    let integration_modules_loaded = integration_loaded_template
        .replace("{count}", &integration_module_buttons.len().to_string());
    let integration_modules_bootstrap =
        !snapshot.ui.snapshot_loaded && integration_module_buttons.is_empty();
    let tracked_apps_bootstrap = !snapshot.ui.snapshot_loaded && snapshot.tracked_apps.is_empty();
    let selected_integration_module = selected_integration_module_id()
        .and_then(|selected_id| {
            integration_module_buttons
                .iter()
                .find(|module| module.id == selected_id)
                .cloned()
        })
        .or_else(|| integration_module_buttons.first().cloned());
    let integration_module_dialog_title = selected_integration_module
        .as_ref()
        .map(|module| module.display_name.clone())
        .unwrap_or_else(|| integration_title.clone());
    let integration_module_dialog_help = selected_integration_module
        .as_ref()
        .map(|module| {
            if module.tooltip.trim().is_empty() {
                module.display_name.clone()
            } else {
                module.tooltip.clone()
            }
        })
        .unwrap_or_else(|| integration_subtitle.clone());
    let modules_order_switch_class = if module_order_editing() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let integration_provider_buttons = snapshot.integration_providers.clone();
    let default_profile_export_path_value = default_profile_export_path(&snapshot.integration);
    let profile_export_profile_options =
        profile_export_profile_options(&snapshot.integration, &default_profile_export_path_value);
    let profile_export_mode_value = profile_export_mode();
    let profile_export_path_input_value = match profile_export_mode_value {
        ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
        ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
        ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
    };
    let profile_export_selected_profile_value = if profile_export_path_input_value.trim().is_empty()
    {
        default_profile_export_path_value.clone()
    } else {
        profile_export_path_input_value.clone()
    };
    let profile_export_attach_class =
        if profile_export_mode_value == ExportModeDto::AttachNetstitchLists {
            "tabs__tab tabs__tab--active"
        } else {
            "tabs__tab"
        };
    let profile_export_patch_class =
        if profile_export_mode_value == ExportModeDto::PatchSelectedProfile {
            "tabs__tab tabs__tab--active"
        } else {
            "tabs__tab"
        };
    let profile_export_merge_class =
        if profile_export_mode_value == ExportModeDto::MergeIntoExistingLists {
            "tabs__tab tabs__tab--active"
        } else {
            "tabs__tab"
        };
    let profile_export_merge_blocked = profile_export_mode_value
        == ExportModeDto::MergeIntoExistingLists
        && !profile_export_dangerous_confirmed();
    let profile_export_preview_value = if profile_export_merge_blocked {
        None
    } else {
        profile_export_preview()
    };
    let profile_export_dangerous_switch_class = if profile_export_dangerous_confirmed() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let profile_export_advanced_create_rules_class = if profile_export_advanced_create_rules() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let profile_export_advanced_exclude_ips_class = if profile_export_advanced_exclude_ips() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let profile_export_advanced_whois_ranges_class = if profile_export_advanced_whois_ranges() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let profile_export_advanced_domains_class = if profile_export_advanced_domains() {
        "input-box switch switch--on"
    } else {
        "input-box switch"
    };
    let integration_download_state = integration_download();
    let integration_download_percent = integration_download_state.visual_percent();
    let integration_download_meta =
        integration_download_state.progress_meta(&integration_progress_labels);
    let integration_download_cancel_disabled = !integration_download_state.active;
    let ignored_addresses_loading =
        !snapshot.ui.watcher_connected && snapshot.ignored_addresses.is_empty();
    let integration_progress_stages = vec![
        ProgressStage::new(
            dialog_integration_stage_downloading.clone(),
            75,
            "progress-bar__segment--accent",
        ),
        ProgressStage::new(
            dialog_integration_stage_extracting.clone(),
            25,
            "progress-bar__segment--success",
        ),
    ];

    use_effect({
        let snapshot = snapshot.clone();
        let mut module_order = module_order;
        move || {
            let saved_order = snapshot.app_settings.module_order.clone();
            if saved_order.is_empty() || module_order() == saved_order {
                return;
            }
            module_order.set(saved_order);
        }
    });

    use_effect({
        let mut watcher = watcher;
        let pending_selection_confirm = pending_observation_selection_confirm.clone();
        let mut profile_export_preview = profile_export_preview;
        let profile_export_manual_plan_key = profile_export_manual_plan_key;
        let mut profile_export_feedback = profile_export_feedback;
        let mut status_history = status_history;
        move || {
            if !show_profile_export_prompt() {
                return;
            }

            if profile_export_mode() == ExportModeDto::MergeIntoExistingLists
                && !profile_export_dangerous_confirmed()
            {
                profile_export_preview.set(None);
                profile_export_feedback.set(None);
                return;
            }

            flush_observation_selection_confirm(&mut watcher.write(), &pending_selection_confirm);
            let integration = watcher.read().snapshot().integration.clone();
            let selected_profile_path = match profile_export_mode() {
                ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
                ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
                ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
            };
            match profile_export_request(
                &integration,
                profile_export_mode(),
                &selected_profile_path,
                &profile_export_generated_name(),
                profile_export_dangerous_confirmed(),
            ) {
                Ok(request) => {
                    let module_id = selected_integration_module_id();
                    if profile_export_manual_plan_key().as_deref()
                        == Some(profile_export_request_key(&request).as_str())
                    {
                        return;
                    }
                    match watcher.write().preview_profile_export(module_id, request) {
                        Ok(plan) => {
                            profile_export_preview.set(Some(plan));
                            profile_export_feedback.set(None);
                        }
                        Err(message) => {
                            profile_export_preview.set(None);
                            profile_export_feedback.set(None);
                            push_status_history_line(status_history, message);
                        }
                    }
                }
                Err(message) => {
                    profile_export_preview.set(None);
                    profile_export_feedback.set(None);
                    push_status_history_line(status_history, message);
                }
            }
        }
    });

    use_effect({
        let mut status_history = status_history;
        let mut last_domain_capture_error = last_domain_capture_error;
        let snapshot = snapshot.clone();
        let footer_domain_capture_failed = footer_domain_capture_failed.clone();
        let footer_domain_capture_admin_required = footer_domain_capture_admin_required.clone();
        move || {
            let error = snapshot
                .runtime_status
                .domain_capture
                .backend_error
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_default()
                .to_string();
            if !snapshot.app_settings.domain_capture_enabled || error.is_empty() {
                last_domain_capture_error.set(String::new());
                return;
            }
            if last_domain_capture_error() == error {
                return;
            }
            last_domain_capture_error.set(error.clone());
            let suffix = if snapshot.runtime_status.is_elevated {
                error
            } else {
                format!("{error}; {footer_domain_capture_admin_required}")
            };
            push_status_history_line(
                status_history,
                format!("{footer_domain_capture_failed}: {suffix}"),
            );
        }
    });

    use_effect({
        let watcher = watcher;
        let mut status_history = status_history;
        let mut last_domain_capture_admin_rejected = last_domain_capture_admin_rejected;
        let mut show_domain_capture_admin_prompt = show_domain_capture_admin_prompt;
        let footer_domain_capture_admin_required = footer_domain_capture_admin_required.clone();
        move || {
            let snapshot = watcher.read().snapshot();
            if snapshot.ui.snapshot_loaded && snapshot.runtime_status.domain_capture_admin_disabled
            {
                if !last_domain_capture_admin_rejected() {
                    show_domain_capture_admin_prompt.set(true);
                    push_status_history_line(
                        status_history,
                        footer_domain_capture_admin_required.clone(),
                    );
                }
                last_domain_capture_admin_rejected.set(true);
                return;
            }
            if snapshot.ui.snapshot_loaded && last_domain_capture_admin_rejected() {
                last_domain_capture_admin_rejected.set(false);
            }
        }
    });

    use_effect({
        let watcher = watcher;
        let mut status_history = status_history;
        let mut last_watcher_connected = last_watcher_connected;
        let footer_watcher_connected_event = footer_watcher_connected_event.clone();
        let footer_watcher_disconnected_event = footer_watcher_disconnected_event.clone();
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        move || {
            let _ = monitoring_ui_refresh_nonce();
            let connected = watcher.read().snapshot().ui.watcher_connected;
            let previous = last_watcher_connected();
            if previous == Some(connected) {
                return;
            }

            if let Some(was_connected) = previous {
                let line = if connected && !was_connected {
                    Some(footer_watcher_connected_event.clone())
                } else if !connected && was_connected {
                    Some(footer_watcher_disconnected_event.clone())
                } else {
                    None
                };
                if let Some(line) = line {
                    push_status_history_line(status_history, line);
                }
            } else if connected {
                push_status_history_line(status_history, footer_watcher_connected_event.clone());
            }

            last_watcher_connected.set(Some(connected));
        }
    });

    use_effect({
        let watcher = watcher;
        let mut status_history = status_history;
        let mut last_build_status_event = last_build_status_event;
        let mut last_current_status_event = last_current_status_event;
        let mut last_app_status_event = last_app_status_event;
        let mut last_watcher_status_event = last_watcher_status_event;
        let mut last_tool_status_event = last_tool_status_event;
        let mut last_web_server_status_event = last_web_server_status_event;
        let mut last_dns_status_event = last_dns_status_event;
        let status_build_version = status_build_version.clone();
        let status_current = status_current.clone();
        let status_current_app = status_current_app.clone();
        let status_watcher = status_watcher.clone();
        let status_tool = status_tool.clone();
        let status_web_server = status_web_server.clone();
        let status_dns = status_dns.clone();
        let status_available = status_available.clone();
        let status_unavailable = status_unavailable.clone();
        let status_disabled = status_disabled.clone();
        let status_waiting_for_watcher = status_waiting_for_watcher.clone();
        let footer_network_checking = footer_network_checking.clone();
        let web_server_url = web_server_url.clone();
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        move || {
            let _ = monitoring_ui_refresh_nonce();
            let snapshot = watcher.read().snapshot();
            if !snapshot.ui.snapshot_loaded {
                return;
            }
            let watcher_base_url = watcher
                .read()
                .live_base_url()
                .unwrap_or_else(|| "https://127.0.0.1:46473".to_string());

            let build_line = format!(
                "{}: NetStitch v{}",
                status_build_version,
                runtime_build_version()
            );
            if last_build_status_event().as_deref() != Some(build_line.as_str()) {
                push_status_history_line(status_history, build_line.clone());
                last_build_status_event.set(Some(build_line));
            }

            let current_line = format!("{}: {}", status_current, snapshot.ui.status_text);
            if last_current_status_event().as_deref() != Some(current_line.as_str()) {
                push_status_history_line(status_history, current_line.clone());
                last_current_status_event.set(Some(current_line));
            }

            let app_line = format!(
                "{}: {}",
                status_current_app,
                tracked_apps_status_text(&snapshot)
            );
            if last_app_status_event().as_deref() != Some(app_line.as_str()) {
                push_status_history_line(status_history, app_line.clone());
                last_app_status_event.set(Some(app_line));
            }

            let watcher_module = module_version_detail("watcher", "embedded watcher");
            let watcher_line = availability_status_line(
                &status_watcher,
                snapshot.ui.watcher_connected,
                &module_status_detail(watcher_base_url.trim_end_matches('/'), &watcher_module),
                &status_available,
                &status_unavailable,
            );
            if last_watcher_status_event().as_deref() != Some(watcher_line.as_str()) {
                push_status_history_line(status_history, watcher_line.clone());
                last_watcher_status_event.set(Some(watcher_line));
            }

            let tool_line = availability_status_line(
                &status_tool,
                snapshot.runtime_status.tool_available,
                &module_version_detail("tool", "netstitch-tool native"),
                &status_available,
                &status_unavailable,
            );
            if last_tool_status_event().as_deref() != Some(tool_line.as_str()) {
                push_status_history_line(status_history, tool_line.clone());
                last_tool_status_event.set(Some(tool_line));
            }

            let web_line = web_server_status_line(
                &status_web_server,
                snapshot.app_settings.web_access_localhost,
                snapshot.ui.watcher_connected,
                &module_status_detail(web_server_log_url(&web_server_url), &watcher_module),
                &status_available,
                &status_unavailable,
                &status_disabled,
            );
            if last_web_server_status_event().as_deref() != Some(web_line.as_str()) {
                push_status_history_line(status_history, web_line.clone());
                last_web_server_status_event.set(Some(web_line));
            }

            let dns_line = dns_status_line(
                &status_dns,
                &snapshot.runtime_status.endpoint_probe,
                snapshot.ui.watcher_connected,
                &status_available,
                &status_unavailable,
                &footer_network_checking,
                &status_waiting_for_watcher,
                &module_version_detail("tool", "netstitch-tool native"),
            );
            if last_dns_status_event().as_deref() != Some(dns_line.as_str()) {
                push_status_history_line(status_history, dns_line.clone());
                last_dns_status_event.set(Some(dns_line));
            }
        }
    });

    use_effect({
        let watcher = watcher;
        let mut status_history = status_history;
        let mut initial_connector_events_seeded = initial_connector_events_seeded;
        let footer_connectors_loaded_prefix = footer_connectors_loaded_prefix.clone();
        let footer_connectors_loaded_empty = footer_connectors_loaded_empty.clone();
        let footer_connector_apps_suffix = footer_connector_apps_suffix.clone();
        let footer_tracked_app_unavailable_prefix = footer_tracked_app_unavailable_prefix.clone();
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        move || {
            let _ = monitoring_ui_refresh_nonce();
            if initial_connector_events_seeded() {
                return;
            }

            let snapshot = watcher.read().snapshot();
            if !snapshot.ui.watcher_connected || snapshot.tracked_apps.is_empty() {
                return;
            }

            push_status_history_line(
                status_history,
                connector_loaded_status_line(
                    snapshot.runtime_status.connector_apps_detected,
                    &footer_connectors_loaded_prefix,
                    &footer_connectors_loaded_empty,
                    &footer_connector_apps_suffix,
                ),
            );
            for app in &snapshot.runtime_status.unavailable_tracked_apps {
                push_status_history_line(
                    status_history,
                    tracked_app_availability_line(
                        &footer_tracked_app_unavailable_prefix,
                        &app.display_name,
                        &app.exe_path,
                    ),
                );
            }
            initial_connector_events_seeded.set(true);
        }
    });

    use_effect({
        let watcher = watcher;
        let mut status_history = status_history;
        let mut module_status_event_keys = module_status_event_keys;
        let footer_module_connected_prefix = footer_module_connected_prefix.clone();
        let footer_module_failed_prefix = footer_module_failed_prefix.clone();
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        move || {
            let _ = monitoring_ui_refresh_nonce();
            let snapshot = watcher.read().snapshot();
            if !snapshot.ui.watcher_connected || !snapshot.ui.snapshot_loaded {
                return;
            }
            if snapshot.runtime_status.integration_modules.is_empty() {
                return;
            }

            let mut shown_keys = module_status_event_keys();
            let mut changed = false;
            for module in &snapshot.runtime_status.integration_modules {
                let key = integration_module_status_key(module);
                if shown_keys.iter().any(|shown| shown == &key) {
                    continue;
                }
                let line = integration_module_status_line(
                    module,
                    &footer_module_connected_prefix,
                    &footer_module_failed_prefix,
                );
                if module.connected {
                    push_status_history_line(status_history, line);
                } else {
                    push_status_history_error_line(status_history, line);
                }
                shown_keys.push(key);
                changed = true;
            }
            if changed {
                module_status_event_keys.set(shown_keys);
            }
        }
    });

    use_effect({
        let tray = tray.clone();
        let watcher = watcher;
        let mut hide_when_minimized = hide_when_minimized;
        let mut remember_window_placement = remember_window_placement;
        let mut window_visible = window_visible;
        move || {
            let snapshot = watcher.read().snapshot();
            let remember_enabled = snapshot.app_settings.remember_window_placement;
            if remember_window_placement() != remember_enabled {
                remember_window_placement.set(remember_enabled);
            }
            let hide_enabled = snapshot.app_settings.hide_when_minimized;
            if hide_when_minimized() != hide_enabled {
                hide_when_minimized.set(hide_enabled);
            }
            tray.sync(
                snapshot.ui.monitoring,
                hide_enabled,
                snapshot.app_settings.web_access_localhost,
                remember_enabled,
                window_visible(),
            );
        }
    });

    use_effect({
        let watcher = watcher;
        let language_catalog = language_catalog.clone();
        let mut selected_language = selected_language;
        move || {
            let snapshot = watcher.read().snapshot();
            let Some(saved_language) = snapshot.app_settings.language_code.as_deref() else {
                return;
            };
            let resolved_language = language_catalog.preferred_language_code(Some(saved_language));
            if selected_language() != resolved_language {
                selected_language.set(resolved_language);
            }
        }
    });

    use_effect({
        let mut integration_download = integration_download;
        let mut integration_prompt_was_open = integration_prompt_was_open;
        let mut integration_path_applied = integration_path_applied;
        move || {
            let is_open = show_integration_prompt();
            let was_open = integration_prompt_was_open();
            if is_open && !was_open {
                integration_download.set(IntegrationDownloadUiState::default());
                integration_path_applied.set(integration_path_input().trim().to_string());
            }
            if was_open != is_open {
                integration_prompt_was_open.set(is_open);
            }
        }
    });

    desktop::use_tray_menu_event_handler({
        let window = window.clone();
        let tray = tray.clone();
        let watcher = watcher;
        let show_integration_prompt = show_integration_prompt;
        let integration_path_input = integration_path_input;
        let integration_feedback = integration_feedback;
        let pending_scan_after_setup = pending_scan_after_setup;
        let hide_when_minimized = hide_when_minimized;
        let remember_window_placement = remember_window_placement;
        let remembered_window_placement = remembered_window_placement;
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        let window_visible = window_visible;
        let hidden_window_was_maximized = hidden_window_was_maximized;
        let mut last_tray_menu_event = last_tray_menu_event;
        let integration_folder_dialog_open = integration_folder_dialog_open;
        move |event| {
            let id = event.id().as_ref().to_string();
            if should_skip_duplicate_tray_menu_event(last_tray_menu_event, &id) {
                return;
            }
            handle_tray_menu_action(
                tray.action_from_menu_id(event.id()),
                &window,
                &tray,
                watcher,
                show_integration_prompt,
                integration_path_input,
                integration_feedback,
                pending_scan_after_setup,
                hide_when_minimized,
                remember_window_placement,
                remembered_window_placement,
                monitoring_ui_refresh_nonce,
                window_visible,
                hidden_window_was_maximized,
                integration_folder_dialog_open,
            );
        }
    });

    desktop::use_muda_event_handler({
        let window = window.clone();
        let tray = tray.clone();
        let watcher = watcher;
        let show_integration_prompt = show_integration_prompt;
        let integration_path_input = integration_path_input;
        let integration_feedback = integration_feedback;
        let pending_scan_after_setup = pending_scan_after_setup;
        let hide_when_minimized = hide_when_minimized;
        let remember_window_placement = remember_window_placement;
        let remembered_window_placement = remembered_window_placement;
        let monitoring_ui_refresh_nonce = monitoring_ui_refresh_nonce;
        let window_visible = window_visible;
        let hidden_window_was_maximized = hidden_window_was_maximized;
        let mut last_tray_menu_event = last_tray_menu_event;
        let integration_folder_dialog_open = integration_folder_dialog_open;
        move |event| {
            let id = event.id().as_ref().to_string();
            if should_skip_duplicate_tray_menu_event(last_tray_menu_event, &id) {
                return;
            }
            handle_tray_menu_action(
                tray_menu_action_from_id(event.id().as_ref()),
                &window,
                &tray,
                watcher,
                show_integration_prompt,
                integration_path_input,
                integration_feedback,
                pending_scan_after_setup,
                hide_when_minimized,
                remember_window_placement,
                remembered_window_placement,
                monitoring_ui_refresh_nonce,
                window_visible,
                hidden_window_was_maximized,
                integration_folder_dialog_open,
            );
        }
    });

    desktop::use_tray_icon_event_handler({
        let window = window.clone();
        let mut window_visible = window_visible;
        let remember_window_placement = remember_window_placement;
        let mut remembered_window_placement = remembered_window_placement;
        let hidden_window_was_maximized = hidden_window_was_maximized;
        move |event| match event {
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                restore_window(
                    &window,
                    hidden_window_was_maximized(),
                    remember_window_placement(),
                    &mut remembered_window_placement,
                );
                window_visible.set(true);
            }
            _ => {}
        }
    });

    desktop::use_wry_event_handler({
        let window = window.clone();
        let mut watcher = watcher;
        let hide_when_minimized = hide_when_minimized;
        let remember_window_placement = remember_window_placement;
        let mut remembered_window_placement = remembered_window_placement;
        let mut window_visible = window_visible;
        let mut hidden_window_was_maximized = hidden_window_was_maximized;
        let show_integration_module_prompt = show_integration_module_prompt;
        let show_integration_prompt = show_integration_prompt;
        move |event, _| {
            if let Event::WindowEvent { event, .. } = event {
                if !show_integration_module_prompt()
                    && !show_integration_prompt()
                    && !show_cloud_sync_prompt()
                    && hide_when_minimized()
                    && window.is_minimized()
                    && window_visible()
                {
                    hide_window(
                        &window,
                        &mut hidden_window_was_maximized,
                        &mut remembered_window_placement,
                    );
                    window_visible.set(false);
                }

                if remember_window_placement()
                    && matches!(&event, WindowEvent::Moved(_) | WindowEvent::Resized(_))
                    && window.is_visible()
                    && !window.is_minimized()
                {
                    if let Ok(placement) = capture_current_window_placement(&window, false) {
                        remembered_window_placement.set(Some(placement));
                    }
                }

                if matches!(&event, WindowEvent::CloseRequested) {
                    exit_app(
                        &window,
                        &mut watcher,
                        remember_window_placement(),
                        !window_visible(),
                        remembered_window_placement(),
                    );
                }

                if matches!(&event, WindowEvent::Destroyed) {
                    window_visible.set(false);
                }
            }
        }
    });

    let executable_picker_window = window.clone();
    let integration_picker_window = window.clone();
    let profile_export_picker_window = window.clone();
    let profile_export_picker_window_attach = profile_export_picker_window.clone();
    let profile_export_picker_window_patch = profile_export_picker_window.clone();
    let profile_export_picker_window_merge = profile_export_picker_window.clone();
    let csv_import_picker_window = window.clone();
    let csv_export_picker_window = window.clone();
    let cloud_csv_export_picker_window = csv_export_picker_window.clone();
    let cloud_csv_export_completed = footer_csv_export_completed.clone();
    let cloud_csv_export_failed = footer_csv_export_failed.clone();
    let cloud_refresh_done_for_import_open = dialog_cloud_sync_refresh_done.clone();
    let cloud_refresh_done_for_export_open = dialog_cloud_sync_refresh_done.clone();
    let cloud_message_labels_for_import_open = cloud_message_labels.clone();
    let cloud_message_labels_for_export_open = cloud_message_labels.clone();

    use_future({
        let cloud_state = cloud_state;
        let status_history = status_history;
        let show_cloud_sync_prompt = show_cloud_sync_prompt;
        let cloud_app_search = cloud_app_search;
        let cloud_publisher_search = cloud_publisher_search;
        let cloud_ip_search = cloud_ip_search;
        let cloud_domain_search = cloud_domain_search;
        let cloud_port_search = cloud_port_search;
        let cloud_protocol_filter = cloud_protocol_filter;
        let cloud_source_search = cloud_source_search;
        let cloud_selected_app_id = cloud_selected_app_id;
        let cloud_scope_mine = cloud_scope_mine;
        let cloud_visibility_scope_filter = cloud_visibility_scope_filter;
        let refresh_done_message = dialog_cloud_sync_refresh_done.clone();
        let cloud_message_labels = cloud_message_labels.clone();
        move || {
            let refresh_done_message = refresh_done_message.clone();
            let cloud_message_labels = cloud_message_labels.clone();
            async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    if show_cloud_sync_prompt() {
                        start_cloud_refresh(
                            cloud_state,
                            status_history,
                            current_cloud_filters(
                                &cloud_app_search,
                                &cloud_publisher_search,
                                &cloud_ip_search,
                                &cloud_domain_search,
                                &cloud_port_search,
                                &cloud_protocol_filter,
                                &cloud_source_search,
                                &cloud_selected_app_id,
                                &cloud_scope_mine,
                                &cloud_visibility_scope_filter,
                            ),
                            refresh_done_message.clone(),
                            cloud_message_labels.clone(),
                            false,
                        );
                    }
                }
            }
        }
    });

    use_future({
        let cloud_state = cloud_state;
        let status_history = status_history;
        let show_cloud_sync_prompt = show_cloud_sync_prompt;
        let cloud_app_search = cloud_app_search;
        let cloud_publisher_search = cloud_publisher_search;
        let cloud_ip_search = cloud_ip_search;
        let cloud_domain_search = cloud_domain_search;
        let cloud_port_search = cloud_port_search;
        let cloud_protocol_filter = cloud_protocol_filter;
        let cloud_source_search = cloud_source_search;
        let cloud_selected_app_id = cloud_selected_app_id;
        let cloud_scope_mine = cloud_scope_mine;
        let cloud_visibility_scope_filter = cloud_visibility_scope_filter;
        let cloud_filter_generation = cloud_filter_generation;
        let refresh_done_message = dialog_cloud_sync_refresh_done.clone();
        let cloud_message_labels = cloud_message_labels.clone();
        move || {
            let refresh_done_message = refresh_done_message.clone();
            let cloud_message_labels = cloud_message_labels.clone();
            async move {
                let mut seen_generation = cloud_filter_generation();
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    let current_generation = cloud_filter_generation();
                    if current_generation != seen_generation {
                        seen_generation = current_generation;
                        if show_cloud_sync_prompt() {
                            start_cloud_refresh(
                                cloud_state,
                                status_history,
                                current_cloud_filters(
                                    &cloud_app_search,
                                    &cloud_publisher_search,
                                    &cloud_ip_search,
                                    &cloud_domain_search,
                                    &cloud_port_search,
                                    &cloud_protocol_filter,
                                    &cloud_source_search,
                                    &cloud_selected_app_id,
                                    &cloud_scope_mine,
                                    &cloud_visibility_scope_filter,
                                ),
                                refresh_done_message.clone(),
                                cloud_message_labels.clone(),
                                false,
                            );
                        }
                    }
                }
            }
        }
    });

    use_future({
        let cloud_state = cloud_state;
        let show_cloud_sync_prompt = show_cloud_sync_prompt;
        let cloud_upload_nickname = cloud_upload_nickname;
        let cloud_nickname_check_status = cloud_nickname_check_status;
        let cloud_nickname_check_generation = cloud_nickname_check_generation;
        move || async move {
            let mut seen_generation = cloud_nickname_check_generation();
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                let current_generation = cloud_nickname_check_generation();
                if current_generation != seen_generation {
                    seen_generation = current_generation;
                    if show_cloud_sync_prompt() {
                        start_cloud_nickname_check(
                            cloud_nickname_check_status,
                            cloud_nickname_check_generation,
                            cloud_state(),
                            cloud_upload_nickname(),
                            current_generation,
                        );
                    }
                }
            }
        }
    });

    let clear_monitoring_total_count = snapshot.observations.len();
    let clear_monitoring_selected_count = selected_observation_ids.len();
    let cloud_import_clear_total_count = cloud_status.downloaded_rows.len();
    let cloud_import_clear_selected_count = cloud_status.selected_download_row_ids.len();
    let clear_all_selection_store = observation_selection_store.clone();
    let clear_all_event_label = action_clear_monitoring_event.clone();
    let clear_selected_event_label = action_clear_monitoring_event.clone();
    let cloud_import_clear_all_event_label = action_cloud_import_delete_rows.clone();
    let cloud_import_clear_selected_event_label = action_cloud_import_delete_rows.clone();
    let clear_dialog_title = if cloud_import_active {
        dialog_cloud_import_clear_title.clone()
    } else {
        dialog_clear_monitoring_title.clone()
    };
    let clear_dialog_help = if cloud_import_active {
        dialog_cloud_import_clear_help.clone()
    } else {
        dialog_clear_monitoring_help.clone()
    };
    let clear_dialog_total_label = if cloud_import_active {
        dialog_cloud_import_clear_all_count.clone()
    } else {
        dialog_clear_monitoring_all_count.clone()
    };
    let clear_dialog_selected_label = if cloud_import_active {
        dialog_cloud_import_clear_selected_count.clone()
    } else {
        dialog_clear_monitoring_selected_count.clone()
    };
    let clear_dialog_total_count = if cloud_import_active {
        cloud_import_clear_total_count
    } else {
        clear_monitoring_total_count
    };
    let clear_dialog_selected_count = if cloud_import_active {
        cloud_import_clear_selected_count
    } else {
        clear_monitoring_selected_count
    };
    let clear_dialog_all_button_label = if cloud_import_active {
        dialog_cloud_import_clear_all.clone()
    } else {
        dialog_clear_monitoring_all.clone()
    };
    let clear_dialog_selected_button_label = if cloud_import_active {
        dialog_cloud_import_clear_selected.clone()
    } else {
        dialog_clear_monitoring_selected.clone()
    };
    let header_confirm_filtered_label = if cloud_import_active {
        action_cloud_import_select_visible.clone()
    } else {
        action_confirm_filtered.clone()
    };
    let header_unconfirm_filtered_label = if cloud_import_active {
        action_cloud_import_clear_selection.clone()
    } else {
        action_unconfirm_filtered.clone()
    };
    let header_clear_rows_label = if cloud_import_active {
        action_cloud_import_delete_rows.clone()
    } else {
        action_clear_monitoring.clone()
    };
    let header_filter_context_tooltip = |label: &str| {
        if cloud_import_active {
            format!("{action_cloud_import}: {label}")
        } else {
            label.to_string()
        }
    };
    let header_app_filter_value = if cloud_import_active {
        cloud_app_search()
    } else {
        snapshot.filters.app_search.clone()
    };
    let header_ip_filter_value = if cloud_import_active {
        cloud_ip_search_draft()
    } else {
        monitoring_ip_filter_draft()
    };
    let header_domain_filter_value = if cloud_import_active {
        cloud_domain_search_draft()
    } else {
        monitoring_domain_filter_draft()
    };
    let header_port_filter_value = if cloud_import_active {
        cloud_port_search_draft()
    } else {
        monitoring_port_filter_draft()
    };
    let header_protocol_filter_value = if cloud_import_active {
        match cloud_protocol_filter().trim().to_ascii_lowercase().as_str() {
            "tcp" => "TCP".to_string(),
            "udp" => "UDP".to_string(),
            "other" => "OTHER".to_string(),
            _ => "All".to_string(),
        }
    } else {
        snapshot.filters.protocol.clone()
    };
    let header_status_filter_value = if cloud_import_active {
        cloud_row_status_filter().as_str().to_string()
    } else {
        snapshot.filters.observation_filter.as_str().to_string()
    };
    let header_filter_app_tooltip = header_filter_context_tooltip(&filter_app);
    let header_filter_ip_tooltip = header_filter_context_tooltip(&table_ip);
    let header_filter_domain_tooltip = header_filter_context_tooltip(&filter_domain_tooltip);
    let header_filter_port_tooltip = header_filter_context_tooltip(&filter_port);
    let header_filter_protocol_tooltip = header_filter_context_tooltip(&filter_protocol);
    let header_filter_status_tooltip = header_filter_context_tooltip(&table_state);
    let header_ip_filter_class = if input_apply_pulse() == Some("header-ip") {
        "input-box field header-search-field input--apply-pulse"
    } else {
        "input-box field header-search-field"
    };
    let header_domain_filter_class = if input_apply_pulse() == Some("header-domain") {
        "input-box field header-search-field header-search-field--domain input--apply-pulse"
    } else {
        "input-box field header-search-field header-search-field--domain"
    };
    let header_port_filter_class = if input_apply_pulse() == Some("header-port") {
        "input-box field header-search-field header-search-field--port input--apply-pulse"
    } else {
        "input-box field header-search-field header-search-field--port"
    };
    let clear_header_ip_filter_disabled = header_ip_filter_value.is_empty();
    let clear_header_domain_filter_disabled = header_domain_filter_value.is_empty();
    let clear_header_port_filter_disabled = header_port_filter_value.is_empty();
    let pending_exe_path_value = pending_exe_path_draft();
    let clear_pending_exe_path_disabled = pending_exe_path_value.is_empty();
    let pending_exe_path_class = if input_apply_pulse() == Some("tracked-app-path") {
        "input-box input input--apply-pulse"
    } else {
        "input-box input"
    };
    let clear_cloud_app_search_disabled = cloud_app_search_draft().is_empty();
    let clear_cloud_publisher_search_disabled = cloud_publisher_search_draft().is_empty();
    let clear_cloud_source_search_disabled = cloud_source_search_draft().is_empty();
    let cloud_app_search_class = if input_apply_pulse() == Some("cloud-app") {
        "input-box input input--apply-pulse"
    } else {
        "input-box input"
    };
    let cloud_publisher_search_class = if input_apply_pulse() == Some("cloud-company") {
        "input-box input input--apply-pulse"
    } else {
        "input-box input"
    };
    let cloud_source_search_class = if input_apply_pulse() == Some("cloud-author") {
        "input-box input input--apply-pulse"
    } else {
        "input-box input"
    };
    let integration_path_input_class = if input_apply_pulse() == Some("integration-path") {
        "input-box input input--apply-pulse"
    } else {
        "input-box input"
    };
    let clear_integration_path_disabled = integration_path_input().is_empty();
    let cloud_my_publications_active = cloud_scope_mine();
    let cloud_my_publications_button_class = if cloud_my_publications_active {
        "button button--icon header-action-button button--cloud-active cloud-sync-my-publications-button"
    } else {
        "button button--icon header-action-button cloud-sync-my-publications-button"
    };
    let my_publications_button_src = inline_svg_data_uri(MY_PUBLICATIONS_ICON_SVG);
    let cloud_message_labels_for_google_auth = cloud_message_labels.clone();
    let cloud_message_labels_for_upload_action = cloud_message_labels.clone();
    let module_shell_active = show_integration_module_prompt()
        || (show_integration_prompt() && return_integration_prompt_to_module())
        || (show_profile_export_prompt() && return_profile_export_to_module())
        || (show_profile_export_advanced_wizard() && return_profile_export_to_module());
    let module_header_title = integration_module_dialog_title.clone();
    let module_header_actions = selected_integration_module
        .as_ref()
        .map(|module| {
            let module_latest_rows =
                module_context_latest_rows(module.background_active, &snapshot.observations);
            let module_latest_rows_count = module_context_latest_rows_count(
                module.background_active,
                &snapshot.observations,
                5,
            );
            let module_last_row =
                module_context_last_row(module.background_active, &snapshot.observations);
            module
                .header_actions
                .iter()
                .cloned()
                .map(|mut action| {
                    action.label = module_ui_context_value(
                        &action.label,
                        selected_observation_ids.len(),
                        visible_observations.len(),
                        snapshot.observations.len(),
                        module.background_active,
                        &module_last_row,
                        &module_latest_rows,
                        module_latest_rows_count,
                    );
                    action.tooltip = action.tooltip.map(|value| {
                        module_ui_context_value(
                            &value,
                            selected_observation_ids.len(),
                            visible_observations.len(),
                            snapshot.observations.len(),
                            module.background_active,
                            &module_last_row,
                            &module_latest_rows,
                            module_latest_rows_count,
                        )
                    });
                    action
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let integration_root_dialog_title = if return_integration_prompt_to_module() {
        selected_integration_module
            .as_ref()
            .and_then(|module| module.provider_title.clone())
            .unwrap_or_else(|| dialog_integration_title.clone())
    } else {
        dialog_integration_title.clone()
    };
    let profile_export_dialog_title = if return_profile_export_to_module() {
        selected_integration_module
            .as_ref()
            .and_then(|module| module.export_title.clone())
            .unwrap_or_else(|| dialog_export_title.clone())
    } else {
        dialog_export_title.clone()
    };

    rsx! {
        style { "{theme::GLOBAL_STYLE}" }
        main {
            id: ui::id::MAIN,
            "data-ui-entity": ui::entity::APP_ROOT,
            div {
                id: ui::id::SHELL,
                class: "{shell_class}",
                "data-ui-entity": ui::entity::SHELL,
                "data-ui-disabled": "{ui_controls_disabled}",
                "data-cloud-overlay-active": "{cloud_overlay_active}",
                header {
                    id: ui::id::APP_HEADER,
                    class: "app-chrome app-chrome--header",
                    "data-ui-entity": ui::entity::APP_HEADER,
                    if module_shell_active {
                        div {
                            class: "module-header-content",
                            "data-ui-entity": ui::entity::MODULE_HEADER,
                            div { class: "module-header-title", "{module_header_title}" }
                            div {
                                class: "module-header-controls",
                                div {
                                    id: ui::id::MODULE_HEADER_ACTIONS,
                                    class: "module-header-actions",
                                    "data-ui-entity": ui::entity::MODULE_HEADER_ACTIONS,
                                    for action in module_header_actions.iter() {
                                        {
                                            let action_id = action.id.clone();
                                            let action_label = action.label.clone();
                                            let action_tooltip = action
                                                .tooltip
                                                .clone()
                                                .filter(|value| !value.trim().is_empty())
                                                .unwrap_or_else(|| action_label.clone());
                                            let action_icon_src = module_action_icon_src(action);
                                            let module_title_for_action = module_header_title.clone();
                                            let action_label_for_click = action_label.clone();
                                            let module_id_for_action = selected_integration_module_id();
                                            let module_for_host_commands = selected_integration_module.clone();
                                            let module_background_active_for_action =
                                                selected_integration_module
                                                    .as_ref()
                                                    .is_some_and(|module| module.background_active);
                                            let selected_ids_for_action = selected_observation_ids
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();
                                            let displayed_ids_for_action = visible_observations
                                                .iter()
                                                .map(|observation| observation.id)
                                                .collect::<Vec<_>>();
                                            let filters_for_action =
                                                shared_filters_from_snapshot(&snapshot);
                                            let window_for_action = window.clone();
                                            let action_pulse_class = module_action_pulse_class(
                                                action,
                                                selected_integration_module
                                                    .as_ref()
                                                    .is_some_and(|module| module.background_active),
                                            );
                                            rsx! {
                                                button {
                                                    id: "netstitch-ui-module-header-action-{action_id}",
                                                    class: "input-box button button--icon header-action-button {action_pulse_class}",
                                                    r#type: "button",
                                                    disabled: !action.enabled,
                                                    "data-ui-action": "module-header-action",
                                                    "data-ui-key": "{action_id}",
                                                    "aria-label": "{action_tooltip}",
                                                    "data-tooltip": "{action_tooltip}",
                                                    "data-tooltip-align": "end",
                                                    onclick: move |_| {
                                                        let Some(module_id) = module_id_for_action.clone() else {
                                                            push_status_history_line(
                                                                status_history,
                                                                format!("{module_title_for_action}: {action_label_for_click}"),
                                                            );
                                                            return;
                                                        };
                                                        let Some(module_for_host_commands) = module_for_host_commands.clone() else {
                                                            push_status_history_line(
                                                                status_history,
                                                                format!("{module_title_for_action}: {action_label_for_click}"),
                                                            );
                                                            return;
                                                        };
                                                        start_module_ui_action(
                                                            watcher,
                                                            status_history,
                                                            module_host_dialog,
                                                            module_ui_page,
                                                            module_ui_values,
                                                            module_ui_action_generation,
                                                            module_path_picker_dialog_open,
                                                            window_for_action.clone(),
                                                            module_for_host_commands,
                                                            module_title_for_action.clone(),
                                                            action_label_for_click.clone(),
                                                            IntegrationModuleUiActionClientRequestDto {
                                                                module_id,
                                                                action_id: action_id.clone(),
                                                                ui_action_token: String::new(),
                                                                selected_monitoring_row_ids: selected_ids_for_action.clone(),
                                                                displayed_monitoring_row_ids: displayed_ids_for_action.clone(),
                                                                filters: filters_for_action.clone(),
                                                                payload: module_ui_action_payload(
                                                                    module_ui_values,
                                                                    module_background_active_for_action,
                                                                ),
                                                            },
                                                        );
                                                    },
                                                    if let Some(icon_src) = action_icon_src.clone() {
                                                        img {
                                                            class: "button__icon button__icon--module-action",
                                                            src: "{icon_src}",
                                                            alt: ""
                                                        }
                                                    } else {
                                                        span {
                                                            class: "button__icon-fallback button__icon-fallback--module-action",
                                                            "{module_action_fallback_label(&action_label)}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "header-action-separator", "aria-hidden": "true" }
                                div {
                                    class: "module-header-standard-actions",
                                    button {
                                        id: "netstitch-ui-module-header-stop-button",
                                        class: "input-box button button--icon header-action-button",
                                        r#type: "button",
                                        "data-ui-action": "stop-module-background",
                                        "aria-label": "{modules_stop}",
                                        "data-tooltip": "{modules_stop_tooltip}",
                                        "data-tooltip-align": "end",
                                        onclick: move |_| {
                                            module_ui_action_generation.set(module_ui_action_generation().wrapping_add(1));
                                            if let Some(module_id) = selected_integration_module_id() {
                                                match watcher.write().stop_integration_module_background(module_id.clone()) {
                                                    Ok(stopped) => {
                                                        let message = if stopped {
                                                            format!("{module_header_title}: {modules_stop}")
                                                        } else {
                                                            format!("{module_header_title}: listener already stopped")
                                                        };
                                                        push_status_history_line(status_history, message);
                                                    }
                                                    Err(message) => push_status_history_line(
                                                        status_history,
                                                        format!("{module_header_title}: {message}"),
                                                    ),
                                                }
                                            }
                                            show_profile_export_advanced_wizard.set(false);
                                            show_profile_export_prompt.set(false);
                                            show_integration_prompt.set(false);
                                            show_integration_module_prompt.set(false);
                                            return_integration_prompt_to_module.set(false);
                                            return_profile_export_to_module.set(false);
                                            profile_export_manual_plan_key.set(None);
                                            selected_integration_module_id.set(None);
                                            module_ui_page.set("main".to_string());
                                            module_host_dialog.set(None);
                                        },
                                        img {
                                            class: "button__icon button__icon--module-stop",
                                            src: "{module_stop_button_src}",
                                            alt: ""
                                        }
                                    }
                                    button {
                                        id: ui::id::MODULE_HEADER_CLOSE_BUTTON,
                                        class: "input-box button button--icon header-action-button",
                                        r#type: "button",
                                        "data-ui-action": ui::action::CLOSE_MODULE_OVERLAYS,
                                        "aria-label": "{dialog_close}",
                                        "data-tooltip": "{dialog_close}",
                                        "data-tooltip-align": "end",
                                        onclick: move |_| {
                                            module_ui_action_generation.set(module_ui_action_generation().wrapping_add(1));
                                            show_profile_export_advanced_wizard.set(false);
                                            show_profile_export_prompt.set(false);
                                            show_integration_prompt.set(false);
                                            show_integration_module_prompt.set(false);
                                            return_integration_prompt_to_module.set(false);
                                            return_profile_export_to_module.set(false);
                                            profile_export_manual_plan_key.set(None);
                                            selected_integration_module_id.set(None);
                                            module_ui_page.set("main".to_string());
                                            module_host_dialog.set(None);
                                        },
                                        img {
                                            class: "button__icon button__icon--module-close",
                                            src: "{module_close_button_src}",
                                            alt: ""
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: if module_shell_active { "hero-labels hero-labels--hidden" } else { "hero-labels" },
                        div {
                            class: "header-monitor-controls header-system-tools",
                            "data-ui-alias": "system-tools",
                            button {
                                id: ui::id::TOGGLE_MONITORING_BUTTON,
                                class: if monitoring_active_now {
                                    "input-box button button--icon button--monitoring button--monitoring-active header-action-button"
                                } else {
                                    "input-box button button--icon button--monitoring header-action-button"
                                },
                                "data-ui-action": ui::action::TOGGLE_MONITORING,
                                "aria-label": "{monitoring_action_label}",
                                "data-tooltip": "{monitoring_action_label}",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    let started = toggle_monitoring(&mut watcher);
                                    if started {
                                        push_status_history_line(
                                            status_history,
                                            footer_monitoring_started.clone(),
                                        );
                                    } else {
                                        push_status_history_line(
                                            status_history,
                                            footer_monitoring_stopped.clone(),
                                        );
                                    }
                                },
                                span {
                                    class: "button__icon-shell button__icon-shell--monitoring",
                                    span {
                                        class: "button__icon-scale button__icon-scale--monitoring",
                                        span {
                                            class: "button__icon-rotor button__icon-rotor--monitoring",
                                            img {
                                                class: "button__icon button__icon--monitoring",
                                                src: "{monitoring_button_src}",
                                                alt: ""
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "header-action-separator", "aria-hidden": "true" }
                            div { class: "header-filter-block header-filter-block--app",
                                span { class: "header-filter-block__label", "{table_app}" }
                                select {
                                    id: ui::id::OBSERVATION_APP_FILTER_INPUT,
                                    class: "input-box select header-filter-select header-filter-select--app",
                                    "data-ui-entity": ui::control::OBSERVATION_APP_FILTER_INPUT,
                                    value: "{header_app_filter_value}",
                                    "aria-label": "{filter_app}",
                                    "data-tooltip": "{header_filter_app_tooltip}",
                                    "data-tooltip-align": "end",
                                    onchange: move |event| {
                                        let value = event.value().to_string();
                                        if cloud_import_active {
                                            cloud_app_search_draft.set(value.clone());
                                            cloud_app_search.set(value);
                                            cloud_selected_app_id.set(None);
                                            cloud_loading_app_id.set(None);
                                            cloud_filter_generation.set(
                                                cloud_filter_generation().wrapping_add(1),
                                            );
                                        } else {
                                            let mut current = watcher.write();
                                            let next = current.snapshot();
                                            current.set_filters(crate::watcher_api::SetFilterRequest {
                                                app_search: value,
                                                search_text: next.filters.search_text,
                                                domain_search: next.filters.domain_search,
                                                port_search: next.filters.port_search,
                                                protocol: next.filters.protocol,
                                                public_ip: next.filters.public_ip,
                                                observation_filter: next.filters.observation_filter,
                                            });
                                        }
                                    },
                                    option { value: "", "{filter_all}" }
                                    for app_name in app_filter_options.iter() {
                                        option { value: "{app_name}", "{app_name}" }
                                    }
                                }
                            }
                            div { class: "header-filter-block header-filter-block--ip",
                                span { class: "header-filter-block__label", "{table_ip}" }
                                div { class: "path-input-shell header-filter-field-shell",
                                    input {
                                        id: ui::id::OBSERVATION_SEARCH_INPUT,
                                        class: "{header_ip_filter_class}",
                                        "data-ui-entity": ui::control::OBSERVATION_SEARCH_INPUT,
                                        r#type: "text",
                                        placeholder: "{filter_search_placeholder}",
                                        "data-committed-value": "{header_ip_filter_value}",
                                        "data-commit-on-enter": "true",
                                        "data-clear-button": "true",
                                        "data-preserve-draft": "true",
                                        "aria-label": "{header_filter_ip_tooltip}",
                                        "data-tooltip": "{header_filter_ip_tooltip}",
                                        "data-tooltip-align": "end",
                                        onchange: move |event| {
                                            let value = event.value().to_string();
                                            pulse_text_input(input_apply_pulse, "header-ip");
                                            if cloud_import_active {
                                                cloud_ip_search_draft.set(value.clone());
                                                cloud_ip_search.set(value);
                                            } else {
                                                monitoring_ip_filter_draft.set(value.clone());
                                                monitoring_ip_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: value,
                                                    domain_search: next.filters.domain_search,
                                                    port_search: next.filters.port_search,
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        onkeydown: move |event| {
                                            if event.key() != Key::Enter {
                                                return;
                                            }
                                            pulse_text_input(input_apply_pulse, "header-ip");
                                        }
                                    }
                                    button {
                                        id: ui::id::CLEAR_OBSERVATION_SEARCH_BUTTON,
                                        class: "path-input-clear",
                                        r#type: "button",
                                        disabled: clear_header_ip_filter_disabled,
                                        "data-clear-button": "true",
                                        "data-ui-action": ui::action::CLEAR_OBSERVATION_SEARCH,
                                        "aria-label": "{input_clear}",
                                        "data-tooltip": "{input_clear}",
                                        "data-tooltip-align": "end",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            if cloud_import_active {
                                                cloud_ip_search_draft.set(String::new());
                                                cloud_ip_search.set(String::new());
                                            } else {
                                                monitoring_ip_filter_draft.set(String::new());
                                                monitoring_ip_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: String::new(),
                                                    domain_search: next.filters.domain_search,
                                                    port_search: next.filters.port_search,
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                    }
                                }
                            }
                            div { class: "header-filter-block header-filter-block--domain",
                                span { class: "header-filter-block__label", "{table_domain}" }
                                div { class: "path-input-shell header-filter-field-shell",
                                    input {
                                        id: ui::id::OBSERVATION_DOMAIN_FILTER_INPUT,
                                        class: "{header_domain_filter_class}",
                                        "data-ui-entity": ui::control::OBSERVATION_DOMAIN_FILTER_INPUT,
                                        r#type: "text",
                                        placeholder: "{filter_domain_placeholder}",
                                        "data-committed-value": "{header_domain_filter_value}",
                                        "data-commit-on-enter": "true",
                                        "data-clear-button": "true",
                                        "data-preserve-draft": "true",
                                        "aria-label": "{header_filter_domain_tooltip}",
                                        "data-tooltip": "{header_filter_domain_tooltip}",
                                        "data-tooltip-align": "end",
                                        onchange: move |event| {
                                            let value = event.value().to_string();
                                            pulse_text_input(input_apply_pulse, "header-domain");
                                            if cloud_import_active {
                                                cloud_domain_search_draft.set(value.clone());
                                                cloud_domain_search.set(value);
                                            } else {
                                                monitoring_domain_filter_draft.set(value.clone());
                                                monitoring_domain_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: next.filters.search_text,
                                                    domain_search: value,
                                                    port_search: next.filters.port_search,
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        onkeydown: move |event| {
                                            if event.key() != Key::Enter {
                                                return;
                                            }
                                            pulse_text_input(input_apply_pulse, "header-domain");
                                        }
                                    }
                                    button {
                                        id: ui::id::CLEAR_OBSERVATION_DOMAIN_SEARCH_BUTTON,
                                        class: "path-input-clear",
                                        r#type: "button",
                                        disabled: clear_header_domain_filter_disabled,
                                        "data-clear-button": "true",
                                        "data-ui-action": ui::action::CLEAR_OBSERVATION_DOMAIN_SEARCH,
                                        "aria-label": "{input_clear}",
                                        "data-tooltip": "{input_clear}",
                                        "data-tooltip-align": "end",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            if cloud_import_active {
                                                cloud_domain_search_draft.set(String::new());
                                                cloud_domain_search.set(String::new());
                                            } else {
                                                monitoring_domain_filter_draft.set(String::new());
                                                monitoring_domain_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: next.filters.search_text,
                                                    domain_search: String::new(),
                                                    port_search: next.filters.port_search,
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                    }
                                }
                            }
                            div { class: "header-filter-block header-filter-block--port",
                                span { class: "header-filter-block__label", "{table_port}" }
                                div { class: "path-input-shell header-filter-field-shell header-port-field-shell",
                                    input {
                                        id: ui::id::OBSERVATION_PORT_FILTER_INPUT,
                                        class: "{header_port_filter_class}",
                                        "data-ui-entity": ui::control::OBSERVATION_PORT_FILTER_INPUT,
                                        r#type: "text",
                                        inputmode: "numeric",
                                        placeholder: "{filter_port_placeholder}",
                                        "data-committed-value": "{header_port_filter_value}",
                                        "data-commit-on-enter": "true",
                                        "data-clear-button": "true",
                                        "data-preserve-draft": "true",
                                        "aria-label": "{filter_port}",
                                        "data-tooltip": "{header_filter_port_tooltip}",
                                        "data-tooltip-align": "end",
                                        onchange: move |event| {
                                            let value = event.value().to_string();
                                            pulse_text_input(input_apply_pulse, "header-port");
                                            if cloud_import_active {
                                                cloud_port_search_draft.set(value.clone());
                                                cloud_port_search.set(value);
                                            } else {
                                                monitoring_port_filter_draft.set(value.clone());
                                                monitoring_port_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: next.filters.search_text,
                                                    domain_search: next.filters.domain_search,
                                                    port_search: value,
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        onkeydown: move |event| {
                                            if event.key() != Key::Enter {
                                                return;
                                            }
                                            pulse_text_input(input_apply_pulse, "header-port");
                                        }
                                    }
                                    button {
                                        class: "path-input-clear path-input-clear--port",
                                        r#type: "button",
                                        disabled: clear_header_port_filter_disabled,
                                        "data-clear-button": "true",
                                        "data-ui-action": ui::action::CLEAR_OBSERVATION_PORT_SEARCH,
                                        "aria-label": "{input_clear}",
                                        "data-tooltip": "{input_clear}",
                                        "data-tooltip-align": "end",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            if cloud_import_active {
                                                cloud_port_search_draft.set(String::new());
                                                cloud_port_search.set(String::new());
                                            } else {
                                                monitoring_port_filter_draft.set(String::new());
                                                monitoring_port_filter_dirty.set(false);
                                                let mut current = watcher.write();
                                                let next = current.snapshot();
                                                current.set_filters(crate::watcher_api::SetFilterRequest {
                                                    app_search: next.filters.app_search,
                                                    search_text: next.filters.search_text,
                                                    domain_search: next.filters.domain_search,
                                                    port_search: String::new(),
                                                    protocol: next.filters.protocol,
                                                    public_ip: next.filters.public_ip,
                                                    observation_filter: next.filters.observation_filter,
                                                });
                                            }
                                        },
                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                    }
                                }
                            }
                            div { class: "header-filter-block header-filter-block--protocol",
                                span { class: "header-filter-block__label", "{filter_protocol}" }
                                select {
                                    id: ui::id::OBSERVATION_PROTOCOL_FILTER_SELECT,
                                    class: "input-box select header-filter-select header-filter-select--protocol",
                                    "data-ui-entity": ui::control::OBSERVATION_PROTOCOL_FILTER_SELECT,
                                    value: "{header_protocol_filter_value}",
                                    "aria-label": "{filter_protocol}",
                                    "data-tooltip": "{header_filter_protocol_tooltip}",
                                    "data-tooltip-align": "end",
                                    onchange: move |event| {
                                        let value = event.value().to_string();
                                        if cloud_import_active {
                                            cloud_protocol_filter.set(match value.as_str() {
                                                "TCP" => "tcp".to_string(),
                                                "UDP" => "udp".to_string(),
                                                "OTHER" => "other".to_string(),
                                                _ => "all".to_string(),
                                            });
                                        } else {
                                            let mut current = watcher.write();
                                            let next = current.snapshot();
                                            current.set_filters(crate::watcher_api::SetFilterRequest {
                                                app_search: next.filters.app_search,
                                                search_text: next.filters.search_text,
                                                domain_search: next.filters.domain_search,
                                                port_search: next.filters.port_search,
                                                protocol: value,
                                                public_ip: next.filters.public_ip,
                                                observation_filter: next.filters.observation_filter,
                                            });
                                        }
                                    },
                                    option { value: "All", "{filter_protocol_all}" }
                                    option { value: "TCP", "TCP" }
                                    option { value: "UDP", "UDP" }
                                    option { value: "OTHER", "OTHER" }
                                }
                            }
                            div { class: "header-filter-block header-filter-block--state",
                                span { class: "header-filter-block__label", "{table_state}" }
                                select {
                                    id: ui::id::OBSERVATION_FILTER_SELECT,
                                    class: "input-box select header-filter-select header-filter-select--state",
                                    "data-ui-entity": ui::control::OBSERVATION_FILTER_SELECT,
                                    value: "{header_status_filter_value}",
                                    "aria-label": "{header_filter_status_tooltip}",
                                    "data-tooltip": "{header_filter_status_tooltip}",
                                    "data-tooltip-align": "end",
                                    onchange: move |event| {
                                        let selected = match event.value().as_str() {
                                            "Unconfirmed" => ObservationFilterDto::Unconfirmed,
                                            "Confirmed" => ObservationFilterDto::Confirmed,
                                            "Success" => ObservationFilterDto::Success,
                                            "Failed" => ObservationFilterDto::Failed,
                                            _ => ObservationFilterDto::All,
                                        };
                                        if cloud_import_active {
                                            cloud_row_status_filter.set(selected);
                                        } else {
                                            let mut current = watcher.write();
                                            let next = current.snapshot();
                                            current.set_filters(crate::watcher_api::SetFilterRequest {
                                                app_search: next.filters.app_search,
                                                search_text: next.filters.search_text,
                                                domain_search: next.filters.domain_search,
                                                port_search: next.filters.port_search,
                                                protocol: next.filters.protocol,
                                                public_ip: next.filters.public_ip,
                                                observation_filter: selected,
                                            });
                                        }
                                    },
                                    option { value: "All", "{filter_all}" }
                                    option { value: "Unconfirmed", "{filter_unconfirmed}" }
                                    option { value: "Confirmed", "{filter_confirmed}" }
                                    option { value: "Success", "{filter_success}" }
                                    option { value: "Failed", "{filter_failed}" }
                                }
                            }
                            button {
                                id: ui::id::CONFIRM_FILTERED_BUTTON,
                                class: "input-box button button--icon header-action-button",
                                "data-ui-action": ui::action::CONFIRM_FILTERED,
                                "aria-label": "{header_confirm_filtered_label}",
                                "data-tooltip": "{header_confirm_filtered_label}",
                                "data-tooltip-align": "end",
                                disabled: cloud_import_loading,
                                onclick: {
                                    let header_selection_store = observation_selection_store.clone();
                                    let header_pending_confirm =
                                        pending_observation_selection_confirm.clone();
                                    let cloud_rows = cloud_downloaded_rows.clone();
                                    move |_| {
                                        if cloud_import_active {
                                            let ids: std::collections::BTreeSet<_> = cloud_rows
                                                .iter()
                                                .map(|row| row.row_id.clone())
                                                .collect();
                                            let selected_count = ids.len();
                                            let mut state = cloud_state();
                                            state.selected_download_row_ids = ids;
                                            cloud_state.set(state);
                                            if let Some(last_row) = cloud_rows.last() {
                                                last_cloud_download_selection_anchor.set(Some(last_row.row_id.clone()));
                                            }
                                            push_status_history_line(
                                                status_history,
                                                format!("{action_cloud_import_select_visible}: {selected_count}"),
                                            );
                                        } else {
                                            let snapshot = watcher.read().snapshot();
                                            let ids: Vec<_> = filter_observations_with_header_filters(
                                                &snapshot,
                                                &HeaderObservationFilters {
                                                    app_search: snapshot.filters.app_search.clone(),
                                                    domain_search: snapshot.filters.domain_search.clone(),
                                                    port_search: snapshot.filters.port_search.clone(),
                                                    protocol: snapshot.filters.protocol.clone(),
                                                    public_ip: snapshot.filters.public_ip,
                                                },
                                            )
                                                .into_iter()
                                                .map(|observation| observation.id)
                                                .collect();
                                            let confirmed_count = ids.len();
                                            apply_observation_selection_ids(
                                                &header_selection_store,
                                                &header_pending_confirm,
                                                &ids,
                                                true,
                                            );
                                            if let Some(last_id) = ids.last().copied() {
                                                last_observation_selection_anchor.set(Some(last_id));
                                            }
                                            push_status_history_line(
                                                status_history,
                                                format!("{footer_confirmed_filtered_prefix} {confirmed_count}"),
                                            );
                                        }
                                    }
                                },
                                img {
                                    class: "button__icon button__icon--confirm-filtered",
                                    src: "{confirm_filtered_button_src}",
                                    alt: ""
                                }
                            }
                            button {
                                id: ui::id::UNCONFIRM_FILTERED_BUTTON,
                                class: "input-box button button--icon header-action-button",
                                "data-ui-action": ui::action::UNCONFIRM_FILTERED,
                                "aria-label": "{header_unconfirm_filtered_label}",
                                "data-tooltip": "{header_unconfirm_filtered_label}",
                                "data-tooltip-align": "end",
                                disabled: cloud_import_loading,
                                onclick: {
                                    let header_selection_store = observation_selection_store.clone();
                                    let header_pending_confirm =
                                        pending_observation_selection_confirm.clone();
                                    move |_| {
                                        if cloud_import_active {
                                            let cleared_count = cloud_state().selected_download_row_ids.len();
                                            let mut state = cloud_state();
                                            state.selected_download_row_ids.clear();
                                            cloud_state.set(state);
                                            last_cloud_download_selection_anchor.set(None);
                                            push_status_history_line(
                                                status_history,
                                                format!("{action_cloud_import_clear_selection}: {cleared_count}"),
                                            );
                                        } else {
                                            let ids = clone_observation_selection_store(&header_selection_store)
                                                .into_iter()
                                                .collect::<Vec<_>>();
                                            let unconfirmed_count = ids.len();
                                            apply_observation_selection_ids(
                                                &header_selection_store,
                                                &header_pending_confirm,
                                                &ids,
                                                false,
                                            );
                                            last_observation_selection_anchor.set(None);
                                            profile_export_advanced_domains.set(false);
                                            profile_export_advanced_manual_domains.set(String::new());
                                            profile_export_advanced_ready_for_export.set(false);
                                            push_status_history_line(
                                                status_history,
                                                format!("{action_unconfirm_filtered}: {unconfirmed_count}"),
                                            );
                                        }
                                    }
                                },
                                img {
                                    class: "button__icon button__icon--unconfirm-filtered",
                                    src: "{unconfirm_filtered_button_src}",
                                    alt: ""
                                }
                            }
                            button {
                                id: ui::id::CLEAR_MONITORING_BUTTON,
                                class: "input-box button button--danger button--icon header-action-button",
                                "data-ui-action": ui::action::CLEAR_MONITORING,
                                "aria-label": "{header_clear_rows_label}",
                                "data-tooltip": "{header_clear_rows_label}",
                                "data-tooltip-align": "end",
                                disabled: cloud_import_loading,
                                onclick: move |_| {
                                    if cloud_import_active {
                                        let total_count = cloud_state().downloaded_rows.len();
                                        if total_count == 0 {
                                            push_status_history_line(
                                                status_history,
                                                format!("{action_cloud_import_delete_rows}: 0"),
                                            );
                                        } else {
                                            show_clear_monitoring_prompt.set(true);
                                        }
                                    } else {
                                        let total_count = watcher.read().snapshot().observations.len();
                                        if total_count == 0 {
                                            push_status_history_line(
                                                status_history,
                                                format!("{action_clear_monitoring}: 0"),
                                            );
                                        } else {
                                            show_clear_monitoring_prompt.set(true);
                                        }
                                    }
                                },
                                img {
                                    class: "button__icon button__icon--clear-monitoring",
                                    src: "{clear_monitoring_button_src}",
                                    alt: ""
                                }
                            }
                            div { class: "header-action-separator", "aria-hidden": "true" }
                            button {
                                id: ui::id::OPEN_CLOUD_IMPORT_BUTTON,
                                class: if show_cloud_sync_prompt() && cloud_overlay_mode() == CloudOverlayMode::Download {
                                    "input-box button button--icon button--cloud-active header-action-button"
                                } else {
                                    "input-box button button--icon header-action-button"
                                },
                                "data-ui-action": ui::action::OPEN_CLOUD_IMPORT,
                                "aria-label": "{action_cloud_import}",
                                "data-tooltip": "{action_cloud_import}",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    if show_cloud_sync_prompt() && cloud_overlay_mode() == CloudOverlayMode::Download {
                                        show_cloud_sync_prompt.set(false);
                                        return;
                                    }
                                    cloud_overlay_mode.set(CloudOverlayMode::Download);
                                    show_cloud_sync_prompt.set(true);
                                    cloud_nickname_check_generation.set(
                                        cloud_nickname_check_generation().wrapping_add(1),
                                    );
                                    start_cloud_refresh(
                                        cloud_state,
                                        status_history,
                                        current_cloud_filters(
                                            &cloud_app_search,
                                            &cloud_publisher_search,
                                            &cloud_ip_search,
                                            &cloud_domain_search,
                                            &cloud_port_search,
                                            &cloud_protocol_filter,
                                            &cloud_source_search,
                                            &cloud_selected_app_id,
                                            &cloud_scope_mine,
                                            &cloud_visibility_scope_filter,
                                        ),
                                        cloud_refresh_done_for_import_open.clone(),
                                        cloud_message_labels_for_import_open.clone(),
                                        true,
                                    );
                                },
                                img {
                                    class: "button__icon button__icon--cloud-import",
                                    src: "{cloud_import_button_src}",
                                    alt: ""
                                }
                            }
                            button {
                                id: ui::id::OPEN_CLOUD_EXPORT_BUTTON,
                                class: if show_cloud_sync_prompt() && cloud_overlay_mode() == CloudOverlayMode::Upload {
                                    "input-box button button--icon button--cloud-active header-action-button"
                                } else {
                                    "input-box button button--icon header-action-button"
                                },
                                "data-ui-action": ui::action::OPEN_CLOUD_EXPORT,
                                "aria-label": "{action_cloud_export}",
                                "data-tooltip": "{action_cloud_export}",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    if show_cloud_sync_prompt() && cloud_overlay_mode() == CloudOverlayMode::Upload {
                                        show_cloud_sync_prompt.set(false);
                                        return;
                                    }
                                    cloud_overlay_mode.set(CloudOverlayMode::Upload);
                                    show_cloud_sync_prompt.set(true);
                                    cloud_nickname_check_generation.set(
                                        cloud_nickname_check_generation().wrapping_add(1),
                                    );
                                    start_cloud_refresh(
                                        cloud_state,
                                        status_history,
                                        current_cloud_filters(
                                            &cloud_app_search,
                                            &cloud_publisher_search,
                                            &cloud_ip_search,
                                            &cloud_domain_search,
                                            &cloud_port_search,
                                            &cloud_protocol_filter,
                                            &cloud_source_search,
                                            &cloud_selected_app_id,
                                            &cloud_scope_mine,
                                            &cloud_visibility_scope_filter,
                                        ),
                                        cloud_refresh_done_for_export_open.clone(),
                                        cloud_message_labels_for_export_open.clone(),
                                        true,
                                    );
                                },
                                img {
                                    class: "button__icon button__icon--cloud-sync",
                                    src: "{cloud_sync_button_src}",
                                    alt: ""
                                }
                            }
                            button {
                                id: ui::id::IMPORT_CSV_BUTTON,
                                class: "input-box button button--icon header-action-button",
                                "data-ui-action": ui::action::IMPORT_CSV,
                                "aria-label": "{action_import_csv}",
                                "data-tooltip": "{action_import_csv}",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    open_csv_import_file_dialog(
                                        watcher,
                                        status_history,
                                        footer_csv_import_completed.clone(),
                                        footer_csv_import_failed.clone(),
                                        footer_csv_import_empty.clone(),
                                        csv_import_file_dialog_open,
                                        csv_import_picker_window.clone(),
                                    );
                                },
                                img {
                                    class: "button__icon button__icon--import-csv",
                                    src: "{import_csv_button_src}",
                                    alt: ""
                                }
                            }
                            button {
                                id: ui::id::EXPORT_CSV_BUTTON,
                                class: "input-box button button--icon header-action-button",
                                "data-ui-action": ui::action::EXPORT_CSV,
                                "aria-label": "{action_export_csv}",
                                "data-tooltip": "{action_export_csv}",
                                "data-tooltip-align": "end",
                                onclick: {
                                    let csv_pending_confirm =
                                        pending_observation_selection_confirm.clone();
                                    move |_| {
                                    flush_observation_selection_confirm(
                                        &mut watcher.write(),
                                        &csv_pending_confirm,
                                    );
                                    let rows = selected_csv_export_rows(&watcher.read().snapshot());
                                    if rows.is_empty() {
                                        push_status_history_line(status_history, footer_csv_export_empty.clone());
                                        return;
                                    }
                                    open_csv_export_file_dialog(
                                        rows,
                                        status_history,
                                        footer_csv_export_completed.clone(),
                                        footer_csv_export_failed.clone(),
                                        csv_export_file_dialog_open,
                                        csv_export_picker_window.clone(),
                                        watcher.read().live_base_url(),
                                    );
                                    }
                                },
                                img {
                                    class: "button__icon button__icon--export-csv",
                                    src: "{export_csv_button_src}",
                                    alt: ""
                                }
                            }
                            div { class: "header-action-separator", "aria-hidden": "true" }
                            button {
                                id: ui::id::OPEN_INFORMATION_BUTTON,
                                class: "{information_button_class}",
                                "data-ui-action": ui::action::OPEN_INFORMATION,
                                "aria-label": "{action_information}",
                                "data-tooltip": "{information_button_tooltip}",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    show_information_prompt.set(true);
                                },
                                img {
                                    class: "button__icon button__icon--information",
                                    src: "{information_button_src}",
                                    alt: ""
                                }
                            }
                        }
                        div { class: "header-switch-stack",
                            div { class: "header-switch-row header-switch-row--domain-capture",
                                span { class: "header-switch-row__label", "{header_domain_capture}" }
                                button {
                                    class: if snapshot.app_settings.domain_capture_enabled { "input-box switch switch--on" } else { "input-box switch" },
                                    r#type: "button",
                                    role: "switch",
                                    "aria-checked": "{snapshot.app_settings.domain_capture_enabled}",
                                    "aria-label": "{domain_capture_tooltip}",
                                    "data-tooltip": "{domain_capture_tooltip}",
                                    "data-tooltip-align": "end",
                                    onclick: move |_| {
                                        let snapshot = watcher.read().snapshot();
                                        let enabled = !snapshot.app_settings.domain_capture_enabled;
                                        if enabled && snapshot.ui.snapshot_loaded && !snapshot.runtime_status.is_elevated {
                                            watcher.write().set_domain_capture_enabled(false);
                                            show_domain_capture_admin_prompt.set(true);
                                            push_status_history_line(
                                                status_history,
                                                footer_domain_capture_admin_required.clone(),
                                            );
                                            return;
                                        }
                                        watcher.write().set_domain_capture_enabled(enabled);
                                        push_status_history_line(
                                            status_history,
                                            if enabled {
                                                footer_domain_capture_enabled.clone()
                                            } else {
                                                footer_domain_capture_disabled.clone()
                                            },
                                        );
                                    },
                                    span { class: "switch__knob" }
                                }
                            }
                            div { class: "header-switch-row header-switch-row--server",
                                span { class: "header-switch-row__label", "{header_web_access_localhost}" }
                                button {
                                    id: ui::id::WEB_ACCESS_LOCALHOST_BUTTON,
                                    class: if snapshot.app_settings.web_access_localhost { "input-box switch switch--on" } else { "input-box switch" },
                                    "data-ui-action": ui::action::TOGGLE_WEB_ACCESS_LOCALHOST,
                                    role: "switch",
                                    "aria-checked": "{snapshot.app_settings.web_access_localhost}",
                                    "aria-label": "{web_access_tooltip}",
                                    "data-tooltip": "{web_access_tooltip}",
                                    "data-tooltip-align": "end",
                                    onclick: move |_| {
                                        let enabled = !watcher
                                            .read()
                                            .snapshot()
                                            .app_settings
                                            .web_access_localhost;
                                        watcher.write().set_web_access_localhost(enabled);
                                    },
                                    span { class: "switch__knob" }
                                }
                            }
                        }
                    }
                }

                div {
                    id: ui::id::APP_CONTENT,
                    class: "shell__content",
                    "data-ui-entity": ui::entity::APP_CONTENT,
                    div { class: "workspace",
                    div { class: "column",
                        section {
                            id: ui::id::TRACKED_APPS_PANEL,
                            class: "card tracked-apps-card",
                            "data-ui-entity": ui::entity::TRACKED_APPS_PANEL,
                            div {
                                id: ui::id::TRACKED_APPS_HEADER,
                                class: "card__header tracked-apps-header",
                                "data-ui-entity": ui::entity::TRACKED_APPS_HEADER,
                                div { class: "title-with-help",
                                    h2 { "{tracked_apps_title}" }
                                    HelpIcon {
                                        icon_src: help_icon_src.clone(),
                                        tooltip: tracked_apps_help.clone(),
                                    }
                                }
                                div { class: "tracked-apps-header-meta",
                                    span { class: "{tracked_apps_enabled_meta_class}",
                                        "{tracked_apps_enabled_status}: {enabled_tracked_apps_count}"
                                    }
                                }
                            }
                            div { class: "stack",
                                div { class: "exe-path-row",
                                    div { class: "path-input-shell",
                                        input {
                                            id: ui::id::EXE_PATH_INPUT,
                                            class: "{pending_exe_path_class}",
                                            "data-ui-entity": ui::control::EXE_PATH_INPUT,
                                            r#type: "text",
                                            placeholder: "{tracked_apps_path_placeholder}",
                                            "data-committed-value": "{pending_exe_path_value}",
                                            "data-commit-on-enter": "true",
                                            "data-clear-button": "true",
                                            "data-preserve-draft": "true",
                                            "data-enter-click-target": ui::id::ADD_EXE_BUTTON,
                                            onchange: move |event| {
                                                pulse_text_input(input_apply_pulse, "tracked-app-path");
                                                let value = event.value().to_string();
                                                pending_exe_path_draft.set(value.clone());
                                                pending_exe_path_dirty.set(false);
                                                watcher.write().set_pending_exe_path(value);
                                            },
                                            onkeydown: move |event| {
                                                if event.key() != Key::Enter {
                                                    return;
                                                }
                                                pulse_text_input(input_apply_pulse, "tracked-app-path");
                                            }
                                        }
                                        button {
                                            id: ui::id::CLEAR_EXE_PATH_BUTTON,
                                            class: "path-input-clear",
                                            r#type: "button",
                                            disabled: clear_pending_exe_path_disabled,
                                            "data-clear-button": "true",
                                            "data-ui-action": ui::action::CLEAR_EXE_PATH,
                                            "aria-label": "{input_clear}",
                                            "data-tooltip": "{input_clear}",
                                            "data-tooltip-align": "end",
                                            onclick: move |event| {
                                                event.stop_propagation();
                                                pulse_text_input(input_apply_pulse, "tracked-app-path");
                                                pending_exe_path_draft.set(String::new());
                                                pending_exe_path_dirty.set(false);
                                                watcher.write().set_pending_exe_path(String::new());
                                            },
                                            img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                        }
                                    }
                                    button {
                                        id: ui::id::ADD_EXE_BUTTON,
                                        class: "input-box button button--icon",
                                        "data-ui-action": ui::action::ADD_EXE,
                                        "aria-label": "{tracked_apps_add_exe}",
                                        "data-tooltip": "{tracked_apps_add_exe}",
                                        "data-tooltip-align": "end",
                                        onclick: move |_| {
                                            add_pending_or_pick_executable(
                                                watcher,
                                                exe_file_dialog_open,
                                                executable_picker_window.clone(),
                                                Some(pending_exe_path_value.clone()),
                                            );
                                        },
                                        "+"
                                    }
                                }
                                div { class: "tracked-apps-toolbar-spacer", aria_hidden: "true" }
                                div { class: "tracked-apps-bulk-row",
                                    label { class: "tracked-apps-bulk-label", "{tracked_apps_enable_all}" }
                                    button {
                                        id: ui::id::TOGGLE_ALL_APPS_BUTTON,
                                        class: if enable_all_overlay { "input-box switch switch--on" } else { "input-box switch" },
                                        "data-ui-action": ui::action::TOGGLE_ALL_APPS,
                                        role: "switch",
                                        "aria-checked": "{enable_all_overlay}",
                                        "aria-label": "{toggle_all_tooltip}",
                                        "data-tooltip": "{toggle_all_tooltip}",
                                        "data-tooltip-align": "start",
                                        onclick: move |_| {
                                            let mut current = watcher.write();
                                            let enable_all = !current
                                                .snapshot()
                                                .app_settings
                                                .enable_all_overlay;
                                            current.set_all_tracked_apps_enabled(
                                                SetAllTrackedAppsEnabledRequest {
                                                    enabled: enable_all,
                                                },
                                            );
                                        },
                                        span { class: "switch__knob" }
                                    }
                                    span { class: "tracked-apps-count-label", "{snapshot.tracked_apps.len()} {tracked_apps_items}" }
                                }
                            }
                            div {
                                id: ui::id::TRACKED_APP_LIST,
                                class: "list",
                                "data-ui-entity": ui::entity::TRACKED_APP_LIST,
                                if tracked_apps_bootstrap {
                                    for _ in 0..5 {
                                        div { class: "tracked-app-skeleton" }
                                    }
                                }
                                for app in snapshot.tracked_apps.clone() {
                                    TrackedAppItem {
                                        app: rendered_tracked_app(&app, enable_all_overlay),
                                        enable_tooltip: switch_enable_app.clone(),
                                        disable_tooltip: switch_disable_app.clone(),
                                        open_folder_label: tracked_app_open_folder.clone(),
                                        delete_label: tracked_app_delete.clone(),
                                        onclick_toggle: move |_| {
                                            if watcher.read().snapshot().app_settings.enable_all_overlay {
                                                return;
                                            }
                                            let mut current = watcher.write();
                                            current.toggle_tracked_app(ToggleTrackedAppRequest { app_id: app.id });
                                        },
                                        onclick_delete: move |_| {
                                            pending_delete_app.set(Some(app.clone()));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "column",
                        section {
                            id: ui::id::INTEGRATION_PANEL,
                            class: "card modules-card",
                            "data-ui-entity": ui::entity::INTEGRATION_PANEL,
                            div { class: "card__header",
                                div { class: "title-with-help",
                                    h2 { "{integration_title}" }
                                    HelpIcon {
                                        icon_src: help_icon_src.clone(),
                                        tooltip: integration_subtitle.clone(),
                                    }
                                }
                                div { class: "modules-card__header-actions",
                                    span { class: "modules-card__order-label", "{modules_order_label}" }
                                    button {
                                        class: "{modules_order_switch_class}",
                                        r#type: "button",
                                        "aria-label": "{modules_order_label}",
                                        "data-tooltip": "{modules_order_label}",
                                        "data-tooltip-align": "end",
                                        onclick: move |_| module_order_editing.set(!module_order_editing()),
                                        span { class: "switch__knob" }
                                    }
                                    if integration_modules_ready {
                                        span { class: "panel-header-meta panel-header-meta--success", "{integration_modules_loaded}" }
                                    }
                                }
                            }
                            div { class: "integration-status integration-status--compact",
                                div { class: "integration-module-grid",
                                    if integration_modules_bootstrap {
                                        for _ in 0..10 {
                                            div {
                                                class: "integration-module-button-skeleton",
                                                "aria-hidden": "true",
                                            }
                                        }
                                    }
                                    for module in integration_module_buttons.iter() {
                                        {
                                            let module_id = module.id.clone();
                                            let module_name = module.display_name.clone();
                                            let module_tooltip = if module.tooltip.trim().is_empty() {
                                                module.display_name.clone()
                                            } else {
                                                module.tooltip.clone()
                                            };
                                            let module_tooltip = if module.background_active {
                                                format!(
                                                    "{module_tooltip} - {modules_background_running}"
                                                )
                                            } else {
                                                module_tooltip
                                            };
                                            let module_button_style = module_button_style(module);
                                            let module_button_class = if module.background_active {
                                                "input-box button button--icon integration-module-button integration-module-button--background-active"
                                            } else {
                                                "input-box button button--icon integration-module-button"
                                            };
                                            let module_icon_src = module_icon_src(module);
                                            let repo_path_for_click =
                                                integration_repo_path_for_prompt.clone();
                                            let module_id_for_click = module_id.clone();
                                            let module_icon = {
                                                let label = module.icon_label.trim();
                                                if label.is_empty() {
                                                    module
                                                        .display_name
                                                        .chars()
                                                        .next()
                                                        .map(|ch| ch.to_string())
                                                        .unwrap_or_else(|| "*".to_string())
                                                } else {
                                                    label.chars().take(4).collect::<String>()
                                                }
                                            };
                                            let reorder_modules_left = integration_module_buttons.clone();
                                            let reorder_modules_right = integration_module_buttons.clone();
                                            let module_id_for_left = module_id.clone();
                                            let module_id_for_right = module_id.clone();
                                            let module_reorder_left_icon_src =
                                                module_reorder_left_button_src.clone();
                                            let module_reorder_right_icon_src =
                                                module_reorder_left_button_src.clone();
                                            let mut watcher_for_left = watcher;
                                            let mut watcher_for_right = watcher;
                                            let mut watcher_for_open = watcher;
                                            let module_for_open = module.clone();
                                            let selected_ids_for_open = selected_observation_ids
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();
                                            let displayed_ids_for_open = visible_observations
                                                .iter()
                                                .map(|observation| observation.id)
                                                .collect::<Vec<_>>();
                                            let filters_for_open =
                                                shared_filters_from_snapshot(&snapshot);
                                            let window_for_open = window.clone();
                                            rsx! {
                                                div { class: "integration-module-button-shell",
                                                    if module_order_editing() {
                                                        button {
                                                            class: "input-box button button--mini integration-module-reorder-button integration-module-reorder-button--left",
                                                            r#type: "button",
                                                            "aria-label": "{modules_order_move_left}",
                                                            "data-tooltip": "{modules_order_move_left}",
                                                            "data-tooltip-align": "start",
                                                            onclick: move |_| {
                                                                let next_order = move_module_in_order(
                                                                    &module_order(),
                                                                    &reorder_modules_left,
                                                                    &module_id_for_left,
                                                                    -1,
                                                                );
                                                                module_order.set(next_order.clone());
                                                                watcher_for_left.write().set_module_order(next_order);
                                                            },
                                                            img {
                                                                class: "button__icon integration-module-reorder-button__icon",
                                                                src: "{module_reorder_left_icon_src}",
                                                                alt: ""
                                                            }
                                                        }
                                                    }
                                                    button {
                                                        id: "netstitch-ui-integration-module-{module_id}",
                                                        class: "{module_button_class}",
                                                        style: "{module_button_style}",
                                                        "data-ui-action": ui::action::OPEN_INTEGRATION_MODULE,
                                                        "data-ui-key": "{module_id}",
                                                        "aria-label": "{module_name}",
                                                        "data-tooltip": "{module_tooltip}",
                                                        "data-tooltip-align": "start",
                                                        onclick: move |_| {
                                                            if module_order_editing() {
                                                                return;
                                                            }
                                                            selected_integration_module_id
                                                                .set(Some(module_id_for_click.clone()));
                                                            module_ui_page.set("main".to_string());
                                                            show_integration_module_prompt.set(true);
                                                            integration_feedback.set(None);
                                                            if integration_path_input().is_empty()
                                                                && repo_path_for_click != "Not configured"
                                                            {
                                                                integration_path_input
                                                                    .set(repo_path_for_click.clone());
                                                            }
                                                            if let Some(open_action_id) = module_open_action_id(&module_for_open) {
                                                                start_module_ui_action(
                                                                    watcher_for_open,
                                                                    status_history,
                                                                    module_host_dialog,
                                                                    module_ui_page,
                                                                    module_ui_values,
                                                                    module_ui_action_generation,
                                                                    module_path_picker_dialog_open,
                                                                    window_for_open.clone(),
                                                                    module_for_open.clone(),
                                                                    module_name.clone(),
                                                                    open_action_id.clone(),
                                                                    IntegrationModuleUiActionClientRequestDto {
                                                                        module_id: module_id_for_click.clone(),
                                                                        action_id: open_action_id,
                                                                        ui_action_token: String::new(),
                                                                        selected_monitoring_row_ids: selected_ids_for_open.clone(),
                                                                        displayed_monitoring_row_ids: displayed_ids_for_open.clone(),
                                                                        filters: filters_for_open.clone(),
                                                                        payload: serde_json::Value::Null,
                                                                    },
                                                                );
                                                            }
                                                        },
                                                        if let Some(icon_src) = module_icon_src {
                                                            img {
                                                                class: "integration-module-button__image",
                                                                src: "{icon_src}",
                                                                alt: ""
                                                            }
                                                        } else {
                                                            span { class: "integration-module-button__icon", "{module_icon}" }
                                                        }
                                                    }
                                                    if module_order_editing() {
                                                        button {
                                                            class: "input-box button button--mini integration-module-reorder-button integration-module-reorder-button--right",
                                                            r#type: "button",
                                                            "aria-label": "{modules_order_move_right}",
                                                            "data-tooltip": "{modules_order_move_right}",
                                                            "data-tooltip-align": "end",
                                                            onclick: move |_| {
                                                                let next_order = move_module_in_order(
                                                                    &module_order(),
                                                                    &reorder_modules_right,
                                                                    &module_id_for_right,
                                                                    1,
                                                                );
                                                                module_order.set(next_order.clone());
                                                                watcher_for_right.write().set_module_order(next_order);
                                                            },
                                                            img {
                                                                class: "button__icon integration-module-reorder-button__icon integration-module-reorder-button__icon--right",
                                                                src: "{module_reorder_right_icon_src}",
                                                                alt: ""
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        section {
                            id: ui::id::IGNORED_ADDRESSES_PANEL,
                            class: "card ignored-addresses-card",
                            "data-ui-entity": ui::entity::IGNORED_ADDRESSES_PANEL,
                            div { class: "card__header",
                                div { class: "title-with-help",
                                    h2 { "{ignored_addresses_title}" }
                                    HelpIcon {
                                        icon_src: help_icon_src.clone(),
                                        tooltip: ignored_addresses_help.clone(),
                                    }
                                }
                            }
                            div { class: "ignored-addresses-section",
                                div { class: "ignored-addresses",
                                    if ignored_addresses_loading {
                                        for _ in 0..5 {
                                            div { class: "ignored-addresses__skeleton" }
                                        }
                                    } else if snapshot.ignored_addresses.is_empty() {
                                        div { class: "ignored-addresses__empty", "{ignored_addresses_empty}" }
                                    }
                                    for rule in snapshot.ignored_addresses.clone() {
                                            IgnoredAddressRowView {
                                                rule: rule.clone(),
                                                help_icon_src: help_icon_src.clone(),
                                                domain_label: enrichment_domain_label.clone(),
                                                owner_label: enrichment_owner_label.clone(),
                                                range_label: enrichment_range_label.clone(),
                                            registry_label: enrichment_registry_label.clone(),
                                            source_label: enrichment_source_label.clone(),
                                            unknown_label: enrichment_unknown.clone(),
                                            localhost_rule_label: enrichment_localhost_rule.clone(),
                                            local_ip_rule_label: enrichment_local_ip_rule.clone(),
                                            delete_label: ignored_addresses_remove.clone(),
                                            close_icon_src: close_button_src.clone(),
                                            on_delete: move |event: MouseEvent| {
                                                event.stop_propagation();
                                                pending_delete_ignored_address.set(Some(rule.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    section {
                        id: ui::id::OBSERVATIONS_PANEL,
                        class: "card observations-card",
                        "data-ui-entity": ui::entity::OBSERVATIONS_PANEL,
                        div { class: "card__header",
                            div { class: "title-with-help",
                                h2 { "{observations_title}" }
                                HelpIcon {
                                    icon_src: help_icon_src.clone(),
                                    tooltip: observations_subtitle.clone(),
                                }
                            }
                            div { class: "monitoring-header-meta",
                                label {
                                    class: "header-switch-row header-switch-row--filter monitoring-public-toggle",
                                    "data-tooltip": "{observations_public_ip_tooltip}",
                                    "data-tooltip-align": "end",
                                    span { class: "header-switch-row__label", "{observations_public_ip}" }
                                    button {
                                        class: if snapshot.filters.public_ip { "input-box switch switch--on" } else { "input-box switch" },
                                        r#type: "button",
                                        role: "switch",
                                        "aria-checked": "{snapshot.filters.public_ip}",
                                        "aria-label": "{observations_public_ip}",
                                        "data-ui-action": ui::action::TOGGLE_PUBLIC_IP_FILTER,
                                        onclick: move |_| {
                                            let mut current = watcher.write();
                                            let next = current.snapshot();
                                            current.set_filters(crate::watcher_api::SetFilterRequest {
                                                app_search: next.filters.app_search,
                                                search_text: next.filters.search_text,
                                                domain_search: next.filters.domain_search,
                                                port_search: next.filters.port_search,
                                                protocol: next.filters.protocol,
                                                public_ip: !next.filters.public_ip,
                                                observation_filter: next.filters.observation_filter,
                                            });
                                        },
                                        span { class: "switch__knob" }
                                    }
                                }
                            }
                        }
                        div { class: "table-wrap",
                            div { class: "table-header-wrap",
                                div { class: "table-header-scroll",
                                    table {
                                        class: "observations-table observations-table--header",
                                        thead {
                                            tr {
                                                SortHeaderCell { label: table_app.clone(), tooltip: table_app.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::App), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::App)) }
                                                SortHeaderCell { label: table_ip.clone(), tooltip: table_ip.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Ip), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Ip)) }
                                                SortHeaderCell { label: table_domain.clone(), tooltip: table_domain_tooltip.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Domain), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Domain)) }
                                                SortHeaderCell { label: table_port.clone(), tooltip: table_port.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Port), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Port)) }
                                                SortHeaderCell { label: table_proto.clone(), tooltip: filter_protocol.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Protocol), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Protocol)) }
                                                SortHeaderCell { label: table_conn.clone(), tooltip: table_conn_tooltip.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Connection), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Connection)) }
                                                SortHeaderCell { label: table_hits.clone(), tooltip: table_hits_tooltip.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::Hits), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::Hits)) }
                                                SortHeaderCell { label: table_first_seen.clone(), tooltip: table_first_seen.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::FirstSeen), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::FirstSeen)) }
                                                SortHeaderCell { label: table_last_seen.clone(), tooltip: table_last_seen.clone(), sort_state: observation_sort().indicator_state(ObservationSortColumn::LastSeen), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| observation_sort.set(observation_sort().toggled(ObservationSortColumn::LastSeen)) }
                                                th { "data-tooltip": "{table_action}", "data-tooltip-align": "start", "{table_action}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "table-header-scrollbar-fill", "aria-hidden": "true" }
                            }
                            div { class: "table-body-wrap",
                                table {
                                    id: ui::id::OBSERVATIONS_TABLE,
                                    class: "observations-table observations-table--body",
                                    "data-ui-entity": ui::entity::OBSERVATIONS_TABLE,
                                    tbody {
                                    if !snapshot.ui.snapshot_loaded && visible_observations.is_empty() {
                                        tr {
                                            td {
                                                colspan: "10",
                                                div { class: "observations-empty-state" }
                                            }
                                        }
                                    } else {
                                        for observation in visible_observations.clone() {
                                            ObservationRowView {
                                                key: "{observation.id}",
                                                observation: observation.clone(),
                                                is_selected: observation_row_is_selected(&observation_selection_store, observation.id),
                                                app_name: app_name_for_observation(&snapshot, &observation),
                                                confirm_label: action_confirm.clone(),
                                                unconfirm_label: action_unconfirm.clone(),
                                                ignore_label: action_ignore_address.clone(),
                                                delete_label: action_delete_observation.clone(),
                                                help_icon_src: help_icon_src.clone(),
                                                close_icon_src: close_button_src.clone(),
                                                domain_label: enrichment_domain_label.clone(),
                                                owner_label: enrichment_owner_label.clone(),
                                                range_label: enrichment_range_label.clone(),
                                                registry_label: enrichment_registry_label.clone(),
                                                source_label: enrichment_source_label.clone(),
                                                unknown_label: enrichment_unknown.clone(),
                                                confirm_icon_src: confirm_filtered_button_src.clone(),
                                                unconfirm_icon_src: unconfirm_filtered_button_src.clone(),
                                                ignore_icon_src: ignore_address_button_src.clone(),
                                                on_row_click: {
                                                    let row_observations =
                                                        visible_observations_for_selection.clone();
                                                    let row_selection_store = observation_selection_store.clone();
                                                    let row_pending_confirm =
                                                        pending_observation_selection_confirm.clone();
                                                    let row_observation_id = observation.id;
                                                    move |event: MouseEvent| {
                                                        let modifiers = event.modifiers();
                                                        apply_observation_selection_click(
                                                            last_observation_selection_anchor,
                                                            &row_selection_store,
                                                            &row_pending_confirm,
                                                            row_observations.as_slice(),
                                                            row_observation_id,
                                                            modifiers.contains(Modifiers::CONTROL),
                                                            modifiers.contains(Modifiers::SHIFT),
                                                        );
                                                    }
                                                },
                                                on_toggle_confirmed: {
                                                    let button_selection_store = observation_selection_store.clone();
                                                    let button_pending_confirm =
                                                        pending_observation_selection_confirm.clone();
                                                    let button_observation_id = observation.id;
                                                    move |event: MouseEvent| {
                                                        event.stop_propagation();
                                                        toggle_observation_selection_button(
                                                            last_observation_selection_anchor,
                                                            &button_selection_store,
                                                            &button_pending_confirm,
                                                            button_observation_id,
                                                        );
                                                    }
                                                },
                                                on_ignore_address: {
                                                    let remote_ip = observation.remote_ip.clone();
                                                    move |event: MouseEvent| {
                                                        event.stop_propagation();
                                                        let mut current = watcher.write();
                                                        current.ignore_address(IgnoreAddressRequest {
                                                            address_pattern: remote_ip.clone(),
                                                        });
                                                    }
                                                },
                                                on_delete: {
                                                    let observation_for_delete = observation.clone();
                                                    move |event: MouseEvent| {
                                                        event.stop_propagation();
                                                        pending_delete_observation.set(Some(observation_for_delete.clone()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                }
                            }
                        }
                        div {
                            class: "panel-footer monitoring-panel-footer",
                            "data-ui-entity": ui::entity::PANEL_FOOTER,
                            span {
                                class: "panel-footer-meta",
                                span { class: "panel-footer-meta__item", "{observations_total}: {monitoring_total_count}" }
                                span { class: "panel-footer-meta__item", "{observations_displayed}: {monitoring_displayed_count}" }
                                span { class: "panel-footer-meta__item", "{observations_selected}: {monitoring_selected_count}" }
                            }
                        }
                    }
                }
                }
                footer {
                    id: ui::id::APP_FOOTER,
                    class: "app-chrome app-chrome--footer",
                    "data-ui-entity": ui::entity::APP_FOOTER,
                    div { class: "app-footer-panel app-footer-panel--static",
                        "data-ui-entity": ui::entity::APP_FOOTER_APPS_PANEL,
                        "{footer_apps}: {enabled_tracked_apps_count}/{snapshot.tracked_apps.len()}"
                    }
                    div {
                        id: ui::id::APP_FOOTER_WATCHER_STATUS,
                        class: "app-footer-panel footer-watcher-status",
                        "data-ui-entity": ui::entity::APP_FOOTER_WATCHER_STATUS,
                        "data-tooltip": "{watcher_connection_tooltip}",
                        "data-tooltip-align": "start",
                        tabindex: "0",
                        span { class: "footer-watcher-status__label", "{footer_watcher_label}" }
                        span {
                            class: if !footer_status_ready {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            } else if snapshot.ui.watcher_connected {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--connected"
                            } else {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--disconnected"
                        },
                        "aria-hidden": "true",
                        }
                    }
                    div {
                        id: ui::id::APP_FOOTER_TOOL_STATUS,
                        class: "app-footer-panel footer-tool-status",
                        "data-ui-entity": ui::entity::APP_FOOTER_TOOL_STATUS,
                        "data-tooltip": "{tool_connection_tooltip}",
                        "data-tooltip-align": "start",
                        tabindex: "0",
                        span { class: "footer-tool-status__label", "{footer_tool_label}" }
                        span {
                            class: if !footer_status_ready {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            } else if tool_available {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--connected"
                            } else {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--disconnected"
                            },
                            "aria-hidden": "true",
                        }
                    }
                    div {
                        id: ui::id::APP_FOOTER_WEB_SERVER_STATUS,
                        class: "app-footer-panel footer-web-server-status",
                        "data-ui-entity": ui::entity::APP_FOOTER_WEB_SERVER_STATUS,
                        "data-tooltip": "{web_server_tooltip}",
                        "data-tooltip-align": "start",
                        role: "button",
                        tabindex: "0",
                        onclick: move |event| {
                            event.stop_propagation();
                            copy_text_to_clipboard(&clipboard_window, &web_server_url_for_footer.url);
                            if web_server_url_for_footer.used_localhost_fallback {
                                push_status_history_line(status_history, footer_web_server_localhost_fallback_for_footer.clone());
                            }
                            push_status_history_line(status_history, web_server_copied_message.clone());
                        },
                        span { class: "footer-web-server-status__label", "{footer_web_server_label}" }
                        span {
                            class: if !footer_status_ready {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            } else if web_server_available {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--connected"
                            } else if snapshot.app_settings.web_access_localhost {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--disconnected"
                            } else {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            },
                            "aria-hidden": "true",
                        }
                    }
                    div {
                        id: ui::id::APP_FOOTER_NETWORK_STATUS,
                        class: "app-footer-panel footer-network-status",
                        "data-ui-entity": ui::entity::APP_FOOTER_NETWORK_STATUS,
                        "data-tooltip": "{network_probe_tooltip}",
                        "data-tooltip-align": "start",
                        tabindex: "0",
                        span { class: "footer-network-status__label", "{footer_network_label}" }
                        span {
                            class: if !footer_status_ready {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            } else if network_probe_available {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--connected"
                            } else if network_probe_checking {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--inactive"
                            } else {
                                "footer-watcher-status__indicator footer-watcher-status__indicator--disconnected"
                            },
                            "aria-hidden": "true",
                        }
                    }
                    div {
                        id: ui::id::APP_FOOTER_MESSAGE_PANEL,
                        class: "app-footer-panel app-footer-message-panel",
                        "data-ui-entity": ui::entity::APP_FOOTER_MESSAGE_PANEL,
                        button {
                            class: "input-box button button--square button--copy-status",
                            "data-ui-action": ui::action::COPY_STATUS_HISTORY,
                            "aria-label": "{footer_status_copy_tooltip}",
                            "data-tooltip": "{footer_status_copy_tooltip}",
                            "data-tooltip-align": "start",
                            onclick: move |event| {
                                event.stop_propagation();
                                let footer_status_copy_text =
                                    footer_message_copy_text(&load_footer_message_history_limit(100));
                                copy_text_to_clipboard(&status_history_window, &footer_status_copy_text);
                                push_status_history_line(status_history, footer_status_copied.clone());
                            },
                            img {
                                class: "button__icon",
                                src: "{copy_button_src}",
                                alt: ""
                            }
                        }
                        span { class: "footer-message-label", "{footer_message_label}:" }
                        input {
                            class: "{footer_status_text_class}",
                            r#type: "text",
                            readonly: true,
                            value: "{footer_status_text}",
                            "aria-label": "{footer_message_label}",
                            "data-tooltip": "{footer_status_tooltip}",
                            "data-tooltip-align": "end",
                        }
                    }
                    div {
                        id: ui::id::APP_FOOTER_LANGUAGE_PANEL,
                        class: "app-footer-panel app-footer-actions app-footer-language-panel",
                        "data-ui-entity": ui::entity::APP_FOOTER_LANGUAGE_PANEL,
                        span { class: "footer-language-label", "{footer_language_label}:" }
                        div { class: "language-select-shell",
                            "data-ui-entity": ui::entity::APP_FOOTER_LANGUAGE_SELECT,
                            button {
                                id: ui::id::LANGUAGE_SELECT,
                                class: "language-select-control input-box",
                                r#type: "button",
                                "aria-label": "{footer_language_label}",
                                "aria-haspopup": "listbox",
                                "aria-expanded": "{language_menu_open()}",
                                onclick: move |event| {
                                    event.stop_propagation();
                                    language_menu_open.set(!language_menu_open());
                                },
                                span { class: "language-select-control__label", "{selected_language_label}" }
                            }
                            if language_menu_open() {
                                div {
                                    class: "language-select-menu",
                                    role: "listbox",
                                    for language in language_options.clone() {
                                        button {
                                            class: if language.code == selected_language_code {
                                                "language-select-option language-select-option--selected"
                                            } else {
                                                "language-select-option"
                                            },
                                            r#type: "button",
                                            role: "option",
                                            "aria-selected": "{language.code == selected_language_code}",
                                            onclick: move |event| {
                                                event.stop_propagation();
                                                selected_language.set(language.code.clone());
                                                watcher.write().set_language_code(language.code.clone());
                                                language_menu_open.set(false);
                                            },
                                            "{language.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if show_integration_module_prompt() {
            div { class: "modal-backdrop module-overlay-backdrop",
                div {
                    id: ui::id::INTEGRATION_MODULE_DIALOG,
                    class: "modal modal--panel integration-module-dialog",
                    "data-ui-entity": ui::entity::INTEGRATION_MODULE_DIALOG,
                    role: "dialog",
                    "aria-modal": "true",
                    div {
                        class: "modal__header",
                        "data-ui-entity": ui::entity::PANEL_HEADER,
                        div { class: "title-with-help",
                            h2 { "{integration_module_dialog_title}" }
                            HelpIcon {
                                icon_src: help_icon_src.clone(),
                                tooltip: integration_module_dialog_help.clone(),
                            }
                        }
                    }
                    div {
                        class: "modal__body integration-module-dialog__body",
                        "data-ui-entity": ui::entity::SUBPANEL,
                        if let Some(module) = selected_integration_module.as_ref() {
                            if !module.ui_schema.is_empty() {
                                {
                                    let module_latest_rows = module_context_latest_rows(
                                        module.background_active,
                                        &snapshot.observations,
                                    );
                                    let module_latest_rows_count = module_context_latest_rows_count(
                                        module.background_active,
                                        &snapshot.observations,
                                        5,
                                    );
                                    let module_last_row = module_context_last_row(
                                        module.background_active,
                                        &snapshot.observations,
                                    );
                                    let module_ui_schema = module_ui_schema_with_context(
                                        &module.ui_schema,
                                        &module_ui_page(),
                                        selected_observation_ids.len(),
                                        visible_observations.len(),
                                        snapshot.observations.len(),
                                        module.background_active,
                                        &module_last_row,
                                        &module_latest_rows,
                                        module_latest_rows_count,
                                    );
                                    let module_ui_schema = module_ui_entities_without_footers(module_ui_schema);
                                    rsx! {
                                        div {
                                            class: "module-ui-schema",
                                            "data-ui-entity": "module-ui-schema",
                                            for entity in module_ui_schema.iter().cloned() {
                                                IntegrationUiEntityView {
                                                    entity,
                                                    module_id: selected_integration_module_id(),
                                                    module: module.clone(),
                                                    selected_monitoring_row_ids: selected_observation_ids
                                                        .iter()
                                                        .copied()
                                                        .collect::<Vec<_>>(),
                                                    displayed_monitoring_row_ids: visible_observations
                                                        .iter()
                                                        .map(|observation| observation.id)
                                                        .collect::<Vec<_>>(),
                                                    filters: shared_filters_from_snapshot(&snapshot),
                                                    watcher,
                                                    status_history,
                                                    module_host_dialog,
                                                    module_ui_page,
                                                    module_ui_values,
                                                    module_ui_action_generation,
                                                    module_path_picker_dialog_open,
                                                    module_title: integration_module_dialog_title.clone(),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "modal__footer",
                        "data-ui-entity": ui::entity::PANEL_FOOTER,
                            {
                                let module_footer_entities = selected_integration_module
                                    .as_ref()
                                    .map(|module| {
                                        let module_latest_rows = module_context_latest_rows(
                                            module.background_active,
                                            &snapshot.observations,
                                        );
                                        let module_latest_rows_count = module_context_latest_rows_count(
                                            module.background_active,
                                            &snapshot.observations,
                                            5,
                                        );
                                        let module_last_row = module_context_last_row(
                                            module.background_active,
                                            &snapshot.observations,
                                        );
                                        module_ui_footer_entities(module_ui_schema_with_context(
                                            &module.ui_schema,
                                            &module_ui_page(),
                                            selected_observation_ids.len(),
                                            visible_observations.len(),
                                            snapshot.observations.len(),
                                            module.background_active,
                                            &module_last_row,
                                            &module_latest_rows,
                                            module_latest_rows_count,
                                        ))
                                    })
                                    .unwrap_or_default();
                                let hide_host_back_button = module_footer_entities
                                    .iter()
                                    .any(|entity| entity.hide_host_back_button);
                                rsx! {
                                    for entity in module_footer_entities.iter().cloned() {
                                        if let Some(module) = selected_integration_module.as_ref() {
                                            IntegrationUiEntityView {
                                                entity,
                                                module_id: selected_integration_module_id(),
                                                module: module.clone(),
                                                selected_monitoring_row_ids: selected_observation_ids
                                                    .iter()
                                                    .copied()
                                                    .collect::<Vec<_>>(),
                                                displayed_monitoring_row_ids: visible_observations
                                                    .iter()
                                                    .map(|observation| observation.id)
                                                    .collect::<Vec<_>>(),
                                                filters: shared_filters_from_snapshot(&snapshot),
                                                watcher,
                                                status_history,
                                                module_host_dialog,
                                                module_ui_page,
                                                module_ui_values,
                                                module_ui_action_generation,
                                                module_path_picker_dialog_open,
                                                module_title: integration_module_dialog_title.clone(),
                                            }
                                        }
                                    }
                                    if !hide_host_back_button {
                                        button {
                                            id: ui::id::CLOSE_INTEGRATION_MODULE_BUTTON,
                                            class: "input-box button button--secondary module-ui-schema__footer-nav",
                                            "data-ui-action": ui::action::CLOSE_INTEGRATION_MODULE,
                                            onclick: move |_| {
                                                if module_ui_page() != "main" {
                                                    module_ui_page.set("main".to_string());
                                                } else {
                                                    show_integration_module_prompt.set(false);
                                                    selected_integration_module_id.set(None);
                                                    module_ui_page.set("main".to_string());
                                                    module_host_dialog.set(None);
                                                    return_integration_prompt_to_module.set(false);
                                                    return_profile_export_to_module.set(false);
                                                }
                                            },
                                            "{dialog_back}"
                                        }
                                    }
                                }
                            }
                    }
                }
            }
        }

        if let Some(dialog) = module_host_dialog() {
            {
                let ok_dialog = dialog.clone();
                let cancel_dialog = dialog.clone();
                rsx! {
                    div {
                        class: "modal-backdrop module-host-dialog-backdrop",
                        div {
                            class: "modal modal--compact module-host-dialog",
                            "data-ui-entity": "module-host-dialog",
                            "data-ui-module-id": "{dialog.module_id}",
                            "data-ui-dialog-id": "{dialog.dialog_id}",
                            div { class: "modal__header",
                                div { class: "title-with-help",
                                    h2 { "{dialog.title}" }
                                }
                            }
                            div { class: "modal__body module-host-dialog__body",
                                div { class: "module-host-dialog__icon", "aria-hidden": "true",
                                    if let Some(icon_src) = dialog.icon_src.clone() {
                                        img {
                                            class: "module-host-dialog__icon-image",
                                            src: "{icon_src}",
                                            alt: ""
                                        }
                                    } else {
                                        span { class: "module-host-dialog__icon-fallback", "{dialog.icon_fallback}" }
                                    }
                                }
                                div { class: "module-host-dialog__message", "{dialog.message}" }
                            }
                            div { class: "modal__footer",
                                if dialog.show_cancel {
                                    button {
                                        class: "input-box button button--secondary",
                                        r#type: "button",
                                        onclick: move |_| {
                                            let delivery = watcher.write().send_integration_module_dialog_result(
                                                cancel_dialog.module_id.clone(),
                                                cancel_dialog.dialog_id.clone(),
                                                "cancel".to_string(),
                                            );
                                            match delivery {
                                                Ok(response) => {
                                                    push_status_history_line(
                                                        status_history,
                                                        format!("{}: dialog cancel", cancel_dialog.title),
                                                    );
                                                    let mut next_dialog = None;
                                                    for command in &response.commands {
                                                        if let Some(dialog) = module_host_dialog_from_owner_dialog(command, &cancel_dialog) {
                                                            next_dialog = Some(dialog);
                                                            break;
                                                        }
                                                    }
                                                    module_host_dialog.set(next_dialog);
                                                }
                                                Err(message) => {
                                                    push_status_history_error_line(status_history, message);
                                                    module_host_dialog.set(None);
                                                }
                                            }
                                        },
                                        "{dialog_cancel}"
                                    }
                                }
                                button {
                                    class: "input-box button button--primary",
                                    r#type: "button",
                                    onclick: move |_| {
                                            let delivery = watcher.write().send_integration_module_dialog_result(
                                                ok_dialog.module_id.clone(),
                                                ok_dialog.dialog_id.clone(),
                                                "ok".to_string(),
                                            );
                                            match delivery {
                                                Ok(response) => {
                                                    push_status_history_line(
                                                        status_history,
                                                        format!("{}: dialog ok", ok_dialog.title),
                                                    );
                                                    let mut next_dialog = None;
                                                    for command in &response.commands {
                                                        if let Some(dialog) = module_host_dialog_from_owner_dialog(command, &ok_dialog) {
                                                            next_dialog = Some(dialog);
                                                            break;
                                                        }
                                                    }
                                                    module_host_dialog.set(next_dialog);
                                                }
                                                Err(message) => {
                                                    push_status_history_error_line(status_history, message);
                                                    module_host_dialog.set(None);
                                                }
                                            }
                                        },
                                        "{dialog_ok}"
                                    }
                            }
                        }
                    }
                }
            }
        }

        if show_integration_prompt() {
            div {
                class: if return_integration_prompt_to_module() { "modal-backdrop module-overlay-backdrop" } else { "modal-backdrop" },
                div {
                    id: ui::id::INTEGRATION_ROOT_DIALOG,
                    class: "modal modal--panel",
                    "data-ui-entity": ui::entity::INTEGRATION_ROOT_DIALOG,
                    div { class: "modal__header",
                        div {
                            div { class: "title-with-help",
                                h2 { "{integration_root_dialog_title}" }
                                HelpIcon {
                                    icon_src: help_icon_src.clone(),
                                    tooltip: dialog_integration_help.clone(),
                                }
                            }
                        }
                    }
                    div { class: "modal__body",
                        div { class: "integration-dialog-status-list",
                            div { class: "integration-dialog-status-row",
                                span { class: "integration-dialog-status-row__label", "{dialog_integration_status}" }
                                span { class: "{integration_dialog_preview.status_class}", "{integration_dialog_preview.status_text}" }
                            }
                            if let Some(message) = integration_feedback() {
                                div { class: "integration-dialog-status-row",
                                    span { class: "integration-dialog-status-row__label", "{dialog_integration_validation}" }
                                    span { class: "integration-dialog-status-text integration-dialog-status-text--danger", "{message}" }
                                }
                            }
                            div { class: "integration-dialog-status-row",
                                span { class: "integration-dialog-status-row__label", "{integration_provider_label}" }
                                span { class: "integration-dialog-status-row__value", "{integration_dialog_preview.provider_name}" }
                            }
                            div { class: "integration-dialog-status-row",
                                span { class: "integration-dialog-status-row__label", "{dialog_integration_primary_path}:" }
                                input {
                                    class: "integration-dialog-path__input path-field",
                                    r#type: "text",
                                    readonly: true,
                                    value: "{integration_dialog_preview.repo_path}",
                                }
                            }
                            div { class: "integration-dialog-status-row",
                                span { class: "integration-dialog-status-row__label", "{dialog_integration_export_path}:" }
                                input {
                                    class: "integration-dialog-path__input path-field",
                                    r#type: "text",
                                    readonly: true,
                                    value: "{integration_dialog_preview.export_path}",
                                }
                            }
                            if integration_download_state.visible {
                                div { class: "integration-dialog-status-row integration-dialog-status-row--progress",
                                    div { class: "integration-dialog-progress-main",
                                        ProgressBar {
                                            label: dialog_integration_download_progress.clone(),
                                            meta: integration_download_meta.clone(),
                                            percent: integration_download_percent,
                                            stages: integration_progress_stages.clone(),
                                        }
                                    }
                                    button {
                                        id: ui::id::CANCEL_INTEGRATION_DOWNLOAD_BUTTON,
                                        class: "input-box button button--secondary",
                                        "data-ui-action": ui::action::CANCEL_INTEGRATION_DOWNLOAD,
                                        disabled: integration_download_cancel_disabled,
                                        onclick: move |_| {
                                            let mut state = integration_download();
                                            if !state.active {
                                                return;
                                            }
                                            state.active = false;
                                            state.cancelled = true;
                                            integration_download.set(state);
                                        },
                                        "{dialog_integration_cancel_download}"
                                    }
                                }
                            }
                        }
                        div { class: "button-row integration-download-actions",
                            for provider in integration_provider_buttons.iter() {
                                {
                                    let provider_id = provider.id.clone();
                                    let provider_name = provider.display_name.clone();
                                    let provider_tooltip = provider.display_name.clone();
                                    let provider_id_for_click = provider_id.clone();
                                    let progress_labels_for_click = integration_progress_labels.clone();
                                    let progress_label_for_click =
                                        dialog_integration_download_progress.clone();
                                    let module_id_for_download = selected_integration_module_id();
                                    rsx! {
                                        button {
                                            id: "netstitch-ui-download-integration-provider-{provider_id}",
                                            class: "input-box button button--secondary",
                                            "data-ui-action": ui::action::DOWNLOAD_INTEGRATION_PROVIDER,
                                            "aria-label": "{provider_tooltip}",
                                            "data-tooltip": "{provider_tooltip}",
                                            "data-tooltip-align": "start",
                                            onclick: move |_| {
                                                start_integration_download(
                                                    watcher,
                                                    integration_path_input,
                                                    integration_feedback,
                                                    integration_download,
                                                    status_history,
                                                    progress_labels_for_click.clone(),
                                                    progress_label_for_click.clone(),
                                                    module_id_for_download.clone(),
                                                    provider_id_for_click.clone(),
                                                );
                                            },
                                            "{provider_name}"
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "integration-folder-field",
                            label { class: "integration-folder-path-help", r#for: ui::id::INTEGRATION_ROOT_INPUT, "{dialog_integration_path_help}" }
                            div { class: "field-row integration-folder-field-row",
                                div { class: "path-input-shell",
                                    input {
                                        id: ui::id::INTEGRATION_ROOT_INPUT,
                                        class: "{integration_path_input_class}",
                                        "data-ui-entity": ui::control::INTEGRATION_ROOT_INPUT,
                                        r#type: "text",
                                        "data-committed-value": "{integration_path_input()}",
                                        placeholder: "{tracked_apps_path_placeholder}",
                                        "data-commit-on-enter": "true",
                                        "data-clear-button": "true",
                                        "data-preserve-draft": "true",
                                        onchange: move |event| {
                                            integration_path_input.set(event.value().to_string());
                                            integration_feedback.set(None);
                                        },
                                        onkeydown: move |event| {
                                            if event.key() != Key::Enter {
                                                return;
                                            }
                                            integration_path_applied.set(integration_path_input().trim().to_string());
                                            integration_feedback.set(None);
                                            pulse_text_input(input_apply_pulse, "integration-path");
                                        }
                                    }
                                    button {
                                        id: ui::id::CLEAR_INTEGRATION_ROOT_BUTTON,
                                        class: "path-input-clear",
                                        r#type: "button",
                                        disabled: clear_integration_path_disabled,
                                        "data-clear-button": "true",
                                        "data-ui-action": ui::action::CLEAR_INTEGRATION_ROOT,
                                        "aria-label": "{input_clear}",
                                        "data-tooltip": "{input_clear}",
                                        "data-tooltip-align": "end",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            integration_path_input.set(String::new());
                                            integration_path_applied.set(String::new());
                                            integration_feedback.set(None);
                                            pulse_text_input(input_apply_pulse, "integration-path");
                                        },
                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                    }
                                }
                                button {
                                    id: ui::id::BROWSE_INTEGRATION_ROOT_BUTTON,
                                    class: "input-box button button--secondary",
                                    "data-ui-action": ui::action::BROWSE_INTEGRATION_ROOT,
                                    onclick: move |_| {
                                        open_integration_folder_dialog(
                                            integration_path_input,
                                            integration_path_applied,
                                            integration_feedback,
                                            integration_folder_dialog_open,
                                            integration_picker_window.clone(),
                                            input_apply_pulse,
                                        );
                                    },
                                    "{dialog_integration_browse}"
                                }
                            }
                        }
                    }
                    div { class: "modal__footer",
                        button {
                            id: ui::id::CANCEL_INTEGRATION_ROOT_BUTTON,
                            class: "input-box button button--secondary",
                            "data-ui-action": ui::action::CANCEL_INTEGRATION_ROOT,
                            onclick: move |_| {
                                let should_return_to_module = return_integration_prompt_to_module();
                                show_integration_prompt.set(false);
                                return_integration_prompt_to_module.set(false);
                                if should_return_to_module {
                                    show_integration_module_prompt.set(true);
                                }
                            },
                            "{dialog_cancel}"
                        }
                        button {
                            id: ui::id::USE_INTEGRATION_ROOT_BUTTON,
                            class: "input-box button",
                            "data-ui-action": ui::action::USE_INTEGRATION_ROOT,
                            onclick: move |_| {
                                let candidate = integration_path_input().trim().to_string();
                                integration_path_applied.set(candidate.clone());
                                pulse_text_input(input_apply_pulse, "integration-path");
                                let module_id = selected_integration_module_id();
                                let mut current = watcher.write();
                                match current.configure_integration_folder(module_id, &candidate) {
                                    Ok(()) => {
                                        let should_return_to_module =
                                            return_integration_prompt_to_module();
                                        show_integration_prompt.set(false);
                                        return_integration_prompt_to_module.set(false);
                                        if should_return_to_module {
                                            show_integration_module_prompt.set(true);
                                        }
                                        integration_feedback.set(None);

                                        if pending_scan_after_setup() {
                                            current.start_monitoring();
                                            pending_scan_after_setup.set(false);
                                        }
                                    }
                                    Err(message) => {
                                        integration_feedback.set(Some(localize_integration_folder_error(
                                            &message,
                                            &dialog_integration_validation_folder_missing,
                                            &dialog_integration_validation_folder_invalid,
                                        )));
                                        show_integration_prompt.set(true);
                                    }
                                }
                            },
                            "{dialog_integration_use_folder}"
                        }
                    }
                }
            }
        }

        if show_domain_capture_admin_prompt() {
            div {
                class: if return_profile_export_to_module() { "modal-backdrop module-overlay-backdrop" } else { "modal-backdrop" },
                div {
                    id: ui::id::DOMAIN_CAPTURE_ADMIN_DIALOG,
                    class: "modal",
                    "data-ui-entity": ui::entity::DOMAIN_CAPTURE_ADMIN_DIALOG,
                    role: "dialog",
                    "aria-modal": "true",
                    div { class: "modal__header",
                        h2 { "{dialog_domain_capture_admin_title}" }
                    }
                    div { class: "modal__body",
                        p { "{dialog_domain_capture_admin_help}" }
                    }
                    div { class: "modal__footer",
                        button {
                            class: "input-box button",
                            "data-ui-action": ui::action::CLOSE_DOMAIN_CAPTURE_ADMIN,
                            onclick: move |_| show_domain_capture_admin_prompt.set(false),
                            "{dialog_domain_capture_admin_close}"
                        }
                    }
                }
            }
        }

        if show_cloud_sync_prompt() {
            div { class: "modal-backdrop workspace-overlay-backdrop cloud-sync-backdrop",
                div {
                    id: ui::id::CLOUD_SYNC_DIALOG,
                    class: "workspace-overlay-dialog cloud-sync-dialog",
                    "data-ui-entity": ui::entity::CLOUD_SYNC_DIALOG,
                    role: "dialog",
                    "aria-modal": "true",
                    div { class: "cloud-sync-dialog__body",
                        if cloud_overlay_mode() == CloudOverlayMode::Download {
                            section { class: "cloud-sync-panel cloud-sync-download-panel",
                            div { class: "card__header cloud-sync-panel__header",
                                h3 { "{dialog_cloud_sync_download_title}" }
                                span {
                                    class: "panel-header-meta cloud-sync-panel-service",
                                    "data-tooltip": "{cloud_base_url}",
                                    "data-tooltip-align": "end",
                                    span { class: "cloud-sync-panel-service__text", "{cloud_availability_value}" }
                                    span {
                                        class: "{cloud_availability_indicator_class}",
                                        "aria-hidden": "true",
                                    }
                                }
                            }
                            div { class: "cloud-sync-panel__body cloud-sync-download-panel__body",
                                div { class: "cloud-sync-download-subpanel cloud-sync-download-apps-subpanel",
                                    div { class: "cloud-sync-filter-grid",
                                        div { class: "cloud-sync-filter-block cloud-sync-filter-block--app",
                                            span { class: "header-filter-block__label", "{table_app}" }
                                            div { class: "path-input-shell cloud-sync-filter-field-shell",
                                                input {
                                                    id: ui::id::CLOUD_APP_SEARCH_INPUT,
                                                    class: "{cloud_app_search_class}",
                                                    "data-ui-entity": ui::control::CLOUD_APP_SEARCH_INPUT,
                                                    r#type: "text",
                                                    placeholder: "{dialog_cloud_sync_app_search}",
                                                    "data-committed-value": "{cloud_app_search_draft()}",
                                                    "data-commit-on-enter": "true",
                                                    "data-clear-button": "true",
                                                    "data-preserve-draft": "true",
                                                    onchange: move |event| {
                                                        let value = event.value().to_string();
                                                        cloud_app_search_draft.set(value.clone());
                                                        cloud_app_search.set(value);
                                                        cloud_selected_app_id.set(None);
                                                        cloud_loading_app_id.set(None);
                                                        pulse_text_input(input_apply_pulse, "cloud-app");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    onkeydown: move |event| {
                                                        if event.key() != Key::Enter {
                                                            return;
                                                        }
                                                        pulse_text_input(input_apply_pulse, "cloud-app");
                                                    }
                                                }
                                                button {
                                                    class: "path-input-clear",
                                                    r#type: "button",
                                                    disabled: clear_cloud_app_search_disabled,
                                                    "data-clear-button": "true",
                                                    "aria-label": "{input_clear}",
                                                    "data-tooltip": "{input_clear}",
                                                    "data-tooltip-align": "end",
                                                    onclick: move |event| {
                                                        event.stop_propagation();
                                                        cloud_app_search_draft.set(String::new());
                                                        cloud_app_search.set(String::new());
                                                        cloud_selected_app_id.set(None);
                                                        cloud_loading_app_id.set(None);
                                                        pulse_text_input(input_apply_pulse, "cloud-app");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                }
                                            }
                                        }
                                        div { class: "cloud-sync-filter-block cloud-sync-filter-block--company",
                                            span { class: "header-filter-block__label", "{dialog_cloud_sync_company}" }
                                            div { class: "path-input-shell cloud-sync-filter-field-shell",
                                                input {
                                                    class: "{cloud_publisher_search_class}",
                                                    r#type: "text",
                                                    placeholder: "{dialog_cloud_sync_publisher_search}",
                                                    "data-committed-value": "{cloud_publisher_search_draft()}",
                                                    "data-commit-on-enter": "true",
                                                    "data-clear-button": "true",
                                                    "data-preserve-draft": "true",
                                                    onchange: move |event| {
                                                        let value = event.value().to_string();
                                                        cloud_publisher_search_draft.set(value.clone());
                                                        cloud_publisher_search.set(value);
                                                        cloud_selected_app_id.set(None);
                                                        cloud_loading_app_id.set(None);
                                                        pulse_text_input(input_apply_pulse, "cloud-company");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    onkeydown: move |event| {
                                                        if event.key() != Key::Enter {
                                                            return;
                                                        }
                                                        pulse_text_input(input_apply_pulse, "cloud-company");
                                                    }
                                                }
                                                button {
                                                    class: "path-input-clear",
                                                    r#type: "button",
                                                    disabled: clear_cloud_publisher_search_disabled,
                                                    "data-clear-button": "true",
                                                    "aria-label": "{input_clear}",
                                                    "data-tooltip": "{input_clear}",
                                                    "data-tooltip-align": "end",
                                                    onclick: move |event| {
                                                        event.stop_propagation();
                                                        cloud_publisher_search_draft.set(String::new());
                                                        cloud_publisher_search.set(String::new());
                                                        cloud_selected_app_id.set(None);
                                                        cloud_loading_app_id.set(None);
                                                        pulse_text_input(input_apply_pulse, "cloud-company");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                }
                                            }
                                        }
                                        div { class: "cloud-sync-filter-block cloud-sync-filter-block--author",
                                            span { class: "header-filter-block__label", "{dialog_cloud_sync_source}" }
                                            div { class: "path-input-shell cloud-sync-filter-field-shell",
                                                input {
                                                    class: "{cloud_source_search_class}",
                                                    r#type: "text",
                                                    placeholder: "{dialog_cloud_sync_source_search}",
                                                    "data-committed-value": "{cloud_source_search_draft()}",
                                                    "data-commit-on-enter": "true",
                                                    "data-clear-button": "true",
                                                    "data-preserve-draft": "true",
                                                    onchange: move |event| {
                                                        let value = event.value().to_string();
                                                        cloud_source_search_draft.set(value.clone());
                                                        cloud_source_search.set(value);
                                                        pulse_text_input(input_apply_pulse, "cloud-author");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    onkeydown: move |event| {
                                                        if event.key() != Key::Enter {
                                                            return;
                                                        }
                                                        pulse_text_input(input_apply_pulse, "cloud-author");
                                                    }
                                                }
                                                button {
                                                    class: "path-input-clear",
                                                    r#type: "button",
                                                    disabled: clear_cloud_source_search_disabled,
                                                    "data-clear-button": "true",
                                                    "aria-label": "{input_clear}",
                                                    "data-tooltip": "{input_clear}",
                                                    "data-tooltip-align": "end",
                                                    onclick: move |event| {
                                                        event.stop_propagation();
                                                        cloud_source_search_draft.set(String::new());
                                                        cloud_source_search.set(String::new());
                                                        cloud_scope_mine.set(false);
                                                        pulse_text_input(input_apply_pulse, "cloud-author");
                                                        cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                                    },
                                                    img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                }
                                            }
                                        }
                                        div { class: "cloud-sync-filter-block cloud-sync-filter-block--visibility",
                                            span { class: "header-filter-block__label", "{dialog_cloud_sync_visibility_scope}" }
                                            select {
                                                class: "input-box select cloud-sync-filter-select",
                                                value: "{cloud_visibility_scope_value(cloud_visibility_scope_filter())}",
                                                "aria-label": "{dialog_cloud_sync_visibility_scope}",
                                                "data-tooltip": "{dialog_cloud_sync_visibility_scope}",
                                                "data-tooltip-align": "end",
                                                onchange: move |event| {
                                                    let scope = match event.value().as_str() {
                                                        "Public" => CloudObservationVisibilityScope::Public,
                                                        "Private" => CloudObservationVisibilityScope::Private,
                                                        _ => CloudObservationVisibilityScope::All,
                                                    };
                                                    cloud_visibility_scope_filter.set(scope);
                                                    cloud_selected_app_id.set(None);
                                                    cloud_loading_app_id.set(None);
                                                    let mut state = cloud_state();
                                                    state.downloaded_rows.clear();
                                                    state.selected_download_row_ids.clear();
                                                    cloud_state.set(state);
                                                    cloud_filter_generation.set(
                                                        cloud_filter_generation().wrapping_add(1),
                                                    );
                                                },
                                                option { value: "All", "{dialog_cloud_sync_visibility_all}" }
                                                option { value: "Public", "{dialog_cloud_sync_visibility_public}" }
                                                option { value: "Private", "{dialog_cloud_sync_visibility_private}" }
                                            }
                                        }
                                        button {
                                            id: ui::id::CLOUD_SCOPE_MINE_BUTTON,
                                            class: "{cloud_my_publications_button_class}",
                                            r#type: "button",
                                            "data-ui-action": ui::action::SELECT_CLOUD_SCOPE_MINE,
                                            disabled: cloud_session_login.is_none(),
                                            "aria-pressed": "{cloud_my_publications_active}",
                                            "aria-label": "{dialog_cloud_sync_scope_mine}",
                                            "data-tooltip": "{dialog_cloud_sync_scope_mine}",
                                            "data-tooltip-align": "end",
                                            onclick: move |_| {
                                                let next_scope = !cloud_my_publications_active;
                                                cloud_scope_mine.set(next_scope);
                                                if next_scope {
                                                    let author = cloud_author_filter_display.trim().to_string();
                                                    if !author.is_empty() {
                                                        cloud_source_search_draft.set(author.clone());
                                                        cloud_source_search.set(author);
                                                        pulse_text_input(input_apply_pulse, "cloud-author");
                                                    }
                                                }
                                                cloud_filter_generation.set(cloud_filter_generation().wrapping_add(1));
                                            },
                                            img {
                                                class: "button__icon button__icon--my-publications",
                                                src: "{my_publications_button_src}",
                                                alt: ""
                                            }
                                        }
                                    }
                                    div { class: "table-wrap cloud-sync-app-list",
                                        div { class: "table-header-wrap",
                                            div { class: "table-header-scroll",
                                                table { class: "cloud-sync-table cloud-sync-app-table cloud-sync-app-table--header",
                                                    thead {
                                                        tr {
                                                            SortHeaderCell { label: table_app.clone(), tooltip: table_app.clone(), sort_state: cloud_app_sort().indicator_state(CloudAppSortColumn::App), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_app_sort.set(cloud_app_sort().toggled(CloudAppSortColumn::App)) }
                                                            SortHeaderCell { label: dialog_cloud_sync_company.clone(), tooltip: dialog_cloud_sync_company.clone(), sort_state: cloud_app_sort().indicator_state(CloudAppSortColumn::Company), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_app_sort.set(cloud_app_sort().toggled(CloudAppSortColumn::Company)) }
                                                            SortHeaderCell { label: dialog_cloud_sync_available_rows.clone(), tooltip: dialog_cloud_sync_available_rows.clone(), sort_state: cloud_app_sort().indicator_state(CloudAppSortColumn::AvailableRows), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_app_sort.set(cloud_app_sort().toggled(CloudAppSortColumn::AvailableRows)) }
                                                            SortHeaderCell { label: dialog_cloud_sync_stats.clone(), tooltip: dialog_cloud_sync_stats.clone(), sort_state: cloud_app_sort().indicator_state(CloudAppSortColumn::Authors), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_app_sort.set(cloud_app_sort().toggled(CloudAppSortColumn::Authors)) }
                                                            th { class: "cloud-sync-action-header", "aria-hidden": "true", "" }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "table-header-scrollbar-fill", "aria-hidden": "true" }
                                        }
                                        div { class: "table-body-wrap",
                                            table { class: "cloud-sync-table cloud-sync-app-table cloud-sync-app-table--body",
                                                tbody {
                                                    if cloud_apps.is_empty() && cloud_status.service_available.is_none() {
                                                        tr { td { class: "cloud-sync-empty-cell", colspan: "5", "{dialog_cloud_sync_apps_not_loaded}" } }
                                                    } else if cloud_apps.is_empty() {
                                                        tr { td { class: "cloud-sync-empty-cell", colspan: "5", "{dialog_cloud_sync_apps_empty}" } }
                                                    }
                                                    for app in cloud_apps.clone() {
                                                        CloudAppCatalogRowView {
                                                            key: "{app.app_id}",
                                                            app: app.clone(),
                                                            selected: cloud_selected_app_id().as_deref() == Some(app.app_id.as_str()),
                                                            loading: cloud_loading_app_id().as_deref() == Some(app.app_id.as_str()),
                                                            filters: current_cloud_filters(
                                                                &cloud_app_search,
                                                                &cloud_publisher_search,
                                                                &cloud_ip_search,
                                                                &cloud_domain_search,
                                                                &cloud_port_search,
                                                                &cloud_protocol_filter,
                                                                &cloud_source_search,
                                                                &cloud_selected_app_id,
                                                                &cloud_scope_mine,
                                                                &cloud_visibility_scope_filter,
                                                            ),
                                                            download_label: dialog_cloud_sync_download_action.clone(),
                                                            started_message: dialog_cloud_sync_download_started.clone(),
                                                            message_labels: cloud_message_labels.clone(),
                                                            cloud_import_button_src: cloud_import_button_src.clone(),
                                                            cloud_state,
                                                            cloud_loading_app_id,
                                                            cloud_selected_app_id,
                                                            footer_progress: cloud_download_progress,
                                                            status_history,
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "cloud-sync-download-subpanel cloud-sync-download-rows-subpanel",
                                div { class: "table-wrap cloud-sync-staging-table",
                                    div { class: "table-header-wrap",
                                        div { class: "table-header-scroll",
                                            table { class: "cloud-sync-table cloud-sync-staging-data-table cloud-sync-staging-data-table--header",
                                                thead {
                                                    tr {
                                                        SortHeaderCell { label: table_app.clone(), tooltip: table_app.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::App), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::App)) }
                                                        SortHeaderCell { label: table_ip.clone(), tooltip: table_ip.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Ip), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Ip)) }
                                                        SortHeaderCell { label: table_domain.clone(), tooltip: table_domain_tooltip.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Domain), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Domain)) }
                                                        SortHeaderCell { label: table_port.clone(), tooltip: table_port.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Port), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Port)) }
                                                        SortHeaderCell { label: table_proto.clone(), tooltip: filter_protocol.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Protocol), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Protocol)) }
                                                        SortHeaderCell { label: table_conn.clone(), tooltip: table_conn_tooltip.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Connection), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Connection)) }
                                                        SortHeaderCell { label: table_hits.clone(), tooltip: table_hits_tooltip.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Hits), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Hits)) }
                                                        SortHeaderCell { label: dialog_cloud_sync_source.clone(), tooltip: dialog_cloud_sync_source.clone(), sort_state: cloud_row_sort().indicator_state(CloudRowSortColumn::Source), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_row_sort.set(cloud_row_sort().toggled(CloudRowSortColumn::Source)) }
                                                        th {
                                                            class: "cloud-sync-privacy-header",
                                                            "data-tooltip": "{dialog_cloud_sync_privacy_column_tooltip}",
                                                            "data-tooltip-align": "end",
                                                            "{dialog_cloud_sync_privacy_column}"
                                                        }
                                                        th { class: "cloud-sync-action-header", "aria-hidden": "true", "" }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "table-header-scrollbar-fill", "aria-hidden": "true" }
                                    }
                                    div { class: "table-body-wrap",
                                        table { class: "cloud-sync-table cloud-sync-staging-data-table cloud-sync-staging-data-table--body",
                                            tbody {
                                                if cloud_status.downloaded_rows.is_empty() {
                                                    tr { td { class: "cloud-sync-empty-cell", colspan: "10", "{dialog_cloud_sync_no_downloaded_rows}" } }
                                                }
                                                for item in cloud_downloaded_rows.clone() {
                                                    tr {
                                                        class: if cloud_status.selected_download_row_ids.contains(&item.row_id) { "observation-row observation-row--confirmed cloud-sync-staging-row" } else { "observation-row cloud-sync-staging-row" },
                                                        onclick: {
                                                            let row_items = cloud_downloaded_rows_for_selection.clone();
                                                            let row_id = item.row_id.clone();
                                                            move |event: MouseEvent| {
                                                                let modifiers = event.modifiers();
                                                                apply_cloud_download_selection_click(
                                                                    cloud_state,
                                                                    last_cloud_download_selection_anchor,
                                                                    row_items.as_slice(),
                                                                    &row_id,
                                                                    modifiers.contains(Modifiers::CONTROL),
                                                                    modifiers.contains(Modifiers::SHIFT),
                                                                );
                                                            }
                                                        },
                                                        td { "{item.app_display_name}" }
                                                        td { "{item.row.remote_ip}" }
                                                        td { "{item.domain_label}" }
                                                        td { "{item.row.remote_port}" }
                                                        td { "{item.protocol_label}" }
                                                        td { "{item.connection_label}" }
                                                        td { "{item.requests_label}" }
                                                        td { "{item.source_label}" }
                                                        td {
                                                            class: "cloud-sync-privacy-cell",
                                                            "data-tooltip": "{dialog_cloud_sync_privacy_column_tooltip}",
                                                            "data-tooltip-align": "end",
                                                            "{item.privacy_label}"
                                                        }
                                                        td {
                                                            div { class: "table-actions",
                                                                button {
                                                                    class: "input-box button button--icon button--square table-action-button",
                                                                    "data-ui-action": ui::action::TOGGLE_CLOUD_DOWNLOAD_ROW_SELECTION,
                                                                    "data-ui-key": "{item.row_id}",
                                                                    "aria-label": if cloud_status.selected_download_row_ids.contains(&item.row_id) { "{dialog_cloud_sync_unselect_row}" } else { "{dialog_cloud_sync_select_row}" },
                                                                    "data-tooltip": if cloud_status.selected_download_row_ids.contains(&item.row_id) { "{dialog_cloud_sync_unselect_row_tooltip}" } else { "{dialog_cloud_sync_select_row_tooltip}" },
                                                                    "data-tooltip-align": "end",
                                                                    onclick: {
                                                                        let row_id = item.row_id.clone();
                                                                        move |event: MouseEvent| {
                                                                            event.stop_propagation();
                                                                            toggle_cloud_download_row_selection(
                                                                                cloud_state,
                                                                                last_cloud_download_selection_anchor,
                                                                                row_id.clone(),
                                                                            );
                                                                        }
                                                                    },
                                                                    img {
                                                                        class: if cloud_status.selected_download_row_ids.contains(&item.row_id) {
                                                                            "button__icon button__icon--unconfirm-filtered table-action-button__icon"
                                                                        } else {
                                                                            "button__icon button__icon--confirm-filtered table-action-button__icon"
                                                                        },
                                                                        src: if cloud_status.selected_download_row_ids.contains(&item.row_id) { "{unconfirm_filtered_button_src}" } else { "{confirm_filtered_button_src}" },
                                                                        alt: "",
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            }
                            div { class: "cloud-sync-panel__footer",
                                span {
                                    class: "panel-footer-meta cloud-sync-panel__footer-summary",
                                    span { class: "panel-footer-meta__item", "{observations_total}: {cloud_status.downloaded_rows.len()}" }
                                    span { class: "panel-footer-meta__item", "{observations_selected}: {cloud_status.selected_download_row_ids.len()}" }
                                    span {
                                        class: "panel-footer-meta__item",
                                        "data-tooltip": "{cloud_download_quota_tooltip}",
                                        "data-tooltip-align": "start",
                                        span { class: "cloud-sync-footer-quota__label", "{dialog_cloud_sync_download}:" }
                                        span { class: "{cloud_download_quota_value_class}", "{cloud_download_quota_value}" }
                                    }
                                }
                                if let Some(progress) = cloud_download_progress() {
                                    div {
                                        id: ui::id::CLOUD_DOWNLOAD_PROGRESS_BAR,
                                        class: "cloud-sync-panel__footer-progress",
                                        "data-ui-entity": ui::entity::PROGRESS_BAR,
                                        "data-ui-key": "cloud-download-footer-progress",
                                        ProgressBar {
                                            label: progress.label,
                                            percent: progress.percent,
                                            meta: progress.meta,
                                            stages: progress.stages,
                                            compact: true,
                                            hide_label: true,
                                        }
                                    }
                                }
                                div { class: "cloud-sync-panel__footer-actions",
                                    button {
                                        id: ui::id::CLOUD_DOWNLOAD_ADD_TO_MONITORING_BUTTON,
                                        class: "input-box button cloud-sync-panel__action",
                                        "data-ui-action": ui::action::ADD_CLOUD_DOWNLOAD_ROWS_TO_MONITORING,
                                        disabled: cloud_import_loading || cloud_status.selected_download_row_ids.is_empty(),
                                        onclick: move |_| {
                                            let rows = cloud_import_rows_for_add_to_monitoring(&cloud_state());
                                            if rows.is_empty() {
                                                return;
                                            }
                                            match watcher.write().import_monitoring_csv(MonitoringCsvImportRequestDto {
                                                import_source: MonitoringImportSourceDto::CloudDownload,
                                                rows,
                                            }) {
                                                Ok(result) => push_status_history_success_line(status_history, cloud_import_status_line(&dialog_cloud_sync_import_done_detail, &result)),
                                                Err(error) => push_status_history_error_line(status_history, error),
                                            }
                                        },
                                        "{dialog_cloud_sync_add_to_monitoring}"
                                    }
                                    button {
                                        id: ui::id::CLOUD_DOWNLOAD_EXPORT_CSV_BUTTON,
                                        class: "input-box button cloud-sync-panel__action",
                                        "data-ui-action": ui::action::EXPORT_CLOUD_DOWNLOAD_ROWS_CSV,
                                        disabled: cloud_import_loading || cloud_status.selected_download_row_ids.is_empty(),
                                        onclick: move |_| {
                                            let rows = selected_cloud_csv_export_rows(&cloud_state());
                                            open_csv_export_file_dialog(
                                                rows,
                                                status_history,
                                                cloud_csv_export_completed.clone(),
                                                cloud_csv_export_failed.clone(),
                                                csv_export_file_dialog_open,
                                                cloud_csv_export_picker_window.clone(),
                                                watcher.read().live_base_url(),
                                            );
                                        },
                                        "{dialog_cloud_sync_export_selected}"
                                    }
                                }
                            }
                            }
                        }
                        if cloud_overlay_mode() == CloudOverlayMode::Upload {
                            section { class: "cloud-sync-panel cloud-sync-upload-panel",
                            div { class: "card__header cloud-sync-panel__header",
                                h3 { "{dialog_cloud_sync_upload_title}" }
                                span {
                                    class: "panel-header-meta cloud-sync-panel-service",
                                    "data-tooltip": "{cloud_base_url}",
                                    "data-tooltip-align": "end",
                                    span { class: "cloud-sync-panel-service__text", "{cloud_availability_value}" }
                                    span {
                                        class: "{cloud_availability_indicator_class}",
                                        "aria-hidden": "true",
                                    }
                                }
                            }
                            div { class: "cloud-sync-panel__body cloud-sync-upload-panel__body",
                                div { class: "cloud-sync-upload-auth-subpanel",
                                div { class: "cloud-sync-auth-row",
                                    button {
                                        id: ui::id::START_CLOUD_GOOGLE_OAUTH_BUTTON,
                                        class: "input-box button button--secondary",
                                        "data-ui-action": ui::action::START_CLOUD_GOOGLE_OAUTH,
                                        disabled: cloud_session_login.is_some(),
                                        onclick: move |_| {
                                            start_cloud_google_auth(
                                                cloud_state,
                                                cloud_upload_nickname,
                                                cloud_nickname_check_status,
                                                cloud_nickname_check_generation,
                                                status_history,
                                                current_cloud_filters(
                                                    &cloud_app_search,
                                                    &cloud_publisher_search,
                                                    &cloud_ip_search,
                                                    &cloud_domain_search,
                                                    &cloud_port_search,
                                                    &cloud_protocol_filter,
                                                    &cloud_source_search,
                                                    &cloud_selected_app_id,
                                                    &cloud_scope_mine,
                                                    &cloud_visibility_scope_filter,
                                                ),
                                                selected_language(),
                                                dialog_cloud_sync_sign_in_started.clone(),
                                                dialog_cloud_sync_sign_in_done.clone(),
                                                cloud_credentials_save_failed_for_google.clone(),
                                                cloud_message_labels_for_google_auth.clone(),
                                            );
                                        },
                                        "{dialog_cloud_sync_authorization}"
                                    }
                                    button {
                                        id: ui::id::SIGN_OUT_CLOUD_BUTTON,
                                        class: "input-box button button--secondary",
                                        "data-ui-action": ui::action::SIGN_OUT_CLOUD,
                                        disabled: cloud_session_login.is_none(),
                                        onclick: move |_| {
                                            if let Err(error) = clear_persisted_cloud_auth_state() {
                                                push_status_history_line(status_history, error);
                                            }
                                            let mut state = cloud_state();
                                            state.auth_generation = state.auth_generation.wrapping_add(1);
                                            state.session = None;
                                            state.client_private_key_pkcs8_der = None;
                                            state.my_apps.clear();
                                            state.uploaded_observation_ids.clear();
                                            state.last_error = None;
                                            state.last_response_json = None;
                                            cloud_state.set(state);
                                            push_status_history_line(
                                                status_history,
                                                dialog_cloud_sync_signed_out.clone(),
                                            );
                                        },
                                        "{dialog_cloud_sync_sign_out}"
                                    }
                                }
                                div { class: "cloud-sync-nickname-block",
                                    div { class: "cloud-sync-nickname-row",
                                        input {
                                            class: "input-box input",
                                            r#type: "text",
                                            maxlength: "32",
                                            placeholder: "{dialog_cloud_sync_nickname}",
                                            "data-committed-value": "{cloud_upload_nickname_draft()}",
                                            "data-commit-on-enter": "true",
                                            "data-preserve-draft": "true",
                                            onchange: move |event| {
                                                let value = event.value().to_string();
                                                cloud_upload_nickname_draft.set(value.clone());
                                                cloud_upload_nickname_dirty.set(false);
                                                cloud_upload_nickname.set(value.clone());
                                                cloud_nickname_check_status.set(
                                                    if value.trim().is_empty() {
                                                        CloudNicknameCheckStatus::Idle
                                                    } else {
                                                        CloudNicknameCheckStatus::Checking
                                                    },
                                                );
                                                cloud_nickname_check_generation.set(
                                                    cloud_nickname_check_generation().wrapping_add(1),
                                                );
                                                if value.trim().is_empty() || validate_author_signature(&value).is_ok() {
                                                    let _ = persist_cloud_upload_nickname_to_sqlite(&value);
                                                }
                                            },
                                            onkeydown: move |event| {
                                                if event.key() != Key::Enter {
                                                    return;
                                                }
                                                pulse_text_input(input_apply_pulse, "cloud-author");
                                            }
                                        }
                                        label {
                                            class: "cloud-sync-private-row",
                                            "data-tooltip": "{dialog_cloud_sync_private_upload_tooltip}",
                                            "data-tooltip-align": "end",
                                            input {
                                                r#type: "checkbox",
                                                checked: cloud_upload_private(),
                                                onchange: move |event| {
                                                    cloud_upload_private.set(event.checked());
                                                }
                                            }
                                            "{dialog_cloud_sync_private_upload}"
                                        }
                                    }
                                    if !cloud_upload_nickname_status_label.is_empty() {
                                        span {
                                            class: "{cloud_upload_nickname_status_class}",
                                            "{cloud_upload_nickname_status_label}"
                                        }
                                    }
                                }
                                }
                                div { class: "cloud-sync-upload-publications-subpanel",
                                    div { class: "cloud-sync-publications-title",
                                        "{dialog_cloud_sync_publications}"
                                    }
                                    div { class: "table-wrap cloud-sync-publications-table",
                                        div { class: "table-header-wrap",
                                            div { class: "table-header-scroll",
                                                table { class: "observations-table observations-table--header cloud-sync-publications-data-table cloud-sync-publications-data-table--header",
                                                thead {
                                                    tr {
                                                        SortHeaderCell { label: table_app.clone(), tooltip: table_app.clone(), sort_state: cloud_publication_sort().indicator_state(CloudPublicationSortColumn::App), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_publication_sort.set(cloud_publication_sort().toggled(CloudPublicationSortColumn::App)) }
                                                        SortHeaderCell { label: dialog_cloud_sync_new_rows.clone(), tooltip: dialog_cloud_sync_new_rows.clone(), sort_state: cloud_publication_sort().indicator_state(CloudPublicationSortColumn::NewRows), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_publication_sort.set(cloud_publication_sort().toggled(CloudPublicationSortColumn::NewRows)) }
                                                        SortHeaderCell { label: dialog_cloud_sync_non_public_rows.clone(), tooltip: dialog_cloud_sync_non_public_rows_tooltip.clone(), sort_state: cloud_publication_sort().indicator_state(CloudPublicationSortColumn::NonPublicRows), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_publication_sort.set(cloud_publication_sort().toggled(CloudPublicationSortColumn::NonPublicRows)) }
                                                        SortHeaderCell { label: dialog_cloud_sync_author_rows.clone(), tooltip: dialog_cloud_sync_author_rows.clone(), sort_state: cloud_publication_sort().indicator_state(CloudPublicationSortColumn::AuthorRows), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_publication_sort.set(cloud_publication_sort().toggled(CloudPublicationSortColumn::AuthorRows)) }
                                                        SortHeaderCell { label: dialog_cloud_sync_total_rows.clone(), tooltip: dialog_cloud_sync_total_rows.clone(), sort_state: cloud_publication_sort().indicator_state(CloudPublicationSortColumn::TotalRows), sort_idle_icon_src: sort_indicator_idle_src.clone(), sort_asc_icon_src: sort_indicator_asc_src.clone(), sort_desc_icon_src: sort_indicator_desc_src.clone(), on_click: move |_| cloud_publication_sort.set(cloud_publication_sort().toggled(CloudPublicationSortColumn::TotalRows)) }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "table-header-scrollbar-fill", "aria-hidden": "true" }
                                    }
                                    div { class: "table-body-wrap",
                                        table { class: "observations-table observations-table--body cloud-sync-publications-data-table cloud-sync-publications-data-table--body",
                                            tbody {
                                                if cloud_status.session.is_none() {
                                                    tr { td { class: "cloud-sync-empty-cell", colspan: "5", "{dialog_cloud_sync_my_apps_not_loaded}" } }
                                                } else if cloud_author_publication_rows.is_empty() {
                                                    tr { td { class: "cloud-sync-empty-cell", colspan: "5", "{dialog_cloud_sync_apps_empty}" } }
                                                } else {
                                                    for app in cloud_author_publication_rows.clone() {
                                                        tr { class: "observation-row cloud-sync-publication-row",
                                                            td { "{app.app_name}" }
                                                            td {
                                                                if app.new_rows > 0 {
                                                                    span { class: "state-label state-label--success", "{app.new_rows}" }
                                                                }
                                                            }
                                                            td {
                                                                if app.non_public_rows > 0 {
                                                                    span {
                                                                        class: "state-label state-label--warning",
                                                                        "{app.non_public_rows}"
                                                                    }
                                                                }
                                                            }
                                                            td { "{app.author_rows}" }
                                                            td { "{app.total_rows_label}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                }
                            }
                            div { class: "cloud-sync-panel__footer",
                                span {
                                    class: "cloud-sync-footer-quota",
                                    "data-tooltip": "{cloud_upload_quota_tooltip}",
                                    "data-tooltip-align": "start",
                                    span { class: "cloud-sync-footer-quota__label", "{dialog_cloud_sync_upload}:" }
                                    span { class: "{cloud_upload_quota_value_class}", "{cloud_upload_quota_value}" }
                                }
                                if let Some(progress) = cloud_upload_progress() {
                                    div {
                                        id: ui::id::CLOUD_UPLOAD_PROGRESS_BAR,
                                        class: "cloud-sync-panel__footer-progress",
                                        "data-ui-entity": ui::entity::PROGRESS_BAR,
                                        "data-ui-key": "cloud-upload-footer-progress",
                                        ProgressBar {
                                            label: progress.label,
                                            percent: progress.percent,
                                            meta: progress.meta,
                                            stages: progress.stages,
                                            compact: true,
                                            hide_label: true,
                                        }
                                    }
                                }
                                span {
                                    class: "cloud-sync-footer-identifier",
                                    "data-tooltip": "{cloud_identifier_tooltip}",
                                    "data-tooltip-align": "start",
                                    role: "button",
                                    tabindex: "0",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        if !cloud_identifier_copy_value.is_empty() {
                                            copy_text_to_clipboard(&cloud_identifier_clipboard_window, &cloud_identifier_copy_value);
                                            push_status_history_line(status_history, dialog_cloud_sync_identifier_copied.clone());
                                        }
                                    },
                                    span { class: "cloud-sync-footer-identifier__label", "{dialog_cloud_sync_identifier}:" }
                                    span { class: "cloud-sync-footer-identifier__value", "{cloud_identifier_value}" }
                                }
                                span {
                                    class: "cloud-sync-footer-auth",
                                    "data-tooltip": "{dialog_cloud_sync_authorization_tooltip}",
                                    "data-tooltip-align": "end",
                                    span { class: "cloud-sync-footer-identifier__label", "{dialog_cloud_sync_authorization}:" }
                                    span { class: "cloud-sync-footer-identifier__value", "{cloud_authorization_value}" }
                                }
                                button {
                                    id: ui::id::CLOUD_UPLOAD_BUTTON,
                                    class: "input-box button cloud-sync-panel__action",
                                    "data-ui-action": ui::action::UPLOAD_CLOUD_DATA,
                                    disabled: cloud_session_login.is_none()
                                        || !cloud_upload_nickname_valid
                                        || cloud_upload_nickname_blocks_upload
                                        || cloud_upload_quota_exhausted
                                        || cloud_upload_progress().is_some()
                                        || !cloud_upload_has_candidates,
                                    onclick: {
                                        let upload_pending_confirm =
                                            pending_observation_selection_confirm.clone();
                                        move |_| {
                                        flush_observation_selection_confirm(
                                            &mut watcher.write(),
                                            &upload_pending_confirm,
                                        );
                                        start_cloud_upload(
                                            cloud_state,
                                            cloud_upload_progress,
                                            status_history,
                                            watcher.read().snapshot(),
                                            selected_observation_ids.clone(),
                                            cloud_upload_nickname(),
                                            if cloud_upload_private() {
                                                CloudObservationVisibility::Private
                                            } else {
                                                CloudObservationVisibility::Public
                                            },
                                            dialog_cloud_sync_upload_started.clone(),
                                            cloud_message_labels_for_upload_action.clone(),
                                        );
                                        }
                                    },
                                    "{dialog_cloud_sync_upload_action}"
                                }
                            }
                            }
                        }
                    }
                }
            }
        }

        if show_information_prompt() {
            div { class: "modal-backdrop",
                div {
                    id: ui::id::INFORMATION_DIALOG,
                    class: "modal modal--panel information-dialog",
                    "data-ui-entity": ui::entity::INFORMATION_DIALOG,
                    role: "dialog",
                    "aria-modal": "true",
                    div { class: "modal__header",
                        h2 { "{dialog_info_title}" }
                    }
                    div { class: "modal__body information-dialog__body",
                        section { class: "info-panel info-panel--about",
                            div { class: "info-panel__header",
                                h3 { "{dialog_info_about_title}" }
                            }
                            div { class: "info-panel__body info-about-grid",
                                div { class: "info-subpanel info-logo-panel",
                                    img {
                                        class: "info-logo",
                                        src: "{app_logo_src}",
                                        alt: "NetStitch"
                                    }
                                }
                                div { class: "info-subpanel info-about-text",
                                    dl { class: "info-meta-list",
                                        div { class: "info-meta-row",
                                            dt { "{dialog_info_product_name}" }
                                            dd { "NetStitch" }
                                        }
                                        div { class: "info-meta-row",
                                            dt { "{dialog_info_author}" }
                                            dd { "{app_author}" }
                                        }
                                        div { class: "info-meta-row",
                                            dt { "{dialog_info_version}" }
                                            dd { "v{runtime_build_version()}" }
                                        }
                                        div { class: "info-meta-row",
                                            dt { "{dialog_info_email}" }
                                            dd {
                                                a {
                                                    class: "info-link",
                                                    href: "{app_email_href}",
                                                    "{app_email}"
                                                }
                                            }
                                        }
                                        div { class: "info-meta-row",
                                            dt { "{dialog_info_repository}" }
                                            dd {
                                                a {
                                                    class: "info-link",
                                                    href: "{repository_url}",
                                                    target: "_blank",
                                                    rel: "noreferrer",
                                                    "{repository_url}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "info-panel__footer",
                                "{dialog_cloud_sync_identifier}: {cloud_identifier_value}"
                            }
                        }
                        section { class: "info-panel info-panel--details",
                            div { class: "info-panel__header",
                                h3 { "{dialog_info_info_title}" }
                            }
                            div { class: "info-panel__body info-panel__body--details",
                                div { class: "info-subpanel info-details-subpanel",
                                    div { class: "info-details-scroll",
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_monitoring}" }
                                            ul {
                                                li { "{dialog_info_monitoring_1}" }
                                                li { "{dialog_info_monitoring_2}" }
                                                li { "{dialog_info_monitoring_5}" }
                                                li { "{dialog_info_monitoring_3}" }
                                                li { "{dialog_info_monitoring_4}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_domains}" }
                                            ul {
                                                li { "{dialog_info_domains_1}" }
                                                li { "{dialog_info_domains_2}" }
                                                li { "{dialog_info_domains_4}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_csv}" }
                                            ul {
                                                li { "{dialog_info_csv_1}" }
                                                li { "{dialog_info_csv_2}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_cloud_sync}" }
                                            ul {
                                                li { "{dialog_info_cloud_sync_1}" }
                                                li { "{dialog_info_cloud_sync_2}" }
                                                li { "{dialog_info_cloud_sync_3}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_web_access}" }
                                            ul {
                                                li { "{dialog_info_web_access_1}" }
                                                li { "{dialog_info_web_access_2}" }
                                                li { "{dialog_info_web_access_3}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_modules}" }
                                            ul {
                                                li { "{dialog_info_modules_1}" }
                                                li { "{dialog_info_modules_2}" }
                                                li { "{dialog_info_modules_3}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_tracked_apps}" }
                                            ul {
                                                li { "{dialog_info_tracked_apps_1}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            h4 { "{dialog_info_group_ignored_addresses}" }
                                            ul {
                                                li { "{dialog_info_ignored_addresses_1}" }
                                                li { "{dialog_info_ignored_addresses_2}" }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "info-panel__footer info-panel__footer--success",
                                "{dialog_info_info_footer}"
                            }
                        }
                    }
                    div { class: "modal__footer",
                        if app_update_available {
                            button {
                                id: ui::id::UPDATE_APPLICATION_BUTTON,
                                class: "input-box button button--primary",
                                "data-ui-action": ui::action::UPDATE_APPLICATION,
                                onclick: {
                                    let update_url = app_update_url.clone();
                                    move |_| {
                                        if let Err(message) = open_browser_url(&update_url) {
                                            push_status_history_line(status_history, message);
                                        }
                                    }
                                },
                                "{dialog_info_update_program}"
                            }
                        }
                        button {
                            class: "input-box button button--secondary",
                            "data-ui-action": ui::action::CLOSE_INFORMATION,
                            onclick: move |_| show_information_prompt.set(false),
                            "{dialog_info_close}"
                        }
                    }
                }
            }
        }

        if show_profile_export_prompt() {
            div { class: "modal-backdrop workspace-overlay-backdrop",
                div {
                    id: ui::id::PROFILE_EXPORT_DIALOG,
                    class: "modal modal--panel workspace-overlay-dialog profile-export-dialog",
                    "data-ui-entity": ui::entity::PROFILE_EXPORT_DIALOG,
                        div { class: "modal__header",
                            div { class: "title-with-help",
                            h2 { "{profile_export_dialog_title}" }
                            HelpIcon {
                                icon_src: help_icon_src.clone(),
                                tooltip: dialog_export_help.clone(),
                            }
                        }
                    }
                    div { class: "modal__body",
                        div { class: "profile-export-section",
                            div { class: "profile-export-field profile-export-field--wide",
                                div {
                                    class: "tabs profile-export-tabs",
                                    "data-ui-entity": ui::entity::TABS,
                                    div {
                                        id: ui::id::PROFILE_EXPORT_TABS,
                                        class: "tabs__list",
                                        role: "tablist",
                                        button {
                                            id: ui::id::PROFILE_EXPORT_MODE_ATTACH_BUTTON,
                                            class: "{profile_export_attach_class}",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": "{profile_export_mode_value == ExportModeDto::AttachNetstitchLists}",
                                            "data-ui-action": ui::action::SELECT_PROFILE_EXPORT_MODE,
                                            onclick: move |_| {
                                                profile_export_mode.set(ExportModeDto::AttachNetstitchLists);
                                                profile_export_feedback.set(None);
                                                profile_export_preview.set(None);
                                            },
                                            "{dialog_export_mode_attach}"
                                        }
                                        button {
                                            id: ui::id::PROFILE_EXPORT_MODE_PATCH_BUTTON,
                                            class: "{profile_export_patch_class}",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": "{profile_export_mode_value == ExportModeDto::PatchSelectedProfile}",
                                            "data-ui-action": ui::action::SELECT_PROFILE_EXPORT_MODE,
                                            onclick: move |_| {
                                                profile_export_mode.set(ExportModeDto::PatchSelectedProfile);
                                                profile_export_feedback.set(None);
                                                profile_export_preview.set(None);
                                            },
                                            "{dialog_export_mode_patch}"
                                        }
                                        button {
                                            id: ui::id::PROFILE_EXPORT_MODE_MERGE_BUTTON,
                                            class: "{profile_export_merge_class}",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": "{profile_export_mode_value == ExportModeDto::MergeIntoExistingLists}",
                                            "data-ui-action": ui::action::SELECT_PROFILE_EXPORT_MODE,
                                            onclick: move |_| {
                                                profile_export_mode.set(ExportModeDto::MergeIntoExistingLists);
                                                profile_export_feedback.set(None);
                                                profile_export_preview.set(None);
                                            },
                                            "{dialog_export_mode_merge}"
                                        }
                                    }
                                    div {
                                        id: ui::id::PROFILE_EXPORT_TABS_BODY,
                                        class: "tabs__body profile-export-tabs-body",
                                        "data-ui-entity": ui::entity::TABS_BODY,
                                        div { class: "tabs__panel", role: "tabpanel",
                                            if profile_export_mode_value == ExportModeDto::AttachNetstitchLists {
                                                div { class: "profile-export-grid profile-export-grid--attach",
                                        div { class: "profile-export-field",
                                            label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_PROFILE_INPUT, "{dialog_export_profile_path}" }
                                            div { class: "field-row profile-export-path-row",
                                                div { class: "path-input-shell",
                                                    input {
                                                        id: ui::id::PROFILE_EXPORT_PROFILE_INPUT,
                                                        class: "input-box input",
                                                        "data-ui-entity": ui::control::PROFILE_EXPORT_PROFILE_INPUT,
                                                        r#type: "text",
                                                        autocomplete: "off",
                                                        "data-committed-value": "{profile_export_attach_path_draft()}",
                                                        placeholder: "{default_profile_export_path_value}",
                                                        "data-commit-on-enter": "true",
                                                        "data-clear-button": "true",
                                                        "data-preserve-draft": "true",
                                                        onchange: move |event| {
                                                            let value = event.value().to_string();
                                                            profile_export_attach_path_draft.set(value.clone());
                                                            profile_export_attach_path_input.set(value);
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        }
                                                    }
                                                    button {
                                                        id: ui::id::CLEAR_PROFILE_EXPORT_PROFILE_BUTTON,
                                                        class: "path-input-clear",
                                                        r#type: "button",
                                                        disabled: profile_export_attach_path_draft().is_empty(),
                                                        "data-clear-button": "true",
                                                        "data-ui-action": ui::action::CLEAR_PROFILE_EXPORT_PROFILE,
                                                        "aria-label": "{input_clear}",
                                                        "data-tooltip": "{input_clear}",
                                                        "data-tooltip-align": "end",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            profile_export_attach_path_draft.set(String::new());
                                                            profile_export_attach_path_input.set(String::new());
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        },
                                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                    }
                                                }
                                                button {
                                                    id: ui::id::BROWSE_PROFILE_EXPORT_PROFILE_BUTTON,
                                                    class: "input-box button button--secondary",
                                                    r#type: "button",
                                                    "data-ui-action": ui::action::BROWSE_PROFILE_EXPORT_PROFILE,
                                                    onclick: move |_| {
                                                        open_profile_export_file_dialog(
                                                            profile_export_attach_path_input,
                                                            profile_export_attach_path_draft,
                                                            profile_export_feedback,
                                                            profile_export_preview,
                                                            profile_export_file_dialog_open,
                                                            profile_export_picker_window_attach.clone(),
                                                        );
                                                    },
                                                    "{dialog_integration_browse}"
                                                }
                                            }
                                        }
                                        div { class: "profile-export-field",
                                            label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_PROFILE_SELECT, "{dialog_profile_export_available_profiles}" }
                                            select {
                                                id: ui::id::PROFILE_EXPORT_PROFILE_SELECT,
                                                class: "input-box select",
                                                value: "{profile_export_selected_profile_value}",
                                                "data-ui-entity": ui::control::PROFILE_EXPORT_PROFILE_SELECT,
                                                        onchange: move |event| {
                                                            let value = event.value();
                                                            if !value.trim().is_empty() {
                                                                profile_export_attach_path_draft.set(value.clone());
                                                                profile_export_attach_path_input.set(value);
                                                                profile_export_feedback.set(None);
                                                                profile_export_preview.set(None);
                                                            }
                                                },
                                                if profile_export_profile_options.is_empty() {
                                                    option { value: "", "{dialog_profile_export_no_profiles}" }
                                                } else {
                                                    for profile_path in profile_export_profile_options.clone() {
                                                        option {
                                                            value: "{profile_path}",
                                                            "{profile_export_profile_option_label(&profile_path, &snapshot.integration.repo_path)}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "profile-export-field",
                                            label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_GENERATED_NAME_INPUT, "{dialog_export_generated_name}" }
                                            input {
                                                id: ui::id::PROFILE_EXPORT_GENERATED_NAME_INPUT,
                                                class: "input-box input",
                                                "data-ui-entity": ui::control::PROFILE_EXPORT_GENERATED_NAME_INPUT,
                                                r#type: "text",
                                                autocomplete: "off",
                                                "data-committed-value": "{profile_export_generated_name_draft()}",
                                                "data-commit-on-enter": "true",
                                                "data-preserve-draft": "true",
                                                onchange: move |event| {
                                                    let value = event.value().to_string();
                                                    profile_export_generated_name_draft.set(value.clone());
                                                    profile_export_generated_name.set(value);
                                                    profile_export_feedback.set(None);
                                                    profile_export_preview.set(None);
                                                }
                                            }
                                        }
                                        div {
                                            class: "control-help-text profile-export-tab-help",
                                            "data-ui-entity": ui::entity::HELP_TEXT,
                                            "{dialog_export_note_attach}"
                                        }
                                    }
                                } else if profile_export_mode_value == ExportModeDto::PatchSelectedProfile {
                                    div { class: "profile-export-grid profile-export-grid--single",
                                        div { class: "profile-export-field",
                                            label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_PROFILE_INPUT, "{dialog_export_profile_path}" }
                                            div { class: "field-row profile-export-path-row",
                                                div { class: "path-input-shell",
                                                    input {
                                                        id: ui::id::PROFILE_EXPORT_PROFILE_INPUT,
                                                        class: "input-box input",
                                                        "data-ui-entity": ui::control::PROFILE_EXPORT_PROFILE_INPUT,
                                                        r#type: "text",
                                                        autocomplete: "off",
                                                        "data-committed-value": "{profile_export_patch_path_draft()}",
                                                        placeholder: "{default_profile_export_path_value}",
                                                        "data-commit-on-enter": "true",
                                                        "data-clear-button": "true",
                                                        "data-preserve-draft": "true",
                                                        onchange: move |event| {
                                                            let value = event.value().to_string();
                                                            profile_export_patch_path_draft.set(value.clone());
                                                            profile_export_patch_path_input.set(value);
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        }
                                                    }
                                                    button {
                                                        id: ui::id::CLEAR_PROFILE_EXPORT_PROFILE_BUTTON,
                                                        class: "path-input-clear",
                                                        r#type: "button",
                                                        disabled: profile_export_patch_path_draft().is_empty(),
                                                        "data-clear-button": "true",
                                                        "data-ui-action": ui::action::CLEAR_PROFILE_EXPORT_PROFILE,
                                                        "aria-label": "{input_clear}",
                                                        "data-tooltip": "{input_clear}",
                                                        "data-tooltip-align": "end",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            profile_export_patch_path_draft.set(String::new());
                                                            profile_export_patch_path_input.set(String::new());
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        },
                                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                    }
                                                }
                                                button {
                                                    id: ui::id::BROWSE_PROFILE_EXPORT_PROFILE_BUTTON,
                                                    class: "input-box button button--secondary",
                                                    r#type: "button",
                                                    "data-ui-action": ui::action::BROWSE_PROFILE_EXPORT_PROFILE,
                                                    onclick: move |_| {
                                                        open_profile_export_file_dialog(
                                                            profile_export_patch_path_input,
                                                            profile_export_patch_path_draft,
                                                            profile_export_feedback,
                                                            profile_export_preview,
                                                            profile_export_file_dialog_open,
                                                            profile_export_picker_window_patch.clone(),
                                                        );
                                                    },
                                                    "{dialog_integration_browse}"
                                                }
                                            }
                                            div { class: "profile-export-field",
                                                label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_PROFILE_SELECT, "{dialog_profile_export_available_profiles}" }
                                                select {
                                                    id: ui::id::PROFILE_EXPORT_PROFILE_SELECT,
                                                    class: "input-box select",
                                                    value: "{profile_export_selected_profile_value}",
                                                    "data-ui-entity": ui::control::PROFILE_EXPORT_PROFILE_SELECT,
                                                        onchange: move |event| {
                                                            let value = event.value();
                                                            if !value.trim().is_empty() {
                                                                profile_export_patch_path_draft.set(value.clone());
                                                                profile_export_patch_path_input.set(value);
                                                                profile_export_feedback.set(None);
                                                                profile_export_preview.set(None);
                                                            }
                                                    },
                                                    if profile_export_profile_options.is_empty() {
                                                        option { value: "", "{dialog_profile_export_no_profiles}" }
                                                    } else {
                                                        for profile_path in profile_export_profile_options.clone() {
                                                            option {
                                                                value: "{profile_path}",
                                                                "{profile_export_profile_option_label(&profile_path, &snapshot.integration.repo_path)}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div {
                                            class: "control-help-text profile-export-tab-help",
                                            "data-ui-entity": ui::entity::HELP_TEXT,
                                            "{dialog_export_note_patch}"
                                        }
                                    }
                                } else {
                                    div { class: "profile-export-grid profile-export-grid--single",
                                        div { class: "profile-export-field",
                                            label { class: "value-label", r#for: ui::id::PROFILE_EXPORT_PROFILE_INPUT, "{dialog_export_profile_path}" }
                                            div { class: "field-row profile-export-path-row",
                                                div { class: "path-input-shell",
                                                    input {
                                                        id: ui::id::PROFILE_EXPORT_PROFILE_INPUT,
                                                        class: "input-box input",
                                                        "data-ui-entity": ui::control::PROFILE_EXPORT_PROFILE_INPUT,
                                                        r#type: "text",
                                                        autocomplete: "off",
                                                        "data-committed-value": "{profile_export_merge_path_draft()}",
                                                        placeholder: "{default_profile_export_path_value}",
                                                        "data-commit-on-enter": "true",
                                                        "data-clear-button": "true",
                                                        "data-preserve-draft": "true",
                                                        onchange: move |event| {
                                                            let value = event.value().to_string();
                                                            profile_export_merge_path_draft.set(value.clone());
                                                            profile_export_merge_path_input.set(value);
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        }
                                                    }
                                                    button {
                                                        id: ui::id::CLEAR_PROFILE_EXPORT_PROFILE_BUTTON,
                                                        class: "path-input-clear",
                                                        r#type: "button",
                                                        disabled: profile_export_merge_path_draft().is_empty(),
                                                        "data-clear-button": "true",
                                                        "data-ui-action": ui::action::CLEAR_PROFILE_EXPORT_PROFILE,
                                                        "aria-label": "{input_clear}",
                                                        "data-tooltip": "{input_clear}",
                                                        "data-tooltip-align": "end",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            profile_export_merge_path_draft.set(String::new());
                                                            profile_export_merge_path_input.set(String::new());
                                                            profile_export_feedback.set(None);
                                                            profile_export_preview.set(None);
                                                        },
                                                        img { class: "button__icon", src: "{close_button_src}", alt: "" }
                                                    }
                                                }
                                                button {
                                                    id: ui::id::BROWSE_PROFILE_EXPORT_PROFILE_BUTTON,
                                                    class: "input-box button button--secondary",
                                                    r#type: "button",
                                                    "data-ui-action": ui::action::BROWSE_PROFILE_EXPORT_PROFILE,
                                                    onclick: move |_| {
                                                        open_profile_export_file_dialog(
                                                            profile_export_merge_path_input,
                                                            profile_export_merge_path_draft,
                                                            profile_export_feedback,
                                                            profile_export_preview,
                                                            profile_export_file_dialog_open,
                                                            profile_export_picker_window_merge.clone(),
                                                        );
                                                    },
                                                    "{dialog_integration_browse}"
                                                }
                                            }
                                        }
                                        div { class: "profile-export-switch-row",
                                            span { class: "profile-export-switch-row__label", "{dialog_export_dangerous_confirm}" }
                                            button {
                                                class: "{profile_export_dangerous_switch_class}",
                                                r#type: "button",
                                                role: "switch",
                                                "aria-checked": "{profile_export_dangerous_confirmed()}",
                                                "aria-label": "{dialog_export_dangerous_confirm}",
                                                "data-ui-action": ui::action::TOGGLE_PROFILE_EXPORT_DANGEROUS,
                                                onclick: move |_| {
                                                    profile_export_dangerous_confirmed.set(!profile_export_dangerous_confirmed());
                                                    profile_export_feedback.set(None);
                                                    profile_export_preview.set(None);
                                                },
                                                span { class: "switch__knob" }
                                            }
                                        }
                                        div {
                                            class: "control-help-text profile-export-tab-help",
                                            "data-ui-entity": ui::entity::HELP_TEXT,
                                            "{dialog_export_note_merge}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                            div { class: "profile-export-preview-panel",
                                div { class: "profile-export-preview-actions",
                                    button {
                                        id: ui::id::PROFILE_EXPORT_ANALYZE_BUTTON,
                                        class: "input-box button button--secondary",
                                        r#type: "button",
                                        disabled: profile_export_merge_blocked,
                                        "data-ui-action": ui::action::ANALYZE_PROFILE_EXPORT,
                                        onclick: {
                                            let export_pending_confirm =
                                                pending_observation_selection_confirm.clone();
                                            move |_| {
                                            flush_observation_selection_confirm(
                                                &mut watcher.write(),
                                                &export_pending_confirm,
                                            );
                                            let integration = watcher.read().snapshot().integration.clone();
                                            let selected_profile_path = match profile_export_mode() {
                                                ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
                                                ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
                                                ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
                                            };
                                            match profile_export_request(
                                                &integration,
                                                profile_export_mode(),
                                                &selected_profile_path,
                                                &profile_export_generated_name(),
                                                profile_export_dangerous_confirmed(),
                                            ) {
                                                    Ok(request) => {
                                                        let module_id = selected_integration_module_id();
                                                        let request_key = profile_export_request_key(&request);
                                                        let advanced_domains =
                                                            split_manual_domains(&profile_export_advanced_manual_domains());
                                                    let advanced_settings_active =
                                                        profile_export_advanced_ready_for_export()
                                                            || profile_export_advanced_create_rules()
                                                            || profile_export_advanced_exclude_ips()
                                                            || profile_export_advanced_whois_ranges()
                                                            || (profile_export_advanced_domains()
                                                                && !advanced_domains.is_empty())
                                                            || profile_export_advanced_domain_cleanup_requested();
                                                    let analyze_result = if advanced_settings_active {
                                                        let settings = ExportProfileAdvancedSettingsDto {
                                                            create_rules_for_uncovered: profile_export_advanced_create_rules(),
                                                            add_ips_to_exclude: profile_export_advanced_exclude_ips(),
                                                            use_whois_ranges_for_export: profile_export_advanced_whois_ranges(),
                                                            add_detected_domains: profile_export_advanced_domains(),
                                                            remove_detected_domains: profile_export_advanced_domain_cleanup_requested(),
                                                            manual_domains: advanced_domains,
                                                            template_rule_source: if profile_export_advanced_template_rule().trim().is_empty() {
                                                                None
                                                            } else {
                                                                Some(profile_export_advanced_template_rule())
                                                            },
                                                        };
                                                        watcher.write().analyze_profile_export_advanced_settings(
                                                            module_id.clone(),
                                                            ExportProfileAdvancedSettingsRequestDto { export: request, settings }
                                                        )
                                                    } else {
                                                        watcher.write().analyze_profile_export(module_id, request)
                                                    };
                                                    match analyze_result {
                                                    Ok(plan) => {
                                                        profile_export_manual_plan_key.set(Some(request_key));
                                                        profile_export_preview.set(Some(plan));
                                                        profile_export_feedback.set(None);
                                                        push_status_history_line(status_history, footer_profile_export_analyzed.clone());
                                                    }
                                                    Err(message) => {
                                                        profile_export_feedback.set(None);
                                                        push_status_history_line(status_history, message);
                                                    }
                                                    }
                                                }
                                                Err(message) => {
                                                    profile_export_feedback.set(None);
                                                    push_status_history_line(status_history, message);
                                                }
                                            }
                                            }
                                        },
                                        "{dialog_export_analyze}"
                                    }
                                    button {
                                        id: ui::id::PROFILE_EXPORT_ADVANCED_WIZARD_BUTTON,
                                        class: "input-box button button--secondary",
                                        r#type: "button",
                                        "data-ui-action": ui::action::OPEN_PROFILE_EXPORT_ADVANCED_WIZARD,
                                        onclick: move |_| {
                                            show_profile_export_advanced_wizard.set(true);
                                        },
                                        "{dialog_profile_export_advanced_wizard}"
                                    }
                                    div { class: "profile-export-preview-actions__right",
                                        button {
                                            id: ui::id::PROFILE_EXPORT_BACKUP_BUTTON,
                                            class: "input-box button button--secondary",
                                            r#type: "button",
                                            disabled: profile_export_merge_blocked,
                                            "data-ui-action": ui::action::BACKUP_PROFILE_EXPORT,
                                            onclick: {
                                                let export_pending_confirm =
                                                    pending_observation_selection_confirm.clone();
                                                move |_| {
                                                flush_observation_selection_confirm(
                                                    &mut watcher.write(),
                                                    &export_pending_confirm,
                                                );
                                                let integration = watcher.read().snapshot().integration.clone();
                                                let selected_profile_path = match profile_export_mode() {
                                                    ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
                                                    ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
                                                    ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
                                                };
                                                match profile_export_request(
                                                    &integration,
                                                    profile_export_mode(),
                                                    &selected_profile_path,
                                                    &profile_export_generated_name(),
                                                    profile_export_dangerous_confirmed(),
                                                ) {
                                                    Ok(request) => {
                                                        let module_id = selected_integration_module_id();
                                                        let request_key = profile_export_request_key(&request);
                                                        match watcher.write().backup_profile_export(module_id, request) {
                                                            Ok(plan) => {
                                                                profile_export_manual_plan_key.set(Some(request_key));
                                                                let changed = plan.file_changes.iter()
                                                                    .filter(|change| change.operation != ExportFileOperationDto::Skip)
                                                                    .map(|change| change.path.display().to_string())
                                                                    .collect::<Vec<_>>()
                                                                    .join(", ");
                                                                profile_export_preview.set(Some(plan));
                                                                profile_export_feedback.set(None);
                                                                push_status_history_line(
                                                                    status_history,
                                                                    if changed.is_empty() {
                                                                        footer_profile_export_backup_created.clone()
                                                                    } else {
                                                                        format!("{}: {}", footer_profile_export_backup_created, changed)
                                                                    },
                                                                );
                                                            }
                                                            Err(message) => {
                                                                profile_export_feedback.set(None);
                                                                push_status_history_line(status_history, message);
                                                            }
                                                        }
                                                    }
                                                    Err(message) => {
                                                        profile_export_feedback.set(None);
                                                        push_status_history_line(status_history, message);
                                                    }
                                                }
                                                }
                                            },
                                            "{dialog_export_backup}"
                                        }
                                        button {
                                            id: ui::id::PROFILE_EXPORT_REVERT_BUTTON,
                                            class: "input-box button button--secondary",
                                            r#type: "button",
                                            "data-ui-action": ui::action::REVERT_PROFILE_EXPORT,
                                            onclick: {
                                                let export_pending_confirm =
                                                    pending_observation_selection_confirm.clone();
                                                move |_| {
                                                flush_observation_selection_confirm(
                                                    &mut watcher.write(),
                                                    &export_pending_confirm,
                                                );
                                                let integration = watcher.read().snapshot().integration.clone();
                                                let selected_profile_path = match profile_export_mode() {
                                                    ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
                                                    ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
                                                    ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
                                                };
                                                match profile_export_request(
                                                    &integration,
                                                    profile_export_mode(),
                                                    &selected_profile_path,
                                                    &profile_export_generated_name(),
                                                    profile_export_dangerous_confirmed(),
                                                ) {
                                                    Ok(request) => {
                                                        let module_id = selected_integration_module_id();
                                                        let request_key = profile_export_request_key(&request);
                                                        match watcher.write().revert_profile_export(module_id, request) {
                                                            Ok(plan) => {
                                                                profile_export_manual_plan_key.set(Some(request_key));
                                                                let changed = plan.file_changes.iter()
                                                                    .filter(|change| change.operation != ExportFileOperationDto::Skip)
                                                                    .map(|change| change.path.display().to_string())
                                                                    .collect::<Vec<_>>()
                                                                    .join(", ");
                                                                profile_export_preview.set(Some(plan));
                                                                profile_export_feedback.set(None);
                                                                push_status_history_line(
                                                                    status_history,
                                                                    if changed.is_empty() {
                                                                        dialog_export_revert_success.clone()
                                                                    } else {
                                                                        format!("{}: {}", dialog_export_revert_success, changed)
                                                                    },
                                                                );
                                                            }
                                                            Err(message) => {
                                                                profile_export_feedback.set(None);
                                                                push_status_history_line(status_history, message);
                                                            }
                                                        }
                                                    }
                                                    Err(message) => {
                                                        profile_export_feedback.set(None);
                                                        push_status_history_line(status_history, message);
                                                    }
                                                }
                                                }
                                            },
                                            "{dialog_export_revert}"
                                        }
                                    }
                                }
                                div { class: "profile-export-preview",
                                    if let Some(plan) = profile_export_preview_value.clone() {
                                        div { class: "profile-export-preview__summary",
                                            div { class: "profile-export-preview__section",
                                                strong { class: "profile-export-preview__title", "{dialog_profile_export_group_addresses}:" }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_selected}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--total", "{plan.exported_count}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_will_add}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--ready", "{profile_export_address_new_count(&plan)}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_skipped}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--excluded", "{profile_export_address_skipped_count(&plan)}" }
                                                }
                                            }
                                            div { class: "profile-export-preview__section",
                                                strong { class: "profile-export-preview__title", "{dialog_profile_export_group_domains}:" }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_selected}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--total", "{plan.advanced_domain_count}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_will_add}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--ready", "{plan.advanced_new_domain_count}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_skipped}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--excluded", "{plan.advanced_existing_domain_count}" }
                                                }
                                            }
                                            div { class: "profile-export-preview__section",
                                                strong { class: "profile-export-preview__title", "{dialog_profile_export_group_ranges}:" }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_selected}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--total", "{plan.covered_range_count}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_will_add}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--ready", "{plan.new_range_count}" }
                                                }
                                                div { class: "profile-export-preview__row profile-export-preview__metric",
                                                    span { class: "profile-export-preview__muted", "{dialog_profile_export_skipped}" }
                                                    span { class: "profile-export-preview__metric-value profile-export-preview__metric-value--excluded", "{plan.existing_range_count}" }
                                                }
                                            }
                                        }
                                        if !plan.uncovered_targets.is_empty() {
                                            div { class: "profile-export-preview__group profile-export-preview__group--warning",
                                                strong { class: "profile-export-preview__title", "{dialog_profile_export_uncovered_title}:" }
                                                ul { class: "profile-export-list profile-export-list--plain",
                                                    for target in plan.uncovered_targets.clone() {
                                                        li {
                                                            "{target.protocol}:{target.port} - {target.ip_count} IP"
                                                        }
                                                    }
                                                }
                                                p { class: "profile-export-preview__note", "{dialog_profile_export_uncovered_help}" }
                                            }
                                        }
                                        div { class: "profile-export-preview__group",
                                            strong { class: "profile-export-preview__title", "{dialog_export_files}:" }
                                            if profile_export_file_changes(&plan).is_empty() {
                                                p { class: "muted small", "{dialog_export_no_files}" }
                                            } else {
                                                ul { class: "profile-export-list",
                                                    for change in profile_export_file_changes(&plan) {
                                                        li {
                                                            "{export_file_change_label(&change)}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if plan.warnings.iter().any(|warning| warning.code != "uncovered_protocol_port" && warning.code != "profile_merge_target_skipped") {
                                            div { class: "profile-export-preview__group",
                                                strong { class: "profile-export-preview__title", "{dialog_export_warnings}:" }
                                                ul { class: "profile-export-list",
                                                    for warning in plan.warnings.clone().into_iter().filter(|warning| warning.code != "uncovered_protocol_port" && warning.code != "profile_merge_target_skipped") {
                                                        li { "{warning.message}" }
                                                    }
                                                }
                                            }
                                        }
                                        if !plan.matched_rules.is_empty() {
                                            div { class: "profile-export-preview__group",
                                                strong { class: "profile-export-preview__title", "{dialog_export_rules}:" }
                                                ul { class: "profile-export-list",
                                                    for rule in plan.matched_rules.clone() {
                                                        li { "{export_profile_rule_label(&rule)}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "modal__footer profile-export-footer",
                        button {
                            id: ui::id::PROFILE_EXPORT_APPLY_BUTTON,
                            class: "input-box button button--primary",
                            r#type: "button",
                            disabled: profile_export_merge_blocked,
                            "data-ui-action": ui::action::APPLY_PROFILE_EXPORT,
                            onclick: {
                                let export_pending_confirm =
                                    pending_observation_selection_confirm.clone();
                                move |_| {
                                flush_observation_selection_confirm(
                                    &mut watcher.write(),
                                    &export_pending_confirm,
                                );
                                let integration = watcher.read().snapshot().integration.clone();
                                let selected_profile_path = match profile_export_mode() {
                                    ExportModeDto::AttachNetstitchLists => profile_export_attach_path_input(),
                                    ExportModeDto::PatchSelectedProfile => profile_export_patch_path_input(),
                                    ExportModeDto::MergeIntoExistingLists => profile_export_merge_path_input(),
                                };
                                match profile_export_request(
                                    &integration,
                                    profile_export_mode(),
                                    &selected_profile_path,
                                    &profile_export_generated_name(),
                                    profile_export_dangerous_confirmed(),
                                ) {
                                    Ok(request) => {
                                        let module_id = selected_integration_module_id();
                                        let request_key = profile_export_request_key(&request);
                                        let apply_result = if profile_export_advanced_ready_for_export() {
                                            let settings = ExportProfileAdvancedSettingsDto {
                                                create_rules_for_uncovered: profile_export_advanced_create_rules(),
                                                add_ips_to_exclude: profile_export_advanced_exclude_ips(),
                                                use_whois_ranges_for_export: profile_export_advanced_whois_ranges(),
                                                add_detected_domains: profile_export_advanced_domains(),
                                                remove_detected_domains: profile_export_advanced_domain_cleanup_requested(),
                                                manual_domains: split_manual_domains(&profile_export_advanced_manual_domains()),
                                                template_rule_source: if profile_export_advanced_template_rule().trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(profile_export_advanced_template_rule())
                                                },
                                            };
                                            watcher.write().apply_profile_export_advanced_settings(
                                                module_id.clone(),
                                                ExportProfileAdvancedSettingsRequestDto { export: request, settings }
                                            )
                                        } else {
                                            watcher.write().apply_profile_export(module_id, request)
                                        };
                                        match apply_result {
                                            Ok(apply_plan) => {
                                                profile_export_manual_plan_key.set(Some(request_key));
                                                let mut changed_files = apply_plan.file_changes.iter()
                                                    .filter(|change| change.operation != ExportFileOperationDto::Skip)
                                                    .map(|change| change.path.display().to_string())
                                                    .collect::<Vec<_>>();
                                                changed_files.extend(
                                                    apply_plan.advanced_file_changes.iter()
                                                        .filter(|change| change.operation != ExportFileOperationDto::Skip)
                                                        .map(|change| change.path.display().to_string())
                                                );
                                                let preview_plan = apply_plan;
                                                let changed = changed_files.join(", ");
                                                profile_export_preview.set(Some(preview_plan));
                                                profile_export_feedback.set(None);
                                                push_status_history_line(
                                                    status_history,
                                                    if changed.is_empty() {
                                                        footer_export_completed.clone()
                                                    } else {
                                                        format!("{}: {}", footer_export_completed, changed)
                                                    },
                                                );
                                            }
                                            Err(message) => {
                                                profile_export_feedback.set(None);
                                                push_status_history_line(status_history, message);
                                            }
                                        }
                                    }
                                    Err(message) => {
                                        profile_export_feedback.set(None);
                                        push_status_history_line(status_history, message);
                                    }
                                }
                                }
                            },
                            "{dialog_export_apply}"
                        }
                        button {
                            id: ui::id::PROFILE_EXPORT_CANCEL_BUTTON,
                            class: "input-box button button--secondary",
                            r#type: "button",
                            "data-ui-action": ui::action::CANCEL_PROFILE_EXPORT,
                            onclick: move |_| {
                                let should_return_to_module = return_profile_export_to_module();
                                show_profile_export_prompt.set(false);
                                show_profile_export_advanced_wizard.set(false);
                                return_profile_export_to_module.set(false);
                                profile_export_manual_plan_key.set(None);
                                if should_return_to_module {
                                    show_integration_module_prompt.set(true);
                                }
                            },
                            "{dialog_cancel}"
                        }
                    }
                }
            }
        }

        if show_profile_export_advanced_wizard() {
            div { class: "modal-backdrop",
                section {
                    id: ui::id::PROFILE_EXPORT_ADVANCED_WIZARD_DIALOG,
                    class: "modal modal--panel profile-export-advanced-dialog",
                    role: "dialog",
                    "aria-modal": "true",
                    "data-ui-entity": ui::entity::PROFILE_EXPORT_DIALOG,
                    div { class: "modal__header",
                        div { class: "title-with-help",
                            h2 { "{dialog_profile_export_advanced_wizard_title}" }
                            HelpIcon {
                                icon_src: help_icon_src.clone(),
                                tooltip: dialog_profile_export_advanced_wizard_help.clone(),
                            }
                        }
                    }
                    div { class: "modal__body",
                        div { class: "profile-export-advanced-body",
                            div { class: "control-help-text profile-export-advanced-intro",
                                "{dialog_profile_export_advanced_wizard_help}"
                            }
                            div { class: "profile-export-advanced-panel profile-export-advanced-panel--uncovered",
                                strong { class: "profile-export-preview__title profile-export-preview__title--warning", "{dialog_profile_export_uncovered_title}:" }
                                if let Some(plan) = profile_export_preview_value.clone() {
                                    if plan.uncovered_targets.is_empty() {
                                        p { class: "muted small", "{dialog_profile_export_advanced_wizard_empty}" }
                                    } else {
                                        ul { class: "profile-export-list",
                                            for target in plan.uncovered_targets.clone() {
                                                li { "{target.protocol}:{target.port} - {target.ip_count} IP" }
                                            }
                                        }
                                    }
                                } else {
                                    p { class: "muted small", "{dialog_profile_export_advanced_wizard_empty}" }
                                }
                            }
                            div { class: "profile-export-advanced-panel profile-export-advanced-panel--settings",
                                div { class: "profile-export-advanced-option",
                                    div { class: "profile-export-switch-row",
                                        span { class: "profile-export-switch-row__label", "{dialog_profile_export_advanced_create_rules}" }
                                        button {
                                            class: "{profile_export_advanced_create_rules_class}",
                                            r#type: "button",
                                            role: "switch",
                                            "aria-checked": "{profile_export_advanced_create_rules()}",
                                            onclick: move |_| {
                                                profile_export_advanced_create_rules.set(!profile_export_advanced_create_rules());
                                                profile_export_advanced_ready_for_export.set(false);
                                            },
                                            span { class: "switch__knob" }
                                        }
                                    }
                                    div { class: "control-help-text", "{dialog_profile_export_advanced_create_rules_help}" }
                                    div { class: "profile-export-field profile-export-field--template",
                                        label { class: "value-label", "{dialog_profile_export_advanced_template_rule}" }
                                        select {
                                            class: "input-box select",
                                            value: "{profile_export_advanced_template_rule()}",
                                            onchange: move |event| {
                                                profile_export_advanced_template_rule.set(event.value().to_string());
                                                profile_export_advanced_ready_for_export.set(false);
                                            },
                                            option { value: "", "{dialog_profile_export_advanced_wizard_step_template}" }
                                            if let Some(plan) = profile_export_preview_value.clone() {
                                                for rule in plan.matched_rules.clone() {
                                                    option {
                                                        value: "{rule.source}",
                                                        "{export_profile_rule_label(&rule)}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "profile-export-advanced-option",
                                    div { class: "profile-export-switch-row",
                                        span { class: "profile-export-switch-row__label", "{dialog_profile_export_advanced_exclude_ips}" }
                                        button {
                                            class: "{profile_export_advanced_exclude_ips_class}",
                                            r#type: "button",
                                            role: "switch",
                                            "aria-checked": "{profile_export_advanced_exclude_ips()}",
                                            onclick: move |_| {
                                                profile_export_advanced_exclude_ips.set(!profile_export_advanced_exclude_ips());
                                                profile_export_advanced_ready_for_export.set(false);
                                            },
                                            span { class: "switch__knob" }
                                        }
                                    }
                                    div { class: "control-help-text", "{dialog_profile_export_advanced_exclude_ips_help}" }
                                }
                                div { class: "profile-export-advanced-option",
                                    div { class: "profile-export-switch-row",
                                        span { class: "profile-export-switch-row__label", "{dialog_profile_export_advanced_whois_ranges}" }
                                        button {
                                            class: "{profile_export_advanced_whois_ranges_class}",
                                            r#type: "button",
                                            role: "switch",
                                            "aria-checked": "{profile_export_advanced_whois_ranges()}",
                                            onclick: move |_| {
                                                profile_export_advanced_whois_ranges.set(!profile_export_advanced_whois_ranges());
                                                profile_export_advanced_ready_for_export.set(false);
                                            },
                                            span { class: "switch__knob" }
                                        }
                                    }
                                    div { class: "control-help-text", "{dialog_profile_export_advanced_whois_ranges_help}" }
                                }
                                div { class: "profile-export-advanced-option",
                                    div { class: "profile-export-switch-row",
                                        span { class: "profile-export-switch-row__label", "{dialog_profile_export_advanced_domains}" }
                                        button {
                                            class: "{profile_export_advanced_domains_class}",
                                            r#type: "button",
                                            role: "switch",
                                            "aria-checked": "{profile_export_advanced_domains()}",
                                            onclick: move |_| {
                                                profile_export_advanced_domains.set(!profile_export_advanced_domains());
                                                profile_export_advanced_ready_for_export.set(false);
                                            },
                                            span { class: "switch__knob" }
                                        }
                                    }
                                    div { class: "control-help-text", "{dialog_profile_export_advanced_domains_help}" }
                                }
                                div { class: "profile-export-field profile-export-field--domains profile-export-field--manual-domains",
                                    div { class: "profile-export-domain-field-header",
                                        label { class: "value-label", "{dialog_profile_export_advanced_manual_domains}" }
                                        button {
                                            class: "input-box button button--secondary",
                                            r#type: "button",
                                            "data-ui-action": "add-profile-export-domains",
                                            onclick: {
                                                let domain_pending_confirm =
                                                    pending_observation_selection_confirm.clone();
                                                move |_| {
                                                flush_observation_selection_confirm(
                                                    &mut watcher.write(),
                                                    &domain_pending_confirm,
                                                );
                                                let selected_domains = selected_profile_export_domains(&watcher.read().snapshot());
                                                let next_domains = merge_manual_domains(
                                                    &profile_export_advanced_manual_domains(),
                                                    &selected_domains,
                                                );
                                                if !next_domains.trim().is_empty() {
                                                    profile_export_advanced_domains.set(true);
                                                }
                                                profile_export_advanced_manual_domains.set(next_domains);
                                                profile_export_advanced_ready_for_export.set(false);
                                                profile_export_advanced_domain_cleanup_requested.set(false);
                                                }
                                            },
                                            "{dialog_profile_export_advanced_add_domains}"
                                        }
                                    }
                                    div { class: "path-input-shell path-input-shell--textarea",
                                        textarea {
                                            class: "input-box input profile-export-domains-input",
                                            rows: "8",
                                            "data-committed-value": "{profile_export_advanced_manual_domains()}",
                                            placeholder: "{dialog_profile_export_advanced_manual_domains_placeholder}",
                                            "data-clear-button": "true",
                                            "data-preserve-draft": "true",
                                            onchange: move |event| {
                                                let value = event.value().to_string();
                                                let is_empty = value.trim().is_empty();
                                                profile_export_advanced_domains.set(!is_empty);
                                                profile_export_advanced_manual_domains.set(value);
                                                profile_export_advanced_ready_for_export.set(false);
                                                profile_export_advanced_domain_cleanup_requested.set(is_empty);
                                            }
                                        }
                                        button {
                                            class: "path-input-clear",
                                            r#type: "button",
                                            disabled: profile_export_advanced_manual_domains().is_empty(),
                                            "data-clear-button": "true",
                                            "aria-label": "{input_clear}",
                                            "data-tooltip": "{input_clear}",
                                            "data-tooltip-align": "end",
                                            onclick: move |_| {
                                                profile_export_advanced_domains.set(false);
                                                profile_export_advanced_manual_domains.set(String::new());
                                                profile_export_advanced_ready_for_export.set(false);
                                                profile_export_advanced_domain_cleanup_requested.set(true);
                                            },
                                            img {
                                                class: "button__icon",
                                                src: "{close_button_src}",
                                                alt: "",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "modal__footer",
                        button {
                            class: "input-box button button--primary",
                            r#type: "button",
                            "data-ui-action": ui::action::APPLY_PROFILE_EXPORT,
                            onclick: move |_| {
                                let has_domain_settings = profile_export_advanced_domains()
                                    && !split_manual_domains(&profile_export_advanced_manual_domains()).is_empty();
                                let ready = profile_export_advanced_create_rules()
                                    || profile_export_advanced_exclude_ips()
                                    || profile_export_advanced_whois_ranges()
                                    || has_domain_settings
                                    || profile_export_advanced_domain_cleanup_requested();
                                profile_export_advanced_ready_for_export.set(ready);
                                profile_export_feedback.set(None);
                                show_profile_export_advanced_wizard.set(false);
                                push_status_history_line(status_history, dialog_profile_export_advanced_apply.clone());
                            },
                            "{dialog_profile_export_advanced_apply}"
                        }
                        button {
                            class: "input-box button button--secondary",
                            r#type: "button",
                            "data-ui-action": "cancel-profile-export-advanced-settings",
                            onclick: move |_| {
                                profile_export_advanced_create_rules.set(false);
                                profile_export_advanced_exclude_ips.set(false);
                                profile_export_advanced_whois_ranges.set(false);
                                profile_export_advanced_domains.set(false);
                                profile_export_advanced_manual_domains.set(String::new());
                                profile_export_advanced_template_rule.set(String::new());
                                profile_export_advanced_ready_for_export.set(false);
                                profile_export_advanced_domain_cleanup_requested.set(false);
                            },
                            "{dialog_profile_export_advanced_cancel}"
                        }
                        div { class: "modal-footer-separator", "aria-hidden": "true" }
                        button {
                            class: "input-box button button--secondary",
                            r#type: "button",
                            "data-ui-action": ui::action::CLOSE_PROFILE_EXPORT_ADVANCED_WIZARD,
                            onclick: move |_| {
                                show_profile_export_advanced_wizard.set(false);
                            },
                            "{dialog_export_close}"
                        }
                    }
                }
            }
        }

        if let Some(app_to_delete) = pending_delete_app() {
            ConfirmDeleteDialog {
                dialog_id: ui::id::DELETE_TRACKED_APP_DIALOG,
                entity: ui::entity::DELETE_TRACKED_APP_DIALOG,
                title: dialog_delete_title.clone(),
                help: dialog_delete_help.clone(),
                lines: vec![
                    ConfirmDialogLine::normal(format!("{dialog_delete_app}: {}", app_to_delete.display_name)),
                    ConfirmDialogLine::muted(format!("{dialog_delete_path}: {}", app_to_delete.exe_path)),
                ],
                cancel_button_id: ui::id::CANCEL_DELETE_TRACKED_APP_BUTTON,
                confirm_button_id: ui::id::CONFIRM_DELETE_TRACKED_APP_BUTTON,
                cancel_action: ui::action::CANCEL_DELETE_TRACKED_APP,
                confirm_action: ui::action::CONFIRM_DELETE_TRACKED_APP,
                cancel_label: dialog_no.clone(),
                confirm_label: dialog_yes_delete.clone(),
                on_cancel: move |_| {
                    pending_delete_app.set(None);
                },
                on_confirm: move |_| {
                    let mut current = watcher.write();
                    current.delete_tracked_app(DeleteTrackedAppRequest {
                        app_id: app_to_delete.id,
                    });
                    pending_delete_app.set(None);
                },
            }
        }

        if let Some(rule_to_delete) = pending_delete_ignored_address() {
            ConfirmDeleteDialog {
                dialog_id: ui::id::DELETE_IGNORED_ADDRESS_DIALOG,
                entity: ui::entity::DELETE_IGNORED_ADDRESS_DIALOG,
                title: dialog_delete_ignored_title.clone(),
                help: dialog_delete_ignored_help.clone(),
                lines: ignored_address_delete_dialog_lines(
                    &rule_to_delete.enrichment,
                    &rule_to_delete.address_pattern,
                    &dialog_delete_ignored_address,
                    &enrichment_domain_label,
                    &enrichment_owner_label,
                    &enrichment_range_label,
                    &enrichment_registry_label,
                    &enrichment_source_label,
                    &enrichment_unknown,
                    &enrichment_localhost_rule,
                    &enrichment_local_ip_rule,
                ),
                cancel_button_id: ui::id::CANCEL_DELETE_IGNORED_ADDRESS_BUTTON,
                confirm_button_id: ui::id::CONFIRM_DELETE_IGNORED_ADDRESS_BUTTON,
                cancel_action: ui::action::CANCEL_DELETE_IGNORED_ADDRESS,
                confirm_action: ui::action::CONFIRM_DELETE_IGNORED_ADDRESS,
                cancel_label: dialog_no.clone(),
                confirm_label: dialog_yes_delete.clone(),
                on_cancel: move |_| {
                    pending_delete_ignored_address.set(None);
                },
                on_confirm: move |_| {
                    let mut current = watcher.write();
                    current.delete_ignored_address(DeleteIgnoredAddressRequest {
                        ignored_address_id: rule_to_delete.id,
                    });
                    pending_delete_ignored_address.set(None);
                },
            }
        }

        if let Some(observation_to_delete) = pending_delete_observation() {
            ConfirmDeleteDialog {
                dialog_id: ui::id::DELETE_OBSERVATION_DIALOG,
                entity: ui::entity::DELETE_OBSERVATION_DIALOG,
                title: dialog_delete_observation_title.clone(),
                help: dialog_delete_observation_help.clone(),
                lines: observation_delete_dialog_lines(
                    &observation_to_delete,
                    &app_name_for_observation(&snapshot, &observation_to_delete),
                    &dialog_delete_observation_app,
                    &dialog_delete_observation_endpoint,
                    &dialog_delete_observation_state,
                ),
                cancel_button_id: ui::id::CANCEL_DELETE_OBSERVATION_BUTTON,
                confirm_button_id: ui::id::CONFIRM_DELETE_OBSERVATION_BUTTON,
                cancel_action: ui::action::CANCEL_DELETE_OBSERVATION,
                confirm_action: ui::action::CONFIRM_DELETE_OBSERVATION,
                cancel_label: dialog_no.clone(),
                confirm_label: dialog_yes_delete.clone(),
                on_cancel: move |_| {
                    pending_delete_observation.set(None);
                },
                on_confirm: move |_| {
                    let mut current = watcher.write();
                    match current.delete_observation(DeleteObservationRequest {
                        observation_id: observation_to_delete.id,
                    }) {
                        Ok(_) => pending_delete_observation.set(None),
                        Err(message) => push_status_history_line(status_history, message),
                    }
                }
            }
        }

        if show_clear_monitoring_prompt() {
            ClearMonitoringDialog {
                dialog_id: ui::id::CLEAR_MONITORING_DIALOG,
                entity: ui::entity::CLEAR_MONITORING_DIALOG,
                title: clear_dialog_title,
                help: clear_dialog_help,
                lines: vec![
                    ConfirmDialogLine::normal(format!("{clear_dialog_total_label}: {clear_dialog_total_count}")),
                    ConfirmDialogLine::normal(format!("{clear_dialog_selected_label}: {clear_dialog_selected_count}")),
                ],
                cancel_button_id: ui::id::CANCEL_CLEAR_MONITORING_BUTTON,
                clear_all_button_id: ui::id::CONFIRM_CLEAR_MONITORING_ALL_BUTTON,
                clear_selected_button_id: ui::id::CONFIRM_CLEAR_MONITORING_SELECTED_BUTTON,
                cancel_action: ui::action::CANCEL_CLEAR_MONITORING,
                clear_all_action: ui::action::CONFIRM_CLEAR_MONITORING_ALL,
                clear_selected_action: ui::action::CONFIRM_CLEAR_MONITORING_SELECTED,
                cancel_label: dialog_cancel.clone(),
                clear_all_label: clear_dialog_all_button_label,
                clear_selected_label: clear_dialog_selected_button_label,
                selected_disabled: clear_dialog_selected_count == 0,
                on_cancel: move |_| {
                    show_clear_monitoring_prompt.set(false);
                },
                on_clear_all: move |_| {
                    if cloud_import_active {
                        let mut state = cloud_state();
                        let cleared_count = state.downloaded_rows.len();
                        state.downloaded_rows.clear();
                        state.selected_download_row_ids.clear();
                        cloud_state.set(state);
                        last_cloud_download_selection_anchor.set(None);
                        show_clear_monitoring_prompt.set(false);
                        push_status_history_line(
                            status_history,
                            format!("{cloud_import_clear_all_event_label}: {cleared_count}"),
                        );
                    } else {
                        let mut current = watcher.write();
                        let ids: Vec<_> = current
                            .snapshot()
                            .observations
                            .iter()
                            .map(|observation| observation.id)
                            .collect();
                        match current.delete_observations(DeleteObservationsRequest {
                            observation_ids: ids.clone(),
                        }) {
                            Ok(cleared_count) => {
                                update_observation_selection_store(&clear_all_selection_store, &ids, false);
                                show_clear_monitoring_prompt.set(false);
                                push_status_history_line(
                                    status_history,
                                    format!("{clear_all_event_label}: {cleared_count}"),
                                );
                            }
                            Err(message) => push_status_history_line(status_history, message),
                        }
                    }
                },
                on_clear_selected: {
                    let clear_pending_confirm = pending_observation_selection_confirm.clone();
                    let clear_selection_store = observation_selection_store.clone();
                    move |_| {
                        if cloud_import_active {
                            let mut state = cloud_state();
                            let selected = state.selected_download_row_ids.clone();
                            let cleared_count = selected.len();
                            state
                                .downloaded_rows
                                .retain(|row| !selected.contains(&row.row_id));
                            state.selected_download_row_ids.clear();
                            cloud_state.set(state);
                            last_cloud_download_selection_anchor.set(None);
                            show_clear_monitoring_prompt.set(false);
                            push_status_history_line(
                                status_history,
                                format!("{cloud_import_clear_selected_event_label}: {cleared_count}"),
                            );
                        } else {
                            flush_observation_selection_confirm(
                                &mut watcher.write(),
                                &clear_pending_confirm,
                            );
                            let mut current = watcher.write();
                            let ids = clone_observation_selection_store(&clear_selection_store)
                                .into_iter()
                                .collect::<Vec<_>>();
                            match current.delete_observations(DeleteObservationsRequest {
                                observation_ids: ids.clone(),
                            }) {
                                Ok(cleared_count) => {
                                    update_observation_selection_store(&clear_selection_store, &ids, false);
                                    show_clear_monitoring_prompt.set(false);
                                    push_status_history_line(
                                        status_history,
                                        format!("{clear_selected_event_label}: {cleared_count}"),
                                    );
                                }
                                Err(message) => push_status_history_line(status_history, message),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConfirmDialogLine {
    text: String,
    muted: bool,
}

impl ConfirmDialogLine {
    fn normal(text: String) -> Self {
        Self { text, muted: false }
    }

    fn muted(text: String) -> Self {
        Self { text, muted: true }
    }
}

#[component]
fn ConfirmDeleteDialog(
    dialog_id: &'static str,
    entity: &'static str,
    title: String,
    help: String,
    lines: Vec<ConfirmDialogLine>,
    cancel_button_id: &'static str,
    confirm_button_id: &'static str,
    cancel_action: &'static str,
    confirm_action: &'static str,
    cancel_label: String,
    confirm_label: String,
    on_cancel: EventHandler<MouseEvent>,
    on_confirm: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "modal-backdrop",
            div {
                id: "{dialog_id}",
                class: "modal modal--compact",
                "data-ui-entity": "{entity}",
                div { class: "modal__header",
                    div {
                        h2 { "{title}" }
                    }
                }
                div { class: "modal__body",
                    p { class: "help-text",
                        "{help}"
                    }
                    div { class: "note-box",
                        for line in lines {
                            p { class: if line.muted { "muted small" } else { "small" },
                                "{line.text}"
                            }
                        }
                    }
                }
                div { class: "modal__footer",
                    button {
                        id: "{cancel_button_id}",
                        class: "input-box button button--secondary",
                        "data-ui-action": "{cancel_action}",
                        onclick: on_cancel,
                        "{cancel_label}"
                    }
                    button {
                        id: "{confirm_button_id}",
                        class: "input-box button button--danger",
                        "data-ui-action": "{confirm_action}",
                        onclick: on_confirm,
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}

#[component]
fn ClearMonitoringDialog(
    dialog_id: &'static str,
    entity: &'static str,
    title: String,
    help: String,
    lines: Vec<ConfirmDialogLine>,
    cancel_button_id: &'static str,
    clear_all_button_id: &'static str,
    clear_selected_button_id: &'static str,
    cancel_action: &'static str,
    clear_all_action: &'static str,
    clear_selected_action: &'static str,
    cancel_label: String,
    clear_all_label: String,
    clear_selected_label: String,
    selected_disabled: bool,
    on_cancel: EventHandler<MouseEvent>,
    on_clear_all: EventHandler<MouseEvent>,
    on_clear_selected: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "modal-backdrop",
            div {
                id: "{dialog_id}",
                class: "modal modal--compact",
                "data-ui-entity": "{entity}",
                div { class: "modal__header",
                    div {
                        h2 { "{title}" }
                    }
                }
                div { class: "modal__body",
                    p { class: "help-text",
                        "{help}"
                    }
                    div { class: "note-box",
                        for line in lines {
                            p { class: if line.muted { "muted small" } else { "small" },
                                "{line.text}"
                            }
                        }
                    }
                }
                div { class: "modal__footer",
                    button {
                        id: "{cancel_button_id}",
                        class: "input-box button button--secondary",
                        "data-ui-action": "{cancel_action}",
                        onclick: on_cancel,
                        "{cancel_label}"
                    }
                    button {
                        id: "{clear_selected_button_id}",
                        class: "input-box button button--danger",
                        "data-ui-action": "{clear_selected_action}",
                        disabled: selected_disabled,
                        onclick: on_clear_selected,
                        "{clear_selected_label}"
                    }
                    button {
                        id: "{clear_all_button_id}",
                        class: "input-box button button--danger",
                        "data-ui-action": "{clear_all_action}",
                        onclick: on_clear_all,
                        "{clear_all_label}"
                    }
                }
            }
        }
    }
}

fn ignored_address_delete_dialog_lines(
    enrichment: &Option<crate::watcher_api::IpEnrichmentDto>,
    address_pattern: &str,
    address_label: &str,
    domain_label: &str,
    owner_label: &str,
    range_label: &str,
    registry_label: &str,
    source_label: &str,
    unknown_label: &str,
    localhost_rule_label: &str,
    local_ip_rule_label: &str,
) -> Vec<ConfirmDialogLine> {
    let mut lines = vec![ConfirmDialogLine::normal(format!(
        "{address_label}: {address_pattern}"
    ))];
    for line in ignored_address_tooltip(
        address_pattern,
        enrichment,
        domain_label,
        owner_label,
        range_label,
        registry_label,
        source_label,
        unknown_label,
        localhost_rule_label,
        local_ip_rule_label,
    )
    .lines()
    .filter(|line| !line.trim().is_empty())
    {
        lines.push(ConfirmDialogLine::muted(line.to_string()));
    }
    lines
}

fn observation_delete_dialog_lines(
    observation: &ObservationDto,
    app_name: &str,
    app_label: &str,
    endpoint_label: &str,
    state_label: &str,
) -> Vec<ConfirmDialogLine> {
    vec![
        ConfirmDialogLine::normal(format!("{app_label}: {app_name}")),
        ConfirmDialogLine::normal(format!(
            "{endpoint_label}: {}:{} {}",
            observation.remote_ip,
            observation.remote_port,
            observation.protocol.as_str()
        )),
        ConfirmDialogLine::muted(format!(
            "{state_label}: {} ({}/{})",
            observation.connection_state.as_str(),
            observation.failed_hits,
            observation.successful_hits
        )),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProgressStage {
    label: String,
    width: u8,
    class_name: String,
    style: String,
}

impl ProgressStage {
    fn new(label: String, width: u8, class_name: impl Into<String>) -> Self {
        Self {
            label,
            width,
            class_name: class_name.into(),
            style: String::new(),
        }
    }
}

#[component]
fn ProgressBar(
    label: String,
    percent: u8,
    meta: String,
    stages: Vec<ProgressStage>,
    #[props(default = false)] compact: bool,
    #[props(default = false)] hide_label: bool,
) -> Element {
    let bounded_percent = percent.min(100);
    let remaining_percent = 100_u8.saturating_sub(bounded_percent);
    let normalized_stages = normalize_progress_stages(stages);
    let stage_columns = normalized_stages
        .iter()
        .map(|stage| format!("{}fr", stage.width.max(1)))
        .collect::<Vec<_>>()
        .join(" ");
    let current_label = progress_current_stage_label(&normalized_stages, bounded_percent);
    let current_text = progress_current_text(bounded_percent, &current_label);
    let tooltip = if meta.trim().is_empty() {
        current_text.clone()
    } else if current_label.trim().is_empty() || meta.contains(current_label.trim()) {
        meta.clone()
    } else {
        format!("{meta} | {current_text}")
    };
    let visual_stages = if compact {
        merge_adjacent_progress_stages(normalized_stages.clone())
    } else {
        normalized_stages.clone()
    };

    rsx! {
        div {
            class: if compact { "progress-bar progress-bar--compact" } else { "progress-bar" },
            "data-ui-entity": ui::entity::PROGRESS_BAR,
            if !compact && !hide_label {
                div { class: "progress-bar__header",
                    span { class: "progress-bar__label", "{label}" }
                    span { class: "progress-bar__meta", "{meta}" }
                }
            }
            div { class: "progress-bar__track",
                "aria-label": "{label}",
                "data-tooltip": "{tooltip}",
                "data-tooltip-align": "end",
                div { class: "progress-bar__segments",
                    for stage in visual_stages {
                        {
                            let segment_style = progress_segment_style(&stage);
                            rsx! {
                        div {
                            class: "progress-bar__segment {stage.class_name}",
                            style: "{segment_style}",
                        }
                            }
                        }
                    }
                }
                div {
                    class: "progress-bar__remaining",
                    style: "width: {remaining_percent}%;",
                }
            }
            if !compact && !hide_label {
                div {
                    class: "progress-bar__stages",
                    style: "grid-template-columns: {stage_columns};",
                    for stage in normalized_stages {
                        span { "{stage.label}" }
                    }
                }
                div { class: "progress-bar__current", "{current_text}" }
            }
        }
    }
}

fn normalize_progress_stages(stages: Vec<ProgressStage>) -> Vec<ProgressStage> {
    if stages.is_empty() {
        return vec![ProgressStage::new(
            "Progress".to_string(),
            100,
            "progress-bar__segment--accent",
        )];
    }

    let total = stages
        .iter()
        .map(|stage| stage.width.max(1) as u16)
        .sum::<u16>()
        .max(1);
    let mut normalized = stages
        .into_iter()
        .map(|stage| ProgressStage {
            label: stage.label,
            width: (((stage.width.max(1) as u16) * 100) / total).max(1) as u8,
            class_name: stage.class_name,
            style: stage.style,
        })
        .collect::<Vec<_>>();
    let used = normalized
        .iter()
        .map(|stage| stage.width as u16)
        .sum::<u16>();
    if let Some(last) = normalized.last_mut() {
        if used != 100 {
            let corrected = (last.width as i16 + 100_i16 - used as i16).max(1) as u8;
            last.width = corrected;
        }
    }
    normalized
}

fn progress_segment_style(stage: &ProgressStage) -> String {
    let mut style = format!("width: {}%;", stage.width.max(1));
    if !stage.style.is_empty() {
        style.push(' ');
        style.push_str(&stage.style);
    }
    style
}

fn merge_adjacent_progress_stages(stages: Vec<ProgressStage>) -> Vec<ProgressStage> {
    let mut merged: Vec<ProgressStage> = Vec::new();
    for stage in stages {
        if let Some(last) = merged.last_mut() {
            if last.class_name == stage.class_name && last.style == stage.style {
                last.width = last.width.saturating_add(stage.width).min(100);
                if last.label.is_empty() {
                    last.label = stage.label;
                }
                continue;
            }
        }
        merged.push(stage);
    }
    merged
}

fn progress_current_stage_label(stages: &[ProgressStage], percent: u8) -> String {
    let mut boundary = 0_u16;
    for stage in stages {
        boundary = (boundary + stage.width.max(1) as u16).min(100);
        if percent as u16 <= boundary {
            return stage.label.trim().to_string();
        }
    }
    stages
        .last()
        .map(|stage| stage.label.trim().to_string())
        .unwrap_or_default()
}

fn progress_current_text(percent: u8, stage_label: &str) -> String {
    let stage_label = stage_label.trim();
    if stage_label.is_empty() {
        format!("{percent}%")
    } else {
        format!("{percent}% | {stage_label}")
    }
}

#[component]
fn TrackedAppItem(
    app: crate::watcher_api::TrackedAppDto,
    enable_tooltip: String,
    disable_tooltip: String,
    open_folder_label: String,
    delete_label: String,
    onclick_toggle: EventHandler<MouseEvent>,
    onclick_delete: EventHandler<MouseEvent>,
) -> Element {
    let item_id = ui::tracked_app_item_id(app.id);
    let toggle_id = ui::tracked_app_toggle_id(app.id);
    let delete_id = ui::tracked_app_delete_id(app.id);
    let app_key = app.id.to_string();
    let icon_label = app_icon_label(&app.icon_key);
    let icon_class = format!("app-icon app-icon--{}", icon_key_class(&app.icon_key));
    let icon_url = app.icon_path.as_deref().map(icon_image_src);
    let close_icon_src = inline_svg_data_uri(CLOSE_TIMES_ICON_SVG);
    let exe_path_for_open = app.exe_path.clone();
    let path_input_size = tracked_path_field_size(&app.exe_path);
    let can_delete = app.icon_key == "manual";
    let switch_tooltip = if app.enabled {
        disable_tooltip
    } else {
        enable_tooltip
    };
    let row_class = if app.enabled {
        "list-item tracked-app tracked-app--enabled"
    } else {
        "list-item tracked-app"
    };
    let actions_class = if can_delete {
        "tracked-app__actions tracked-app__actions--with-delete"
    } else {
        "tracked-app__actions tracked-app__actions--toggle-only"
    };

    rsx! {
        div {
            id: "{item_id}",
            class: "{row_class}",
            "data-ui-entity": ui::entity::TRACKED_APP_ITEM,
            "data-ui-action": ui::action::TOGGLE_TRACKED_APP,
            "data-ui-key": "{app_key}",
            onclick: move |event| {
                onclick_toggle.call(event);
            },
            div {
                class: "{icon_class}",
                "aria-hidden": "true",
                if let Some(icon_url) = icon_url {
                    img {
                        class: "app-icon__image",
                        src: "{icon_url}",
                        alt: ""
                    }
                } else {
                    "{icon_label}"
                }
            }
            div {
                class: "tracked-app__main",
                strong { class: "tracked-app__title", "{app.display_name}" }
                input {
                    class: "tracked-app__path path-field path-field--clickable",
                    r#type: "text",
                    readonly: true,
                    style: "width: min(100%, {path_input_size}ch);",
                    value: "{app.exe_path}",
                    "aria-label": "{open_folder_label}",
                    "data-tooltip": "{open_folder_label}",
                    "data-tooltip-align": "start",
                    onclick: move |event| {
                        event.stop_propagation();
                        open_app_folder(&exe_path_for_open);
                    },
                }
            }
            div {
                class: "{actions_class}",
                if can_delete {
                    button {
                        id: "{delete_id}",
                        class: "input-box button button--danger button--square button--close",
                        "aria-label": "{delete_label}",
                        "data-tooltip": "{delete_label}",
                        "data-tooltip-align": "end",
                        "data-ui-action": ui::action::DELETE_TRACKED_APP,
                        "data-ui-key": "{app_key}",
                        onclick: move |event| {
                            event.stop_propagation();
                            onclick_delete.call(event);
                        },
                        img {
                            class: "button__icon",
                            src: "{close_icon_src}",
                            alt: ""
                        }
                    }
                }
                div {
                    class: "tracked-app__action-row",
                    button {
                        id: "{toggle_id}",
                        class: if app.enabled { "input-box switch switch--on" } else { "input-box switch" },
                        "data-ui-action": ui::action::TOGGLE_TRACKED_APP,
                        "data-ui-key": "{app_key}",
                        role: "switch",
                        "aria-checked": "{app.enabled}",
                        "aria-label": "{switch_tooltip}",
                        "data-tooltip": "{switch_tooltip}",
                        "data-tooltip-align": "end",
                        onclick: move |event| {
                            event.stop_propagation();
                            onclick_toggle.call(event);
                        },
                        span { class: "switch__knob" }
                    }
                }
            }
        }
    }
}

#[component]
fn IgnoredAddressRowView(
    rule: crate::watcher_api::IgnoredAddressDto,
    help_icon_src: String,
    domain_label: String,
    owner_label: String,
    range_label: String,
    registry_label: String,
    source_label: String,
    unknown_label: String,
    localhost_rule_label: String,
    local_ip_rule_label: String,
    delete_label: String,
    close_icon_src: String,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let domain_text = ignored_address_domain_text(&rule.enrichment);
    let tooltip = ignored_address_tooltip(
        &rule.address_pattern,
        &rule.enrichment,
        &domain_label,
        &owner_label,
        &range_label,
        &registry_label,
        &source_label,
        &unknown_label,
        &localhost_rule_label,
        &local_ip_rule_label,
    );

    rsx! {
        div { class: "ignored-addresses__row",
            input {
                class: "path-field ignored-addresses__address",
                r#type: "text",
                readonly: true,
                value: "{rule.address_pattern}",
                "aria-label": "{rule.address_pattern}",
            }
            if !tooltip.is_empty() {
                HelpIcon {
                    class: "help-icon ignored-addresses__help".to_string(),
                    icon_src: help_icon_src,
                    tooltip: tooltip,
                    align: "end",
                }
            }
            input {
                class: "path-field ignored-addresses__domain",
                r#type: "text",
                readonly: true,
                value: "{domain_text}",
                "aria-label": "{domain_label}",
            }
            button {
                class: "input-box button button--danger button--square button--close",
                "data-ui-action": ui::action::DELETE_IGNORED_ADDRESS,
                "data-ui-key": "{rule.id}",
                "aria-label": "{delete_label}",
                "data-tooltip": "{delete_label}",
                "data-tooltip-align": "end",
                onclick: on_delete,
                img {
                    class: "button__icon",
                    src: "{close_icon_src}",
                    alt: "",
                }
            }
        }
    }
}

#[component]
fn SortHeaderCell(
    label: String,
    tooltip: String,
    sort_state: &'static str,
    on_click: EventHandler<MouseEvent>,
    sort_idle_icon_src: String,
    sort_asc_icon_src: String,
    sort_desc_icon_src: String,
) -> Element {
    let sort_icon_src = match sort_state {
        "asc" => sort_asc_icon_src,
        "desc" => sort_desc_icon_src,
        _ => sort_idle_icon_src,
    };
    rsx! {
        th {
            class: "table-sortable",
            "data-tooltip": "{tooltip}",
            "data-tooltip-align": "start",
            onclick: on_click,
            div { class: "table-sortable__content",
                img {
                    class: "table-sortable__icon table-sortable__icon--{sort_state}",
                    src: "{sort_icon_src}",
                    alt: "",
                    "aria-hidden": "true",
                }
                span { class: "table-sortable__label", "{label}" }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ModuleTableSortState {
    column: Option<usize>,
    descending: bool,
}

impl ModuleTableSortState {
    fn toggled(self, column: usize) -> Self {
        if self.column == Some(column) {
            if self.descending {
                Self::default()
            } else {
                Self {
                    column: Some(column),
                    descending: true,
                }
            }
        } else {
            Self {
                column: Some(column),
                descending: false,
            }
        }
    }

    fn indicator_state(self, column: usize) -> &'static str {
        if self.column != Some(column) {
            "idle"
        } else if self.descending {
            "desc"
        } else {
            "asc"
        }
    }
}

#[component]
fn CloudAppCatalogRowView(
    app: CloudCatalogApp,
    selected: bool,
    loading: bool,
    filters: CloudSearchFilters,
    download_label: String,
    started_message: String,
    message_labels: CloudMessageLabels,
    cloud_import_button_src: String,
    mut cloud_state: Signal<CloudSyncUiState>,
    mut cloud_loading_app_id: Signal<Option<String>>,
    mut cloud_selected_app_id: Signal<Option<String>>,
    footer_progress: Signal<Option<FooterProgressState>>,
    status_history: Signal<Vec<StatusHistoryLine>>,
) -> Element {
    let row_class = if loading {
        "observation-row observation-row--confirmed cloud-sync-app-row cloud-sync-app-row--selected cloud-sync-app-row--loading"
    } else if selected {
        "observation-row observation-row--confirmed cloud-sync-app-row cloud-sync-app-row--selected"
    } else {
        "observation-row cloud-sync-app-row"
    };
    let app_id = app.app_id.clone();
    let app_key = app.app_id.clone();
    let authors_label = cloud_app_authors_label(&app);
    let available_rows_label = cloud_app_available_row_count(&app).to_string();
    let app_name = app.display_name.clone();
    let publisher_name = app.publisher_name.clone();

    rsx! {
        tr {
            class: "{row_class}",
            "data-ui-key": "{app_key}",
            td {
                input {
                    class: "path-field cloud-sync-app-field",
                    r#type: "text",
                    readonly: true,
                    value: "{app_name}",
                    "aria-label": "{app_name}",
                    "data-tooltip": "{app_name}",
                    "data-tooltip-align": "start",
                }
            }
            td {
                input {
                    class: "path-field cloud-sync-app-field",
                    r#type: "text",
                    readonly: true,
                    value: "{publisher_name}",
                    "aria-label": "{publisher_name}",
                    "data-tooltip": "{publisher_name}",
                    "data-tooltip-align": "start",
                }
            }
            td {
                input {
                    class: "path-field cloud-sync-app-field cloud-sync-app-count-field",
                    r#type: "text",
                    readonly: true,
                    value: "{available_rows_label}",
                    "aria-label": "{available_rows_label}",
                    "data-tooltip": "{available_rows_label}",
                    "data-tooltip-align": "start",
                }
            }
            td {
                input {
                    class: "path-field cloud-sync-app-field",
                    r#type: "text",
                    readonly: true,
                    value: "{authors_label}",
                    "aria-label": "{authors_label}",
                    "data-tooltip": "{authors_label}",
                    "data-tooltip-align": "start",
                }
            }
            td {
                div { class: "table-actions",
                    button {
                        class: "input-box button button--icon button--square table-action-button",
                        "data-ui-action": ui::action::DOWNLOAD_CLOUD_APP_DATA,
                        "data-ui-key": "{app_key}",
                        "aria-label": "{download_label}",
                        "data-tooltip": "{download_label}",
                        "data-tooltip-align": "end",
                        onclick: move |event| {
                            event.stop_propagation();
                            let selected_app_id = app_id.clone();
                            cloud_selected_app_id.set(Some(selected_app_id.clone()));
                            let mut next_filters = filters.clone();
                            next_filters.selected_app_id = Some(selected_app_id);
                            next_filters.remote_ip_query.clear();
                            next_filters.remote_domain_query.clear();
                            next_filters.remote_port_query.clear();
                            next_filters.protocol = "all".to_string();
                            start_cloud_download(
                                cloud_state,
                                cloud_loading_app_id,
                                footer_progress,
                                status_history,
                                next_filters,
                                started_message.clone(),
                                message_labels.clone(),
                            );
                        },
                        img {
                            class: "button__icon button__icon--cloud-import table-action-button__icon",
                            src: "{cloud_import_button_src}",
                            alt: "",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ObservationRowView(
    observation: ObservationDto,
    is_selected: bool,
    app_name: String,
    confirm_label: String,
    unconfirm_label: String,
    ignore_label: String,
    delete_label: String,
    help_icon_src: String,
    close_icon_src: String,
    domain_label: String,
    owner_label: String,
    range_label: String,
    registry_label: String,
    source_label: String,
    unknown_label: String,
    confirm_icon_src: String,
    unconfirm_icon_src: String,
    ignore_icon_src: String,
    on_row_click: EventHandler<MouseEvent>,
    on_toggle_confirmed: EventHandler<MouseEvent>,
    on_ignore_address: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let row_id = ui::observation_row_id(observation.id);
    let confirm_id = ui::observation_confirm_id(observation.id);
    let delete_id = ui::observation_delete_id(observation.id);
    let observation_key = observation.id.to_string();
    let row_class = if is_selected {
        "observation-row observation-row--confirmed"
    } else {
        "observation-row"
    };
    let export_ready = is_selected;
    let domain_text = enrichment_domain_text(&observation.enrichment);
    let ip_tooltip = ip_enrichment_tooltip(
        &observation.enrichment,
        &domain_label,
        &owner_label,
        &range_label,
        &registry_label,
        &source_label,
        &unknown_label,
    );

    rsx! {
        tr {
            id: "{row_id}",
            class: "{row_class}",
            "data-ui-entity": ui::entity::OBSERVATION_ROW,
            "data-ui-key": "{observation_key}",
            onclick: on_row_click,
            td {
                input {
                    class: "path-field observation-app-field",
                    r#type: "text",
                    readonly: true,
                    value: "{app_name}",
                    "aria-label": "{app_name}",
                    "data-tooltip": "{app_name}",
                    "data-tooltip-align": "start",
                }
            }
            td {
                div { class: "ip-cell",
                    input {
                        class: "path-field observation-ip-field",
                        r#type: "text",
                        readonly: true,
                        value: "{observation.remote_ip}",
                        "aria-label": "{observation.remote_ip}",
                        "data-tooltip": "{observation.remote_ip}",
                        "data-tooltip-align": "start",
                    }
                    if !ip_tooltip.is_empty() {
                        HelpIcon {
                            class: "help-icon ip-cell__help".to_string(),
                            icon_src: help_icon_src,
                            tooltip: ip_tooltip,
                        }
                    }
                }
            }
            td {
                input {
                    class: "path-field domain-field",
                    r#type: "text",
                    readonly: true,
                    value: "{domain_text}",
                    "aria-label": "{domain_label}",
                    "data-tooltip": if domain_text.is_empty() { String::new() } else { domain_text.clone() },
                    "data-tooltip-align": "start",
                }
            }
            td { "{observation.remote_port}" }
            td { "{observation.protocol.as_str()}" }
            td {
                div { class: "observation-connection",
                    span { class: "{connection_state_label_class(&observation.connection_state)}",
                        "{observation.connection_state.as_str()}"
                    }
                    span { class: "connection-metrics",
                        "("
                        span { class: "connection-metrics__success", "{observation.successful_hits}" }
                        "/"
                        span { class: "connection-metrics__failure", "{observation.failed_hits}" }
                        ")"
                    }
                }
            }
            td { "{observation.hits}" }
            td { "{observation.first_seen}" }
            td { "{observation.last_seen}" }
            td {
                div { class: "table-actions",
                button {
                    id: "{confirm_id}",
                    class: "input-box button button--icon button--square table-action-button",
                    "data-ui-action": ui::action::TOGGLE_OBSERVATION_CONFIRMED,
                    "data-ui-key": "{observation_key}",
                    "aria-label": if export_ready { "{unconfirm_label}" } else { "{confirm_label}" },
                    "data-tooltip": if export_ready { "{unconfirm_label}" } else { "{confirm_label}" },
                    "data-tooltip-align": "end",
                    onclick: on_toggle_confirmed,
                    img {
                        class: if export_ready {
                            "button__icon button__icon--unconfirm-filtered table-action-button__icon"
                        } else {
                            "button__icon button__icon--confirm-filtered table-action-button__icon"
                        },
                        src: if export_ready { "{unconfirm_icon_src}" } else { "{confirm_icon_src}" },
                        alt: "",
                    }
                }
                button {
                    class: "input-box button button--icon button--square table-action-button",
                    "data-ui-action": ui::action::IGNORE_ADDRESS,
                    "data-ui-key": "{observation.remote_ip}",
                    "aria-label": "{ignore_label}",
                    "data-tooltip": "{ignore_label}",
                    "data-tooltip-align": "end",
                    onclick: on_ignore_address,
                    img {
                        class: "button__icon button__icon--unconfirm-filtered table-action-button__icon",
                        src: "{ignore_icon_src}",
                        alt: "",
                    }
                }
                button {
                    id: "{delete_id}",
                    class: "input-box button button--danger button--square button--close",
                    "data-ui-action": ui::action::DELETE_OBSERVATION,
                    "data-ui-key": "{observation_key}",
                    "aria-label": "{delete_label}",
                    "data-tooltip": "{delete_label}",
                    "data-tooltip-align": "end",
                    onclick: on_delete,
                    img {
                        class: "button__icon",
                        src: "{close_icon_src}",
                        alt: "",
                    }
                }
                }
            }
        }
    }
}

#[component]
fn IntegrationUiEntityView(
    entity: IntegrationUiEntityDto,
    module_id: Option<String>,
    module: IntegrationModuleDto,
    selected_monitoring_row_ids: Vec<ObservedEndpointId>,
    displayed_monitoring_row_ids: Vec<ObservedEndpointId>,
    filters: SharedUiFiltersDto,
    mut watcher: Signal<AppWatcherApi>,
    mut status_history: Signal<Vec<StatusHistoryLine>>,
    module_host_dialog: Signal<Option<ModuleHostDialogState>>,
    module_ui_page: Signal<String>,
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    module_ui_action_generation: Signal<u64>,
    module_path_picker_dialog_open: Signal<bool>,
    module_title: String,
) -> Element {
    let window = desktop::use_window();
    let entity = module_ui_entity_with_layout_defaults(entity);
    let entity_type = entity.entity_type.replace('_', "-").to_ascii_lowercase();
    let mut module_table_sort = use_signal(ModuleTableSortState::default);
    let module_input_clear_src = inline_svg_data_uri(CLOSE_TIMES_ICON_SVG);
    if entity.hidden || entity.visible == Some(false) {
        return rsx! {};
    }
    let title = entity.title.clone().unwrap_or_default();
    let tooltip = entity.tooltip.clone().unwrap_or_default();
    let value = entity.value.clone().unwrap_or_default();
    let placeholder = entity.placeholder.clone().unwrap_or_default();
    let entity_id = entity.id.clone();
    let disabled = entity.disabled;
    let readonly = entity.readonly;
    let checked = entity.checked.unwrap_or_else(|| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    });
    let control_value = module_ui_control_value(module_ui_values, &entity_id, &value);
    let control_checked = module_ui_control_checked(module_ui_values, &entity_id, checked);
    let scroll_class = module_ui_scroll_class(entity.scroll.as_deref());
    let size_class = module_ui_size_class(entity.size.as_deref());
    let align_class = module_ui_align_class(entity.align.as_deref());
    let layout_class = module_ui_layout_class(&entity_type, &size_class, &align_class);
    let titleless_row_class = if title.is_empty() {
        " module-ui-schema__row--no-title"
    } else {
        ""
    };
    let entity_style = module_ui_entity_style(&entity);
    let title_node = rsx! {
        if !title.is_empty() {
            div { class: "title-with-help module-ui-schema__title",
                h3 { "{title}" }
                if !tooltip.is_empty() {
                    HelpIcon {
                        icon_src: String::new(),
                        tooltip: tooltip.clone(),
                    }
                }
            }
        }
    };
    let actions_node = rsx! {
        if !entity.actions.is_empty() {
            {
                let actions_layout_class = module_actions_layout_class(&entity.actions);
                rsx! {
            div { class: "module-ui-schema__actions {align_class} {actions_layout_class}",
                for action in entity.actions.iter().cloned() {
                    {
                        let action_id = action.id.clone();
                        let action_label = action.label.clone();
                        let action_tooltip = action
                            .tooltip
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| action_label.clone());
                        let action_icon_src = module_action_icon_src(&action);
                        let module_id_for_action = module_id.clone();
                        let selected_ids_for_action = selected_monitoring_row_ids.clone();
                        let displayed_ids_for_action = displayed_monitoring_row_ids.clone();
                        let filters_for_action = filters.clone();
                        let module_for_host_commands = module.clone();
                        let module_title_for_action = module_title.clone();
                        let action_label_for_click = action_label.clone();
                        let action_style_class = module_action_style_class(action.style.as_deref());
                        let action_align_class = module_action_align_class(action.align.as_deref());
                        let action_pulse_class =
                            module_action_pulse_class(&action, module.background_active);
                        let module_ui_values_for_action = module_ui_values;
                        let window_for_action = window.clone();
                        rsx! {
                            div { class: "module-ui-schema__action-slot {action_align_class}",
                                button {
                                    class: "input-box button module-ui-schema__action {action_style_class} {action_align_class} {action_pulse_class}",
                                    r#type: "button",
                                    disabled: disabled || !action.enabled,
                                    "data-ui-action": "module-ui-action",
                                    "data-ui-key": "{action_id}",
                                    "data-tooltip": "{action_tooltip}",
                                    onclick: move |_| {
                                        let Some(module_id) = module_id_for_action.clone() else {
                                            push_status_history_line(
                                                status_history,
                                                format!("{module_title_for_action}: {action_label_for_click}"),
                                            );
                                            return;
                                        };
                                        start_module_ui_action(
                                            watcher,
                                            status_history,
                                            module_host_dialog,
                                            module_ui_page,
                                            module_ui_values_for_action,
                                            module_ui_action_generation,
                                            module_path_picker_dialog_open,
                                            window_for_action.clone(),
                                            module_for_host_commands.clone(),
                                            module_title_for_action.clone(),
                                            action_label_for_click.clone(),
                                            IntegrationModuleUiActionClientRequestDto {
                                                module_id,
                                                action_id: action_id.clone(),
                                                ui_action_token: String::new(),
                                                selected_monitoring_row_ids: selected_ids_for_action.clone(),
                                                displayed_monitoring_row_ids: displayed_ids_for_action.clone(),
                                                filters: filters_for_action.clone(),
                                                payload: module_ui_action_payload(
                                                    module_ui_values_for_action,
                                                    module_for_host_commands.background_active,
                                                ),
                                            },
                                        );
                                    },
                                    if let Some(icon_src) = action_icon_src.clone() {
                                        img {
                                            class: "button__icon button__icon--module-action",
                                            src: "{icon_src}",
                                            alt: ""
                                        }
                                    } else {
                                        "{action_label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
                }
            }
        }
    };
    let children_node = rsx! {
        for child in entity.children.iter().cloned() {
            IntegrationUiEntityView {
                entity: child,
                module_id: module_id.clone(),
                module: module.clone(),
                selected_monitoring_row_ids: selected_monitoring_row_ids.clone(),
                displayed_monitoring_row_ids: displayed_monitoring_row_ids.clone(),
                filters: filters.clone(),
                watcher,
                status_history,
                module_host_dialog,
                module_ui_page,
                module_ui_values,
                module_ui_action_generation,
                module_path_picker_dialog_open,
                module_title: module_title.clone(),
            }
        }
    };

    match entity_type.as_str() {
        "separator" => rsx! {
            div {
                class: "module-ui-schema__separator {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "separator",
                "data-ui-key": "{entity_id}",
            }
        },
        "help-text" => rsx! {
            p {
                class: "module-ui-schema__help {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "help-text",
                "data-ui-key": "{entity_id}",
                "{value}"
            }
        },
        "path-field" => rsx! {
            div {
                class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "path-field",
                "data-ui-key": "{entity_id}",
                {title_node}
                input {
                    class: "path-field module-ui-schema__path",
                    r#type: "text",
                    readonly: true,
                    value: "{control_value}",
                }
                {actions_node}
                {children_node}
            }
        },
        "input" | "text-input" | "text-field" => {
            let input_entity_id = entity_id.clone();
            let clear_entity_id = entity_id.clone();
            let clear_enabled = entity.clear_button;
            let clear_disabled = control_value.is_empty() || disabled || readonly;
            rsx! {
                div {
                    class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                    style: "{entity_style}",
                    "data-ui-entity": "text-input",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    div { class: "path-input-shell module-ui-schema__input-shell",
                        input {
                            class: "input-box input module-ui-schema__input",
                            r#type: "text",
                            disabled,
                            readonly,
                            value: "{control_value}",
                            placeholder: "{placeholder}",
                            "data-clear-button": "{clear_enabled}",
                            "data-commit-on-enter": "{entity.commit_on_enter}",
                            oninput: move |event| {
                                module_ui_values.write().insert(
                                    input_entity_id.clone(),
                                    serde_json::Value::String(event.value()),
                                );
                            },
                        }
                        if clear_enabled {
                            button {
                                class: "path-input-clear module-ui-schema__clear",
                                r#type: "button",
                                disabled: clear_disabled,
                                "data-clear-button": "true",
                                "aria-label": "Clear",
                                "data-tooltip": "Clear",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    module_ui_values
                                        .write()
                                        .insert(clear_entity_id.clone(), serde_json::Value::String(String::new()));
                                },
                                img { class: "button__icon", src: "{module_input_clear_src}", alt: "" }
                            }
                        }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "textarea" | "text-area" => {
            let input_entity_id = entity_id.clone();
            let clear_entity_id = entity_id.clone();
            let clear_enabled = entity.clear_button;
            let clear_disabled = control_value.is_empty() || disabled || readonly;
            let textarea_row_style = module_ui_textarea_row_style(&entity);
            let textarea_control_style = module_ui_textarea_control_style(&entity);
            rsx! {
                div {
                    class: "module-ui-schema__row module-ui-schema__row--textarea{titleless_row_class} {layout_class}",
                    style: "{textarea_row_style}",
                    "data-ui-entity": "textarea",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    div { class: "path-input-shell module-ui-schema__input-shell module-ui-schema__input-shell--textarea",
                        textarea {
                            class: "input-box input module-ui-schema__textarea",
                            style: "{textarea_control_style}",
                            disabled,
                            readonly,
                            placeholder: "{placeholder}",
                            value: "{control_value}",
                            "data-clear-button": "{clear_enabled}",
                            "data-commit-on-enter": "{entity.commit_on_enter}",
                            oninput: move |event| {
                                module_ui_values.write().insert(
                                    input_entity_id.clone(),
                                    serde_json::Value::String(event.value()),
                                );
                            },
                        }
                        if clear_enabled {
                            button {
                                class: "path-input-clear module-ui-schema__clear module-ui-schema__clear--textarea",
                                r#type: "button",
                                disabled: clear_disabled,
                                "data-clear-button": "true",
                                "aria-label": "Clear",
                                "data-tooltip": "Clear",
                                "data-tooltip-align": "end",
                                onclick: move |_| {
                                    module_ui_values
                                        .write()
                                        .insert(clear_entity_id.clone(), serde_json::Value::String(String::new()));
                                },
                                img { class: "button__icon", src: "{module_input_clear_src}", alt: "" }
                            }
                        }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "select" | "dropdown" | "combo-box" => {
            let select_entity_id = entity_id.clone();
            rsx! {
                div {
                    class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                    style: "{entity_style}",
                    "data-ui-entity": "select",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    select {
                        class: "input-box select module-ui-schema__select",
                        disabled,
                        value: "{control_value}",
                        onchange: move |event| {
                            module_ui_values.write().insert(
                                select_entity_id.clone(),
                                serde_json::Value::String(event.value()),
                            );
                        },
                        for option in entity.options.iter() {
                            option {
                                value: "{option.value}",
                                selected: option.value == control_value,
                                "{option.label}"
                            }
                        }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "switch" | "toggle" => {
            let switch_entity_id = entity_id.clone();
            rsx! {
                div {
                    class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                    style: "{entity_style}",
                    "data-ui-entity": "switch",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    button {
                        class: if control_checked { "input-box switch switch--on" } else { "input-box switch" },
                        r#type: "button",
                        disabled,
                        "aria-pressed": "{control_checked}",
                        "aria-label": "{title}",
                        onclick: move |_| {
                            if !disabled {
                                let current = module_ui_values
                                    .read()
                                    .get(&switch_entity_id)
                                    .and_then(|value| value.as_bool())
                                    .unwrap_or(control_checked);
                                module_ui_values
                                    .write()
                                    .insert(switch_entity_id.clone(), serde_json::Value::Bool(!current));
                            }
                        },
                        span { class: "switch__knob" }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "status" | "status-label" => rsx! {
            div {
                class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "status-label",
                "data-ui-key": "{entity_id}",
                {title_node}
                span { class: "state-label module-ui-schema__status", "{control_value}" }
                {actions_node}
                {children_node}
            }
        },
        "value" | "value-label" => rsx! {
            div {
                class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "value-label",
                "data-ui-key": "{entity_id}",
                {title_node}
                span { class: "module-ui-schema__value", "{control_value}" }
                {actions_node}
                {children_node}
            }
        },
        "button" | "action-button" => {
            let button_row_style = module_ui_button_row_style(&entity);
            let button_content_style = module_ui_button_content_style(&entity);
            rsx! {
                div {
                    class: "module-ui-schema__button-row {layout_class}",
                    style: "{button_row_style}",
                    "data-ui-entity": "button",
                    "data-ui-key": "{entity_id}",
                    div {
                        class: "module-ui-schema__button-row-content",
                        style: "{button_content_style}",
                        {actions_node}
                        {children_node}
                    }
                }
            }
        }
        "tabs" | "tab-view" => {
            let active_tab = module_ui_active_tab(&entity, &control_value);
            let active_children = module_ui_active_tab_children(&entity, &active_tab);
            rsx! {
                div {
                    class: "module-ui-schema__tabs {layout_class}",
                    style: "{entity_style}",
                    "data-ui-entity": "tabs",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    div { class: "tabs module-ui-schema__tabs-shell",
                        div { class: "tabs__list", role: "tablist",
                            for option in entity.options.iter() {
                                {
                                    let tab_entity_id = entity_id.clone();
                                    let tab_value = option.value.clone();
                                    let tab_label = option.label.clone();
                                    let is_active = tab_value == active_tab;
                                    rsx! {
                                        button {
                                            class: if is_active { "tabs__tab tabs__tab--active" } else { "tabs__tab" },
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": "{is_active}",
                                            disabled,
                                            onclick: move |_| {
                                                if !disabled {
                                                    module_ui_values.write().insert(
                                                        tab_entity_id.clone(),
                                                        serde_json::Value::String(tab_value.clone()),
                                                    );
                                                }
                                            },
                                            "{tab_label}"
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "tabs__body module-ui-schema__tabs-body {scroll_class}",
                            for child in active_children.iter().cloned() {
                                IntegrationUiEntityView {
                                    entity: child,
                                    module_id: module_id.clone(),
                                    module: module.clone(),
                                    selected_monitoring_row_ids: selected_monitoring_row_ids.clone(),
                                    displayed_monitoring_row_ids: displayed_monitoring_row_ids.clone(),
                                    filters: filters.clone(),
                                    watcher,
                                    status_history,
                                    module_host_dialog,
                                    module_ui_page,
                                    module_ui_values,
                                    module_ui_action_generation,
                                    module_path_picker_dialog_open,
                                    module_title: module_title.clone(),
                                }
                            }
                        }
                    }
                    {actions_node}
                }
            }
        }
        "footer" => rsx! {
            div {
                class: "module-ui-schema__footer {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "footer",
                "data-ui-key": "{entity_id}",
                if !value.is_empty() {
                    span { class: "module-ui-schema__value", "{value}" }
                }
                {actions_node}
                {children_node}
            }
        },
        "table" => {
            let table_shell_style = module_ui_table_shell_style(&entity);
            let table_viewport_style = module_ui_table_viewport_style(&entity);
            let rows = module_ui_table_rows(&value);
            let sort_idle_icon_src = inline_svg_data_uri(SORT_IDLE_ICON_SVG);
            let sort_asc_icon_src = inline_svg_data_uri(SORT_ASC_ICON_SVG);
            let sort_desc_icon_src = inline_svg_data_uri(SORT_DESC_ICON_SVG);
            rsx! {
                div {
                    class: "module-ui-schema__table {layout_class}",
                    style: "{table_shell_style}",
                    "data-ui-entity": "table",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    div {
                        class: "module-ui-schema__table-frame table-wrap",
                        style: "{table_viewport_style}",
                        if let Some(header) = rows.first() {
                            div {
                                class: "table-header-wrap",
                                div {
                                    class: "table-header-scroll",
                                    table { class: "ui-entity-table ui-entity-table--header",
                                    colgroup {
                                        for column_index in 0..header.len() {
                                            col { style: "{module_ui_table_column_style(&entity, column_index)}" }
                                        }
                                    }
                                    thead {
                                        tr {
                                            for (column_index, cell) in header.iter().enumerate() {
                                                SortHeaderCell {
                                                    label: cell.clone(),
                                                    tooltip: cell.clone(),
                                                    sort_state: module_table_sort().indicator_state(column_index),
                                                    sort_idle_icon_src: sort_idle_icon_src.clone(),
                                                    sort_asc_icon_src: sort_asc_icon_src.clone(),
                                                    sort_desc_icon_src: sort_desc_icon_src.clone(),
                                                    on_click: move |_| {
                                                        module_table_sort.set(module_table_sort().toggled(column_index));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                }
                                div { class: "table-header-scrollbar-fill" }
                            }
                            div {
                                class: "table-body-wrap module-ui-schema__table-body {scroll_class}",
                                {
                                    let body_rows = module_ui_sorted_table_body(&rows, module_table_sort());
                                    rsx! {
                                        table { class: "ui-entity-table ui-entity-table--body",
                                    colgroup {
                                        for column_index in 0..header.len() {
                                            col { style: "{module_ui_table_column_style(&entity, column_index)}" }
                                        }
                                    }
                                    tbody {
                                        if body_rows.is_empty() {
                                            tr {
                                                td { colspan: "{header.len().max(1)}", "-" }
                                            }
                                        } else {
                                            for row in body_rows.iter() {
                                                tr {
                                                    for (column_index, cell) in row.iter().enumerate() {
                                                        {
                                                            let cell_class = module_ui_table_cell_class(&entity, column_index);
                                                            let cell_style = module_ui_table_column_style(&entity, column_index);
                                                            let use_text_field = module_ui_table_column_text_field(&entity, column_index);
                                                            rsx! {
                                                                td {
                                                                    class: "{cell_class}",
                                                                    style: "{cell_style}",
                                                                    if use_text_field {
                                                                        input {
                                                                            class: "path-field module-ui-schema__table-cell-field",
                                                                            r#type: "text",
                                                                            readonly: true,
                                                                            tabindex: "0",
                                                                            value: "{cell}",
                                                                            "aria-label": "{cell}",
                                                                        }
                                                                    } else {
                                                                        "{cell}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: "table-body-wrap module-ui-schema__table-body {scroll_class}",
                                table { class: "ui-entity-table ui-entity-table--body",
                                    tbody {
                                        tr { td { "-" } }
                                    }
                                }
                            }
                        }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "progress" => {
            let percent = module_ui_parse_progress_percent(&value);
            let label = if title.is_empty() {
                "Progress".to_string()
            } else {
                title
            };
            let compact = entity.compact;
            let hide_label = entity.hide_label;
            let compact_label_class = if compact && !hide_label {
                " module-ui-schema__progress--compact-labeled"
            } else {
                ""
            };
            let progress_stages = module_ui_progress_stages(&entity);
            let progress_meta = {
                let normalized = normalize_progress_stages(progress_stages.clone());
                let current_stage = progress_current_stage_label(&normalized, percent);
                progress_current_text(percent, &current_stage)
            };
            rsx! {
                div {
                    class: "module-ui-schema__progress {layout_class}{compact_label_class}",
                    style: "{entity_style}",
                    "data-ui-entity": "progress-bar",
                    "data-ui-key": "{entity_id}",
                    if compact && !hide_label {
                        {title_node}
                    }
                    ProgressBar {
                        label,
                        meta: progress_meta,
                        percent,
                        stages: progress_stages,
                        compact,
                        hide_label,
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "grid" | "layout-grid" => {
            let grid_style = module_ui_grid_style(&entity);
            rsx! {
                div {
                    class: "module-ui-schema__panel module-ui-schema__grid {scroll_class} {layout_class}",
                    style: "{grid_style}",
                    "data-ui-entity": "grid",
                    "data-ui-key": "{entity_id}",
                    {title_node}
                    if !value.is_empty() {
                        span { class: "module-ui-schema__value", "{value}" }
                    }
                    {actions_node}
                    {children_node}
                }
            }
        }
        "panel" | "subpanel" => rsx! {
            div {
                class: "module-ui-schema__panel {scroll_class} {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "{entity_type}",
                "data-ui-key": "{entity_id}",
                {title_node}
                if !value.is_empty() {
                    span { class: "module-ui-schema__value", "{value}" }
                }
                {actions_node}
                {children_node}
            }
        },
        _ => rsx! {
            div {
                class: "module-ui-schema__row{titleless_row_class} {layout_class}",
                style: "{entity_style}",
                "data-ui-entity": "{entity_type}",
                "data-ui-key": "{entity_id}",
                {title_node}
                if !value.is_empty() {
                    span { class: "module-ui-schema__value", "{value}" }
                }
                {actions_node}
                {children_node}
            }
        },
    }
}

#[component]
fn HelpIcon(
    icon_src: String,
    tooltip: String,
    #[props(default = "start")] align: &'static str,
    #[props(default = "help-icon".to_string())] class: String,
) -> Element {
    let _ = icon_src;
    rsx! {
        span {
            class: "{class}",
            "data-tooltip": "{tooltip}",
            "data-tooltip-align": "{align}",
            "aria-label": "{tooltip}",
            tabindex: "0",
            "?"
        }
    }
}

fn handle_scan_toggle(
    watcher: &mut Signal<AppWatcherApi>,
    window: &DesktopContext,
    show_integration_prompt: Signal<bool>,
    integration_path_input: Signal<String>,
    integration_feedback: Signal<Option<String>>,
    pending_scan_after_setup: Signal<bool>,
    mut window_visible: Signal<bool>,
    remember_window_placement: bool,
    mut remembered_window_placement: Signal<Option<PersistedWindowPlacement>>,
    hidden_window_was_maximized: Signal<bool>,
    integration_folder_dialog_open: Signal<bool>,
) -> Option<bool> {
    let ready = watcher.read().snapshot().integration.ready;
    let monitoring = watcher.read().snapshot().ui.monitoring;

    if !ready {
        restore_window(
            window,
            hidden_window_was_maximized(),
            remember_window_placement,
            &mut remembered_window_placement,
        );
        window_visible.set(true);
        open_integration_folder_dialog_for_scan(
            *watcher,
            show_integration_prompt,
            integration_path_input,
            integration_feedback,
            pending_scan_after_setup,
            monitoring,
            integration_folder_dialog_open,
            window.clone(),
        );
        return None;
    }

    Some(toggle_monitoring(watcher))
}

fn toggle_monitoring(watcher: &mut Signal<AppWatcherApi>) -> bool {
    let mut current = watcher.write();
    if current.snapshot().ui.monitoring {
        current.stop_monitoring();
        false
    } else {
        current.start_monitoring();
        true
    }
}

fn apply_observation_selection_click(
    mut last_anchor: Signal<Option<u64>>,
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
    visible_observations: &[ObservationDto],
    observation_id: u64,
    ctrl_pressed: bool,
    shift_pressed: bool,
) {
    if !ctrl_pressed && !shift_pressed {
        last_anchor.set(Some(observation_id));
        return;
    }
    let anchor_id = last_anchor();
    let (select_ids, deselect_ids) = observation_selection_batches(
        visible_observations,
        observation_id,
        anchor_id,
        &clone_observation_selection_store(selection_store),
        ctrl_pressed,
        shift_pressed,
    );
    if !select_ids.is_empty() {
        update_observation_selection_store(selection_store, &select_ids, true);
        queue_observation_selection_confirm(pending_confirm, &select_ids, true);
    }
    if !deselect_ids.is_empty() {
        update_observation_selection_store(selection_store, &deselect_ids, false);
        queue_observation_selection_confirm(pending_confirm, &deselect_ids, false);
    }
    last_anchor.set(Some(observation_id));
}

fn toggle_observation_selection_button(
    mut last_anchor: Signal<Option<u64>>,
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
    observation_id: u64,
) {
    let selected = !observation_row_is_selected(selection_store, observation_id);
    apply_observation_selection_ids(
        selection_store,
        pending_confirm,
        &[observation_id],
        selected,
    );
    last_anchor.set(Some(observation_id));
}

fn apply_observation_selection_ids(
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
    observation_ids: &[u64],
    selected: bool,
) {
    if observation_ids.is_empty() {
        return;
    }
    update_observation_selection_store(selection_store, observation_ids, selected);
    queue_observation_selection_confirm(pending_confirm, observation_ids, selected);
}

fn apply_cloud_download_selection_click(
    mut cloud_state: Signal<CloudSyncUiState>,
    mut last_anchor: Signal<Option<String>>,
    visible_rows: &[CloudDownloadedObservation],
    row_id: &str,
    ctrl_pressed: bool,
    shift_pressed: bool,
) {
    if !ctrl_pressed && !shift_pressed {
        last_anchor.set(Some(row_id.to_string()));
        return;
    }
    let state = cloud_state();
    let anchor_id = last_anchor();
    let (select_ids, deselect_ids) = cloud_download_selection_batches(
        visible_rows,
        row_id,
        anchor_id.as_deref(),
        &state.selected_download_row_ids,
        ctrl_pressed,
        shift_pressed,
    );
    if select_ids.is_empty() && deselect_ids.is_empty() {
        last_anchor.set(Some(row_id.to_string()));
        return;
    }
    let mut next_state = state;
    for id in select_ids {
        next_state.selected_download_row_ids.insert(id);
    }
    for id in deselect_ids {
        next_state.selected_download_row_ids.remove(&id);
    }
    cloud_state.set(next_state);
    last_anchor.set(Some(row_id.to_string()));
}

fn toggle_cloud_download_row_selection(
    mut cloud_state: Signal<CloudSyncUiState>,
    mut last_anchor: Signal<Option<String>>,
    row_id: String,
) {
    let mut state = cloud_state();
    if state.selected_download_row_ids.contains(&row_id) {
        state.selected_download_row_ids.remove(&row_id);
    } else {
        state.selected_download_row_ids.insert(row_id.clone());
    }
    cloud_state.set(state);
    last_anchor.set(Some(row_id));
}

fn sync_observation_selection_store(
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
    observations: &[ObservationDto],
) {
    let visible_ids = observations
        .iter()
        .map(|observation| observation.id)
        .collect::<BTreeSet<_>>();
    let snapshot_selected_ids = observations
        .iter()
        .filter(|observation| observation.is_confirmed && !observation.is_exported)
        .map(|observation| observation.id)
        .collect::<BTreeSet<_>>();
    let exported_ids = observations
        .iter()
        .filter(|observation| observation.is_exported)
        .map(|observation| observation.id)
        .collect::<BTreeSet<_>>();
    let mut selected_ids = snapshot_selected_ids.clone();
    if let Ok(store) = selection_store.lock() {
        if store.initialized {
            selected_ids.extend(
                store
                    .selected_ids
                    .iter()
                    .copied()
                    .filter(|id| exported_ids.contains(id)),
            );
        }
    }
    if let Ok(mut pending) = pending_confirm.lock() {
        for observation_id in &pending.select_ids {
            if visible_ids.contains(observation_id) {
                selected_ids.insert(*observation_id);
            }
        }
        for observation_id in &pending.deselect_ids {
            selected_ids.remove(observation_id);
        }
        for observation_id in &pending.in_flight_select_ids {
            if visible_ids.contains(observation_id) {
                selected_ids.insert(*observation_id);
            }
        }
        for observation_id in &pending.in_flight_deselect_ids {
            selected_ids.remove(observation_id);
        }
        pending
            .in_flight_select_ids
            .retain(|id| visible_ids.contains(id) && !snapshot_selected_ids.contains(id));
        pending
            .in_flight_deselect_ids
            .retain(|id| visible_ids.contains(id) && snapshot_selected_ids.contains(id));
    }
    if let Ok(mut store) = selection_store.lock() {
        store.selected_ids = selected_ids;
        store.initialized = true;
    }
}

fn clone_observation_selection_store(
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
) -> BTreeSet<u64> {
    selection_store
        .lock()
        .map(|store| store.selected_ids.clone())
        .unwrap_or_default()
}

fn observation_row_is_selected(
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    observation_id: u64,
) -> bool {
    selection_store
        .lock()
        .ok()
        .is_some_and(|store| store.selected_ids.contains(&observation_id))
}

fn module_ui_schema_with_context(
    entities: &[IntegrationUiEntityDto],
    page: &str,
    selected_rows: usize,
    displayed_rows: usize,
    total_rows: usize,
    module_background_active: bool,
    last_row: &str,
    latest_rows: &str,
    latest_rows_count: usize,
) -> Vec<IntegrationUiEntityDto> {
    entities
        .iter()
        .cloned()
        .filter(|entity| module_ui_entity_matches_page(entity, page))
        .map(|entity| {
            module_ui_entity_with_context(
                entity,
                page,
                selected_rows,
                displayed_rows,
                total_rows,
                module_background_active,
                last_row,
                latest_rows,
                latest_rows_count,
            )
        })
        .collect()
}

fn module_ui_entity_matches_page(entity: &IntegrationUiEntityDto, page: &str) -> bool {
    if entity.hidden || entity.visible == Some(false) {
        return false;
    }
    entity
        .page
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none_or(|entity_page| entity_page == page)
}

fn module_ui_entity_type_is_tabs(entity_type: &str) -> bool {
    matches!(
        entity_type.replace('_', "-").to_ascii_lowercase().as_str(),
        "tabs" | "tab-view"
    )
}

fn module_ui_entity_with_context(
    mut entity: IntegrationUiEntityDto,
    page: &str,
    selected_rows: usize,
    displayed_rows: usize,
    total_rows: usize,
    module_background_active: bool,
    last_row: &str,
    latest_rows: &str,
    latest_rows_count: usize,
) -> IntegrationUiEntityDto {
    entity.title = entity.title.map(|value| {
        module_ui_context_value(
            &value,
            selected_rows,
            displayed_rows,
            total_rows,
            module_background_active,
            last_row,
            latest_rows,
            latest_rows_count,
        )
    });
    entity.tooltip = entity.tooltip.map(|value| {
        module_ui_context_value(
            &value,
            selected_rows,
            displayed_rows,
            total_rows,
            module_background_active,
            last_row,
            latest_rows,
            latest_rows_count,
        )
    });
    entity.value = entity.value.map(|value| {
        module_ui_context_value(
            &value,
            selected_rows,
            displayed_rows,
            total_rows,
            module_background_active,
            last_row,
            latest_rows,
            latest_rows_count,
        )
    });
    entity.placeholder = entity.placeholder.map(|value| {
        module_ui_context_value(
            &value,
            selected_rows,
            displayed_rows,
            total_rows,
            module_background_active,
            last_row,
            latest_rows,
            latest_rows_count,
        )
    });
    entity.options = entity
        .options
        .into_iter()
        .map(|mut option| {
            option.label = module_ui_context_value(
                &option.label,
                selected_rows,
                displayed_rows,
                total_rows,
                module_background_active,
                last_row,
                latest_rows,
                latest_rows_count,
            );
            option
        })
        .collect();
    entity.actions = entity
        .actions
        .into_iter()
        .map(|mut action| {
            action.label = module_ui_context_value(
                &action.label,
                selected_rows,
                displayed_rows,
                total_rows,
                module_background_active,
                last_row,
                latest_rows,
                latest_rows_count,
            );
            action.tooltip = action.tooltip.map(|value| {
                module_ui_context_value(
                    &value,
                    selected_rows,
                    displayed_rows,
                    total_rows,
                    module_background_active,
                    last_row,
                    latest_rows,
                    latest_rows_count,
                )
            });
            action
        })
        .collect();
    let child_page = entity
        .page
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(page)
        .to_string();
    let filter_children_by_page = !module_ui_entity_type_is_tabs(&entity.entity_type);
    entity.children = entity
        .children
        .into_iter()
        .filter(|child| {
            !filter_children_by_page || module_ui_entity_matches_page(child, &child_page)
        })
        .map(|child| {
            module_ui_entity_with_context(
                child,
                &child_page,
                selected_rows,
                displayed_rows,
                total_rows,
                module_background_active,
                last_row,
                latest_rows,
                latest_rows_count,
            )
        })
        .collect();
    entity
}

fn module_ui_active_tab(entity: &IntegrationUiEntityDto, control_value: &str) -> String {
    let requested = control_value.trim();
    if !requested.is_empty()
        && entity
            .options
            .iter()
            .any(|option| option.value.trim() == requested)
    {
        return requested.to_string();
    }
    entity
        .value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| entity.options.first().map(|option| option.value.clone()))
        .unwrap_or_default()
}

fn module_ui_active_tab_children(
    entity: &IntegrationUiEntityDto,
    active_tab: &str,
) -> Vec<IntegrationUiEntityDto> {
    entity
        .children
        .iter()
        .filter(|child| {
            child
                .page
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none_or(|page| page == active_tab)
        })
        .cloned()
        .collect()
}

fn module_ui_entities_without_footers(
    entities: Vec<IntegrationUiEntityDto>,
) -> Vec<IntegrationUiEntityDto> {
    entities
        .into_iter()
        .filter_map(|mut entity| {
            if module_ui_entity_type_is_footer(&entity.entity_type) {
                return None;
            }
            entity.children = module_ui_entities_without_footers(entity.children);
            Some(entity)
        })
        .collect()
}

fn module_ui_footer_entities(entities: Vec<IntegrationUiEntityDto>) -> Vec<IntegrationUiEntityDto> {
    let mut footers = Vec::new();
    collect_module_ui_footer_entities(entities, &mut footers);
    footers
}

fn collect_module_ui_footer_entities(
    entities: Vec<IntegrationUiEntityDto>,
    footers: &mut Vec<IntegrationUiEntityDto>,
) {
    for mut entity in entities {
        let children = std::mem::take(&mut entity.children);
        if module_ui_entity_type_is_footer(&entity.entity_type) {
            entity.children = children;
            footers.push(entity);
        } else {
            collect_module_ui_footer_entities(children, footers);
        }
    }
}

fn module_ui_entity_type_is_footer(entity_type: &str) -> bool {
    entity_type.replace('_', "-").eq_ignore_ascii_case("footer")
}

fn module_ui_action_payload(
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    module_background_active: bool,
) -> serde_json::Value {
    serde_json::json!({
        "ui_values": module_ui_values.read().clone(),
        "module_background_active": module_background_active,
        "module": {
            "background_active": module_background_active,
        },
    })
}

fn module_ui_entity_with_layout_defaults(
    mut entity: IntegrationUiEntityDto,
) -> IntegrationUiEntityDto {
    let entity_type = entity.entity_type.replace('_', "-").to_ascii_lowercase();
    set_option_if_blank(&mut entity.size, "stretch");
    set_option_if_blank(&mut entity.width, "100%");
    set_option_if_blank(&mut entity.min_width, "0");
    set_string_if_blank(&mut entity.opacity, "100%");
    match entity_type.as_str() {
        "panel" | "subpanel" | "grid" | "layout-grid" | "tabs" | "tab-view" => {
            set_option_if_blank(&mut entity.height, "auto");
            set_option_if_blank(&mut entity.min_height, "0");
            set_option_if_blank(&mut entity.scroll, "off");
        }
        "button" | "action-button" => {
            set_option_if_blank(&mut entity.margin, "8px 0 0");
            set_option_if_blank(&mut entity.padding, "0");
        }
        "separator" | "help-text" | "help" | "table" | "progress" | "footer" => {
            set_option_if_blank(&mut entity.min_height, "0");
        }
        _ => {}
    }
    if matches!(entity_type.as_str(), "grid" | "layout-grid") {
        set_option_if_blank(&mut entity.columns, "repeat(auto-fit, minmax(180px, 1fr))");
        set_option_if_blank(&mut entity.gap, "8px");
    }
    if entity
        .align
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        entity.align = Some("left".to_string());
    }
    entity
}

fn set_string_if_blank(target: &mut String, value: &str) {
    if target.trim().is_empty() {
        *target = value.to_string();
    }
}

fn set_option_if_blank(target: &mut Option<String>, value: &str) {
    if target
        .as_ref()
        .is_none_or(|current| current.trim().is_empty())
    {
        *target = Some(value.to_string());
    }
}

#[allow(clippy::too_many_arguments)]
fn start_module_ui_action(
    watcher: Signal<AppWatcherApi>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    module_host_dialog: Signal<Option<ModuleHostDialogState>>,
    module_ui_page: Signal<String>,
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    module_ui_action_generation: Signal<u64>,
    module_path_picker_dialog_open: Signal<bool>,
    window: DesktopContext,
    module: IntegrationModuleDto,
    module_title: String,
    action_label: String,
    mut request: IntegrationModuleUiActionClientRequestDto,
) {
    let generation = module_ui_action_generation();
    let ui_action_token = module_ui_action_token(&module.id, &request.action_id, generation);
    request.ui_action_token = ui_action_token.clone();
    let job = watcher
        .read()
        .start_integration_module_ui_action_job(request);
    spawn(async move {
        let mut last_event_seq = 0_u64;
        loop {
            if module_ui_action_generation() != generation {
                return;
            }
            last_event_seq = poll_module_ui_action_events(
                watcher,
                &module,
                &ui_action_token,
                last_event_seq,
                module_ui_values,
                module_ui_action_generation,
                generation,
            );
            if let Some(result) = job.try_finish() {
                if module_ui_action_generation() != generation {
                    return;
                }
                let _ = poll_module_ui_action_events(
                    watcher,
                    &module,
                    &ui_action_token,
                    0,
                    module_ui_values,
                    module_ui_action_generation,
                    generation,
                );
                match result {
                    Ok(response) => {
                        watcher
                            .read()
                            .apply_integration_module_ui_action_response(&response);
                        apply_module_host_commands(
                            &module,
                            &response.commands,
                            module_host_dialog,
                            module_ui_page,
                            module_ui_values,
                            status_history,
                            module_path_picker_dialog_open,
                            window.clone(),
                        );
                        if let Some(message) = response.message {
                            push_status_history_line(status_history, message);
                        } else {
                            push_status_history_line(
                                status_history,
                                format!("{module_title}: {action_label}"),
                            );
                        }
                    }
                    Err(message) => push_status_history_line(
                        status_history,
                        format!("{module_title}: {message}"),
                    ),
                }
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
}

fn module_ui_action_token(module_id: &str, action_id: &str, generation: u64) -> String {
    let counter = MODULE_UI_ACTION_TOKEN_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    format!("{module_id}:{action_id}:{generation}:{counter}")
}

fn poll_module_ui_action_events(
    watcher: Signal<AppWatcherApi>,
    module: &IntegrationModuleDto,
    ui_action_token: &str,
    after: u64,
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    module_ui_action_generation: Signal<u64>,
    generation: u64,
) -> u64 {
    if ui_action_token.trim().is_empty() || module_ui_action_generation() != generation {
        return after;
    }
    let Ok(response) =
        watcher
            .read()
            .integration_module_ui_action_events(&module.id, ui_action_token, after)
    else {
        return after;
    };
    if module_ui_action_generation() != generation {
        return after;
    }
    let mut last_seq = after;
    for event in response.events {
        last_seq = last_seq.max(event.seq);
        if event.event_type == "ui_values" {
            apply_module_ui_values_payload(&event.payload, module_ui_values);
        }
    }
    last_seq
}

fn module_ui_control_value(
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    entity_id: &str,
    default_value: &str,
) -> String {
    module_ui_values
        .read()
        .get(entity_id)
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_value.to_string())
}

fn module_ui_control_checked(
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    entity_id: &str,
    default_value: bool,
) -> bool {
    module_ui_values
        .read()
        .get(entity_id)
        .and_then(|value| value.as_bool())
        .unwrap_or(default_value)
}

fn module_action_style_class(style: Option<&str>) -> &'static str {
    match style
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "primary" | "blue" | "accent" => "button--primary",
        _ => "button--secondary",
    }
}

fn module_action_align_class(align: Option<&str>) -> &'static str {
    match align
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "center" | "centre" => "module-ui-schema__action--align-center",
        "right" | "end" => "module-ui-schema__action--align-right",
        _ => "module-ui-schema__action--align-left",
    }
}

fn module_actions_layout_class(actions: &[IntegrationModuleActionDto]) -> &'static str {
    if actions.iter().any(|action| {
        action
            .align
            .as_deref()
            .is_some_and(|align| !align.trim().is_empty())
    }) {
        "module-ui-schema__actions--split"
    } else {
        ""
    }
}

fn module_action_pulse_class(
    action: &IntegrationModuleActionDto,
    module_background_active: bool,
) -> &'static str {
    if action.pulse || (action.pulse_when_background_active && module_background_active) {
        "module-action-button--pulse"
    } else {
        ""
    }
}

fn module_ui_scroll_class(scroll: Option<&str>) -> &'static str {
    match scroll
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "x" | "horizontal" => "module-ui-schema--scroll-x",
        "y" | "vertical" => "module-ui-schema--scroll-y",
        "both" | "xy" | "x-y" => "module-ui-schema--scroll-both",
        _ => "",
    }
}

fn module_ui_size_class(size: Option<&str>) -> &'static str {
    match size
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "auto" | "content" | "fit" | "fit-content" => "module-ui-schema--size-auto",
        "stretch" | "fill" | "wide" => "module-ui-schema--size-stretch",
        "fullscreen" | "full-screen" | "full" => "module-ui-schema--size-fullscreen",
        _ => "",
    }
}

fn module_ui_align_class(align: Option<&str>) -> &'static str {
    match align
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "left" | "start" => "module-ui-schema--align-left",
        "center" | "centre" | "middle" => "module-ui-schema--align-center",
        "right" | "end" => "module-ui-schema--align-right",
        _ => "module-ui-schema--align-left",
    }
}

fn module_ui_layout_class(entity_type: &str, size_class: &str, align_class: &str) -> String {
    let _ = entity_type;
    match (size_class.is_empty(), align_class.is_empty()) {
        (true, true) => String::new(),
        (false, true) => size_class.to_string(),
        (true, false) => align_class.to_string(),
        (false, false) => format!("{size_class} {align_class}"),
    }
}

fn module_ui_entity_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("width", entity.width.as_deref()),
            ("height", entity.height.as_deref()),
            ("min-width", entity.min_width.as_deref()),
            ("min-height", entity.min_height.as_deref()),
            ("max-width", entity.max_width.as_deref()),
            ("max-height", entity.max_height.as_deref()),
            ("margin", entity.margin.as_deref()),
            ("padding", entity.padding.as_deref()),
            ("grid-column", entity.grid_column.as_deref()),
            ("grid-row", entity.grid_row.as_deref()),
        ],
        Some(&entity.opacity),
    )
}

fn module_ui_textarea_row_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("width", entity.width.as_deref()),
            ("min-width", entity.min_width.as_deref()),
            ("max-width", entity.max_width.as_deref()),
            ("margin", entity.margin.as_deref()),
            ("padding", entity.padding.as_deref()),
            ("grid-column", entity.grid_column.as_deref()),
            ("grid-row", entity.grid_row.as_deref()),
        ],
        Some(&entity.opacity),
    )
}

fn module_ui_textarea_control_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("height", entity.height.as_deref()),
            ("min-height", entity.min_height.as_deref()),
            ("max-height", entity.max_height.as_deref()),
        ],
        None,
    )
}

fn module_ui_grid_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("width", entity.width.as_deref()),
            ("height", entity.height.as_deref()),
            ("min-width", entity.min_width.as_deref()),
            ("min-height", entity.min_height.as_deref()),
            ("max-width", entity.max_width.as_deref()),
            ("max-height", entity.max_height.as_deref()),
            ("margin", entity.margin.as_deref()),
            ("padding", entity.padding.as_deref()),
            ("grid-template-columns", entity.columns.as_deref()),
            ("grid-template-rows", entity.rows.as_deref()),
            ("gap", entity.gap.as_deref()),
            ("grid-column", entity.grid_column.as_deref()),
            ("grid-row", entity.grid_row.as_deref()),
        ],
        Some(&entity.opacity),
    )
}

fn module_ui_table_shell_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("width", entity.width.as_deref()),
            ("height", entity.height.as_deref()),
            ("min-width", entity.min_width.as_deref()),
            ("min-height", entity.min_height.as_deref()),
            ("max-width", entity.max_width.as_deref()),
            ("max-height", entity.max_height.as_deref()),
            ("margin", entity.margin.as_deref()),
            ("padding", entity.padding.as_deref()),
            ("grid-column", entity.grid_column.as_deref()),
            ("grid-row", entity.grid_row.as_deref()),
        ],
        Some(&entity.opacity),
    )
}

fn module_ui_table_viewport_style(entity: &IntegrationUiEntityDto) -> String {
    let _ = entity;
    String::new()
}

fn module_ui_table_column(
    entity: &IntegrationUiEntityDto,
    index: usize,
) -> Option<&netstitch_shared::models::IntegrationUiTableColumnDto> {
    entity
        .table_columns
        .iter()
        .find(|column| column.index == index)
}

fn module_ui_table_column_style(entity: &IntegrationUiEntityDto, index: usize) -> String {
    let Some(column) = module_ui_table_column(entity, index) else {
        return String::new();
    };
    module_ui_style_from_pairs(
        &[
            ("width", column.width.as_deref()),
            ("min-width", column.min_width.as_deref()),
            ("max-width", column.max_width.as_deref()),
        ],
        None,
    )
}

fn module_ui_table_cell_class(entity: &IntegrationUiEntityDto, index: usize) -> &'static str {
    module_ui_table_column(entity, index)
        .map(|column| module_ui_align_class(column.align.as_deref()))
        .unwrap_or("module-ui-schema--align-left")
}

fn module_ui_table_column_text_field(entity: &IntegrationUiEntityDto, index: usize) -> bool {
    module_ui_table_column(entity, index)
        .map(|column| column.text_field)
        .unwrap_or(false)
}

fn module_ui_button_row_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(
        &[
            ("width", entity.width.as_deref()),
            ("height", entity.height.as_deref()),
            ("min-width", entity.min_width.as_deref()),
            ("min-height", entity.min_height.as_deref()),
            ("max-width", entity.max_width.as_deref()),
            ("max-height", entity.max_height.as_deref()),
            ("margin", entity.margin.as_deref()),
            ("grid-column", entity.grid_column.as_deref()),
            ("grid-row", entity.grid_row.as_deref()),
        ],
        Some(&entity.opacity),
    )
}

fn module_ui_button_content_style(entity: &IntegrationUiEntityDto) -> String {
    module_ui_style_from_pairs(&[("padding", entity.padding.as_deref())], None)
}

fn module_ui_style_from_pairs(pairs: &[(&str, Option<&str>)], opacity: Option<&str>) -> String {
    pairs
        .iter()
        .filter_map(|(name, value)| {
            module_ui_sanitize_dimension(*value).map(|value| format!("{name}: {value}"))
        })
        .chain(
            opacity
                .and_then(module_ui_sanitize_opacity)
                .map(|value| format!("opacity: {value}")),
        )
        .collect::<Vec<_>>()
        .join("; ")
}

fn module_ui_sanitize_dimension(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty()
        || value.len() > 96
        || value.contains(';')
        || value.contains('{')
        || value.contains('}')
        || value.contains(':')
    {
        return None;
    }
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                ' ' | '\t' | '.' | '%' | '(' | ')' | ',' | '+' | '-' | '*' | '/'
            )
    }) {
        Some(value.to_string())
    } else {
        None
    }
}

fn module_ui_sanitize_opacity(value: &str) -> Option<String> {
    let value = value.trim();
    let percent = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
    if !(0.0..=100.0).contains(&percent) {
        return None;
    }
    if percent.fract() == 0.0 {
        Some(format!("{}%", percent as u8))
    } else {
        Some(format!("{percent:.2}%"))
    }
}

fn module_ui_parse_progress_percent(value: &str) -> u8 {
    value
        .trim()
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok()
        .map(|percent| percent.clamp(0.0, 100.0).round() as u8)
        .unwrap_or(0)
}

fn module_ui_progress_stages(entity: &IntegrationUiEntityDto) -> Vec<ProgressStage> {
    if entity.progress_stages.is_empty() {
        return vec![ProgressStage::new(
            "Progress".to_string(),
            100,
            "progress-bar__segment--accent",
        )];
    }

    let mut stages = Vec::new();
    let mut previous_percent = 0_u8;
    let mut last_class = "progress-bar__segment--accent".to_string();
    let mut last_style = String::new();

    for stage in &entity.progress_stages {
        let Some(boundary) = module_ui_progress_stage_percent(stage.percent.as_ref()) else {
            continue;
        };
        let boundary = boundary.min(100);
        if boundary <= previous_percent {
            continue;
        }
        let color = stage.color.as_deref();
        let mut progress_stage = ProgressStage::new(
            stage.name.clone().unwrap_or_default(),
            boundary.saturating_sub(previous_percent),
            module_ui_progress_stage_class(color),
        );
        progress_stage.style = module_ui_progress_stage_style(color);
        last_class = progress_stage.class_name.clone();
        last_style = progress_stage.style.clone();
        stages.push(progress_stage);
        previous_percent = boundary;
        if previous_percent >= 100 {
            break;
        }
    }

    if stages.is_empty() {
        return vec![ProgressStage::new(
            "Progress".to_string(),
            100,
            "progress-bar__segment--accent",
        )];
    }
    if previous_percent < 100 {
        let mut remainder = ProgressStage::new(
            String::new(),
            100_u8.saturating_sub(previous_percent),
            last_class,
        );
        remainder.style = last_style;
        stages.push(remainder);
    }
    stages
}

fn module_ui_progress_stage_percent(value: Option<&serde_json::Value>) -> Option<u8> {
    match value? {
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(|percent| percent.clamp(0.0, 100.0).round() as u8),
        serde_json::Value::String(value) => Some(module_ui_parse_progress_percent(value)),
        _ => None,
    }
}

fn module_ui_progress_stage_class(color: Option<&str>) -> &'static str {
    match color
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "success" | "green" => "progress-bar__segment--success",
        "warning" | "yellow" => "progress-bar__segment--warning",
        "danger" | "error" | "red" => "progress-bar__segment--danger",
        "rust" | "orange" => "progress-bar__segment--rust",
        "muted" | "gray" | "grey" => "progress-bar__segment--muted",
        _ => "progress-bar__segment--accent",
    }
}

fn module_ui_progress_stage_style(color: Option<&str>) -> String {
    module_ui_sanitize_progress_color(color).unwrap_or_default()
}

fn module_ui_sanitize_progress_color(color: Option<&str>) -> Option<String> {
    let color = color?.trim();
    if !module_ui_color_is_hex(color) {
        return None;
    }
    Some(format!("background: {color};"))
}

fn module_ui_color_is_hex(color: &str) -> bool {
    let value = color.strip_prefix('#').unwrap_or_default();
    matches!(value.len(), 3 | 4 | 6 | 8) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn module_ui_sorted_table_body(
    rows: &[Vec<String>],
    sort_state: ModuleTableSortState,
) -> Vec<Vec<String>> {
    let mut body = rows.iter().skip(1).cloned().collect::<Vec<_>>();
    let Some(column) = sort_state.column else {
        return body;
    };
    body.sort_by(|left, right| {
        let left_value = left.get(column).map(String::as_str).unwrap_or_default();
        let right_value = right.get(column).map(String::as_str).unwrap_or_default();
        let ordering = module_ui_table_cell_ordering(left_value, right_value);
        if sort_state.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    body
}

fn module_ui_table_cell_ordering(left: &str, right: &str) -> Ordering {
    let left_number = left.trim().parse::<f64>();
    let right_number = right.trim().parse::<f64>();
    match (left_number, right_number) {
        (Ok(left), Ok(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()),
    }
}

fn module_ui_context_value(
    value: &str,
    selected_rows: usize,
    displayed_rows: usize,
    total_rows: usize,
    module_background_active: bool,
    last_row: &str,
    latest_rows: &str,
    latest_rows_count: usize,
) -> String {
    value
        .replace(
            "{context.tables.monitoring.selected_rows}",
            &selected_rows.to_string(),
        )
        .replace(
            "{context.tables.monitoring.selected_total}",
            &format!("{selected_rows}/{total_rows}"),
        )
        .replace(
            "{context.tables.monitoring.displayed_rows}",
            &displayed_rows.to_string(),
        )
        .replace(
            "{context.tables.monitoring.total_rows}",
            &total_rows.to_string(),
        )
        .replace(
            "{context.module.background_active}",
            if module_background_active {
                "true"
            } else {
                "false"
            },
        )
        .replace(
            "{context.module.background_action_label}",
            if module_background_active {
                "Stop"
            } else {
                "Start"
            },
        )
        .replace(
            "{context.module.background_status}",
            if module_background_active {
                "Running in background"
            } else {
                "Stopped"
            },
        )
        .replace("{context.monitoring.last_row}", last_row)
        .replace("{context.monitoring.latest_rows}", latest_rows)
        .replace(
            "{context.monitoring.latest_rows_count}",
            &latest_rows_count.to_string(),
        )
}

fn module_context_last_row(background_active: bool, observations: &[ObservationDto]) -> String {
    if background_active {
        module_last_monitoring_row_label(observations)
    } else {
        "-".to_string()
    }
}

fn module_context_latest_rows(background_active: bool, observations: &[ObservationDto]) -> String {
    if background_active {
        module_latest_monitoring_rows_label(observations, 5)
    } else {
        module_latest_monitoring_rows_label(&[], 5)
    }
}

fn module_context_latest_rows_count(
    background_active: bool,
    observations: &[ObservationDto],
    limit: usize,
) -> usize {
    if background_active {
        observations.len().min(limit)
    } else {
        0
    }
}

fn module_last_monitoring_row_label(observations: &[ObservationDto]) -> String {
    observations
        .iter()
        .max_by_key(|observation| observation.id)
        .map(module_monitoring_row_label)
        .unwrap_or_else(|| "-".to_string())
}

fn module_latest_monitoring_rows_label(observations: &[ObservationDto], limit: usize) -> String {
    let mut rows = observations.iter().collect::<Vec<_>>();
    rows.sort_by_key(|observation| std::cmp::Reverse(observation.id));
    let body = rows
        .into_iter()
        .take(limit)
        .map(module_monitoring_row_table_line)
        .collect::<Vec<_>>()
        .join("\n");
    if body.is_empty() {
        "App\tIP\tDomain\tPort\tProtocol\tConn".to_string()
    } else {
        format!("App\tIP\tDomain\tPort\tProtocol\tConn\n{body}")
    }
}

fn module_monitoring_row_label(observation: &ObservationDto) -> String {
    let domain = observation
        .enrichment
        .as_ref()
        .and_then(|enrichment| enrichment.domain_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    format!(
        "{} {}:{} {} {} ({}/{})",
        observation.process_name,
        observation.remote_ip,
        observation.remote_port,
        observation.protocol.as_str(),
        domain,
        observation.successful_hits,
        observation.failed_hits
    )
}

fn module_monitoring_row_table_line(observation: &ObservationDto) -> String {
    let domain = observation
        .enrichment
        .as_ref()
        .and_then(|enrichment| enrichment.domain_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("-");
    format!(
        "{}\t{}\t{}\t{}\t{}\t{} ({}/{})",
        observation.process_name,
        observation.remote_ip,
        domain,
        observation.remote_port,
        observation.protocol.as_str(),
        observation.connection_state.as_str(),
        observation.successful_hits,
        observation.failed_hits
    )
}

fn module_ui_table_rows(value: &str) -> Vec<Vec<String>> {
    value
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split('\t')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

fn update_observation_selection_store(
    selection_store: &Arc<Mutex<ObservationSelectionStore>>,
    observation_ids: &[u64],
    selected: bool,
) {
    if let Ok(mut store) = selection_store.lock() {
        store.initialized = true;
        for observation_id in observation_ids {
            if selected {
                store.selected_ids.insert(*observation_id);
            } else {
                store.selected_ids.remove(observation_id);
            }
        }
    }
}

fn snapshot_ui_render_relevant_changed(
    before: &SnapshotResponse,
    after: &SnapshotResponse,
) -> bool {
    before.tracked_apps != after.tracked_apps
        || before.observations != after.observations
        || before.ignored_addresses != after.ignored_addresses
        || before.integration != after.integration
        || before.ui != after.ui
        || before.app_settings != after.app_settings
        || before.filters != after.filters
        || before.pending_exe_path != after.pending_exe_path
        || runtime_status_ui_render_relevant_changed(before, after)
}

fn runtime_status_ui_render_relevant_changed(
    before: &SnapshotResponse,
    after: &SnapshotResponse,
) -> bool {
    let left = &before.runtime_status;
    let right = &after.runtime_status;

    left.connector_apps_detected != right.connector_apps_detected
        || left.unavailable_tracked_apps != right.unavailable_tracked_apps
        || left.tool_available != right.tool_available
        || left.domain_capture.backend_started != right.domain_capture.backend_started
        || left.domain_capture.backend_error != right.domain_capture.backend_error
        || left.flow_capture.flow_backend_started != right.flow_capture.flow_backend_started
        || left.flow_capture.packet_backend_started != right.flow_capture.packet_backend_started
        || left.flow_capture.backend_error != right.flow_capture.backend_error
        || left.domain_capture_admin_disabled != right.domain_capture_admin_disabled
        || left.is_elevated != right.is_elevated
}

fn queue_observation_selection_confirm(
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
    observation_ids: &[u64],
    selected: bool,
) {
    if let Ok(mut pending) = pending_confirm.lock() {
        for observation_id in observation_ids {
            if selected {
                pending.deselect_ids.remove(observation_id);
                pending.in_flight_deselect_ids.remove(observation_id);
                pending.select_ids.insert(*observation_id);
            } else {
                pending.select_ids.remove(observation_id);
                pending.in_flight_select_ids.remove(observation_id);
                pending.deselect_ids.insert(*observation_id);
            }
        }
        pending.changed_at = Some(Instant::now());
    }
}

fn drain_ready_observation_selection_confirm(
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
) -> Option<(Vec<u64>, Vec<u64>)> {
    let mut pending = pending_confirm.lock().ok()?;
    let changed_at = pending.changed_at?;
    if changed_at.elapsed() < Duration::from_millis(OBSERVATION_SELECTION_CONFIRM_DEBOUNCE_MS) {
        return None;
    }
    let select_ids = pending.select_ids.iter().copied().collect::<Vec<_>>();
    let deselect_ids = pending.deselect_ids.iter().copied().collect::<Vec<_>>();
    pending.select_ids.clear();
    pending.deselect_ids.clear();
    pending.changed_at = None;
    pending
        .in_flight_select_ids
        .extend(select_ids.iter().copied());
    pending
        .in_flight_deselect_ids
        .extend(deselect_ids.iter().copied());
    if select_ids.is_empty() && deselect_ids.is_empty() {
        None
    } else {
        Some((select_ids, deselect_ids))
    }
}

fn drain_observation_selection_confirm_now(
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
) -> Option<(Vec<u64>, Vec<u64>)> {
    let mut pending = pending_confirm.lock().ok()?;
    let select_ids = pending.select_ids.iter().copied().collect::<Vec<_>>();
    let deselect_ids = pending.deselect_ids.iter().copied().collect::<Vec<_>>();
    pending.select_ids.clear();
    pending.deselect_ids.clear();
    pending.changed_at = None;
    pending
        .in_flight_select_ids
        .extend(select_ids.iter().copied());
    pending
        .in_flight_deselect_ids
        .extend(deselect_ids.iter().copied());
    if select_ids.is_empty() && deselect_ids.is_empty() {
        None
    } else {
        Some((select_ids, deselect_ids))
    }
}

fn flush_observation_selection_confirm(
    watcher: &mut AppWatcherApi,
    pending_confirm: &Arc<Mutex<PendingObservationSelectionConfirm>>,
) {
    let Some((select_ids, deselect_ids)) = drain_observation_selection_confirm_now(pending_confirm)
    else {
        return;
    };
    if !select_ids.is_empty() {
        watcher.confirm_observations(ConfirmObservationsRequest {
            observation_ids: select_ids,
            confirmed: Some(true),
        });
    }
    if !deselect_ids.is_empty() {
        watcher.confirm_observations(ConfirmObservationsRequest {
            observation_ids: deselect_ids,
            confirmed: Some(false),
        });
    }
}

fn observation_selection_batches(
    visible_observations: &[ObservationDto],
    observation_id: u64,
    anchor_id: Option<u64>,
    selected_observation_ids: &BTreeSet<u64>,
    ctrl_pressed: bool,
    shift_pressed: bool,
) -> (Vec<u64>, Vec<u64>) {
    let current_index = match visible_observations
        .iter()
        .position(|observation| observation.id == observation_id)
    {
        Some(index) => index,
        None => return (Vec::new(), Vec::new()),
    };
    let anchor_index = anchor_id.and_then(|anchor_id| {
        visible_observations
            .iter()
            .position(|observation| observation.id == anchor_id)
    });
    let range = if shift_pressed {
        anchor_index
            .map(|anchor_index| {
                let start = anchor_index.min(current_index);
                let end = anchor_index.max(current_index);
                start..=end
            })
            .unwrap_or(current_index..=current_index)
    } else if ctrl_pressed {
        current_index..=current_index
    } else {
        return (Vec::new(), Vec::new());
    };

    let mut select_ids = Vec::new();
    let mut deselect_ids = Vec::new();
    for observation in visible_observations[range].iter() {
        if selected_observation_ids.contains(&observation.id) {
            deselect_ids.push(observation.id);
        } else {
            select_ids.push(observation.id);
        }
    }
    (select_ids, deselect_ids)
}

fn cloud_download_selection_batches(
    visible_rows: &[CloudDownloadedObservation],
    row_id: &str,
    anchor_id: Option<&str>,
    selected_row_ids: &BTreeSet<String>,
    ctrl_pressed: bool,
    shift_pressed: bool,
) -> (Vec<String>, Vec<String>) {
    let current_index = match visible_rows.iter().position(|row| row.row_id == row_id) {
        Some(index) => index,
        None => return (Vec::new(), Vec::new()),
    };
    let anchor_index =
        anchor_id.and_then(|anchor_id| visible_rows.iter().position(|row| row.row_id == anchor_id));
    let range = if shift_pressed {
        anchor_index
            .map(|anchor_index| {
                let start = anchor_index.min(current_index);
                let end = anchor_index.max(current_index);
                start..=end
            })
            .unwrap_or(current_index..=current_index)
    } else if ctrl_pressed {
        current_index..=current_index
    } else {
        return (Vec::new(), Vec::new());
    };

    let mut select_ids = Vec::new();
    let mut deselect_ids = Vec::new();
    for row in visible_rows[range].iter() {
        if selected_row_ids.contains(&row.row_id) {
            deselect_ids.push(row.row_id.clone());
        } else {
            select_ids.push(row.row_id.clone());
        }
    }
    (select_ids, deselect_ids)
}

fn add_pending_or_pick_executable(
    mut watcher: Signal<AppWatcherApi>,
    dialog_open: Signal<bool>,
    window: DesktopContext,
    draft_exe_path: Option<String>,
) {
    let snapshot = watcher.read().snapshot();
    let pending_exe_path = draft_exe_path
        .as_deref()
        .unwrap_or(snapshot.pending_exe_path.as_str())
        .trim()
        .to_string();
    if pending_exe_path.is_empty() {
        open_executable_file_dialog(watcher, dialog_open, window);
        return;
    }

    if snapshot
        .tracked_apps
        .iter()
        .any(|app| paths_match_for_duplicate_check(&app.exe_path, &pending_exe_path))
    {
        return;
    }

    watcher.write().add_tracked_app(AddTrackedAppRequest {
        exe_path: pending_exe_path,
    });
}

fn open_executable_file_dialog(
    mut watcher: Signal<AppWatcherApi>,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        let selected = pick_executable_path(&window).await;
        if let Some(path) = selected {
            let mut current = watcher.write();
            current.add_tracked_app(AddTrackedAppRequest {
                exe_path: path.display().to_string(),
            });
        }
        dialog_open.set(false);
    });
}

fn cloud_quota_value_class(quota: Option<&CloudQuotaSnapshot>) -> &'static str {
    if quota_is_exhausted(quota) {
        "cloud-sync-footer-quota__value cloud-sync-footer-quota__value--exhausted"
    } else {
        "cloud-sync-footer-quota__value"
    }
}

fn cloud_quota_tooltip(
    quota: Option<&CloudQuotaSnapshot>,
    expires_template: &str,
    fresh_template: &str,
    hours_suffix: &str,
) -> String {
    let Some(quota) = quota else {
        return String::new();
    };
    let template = if quota.used_count > 0 {
        expires_template
    } else {
        fresh_template
    };
    template
        .replace("{limit}", &quota.limit_count.to_string())
        .replace("{window}", &quota_window_label(Some(quota), hours_suffix))
        .replace(
            "{expires}",
            &format_cloud_quota_expires_at(quota.window_ends_at_ms),
        )
}

fn format_cloud_quota_expires_at(timestamp_ms: u64) -> String {
    Local
        .timestamp_millis_opt(timestamp_ms as i64)
        .single()
        .map(|date_time| date_time.format("%d.%m.%Y %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn cloud_visibility_scope_value(scope: CloudObservationVisibilityScope) -> &'static str {
    match scope {
        CloudObservationVisibilityScope::All => "All",
        CloudObservationVisibilityScope::Public => "Public",
        CloudObservationVisibilityScope::Private => "Private",
    }
}

fn current_cloud_filters(
    app_query: &Signal<String>,
    publisher_query: &Signal<String>,
    remote_ip_query: &Signal<String>,
    remote_domain_query: &Signal<String>,
    remote_port_query: &Signal<String>,
    protocol: &Signal<String>,
    source_query: &Signal<String>,
    selected_app_id: &Signal<Option<String>>,
    own_scope: &Signal<bool>,
    visibility_scope: &Signal<CloudObservationVisibilityScope>,
) -> CloudSearchFilters {
    CloudSearchFilters {
        app_query: app_query.read().clone(),
        publisher_query: publisher_query.read().clone(),
        selected_app_id: selected_app_id.read().clone(),
        remote_ip_query: remote_ip_query.read().clone(),
        remote_domain_query: remote_domain_query.read().clone(),
        remote_port_query: remote_port_query.read().clone(),
        protocol: protocol.read().clone(),
        source_query: source_query.read().clone(),
        own_scope: *own_scope.read(),
        visibility_scope: *visibility_scope.read(),
    }
}

fn selected_cloud_import_rows(state: &CloudSyncUiState) -> Vec<MonitoringCsvImportRowDto> {
    state
        .downloaded_rows
        .iter()
        .filter(|item| state.selected_download_row_ids.contains(&item.row_id))
        .map(|item| cloud_row_to_import_row(state, item))
        .collect()
}

fn cloud_import_rows_for_add_to_monitoring(
    state: &CloudSyncUiState,
) -> Vec<MonitoringCsvImportRowDto> {
    selected_cloud_import_rows(state)
}

fn selected_cloud_csv_export_rows(state: &CloudSyncUiState) -> Vec<CsvExportRow> {
    state
        .downloaded_rows
        .iter()
        .filter(|item| state.selected_download_row_ids.contains(&item.row_id))
        .map(|item| {
            let row = &item.row;
            CsvExportRow {
                application: cloud_app_name_for_row(state, item),
                app_connector_id: netstitch_shared::cloud_connector_id_for_app_id(&row.app_id)
                    .unwrap_or_default()
                    .to_string(),
                cloud_app_id: row.app_id.clone(),
                app_signature_key: row.app_signature_key.clone().unwrap_or_default(),
                app_signature_subject: row.app_signature_subject.clone().unwrap_or_default(),
                app_signature_issuer: row.app_signature_issuer.clone().unwrap_or_default(),
                ip: row.remote_ip.to_string(),
                domain: row.domain_raw.clone().unwrap_or_default(),
                port: row.remote_port.to_string(),
                protocol: row.protocol.as_str().to_string(),
                connection: format!(
                    "{} ({}/{}/{})",
                    row.connection_state.as_str(),
                    row.successful_hits,
                    row.failed_hits,
                    row.requests
                ),
                requests: row.requests.to_string(),
                first_seen: format_cloud_row_timestamp(row.first_seen_ms),
                last_seen: format_cloud_row_timestamp(row.last_seen_ms),
            }
        })
        .collect()
}

fn cloud_row_to_import_row(
    state: &CloudSyncUiState,
    item: &CloudDownloadedObservation,
) -> MonitoringCsvImportRowDto {
    let row = &item.row;
    MonitoringCsvImportRowDto {
        application: cloud_app_name_for_row(state, item),
        app_connector_id: netstitch_shared::cloud_connector_id_for_app_id(&row.app_id)
            .map(ToOwned::to_owned),
        cloud_app_id: Some(row.app_id.clone()),
        app_signature_key: row.app_signature_key.clone(),
        app_signature_subject: row.app_signature_subject.clone(),
        app_signature_issuer: row.app_signature_issuer.clone(),
        remote_ip: row.remote_ip,
        domain: row.domain_raw.clone(),
        remote_port: row.remote_port,
        protocol: row.protocol,
        connection_state: row.connection_state,
        first_seen_ms: row.first_seen_ms,
        last_seen_ms: row.last_seen_ms,
        hits: row.requests,
        failed_hits: row.failed_hits,
        successful_hits: row.successful_hits,
    }
}

fn cloud_app_name_for_row(_state: &CloudSyncUiState, item: &CloudDownloadedObservation) -> String {
    if item.app_display_name.trim().is_empty() {
        item.row.app_id.clone()
    } else {
        item.app_display_name.clone()
    }
}

fn format_cloud_row_timestamp(timestamp_ms: u64) -> String {
    Local
        .timestamp_millis_opt(timestamp_ms as i64)
        .single()
        .map(|date_time| date_time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn start_cloud_refresh(
    mut cloud_state: Signal<CloudSyncUiState>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    filters: CloudSearchFilters,
    success_message: String,
    message_labels: CloudMessageLabels,
    publish_status: bool,
) {
    let mut pending_state = cloud_state();
    pending_state.last_error = None;
    pending_state.refresh_generation = pending_state.refresh_generation.wrapping_add(1);
    let pending_refresh_generation = pending_state.refresh_generation;
    cloud_state.set(pending_state.clone());

    spawn(async move {
        let refresh_task = tokio::task::spawn_blocking(move || {
            let mut state = pending_state;
            let error = match refresh_cloud_state(&mut state, &filters) {
                Ok(()) => None,
                Err(message) => {
                    state.service_available = Some(false);
                    state.last_error = Some(message.clone());
                    Some(message)
                }
            };
            CloudRefreshRunResult { state, error }
        });
        let refresh_result =
            tokio::time::timeout(std::time::Duration::from_millis(9500), refresh_task)
                .await
                .map_err(|_| "cloud refresh timed out".to_string())
                .and_then(|result| result.map_err(|error| format!("{error}")));

        match refresh_result {
            Ok(result) => {
                let current = cloud_state();
                if result.state.refresh_generation != current.refresh_generation {
                    return;
                }
                let message = if publish_status {
                    result.error.clone().unwrap_or(success_message)
                } else {
                    String::new()
                };
                let is_error = result.error.is_some();
                cloud_state.set(merge_cloud_state_from_async_result(result.state, &current));
                if publish_status {
                    push_cloud_status_history_line(
                        status_history,
                        message,
                        is_error,
                        &message_labels,
                    );
                }
            }
            Err(error) => {
                let message = format!("cloud refresh task failed: {error}");
                let mut state = cloud_state();
                if pending_refresh_generation != state.refresh_generation {
                    return;
                }
                state.service_available = Some(false);
                state.last_error = Some(message.clone());
                cloud_state.set(state);
                if publish_status {
                    push_status_history_error_line(status_history, message);
                }
            }
        }
    });
}

fn start_cloud_nickname_check(
    mut status: Signal<CloudNicknameCheckStatus>,
    generation_signal: Signal<u64>,
    state: CloudSyncUiState,
    nickname: String,
    generation: u64,
) {
    spawn(async move {
        let check_result = tokio::task::spawn_blocking(move || {
            check_author_signature_availability(&state, &nickname)
        })
        .await
        .map_err(|error| format!("{error}"))
        .and_then(|result| result);

        if generation_signal() != generation {
            return;
        }

        let next_status = match check_result {
            Ok(status) => status,
            Err(_) => CloudNicknameCheckStatus::Error,
        };
        status.set(next_status);
    });
}

fn start_cloud_download(
    mut cloud_state: Signal<CloudSyncUiState>,
    mut cloud_loading_app_id: Signal<Option<String>>,
    mut footer_progress: Signal<Option<FooterProgressState>>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    filters: CloudSearchFilters,
    started_message: String,
    message_labels: CloudMessageLabels,
) {
    let mut pending_state = cloud_state();
    pending_state.last_error = None;
    reset_cloud_download_staging_for_download(&mut pending_state);
    cloud_state.set(pending_state.clone());
    let pending_app_id = filters.selected_app_id.clone();
    cloud_loading_app_id.set(pending_app_id.clone());
    footer_progress.set(Some(footer_progress_state(
        message_labels.download.clone(),
        0,
        None,
    )));
    push_status_history_warning_line(status_history, started_message);

    spawn(async move {
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
        let download_task = tokio::task::spawn_blocking(move || {
            let mut state = pending_state;
            let error = match download_observations(&mut state, &filters, move |loaded, total| {
                let _ = progress_tx.send((loaded, total));
            }) {
                Ok(_) => None,
                Err(message) => {
                    state.last_error = Some(message.clone());
                    Some(message)
                }
            };
            CloudDownloadRunResult { state, error }
        });
        tokio::pin!(download_task);

        let download_result = loop {
            tokio::select! {
                progress = progress_rx.recv() => {
                    if let Some((loaded, total)) = progress {
                        footer_progress.set(Some(footer_progress_state(
                            message_labels.download.clone(),
                            loaded,
                            total,
                        )));
                    }
                }
                result = &mut download_task => break result,
            }
        };

        match download_result {
            Ok(result) => {
                let message = result
                    .error
                    .clone()
                    .or_else(|| result.state.last_response_json.clone())
                    .unwrap_or_else(|| {
                        format!("{}: 0 {}", message_labels.download, message_labels.rows)
                    });
                let is_error = result.error.is_some();
                let current = cloud_state();
                cloud_state.set(merge_cloud_state_from_async_result(result.state, &current));
                if cloud_loading_app_id() == pending_app_id {
                    cloud_loading_app_id.set(None);
                }
                footer_progress.set(None);
                push_cloud_status_history_line(status_history, message, is_error, &message_labels);
            }
            Err(error) => {
                let message = format!("cloud download task failed: {error}");
                let mut state = cloud_state();
                state.last_error = Some(message.clone());
                cloud_state.set(state);
                if cloud_loading_app_id() == pending_app_id {
                    cloud_loading_app_id.set(None);
                }
                footer_progress.set(None);
                push_status_history_error_line(status_history, message);
            }
        }
    });
}

fn reset_cloud_download_staging_for_download(state: &mut CloudSyncUiState) {
    state.downloaded_rows.clear();
    state.selected_download_row_ids.clear();
    state.last_response_json = None;
}

fn mark_uploaded_public_observations(state: &mut CloudSyncUiState, snapshot: &SnapshotResponse) {
    let mut local_upload_apps = BTreeMap::new();
    for observation in snapshot
        .observations
        .iter()
        .filter(|observation| observation.is_confirmed)
    {
        let is_public_ip = observation
            .remote_ip
            .parse::<IpAddr>()
            .map(netstitch_shared::cloud_observation_ip_is_public)
            .unwrap_or(false);
        if is_public_ip
            && cloud_upload_app_preview_for_observation(
                snapshot,
                observation,
                &mut local_upload_apps,
            )
            .is_some()
        {
            state.uploaded_observation_ids.insert(observation.id);
        }
    }
}

fn footer_progress_state(
    label: String,
    loaded: usize,
    total: Option<usize>,
) -> FooterProgressState {
    let percent = total
        .filter(|value| *value > 0)
        .map(|total| ((loaded.saturating_mul(100)) / total).min(100) as u8)
        .unwrap_or(if loaded > 0 { 100 } else { 1 });
    let meta = total
        .filter(|value| *value > 0)
        .map(|total| format!("{loaded}/{total}"))
        .unwrap_or_else(|| loaded.to_string());
    FooterProgressState {
        label,
        percent,
        meta,
        stages: cloud_progress_stages(total.unwrap_or(loaded)),
    }
}

fn cloud_progress_stages(total_rows: usize) -> Vec<ProgressStage> {
    const CLOUD_PROGRESS_CHUNK_ROWS: usize = 500;
    let chunk_count = total_rows.max(1).div_ceil(CLOUD_PROGRESS_CHUNK_ROWS).max(1);
    (0..chunk_count)
        .map(|_| ProgressStage::new(String::new(), 1, "progress-bar__segment--accent"))
        .collect()
}

fn start_cloud_upload(
    mut cloud_state: Signal<CloudSyncUiState>,
    mut cloud_upload_progress: Signal<Option<FooterProgressState>>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    mut snapshot: SnapshotResponse,
    selected_observation_ids: BTreeSet<u64>,
    author_signature: String,
    visibility: CloudObservationVisibility,
    started_message: String,
    message_labels: CloudMessageLabels,
) {
    snapshot.observations.retain(|observation| {
        selected_observation_ids.contains(&observation.id) && observation.is_confirmed
    });
    let mut pending_state = cloud_state();
    pending_state.last_error = None;
    cloud_state.set(pending_state.clone());
    let upload_row_count = snapshot.observations.len();
    cloud_upload_progress.set(Some(FooterProgressState {
        label: message_labels.upload.clone(),
        percent: 20,
        meta: upload_row_count.to_string(),
        stages: cloud_progress_stages(upload_row_count),
    }));
    push_status_history_warning_line(status_history, started_message);

    spawn(async move {
        let upload_result = tokio::task::spawn_blocking(move || {
            let mut state = pending_state;
            let error = match upload_confirmed_observations(
                &mut state,
                &snapshot,
                &runtime_build_version(),
                &author_signature,
                visibility,
            ) {
                Ok(_) => {
                    mark_uploaded_public_observations(&mut state, &snapshot);
                    None
                }
                Err(message) => {
                    state.last_error = Some(message.clone());
                    Some(message)
                }
            };
            CloudUploadRunResult { state, error }
        })
        .await;

        match upload_result {
            Ok(result) => {
                let message = result
                    .error
                    .clone()
                    .or_else(|| result.state.last_response_json.clone())
                    .unwrap_or_else(|| {
                        format!("{}: 0 {}", message_labels.upload, message_labels.rows)
                    });
                let is_error = result.error.is_some();
                let current = cloud_state();
                cloud_state.set(merge_cloud_state_from_async_result(result.state, &current));
                cloud_upload_progress.set(None);
                push_cloud_status_history_line(status_history, message, is_error, &message_labels);
            }
            Err(error) => {
                let message = format!("cloud upload task failed: {error}");
                let mut state = cloud_state();
                state.last_error = Some(message.clone());
                cloud_state.set(state);
                cloud_upload_progress.set(None);
                push_status_history_error_line(status_history, message);
            }
        }
    });
}

fn start_cloud_google_auth(
    mut cloud_state: Signal<CloudSyncUiState>,
    cloud_upload_nickname: Signal<String>,
    mut cloud_nickname_check_status: Signal<CloudNicknameCheckStatus>,
    mut cloud_nickname_check_generation: Signal<u64>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    filters: CloudSearchFilters,
    ui_language: String,
    started_message: String,
    success_message: String,
    credentials_save_failed_message: String,
    message_labels: CloudMessageLabels,
) {
    let mut pending_state = cloud_state();
    pending_state.last_error = None;
    pending_state.auth_generation = pending_state.auth_generation.wrapping_add(1);
    let auth_flow_generation = pending_state.auth_generation;
    cloud_state.set(pending_state.clone());
    push_status_history_warning_line(status_history, started_message);

    spawn(async move {
        let auth_result = tokio::task::spawn_blocking(move || {
            let mut state = pending_state;
            let auth_result = login_cloud_user_with_google(&mut state, &ui_language);
            let mut refresh_error = None;
            let error = match auth_result {
                Ok(()) => {
                    if let Err(message) = refresh_cloud_state(&mut state, &filters) {
                        refresh_error = Some(message);
                    }
                    None
                }
                Err(message) => {
                    state.last_error = Some(message.clone());
                    Some(message)
                }
            };
            CloudAuthRunResult {
                state,
                error,
                refresh_error,
            }
        })
        .await;

        match auth_result {
            Ok(result) => {
                let current = cloud_state();
                if let Some(message) = result.error {
                    if current.session.is_some() || auth_flow_generation != current.auth_generation
                    {
                        return;
                    }
                    cloud_state.set(merge_cloud_state_from_async_result(result.state, &current));
                    push_cloud_status_history_line(status_history, message, true, &message_labels);
                } else {
                    if current.session.is_some() {
                        return;
                    }
                    let mut next_state = result.state;
                    next_state.auth_generation = current.auth_generation.wrapping_add(1);
                    let credentials_saved =
                        persist_cloud_google_session_metadata_to_sqlite(&next_state).is_ok();
                    cloud_state.set(next_state.clone());
                    let nickname = cloud_upload_nickname().trim().to_string();
                    if nickname.is_empty() {
                        cloud_nickname_check_status.set(CloudNicknameCheckStatus::Idle);
                    } else {
                        let next_nickname_generation =
                            cloud_nickname_check_generation().wrapping_add(1);
                        cloud_nickname_check_generation.set(next_nickname_generation);
                        if validate_author_signature(&nickname).is_ok() {
                            cloud_nickname_check_status.set(CloudNicknameCheckStatus::Checking);
                            start_cloud_nickname_check(
                                cloud_nickname_check_status,
                                cloud_nickname_check_generation,
                                next_state,
                                nickname,
                                next_nickname_generation,
                            );
                        } else {
                            cloud_nickname_check_status.set(CloudNicknameCheckStatus::Invalid);
                        }
                    }
                    if !credentials_saved {
                        push_status_history_error_line(
                            status_history,
                            credentials_save_failed_message,
                        );
                    }
                    if let Some(message) = result.refresh_error {
                        push_status_history_warning_line(status_history, message);
                    }
                    push_status_history_success_line(status_history, success_message);
                }
            }
            Err(error) => {
                let message = format!("cloud google auth task failed: {error}");
                let mut state = cloud_state();
                state.last_error = Some(message.clone());
                cloud_state.set(state);
                push_status_history_error_line(status_history, message);
            }
        }
    });
}

fn merge_cloud_auth_from_current_if_stale(
    mut next: CloudSyncUiState,
    current: &CloudSyncUiState,
) -> CloudSyncUiState {
    if next.auth_generation != current.auth_generation {
        next.session = current.session.clone();
        next.client_private_key_pkcs8_der = current.client_private_key_pkcs8_der.clone();
        next.my_apps = current.my_apps.clone();
        next.auth_generation = current.auth_generation;
    }
    next
}

fn merge_cloud_state_from_async_result(
    next: CloudSyncUiState,
    current: &CloudSyncUiState,
) -> CloudSyncUiState {
    let mut next = merge_cloud_auth_from_current_if_stale(next, current);
    let next_row_ids = next
        .downloaded_rows
        .iter()
        .map(|row| &row.row_id)
        .collect::<Vec<_>>();
    let current_row_ids = current
        .downloaded_rows
        .iter()
        .map(|row| &row.row_id)
        .collect::<Vec<_>>();
    if next_row_ids == current_row_ids {
        next.selected_download_row_ids = current.selected_download_row_ids.clone();
    }
    next.uploaded_observation_ids
        .extend(current.uploaded_observation_ids.iter().copied());
    next
}

fn persist_cloud_google_session_metadata_to_sqlite(state: &CloudSyncUiState) -> Result<(), String> {
    let session = state
        .session
        .as_ref()
        .ok_or_else(|| "cloud session is missing after Google authentication".to_string())?;
    let private_key = state
        .client_private_key_pkcs8_der
        .as_deref()
        .ok_or_else(|| {
            "cloud client private key is missing after Google authentication".to_string()
        })?;
    persist_cloud_session_to_sqlite(session, private_key)
}

fn push_cloud_status_history_line(
    status_history: Signal<Vec<StatusHistoryLine>>,
    message: String,
    is_error_response: bool,
    labels: &CloudMessageLabels,
) {
    let display_message = humanize_cloud_status_message(&message, labels);
    match cloud_status_history_kind(&message, is_error_response) {
        StatusHistoryKind::Success => {
            push_status_history_success_line(status_history, display_message)
        }
        StatusHistoryKind::Warning => {
            push_status_history_warning_line(status_history, display_message)
        }
        StatusHistoryKind::Error => push_status_history_error_line(status_history, display_message),
        StatusHistoryKind::Info => push_status_history_line(status_history, display_message),
    }
}

fn humanize_cloud_status_message(message: &str, labels: &CloudMessageLabels) -> String {
    let trimmed = message.trim();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return normalize_cloud_plain_message(trimmed);
    };

    if let Some(error) = value.get("error") {
        let code = error
            .get("code")
            .and_then(|item| item.as_str())
            .unwrap_or_default()
            .trim();
        let text = error
            .get("message")
            .and_then(|item| item.as_str())
            .unwrap_or(code)
            .trim();
        if text.is_empty() && code.is_empty() {
            return labels.refresh_done.clone();
        }
        if code.is_empty() || text == code {
            return text.to_string();
        }
        return format!("{text} ({code})");
    }

    if let Some(rows) = value
        .get("download")
        .and_then(|download| download.get("rows"))
        .and_then(|rows| rows.as_u64())
    {
        return format!("{}: {rows} {}", labels.download, labels.rows);
    }

    if let Some(upload) = value.get("upload") {
        if let Some(rows) = upload.get("accepted_rows").and_then(|rows| rows.as_u64()) {
            let requests = upload
                .get("request_count")
                .and_then(|count| count.as_u64())
                .unwrap_or(0);
            let visibility = upload
                .get("visibility")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty());
            if requests > 0 {
                let mut message = format!(
                    "{}: {rows} {}, {}: {requests}",
                    labels.upload, labels.rows, labels.requests
                );
                if let Some(skipped) = upload
                    .get("skipped_non_public_rows")
                    .and_then(|count| count.as_u64())
                    .filter(|count| *count > 0)
                {
                    message = format!("{message}, {}: {skipped}", labels.skipped_non_public);
                }
                return visibility
                    .map(|visibility| format!("{message}, visibility: {visibility}"))
                    .unwrap_or(message);
            }
            let message = format!("{}: {rows} {}", labels.upload, labels.rows);
            return visibility
                .map(|visibility| format!("{message}, visibility: {visibility}"))
                .unwrap_or(message);
        }
    }

    if value.get("health").is_some() || value.get("quota").is_some() || value.get("apps").is_some()
    {
        return labels.refresh_done.clone();
    }

    normalize_cloud_plain_message(trimmed)
}

fn normalize_cloud_plain_message(message: &str) -> String {
    message.trim().trim_end_matches('.').to_string()
}

fn cloud_status_history_kind(message: &str, is_error_response: bool) -> StatusHistoryKind {
    match cloud_response_code(message).as_deref() {
        Some("nickname_taken" | "nickname_blocked") => StatusHistoryKind::Warning,
        _ if is_error_response => StatusHistoryKind::Error,
        _ => StatusHistoryKind::Success,
    }
}

fn cloud_response_code(message: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(message)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(|code| code.as_str())
                .map(ToOwned::to_owned)
        })
}

fn paths_match_for_duplicate_check(left: &str, right: &str) -> bool {
    normalize_path_for_duplicate_check(left) == normalize_path_for_duplicate_check(right)
}

fn normalize_path_for_duplicate_check(value: &str) -> String {
    value.trim().replace('/', "\\").to_ascii_lowercase()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IntegrationDownloadUiState {
    visible: bool,
    active: bool,
    cancelled: bool,
    generation: u64,
    stage: String,
    percent: Option<u8>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
    extracted_entries: Option<u64>,
    total_entries: Option<u64>,
    message: Option<String>,
    repo_root: Option<String>,
}

impl Default for IntegrationDownloadUiState {
    fn default() -> Self {
        Self {
            active: false,
            visible: false,
            cancelled: false,
            generation: 0,
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
}

impl IntegrationDownloadUiState {
    fn from_dto(
        dto: IntegrationDownloadProgressDto,
        generation: u64,
        cancelled: bool,
    ) -> IntegrationDownloadUiState {
        IntegrationDownloadUiState {
            active: dto.active,
            visible: dto.active || dto.stage != "idle",
            cancelled,
            generation,
            stage: dto.stage,
            percent: dto.percent,
            downloaded_bytes: dto.downloaded_bytes,
            total_bytes: dto.total_bytes,
            extracted_entries: dto.extracted_entries,
            total_entries: dto.total_entries,
            message: dto.message,
            repo_root: dto.repo_root.map(|path| path.display().to_string()),
        }
    }

    fn progress_meta(&self, labels: &IntegrationProgressLabels) -> String {
        let stage = labels.stage_label(&self.stage);
        let Some(percent) = self.percent.map(|value| value.min(100)) else {
            if let Some(downloaded) = self.downloaded_bytes {
                return format!("{stage} | {} {}", downloaded / 1024, labels.kb);
            }
            return stage;
        };

        if self.stage == "extracting" {
            if let (Some(done), Some(total)) = (self.extracted_entries, self.total_entries) {
                return format!("{percent}% | {done}/{total} {}", labels.entries);
            }
        }

        if let Some(downloaded) = self.downloaded_bytes {
            let downloaded_kb = downloaded / 1024;
            if let Some(total) = self.total_bytes {
                return format!(
                    "{percent}% | {downloaded_kb}/{} {}",
                    total / 1024,
                    labels.kb
                );
            }
            return format!("{percent}% | {downloaded_kb} {}", labels.kb);
        }

        format!("{percent}% | {stage}")
    }

    fn visual_percent(&self) -> u8 {
        if let Some(percent) = self.percent.map(|value| value.min(100)) {
            return if self.stage == "extracting" {
                75 + ((percent as u16 * 25) / 100) as u8
            } else if self.stage == "downloading" {
                ((percent as u16 * 75) / 100) as u8
            } else {
                percent
            };
        }

        match self.stage.as_str() {
            "preparing" | "downloading" => 0,
            "extracting" => 75,
            "detecting" | "installing" | "complete" => 100,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IntegrationProgressLabels {
    kb: String,
    entries: String,
    preparing: String,
    downloading: String,
    extracting: String,
    detecting: String,
    installing: String,
    complete: String,
    failed: String,
    idle: String,
}

impl IntegrationProgressLabels {
    fn stage_label(&self, stage: &str) -> String {
        match stage {
            "preparing" => self.preparing.clone(),
            "downloading" => self.downloading.clone(),
            "extracting" => self.extracting.clone(),
            "detecting" => self.detecting.clone(),
            "installing" => self.installing.clone(),
            "complete" => self.complete.clone(),
            "failed" => self.failed.clone(),
            "idle" => self.idle.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IntegrationDialogPreview {
    status_text: String,
    status_class: &'static str,
    repo_path: String,
    export_path: String,
    provider_name: String,
}

fn integration_dialog_preview(
    input_path: &str,
    current: &crate::watcher_api::IntegrationIntegrationDto,
    configured_label: &str,
    missing_label: &str,
    invalid_label: &str,
    _entered_label: &str,
    provider_unset_label: &str,
) -> IntegrationDialogPreview {
    let trimmed = input_path.trim();
    if trimmed.is_empty() {
        return IntegrationDialogPreview {
            status_text: if current.ready {
                configured_label.to_string()
            } else {
                missing_label.to_string()
            },
            status_class: if current.ready {
                "integration-dialog-status-text integration-dialog-status-text--success"
            } else {
                "integration-dialog-status-text integration-dialog-status-text--warning"
            },
            repo_path: current.repo_path.clone(),
            export_path: current.export_path.clone(),
            provider_name: current
                .provider_name
                .clone()
                .unwrap_or_else(|| provider_unset_label.to_string()),
        };
    }

    let same_path =
        current.ready && paths_match_for_integration_preview(trimmed, &current.repo_path);

    IntegrationDialogPreview {
        status_text: if same_path {
            configured_label.to_string()
        } else {
            invalid_label.to_string()
        },
        status_class: if same_path {
            "integration-dialog-status-text integration-dialog-status-text--success"
        } else {
            "integration-dialog-status-text integration-dialog-status-text--warning"
        },
        repo_path: if same_path {
            current.repo_path.clone()
        } else {
            trimmed.to_string()
        },
        export_path: if same_path {
            current.export_path.clone()
        } else {
            String::new()
        },
        provider_name: if same_path {
            current
                .provider_name
                .clone()
                .unwrap_or_else(|| provider_unset_label.to_string())
        } else {
            provider_unset_label.to_string()
        },
    }
}

fn paths_match_for_integration_preview(left: &str, right: &str) -> bool {
    fn normalize(value: &str) -> String {
        value
            .trim()
            .trim_end_matches(['\\', '/'])
            .replace('/', "\\")
            .to_ascii_lowercase()
    }

    let left = normalize(left);
    let right = normalize(right);
    !left.is_empty() && left == right
}

fn localize_integration_folder_error(
    message: &str,
    missing_label: &str,
    invalid_label: &str,
) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("does not exist") {
        return missing_label.to_string();
    }
    if lower.contains("unable to resolve")
        || lower.contains("not a directory")
        || lower.contains("ambiguous")
        || lower.contains("integration unavailable")
    {
        return invalid_label.to_string();
    }
    message.to_string()
}

fn integration_provider_id(current: &crate::watcher_api::IntegrationIntegrationDto) -> String {
    if let Some(provider_id) = current.provider_id.as_deref() {
        let provider_id = provider_id.trim();
        if !provider_id.is_empty() {
            return provider_id.to_string();
        }
    }

    "default".to_string()
}

fn ordered_integration_modules(
    mut modules: Vec<IntegrationModuleDto>,
    order: &[String],
) -> Vec<IntegrationModuleDto> {
    modules.sort_by(|left, right| {
        let left_index = order
            .iter()
            .position(|id| id == &left.id)
            .unwrap_or(usize::MAX);
        let right_index = order
            .iter()
            .position(|id| id == &right.id)
            .unwrap_or(usize::MAX);
        left_index
            .cmp(&right_index)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    modules
}

fn move_module_in_order(
    order: &[String],
    modules: &[IntegrationModuleDto],
    module_id: &str,
    offset: isize,
) -> Vec<String> {
    let mut normalized = order
        .iter()
        .filter(|id| modules.iter().any(|module| &module.id == *id))
        .cloned()
        .collect::<Vec<_>>();
    for module in modules {
        if !normalized.iter().any(|id| id == &module.id) {
            normalized.push(module.id.clone());
        }
    }
    let Some(index) = normalized.iter().position(|id| id == module_id) else {
        return normalized;
    };
    let target = if offset.is_negative() {
        index.saturating_sub(offset.unsigned_abs())
    } else {
        (index + offset as usize).min(normalized.len().saturating_sub(1))
    };
    normalized.swap(index, target);
    normalized
}

fn module_button_style(module: &IntegrationModuleDto) -> String {
    module
        .button_color
        .as_deref()
        .filter(|color| safe_hex_color(color))
        .map(|color| format!("background: {color}; border-color: {color};"))
        .unwrap_or_default()
}

fn module_icon_src(module: &IntegrationModuleDto) -> Option<String> {
    module
        .icon_data_uri
        .clone()
        .or_else(|| module.icon_svg.as_deref().map(inline_svg_data_uri))
        .or_else(|| {
            module
                .icon_path
                .as_ref()
                .map(|path| icon_image_src(&path.to_string_lossy()))
        })
}

fn module_action_icon_src(action: &IntegrationModuleActionDto) -> Option<String> {
    action
        .icon_data_uri
        .clone()
        .or_else(|| action.icon_svg.as_deref().map(inline_svg_data_uri))
        .or_else(|| {
            action
                .icon_path
                .as_ref()
                .map(|path| icon_image_src(&path.to_string_lossy()))
        })
}

fn module_open_action_id(module: &IntegrationModuleDto) -> Option<String> {
    module
        .header_actions
        .iter()
        .chain(module_ui_entity_actions(&module.ui_schema).into_iter())
        .find(|action| matches!(action.id.as_str(), "module.open" | "open") && action.enabled)
        .map(|action| action.id.clone())
}

fn module_ui_entity_actions<'a>(
    entities: &'a [IntegrationUiEntityDto],
) -> Vec<&'a IntegrationModuleActionDto> {
    let mut actions = Vec::new();
    for entity in entities {
        actions.extend(entity.actions.iter());
        actions.extend(module_ui_entity_actions(&entity.children));
    }
    actions
}

fn module_action_fallback_label(label: &str) -> String {
    label
        .trim()
        .chars()
        .next()
        .map(|ch| ch.to_uppercase().collect::<String>())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "*".to_string())
}

fn module_host_dialog_from_command(
    command: &IntegrationModuleHostCommandDto,
    module: &IntegrationModuleDto,
) -> Option<ModuleHostDialogState> {
    module_host_dialog_from_owner_parts(
        command,
        &module.id,
        &module.display_name,
        module_icon_src(module),
        &module_action_fallback_label(&module.icon_label),
    )
}

fn module_host_dialog_from_owner_dialog(
    command: &IntegrationModuleHostCommandDto,
    owner: &ModuleHostDialogState,
) -> Option<ModuleHostDialogState> {
    module_host_dialog_from_owner_parts(
        command,
        &owner.module_id,
        &owner.title,
        owner.icon_src.clone(),
        &owner.icon_fallback,
    )
}

fn module_host_dialog_from_owner_parts(
    command: &IntegrationModuleHostCommandDto,
    module_id: &str,
    title: &str,
    icon_src: Option<String>,
    icon_fallback: &str,
) -> Option<ModuleHostDialogState> {
    if command.command_type != "show_dialog" {
        return None;
    }
    let object = command.payload.as_object()?;
    let message = object
        .get("message")
        .or_else(|| object.get("text"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .chars()
        .take(2_000)
        .collect::<String>();
    let buttons = object
        .get("buttons")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("ok");
    let dialog_id = object
        .get("dialog_id")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{}-{}", module_id, Utc::now().timestamp_millis().max(0)));
    Some(ModuleHostDialogState {
        module_id: module_id.to_string(),
        dialog_id,
        title: title.to_string(),
        message,
        show_cancel: buttons.eq_ignore_ascii_case("ok_cancel"),
        icon_src,
        icon_fallback: icon_fallback.to_string(),
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleBrowseWindowMode {
    Folder,
    FileOpen,
    FileSave,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleBrowseWindowOverwritePolicy {
    Prompt,
    Allow,
    Deny,
}

#[derive(Clone)]
struct ModuleBrowseWindowFilter {
    name: String,
    extensions: Vec<String>,
}

#[derive(Clone)]
struct ModuleBrowseWindowOptions {
    target: String,
    status_target: String,
    mode: ModuleBrowseWindowMode,
    title: String,
    start_dir: String,
    default_name: String,
    default_extension: String,
    confirm_label: String,
    selected_status: String,
    overwrite_policy: ModuleBrowseWindowOverwritePolicy,
    can_create_directories: bool,
    filters: Vec<ModuleBrowseWindowFilter>,
}

fn module_browse_window_options_from_payload(
    payload: &serde_json::Value,
) -> Result<ModuleBrowseWindowOptions, String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "browse_window payload must be an object".to_string())?;
    let target = module_browse_window_string(object, "target", 120)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "browse_window target is required".to_string())?;
    let mode = match module_browse_window_string(object, "mode", 40)
        .unwrap_or_else(|| "file_open".to_string())
        .as_str()
    {
        "folder" => ModuleBrowseWindowMode::Folder,
        "file_open" => ModuleBrowseWindowMode::FileOpen,
        "file_save" => ModuleBrowseWindowMode::FileSave,
        _ => return Err("browse_window mode must be folder, file_open or file_save".to_string()),
    };
    let overwrite_policy = match module_browse_window_string(object, "overwrite_policy", 40)
        .unwrap_or_else(|| "prompt".to_string())
        .as_str()
    {
        "prompt" => ModuleBrowseWindowOverwritePolicy::Prompt,
        "allow" => ModuleBrowseWindowOverwritePolicy::Allow,
        "deny" => ModuleBrowseWindowOverwritePolicy::Deny,
        _ => {
            return Err("browse_window overwrite_policy must be prompt, allow or deny".to_string());
        }
    };
    let title = module_browse_window_string(object, "title", 200)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| module_browse_window_default_title(mode));
    Ok(ModuleBrowseWindowOptions {
        target,
        status_target: module_browse_window_string(object, "status_target", 120)
            .unwrap_or_default(),
        mode,
        title,
        start_dir: module_browse_window_string(object, "start_dir", 4096).unwrap_or_default(),
        default_name: module_browse_window_string(object, "default_name", 255).unwrap_or_default(),
        default_extension: module_browse_window_string(object, "default_extension", 32)
            .map(|value| module_browse_window_extension(&value))
            .transpose()?
            .unwrap_or_default(),
        confirm_label: module_browse_window_string(object, "confirm_label", 80).unwrap_or_default(),
        selected_status: module_browse_window_string(object, "selected_status", 240)
            .unwrap_or_default(),
        overwrite_policy,
        can_create_directories: object
            .get("can_create_directories")
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
        filters: module_browse_window_filters(object)?,
    })
}

fn module_browse_window_string(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    max_len: usize,
) -> Option<String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .map(|value| value.chars().take(max_len).collect::<String>())
}

fn module_browse_window_default_title(mode: ModuleBrowseWindowMode) -> String {
    match mode {
        ModuleBrowseWindowMode::Folder => "Choose folder".to_string(),
        ModuleBrowseWindowMode::FileOpen => "Choose file".to_string(),
        ModuleBrowseWindowMode::FileSave => "Save file".to_string(),
    }
}

fn module_browse_window_extension(value: &str) -> Result<String, String> {
    let extension = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if extension.is_empty() {
        return Ok(String::new());
    }
    if extension.len() > 32
        || !extension
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(
            "browse_window extensions may contain only ASCII letters, digits, underscore or dash"
                .to_string(),
        );
    }
    Ok(extension)
}

fn module_browse_window_filters(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<ModuleBrowseWindowFilter>, String> {
    let Some(filters) = object.get("filters") else {
        return Ok(Vec::new());
    };
    let filters = filters
        .as_array()
        .ok_or_else(|| "browse_window filters must be an array".to_string())?;
    let mut parsed = Vec::new();
    for filter in filters.iter().take(16) {
        let filter = filter
            .as_object()
            .ok_or_else(|| "browse_window filter must be an object".to_string())?;
        let mut extensions = Vec::new();
        let raw_extensions = filter
            .get("extensions")
            .and_then(|value| value.as_array())
            .ok_or_else(|| "browse_window filter extensions must be an array".to_string())?;
        for extension in raw_extensions.iter().take(32) {
            let extension = module_browse_window_extension(extension.as_str().unwrap_or_default())?;
            if !extension.is_empty() && !extensions.iter().any(|item| item == &extension) {
                extensions.push(extension);
            }
        }
        if extensions.is_empty() {
            continue;
        }
        let name = module_browse_window_string(filter, "name", 80)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| extensions.join(", "));
        parsed.push(ModuleBrowseWindowFilter { name, extensions });
    }
    Ok(parsed)
}

fn module_browse_window_initial_directory(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let path = Path::new(value);
    if path.is_dir() {
        return value.to_string();
    }
    path.parent()
        .map(|parent| parent.display().to_string())
        .filter(|parent| !parent.trim().is_empty())
        .unwrap_or_else(|| value.to_string())
}

fn module_browse_window_path_with_default_extension(
    mut path: PathBuf,
    default_extension: &str,
) -> PathBuf {
    if !default_extension.trim().is_empty()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none()
    {
        path.set_extension(default_extension.trim().trim_start_matches('.'));
    }
    path
}

fn open_module_browse_window_dialog(
    command: &IntegrationModuleHostCommandDto,
    module: &IntegrationModuleDto,
    mut module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    if dialog_open() {
        return;
    }
    let mut options = match module_browse_window_options_from_payload(&command.payload) {
        Ok(options) => options,
        Err(error) => {
            push_status_history_error_line(
                status_history,
                format!("{}: {error}", module.display_name),
            );
            return;
        }
    };
    if options.start_dir.trim().is_empty() {
        options.start_dir = module_ui_values
            .read()
            .get(&options.target)
            .and_then(|value| value.as_str())
            .map(module_browse_window_initial_directory)
            .unwrap_or_default();
    }

    let module_title = module.display_name.clone();
    dialog_open.set(true);
    spawn(async move {
        if let Some(mut path) = pick_module_browse_window_path(&window, &options).await {
            if options.mode == ModuleBrowseWindowMode::FileSave {
                path = module_browse_window_path_with_default_extension(
                    path,
                    &options.default_extension,
                );
                if options.overwrite_policy == ModuleBrowseWindowOverwritePolicy::Deny
                    && path.exists()
                {
                    push_status_history_error_line(
                        status_history,
                        format!("{module_title}: selected file already exists"),
                    );
                    dialog_open.set(false);
                    return;
                }
            }
            let selected_path = path.display().to_string();
            let mut write = module_ui_values.write();
            write.insert(
                options.target.clone(),
                serde_json::Value::String(selected_path),
            );
            if !options.status_target.trim().is_empty()
                && !options.selected_status.trim().is_empty()
            {
                write.insert(
                    options.status_target.clone(),
                    serde_json::Value::String(options.selected_status.clone()),
                );
            }
        }
        dialog_open.set(false);
    });
}

async fn pick_module_browse_window_path(
    window: &DesktopContext,
    options: &ModuleBrowseWindowOptions,
) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title(&options.title);
    if !options.start_dir.trim().is_empty() {
        dialog = dialog.set_directory(Path::new(&options.start_dir));
    }
    dialog = dialog.set_can_create_directories(options.can_create_directories);
    if !options.default_name.trim().is_empty() {
        let default_name = if options.mode == ModuleBrowseWindowMode::FileSave {
            module_browse_window_path_with_default_extension(
                PathBuf::from(options.default_name.trim()),
                &options.default_extension,
            )
            .display()
            .to_string()
        } else {
            options.default_name.clone()
        };
        dialog = dialog.set_file_name(default_name);
    }
    for filter in &options.filters {
        let extensions = filter
            .extensions
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    match options.mode {
        ModuleBrowseWindowMode::Folder => dialog
            .pick_folder()
            .await
            .map(|folder| folder.path().to_path_buf()),
        ModuleBrowseWindowMode::FileOpen => dialog
            .pick_file()
            .await
            .map(|file| file.path().to_path_buf()),
        ModuleBrowseWindowMode::FileSave => {
            let _ = options.confirm_label.trim();
            let _ = options.overwrite_policy == ModuleBrowseWindowOverwritePolicy::Allow;
            dialog
                .save_file()
                .await
                .map(|file| file.path().to_path_buf())
        }
    }
}

fn apply_module_host_commands(
    module: &IntegrationModuleDto,
    commands: &[IntegrationModuleHostCommandDto],
    mut module_host_dialog: Signal<Option<ModuleHostDialogState>>,
    mut module_ui_page: Signal<String>,
    module_ui_values: Signal<HashMap<String, serde_json::Value>>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    module_path_picker_dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    for command in commands {
        if command.command_type == "browse_window" {
            open_module_browse_window_dialog(
                command,
                module,
                module_ui_values,
                status_history,
                module_path_picker_dialog_open,
                window.clone(),
            );
            continue;
        }
        if command.command_type == "set_module_page" {
            if let Some(page) = command
                .payload
                .get("page")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                module_ui_page.set(page.chars().take(80).collect());
            }
            continue;
        }
        if command.command_type == "set_ui_values" {
            apply_module_ui_values_payload(&command.payload, module_ui_values);
            continue;
        }
        if let Some(dialog) = module_host_dialog_from_command(command, module) {
            let mut write = module_host_dialog.write();
            if write
                .as_ref()
                .is_none_or(|existing| existing.module_id == dialog.module_id)
            {
                *write = Some(dialog);
            }
        }
    }
}

fn apply_module_ui_values_payload(
    payload: &serde_json::Value,
    mut module_ui_values: Signal<HashMap<String, serde_json::Value>>,
) {
    let values = payload
        .get("values")
        .unwrap_or(payload)
        .as_object()
        .cloned()
        .unwrap_or_default();
    let mut write = module_ui_values.write();
    for (key, value) in values {
        let key = key.trim().chars().take(120).collect::<String>();
        if key.is_empty() {
            continue;
        }
        if value.is_null() {
            write.remove(&key);
        } else {
            write.insert(key, value);
        }
    }
}

fn safe_hex_color(value: &str) -> bool {
    let Some(hex) = value.trim().strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 6 | 8) && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn default_profile_export_path(current: &crate::watcher_api::IntegrationIntegrationDto) -> String {
    if let Some(path) = current
        .profile_paths
        .iter()
        .map(|path| path.trim())
        .find(|path| !path.is_empty())
    {
        return path.to_string();
    }
    if !path_is_configured(&current.repo_path) {
        return String::new();
    }
    current.repo_path.clone()
}

fn profile_export_integration_repo_key(
    current: &crate::watcher_api::IntegrationIntegrationDto,
) -> String {
    if path_is_configured(&current.repo_path) {
        normalize_path_for_duplicate_check(&current.repo_path)
    } else {
        String::new()
    }
}

fn profile_export_profile_options(
    current: &crate::watcher_api::IntegrationIntegrationDto,
    fallback: &str,
) -> Vec<String> {
    let mut options = current
        .profile_paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if options.is_empty() && !fallback.trim().is_empty() {
        options.push(fallback.trim().to_string());
    }
    options.sort_by_key(|path| {
        (
            path != fallback,
            profile_export_profile_option_label(path, &current.repo_path).to_ascii_lowercase(),
        )
    });
    options.dedup();
    options
}

fn profile_export_profile_option_label(path: &str, repo_root: &str) -> String {
    let file_name = PathBuf::from(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string());
    if repo_root.trim().is_empty() {
        return file_name;
    }

    let normalized_path = path.replace('/', "\\").to_ascii_lowercase();
    let normalized_root = repo_root
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase();
    if normalized_path.starts_with(&normalized_root) {
        file_name
    } else {
        path.to_string()
    }
}

fn default_profile_export_ui_state(
    repo_root_key: String,
    default_path: String,
) -> ProfileExportUiStateDto {
    ProfileExportUiStateDto {
        repo_root_key,
        mode: ExportModeDto::default(),
        attach_profile_path: default_path.clone(),
        patch_profile_path: default_path.clone(),
        merge_profile_path: default_path,
        generated_profile_name: "NetStitch".to_string(),
        dangerous_confirmed: false,
        advanced_create_rules_for_uncovered: false,
        advanced_add_ips_to_exclude: false,
        advanced_use_whois_ranges_for_export: false,
        advanced_add_detected_domains: false,
        advanced_manual_domains: String::new(),
        advanced_template_rule_source: String::new(),
        advanced_ready_for_export: false,
        advanced_domain_cleanup_requested: false,
    }
}

fn profile_export_ui_state_with_defaults(
    state: ProfileExportUiStateDto,
    _default_path: &str,
) -> ProfileExportUiStateDto {
    state
}

fn profile_export_request(
    current: &crate::watcher_api::IntegrationIntegrationDto,
    mode: ExportModeDto,
    selected_profile_path: &str,
    generated_profile_name: &str,
    dangerous_confirmed: bool,
) -> Result<ExportProfileRequestDto, String> {
    if !current.ready || !path_is_configured(&current.repo_path) {
        return Err(
            "Configure the integration folder before exporting confirmed IP addresses.".to_string(),
        );
    }

    let selected_profile_path = selected_profile_path.trim();
    let selected_profile_path = if selected_profile_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(selected_profile_path))
    };
    let generated_profile_name = generated_profile_name.trim();
    let generated_profile_name = if generated_profile_name.is_empty() {
        None
    } else {
        Some(generated_profile_name.to_string())
    };

    Ok(ExportProfileRequestDto {
        provider_id: integration_provider_id(current),
        repo_root: PathBuf::from(&current.repo_path),
        selected_profile_path,
        generated_profile_name,
        mode,
        dangerous_confirmed,
    })
}

fn profile_export_request_key(request: &ExportProfileRequestDto) -> String {
    format!(
        "{:?}|{}|{}|{}|{}",
        request.mode,
        normalize_path_for_duplicate_check(&request.repo_root.display().to_string()),
        request
            .selected_profile_path
            .as_ref()
            .map(|path| normalize_path_for_duplicate_check(&path.display().to_string()))
            .unwrap_or_default(),
        request.generated_profile_name.clone().unwrap_or_default(),
        request.dangerous_confirmed
    )
}

fn split_manual_domains(value: &str) -> Vec<String> {
    value
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn selected_profile_export_domains(snapshot: &SnapshotResponse) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut domains = Vec::new();
    for observation in snapshot
        .observations
        .iter()
        .filter(|observation| observation.is_confirmed && !observation.is_exported)
    {
        let domain = csv_export_domain_text(&observation.enrichment);
        let key = domain.trim().to_ascii_lowercase();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        domains.push(domain);
    }
    domains
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CloudAuthorPublicationRow {
    app_key: String,
    app_name: String,
    new_rows: u64,
    non_public_rows: u64,
    author_rows: u64,
    total_rows_label: String,
}

fn build_cloud_author_publication_rows(
    snapshot: &SnapshotResponse,
    my_apps: &[CloudUserAppSummary],
    selected_observation_ids: &BTreeSet<u64>,
    uploaded_observation_ids: &BTreeSet<u64>,
    _upload_visibility: CloudObservationVisibility,
    sort_state: CloudPublicationSortState,
) -> Vec<CloudAuthorPublicationRow> {
    let cloud_app_names_by_id = my_apps
        .iter()
        .filter_map(|app| {
            let app_id = normalized_cloud_author_app_id(&app.app_id)?;
            let display_name = app.display_name.trim();
            (!display_name.is_empty()).then(|| (app_id, display_name.to_string()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut rows_by_key = BTreeMap::<String, CloudAuthorPublicationRow>::new();
    for app in my_apps {
        let app_key = cloud_author_publication_key(&app.app_id);
        let app_name = app.display_name.trim();
        let row = rows_by_key
            .entry(app_key.clone())
            .or_insert_with(|| CloudAuthorPublicationRow {
                app_key,
                app_name: if app_name.is_empty() {
                    app.app_id.clone()
                } else {
                    app_name.to_string()
                },
                new_rows: 0,
                non_public_rows: 0,
                author_rows: 0,
                total_rows_label: "0".to_string(),
            });
        row.author_rows = row.author_rows.saturating_add(app.endpoint_count);
        let total_rows = row
            .total_rows_label
            .parse::<u64>()
            .unwrap_or(0)
            .saturating_add(app.total_endpoint_count);
        row.total_rows_label = total_rows.to_string();
    }
    let mut local_upload_apps = BTreeMap::new();
    let mut selected_rows_by_key = BTreeMap::<String, u64>::new();
    let mut non_public_rows_by_key = BTreeMap::<String, u64>::new();

    for observation in snapshot.observations.iter().filter(|observation| {
        selected_observation_ids.contains(&observation.id)
            && !uploaded_observation_ids.contains(&observation.id)
            && observation.is_confirmed
    }) {
        let Some(upload_app) =
            cloud_upload_app_preview_for_observation(snapshot, observation, &mut local_upload_apps)
        else {
            continue;
        };
        let app_key = cloud_author_publication_key(&upload_app.app_id);
        let app_name = upload_app
            .display_name
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                normalized_cloud_author_app_id(&upload_app.app_id)
                    .and_then(|app_id| cloud_app_names_by_id.get(&app_id).cloned())
            })
            .unwrap_or(upload_app.app_id);
        let is_public_ip = observation
            .remote_ip
            .parse::<IpAddr>()
            .map(netstitch_shared::cloud_observation_ip_is_public)
            .unwrap_or(false);
        if is_public_ip {
            *selected_rows_by_key.entry(app_key.clone()).or_default() += 1;
        } else {
            *non_public_rows_by_key.entry(app_key.clone()).or_default() += 1;
        }
        rows_by_key
            .entry(app_key.clone())
            .or_insert_with(|| CloudAuthorPublicationRow {
                app_key,
                app_name: app_name.clone(),
                new_rows: 0,
                non_public_rows: 0,
                author_rows: 0,
                total_rows_label: "0".to_string(),
            })
            .app_name = app_name.clone();
    }

    for (app_key, selected_count) in selected_rows_by_key {
        if let Some(row) = rows_by_key.get_mut(&app_key) {
            row.new_rows = selected_count;
        }
    }
    for (app_key, skipped_count) in non_public_rows_by_key {
        if let Some(row) = rows_by_key.get_mut(&app_key) {
            row.non_public_rows = skipped_count;
        }
    }

    let mut rows = rows_by_key.into_values().collect::<Vec<_>>();
    sort_cloud_author_publication_rows(&mut rows, sort_state);
    rows
}

fn cloud_author_publication_key(app_id: &str) -> String {
    normalized_cloud_author_app_id(app_id).unwrap_or_default()
}

fn normalized_cloud_author_app_id(app_id: &str) -> Option<String> {
    let normalized = app_id
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn sort_cloud_author_publication_rows(
    rows: &mut [CloudAuthorPublicationRow],
    sort_state: CloudPublicationSortState,
) {
    let column = sort_state.column.unwrap_or(CloudPublicationSortColumn::App);
    rows.sort_by(|left, right| {
        let ordering = match column {
            CloudPublicationSortColumn::App => left
                .app_name
                .to_lowercase()
                .cmp(&right.app_name.to_lowercase())
                .then_with(|| left.app_name.cmp(&right.app_name)),
            CloudPublicationSortColumn::NewRows => left.new_rows.cmp(&right.new_rows),
            CloudPublicationSortColumn::NonPublicRows => {
                left.non_public_rows.cmp(&right.non_public_rows)
            }
            CloudPublicationSortColumn::AuthorRows => left.author_rows.cmp(&right.author_rows),
            CloudPublicationSortColumn::TotalRows => left
                .total_rows_label
                .parse::<u64>()
                .unwrap_or(0)
                .cmp(&right.total_rows_label.parse::<u64>().unwrap_or(0)),
        }
        .then_with(|| {
            left.app_name
                .to_lowercase()
                .cmp(&right.app_name.to_lowercase())
        })
        .then_with(|| left.app_name.cmp(&right.app_name))
        .then_with(|| left.app_key.cmp(&right.app_key));
        if sort_state.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn merge_manual_domains(current: &str, additions: &[String]) -> String {
    let mut seen = BTreeSet::new();
    let mut domains = Vec::new();
    for domain in split_manual_domains(current)
        .into_iter()
        .chain(additions.iter().cloned())
    {
        let value = domain.trim();
        let key = value.to_ascii_lowercase();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        domains.push(value.to_string());
    }
    domains.join("\n")
}

fn export_file_operation_label(operation: ExportFileOperationDto) -> &'static str {
    match operation {
        ExportFileOperationDto::Create => "create",
        ExportFileOperationDto::Update => "update",
        ExportFileOperationDto::Skip => "skip",
    }
}

fn export_file_change_label(change: &ExportFileChangeDto) -> String {
    format!(
        "{}: {}",
        export_file_operation_label(change.operation),
        change.path.display()
    )
}

fn profile_export_file_changes(plan: &ExportProfilePlanDto) -> Vec<ExportFileChangeDto> {
    plan.file_changes
        .iter()
        .chain(plan.advanced_file_changes.iter())
        .cloned()
        .collect()
}

fn profile_export_address_new_count(plan: &ExportProfilePlanDto) -> usize {
    plan.new_ips.len()
}

fn profile_export_address_skipped_count(plan: &ExportProfilePlanDto) -> usize {
    plan.existing_ips.len()
        + plan.skipped_unconfirmed
        + plan.skipped_exported
        + plan.skipped_duplicates
}

fn export_profile_rule_label(rule: &ExportProfileRuleDto) -> String {
    let ports = if !rule.ports_display.trim().is_empty() {
        rule.ports_display.clone()
    } else if rule.ports.is_empty() {
        "*".to_string()
    } else {
        compact_port_ranges(&rule.ports)
    };
    let lists = if rule.list_refs.is_empty() {
        "-".to_string()
    } else {
        rule.list_refs.join(", ")
    };
    format!("{} {ports} -> {lists}", rule.protocol.as_str())
}

fn compact_port_ranges(ports: &[u16]) -> String {
    if ports.is_empty() {
        return "*".to_string();
    }

    let mut compact = Vec::new();
    let mut start = ports[0];
    let mut previous = ports[0];

    for &port in ports.iter().skip(1) {
        if port == previous.saturating_add(1) {
            previous = port;
            continue;
        }

        compact.push(if start == previous {
            start.to_string()
        } else {
            format!("{start}-{previous}")
        });
        start = port;
        previous = port;
    }

    compact.push(if start == previous {
        start.to_string()
    } else {
        format!("{start}-{previous}")
    });
    compact.join(", ")
}

fn start_integration_download(
    mut watcher: Signal<AppWatcherApi>,
    mut integration_path_input: Signal<String>,
    mut integration_feedback: Signal<Option<String>>,
    mut integration_download: Signal<IntegrationDownloadUiState>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    progress_labels: IntegrationProgressLabels,
    progress_label: String,
    module_id: Option<String>,
    provider_id: String,
) {
    let generation = integration_download().generation.wrapping_add(1);
    let initial_state = IntegrationDownloadUiState {
        active: true,
        visible: true,
        cancelled: false,
        generation,
        stage: "preparing".to_string(),
        percent: None,
        downloaded_bytes: None,
        total_bytes: None,
        extracted_entries: None,
        total_entries: None,
        message: None,
        repo_root: None,
    };
    push_status_history_line(
        status_history,
        integration_progress_footer_line(&progress_label, &initial_state, &progress_labels),
    );
    integration_download.set(initial_state);
    integration_feedback.set(None);

    spawn(async move {
        let live_base_url = watcher.read().live_base_url();
        let Some(base_url) = live_base_url else {
            {
                let mut current = watcher.write();
                current.download_integration(DownloadIntegrationRequest {
                    module_id: module_id.clone(),
                    provider_id: provider_id.clone(),
                });
            }

            let mut state = integration_download();
            if state.generation != generation || state.cancelled {
                return;
            }

            let snapshot = watcher.read().snapshot();
            let repo_path = snapshot.integration.repo_path;
            if path_is_configured(&repo_path) {
                integration_path_input.set(repo_path.clone());
                integration_feedback.set(None);
            }

            state.active = false;
            state.stage = "complete".to_string();
            state.percent = Some(100);
            state.repo_root = Some(repo_path);
            push_status_history_line(
                status_history,
                integration_progress_footer_line(&progress_label, &state, &progress_labels),
            );
            integration_download.set(state);
            return;
        };

        let provider_id_string = provider_id.to_string();
        let start_base_url = base_url.clone();
        let start_module_id = module_id.clone();
        let start_provider_id = provider_id_string.clone();
        let start_result = tokio::task::spawn_blocking(move || {
            start_integration_provider_via_http(
                &start_base_url,
                start_module_id.as_deref(),
                &start_provider_id,
            )
        })
        .await;
        let mut last_logged_stage = "preparing".to_string();

        match start_result {
            Ok(Ok(progress)) => {
                let state = integration_download();
                if state.generation != generation || state.cancelled {
                    return;
                }
                let next_state = IntegrationDownloadUiState::from_dto(progress, generation, false);
                if next_state.stage != last_logged_stage {
                    push_status_history_line(
                        status_history,
                        integration_progress_footer_line(
                            &progress_label,
                            &next_state,
                            &progress_labels,
                        ),
                    );
                    last_logged_stage = next_state.stage.clone();
                }
                integration_download.set(next_state);
            }
            Ok(Err(message)) => {
                let mut state = integration_download();
                if state.generation != generation || state.cancelled {
                    return;
                }
                state.active = false;
                state.visible = true;
                state.stage = "failed".to_string();
                state.message = Some(message.clone());
                integration_feedback.set(Some(message));
                integration_download.set(state.clone());
                push_status_history_error_line(
                    status_history,
                    integration_progress_footer_line(&progress_label, &state, &progress_labels),
                );
                return;
            }
            Err(error) => {
                let mut state = integration_download();
                if state.generation != generation || state.cancelled {
                    return;
                }
                let message = error.to_string();
                state.active = false;
                state.visible = true;
                state.stage = "failed".to_string();
                state.message = Some(message.clone());
                integration_feedback.set(Some(message));
                integration_download.set(state.clone());
                push_status_history_error_line(
                    status_history,
                    integration_progress_footer_line(&progress_label, &state, &progress_labels),
                );
                return;
            }
        }

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            let progress_base_url = base_url.clone();
            let progress_result = tokio::task::spawn_blocking(move || {
                fetch_integration_download_progress_via_http(&progress_base_url)
            })
            .await;
            let Ok(Ok(progress)) = progress_result else {
                continue;
            };
            let state = integration_download();
            if state.generation != generation || state.cancelled {
                return;
            }
            let next_state = IntegrationDownloadUiState::from_dto(progress, generation, false);
            if next_state.stage != last_logged_stage {
                let line = integration_progress_footer_line(
                    &progress_label,
                    &next_state,
                    &progress_labels,
                );
                if next_state.stage == "failed" {
                    push_status_history_error_line(status_history, line);
                } else if next_state.stage == "complete" {
                    push_status_history_success_line(status_history, line);
                } else {
                    push_status_history_line(status_history, line);
                }
                last_logged_stage = next_state.stage.clone();
            }
            let terminal_stage = next_state.stage == "complete" || next_state.stage == "failed";
            if next_state.stage == "complete" {
                if let Some(repo_path) = next_state.repo_root.clone() {
                    integration_path_input.set(repo_path.clone());
                    integration_feedback.set(None);
                    let _ = watcher
                        .write()
                        .configure_integration_folder(module_id.clone(), &repo_path);
                }
            } else if next_state.stage == "failed" {
                let message = next_state
                    .message
                    .clone()
                    .unwrap_or_else(|| "integration download failed".to_string());
                integration_feedback.set(Some(message));
            }
            integration_download.set(next_state);
            if terminal_stage {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        let mut latest = integration_download();
        if latest.generation == generation && !latest.cancelled {
            latest.active = false;
            integration_download.set(latest);
        }
    });
}

fn start_integration_provider_via_http(
    base_url: &str,
    module_id: Option<&str>,
    provider_id: &str,
) -> Result<IntegrationDownloadProgressDto, String> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(desktop_http_headers())
        .build()
        .map_err(|error| format!("failed to construct HTTP client: {error}"))?;
    let response = client
        .post(format!("{base_url}/v1/integrations/download/start"))
        .json(&SharedDownloadIntegrationProviderRequest {
            module_id: module_id.map(ToOwned::to_owned),
            provider_id: provider_id.to_string(),
        })
        .send()
        .map_err(|error| format!("request failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let details = response.text().unwrap_or_default();
        return Err(format!(
            "integration download failed with {status}: {details}"
        ));
    }

    response
        .json::<IntegrationDownloadProgressDto>()
        .map_err(|error| format!("invalid integration download start response: {error}"))
}

fn fetch_integration_download_progress_via_http(
    base_url: &str,
) -> Result<IntegrationDownloadProgressDto, String> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(desktop_http_headers())
        .build()
        .map_err(|error| format!("failed to construct HTTP client: {error}"))?;
    let response = client
        .get(format!("{base_url}/v1/integrations/download-progress"))
        .send()
        .map_err(|error| format!("request failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let details = response.text().unwrap_or_default();
        return Err(format!(
            "integration progress failed with {status}: {details}"
        ));
    }

    response
        .json::<IntegrationDownloadProgressDto>()
        .map_err(|error| format!("invalid integration progress response: {error}"))
}

fn integration_progress_footer_line(
    label: &str,
    state: &IntegrationDownloadUiState,
    labels: &IntegrationProgressLabels,
) -> String {
    let stage = labels.stage_label(&state.stage);
    if let Some(message) = state
        .message
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        return format!("{label}: {stage} - {message}");
    }
    if state.stage == "complete" {
        if let Some(repo_root) = state
            .repo_root
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            return format!("{label}: {stage} - {repo_root}");
        }
    }
    format!("{label}: {stage}")
}

fn open_integration_folder_dialog(
    mut integration_path_input: Signal<String>,
    mut integration_path_applied: Signal<String>,
    mut integration_feedback: Signal<Option<String>>,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
    input_apply_pulse: Signal<Option<&'static str>>,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        if let Some(folder) = pick_integration_folder_path(&window).await {
            let selected = folder.display().to_string();
            integration_path_input.set(selected.clone());
            integration_path_applied.set(selected);
            integration_feedback.set(None);
            pulse_text_input(input_apply_pulse, "integration-path");
        }
        dialog_open.set(false);
    });
}

fn open_integration_folder_dialog_for_scan(
    mut watcher: Signal<AppWatcherApi>,
    mut show_integration_prompt: Signal<bool>,
    mut integration_path_input: Signal<String>,
    mut integration_feedback: Signal<Option<String>>,
    mut pending_scan_after_setup: Signal<bool>,
    monitoring: bool,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        if let Some(folder) = pick_integration_folder_path(&window).await {
            let candidate = folder.display().to_string();
            let mut current = watcher.write();
            if current
                .configure_integration_folder(None, &candidate)
                .is_ok()
            {
                if monitoring {
                    current.stop_monitoring();
                } else {
                    current.start_monitoring();
                }
                pending_scan_after_setup.set(false);
                dialog_open.set(false);
                return;
            }
        }

        show_integration_prompt.set(true);
        integration_feedback.set(Some(
            "Select the integration folder before scanning can start.".to_string(),
        ));
        if integration_path_input().is_empty() {
            let repo_path = watcher.read().snapshot().integration.repo_path;
            if repo_path != "Not configured" {
                integration_path_input.set(repo_path);
            }
        }
        pending_scan_after_setup.set(!monitoring);
        dialog_open.set(false);
    });
}

fn open_profile_export_file_dialog(
    mut profile_export_path_input: Signal<String>,
    mut profile_export_path_draft: Signal<String>,
    mut profile_export_feedback: Signal<Option<String>>,
    mut profile_export_preview: Signal<Option<ExportProfilePlanDto>>,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        if let Some(file) = pick_integration_profile_file_path(&window).await {
            let value = file.display().to_string();
            profile_export_path_draft.set(value.clone());
            profile_export_path_input.set(value);
            profile_export_feedback.set(None);
            profile_export_preview.set(None);
        }
        dialog_open.set(false);
    });
}

fn open_csv_export_file_dialog(
    rows: Vec<CsvExportRow>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    success_message: String,
    failed_message: String,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
    watcher_base_url: Option<String>,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        if let Some(path) = save_csv_export_file_path(&window).await {
            let path = ensure_csv_extension(path);
            let contents = render_csv_export_rows(&rows);
            match std::fs::write(&path, contents.as_bytes()) {
                Ok(()) => {
                    record_system_event_via_watcher(
                        watcher_base_url.clone(),
                        SystemEventRequestDto {
                            source: Some("system".to_string()),
                            component: "csv".to_string(),
                            action_type: "export_file".to_string(),
                            severity: "info".to_string(),
                            entity_type: Some("file".to_string()),
                            entity_id: Some(path.display().to_string()),
                            payload: serde_json::json!({
                                "path": path.display().to_string(),
                                "rows": rows.len(),
                                "bytes": contents.len(),
                            }),
                        },
                    );
                    push_status_history_line(
                        status_history,
                        format!("{success_message}: {}", path.display()),
                    );
                }
                Err(error) => push_status_history_error_line(
                    status_history,
                    format!("{failed_message}: {error}"),
                ),
            }
        }
        dialog_open.set(false);
    });
}

fn open_csv_import_file_dialog(
    mut watcher: Signal<AppWatcherApi>,
    status_history: Signal<Vec<StatusHistoryLine>>,
    success_message: String,
    failed_message: String,
    empty_message: String,
    mut dialog_open: Signal<bool>,
    window: DesktopContext,
) {
    if dialog_open() {
        return;
    }

    dialog_open.set(true);
    spawn(async move {
        if let Some(path) = pick_csv_import_file_path(&window).await {
            let result = std::fs::read_to_string(&path)
                .map_err(|error| format!("{failed_message}: {error}"))
                .and_then(|contents| parse_csv_import_request(&contents));
            match result {
                Ok(request) if request.rows.is_empty() => {
                    push_status_history_line(status_history, empty_message.clone());
                }
                Ok(request) => {
                    let watcher_base_url = watcher.read().live_base_url();
                    match watcher.write().import_monitoring_csv(request) {
                        Ok(result) => {
                            record_system_event_via_watcher(
                                watcher_base_url,
                                SystemEventRequestDto {
                                    source: Some("system".to_string()),
                                    component: "csv".to_string(),
                                    action_type: "import_file".to_string(),
                                    severity: "info".to_string(),
                                    entity_type: Some("file".to_string()),
                                    entity_id: Some(path.display().to_string()),
                                    payload: serde_json::json!({
                                        "path": path.display().to_string(),
                                        "requested_count": result.requested_count,
                                        "imported_count": result.imported_count,
                                        "skipped_count": result.skipped_count,
                                    }),
                                },
                            );
                            push_status_history_line(
                                status_history,
                                csv_import_status_line(&success_message, &path, &result),
                            );
                        }
                        Err(error) => push_status_history_error_line(
                            status_history,
                            format!("{failed_message}: {error}"),
                        ),
                    }
                }
                Err(error) => push_status_history_error_line(status_history, error),
            }
        }
        dialog_open.set(false);
    });
}

async fn pick_executable_path(window: &DesktopContext) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title("Choose executable file")
        .add_filter("Executable files", &["exe", "lnk"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn pick_integration_profile_file_path(window: &DesktopContext) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title("Choose integration profile")
        .add_filter("Integration profiles", &["bat", "cmd", "conf", "txt"])
        .add_filter("Batch files", &["bat", "cmd"])
        .add_filter("Config files", &["conf", "txt"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn pick_integration_folder_path(window: &DesktopContext) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title("Choose integration folder")
        .pick_folder()
        .await
        .map(|folder| folder.path().to_path_buf())
}

async fn pick_csv_import_file_path(window: &DesktopContext) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title("Импортировать мониторинг из CSV")
        .add_filter("CSV files", &["csv"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn save_csv_export_file_path(window: &DesktopContext) -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_parent(window.window.as_ref())
        .set_title("Сохранить выбранные строки мониторинга в CSV")
        .set_file_name("NetStitch-monitoring-selected.csv")
        .add_filter("CSV files", &["csv"])
        .save_file()
        .await
        .map(|file| file.path().to_path_buf())
}

fn ensure_csv_extension(mut path: std::path::PathBuf) -> std::path::PathBuf {
    let has_csv_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("csv"))
        .unwrap_or(false);
    if !has_csv_extension {
        path.set_extension("csv");
    }
    path
}

fn restore_window(
    window: &DesktopContext,
    maximized: bool,
    remember_window_placement: bool,
    remembered_window_placement: &mut Signal<Option<PersistedWindowPlacement>>,
) {
    window.set_minimized(false);
    if remember_window_placement {
        if let Some(placement) = remembered_window_placement().as_ref() {
            apply_persisted_window_placement(window, placement);
        }
    } else {
        center_window_at_minimum_size(window);
    }
    window.set_visible(true);
    window.set_maximized(maximized);
    window.set_focus();
    if remember_window_placement {
        if let Ok(placement) = capture_current_window_placement(window, false) {
            remembered_window_placement.set(Some(placement));
        } else if let Some(mut placement) = remembered_window_placement() {
            placement.hidden = false;
            remembered_window_placement.set(Some(placement));
        }
    }
}

fn hide_window(
    window: &DesktopContext,
    hidden_window_was_maximized: &mut Signal<bool>,
    remembered_window_placement: &mut Signal<Option<PersistedWindowPlacement>>,
) {
    hidden_window_was_maximized.set(window.is_maximized());
    if let Some(mut placement) = remembered_window_placement() {
        placement.hidden = true;
        remembered_window_placement.set(Some(placement));
    }
    window.set_visible(false);
}

fn apply_persisted_window_placement(window: &DesktopContext, placement: &PersistedWindowPlacement) {
    window.set_maximized(false);
    window.set_inner_size(PhysicalSize::new(
        placement.width.max(crate::WINDOW_MIN_WIDTH) as u32,
        placement.height.max(crate::WINDOW_MIN_HEIGHT) as u32,
    ));
    window.set_outer_position(PhysicalPosition::new(placement.x, placement.y));
}

fn center_window_at_minimum_size(window: &DesktopContext) {
    window.set_maximized(false);
    window.set_inner_size(PhysicalSize::new(
        crate::WINDOW_MIN_WIDTH as u32,
        crate::WINDOW_MIN_HEIGHT as u32,
    ));
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let width = crate::WINDOW_MIN_WIDTH as i32;
        let height = crate::WINDOW_MIN_HEIGHT as i32;
        let x = monitor_position.x + ((monitor_size.width as i32 - width) / 2).max(0);
        let y = monitor_position.y + ((monitor_size.height as i32 - height) / 2).max(0);
        window.set_outer_position(PhysicalPosition::new(x, y));
    }
}

fn persist_current_window_placement(window: &DesktopContext, hidden: bool) -> Result<(), String> {
    let placement = capture_current_window_placement(window, hidden)?;
    persist_window_placement_to_sqlite(&placement)
}

fn capture_current_window_placement(
    window: &DesktopContext,
    hidden: bool,
) -> Result<PersistedWindowPlacement, String> {
    let position = window
        .outer_position()
        .map_err(|error| format!("failed to read window position: {error}"))?;
    let size = window.inner_size();
    Ok(PersistedWindowPlacement {
        x: position.x,
        y: position.y,
        width: size.width as f64,
        height: size.height as f64,
        hidden,
    })
}

fn should_skip_duplicate_tray_menu_event(
    mut last_tray_menu_event: Signal<Option<(String, Instant)>>,
    id: &str,
) -> bool {
    let now = Instant::now();
    let duplicate = last_tray_menu_event()
        .as_ref()
        .is_some_and(|(last_id, at)| {
            last_id == id && now.duration_since(*at) <= Duration::from_millis(200)
        });
    last_tray_menu_event.set(Some((id.to_string(), now)));
    duplicate
}

#[allow(clippy::too_many_arguments)]
fn handle_tray_menu_action(
    action: TrayMenuAction,
    window: &DesktopContext,
    tray: &TrayController,
    mut watcher: Signal<AppWatcherApi>,
    show_integration_prompt: Signal<bool>,
    integration_path_input: Signal<String>,
    integration_feedback: Signal<Option<String>>,
    pending_scan_after_setup: Signal<bool>,
    mut hide_when_minimized: Signal<bool>,
    mut remember_window_placement: Signal<bool>,
    mut remembered_window_placement: Signal<Option<PersistedWindowPlacement>>,
    mut monitoring_ui_refresh_nonce: Signal<u64>,
    mut window_visible: Signal<bool>,
    mut hidden_window_was_maximized: Signal<bool>,
    integration_folder_dialog_open: Signal<bool>,
) {
    match action {
        TrayMenuAction::ToggleWindowVisibility => {
            let currently_visible = window.is_visible() && window_visible();
            if currently_visible {
                hide_window(
                    window,
                    &mut hidden_window_was_maximized,
                    &mut remembered_window_placement,
                );
                window_visible.set(false);
            } else {
                restore_window(
                    window,
                    hidden_window_was_maximized(),
                    remember_window_placement(),
                    &mut remembered_window_placement,
                );
                window_visible.set(true);
            }
            let snapshot = watcher.read().snapshot();
            tray.sync(
                snapshot.ui.monitoring,
                hide_when_minimized(),
                snapshot.app_settings.web_access_localhost,
                remember_window_placement(),
                !currently_visible,
            );
        }
        TrayMenuAction::ToggleHideWhenMinimized => {
            let next_hide_when_minimized = !hide_when_minimized();
            hide_when_minimized.set(next_hide_when_minimized);
            watcher
                .write()
                .set_hide_when_minimized(next_hide_when_minimized);
            let snapshot = watcher.read().snapshot();
            tray.sync(
                snapshot.ui.monitoring,
                next_hide_when_minimized,
                snapshot.app_settings.web_access_localhost,
                remember_window_placement(),
                window_visible(),
            );
        }
        TrayMenuAction::ToggleWebServer => {
            let enabled = !watcher.read().snapshot().app_settings.web_access_localhost;
            watcher.write().set_web_access_localhost(enabled);
            let snapshot = watcher.read().snapshot();
            tray.sync(
                snapshot.ui.monitoring,
                hide_when_minimized(),
                enabled,
                remember_window_placement(),
                window_visible(),
            );
        }
        TrayMenuAction::ToggleRememberWindowPlacement => {
            let enabled = !remember_window_placement();
            remember_window_placement.set(enabled);
            watcher.write().set_remember_window_placement(enabled);
            let snapshot = watcher.read().snapshot();
            tray.sync(
                snapshot.ui.monitoring,
                hide_when_minimized(),
                snapshot.app_settings.web_access_localhost,
                enabled,
                window_visible(),
            );
        }
        TrayMenuAction::ToggleScanning => {
            let _ = handle_scan_toggle(
                &mut watcher,
                window,
                show_integration_prompt,
                integration_path_input,
                integration_feedback,
                pending_scan_after_setup,
                window_visible,
                remember_window_placement(),
                remembered_window_placement,
                hidden_window_was_maximized,
                integration_folder_dialog_open,
            );
            monitoring_ui_refresh_nonce.set(monitoring_ui_refresh_nonce().wrapping_add(1));
            let snapshot = watcher.read().snapshot();
            tray.sync(
                snapshot.ui.monitoring,
                hide_when_minimized(),
                snapshot.app_settings.web_access_localhost,
                remember_window_placement(),
                window_visible(),
            );
        }
        TrayMenuAction::Exit => exit_app(
            window,
            &mut watcher,
            remember_window_placement(),
            !window_visible(),
            remembered_window_placement(),
        ),
        TrayMenuAction::Ignore => {}
    }
}

fn path_is_configured(path: &str) -> bool {
    let trimmed = path.trim();
    !trimmed.is_empty()
        && !trimmed.eq_ignore_ascii_case("not configured")
        && !trimmed.eq_ignore_ascii_case("not found")
}

fn update_check_interval_duration(minutes: u64) -> Duration {
    Duration::from_secs(minutes.clamp(1, 24 * 60) * 60)
}

fn update_check_system_event(
    current_version: &str,
    result: &AppUpdateCheckState,
) -> SystemEventRequestDto {
    SystemEventRequestDto {
        source: Some("core".to_string()),
        component: "updates".to_string(),
        action_type: "check_response".to_string(),
        severity: if result.error.is_some() {
            "warning".to_string()
        } else {
            "info".to_string()
        },
        entity_type: Some("github_release".to_string()),
        entity_id: result.latest_version.clone(),
        payload: serde_json::json!({
            "current_version": current_version,
            "latest_version": result.latest_version,
            "update_available": result.update_available,
            "error": result.error,
        }),
    }
}

fn record_system_event_via_watcher(base_url: Option<String>, request: SystemEventRequestDto) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn_blocking(move || send_system_event_to_watcher(base_url, request));
    } else {
        send_system_event_to_watcher(base_url, request);
    }
}

fn send_system_event_to_watcher(base_url: Option<String>, request: SystemEventRequestDto) {
    let Some(base_url) = base_url else {
        return;
    };
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .default_headers(desktop_http_headers())
        .build()
    else {
        return;
    };
    let _ = client
        .post(format!("{base_url}/v1/system-events"))
        .json(&request)
        .send();
}

fn check_latest_app_update(current_version: &str) -> AppUpdateCheckState {
    let response = match reqwest::blocking::Client::builder()
        .timeout(UPDATE_CHECK_TIMEOUT)
        .build()
        .and_then(|client| {
            client
                .get(APP_LATEST_RELEASE_API_URL)
                .header(reqwest::header::USER_AGENT, "NetStitch update check")
                .header(reqwest::header::ACCEPT, "application/vnd.github+json")
                .send()
        }) {
        Ok(response) => response,
        Err(error) => {
            return AppUpdateCheckState {
                error: Some(format!("failed to check GitHub release: {error}")),
                ..AppUpdateCheckState::default()
            };
        }
    };

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return AppUpdateCheckState::default();
    }
    if !response.status().is_success() {
        return AppUpdateCheckState {
            error: Some(format!(
                "GitHub release check returned {}",
                response.status()
            )),
            ..AppUpdateCheckState::default()
        };
    }

    let release = match response.json::<serde_json::Value>() {
        Ok(release) => release,
        Err(error) => {
            return AppUpdateCheckState {
                error: Some(format!("GitHub release response is invalid: {error}")),
                ..AppUpdateCheckState::default()
            };
        }
    };
    let latest_version = release
        .get("tag_name")
        .and_then(|value| value.as_str())
        .and_then(extract_version_text);
    let latest_url = release
        .get("html_url")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| Some(APP_RELEASES_URL.to_string()));
    let update_available = latest_version
        .as_deref()
        .is_some_and(|version| compare_version_text(version, current_version) == Ordering::Greater);

    AppUpdateCheckState {
        latest_version,
        latest_url,
        update_available,
        error: None,
    }
}

fn extract_version_text(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches(['v', 'V']);
    let first_digit = trimmed.find(|ch: char| ch.is_ascii_digit())?;
    let version = trimmed[first_digit..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .trim_matches('.')
        .to_string();
    (!version.is_empty()).then_some(version)
}

fn compare_version_text(left: &str, right: &str) -> Ordering {
    let mut left_parts = version_number_parts(left);
    let mut right_parts = version_number_parts(right);
    let part_count = left_parts.len().max(right_parts.len());
    left_parts.resize(part_count, 0);
    right_parts.resize(part_count, 0);
    left_parts.cmp(&right_parts)
}

fn version_number_parts(value: &str) -> Vec<u64> {
    extract_version_text(value)
        .unwrap_or_else(|| value.to_string())
        .split('.')
        .filter_map(|part| part.trim().parse::<u64>().ok())
        .collect()
}

fn open_app_folder(exe_path: &str) {
    let path = std::path::Path::new(exe_path);
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("explorer.exe");
        command.arg(folder);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("open");
        command.arg(folder);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(folder);
        command
    };

    let _ = command.spawn();
}

fn local_file_url(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    format!("file:///{}", percent_encode_path(&normalized))
}

fn icon_image_src(path: &str) -> String {
    if let Ok(cache) = ICON_IMAGE_SRC_CACHE.lock() {
        if let Some(cached) = cache.get(path) {
            return cached.clone();
        }
    }

    let src = compute_icon_image_src(path);
    if let Ok(mut cache) = ICON_IMAGE_SRC_CACHE.lock() {
        cache.insert(path.to_string(), src.clone());
    }
    src
}

fn compute_icon_image_src(path: &str) -> String {
    let path_ref = std::path::Path::new(path);
    let extension = path_ref
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());
    if extension.as_deref() == Some("svg") {
        if let Ok(svg) = std::fs::read_to_string(path_ref) {
            return inline_svg_data_uri(&svg);
        }
    }
    if let Some(mime) = extension.as_deref().and_then(bitmap_icon_mime_type) {
        if let Ok(bytes) = std::fs::read(path_ref) {
            return format!(
                "data:{mime};base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            );
        }
    }

    local_file_url(path)
}

fn inline_svg_data_uri(svg: &str) -> String {
    format!(
        "data:image/svg+xml;charset=utf-8,{}",
        percent_encode_data(svg)
    )
}

fn bitmap_icon_mime_type(extension: &str) -> Option<&'static str> {
    match extension {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "ico" => Some("image/x-icon"),
        _ => None,
    }
}

fn percent_encode_path(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.');
        if keep {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn percent_encode_data(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');
        if keep {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn exit_app(
    window: &DesktopContext,
    watcher: &mut Signal<AppWatcherApi>,
    remember_window_placement: bool,
    hidden: bool,
    remembered_window_placement: Option<PersistedWindowPlacement>,
) {
    if remember_window_placement {
        let _ = if hidden {
            remembered_window_placement
                .map(|mut placement| {
                    placement.hidden = true;
                    persist_window_placement_to_sqlite(&placement)
                })
                .unwrap_or_else(|| persist_current_window_placement(window, true))
        } else {
            persist_current_window_placement(window, false)
        };
    }
    watcher.write().shutdown_embedded_watcher();
    window.set_close_behavior(WindowCloseBehaviour::WindowCloses);
    window.close();
    std::process::exit(0);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BrowserUiUrl {
    url: String,
    used_localhost_fallback: bool,
}

impl std::fmt::Display for BrowserUiUrl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.url)
    }
}

fn browser_ui_url() -> BrowserUiUrl {
    let explicit_raw_addr = std::env::var("NETSTITCH__WATCHER_ADDR")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let raw_addr = explicit_raw_addr
        .clone()
        .unwrap_or_else(default_watcher_addr);
    let (scheme, authority) = split_watcher_addr(&raw_addr);

    if explicit_raw_addr.is_some() && !authority_is_unspecified(&authority) {
        return BrowserUiUrl {
            url: format!("{scheme}://{authority}/{}", web_access_fragment()),
            used_localhost_fallback: authority_is_loopback(&authority),
        };
    }

    let Some(local_host) = local_machine_ip_for_web_url() else {
        return BrowserUiUrl {
            url: format!("{scheme}://{authority}/{}", web_access_fragment()),
            used_localhost_fallback: true,
        };
    };
    let local_authority =
        authority_with_host(&authority, &local_host).unwrap_or_else(|| authority.to_string());

    BrowserUiUrl {
        url: format!("{scheme}://{local_authority}/{}", web_access_fragment()),
        used_localhost_fallback: false,
    }
}

fn web_access_fragment() -> String {
    local_client_identifier()
        .ok()
        .and_then(|identifier| derive_web_access_key(&identifier))
        .map(|key| format!("#web_key={key}"))
        .unwrap_or_default()
}

fn web_server_event_line(prefix: &str, web_url: &BrowserUiUrl) -> String {
    format!("{prefix} {}", web_server_log_url(web_url))
}

fn split_watcher_addr(raw_addr: &str) -> (&str, String) {
    let trimmed = raw_addr.trim();
    let (scheme, rest) = if let Some(rest) = trimmed.strip_prefix("http://") {
        ("http", rest)
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        ("https", rest)
    } else {
        (default_web_scheme(), trimmed)
    };

    let authority = rest
        .split('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("127.0.0.1:46473")
        .to_string();

    (scheme, authority)
}

fn default_web_scheme() -> &'static str {
    match std::env::var(APP_ENV_WEB_SCHEME) {
        Ok(value) if value.trim().eq_ignore_ascii_case("http") => "http",
        _ => "https",
    }
}

fn authority_is_loopback(authority: &str) -> bool {
    authority.eq_ignore_ascii_case("localhost")
        || authority.starts_with("localhost:")
        || authority.starts_with("127.")
        || authority.starts_with("[::1]")
}

fn authority_is_unspecified(authority: &str) -> bool {
    authority.starts_with("0.0.0.0:") || authority.starts_with("[::]:")
}

fn authority_with_host(authority: &str, host: &str) -> Option<String> {
    let (_, port) = authority.rsplit_once(':')?;
    if port.is_empty() {
        return None;
    }
    Some(format!("{host}:{port}"))
}

fn default_watcher_addr() -> String {
    let host = local_machine_ip_for_web_url().unwrap_or_else(|| "127.0.0.1".to_string());
    format!("{host}:{DEFAULT_WATCHER_PORT}")
}

fn local_machine_ip_for_web_url() -> Option<String> {
    LOCAL_MACHINE_WEB_IP_CACHE.clone()
}

fn copy_text_to_clipboard(window: &DesktopContext, text: &str) {
    let text_json = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"(async () => {{
  const text = {text_json};
  try {{
    if (navigator.clipboard && window.isSecureContext) {{
      await navigator.clipboard.writeText(text);
      return;
    }}
  }} catch (_) {{}}
  const area = document.createElement("textarea");
  area.value = text;
  area.setAttribute("readonly", "");
  area.style.position = "fixed";
  area.style.left = "-9999px";
  document.body.appendChild(area);
  area.select();
  try {{
    document.execCommand("copy");
  }} catch (_) {{}}
  document.body.removeChild(area);
}})();"#
    );

    let _ = window.webview.evaluate_script(&script);
}

fn endpoint_probe_tooltip(
    status: &crate::watcher_api::EndpointProbeStatusDto,
    watcher_connected: bool,
    available_prefix: &str,
    unavailable: &str,
    checking: &str,
    waiting_for_watcher: &str,
    true_label: &str,
    false_label: &str,
    unknown_label: &str,
) -> String {
    let mut lines = Vec::new();
    if let Some(target) = status.first_successful_target.as_deref() {
        lines.push(format!("{available_prefix} {target}"));
    } else if !watcher_connected {
        lines.push(waiting_for_watcher.to_string());
    } else if status.is_checking {
        lines.push(checking.to_string());
    } else if let Some(detail) = endpoint_probe_failure_detail(status) {
        lines.push(format!("{unavailable}: {detail}"));
    } else {
        lines.push(unavailable.to_string());
    }

    lines.extend(status.probes.iter().map(|probe| {
        let state = match probe.available {
            Some(true) => true_label,
            Some(false) => false_label,
            None => unknown_label,
        };
        match probe
            .error
            .as_deref()
            .filter(|error| !error.trim().is_empty())
        {
            Some(error) => format!("{}: {state} ({error})", probe.target),
            None => format!("{}: {state}", probe.target),
        }
    }));

    lines.join("\n")
}

fn push_status_history_error_line(history: Signal<Vec<StatusHistoryLine>>, line: String) {
    push_status_history_typed_line(history, line, StatusHistoryKind::Error);
}

fn push_status_history_success_line(history: Signal<Vec<StatusHistoryLine>>, line: String) {
    push_status_history_typed_line(history, line, StatusHistoryKind::Success);
}

fn push_status_history_warning_line(history: Signal<Vec<StatusHistoryLine>>, line: String) {
    push_status_history_typed_line(history, line, StatusHistoryKind::Warning);
}

fn push_status_history_line(history: Signal<Vec<StatusHistoryLine>>, line: String) {
    let kind = infer_status_history_kind(&line);
    push_status_history_typed_line(history, line, kind);
}

fn push_status_history_typed_line(
    mut history: Signal<Vec<StatusHistoryLine>>,
    line: String,
    kind: StatusHistoryKind,
) {
    let normalized = normalize_log_line(line);
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return;
    }
    let mut write = history.write();
    let _ = push_status_history_entry_to_vec(&mut write, trimmed, kind);
    drop(write);
    persist_footer_message_event(trimmed.to_string(), kind);
}

#[cfg(test)]
fn push_status_history_line_to_vec(history: &mut Vec<StatusHistoryLine>, line: &str) {
    let _ = push_status_history_entry_to_vec(history, line, StatusHistoryKind::Info);
}

fn push_status_history_entry_to_vec(
    history: &mut Vec<StatusHistoryLine>,
    line: &str,
    kind: StatusHistoryKind,
) -> bool {
    let normalized = normalize_log_line(line);
    let trimmed = normalized.trim();
    if trimmed.is_empty()
        || history
            .last()
            .is_some_and(|last| last.text == trimmed && last.kind == kind)
    {
        return false;
    }

    history.push(match kind {
        StatusHistoryKind::Info => StatusHistoryLine::info(trimmed),
        StatusHistoryKind::Success => StatusHistoryLine::success(trimmed),
        StatusHistoryKind::Warning => StatusHistoryLine::warning(trimmed),
        StatusHistoryKind::Error => StatusHistoryLine::error(trimmed),
    });
    let extra = history.len().saturating_sub(10);
    if extra > 0 {
        history.drain(0..extra);
    }
    true
}

fn persist_footer_message_event(line: String, kind: StatusHistoryKind) {
    std::thread::spawn(move || {
        let _ = append_ui_message_to_sqlite(&line, kind.severity());
    });
}

fn load_footer_message_history() -> Vec<StatusHistoryLine> {
    load_footer_message_history_limit(10)
}

fn load_footer_message_history_limit(limit: usize) -> Vec<StatusHistoryLine> {
    read_ui_messages_from_sqlite(limit)
        .into_iter()
        .map(|(severity, text)| {
            let kind = StatusHistoryKind::from_severity(&severity);
            match kind {
                StatusHistoryKind::Info | StatusHistoryKind::Success => {
                    StatusHistoryLine::info(text)
                }
                StatusHistoryKind::Warning => StatusHistoryLine::warning(text),
                StatusHistoryKind::Error => StatusHistoryLine::error(text),
            }
        })
        .collect()
}

fn merge_persisted_status_history_if_empty(
    history: &mut Vec<StatusHistoryLine>,
    persisted: Vec<StatusHistoryLine>,
) -> bool {
    if !history.is_empty() || persisted.is_empty() {
        return false;
    }
    *history = persisted.into_iter().rev().take(10).collect::<Vec<_>>();
    history.reverse();
    true
}

fn infer_status_history_kind(line: &str) -> StatusHistoryKind {
    let text = line.trim().to_lowercase();
    if text.is_empty() {
        return StatusHistoryKind::Info;
    }
    if text.starts_with("{\"error\"")
        || text.contains("error code:")
        || text.contains("ошибка")
        || text.contains("error")
        || text.contains("failed")
        || text.contains("failure")
        || text.contains("недоступ")
        || text.contains("потеряно")
        || text.contains("disconnected")
        || text.contains("lost")
        || text.contains("unavailable")
        || text.contains("invalid")
        || text.contains("denied")
    {
        return StatusHistoryKind::Error;
    }
    if text.contains("треб")
        || text.contains("administrator")
        || text.contains("администратор")
        || text.contains("нет ")
        || text.contains(" no ")
        || text.contains("not ")
        || text.contains("missing")
        || text.contains("empty")
        || text.contains("не найден")
        || text.contains("не configured")
        || text.contains("localhost fallback")
        || text.contains("localhost url")
        || text.contains("выключ")
        || text.contains("disabled")
        || text.contains("останов")
        || text.contains("stopped")
        || text.contains("пропущ")
        || text.contains("skipped")
    {
        return StatusHistoryKind::Warning;
    }
    if text.contains("запущ")
        || text.contains("started")
        || text.contains("включ")
        || text.contains("enabled")
        || text.contains("готов")
        || text.contains("ready")
        || text.contains("установлено")
        || text.contains("connected")
        || text.contains("доступен")
        || text.contains("available")
        || text.contains("скопирован")
        || text.contains("copied")
        || text.contains("подтвержден")
        || text.contains("confirmed")
        || text.contains("экспортирован")
        || text.contains("exported")
        || text.contains("импортирован")
        || text.contains("imported")
        || text.contains("выполнен")
        || text.contains("completed")
        || text.contains("создан")
        || text.contains("created")
        || text.contains("примен")
        || text.contains("applied")
        || text.contains("обновлено")
        || text.contains("refreshed")
        || text.contains("signed in")
        || text.contains("registered")
    {
        return StatusHistoryKind::Success;
    }
    StatusHistoryKind::Info
}

fn normalize_log_line(line: impl AsRef<str>) -> String {
    line.as_ref().trim().trim_end_matches('.').to_string()
}

fn module_version_detail(module_key: &str, module_name: &str) -> String {
    format!("{module_name} v{}", runtime_module_version(module_key))
}

fn module_status_detail(primary: &str, module_detail: &str) -> String {
    format!("{primary}; {module_detail}")
}

fn availability_status_line(
    label: &str,
    available: bool,
    detail: &str,
    available_label: &str,
    unavailable_label: &str,
) -> String {
    format!(
        "{label}: {} ({detail})",
        if available {
            available_label
        } else {
            unavailable_label
        }
    )
}

fn with_flow_capture_status(
    base: String,
    label: &str,
    flow_events_label: &str,
    packet_events_label: &str,
    emitted_label: &str,
    dropped_label: &str,
    status: &crate::watcher_api::FlowCaptureStatusDto,
) -> String {
    let mut parts = vec![format!(
        "{label}: {flow_events_label} {}/{}, {packet_events_label} {}/{}, {emitted_label} {}, {dropped_label} {}/{}/{}",
        status.udp_flow_events,
        status.flow_events,
        status.udp_packet_events,
        status.packet_events,
        status.observations_emitted,
        status.dropped_no_owner,
        status.dropped_no_process,
        status.dropped_untracked_process
    )];
    if let Some(error) = status.backend_error.as_deref() {
        if !error.trim().is_empty() {
            parts.push(error.to_string());
        }
    }
    format!("{base}\n{}", parts.join("; "))
}

fn web_server_status_line(
    label: &str,
    enabled: bool,
    available: bool,
    detail: &str,
    available_label: &str,
    unavailable_label: &str,
    disabled_label: &str,
) -> String {
    let state = if enabled {
        if available {
            available_label
        } else {
            unavailable_label
        }
    } else {
        disabled_label
    };
    format!("{label}: {state} ({detail})")
}

fn dns_status_line(
    label: &str,
    status: &crate::watcher_api::EndpointProbeStatusDto,
    watcher_connected: bool,
    available_label: &str,
    unavailable_label: &str,
    checking_label: &str,
    waiting_for_watcher_label: &str,
    module_detail: &str,
) -> String {
    let state = if let Some(target) = status.first_successful_target.as_deref() {
        format!("{available_label} ({target}; {module_detail})")
    } else if !watcher_connected {
        format!("{waiting_for_watcher_label} ({module_detail})")
    } else if status.is_checking {
        format!("{checking_label} ({module_detail})")
    } else if let Some(detail) = endpoint_probe_failure_detail(status) {
        format!("{unavailable_label} ({detail}; {module_detail})")
    } else {
        format!("{unavailable_label} ({module_detail})")
    };
    format!("{label}: {state}")
}

fn endpoint_probe_failure_detail(
    status: &crate::watcher_api::EndpointProbeStatusDto,
) -> Option<String> {
    status.probes.iter().find_map(|probe| {
        let target = probe.target.trim();
        let error = probe
            .error
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if target.is_empty() {
            error.map(ToOwned::to_owned)
        } else if let Some(error) = error {
            Some(format!("{target}: {error}"))
        } else {
            Some(target.to_string())
        }
    })
}

fn web_server_log_url(web_server_url: &BrowserUiUrl) -> &str {
    web_server_url.url.trim_end_matches('/')
}

fn status_history_tooltip<T: FooterHistoryLine>(history: &[T]) -> String {
    if history.is_empty() {
        return String::new();
    }

    let keep = 6_usize.min(history.len());
    let hidden = history.len().saturating_sub(keep);
    let mut lines = Vec::with_capacity(keep + usize::from(hidden > 0));
    if hidden > 0 {
        lines.push(format!("...{hidden}"));
    }
    lines.extend(
        history[history.len() - keep..]
            .iter()
            .map(|line| line.footer_text().to_string()),
    );
    lines.join("\n")
}

fn footer_message_text<T: FooterHistoryLine>(history: &[T]) -> String {
    history
        .last()
        .map(|line| line.footer_text().to_string())
        .unwrap_or_default()
}

fn footer_message_tooltip<T: FooterHistoryLine>(history: &[T]) -> String {
    status_history_tooltip(history)
}

fn footer_message_copy_text<T: FooterHistoryLine>(history: &[T]) -> String {
    history
        .iter()
        .map(|line| line.footer_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn footer_message_class_suffix<T: FooterHistoryLine>(history: &[T]) -> &'static str {
    if let Some(line) = history.last() {
        return line.footer_class_suffix();
    }
    ""
}

fn connector_loaded_status_line(
    connector_apps_detected: usize,
    loaded_prefix: &str,
    loaded_empty: &str,
    connector_apps_suffix: &str,
) -> String {
    if connector_apps_detected == 0 {
        return normalize_log_line(loaded_empty);
    }
    normalize_log_line(format!(
        "{loaded_prefix} {connector_apps_detected} {connector_apps_suffix}"
    ))
}

fn tracked_app_availability_line(prefix: &str, display_name: &str, exe_path: &str) -> String {
    format!("{prefix} {display_name} - {exe_path}")
}

fn integration_module_status_key(
    module: &crate::watcher_api::IntegrationModuleRuntimeStatusDto,
) -> String {
    format!(
        "{}:{}:{}:{}",
        module.id,
        module.manifest_path,
        module.connected,
        module.error.as_deref().unwrap_or_default()
    )
}

fn integration_module_status_line(
    module: &crate::watcher_api::IntegrationModuleRuntimeStatusDto,
    connected_prefix: &str,
    failed_prefix: &str,
) -> String {
    if module.connected {
        return format!("{connected_prefix} {}", module.display_name);
    }
    let error = module
        .error
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown error");
    format!("{failed_prefix} {} - {error}", module.display_name)
}

fn shell_controls_disabled(snapshot: &crate::watcher_api::SnapshotResponse) -> bool {
    !snapshot.ui.snapshot_loaded
        && snapshot.tracked_apps.is_empty()
        && snapshot.observations.is_empty()
        && snapshot.ignored_addresses.is_empty()
}

fn effective_tracked_app_enabled(
    app: &crate::watcher_api::TrackedAppDto,
    enable_all_overlay: bool,
) -> bool {
    enable_all_overlay || app.enabled
}

fn rendered_tracked_app(
    app: &crate::watcher_api::TrackedAppDto,
    enable_all_overlay: bool,
) -> crate::watcher_api::TrackedAppDto {
    let mut rendered = app.clone();
    rendered.enabled = effective_tracked_app_enabled(app, enable_all_overlay);
    rendered
}

fn effective_enabled_tracked_apps_count(snapshot: &crate::watcher_api::SnapshotResponse) -> usize {
    snapshot
        .tracked_apps
        .iter()
        .filter(|app| effective_tracked_app_enabled(app, snapshot.app_settings.enable_all_overlay))
        .count()
}

fn tracked_path_field_size(path: &str) -> usize {
    path.chars().count().clamp(8, 120)
}

fn tracked_apps_status_text(snapshot: &SnapshotResponse) -> String {
    format!(
        "{}\\{}",
        effective_enabled_tracked_apps_count(snapshot),
        snapshot.tracked_apps.len()
    )
}

fn app_icon_label(icon_key: &str) -> &'static str {
    match icon_key {
        "chrome" => "C",
        "discord" => "D",
        "firefox" => "F",
        "telegram" => "T",
        "whatsapp" => "W",
        "yandex_browser" => "Y",
        _ => "+",
    }
}

fn icon_key_class(icon_key: &str) -> &'static str {
    match icon_key {
        "chrome" => "chrome",
        "discord" => "discord",
        "firefox" => "firefox",
        "telegram" => "telegram",
        "whatsapp" => "whatsapp",
        "yandex_browser" => "yandex",
        _ => "manual",
    }
}

fn app_name_for_observation(snapshot: &SnapshotResponse, observation: &ObservationDto) -> String {
    snapshot
        .tracked_apps
        .iter()
        .find(|app| app.id == observation.tracked_app_id)
        .map(|app| app.display_name.clone())
        .or_else(|| {
            let value = observation.process_name.trim();
            (!value.is_empty()).then(|| value.trim_end_matches(".exe").to_string())
        })
        .unwrap_or_else(|| "Imported app".to_string())
}

fn enrichment_domain_text(enrichment: &Option<crate::watcher_api::IpEnrichmentDto>) -> String {
    enrichment
        .as_ref()
        .and_then(|item| item.domain_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_default()
}

fn csv_export_domain_text(enrichment: &Option<crate::watcher_api::IpEnrichmentDto>) -> String {
    enrichment
        .as_ref()
        .filter(|item| {
            matches!(
                item.domain_source.as_deref(),
                Some(netstitch_shared::DOMAIN_SOURCE_HTTP_HOST)
                    | Some(netstitch_shared::DOMAIN_SOURCE_TLS_SNI)
                    | Some(netstitch_shared::DOMAIN_SOURCE_QUIC_SNI)
                    | Some(netstitch_shared::DOMAIN_SOURCE_CSV_IMPORT)
            )
        })
        .and_then(|item| item.domain_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_default()
}

fn connection_state_label_class(state: &crate::watcher_api::ConnectionStateDto) -> &'static str {
    if state.is_failure_like() {
        "state-label state-label--danger"
    } else if matches!(
        state,
        crate::watcher_api::ConnectionStateDto::Established
            | crate::watcher_api::ConnectionStateDto::Closing
    ) {
        "state-label state-label--success"
    } else {
        "state-label"
    }
}

fn ignored_address_domain_text(enrichment: &Option<crate::watcher_api::IpEnrichmentDto>) -> String {
    enrichment
        .as_ref()
        .and_then(|item| item.domain_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "-".to_string())
}

fn selected_csv_export_rows(snapshot: &SnapshotResponse) -> Vec<CsvExportRow> {
    snapshot
        .observations
        .iter()
        .filter(|observation| observation.is_confirmed && !observation.is_exported)
        .filter(|observation| !observation.remote_ip.trim().is_empty())
        .map(|observation| {
            let app = snapshot
                .tracked_apps
                .iter()
                .find(|app| app.id == observation.tracked_app_id);
            let connector_id = app
                .and_then(|app| app.connector_id.as_deref())
                .unwrap_or_default();
            let cloud_app_id = observation
                .cloud_app_id
                .as_deref()
                .or_else(|| {
                    netstitch_shared::cloud_app_for_connector_id(connector_id).map(|app| app.app_id)
                })
                .unwrap_or_default();

            CsvExportRow {
                application: app_name_for_observation(snapshot, observation),
                app_connector_id: connector_id.to_string(),
                cloud_app_id: cloud_app_id.to_string(),
                app_signature_key: observation
                    .app_signature_key
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
                app_signature_subject: observation
                    .app_signature_subject
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
                app_signature_issuer: observation
                    .app_signature_issuer
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
                ip: observation.remote_ip.trim().to_string(),
                domain: csv_export_domain_text(&observation.enrichment),
                port: observation.remote_port.to_string(),
                protocol: observation.protocol.as_str().to_string(),
                connection: csv_connection_value(observation),
                requests: observation.hits.to_string(),
                first_seen: observation.first_seen.clone(),
                last_seen: observation.last_seen.clone(),
            }
        })
        .collect()
}

fn render_csv_export_rows(rows: &[CsvExportRow]) -> String {
    let mut output = String::from(
        "application,app_connector_id,cloud_app_id,app_signature_key,app_signature_subject,app_signature_issuer,ip,domain,port,protocol,connection,requests,first_seen,last_seen\r\n",
    );
    for row in rows {
        output.push_str(&csv_escape(&row.application));
        output.push(',');
        output.push_str(&csv_escape(&row.app_connector_id));
        output.push(',');
        output.push_str(&csv_escape(&row.cloud_app_id));
        output.push(',');
        output.push_str(&csv_escape(&row.app_signature_key));
        output.push(',');
        output.push_str(&csv_escape(&row.app_signature_subject));
        output.push(',');
        output.push_str(&csv_escape(&row.app_signature_issuer));
        output.push(',');
        output.push_str(&csv_escape(&row.ip));
        output.push(',');
        output.push_str(&csv_escape(&row.domain));
        output.push(',');
        output.push_str(&csv_escape(&row.port));
        output.push(',');
        output.push_str(&csv_escape(&row.protocol));
        output.push(',');
        output.push_str(&csv_escape(&row.connection));
        output.push(',');
        output.push_str(&csv_escape(&row.requests));
        output.push(',');
        output.push_str(&csv_escape(&row.first_seen));
        output.push(',');
        output.push_str(&csv_escape(&row.last_seen));
        output.push_str("\r\n");
    }
    output
}

fn csv_connection_value(observation: &ObservationDto) -> String {
    format!(
        "{} ({}/{}/{})",
        observation.connection_state.as_str(),
        observation.successful_hits,
        observation.failed_hits,
        observation.hits
    )
}

fn parse_csv_import_request(content: &str) -> Result<MonitoringCsvImportRequestDto, String> {
    let records = parse_csv_records(content)?;
    let Some(header) = records.first() else {
        return Ok(MonitoringCsvImportRequestDto {
            import_source: MonitoringImportSourceDto::Csv,
            rows: Vec::new(),
        });
    };
    let mut columns = BTreeMap::<String, usize>::new();
    for (index, value) in header.iter().enumerate() {
        columns.insert(normalize_csv_header(value), index);
    }

    let mut rows = Vec::new();
    for record in records.iter().skip(1) {
        if record.iter().all(|value| value.trim().is_empty()) {
            continue;
        }
        let Some(ip) = csv_column(record, &columns, &["ip"]) else {
            continue;
        };
        let Ok(remote_ip) = ip.trim().parse() else {
            continue;
        };
        let Some(port) = csv_column(record, &columns, &["port"]) else {
            continue;
        };
        let Ok(remote_port) = port.trim().parse::<u16>() else {
            continue;
        };
        let protocol = csv_column(record, &columns, &["protocol"])
            .map(|value| SharedProtocol::from_name(value.trim()))
            .filter(|value| *value != SharedProtocol::Other)
            .unwrap_or(SharedProtocol::Tcp);
        let connection = csv_column(record, &columns, &["connection"]).unwrap_or_default();
        let requests = csv_column(record, &columns, &["requests"])
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(1);
        let metrics = parse_csv_connection_value(connection, requests);
        let domain = csv_column(record, &columns, &["domain"])
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        rows.push(MonitoringCsvImportRowDto {
            application: csv_column(record, &columns, &["application"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("CSV import")
                .to_string(),
            app_connector_id: csv_column(record, &columns, &["app_connector_id"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            cloud_app_id: csv_column(record, &columns, &["cloud_app_id"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            app_signature_key: csv_column(record, &columns, &["app_signature_key"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            app_signature_subject: csv_column(record, &columns, &["app_signature_subject"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            app_signature_issuer: csv_column(record, &columns, &["app_signature_issuer"])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            remote_ip,
            domain,
            remote_port,
            protocol,
            connection_state: metrics.0,
            first_seen_ms: csv_column(record, &columns, &["first_seen"])
                .and_then(parse_csv_timestamp_ms)
                .unwrap_or_else(current_timestamp_ms_for_ui),
            last_seen_ms: csv_column(record, &columns, &["last_seen"])
                .and_then(parse_csv_timestamp_ms)
                .unwrap_or_else(current_timestamp_ms_for_ui),
            hits: requests.max(metrics.1.saturating_add(metrics.2)).max(1),
            successful_hits: metrics.1,
            failed_hits: metrics.2,
        });
    }

    Ok(MonitoringCsvImportRequestDto {
        import_source: MonitoringImportSourceDto::Csv,
        rows,
    })
}

fn parse_csv_records(content: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::<Vec<String>>::new();
    let mut record = Vec::<String>::new();
    let mut field = String::new();
    let mut chars = content.chars().peekable();
    let mut quoted = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                record.push(std::mem::take(&mut field));
            }
            '\r' if !quoted => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            '\n' if !quoted => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            _ => field.push(ch),
        }
    }

    if quoted {
        return Err("CSV import failed: unclosed quoted field".to_string());
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    Ok(records)
}

fn csv_column<'a>(
    record: &'a [String],
    columns: &BTreeMap<String, usize>,
    aliases: &[&str],
) -> Option<&'a str> {
    aliases
        .iter()
        .filter_map(|alias| columns.get(&normalize_csv_header(alias)))
        .filter_map(|index| record.get(*index))
        .map(String::as_str)
        .next()
}

fn normalize_csv_header(value: &str) -> String {
    value
        .trim()
        .trim_matches('\u{feff}')
        .to_lowercase()
        .replace('.', "")
        .replace(':', "")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_csv_connection_value(
    value: &str,
    fallback_total: u64,
) -> (SharedConnectionState, u64, u64) {
    let trimmed = value.trim();
    let state_name = trimmed
        .split_once('(')
        .map(|(state, _)| state.trim())
        .unwrap_or(trimmed);
    let state = SharedConnectionState::from_name(state_name);
    let mut successful_hits = 0u64;
    let mut failed_hits = 0u64;
    if let Some(metrics) = trimmed
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')').map(|(metrics, _)| metrics))
    {
        let parts = metrics
            .split('/')
            .filter_map(|part| part.trim().parse::<u64>().ok())
            .collect::<Vec<_>>();
        if parts.len() >= 2 {
            successful_hits = parts[0];
            failed_hits = parts[1];
        }
    }
    if successful_hits == 0 && failed_hits == 0 {
        if state.is_failure_like() {
            failed_hits = fallback_total;
        } else if state.is_success_like() {
            successful_hits = fallback_total;
        }
    }
    (state, successful_hits, failed_hits)
}

fn parse_csv_timestamp_ms(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("n/a") {
        return None;
    }
    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|datetime| Local.from_local_datetime(&datetime).single())
        .map(|datetime| datetime.timestamp_millis().max(0) as u64)
}

fn current_timestamp_ms_for_ui() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn csv_import_status_line(
    message: &str,
    path: &std::path::Path,
    result: &MonitoringCsvImportResultDto,
) -> String {
    format!(
        "{message}: {} ({}/{} imported, {} skipped)",
        path.display(),
        result.imported_count,
        result.requested_count,
        result.skipped_count
    )
}

fn cloud_import_status_line(template: &str, result: &MonitoringCsvImportResultDto) -> String {
    template
        .replace("{imported}", &result.imported_count.to_string())
        .replace("{requested}", &result.requested_count.to_string())
        .replace("{skipped}", &result.skipped_count.to_string())
}

fn csv_escape(value: &str) -> String {
    let needs_quotes = value
        .chars()
        .any(|ch| matches!(ch, ',' | '"' | '\r' | '\n'));
    if !needs_quotes {
        return value.to_string();
    }
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[allow(clippy::too_many_arguments)]
fn ignored_address_tooltip(
    pattern: &str,
    enrichment: &Option<crate::watcher_api::IpEnrichmentDto>,
    domain_label: &str,
    owner_label: &str,
    range_label: &str,
    registry_label: &str,
    source_label: &str,
    unknown_label: &str,
    localhost_rule_label: &str,
    local_ip_rule_label: &str,
) -> String {
    let mut lines = Vec::new();
    if ignored_rule_is_loopback(pattern) {
        lines.push(localhost_rule_label.to_string());
    } else if ignored_rule_is_local_machine_candidate(pattern) {
        lines.push(local_ip_rule_label.to_string());
    }

    let enrichment_tooltip = ip_enrichment_tooltip(
        enrichment,
        domain_label,
        owner_label,
        range_label,
        registry_label,
        source_label,
        unknown_label,
    );
    if !enrichment_tooltip.is_empty() {
        lines.push(enrichment_tooltip);
    }

    lines.join("\n")
}

fn ip_enrichment_tooltip(
    enrichment: &Option<crate::watcher_api::IpEnrichmentDto>,
    domain_label: &str,
    owner_label: &str,
    range_label: &str,
    registry_label: &str,
    source_label: &str,
    unknown_label: &str,
) -> String {
    let Some(enrichment) = enrichment.as_ref() else {
        return String::new();
    };

    let lines = [
        (
            domain_label,
            enrichment.domain_name.as_deref().unwrap_or(unknown_label),
        ),
        (
            owner_label,
            enrichment.owner_name.as_deref().unwrap_or(unknown_label),
        ),
        (
            range_label,
            enrichment.owner_range.as_deref().unwrap_or(unknown_label),
        ),
        (
            registry_label,
            enrichment.registry.as_deref().unwrap_or(unknown_label),
        ),
        (
            source_label,
            enrichment.source.as_deref().unwrap_or(unknown_label),
        ),
    ];
    lines
        .into_iter()
        .map(|(label, value)| format!("{label}: {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg_attr(not(test), allow(dead_code))]
fn filter_observations(snapshot: &SnapshotResponse) -> Vec<ObservationDto> {
    filter_observations_with_header_filters(snapshot, &HeaderObservationFilters::default())
}

fn shared_filters_from_snapshot(snapshot: &SnapshotResponse) -> SharedUiFiltersDto {
    SharedUiFiltersDto {
        app_search: snapshot.filters.app_search.clone(),
        ip_search: snapshot.filters.search_text.clone(),
        domain_search: snapshot.filters.domain_search.clone(),
        port_search: snapshot.filters.port_search.clone(),
        protocol: snapshot.filters.protocol.clone(),
        public_ip: snapshot.filters.public_ip,
        observation_filter: match snapshot.filters.observation_filter {
            ObservationFilterDto::All => SharedUiObservationFilterDto::All,
            ObservationFilterDto::Unconfirmed => SharedUiObservationFilterDto::Unconfirmed,
            ObservationFilterDto::Confirmed => SharedUiObservationFilterDto::Confirmed,
            ObservationFilterDto::Success => SharedUiObservationFilterDto::Success,
            ObservationFilterDto::Exported => SharedUiObservationFilterDto::Exported,
            ObservationFilterDto::Failed => SharedUiObservationFilterDto::Failed,
        },
    }
}

fn filter_observations_with_header_filters(
    snapshot: &SnapshotResponse,
    header_filters: &HeaderObservationFilters,
) -> Vec<ObservationDto> {
    filter_observations_internal(snapshot, header_filters, true)
}

fn sort_observations(
    snapshot: &SnapshotResponse,
    mut observations: Vec<ObservationDto>,
    sort_state: ObservationSortState,
) -> Vec<ObservationDto> {
    let column = sort_state.column.unwrap_or(ObservationSortColumn::LastSeen);

    observations.sort_by(|left, right| {
        let ordering = match column {
            ObservationSortColumn::App => app_name_for_observation(snapshot, left)
                .cmp(&app_name_for_observation(snapshot, right)),
            ObservationSortColumn::Ip => left.remote_ip.cmp(&right.remote_ip),
            ObservationSortColumn::Domain => enrichment_domain_text(&left.enrichment)
                .cmp(&enrichment_domain_text(&right.enrichment)),
            ObservationSortColumn::Port => left.remote_port.cmp(&right.remote_port),
            ObservationSortColumn::Protocol => left.protocol.as_str().cmp(right.protocol.as_str()),
            ObservationSortColumn::Connection => left
                .connection_state
                .as_str()
                .cmp(right.connection_state.as_str()),
            ObservationSortColumn::FirstSeen => left.first_seen.cmp(&right.first_seen),
            ObservationSortColumn::LastSeen => left.last_seen.cmp(&right.last_seen),
            ObservationSortColumn::Hits => left.hits.cmp(&right.hits),
        };
        let ordering = if ordering == std::cmp::Ordering::Equal {
            left.id.cmp(&right.id)
        } else {
            ordering
        };
        if sort_state.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    observations
}

fn cloud_apps_for_visibility_scope(
    catalog_apps: Vec<CloudCatalogApp>,
    my_apps: &[CloudUserAppSummary],
    scope: CloudObservationVisibilityScope,
    app_query: &str,
    publisher_query: &str,
    source_query: &str,
    own_author_display: &str,
    own_scope: bool,
) -> Vec<CloudCatalogApp> {
    let app_query = app_query.trim().to_lowercase();
    let publisher_query = publisher_query.trim().to_lowercase();
    let source_display = source_query.trim().to_string();
    let source_query = source_display.to_lowercase();
    if !own_scope
        && app_query.chars().count() < 2
        && publisher_query.chars().count() < 2
        && source_query.chars().count() < 2
    {
        return Vec::new();
    }

    if scope == CloudObservationVisibilityScope::Public && !own_scope {
        return catalog_apps;
    }

    let mut apps_by_id = if !own_scope
        && (scope == CloudObservationVisibilityScope::All
            || scope == CloudObservationVisibilityScope::Public)
    {
        catalog_apps
            .into_iter()
            .map(|app| (app.app_id.clone(), app))
            .collect::<BTreeMap<_, _>>()
    } else {
        BTreeMap::new()
    };
    for app in my_apps.iter().filter(|app| {
        let include_for_scope = match scope {
            CloudObservationVisibilityScope::All => true,
            CloudObservationVisibilityScope::Public => {
                app.visibility == CloudObservationVisibility::Public
            }
            CloudObservationVisibilityScope::Private => {
                app.visibility == CloudObservationVisibility::Private
            }
        };
        include_for_scope && (own_scope || app.visibility == CloudObservationVisibility::Private)
    }) {
        let entry = apps_by_id
            .entry(app.app_id.clone())
            .or_insert_with(|| CloudCatalogApp {
                app_id: app.app_id.clone(),
                display_name: app.display_name.clone(),
                publisher_name: app.publisher_name.clone().unwrap_or_default(),
                authors: app
                    .author_signature
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .or_else(|| {
                        let own_author_display = own_author_display.trim();
                        (!own_author_display.is_empty()).then_some(own_author_display)
                    })
                    .or_else(|| (!source_display.is_empty()).then_some(source_display.as_str()))
                    .map(|value| vec![value.to_string()])
                    .unwrap_or_default(),
                endpoint_count: 0,
                available_row_count: 0,
                author_count: 0,
                last_seen_ms: app.last_seen_ms.or(app.last_uploaded_at_ms),
            });
        if let Some(author) = app
            .author_signature
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let own_author_display = own_author_display.trim();
                (!own_author_display.is_empty()).then_some(own_author_display)
            })
        {
            if !entry
                .authors
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(author))
            {
                entry.authors.push(author.to_string());
            }
        }
        entry.endpoint_count = entry.endpoint_count.saturating_add(app.endpoint_count);
        entry.available_row_count = entry
            .available_row_count
            .saturating_add(app.available_row_count.max(app.endpoint_count));
        entry.last_seen_ms = entry
            .last_seen_ms
            .max(app.last_seen_ms.or(app.last_uploaded_at_ms));
    }
    apps_by_id
        .into_values()
        .filter(|app| {
            (app_query.chars().count() < 2 || app.display_name.to_lowercase().contains(&app_query))
                && (publisher_query.chars().count() < 2
                    || app.publisher_name.to_lowercase().contains(&publisher_query))
                && (source_query.chars().count() < 2
                    || cloud_app_authors_label(app)
                        .to_lowercase()
                        .contains(&source_query))
        })
        .collect()
}

fn sort_cloud_apps(
    mut apps: Vec<CloudCatalogApp>,
    sort_state: CloudAppSortState,
) -> Vec<CloudCatalogApp> {
    let Some(column) = sort_state.column else {
        return apps;
    };

    apps.sort_by(|left, right| {
        let ordering = match column {
            CloudAppSortColumn::App => left
                .display_name
                .to_lowercase()
                .cmp(&right.display_name.to_lowercase()),
            CloudAppSortColumn::Company => left
                .publisher_name
                .to_lowercase()
                .cmp(&right.publisher_name.to_lowercase()),
            CloudAppSortColumn::AvailableRows => {
                cloud_app_available_row_count(left).cmp(&cloud_app_available_row_count(right))
            }
            CloudAppSortColumn::Authors => cloud_app_authors_sort_key(left)
                .cmp(&cloud_app_authors_sort_key(right))
                .then_with(|| left.author_count.cmp(&right.author_count)),
        };
        let ordering = ordering
            .then_with(|| {
                left.display_name
                    .to_lowercase()
                    .cmp(&right.display_name.to_lowercase())
            })
            .then_with(|| left.app_id.cmp(&right.app_id));
        if sort_state.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    apps
}

fn cloud_app_authors_label(app: &CloudCatalogApp) -> String {
    if app.authors.is_empty() {
        "-".to_string()
    } else {
        app.authors.join(", ")
    }
}

fn cloud_app_available_row_count(app: &CloudCatalogApp) -> u64 {
    if app.available_row_count > 0 {
        app.available_row_count
    } else {
        app.endpoint_count
    }
}

fn cloud_app_authors_sort_key(app: &CloudCatalogApp) -> String {
    app.authors
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn filter_cloud_downloaded_rows(
    rows: Vec<CloudDownloadedObservation>,
    app_query: &str,
    ip_query: &str,
    domain_query: &str,
    port_query: &str,
    protocol_filter: &str,
    status_filter: ObservationFilterDto,
    selected_row_ids: &std::collections::BTreeSet<String>,
) -> Vec<CloudDownloadedObservation> {
    let app_query = app_query.trim().to_lowercase();
    let ip_query = ip_query.trim().to_lowercase();
    let domain_query = domain_query.trim().to_lowercase();
    let port_query = port_query.trim().to_lowercase();
    let protocol_filter = protocol_filter.trim().to_lowercase();

    rows.into_iter()
        .filter(|item| {
            let selected = selected_row_ids.contains(&item.row_id);
            let status_matches = match status_filter {
                ObservationFilterDto::All | ObservationFilterDto::Exported => true,
                ObservationFilterDto::Unconfirmed => !selected,
                ObservationFilterDto::Confirmed => selected,
                ObservationFilterDto::Success => {
                    item.row.successful_hits > 0 || item.row.connection_state.is_success_like()
                }
                ObservationFilterDto::Failed => {
                    item.row.failed_hits > 0 || item.row.connection_state.is_failure_like()
                }
            };

            status_matches
                && (app_query.is_empty()
                    || item.app_display_name.to_lowercase().contains(&app_query))
                && (ip_query.is_empty()
                    || item
                        .row
                        .remote_ip
                        .to_string()
                        .to_lowercase()
                        .contains(&ip_query))
                && (domain_query.is_empty()
                    || item.domain_label.to_lowercase().contains(&domain_query))
                && (port_query.is_empty()
                    || item
                        .row
                        .remote_port
                        .to_string()
                        .to_lowercase()
                        .contains(&port_query))
                && (protocol_filter.is_empty()
                    || protocol_filter == "all"
                    || item.protocol_label.to_lowercase() == protocol_filter)
        })
        .collect()
}

fn sort_cloud_downloaded_rows(
    mut rows: Vec<CloudDownloadedObservation>,
    sort_state: CloudRowSortState,
) -> Vec<CloudDownloadedObservation> {
    let Some(column) = sort_state.column else {
        return rows;
    };

    rows.sort_by(|left, right| {
        let ordering = match column {
            CloudRowSortColumn::App => left
                .app_display_name
                .to_lowercase()
                .cmp(&right.app_display_name.to_lowercase()),
            CloudRowSortColumn::Source => left
                .source_label
                .to_lowercase()
                .cmp(&right.source_label.to_lowercase()),
            CloudRowSortColumn::Ip => left.row.remote_ip.cmp(&right.row.remote_ip),
            CloudRowSortColumn::Domain => left
                .domain_label
                .to_lowercase()
                .cmp(&right.domain_label.to_lowercase()),
            CloudRowSortColumn::Port => left.row.remote_port.cmp(&right.row.remote_port),
            CloudRowSortColumn::Protocol => left
                .protocol_label
                .to_lowercase()
                .cmp(&right.protocol_label.to_lowercase()),
            CloudRowSortColumn::Connection => left
                .connection_label
                .to_lowercase()
                .cmp(&right.connection_label.to_lowercase()),
            CloudRowSortColumn::Hits => left.row.requests.cmp(&right.row.requests),
        };
        let ordering = ordering.then_with(|| left.row_id.cmp(&right.row_id));
        if sort_state.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    rows
}

fn filter_observations_internal(
    snapshot: &SnapshotResponse,
    header_filters: &HeaderObservationFilters,
    apply_port_filter: bool,
) -> Vec<ObservationDto> {
    let search = snapshot.filters.search_text.trim().to_lowercase();
    let app_search = header_filters.app_search.trim().to_lowercase();
    let domain_search = header_filters.domain_search.trim().to_lowercase();
    let port_search = header_filters.port_search.trim();
    let protocol = header_filters.protocol.trim().to_ascii_uppercase();

    snapshot
        .observations
        .iter()
        .filter(|observation| {
            let is_ignored_address = snapshot.ignored_addresses.iter().any(|rule| {
                address_matches_ignore_rule(&observation.remote_ip, &rule.address_pattern)
            });
            let matches_ignore_list = !is_ignored_address;

            let matches_filter = match snapshot.filters.observation_filter {
                ObservationFilterDto::All => true,
                ObservationFilterDto::Unconfirmed => {
                    !observation.is_confirmed && !observation.is_exported
                }
                ObservationFilterDto::Confirmed => {
                    observation.is_confirmed && !observation.is_exported
                }
                ObservationFilterDto::Success => {
                    observation.successful_hits > 0
                        || matches!(
                            observation.connection_state,
                            crate::watcher_api::ConnectionStateDto::Established
                                | crate::watcher_api::ConnectionStateDto::Closing
                        )
                }
                ObservationFilterDto::Exported => observation.is_exported,
                ObservationFilterDto::Failed => {
                    observation.failed_hits > 0 || observation.connection_state.is_failure_like()
                }
            };

            let matches_search = if search.is_empty() {
                true
            } else {
                observation.remote_ip.contains(&search)
            };
            let matches_app_search = app_search.is_empty()
                || app_name_for_observation(snapshot, observation)
                    .to_lowercase()
                    .contains(&app_search)
                || observation
                    .process_name
                    .to_lowercase()
                    .contains(&app_search);
            let matches_port = !apply_port_filter
                || port_search.is_empty()
                || observation.remote_port.to_string().contains(port_search);
            let matches_domain = domain_filter_matches(
                &enrichment_domain_text(&observation.enrichment),
                &domain_search,
            );
            let matches_protocol = protocol.is_empty()
                || protocol == "ALL"
                || observation
                    .protocol
                    .as_str()
                    .eq_ignore_ascii_case(&protocol);
            let matches_public = observation_ip_is_public(observation) == header_filters.public_ip;

            matches_ignore_list
                && matches_filter
                && matches_search
                && matches_app_search
                && matches_domain
                && matches_port
                && matches_protocol
                && matches_public
        })
        .cloned()
        .collect()
}

fn domain_filter_matches(value: &str, pattern: &str) -> bool {
    let value = value.trim().to_lowercase();
    let pattern = pattern.trim().to_lowercase();
    if pattern.is_empty() {
        return true;
    }
    if !pattern.contains('*') {
        return value.contains(&pattern);
    }
    wildcard_pattern_matches(&value, &pattern)
}

fn wildcard_pattern_matches(value: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');
    let mut position = 0usize;

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 && anchored_start {
            if !value[position..].starts_with(part) {
                return false;
            }
            position += part.len();
            continue;
        }
        let Some(found) = value[position..].find(part) else {
            return false;
        };
        position += found + part.len();
    }

    if anchored_end {
        if let Some(last) = parts.iter().rev().find(|part| !part.is_empty()) {
            return value.ends_with(last);
        }
    }

    true
}

fn observation_ip_is_public(observation: &ObservationDto) -> bool {
    observation
        .remote_ip
        .parse::<IpAddr>()
        .map(cloud_observation_ip_is_public)
        .unwrap_or(false)
}

#[cfg_attr(not(test), allow(dead_code))]
fn language_is_russian(language_code: &str) -> bool {
    language_code.trim().to_ascii_lowercase().starts_with("ru")
}

fn tracked_app_filter_options(snapshot: &SnapshotResponse) -> Vec<String> {
    snapshot
        .tracked_apps
        .iter()
        .filter_map(|app| {
            let display_name = app.display_name.trim();
            (!display_name.is_empty()).then_some(display_name.to_string())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn address_matches_ignore_rule(address: &str, pattern: &str) -> bool {
    let Ok(address) = address.parse::<std::net::IpAddr>() else {
        return address.eq_ignore_ascii_case(pattern.trim());
    };
    let pattern = pattern.trim();
    if let Some((network, prefix)) = pattern.split_once('/') {
        let Ok(network) = network.trim().parse::<std::net::IpAddr>() else {
            return false;
        };
        let Ok(prefix) = prefix.trim().parse::<u8>() else {
            return false;
        };
        return ip_in_cidr(address, network, prefix);
    }

    pattern
        .parse::<std::net::IpAddr>()
        .is_ok_and(|ignored| ignored == address)
}

fn ip_in_cidr(address: std::net::IpAddr, network: std::net::IpAddr, prefix: u8) -> bool {
    match (address, network) {
        (std::net::IpAddr::V4(address), std::net::IpAddr::V4(network)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(address) & mask) == (u32::from(network) & mask)
        }
        (std::net::IpAddr::V6(address), std::net::IpAddr::V6(network)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            (u128::from(address) & mask) == (u128::from(network) & mask)
        }
        _ => false,
    }
}

fn ignored_rule_is_loopback(pattern: &str) -> bool {
    ignored_rule_ip_and_prefix(pattern).is_some_and(|(ip, prefix)| match ip {
        std::net::IpAddr::V4(address) => {
            address.is_loopback() && prefix.is_none_or(|prefix| prefix <= 32)
        }
        std::net::IpAddr::V6(address) => {
            address.is_loopback() && prefix.is_none_or(|prefix| prefix <= 128)
        }
    })
}

fn ignored_rule_is_local_machine_candidate(pattern: &str) -> bool {
    let local_ip =
        local_machine_ip_for_web_url().and_then(|value| value.parse::<std::net::IpAddr>().ok());
    ignored_rule_matches_local_machine_ip(pattern, local_ip)
}

fn ignored_rule_matches_local_machine_ip(
    pattern: &str,
    local_ip: Option<std::net::IpAddr>,
) -> bool {
    let Some(local_ip) = local_ip else {
        return false;
    };
    let Some((ip, prefix)) = ignored_rule_ip_and_prefix(pattern) else {
        return false;
    };
    if ip.is_loopback() || ip.is_unspecified() {
        return false;
    }
    let is_exact_host_rule = match ip {
        std::net::IpAddr::V4(_) => prefix.is_none_or(|prefix| prefix == 32),
        std::net::IpAddr::V6(_) => prefix.is_none_or(|prefix| prefix == 128),
    };
    is_exact_host_rule && ip == local_ip
}

fn ignored_rule_ip_and_prefix(pattern: &str) -> Option<(std::net::IpAddr, Option<u8>)> {
    let pattern = pattern.trim();
    if let Some((network, prefix)) = pattern.split_once('/') {
        let Ok(ip) = network.trim().parse::<std::net::IpAddr>() else {
            return None;
        };
        let Ok(prefix) = prefix.trim().parse::<u8>() else {
            return None;
        };
        return Some((ip, Some(prefix)));
    }
    pattern
        .parse::<std::net::IpAddr>()
        .ok()
        .map(|ip| (ip, None))
}

#[cfg(test)]
mod tests {
    use super::{
        BrowserUiUrl, CLOSE_TIMES_ICON_SVG, CloudCatalogApp, CloudDownloadedObservation,
        CloudPublicationSortState, CloudUserAppSummary, CsvExportRow, HeaderObservationFilters,
        IntegrationDownloadUiState, IntegrationProgressLabels, IntegrationUiEntityDto,
        OBSERVATION_SELECTION_CONFIRM_DEBOUNCE_MS, ObservationSelectionStore,
        ObservationSortColumn, ObservationSortState, PendingObservationSelectionConfirm,
        ProgressStage, StatusHistoryLine, build_cloud_author_publication_rows,
        clone_observation_selection_store, cloud_app_authors_label, cloud_app_available_row_count,
        cloud_apps_for_visibility_scope, cloud_download_selection_batches,
        cloud_import_rows_for_add_to_monitoring, cloud_progress_stages, compare_version_text,
        compute_icon_image_src, connector_loaded_status_line, csv_escape, dns_status_line,
        domain_filter_matches, drain_ready_observation_selection_confirm,
        effective_enabled_tracked_apps_count, effective_tracked_app_enabled, extract_version_text,
        filter_observations, filter_observations_with_header_filters, footer_message_copy_text,
        footer_message_text, footer_message_tooltip, icon_image_src, ignored_address_domain_text,
        ignored_address_tooltip, ignored_rule_is_local_machine_candidate, ignored_rule_is_loopback,
        ignored_rule_matches_local_machine_ip, inline_svg_data_uri, integration_dialog_preview,
        integration_progress_footer_line, language_is_russian, mark_uploaded_public_observations,
        merge_adjacent_progress_stages, merge_manual_domains, module_ui_active_tab_children,
        module_ui_parse_progress_percent, module_ui_progress_stages, module_ui_schema_with_context,
        normalize_progress_stages, observation_selection_batches, parse_csv_import_request,
        paths_match_for_duplicate_check, progress_current_stage_label, progress_current_text,
        push_status_history_line_to_vec, queue_observation_selection_confirm,
        render_csv_export_rows, reset_cloud_download_staging_for_download,
        selected_csv_export_rows, selected_profile_export_domains, shell_controls_disabled,
        snapshot_ui_render_relevant_changed, sort_observations, split_manual_domains,
        status_history_tooltip, sync_observation_selection_store, tracked_app_availability_line,
        tracked_path_field_size, web_server_event_line,
    };
    use crate::cloud_sync::CloudSyncUiState;
    use crate::watcher_api::{
        AppSettingsDto, ConnectionStateDto, FiltersDto, IgnoredAddressDto,
        IntegrationIntegrationDto, IpEnrichmentDto, ObservationDto, ObservationFilterDto,
        ProtocolDto, RuntimeStatusDto, SnapshotResponse, TrackedAppDto, UiStatusDto,
    };
    use netstitch_shared::models::{
        ConnectionState as SharedConnectionState, Protocol as SharedProtocol,
    };
    use netstitch_shared::{
        CloudDomainStatus, CloudObservationRow, CloudObservationVisibility,
        CloudObservationVisibilityScope, CloudSourceKind, CloudTrustLevel,
    };
    use std::{
        collections::BTreeSet,
        fs,
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };

    #[test]
    fn manual_domain_input_uses_one_domain_per_line() {
        assert_eq!(
            split_manual_domains("example.com\n cdn.example.net \n\n"),
            vec!["example.com".to_string(), "cdn.example.net".to_string()]
        );
        assert_eq!(
            split_manual_domains("example.com, cdn.example.net"),
            vec!["example.com, cdn.example.net".to_string()]
        );
    }

    #[test]
    fn selected_profile_export_domains_are_added_without_duplicates() {
        let mut selected = observation_with_ip(1, "203.0.113.10");
        selected.is_confirmed = true;
        selected.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("launcher.example.test".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_TLS_SNI.to_string()),
            ..IpEnrichmentDto::default()
        });
        let mut duplicate = observation_with_ip(2, "203.0.113.11");
        duplicate.is_confirmed = true;
        duplicate.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("Launcher.Example.Test".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_QUIC_SNI.to_string()),
            ..IpEnrichmentDto::default()
        });
        let mut unproven = observation_with_ip(3, "203.0.113.12");
        unproven.is_confirmed = true;
        unproven.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("owner.example.test".to_string()),
            domain_source: None,
            ..IpEnrichmentDto::default()
        });
        let mut exported = observation_with_ip(4, "203.0.113.13");
        exported.is_confirmed = true;
        exported.is_exported = true;
        exported.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("exported.example.test".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_HTTP_HOST.to_string()),
            ..IpEnrichmentDto::default()
        });

        let domains = selected_profile_export_domains(&snapshot_with_observations(vec![
            selected, duplicate, unproven, exported,
        ]));

        assert_eq!(domains, vec!["launcher.example.test".to_string()]);
        assert_eq!(
            merge_manual_domains("manual.example.test\nlauncher.example.test", &domains),
            "manual.example.test\nlauncher.example.test"
        );
    }

    #[test]
    fn cloud_author_publications_skip_rows_without_cloud_identity() {
        let mut uploadable = observation_with_ip(1, "8.8.8.8");
        uploadable.is_confirmed = true;
        uploadable.cloud_app_id = Some("netstitch.app.demo".to_string());
        let mut missing_identity = observation_with_ip(2, "8.8.4.4");
        missing_identity.is_confirmed = true;
        missing_identity.tracked_app_id = 0;
        missing_identity.process_name = String::new();
        let selected_ids = BTreeSet::from([uploadable.id, missing_identity.id]);

        let rows = build_cloud_author_publication_rows(
            &snapshot_with_observations(vec![uploadable, missing_identity]),
            &[],
            &selected_ids,
            &BTreeSet::new(),
            CloudObservationVisibility::Public,
            CloudPublicationSortState::default(),
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].new_rows, 1);
        assert_eq!(rows[0].app_key, "netstitch.app.demo");
    }

    #[test]
    fn cloud_author_publications_show_non_public_rows_separately() {
        let mut public = observation_with_ip(1, "8.8.8.8");
        public.is_confirmed = true;
        public.cloud_app_id = Some("netstitch.app.demo".to_string());
        let mut private = observation_with_ip(2, "192.168.1.20");
        private.is_confirmed = true;
        private.cloud_app_id = Some("netstitch.app.demo".to_string());
        let selected_ids = BTreeSet::from([public.id, private.id]);

        let rows = build_cloud_author_publication_rows(
            &snapshot_with_observations(vec![public, private]),
            &[],
            &selected_ids,
            &BTreeSet::new(),
            CloudObservationVisibility::Public,
            CloudPublicationSortState::default(),
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].new_rows, 1);
        assert_eq!(rows[0].non_public_rows, 1);
        assert_eq!(rows[0].app_key, "netstitch.app.demo");
    }

    #[test]
    fn cloud_author_publications_hide_uploaded_rows_after_success() {
        let mut observation = observation_with_ip(1, "8.8.8.8");
        observation.is_confirmed = true;
        observation.cloud_app_id = Some("netstitch.app.demo".to_string());
        let selected_ids = BTreeSet::from([observation.id]);
        let snapshot = snapshot_with_observations(vec![observation]);
        let mut state = CloudSyncUiState::default();

        mark_uploaded_public_observations(&mut state, &snapshot);
        let rows = build_cloud_author_publication_rows(
            &snapshot,
            &[CloudUserAppSummary {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Demo Game".to_string(),
                publisher_name: None,
                author_signature: Some("Shin0by".to_string()),
                visibility: CloudObservationVisibility::Public,
                endpoint_count: 1,
                total_endpoint_count: 1,
                available_row_count: 1,
                last_seen_ms: None,
                last_uploaded_at_ms: None,
            }],
            &selected_ids,
            &state.uploaded_observation_ids,
            CloudObservationVisibility::Public,
            CloudPublicationSortState::default(),
        );

        assert_eq!(state.uploaded_observation_ids, selected_ids);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].new_rows, 0);
        assert_eq!(rows[0].author_rows, 1);
    }

    #[test]
    fn cloud_author_publications_merge_new_rows_with_existing_upload_app_id() {
        let mut orphan = observation_with_ip(1, "8.8.8.8");
        orphan.is_confirmed = true;
        orphan.tracked_app_id = 0;
        orphan.process_name = "Demo App".to_string();
        let selected_ids = BTreeSet::from([orphan.id]);
        let mut snapshot = snapshot_with_observations(vec![orphan]);
        snapshot.tracked_apps.push(TrackedAppDto {
            id: 42,
            connector_id: None,
            cloud_app_id: Some("netstitch.app.demo".to_string()),
            display_name: "Demo".to_string(),
            icon_key: String::new(),
            icon_path: None,
            exe_path: r"C:\Games\DemoApp.exe".to_string(),
            enabled: false,
            created_at: String::new(),
        });

        let rows = build_cloud_author_publication_rows(
            &snapshot,
            &[CloudUserAppSummary {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Cloud Demo".to_string(),
                publisher_name: None,
                author_signature: Some("Shin0by".to_string()),
                visibility: CloudObservationVisibility::Public,
                endpoint_count: 600,
                total_endpoint_count: 600,
                available_row_count: 600,
                last_seen_ms: None,
                last_uploaded_at_ms: None,
            }],
            &selected_ids,
            &BTreeSet::new(),
            CloudObservationVisibility::Public,
            CloudPublicationSortState::default(),
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].new_rows, 1);
        assert_eq!(rows[0].author_rows, 600);
        assert_eq!(rows[0].app_name, "Cloud Demo");
        assert_eq!(rows[0].app_key, "netstitch.app.demo");
    }

    #[test]
    fn cloud_author_publications_use_cloud_name_for_private_new_rows() {
        let mut observation = observation_with_ip(1, "8.8.8.8");
        observation.is_confirmed = true;
        observation.tracked_app_id = 42;
        let selected_ids = BTreeSet::from([observation.id]);
        let mut snapshot = snapshot_with_observations(vec![observation]);
        snapshot.tracked_apps.push(TrackedAppDto {
            id: 42,
            connector_id: None,
            cloud_app_id: Some("netstitch.app.demo".to_string()),
            display_name: "DG".to_string(),
            icon_key: String::new(),
            icon_path: None,
            exe_path: r"C:\Games\DemoGame.exe".to_string(),
            enabled: false,
            created_at: String::new(),
        });

        let rows = build_cloud_author_publication_rows(
            &snapshot,
            &[CloudUserAppSummary {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Demo Game".to_string(),
                publisher_name: None,
                author_signature: Some("Shin0by".to_string()),
                visibility: CloudObservationVisibility::Public,
                endpoint_count: 600,
                total_endpoint_count: 600,
                available_row_count: 600,
                last_seen_ms: None,
                last_uploaded_at_ms: None,
            }],
            &selected_ids,
            &BTreeSet::new(),
            CloudObservationVisibility::Private,
            CloudPublicationSortState::default(),
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].app_name, "Demo Game");
        assert_eq!(rows[0].app_key, "netstitch.app.demo");
        assert_eq!(rows[0].author_rows, 600);
        assert_eq!(rows[0].new_rows, 1);
    }

    #[test]
    fn cloud_author_publications_merge_public_and_private_author_rows() {
        let rows = build_cloud_author_publication_rows(
            &snapshot_with_observations(Vec::new()),
            &[
                CloudUserAppSummary {
                    app_id: "netstitch.app.demo".to_string(),
                    display_name: "Demo Game".to_string(),
                    publisher_name: None,
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Public,
                    endpoint_count: 600,
                    total_endpoint_count: 600,
                    available_row_count: 600,
                    last_seen_ms: None,
                    last_uploaded_at_ms: None,
                },
                CloudUserAppSummary {
                    app_id: " NETSTITCH.APP.DEMO ".to_string(),
                    display_name: "DG".to_string(),
                    publisher_name: None,
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Private,
                    endpoint_count: 2,
                    total_endpoint_count: 2,
                    available_row_count: 2,
                    last_seen_ms: None,
                    last_uploaded_at_ms: None,
                },
            ],
            &BTreeSet::new(),
            &BTreeSet::new(),
            CloudObservationVisibility::Private,
            CloudPublicationSortState::default(),
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].app_name, "Demo Game");
        assert_eq!(rows[0].author_rows, 602);
        assert_eq!(rows[0].total_rows_label, "602");
    }

    #[test]
    fn cloud_app_list_stays_empty_until_a_search_filter_is_applied() {
        let apps = cloud_apps_for_visibility_scope(
            Vec::new(),
            &[CloudUserAppSummary {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Demo Game".to_string(),
                publisher_name: Some("Demo Publisher".to_string()),
                author_signature: Some("Shin0by".to_string()),
                visibility: CloudObservationVisibility::Private,
                endpoint_count: 2,
                total_endpoint_count: 2,
                available_row_count: 2,
                last_seen_ms: None,
                last_uploaded_at_ms: None,
            }],
            CloudObservationVisibilityScope::All,
            "",
            "",
            "",
            "",
            false,
        );

        assert!(
            apps.is_empty(),
            "author-owned private apps must not appear as a default app list without filters"
        );
    }

    #[test]
    fn cloud_app_list_all_scope_adds_private_rows_to_public_catalog() {
        let apps = cloud_apps_for_visibility_scope(
            vec![CloudCatalogApp {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Demo Game".to_string(),
                publisher_name: "Demo Publisher".to_string(),
                authors: vec!["Shin0by".to_string()],
                endpoint_count: 4,
                available_row_count: 4,
                author_count: 1,
                last_seen_ms: Some(10),
            }],
            &[
                CloudUserAppSummary {
                    app_id: "netstitch.app.demo".to_string(),
                    display_name: "Demo Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Public,
                    endpoint_count: 4,
                    total_endpoint_count: 4,
                    available_row_count: 4,
                    last_seen_ms: Some(10),
                    last_uploaded_at_ms: None,
                },
                CloudUserAppSummary {
                    app_id: "netstitch.app.demo".to_string(),
                    display_name: "Demo Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Private,
                    endpoint_count: 2,
                    total_endpoint_count: 2,
                    available_row_count: 2,
                    last_seen_ms: Some(20),
                    last_uploaded_at_ms: None,
                },
            ],
            CloudObservationVisibilityScope::All,
            "demo",
            "",
            "",
            "",
            false,
        );

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].app_id, "netstitch.app.demo");
        assert_eq!(cloud_app_available_row_count(&apps[0]), 6);
    }

    #[test]
    fn cloud_app_list_own_scope_shows_user_public_and_private_without_text_filter() {
        let apps = cloud_apps_for_visibility_scope(
            Vec::new(),
            &[
                CloudUserAppSummary {
                    app_id: "netstitch.app.public".to_string(),
                    display_name: "Public Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Public,
                    endpoint_count: 4,
                    total_endpoint_count: 4,
                    available_row_count: 4,
                    last_seen_ms: Some(10),
                    last_uploaded_at_ms: None,
                },
                CloudUserAppSummary {
                    app_id: "netstitch.app.private".to_string(),
                    display_name: "Private Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Private,
                    endpoint_count: 2,
                    total_endpoint_count: 2,
                    available_row_count: 2,
                    last_seen_ms: Some(20),
                    last_uploaded_at_ms: None,
                },
            ],
            CloudObservationVisibilityScope::All,
            "",
            "",
            "",
            "",
            true,
        );

        assert_eq!(apps.len(), 2);
        assert_eq!(cloud_app_available_row_count(&apps[0]), 2);
        assert_eq!(cloud_app_available_row_count(&apps[1]), 4);
        assert!(
            apps.iter()
                .all(|app| cloud_app_authors_label(app) == "Shin0by")
        );
    }

    #[test]
    fn cloud_app_list_own_scope_does_not_double_count_public_catalog_rows() {
        let apps = cloud_apps_for_visibility_scope(
            vec![CloudCatalogApp {
                app_id: "netstitch.app.demo".to_string(),
                display_name: "Demo Game".to_string(),
                publisher_name: "Demo Publisher".to_string(),
                authors: vec!["Shin0by".to_string()],
                endpoint_count: 602,
                available_row_count: 602,
                author_count: 1,
                last_seen_ms: Some(10),
            }],
            &[
                CloudUserAppSummary {
                    app_id: "netstitch.app.demo".to_string(),
                    display_name: "Demo Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Public,
                    endpoint_count: 602,
                    total_endpoint_count: 602,
                    available_row_count: 602,
                    last_seen_ms: Some(10),
                    last_uploaded_at_ms: None,
                },
                CloudUserAppSummary {
                    app_id: "netstitch.app.demo".to_string(),
                    display_name: "Demo Game".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Shin0by".to_string()),
                    visibility: CloudObservationVisibility::Private,
                    endpoint_count: 3,
                    total_endpoint_count: 3,
                    available_row_count: 3,
                    last_seen_ms: Some(20),
                    last_uploaded_at_ms: None,
                },
            ],
            CloudObservationVisibilityScope::All,
            "",
            "",
            "",
            "",
            true,
        );

        assert_eq!(apps.len(), 1);
        assert_eq!(cloud_app_available_row_count(&apps[0]), 605);
    }

    #[test]
    fn failed_filter_keeps_only_failure_like_rows() {
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: vec![
                observation(1, ConnectionStateDto::Established, 0, 4),
                observation(2, ConnectionStateDto::Attempting, 1, 0),
                observation(3, ConnectionStateDto::Failed, 2, 0),
            ],
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::Failed,
            },
            pending_exe_path: String::new(),
        };

        let rows = filter_observations(&snapshot);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| row.id == 2));
        assert!(rows.iter().any(|row| row.id == 3));
    }

    #[test]
    fn header_filters_apply_locally_over_snapshot_results() {
        let mut process_match = observation(1, ConnectionStateDto::Established, 0, 1);
        process_match.process_name = "Discord.exe".to_string();
        process_match.remote_ip = "203.0.113.10".to_string();
        process_match.remote_port = 443;
        process_match.protocol = ProtocolDto::Tcp;

        let mut ip_match = observation(2, ConnectionStateDto::Established, 0, 1);
        ip_match.process_name = "Updater.exe".to_string();
        ip_match.remote_ip = "198.51.100.77".to_string();
        ip_match.remote_port = 53;
        ip_match.protocol = ProtocolDto::Udp;

        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: vec![process_match, ip_match],
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: "203.0.113".to_string(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        let broad_rows = filter_observations_with_header_filters(
            &snapshot,
            &HeaderObservationFilters {
                app_search: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
            },
        );
        assert_eq!(broad_rows.len(), 1);
        assert_eq!(broad_rows[0].id, 1);

        let udp_rows = filter_observations_with_header_filters(
            &SnapshotResponse {
                filters: FiltersDto {
                    app_search: String::new(),
                    search_text: String::new(),
                    domain_search: String::new(),
                    port_search: String::new(),
                    protocol: "All".to_string(),
                    public_ip: false,
                    observation_filter: ObservationFilterDto::All,
                },
                ..snapshot.clone()
            },
            &HeaderObservationFilters {
                app_search: "update".to_string(),
                domain_search: String::new(),
                port_search: "53".to_string(),
                protocol: "UDP".to_string(),
                public_ip: false,
            },
        );
        assert_eq!(udp_rows.len(), 1);
        assert_eq!(udp_rows[0].id, 2);
    }

    #[test]
    fn domain_filter_applies_to_new_snapshot_rows() {
        let mut first_seen = observation_with_ip(1, "203.0.113.10");
        first_seen.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("api.example.com".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_TLS_SNI.to_string()),
            ..IpEnrichmentDto::default()
        });
        let mut new_scan_row = observation_with_ip(2, "203.0.113.11");
        new_scan_row.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("assets.example.com".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_HTTP_HOST.to_string()),
            ..IpEnrichmentDto::default()
        });
        let mut unrelated_new_row = observation_with_ip(3, "203.0.113.12");
        unrelated_new_row.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("assets.example.net".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_HTTP_HOST.to_string()),
            ..IpEnrichmentDto::default()
        });

        let rows = filter_observations_with_header_filters(
            &snapshot_with_observations(vec![first_seen, new_scan_row, unrelated_new_row]),
            &HeaderObservationFilters {
                app_search: String::new(),
                domain_search: "*example.com".to_string(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
            },
        );

        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn language_is_russian_accepts_common_locale_codes() {
        assert!(language_is_russian("ru-ru"));
        assert!(language_is_russian("RU"));
        assert!(!language_is_russian("en-en"));
    }

    #[test]
    fn svg_icon_source_is_inlined_as_data_uri() {
        let root =
            std::env::temp_dir().join(format!("netstitch-ui-svg-icon-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temp dir");
        let svg_path = root.join("icon.svg");
        fs::write(
            &svg_path,
            r##"<svg xmlns="http://www.w3.org/2000/svg"><path fill="#fff"/></svg>"##,
        )
        .expect("svg fixture");

        let src = compute_icon_image_src(&svg_path.to_string_lossy());

        assert!(src.starts_with("data:image/svg+xml;charset=utf-8,"));
        assert!(src.contains("%3Csvg"));
        assert!(src.contains("%23fff"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn png_icon_source_is_inlined_as_data_uri() {
        let root =
            std::env::temp_dir().join(format!("netstitch-ui-png-icon-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temp dir");
        let png_path = root.join("icon.png");
        fs::write(&png_path, [0x89, b'P', b'N', b'G']).expect("png fixture");

        let src = compute_icon_image_src(&png_path.to_string_lossy());

        assert_eq!(src, "data:image/png;base64,iVBORw==");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn icon_image_src_caches_file_reads_for_render_path() {
        let root =
            std::env::temp_dir().join(format!("netstitch-ui-icon-cache-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temp dir");
        let svg_path = root.join("icon.svg");
        fs::write(&svg_path, r##"<svg><path fill="#111"/></svg>"##).expect("svg fixture");

        let first = icon_image_src(&svg_path.to_string_lossy());
        fs::write(&svg_path, r##"<svg><path fill="#222"/></svg>"##).expect("svg fixture update");
        let second = icon_image_src(&svg_path.to_string_lossy());

        assert_eq!(first, second);
        assert!(
            first.contains("%23111"),
            "first render should compute and cache the original file content"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn close_button_icon_is_embedded_as_svg_data_uri() {
        let src = inline_svg_data_uri(CLOSE_TIMES_ICON_SVG);

        assert!(src.starts_with("data:image/svg+xml;charset=utf-8,"));
        assert!(src.contains("Path_23"));
        assert!(src.contains("%23") || src.contains("fill"));
    }

    #[test]
    fn ignored_address_delete_reuses_square_close_button_contract() {
        let source = include_str!("app.rs");
        let action = "\"data-ui-action\": ui::action::DELETE_IGNORED_ADDRESS";
        let action_pos = source.find(action).expect("ignored-address delete action");
        let before_action = &source[..action_pos];
        let after_action = &source[action_pos..];
        let close_class = "class: \"input-box button button--danger button--square button--close\"";
        let icon_class = "class: \"button__icon\"";
        let icon_src = "src: \"{close_icon_src}\"";

        assert!(
            before_action.rfind(close_class).is_some(),
            "ignored-address delete action should be on the shared square close button"
        );
        assert!(
            after_action
                .find(icon_class)
                .is_some_and(|offset| offset < 1_200),
            "ignored-address delete button should render the close icon image"
        );
        assert!(
            after_action
                .find(icon_src)
                .is_some_and(|offset| offset < 1_200),
            "ignored-address delete button should reuse the embedded close svg"
        );
        assert!(
            !after_action
                .find("\"x\"")
                .is_some_and(|offset| offset < 1_200),
            "ignored-address delete button should not fall back to a text x"
        );
        assert!(
            source.contains("pending_delete_ignored_address.set(Some(rule.clone()))"),
            "ignored-address delete action should open a confirmation modal"
        );
        assert!(source.contains("dialog_id: ui::id::DELETE_IGNORED_ADDRESS_DIALOG"));
        assert!(source.contains("ui::entity::DELETE_IGNORED_ADDRESS_DIALOG"));
        assert!(source.contains("ui::action::CONFIRM_DELETE_IGNORED_ADDRESS"));
        assert!(source.contains("ui::action::CANCEL_DELETE_IGNORED_ADDRESS"));
        assert!(source.contains("dialog.delete_ignored.localhost_note"));
        assert!(source.contains("dialog.delete_ignored.local_ip_note"));
    }

    #[test]
    fn clear_monitoring_dialog_offers_all_selected_and_cancel_actions() {
        let source = include_str!("app.rs");

        assert!(source.contains("let mut show_clear_monitoring_prompt = use_signal(|| false);"));
        assert!(source.contains("show_clear_monitoring_prompt.set(true);"));
        assert!(source.contains("ClearMonitoringDialog"));
        assert!(source.contains("id: ui::id::CLEAR_MONITORING_DIALOG"));
        assert!(source.contains("ui::entity::CLEAR_MONITORING_DIALOG"));
        assert!(source.contains("ui::action::CANCEL_CLEAR_MONITORING"));
        assert!(source.contains("ui::action::CONFIRM_CLEAR_MONITORING_ALL"));
        assert!(source.contains("ui::action::CONFIRM_CLEAR_MONITORING_SELECTED"));
        assert!(source.contains("ui::id::CONFIRM_CLEAR_MONITORING_ALL_BUTTON"));
        assert!(source.contains("ui::id::CONFIRM_CLEAR_MONITORING_SELECTED_BUTTON"));
        assert!(source.contains("dialog.clear_monitoring.all"));
        assert!(source.contains("dialog.clear_monitoring.selected"));
        assert!(source.contains("dialog.clear_monitoring.all_count"));
        assert!(source.contains("dialog.clear_monitoring.selected_count"));
        assert!(
            source.contains(
                ".filter(|observation| observation.is_confirmed && !observation.is_exported)"
            ),
            "selected monitoring cleanup should use the same highlighted selection as export-ready rows"
        );
    }

    #[test]
    fn clear_monitoring_actions_send_expected_ids_and_report_backend_count() {
        let source = include_str!("app.rs").replace('\r', "");
        let clear_all_start = source
            .find("on_clear_all: move |_|")
            .expect("clear all action");
        let clear_selected_start = source
            .find("on_clear_selected:")
            .expect("clear selected action");
        let clear_all_source = &source[clear_all_start..clear_selected_start];
        let clear_selected_source = &source[clear_selected_start..];

        assert!(
            clear_all_source.contains(".snapshot()\n                            .observations\n                            .iter()\n                            .map(|observation| observation.id)"),
            "delete all must send the ids of current monitoring rows"
        );
        assert!(
            clear_all_source.contains("DeleteObservationsRequest {\n                            observation_ids: ids.clone(),"),
            "delete all must call the batch delete endpoint with row ids"
        );
        assert!(
            clear_all_source.contains("Ok(cleared_count) =>")
                && clear_all_source
                    .contains("format!(\"{clear_all_event_label}: {cleared_count}\""),
            "delete all must report the backend deleted count"
        );
        assert!(
            clear_selected_source.contains("flush_observation_selection_confirm("),
            "delete selected must flush pending UI selection before collecting ids"
        );
        assert!(
            clear_selected_source.contains("let ids = clone_observation_selection_store(&clear_selection_store)\n                                .into_iter()\n                                .collect::<Vec<_>>();"),
            "delete selected must send ids from the selection store"
        );
        assert!(
            clear_selected_source.contains("Ok(cleared_count) =>")
                && clear_selected_source
                    .contains("format!(\"{clear_selected_event_label}: {cleared_count}\""),
            "delete selected must report the backend deleted count"
        );
    }

    #[test]
    fn observation_selection_click_batches_support_ctrl_toggle_and_shift_range_invert() {
        let mut observations = vec![
            observation_with_ip(1, "203.0.113.1"),
            observation_with_ip(2, "203.0.113.2"),
            observation_with_ip(3, "203.0.113.3"),
            observation_with_ip(4, "203.0.113.4"),
        ];
        observations[1].is_confirmed = true;
        let selected = BTreeSet::from([2]);

        let (select_ids, deselect_ids) =
            observation_selection_batches(&observations, 1, None, &selected, false, false);
        assert!(select_ids.is_empty());
        assert!(deselect_ids.is_empty());

        let (select_ids, deselect_ids) =
            observation_selection_batches(&observations, 2, None, &selected, true, false);
        assert!(select_ids.is_empty());
        assert_eq!(deselect_ids, vec![2]);

        let (select_ids, deselect_ids) =
            observation_selection_batches(&observations, 4, Some(1), &selected, false, true);
        assert_eq!(select_ids, vec![1, 3, 4]);
        assert_eq!(deselect_ids, vec![2]);
    }

    #[test]
    fn cloud_download_selection_batches_support_ctrl_toggle_and_shift_range_invert() {
        let rows = vec![
            cloud_downloaded_observation("row-1"),
            cloud_downloaded_observation("row-2"),
            cloud_downloaded_observation("row-3"),
            cloud_downloaded_observation("row-4"),
        ];
        let selected = BTreeSet::from(["row-2".to_string()]);

        let (select_ids, deselect_ids) =
            cloud_download_selection_batches(&rows, "row-1", None, &selected, false, false);
        assert!(select_ids.is_empty());
        assert!(deselect_ids.is_empty());

        let (select_ids, deselect_ids) =
            cloud_download_selection_batches(&rows, "row-2", None, &selected, true, false);
        assert!(select_ids.is_empty());
        assert_eq!(deselect_ids, vec!["row-2".to_string()]);

        let (select_ids, deselect_ids) =
            cloud_download_selection_batches(&rows, "row-4", Some("row-1"), &selected, false, true);
        assert_eq!(
            select_ids,
            vec![
                "row-1".to_string(),
                "row-3".to_string(),
                "row-4".to_string()
            ]
        );
        assert_eq!(deselect_ids, vec!["row-2".to_string()]);
    }

    #[test]
    fn cloud_import_add_to_monitoring_requires_selected_rows() {
        let mut state = CloudSyncUiState::default();
        state.downloaded_rows = (0..602)
            .map(|index| cloud_downloaded_observation(&format!("row-{index}")))
            .collect();

        assert_eq!(cloud_import_rows_for_add_to_monitoring(&state).len(), 0);

        state
            .selected_download_row_ids
            .insert("row-501".to_string());
        assert_eq!(cloud_import_rows_for_add_to_monitoring(&state).len(), 1);
    }

    #[test]
    fn cloud_import_add_to_monitoring_uses_all_selected_rows() {
        let mut state = CloudSyncUiState::default();
        state.downloaded_rows = (0..602)
            .map(|index| cloud_downloaded_observation(&format!("row-{index}")))
            .collect();
        state.selected_download_row_ids = state
            .downloaded_rows
            .iter()
            .map(|row| row.row_id.clone())
            .collect();

        assert_eq!(state.selected_download_row_ids.len(), 602);
        assert_eq!(cloud_import_rows_for_add_to_monitoring(&state).len(), 602);
    }

    #[test]
    fn cloud_download_start_clears_stale_staging_rows_and_selection() {
        let mut state = CloudSyncUiState::default();
        state.downloaded_rows = (0..500)
            .map(|index| cloud_downloaded_observation(&format!("row-{index}")))
            .collect();
        state
            .selected_download_row_ids
            .insert("row-499".to_string());
        state.last_response_json = Some("{\"download\":{\"rows\":500}}".to_string());

        reset_cloud_download_staging_for_download(&mut state);

        assert!(state.downloaded_rows.is_empty());
        assert!(state.selected_download_row_ids.is_empty());
        assert!(state.last_response_json.is_none());
    }

    #[test]
    fn cloud_import_actions_are_disabled_while_download_is_loading() {
        let source = include_str!("app.rs").replace('\r', "");
        assert!(source.contains(
            "let cloud_import_loading = cloud_import_active && cloud_loading_app_id().is_some();"
        ));
        let add_button_start = source
            .find("id: ui::id::CLOUD_DOWNLOAD_ADD_TO_MONITORING_BUTTON")
            .expect("cloud import add-to-monitoring button");
        let export_button_start = source
            .find("id: ui::id::CLOUD_DOWNLOAD_EXPORT_CSV_BUTTON")
            .expect("cloud import export CSV button");
        let upload_panel_start = source[export_button_start..]
            .find("if cloud_overlay_mode() == CloudOverlayMode::Upload")
            .map(|offset| export_button_start + offset)
            .expect("cloud upload panel after download footer");
        let footer_buttons = &source[add_button_start..upload_panel_start];
        assert!(footer_buttons.contains(
            "disabled: cloud_import_loading || cloud_status.selected_download_row_ids.is_empty(),"
        ));
        assert!(
            !source[add_button_start..export_button_start]
                .contains("cloud_status.downloaded_rows.is_empty(),")
        );
        assert!(!footer_buttons.contains(
            "disabled: cloud_import_loading || cloud_status.downloaded_rows.is_empty(),"
        ));
    }

    #[test]
    fn cloud_progress_stages_use_500_row_chunks() {
        assert_eq!(cloud_progress_stages(1).len(), 1);
        assert_eq!(cloud_progress_stages(500).len(), 1);
        assert_eq!(cloud_progress_stages(501).len(), 2);
        assert_eq!(cloud_progress_stages(1600).len(), 4);
    }

    #[test]
    fn cloud_download_staging_table_uses_action_button_without_checkboxes() {
        let source = include_str!("app.rs").replace('\r', "");
        let table_start = source
            .find("table { class: \"cloud-sync-table cloud-sync-staging-data-table cloud-sync-staging-data-table--body\"")
            .expect("cloud staging body table should exist");
        let table_end = source[table_start..]
            .find("div { class: \"cloud-sync-panel__footer\"")
            .map(|offset| table_start + offset)
            .expect("cloud staging table should be followed by panel footer");
        let table_source = &source[table_start..table_end];

        assert!(!table_source.contains("r#type: \"checkbox\""));
        assert!(table_source.contains("ui::action::TOGGLE_CLOUD_DOWNLOAD_ROW_SELECTION"));
        assert!(table_source.contains("apply_cloud_download_selection_click("));
        assert!(table_source.contains("toggle_cloud_download_row_selection("));
        assert!(table_source.contains("Modifiers::CONTROL"));
        assert!(table_source.contains("Modifiers::SHIFT"));
    }

    #[test]
    fn observation_selection_confirm_queue_debounces_and_coalesces() {
        let pending = Arc::new(Mutex::new(PendingObservationSelectionConfirm::default()));

        queue_observation_selection_confirm(&pending, &[1, 2], true);
        queue_observation_selection_confirm(&pending, &[2], false);

        assert!(drain_ready_observation_selection_confirm(&pending).is_none());
        {
            let mut state = pending.lock().expect("pending selection lock");
            state.changed_at = Some(
                Instant::now()
                    - Duration::from_millis(OBSERVATION_SELECTION_CONFIRM_DEBOUNCE_MS + 1),
            );
        }

        let (select_ids, deselect_ids) =
            drain_ready_observation_selection_confirm(&pending).expect("ready batch");
        assert_eq!(select_ids, vec![1]);
        assert_eq!(deselect_ids, vec![2]);
        assert!(drain_ready_observation_selection_confirm(&pending).is_none());
    }

    #[test]
    fn observation_selection_store_resyncs_snapshot_with_pending_overlay() {
        let store = Arc::new(Mutex::new(ObservationSelectionStore::default()));
        let pending = Arc::new(Mutex::new(PendingObservationSelectionConfirm::default()));
        let mut observations = vec![
            observation_with_ip(1, "203.0.113.1"),
            observation_with_ip(2, "203.0.113.2"),
        ];
        observations[0].is_confirmed = true;

        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1])
        );

        observations[0].is_confirmed = false;
        observations[1].is_confirmed = true;
        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([2])
        );

        queue_observation_selection_confirm(&pending, &[1], true);
        queue_observation_selection_confirm(&pending, &[2], false);
        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1])
        );
    }

    #[test]
    fn observation_selection_store_keeps_exported_rows_selected() {
        let store = Arc::new(Mutex::new(ObservationSelectionStore::default()));
        let pending = Arc::new(Mutex::new(PendingObservationSelectionConfirm::default()));
        let mut observations = vec![observation_with_ip(1, "203.0.113.1")];
        observations[0].is_confirmed = true;

        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1])
        );

        observations[0].is_exported = true;
        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1])
        );
    }

    #[test]
    fn observation_selection_store_keeps_in_flight_overlay_until_snapshot_catches_up() {
        let store = Arc::new(Mutex::new(ObservationSelectionStore::default()));
        let pending = Arc::new(Mutex::new(PendingObservationSelectionConfirm::default()));
        let mut observations = vec![observation_with_ip(1, "203.0.113.1")];

        queue_observation_selection_confirm(&pending, &[1], true);
        {
            let mut state = pending.lock().expect("pending selection lock");
            state.changed_at = Some(
                Instant::now() - Duration::from_millis(OBSERVATION_SELECTION_CONFIRM_DEBOUNCE_MS),
            );
        }
        let _ = drain_ready_observation_selection_confirm(&pending)
            .expect("selection batch should be sent to watcher");

        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1]),
            "selection should not flicker off while watcher snapshot is still stale"
        );

        observations[0].is_confirmed = true;
        sync_observation_selection_store(&store, &pending, &observations);
        assert_eq!(
            clone_observation_selection_store(&store),
            BTreeSet::from([1])
        );
        assert!(
            pending
                .lock()
                .expect("pending selection lock")
                .in_flight_select_ids
                .is_empty(),
            "in-flight overlay should clear after snapshot catches up"
        );
    }

    #[test]
    fn observation_selection_uses_deferred_batched_watcher_confirmation() {
        let source = include_str!("app.rs").replace('\r', "");
        let app_source = &source[..source
            .find("#[cfg(test)]")
            .expect("test module marker should exist")];
        let apply_start = source
            .find("fn apply_observation_selection_click(")
            .expect("selection helper should exist");
        let apply_end = source[apply_start..]
            .find("fn apply_cloud_download_selection_click(")
            .map(|offset| apply_start + offset)
            .expect("cloud selection helper should follow monitoring selection helpers");
        let helper_source = &source[apply_start..apply_end];

        assert!(source.contains("confirm_observations_deferred("));
        assert!(helper_source.contains("queue_observation_selection_confirm("));
        assert!(
            !helper_source.contains("confirm_observations("),
            "row selection must not call the blocking watcher confirmation path"
        );
        assert!(
            !helper_source.contains("evaluate_script(")
                && !app_source.contains("set_observation_rows_selected_dom"),
            "row selection should have one visual source of truth through the reactive selection store"
        );
    }

    #[test]
    fn observation_row_selection_reuses_visible_rows_without_per_row_full_clone() {
        let source = include_str!("app.rs").replace('\r', "");
        let render_start = source
            .find("for observation in visible_observations.clone()")
            .expect("observation render loop should exist");
        let render_end = source[render_start..]
            .find("fn CloudSyncAppRowView(")
            .map(|offset| render_start + offset)
            .expect("cloud app row component should follow render loop");
        let render_source = &source[render_start..render_end];
        assert!(
            source.contains(
                "let visible_observations_for_selection = Arc::new(visible_observations.clone());"
            ),
            "selection handlers should share one visible-row snapshot per render"
        );
        assert!(
            !render_source.contains("let row_observations = visible_observations.clone();"),
            "each rendered row must not clone the full visible observation list"
        );
    }

    #[test]
    fn footer_status_logging_tracks_snapshot_refresh_nonce() {
        let source = include_str!("app.rs");
        assert!(
            source
                .matches("let _ = monitoring_ui_refresh_nonce();")
                .count()
                >= 4,
            "footer status, connector and module logging effects must rerun after snapshot refresh"
        );
    }

    #[test]
    fn information_dialog_uses_branding_and_panel_contract() {
        let source = include_str!("app.rs");

        assert!(source.contains("resources/branding/images/NetStitch.png"));
        assert!(source.contains("resources/ui/icons/help-info-svgrepo-com.svg"));
        assert!(source.contains("id: ui::id::OPEN_INFORMATION_BUTTON"));
        assert!(source.contains("let information_button_class = if app_update_available"));
        assert!(source.contains("header-action-button--update-available"));
        let obsolete_info_class = [
            "class: \"input-box button button--icon header-action-button",
            "header-info-button\"",
        ]
        .join(" ");
        assert!(!source.contains(&obsolete_info_class));
        assert!(source.contains("class: \"button__icon button__icon--information\""));
        assert!(source.contains("\"data-ui-action\": ui::action::OPEN_INFORMATION"));
        assert!(source.contains(
            "id: ui::id::EXPORT_CSV_BUTTON,\n                                class: \"input-box button button--icon header-action-button\","
        ));
        assert!(source.contains(
            "div { class: \"header-action-separator\", \"aria-hidden\": \"true\" }\n                            button {\n                                id: ui::id::OPEN_INFORMATION_BUTTON"
        ));
        assert!(source.contains("id: ui::id::INFORMATION_DIALOG"));
        assert!(source.contains("class: \"modal modal--panel information-dialog\""));
        assert!(source.contains("class: \"info-panel info-panel--about\""));
        assert!(source.contains("class: \"info-panel info-panel--details\""));
        assert!(source.contains("class: \"info-panel__footer info-panel__footer--success\""));
        assert!(source.contains("class: \"info-subpanel info-logo-panel\""));
        assert!(source.contains("class: \"info-subpanel info-details-subpanel\""));
        assert!(source.contains("class: \"info-details-scroll\""));
        assert!(source.contains("const APP_AUTHOR: &str = \"Shin0by\";"));
        assert!(source.contains("const APP_EMAIL: &str = \"warfactory@gmail.com\";"));
        assert!(source.contains(
            "const APP_REPOSITORY_URL: &str = \"https://github.com/Shin0by/NetStitch\";"
        ));
        assert!(source.contains("href: \"{app_email_href}\""));
        assert!(source.contains("href: \"{repository_url}\""));
        assert!(source.contains("id: ui::id::UPDATE_APPLICATION_BUTTON"));
        assert!(source.contains("\"data-ui-action\": ui::action::UPDATE_APPLICATION"));
        assert!(source.contains("class: \"input-box button button--primary\""));
        assert!(source.contains("open_browser_url(&update_url)"));
        assert!(source.contains("update_check_interval_duration("));
        assert!(source.contains(".update_check_interval_minutes"));
        assert!(source.contains("tokio::time::sleep(interval).await;"));
        assert!(source.contains("record_system_event_via_watcher(base_url, event);"));
        let dialog_start = source
            .find("id: ui::id::INFORMATION_DIALOG")
            .expect("information dialog");
        let dialog_end = source[dialog_start..]
            .find("if show_profile_export_prompt()")
            .map(|offset| dialog_start + offset)
            .expect("next modal");
        let dialog_source = &source[dialog_start..dialog_end];
        assert!(
            !dialog_source.contains("HelpIcon"),
            "information dialog header should not use a help '?' button"
        );
        assert!(dialog_source.contains("dt { \"{dialog_info_author}\" }"));
        assert!(dialog_source.contains("dt { \"{dialog_info_email}\" }"));
        assert!(dialog_source.contains("dt { \"{dialog_info_repository}\" }"));
    }

    #[test]
    fn update_check_compares_full_revision_versions() {
        assert_eq!(
            compare_version_text("v1.1.0.437", "1.1.0.436"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_version_text("1.1.0", "1.1.0.0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_version_text("1.1.0.435", "1.1.0.436"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            extract_version_text("NetStitch v1.2.3.4"),
            Some("1.2.3.4".to_string())
        );
    }

    #[test]
    fn cloud_sync_dialog_is_user_action_entrypoint() {
        let source = include_str!("app.rs");
        let app_source = &source[..source
            .find("#[cfg(test)]")
            .expect("test module marker should exist")];

        assert!(source.contains("resources/ui/icons/upload-to-cloud-svgrepo-com.svg"));
        assert!(app_source.contains("let mut show_cloud_sync_prompt = use_signal(|| false);"));
        assert!(app_source.contains("let mut cloud_app_search = use_signal(String::new);"));
        assert!(app_source.contains("let mut cloud_app_search_draft = use_signal(String::new);"));
        assert!(
            app_source.contains("let mut cloud_publisher_search_draft = use_signal(String::new);")
        );
        assert!(
            app_source.contains("let mut cloud_source_search_draft = use_signal(String::new);")
        );
        assert!(!app_source.contains("cloud_auth_in_progress"));
        assert!(!app_source.contains("let mut cloud_stay_signed_in ="));
        assert!(app_source.contains("session: read_persisted_cloud_session(),"));
        assert!(source.contains("id: ui::id::OPEN_CLOUD_IMPORT_BUTTON"));
        assert!(source.contains("id: ui::id::OPEN_CLOUD_EXPORT_BUTTON"));
        assert!(source.contains("\"data-ui-action\": ui::action::OPEN_CLOUD_IMPORT"));
        assert!(source.contains("\"data-ui-action\": ui::action::OPEN_CLOUD_EXPORT"));
        assert!(source.contains("cloud_overlay_mode.set(CloudOverlayMode::Download);"));
        assert!(source.contains("cloud_overlay_mode.set(CloudOverlayMode::Upload);"));
        assert!(source.contains("show_cloud_sync_prompt.set(true);"));
        assert!(source.contains("if show_cloud_sync_prompt()"));
        assert!(source.contains("id: ui::id::CLOUD_SYNC_DIALOG"));
        let cloud_dialog_start = app_source
            .find("id: ui::id::CLOUD_SYNC_DIALOG")
            .expect("cloud dialog markup should exist");
        let cloud_dialog_end = app_source[cloud_dialog_start..]
            .find("if show_information_prompt()")
            .map(|offset| cloud_dialog_start + offset)
            .expect("next overlay marker should exist");
        let cloud_dialog_source = &app_source[cloud_dialog_start..cloud_dialog_end];
        assert!(
            !cloud_dialog_source.contains(
                "class: \"modal modal--panel workspace-overlay-dialog cloud-sync-dialog\""
            )
        );
        assert!(!cloud_dialog_source.contains("div { class: \"modal__header\""));
        assert!(!app_source.contains("id: ui::id::REFRESH_CLOUD_SYNC_BUTTON"));
        assert!(!app_source.contains("\"data-ui-action\": ui::action::REFRESH_CLOUD_SYNC"));
        assert!(source.contains("tokio::time::sleep(std::time::Duration::from_secs(10)).await;"));
        assert!(source.contains(
            "tokio::time::timeout(std::time::Duration::from_millis(9500), refresh_task)"
        ));
        assert!(source.contains("merge_cloud_state_from_async_result("));
        assert!(source.contains(
            "next.selected_download_row_ids = current.selected_download_row_ids.clone();"
        ));
        assert!(source.contains(
            "pending_state.refresh_generation = pending_state.refresh_generation.wrapping_add(1);"
        ));
        assert!(
            source.contains("if result.state.refresh_generation != current.refresh_generation")
        );
        assert!(
            source.contains("auth_generation = pending_state.auth_generation.wrapping_add(1);")
        );
        assert!(source.contains("let auth_flow_generation = pending_state.auth_generation;"));
        assert!(source.contains(
            "if current.session.is_some() || auth_flow_generation != current.auth_generation"
        ));
        assert!(source.contains("if current.session.is_some()"));
        assert!(
            source
                .contains("next_state.auth_generation = current.auth_generation.wrapping_add(1);")
        );
        assert!(source.contains("id: ui::id::START_CLOUD_GOOGLE_OAUTH_BUTTON"));
        assert!(source.contains("id: ui::id::SIGN_OUT_CLOUD_BUTTON"));
        assert!(source.contains("\"data-ui-action\": ui::action::START_CLOUD_GOOGLE_OAUTH"));
        assert!(source.contains("start_cloud_google_auth("));
        assert!(source.contains("cloud_upload_nickname,"));
        assert!(source.contains("cloud_nickname_check_status,"));
        assert!(source.contains("cloud_nickname_check_generation,"));
        assert!(
            source.contains("push_status_history_warning_line(status_history, started_message);")
        );
        assert!(source.contains("disabled: cloud_session_login.is_some()"));
        assert!(source.contains("\"data-ui-action\": ui::action::SIGN_OUT_CLOUD"));
        assert!(source.contains("\"data-ui-action\": ui::action::UPLOAD_CLOUD_DATA"));
        assert!(!source.contains("\"data-ui-action\": \"cloud.upload\""));
        assert!(source.contains("disabled: cloud_session_login.is_none()"));
        assert!(source.contains("clear_persisted_cloud_auth_state()"));
        assert!(source.contains("id: ui::id::CLOUD_APP_SEARCH_INPUT"));
        assert!(source.contains("\"data-ui-entity\": ui::control::CLOUD_APP_SEARCH_INPUT"));
        assert!(source.contains("value: \"{cloud_app_search_draft()}\""));
        assert!(source.contains("cloud_app_search_draft.set(event.value().to_string())"));
        assert!(source.contains("cloud_app_search.set(cloud_app_search_draft())"));
        assert!(source.contains("cloud_publisher_search.set(cloud_publisher_search_draft())"));
        assert!(source.contains("cloud_source_search.set(cloud_source_search_draft())"));
        assert!(source.contains("onkeydown: move |event|"));
        assert!(source.contains("pulse_text_input(input_apply_pulse, \"cloud-app\")"));
        assert!(!app_source.contains("\"data-ui-action\": ui::action::CLOSE_CLOUD_SYNC"));
        assert!(source.contains("id: ui::id::CLOUD_SCOPE_MINE_BUTTON"));
        assert!(source.contains("\"data-ui-action\": ui::action::SELECT_CLOUD_SCOPE_MINE"));
        assert!(source.contains("button__icon--my-publications"));
        assert!(source.contains("let cloud_my_publications_active = cloud_scope_mine();"));
        assert!(source.contains("let next_scope = !cloud_my_publications_active;"));
        assert!(source.contains("cloud_scope_mine.set(next_scope);"));
        assert!(source.contains("let author = cloud_author_filter_display.trim().to_string();"));
        assert!(source.contains("cloud_source_search_draft.set(author.clone());"));
        assert!(source.contains("cloud_source_search.set(author);"));
        assert!(source.contains("dialog.cloud_sync.download_title"));
        assert!(source.contains("dialog.cloud_sync.upload_title"));
        assert!(source.contains("dialog.cloud_sync.download_action"));
        assert!(source.contains("dialog.cloud_sync.upload_action"));
        assert!(source.contains("dialog.cloud_sync.download_started"));
        assert!(source.contains("cloud-sync-panel__footer"));
        assert!(source.contains("cloud-sync-panel__footer-summary"));
        assert!(
            source.contains(
                "span { class: \"panel-footer-meta__item\", \"{observations_total}: {cloud_status.downloaded_rows.len()}\" }"
            ) && source.contains(
                "span { class: \"panel-footer-meta__item\", \"{observations_selected}: {cloud_status.selected_download_row_ids.len()}\" }"
            ) && source.contains(
                "span { class: \"{cloud_download_quota_value_class}\", \"{cloud_download_quota_value}\" }"
            )
        );
        assert!(source.contains("cloud-sync-panel__footer-actions"));
        assert!(source.contains("cloud-sync-panel__action"));
        assert!(source.contains("cloud-sync-panel-service"));
        assert!(source.contains("dialog.cloud_sync.authorization"));
        assert!(source.contains("dialog.cloud_sync.authorization_active"));
        assert!(source.contains("dialog.cloud_sync.authorization_tooltip"));
        assert!(source.contains(".email"));
        assert!(source.contains("\"data-tooltip\": \"{dialog_cloud_sync_authorization_tooltip}\""));
        assert!(source.contains("dialog.cloud_sync.nickname_invalid"));
        assert!(source.contains("dialog.cloud_sync.nickname_taken"));
        assert!(source.contains("dialog.cloud_sync.nickname_accepted"));
        assert!(source.contains("start_cloud_nickname_check("));
        assert!(source.contains("cloud_state.set(next_state.clone());"));
        assert!(source.contains("let nickname = cloud_upload_nickname().trim().to_string();"));
        assert!(source.contains("cloud_nickname_check_generation.set(next_nickname_generation);"));
        assert!(source.contains("CloudNicknameCheckStatus::Checking"));
        assert!(source.contains("cloud_upload_nickname_blocks_upload"));
        assert!(source.contains("|| cloud_upload_progress().is_some()"));
        assert!(source.contains("mark_uploaded_public_observations(&mut state, &snapshot);"));
        assert!(source.contains("&cloud_status.uploaded_observation_ids"));
        assert!(source.contains("dialog.cloud_sync.sign_in_done"));
        assert!(source.contains("dialog.cloud_sync.sign_out"));
        assert!(source.contains("dialog.cloud_sync.signed_out"));
        assert!(source.contains("read_persisted_cloud_upload_nickname"));
        assert!(source.contains("persist_cloud_upload_nickname_to_sqlite"));
        assert!(source.contains("dialog.cloud_sync.identifier"));
        assert!(source.contains("dialog.cloud_sync.upload_limit"));
        assert!(source.contains("dialog.cloud_sync.download_limit"));
        assert!(source.contains("dialog.cloud_sync.scope_mine"));
        assert!(source.contains("dialog.cloud_sync.my_apps"));
        assert!(source.contains("dialog.cloud_sync.non_public_rows"));
        assert!(source.contains("dialog.cloud_sync.non_public_rows_tooltip"));
        assert!(source.contains("CloudPublicationSortColumn::NonPublicRows"));
        assert!(source.contains("dialog.cloud_sync.app_search"));
        assert!(source.contains("dialog.cloud_sync.source"));
        assert!(source.contains("dialog.cloud_sync.source_search"));
        assert!(source.contains("table-wrap cloud-sync-app-list"));
        assert!(source.contains("table-wrap cloud-sync-staging-table"));
        assert!(source.contains("CloudAppSortColumn::App"));
        assert!(source.contains("CloudAppSortColumn::AvailableRows"));
        assert!(source.contains("CloudAppSortColumn::Authors"));
        assert!(source.contains("CloudRowSortColumn::Source"));
        assert!(source.contains("cloud_row_to_import_row"));
        assert!(source.contains("selected_cloud_csv_export_rows"));
    }

    #[test]
    fn invisible_window_watchdog_does_not_exit_process() {
        let source = include_str!("app.rs");
        let watchdog_source = source
            .split("if window_visible() && !window.is_visible() && !window.is_minimized()")
            .nth(1)
            .expect("invisible window watchdog should exist");
        let watchdog_block = watchdog_source
            .split("tokio::time::sleep(std::time::Duration::from_secs(1)).await;")
            .next()
            .unwrap_or(watchdog_source);

        assert!(watchdog_block.contains("window_visible.set(false);"));
        assert!(!watchdog_block.contains("std::process::exit(0);"));
        assert!(!watchdog_block.contains("shutdown_embedded_watcher"));
    }

    #[test]
    fn cloud_overlay_suppresses_invisible_window_watchdog() {
        let source = include_str!("app.rs");
        let watchdog_source = source
            .split("let show_cloud_sync_prompt = show_cloud_sync_prompt;")
            .nth(1)
            .expect("invisible window watchdog should capture cloud overlay state");
        let watchdog_guard = watchdog_source
            .split("if window_visible() && !window.is_visible() && !window.is_minimized()")
            .next()
            .unwrap_or(watchdog_source);

        assert!(watchdog_guard.contains("|| show_cloud_sync_prompt()"));
    }

    #[test]
    fn cloud_flow_keeps_window_close_behaviour() {
        let source = include_str!("app.rs");
        let app_source = &source[..source
            .find("#[cfg(test)]")
            .expect("test module marker should exist")];

        assert!(
            app_source.contains("window.set_close_behavior(WindowCloseBehaviour::WindowCloses);")
        );
        assert!(
            !app_source.contains("WindowCloseBehaviour::WindowHides"),
            "desktop close button must keep closing the app while cloud import/export is active"
        );
    }

    #[test]
    fn close_requested_during_cloud_flow_exits_process() {
        let source = include_str!("app.rs");
        let close_requested_source = source
            .split("if matches!(&event, WindowEvent::CloseRequested)")
            .nth(1)
            .expect("close requested handler should exist");
        let close_requested_block = close_requested_source
            .split("if matches!(&event, WindowEvent::Destroyed)")
            .next()
            .unwrap_or(close_requested_source);

        assert!(close_requested_block.contains("exit_app("));
        assert!(!close_requested_block.contains("show_cloud_sync_prompt()"));
        assert!(!close_requested_block.contains("cloud_upload_progress().is_some()"));
        assert!(!close_requested_block.contains("cloud_download_progress().is_some()"));
        assert!(!close_requested_block.contains("WindowCloseBehaviour::WindowHides"));
        assert!(!close_requested_block.contains("return;"));
    }

    #[test]
    fn window_destroyed_event_does_not_exit_process() {
        let source = include_str!("app.rs");
        let destroyed_source = source
            .split("if matches!(&event, WindowEvent::Destroyed)")
            .nth(1)
            .expect("window destroyed handler should exist");
        let destroyed_block = destroyed_source
            .split("}")
            .next()
            .unwrap_or(destroyed_source);

        assert!(destroyed_block.contains("window_visible.set(false);"));
        assert!(!destroyed_block.contains("std::process::exit(0);"));
        assert!(!destroyed_block.contains("shutdown_embedded_watcher"));
    }

    #[test]
    fn cloud_sync_overlay_suppresses_hide_when_minimized_autohide() {
        let source = include_str!("app.rs");
        let handler_source = source
            .split("desktop::use_wry_event_handler")
            .nth(1)
            .expect("wry event handler should exist");
        let hide_guard = handler_source
            .split("if !show_integration_module_prompt()")
            .nth(1)
            .expect("hide-when-minimized guard should exist")
            .split("hide_window(")
            .next()
            .unwrap_or(handler_source);

        assert!(hide_guard.contains("&& !show_cloud_sync_prompt()"));
    }

    #[test]
    fn domain_capture_admin_prompt_keeps_switch_off_without_elevation() {
        let source = include_str!("app.rs");

        assert!(
            source.contains("let mut show_domain_capture_admin_prompt = use_signal(|| false);")
        );
        assert!(source.contains("id: ui::id::DOMAIN_CAPTURE_ADMIN_DIALOG"));
        assert!(source.contains("ui::entity::DOMAIN_CAPTURE_ADMIN_DIALOG"));
        assert!(source.contains("ui::action::CLOSE_DOMAIN_CAPTURE_ADMIN"));
        assert!(source.contains("dialog.domain_capture_admin.title"));
        assert!(source.contains("dialog.domain_capture_admin.help"));
        assert!(source.contains("dialog.domain_capture_admin.close"));
        assert!(source.contains(
            "if enabled && snapshot.ui.snapshot_loaded && !snapshot.runtime_status.is_elevated"
        ));
        assert!(source.contains("snapshot.runtime_status.domain_capture_admin_disabled"));
        assert!(source.contains("show_domain_capture_admin_prompt.set(true);"));
        assert!(source.contains("footer_domain_capture_admin_required.clone()"));
    }

    #[test]
    fn ignored_address_value_uses_scrollable_path_field() {
        let source = include_str!("app.rs");
        let row_pos = source
            .find("fn IgnoredAddressRowView")
            .expect("ignored address row view");
        let row_source = &source[row_pos..];

        assert!(
            row_source.contains("class: \"path-field ignored-addresses__address\""),
            "ignored address should use the scrollable path-field contract"
        );
        assert!(
            row_source.contains("value: \"{rule.address_pattern}\""),
            "ignored address field should show the full rule value"
        );
        assert!(
            !row_source[..row_source.find("button {").expect("delete button")]
                .contains("code { class: \"ignored-addresses__address\""),
            "ignored address should not be rendered as clipped code text"
        );
    }

    #[test]
    fn cloud_import_app_catalog_uses_scrollable_path_fields() {
        let source = include_str!("app.rs");
        let row_pos = source
            .find("fn CloudAppCatalogRowView")
            .expect("cloud app catalog row view");
        let row_source = &source[row_pos
            ..source[row_pos..]
                .find("#[component]\nfn ObservationRowView")
                .map(|end| row_pos + end)
                .expect("observation row view")];

        assert_eq!(
            row_source
                .matches("class: \"path-field cloud-sync-app-field")
                .count(),
            4,
            "cloud app, company, row count and authors cells should use scrollable path-field inputs"
        );
        assert!(
            row_source.contains("value: \"{app_name}\"")
                && row_source.contains("value: \"{publisher_name}\"")
                && row_source.contains("value: \"{available_rows_label}\"")
                && row_source.contains("value: \"{authors_label}\""),
            "cloud import app catalog should keep full text values in readonly fields"
        );
        let app_pos = row_source.find("value: \"{app_name}\"").expect("app cell");
        let company_pos = row_source
            .find("value: \"{publisher_name}\"")
            .expect("company cell");
        let rows_pos = row_source
            .find("value: \"{available_rows_label}\"")
            .expect("rows cell");
        let authors_pos = row_source
            .find("value: \"{authors_label}\"")
            .expect("authors cell");
        assert!(
            app_pos < company_pos && company_pos < rows_pos && rows_pos < authors_pos,
            "cloud import app row cells should match header order: App, Company, Rows, Authors"
        );
        assert_eq!(
            row_source.matches("cloud-sync-app-count-field").count(),
            1,
            "only the numeric row-count cell should use the right-aligned count class"
        );
        let count_class_pos = row_source
            .find("cloud-sync-app-count-field")
            .expect("count class");
        assert!(
            company_pos < count_class_pos && count_class_pos < rows_pos,
            "count alignment class should be on the Rows cell, not on Company"
        );
    }

    #[test]
    fn ignored_address_delete_notes_classify_local_rules() {
        assert!(ignored_rule_is_loopback("127.0.0.0/8"));
        assert!(ignored_rule_is_loopback("::1/128"));
        assert!(!ignored_rule_is_loopback("192.168.69.121/32"));
        assert!(ignored_rule_matches_local_machine_ip(
            "192.168.69.121/32",
            Some("192.168.69.121".parse().expect("local ip fixture"))
        ));
        assert!(ignored_rule_matches_local_machine_ip(
            "fd00::1/128",
            Some("fd00::1".parse().expect("local ipv6 fixture"))
        ));
        assert!(!ignored_rule_matches_local_machine_ip(
            "51.105.71.136",
            Some("192.168.69.121".parse().expect("local ip fixture"))
        ));
        assert!(!ignored_rule_matches_local_machine_ip(
            "192.168.69.0/24",
            Some("192.168.69.121".parse().expect("local ip fixture"))
        ));
        assert!(!ignored_rule_is_local_machine_candidate("127.0.0.0/8"));
        assert!(!ignored_rule_is_local_machine_candidate("::1/128"));
    }

    #[test]
    fn ignored_address_domain_and_tooltip_contract_handles_unknown_and_local_rules() {
        assert_eq!(ignored_address_domain_text(&None), "-");

        let enrichment = Some(crate::watcher_api::IpEnrichmentDto {
            domain_name: Some("example.test".to_string()),
            domain_source: Some("tls_sni".to_string()),
            owner_name: Some("Example owner".to_string()),
            owner_range: Some("203.0.113.0/24".to_string()),
            registry: Some("test".to_string()),
            country: None,
            source: Some("Fixture".to_string()),
        });
        assert_eq!(ignored_address_domain_text(&enrichment), "example.test");

        let localhost_tooltip = ignored_address_tooltip(
            "127.0.0.0/8",
            &None,
            "Domain",
            "Owner",
            "Range",
            "Registry",
            "Source",
            "Unknown",
            "Localhost subnet",
            "Local machine subnet",
        );
        assert_eq!(localhost_tooltip, "Localhost subnet");

        if let Some(local_ip) = super::local_machine_ip_for_web_url() {
            let suffix = if local_ip.contains(':') {
                "/128"
            } else {
                "/32"
            };
            let local_tooltip = ignored_address_tooltip(
                &format!("{local_ip}{suffix}"),
                &enrichment,
                "Domain",
                "Owner",
                "Range",
                "Registry",
                "Source",
                "Unknown",
                "Localhost subnet",
                "Local machine subnet",
            );
            assert!(local_tooltip.starts_with("Local machine subnet\n"));
            assert!(local_tooltip.contains("Domain: example.test"));
        }

        let public_tooltip = ignored_address_tooltip(
            "51.105.71.136",
            &enrichment,
            "Domain",
            "Owner",
            "Range",
            "Registry",
            "Source",
            "Unknown",
            "Localhost subnet",
            "Local machine subnet",
        );
        assert!(!public_tooltip.starts_with("Local machine subnet"));
        assert!(public_tooltip.contains("Domain: example.test"));
    }

    #[test]
    fn footer_status_entities_have_stable_indicators_and_safe_tooltips() {
        let source = include_str!("app.rs");
        let render_source = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(source.contains("id: ui::id::APP_FOOTER_WATCHER_STATUS"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_WATCHER_STATUS"));
        assert!(source.contains("id: ui::id::APP_FOOTER_TOOL_STATUS"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_TOOL_STATUS"));
        assert!(source.contains("id: ui::id::APP_FOOTER_WEB_SERVER_STATUS"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_WEB_SERVER_STATUS"));
        assert!(source.contains("id: ui::id::APP_FOOTER_NETWORK_STATUS"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_NETWORK_STATUS"));
        assert!(source.contains("id: ui::id::APP_FOOTER_MESSAGE_PANEL"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_MESSAGE_PANEL"));
        assert!(source.contains("id: ui::id::APP_FOOTER_LANGUAGE_PANEL"));
        assert!(source.contains("\"data-ui-entity\": ui::entity::APP_FOOTER_LANGUAGE_PANEL"));
        assert!(source.contains("let monitoring_total_count = snapshot.observations.len();"));
        assert!(source.contains("let monitoring_displayed_count = visible_observations.len();"));
        assert!(source.contains("let monitoring_selected_count = selected_observation_ids.len();"));
        assert!(!render_source.contains("observation_monitoring_summary("));
        assert!(source.contains("let observations_total = t(\"observations.total\");"));
        assert!(source.contains("let observations_selected = t(\"observations.selected\");"));
        let removed_non_public_key = ["observations", "non_public"].join(".");
        let removed_non_public_binding = ["observations", "_non_public"].concat();
        assert!(!source.contains(&format!(
            "let {removed_non_public_binding} = t(\"{removed_non_public_key}\");"
        )));
        assert!(source.contains("let observations_public_ip = t(\"observations.public_ip\");"));
        assert!(source.contains("ui::action::TOGGLE_PUBLIC_IP_FILTER"));
        assert!(source.contains("class: \"monitoring-header-meta\""));
        assert!(source.contains("class: \"panel-footer monitoring-panel-footer\""));
        assert!(source.contains("{observations_selected}: {monitoring_selected_count}"));
        assert!(source.contains("panel-footer-meta__item"));
        assert!(!render_source.contains("let table_ip_public = t(\"table.ip_public\");"));
        assert!(!render_source.contains("class: ip_public_class"));
        assert!(source.contains("class: \"app-footer-panel footer-watcher-status\""));
        assert!(source.contains("class: \"app-footer-panel footer-tool-status\""));
        assert!(source.contains("class: \"app-footer-panel footer-web-server-status\""));
        assert!(source.contains("class: \"app-footer-panel footer-network-status\""));
        assert!(source.contains("class: \"app-footer-panel app-footer-message-panel\""));
        assert!(source.contains("let ui_controls_disabled = shell_controls_disabled(&snapshot);"));
        assert!(source.contains("let cloud_overlay_active = show_cloud_sync_prompt();"));
        assert!(source.contains("\"data-ui-disabled\": \"{ui_controls_disabled}\""));
        assert!(source.contains("\"data-cloud-overlay-active\": \"{cloud_overlay_active}\""));
        assert!(source.contains("shell shell--controls-disabled"));
        assert!(source.contains("shell shell--cloud-overlay-active"));
        assert!(source.contains("shell shell--controls-disabled shell--cloud-overlay-active"));
        assert!(source.contains("endpoint_probe_tooltip"));
        assert!(source.contains("footer.network.available_prefix"));
        assert!(source.contains("status.waiting_for_watcher"));
        assert!(
            source.contains(
                "class: \"app-footer-panel app-footer-actions app-footer-language-panel\""
            )
        );
        assert!(source.contains(
            "copy_text_to_clipboard(&clipboard_window, &web_server_url_for_footer.url);"
        ));
        assert!(source.contains("footer.web_server.localhost_fallback"));
        assert!(source.contains("web_server_event_line"));
        let removed_subtitle_var = ["let ", "app_", "subtitle"].concat();
        let removed_subtitle_key = ["t(\"app", ".subtitle\")"].concat();
        assert!(
            !source.contains(&removed_subtitle_var) && !source.contains(&removed_subtitle_key),
            "desktop header should not render the removed subtitle block"
        );
        let header_source = render_source
            .split("header {")
            .nth(1)
            .and_then(|rest| rest.split("id: ui::id::APP_CONTENT").next())
            .unwrap_or_default();
        assert!(
            !header_source.contains("app_title")
                && !header_source.contains("app.title")
                && !header_source.contains("NetStitch MVP"),
            "desktop window header should not render the application title"
        );
        for removed_key in [
            "badge.monitoring_active",
            "badge.monitoring_paused",
            "badge.tracked_apps",
            "badge.confirmed_ips",
            "badge.ready_for_export",
            "badge.tray_minimize_on",
            "badge.tray_minimize_off",
        ] {
            assert!(
                !header_source.contains(removed_key),
                "desktop window header should not render {removed_key}"
            );
        }
        assert!(source.contains("hero-labels"));
        assert!(source.contains("header-label"));
        assert!(source.contains("header-system-tools"));
        assert!(source.contains("state-label"));
        assert!(source.contains("ui::id::TOGGLE_MONITORING_BUTTON"));
        assert!(source.contains("ui::action::TOGGLE_MONITORING"));
        assert!(source.contains("button__icon--monitoring"));
        assert!(source.contains("button__icon--confirm-filtered"));
        assert!(source.contains("ui::id::IMPORT_CSV_BUTTON"));
        assert!(source.contains("ui::action::IMPORT_CSV"));
        assert!(source.contains("button__icon--import-csv"));
        assert!(source.contains("header-action-separator"));
        assert!(source.contains("ui::id::OPEN_CLOUD_IMPORT_BUTTON"));
        assert!(source.contains("ui::action::OPEN_CLOUD_IMPORT"));
        assert!(source.contains("ui::id::OPEN_CLOUD_EXPORT_BUTTON"));
        assert!(source.contains("ui::action::OPEN_CLOUD_EXPORT"));
        assert!(source.contains("button__icon--cloud-import"));
        assert!(source.contains("button__icon--cloud-sync"));
        let cloud_button = source
            .find("id: ui::id::OPEN_CLOUD_IMPORT_BUTTON")
            .expect("cloud button");
        let import_csv_button = source
            .find("id: ui::id::IMPORT_CSV_BUTTON")
            .expect("CSV import button");
        assert!(
            cloud_button < import_csv_button,
            "cloud sync button should be before CSV actions"
        );
        assert!(source.contains("ui::id::EXPORT_CSV_BUTTON"));
        assert!(source.contains("ui::action::EXPORT_CSV"));
        assert!(source.contains("button__icon--export-csv"));
        let removed_header_export_id = format!("{}{}", "ui::id::EXPORT_", "CONFIRMED_BUTTON");
        let removed_header_export_icon = format!("{}{}", "button__icon--export", "-confirmed");
        assert!(!source.contains(&removed_header_export_id));
        assert!(!source.contains(&removed_header_export_icon));
        assert!(!source.contains("ui::id::EXPORT_INTEGRATION_MODULE_BUTTON"));
        assert!(!source.contains("ui::action::EXPORT_INTEGRATION_MODULE"));
        assert!(
            source.contains(
                "let export_ready = observation.is_confirmed && !observation.is_exported;"
            )
        );
        assert!(source.contains("confirmed: Some(!export_ready)"));
        assert!(source.contains("ui::id::PROFILE_EXPORT_DIALOG"));
        assert!(source.contains("ui::action::PREVIEW_PROFILE_EXPORT"));
        assert!(source.contains("ui::action::BACKUP_PROFILE_EXPORT"));
        assert!(source.contains("ui::action::APPLY_PROFILE_EXPORT"));
        assert!(source.contains("ui::action::REVERT_PROFILE_EXPORT"));
        assert!(source.contains("class: \"profile-export-switch-row\""));
        assert!(source.contains("role: \"switch\""));
        assert!(source.contains("profile_export_merge_blocked"));
        assert!(source.contains("preview_profile_export(module_id, request)"));
        assert!(source.contains("analyze_profile_export(module_id, request)"));
        assert!(source.contains("backup_profile_export(module_id, request)"));
        assert!(source.contains("apply_profile_export(module_id, request)"));
        assert!(source.contains("revert_profile_export(module_id, request)"));
        assert!(source.contains("profile-export-preview-panel"));
        assert!(source.contains("profile-export-preview__metric-value--ready"));
        assert!(source.contains("profile-export-preview__metric-value--excluded"));
        assert!(source.contains("profile-export-preview__metric .profile-export-preview__muted"));
        assert!(source.contains("profile-export-preview__note"));
        assert!(source.contains("{dialog_profile_export_group_addresses}:"));
        assert!(source.contains("{dialog_profile_export_group_domains}:"));
        assert!(source.contains("{dialog_profile_export_group_ranges}:"));
        assert!(source.contains("{dialog_profile_export_uncovered_title}:"));
        assert!(source.contains("warning.code != \"profile_merge_target_skipped\""));
        assert!(source.contains("profile_export_repo_root_key"));
        assert!(source.contains("profile_export_integration_repo_key(&integration)"));
        assert!(source.contains("profile_export_attach_path_input"));
        assert!(source.contains("profile_export_patch_path_input"));
        assert!(source.contains("profile_export_merge_path_input"));
        assert!(
            source.contains("profile_export_mode.set(ExportModeDto::default())"),
            "profile export controls should reset to defaults when integration root changes"
        );
        assert!(source.contains("dialog.profile_export.mode_attach"));
        assert!(source.contains("dialog.profile_export.note_attach"));
        assert!(source.contains("ui::action::DELETE_OBSERVATION"));
        assert!(source.contains("ConfirmDeleteDialog"));
        assert!(
            source.contains("pending_delete_observation.set(Some(observation_for_delete.clone()))")
        );
        assert!(source.contains("dialog_id: ui::id::DELETE_OBSERVATION_DIALOG"));
        assert!(source.contains("ui::entity::DELETE_OBSERVATION_DIALOG"));
        assert!(source.contains("ui::action::CANCEL_DELETE_OBSERVATION"));
        assert!(source.contains("ui::action::CONFIRM_DELETE_OBSERVATION"));
        assert!(source.contains("dialog.delete_observation.title"));
        let confirm_dialog_source = source
            .split("fn ConfirmDeleteDialog")
            .nth(1)
            .expect("confirm delete dialog component");
        let confirm_header_source = confirm_dialog_source
            .split("div { class: \"modal__body\"")
            .next()
            .unwrap_or(confirm_dialog_source);
        assert!(
            !confirm_header_source.contains("help-text"),
            "confirm overlay help text belongs in modal body, not header"
        );
        assert!(
            confirm_dialog_source.contains("div { class: \"modal__body\"")
                && confirm_dialog_source.contains("p { class: \"help-text\""),
            "confirm overlay body should render the descriptive help text"
        );
        assert!(source.contains("current.delete_observation(DeleteObservationRequest"));
        assert!(source.contains("Ok(cleared_count) =>"));
        assert!(
            !source.contains("let cleared_count = ids.len();\n                        match current.delete_observations"),
            "clear all must report the backend deleted count, not the requested id count"
        );
        assert!(
            !source.contains("let cleared_count = ids.len();\n                            match current.delete_observations"),
            "clear selected must report the backend deleted count, not the requested id count"
        );
        let removed_start_id = ["START", "_MONITORING_BUTTON"].concat();
        let removed_stop_id = ["STOP", "_MONITORING_BUTTON"].concat();
        assert!(!render_source.contains(&removed_start_id));
        assert!(!render_source.contains(&removed_stop_id));
        assert!(
            source.contains("integration-dialog-path__input"),
            "integration dialog paths should be readonly input-like fields for long paths"
        );
        assert!(source.contains("dialog.integration.primary_path"));
        assert!(source.contains("dialog.integration.export_path"));
        assert!(
            !render_source.contains("integration-dialog-status-row--paths")
                && !render_source.contains("integration-dialog-paths"),
            "integration dialog should render direct path rows without a nested paths group"
        );
        assert!(source.contains("let integration_modules_bootstrap ="));
        assert!(
            source
                .contains("!snapshot.ui.snapshot_loaded && integration_module_buttons.is_empty()")
        );
        assert!(source.contains("let tracked_apps_bootstrap ="));
        assert!(
            source.contains("!snapshot.ui.snapshot_loaded && snapshot.tracked_apps.is_empty()"),
            "tracked apps skeletons should be shown only during cold bootstrap, not for a loaded empty Linux app list"
        );
        assert!(source.contains("if tracked_apps_bootstrap {"));
        assert!(
            !source.contains("if snapshot.tracked_apps.is_empty() {\n                                    for _ in 0..5"),
            "tracked apps must not keep loading skeletons forever when discovery legitimately finds zero apps"
        );
        assert!(render_source.contains("class: \"integration-module-button-skeleton\""));
        let removed_empty_state_key = ["integration", "missing"].join(".");
        let removed_empty_state_var = ["integration", "_missing"].concat();
        assert!(!source.contains(&removed_empty_state_key));
        assert!(!source.contains(&removed_empty_state_var));
        assert!(source.contains(
            "push_status_history_line(status_history, web_server_copied_message.clone());"
        ));
        assert!(
            source.contains(
                "push_status_history_line(status_history, footer_status_copied.clone());"
            )
        );
        let removed_footer_snapshot = ["last", "footer", "status", "lines"].join("_");
        assert!(!render_source.contains(&removed_footer_snapshot));
        assert!(source.contains("let web_server_tooltip = web_server_status_line("));
        assert!(source.contains("footer-watcher-status__indicator--connected"));
        assert!(source.contains("footer-watcher-status__indicator--disconnected"));
        let removed_state_class = ["footer-watcher-status__", "state"].join("");
        assert!(!source.contains(&removed_state_class));
        assert!(source.contains("\"data-tooltip\": \"{watcher_connection_tooltip}\""));
        assert!(source.contains("\"data-tooltip\": \"{web_server_tooltip}\""));
        assert!(source.contains("\"data-tooltip-align\": \"start\""));
        assert!(source.contains("watcher.read().tool_available()"));
        assert!(source.contains("snapshot.ui.watcher_connected"));
        assert!(source.contains("snapshot.app_settings.web_access_localhost"));
        assert!(source.contains("footer-watcher-status__indicator--inactive"));
        assert!(source.contains("class: \"{footer_status_text_class}\""));
        assert!(source.contains("footer-message-text--success"));
        assert!(source.contains("footer-message-text--warning"));
        assert!(source.contains("footer-message-text--error"));
        assert!(source.contains("\"aria-label\": \"{footer_message_label}\""));
        let removed_snapshot_history_state = ["initial", "footer", "events", "seeded"].join("_");
        assert!(
            !source.contains(&removed_snapshot_history_state),
            "desktop footer should not log periodic status snapshots as event history"
        );
        let removed_integration_note_state = ["status", "integration", "note"].join("_");
        assert!(
            !source.contains(&removed_integration_note_state),
            "desktop footer should not show the old integration note fallback"
        );
    }

    #[test]
    fn endpoint_probe_tooltip_lists_first_success_and_all_probe_states() {
        let status = crate::watcher_api::EndpointProbeStatusDto {
            is_checking: false,
            first_successful_target: Some("1.1.1.1:53".to_string()),
            probes: vec![
                crate::watcher_api::EndpointProbeTargetDto {
                    target: "8.8.8.8:53".to_string(),
                    available: Some(false),
                    error: Some("timeout".to_string()),
                },
                crate::watcher_api::EndpointProbeTargetDto {
                    target: "1.1.1.1:53".to_string(),
                    available: Some(true),
                    error: None,
                },
                crate::watcher_api::EndpointProbeTargetDto {
                    target: "[2606:4700:4700::1111]:53".to_string(),
                    available: None,
                    error: None,
                },
            ],
        };

        let tooltip = super::endpoint_probe_tooltip(
            &status,
            true,
            "First reachable DNS:",
            "All endpoint probes are unavailable",
            "endpoint probes are checking",
            "Waiting for watcher",
            "True",
            "False",
            "Checking",
        );

        assert!(tooltip.starts_with("First reachable DNS: 1.1.1.1:53"));
        assert!(tooltip.contains("8.8.8.8:53: False (timeout)"));
        assert!(tooltip.contains("1.1.1.1:53: True"));
        assert!(tooltip.contains("[2606:4700:4700::1111]:53: Checking"));
    }

    #[test]
    fn endpoint_probe_tooltip_repeats_first_failure_detail_in_summary_line() {
        let status = crate::watcher_api::EndpointProbeStatusDto {
            is_checking: false,
            first_successful_target: None,
            probes: vec![crate::watcher_api::EndpointProbeTargetDto {
                target: "udp://94.140.14.14:53".to_string(),
                available: Some(false),
                error: Some("connection timed out".to_string()),
            }],
        };

        let tooltip = super::endpoint_probe_tooltip(
            &status,
            true,
            "First reachable DNS:",
            "All endpoint probes are unavailable",
            "endpoint probes are checking",
            "Waiting for watcher",
            "True",
            "False",
            "Checking",
        );

        assert!(tooltip.starts_with(
            "All endpoint probes are unavailable: udp://94.140.14.14:53: connection timed out"
        ));
    }

    #[test]
    fn status_history_tooltip_counts_only_hidden_lines() {
        let six_lines = (1..=6)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>();
        assert_eq!(status_history_tooltip(&six_lines), six_lines.join("\n"));

        let seven_lines = (1..=7)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>();
        assert_eq!(
            status_history_tooltip(&seven_lines),
            "...1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7"
        );

        let hundred_lines = (1..=100)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>();
        assert_eq!(
            status_history_tooltip(&hundred_lines),
            "...94\nline 95\nline 96\nline 97\nline 98\nline 99\nline 100"
        );
    }

    #[test]
    fn ordinary_text_inputs_commit_explicitly_instead_of_on_each_keypress() {
        let desktop_source = include_str!("app.rs");
        let browser_source = include_str!("../../netstitch-watcher/src/lib.rs");
        let ordinary_desktop_source = desktop_source
            .split("fn IntegrationUiEntityView(")
            .next()
            .expect("ordinary desktop source before module renderer");
        let input_commit_script = {
            let start = desktop_source
                .find("const INPUT_COMMIT_SCRIPT")
                .expect("input commit script");
            let end = desktop_source[start..]
                .find("fn pulse_text_input")
                .map(|offset| start + offset)
                .expect("input commit script end");
            &desktop_source[start..end]
        };

        assert!(
            !ordinary_desktop_source.contains("oninput:"),
            "ordinary app inputs must not use Dioxus oninput; per-key updates are reserved for module UI fields"
        );
        assert!(
            desktop_source.contains("\"data-committed-value\": \"{header_ip_filter_value}\"")
                && desktop_source
                    .contains("\"data-committed-value\": \"{header_domain_filter_value}\"")
                && desktop_source
                    .contains("\"data-committed-value\": \"{header_port_filter_value}\"")
                && desktop_source
                    .contains("\"data-committed-value\": \"{pending_exe_path_value}\"")
                && desktop_source
                    .contains("\"data-committed-value\": \"{cloud_app_search_draft()}\"")
                && desktop_source
                    .contains("\"data-committed-value\": \"{cloud_upload_nickname_draft()}\"")
                && desktop_source.contains(
                    "\"data-committed-value\": \"{profile_export_advanced_manual_domains()}\""
                ),
            "ordinary editable app inputs must expose committed values through DOM data, not Dioxus value"
        );
        let lines = desktop_source.lines().collect::<Vec<_>>();
        for (line_index, line) in lines.iter().enumerate() {
            if !line.contains("\"data-preserve-draft\": \"true\"") {
                continue;
            }
            let window_start = line_index.saturating_sub(8);
            let preceding_control_lines = lines[window_start..line_index].join("\n");
            assert!(
                !preceding_control_lines.contains("value: \"{"),
                "data-preserve-draft inputs must not be controlled by Dioxus value near line {}",
                line_index + 1
            );
            assert!(
                preceding_control_lines.contains("\"data-committed-value\":"),
                "data-preserve-draft inputs must carry data-committed-value near line {}",
                line_index + 1
            );
        }
        assert!(
            desktop_source.contains("fn IntegrationUiEntityView(")
                && desktop_source.contains("module_ui_values.write().insert(\n                                    input_entity_id.clone(),\n                                    serde_json::Value::String(event.value()),\n                                );"),
            "module UI inputs remain the explicit exception and may keep their existing Dioxus oninput behavior"
        );
        assert!(
            browser_source.contains("if (document.activeElement !== ipFilter) ipFilter.value = text(filters.ip_search);")
                && browser_source.contains("if (document.activeElement !== domainFilter) domainFilter.value = text(filters.domain_search);")
                && browser_source.contains("if (document.activeElement !== portFilter) portFilter.value = text(filters.port_search);"),
            "browser header text filters must not overwrite focused user input during renders"
        );
        assert!(
            desktop_source.contains("const INPUT_COMMIT_SCRIPT")
                && desktop_source.contains("\"data-commit-on-enter\": \"true\""),
            "desktop fields that apply on Enter must use the shared commit bridge"
        );
        assert!(
            input_commit_script.contains("document.addEventListener('input', (event) => {")
                && input_commit_script.contains("syncClearButton(event.target);")
                && input_commit_script.contains("const committedObserver = new MutationObserver")
                && input_commit_script.contains("if (document.activeElement === control)")
                && !input_commit_script.contains("restoreFocusedDraft")
                && !input_commit_script.contains("focusedDraftGuard"),
            "desktop clear-button dimming and committed-value sync must be DOM-only and must not commit app state on every keystroke"
        );
        assert!(
            browser_source.contains("document.addEventListener('keydown', (event) => {")
                && browser_source.contains("if (target.dataset?.commitOnEnter !== 'true') return;")
                && browser_source.contains("if (target.getAttribute('onkeydown')) return;"),
            "browser commit-on-enter bridge must be available for module fields without duplicating explicit app handlers"
        );
        assert!(
            desktop_source.contains("\"data-enter-click-target\": ui::id::ADD_EXE_BUTTON")
                && desktop_source
                    .contains("const clickTargetId = target.dataset.enterClickTarget;"),
            "manual executable path Enter must route through the same action as the plus button"
        );
        assert!(
            desktop_source.contains("pulse_text_input(input_apply_pulse, \"header-ip\")")
                && desktop_source
                    .contains("pulse_text_input(input_apply_pulse, \"header-domain\")")
                && desktop_source.contains("pulse_text_input(input_apply_pulse, \"header-port\")"),
            "main header filters must keep the existing Enter pulse feedback"
        );
    }

    #[test]
    fn app_clear_buttons_dim_when_bound_fields_are_empty() {
        let source = include_str!("app.rs").replace('\r', "");
        let theme = include_str!("theme.rs").replace('\r', "");

        for token in [
            "let clear_header_ip_filter_disabled = header_ip_filter_value.is_empty();",
            "let clear_header_domain_filter_disabled = header_domain_filter_value.is_empty();",
            "let clear_header_port_filter_disabled = header_port_filter_value.is_empty();",
            "let clear_pending_exe_path_disabled = pending_exe_path_value.is_empty();",
            "let clear_integration_path_disabled = integration_path_input().is_empty();",
            "let clear_cloud_app_search_disabled = cloud_app_search_draft().is_empty();",
            "let clear_cloud_publisher_search_disabled = cloud_publisher_search_draft().is_empty();",
            "let clear_cloud_source_search_disabled = cloud_source_search_draft().is_empty();",
            "disabled: clear_header_ip_filter_disabled,",
            "disabled: clear_header_domain_filter_disabled,",
            "disabled: clear_header_port_filter_disabled,",
            "disabled: clear_pending_exe_path_disabled,",
            "disabled: clear_integration_path_disabled,",
            "disabled: clear_cloud_app_search_disabled,",
            "disabled: clear_cloud_publisher_search_disabled,",
            "disabled: clear_cloud_source_search_disabled,",
            "disabled: profile_export_attach_path_draft().is_empty(),",
            "disabled: profile_export_patch_path_draft().is_empty(),",
            "disabled: profile_export_merge_path_draft().is_empty(),",
            "disabled: profile_export_advanced_manual_domains().is_empty(),",
            "\"data-clear-button\": \"true\",",
            "const hasClearButton = (control) => control?.dataset?.clearButton === 'true';",
            "const clearButtonForControl = (control) => {",
            "if (!hasClearButton(control)) return null;",
            "child.dataset?.clearButton === 'true'",
            "button.disabled = control.value.length === 0 || control.disabled || control.readOnly;",
            "document.addEventListener('input', (event) => {",
            "document.addEventListener('click', (event) => {",
            "button.dataset?.clearButton !== 'true'",
            "control.value = '';",
            "control.dispatchEvent(new Event('input', { bubbles: true }));",
            "document.querySelectorAll('.path-input-shell input[data-clear-button=\"true\"], .path-input-shell textarea[data-clear-button=\"true\"]')",
        ] {
            assert!(
                source.contains(token),
                "desktop clear buttons should keep empty-field disabled token {token}"
            );
        }

        assert!(
            theme.contains(".path-input-clear:disabled {\n  cursor: default;\n  opacity: 0.32;"),
            "desktop clear button disabled state should stay visually dimmed"
        );
    }

    #[test]
    fn module_ui_tabs_keep_page_scoped_grid_children() {
        let entities: Vec<IntegrationUiEntityDto> = serde_json::from_value(serde_json::json!([
            {
                "id": "tabs",
                "entity_type": "tabs",
                "value": "ui_entities",
                "children": [
                    {
                        "id": "controls",
                        "entity_type": "grid",
                        "page": "ui_entities",
                        "children": [
                            {
                                "id": "input",
                                "entity_type": "text_input",
                                "page": "ui_entities",
                                "value": "editable"
                            },
                            {
                                "id": "disabled",
                                "entity_type": "text_input",
                                "page": "ui_entities",
                                "disabled": true
                            }
                        ]
                    },
                    {
                        "id": "background",
                        "entity_type": "table",
                        "page": "background_subscription"
                    }
                ]
            }
        ]))
        .expect("module UI schema fixture should deserialize");

        let schema = module_ui_schema_with_context(&entities, "main", 0, 0, 0, false, "-", "", 0);
        let tabs = schema
            .iter()
            .find(|entity| entity.id == "tabs")
            .expect("tabs entity should survive main page filtering");
        let active_children = module_ui_active_tab_children(tabs, "ui_entities");
        let grid = active_children
            .iter()
            .find(|entity| entity.id == "controls")
            .expect("active tab should include page-scoped grid");

        assert_eq!(
            grid.children.len(),
            2,
            "page-scoped grid must keep same-page child controls"
        );
    }

    #[test]
    fn module_ui_progress_stages_support_phase_boundaries_and_colors() {
        let entity: IntegrationUiEntityDto = serde_json::from_value(serde_json::json!({
            "id": "phase-progress",
            "entity_type": "progress",
            "value": "35%",
            "progress_stages": [
                { "color": "accent", "percent": 30, "name": "Queued" },
                { "color": "rust", "percent": "40%", "name": "Processing" },
                { "color": "#44aa88", "percent": 100, "name": "Done" }
            ]
        }))
        .expect("progress entity should deserialize");

        assert_eq!(module_ui_parse_progress_percent("35%"), 35);
        let stages = module_ui_progress_stages(&entity);
        assert_eq!(
            stages.iter().map(|stage| stage.width).collect::<Vec<_>>(),
            vec![30, 10, 60]
        );
        assert_eq!(stages[1].class_name, "progress-bar__segment--rust");
        assert_eq!(stages[2].style, "background: #44aa88;");
        let normalized = normalize_progress_stages(stages);
        assert_eq!(progress_current_stage_label(&normalized, 35), "Processing");
        assert_eq!(
            progress_current_text(35, &progress_current_stage_label(&normalized, 35)),
            "35% | Processing"
        );

        let compact_visual = merge_adjacent_progress_stages(vec![
            ProgressStage::new(String::new(), 50, "progress-bar__segment--accent"),
            ProgressStage::new(String::new(), 50, "progress-bar__segment--accent"),
        ]);
        assert_eq!(compact_visual.len(), 1);
        assert_eq!(compact_visual[0].width, 100);
    }

    #[test]
    fn module_ui_controls_reuse_standard_input_contracts() {
        let desktop_source = include_str!("app.rs").replace('\r', "");
        let browser_source = include_str!("../../netstitch-watcher/src/lib.rs").replace('\r', "");
        let desktop_theme = include_str!("theme.rs").replace('\r', "");
        let browser_theme = browser_source.as_str();

        for token in [
            "class: \"path-input-shell module-ui-schema__input-shell\"",
            "class: \"input-box input module-ui-schema__input\"",
            "class: \"path-input-clear module-ui-schema__clear\"",
            "\"data-clear-button\": \"{clear_enabled}\"",
            "\"data-commit-on-enter\": \"{entity.commit_on_enter}\"",
            "\"data-clear-button\": \"true\"",
            "let clear_enabled = entity.clear_button;",
            "if clear_enabled {",
            "class: \"path-input-shell module-ui-schema__input-shell module-ui-schema__input-shell--textarea\"",
            "class: \"input-box input module-ui-schema__textarea\"",
            "class: \"path-input-clear module-ui-schema__clear module-ui-schema__clear--textarea\"",
            "class: \"input-box select module-ui-schema__select\"",
            "class: \"module-ui-schema__table {layout_class}\"",
            "class: \"module-ui-schema__table-frame table-wrap\"",
            "class: \"table-body-wrap module-ui-schema__table-body {scroll_class}\"",
            "class: \"path-field module-ui-schema__table-cell-field\"",
            "module_ui_table_shell_style(&entity)",
            "module_ui_table_viewport_style(&entity)",
            "module_ui_table_column_style(&entity, column_index)",
            "module_ui_table_column_text_field(&entity, column_index)",
            "module_ui_textarea_row_style(&entity)",
            "module_ui_textarea_control_style(&entity)",
            "module_ui_grid_style(&entity)",
            "(\"height\", entity.height.as_deref())",
            "(\"min-height\", entity.min_height.as_deref())",
            "(\"max-height\", entity.max_height.as_deref())",
            "(\"grid-template-columns\", entity.columns.as_deref())",
            "(\"grid-template-rows\", entity.rows.as_deref())",
            "(\"gap\", entity.gap.as_deref())",
            "(\"grid-column\", entity.grid_column.as_deref())",
            "(\"grid-row\", entity.grid_row.as_deref())",
            "module_ui_button_row_style(&entity)",
            "module_ui_button_content_style(&entity)",
            "module_ui_entity_with_layout_defaults(entity)",
            "set_option_if_blank(&mut entity.size, \"stretch\")",
            "set_option_if_blank(&mut entity.width, \"100%\")",
            "set_option_if_blank(&mut entity.min_width, \"0\")",
            "set_string_if_blank(&mut entity.opacity, \"100%\")",
            "set_option_if_blank(&mut entity.height, \"auto\")",
            "set_option_if_blank(&mut entity.min_height, \"0\")",
            "\"separator\" | \"help-text\" | \"help\" | \"table\" | \"progress\" | \"footer\" =>",
            "repeat(auto-fit, minmax(180px, 1fr))",
            "set_option_if_blank(&mut entity.margin, \"8px 0 0\")",
            "ui_action_token: String::new()",
            "start_module_ui_action(",
            "start_integration_module_ui_action_job(request)",
            "request.ui_action_token = ui_action_token.clone();",
            "fn module_ui_action_token(",
            "poll_module_ui_action_events(",
            "integration_module_ui_action_events(",
            "event.event_type == \"ui_values\"",
            "apply_module_ui_values_payload(&event.payload",
            "module_ui_action_generation.set(module_ui_action_generation().wrapping_add(1))",
            "module_ui_footer_entities(",
            "hide_host_back_button",
            "data-ui-entity\": \"grid\"",
            "entity.table_columns.iter().find(|column| column.index == index)",
            "module_ui_progress_stages(&entity)",
            "module_ui_parse_progress_percent(&value)",
            "progress_current_text(percent, &current_stage)",
            "let titleless_row_class = if title.is_empty()",
            "module-ui-schema__row--no-title",
        ] {
            assert!(
                desktop_source.contains(token),
                "desktop module UI controls must keep standard control token {token}"
            );
        }

        for token in [
            "pub opacity: String",
            "pub clear_button: bool",
            "pub commit_on_enter: bool",
            "pub hide_host_back_button: bool",
            "pub progress_stages: Vec<IntegrationUiProgressStageDto>",
            "module_ui_sanitize_opacity",
        ] {
            assert!(
                desktop_source.contains(token)
                    || include_str!("../../netstitch-shared/src/models.rs").contains(token),
                "desktop module UI contract must keep opacity token {token}"
            );
        }

        for token in [
            "<div class=\"path-input-shell module-ui-schema__input-shell\"><input class=\"input-box input module-ui-schema__input\"",
            "<button class=\"path-input-clear module-ui-schema__clear\"",
            "<div class=\"path-input-shell module-ui-schema__input-shell module-ui-schema__input-shell--textarea\"><textarea class=\"input-box input module-ui-schema__textarea\"",
            "<button class=\"path-input-clear module-ui-schema__clear module-ui-schema__clear--textarea\"",
            "const clearDisabled = !controlValue || disabled || readonly;",
            "const clearEnabled = entity.clear_button === true;",
            "const clearAttr = clearEnabled ? ' data-clear-button=\"true\"' : '';",
            "const commitAttr = entity.commit_on_enter === true ? ' data-commit-on-enter=\"true\"' : '';",
            "const clearHtml = clearEnabled ? '<button class=\"path-input-clear module-ui-schema__clear\"",
            "const clearHtml = clearEnabled ? '<button class=\"path-input-clear module-ui-schema__clear module-ui-schema__clear--textarea\"",
            "<select class=\"input-box select module-ui-schema__select\"",
            "function moduleUiTableShellStyle(entity)",
            "function moduleUiTableViewportStyle(entity)",
            "function moduleUiTableHtml(entity, value, id, scrollClass = '', styleAttr = '')",
            "function moduleUiTableColumnTextField(entity, index)",
            "function moduleUiTextareaRowStyle(entity)",
            "function moduleUiTextareaControlStyle(entity)",
            "module-ui-schema__table-cell-field",
            "function moduleUiProgressStages(entity)",
            "function progressBarCurrentText(percent, stageLabel)",
            "function parseProgressPercent(value)",
            "progress_stages",
            "moduleUiActionPollTokens: {}",
            "function moduleUiActionToken(",
            "function pollModuleUiActionEvents(",
            "/v1/integrations/ui-action-events",
            "ui_action_token: actionToken",
            "text(event?.event_type) === 'ui_values'",
            "state.moduleUiActionPollTokens = { ...(state.moduleUiActionPollTokens || {}), [actionToken]: true };",
            "state.moduleUiActionPollTokens = { ...(state.moduleUiActionPollTokens || {}), [actionToken]: false };",
            "function moduleUiGridStyle(entity)",
            "['height', 'height'],",
            "['min_height', 'min-height'],",
            "['max_height', 'max-height'],",
            "['columns', 'grid-template-columns'],",
            "['rows', 'grid-template-rows'],",
            "['gap', 'gap'],",
            "['grid_column', 'grid-column'],",
            "['grid_row', 'grid-row']",
            "function moduleUiTableViewportStyle(entity) {\n      return '';",
            "function moduleUiButtonRowStyle(entity)",
            "function moduleUiButtonContentStyle(entity)",
            "function moduleUiEntityWithLayoutDefaults(entity)",
            "setDefault('size', 'stretch');",
            "setDefault('width', '100%');",
            "setDefault('min_width', '0');",
            "setDefault('opacity', '100%');",
            "setDefault('height', 'auto');",
            "setDefault('min_height', '0');",
            "['separator', 'help-text', 'help', 'table', 'progress', 'footer'].includes(type)",
            "setDefault('columns', 'repeat(auto-fit, minmax(180px, 1fr))');",
            "state.moduleUiActionGeneration += 1;",
            "if (state.moduleUiActionGeneration !== generation) return;",
            "function moduleUiFooterEntities(entities)",
            "function moduleUiFooterHidesHostBack(entities, context = {})",
            "hide_host_back_button === true",
            "integration-module-modal-footer-actions",
            "module-ui-schema__table-frame table-wrap",
            "table-body-wrap module-ui-schema__table-body",
            "module-ui-schema__button-row-content",
            "module-ui-schema__footer-nav",
            "const opacity = moduleUiSanitizedOpacity(entity?.opacity);",
            "parts.push('opacity: ' + opacity);",
            ".module-ui-schema__footer > .module-ui-schema__actions",
            "width: auto;",
            "const rowClass = 'module-ui-schema__row' + (title ? '' : ' module-ui-schema__row--no-title') + sizeClass;",
        ] {
            assert!(
                browser_source.contains(token),
                "browser module UI controls must keep standard control token {token}"
            );
        }

        for token in [
            ".module-ui-schema__input-shell {\n  position: relative;\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) var(--size-close-button);",
            ".module-ui-schema__input,\n.module-ui-schema__select {\n  width: 100%;\n  height: var(--size-compact-control);\n  min-height: var(--size-compact-control);",
            ".module-ui-schema__grid {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));",
            ".module-ui-schema__input-shell > .input {\n  grid-column: 1 / -1;\n  grid-row: 1;",
            ".path-input-clear:disabled {\n  cursor: default;\n  opacity: 0.32;",
            ".progress-bar__segment--rust",
            ".progress-bar__current",
            ".module-ui-schema__progress--compact-labeled {\n  grid-template-columns: minmax(120px, auto) minmax(220px, 1fr);",
            ".module-ui-schema__row--textarea {\n  align-items: start;\n  min-height: 0;",
            ".module-ui-schema__row--no-title {\n  grid-template-columns: minmax(0, 1fr) auto;",
            ".module-ui-schema__clear {\n  position: absolute;\n  top: 50%;",
            "transform: translateY(-50%);",
            ".module-ui-schema__clear--textarea {\n  top: 6px;\n  transform: none;",
            ".module-ui-schema__table-cell-field.path-field {\n  display: block;\n  width: 100%;",
            "overflow-x: hidden;\n  overflow-y: hidden;\n  text-overflow: ellipsis;",
            ".module-ui-schema__button-row {\n  display: grid;\n  width: 100%;\n  min-width: 0;\n  min-height: var(--size-compact-control);",
            ".module-ui-schema__button-row {\n  display: grid;\n  width: 100%;\n  min-width: 0;\n  min-height: var(--size-compact-control);\n  margin: 8px 0 0;\n  padding: 0;\n  align-items: center;\n  align-self: stretch;\n  align-content: center;\n  box-sizing: border-box;\n  clear: both;\n  overflow: visible;",
            ".module-ui-schema__button-row-content {\n  display: grid;\n  width: 100%;\n  min-width: 0;\n  min-height: var(--size-compact-control);",
            ".module-ui-schema__tabs-body > .module-ui-schema__button-row {\n  min-height: var(--size-compact-control);",
            ".module-ui-schema__table {\n  display: grid;\n  width: 100%;\n  min-width: 0;",
            "grid-template-rows: auto minmax(0, 1fr) auto;\n  gap: 0;\n  align-content: stretch;\n  align-self: stretch;\n  overflow: hidden;",
            ".module-ui-schema__table-frame,\n.module-ui-schema__table-frame.table-wrap {\n  display: flex;\n  flex-direction: column;",
            ".module-ui-schema__table-body.module-ui-schema--scroll-y {\n  overflow-x: hidden;\n  overflow-y: auto;",
            ".module-ui-schema__tabs {\n  display: flex;\n  flex-direction: column;\n  gap: 8px;\n  min-width: 0;\n  min-height: 0;\n  box-sizing: border-box;\n  overflow: hidden;",
            ".module-ui-schema__tabs-body {\n  display: grid;\n  align-content: start;\n  gap: 8px;\n  flex: 1 1 auto;\n  min-width: 0;\n  min-height: 0;\n  box-sizing: border-box;\n  overflow: hidden;",
            ".ui-entity-table {\n  width: 100%;\n  min-width: 100%;",
            ".ui-entity-table th {\n  padding: 1px 10px;",
            ".ui-entity-table td {\n  padding: 2px 10px;",
            ".module-ui-schema__footer {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 8px;\n  width: 100%;",
            ".module-ui-schema__footer-host {\n  display: flex;\n  align-items: center;\n  gap: 8px;",
            ".module-ui-schema__footer > .module-ui-schema__actions {\n  flex: 0 0 auto;\n  width: auto;",
            ".integration-module-dialog > .modal__footer[data-ui-entity=\"panel-footer\"] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) auto;",
        ] {
            assert!(
                desktop_theme.contains(token),
                "desktop module UI theme must keep header-sized control token {token}"
            );
        }

        for token in [
            ".module-ui-schema__button-row {\n      display: grid;\n      width: 100%;\n      min-width: 0;\n      min-height: var(--size-compact-control);",
            ".module-ui-schema__grid {\n      display: grid;\n      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));",
            ".path-input-clear:disabled {\n      cursor: default;\n      opacity: 0.32;",
            ".progress-bar__segment--rust",
            ".progress-bar__current",
            ".module-ui-schema__progress--compact-labeled {\n      grid-template-columns: minmax(120px, auto) minmax(220px, 1fr);",
            ".module-ui-schema__row--textarea {\n      align-items: start;\n      min-height: 0;",
            ".module-ui-schema__row--no-title {\n      grid-template-columns: minmax(0, 1fr) auto;",
            ".module-ui-schema__clear {\n      position: absolute;\n      top: 50%;",
            "transform: translateY(-50%);",
            ".module-ui-schema__clear--textarea {\n      top: 6px;\n      transform: none;",
            ".module-ui-schema__table-cell-field.path-field {\n      display: block;\n      width: 100%;",
            ".module-ui-schema__button-row {\n      display: grid;\n      width: 100%;\n      min-width: 0;\n      min-height: var(--size-compact-control);\n      margin: 8px 0 0;\n      padding: 0;\n      align-items: center;\n      align-self: stretch;\n      align-content: center;\n      box-sizing: border-box;\n      clear: both;\n      overflow: visible;",
            ".module-ui-schema__button-row-content {\n      display: grid;\n      width: 100%;\n      min-width: 0;\n      min-height: var(--size-compact-control);",
            ".module-ui-schema__tabs-body > .module-ui-schema__button-row {\n      min-height: var(--size-compact-control);",
            ".module-ui-schema__footer {\n      display: flex;\n      align-items: center;\n      justify-content: space-between;\n      gap: 8px;\n      width: 100%;",
            ".module-ui-schema__footer-host {\n      display: flex;\n      align-items: center;\n      gap: 8px;",
            ".module-ui-schema__footer > .module-ui-schema__actions {\n      flex: 0 0 auto;\n      width: auto;",
            ".integration-module-dialog > .modal__footer[data-ui-entity=\"panel-footer\"] {\n      display: grid;\n      grid-template-columns: minmax(0, 1fr) auto;",
            ".module-ui-schema__table {\n      display: grid;\n      width: 100%;\n      min-width: 0;",
            "grid-template-rows: auto minmax(0, 1fr) auto;\n      gap: 0;\n      align-content: stretch;\n      align-self: stretch;\n      overflow: hidden;",
            ".module-ui-schema__table-frame,\n    .module-ui-schema__table-frame.table-wrap {\n      display: flex;\n      flex-direction: column;",
            ".module-ui-schema__table-body.module-ui-schema--scroll-y {\n      overflow-x: hidden;\n      overflow-y: auto;",
            ".module-ui-schema__tabs {\n      display: flex;\n      flex-direction: column;\n      gap: 8px;\n      min-width: 0;\n      min-height: 0;\n      box-sizing: border-box;\n      overflow: hidden;",
            ".module-ui-schema__tabs-body {\n      display: grid;\n      align-content: start;\n      gap: 8px;\n      flex: 1 1 auto;\n      min-width: 0;\n      min-height: 0;\n      box-sizing: border-box;\n      overflow: hidden;",
        ] {
            assert!(
                browser_theme.contains(token),
                "browser module UI button rows must reserve their own height token {token}"
            );
        }

        let desktop_button_size_bypass = [
            "if matches!(entity_type, \"button\"",
            " | \"action-button\")",
        ]
        .concat();
        let browser_button_size_bypass = [
            "if (type === 'button' || type === ",
            "'action-button') return '';",
        ]
        .concat();
        for forbidden in [
            desktop_button_size_bypass.as_str(),
            browser_button_size_bypass.as_str(),
        ] {
            assert!(
                !desktop_source.contains(forbidden) && !browser_source.contains(forbidden),
                "module button/action_button entities must use the same size/alignment contract as other UI entities"
            );
        }

        let old_table_class = ["module-ui-schema__", "table-grid"].concat();
        for forbidden in [
            old_table_class.as_str(),
            "min-width: 720px",
            "module-ui-schema__table-viewport",
            "module-ui-schema__table-grid",
            "module-ui-schema__panel module-ui-schema__table",
        ] {
            assert!(
                !desktop_theme.contains(forbidden),
                "module table must not use custom table styling token {forbidden}"
            );
            assert!(
                !browser_theme.contains(forbidden),
                "browser module table must not use custom table styling token {forbidden}"
            );
        }
    }

    #[test]
    fn status_history_keeps_url_copy_as_latest_log_line() {
        let mut history = vec![
            StatusHistoryLine::info("Current status: Stopped"),
            StatusHistoryLine::info("Tracked apps: 0\\0"),
        ];

        push_status_history_line_to_vec(
            &mut history,
            "Web server address copied to clipboard: http://127.0.0.1:46473",
        );

        assert_eq!(
            history.last().map(|line| line.text.as_str()),
            Some("Web server address copied to clipboard: http://127.0.0.1:46473")
        );
    }

    #[test]
    fn status_history_removes_trailing_periods_from_log_lines() {
        let mut history = Vec::new();

        push_status_history_line_to_vec(&mut history, "Monitoring started.");
        push_status_history_line_to_vec(&mut history, "Monitoring stopped...");

        assert_eq!(
            history,
            vec![
                StatusHistoryLine::info("Monitoring started"),
                StatusHistoryLine::info("Monitoring stopped")
            ]
        );
    }

    #[test]
    fn status_history_loads_persisted_lines_only_while_empty() {
        let mut history = Vec::new();
        let persisted = vec![
            StatusHistoryLine::info("Build version"),
            StatusHistoryLine::success("Watcher connected"),
        ];

        assert!(super::merge_persisted_status_history_if_empty(
            &mut history,
            persisted.clone()
        ));
        assert_eq!(history, persisted);

        let later_persisted = vec![StatusHistoryLine::error("Delayed storage line")];
        assert!(!super::merge_persisted_status_history_if_empty(
            &mut history,
            later_persisted
        ));
        assert_eq!(
            history,
            vec![
                StatusHistoryLine::info("Build version"),
                StatusHistoryLine::success("Watcher connected"),
            ]
        );
    }

    #[test]
    fn cloud_status_history_classifies_success_warning_and_error() {
        assert_eq!(
            super::cloud_status_history_kind("{\"ok\":true}", false),
            super::StatusHistoryKind::Success
        );
        assert_eq!(
            super::cloud_status_history_kind(
                "{\"error\":{\"code\":\"nickname_taken\",\"message\":\"Nickname is already registered\"}}",
                true,
            ),
            super::StatusHistoryKind::Warning
        );
        assert_eq!(
            super::cloud_status_history_kind("cloud request failed", true),
            super::StatusHistoryKind::Error
        );
    }

    #[test]
    fn cloud_status_messages_are_human_readable() {
        let labels = super::CloudMessageLabels {
            refresh_done: "Облако обновлено".to_string(),
            download: "Загрузка".to_string(),
            upload: "Выгрузка".to_string(),
            rows: "строк".to_string(),
            requests: "запросов".to_string(),
            skipped_non_public: "пропущено non-public".to_string(),
        };

        assert_eq!(
            super::humanize_cloud_status_message("{\"download\":{\"rows\":390}}", &labels),
            "Загрузка: 390 строк"
        );
        assert_eq!(
            super::humanize_cloud_status_message(
                "{\"upload\":{\"accepted_rows\":549,\"request_count\":2}}",
                &labels,
            ),
            "Выгрузка: 549 строк, запросов: 2"
        );
        assert_eq!(
            super::humanize_cloud_status_message(
                "{\"apps\":{\"items\":[]},\"health\":{\"ok\":true},\"quota\":{}}",
                &labels,
            ),
            "Облако обновлено"
        );
        assert_eq!(
            super::humanize_cloud_status_message(
                "{\"error\":{\"code\":\"invalid_batch_size\",\"message\":\"Rows count must be 1..65535\"}}",
                &labels,
            ),
            "Rows count must be 1..65535 (invalid_batch_size)"
        );
    }

    #[test]
    fn status_history_auto_classifies_common_footer_messages() {
        assert_eq!(
            super::infer_status_history_kind("Соединение с watcher установлено"),
            super::StatusHistoryKind::Success
        );
        assert_eq!(
            super::infer_status_history_kind("Нет выбранных строк для CSV"),
            super::StatusHistoryKind::Warning
        );
        assert_eq!(
            super::infer_status_history_kind("Ошибка CSV импорта: invalid row"),
            super::StatusHistoryKind::Error
        );
        assert_eq!(
            super::infer_status_history_kind("Запрошен экспорт подтверждённых IP"),
            super::StatusHistoryKind::Info
        );
    }

    #[test]
    fn footer_messages_use_system_event_history_without_snapshot_fallback() {
        let current_status_lines = vec![
            "Current status: Embedded monitoring runtime is starting".to_string(),
            "Tracked apps: 0\\0".to_string(),
        ];
        let mut event_history = Vec::new();

        assert_eq!(footer_message_text(&event_history), "");
        assert_eq!(footer_message_tooltip(&event_history), "");
        assert_eq!(footer_message_copy_text(&event_history), "");
        assert!(
            !footer_message_copy_text(&event_history).contains(&current_status_lines.join("\n"))
        );

        push_status_history_line_to_vec(
            &mut event_history,
            "Web server address copied to clipboard: http://127.0.0.1:46473",
        );

        assert_eq!(
            footer_message_text(&event_history),
            "Web server address copied to clipboard: http://127.0.0.1:46473"
        );
        assert_eq!(
            footer_message_copy_text(&event_history),
            "Web server address copied to clipboard: http://127.0.0.1:46473"
        );
        assert_eq!(
            footer_message_tooltip(&event_history),
            "Web server address copied to clipboard: http://127.0.0.1:46473"
        );
        assert_eq!(event_history.len(), 1);
    }

    #[test]
    fn footer_copy_uses_full_hundred_line_event_history() {
        let event_history = (1..=100)
            .map(|index| format!("event {index}"))
            .collect::<Vec<_>>();
        let current_status_lines = vec![
            "Current status: Monitoring paused".to_string(),
            "DNS status: Available".to_string(),
        ];

        assert_eq!(footer_message_text(&event_history), "event 100");
        assert_eq!(
            footer_message_copy_text(&event_history),
            event_history.join("\n")
        );
        assert!(!footer_message_copy_text(&event_history).contains(&current_status_lines[0]));
    }

    #[test]
    fn web_server_event_line_omits_trailing_slash() {
        let url = BrowserUiUrl {
            url: "http://127.0.0.1:46473/".to_string(),
            used_localhost_fallback: true,
        };

        assert_eq!(
            web_server_event_line("Web server enabled:", &url),
            "Web server enabled: http://127.0.0.1:46473"
        );
    }

    #[test]
    fn dns_status_line_reports_checking_before_first_success() {
        let status = crate::watcher_api::EndpointProbeStatusDto {
            is_checking: true,
            first_successful_target: None,
            probes: Vec::new(),
        };

        assert_eq!(
            dns_status_line(
                "DNS status",
                &status,
                true,
                "Available",
                "Unavailable",
                "Checking",
                "Waiting for watcher",
                "netstitch-tool native v1.1.0",
            ),
            "DNS status: Checking (netstitch-tool native v1.1.0)"
        );
    }

    #[test]
    fn dns_status_line_waits_for_watcher_before_reporting_unavailable() {
        let status = crate::watcher_api::EndpointProbeStatusDto {
            is_checking: false,
            first_successful_target: None,
            probes: Vec::new(),
        };

        assert_eq!(
            dns_status_line(
                "DNS status",
                &status,
                false,
                "Available",
                "Unavailable",
                "Checking",
                "Waiting for watcher",
                "netstitch-tool native v1.1.0",
            ),
            "DNS status: Waiting for watcher (netstitch-tool native v1.1.0)"
        );
    }

    #[test]
    fn dns_status_line_reports_first_probe_failure_detail_when_unavailable() {
        let status = crate::watcher_api::EndpointProbeStatusDto {
            is_checking: false,
            first_successful_target: None,
            probes: vec![crate::watcher_api::EndpointProbeTargetDto {
                target: "udp://94.140.14.14:53".to_string(),
                available: Some(false),
                error: Some("connection timed out".to_string()),
            }],
        };

        assert_eq!(
            dns_status_line(
                "DNS status",
                &status,
                true,
                "Available",
                "Unavailable",
                "Checking",
                "Waiting for watcher",
                "netstitch-tool native v1.1.0",
            ),
            "DNS status: Unavailable (udp://94.140.14.14:53: connection timed out; netstitch-tool native v1.1.0)"
        );
    }

    #[test]
    fn integration_dialog_preview_reuses_configured_export_path_for_current_folder() {
        let current = IntegrationIntegrationDto {
            repo_path: r"C:\Tools\integration".to_string(),
            export_path: r"C:\Tools\integration\data\user-export.txt".to_string(),
            reference_data_path: r"C:\Tools\integration\data\reference-data.txt".to_string(),
            profile_paths: Vec::new(),
            ready: true,
            status_text: "Integration folder configured".to_string(),
            last_export_text: String::new(),
            provider_id: None,
            provider_name: Some("Sample integration".to_string()),
            repository_url: None,
        };

        let preview = integration_dialog_preview(
            r"C:/Tools/integration/",
            &current,
            "Module configured",
            "Module not specified",
            "Module not found",
            "Path entered",
            "Not specified",
        );

        assert_eq!(preview.status_text, "Module configured");
        assert_eq!(preview.repo_path, current.repo_path);
        assert_eq!(preview.export_path, current.export_path);
        assert_eq!(preview.provider_name, "Sample integration");
    }

    #[test]
    fn integration_dialog_preview_does_not_detect_selected_folder_before_assignment() {
        let repo_root = std::env::temp_dir().join(format!(
            "netstitch-ui-integration-preview-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&repo_root);
        fs::create_dir_all(repo_root.join("config")).expect("config dir");
        fs::create_dir_all(repo_root.join("data")).expect("data dir");
        fs::write(repo_root.join("config").join("profile.conf"), b"profile\n")
            .expect("profile fixture");
        fs::write(
            repo_root.join("data").join("reference-data.txt"),
            b"1.1.1.1\n",
        )
        .expect("reference data fixture");

        let current = IntegrationIntegrationDto {
            repo_path: "Not configured".to_string(),
            export_path: String::new(),
            reference_data_path: String::new(),
            profile_paths: Vec::new(),
            ready: false,
            status_text: String::new(),
            last_export_text: String::new(),
            provider_id: None,
            provider_name: None,
            repository_url: None,
        };

        let preview = integration_dialog_preview(
            &repo_root.join("bin").display().to_string(),
            &current,
            "Module configured",
            "Module not specified",
            "Module not found",
            "Path entered",
            "Not specified",
        );

        assert_eq!(preview.status_text, "Module not found");
        assert!(preview.status_class.contains("--warning"));
        assert_eq!(
            preview.repo_path,
            repo_root.join("bin").display().to_string()
        );
        assert!(preview.export_path.is_empty());
        assert_eq!(preview.provider_name, "Not specified");

        let _ = fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn integration_progress_stage_lines_are_footer_events() {
        let labels = IntegrationProgressLabels {
            kb: "KB".to_string(),
            entries: "entries".to_string(),
            preparing: "Preparing".to_string(),
            downloading: "Downloading".to_string(),
            extracting: "Extracting".to_string(),
            detecting: "Detecting".to_string(),
            installing: "Installing".to_string(),
            complete: "Complete".to_string(),
            failed: "Failed".to_string(),
            idle: "Idle".to_string(),
        };
        let state = IntegrationDownloadUiState {
            active: true,
            visible: true,
            cancelled: false,
            generation: 1,
            stage: "extracting".to_string(),
            percent: Some(42),
            downloaded_bytes: Some(2048),
            total_bytes: Some(4096),
            extracted_entries: Some(8),
            total_entries: Some(20),
            message: None,
            repo_root: None,
        };

        assert_eq!(
            integration_progress_footer_line("Module download", &state, &labels),
            "Module download: Extracting"
        );
        assert_eq!(state.progress_meta(&labels), "42% | 8/20 entries");
    }

    #[test]
    fn connector_diagnostic_lines_are_current_run_events() {
        assert_eq!(
            connector_loaded_status_line(
                3,
                "Application connectors loaded:",
                "Application connectors loaded: no detected apps.",
                "detected apps."
            ),
            "Application connectors loaded: 3 detected apps"
        );
        assert_eq!(
            connector_loaded_status_line(
                0,
                "Application connectors loaded:",
                "Application connectors loaded: no detected apps.",
                "detected apps."
            ),
            "Application connectors loaded: no detected apps"
        );
        assert_eq!(
            tracked_app_availability_line(
                "Tracked app unavailable:",
                "Demo Game",
                r"D:\Games\DemoGame.exe"
            ),
            r"Tracked app unavailable: Demo Game - D:\Games\DemoGame.exe"
        );
    }

    #[test]
    fn missing_bitmap_icon_source_falls_back_to_file_url() {
        let src = compute_icon_image_src(r"C:\NetStitch Icons\missing.png");

        assert_eq!(src, "file:///C:/NetStitch%20Icons/missing.png");
    }

    #[test]
    fn duplicate_path_check_normalizes_slashes_and_case() {
        assert!(paths_match_for_duplicate_check(
            r"C:\Games\DemoGame.exe",
            r"c:/games/demogame.exe"
        ));
        assert!(!paths_match_for_duplicate_check(
            r"C:\Games\DemoGame.exe",
            r"C:\Games\Other.exe"
        ));
    }

    #[test]
    fn default_ignored_address_rules_filter_loopback_observations() {
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: vec![
                observation_with_ip(1, "127.0.0.1"),
                observation_with_ip(2, "::1"),
                observation_with_ip(3, "1.1.1.1"),
            ],
            ignored_addresses: vec![
                IgnoredAddressDto {
                    id: 1,
                    address_pattern: "127.0.0.0/8".to_string(),
                    created_at: String::new(),
                    enrichment: None,
                },
                IgnoredAddressDto {
                    id: 2,
                    address_pattern: "::1/128".to_string(),
                    created_at: String::new(),
                    enrichment: None,
                },
            ],
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
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
        };

        let rows = filter_observations(&snapshot);
        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn user_ignored_address_rules_filter_exact_and_cidr_matches() {
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: vec![
                observation_with_ip(1, "10.10.10.2"),
                observation_with_ip(2, "203.0.113.5"),
                observation_with_ip(3, "1.1.1.1"),
            ],
            ignored_addresses: vec![
                IgnoredAddressDto {
                    id: 1,
                    address_pattern: "10.10.10.0/24".to_string(),
                    created_at: String::new(),
                    enrichment: None,
                },
                IgnoredAddressDto {
                    id: 2,
                    address_pattern: "203.0.113.5".to_string(),
                    created_at: String::new(),
                    enrichment: None,
                },
            ],
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
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
        };

        let rows = filter_observations(&snapshot);
        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn observation_filter_searches_only_ip_column_substrings() {
        let mut observation = observation_with_ip(1, "8.8.8.45");
        observation.enrichment = Some(crate::watcher_api::IpEnrichmentDto {
            domain_name: Some("launcher.example.test".to_string()),
            domain_source: Some("tls_sni".to_string()),
            owner_name: Some("Example Game Network".to_string()),
            owner_range: Some("8.8.8.0/24".to_string()),
            registry: Some("test".to_string()),
            country: Some("ZZ".to_string()),
            source: Some("test".to_string()),
        });
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: vec![observation],
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: "8.8.8".to_string(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: true,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        let rows = filter_observations(&snapshot);

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![1]);
    }

    #[test]
    fn observation_sort_cycles_ascending_descending_then_default() {
        let asc = ObservationSortState::default().toggled(ObservationSortColumn::Hits);
        assert_eq!(asc.column, Some(ObservationSortColumn::Hits));
        assert!(!asc.descending);

        let desc = asc.toggled(ObservationSortColumn::Hits);
        assert_eq!(desc.column, Some(ObservationSortColumn::Hits));
        assert!(desc.descending);

        let reset = desc.toggled(ObservationSortColumn::Hits);
        assert_eq!(reset, ObservationSortState::default());
    }

    #[test]
    fn observation_default_sort_uses_latest_last_seen_first() {
        let mut older = observation_with_ip(1, "203.0.113.1");
        older.first_seen = "2026-04-20 12:00:00".to_string();
        older.last_seen = "2026-04-20 12:05:00".to_string();
        let mut newer = observation_with_ip(2, "203.0.113.2");
        newer.first_seen = "2026-04-20 11:00:00".to_string();
        newer.last_seen = "2026-04-20 12:10:00".to_string();
        let snapshot = snapshot_with_observations(vec![older.clone(), newer.clone()]);

        let sorted = sort_observations(
            &snapshot,
            vec![older, newer],
            ObservationSortState::default(),
        );

        assert_eq!(
            sorted.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![2, 1]
        );
        assert_eq!(
            ObservationSortState::default().indicator_state(ObservationSortColumn::LastSeen),
            "desc"
        );
    }

    #[test]
    fn observation_rows_are_keyed_by_stable_id_for_enrichment_updates() {
        let source = include_str!("app.rs").replace('\r', "");
        assert!(
            source.contains("ObservationRowView {\n                                                key: \"{observation.id}\","),
            "observation rows should stay keyed by stable id so enrichment fields do not hop between IP rows"
        );
    }

    #[test]
    fn csv_export_uses_selected_monitoring_table_columns() {
        let mut selected = observation_with_ip(1, "203.0.113.10");
        selected.is_confirmed = true;
        selected.hits = 3;
        selected.successful_hits = 2;
        selected.failed_hits = 1;
        selected.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("launcher.example.test".to_string()),
            domain_source: Some(netstitch_shared::DOMAIN_SOURCE_TLS_SNI.to_string()),
            ..IpEnrichmentDto::default()
        });
        let mut duplicate = observation_with_ip(2, "203.0.113.10");
        duplicate.is_confirmed = true;
        duplicate.enrichment = Some(IpEnrichmentDto {
            domain_name: Some("unproven.example.test".to_string()),
            domain_source: None,
            ..IpEnrichmentDto::default()
        });
        let mut exported = observation_with_ip(3, "198.51.100.7");
        exported.is_confirmed = true;
        exported.is_exported = true;

        let rows = selected_csv_export_rows(&snapshot_with_observations(vec![
            selected,
            duplicate,
            exported,
            observation_with_ip(4, "192.0.2.9"),
        ]));

        assert_eq!(
            rows,
            vec![
                CsvExportRow {
                    application: "demo".to_string(),
                    app_connector_id: String::new(),
                    cloud_app_id: String::new(),
                    app_signature_key: String::new(),
                    app_signature_subject: String::new(),
                    app_signature_issuer: String::new(),
                    ip: "203.0.113.10".to_string(),
                    domain: "launcher.example.test".to_string(),
                    port: "443".to_string(),
                    protocol: "TCP".to_string(),
                    connection: "Established (2/1/3)".to_string(),
                    requests: "3".to_string(),
                    first_seen: "2026-04-20 00:00:00".to_string(),
                    last_seen: "2026-04-20 00:00:00".to_string(),
                },
                CsvExportRow {
                    application: "demo".to_string(),
                    app_connector_id: String::new(),
                    cloud_app_id: String::new(),
                    app_signature_key: String::new(),
                    app_signature_subject: String::new(),
                    app_signature_issuer: String::new(),
                    ip: "203.0.113.10".to_string(),
                    domain: String::new(),
                    port: "443".to_string(),
                    protocol: "TCP".to_string(),
                    connection: "Established (1/0/1)".to_string(),
                    requests: "1".to_string(),
                    first_seen: "2026-04-20 00:00:00".to_string(),
                    last_seen: "2026-04-20 00:00:00".to_string(),
                },
            ]
        );
        assert_eq!(
            render_csv_export_rows(&rows),
            "application,app_connector_id,cloud_app_id,app_signature_key,app_signature_subject,app_signature_issuer,ip,domain,port,protocol,connection,requests,first_seen,last_seen\r\ndemo,,,,,,203.0.113.10,launcher.example.test,443,TCP,Established (2/1/3),3,2026-04-20 00:00:00,2026-04-20 00:00:00\r\ndemo,,,,,,203.0.113.10,,443,TCP,Established (1/0/1),1,2026-04-20 00:00:00,2026-04-20 00:00:00\r\n"
        );
        assert_eq!(csv_escape("a,b\"c"), "\"a,b\"\"c\"");
    }

    #[test]
    fn domain_filter_supports_contains_and_wildcards() {
        assert!(domain_filter_matches("api.example.com", "example"));
        assert!(domain_filter_matches("api.example.com", "*.example.com"));
        assert!(domain_filter_matches("assets.example.com", "assets*.com"));
        assert!(domain_filter_matches(
            "assets.cdn.example.com",
            "assets*.com"
        ));
        assert!(!domain_filter_matches("assets.example.net", "assets*.com"));
        assert!(!domain_filter_matches("badexample.com", "*.example.com"));
    }

    #[test]
    fn csv_import_parses_monitoring_table_columns() {
        let request = parse_csv_import_request(
            "application,ip,domain,port,protocol,connection,requests,first_seen,last_seen\r\n\
             Code,203.0.113.10,launcher.example.test,443,TCP,Established (2/1/3),3,2026-04-20 00:00:00,2026-04-20 00:00:01\r\n",
        )
        .expect("CSV should parse");

        assert_eq!(request.rows.len(), 1);
        let row = &request.rows[0];
        assert_eq!(row.application, "Code");
        assert_eq!(row.app_connector_id, None);
        assert_eq!(row.cloud_app_id, None);
        assert_eq!(row.app_signature_key, None);
        assert_eq!(row.remote_ip.to_string(), "203.0.113.10");
        assert_eq!(row.domain.as_deref(), Some("launcher.example.test"));
        assert_eq!(row.remote_port, 443);
        assert_eq!(row.protocol, SharedProtocol::Tcp);
        assert_eq!(row.connection_state, SharedConnectionState::Established);
        assert_eq!(row.successful_hits, 2);
        assert_eq!(row.failed_hits, 1);
        assert_eq!(row.hits, 3);
    }

    #[test]
    fn csv_import_parses_application_identifiers() {
        let request = parse_csv_import_request(
            "application,app_connector_id,cloud_app_id,app_signature_key,app_signature_subject,app_signature_issuer,ip,domain,port,protocol,connection,requests,first_seen,last_seen\r\n\
             Chrome,chrome,netstitch.app.chrome,spki-key,subject,issuer,203.0.113.10,launcher.example.test,443,TCP,Established (2/1/3),3,2026-04-20 00:00:00,2026-04-20 00:00:01\r\n",
        )
        .expect("CSV should parse");

        let row = &request.rows[0];
        assert_eq!(row.app_connector_id.as_deref(), Some("chrome"));
        assert_eq!(row.cloud_app_id.as_deref(), Some("netstitch.app.chrome"));
        assert_eq!(row.app_signature_key.as_deref(), Some("spki-key"));
        assert_eq!(row.app_signature_subject.as_deref(), Some("subject"));
        assert_eq!(row.app_signature_issuer.as_deref(), Some("issuer"));
    }

    #[test]
    fn background_snapshot_refresh_bumps_ui_nonce_for_monitoring_state_sync() {
        let source = include_str!("app.rs").replace('\r', "");
        assert!(
            source.contains("let before_snapshot = watcher.read().snapshot();")
                && source
                    .contains("let changed = watcher.read().apply_snapshot_refresh_result(result);")
                && source.contains("let after_snapshot = watcher.read().snapshot();")
                && source.contains(
                    "if changed\n                                && snapshot_ui_render_relevant_changed"
                ),
            "desktop background snapshot refresh should bump a UI nonce only when a render-relevant snapshot change was applied"
        );
    }

    #[test]
    fn background_snapshot_refresh_ignores_volatile_runtime_counters_for_table_smoothness() {
        let before = snapshot_with_observations(vec![observation_with_ip(1, "203.0.113.10")]);
        let mut after = before.clone();

        after.runtime_status.flow_capture.observations_emitted += 1;
        after.runtime_status.domain_capture.packets_seen += 1;
        assert!(
            !snapshot_ui_render_relevant_changed(&before, &after),
            "volatile runtime counters must not force a full desktop table rerender"
        );

        after.observations[0].hits += 1;
        assert!(
            snapshot_ui_render_relevant_changed(&before, &after),
            "actual monitoring row changes must still refresh the visible table"
        );
    }

    #[test]
    fn shell_controls_stay_enabled_after_snapshot_even_when_watcher_and_tool_are_unavailable() {
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: Vec::new(),
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: false,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
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
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        assert!(
            !shell_controls_disabled(&snapshot),
            "watcher/tool availability should not lock the UI once a snapshot is present"
        );
    }

    #[test]
    fn shell_controls_are_disabled_only_before_first_snapshot_load() {
        let snapshot = SnapshotResponse {
            tracked_apps: Vec::new(),
            observations: Vec::new(),
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
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
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        assert!(shell_controls_disabled(&snapshot));
    }

    #[test]
    fn shell_controls_stay_enabled_when_fallback_data_is_visible_even_before_snapshot_flag() {
        let snapshot = SnapshotResponse {
            tracked_apps: vec![crate::watcher_api::TrackedAppDto {
                id: 1,
                connector_id: Some("discord".to_string()),
                cloud_app_id: Some("netstitch.app.discord".to_string()),
                display_name: "Discord".to_string(),
                icon_key: "discord".to_string(),
                icon_path: None,
                exe_path: r"C:\Apps\Discord.exe".to_string(),
                enabled: true,
                created_at: String::new(),
            }],
            observations: Vec::new(),
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
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
                status_text: "Connecting to local watcher".to_string(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        assert!(
            !shell_controls_disabled(&snapshot),
            "visible fallback rows should stay clickable even if snapshot_loaded is still false"
        );
    }

    #[test]
    fn tracked_path_field_size_tracks_content_but_stays_clamped() {
        assert_eq!(tracked_path_field_size("C:\\Net"), 8);
        assert_eq!(
            tracked_path_field_size(r"C:\Users\Demo\AppData\Local\Discord\app-1.0.0\Discord.exe"),
            57
        );
        assert_eq!(tracked_path_field_size(&"x".repeat(400)), 120);
    }

    #[test]
    fn enable_all_overlay_marks_rows_enabled_without_losing_underlying_state() {
        let app = crate::watcher_api::TrackedAppDto {
            id: 1,
            connector_id: Some("discord".to_string()),
            cloud_app_id: Some("netstitch.app.discord".to_string()),
            display_name: "Discord".to_string(),
            icon_key: "discord".to_string(),
            icon_path: None,
            exe_path: r"C:\Apps\Discord.exe".to_string(),
            enabled: false,
            created_at: String::new(),
        };

        assert!(!effective_tracked_app_enabled(&app, false));
        assert!(effective_tracked_app_enabled(&app, true));
        assert!(
            !app.enabled,
            "overlay rendering must not mutate the stored per-app enabled flag"
        );
    }

    #[test]
    fn enable_all_overlay_makes_enabled_count_match_visible_state() {
        let snapshot = SnapshotResponse {
            tracked_apps: vec![
                crate::watcher_api::TrackedAppDto {
                    id: 1,
                    connector_id: Some("discord".to_string()),
                    cloud_app_id: Some("netstitch.app.discord".to_string()),
                    display_name: "Discord".to_string(),
                    icon_key: "discord".to_string(),
                    icon_path: None,
                    exe_path: r"C:\Apps\Discord.exe".to_string(),
                    enabled: false,
                    created_at: String::new(),
                },
                crate::watcher_api::TrackedAppDto {
                    id: 2,
                    connector_id: Some("telegram".to_string()),
                    cloud_app_id: Some("netstitch.app.telegram".to_string()),
                    display_name: "Telegram".to_string(),
                    icon_key: "telegram".to_string(),
                    icon_path: None,
                    exe_path: r"C:\Apps\Telegram.exe".to_string(),
                    enabled: true,
                    created_at: String::new(),
                },
            ],
            observations: Vec::new(),
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: true,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        };

        assert_eq!(effective_enabled_tracked_apps_count(&snapshot), 2);
        assert!(
            !snapshot.tracked_apps[0].enabled,
            "overlay should not overwrite the stored disabled flag for later restore"
        );
    }

    fn observation(
        id: u64,
        connection_state: ConnectionStateDto,
        failed_hits: u32,
        successful_hits: u32,
    ) -> ObservationDto {
        ObservationDto {
            id,
            tracked_app_id: 1,
            cloud_app_id: None,
            app_signature_key: None,
            app_signature_subject: None,
            app_signature_issuer: None,
            app_signature_source: None,
            process_name: "demo.exe".to_string(),
            remote_ip: "1.1.1.1".to_string(),
            remote_port: 443,
            protocol: ProtocolDto::Tcp,
            first_seen_ms: 1_776_624_000_000,
            first_seen: "2026-04-20 00:00:00".to_string(),
            last_seen_ms: 1_776_624_000_000,
            last_seen: "2026-04-20 00:00:00".to_string(),
            hits: failed_hits.saturating_add(successful_hits),
            connection_state,
            failed_hits,
            successful_hits,
            is_confirmed: false,
            is_exported: false,
            enrichment: None,
        }
    }

    fn observation_with_ip(id: u64, remote_ip: &str) -> ObservationDto {
        ObservationDto {
            remote_ip: remote_ip.to_string(),
            ..observation(id, ConnectionStateDto::Established, 0, 1)
        }
    }

    fn cloud_downloaded_observation(row_id: &str) -> CloudDownloadedObservation {
        CloudDownloadedObservation {
            row_id: row_id.to_string(),
            app_display_name: "Demo".to_string(),
            privacy_label: String::new(),
            source_label: "Author".to_string(),
            protocol_label: "TCP".to_string(),
            connection_label: "Established".to_string(),
            requests_label: "1".to_string(),
            first_seen_label: "2026-04-20 00:00:00".to_string(),
            last_seen_label: "2026-04-20 00:00:00".to_string(),
            domain_label: String::new(),
            row: CloudObservationRow {
                app_id: "netstitch.app.demo".to_string(),
                author_signature: Some("Author".to_string()),
                remote_ip: "203.0.113.10".parse().expect("valid test IP"),
                remote_port: 443,
                protocol: SharedProtocol::Tcp,
                connection_state: SharedConnectionState::Established,
                requests: 1,
                first_seen_ms: 1_776_624_000_000,
                last_seen_ms: 1_776_624_000_000,
                failed_hits: 0,
                successful_hits: 1,
                domain_raw: None,
                domain_verified: None,
                domain_status: CloudDomainStatus::None,
                trust_level: CloudTrustLevel::CommunityVerified,
                source_kind: CloudSourceKind::VerifiedUpload,
                app_signature_key: None,
                app_signature_subject: None,
                app_signature_issuer: None,
                cloud_observation_id: None,
            },
        }
    }

    fn snapshot_with_observations(observations: Vec<ObservationDto>) -> SnapshotResponse {
        SnapshotResponse {
            tracked_apps: Vec::new(),
            observations,
            ignored_addresses: Vec::new(),
            integration: IntegrationIntegrationDto {
                repo_path: String::new(),
                export_path: String::new(),
                reference_data_path: String::new(),
                profile_paths: Vec::new(),
                ready: false,
                status_text: String::new(),
                last_export_text: String::new(),
                provider_id: None,
                provider_name: None,
                repository_url: None,
            },
            integration_modules: Vec::new(),
            integration_providers: Vec::new(),
            ui: UiStatusDto {
                monitoring: false,
                watcher_connected: true,
                snapshot_loaded: true,
                status_text: String::new(),
                error_text: None,
            },
            app_settings: AppSettingsDto {
                language_code: None,
                enable_all_overlay: false,
                remember_window_placement: false,
                hide_when_minimized: true,
                module_order: Vec::new(),
                web_access_localhost: false,
                domain_capture_enabled: false,
                update_check_interval_minutes: 10,
                profile_export_ui_state: None,
            },
            runtime_status: RuntimeStatusDto::default(),
            filters: FiltersDto {
                app_search: String::new(),
                search_text: String::new(),
                domain_search: String::new(),
                port_search: String::new(),
                protocol: "All".to_string(),
                public_ip: false,
                observation_filter: ObservationFilterDto::All,
            },
            pending_exe_path: String::new(),
        }
    }
}
