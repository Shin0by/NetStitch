use crate::{
    models::{
        AppSettings, ConnectionState, Filters, IgnoredAddress, IntegrationIntegration,
        ObservationFilter, ObservationRow, Protocol, TrackedApp, UiState, WorkspaceState,
    },
    watcher_api::{
        AddTrackedAppRequest, AppSettingsDto, ClearObservationTagsRequest,
        ConfirmObservationsRequest, DeleteIgnoredAddressRequest, DeleteLocalTagRequest,
        DeleteObservationRequest, DeleteObservationsRequest, DeleteTrackedAppRequest,
        DownloadIntegrationRequest, IgnoreAddressRequest, IgnoredAddressDto,
        MarkObservationsExportedRequest, ObservationFilterDto, ProtocolDto,
        SetAllTrackedAppsEnabledRequest, SetFilterRequest, SetTrackedAppTagRequest,
        SnapshotResponse, ToggleTrackedAppRequest, WatcherApiClient,
    },
};
use netstitch_shared::models::{
    IntegrationModuleUiActionClientRequestDto, IntegrationModuleUiActionResponseDto,
};
use netstitch_shared::models::{MonitoringCsvImportRequestDto, MonitoringCsvImportResultDto};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct MockWatcherApi {
    workspace: WorkspaceState,
}

impl MockWatcherApi {
    pub fn seeded() -> Self {
        Self {
            workspace: seeded_workspace(),
        }
    }

    pub fn configure_integration_folder(
        &mut self,
        _module_id: Option<String>,
        candidate: &str,
    ) -> Result<(), String> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            self.workspace.integration.repo_path = "Not configured".to_string();
            self.workspace.integration.export_path =
                "Configure an integration module before exporting.".to_string();
            self.workspace.integration.reference_data_path =
                "The module primary list path appears after validation.".to_string();
            self.workspace.integration.ready = false;
            self.workspace.integration.status_text =
                "Integration root is not configured yet".to_string();
            self.workspace.integration.last_export_text = "No export yet".to_string();
            self.workspace.integration.provider_name = None;
            self.workspace.integration.repository_url = None;
            self.workspace.ui.error_text = None;
            self.workspace.ui.status_text = "Integration root cleared".to_string();
            return Ok(());
        }

        let root = std::path::PathBuf::from(candidate);
        self.workspace.integration.repo_path = root.display().to_string();
        self.workspace.integration.export_path =
            root.join("data").join("export.txt").display().to_string();
        self.workspace.integration.reference_data_path =
            root.join("data").join("source.txt").display().to_string();
        self.workspace.integration.ready = true;
        self.workspace.integration.status_text = format!(
            "Integration root resolved to {}",
            self.workspace.integration.repo_path
        );
        self.workspace.integration.last_export_text =
            "Integration configured for user exports".to_string();
        self.workspace.ui.error_text = None;
        self.workspace.ui.status_text = self.workspace.integration.status_text.clone();
        Ok(())
    }
}

impl WatcherApiClient for MockWatcherApi {
    fn snapshot(&self) -> SnapshotResponse {
        workspace_to_snapshot(&self.workspace)
    }

    fn set_pending_exe_path(&mut self, pending_exe_path: String) {
        self.workspace.pending_exe_path = pending_exe_path;
    }

    fn add_tracked_app(&mut self, request: AddTrackedAppRequest) {
        self.workspace.pending_exe_path = request.exe_path;
        add_tracked_app(&mut self.workspace);
    }

    fn toggle_tracked_app(&mut self, request: ToggleTrackedAppRequest) {
        toggle_tracking(&mut self.workspace, request.app_id);
    }

    fn delete_tracked_app(&mut self, request: DeleteTrackedAppRequest) {
        delete_tracked_app(&mut self.workspace, request.app_id);
    }

    fn set_tracked_app_tag(&mut self, _request: SetTrackedAppTagRequest) -> Result<(), String> {
        Ok(())
    }

    fn set_all_tracked_apps_enabled(&mut self, request: SetAllTrackedAppsEnabledRequest) {
        set_all_tracked_apps_enabled(&mut self.workspace, request.enabled);
    }

    fn start_monitoring(&mut self) {
        start_monitoring(&mut self.workspace);
    }

    fn stop_monitoring(&mut self) {
        stop_monitoring(&mut self.workspace);
    }

    fn confirm_observations(&mut self, request: ConfirmObservationsRequest) {
        confirm_all_filtered(
            &mut self.workspace,
            &request.observation_ids,
            request.confirmed.unwrap_or(true),
        );
    }

    fn mark_observations_exported(
        &mut self,
        request: MarkObservationsExportedRequest,
    ) -> Result<usize, String> {
        let ids = request.observation_ids.into_iter().collect::<BTreeSet<_>>();
        let mut updated = 0usize;
        for observation in &mut self.workspace.observations {
            if ids.contains(&observation.id) {
                observation.is_exported = true;
                updated += 1;
            }
        }
        Ok(updated)
    }

