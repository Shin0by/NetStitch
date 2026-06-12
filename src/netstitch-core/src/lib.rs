use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use netstitch_connectors::DetectedApp;
use netstitch_integrations::IntegrationService;
use netstitch_shared::ipc::{
    AddIgnoredAddressRequest, AddTrackedAppRequest, ConfirmEndpointsRequest,
    DeleteIgnoredAddressRequest, DeleteObservationRequest, DeleteObservationsRequest,
    ExportConfirmedRequest, SetTrackedAppEnabledRequest,
};
use netstitch_shared::models::{
    AggregatedIpDto, AppSettingsDto, CapabilityReport, ConnectionState,
    ExportProfileAdvancedSettingsRequestDto, ExportProfilePlanDto, ExportProfileRequestDto,
    ExportResultDto, ExportRun, IgnoredAddressRule, IntegrationModuleBackgroundEventDto,
    IntegrationModuleDto, IntegrationModuleHostContextDto, IntegrationModuleRuntimeStatusDto,
    IntegrationModuleTableContextDto, IntegrationModuleUiActionClientRequestDto,
    IntegrationModuleUiActionRequestDto, IntegrationModuleUiActionResponseDto,
    IntegrationStatusDto, IpEnrichmentDto, MonitorStatus, MonitoringCsvImportRequestDto,
    MonitoringCsvImportResultDto, MonitoringCsvImportRowDto, ObservedEndpoint,
    ProfileExportUiStateDto, Protocol, RuntimeStatusDto, SETTING_DOMAIN_CAPTURE_ENABLED,
    SETTING_IGNORE_DEFAULTS_SEEDED, SETTING_IP_ENRICHMENT_ENABLED, SETTING_UI_ENABLE_ALL_OVERLAY,
    SETTING_UI_HIDE_WHEN_MINIMIZED, SETTING_UI_LANGUAGE, SETTING_UI_MODULE_ORDER,
    SETTING_UI_MONITORING_PUBLIC_IP, SETTING_UI_REMEMBER_WINDOW_PLACEMENT,
    SETTING_UPDATE_CHECK_INTERVAL_MINUTES, SETTING_WEB_ACCESS_LOCALHOST, SnapshotResponse,
    SystemEventDto, SystemEventRequestDto, TrackedApp, TrackedAppAvailabilityDto, TrackedAppId,
    UiFiltersDto,
};
use netstitch_shared::runtime_build_version;
use rusqlite::{Connection, OptionalExtension, params};

const APP_ENV_DATA_DIR: &str = "NETSTITCH__DATA_DIR";
const APP_ENV_WEB_UI: &str = "NETSTITCH__WEB_UI";
const CLEAR_SYSTEM_EVENTS_MARKER: &str = "clear-system-events-on-next-start";
const MANUAL_ICON_KEY: &str = "manual";
const DEFAULT_IGNORED_ADDRESS_RULES: &[&str] = &["127.0.0.0/8", "::1/128"];
const MAX_WHOIS_RANGE_CACHE_ROWS: usize = 1000;
const MAX_SYSTEM_EVENT_ROWS: i64 = 9999;
const LOCAL_IPV4_ROUTE_PROBE_TARGETS: &[&str] = &[
    "8.8.8.8:80",
    "8.8.4.4:80",
    "1.1.1.1:80",
    "1.0.0.1:80",
    "9.9.9.9:80",
    "208.67.222.222:80",
    "94.140.14.14:80",
    "4.2.2.1:80",
];
const LOCAL_IPV6_ROUTE_PROBE_TARGETS: &[&str] = &[
    "[2001:4860:4860::8888]:80",
    "[2001:4860:4860::8844]:80",
    "[2606:4700:4700::1111]:80",
    "[2606:4700:4700::1001]:80",
    "[2620:fe::fe]:80",
    "[2620:fe::9]:80",
    "[2620:119:35::35]:80",
    "[2a10:50c0::ad1:ff]:80",
];
static PATH_IS_FILE_CACHE: LazyLock<Mutex<BTreeMap<PathBuf, bool>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
const TRACKED_APPS_LIST_QUERY: &str = r#"
    SELECT id, exe_path, connector_id, cloud_app_id, process_name, display_name, icon_key, icon_path, enabled, created_at_ms
    FROM tracked_apps
    WHERE COALESCE(archived, 0) = 0
    ORDER BY
        CASE WHEN COALESCE(icon_key, 'manual') = 'manual' THEN 0 ELSE 1 END ASC,
        CASE WHEN COALESCE(icon_key, 'manual') = 'manual' THEN created_at_ms ELSE NULL END DESC,
        CASE WHEN COALESCE(icon_key, 'manual') = 'manual' THEN id ELSE NULL END DESC,
        CASE WHEN COALESCE(icon_key, 'manual') <> 'manual' THEN LOWER(COALESCE(display_name, process_name, exe_path)) ELSE NULL END ASC,
        id ASC
"#;

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub export_dir: PathBuf,
    pub icon_cache_dir: PathBuf,
    pub connector_icon_dir: PathBuf,
    pub external_apps_dir: PathBuf,
    pub integrations_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ObservationRecord {
    pub tracked_app_id: u64,
    pub process_id: Option<u32>,
    pub process_name: String,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub connection_state: ConnectionState,
    pub observed_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct IpDomainCacheRecord {
    pub remote_ip: IpAddr,
    pub lookup_status: String,
    pub error_text: Option<String>,
    pub last_attempt_ms: u64,
    pub updated_at_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct EndpointDomainCacheRecord {
    pub tracked_app_id: u64,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub domain_name: String,
    pub domain_source: String,
    pub observed_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct WhoisRangeCacheRecord {
    pub cidr: String,
    pub owner_name: Option<String>,
    pub registry: Option<String>,
    pub country: Option<String>,
    pub source: Option<String>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug)]
struct WhoisRangeRow {
    cidr: String,
    owner_name: Option<String>,
    registry: Option<String>,
    country: Option<String>,
    source: Option<String>,
    updated_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct NetstitchCore {
    paths: AppPaths,
    integration_service: IntegrationService,
    integration_module_catalog: Arc<Mutex<Option<IntegrationModuleCatalog>>>,
    integration_background_tasks: Arc<Mutex<BTreeMap<String, IntegrationBackgroundTask>>>,
}

#[derive(Clone, Debug)]
struct IntegrationModuleCatalog {
    language_code: Option<String>,
    modules: Vec<IntegrationModuleDto>,
    statuses: Vec<IntegrationModuleRuntimeStatusDto>,
}

#[derive(Clone, Debug)]
struct IntegrationBackgroundTask {
    subscriptions: BTreeSet<String>,
    started_at_ms: u64,
    last_event_at_ms: Option<u64>,
    last_status: Option<String>,
    last_error: Option<String>,
    in_flight: bool,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let data_dir = if let Ok(explicit) = env::var(APP_ENV_DATA_DIR) {
            PathBuf::from(explicit)
        } else {
            let cwd = env::current_dir().context("failed to resolve current directory")?;
            if cwd.join("storage").exists() {
                cwd.join("storage")
            } else if let Some(project_dirs) =
                ProjectDirs::from("netstitch", "NetStitch", "NetStitch")
            {
                project_dirs.data_local_dir().to_path_buf()
            } else {
                cwd.join("storage")
            }
        };

        let app_icon_dir = runtime_connector_dir()
            .map(|dir| dir.join("icons"))
            .unwrap_or_else(|| data_dir.join("icons"));

        Ok(Self {
            database_path: data_dir.join("netstitch.sqlite3"),
            export_dir: data_dir.join("exports"),
            icon_cache_dir: app_icon_dir.clone(),
            connector_icon_dir: app_icon_dir,
            external_apps_dir: data_dir.join("external-apps"),
            integrations_dir: data_dir.join("integrations"),
            data_dir,
        })
    }

    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("failed to create {}", self.data_dir.display()))?;
        fs::create_dir_all(&self.export_dir)
            .with_context(|| format!("failed to create {}", self.export_dir.display()))?;
        fs::create_dir_all(&self.icon_cache_dir)
            .with_context(|| format!("failed to create {}", self.icon_cache_dir.display()))?;
        fs::create_dir_all(&self.connector_icon_dir)
            .with_context(|| format!("failed to create {}", self.connector_icon_dir.display()))?;
        Ok(())
    }
}