    fn delete_observation(&mut self, request: DeleteObservationRequest) -> Result<usize, String> {
        self.delete_observations(DeleteObservationsRequest {
            observation_ids: vec![request.observation_id],
        })
    }

    fn delete_observations(&mut self, request: DeleteObservationsRequest) -> Result<usize, String> {
        let ids = request.observation_ids.into_iter().collect::<BTreeSet<_>>();
        let before = self.workspace.observations.len();
        self.workspace
            .observations
            .retain(|observation| !ids.contains(&observation.id));
        Ok(before.saturating_sub(self.workspace.observations.len()))
    }

    fn clear_observation_tags(
        &mut self,
        _request: ClearObservationTagsRequest,
    ) -> Result<usize, String> {
        Ok(0)
    }

    fn delete_local_tag(&mut self, _request: DeleteLocalTagRequest) -> Result<usize, String> {
        Ok(0)
    }

    fn import_monitoring_csv(
        &mut self,
        request: MonitoringCsvImportRequestDto,
    ) -> Result<MonitoringCsvImportResultDto, String> {
        let requested_count = request.rows.len();
        let mut imported_count = 0usize;
        let mut skipped_count = 0usize;
        for row in request.rows {
            if row.remote_port == 0 {
                skipped_count += 1;
                continue;
            }
            let app_name = if row.application.trim().is_empty() {
                "CSV import".to_string()
            } else {
                row.application.trim().to_string()
            };
            let tracked_app_id = ensure_mock_csv_app(&mut self.workspace, &app_name);
            if let Some(existing) = self.workspace.observations.iter_mut().find(|item| {
                item.remote_ip == row.remote_ip.to_string()
                    && item.remote_port == row.remote_port
                    && protocol_matches_shared(&item.protocol, row.protocol)
            }) {
                let _ = existing;
                skipped_count += 1;
                continue;
            } else {
                let id = self.workspace.next_observation_id;
                self.workspace.next_observation_id += 1;
                self.workspace.observations.push(ObservationRow {
                    id,
                    tracked_app_id,
                    process_name: app_name.clone(),
                    remote_ip: row.remote_ip.to_string(),
                    remote_port: row.remote_port,
                    protocol: match row.protocol {
                        netstitch_shared::models::Protocol::Udp => Protocol::Udp,
                        _ => Protocol::Tcp,
                    },
                    first_seen: "CSV import".to_string(),
                    last_seen: "CSV import".to_string(),
                    hits: row.hits.max(1) as u32,
                    connection_state: match row.connection_state {
                        netstitch_shared::models::ConnectionState::Attempting => {
                            ConnectionState::Attempting
                        }
                        netstitch_shared::models::ConnectionState::Established => {
                            ConnectionState::Established
                        }
                        netstitch_shared::models::ConnectionState::Closing => {
                            ConnectionState::Closing
                        }
                        netstitch_shared::models::ConnectionState::Failed => {
                            ConnectionState::Failed
                        }
                        netstitch_shared::models::ConnectionState::Unknown => {
                            ConnectionState::Unknown
                        }
                    },
                    failed_hits: row.failed_hits as u32,
                    successful_hits: row.successful_hits as u32,
                    is_confirmed: false,
                    is_exported: false,
                    enrichment: row
                        .domain
                        .filter(|value| !value.trim().is_empty())
                        .map(|domain| crate::watcher_api::IpEnrichmentDto {
                            domain_name: Some(domain),
                            domain_source: Some(
                                netstitch_shared::DOMAIN_SOURCE_CSV_IMPORT.to_string(),
                            ),
                            ..crate::watcher_api::IpEnrichmentDto::default()
                        }),
                });
            }
            imported_count += 1;
        }
        Ok(MonitoringCsvImportResultDto {
            requested_count,
            imported_count,
            skipped_count,
        })
    }

    fn preview_profile_export(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        mock_profile_export_plan(&self.workspace, request)
    }

    fn analyze_profile_export(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let mut plan = mock_profile_export_plan(&self.workspace, request)?;
        plan.analysis_performed = true;
        plan.new_ips = plan.exported_ips.clone();
        plan.new_count = plan.new_ips.len();
        plan.new_targets = plan.covered_targets.clone();
        plan.new_target_count = plan.new_targets.len();
        Ok(plan)
    }

    fn analyze_profile_export_advanced_settings(
        &mut self,
        module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let domain_count = request.settings.manual_domains.len();
        let mut plan = self.analyze_profile_export(module_id, request.export)?;
        plan.advanced_domain_count = domain_count;
        plan.advanced_file_changes = plan.file_changes.clone();
        Ok(plan)
    }

    fn apply_profile_export(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let plan = mock_profile_export_plan(&self.workspace, request)?;
        self.workspace.ui.error_text = None;
        self.workspace.integration.last_export_text =
            format!("Profile export completed: {} IP(s)", plan.exported_count);
        self.workspace.ui.status_text = self.workspace.integration.last_export_text.clone();
        Ok(plan)
    }