fn runtime_connector_dir() -> Option<PathBuf> {
    if let Some(explicit) = env::var_os("NETSTITCH__CONNECTORS_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return explicit.is_dir().then_some(explicit);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join("apps");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    env::current_dir()
        .ok()
        .map(|cwd| cwd.join("apps"))
        .filter(|candidate| candidate.is_dir())
}

impl NetstitchCore {
    pub fn bootstrap() -> Result<Self> {
        let paths = AppPaths::discover()?;
        paths.ensure_directories()?;
        let core = Self {
            integration_service: IntegrationService::new(paths.integrations_dir.clone()),
            integration_module_catalog: Arc::new(Mutex::new(None)),
            integration_background_tasks: Arc::new(Mutex::new(BTreeMap::new())),
            paths,
        };
        core.initialize_schema()?;
        core.apply_storage_maintenance_markers()?;
        core.ensure_connector_tracked_apps()?;
        core.refresh_manual_tracked_app_icons()?;
        Ok(core)
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn add_tracked_app(&self, request: AddTrackedAppRequest) -> Result<TrackedApp> {
        let exe_path = resolve_executable_entry_path(&request.exe_path)?;
        if exe_path.as_os_str().is_empty() {
            return Err(anyhow!("exe_path must not be empty"));
        }

        let conn = self.open_connection()?;
        let created_at_ms = current_timestamp_ms();
        let process_name = derive_process_name(&exe_path);
        let display_name = display_name_for_manual_app(&exe_path);
        let icon_path = cache_manual_icon(&exe_path, &self.paths.icon_cache_dir)
            .map(|path| path.to_string_lossy().to_string());
        let exe_path_text = exe_path.to_string_lossy().to_string();
        if let Some(existing_manual_id) = conn
            .query_row(
                r#"
                SELECT id FROM tracked_apps
                WHERE exe_path = ?1
                  AND connector_id IS NULL
                ORDER BY id ASC
                LIMIT 1
                "#,
                params![&exe_path_text],
                |row| row.get::<_, i64>(0).map(|id| id as TrackedAppId),
            )
            .optional()
            .context("failed to look up manual tracked app by path")?
        {
            conn.execute(
                r#"
                UPDATE tracked_apps
                SET enabled = ?2,
                    archived = 0,
                    process_name = COALESCE(?3, process_name),
                    display_name = COALESCE(?4, display_name),
                    icon_key = ?5,
                    icon_path = COALESCE(?6, icon_path)
                WHERE id = ?1
                "#,
                params![
                    existing_manual_id as i64,
                    request.enabled,
                    process_name,
                    display_name,
                    MANUAL_ICON_KEY,
                    icon_path,
                ],
            )?;
            return self
                .find_tracked_app_by_id(existing_manual_id)?
                .ok_or_else(|| anyhow!("tracked app was not persisted"));
        }

        if let Some(existing_id) = conn
            .query_row(
                r#"
                SELECT id FROM tracked_apps
                WHERE exe_path = ?1
                  AND COALESCE(archived, 0) = 0
                ORDER BY id ASC
                LIMIT 1
                "#,
                params![&exe_path_text],
                |row| row.get::<_, i64>(0).map(|id| id as TrackedAppId),
            )
            .optional()
            .context("failed to look up tracked app by path")?
        {
            return self
                .find_tracked_app_by_id(existing_id)?
                .ok_or_else(|| anyhow!("tracked app was not persisted"));
        }

        conn.execute(
            r#"
            INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                exe_path_text,
                request.enabled,
                process_name,
                display_name,
                MANUAL_ICON_KEY,
                icon_path,
                created_at_ms as i64
            ],
        )?;

        self.find_tracked_app_by_id(conn.last_insert_rowid() as TrackedAppId)?
            .ok_or_else(|| anyhow!("tracked app was not persisted"))
    }

    pub fn list_tracked_apps(&self) -> Result<Vec<TrackedApp>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(TRACKED_APPS_LIST_QUERY)?;
        let rows = stmt.query_map([], map_tracked_app)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn list_all_tracked_apps(&self) -> Result<Vec<TrackedApp>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, exe_path, connector_id, cloud_app_id, process_name, display_name, icon_key, icon_path, enabled, created_at_ms
            FROM tracked_apps
            ORDER BY id ASC
            "#,
        )?;
        let rows = stmt.query_map([], map_tracked_app)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn app_settings(&self) -> Result<AppSettingsDto> {
        Ok(AppSettingsDto {
            ui_language_code: self.get_app_setting(SETTING_UI_LANGUAGE)?,
            ui_enable_all_overlay: self
                .get_app_setting(SETTING_UI_ENABLE_ALL_OVERLAY)?
                .is_some_and(|value| setting_truthy(&value)),
            ui_remember_window_placement: self
                .get_app_setting(SETTING_UI_REMEMBER_WINDOW_PLACEMENT)?
                .is_some_and(|value| setting_truthy(&value)),
            ui_hide_when_minimized: self
                .get_app_setting(SETTING_UI_HIDE_WHEN_MINIMIZED)?
                .map(|value| setting_truthy(&value))
                .unwrap_or(true),
            ui_module_order: self
                .get_app_setting(SETTING_UI_MODULE_ORDER)?
                .map(|value| parse_string_list_setting(&value))
                .unwrap_or_default(),
            web_access_localhost: self.web_access_localhost_enabled()?,
            domain_capture_enabled: self.domain_capture_enabled()?,
            update_check_interval_minutes: self
                .get_app_setting(SETTING_UPDATE_CHECK_INTERVAL_MINUTES)?
                .as_deref()
                .map(parse_update_check_interval_minutes)
                .unwrap_or_else(netstitch_shared::models::default_update_check_interval_minutes),
        })
    }

    pub fn web_access_localhost_enabled(&self) -> Result<bool> {
        Ok(match self.get_app_setting(SETTING_WEB_ACCESS_LOCALHOST)? {
            Some(value) => setting_truthy(&value),
            None => env_truthy(APP_ENV_WEB_UI),
        })
    }

    pub fn domain_capture_enabled(&self) -> Result<bool> {
        Ok(self
            .get_app_setting(SETTING_DOMAIN_CAPTURE_ENABLED)?
            .is_some_and(|value| setting_truthy(&value)))
    }

    pub fn monitoring_public_ip_filter_enabled(&self) -> Result<bool> {
        Ok(
            match self.get_app_setting(SETTING_UI_MONITORING_PUBLIC_IP)? {
                Some(value) => setting_truthy(&value),
                None => true,
            },
        )
    }

    pub fn ip_enrichment_enabled(&self) -> Result<bool> {
        Ok(match self.get_app_setting(SETTING_IP_ENRICHMENT_ENABLED)? {
            Some(value) => setting_truthy(&value),
            None => true,
        })
    }

    pub fn configured_integration_root(&self) -> Result<Option<PathBuf>> {
        let Ok(module_id) = self.integration_service.primary_module_id() else {
            return Ok(None);
        };
        self.integration_service.configured_root(&module_id)
    }

    pub fn configure_integration_root(
        &self,
        module_id: Option<&str>,
        candidate: Option<&Path>,
    ) -> Result<Option<IntegrationStatusDto>> {
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .configure_root(&module_id, candidate)
    }

    pub fn set_profile_export_ui_state(&self, state: ProfileExportUiStateDto) -> Result<()> {
        let module_id = self.integration_service.primary_module_id()?;
        self.integration_service
            .set_profile_export_ui_state(&module_id, &state)
    }

    pub fn list_effective_tracked_apps(&self) -> Result<Vec<TrackedApp>> {
        let mut apps = self.list_tracked_apps()?;
        if self.app_settings()?.ui_enable_all_overlay {
            for app in &mut apps {
                app.enabled = true;
            }
        }
        Ok(apps)
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> Result<()> {
        let key = key.trim();
        if key.is_empty() {
            return Err(anyhow!("setting key must not be empty"));
        }

        let conn = self.open_connection()?;
        conn.execute(
            r#"
            INSERT INTO app_settings (setting_key, setting_value, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(setting_key) DO UPDATE SET
                setting_value = excluded.setting_value,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![key, value.trim(), current_timestamp_ms() as i64],
        )
        .context("failed to persist app setting")?;
        Ok(())
    }

    pub fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.open_connection()?;
        conn.query_row(
            "SELECT setting_value FROM app_settings WHERE setting_key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .context("failed to read app setting")
    }

    pub fn append_system_event(&self, request: SystemEventRequestDto) -> Result<SystemEventDto> {
        let source = normalize_system_event_source(request.source.as_deref());
        let component = normalize_system_event_token(&request.component, "app");
        let action_type = normalize_system_event_token(&request.action_type, "event");
        let severity = normalize_system_event_severity(&request.severity);
        let entity_type = normalize_optional_system_event_text(request.entity_type);
        let entity_id = normalize_optional_system_event_text(request.entity_id);
        let payload_json =
            serde_json::to_string(&request.payload).context("failed to serialize event payload")?;
        let created_at_ms = current_timestamp_ms();
        let mut conn = self.open_connection()?;
        let tx = conn.transaction()?;
        let last_event = tx
            .query_row(
                r#"
                SELECT event_id, source, component, action_type, severity, entity_type, entity_id, payload_json
                FROM system_events
                ORDER BY event_id DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)? as u64,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()?;
        let mut repeated_event_id = None;
        if let Some((
            event_id,
            last_source,
            last_component,
            last_action_type,
            last_severity,
            last_entity_type,
            last_entity_id,
            last_payload_json,
        )) = last_event
        {
            if last_source == source
                && last_component == component
                && last_action_type == action_type
                && last_severity == severity
                && last_entity_type == entity_type
                && last_entity_id == entity_id
                && last_payload_json == payload_json
            {
                tx.execute(
                    r#"
                    UPDATE system_events
                    SET created_at_ms = ?1,
                        repeat_count = repeat_count + 1
                    WHERE event_id = ?2
                    "#,
                    params![created_at_ms as i64, event_id as i64],
                )?;
                repeated_event_id = Some(event_id);
            }
        }
        let event_id = if let Some(event_id) = repeated_event_id {
            event_id
        } else {
            tx.execute(
                r#"
                INSERT INTO system_events
                    (created_at_ms, source, component, action_type, severity, entity_type, entity_id, payload_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    created_at_ms as i64,
                    source,
                    component,
                    action_type,
                    severity,
                    entity_type,
                    entity_id,
                    payload_json
                ],
            )?;
            tx.last_insert_rowid() as u64
        };
        rotate_system_events(&tx)?;
        tx.commit()?;
        self.find_system_event(event_id)?
            .ok_or_else(|| anyhow!("system event was not persisted"))
    }

    pub fn list_system_events(&self, limit: usize) -> Result<Vec<SystemEventDto>> {
        let limit = limit.clamp(1, MAX_SYSTEM_EVENT_ROWS as usize);
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT event_id, created_at_ms, repeat_count, source, component, action_type, severity,
                   entity_type, entity_id, payload_json
            FROM system_events
            ORDER BY event_id DESC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], map_system_event)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    fn apply_storage_maintenance_markers(&self) -> Result<()> {
        let marker_path = self.paths.data_dir.join(CLEAR_SYSTEM_EVENTS_MARKER);
        if !marker_path.exists() {
            return Ok(());
        }

        let conn = self.open_connection()?;
        conn.execute("DELETE FROM system_events", [])
            .context("failed to clear system events after packaging marker")?;
        fs::remove_file(&marker_path).with_context(|| {
            format!(
                "failed to remove system event cleanup marker {}",
                marker_path.display()
            )
        })?;
        Ok(())
    }

    fn find_system_event(&self, event_id: u64) -> Result<Option<SystemEventDto>> {
        let conn = self.open_connection()?;
        conn.query_row(
            r#"
            SELECT event_id, created_at_ms, repeat_count, source, component, action_type, severity,
                   entity_type, entity_id, payload_json
            FROM system_events
            WHERE event_id = ?1
            "#,
            params![event_id as i64],
            map_system_event,
        )
        .optional()
        .context("failed to reload system event")
    }

    pub fn set_all_tracked_apps_enabled(&self, enabled: bool) -> Result<usize> {
        let conn = self.open_connection()?;
        conn.execute(
            "UPDATE tracked_apps SET enabled = ?1 WHERE enabled <> ?1",
            params![enabled],
        )
        .context("failed to update tracked app enabled state")
    }

    pub fn set_tracked_app_enabled(&self, request: SetTrackedAppEnabledRequest) -> Result<bool> {
        let conn = self.open_connection()?;
        let updated = conn
            .execute(
                "UPDATE tracked_apps SET enabled = ?1 WHERE id = ?2 AND enabled <> ?1",
                params![request.enabled, request.tracked_app_id as i64],
            )
            .context("failed to update tracked app enabled state")?;
        Ok(updated > 0)
    }

    pub fn list_ignored_addresses(&self) -> Result<Vec<IgnoredAddressRule>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, address_pattern, created_at_ms
            FROM ignored_addresses
            ORDER BY created_at_ms ASC, id ASC
            "#,
        )?;
        let rows = stmt.query_map([], map_ignored_address_rule)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn pending_ip_enrichment_targets(
        &self,
        limit: usize,
        retry_after_ms: u64,
    ) -> Result<Vec<IpAddr>> {
        let now = current_timestamp_ms();
        let mut candidates = BTreeMap::<IpAddr, ()>::new();
        for endpoint in self.list_endpoints()? {
            candidates.insert(endpoint.remote_ip, ());
        }
        for rule in self.list_ignored_addresses()? {
            if let Some(ip) = ignored_rule_base_ip(&rule.address_pattern) {
                candidates.insert(ip, ());
            }
        }

        let conn = self.open_connection()?;
        let mut targets = Vec::new();
        for ip in candidates.keys().copied() {
            let last_attempt = conn
                .query_row(
                    "SELECT last_attempt_ms FROM ip_domain_cache WHERE remote_ip = ?1",
                    params![ip.to_string()],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .map(|value| value.max(0) as u64);
            if last_attempt.is_some_and(|attempt| attempt.saturating_add(retry_after_ms) > now) {
                continue;
            }
            targets.push(ip);
            if targets.len() >= limit {
                break;
            }
        }
        Ok(targets)
    }

    pub fn upsert_ip_domain_cache(&self, record: IpDomainCacheRecord) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute(
            r#"
            INSERT INTO ip_domain_cache (
                remote_ip,
                lookup_status,
                error_text,
                last_attempt_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(remote_ip) DO UPDATE SET
                lookup_status = excluded.lookup_status,
                error_text = excluded.error_text,
                last_attempt_ms = excluded.last_attempt_ms,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![
                record.remote_ip.to_string(),
                record.lookup_status,
                record.error_text,
                record.last_attempt_ms as i64,
                record.updated_at_ms.map(|value| value as i64),
            ],
        )
        .context("failed to persist IP domain cache")?;
        Ok(())
    }

    pub fn upsert_endpoint_domain_cache(&self, record: EndpointDomainCacheRecord) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute(
            r#"
            INSERT INTO endpoint_domain_cache (
                tracked_app_id,
                remote_ip,
                remote_port,
                protocol,
                domain_name,
                domain_source,
                observed_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(tracked_app_id, remote_ip, remote_port, protocol) DO UPDATE SET
                domain_name = excluded.domain_name,
                domain_source = excluded.domain_source,
                observed_at_ms = excluded.observed_at_ms,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![
                record.tracked_app_id as i64,
                record.remote_ip.to_string(),
                i64::from(record.remote_port),
                record.protocol.as_str(),
                record.domain_name,
                record.domain_source,
                record.observed_at_ms as i64,
                record.updated_at_ms as i64,
            ],
        )
        .context("failed to persist verified endpoint domain cache")?;
        Ok(())
    }

    pub fn store_verified_domain(
        &self,
        tracked_app_id: u64,
        remote_ip: IpAddr,
        remote_port: u16,
        protocol: Protocol,
        domain_name: String,
        domain_source: String,
        observed_at_ms: u64,
    ) -> Result<()> {
        self.upsert_endpoint_domain_cache(EndpointDomainCacheRecord {
            tracked_app_id,
            remote_ip,
            remote_port,
            protocol,
            domain_name,
            domain_source,
            observed_at_ms,
            updated_at_ms: observed_at_ms,
        })
    }

    pub fn detect_and_store_verified_domain(
        &self,
        tracked_app_id: u64,
        remote_ip: IpAddr,
        remote_port: u16,
        protocol: Protocol,
        payload: &[u8],
        observed_at_ms: u64,
    ) -> Result<Option<String>> {
        let Some(detection) =
            netstitch_shared::detect_verified_domain(protocol, remote_port, payload)
        else {
            return Ok(None);
        };
        let domain = detection.domain;
        self.store_verified_domain(
            tracked_app_id,
            remote_ip,
            remote_port,
            protocol,
            domain.clone(),
            detection.source.to_string(),
            observed_at_ms,
        )?;
        Ok(Some(domain))
    }

    pub fn upsert_whois_range_cache(&self, record: WhoisRangeCacheRecord) -> Result<()> {
        let (range_start, range_end) = cidr_bounds(&record.cidr)
            .with_context(|| format!("invalid whois CIDR range: {}", record.cidr))?;
        let conn = self.open_connection()?;
        conn.execute(
            r#"
            INSERT INTO ip_whois_ranges (
                cidr,
                range_start,
                range_end,
                owner_name,
                registry,
                country,
                source,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(cidr) DO UPDATE SET
                range_start = excluded.range_start,
                range_end = excluded.range_end,
                owner_name = excluded.owner_name,
                registry = excluded.registry,
                country = excluded.country,
                source = excluded.source,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![
                record.cidr,
                range_start,
                range_end,
                record.owner_name,
                record.registry,
                record.country,
                record.source,
                record.updated_at_ms as i64,
            ],
        )
        .context("failed to persist whois range cache")?;
        prune_whois_range_cache(&conn, MAX_WHOIS_RANGE_CACHE_ROWS)?;
        Ok(())
    }

    pub fn add_ignored_address(
        &self,
        request: AddIgnoredAddressRequest,
    ) -> Result<IgnoredAddressRule> {
        let address_pattern = normalize_ignored_address_pattern(&request.address_pattern)?;
        let conn = self.open_connection()?;
        conn.execute(
            r#"
            INSERT INTO ignored_addresses (address_pattern, created_at_ms)
            VALUES (?1, ?2)
            ON CONFLICT(address_pattern) DO UPDATE SET
                address_pattern = excluded.address_pattern
            "#,
            params![address_pattern, current_timestamp_ms() as i64],
        )
        .context("failed to persist ignored address")?;

        self.find_ignored_address(&address_pattern)?
            .ok_or_else(|| anyhow!("ignored address was not persisted"))
    }

    pub fn delete_ignored_address(&self, request: DeleteIgnoredAddressRequest) -> Result<usize> {
        let conn = self.open_connection()?;
        conn.execute(
            "DELETE FROM ignored_addresses WHERE id = ?1",
            params![request.ignored_address_id as i64],
        )
        .context("failed to delete ignored address")
    }

    fn find_ignored_address(&self, address_pattern: &str) -> Result<Option<IgnoredAddressRule>> {
        let conn = self.open_connection()?;
        conn.query_row(
            "SELECT id, address_pattern, created_at_ms FROM ignored_addresses WHERE address_pattern = ?1",
            params![address_pattern],
            map_ignored_address_rule,
        )
        .optional()
        .context("failed to reload ignored address")
    }

    pub fn delete_tracked_app(&self, tracked_app_id: TrackedAppId) -> Result<usize> {
        let conn = self.open_connection()?;
        conn.execute(
            "UPDATE tracked_apps SET archived = 1, enabled = 0 WHERE id = ?1 AND COALESCE(archived, 0) = 0",
            params![tracked_app_id as i64],
        )
        .context("failed to archive tracked app")
    }

    pub fn find_tracked_app_by_path(&self, exe_path: &Path) -> Result<Option<TrackedApp>> {
        let conn = self.open_connection()?;
        conn.query_row(
            r#"
            SELECT id, exe_path, connector_id, cloud_app_id, process_name, display_name, icon_key, icon_path, enabled, created_at_ms
            FROM tracked_apps
            WHERE exe_path = ?1
            ORDER BY CASE WHEN connector_id IS NULL THEN 0 ELSE 1 END ASC, id ASC
            LIMIT 1
            "#,
            params![normalize_path_buf(exe_path).to_string_lossy().to_string()],
            map_tracked_app,
        )
        .optional()
        .context("failed to look up tracked app by path")
    }

    pub fn find_tracked_app_by_id(
        &self,
        tracked_app_id: TrackedAppId,
    ) -> Result<Option<TrackedApp>> {
        let conn = self.open_connection()?;
        conn.query_row(
            r#"
            SELECT id, exe_path, connector_id, cloud_app_id, process_name, display_name, icon_key, icon_path, enabled, created_at_ms
            FROM tracked_apps
            WHERE id = ?1
            "#,
            params![tracked_app_id as i64],
            map_tracked_app,
        )
        .optional()
        .context("failed to look up tracked app by id")
    }

    pub fn ingest_observation(&self, record: ObservationRecord) -> Result<ObservedEndpoint> {
        let conn = self.open_connection()?;
        let remote_ip = record.remote_ip.to_string();
        let protocol = record.protocol.as_str().to_string();
        let failed_increment = if record.connection_state.is_failure_like() {
            1_i64
        } else {
            0_i64
        };
        let success_increment = if record.connection_state.is_success_like() {
            1_i64
        } else {
            0_i64
        };
        let existing_id: Option<u64> = conn
            .query_row(
                r#"
                SELECT id FROM observed_endpoints
                WHERE tracked_app_id = ?1 AND remote_ip = ?2 AND remote_port = ?3 AND protocol = ?4
                "#,
                params![
                    record.tracked_app_id as i64,
                    remote_ip,
                    i64::from(record.remote_port),
                    protocol
                ],
                |row| row.get::<_, i64>(0).map(|id| id as u64),
            )
            .optional()?;

        match existing_id {
            Some(endpoint_id) => {
                conn.execute(
                    r#"
                    UPDATE observed_endpoints
                    SET process_id = ?2,
                        process_name = ?3,
                        last_seen_ms = ?4,
                        hits = hits + 1,
                        connection_state = ?5,
                        failed_hits = failed_hits + ?6,
                        successful_hits = successful_hits + ?7
                    WHERE id = ?1
                    "#,
                    params![
                        endpoint_id as i64,
                        record.process_id.map(i64::from),
                        record.process_name,
                        record.observed_at_ms as i64,
                        record.connection_state.as_str(),
                        failed_increment,
                        success_increment
                    ],
                )?;
            }
            None => {
                conn.execute(
                    r#"
                    INSERT INTO observed_endpoints (
                        tracked_app_id,
                        process_id,
                        process_name,
                        remote_ip,
                        remote_port,
                        protocol,
                        first_seen_ms,
                        last_seen_ms,
                        hits,
                        connection_state,
                        failed_hits,
                        successful_hits,
                        is_confirmed,
                        is_exported
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?11, 0, 0)
                    "#,
                    params![
                        record.tracked_app_id as i64,
                        record.process_id.map(i64::from),
                        record.process_name,
                        record.remote_ip.to_string(),
                        i64::from(record.remote_port),
                        record.protocol.as_str(),
                        record.observed_at_ms as i64,
                        record.observed_at_ms as i64,
                        record.connection_state.as_str(),
                        failed_increment,
                        success_increment
                    ],
                )?;
            }
        }

        self.find_endpoint(
            record.tracked_app_id,
            record.remote_ip,
            record.remote_port,
            record.protocol,
        )?
        .ok_or_else(|| anyhow!("failed to reload upserted endpoint"))
    }

    pub fn import_monitoring_csv(
        &self,
        request: MonitoringCsvImportRequestDto,
    ) -> Result<MonitoringCsvImportResultDto> {
        let requested_count = request.rows.len();
        let mut conn = self.open_connection()?;
        let tx = conn.transaction()?;
        let now = current_timestamp_ms();
        let mut imported_count = 0usize;
        let mut skipped_count = 0usize;

        for row in request.rows {
            if row.remote_ip.is_unspecified() || row.remote_port == 0 {
                skipped_count += 1;
                continue;
            }
            let inserted = import_monitoring_csv_row(&tx, row, now)
                .with_context(|| "failed to import monitoring CSV row")?;
            if inserted {
                imported_count += 1;
            } else {
                skipped_count += 1;
            }
        }

        tx.commit()
            .context("failed to commit monitoring CSV import")?;
        Ok(MonitoringCsvImportResultDto {
            requested_count,
            imported_count,
            skipped_count,
        })
    }

    pub fn list_endpoints(&self) -> Result<Vec<ObservedEndpoint>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, tracked_app_id, cloud_app_id, app_signature_key, app_signature_subject,
                   app_signature_issuer, app_signature_source, process_id, process_name, remote_ip, remote_port, protocol,
                   first_seen_ms, last_seen_ms, hits, connection_state, failed_hits, successful_hits,
                   is_confirmed, is_exported
            FROM observed_endpoints
            ORDER BY first_seen_ms DESC, id DESC
            "#,
        )?;
        let rows = stmt.query_map([], map_endpoint)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn backfill_observation_cloud_signature(
        &self,
        tracked_app_id: TrackedAppId,
        cloud_app_id: &str,
        signature_key: &str,
        signature_subject: Option<&str>,
        signature_issuer: Option<&str>,
        signature_source: &str,
        dry_run: bool,
    ) -> Result<usize> {
        let cloud_app_id = cloud_app_id.trim();
        let signature_key = signature_key.trim();
        let signature_source = signature_source.trim();
        if cloud_app_id.is_empty() {
            return Err(anyhow!("cloud_app_id must not be empty"));
        }
        if signature_key.is_empty() {
            return Err(anyhow!("signature_key must not be empty"));
        }
        if signature_source.is_empty() {
            return Err(anyhow!("signature_source must not be empty"));
        }

        let conn = self.open_connection()?;
        if dry_run {
            let count = conn.query_row(
                r#"
                SELECT COUNT(*)
                FROM observed_endpoints
                WHERE tracked_app_id = ?1
                  AND (
                    cloud_app_id IS NULL OR TRIM(cloud_app_id) = ''
                    OR app_signature_key IS NULL OR TRIM(app_signature_key) = ''
                  )
                "#,
                params![tracked_app_id as i64],
                |row| row.get::<_, i64>(0),
            )?;
            return Ok(count.max(0) as usize);
        }

        conn.execute(
            r#"
            UPDATE observed_endpoints
            SET cloud_app_id = CASE
                    WHEN cloud_app_id IS NULL OR TRIM(cloud_app_id) = '' THEN ?2
                    ELSE cloud_app_id
                END,
                app_signature_source = CASE
                    WHEN app_signature_key IS NULL OR TRIM(app_signature_key) = '' THEN ?6
                    ELSE app_signature_source
                END,
                app_signature_key = CASE
                    WHEN app_signature_key IS NULL OR TRIM(app_signature_key) = '' THEN ?3
                    ELSE app_signature_key
                END,
                app_signature_subject = CASE
                    WHEN app_signature_subject IS NULL OR TRIM(app_signature_subject) = '' THEN ?4
                    ELSE app_signature_subject
                END,
                app_signature_issuer = CASE
                    WHEN app_signature_issuer IS NULL OR TRIM(app_signature_issuer) = '' THEN ?5
                    ELSE app_signature_issuer
                END
            WHERE tracked_app_id = ?1
              AND (
                cloud_app_id IS NULL OR TRIM(cloud_app_id) = ''
                OR app_signature_key IS NULL OR TRIM(app_signature_key) = ''
              )
            "#,
            params![
                tracked_app_id as i64,
                cloud_app_id,
                signature_key,
                signature_subject,
                signature_issuer,
                signature_source,
            ],
        )
        .context("failed to backfill observed endpoint cloud signature")
    }

    pub fn confirm_endpoints(&self, request: ConfirmEndpointsRequest) -> Result<usize> {
        let mut conn = self.open_connection()?;
        let tx = conn.transaction()?;
        let mut changed = 0usize;
        for endpoint_id in request.endpoint_ids {
            changed += tx.execute(
                "UPDATE observed_endpoints
                 SET is_confirmed = ?2,
                     is_exported = 0
                 WHERE id = ?1",
                params![endpoint_id as i64, request.confirmed],
            )?;
        }
        tx.commit()?;
        Ok(changed)
    }

    pub fn mark_endpoints_exported(
        &self,
        request: netstitch_shared::ipc::MarkEndpointsExportedRequest,
    ) -> Result<usize> {
        let endpoint_ids = request
            .endpoint_ids
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if endpoint_ids.is_empty() {
            return Ok(0);
        }
        self.mark_exported(&endpoint_ids)?;
        Ok(endpoint_ids.len())
    }

    pub fn delete_observation(&self, request: DeleteObservationRequest) -> Result<usize> {
        self.delete_observations(DeleteObservationsRequest {
            endpoint_ids: vec![request.endpoint_id],
        })
    }

    pub fn delete_observations(&self, request: DeleteObservationsRequest) -> Result<usize> {
        let endpoint_ids = request
            .endpoint_ids
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if endpoint_ids.is_empty() {
            return Ok(0);
        }

        let mut conn = self.open_connection()?;
        let tx = conn.transaction()?;
        let mut changed = 0usize;
        for chunk in endpoint_ids.chunks(500) {
            let placeholders = std::iter::repeat_n("?", chunk.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!("DELETE FROM observed_endpoints WHERE id IN ({placeholders})");
            let ids = chunk
                .iter()
                .map(|endpoint_id| *endpoint_id as i64)
                .collect::<Vec<_>>();
            changed += tx
                .execute(&sql, rusqlite::params_from_iter(ids))
                .context("failed to delete observed endpoint batch")?;
        }
        tx.commit()
            .context("failed to commit observed endpoint batch delete")?;
        Ok(changed)
    }

    pub fn list_export_runs(&self) -> Result<Vec<ExportRun>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, target_path, created_at_ms, exported_count FROM export_runs ORDER BY created_at_ms DESC",
        )?;
        let rows = stmt.query_map([], map_export_run)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn snapshot(
        &self,
        monitor_status: MonitorStatus,
        integration_path: Option<&Path>,
    ) -> Result<SnapshotResponse> {
        let mut observed_endpoints = self.list_endpoints()?;
        let aggregated_ips = aggregate_ips(&observed_endpoints);
        let app_settings = self.app_settings()?;
        let integration_catalog =
            self.cached_integration_module_catalog(app_settings.ui_language_code.as_deref())?;
        let mut integration_modules = integration_catalog.modules;
        let integration_module_id = self.integration_service.primary_module_id().ok();
        let integration_providers = if let Some(module_id) = integration_module_id.as_deref() {
            self.integration_service
                .providers(module_id)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let integration_status = if let Some(module_id) = integration_module_id.as_deref() {
            let resolved_integration_path = match integration_path {
                Some(path) => Some(normalize_path_buf(path)),
                None => self
                    .integration_service
                    .configured_root(module_id)
                    .unwrap_or(None),
            };
            if let Some(path) = resolved_integration_path.as_deref() {
                self.integration_service
                    .status_for_root(module_id, path)
                    .ok()
            } else {
                self.integration_service.status(module_id).unwrap_or(None)
            }
        } else {
            None
        };

        let tracked_apps = self.list_effective_tracked_apps()?;
        let mut runtime_status = runtime_status_for_tracked_apps(&tracked_apps);
        runtime_status.integration_modules = integration_catalog.statuses;
        self.annotate_integration_background_state(
            &mut integration_modules,
            &mut runtime_status.integration_modules,
        );
        let mut ignored_addresses = self.list_ignored_addresses()?;
        attach_enrichment(&mut observed_endpoints, &mut ignored_addresses, self)?;

        Ok(SnapshotResponse {
            monitor_status,
            tracked_apps,
            observed_endpoints,
            aggregated_ips,
            ignored_addresses,
            capability_report: capability_report(),
            integration_modules,
            integration_providers,
            integration_status,
            app_settings,
            filters: netstitch_shared::models::UiFiltersDto::default(),
            runtime_status,
        })
    }

    fn cached_integration_module_catalog(
        &self,
        language_code: Option<&str>,
    ) -> Result<IntegrationModuleCatalog> {
        let normalized_language = normalize_integration_module_language(language_code);
        let mut cached = self
            .integration_module_catalog
            .lock()
            .map_err(|_| anyhow!("integration module catalog lock poisoned"))?;
        if let Some(catalog) = cached.as_ref() {
            if catalog.language_code == normalized_language {
                return Ok(catalog.clone());
            }
        }

        let catalog = IntegrationModuleCatalog {
            language_code: normalized_language.clone(),
            modules: self
                .integration_service
                .available_modules_for_language(normalized_language.as_deref())?,
            statuses: self
                .integration_service
                .module_runtime_statuses_for_language(normalized_language.as_deref())?,
        };
        *cached = Some(catalog.clone());
        Ok(catalog)
    }

    pub fn export_confirmed(&self, request: ExportConfirmedRequest) -> Result<ExportResultDto> {
        let endpoints = self.list_confirmed_unexported()?;
        let persisted_integration_root = self.configured_integration_root()?;
        let effective_integration_root = request
            .integration_root
            .as_deref()
            .or(persisted_integration_root.as_deref());
        let target_path = if !request.target.path.as_os_str().is_empty() {
            request.target.path.clone()
        } else if effective_integration_root.is_some() {
            let module_id = self.integration_service.primary_module_id()?;
            self.integration_service.user_export_path(&module_id)?
        } else {
            self.paths.export_dir.join("monitoring-export.txt")
        };

        let result = export_confirmed_endpoints(&target_path, &endpoints)?;
        self.persist_export_run(&result.target_path, result.exported_count)?;
        self.mark_exported(&Self::confirmed_endpoint_ids(&endpoints))?;

        if request.generate_overlay_script {
            if effective_integration_root.is_some() {
                let module_id = self.integration_service.primary_module_id()?;
                self.integration_service
                    .ensure_generated_overlay(&module_id)?;
            }
        }

        Ok(result)
    }

    pub fn preview_profile_export(
        &self,
        module_id: Option<&str>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .preview_profile_export(&module_id, request, &endpoints)
    }

    pub fn analyze_profile_export(
        &self,
        module_id: Option<&str>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .analyze_profile_export(&module_id, request, &endpoints)
    }

    pub fn analyze_profile_export_advanced_settings(
        &self,
        module_id: Option<&str>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .analyze_profile_export_advanced_settings(&module_id, request, &endpoints)
    }

    pub fn apply_profile_export(
        &self,
        module_id: Option<&str>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        let result = self
            .integration_service
            .apply_profile_export(&module_id, request, &endpoints)?;
        self.persist_export_run(
            result
                .file_changes
                .first()
                .map(|change| change.path.as_path())
                .unwrap_or_else(|| self.paths.export_dir.as_path()),
            result.exported_count,
        )?;
        Ok(result)
    }

    pub fn apply_profile_export_advanced_settings(
        &self,
        module_id: Option<&str>,
        request: ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        let result = self
            .integration_service
            .apply_profile_export_advanced_settings(&module_id, request, &endpoints)?;
        self.persist_export_run(
            result
                .advanced_file_changes
                .first()
                .or_else(|| result.file_changes.first())
                .map(|change| change.path.as_path())
                .unwrap_or_else(|| self.paths.export_dir.as_path()),
            result.exported_count,
        )?;
        Ok(result)
    }

    pub fn backup_profile_export(
        &self,
        module_id: Option<&str>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .backup_profile_export(&module_id, request, &endpoints)
    }

    pub fn revert_profile_export(
        &self,
        module_id: Option<&str>,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .revert_profile_export(&module_id, request)
    }

    pub fn summarize_integration(&self, path: &Path) -> Result<IntegrationStatusDto> {
        let module_id = self.integration_service.primary_module_id()?;
        self.integration_service.status_for_root(&module_id, path)
    }

    pub fn integration_providers(
        &self,
    ) -> Result<Vec<netstitch_shared::models::IntegrationProviderDto>> {
        let module_id = self.integration_service.primary_module_id()?;
        self.integration_service.providers(&module_id)
    }

    pub fn download_integration_provider(
        &self,
        module_id: Option<&str>,
        provider_id: &str,
    ) -> Result<IntegrationStatusDto> {
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .download_provider(&module_id, provider_id)
    }

    pub fn download_integration_provider_with_progress(
        &self,
        module_id: Option<&str>,
        provider_id: &str,
        progress: impl FnMut(netstitch_shared::models::IntegrationDownloadProgressDto),
    ) -> Result<IntegrationStatusDto> {
        let module_id = self.integration_module_id(module_id)?;
        self.integration_service
            .download_provider_with_progress(&module_id, provider_id, progress)
    }

    pub fn run_integration_module_ui_action(
        &self,
        monitor_status: MonitorStatus,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto> {
        let module_id = self.integration_module_id(Some(&request.module_id))?;
        let (context, selected_rows) = self.integration_module_host_context(
            &module_id,
            monitor_status,
            request.filters,
            &request.selected_monitoring_row_ids,
            &request.displayed_monitoring_row_ids,
        )?;
        self.integration_service.run_ui_action(
            &module_id,
            IntegrationModuleUiActionRequestDto {
                action_id: request.action_id,
                context,
                monitoring_rows: selected_rows,
                payload: request.payload,
            },
        )
    }

    pub fn integration_module_display_name(&self, module_id: &str) -> Result<String> {
        let module_id = self.integration_module_id(Some(module_id))?;
        let app_settings = self.app_settings()?;
        Ok(self
            .integration_service
            .available_modules_for_language(app_settings.ui_language_code.as_deref())?
            .into_iter()
            .find(|module| module.id == module_id)
            .map(|module| module.display_name)
            .unwrap_or(module_id))
    }

    pub fn run_integration_module_dialog_result(
        &self,
        module_id: &str,
        dialog_id: &str,
        result: &str,
    ) -> Result<IntegrationModuleUiActionResponseDto> {
        let module_id = self.integration_module_id(Some(module_id))?;
        let dialog_id = normalize_optional_system_event_text(Some(dialog_id.to_string()))
            .unwrap_or_else(|| "dialog".to_string());
        let result = match result.trim().to_ascii_lowercase().as_str() {
            "ok" => "ok",
            "cancel" => "cancel",
            _ => return Err(anyhow!("dialog result must be ok or cancel")),
        };
        let context = self
            .integration_module_host_context(
                &module_id,
                MonitorStatus::Stopped,
                UiFiltersDto::default(),
                &[],
                &[],
            )?
            .0;
        self.integration_service.run_background_event(
            &module_id,
            IntegrationModuleBackgroundEventDto {
                event_type: "ui.dialog_result".to_string(),
                created_at_ms: current_timestamp_ms(),
                context,
                payload: serde_json::json!({
                    "dialog_id": dialog_id,
                    "result": result,
                }),
            },
        )
    }

    pub fn start_integration_module_background(
        &self,
        module_id: &str,
        subscriptions: Vec<String>,
    ) -> Result<()> {
        let module_id = self.integration_module_id(Some(module_id))?;
        self.integration_service.ensure_module_storage(&module_id)?;
        let subscriptions = normalize_background_subscriptions(subscriptions);
        let task = IntegrationBackgroundTask {
            subscriptions,
            started_at_ms: current_timestamp_ms(),
            last_event_at_ms: None,
            last_status: Some("started".to_string()),
            last_error: None,
            in_flight: false,
        };
        self.integration_background_tasks
            .lock()
            .expect("integration background task lock poisoned")
            .insert(module_id, task);
        Ok(())
    }

    pub fn stop_integration_module_background(&self, module_id: &str) -> Result<bool> {
        let module_id = self.integration_module_id(Some(module_id))?;
        let removed = self
            .integration_background_tasks
            .lock()
            .expect("integration background task lock poisoned")
            .remove(&module_id)
            .is_some();
        Ok(removed)
    }

    pub fn dispatch_integration_module_event(
        &self,
        monitor_status: MonitorStatus,
        filters: netstitch_shared::models::UiFiltersDto,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) {
        let event_type = event_type.into();
        let now = current_timestamp_ms();
        let selected_modules = {
            let mut tasks = self
                .integration_background_tasks
                .lock()
                .expect("integration background task lock poisoned");
            let mut selected = Vec::new();
            for (module_id, task) in tasks.iter_mut() {
                if !background_task_matches(task, &event_type) {
                    continue;
                }
                if task.in_flight {
                    task.last_status = Some("event_skipped_in_flight".to_string());
                    task.last_error = Some(format!(
                        "background event '{event_type}' skipped because previous call is still running"
                    ));
                    continue;
                }
                task.in_flight = true;
                task.last_event_at_ms = Some(now);
                selected.push(module_id.clone());
            }
            selected
        };
        for module_id in selected_modules {
            let Ok((context, _)) = self.integration_module_host_context(
                &module_id,
                monitor_status.clone(),
                filters.clone(),
                &[],
                &[],
            ) else {
                continue;
            };
            let service = self.integration_service.clone();
            let tasks = self.integration_background_tasks.clone();
            let core = self.clone();
            let event_type_for_log = event_type.clone();
            let event = IntegrationModuleBackgroundEventDto {
                event_type: event_type.clone(),
                created_at_ms: now,
                context: context.clone(),
                payload: payload.clone(),
            };
            thread::spawn(move || {
                let result = service.run_background_event(&module_id, event);
                let mut tasks = tasks
                    .lock()
                    .expect("integration background task lock poisoned");
                let Some(task) = tasks.get_mut(&module_id) else {
                    return;
                };
                task.in_flight = false;
                match result {
                    Ok(response) => {
                        if let Some(message) = response
                            .message
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                        {
                            let _ = core.append_system_event(SystemEventRequestDto {
                                source: Some(module_id.clone()),
                                component: "integration".to_string(),
                                action_type: "background_event".to_string(),
                                severity: response.severity.clone(),
                                entity_type: Some("module".to_string()),
                                entity_id: Some(module_id.clone()),
                                payload: serde_json::json!({
                                    "module_id": module_id,
                                    "event_type": event_type_for_log,
                                    "message": message,
                                }),
                            });
                            task.last_status = Some(message);
                        } else {
                            task.last_status = Some("ok".to_string());
                        }
                        task.last_error = None;
                    }
                    Err(error) => {
                        task.last_status = Some("error".to_string());
                        task.last_error = Some(error.to_string());
                    }
                }
            });
        }
    }

    fn integration_module_host_context(
        &self,
        module_id: &str,
        monitor_status: MonitorStatus,
        filters: netstitch_shared::models::UiFiltersDto,
        selected_monitoring_row_ids: &[u64],
        displayed_monitoring_row_ids: &[u64],
    ) -> Result<(IntegrationModuleHostContextDto, Vec<ObservedEndpoint>)> {
        let endpoints = self.list_endpoints_with_enrichment()?;
        let selected_ids = selected_monitoring_row_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let selected_rows = endpoints
            .iter()
            .filter(|endpoint| endpoint.id.is_some_and(|id| selected_ids.contains(&id)))
            .cloned()
            .collect::<Vec<_>>();
        let displayed_ids = displayed_monitoring_row_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let tracked_apps = self.list_effective_tracked_apps()?;
        let app_settings = self.app_settings()?;
        let modules = self
            .integration_service
            .available_modules_for_language(app_settings.ui_language_code.as_deref())?;
        let module_background_active = self
            .integration_background_tasks
            .lock()
            .expect("integration background task lock poisoned")
            .contains_key(module_id);
        let context = IntegrationModuleHostContextDto {
            app_version: runtime_build_version(),
            language_code: app_settings.ui_language_code,
            monitoring_active: matches!(monitor_status, MonitorStatus::Running),
            module_background_active,
            filters,
            tables: vec![IntegrationModuleTableContextDto {
                id: "monitoring".to_string(),
                total_rows: endpoints.len(),
                displayed_rows: if displayed_ids.is_empty() {
                    endpoints.len()
                } else {
                    displayed_ids.len()
                },
                selected_rows: selected_rows.len(),
            }],
            selected_monitoring_row_ids: selected_rows
                .iter()
                .filter_map(|endpoint| endpoint.id)
                .collect(),
            displayed_monitoring_row_ids: displayed_ids.into_iter().collect(),
            integration_module_count: modules.len(),
            tracked_app_count: tracked_apps.len(),
            enabled_tracked_app_count: tracked_apps.iter().filter(|app| app.enabled).count(),
        };
        Ok((context, selected_rows))
    }

    fn annotate_integration_background_state(
        &self,
        modules: &mut [netstitch_shared::models::IntegrationModuleDto],
        statuses: &mut [netstitch_shared::models::IntegrationModuleRuntimeStatusDto],
    ) {
        let tasks = self
            .integration_background_tasks
            .lock()
            .expect("integration background task lock poisoned");
        for module in modules {
            if let Some(task) = tasks.get(&module.id) {
                module.background_active = true;
                module.background_subscriptions = task.subscriptions.iter().cloned().collect();
                module.background_status = task.last_error.clone().or_else(|| {
                    task.last_status
                        .clone()
                        .or_else(|| Some(format!("started_at:{}", task.started_at_ms)))
                });
            }
        }
        for status in statuses {
            if let Some(task) = tasks.get(&status.id) {
                status.background_active = true;
                status.background_subscriptions = task.subscriptions.iter().cloned().collect();
                status.background_status = task.last_error.clone().or_else(|| {
                    task.last_status
                        .clone()
                        .or_else(|| Some(format!("started_at:{}", task.started_at_ms)))
                });
            }
        }
    }

    fn integration_module_id(&self, module_id: Option<&str>) -> Result<String> {
        let module_id = module_id.unwrap_or_default().trim();
        if module_id.is_empty() {
            self.integration_service.primary_module_id()
        } else {
            Ok(module_id.to_string())
        }
    }

    fn list_confirmed_unexported(&self) -> Result<Vec<ObservedEndpoint>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, tracked_app_id, cloud_app_id, app_signature_key, app_signature_subject,
                   app_signature_issuer, app_signature_source, process_id, process_name, remote_ip, remote_port, protocol,
                   first_seen_ms, last_seen_ms, hits, connection_state, failed_hits, successful_hits,
                   is_confirmed, is_exported
            FROM observed_endpoints
            WHERE is_confirmed = 1 AND is_exported = 0
            ORDER BY last_seen_ms DESC, hits DESC
            "#,
        )?;
        let rows = stmt.query_map([], map_endpoint)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    fn confirmed_endpoint_ids(endpoints: &[ObservedEndpoint]) -> Vec<u64> {
        endpoints
            .iter()
            .filter_map(|endpoint| endpoint.id)
            .collect()
    }

    fn list_endpoints_with_enrichment(&self) -> Result<Vec<ObservedEndpoint>> {
        let mut endpoints = self.list_endpoints()?;
        let mut ignored_addresses = Vec::new();
        attach_enrichment(&mut endpoints, &mut ignored_addresses, self)?;
        Ok(endpoints)
    }

    fn persist_export_run(&self, target_path: &Path, exported_count: usize) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute(
            "INSERT INTO export_runs (target_path, created_at_ms, exported_count) VALUES (?1, ?2, ?3)",
            params![
                target_path.to_string_lossy().to_string(),
                current_timestamp_ms() as i64,
                exported_count as i64
            ],
        )?;
        Ok(())
    }

    fn mark_exported(&self, endpoint_ids: &[u64]) -> Result<()> {
        let mut conn = self.open_connection()?;
        let tx = conn.transaction()?;
        for endpoint_id in endpoint_ids {
            tx.execute(
                "UPDATE observed_endpoints SET is_exported = 1 WHERE id = ?1",
                params![*endpoint_id as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn find_endpoint(
        &self,
        tracked_app_id: u64,
        remote_ip: IpAddr,
        remote_port: u16,
        protocol: Protocol,
    ) -> Result<Option<ObservedEndpoint>> {
        let conn = self.open_connection()?;
        conn.query_row(
            r#"
            SELECT id, tracked_app_id, cloud_app_id, app_signature_key, app_signature_subject,
                   app_signature_issuer, app_signature_source, process_id, process_name, remote_ip, remote_port, protocol,
                   first_seen_ms, last_seen_ms, hits, connection_state, failed_hits, successful_hits,
                   is_confirmed, is_exported
            FROM observed_endpoints
            WHERE tracked_app_id = ?1 AND remote_ip = ?2 AND remote_port = ?3 AND protocol = ?4
            "#,
            params![
                tracked_app_id as i64,
                remote_ip.to_string(),
                i64::from(remote_port),
                protocol.as_str()
            ],
            map_endpoint,
        )
        .optional()
        .context("failed to reload endpoint")
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS tracked_apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exe_path TEXT NOT NULL,
                connector_id TEXT,
                cloud_app_id TEXT,
                process_name TEXT,
                display_name TEXT,
                icon_key TEXT,
                icon_path TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                archived INTEGER NOT NULL DEFAULT 0,
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS observed_endpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tracked_app_id INTEGER NOT NULL,
                cloud_app_id TEXT,
                app_signature_key TEXT,
                app_signature_subject TEXT,
                app_signature_issuer TEXT,
                app_signature_source TEXT,
                process_id INTEGER,
                process_name TEXT,
                remote_ip TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                first_seen_ms INTEGER NOT NULL,
                last_seen_ms INTEGER NOT NULL,
                hits INTEGER NOT NULL DEFAULT 1,
                connection_state TEXT NOT NULL DEFAULT 'unknown',
                failed_hits INTEGER NOT NULL DEFAULT 0,
                successful_hits INTEGER NOT NULL DEFAULT 0,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                is_exported INTEGER NOT NULL DEFAULT 0,
                UNIQUE(tracked_app_id, remote_ip, remote_port, protocol)
            );

            CREATE TABLE IF NOT EXISTS export_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                target_path TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                exported_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                setting_key TEXT PRIMARY KEY,
                setting_value TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ignored_addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address_pattern TEXT NOT NULL UNIQUE,
                created_at_ms INTEGER NOT NULL
            );

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

            CREATE TABLE IF NOT EXISTS ip_domain_cache (
                remote_ip TEXT PRIMARY KEY,
                lookup_status TEXT NOT NULL,
                error_text TEXT,
                last_attempt_ms INTEGER NOT NULL,
                updated_at_ms INTEGER
            );

            CREATE TABLE IF NOT EXISTS endpoint_domain_cache (
                tracked_app_id INTEGER NOT NULL,
                remote_ip TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                domain_name TEXT NOT NULL,
                domain_source TEXT NOT NULL,
                observed_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                PRIMARY KEY(tracked_app_id, remote_ip, remote_port, protocol)
            );

            CREATE TABLE IF NOT EXISTS ip_whois_ranges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cidr TEXT NOT NULL UNIQUE,
                range_start TEXT NOT NULL,
                range_end TEXT NOT NULL,
                owner_name TEXT,
                registry TEXT,
                country TEXT,
                source TEXT,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_ip_whois_ranges_updated
                ON ip_whois_ranges(updated_at_ms DESC, id DESC);
            "#,
        )?;
        ensure_observed_endpoints_columns(&conn)?;
        ensure_system_events_columns(&conn)?;
        ensure_observed_endpoints_without_tracked_app_fk(&conn)?;
        ensure_tracked_apps_columns(&conn)?;
        migrate_csv_import_tracked_apps_to_imported_rows(&conn)?;
        seed_default_ignored_addresses(&conn)?;
        seed_local_machine_ignored_addresses(&conn)?;
        Ok(())
    }

    fn ensure_connector_tracked_apps(&self) -> Result<()> {
        let conn = self.open_connection()?;
        let active_connector_ids = netstitch_connectors::manifests()
            .into_iter()
            .filter(|manifest| !manifest.manual_only)
            .map(|manifest| manifest.id.to_string())
            .collect::<BTreeSet<_>>();
        remove_orphaned_connector_tracked_apps(&conn, &active_connector_ids)?;
        seed_connector_tracked_apps(
            &conn,
            &netstitch_connectors::discover_installed_apps(),
            &self.paths.connector_icon_dir,
            current_timestamp_ms(),
        )?;
        Ok(())
    }

    fn refresh_manual_tracked_app_icons(&self) -> Result<usize> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, exe_path, icon_path
            FROM tracked_apps
            WHERE COALESCE(archived, 0) = 0
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)? as TrackedAppId,
                PathBuf::from(row.get::<_, String>(1)?),
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut refreshed = 0;
        for row in rows {
            let (tracked_app_id, exe_path, existing_icon_path) = row?;
            if tracked_app_icon_path_is_usable(existing_icon_path.as_deref()) {
                continue;
            }
            let Some(icon_path) = cache_executable_icon(&exe_path, &self.paths.icon_cache_dir)
                .or_else(|| cache_manual_icon(&exe_path, &self.paths.icon_cache_dir))
            else {
                continue;
            };
            let icon_path_text = icon_path.to_string_lossy().to_string();
            if existing_icon_path.as_deref() == Some(icon_path_text.as_str()) {
                continue;
            }
            refreshed += conn.execute(
                "UPDATE tracked_apps SET icon_path = ?2 WHERE id = ?1",
                params![tracked_app_id as i64, icon_path_text],
            )?;
        }
        Ok(refreshed)
    }

    fn open_connection(&self) -> Result<Connection> {
        Connection::open(&self.paths.database_path).with_context(|| {
            format!(
                "failed to open SQLite database at {}",
                self.paths.database_path.display()
            )
        })
    }
}

fn map_tracked_app(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackedApp> {
    Ok(TrackedApp {
        id: Some(row.get::<_, i64>(0)? as u64),
        exe_path: PathBuf::from(row.get::<_, String>(1)?),
        connector_id: row.get(2)?,
        cloud_app_id: row.get(3)?,
        process_name: row.get(4)?,
        display_name: row.get(5)?,
        icon_key: row.get(6)?,
        icon_path: row.get::<_, Option<String>>(7)?.map(PathBuf::from),
        enabled: row.get::<_, bool>(8)?,
        created_at_ms: row.get::<_, i64>(9)? as u64,
    })
}

fn map_endpoint(row: &rusqlite::Row<'_>) -> rusqlite::Result<ObservedEndpoint> {
    let remote_ip = row.get::<_, String>(9)?;
    let parsed_ip = remote_ip.parse::<IpAddr>().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(ObservedEndpoint {
        id: Some(row.get::<_, i64>(0)? as u64),
        tracked_app_id: row.get::<_, i64>(1)? as u64,
        cloud_app_id: row.get(2)?,
        app_signature_key: row.get(3)?,
        app_signature_subject: row.get(4)?,
        app_signature_issuer: row.get(5)?,
        app_signature_source: row.get(6)?,
        process_id: row.get::<_, Option<i64>>(7)?.map(|value| value as u32),
        process_name: row.get(8)?,
        remote_ip: parsed_ip,
        remote_port: row.get::<_, i64>(10)? as u16,
        protocol: Protocol::from_db_value(row.get::<_, String>(11)?),
        first_seen_ms: row.get::<_, i64>(12)? as u64,
        last_seen_ms: row.get::<_, i64>(13)? as u64,
        hits: row.get::<_, i64>(14)? as u64,
        connection_state: ConnectionState::from_db_value(row.get::<_, String>(15)?),
        failed_hits: row.get::<_, i64>(16)? as u64,
        successful_hits: row.get::<_, i64>(17)? as u64,
        is_confirmed: row.get::<_, bool>(18)?,
        is_exported: row.get::<_, bool>(19)?,
        enrichment: None,
    })
}

fn map_export_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportRun> {
    Ok(ExportRun {
        id: Some(row.get::<_, i64>(0)? as u64),
        target_path: PathBuf::from(row.get::<_, String>(1)?),
        created_at_ms: row.get::<_, i64>(2)? as u64,
        exported_count: row.get::<_, i64>(3)? as u64,
    })
}

fn map_ignored_address_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<IgnoredAddressRule> {
    Ok(IgnoredAddressRule {
        id: Some(row.get::<_, i64>(0)? as u64),
        address_pattern: row.get(1)?,
        created_at_ms: row.get::<_, i64>(2)? as u64,
        enrichment: None,
    })
}

fn map_system_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<SystemEventDto> {
    let payload_json: String = row.get(9)?;
    let payload = serde_json::from_str(&payload_json).unwrap_or_else(|_| serde_json::json!({}));
    Ok(SystemEventDto {
        event_id: row.get::<_, i64>(0)? as u64,
        created_at_ms: row.get::<_, i64>(1)? as u64,
        repeat_count: row.get::<_, i64>(2)? as u64,
        source: row.get(3)?,
        component: row.get(4)?,
        action_type: row.get(5)?,
        severity: row.get(6)?,
        entity_type: row.get(7)?,
        entity_id: row.get(8)?,
        payload,
    })
}

fn normalize_system_event_token(value: &str, fallback: &str) -> String {
    let mut normalized = String::new();
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '_' | '-' | '.' | ':') {
            normalized.push(ch);
        }
        if normalized.len() >= 64 {
            break;
        }
    }
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized
    }
}

fn normalize_system_event_severity(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" | "info" | "success" | "warning" | "error" => value.trim().to_ascii_lowercase(),
        "warn" => "warning".to_string(),
        _ => "info".to_string(),
    }
}

fn normalize_system_event_source(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "core".to_string();
    };
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

fn normalize_optional_system_event_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.chars().take(256).collect())
        }
    })
}

fn rotate_system_events(tx: &rusqlite::Transaction<'_>) -> Result<()> {
    tx.execute(
        r#"
        DELETE FROM system_events
        WHERE event_id NOT IN (
            SELECT event_id
            FROM system_events
            ORDER BY event_id DESC
            LIMIT ?1
        )
        "#,
        params![MAX_SYSTEM_EVENT_ROWS],
    )
    .context("failed to rotate system events")?;
    Ok(())
}

fn attach_enrichment(
    endpoints: &mut [ObservedEndpoint],
    ignored_addresses: &mut [IgnoredAddressRule],
    core: &NetstitchCore,
) -> Result<()> {
    let mut ips = BTreeMap::<IpAddr, ()>::new();
    for endpoint in endpoints.iter() {
        ips.insert(endpoint.remote_ip, ());
    }
    for rule in ignored_addresses.iter() {
        if let Some(ip) = ignored_rule_base_ip(&rule.address_pattern) {
            ips.insert(ip, ());
        }
    }

    let ip_enrichment = core.ip_enrichment_map_for_ips(ips.keys().copied().collect::<Vec<_>>())?;
    let endpoint_domains = core.verified_domain_map_for_endpoints(endpoints)?;
    for endpoint in endpoints {
        let mut item = ip_enrichment
            .get(&endpoint.remote_ip)
            .cloned()
            .unwrap_or_default();
        if let Some(domain) = endpoint_domains.get(&(
            endpoint.tracked_app_id,
            endpoint.remote_ip,
            endpoint.remote_port,
            endpoint.protocol,
        )) {
            item.domain_name = Some(domain.domain_name.clone());
            item.domain_source = Some(domain.domain_source.clone());
            item.updated_at_ms = Some(
                item.updated_at_ms
                    .unwrap_or(domain.updated_at_ms)
                    .max(domain.updated_at_ms),
            );
        }
        if !item.is_empty() {
            endpoint.enrichment = Some(item);
        }
    }
    for rule in ignored_addresses {
        rule.enrichment = ignored_rule_base_ip(&rule.address_pattern)
            .and_then(|ip| ip_enrichment.get(&ip).cloned());
    }
    Ok(())
}

impl NetstitchCore {
    fn verified_domain_map_for_endpoints(
        &self,
        endpoints: &[ObservedEndpoint],
    ) -> Result<BTreeMap<(u64, IpAddr, u16, Protocol), EndpointDomainCacheRecord>> {
        if endpoints.is_empty() {
            return Ok(BTreeMap::new());
        }

        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT domain_name, domain_source, observed_at_ms, updated_at_ms
            FROM endpoint_domain_cache
            WHERE tracked_app_id = ?1
              AND remote_ip = ?2
              AND remote_port = ?3
              AND protocol = ?4
              AND domain_name IS NOT NULL
              AND TRIM(domain_name) <> ''
            "#,
        )?;
        let mut output = BTreeMap::new();
        for endpoint in endpoints {
            if let Some((domain_name, domain_source, observed_at_ms, updated_at_ms)) = stmt
                .query_row(
                    params![
                        endpoint.tracked_app_id as i64,
                        endpoint.remote_ip.to_string(),
                        i64::from(endpoint.remote_port),
                        endpoint.protocol.as_str(),
                    ],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    },
                )
                .optional()?
            {
                output.insert(
                    (
                        endpoint.tracked_app_id,
                        endpoint.remote_ip,
                        endpoint.remote_port,
                        endpoint.protocol,
                    ),
                    EndpointDomainCacheRecord {
                        tracked_app_id: endpoint.tracked_app_id,
                        remote_ip: endpoint.remote_ip,
                        remote_port: endpoint.remote_port,
                        protocol: endpoint.protocol,
                        domain_name,
                        domain_source,
                        observed_at_ms: observed_at_ms.max(0) as u64,
                        updated_at_ms: updated_at_ms.max(0) as u64,
                    },
                );
            }
        }
        Ok(output)
    }

    fn ip_enrichment_map_for_ips(
        &self,
        ips: Vec<IpAddr>,
    ) -> Result<BTreeMap<IpAddr, IpEnrichmentDto>> {
        if ips.is_empty() {
            return Ok(BTreeMap::new());
        }

        let conn = self.open_connection()?;
        let ranges = load_whois_ranges(&conn)?;
        let mut output = BTreeMap::new();
        for ip in ips {
            let mut item = IpEnrichmentDto::default();
            if let Some(range) = ranges.iter().find(|range| ip_matches_cidr(ip, &range.cidr)) {
                if item.owner_name.is_none() {
                    item.owner_name = range.owner_name.clone();
                }
                if item.owner_range.is_none() {
                    item.owner_range = Some(range.cidr.clone());
                }
                if item.registry.is_none() {
                    item.registry = range.registry.clone();
                }
                if item.country.is_none() {
                    item.country = range.country.clone();
                }
                if item.source.is_none() {
                    item.source = range.source.clone();
                }
                item.updated_at_ms = Some(
                    item.updated_at_ms
                        .unwrap_or(range.updated_at_ms)
                        .max(range.updated_at_ms),
                );
            }
            if !item.is_empty() {
                output.insert(ip, item);
            }
        }
        Ok(output)
    }
}