    fn apply_profile_export_advanced_settings(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileAdvancedSettingsRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let domain_count = request.settings.manual_domains.len();
        let mut plan = mock_profile_export_plan(&self.workspace, request.export)?;
        plan.analysis_performed = true;
        plan.advanced_domain_count = domain_count;
        plan.advanced_file_changes = plan.file_changes.clone();
        self.workspace.ui.error_text = None;
        self.workspace.integration.last_export_text =
            "Profile export additional settings applied".to_string();
        self.workspace.ui.status_text = self.workspace.integration.last_export_text.clone();
        Ok(plan)
    }

    fn backup_profile_export(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let plan = mock_profile_export_plan(&self.workspace, request)?;
        self.workspace.ui.error_text = None;
        self.workspace.integration.last_export_text = "Profile export backup created".to_string();
        self.workspace.ui.status_text = self.workspace.integration.last_export_text.clone();
        Ok(plan)
    }

    fn revert_profile_export(
        &mut self,
        _module_id: Option<String>,
        request: netstitch_shared::models::ExportProfileRequestDto,
    ) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
        let mut plan = mock_profile_export_plan(&self.workspace, request)?;
        plan.exported_count = 0;
        plan.exported_ips.clear();
        self.workspace.ui.error_text = None;
        self.workspace.integration.last_export_text = "Profile export changes reverted".to_string();
        self.workspace.ui.status_text = self.workspace.integration.last_export_text.clone();
        Ok(plan)
    }

    fn set_profile_export_ui_state(
        &mut self,
        state: netstitch_shared::models::ProfileExportUiStateDto,
    ) {
        self.workspace.app_settings.profile_export_ui_state = Some(state);
    }

    fn run_integration_module_ui_action(
        &mut self,
        request: IntegrationModuleUiActionClientRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        let message = format!(
            "Module action {}: {} selected row(s)",
            request.action_id,
            request.selected_monitoring_row_ids.len()
        );
        self.workspace.ui.status_text = message.clone();
        Ok(IntegrationModuleUiActionResponseDto {
            message: Some(message),
            severity: "info".to_string(),
            refresh: false,
            commands: Vec::new(),
        })
    }

    fn stop_integration_module_background(&mut self, module_id: String) -> Result<bool, String> {
        self.workspace.ui.status_text = format!("Module listener stopped: {module_id}");
        Ok(false)
    }

    fn send_integration_module_dialog_result(
        &mut self,
        module_id: String,
        dialog_id: String,
        result: String,
    ) -> Result<IntegrationModuleUiActionResponseDto, String> {
        let message = format!("Module dialog {module_id}/{dialog_id}: {result}");
        self.workspace.ui.status_text = message.clone();
        Ok(IntegrationModuleUiActionResponseDto {
            message: Some(message),
            severity: "info".to_string(),
            refresh: false,
            commands: Vec::new(),
        })
    }

    fn set_filters(&mut self, request: SetFilterRequest) {
        self.workspace.filters = Filters {
            app_search: request.app_search,
            search_text: request.search_text,
            domain_search: request.domain_search,
            port_search: request.port_search,
            protocol: request.protocol,
            public_ip: request.public_ip,
            observation_filter: match request.observation_filter {
                ObservationFilterDto::All => ObservationFilter::All,
                ObservationFilterDto::Unconfirmed => ObservationFilter::Unconfirmed,
                ObservationFilterDto::Confirmed => ObservationFilter::Confirmed,
                ObservationFilterDto::Success => ObservationFilter::Success,
                ObservationFilterDto::Exported => ObservationFilter::Exported,
                ObservationFilterDto::Failed => ObservationFilter::Failed,
            },
        };
    }

    fn set_language_code(&mut self, language_code: String) {
        self.workspace.app_settings.language_code = Some(language_code.trim().to_ascii_lowercase());
    }

    fn set_module_order(&mut self, order: Vec<String>) {
        self.workspace.app_settings.module_order = order;
    }

    fn set_web_access_localhost(&mut self, enabled: bool) {
        self.workspace.app_settings.web_access_localhost = enabled;
    }

    fn set_domain_capture_enabled(&mut self, enabled: bool) {
        self.workspace.app_settings.domain_capture_enabled = enabled;
    }

    fn set_remember_window_placement(&mut self, enabled: bool) {
        self.workspace.app_settings.remember_window_placement = enabled;
    }

    fn set_hide_when_minimized(&mut self, enabled: bool) {
        self.workspace.app_settings.hide_when_minimized = enabled;
    }

    fn ignore_address(&mut self, request: IgnoreAddressRequest) {
        let address_pattern = request.address_pattern.trim();
        if address_pattern.is_empty()
            || self
                .workspace
                .ignored_addresses
                .iter()
                .any(|rule| rule.address_pattern.eq_ignore_ascii_case(address_pattern))
        {
            return;
        }

        let id = self.workspace.next_ignored_address_id;
        self.workspace.next_ignored_address_id += 1;
        self.workspace.ignored_addresses.push(IgnoredAddress {
            id,
            address_pattern: address_pattern.to_string(),
            created_at: "2026-04-20 00:00:00".to_string(),
            enrichment: None,
        });
    }

    fn delete_ignored_address(&mut self, request: DeleteIgnoredAddressRequest) {
        self.workspace
            .ignored_addresses
            .retain(|rule| rule.id != request.ignored_address_id);
    }

    fn download_integration(&mut self, request: DownloadIntegrationRequest) {
        self.workspace.integration.repo_path = format!(
            r"C:\Programming\GitHub\NetStitch\storage\external-apps\{}",
            request.provider_id
        );
        self.workspace.integration.export_path =
            format!(r"{}\data\export.txt", self.workspace.integration.repo_path);
        self.workspace.integration.ready = true;
        self.workspace.integration.provider_name = Some(request.provider_id.clone());
        self.workspace.integration.repository_url = None;
        self.workspace.integration.status_text = format!(
            "Integration folder downloaded: {}",
            self.workspace.integration.repo_path
        );
        self.workspace.ui.status_text = self.workspace.integration.status_text.clone();
    }
}

pub fn seeded_workspace() -> WorkspaceState {
    WorkspaceState {
        tracked_apps: vec![
            TrackedApp {
                id: 4,
                connector_id: None,
                cloud_app_id: None,
                display_name: "Demo Game".to_string(),
                icon_key: "manual".to_string(),
                exe_path: r"D:\Games\Demo Game\DemoGame.exe".to_string(),
                enabled: true,
                created_at: "2026-04-19 09:20:00".to_string(),
            },
            TrackedApp {
                id: 1,
                connector_id: Some("discord".to_string()),
                cloud_app_id: Some("netstitch.app.discord".to_string()),
                display_name: "Discord".to_string(),
                icon_key: "discord".to_string(),
                exe_path: r"C:\Users\Demo\AppData\Local\Discord\app-1.0.0\Discord.exe".to_string(),
                enabled: false,
                created_at: "2026-04-19 09:05:12".to_string(),
            },
            TrackedApp {
                id: 3,
                connector_id: Some("chrome".to_string()),
                cloud_app_id: Some("netstitch.app.chrome".to_string()),
                display_name: "Google Chrome".to_string(),
                icon_key: "chrome".to_string(),
                exe_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
                enabled: false,
                created_at: "2026-04-19 09:10:55".to_string(),
            },
            TrackedApp {
                id: 2,
                connector_id: Some("telegram".to_string()),
                cloud_app_id: Some("netstitch.app.telegram".to_string()),
                display_name: "Telegram".to_string(),
                icon_key: "telegram".to_string(),
                exe_path: r"C:\Users\Demo\AppData\Roaming\Telegram Desktop\Telegram.exe"
                    .to_string(),
                enabled: false,
                created_at: "2026-04-19 09:07:41".to_string(),
            },
        ],
        observations: vec![
            ObservationRow {
                id: 1,
                tracked_app_id: 1,
                process_name: "Discord.exe".to_string(),
                remote_ip: "162.159.133.234".to_string(),
                remote_port: 443,
                protocol: Protocol::Tcp,
                first_seen: "2026-04-19 09:11:03".to_string(),
                last_seen: "2026-04-19 09:12:28".to_string(),
                hits: 7,
                connection_state: ConnectionState::Established,
                failed_hits: 0,
                successful_hits: 7,
                is_confirmed: true,
                is_exported: false,
                enrichment: Some(crate::watcher_api::IpEnrichmentDto {
                    domain_name: Some("discord.com".to_string()),
                    domain_source: Some("tls_sni".to_string()),
                    owner_name: Some("Cloudflare, Inc".to_string()),
                    owner_range: Some("162.159.128.0/18".to_string()),
                    registry: Some("arin".to_string()),
                    country: Some("US".to_string()),
                    source: Some("Mock enrichment".to_string()),
                }),
            },
            ObservationRow {
                id: 2,
                tracked_app_id: 1,
                process_name: "Discord.exe".to_string(),
                remote_ip: "104.16.60.15".to_string(),
                remote_port: 443,
                protocol: Protocol::Tcp,
                first_seen: "2026-04-19 09:14:18".to_string(),
                last_seen: "2026-04-19 09:14:18".to_string(),
                hits: 1,
                connection_state: ConnectionState::Attempting,
                failed_hits: 1,
                successful_hits: 0,
                is_confirmed: false,
                is_exported: false,
                enrichment: Some(crate::watcher_api::IpEnrichmentDto {
                    domain_name: Some("cloudflare.com".to_string()),
                    domain_source: Some("tls_sni".to_string()),
                    owner_name: Some("Cloudflare, Inc".to_string()),
                    owner_range: Some("104.16.0.0/12".to_string()),
                    registry: Some("arin".to_string()),
                    country: Some("US".to_string()),
                    source: Some("Mock enrichment".to_string()),
                }),
            },
            ObservationRow {
                id: 3,
                tracked_app_id: 3,
                process_name: "chrome.exe".to_string(),
                remote_ip: "172.217.16.142".to_string(),
                remote_port: 443,
                protocol: Protocol::Udp,
                first_seen: "2026-04-19 09:14:22".to_string(),
                last_seen: "2026-04-19 09:15:09".to_string(),
                hits: 4,
                connection_state: ConnectionState::Failed,
                failed_hits: 4,
                successful_hits: 0,
                is_confirmed: false,
                is_exported: false,
                enrichment: Some(crate::watcher_api::IpEnrichmentDto {
                    domain_name: Some("googleusercontent.com".to_string()),
                    domain_source: Some("tls_sni".to_string()),
                    owner_name: Some("Google LLC".to_string()),
                    owner_range: Some("172.217.0.0/16".to_string()),
                    registry: Some("arin".to_string()),
                    country: Some("US".to_string()),
                    source: Some("Mock enrichment".to_string()),
                }),
            },
        ],
        ignored_addresses: vec![
            IgnoredAddress {
                id: 1,
                address_pattern: "127.0.0.0/8".to_string(),
                created_at: "2026-04-20 00:00:00".to_string(),
                enrichment: None,
            },
            IgnoredAddress {
                id: 2,
                address_pattern: "::1/128".to_string(),
                created_at: "2026-04-20 00:00:00".to_string(),
                enrichment: None,
            },
        ],
        integration: IntegrationIntegration {
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
        ui: UiState {
            monitoring: false,
            status_text: "Idle and waiting for user input".to_string(),
            error_text: None,
        },
        app_settings: AppSettings {
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
        filters: Filters {
            app_search: String::new(),
            search_text: String::new(),
            domain_search: String::new(),
            port_search: String::new(),
            protocol: "All".to_string(),
            public_ip: true,
            observation_filter: ObservationFilter::All,
        },
        pending_exe_path: String::new(),
        next_app_id: 5,
        next_observation_id: 4,
        next_ignored_address_id: 3,
    }
}

pub fn add_tracked_app(workspace: &mut WorkspaceState) {
    let exe_path = workspace.pending_exe_path.trim().to_string();
    if exe_path.is_empty() {
        workspace.ui.error_text = Some("Enter an exe path before adding it.".to_string());
        return;
    }

    let display_name = exe_path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("Tracked app")
        .trim_end_matches(".exe")
        .to_string();

    workspace.tracked_apps.insert(
        0,
        TrackedApp {
            id: workspace.next_app_id,
            connector_id: None,
            cloud_app_id: None,
            display_name,
            icon_key: "manual".to_string(),
            exe_path,
            enabled: false,
            created_at: "2026-04-19 09:20:00".to_string(),
        },
    );
    workspace.next_app_id += 1;
    workspace.pending_exe_path.clear();
    workspace.ui.error_text = None;
    workspace.ui.status_text = "Tracked app added to the shell state".to_string();
}

fn ensure_mock_csv_app(workspace: &mut WorkspaceState, display_name: &str) -> u64 {
    if let Some(app) = workspace
        .tracked_apps
        .iter()
        .find(|app| app.display_name.eq_ignore_ascii_case(display_name))
    {
        return app.id;
    }

    let id = workspace.next_app_id;
    workspace.next_app_id += 1;
    workspace.tracked_apps.push(TrackedApp {
        id,
        connector_id: None,
        cloud_app_id: None,
        display_name: display_name.to_string(),
        icon_key: "manual".to_string(),
        exe_path: format!("NetStitch-csv-import://{}", display_name.replace(' ', "-")),
        enabled: true,
        created_at: "CSV import".to_string(),
    });
    id
}

fn protocol_matches_shared(
    protocol: &Protocol,
    shared: netstitch_shared::models::Protocol,
) -> bool {
    matches!(
        (protocol, shared),
        (Protocol::Tcp, netstitch_shared::models::Protocol::Tcp)
            | (Protocol::Udp, netstitch_shared::models::Protocol::Udp)
    )
}

pub fn toggle_tracking(workspace: &mut WorkspaceState, app_id: u64) {
    if let Some(app) = workspace
        .tracked_apps
        .iter_mut()
        .find(|app| app.id == app_id)
    {
        app.enabled = !app.enabled;
        workspace.ui.status_text = format!(
            "{} is now {}",
            app.display_name,
            if app.enabled { "enabled" } else { "disabled" }
        );
    }
}

pub fn delete_tracked_app(workspace: &mut WorkspaceState, app_id: u64) {
    let Some(position) = workspace
        .tracked_apps
        .iter()
        .position(|app| app.id == app_id)
    else {
        workspace.ui.error_text = Some("Tracked app not found for delete.".to_string());
        return;
    };

    let removed = workspace.tracked_apps.remove(position);
    workspace.ui.error_text = None;
    workspace.ui.status_text = format!("{} was removed from tracked apps", removed.display_name);
}

pub fn set_all_tracked_apps_enabled(workspace: &mut WorkspaceState, enabled: bool) {
    workspace.app_settings.enable_all_overlay = enabled;
    workspace.ui.status_text = if enabled {
        "Enable-all overlay is active".to_string()
    } else {
        "Enable-all overlay is inactive".to_string()
    };
    workspace.ui.error_text = None;
}

pub fn start_monitoring(workspace: &mut WorkspaceState) {
    let enabled_apps: Vec<_> = workspace
        .tracked_apps
        .iter()
        .filter(|app| workspace.app_settings.enable_all_overlay || app.enabled)
        .cloned()
        .collect();

    if enabled_apps.is_empty() {
        workspace.ui.error_text =
            Some("Enable at least one tracked app before starting.".to_string());
        workspace.ui.monitoring = false;
        return;
    }

    workspace.ui.monitoring = true;
    workspace.ui.error_text = None;
    workspace.ui.status_text = format!("Monitoring {} tracked app(s)", enabled_apps.len());

    for app in enabled_apps {
        workspace.observations.push(ObservationRow {
            id: workspace.next_observation_id,
            tracked_app_id: app.id,
            process_name: format!("{}.exe", app.display_name),
            remote_ip: match app.id % 3 {
                0 => "203.0.113.45",
                1 => "104.18.22.12",
                _ => "162.159.130.234",
            }
            .to_string(),
            remote_port: 443,
            protocol: if app.id % 2 == 0 {
                Protocol::Udp
            } else {
                Protocol::Tcp
            },
            first_seen: "2026-04-19 09:20:00".to_string(),
            last_seen: "2026-04-19 09:20:00".to_string(),
            hits: 1,
            connection_state: ConnectionState::Established,
            failed_hits: 0,
            successful_hits: 1,
            is_confirmed: false,
            is_exported: false,
            enrichment: None,
        });
        workspace.next_observation_id += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{MockWatcherApi, delete_tracked_app, seeded_workspace};

    #[test]
    fn seeded_workspace_contains_detected_connector_apps_disabled() {
        let workspace = seeded_workspace();
        assert_eq!(workspace.tracked_apps.len(), 4);

        let demo_game = workspace
            .tracked_apps
            .first()
            .expect("manual demo app must be first");
        assert_eq!(demo_game.display_name, "Demo Game");
        assert!(demo_game.enabled);
        assert_eq!(
            workspace
                .tracked_apps
                .iter()
                .map(|app| app.display_name.as_str())
                .collect::<Vec<_>>(),
            vec!["Demo Game", "Discord", "Google Chrome", "Telegram"]
        );

        for display_name in ["Discord", "Telegram", "Google Chrome"] {
            let app = workspace
                .tracked_apps
                .iter()
                .find(|app| app.display_name == display_name)
                .expect("connector app must exist");
            assert!(!app.enabled);
        }

        let chrome = workspace
            .tracked_apps
            .iter()
            .find(|app| app.display_name == "Google Chrome")
            .expect("chrome connector app must exist");
        assert_eq!(
            chrome.exe_path,
            r"C:\Program Files\Google\Chrome\Application\chrome.exe"
        );
    }

    #[test]
    fn delete_tracked_app_removes_app_from_tracking_and_keeps_observations() {
        let mut workspace = seeded_workspace();
        workspace.filters.app_search = "Discord".to_string();

        delete_tracked_app(&mut workspace, 1);

        assert!(workspace.tracked_apps.iter().all(|app| app.id != 1));
        assert!(
            workspace
                .observations
                .iter()
                .any(|observation| observation.tracked_app_id == 1)
        );
        assert_eq!(workspace.filters.app_search, "Discord");
        assert!(workspace.ui.error_text.is_none());
    }

    #[test]
    fn empty_integration_path_clears_configuration_without_validation_error() {
        let mut api = MockWatcherApi::seeded();
        api.workspace.integration.ready = true;
        api.workspace.integration.repo_path = r"C:\Tools\integration".to_string();
        api.workspace.integration.export_path = r"C:\Tools\integration\data\export.txt".to_string();
        api.workspace.integration.provider_name = Some("Sample integration".to_string());

        api.configure_integration_folder(None, "   ")
            .expect("empty path should clear integration configuration");

        assert!(!api.workspace.integration.ready);
        assert_eq!(api.workspace.integration.repo_path, "Not configured");
        assert!(api.workspace.integration.provider_name.is_none());
        assert!(api.workspace.ui.error_text.is_none());
    }
}

pub fn stop_monitoring(workspace: &mut WorkspaceState) {
    workspace.ui.monitoring = false;
    workspace.ui.status_text = "Monitoring paused".to_string();
}

pub fn confirm_all_filtered(
    workspace: &mut WorkspaceState,
    observation_ids: &[u64],
    confirmed: bool,
) {
    for observation in workspace
        .observations
        .iter_mut()
        .filter(|observation| observation_ids.contains(&observation.id))
    {
        observation.is_confirmed = confirmed;
        observation.is_exported = false;
    }
    workspace.ui.status_text = if confirmed {
        "Selected observations were marked for export".to_string()
    } else {
        "Selected observations were unconfirmed".to_string()
    };
}

fn mock_profile_export_plan(
    workspace: &WorkspaceState,
    request: netstitch_shared::models::ExportProfileRequestDto,
) -> Result<netstitch_shared::models::ExportProfilePlanDto, String> {
    if !workspace.integration.ready {
        return Err(
            "Configure the integration folder before exporting confirmed IP addresses.".to_string(),
        );
    }
    if request.mode == netstitch_shared::models::ExportModeDto::MergeIntoExistingLists
        && !request.dangerous_confirmed
    {
        return Err("dangerous merge export requires explicit confirmation".to_string());
    }

    let exported_ips = workspace
        .observations
        .iter()
        .filter(|observation| observation.is_confirmed && !observation.is_exported)
        .filter_map(|observation| observation.remote_ip.parse().ok())
        .collect::<std::collections::BTreeSet<std::net::IpAddr>>()
        .into_iter()
        .collect::<Vec<_>>();
    let skipped_unconfirmed = workspace
        .observations
        .iter()
        .filter(|observation| !observation.is_confirmed)
        .count();
    let skipped_exported = workspace
        .observations
        .iter()
        .filter(|observation| observation.is_exported)
        .count();
    let repo_root = if request.repo_root.as_os_str().is_empty() {
        std::path::PathBuf::from(&workspace.integration.repo_path)
    } else {
        request.repo_root
    };
    let file_path = repo_root.join("data").join("netstitch-export.txt");

    Ok(netstitch_shared::models::ExportProfilePlanDto {
        provider_id: request.provider_id,
        repo_root,
        mode: request.mode,
        selected_profile_path: request.selected_profile_path,
        generated_profile_path: request.generated_profile_name.map(|name| {
            std::path::PathBuf::from(&workspace.integration.repo_path)
                .join(format!("{name}.NetStitch.bat"))
        }),
        exported_count: exported_ips.len(),
        covered_count: exported_ips.len(),
        covered_ips: exported_ips.clone(),
        covered_targets: exported_ips.iter().map(ToString::to_string).collect(),
        covered_target_count: exported_ips.len(),
        use_whois_ranges_for_export: false,
        covered_range_count: 0,
        covered_range_address_count: "0".to_string(),
        uncovered_targets: Vec::new(),
        uncovered_count: 0,
        exported_ips,
        analysis_performed: false,
        existing_ips: Vec::new(),
        existing_count: 0,
        existing_targets: Vec::new(),
        existing_target_count: 0,
        existing_range_count: 0,
        existing_range_address_count: "0".to_string(),
        new_ips: Vec::new(),
        new_count: 0,
        new_targets: Vec::new(),
        new_target_count: 0,
        new_range_count: 0,
        new_range_address_count: "0".to_string(),
        advanced_domain_count: 0,
        advanced_existing_domain_count: 0,
        advanced_new_domain_count: 0,
        skipped_unconfirmed,
        skipped_exported,
        skipped_duplicates: 0,
        matched_rules: Vec::new(),
        warnings: Vec::new(),
        file_changes: vec![netstitch_shared::models::ExportFileChangeDto::new(
            file_path,
            netstitch_shared::models::ExportFileOperationDto::Update,
            None,
        )],
        advanced_file_changes: Vec::new(),
        created_at_ms: 0,
    })
}

fn workspace_to_snapshot(workspace: &WorkspaceState) -> crate::watcher_api::SnapshotResponse {
    crate::watcher_api::SnapshotResponse {
        tracked_apps: workspace
            .tracked_apps
            .iter()
            .map(|app| crate::watcher_api::TrackedAppDto {
                id: app.id,
                connector_id: app.connector_id.clone(),
                cloud_app_id: app.cloud_app_id.clone(),
                display_name: app.display_name.clone(),
                icon_key: app.icon_key.clone(),
                icon_path: None,
                current_tag: None,
                exe_path: app.exe_path.clone(),
                enabled: workspace.app_settings.enable_all_overlay || app.enabled,
                created_at: app.created_at.clone(),
            })
            .collect(),
        observations: workspace
            .observations
            .iter()
            .map(|observation| crate::watcher_api::ObservationDto {
                id: observation.id,
                tracked_app_id: observation.tracked_app_id,
                cloud_app_id: None,
                app_signature_key: None,
                app_signature_subject: None,
                app_signature_issuer: None,
                app_signature_source: None,
                process_name: observation.process_name.clone(),
                remote_ip: observation.remote_ip.clone(),
                remote_port: observation.remote_port,
                protocol: match observation.protocol {
                    Protocol::Tcp => ProtocolDto::Tcp,
                    Protocol::Udp => ProtocolDto::Udp,
                },
                first_seen_ms: 0,
                first_seen: observation.first_seen.clone(),
                last_seen_ms: 0,
                last_seen: observation.last_seen.clone(),
                hits: observation.hits,
                connection_state: match observation.connection_state {
                    ConnectionState::Unknown => crate::watcher_api::ConnectionStateDto::Unknown,
                    ConnectionState::Attempting => {
                        crate::watcher_api::ConnectionStateDto::Attempting
                    }
                    ConnectionState::Established => {
                        crate::watcher_api::ConnectionStateDto::Established
                    }
                    ConnectionState::Closing => crate::watcher_api::ConnectionStateDto::Closing,
                    ConnectionState::Failed => crate::watcher_api::ConnectionStateDto::Failed,
                },
                failed_hits: observation.failed_hits,
                successful_hits: observation.successful_hits,
                is_confirmed: observation.is_confirmed,
                is_exported: observation.is_exported,
                tags: Vec::new(),
                enrichment: observation.enrichment.clone(),
            })
            .collect(),
        ignored_addresses: workspace
            .ignored_addresses
            .iter()
            .map(|rule| IgnoredAddressDto {
                id: rule.id,
                address_pattern: rule.address_pattern.clone(),
                created_at: rule.created_at.clone(),
                enrichment: rule.enrichment.clone(),
            })
            .collect(),
        integration: crate::watcher_api::IntegrationIntegrationDto {
            repo_path: workspace.integration.repo_path.clone(),
            export_path: workspace.integration.export_path.clone(),
            reference_data_path: workspace.integration.reference_data_path.clone(),
            profile_paths: workspace.integration.profile_paths.clone(),
            ready: workspace.integration.ready,
            status_text: workspace.integration.status_text.clone(),
            last_export_text: workspace.integration.last_export_text.clone(),
            provider_id: workspace.integration.provider_id.clone(),
            provider_name: workspace.integration.provider_name.clone(),
            repository_url: workspace.integration.repository_url.clone(),
        },
        integration_modules: vec![netstitch_shared::models::IntegrationModuleDto {
            id: "sample".to_string(),
            display_name: "Sample".to_string(),
            tooltip: "Sample integration module".to_string(),
            icon_label: "S".to_string(),
            enabled: true,
            display_name_key: None,
            tooltip_key: None,
            provider_title: Some("Sample: provider".to_string()),
            provider_title_key: None,
            export_title: Some("Sample: export".to_string()),
            export_title_key: None,
            button_color: Some("#1287d8".to_string()),
            icon_svg: None,
            icon_path: None,
            icon_data_uri: None,
            header_actions: Vec::new(),
            ui_schema: Vec::new(),
            background_active: false,
            background_subscriptions: Vec::new(),
            background_status: None,
        }],
        integration_providers: Vec::new(),
        ui: crate::watcher_api::UiStatusDto {
            monitoring: workspace.ui.monitoring,
            watcher_connected: true,
            snapshot_loaded: true,
            status_text: workspace.ui.status_text.clone(),
            error_text: workspace.ui.error_text.clone(),
        },
        app_settings: AppSettingsDto {
            language_code: workspace.app_settings.language_code.clone(),
            enable_all_overlay: workspace.app_settings.enable_all_overlay,
            remember_window_placement: workspace.app_settings.remember_window_placement,
            hide_when_minimized: workspace.app_settings.hide_when_minimized,
            module_order: workspace.app_settings.module_order.clone(),
            web_access_localhost: workspace.app_settings.web_access_localhost,
            domain_capture_enabled: workspace.app_settings.domain_capture_enabled,
            update_check_interval_minutes: workspace.app_settings.update_check_interval_minutes,
            profile_export_ui_state: workspace.app_settings.profile_export_ui_state.clone(),
        },
        runtime_status: crate::watcher_api::RuntimeStatusDto {
            connector_apps_detected: workspace
                .tracked_apps
                .iter()
                .filter(|app| app.icon_key != "manual")
                .count(),
            unavailable_tracked_apps: Vec::new(),
            integration_modules: Vec::new(),
            endpoint_probe: crate::watcher_api::EndpointProbeStatusDto {
                is_checking: false,
                first_successful_target: Some("1.1.1.1:53".to_string()),
                probes: vec![
                    crate::watcher_api::EndpointProbeTargetDto {
                        target: "8.8.8.8:53".to_string(),
                        available: Some(false),
                        error: Some("mock unavailable".to_string()),
                    },
                    crate::watcher_api::EndpointProbeTargetDto {
                        target: "1.1.1.1:53".to_string(),
                        available: Some(true),
                        error: None,
                    },
                ],
            },
            tool_available: true,
            domain_capture: crate::watcher_api::DomainCaptureStatusDto::default(),
            flow_capture: crate::watcher_api::FlowCaptureStatusDto::default(),
            domain_capture_admin_disabled: false,
            is_elevated: true,
        },
        filters: crate::watcher_api::FiltersDto {
            app_search: workspace.filters.app_search.clone(),
            search_text: workspace.filters.search_text.clone(),
            domain_search: workspace.filters.domain_search.clone(),
            port_search: workspace.filters.port_search.clone(),
            protocol: workspace.filters.protocol.clone(),
            public_ip: workspace.filters.public_ip,
            observation_filter: match workspace.filters.observation_filter {
                ObservationFilter::All => ObservationFilterDto::All,
                ObservationFilter::Unconfirmed => ObservationFilterDto::Unconfirmed,
                ObservationFilter::Confirmed => ObservationFilterDto::Confirmed,
                ObservationFilter::Success => ObservationFilterDto::Success,
                ObservationFilter::Exported => ObservationFilterDto::Exported,
                ObservationFilter::Failed => ObservationFilterDto::Failed,
            },
        },
        pending_exe_path: workspace.pending_exe_path.clone(),
    }
}