fn import_monitoring_csv_row(
    tx: &rusqlite::Transaction<'_>,
    row: MonitoringCsvImportRowDto,
    now: u64,
) -> Result<bool> {
    let app_name = normalized_csv_app_name(&row.application);
    let connector_id = normalized_csv_connector_id(&row);
    let cloud_app_id = normalized_csv_cloud_app_id(&row, connector_id.as_deref());
    let app_signature_key = normalized_optional_csv_value(row.app_signature_key.as_deref());
    let app_signature_subject = normalized_optional_csv_value(row.app_signature_subject.as_deref());
    let app_signature_issuer = normalized_optional_csv_value(row.app_signature_issuer.as_deref());
    let app_signature_source = app_signature_key.as_ref().map(|_| "csv".to_string());
    let tracked_app_id = find_csv_import_tracked_app(
        tx,
        &app_name,
        connector_id.as_deref(),
        cloud_app_id.as_deref(),
    )?
    .unwrap_or(0);
    let first_seen_ms = if row.first_seen_ms == 0 {
        now
    } else {
        row.first_seen_ms
    };
    let last_seen_ms = row.last_seen_ms.max(first_seen_ms);
    let hits = row
        .hits
        .max(row.failed_hits.saturating_add(row.successful_hits))
        .max(1);
    let remote_ip = row.remote_ip.to_string();
    let protocol = row.protocol.as_str().to_string();

    let existing_endpoint_id = tx
        .query_row(
            r#"
            SELECT id FROM observed_endpoints
            WHERE tracked_app_id = ?1 AND remote_ip = ?2 AND remote_port = ?3 AND protocol = ?4
            ORDER BY id ASC
            LIMIT 1
            "#,
            params![
                tracked_app_id as i64,
                remote_ip,
                i64::from(row.remote_port),
                protocol
            ],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .context("failed to look up existing imported endpoint")?;

    let endpoint_id = if existing_endpoint_id.is_some() {
        return Ok(false);
    } else {
        tx.execute(
            r#"
            INSERT INTO observed_endpoints (
                tracked_app_id,
                cloud_app_id,
                app_signature_key,
                app_signature_subject,
                app_signature_issuer,
                app_signature_source,
                process_id,
                process_name,
                remote_ip,
                remote_port,
                protocol,
                first_seen_ms,
                last_seen_ms,
                hits,
                connection_state,
                failed_hits,
                successful_hits,
                is_confirmed,
                is_exported
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0, 0)
            "#,
            params![
                tracked_app_id as i64,
                cloud_app_id,
                app_signature_key,
                app_signature_subject,
                app_signature_issuer,
                app_signature_source,
                app_name,
                row.remote_ip.to_string(),
                i64::from(row.remote_port),
                row.protocol.as_str(),
                first_seen_ms as i64,
                last_seen_ms as i64,
                hits as i64,
                row.connection_state.as_str(),
                row.failed_hits as i64,
                row.successful_hits as i64,
            ],
        )
        .context("failed to insert imported endpoint")?;
        tx.last_insert_rowid() as u64
    };

    if let Some(domain) = row
        .domain
        .as_deref()
        .and_then(netstitch_shared::normalize_verified_domain)
    {
        tx.execute(
            r#"
            INSERT INTO endpoint_domain_cache (
                tracked_app_id,
                remote_ip,
                remote_port,
                protocol,
                domain_name,
                domain_source,
                observed_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(tracked_app_id, remote_ip, remote_port, protocol) DO UPDATE SET
                domain_name = excluded.domain_name,
                domain_source = excluded.domain_source,
                observed_at_ms = excluded.observed_at_ms,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![
                tracked_app_id as i64,
                row.remote_ip.to_string(),
                i64::from(row.remote_port),
                row.protocol.as_str(),
                domain,
                netstitch_shared::DOMAIN_SOURCE_CSV_IMPORT,
                last_seen_ms as i64,
                now as i64,
            ],
        )
        .with_context(|| format!("failed to import CSV domain for endpoint {endpoint_id}"))?;
    }

    Ok(true)
}

fn find_csv_import_tracked_app(
    tx: &rusqlite::Transaction<'_>,
    app_name: &str,
    connector_id: Option<&str>,
    cloud_app_id: Option<&str>,
) -> Result<Option<TrackedAppId>> {
    if let Some(cloud_app_id) = cloud_app_id {
        if let Some(id) = tx
            .query_row(
                r#"
                SELECT id FROM tracked_apps
                WHERE cloud_app_id = ?1
                  AND COALESCE(archived, 0) = 0
                ORDER BY
                    CASE WHEN exe_path NOT LIKE 'NetStitch-csv-import://%' THEN 0 ELSE 1 END ASC,
                    id ASC
                LIMIT 1
                "#,
                params![cloud_app_id],
                |row| row.get::<_, i64>(0).map(|id| id as TrackedAppId),
            )
            .optional()
            .context("failed to look up cloud app for CSV import")?
        {
            return Ok(Some(id));
        }
    }

    if let Some(connector_id) = connector_id {
        if let Some(id) = tx
            .query_row(
                r#"
                SELECT id FROM tracked_apps
                WHERE connector_id = ?1
                  AND COALESCE(archived, 0) = 0
                ORDER BY id ASC
                LIMIT 1
                "#,
                params![connector_id],
                |row| row.get::<_, i64>(0).map(|id| id as TrackedAppId),
            )
            .optional()
            .context("failed to look up connector tracked app for CSV import")?
        {
            return Ok(Some(id));
        }
    }

    if let Some(id) = tx
        .query_row(
            r#"
            SELECT id FROM tracked_apps
            WHERE LOWER(COALESCE(display_name, '')) = LOWER(?1)
               OR LOWER(COALESCE(process_name, '')) = LOWER(?1)
            ORDER BY id ASC
            LIMIT 1
            "#,
            params![app_name],
            |row| row.get::<_, i64>(0).map(|id| id as TrackedAppId),
        )
        .optional()
        .context("failed to look up tracked app for CSV import")?
    {
        return Ok(Some(id));
    }

    Ok(None)
}

fn normalized_csv_connector_id(row: &MonitoringCsvImportRowDto) -> Option<String> {
    row.app_connector_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| netstitch_shared::cloud_app_for_connector_id(value))
        .map(|app| app.connector_id.to_string())
        .or_else(|| {
            row.cloud_app_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .and_then(netstitch_shared::cloud_connector_id_for_app_id)
                .map(ToOwned::to_owned)
        })
}

fn normalized_csv_cloud_app_id(
    row: &MonitoringCsvImportRowDto,
    connector_id: Option<&str>,
) -> Option<String> {
    row.cloud_app_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            connector_id.and_then(|value| {
                netstitch_shared::cloud_app_for_connector_id(value)
                    .map(|app| app.app_id.to_string())
            })
        })
}

fn normalized_optional_csv_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalized_csv_app_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "CSV import".to_string()
    } else {
        trimmed.to_string()
    }
}

fn load_whois_ranges(conn: &Connection) -> Result<Vec<WhoisRangeRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT cidr, owner_name, registry, country, source, updated_at_ms
        FROM ip_whois_ranges
        ORDER BY updated_at_ms DESC, id DESC
        LIMIT ?1
        "#,
    )?;
    let rows = stmt.query_map(params![MAX_WHOIS_RANGE_CACHE_ROWS as i64], |row| {
        Ok(WhoisRangeRow {
            cidr: row.get(0)?,
            owner_name: row.get(1)?,
            registry: row.get(2)?,
            country: row.get(3)?,
            source: row.get(4)?,
            updated_at_ms: row.get::<_, i64>(5)?.max(0) as u64,
        })
    })?;

    let mut output = Vec::new();
    for row in rows {
        output.push(row?);
    }
    Ok(output)
}

fn prune_whois_range_cache(conn: &Connection, max_rows: usize) -> Result<()> {
    conn.execute(
        r#"
        DELETE FROM ip_whois_ranges
        WHERE id NOT IN (
            SELECT id
            FROM ip_whois_ranges
            ORDER BY updated_at_ms DESC, id DESC
            LIMIT ?1
        )
        "#,
        params![max_rows as i64],
    )
    .context("failed to rotate whois range cache")?;
    Ok(())
}

fn seed_default_ignored_addresses(conn: &Connection) -> Result<()> {
    let already_seeded = conn
        .query_row(
            "SELECT setting_value FROM app_settings WHERE setting_key = ?1",
            params![SETTING_IGNORE_DEFAULTS_SEEDED],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some_and(|value| setting_truthy(&value));
    if already_seeded {
        return Ok(());
    }

    let now = current_timestamp_ms() as i64;
    for address_pattern in DEFAULT_IGNORED_ADDRESS_RULES {
        conn.execute(
            r#"
            INSERT OR IGNORE INTO ignored_addresses (address_pattern, created_at_ms)
            VALUES (?1, ?2)
            "#,
            params![address_pattern, now],
        )?;
    }
    conn.execute(
        r#"
        INSERT INTO app_settings (setting_key, setting_value, updated_at_ms)
        VALUES (?1, 'true', ?2)
        ON CONFLICT(setting_key) DO UPDATE SET
            setting_value = excluded.setting_value,
            updated_at_ms = excluded.updated_at_ms
        "#,
        params![SETTING_IGNORE_DEFAULTS_SEEDED, now],
    )?;
    Ok(())
}

fn seed_local_machine_ignored_addresses(conn: &Connection) -> Result<()> {
    let now = current_timestamp_ms() as i64;
    for address_pattern in local_machine_ignored_address_rules() {
        conn.execute(
            r#"
            INSERT OR IGNORE INTO ignored_addresses (address_pattern, created_at_ms)
            VALUES (?1, ?2)
            "#,
            params![address_pattern, now],
        )?;
    }
    Ok(())
}

fn local_machine_ignored_address_rules() -> Vec<String> {
    let mut rules = Vec::new();
    for rule in [
        detect_local_ignore_rule_for_routes(
            "0.0.0.0:0",
            LOCAL_IPV4_ROUTE_PROBE_TARGETS,
            local_ip_probe_timeout(),
        ),
        detect_local_ignore_rule_for_routes(
            "[::]:0",
            LOCAL_IPV6_ROUTE_PROBE_TARGETS,
            local_ip_probe_timeout(),
        ),
    ]
    .into_iter()
    .flatten()
    {
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    rules
}

fn local_ip_probe_timeout() -> Duration {
    Duration::from_secs(10)
}

fn detect_local_ignore_rule_for_routes(
    bind: &str,
    targets: &[&str],
    timeout: Duration,
) -> Option<String> {
    targets
        .iter()
        .filter_map(|target| detect_local_ip_for_route(bind, target, timeout))
        .find_map(local_ip_ignore_rule)
}

fn detect_local_ip_for_route(bind: &str, target: &str, timeout: Duration) -> Option<IpAddr> {
    let bind = bind.to_string();
    let target = target.to_string();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(detect_local_ip_for_route_inner(&bind, &target));
    });
    receiver.recv_timeout(timeout).ok().flatten()
}

fn detect_local_ip_for_route_inner(bind: &str, target: &str) -> Option<IpAddr> {
    let socket = UdpSocket::bind(bind).ok()?;
    socket.connect(target).ok()?;
    Some(socket.local_addr().ok()?.ip())
}

fn local_ip_ignore_rule(ip: IpAddr) -> Option<String> {
    if !is_real_local_machine_address(ip) {
        return None;
    }

    match ip {
        IpAddr::V4(address) => Some(format!("{address}/32")),
        IpAddr::V6(address) => Some(format!("{address}/128")),
    }
}

fn is_real_local_machine_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => is_real_local_machine_ipv4(address),
        IpAddr::V6(address) => is_real_local_machine_ipv6(address),
    }
}

fn is_real_local_machine_ipv4(address: Ipv4Addr) -> bool {
    !(address.is_loopback()
        || address.is_unspecified()
        || address.is_broadcast()
        || address.is_multicast()
        || address.is_documentation())
}

fn is_real_local_machine_ipv6(address: Ipv6Addr) -> bool {
    !(address.is_loopback()
        || address.is_unspecified()
        || address.is_multicast()
        || is_ipv6_documentation(address)
        || is_ipv6_teredo(address)
        || is_ipv6_6to4(address))
}

fn is_ipv6_documentation(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    segments[0] == 0x2001 && segments[1] == 0x0db8
}

fn is_ipv6_teredo(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    segments[0] == 0x2001 && segments[1] == 0
}

fn is_ipv6_6to4(address: Ipv6Addr) -> bool {
    address.segments()[0] == 0x2002
}

fn ignored_rule_base_ip(pattern: &str) -> Option<IpAddr> {
    let trimmed = pattern.trim();
    let address = trimmed
        .split_once('/')
        .map_or(trimmed, |(address, _)| address);
    address.trim().parse::<IpAddr>().ok()
}

fn normalize_ignored_address_pattern(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(anyhow!("ignored address must not be empty"));
    }

    if let Some((address, prefix)) = value.split_once('/') {
        let ip = address
            .trim()
            .parse::<IpAddr>()
            .with_context(|| format!("invalid ignored address CIDR: {value}"))?;
        let prefix: u8 = prefix
            .trim()
            .parse()
            .with_context(|| format!("invalid ignored address CIDR prefix: {value}"))?;
        match ip {
            IpAddr::V4(_) if prefix <= 32 => Ok(format!("{ip}/{prefix}")),
            IpAddr::V6(_) if prefix <= 128 => Ok(format!("{ip}/{prefix}")),
            IpAddr::V4(_) => Err(anyhow!("IPv4 CIDR prefix must be <= 32")),
            IpAddr::V6(_) => Err(anyhow!("IPv6 CIDR prefix must be <= 128")),
        }
    } else {
        let ip = value
            .parse::<IpAddr>()
            .with_context(|| format!("invalid ignored address: {value}"))?;
        Ok(ip.to_string())
    }
}

fn cidr_bounds(cidr: &str) -> Option<(String, String)> {
    let (network, prefix) = cidr.split_once('/')?;
    let network = network.trim().parse::<IpAddr>().ok()?;
    let prefix = prefix.trim().parse::<u8>().ok()?;
    match network {
        IpAddr::V4(address) if prefix <= 32 => {
            let mask = prefix_mask_v4(prefix);
            let start = u32::from(address) & mask;
            let end = start | !mask;
            Some((
                Ipv4Addr::from(start).to_string(),
                Ipv4Addr::from(end).to_string(),
            ))
        }
        IpAddr::V6(address) if prefix <= 128 => {
            let mask = prefix_mask_v6(prefix);
            let start = u128::from(address) & mask;
            let end = start | !mask;
            Some((
                Ipv6Addr::from(start).to_string(),
                Ipv6Addr::from(end).to_string(),
            ))
        }
        _ => None,
    }
}

fn ip_matches_cidr(address: IpAddr, cidr: &str) -> bool {
    let Some((network, prefix)) = cidr.split_once('/') else {
        return false;
    };
    let Ok(network) = network.trim().parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.trim().parse::<u8>() else {
        return false;
    };
    match (address, network) {
        (IpAddr::V4(address), IpAddr::V4(network)) if prefix <= 32 => {
            let mask = prefix_mask_v4(prefix);
            (u32::from(address) & mask) == (u32::from(network) & mask)
        }
        (IpAddr::V6(address), IpAddr::V6(network)) if prefix <= 128 => {
            let mask = prefix_mask_v6(prefix);
            (u128::from(address) & mask) == (u128::from(network) & mask)
        }
        _ => false,
    }
}

fn prefix_mask_v4(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

fn prefix_mask_v6(prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    }
}

fn aggregate_ips(endpoints: &[ObservedEndpoint]) -> Vec<AggregatedIpDto> {
    let mut by_ip: BTreeMap<IpAddr, AggregatedIpDto> = BTreeMap::new();

    for endpoint in endpoints {
        let entry = by_ip
            .entry(endpoint.remote_ip)
            .or_insert_with(|| AggregatedIpDto::new(endpoint.remote_ip));
        entry.tracked_app_ids.push(endpoint.tracked_app_id);
        if let Some(process_name) = endpoint.process_name.as_ref() {
            if !entry.process_names.contains(process_name) {
                entry.process_names.push(process_name.clone());
            }
        }
        if let Some(process_id) = endpoint.process_id {
            if !entry.process_ids.contains(&process_id) {
                entry.process_ids.push(process_id);
            }
        }
        if !entry.remote_ports.contains(&endpoint.remote_port) {
            entry.remote_ports.push(endpoint.remote_port);
        }
        if !entry.protocols.contains(&endpoint.protocol) {
            entry.protocols.push(endpoint.protocol);
        }
        entry.first_seen_ms = if entry.first_seen_ms == 0 {
            endpoint.first_seen_ms
        } else {
            entry.first_seen_ms.min(endpoint.first_seen_ms)
        };
        entry.last_seen_ms = entry.last_seen_ms.max(endpoint.last_seen_ms);
        entry.hits = entry.hits.saturating_add(endpoint.hits);
        entry.failed_hits = entry.failed_hits.saturating_add(endpoint.failed_hits);
        entry.successful_hits = entry
            .successful_hits
            .saturating_add(endpoint.successful_hits);
        entry.is_confirmed |= endpoint.is_confirmed;
        entry.is_exported |= endpoint.is_exported;
    }

    by_ip.into_values().collect()
}

fn normalize_path_buf(path: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(normalize_path_text_for_platform(
        &path.as_ref().to_string_lossy(),
        cfg!(windows),
    ))
}

fn resolve_executable_entry_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = normalize_path_buf(path);
    #[cfg(target_os = "windows")]
    {
        if path_extension_eq(&path, "lnk") {
            return resolve_windows_shortcut_target(&path)
                .map(normalize_path_buf)
                .with_context(|| format!("failed to resolve shortcut {}", path.display()));
        }
    }
    Ok(path)
}

#[cfg(any(target_os = "windows", test))]
fn path_extension_eq(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

#[cfg(target_os = "windows")]
fn resolve_windows_shortcut_target(shortcut_path: &Path) -> Result<PathBuf> {
    let bytes = fs::read(shortcut_path)
        .with_context(|| format!("failed to read shortcut {}", shortcut_path.display()))?;
    parse_windows_shortcut_target(&bytes).ok_or_else(|| anyhow!("shortcut target is empty"))
}

#[cfg(target_os = "windows")]
fn parse_windows_shortcut_target(bytes: &[u8]) -> Option<PathBuf> {
    const SHELL_LINK_HEADER_SIZE: usize = 0x4c;
    const HAS_LINK_TARGET_ID_LIST: u32 = 0x0000_0001;
    const HAS_LINK_INFO: u32 = 0x0000_0002;

    if bytes.len() < SHELL_LINK_HEADER_SIZE
        || read_u32_le(bytes, 0)? != SHELL_LINK_HEADER_SIZE as u32
    {
        return None;
    }

    let flags = read_u32_le(bytes, 0x14)?;
    let mut cursor = SHELL_LINK_HEADER_SIZE;
    if flags & HAS_LINK_TARGET_ID_LIST != 0 {
        let id_list_size = read_u16_le(bytes, cursor)? as usize;
        cursor = cursor.checked_add(2)?.checked_add(id_list_size)?;
    }
    if flags & HAS_LINK_INFO == 0 || cursor >= bytes.len() {
        return None;
    }

    let link_info_size = read_u32_le(bytes, cursor)? as usize;
    let link_info_end = cursor.checked_add(link_info_size)?;
    if link_info_size < 0x1c || link_info_end > bytes.len() {
        return None;
    }

    let header_size = read_u32_le(bytes, cursor + 0x04)? as usize;
    let local_base_path_offset = read_u32_le(bytes, cursor + 0x10)? as usize;
    let common_suffix_offset = read_u32_le(bytes, cursor + 0x18)? as usize;
    let local_base_path_unicode_offset = if header_size >= 0x24 {
        read_u32_le(bytes, cursor + 0x1c).unwrap_or(0) as usize
    } else {
        0
    };
    let common_suffix_unicode_offset = if header_size >= 0x24 {
        read_u32_le(bytes, cursor + 0x24).unwrap_or(0) as usize
    } else {
        0
    };

    let base = read_link_info_string(
        bytes,
        cursor,
        link_info_end,
        local_base_path_unicode_offset,
        true,
    )
    .or_else(|| {
        read_link_info_string(bytes, cursor, link_info_end, local_base_path_offset, false)
    })?;
    let suffix = read_link_info_string(
        bytes,
        cursor,
        link_info_end,
        common_suffix_unicode_offset,
        true,
    )
    .or_else(|| read_link_info_string(bytes, cursor, link_info_end, common_suffix_offset, false))
    .unwrap_or_default();
    Some(PathBuf::from(join_shortcut_base_and_suffix(base, suffix)))
}

#[cfg(target_os = "windows")]
fn read_link_info_string(
    bytes: &[u8],
    link_info_start: usize,
    link_info_end: usize,
    offset: usize,
    unicode: bool,
) -> Option<String> {
    if offset == 0 {
        return None;
    }
    let start = link_info_start.checked_add(offset)?;
    if start >= link_info_end {
        return None;
    }
    if unicode {
        let mut words = Vec::new();
        let mut cursor = start;
        while cursor + 1 < link_info_end {
            let word = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
            if word == 0 {
                break;
            }
            words.push(word);
            cursor += 2;
        }
        String::from_utf16(&words).ok()
    } else {
        let end = bytes[start..link_info_end]
            .iter()
            .position(|byte| *byte == 0)
            .map(|relative| start + relative)
            .unwrap_or(link_info_end);
        Some(String::from_utf8_lossy(&bytes[start..end]).to_string())
    }
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn join_shortcut_base_and_suffix(base: String, suffix: String) -> String {
    if suffix.is_empty()
        || base.ends_with(&suffix)
        || Path::new(&base)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(&suffix))
    {
        return base;
    }
    Path::new(&base).join(suffix).to_string_lossy().to_string()
}

#[cfg(target_os = "windows")]
fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
    let data = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([data[0], data[1]]))
}

#[cfg(target_os = "windows")]
fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let data = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

fn normalize_path_text_for_platform(path: &str, windows: bool) -> String {
    if windows {
        path.replace('/', "\\")
    } else {
        path.to_string()
    }
}

fn setting_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn normalize_integration_module_language(language_code: Option<&str>) -> Option<String> {
    language_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn parse_string_list_setting(value: &str) -> Vec<String> {
    let Ok(items) = serde_json::from_str::<Vec<String>>(value.trim()) else {
        return Vec::new();
    };
    let mut parsed = Vec::new();
    for item in items {
        let item = item.trim();
        if item.is_empty() || parsed.iter().any(|existing| existing == item) {
            continue;
        }
        parsed.push(item.to_string());
    }
    parsed
}

fn parse_update_check_interval_minutes(value: &str) -> u64 {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|minutes| *minutes > 0)
        .unwrap_or_else(netstitch_shared::models::default_update_check_interval_minutes)
        .clamp(1, 24 * 60)
}

fn env_truthy(key: &str) -> bool {
    env::var(key)
        .map(|value| setting_truthy(&value))
        .unwrap_or(false)
}

fn ensure_tracked_apps_columns(conn: &Connection) -> Result<()> {
    if !table_has_column(conn, "tracked_apps", "connector_id")? {
        conn.execute("ALTER TABLE tracked_apps ADD COLUMN connector_id TEXT", [])?;
    }
    if !table_has_column(conn, "tracked_apps", "cloud_app_id")? {
        conn.execute("ALTER TABLE tracked_apps ADD COLUMN cloud_app_id TEXT", [])?;
    }
    if !table_has_column(conn, "tracked_apps", "display_name")? {
        conn.execute("ALTER TABLE tracked_apps ADD COLUMN display_name TEXT", [])?;
    }
    if !table_has_column(conn, "tracked_apps", "icon_key")? {
        conn.execute("ALTER TABLE tracked_apps ADD COLUMN icon_key TEXT", [])?;
    }
    if !table_has_column(conn, "tracked_apps", "icon_path")? {
        conn.execute("ALTER TABLE tracked_apps ADD COLUMN icon_path TEXT", [])?;
    }
    if !table_has_column(conn, "tracked_apps", "archived")? {
        conn.execute(
            "ALTER TABLE tracked_apps ADD COLUMN archived INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    ensure_tracked_apps_exe_path_not_unique(conn)?;
    Ok(())
}

fn ensure_tracked_apps_exe_path_not_unique(conn: &Connection) -> Result<()> {
    let create_sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'tracked_apps'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .context("failed to inspect tracked_apps schema")?
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !create_sql.contains("exe_path text not null unique") {
        return Ok(());
    }

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;
        ALTER TABLE tracked_apps RENAME TO tracked_apps_rebuild_source;
        CREATE TABLE tracked_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            exe_path TEXT NOT NULL,
            connector_id TEXT,
            cloud_app_id TEXT,
            process_name TEXT,
            display_name TEXT,
            icon_key TEXT,
            icon_path TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL
        );
        INSERT INTO tracked_apps (
            id,
            exe_path,
            connector_id,
            cloud_app_id,
            process_name,
            display_name,
            icon_key,
            icon_path,
            enabled,
            archived,
            created_at_ms
        )
        SELECT
            id,
            exe_path,
            connector_id,
            cloud_app_id,
            process_name,
            display_name,
            icon_key,
            icon_path,
            enabled,
            archived,
            created_at_ms
        FROM tracked_apps_rebuild_source;
        DROP TABLE tracked_apps_rebuild_source;
        PRAGMA foreign_keys = ON;
        "#,
    )
    .context("failed to rebuild tracked_apps without exe_path uniqueness")
}

fn ensure_observed_endpoints_columns(conn: &Connection) -> Result<()> {
    if !table_has_column(conn, "observed_endpoints", "cloud_app_id")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN cloud_app_id TEXT",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "app_signature_key")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN app_signature_key TEXT",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "app_signature_subject")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN app_signature_subject TEXT",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "app_signature_issuer")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN app_signature_issuer TEXT",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "app_signature_source")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN app_signature_source TEXT",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "connection_state")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN connection_state TEXT NOT NULL DEFAULT 'unknown'",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "failed_hits")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN failed_hits INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !table_has_column(conn, "observed_endpoints", "successful_hits")? {
        conn.execute(
            "ALTER TABLE observed_endpoints ADD COLUMN successful_hits INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

fn ensure_system_events_columns(conn: &Connection) -> Result<()> {
    if !table_has_column(conn, "system_events", "repeat_count")? {
        conn.execute(
            "ALTER TABLE system_events ADD COLUMN repeat_count INTEGER NOT NULL DEFAULT 1",
            [],
        )?;
    }
    if !table_has_column(conn, "system_events", "source")? {
        conn.execute(
            "ALTER TABLE system_events ADD COLUMN source TEXT NOT NULL DEFAULT 'core'",
            [],
        )?;
    }
    let create_sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'system_events'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .context("failed to inspect system_events schema")?
        .unwrap_or_default()
        .to_ascii_lowercase();
    if create_sql.contains("check") && !create_sql.contains("'success'") {
        rebuild_system_events_with_success_severity(conn)?;
    }
    Ok(())
}

fn rebuild_system_events_with_success_severity(conn: &Connection) -> Result<()> {
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
    .context("failed to rebuild system_events severity schema")
}

fn ensure_observed_endpoints_without_tracked_app_fk(conn: &Connection) -> Result<()> {
    let create_sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'observed_endpoints'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .context("failed to inspect observed_endpoints schema")?;
    let Some(create_sql) = create_sql else {
        return Ok(());
    };
    if !create_sql.to_ascii_uppercase().contains("REFERENCES") {
        return Ok(());
    }

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;
        ALTER TABLE observed_endpoints RENAME TO observed_endpoints_rebuild_source;
        CREATE TABLE observed_endpoints (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tracked_app_id INTEGER NOT NULL,
            cloud_app_id TEXT,
            app_signature_key TEXT,
            app_signature_subject TEXT,
            app_signature_issuer TEXT,
            app_signature_source TEXT,
            process_id INTEGER,
            process_name TEXT,
            remote_ip TEXT NOT NULL,
            remote_port INTEGER NOT NULL,
            protocol TEXT NOT NULL,
            first_seen_ms INTEGER NOT NULL,
            last_seen_ms INTEGER NOT NULL,
            hits INTEGER NOT NULL DEFAULT 1,
            connection_state TEXT NOT NULL DEFAULT 'unknown',
            failed_hits INTEGER NOT NULL DEFAULT 0,
            successful_hits INTEGER NOT NULL DEFAULT 0,
            is_confirmed INTEGER NOT NULL DEFAULT 0,
            is_exported INTEGER NOT NULL DEFAULT 0,
            UNIQUE(tracked_app_id, remote_ip, remote_port, protocol)
        );
        INSERT INTO observed_endpoints (
            id,
            tracked_app_id,
            cloud_app_id,
            app_signature_key,
            app_signature_subject,
            app_signature_issuer,
            app_signature_source,
            process_id,
            process_name,
            remote_ip,
            remote_port,
            protocol,
            first_seen_ms,
            last_seen_ms,
            hits,
            connection_state,
            failed_hits,
            successful_hits,
            is_confirmed,
            is_exported
        )
        SELECT
            id,
            tracked_app_id,
            cloud_app_id,
            app_signature_key,
            app_signature_subject,
            app_signature_issuer,
            app_signature_source,
            process_id,
            process_name,
            remote_ip,
            remote_port,
            protocol,
            first_seen_ms,
            last_seen_ms,
            hits,
            connection_state,
            failed_hits,
            successful_hits,
            is_confirmed,
            is_exported
        FROM observed_endpoints_rebuild_source;
        DROP TABLE observed_endpoints_rebuild_source;
        PRAGMA foreign_keys = ON;
        "#,
    )
    .context("failed to rebuild observed_endpoints without tracked app foreign key")
}

fn migrate_csv_import_tracked_apps_to_imported_rows(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        UPDATE observed_endpoints
        SET process_name = COALESCE(
            NULLIF(TRIM(process_name), ''),
            (
                SELECT COALESCE(NULLIF(TRIM(display_name), ''), NULLIF(TRIM(process_name), ''), 'Imported app')
                FROM tracked_apps
                WHERE tracked_apps.id = observed_endpoints.tracked_app_id
            )
        )
        WHERE tracked_app_id IN (
            SELECT id
            FROM tracked_apps
            WHERE LOWER(exe_path) LIKE 'netstitch-csv-import://%'
        )
        "#,
        [],
    )
    .context("failed to preserve imported observation app names before CSV placeholder cleanup")?;
    conn.execute(
        "DELETE FROM tracked_apps WHERE LOWER(exe_path) LIKE 'netstitch-csv-import://%'",
        [],
    )
    .context("failed to remove obsolete CSV import tracked apps")?;
    Ok(())
}

fn remove_orphaned_connector_tracked_apps(
    conn: &Connection,
    active_connector_ids: &BTreeSet<String>,
) -> Result<usize> {
    ensure_tracked_apps_columns(conn)?;
    let orphaned_rows = {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id,
                connector_id,
                COALESCE(NULLIF(TRIM(display_name), ''), NULLIF(TRIM(process_name), ''), 'Imported app')
            FROM tracked_apps
            WHERE connector_id IS NOT NULL
              AND NULLIF(TRIM(connector_id), '') IS NOT NULL
            ORDER BY id ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut items = Vec::new();
        for row in rows {
            let (id, connector_id, display_name) = row?;
            if !active_connector_ids.contains(connector_id.as_str()) {
                items.push((id, display_name));
            }
        }
        items
    };

    for (tracked_app_id, display_name) in &orphaned_rows {
        conn.execute(
            r#"
            UPDATE observed_endpoints
            SET tracked_app_id = 0,
                process_name = COALESCE(NULLIF(TRIM(process_name), ''), ?2)
            WHERE tracked_app_id = ?1
            "#,
            params![tracked_app_id, display_name],
        )
        .context("failed to detach observations from orphaned connector tracked app")?;
        conn.execute(
            "DELETE FROM tracked_apps WHERE id = ?1",
            params![tracked_app_id],
        )
        .context("failed to delete orphaned connector tracked app")?;
    }

    Ok(orphaned_rows.len())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let query = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let existing: String = row.get(1)?;
        if existing.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn seed_connector_tracked_apps(
    conn: &Connection,
    detected_apps: &[DetectedApp],
    icon_cache_dir: &Path,
    base_created_at_ms: u64,
) -> Result<usize> {
    ensure_tracked_apps_columns(conn)?;
    let mut inserted = 0;
    let mut grouped = BTreeMap::<&str, Vec<&DetectedApp>>::new();
    for app in detected_apps {
        grouped.entry(app.connector_id).or_default().push(app);
    }

    for (index, apps) in grouped.into_values().enumerate() {
        let Some(app) = preferred_detected_app(&apps) else {
            continue;
        };
        let exe_path = normalize_path_buf(&app.exe_path)
            .to_string_lossy()
            .to_string();
        let icon_path = cache_connector_icon(app.icon_svg, icon_cache_dir, app.icon_key)
            .map(|path| path.to_string_lossy().to_string());
        let cloud_app_id =
            netstitch_shared::cloud_app_for_connector_id(app.connector_id).map(|item| item.app_id);
        let existing_rows =
            connector_tracked_app_rows(conn, app.connector_id, app.icon_key, &exe_path)?;

        if !existing_rows.is_empty() {
            let replacement_enabled = existing_rows.iter().any(|row| row.enabled);
            let replacement_id = replacement_connector_row_id(&existing_rows, &exe_path);
            for row in existing_rows.iter().filter(|row| row.id != replacement_id) {
                conn.execute(
                    "DELETE FROM observed_endpoints WHERE tracked_app_id = ?1",
                    params![row.id],
                )?;
                conn.execute("DELETE FROM tracked_apps WHERE id = ?1", params![row.id])?;
            }

            conn.execute(
                r#"
                UPDATE tracked_apps
                SET exe_path = ?2,
                    connector_id = ?3,
                    cloud_app_id = COALESCE(cloud_app_id, ?9),
                    process_name = ?4,
                    display_name = ?5,
                    icon_key = ?6,
                    icon_path = COALESCE(?7, icon_path),
                    enabled = ?8,
                    archived = 0
                WHERE id = ?1
                "#,
                params![
                    replacement_id,
                    &exe_path,
                    app.connector_id,
                    app.process_name,
                    app.display_name,
                    app.icon_key,
                    icon_path,
                    replacement_enabled,
                    cloud_app_id
                ],
            )?;
            continue;
        }

        conn.execute(
            r#"
            INSERT INTO tracked_apps (exe_path, connector_id, cloud_app_id, enabled, process_name, display_name, icon_key, icon_path, created_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                &exe_path,
                app.connector_id,
                cloud_app_id,
                false,
                app.process_name,
                app.display_name,
                app.icon_key,
                icon_path,
                (base_created_at_ms + index as u64) as i64
            ],
        )?;
        inserted += 1;
    }

    Ok(inserted)
}

#[derive(Clone, Debug)]
struct ConnectorTrackedAppRow {
    id: i64,
    exe_path: String,
    enabled: bool,
    created_at_ms: i64,
}

fn connector_tracked_app_rows(
    conn: &Connection,
    connector_id: &str,
    icon_key: &str,
    exe_path: &str,
) -> Result<Vec<ConnectorTrackedAppRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, exe_path, enabled, created_at_ms
        FROM tracked_apps
        WHERE connector_id = ?1
           OR (connector_id IS NULL AND exe_path = ?3)
           OR (connector_id IS NULL AND icon_key = ?2)
        ORDER BY id ASC
        "#,
    )?;
    let rows = stmt.query_map(params![connector_id, icon_key, exe_path], |row| {
        Ok(ConnectorTrackedAppRow {
            id: row.get(0)?,
            exe_path: row.get(1)?,
            enabled: row.get(2)?,
            created_at_ms: row.get(3)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn replacement_connector_row_id(rows: &[ConnectorTrackedAppRow], exe_path: &str) -> i64 {
    rows.iter()
        .find(|row| path_text_eq(&row.exe_path, exe_path))
        .or_else(|| rows.iter().find(|row| row.enabled))
        .or_else(|| rows.iter().max_by_key(|row| (row.created_at_ms, row.id)))
        .map(|row| row.id)
        .expect("replacement row requires non-empty rows")
}

fn preferred_detected_app<'a>(apps: &[&'a DetectedApp]) -> Option<&'a DetectedApp> {
    apps.iter()
        .copied()
        .max_by(|left, right| compare_detected_app_preference(left, right))
}

fn compare_detected_app_preference(left: &DetectedApp, right: &DetectedApp) -> Ordering {
    compare_version_tokens(
        &path_version_tokens(&left.exe_path),
        &path_version_tokens(&right.exe_path),
    )
    .then_with(|| {
        normalize_path_key_for_compare(&left.exe_path)
            .cmp(&normalize_path_key_for_compare(&right.exe_path))
    })
}

fn path_version_tokens(path: &Path) -> Option<Vec<u64>> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .filter_map(version_tokens_from_segment)
        .max_by(|left, right| compare_version_tokens(&Some(left.clone()), &Some(right.clone())))
}

fn version_tokens_from_segment(segment: &str) -> Option<Vec<u64>> {
    let lowered = segment.to_ascii_lowercase();
    for candidate in lowered
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '.' || character == '-')
        })
        .filter(|candidate| candidate.contains('.'))
    {
        let version = candidate
            .trim_start_matches("app-")
            .trim_start_matches('v')
            .split('.')
            .map(|part| {
                part.chars()
                    .take_while(|character| character.is_ascii_digit())
                    .collect::<String>()
            })
            .filter(|part| !part.is_empty())
            .map(|part| part.parse::<u64>().ok())
            .collect::<Option<Vec<_>>>();
        if version.as_ref().is_some_and(|items| items.len() >= 2) {
            return version;
        }
    }
    None
}

fn compare_version_tokens(left: &Option<Vec<u64>>, right: &Option<Vec<u64>>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => {
            let max_len = left.len().max(right.len());
            for index in 0..max_len {
                let ordering = left
                    .get(index)
                    .copied()
                    .unwrap_or_default()
                    .cmp(&right.get(index).copied().unwrap_or_default());
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            Ordering::Equal
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn normalize_path_key_for_compare(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if cfg!(windows) {
        raw.replace('/', "\\").to_ascii_lowercase()
    } else {
        raw.to_string()
    }
}

fn path_text_eq(left: &str, right: &str) -> bool {
    if cfg!(windows) {
        left.replace('/', "\\")
            .eq_ignore_ascii_case(&right.replace('/', "\\"))
    } else {
        left == right
    }
}

fn cache_connector_icon(icon_svg: &[u8], icon_cache_dir: &Path, icon_key: &str) -> Option<PathBuf> {
    fs::create_dir_all(icon_cache_dir).ok()?;

    let safe_icon_key = safe_icon_key(icon_key);
    for extension in ["svg", "png"] {
        let existing = icon_cache_dir.join(format!("{safe_icon_key}.{extension}"));
        if existing.is_file() {
            return Some(existing);
        }
    }

    if icon_svg.is_empty() {
        return None;
    }

    let target = icon_cache_dir.join(format!("{safe_icon_key}.svg"));
    if target.is_file() {
        return Some(target);
    }
    fs::write(&target, icon_svg).ok()?;
    Some(target)
}

fn cache_default_icon(icon_cache_dir: &Path) -> Option<PathBuf> {
    cache_connector_icon(
        netstitch_connectors::default_icon_svg(),
        icon_cache_dir,
        "default",
    )
}

fn cache_manual_icon(exe_path: &Path, icon_cache_dir: &Path) -> Option<PathBuf> {
    cache_executable_icon(exe_path, icon_cache_dir).or_else(|| cache_default_icon(icon_cache_dir))
}

fn tracked_app_icon_path_is_usable(icon_path: Option<&str>) -> bool {
    let Some(icon_path) = icon_path.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    let path = Path::new(icon_path);
    if !path.is_file() {
        return false;
    }
    if path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("default.svg"))
    {
        return false;
    }
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    {
        return cached_manual_png_is_usable(path);
    }
    true
}

#[cfg(target_os = "windows")]
fn cache_executable_icon(exe_path: &Path, icon_cache_dir: &Path) -> Option<PathBuf> {
    if !exe_path.is_file() {
        return None;
    }

    fs::create_dir_all(icon_cache_dir).ok()?;
    let target = icon_cache_dir.join(manual_icon_cache_filename(exe_path));
    if target.is_file() && cached_manual_png_is_usable(&target) {
        return Some(target);
    }
    if target.is_file() {
        fs::remove_file(&target).ok();
    }

    let image = extract_windows_executable_icon(exe_path).ok()?;
    image.save(&target).ok()?;
    Some(target)
}

#[cfg(not(target_os = "windows"))]
fn cache_executable_icon(_exe_path: &Path, _icon_cache_dir: &Path) -> Option<PathBuf> {
    None
}

#[cfg(any(target_os = "windows", test))]
fn manual_icon_cache_filename(exe_path: &Path) -> String {
    format!("manual_{:016x}.png", stable_path_hash(exe_path))
}

#[cfg(any(target_os = "windows", test))]
fn stable_path_hash(path: &Path) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    normalized_icon_cache_key(path)
        .as_bytes()
        .iter()
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        })
}

#[cfg(any(target_os = "windows", test))]
fn normalized_icon_cache_key(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if cfg!(windows) {
        raw.replace('/', "\\").to_ascii_lowercase()
    } else {
        raw.to_string()
    }
}

fn cached_manual_png_is_usable(path: &Path) -> bool {
    let Ok(image) = image::open(path).map(|image| image.into_rgba8()) else {
        return false;
    };
    image.width() > 0 && image.height() > 0 && image.pixels().any(|pixel| pixel[3] > 0)
}

#[cfg(target_os = "windows")]
fn extract_windows_executable_icon(exe_path: &Path) -> Result<image::RgbaImage> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC,
        DeleteObject, HGDIOBJ, SelectObject,
    };
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
    use windows::Win32::UI::WindowsAndMessaging::{
        DI_NORMAL, DestroyIcon, DrawIconEx, GetSystemMetrics, SM_CXICON, SM_CYICON,
    };
    use windows::core::PCWSTR;

    let mut wide = exe_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut shell_info = SHFILEINFOW::default();

    let icon_result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_mut_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shell_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if icon_result == 0 || shell_info.hIcon.is_invalid() {
        return Err(anyhow!("Windows shell did not return an icon"));
    }

    let result = (|| {
        let width = unsafe { GetSystemMetrics(SM_CXICON) }.max(64);
        let height = unsafe { GetSystemMetrics(SM_CYICON) }.max(64);
        if width <= 0 || height <= 0 {
            return Err(anyhow!("Windows returned invalid icon metrics"));
        }

        let mut bitmap_info = BITMAPINFO::default();
        bitmap_info.bmiHeader.biSize = std::mem::size_of_val(&bitmap_info.bmiHeader) as u32;
        bitmap_info.bmiHeader.biWidth = width;
        bitmap_info.bmiHeader.biHeight = -height;
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32;
        bitmap_info.bmiHeader.biCompression = BI_RGB.0;

        let byte_len = width as usize * height as usize * 4;
        let memory_dc = unsafe { CreateCompatibleDC(None) };
        if memory_dc.is_invalid() {
            return Err(anyhow!("failed to create icon memory device context"));
        }

        let mut bits = std::ptr::null_mut::<core::ffi::c_void>();
        let bitmap = unsafe {
            CreateDIBSection(
                Some(memory_dc),
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )
        };
        let bitmap = match bitmap {
            Ok(bitmap) => bitmap,
            Err(error) => {
                unsafe {
                    let _ = DeleteDC(memory_dc);
                }
                return Err(error).context("failed to allocate icon DIB section");
            }
        };
        if bitmap.is_invalid() || bits.is_null() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
            }
            return Err(anyhow!("icon DIB section did not return a pixel buffer"));
        }

        let old_object = unsafe { SelectObject(memory_dc, HGDIOBJ(bitmap.0)) };
        let draw_result = unsafe {
            DrawIconEx(
                memory_dc,
                0,
                0,
                shell_info.hIcon,
                width,
                height,
                0,
                None,
                DI_NORMAL,
            )
        };
        if let Err(error) = draw_result {
            unsafe {
                if !old_object.is_invalid() {
                    SelectObject(memory_dc, old_object);
                }
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
            }
            return Err(error).context("failed to draw executable icon");
        }

        let bgra = unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), byte_len) };
        let has_alpha = bgra.chunks_exact(4).any(|pixel| pixel[3] != 0);
        let mut rgba = Vec::with_capacity(byte_len);
        for pixel in bgra.chunks_exact(4) {
            rgba.push(pixel[2]);
            rgba.push(pixel[1]);
            rgba.push(pixel[0]);
            rgba.push(if has_alpha {
                pixel[3]
            } else if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                255
            } else {
                0
            });
        }

        unsafe {
            if !old_object.is_invalid() {
                SelectObject(memory_dc, old_object);
            }
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
        }

        let image = image::RgbaImage::from_vec(width as u32, height as u32, rgba)
            .ok_or_else(|| anyhow!("failed to build RGBA image"))?;
        if !image.pixels().any(|pixel| pixel[3] > 0) {
            return Err(anyhow!("extracted executable icon is fully transparent"));
        }
        Ok(image)
    })();

    unsafe {
        DestroyIcon(shell_info.hIcon).ok();
    }

    result
}

fn safe_icon_key(icon_key: &str) -> String {
    let sanitized = icon_key
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() || item == '-' || item == '_' {
                item
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "manual".to_string()
    } else {
        sanitized
    }
}

fn derive_process_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|part| part.to_str())
        .map(ToOwned::to_owned)
}

fn display_name_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|part| part.to_str())
        .map(|value| value.trim().trim_end_matches(".exe").to_string())
        .filter(|value| !value.is_empty())
}

fn display_name_for_manual_app(path: &Path) -> Option<String> {
    netstitch_connectors::app_file_metadata(path)
        .product_name
        .filter(|value| !value.trim().is_empty())
        .or_else(|| display_name_from_path(path))
}

fn runtime_status_for_tracked_apps(apps: &[TrackedApp]) -> RuntimeStatusDto {
    let mut status = RuntimeStatusDto::default();
    for app in apps {
        if is_csv_import_placeholder_path(&app.exe_path) {
            continue;
        }
        let connector_managed =
            app.icon_key.as_deref().unwrap_or(MANUAL_ICON_KEY) != MANUAL_ICON_KEY;
        let exe_exists = path_is_file_cached(&app.exe_path);
        if connector_managed && exe_exists {
            status.connector_apps_detected += 1;
        }
        if !exe_exists {
            status
                .unavailable_tracked_apps
                .push(TrackedAppAvailabilityDto {
                    tracked_app_id: app.id,
                    display_name: app
                        .display_name
                        .clone()
                        .or_else(|| app.process_name.clone())
                        .or_else(|| display_name_from_path(&app.exe_path)),
                    exe_path: app.exe_path.clone(),
                    connector_managed,
                });
        }
    }
    status
}

fn normalize_background_subscriptions(subscriptions: Vec<String>) -> BTreeSet<String> {
    let mut normalized = subscriptions
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if normalized.is_empty() {
        normalized.insert("*".to_string());
    }
    normalized
}

fn background_task_matches(task: &IntegrationBackgroundTask, event_type: &str) -> bool {
    let event_type = event_type.trim().to_ascii_lowercase();
    task.subscriptions.iter().any(|subscription| {
        subscription == "*"
            || subscription == &event_type
            || subscription
                .strip_suffix(".*")
                .is_some_and(|prefix| event_type.starts_with(&format!("{prefix}.")))
    })
}

fn is_csv_import_placeholder_path(path: &Path) -> bool {
    path.to_string_lossy()
        .to_ascii_lowercase()
        .starts_with("netstitch-csv-import://")
}

fn path_is_file_cached(path: &Path) -> bool {
    if let Ok(cache) = PATH_IS_FILE_CACHE.lock() {
        if let Some(cached) = cache.get(path) {
            return *cached;
        }
    }

    let exists = path.is_file();
    if let Ok(mut cache) = PATH_IS_FILE_CACHE.lock() {
        cache.insert(path.to_path_buf(), exists);
    }
    exists
}

fn capability_report() -> CapabilityReport {
    CapabilityReport {
        monitoring_supported: cfg!(any(target_os = "windows", target_os = "linux")),
        export_supported: true,
        external_integrations_supported: true,
        requires_admin: cfg!(target_os = "windows"),
        ipc_supported: true,
    }
}

fn export_confirmed_endpoints(
    target_path: &Path,
    endpoints: &[ObservedEndpoint],
) -> Result<ExportResultDto> {
    if target_path.as_os_str().is_empty() {
        return Err(anyhow!("export target path is empty"));
    }
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }

    let mut unique = BTreeSet::new();
    let mut skipped_unconfirmed = 0usize;
    for endpoint in endpoints {
        if endpoint.is_confirmed {
            unique.insert(endpoint.remote_ip);
        } else {
            skipped_unconfirmed += 1;
        }
    }
    let exported_ips = unique.into_iter().collect::<Vec<_>>();
    let confirmed_count = endpoints
        .iter()
        .filter(|endpoint| endpoint.is_confirmed)
        .count();
    let skipped_duplicates = confirmed_count.saturating_sub(exported_ips.len());
    let mut contents = exported_ips
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    if !contents.is_empty() {
        contents.push('\n');
    }
    fs::write(target_path, contents)
        .with_context(|| format!("failed to write {}", target_path.display()))?;

    Ok(ExportResultDto::new(
        target_path.to_path_buf(),
        exported_ips,
        skipped_unconfirmed,
        skipped_duplicates,
        current_timestamp_ms(),
    ))
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{
        TRACKED_APPS_LIST_QUERY, TrackedApp, cache_connector_icon, cache_manual_icon,
        cached_manual_png_is_usable, manual_icon_cache_filename, normalize_path_text_for_platform,
        path_text_eq, remove_orphaned_connector_tracked_apps, runtime_status_for_tracked_apps,
        seed_connector_tracked_apps,
    };
    use netstitch_connectors::DetectedApp;
    use netstitch_shared::ipc::SetTrackedAppEnabledRequest;
    use netstitch_shared::models::{
        ConnectionState, MonitorStatus, MonitoringCsvImportRequestDto, MonitoringCsvImportRowDto,
        MonitoringImportSourceDto, Protocol, SystemEventRequestDto,
    };
    use rusqlite::{Connection, params};
    use std::collections::BTreeSet;
    use std::fs;
    use std::net::IpAddr;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn path_normalization_keeps_linux_absolute_paths_intact() {
        assert_eq!(
            normalize_path_text_for_platform("/usr/bin/netstitch", false),
            "/usr/bin/netstitch"
        );
        assert_eq!(
            normalize_path_text_for_platform("C:/Tools/NetStitch.exe", true),
            r"C:\Tools\NetStitch.exe"
        );
    }

    #[test]
    fn path_text_equality_is_platform_aware() {
        if cfg!(windows) {
            assert!(path_text_eq(
                r"C:\Tools\NetStitch.exe",
                "c:/tools/netstitch.exe"
            ));
        } else {
            assert!(!path_text_eq("/usr/bin/netstitch", "/USR/bin/netstitch"));
        }
    }

    #[test]
    fn seed_connector_tracked_apps_inserts_detected_apps_disabled() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        let apps = vec![
            detected_app(
                "discord",
                "Discord",
                r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe",
                "Discord.exe",
            ),
            detected_app(
                "chrome",
                "Google Chrome",
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                "chrome.exe",
            ),
        ];

        let icon_cache = unique_temp_dir("icons");
        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 1_715_000_000_000)
            .expect("seeding should succeed");
        assert_eq!(inserted, 2);

        let mut stmt = conn
            .prepare("SELECT exe_path, enabled, process_name, display_name, icon_key, icon_path FROM tracked_apps ORDER BY created_at_ms ASC")
            .expect("query should prepare");
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .expect("query should run");
        let values = rows
            .collect::<Result<Vec<_>, _>>()
            .expect("rows should parse");

        assert_eq!(values.len(), 2);
        assert!(values.iter().all(|(_, enabled, _, _, _, _)| !*enabled));
        assert!(values.iter().any(
            |(path, _, process_name, display_name, icon_key, icon_path)| {
                path == r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe"
                    && process_name == "Discord.exe"
                    && display_name == "Discord"
                    && icon_key == "discord"
                    && icon_path.is_some()
            }
        ));
        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn seed_connector_tracked_apps_preserves_existing_rows() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        conn.execute(
            "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![r"C:\Custom\custom.exe", true, "custom.exe", "Custom", "manual", Option::<String>::None, 1_i64],
        )
        .expect("custom row insert should succeed");
        let apps = vec![detected_app(
            "manual",
            "Custom",
            r"C:\Custom\custom.exe",
            "custom.exe",
        )];

        let icon_cache = unique_temp_dir("icons-existing");
        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 2_000)
            .expect("seed should preserve row");
        assert_eq!(inserted, 0);

        let enabled: bool = conn
            .query_row(
                "SELECT enabled FROM tracked_apps WHERE exe_path = ?1",
                params![r"C:\Custom\custom.exe"],
                |row| row.get(0),
            )
            .expect("enabled query should work");
        assert!(enabled);
        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_seed_keeps_only_latest_detected_version_per_connector() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        let apps = vec![
            detected_app(
                "discord",
                "Discord",
                r"C:\Users\Demo\AppData\Local\Discord\app-1.0.0\Discord.exe",
                "Discord.exe",
            ),
            detected_app(
                "discord",
                "Discord",
                r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe",
                "Discord.exe",
            ),
        ];

        let icon_cache = unique_temp_dir("connector-latest-detected");
        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 3_000)
            .expect("seed should deduplicate connector versions");
        assert_eq!(inserted, 1);

        let rows = query_tracked_app_identity_rows(&conn);
        assert_eq!(
            rows,
            vec![(
                r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe".to_string(),
                Some("discord".to_string()),
                false,
                "Discord".to_string(),
                "discord".to_string(),
            )]
        );

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_seed_allows_distinct_connector_ids_for_same_executable() {
        let conn = Connection::open_in_memory().expect("db");
        create_tracked_apps_table(&conn);
        let icon_cache = unique_temp_dir("same-exe-distinct-connectors");
        let apps = vec![
            detected_app(
                "discord",
                "Discord",
                r"C:\Users\me\AppData\Local\Discord\app-1\Discord.exe",
                "Discord.exe",
            ),
            detected_app(
                "discord2",
                "Discord2",
                r"C:\Users\me\AppData\Local\Discord\app-1\Discord.exe",
                "Discord.exe",
            ),
        ];

        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 3_000)
            .expect("seed should allow duplicate executable by connector id");
        let rows = conn
            .prepare(
                "SELECT connector_id, display_name FROM tracked_apps ORDER BY connector_id ASC",
            )
            .expect("select")
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .expect("rows")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("rows should map");

        assert_eq!(inserted, 2);
        assert_eq!(
            rows,
            vec![
                ("discord".to_string(), "Discord".to_string()),
                ("discord2".to_string(), "Discord2".to_string())
            ]
        );

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn runtime_app_manifest_discovery_seeds_tracked_app_disabled() {
        let conn = Connection::open_in_memory().expect("db");
        create_tracked_apps_table(&conn);
        let root = unique_temp_dir("runtime-app-bootstrap");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("runtime app dir");
        let app_path = root.join("Demo.exe");
        fs::write(&app_path, b"").expect("demo executable fixture");
        let app_text = app_path.to_string_lossy();
        fs::write(
            root.join("demo.app"),
            format!(
                r#"
                version = 1
                enabled = true
                id = "demo-runtime"
                display_name = "Demo Runtime"
                icon_key = "demo_runtime"
                process_names = ["Demo.exe"]

                [[discovery]]
                kind = "fixed_path"
                path = "{app_text}"
                "#
            ),
        )
        .expect("runtime app manifest");
        fs::write(
            root.join("demo.toml"),
            r#"
            version = 1
            enabled = true
            id = "ignored-toml"
            display_name = "Ignored Toml"
            process_names = ["Demo.exe"]
            "#,
        )
        .expect("ignored toml manifest");

        let apps = netstitch_connectors::discover_installed_apps_from_dir(&root);
        let icon_cache = unique_temp_dir("runtime-app-bootstrap-icons");
        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 3_000)
            .expect("runtime app discovery should seed connector row");
        let rows = query_tracked_app_identity_rows(&conn);

        assert_eq!(inserted, 1);
        assert_eq!(
            rows,
            vec![(
                app_path.to_string_lossy().to_string(),
                Some("demo-runtime".to_string()),
                false,
                "Demo Runtime".to_string(),
                "demo_runtime".to_string(),
            )]
        );

        fs::remove_dir_all(root).ok();
        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_seed_replaces_stale_connector_versions_preserving_enabled_state() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        for (path, enabled, created_at_ms) in [
            (
                r"C:\Users\Demo\AppData\Local\Discord\app-1.0.0\Discord.exe",
                false,
                1_i64,
            ),
            (
                r"C:\Users\Demo\AppData\Local\Discord\app-1.5.0\Discord.exe",
                true,
                2_i64,
            ),
        ] {
            conn.execute(
                "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)",
                params![path, enabled, "Discord.exe", "Discord", "discord", created_at_ms],
            )
            .expect("stale connector row insert");
        }
        conn.execute(
            "INSERT INTO observed_endpoints (tracked_app_id, remote_ip, remote_port, protocol, first_seen_ms, last_seen_ms, hits) VALUES (1, '203.0.113.10', 443, 'tcp', 1, 1, 1)",
            [],
        )
        .expect("stale observation insert");

        let apps = vec![detected_app(
            "discord",
            "Discord",
            r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe",
            "Discord.exe",
        )];

        let icon_cache = unique_temp_dir("connector-replace-stale");
        let inserted = seed_connector_tracked_apps(&conn, &apps, &icon_cache, 3_000)
            .expect("seed should replace stale versions");
        assert_eq!(inserted, 0);

        let rows = query_tracked_app_identity_rows(&conn);
        assert_eq!(
            rows,
            vec![(
                r"C:\Users\Demo\AppData\Local\Discord\app-2.0.0\Discord.exe".to_string(),
                Some("discord".to_string()),
                true,
                "Discord".to_string(),
                "discord".to_string(),
            )]
        );
        let observations: i64 = conn
            .query_row("SELECT COUNT(*) FROM observed_endpoints", [], |row| {
                row.get(0)
            })
            .expect("observation count");
        assert_eq!(observations, 0);

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_seed_restores_archived_detected_app() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        conn.execute(
            "INSERT INTO tracked_apps (exe_path, connector_id, enabled, archived, process_name, display_name, icon_key, icon_path, created_at_ms)
             VALUES (?1, ?2, 0, 1, ?3, ?4, ?5, NULL, 1)",
            params![
                r"D:\Games\Demo\Demo.exe",
                "demo",
                "Demo.exe",
                "Demo",
                "demo"
            ],
        )
        .expect("archived connector row insert");
        let apps = vec![DetectedApp {
            connector_id: "demo",
            display_name: "Demo",
            icon_key: "demo",
            icon_ico: b"ico",
            icon_svg: b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
            exe_path: PathBuf::from(r"D:\Games\Demo\Demo.exe"),
            process_name: "Demo.exe",
        }];
        let icon_cache = unique_temp_dir("connector-restore-archived");

        seed_connector_tracked_apps(&conn, &apps, &icon_cache, 3_000)
            .expect("seed should restore archived detected app");

        let archived: bool = conn
            .query_row(
                "SELECT archived FROM tracked_apps WHERE connector_id = 'demo'",
                [],
                |row| row.get(0),
            )
            .expect("archived flag");
        assert!(!archived);

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_icon_cache_writes_embedded_svg_for_ui() {
        let icon_cache = unique_temp_dir("svg-icons");
        let manifest = netstitch_connectors::modules::all_connectors()
            .into_iter()
            .map(|connector| connector.manifest())
            .find(|manifest| manifest.icon_key == "discord")
            .expect("known connector with embedded svg icon");

        let icon_path = cache_connector_icon(manifest.icon_svg, &icon_cache, manifest.icon_key)
            .expect("embedded icon should be cached");

        assert_eq!(
            icon_path.extension().and_then(|value| value.to_str()),
            Some("svg")
        );
        assert!(icon_path.is_file());
        let content = fs::read_to_string(&icon_path).expect("svg should be readable text");
        assert!(content.contains("<svg"));

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn connector_icon_cache_prefers_user_png_in_apps_icons() {
        let icon_cache = unique_temp_dir("custom-connector-icons");
        fs::create_dir_all(&icon_cache).expect("icon dir");
        let custom_icon = icon_cache.join("my_app.png");
        fs::write(&custom_icon, b"png").expect("custom icon");

        let icon_path = cache_connector_icon(b"<svg></svg>", &icon_cache, "my_app")
            .expect("custom connector icon should resolve");

        assert_eq!(icon_path, custom_icon);
        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn runtime_status_reports_connector_count_and_missing_tracked_apps() {
        let root = unique_temp_dir("runtime-status");
        fs::create_dir_all(&root).expect("temp app dir");
        let existing_connector = root.join("Discord.exe");
        fs::write(&existing_connector, b"stub").expect("test exe should be written");
        let missing_manual = root.join("MissingGame.exe");
        let apps = vec![
            TrackedApp {
                id: Some(1),
                exe_path: existing_connector,
                connector_id: Some("discord".to_string()),
                cloud_app_id: Some("netstitch.app.discord".to_string()),
                process_name: Some("Discord.exe".to_string()),
                display_name: Some("Discord".to_string()),
                icon_key: Some("discord".to_string()),
                icon_path: None,
                enabled: false,
                created_at_ms: 1,
            },
            TrackedApp {
                id: Some(2),
                exe_path: missing_manual.clone(),
                connector_id: None,
                cloud_app_id: None,
                process_name: Some("MissingGame.exe".to_string()),
                display_name: Some("Missing Game".to_string()),
                icon_key: Some("manual".to_string()),
                icon_path: None,
                enabled: true,
                created_at_ms: 2,
            },
        ];

        let status = runtime_status_for_tracked_apps(&apps);

        assert_eq!(status.connector_apps_detected, 1);
        assert_eq!(status.unavailable_tracked_apps.len(), 1);
        assert_eq!(status.unavailable_tracked_apps[0].tracked_app_id, Some(2));
        assert_eq!(
            status.unavailable_tracked_apps[0].display_name.as_deref(),
            Some("Missing Game")
        );
        assert_eq!(status.unavailable_tracked_apps[0].exe_path, missing_manual);
        assert!(!status.unavailable_tracked_apps[0].connector_managed);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn runtime_status_caches_path_existence_for_ui_refreshes() {
        let root = unique_temp_dir("runtime-status-cache");
        fs::create_dir_all(&root).expect("temp app dir");
        let exe = root.join("Discord.exe");
        fs::write(&exe, b"stub").expect("test exe should be written");
        let apps = vec![TrackedApp {
            id: Some(1),
            exe_path: exe.clone(),
            connector_id: Some("discord".to_string()),
            cloud_app_id: Some("netstitch.app.discord".to_string()),
            process_name: Some("Discord.exe".to_string()),
            display_name: Some("Discord".to_string()),
            icon_key: Some("discord".to_string()),
            icon_path: None,
            enabled: false,
            created_at_ms: 1,
        }];

        let first = runtime_status_for_tracked_apps(&apps);
        fs::remove_file(&exe).expect("test exe should be removed");
        let second = runtime_status_for_tracked_apps(&apps);

        assert_eq!(first.connector_apps_detected, 1);
        assert_eq!(
            second.connector_apps_detected, 1,
            "UI refresh should not re-hit filesystem for app availability diagnostics"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn manual_icon_cache_filename_uses_full_path_to_avoid_same_name_collisions() {
        let first = manual_icon_cache_filename(PathBuf::from(r"C:\Games\One\Game.exe").as_path());
        let second = manual_icon_cache_filename(PathBuf::from(r"C:\Games\Two\Game.exe").as_path());

        assert_ne!(first, second);
        assert!(first.starts_with("manual_"));
        assert!(first.ends_with(".png"));
    }

    #[test]
    fn manual_icon_cache_falls_back_to_default_svg_when_extraction_is_unavailable() {
        let icon_cache = unique_temp_dir("manual-icon-fallback");
        let missing_exe = icon_cache.join("missing").join("SameName.exe");

        let icon_path = cache_manual_icon(&missing_exe, &icon_cache)
            .expect("manual icon fallback should be cached");

        assert_eq!(
            icon_path.file_name().and_then(|value| value.to_str()),
            Some("default.svg")
        );
        assert!(icon_path.is_file());

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn fresh_database_does_not_seed_default_manual_apps() {
        let root = unique_temp_dir("fresh-no-manual-seed");
        fs::remove_dir_all(&root).ok();
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let apps = core
            .list_tracked_apps()
            .expect("fresh tracked apps should load");
        assert!(
            apps.iter().all(|app| {
                app.display_name.as_deref() != Some("Demo Game")
                    && app.cloud_app_id.as_deref() != Some("netstitch.app.demo-game")
            }),
            "fresh SQLite databases must not contain default manual app seeds"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn cached_manual_png_rejects_fully_transparent_images() {
        let icon_cache = unique_temp_dir("manual-icon-usability");
        fs::create_dir_all(&icon_cache).expect("icon cache dir");
        let transparent = icon_cache.join("transparent.png");
        let opaque = icon_cache.join("opaque.png");

        image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 0]))
            .save(&transparent)
            .expect("transparent png should save");
        image::RgbaImage::from_pixel(2, 2, image::Rgba([20, 30, 40, 255]))
            .save(&opaque)
            .expect("opaque png should save");

        assert!(!cached_manual_png_is_usable(&transparent));
        assert!(cached_manual_png_is_usable(&opaque));

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn tracked_apps_order_keeps_manual_newest_first_then_connectors_alphabetical() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        for (path, display_name, icon_key, created_at_ms) in [
            (r"C:\Apps\Discord.exe", "Discord", "discord", 500_i64),
            (r"C:\Apps\DemoGame.exe", "Demo Game", "manual", 100_i64),
            (r"C:\Apps\Chrome.exe", "Google Chrome", "chrome", 900_i64),
            (r"C:\Apps\Custom.exe", "Custom", "manual", 700_i64),
        ] {
            conn.execute(
                "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, 1, ?2, ?3, ?4, NULL, ?5)",
                params![path, path.rsplit('\\').next().unwrap_or(path), display_name, icon_key, created_at_ms],
            )
            .expect("row insert");
        }

        let mut stmt = conn
            .prepare(TRACKED_APPS_LIST_QUERY)
            .expect("list query should prepare");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(5))
            .expect("list query should run")
            .collect::<Result<Vec<_>, _>>()
            .expect("rows should parse");

        assert_eq!(
            rows,
            vec![
                "Custom".to_string(),
                "Demo Game".to_string(),
                "Discord".to_string(),
                "Google Chrome".to_string()
            ]
        );
    }

    #[test]
    fn connector_seed_repairs_manualized_existing_connector_identity() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        let icon_cache = unique_temp_dir("connector-repair-icon");
        let path = PathBuf::from(r"C:\WindowsApps\WhatsApp.Root.exe");
        conn.execute(
            "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, 1, ?2, ?3, ?4, NULL, ?5)",
            params![
                path.to_string_lossy(),
                "WhatsApp.Root.exe",
                "WhatsApp.Root",
                "manual",
                10_i64
            ],
        )
        .expect("manualized connector row insert");

        let apps = vec![detected_app(
            "whatsapp",
            "WhatsApp",
            r"C:\WindowsApps\WhatsApp.Root.exe",
            "WhatsApp.exe",
        )];
        let inserted =
            seed_connector_tracked_apps(&conn, &apps, &icon_cache, 20).expect("seed should repair");
        let row = conn
            .query_row(
                "SELECT enabled, process_name, display_name, icon_key FROM tracked_apps WHERE exe_path = ?1",
                params![path.to_string_lossy()],
                |row| {
                    Ok((
                        row.get::<_, bool>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .expect("row should exist");

        assert_eq!(inserted, 0);
        assert!(row.0, "repair must preserve enabled state");
        assert_eq!(row.1, "WhatsApp.exe");
        assert_eq!(row.2, "WhatsApp");
        assert_eq!(row.3, "whatsapp");

        fs::remove_dir_all(icon_cache).ok();
    }

    #[test]
    fn orphaned_connector_tracked_apps_are_deleted_without_touching_manual_apps() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        conn.execute(
            "INSERT INTO tracked_apps (id, exe_path, connector_id, enabled, process_name, display_name, icon_key, created_at_ms) VALUES (1, ?1, 'active', 1, 'Active.exe', 'Active', 'active', 10)",
            params![r"C:\Apps\Active\Active.exe"],
        )
        .expect("active connector row should insert");
        conn.execute(
            "INSERT INTO tracked_apps (id, exe_path, connector_id, enabled, process_name, display_name, icon_key, created_at_ms) VALUES (2, ?1, 'missing', 1, 'Missing.exe', 'Missing Connector', 'missing', 11)",
            params![r"C:\Apps\Missing\Missing.exe"],
        )
        .expect("orphan connector row should insert");
        conn.execute(
            "INSERT INTO tracked_apps (id, exe_path, connector_id, enabled, process_name, display_name, icon_key, created_at_ms) VALUES (3, ?1, NULL, 1, 'Manual.exe', 'Manual App', 'manual', 12)",
            params![r"C:\Apps\Manual\Manual.exe"],
        )
        .expect("manual row should insert");
        conn.execute(
            "INSERT INTO observed_endpoints (tracked_app_id, process_name, remote_ip, remote_port, protocol, first_seen_ms, last_seen_ms, hits) VALUES (2, NULL, '203.0.113.10', 443, 'TCP', 1, 1, 1)",
            [],
        )
        .expect("orphan observation should insert");
        conn.execute(
            "INSERT INTO observed_endpoints (tracked_app_id, process_name, remote_ip, remote_port, protocol, first_seen_ms, last_seen_ms, hits) VALUES (3, 'Manual.exe', '203.0.113.11', 443, 'TCP', 1, 1, 1)",
            [],
        )
        .expect("manual observation should insert");

        let removed =
            remove_orphaned_connector_tracked_apps(&conn, &BTreeSet::from(["active".to_string()]))
                .expect("orphan cleanup should succeed");

        assert_eq!(removed, 1);
        let rows = query_tracked_app_identity_rows(&conn);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| row.1.as_deref() == Some("active")));
        assert!(rows.iter().any(|row| row.1.is_none() && row.4 == "manual"));
        assert!(!rows.iter().any(|row| row.1.as_deref() == Some("missing")));

        let detached: (i64, String) = conn
            .query_row(
                "SELECT tracked_app_id, process_name FROM observed_endpoints WHERE remote_ip = '203.0.113.10'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("detached observation should remain");
        assert_eq!(detached, (0, "Missing Connector".to_string()));
        let manual_tracked_app_id: i64 = conn
            .query_row(
                "SELECT tracked_app_id FROM observed_endpoints WHERE remote_ip = '203.0.113.11'",
                [],
                |row| row.get(0),
            )
            .expect("manual observation should remain attached");
        assert_eq!(manual_tracked_app_id, 3);
    }

    #[test]
    fn set_tracked_app_enabled_preserves_connector_identity_and_order() {
        let root = unique_temp_dir("toggle-connector-identity");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("test root");
        let core = test_core(&root);
        core.initialize_schema()
            .expect("core should initialize test database");
        let conn = core.open_connection().expect("connection");
        let connector_path = root.join("WhatsApp.Root.exe");
        conn.execute(
            "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, 0, ?2, ?3, ?4, NULL, ?5)",
            params![
                connector_path.to_string_lossy(),
                "WhatsApp.Root.exe",
                "WhatsApp",
                "whatsapp",
                1_i64
            ],
        )
        .expect("connector row insert");
        conn.execute(
            "INSERT INTO tracked_apps (exe_path, enabled, process_name, display_name, icon_key, icon_path, created_at_ms) VALUES (?1, 1, ?2, ?3, ?4, NULL, ?5)",
            params![
                root.join("Manual.exe").to_string_lossy(),
                "Manual.exe",
                "Manual",
                "manual",
                2_i64
            ],
        )
        .expect("manual row insert");
        let connector_id = conn
            .query_row(
                "SELECT id FROM tracked_apps WHERE display_name = 'WhatsApp'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|id| id as u64)
            .expect("connector id");

        assert!(
            core.set_tracked_app_enabled(SetTrackedAppEnabledRequest {
                tracked_app_id: connector_id,
                enabled: true,
            })
            .expect("toggle should succeed")
        );
        let rows = core.list_tracked_apps().expect("apps should list");

        assert_eq!(rows[0].display_name.as_deref(), Some("Manual"));
        let connector = rows
            .iter()
            .find(|app| app.id == Some(connector_id))
            .expect("connector row");
        assert!(connector.enabled);
        assert_eq!(connector.display_name.as_deref(), Some("WhatsApp"));
        assert_eq!(connector.icon_key.as_deref(), Some("whatsapp"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn delete_tracked_app_archives_app_and_keeps_observations() {
        let root = unique_temp_dir("delete-tracked-app");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\DeleteMe.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let app_id = app.id.expect("inserted app id");
        core.ingest_observation(super::ObservationRecord {
            tracked_app_id: app_id,
            process_id: Some(10),
            process_name: "DeleteMe.exe".to_string(),
            remote_ip: "203.0.113.10".parse().expect("ip fixture"),
            remote_port: 443,
            protocol: netstitch_shared::models::Protocol::Tcp,
            connection_state: netstitch_shared::models::ConnectionState::Established,
            observed_at_ms: 10_000,
        })
        .expect("observation should insert");

        let deleted = core
            .delete_tracked_app(app_id)
            .expect("tracked app delete should succeed");

        assert_eq!(deleted, 1);
        assert!(
            core.list_tracked_apps()
                .expect("apps should list")
                .is_empty()
        );
        assert!(core.list_endpoints().expect("endpoints should list").len() == 1);

        let restored = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\DeleteMe.exe"),
                enabled: true,
            })
            .expect("archived app should be reactivated by path");
        assert_eq!(restored.id, Some(app_id));
        assert_eq!(
            core.list_tracked_apps()
                .expect("apps should list after restore")
                .len(),
            1
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn delete_observation_removes_only_selected_endpoint() {
        let root = unique_temp_dir("delete-observation");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\Observed.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let app_id = app.id.expect("inserted app id");

        let first = core
            .ingest_observation(super::ObservationRecord {
                tracked_app_id: app_id,
                process_id: Some(10),
                process_name: "Observed.exe".to_string(),
                remote_ip: "203.0.113.10".parse().expect("ip fixture"),
                remote_port: 443,
                protocol: netstitch_shared::models::Protocol::Tcp,
                connection_state: netstitch_shared::models::ConnectionState::Established,
                observed_at_ms: 10_000,
            })
            .expect("first observation should insert");
        core.ingest_observation(super::ObservationRecord {
            tracked_app_id: app_id,
            process_id: Some(10),
            process_name: "Observed.exe".to_string(),
            remote_ip: "203.0.113.11".parse().expect("ip fixture"),
            remote_port: 443,
            protocol: netstitch_shared::models::Protocol::Tcp,
            connection_state: netstitch_shared::models::ConnectionState::Established,
            observed_at_ms: 10_001,
        })
        .expect("second observation should insert");

        let deleted = core
            .delete_observation(netstitch_shared::ipc::DeleteObservationRequest {
                endpoint_id: first.id.expect("endpoint id"),
            })
            .expect("observation delete should succeed");

        let endpoints = core.list_endpoints().expect("endpoints should list");
        assert_eq!(deleted, 1);
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].remote_ip.to_string(), "203.0.113.11");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn delete_observations_removes_selected_endpoints_in_one_batch() {
        let root = unique_temp_dir("delete-observations-batch");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\Observed.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let app_id = app.id.expect("inserted app id");

        let mut endpoint_ids = Vec::new();
        for offset in 0..4 {
            let endpoint = core
                .ingest_observation(super::ObservationRecord {
                    tracked_app_id: app_id,
                    process_id: Some(10),
                    process_name: "Observed.exe".to_string(),
                    remote_ip: format!("203.0.113.{}", 10 + offset)
                        .parse()
                        .expect("ip fixture"),
                    remote_port: 443,
                    protocol: netstitch_shared::models::Protocol::Tcp,
                    connection_state: netstitch_shared::models::ConnectionState::Established,
                    observed_at_ms: 10_000 + offset as u64,
                })
                .expect("observation should insert");
            endpoint_ids.push(endpoint.id.expect("endpoint id"));
        }

        let deleted = core
            .delete_observations(netstitch_shared::ipc::DeleteObservationsRequest {
                endpoint_ids: vec![
                    endpoint_ids[0],
                    endpoint_ids[1],
                    endpoint_ids[1],
                    endpoint_ids[2],
                ],
            })
            .expect("observation batch delete should succeed");

        let endpoints = core.list_endpoints().expect("endpoints should list");
        assert_eq!(deleted, 3);
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].id, Some(endpoint_ids[3]));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn delete_observations_removes_more_than_one_sqlite_parameter_page() {
        let root = unique_temp_dir("delete-observations-large-batch");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\Observed.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let app_id = app.id.expect("inserted app id");

        let mut endpoint_ids = Vec::new();
        for offset in 0..602u16 {
            let endpoint = core
                .ingest_observation(super::ObservationRecord {
                    tracked_app_id: app_id,
                    process_id: Some(10),
                    process_name: "Observed.exe".to_string(),
                    remote_ip: format!("198.51.{}.{}", offset / 255, offset % 255)
                        .parse()
                        .expect("ip fixture"),
                    remote_port: 443,
                    protocol: netstitch_shared::models::Protocol::Tcp,
                    connection_state: netstitch_shared::models::ConnectionState::Established,
                    observed_at_ms: 10_000 + offset as u64,
                })
                .expect("observation should insert");
            endpoint_ids.push(endpoint.id.expect("endpoint id"));
        }

        let deleted = core
            .delete_observations(netstitch_shared::ipc::DeleteObservationsRequest { endpoint_ids })
            .expect("large observation batch delete should succeed");

        let endpoints = core.list_endpoints().expect("endpoints should list");
        assert_eq!(deleted, 602);
        assert!(endpoints.is_empty());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn schema_migration_removes_stale_observed_endpoint_foreign_key() {
        let root = unique_temp_dir("delete-observations-stale-fk");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");

        {
            let conn = Connection::open(&core.paths.database_path).expect("test database");
            conn.execute_batch(
                r#"
                PRAGMA foreign_keys = OFF;
                CREATE TABLE tracked_apps (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    exe_path TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    created_at_ms INTEGER NOT NULL
                );
                INSERT INTO tracked_apps (id, exe_path, enabled, created_at_ms)
                VALUES (1, 'C:\Apps\Observed.exe', 0, 1);
                CREATE TABLE observed_endpoints (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tracked_app_id INTEGER NOT NULL,
                    cloud_app_id TEXT,
                    app_signature_key TEXT,
                    app_signature_subject TEXT,
                    app_signature_issuer TEXT,
                    app_signature_source TEXT,
                    process_id INTEGER,
                    process_name TEXT,
                    remote_ip TEXT NOT NULL,
                    remote_port INTEGER NOT NULL,
                    protocol TEXT NOT NULL,
                    first_seen_ms INTEGER NOT NULL,
                    last_seen_ms INTEGER NOT NULL,
                    hits INTEGER NOT NULL DEFAULT 1,
                    connection_state TEXT NOT NULL DEFAULT 'unknown',
                    failed_hits INTEGER NOT NULL DEFAULT 0,
                    successful_hits INTEGER NOT NULL DEFAULT 0,
                    is_confirmed INTEGER NOT NULL DEFAULT 0,
                    is_exported INTEGER NOT NULL DEFAULT 0,
                    UNIQUE(tracked_app_id, remote_ip, remote_port, protocol),
                    FOREIGN KEY(tracked_app_id) REFERENCES "missing_tracked_apps_source"(id)
                );
                INSERT INTO observed_endpoints (
                    id, tracked_app_id, process_name, remote_ip, remote_port, protocol,
                    first_seen_ms, last_seen_ms, hits, connection_state
                )
                VALUES (1, 1, 'Observed.exe', '203.0.113.10', 443, 'tcp', 1, 1, 1, 'established');
                PRAGMA foreign_keys = ON;
                "#,
            )
            .expect("stale schema fixture");
        }

        core.initialize_schema()
            .expect("schema migration should remove stale fk");
        let create_sql: String = core
            .open_connection()
            .expect("connection")
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'observed_endpoints'",
                [],
                |row| row.get(0),
            )
            .expect("observed schema");
        assert!(
            !create_sql.to_ascii_uppercase().contains("REFERENCES"),
            "observed_endpoints should not keep a stale FK: {create_sql}"
        );

        let deleted = core
            .delete_observations(netstitch_shared::ipc::DeleteObservationsRequest {
                endpoint_ids: vec![1],
            })
            .expect("delete should work after stale fk migration");
        assert_eq!(deleted, 1);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn csv_import_adds_unique_monitoring_rows_without_clearing_existing_data() {
        let root = unique_temp_dir("csv-import");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let first = MonitoringCsvImportRowDto {
            application: "Code".to_string(),
            app_connector_id: Some("chrome".to_string()),
            cloud_app_id: Some("netstitch.app.chrome".to_string()),
            app_signature_key: Some("test-spki".to_string()),
            app_signature_subject: Some("CN=Chrome".to_string()),
            app_signature_issuer: Some("CN=Issuer".to_string()),
            remote_ip: "203.0.113.10".parse().expect("ip fixture"),
            domain: Some("first.example.test".to_string()),
            remote_port: 443,
            protocol: Protocol::Tcp,
            connection_state: ConnectionState::Established,
            first_seen_ms: 10_000,
            last_seen_ms: 10_100,
            hits: 2,
            failed_hits: 0,
            successful_hits: 2,
        };
        let duplicate_from_another_file = MonitoringCsvImportRowDto {
            application: "Code".to_string(),
            app_connector_id: Some("chrome".to_string()),
            cloud_app_id: Some("netstitch.app.chrome".to_string()),
            app_signature_key: Some("test-spki".to_string()),
            app_signature_subject: Some("CN=Chrome".to_string()),
            app_signature_issuer: Some("CN=Issuer".to_string()),
            remote_ip: "203.0.113.10".parse().expect("ip fixture"),
            domain: Some("updated.example.test".to_string()),
            remote_port: 443,
            protocol: Protocol::Tcp,
            connection_state: ConnectionState::Established,
            first_seen_ms: 9_000,
            last_seen_ms: 20_000,
            hits: 5,
            failed_hits: 1,
            successful_hits: 4,
        };
        let second_unique = MonitoringCsvImportRowDto {
            application: "Code".to_string(),
            app_connector_id: Some("chrome".to_string()),
            cloud_app_id: Some("netstitch.app.chrome".to_string()),
            app_signature_key: Some("test-spki".to_string()),
            app_signature_subject: Some("CN=Chrome".to_string()),
            app_signature_issuer: Some("CN=Issuer".to_string()),
            remote_ip: "203.0.113.11".parse().expect("ip fixture"),
            domain: None,
            remote_port: 443,
            protocol: Protocol::Tcp,
            connection_state: ConnectionState::Attempting,
            first_seen_ms: 11_000,
            last_seen_ms: 11_100,
            hits: 1,
            failed_hits: 1,
            successful_hits: 0,
        };

        let first_result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::Csv,
                rows: vec![first],
            })
            .expect("first import should work");
        let second_result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::Csv,
                rows: vec![duplicate_from_another_file, second_unique],
            })
            .expect("second import should append/update");

        assert_eq!(first_result.imported_count, 1);
        assert_eq!(first_result.skipped_count, 0);
        assert_eq!(second_result.imported_count, 1);
        assert_eq!(second_result.skipped_count, 1);

        let endpoints = core.list_endpoints().expect("endpoints should list");
        assert_eq!(
            endpoints.len(),
            2,
            "CSV import uniqueness is tracked app + remote IP + port + protocol, and import must not clear existing rows"
        );
        let merged = endpoints
            .iter()
            .find(|endpoint| endpoint.remote_ip.to_string() == "203.0.113.10")
            .expect("merged endpoint should exist");
        assert_eq!(merged.first_seen_ms, 10_000);
        assert_eq!(merged.last_seen_ms, 10_100);
        assert_eq!(merged.hits, 2);
        assert_eq!(merged.failed_hits, 0);
        assert_eq!(merged.successful_hits, 2);
        let snapshot = core
            .snapshot(MonitorStatus::Stopped, None)
            .expect("snapshot should load");
        let imported = snapshot
            .observed_endpoints
            .iter()
            .find(|endpoint| endpoint.remote_ip.to_string() == "203.0.113.10")
            .expect("imported endpoint should be in snapshot");
        let enrichment = imported
            .enrichment
            .as_ref()
            .expect("CSV domain should be trusted as imported observation data");
        assert_eq!(
            enrichment.domain_name.as_deref(),
            Some("first.example.test")
        );
        assert_eq!(
            enrichment.domain_source.as_deref(),
            Some(netstitch_shared::DOMAIN_SOURCE_CSV_IMPORT)
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn csv_import_without_local_app_does_not_create_tracked_app() {
        let root = unique_temp_dir("csv-import-no-tracked-app");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::Csv,
                rows: vec![MonitoringCsvImportRowDto {
                    application: "Imported Example".to_string(),
                    app_connector_id: None,
                    cloud_app_id: Some("netstitch.app.local.example".to_string()),
                    app_signature_key: Some("appsig_v1_example".to_string()),
                    app_signature_subject: Some("CN=Imported Example".to_string()),
                    app_signature_issuer: None,
                    remote_ip: "203.0.113.44".parse().expect("ip fixture"),
                    domain: Some("imported.example.test".to_string()),
                    remote_port: 443,
                    protocol: Protocol::Tcp,
                    connection_state: ConnectionState::Established,
                    first_seen_ms: 1_000,
                    last_seen_ms: 2_000,
                    hits: 7,
                    failed_hits: 0,
                    successful_hits: 7,
                }],
            })
            .expect("import should keep monitoring data");

        assert_eq!(result.imported_count, 1);
        assert_eq!(
            core.list_tracked_apps()
                .expect("tracked apps should list")
                .len(),
            0,
            "CSV/cloud import must not create tracked apps"
        );
        let endpoints = core.list_endpoints().expect("endpoints should list");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].tracked_app_id, 0);
        assert_eq!(
            endpoints[0].process_name.as_deref(),
            Some("Imported Example")
        );
        assert_eq!(
            endpoints[0].app_signature_key.as_deref(),
            Some("appsig_v1_example")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn csv_import_accepts_more_than_one_cloud_page_without_truncating() {
        let root = unique_temp_dir("csv-import-large-cloud-page");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let rows = (0..602u16)
            .map(|index| MonitoringCsvImportRowDto {
                application: "Cloud Import".to_string(),
                app_connector_id: None,
                cloud_app_id: Some("netstitch.app.cloud-import".to_string()),
                app_signature_key: Some("appsig_v1_cloud_import".to_string()),
                app_signature_subject: Some("CN=Cloud Import".to_string()),
                app_signature_issuer: None,
                remote_ip: format!("198.51.{}.{}", index / 255, index % 255)
                    .parse()
                    .expect("ip fixture"),
                domain: Some(format!("cloud-{index}.example.test")),
                remote_port: 443,
                protocol: Protocol::Tcp,
                connection_state: ConnectionState::Established,
                first_seen_ms: 10_000 + u64::from(index),
                last_seen_ms: 20_000 + u64::from(index),
                hits: 1,
                failed_hits: 0,
                successful_hits: 1,
            })
            .collect::<Vec<_>>();

        let result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::Csv,
                rows,
            })
            .expect("large cloud import should keep every row");

        assert_eq!(result.requested_count, 602);
        assert_eq!(result.imported_count, 602);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(
            core.list_endpoints().expect("endpoints should list").len(),
            602
        );
        let snapshot = core
            .snapshot(MonitorStatus::Stopped, None)
            .expect("snapshot should load");
        assert_eq!(snapshot.observed_endpoints.len(), 602);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn csv_import_repeated_cloud_rows_are_skipped_instead_of_reimported() {
        let root = unique_temp_dir("csv-import-repeat-cloud-rows");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let rows = (0..605u16)
            .map(|index| MonitoringCsvImportRowDto {
                application: "Cloud Import".to_string(),
                app_connector_id: None,
                cloud_app_id: Some("netstitch.app.cloud-import".to_string()),
                app_signature_key: Some("appsig_v1_cloud_import".to_string()),
                app_signature_subject: Some("CN=Cloud Import".to_string()),
                app_signature_issuer: None,
                remote_ip: format!("198.51.{}.{}", index / 255, index % 255)
                    .parse()
                    .expect("ip fixture"),
                domain: Some(format!("cloud-{index}.example.test")),
                remote_port: 443,
                protocol: Protocol::Tcp,
                connection_state: ConnectionState::Established,
                first_seen_ms: 10_000 + u64::from(index),
                last_seen_ms: 20_000 + u64::from(index),
                hits: 1,
                failed_hits: 0,
                successful_hits: 1,
            })
            .collect::<Vec<_>>();

        let first_result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::CloudDownload,
                rows: rows.clone(),
            })
            .expect("first cloud import should insert rows");
        let second_result = core
            .import_monitoring_csv(MonitoringCsvImportRequestDto {
                import_source: MonitoringImportSourceDto::CloudDownload,
                rows,
            })
            .expect("second cloud import should skip existing rows");

        assert_eq!(first_result.requested_count, 605);
        assert_eq!(first_result.imported_count, 605);
        assert_eq!(first_result.skipped_count, 0);
        assert_eq!(second_result.requested_count, 605);
        assert_eq!(second_result.imported_count, 0);
        assert_eq!(second_result.skipped_count, 605);
        assert_eq!(
            core.list_endpoints().expect("endpoints should list").len(),
            605
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn csv_import_placeholder_cleanup_keeps_duplicate_endpoints_without_tracked_apps() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        create_tracked_apps_table(&conn);
        conn.execute(
            r#"
            INSERT INTO tracked_apps (id, exe_path, process_name, display_name, created_at_ms)
            VALUES (1, 'NetStitch-csv-import://first', 'csv-first', 'CSV First', 1)
            "#,
            [],
        )
        .expect("first CSV placeholder app should insert");
        conn.execute(
            r#"
            INSERT INTO tracked_apps (id, exe_path, process_name, display_name, created_at_ms)
            VALUES (2, 'NetStitch-csv-import://second', 'csv-second', 'CSV Second', 1)
            "#,
            [],
        )
        .expect("second CSV placeholder app should insert");
        conn.execute(
            r#"
            INSERT INTO observed_endpoints (
                tracked_app_id, remote_ip, remote_port, protocol,
                first_seen_ms, last_seen_ms, connection_state
            )
            VALUES (1, '203.0.113.10', 443, 'tcp', 1000, 1000, 'established')
            "#,
            [],
        )
        .expect("first CSV placeholder endpoint should insert");
        conn.execute(
            r#"
            INSERT INTO observed_endpoints (
                tracked_app_id, remote_ip, remote_port, protocol,
                first_seen_ms, last_seen_ms, connection_state
            )
            VALUES (2, '203.0.113.10', 443, 'tcp', 2000, 2000, 'established')
            "#,
            [],
        )
        .expect("second CSV placeholder endpoint should insert");

        super::migrate_csv_import_tracked_apps_to_imported_rows(&conn)
            .expect("CSV placeholder cleanup should not collapse rows into one tracked app id");

        let app_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracked_apps", [], |row| row.get(0))
            .expect("tracked app count should query");
        assert_eq!(app_count, 0);

        let mut stmt = conn
            .prepare(
                "SELECT tracked_app_id, process_name FROM observed_endpoints ORDER BY tracked_app_id ASC",
            )
            .expect("endpoint query should prepare");
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .expect("endpoint query should run")
            .collect::<Result<Vec<_>, _>>()
            .expect("endpoint rows should collect");
        assert_eq!(
            rows,
            vec![
                (1, Some("CSV First".to_string())),
                (2, Some("CSV Second".to_string()))
            ]
        );
    }

    #[test]
    fn app_settings_persist_ui_language_in_sqlite() {
        let root = unique_temp_dir("app-settings-language");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\OverlayTarget.exe"),
                enabled: false,
            })
            .expect("tracked app should insert disabled");

        assert_eq!(
            core.app_settings()
                .expect("settings should load")
                .ui_language_code,
            None
        );
        assert!(
            core.monitoring_public_ip_filter_enabled()
                .expect("default monitoring public IP setting should load"),
            "fresh storage must show public monitoring rows by default"
        );

        core.set_app_setting(netstitch_shared::models::SETTING_UI_LANGUAGE, "ru-ru")
            .expect("language setting should persist");

        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_ENABLE_ALL_OVERLAY,
            "true",
        )
        .expect("enable-all overlay should persist");
        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_REMEMBER_WINDOW_PLACEMENT,
            "true",
        )
        .expect("remember-window-placement should persist");
        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_HIDE_WHEN_MINIMIZED,
            "false",
        )
        .expect("hide-when-minimized should persist");
        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_MODULE_ORDER,
            r#"["module-b","module-a"]"#,
        )
        .expect("module order should persist as UI state");
        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_MONITORING_PUBLIC_IP,
            "false",
        )
        .expect("monitoring public IP filter should persist as UI state");
        core.set_app_setting(
            netstitch_shared::models::SETTING_WEB_ACCESS_LOCALHOST,
            "true",
        )
        .expect("web access setting should persist");
        core.set_app_setting(
            netstitch_shared::models::SETTING_UPDATE_CHECK_INTERVAL_MINUTES,
            "10",
        )
        .expect("update check interval should persist");
        assert_eq!(
            core.app_settings()
                .expect("settings should reload")
                .ui_language_code,
            Some("ru-ru".to_string())
        );
        assert!(
            core.app_settings()
                .expect("settings should reload")
                .ui_enable_all_overlay
        );
        assert!(
            core.app_settings()
                .expect("settings should reload")
                .ui_remember_window_placement
        );
        assert!(
            !core
                .app_settings()
                .expect("settings should reload")
                .ui_hide_when_minimized
        );
        assert_eq!(
            core.app_settings()
                .expect("settings should reload")
                .ui_module_order,
            vec!["module-b".to_string(), "module-a".to_string()]
        );
        assert!(
            !core
                .monitoring_public_ip_filter_enabled()
                .expect("monitoring public IP setting should reload"),
            "public IP monitoring switch must preserve an explicit user override"
        );
        core.set_app_setting(
            netstitch_shared::models::SETTING_UI_MONITORING_PUBLIC_IP,
            "true",
        )
        .expect("monitoring public IP filter should persist as UI state");
        assert!(
            core.monitoring_public_ip_filter_enabled()
                .expect("monitoring public IP setting should reload"),
            "public IP monitoring switch must survive watcher restart through SQLite settings"
        );
        assert!(
            core.app_settings()
                .expect("settings should reload")
                .web_access_localhost
        );
        assert_eq!(
            core.app_settings()
                .expect("settings should reload")
                .update_check_interval_minutes,
            10
        );
        assert!(
            !core
                .list_tracked_apps()
                .expect("base apps should load")
                .iter()
                .find(|item| item.id == app.id)
                .expect("base app should exist")
                .enabled,
            "enable-all overlay must not rewrite tracked_apps.enabled"
        );
        assert!(
            core.list_effective_tracked_apps()
                .expect("effective apps should load")
                .iter()
                .find(|item| item.id == app.id)
                .expect("effective app should exist")
                .enabled,
            "enable-all overlay should make the app effective-enabled"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn snapshot_relocalizes_integration_modules_after_language_change() {
        let root = unique_temp_dir("module-language-refresh");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let module_root = root.join("integrations").join("localized-module");
        fs::create_dir_all(module_root.join("bin")).expect("module bin dir");
        fs::create_dir_all(module_root.join("locales")).expect("module locales dir");
        fs::write(module_root.join("bin").join("localized.dll"), []).expect("library marker");
        fs::write(
            module_root.join("locales").join("en-en.ini"),
            r#"[strings]
localized_module.module.display_name=Localized module
localized_module.module.tooltip=English tooltip
localized_module.entity.help.value=English body
localized_module.entity.input.placeholder=English placeholder
"#,
        )
        .expect("English locale");
        fs::write(
            module_root.join("locales").join("ru-ru.ini"),
            r#"[strings]
localized_module.module.display_name=Локализованный модуль
localized_module.module.tooltip=Русская подсказка
localized_module.entity.help.value=Русский текст
localized_module.entity.input.placeholder=Русский placeholder
"#,
        )
        .expect("Russian locale");
        fs::write(
            module_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "localized-module",
              "display_name": "Fallback module",
              "display_name_key": "localized_module.module.display_name",
              "tooltip": "Fallback tooltip",
              "tooltip_key": "localized_module.module.tooltip",
              "icon_label": "L",
              "ui_schema": [
                {
                  "id": "help",
                  "entity_type": "help_text",
                  "value": "Fallback body",
                  "value_key": "localized_module.entity.help.value"
                },
                {
                  "id": "input",
                  "entity_type": "text_input",
                  "placeholder": "Fallback placeholder",
                  "placeholder_key": "localized_module.entity.input.placeholder"
                }
              ],
              "transport": "native_library",
              "library_paths": {
                "default": "bin/localized.dll"
              }
            }"#,
        )
        .expect("module manifest");
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::with_module_roots(
                root.join("integrations"),
                [root.join("integrations")],
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let english = core
            .snapshot(MonitorStatus::Stopped, None)
            .expect("English snapshot should load");
        assert_eq!(
            english.integration_modules[0].display_name,
            "Localized module"
        );
        assert_eq!(
            english.integration_modules[0].ui_schema[0].value.as_deref(),
            Some("English body")
        );
        assert_eq!(
            english.integration_modules[0].ui_schema[1]
                .placeholder
                .as_deref(),
            Some("English placeholder")
        );

        core.set_app_setting(netstitch_shared::models::SETTING_UI_LANGUAGE, "ru-ru")
            .expect("language setting should persist");
        let russian = core
            .snapshot(MonitorStatus::Stopped, None)
            .expect("Russian snapshot should load after language setting change");

        assert_eq!(
            russian.integration_modules[0].display_name,
            "Локализованный модуль"
        );
        assert_eq!(
            russian.runtime_status.integration_modules[0].display_name,
            "Локализованный модуль"
        );
        assert_eq!(
            russian.integration_modules[0].ui_schema[0].value.as_deref(),
            Some("Русский текст")
        );
        assert_eq!(
            russian.integration_modules[0].ui_schema[1]
                .placeholder
                .as_deref(),
            Some("Русский placeholder")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn integration_root_is_not_persisted_in_core_storage_without_module() {
        let root = unique_temp_dir("integration-core-storage-boundary");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let repo_root = root.join("module-owned-root");

        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        assert!(
            core.configured_integration_root()
                .expect("configured integration root should load")
                .is_none(),
            "core storage must not synthesize module roots without an installed module"
        );
        assert!(
            core.configure_integration_root(None, Some(&repo_root))
                .is_err(),
            "module root persistence is module-owned and requires an installed module"
        );

        let conn = core.open_connection().expect("database should open");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM app_settings WHERE setting_key LIKE '%integration%'",
                [],
                |row| row.get(0),
            )
            .expect("app settings should be queryable");
        assert_eq!(
            count, 0,
            "main SQLite app_settings must not contain external module data"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn core_sqlite_schema_excludes_external_module_runtime_state() {
        let root = unique_temp_dir("core-schema-module-boundary");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let conn = core.open_connection().expect("database should open");
        let expected_tables = BTreeSet::from([
            "app_settings".to_string(),
            "endpoint_domain_cache".to_string(),
            "export_runs".to_string(),
            "ignored_addresses".to_string(),
            "ip_domain_cache".to_string(),
            "ip_whois_ranges".to_string(),
            "observed_endpoints".to_string(),
            "system_events".to_string(),
            "tracked_apps".to_string(),
        ]);

        assert_eq!(
            sqlite_user_tables(&conn),
            expected_tables,
            "core SQLite schema must stay limited to core runtime tables; external module runtime data belongs in integrations/<module>/data/module.sqlite3"
        );

        let forbidden_setting_count: i64 = conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM app_settings
                WHERE setting_key LIKE 'integration.%'
                   OR setting_key LIKE 'module.%'
                   OR setting_key LIKE '%repo_root%'
                   OR setting_key LIKE '%profile_export%'
                   OR setting_key LIKE '%provider%'
                   OR setting_key LIKE '%download%'
                "#,
                [],
                |row| row.get(0),
            )
            .expect("app_settings should be queryable");
        assert_eq!(
            forbidden_setting_count, 0,
            "module roots, providers, downloads, and profile UI state must not be stored in core app_settings"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ignored_addresses_seed_localhost_and_can_be_managed() {
        let root = unique_temp_dir("ignored-addresses");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.clone(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        };
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let seeded = core
            .list_ignored_addresses()
            .expect("ignored addresses should list");
        let seeded_patterns = seeded
            .iter()
            .map(|rule| rule.address_pattern.as_str())
            .collect::<Vec<_>>();
        assert!(seeded_patterns.contains(&"127.0.0.0/8"));
        assert!(seeded_patterns.contains(&"::1/128"));
        assert!(
            seeded_patterns.iter().all(|pattern| {
                pattern.parse::<IpAddr>().is_ok()
                    || pattern
                        .split_once('/')
                        .is_some_and(|(address, _)| address.parse::<IpAddr>().is_ok())
            }),
            "seeded ignored address rules should remain IP or CIDR patterns"
        );

        let added = core
            .add_ignored_address(netstitch_shared::ipc::AddIgnoredAddressRequest {
                address_pattern: "203.0.113.10".to_string(),
            })
            .expect("ignored address should insert");
        assert_eq!(added.address_pattern, "203.0.113.10");

        let deleted = core
            .delete_ignored_address(netstitch_shared::ipc::DeleteIgnoredAddressRequest {
                ignored_address_id: added.id.expect("ignored address id"),
            })
            .expect("ignored address should delete");
        assert_eq!(deleted, 1);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn snapshot_attaches_cached_domain_and_whois_enrichment() {
        let root = unique_temp_dir("ip-enrichment");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\EnrichMe.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let remote_ip = "203.0.113.45".parse::<IpAddr>().expect("ip fixture");
        core.ingest_observation(super::ObservationRecord {
            tracked_app_id: app.id.expect("inserted app id"),
            process_id: Some(10),
            process_name: "EnrichMe.exe".to_string(),
            remote_ip,
            remote_port: 443,
            protocol: netstitch_shared::models::Protocol::Tcp,
            connection_state: netstitch_shared::models::ConnectionState::Established,
            observed_at_ms: 10_000,
        })
        .expect("observation should insert");
        core.upsert_endpoint_domain_cache(super::EndpointDomainCacheRecord {
            tracked_app_id: app.id.expect("inserted app id"),
            remote_ip,
            remote_port: 443,
            protocol: netstitch_shared::models::Protocol::Tcp,
            domain_name: "example.test".to_string(),
            domain_source: netstitch_shared::DOMAIN_SOURCE_TLS_SNI.to_string(),
            observed_at_ms: 20_000,
            updated_at_ms: 20_000,
        })
        .expect("domain cache should persist");
        core.upsert_whois_range_cache(super::WhoisRangeCacheRecord {
            cidr: "203.0.113.0/24".to_string(),
            owner_name: Some("Example Network".to_string()),
            registry: Some("test-registry".to_string()),
            country: Some("ZZ".to_string()),
            source: Some("test".to_string()),
            updated_at_ms: 20_000,
        })
        .expect("whois cache should persist");

        let snapshot = core
            .snapshot(netstitch_shared::models::MonitorStatus::Stopped, None)
            .expect("snapshot should load");
        let enrichment = snapshot.observed_endpoints[0]
            .enrichment
            .as_ref()
            .expect("enrichment should attach");

        assert_eq!(enrichment.domain_name.as_deref(), Some("example.test"));
        assert_eq!(
            enrichment.domain_source.as_deref(),
            Some(netstitch_shared::DOMAIN_SOURCE_TLS_SNI)
        );
        assert_eq!(enrichment.owner_name.as_deref(), Some("Example Network"));
        assert_eq!(enrichment.owner_range.as_deref(), Some("203.0.113.0/24"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn verified_payload_domain_is_stored_with_source() {
        let root = unique_temp_dir("verified-payload-domain");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\HttpHost.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let remote_ip = "203.0.113.47".parse::<IpAddr>().expect("ip fixture");
        core.ingest_observation(super::ObservationRecord {
            tracked_app_id: app.id.expect("inserted app id"),
            process_id: Some(12),
            process_name: "HttpHost.exe".to_string(),
            remote_ip,
            remote_port: 80,
            protocol: netstitch_shared::models::Protocol::Tcp,
            connection_state: netstitch_shared::models::ConnectionState::Established,
            observed_at_ms: 10_000,
        })
        .expect("observation should insert");
        let domain = core
            .detect_and_store_verified_domain(
                app.id.expect("inserted app id"),
                remote_ip,
                80,
                netstitch_shared::models::Protocol::Tcp,
                b"GET / HTTP/1.1\r\nHost: Verified.Example:80\r\n\r\n",
                20_000,
            )
            .expect("verified payload domain should store");

        assert_eq!(domain.as_deref(), Some("verified.example"));
        let snapshot = core
            .snapshot(netstitch_shared::models::MonitorStatus::Stopped, None)
            .expect("snapshot should load");
        let enrichment = snapshot.observed_endpoints[0]
            .enrichment
            .as_ref()
            .expect("verified domain should attach");

        assert_eq!(enrichment.domain_name.as_deref(), Some("verified.example"));
        assert_eq!(
            enrichment.domain_source.as_deref(),
            Some(netstitch_shared::DOMAIN_SOURCE_HTTP_HOST)
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn confirming_exported_endpoint_makes_it_ready_for_next_export() {
        let root = unique_temp_dir("confirm-exported");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        let app = core
            .add_tracked_app(netstitch_shared::ipc::AddTrackedAppRequest {
                exe_path: PathBuf::from(r"C:\Apps\Game.exe"),
                enabled: true,
            })
            .expect("tracked app should insert");
        let endpoint = core
            .ingest_observation(super::ObservationRecord {
                tracked_app_id: app.id.expect("inserted app id"),
                process_id: Some(10),
                process_name: "Game.exe".to_string(),
                remote_ip: "203.0.113.99".parse::<IpAddr>().expect("ip fixture"),
                remote_port: 443,
                protocol: netstitch_shared::models::Protocol::Tcp,
                connection_state: netstitch_shared::models::ConnectionState::Established,
                observed_at_ms: 10_000,
            })
            .expect("observation should insert");
        let endpoint_id = endpoint.id.expect("inserted endpoint id");

        core.confirm_endpoints(netstitch_shared::ipc::ConfirmEndpointsRequest {
            endpoint_ids: vec![endpoint_id],
            confirmed: true,
        })
        .expect("endpoint should confirm");
        core.mark_exported(&[endpoint_id])
            .expect("endpoint should mark exported");
        core.confirm_endpoints(netstitch_shared::ipc::ConfirmEndpointsRequest {
            endpoint_ids: vec![endpoint_id],
            confirmed: true,
        })
        .expect("confirming again should re-arm export");

        let snapshot = core
            .snapshot(netstitch_shared::models::MonitorStatus::Stopped, None)
            .expect("snapshot should load");
        let endpoint = snapshot
            .observed_endpoints
            .iter()
            .find(|item| item.id == Some(endpoint_id))
            .expect("endpoint should remain visible");
        assert!(endpoint.is_confirmed);
        assert!(
            !endpoint.is_exported,
            "add for export must clear the old exported marker"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn system_events_are_normalized_and_rotated_to_9999_rows() {
        let root = unique_temp_dir("system-event-rotation");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let first = core
            .append_system_event(SystemEventRequestDto {
                source: None,
                component: "Cloud UI!".to_string(),
                action_type: "Update Check".to_string(),
                severity: "warn".to_string(),
                entity_type: Some(" github_release ".to_string()),
                entity_id: Some(" v1.1.0.441 ".to_string()),
                payload: serde_json::json!({ "update_available": true }),
            })
            .expect("event should be appended");
        assert_eq!(first.component, "cloudui");
        assert_eq!(first.action_type, "updatecheck");
        assert_eq!(first.severity, "warning");
        assert_eq!(first.source, "core");
        assert_eq!(first.entity_type.as_deref(), Some("github_release"));
        assert_eq!(first.entity_id.as_deref(), Some("v1.1.0.441"));

        {
            let mut conn = Connection::open(&core.paths.database_path).expect("test database");
            let tx = conn.transaction().expect("bulk event transaction");
            {
                let mut stmt = tx
                    .prepare(
                        r#"
                        INSERT INTO system_events (
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
                        VALUES (?1, 1, 'core', 'dns', 'probe_available', 'info', NULL, NULL, ?2)
                        "#,
                    )
                    .expect("bulk event insert should prepare");
                for index in 0..10_005 {
                    stmt.execute(params![
                        1_000_000_i64 + index as i64,
                        serde_json::json!({ "index": index }).to_string()
                    ])
                    .expect("bulk event should insert");
                }
            }
            tx.commit().expect("bulk event transaction should commit");
        }

        core.append_system_event(SystemEventRequestDto {
            source: None,
            component: "dns".to_string(),
            action_type: "probe_available".to_string(),
            severity: "info".to_string(),
            entity_type: None,
            entity_id: None,
            payload: serde_json::json!({ "index": 10_005 }),
        })
        .expect("event should be appended");

        let events = core.list_system_events(20_000).expect("events should list");
        assert_eq!(events.len(), 9_999);
        assert!(
            events.iter().all(|event| event.event_id > first.event_id),
            "rotation should remove the oldest event rows"
        );
        assert_eq!(
            events.first().and_then(|event| event.payload.get("index")),
            Some(&serde_json::json!(10_005))
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn system_events_compact_only_consecutive_identical_rows() {
        let root = unique_temp_dir("system-event-repeat-count");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let request = SystemEventRequestDto {
            source: None,
            component: "ui".to_string(),
            action_type: "message".to_string(),
            severity: "success".to_string(),
            entity_type: Some("footer".to_string()),
            entity_id: None,
            payload: serde_json::json!({ "message": "DNS status: checking" }),
        };
        let first = core
            .append_system_event(request.clone())
            .expect("first event should insert");
        let repeated = core
            .append_system_event(request.clone())
            .expect("second identical event should compact");
        assert_eq!(repeated.event_id, first.event_id);
        assert_eq!(repeated.repeat_count, 2);
        assert_eq!(repeated.severity, "success");

        core.append_system_event(SystemEventRequestDto {
            source: None,
            component: "ui".to_string(),
            action_type: "message".to_string(),
            severity: "info".to_string(),
            entity_type: Some("footer".to_string()),
            entity_id: None,
            payload: serde_json::json!({ "message": "Watcher connection established" }),
        })
        .expect("different event should insert");
        let later_same = core
            .append_system_event(request)
            .expect("same event after another line should insert");

        let events = core.list_system_events(10).expect("events should list");
        assert_eq!(events.len(), 3);
        assert_eq!(later_same.repeat_count, 1);
        assert_ne!(later_same.event_id, first.event_id);
        assert_eq!(
            events
                .iter()
                .find(|event| event.event_id == first.event_id)
                .map(|event| event.repeat_count),
            Some(2)
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn system_events_keep_sources_separate() {
        let root = unique_temp_dir("system-event-source");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        let core_event = core
            .append_system_event(SystemEventRequestDto {
                source: Some("core".to_string()),
                component: "ui".to_string(),
                action_type: "message".to_string(),
                severity: "info".to_string(),
                entity_type: Some("footer".to_string()),
                entity_id: None,
                payload: serde_json::json!({ "message": "same text" }),
            })
            .expect("core event should insert");
        let module_event = core
            .append_system_event(SystemEventRequestDto {
                source: Some("Example Module".to_string()),
                component: "integration".to_string(),
                action_type: "module_message".to_string(),
                severity: "info".to_string(),
                entity_type: Some("module".to_string()),
                entity_id: Some("example".to_string()),
                payload: serde_json::json!({ "message": "same text" }),
            })
            .expect("module event should insert");

        assert_ne!(module_event.event_id, core_event.event_id);
        assert_eq!(module_event.source, "Example Module");
        assert_eq!(
            core.list_system_events(10)
                .expect("events should list")
                .len(),
            2
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn storage_maintenance_marker_clears_only_system_events() {
        let root = unique_temp_dir("system-event-cleanup-marker");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");
        core.set_app_setting("ui.language", "ru-ru")
            .expect("setting should persist before cleanup");
        core.append_system_event(SystemEventRequestDto {
            source: None,
            component: "ui".to_string(),
            action_type: "message".to_string(),
            severity: "info".to_string(),
            entity_type: Some("footer".to_string()),
            entity_id: None,
            payload: serde_json::json!({ "message": "old build log" }),
        })
        .expect("event should be appended");

        let marker_path = root.join(super::CLEAR_SYSTEM_EVENTS_MARKER);
        fs::write(
            &marker_path,
            b"clear system_events on next NetStitch startup",
        )
        .expect("cleanup marker should be written");
        core.apply_storage_maintenance_markers()
            .expect("cleanup marker should be applied");

        assert!(
            !marker_path.exists(),
            "cleanup marker must be removed after it is applied"
        );
        assert!(
            core.list_system_events(10)
                .expect("events should list")
                .is_empty(),
            "cleanup marker must clear only system event history"
        );
        assert_eq!(
            core.get_app_setting("ui.language")
                .expect("setting should load")
                .as_deref(),
            Some("ru-ru"),
            "cleanup marker must not reset user settings or the rest of storage"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn whois_range_cache_rotates_to_reasonable_limit() {
        let root = unique_temp_dir("whois-rotation");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).expect("temp data dir");
        let core = test_core(&root);
        core.paths
            .ensure_directories()
            .expect("directories should exist");
        core.initialize_schema().expect("schema should initialize");

        for index in 0..1005_u16 {
            let third = (index / 255) as u8;
            let fourth = (index % 255) as u8;
            core.upsert_whois_range_cache(super::WhoisRangeCacheRecord {
                cidr: format!("10.{third}.{fourth}.0/24"),
                owner_name: Some(format!("Owner {index}")),
                registry: Some("test".to_string()),
                country: Some("ZZ".to_string()),
                source: Some("test".to_string()),
                updated_at_ms: u64::from(index),
            })
            .expect("whois cache should insert");
        }

        let conn = core.open_connection().expect("db should open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ip_whois_ranges", [], |row| row.get(0))
            .expect("count should load");
        assert_eq!(count, 1000);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_ip_ignore_rule_skips_loopback_and_uses_host_cidr() {
        assert_eq!(
            super::local_ip_ignore_rule("192.168.69.121".parse().expect("ipv4 fixture")),
            Some("192.168.69.121/32".to_string())
        );
        assert_eq!(
            super::local_ip_ignore_rule("fd00::1".parse().expect("ipv6 fixture")),
            Some("fd00::1/128".to_string())
        );
        assert_eq!(
            super::local_ip_ignore_rule("127.0.0.1".parse().expect("loopback fixture")),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule("0.0.0.0".parse().expect("unspecified fixture")),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule("255.255.255.255".parse().expect("broadcast fixture")),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule("192.0.2.1".parse().expect("documentation fixture")),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule(
                "2001:0:14c9:d804:1cd3:27d:da69:4e2d"
                    .parse()
                    .expect("teredo fixture")
            ),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule("2002:c000:0201::1".parse().expect("6to4 fixture")),
            None
        );
        assert_eq!(
            super::local_ip_ignore_rule("2001:db8::1".parse().expect("documentation fixture")),
            None
        );
    }

    #[test]
    fn local_ip_route_probe_uses_ordered_provider_fallbacks() {
        assert_eq!(super::LOCAL_IPV4_ROUTE_PROBE_TARGETS.len(), 8);
        assert_eq!(super::LOCAL_IPV6_ROUTE_PROBE_TARGETS.len(), 8);

        assert_eq!(super::LOCAL_IPV4_ROUTE_PROBE_TARGETS[0], "8.8.8.8:80");
        assert_eq!(super::LOCAL_IPV4_ROUTE_PROBE_TARGETS[1], "8.8.4.4:80");
        assert_eq!(super::LOCAL_IPV4_ROUTE_PROBE_TARGETS[2], "1.1.1.1:80");
        assert_eq!(super::LOCAL_IPV4_ROUTE_PROBE_TARGETS[3], "1.0.0.1:80");

        assert_eq!(
            super::LOCAL_IPV6_ROUTE_PROBE_TARGETS[0],
            "[2001:4860:4860::8888]:80"
        );
        assert_eq!(
            super::LOCAL_IPV6_ROUTE_PROBE_TARGETS[2],
            "[2606:4700:4700::1111]:80"
        );
        assert_eq!(super::local_ip_probe_timeout(), Duration::from_secs(10));
    }

    fn detected_app(
        connector_id: &'static str,
        display_name: &'static str,
        exe_path: &'static str,
        process_name: &'static str,
    ) -> DetectedApp {
        DetectedApp {
            connector_id,
            display_name,
            icon_key: connector_id,
            icon_ico: b"ico",
            icon_svg: b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
            exe_path: PathBuf::from(exe_path),
            process_name,
        }
    }

    fn create_tracked_apps_table(conn: &Connection) {
        conn.execute_batch(
            r#"
            CREATE TABLE tracked_apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exe_path TEXT NOT NULL,
                connector_id TEXT,
                cloud_app_id TEXT,
                process_name TEXT,
                display_name TEXT,
                icon_key TEXT,
                icon_path TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                archived INTEGER NOT NULL DEFAULT 0,
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE observed_endpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tracked_app_id INTEGER NOT NULL,
                cloud_app_id TEXT,
                app_signature_key TEXT,
                app_signature_subject TEXT,
                app_signature_issuer TEXT,
                app_signature_source TEXT,
                process_id INTEGER,
                process_name TEXT,
                remote_ip TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                first_seen_ms INTEGER NOT NULL,
                last_seen_ms INTEGER NOT NULL,
                hits INTEGER NOT NULL DEFAULT 1,
                connection_state TEXT NOT NULL DEFAULT 'unknown',
                failed_hits INTEGER NOT NULL DEFAULT 0,
                successful_hits INTEGER NOT NULL DEFAULT 0,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                is_exported INTEGER NOT NULL DEFAULT 0,
                UNIQUE(tracked_app_id, remote_ip, remote_port, protocol)
            );
            "#,
        )
        .expect("tracked_apps table should be created");
    }

    fn query_tracked_app_identity_rows(
        conn: &Connection,
    ) -> Vec<(String, Option<String>, bool, String, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT exe_path, connector_id, enabled, display_name, icon_key FROM tracked_apps ORDER BY id ASC",
            )
            .expect("identity query should prepare");
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .expect("identity query should run")
        .collect::<Result<Vec<_>, _>>()
        .expect("identity rows should parse")
    }

    fn sqlite_user_tables(conn: &Connection) -> BTreeSet<String> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT name
                FROM sqlite_master
                WHERE type = 'table'
                  AND name NOT LIKE 'sqlite_%'
                ORDER BY name
                "#,
            )
            .expect("sqlite_master query should prepare");
        stmt.query_map([], |row| row.get::<_, String>(0))
            .expect("sqlite_master query should run")
            .collect::<Result<BTreeSet<_>, _>>()
            .expect("sqlite table names should parse")
    }

    fn test_core(root: &std::path::Path) -> super::NetstitchCore {
        super::NetstitchCore {
            paths: super::AppPaths {
                data_dir: root.to_path_buf(),
                database_path: root.join("netstitch.sqlite3"),
                export_dir: root.join("exports"),
                icon_cache_dir: root.join("icons"),
                connector_icon_dir: root.join("icons"),
                external_apps_dir: root.join("external-apps"),
                integrations_dir: root.join("integrations"),
            },
            integration_service: netstitch_integrations::IntegrationService::new(
                root.join("integrations"),
            ),
            integration_module_catalog: std::sync::Arc::new(std::sync::Mutex::new(None)),
            integration_background_tasks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::BTreeMap::new(),
            )),
        }
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("netstitch-core-{prefix}-{}", std::process::id()))
    }
}
