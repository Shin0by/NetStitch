use std::collections::BTreeMap;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use libloading::Library;
use netstitch_shared::models::{
    ExportProfileAdvancedSettingsRequestDto, ExportProfilePlanDto, ExportProfileRequestDto,
    INTEGRATION_ABI_CALL_EXPORT, INTEGRATION_ABI_FREE_EXPORT, INTEGRATION_ABI_VERSION,
    IntegrationAbiBuffer, IntegrationDownloadProgressDto, IntegrationHostEvent,
    IntegrationHostRequest, IntegrationHostResponse, IntegrationModuleActionDto,
    IntegrationModuleBackgroundEventDto, IntegrationModuleDto, IntegrationModuleRuntimeStatusDto,
    IntegrationModuleUiActionRequestDto, IntegrationModuleUiActionResponseDto,
    IntegrationOverlayTarget, IntegrationProfileExportRequestDto, IntegrationProviderDto,
    IntegrationRootRequestDto, IntegrationStatusDto, IntegrationUiEntityDto,
    IntegrationUiOptionDto, IntegrationUiProgressStageDto, IntegrationUiTableColumnDto,
    ObservedEndpoint, ProfileExportUiStateDto,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct IntegrationService {
    storage_root: PathBuf,
    module_roots: Vec<PathBuf>,
    manifest_cache: Arc<OnceLock<Vec<ModuleManifestCacheEntry>>>,
}

impl IntegrationService {
    pub fn new(storage_root: impl Into<PathBuf>) -> Self {
        Self {
            storage_root: storage_root.into(),
            module_roots: default_module_roots(),
            manifest_cache: Arc::new(OnceLock::new()),
        }
    }

    pub fn with_module_roots(
        storage_root: impl Into<PathBuf>,
        module_roots: impl IntoIterator<Item = PathBuf>,
    ) -> Self {
        Self {
            storage_root: storage_root.into(),
            module_roots: module_roots.into_iter().collect(),
            manifest_cache: Arc::new(OnceLock::new()),
        }
    }

    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }

    pub fn module_storage_dir(&self, module_id: &str) -> PathBuf {
        self.manifest_for_id(module_id)
            .map(|manifest| manifest.storage_dir())
            .unwrap_or_else(|_| self.storage_root.join(normalize_module_id(module_id)))
    }

    pub fn module_external_apps_dir(&self, module_id: &str) -> PathBuf {
        self.module_storage_dir(module_id).join("external-apps")
    }

    pub fn primary_module_id(&self) -> Result<String> {
        self.available_modules_for_language(None)?
            .into_iter()
            .find(|module| module.enabled)
            .map(|module| module.id)
            .ok_or_else(|| anyhow!("no external integration modules are available"))
    }

    pub fn available_modules(&self) -> Result<Vec<IntegrationModuleDto>> {
        self.available_modules_for_language(None)
    }

    pub fn available_modules_for_language(
        &self,
        language_code: Option<&str>,
    ) -> Result<Vec<IntegrationModuleDto>> {
        let mut modules = Vec::new();
        for manifest in self.module_manifests()? {
            let enabled = manifest.load_error().is_none();
            let locale = ModuleLocale::load(&manifest.base_dir, language_code);
            modules.push(manifest.to_dto(enabled, &locale)?);
        }
        modules.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(modules)
    }

    pub fn module_runtime_statuses_for_language(
        &self,
        language_code: Option<&str>,
    ) -> Result<Vec<IntegrationModuleRuntimeStatusDto>> {
        let mut statuses = Vec::new();
        for entry in self.cached_manifest_entries()? {
            match entry {
                ModuleManifestCacheEntry::Valid(manifest) => {
                    let locale = ModuleLocale::load(&manifest.base_dir, language_code);
                    let display_name = localized_manifest_text(
                        &locale,
                        manifest.display_name_key.as_deref(),
                        &manifest.display_name,
                    );
                    let load_error = manifest.load_error();
                    let connection_error = load_error;
                    statuses.push(IntegrationModuleRuntimeStatusDto {
                        id: manifest.id.clone(),
                        display_name,
                        manifest_path: manifest.base_dir.join("module.json"),
                        connected: connection_error.is_none(),
                        enabled: connection_error.is_none(),
                        error: connection_error,
                        background_active: false,
                        background_subscriptions: Vec::new(),
                        background_status: None,
                    });
                }
                ModuleManifestCacheEntry::Empty => {}
                ModuleManifestCacheEntry::Error {
                    manifest_path,
                    error,
                } => {
                    let id = manifest_path
                        .parent()
                        .and_then(|path| path.file_name())
                        .and_then(|value| value.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    statuses.push(IntegrationModuleRuntimeStatusDto {
                        id: id.clone(),
                        display_name: id,
                        manifest_path,
                        connected: false,
                        enabled: false,
                        error: Some(error),
                        background_active: false,
                        background_subscriptions: Vec::new(),
                        background_status: None,
                    });
                }
            }
        }
        statuses.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(statuses)
    }

    pub fn module_database_path(&self, module_id: &str) -> PathBuf {
        self.module_storage_dir(module_id).join("module.sqlite3")
    }

    pub fn module_storage_allowed_path(&self, module_id: &str, path: &Path) -> Result<bool> {
        let storage_dir = normalize_path_for_boundary(&self.module_storage_dir(module_id))?;
        let candidate = normalize_path_for_boundary(path)?;
        Ok(candidate == storage_dir || candidate.starts_with(storage_dir))
    }

    pub fn ensure_module_storage(&self, module_id: &str) -> Result<PathBuf> {
        let storage_dir = self.module_storage_dir(module_id);
        ensure_module_database(&storage_dir)
    }

    pub fn configured_root(&self, module_id: &str) -> Result<Option<PathBuf>> {
        self.call_module(module_id, "configured_root", serde_json::Value::Null)
    }

    pub fn configure_root(
        &self,
        module_id: &str,
        candidate: Option<&Path>,
    ) -> Result<Option<IntegrationStatusDto>> {
        self.call_module(
            module_id,
            "configure_root",
            serde_json::to_value(IntegrationRootRequestDto {
                repo_root: candidate.map(Path::to_path_buf),
            })?,
        )
    }

    pub fn status(&self, module_id: &str) -> Result<Option<IntegrationStatusDto>> {
        self.call_module(module_id, "status", serde_json::Value::Null)
    }

    pub fn status_for_root(&self, module_id: &str, root: &Path) -> Result<IntegrationStatusDto> {
        self.call_module(
            module_id,
            "status_for_root",
            serde_json::to_value(IntegrationRootRequestDto {
                repo_root: Some(root.to_path_buf()),
            })?,
        )
    }

    pub fn providers(&self, module_id: &str) -> Result<Vec<IntegrationProviderDto>> {
        self.call_module(module_id, "providers", serde_json::Value::Null)
    }

    pub fn user_export_path(&self, module_id: &str) -> Result<PathBuf> {
        self.call_module(module_id, "user_export_path", serde_json::Value::Null)
    }

    pub fn ensure_generated_overlay(&self, module_id: &str) -> Result<Option<PathBuf>> {
        self.call_module(
            module_id,
            "ensure_generated_overlay",
            serde_json::Value::Null,
        )
    }

    pub fn download_provider(
        &self,
        module_id: &str,
        provider_id: &str,
    ) -> Result<IntegrationStatusDto> {
        self.download_provider_with_progress(module_id, provider_id, |_| {})
    }

    pub fn download_provider_with_progress(
        &self,
        module_id: &str,
        provider_id: &str,
        progress: impl FnMut(IntegrationDownloadProgressDto),
    ) -> Result<IntegrationStatusDto> {
        let payload = serde_json::json!({ "provider_id": provider_id });
        self.call_module_with_progress(module_id, "download_provider", payload, progress)
    }

    pub fn preview_profile_export(
        &self,
        module_id: &str,
        request: ExportProfileRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_profile_action(module_id, "preview_profile_export", request, endpoints)
    }

    pub fn analyze_profile_export(
        &self,
        module_id: &str,
        request: ExportProfileRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_profile_action(module_id, "analyze_profile_export", request, endpoints)
    }

    pub fn analyze_profile_export_advanced_settings(
        &self,
        module_id: &str,
        request: ExportProfileAdvancedSettingsRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_module(
            module_id,
            "analyze_profile_export_advanced_settings",
            serde_json::to_value(IntegrationProfileExportRequestDto {
                request,
                endpoints: endpoints.to_vec(),
            })?,
        )
    }

    pub fn apply_profile_export(
        &self,
        module_id: &str,
        request: ExportProfileRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_profile_action(module_id, "apply_profile_export", request, endpoints)
    }

    pub fn apply_profile_export_advanced_settings(
        &self,
        module_id: &str,
        request: ExportProfileAdvancedSettingsRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_module(
            module_id,
            "apply_profile_export_advanced_settings",
            serde_json::to_value(IntegrationProfileExportRequestDto {
                request,
                endpoints: endpoints.to_vec(),
            })?,
        )
    }

    pub fn backup_profile_export(
        &self,
        module_id: &str,
        request: ExportProfileRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_profile_action(module_id, "backup_profile_export", request, endpoints)
    }

    pub fn revert_profile_export(
        &self,
        module_id: &str,
        request: ExportProfileRequestDto,
    ) -> Result<ExportProfilePlanDto> {
        self.call_module(
            module_id,
            "revert_profile_export",
            serde_json::to_value(request)?,
        )
    }

    pub fn profile_export_ui_state(
        &self,
        module_id: &str,
    ) -> Result<Option<ProfileExportUiStateDto>> {
        self.call_module(
            module_id,
            "profile_export_ui_state",
            serde_json::Value::Null,
        )
    }

    pub fn set_profile_export_ui_state(
        &self,
        module_id: &str,
        state: &ProfileExportUiStateDto,
    ) -> Result<()> {
        self.call_module(
            module_id,
            "set_profile_export_ui_state",
            serde_json::to_value(state)?,
        )
    }

    pub fn overlay_target(&self, module_id: &str, root: &Path) -> Result<IntegrationOverlayTarget> {
        self.call_module(
            module_id,
            "overlay_target",
            serde_json::to_value(IntegrationRootRequestDto {
                repo_root: Some(root.to_path_buf()),
            })?,
        )
    }

    pub fn run_ui_action(
        &self,
        module_id: &str,
        request: IntegrationModuleUiActionRequestDto,
    ) -> Result<IntegrationModuleUiActionResponseDto> {
        self.call_module(module_id, "ui_action", serde_json::to_value(request)?)
    }

    pub fn run_ui_action_with_events(
        &self,
        module_id: &str,
        request: IntegrationModuleUiActionRequestDto,
        event_handler: impl FnMut(IntegrationHostEvent),
    ) -> Result<IntegrationModuleUiActionResponseDto> {
        self.call_module_with_events(
            module_id,
            "ui_action",
            serde_json::to_value(request)?,
            event_handler,
        )
    }

    pub fn run_background_event(
        &self,
        module_id: &str,
        event: IntegrationModuleBackgroundEventDto,
    ) -> Result<IntegrationModuleUiActionResponseDto> {
        self.call_module(module_id, "background_event", serde_json::to_value(event)?)
    }

    fn call_profile_action(
        &self,
        module_id: &str,
        action: &str,
        request: ExportProfileRequestDto,
        endpoints: &[ObservedEndpoint],
    ) -> Result<ExportProfilePlanDto> {
        self.call_module(
            module_id,
            action,
            serde_json::to_value(IntegrationProfileExportRequestDto {
                request,
                endpoints: endpoints.to_vec(),
            })?,
        )
    }

    fn call_module<T>(&self, module_id: &str, action: &str, payload: serde_json::Value) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.call_module_with_events(module_id, action, payload, |_| {})
    }

    fn call_module_with_progress<T>(
        &self,
        module_id: &str,
        action: &str,
        payload: serde_json::Value,
        mut progress: impl FnMut(IntegrationDownloadProgressDto),
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.call_module_with_events(module_id, action, payload, |event| {
            if event.event == "download_progress" {
                if let Ok(value) =
                    serde_json::from_value::<IntegrationDownloadProgressDto>(event.payload)
                {
                    progress(value);
                }
            }
        })
    }

    fn call_module_with_events<T>(
        &self,
        module_id: &str,
        action: &str,
        payload: serde_json::Value,
        mut event_handler: impl FnMut(IntegrationHostEvent),
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let manifest = self.manifest_for_id(module_id)?;
        let storage_dir = manifest.storage_dir();
        ensure_module_database(&storage_dir)?;
        let request = IntegrationHostRequest {
            abi_version: INTEGRATION_ABI_VERSION,
            module_id: manifest.id.clone(),
            storage_dir,
            action: action.to_string(),
            payload,
        };
        let result = match manifest.transport {
            IntegrationModuleTransport::NativeLibrary => {
                self.call_native_library_module(&manifest, &request, |event| {
                    event_handler(event);
                })?
            }
        };
        serde_json::from_value(result).context("failed to decode integration module result")
    }

    fn call_native_library_module(
        &self,
        manifest: &IntegrationModuleManifest,
        request: &IntegrationHostRequest,
        mut event_handler: impl FnMut(IntegrationHostEvent),
    ) -> Result<serde_json::Value> {
        let library_path = manifest.library_path()?;
        let request_bytes = serde_json::to_vec(request)?;

        unsafe {
            let library = Library::new(&library_path).with_context(|| {
                format!(
                    "failed to load integration library {}",
                    library_path.display()
                )
            })?;
            let call = library
                .get::<netstitch_shared::models::IntegrationAbiCall>(INTEGRATION_ABI_CALL_EXPORT)
                .with_context(|| {
                    format!(
                        "integration library {} does not expose netstitch_integration_call",
                        library_path.display()
                    )
                })?;
            let free = library
                .get::<netstitch_shared::models::IntegrationAbiFree>(INTEGRATION_ABI_FREE_EXPORT)
                .with_context(|| {
                    format!(
                        "integration library {} does not expose netstitch_integration_free",
                        library_path.display()
                    )
                })?;
            let mut output = IntegrationAbiBuffer {
                ptr: std::ptr::null_mut(),
                len: 0,
            };
            let mut callback_data = CallbackUserData {
                handler: &mut event_handler,
            };
            let code = call(
                request_bytes.as_ptr(),
                request_bytes.len(),
                Some(module_event_callback),
                &mut callback_data as *mut CallbackUserData<'_> as *mut c_void,
                &mut output,
            );
            read_module_response(code, output, *free)
        }
    }

    fn module_manifests(&self) -> Result<Vec<IntegrationModuleManifest>> {
        Ok(self
            .cached_manifest_entries()?
            .into_iter()
            .filter_map(|entry| match entry {
                ModuleManifestCacheEntry::Valid(manifest) => Some(manifest),
                ModuleManifestCacheEntry::Empty | ModuleManifestCacheEntry::Error { .. } => None,
            })
            .collect())
    }

    fn module_manifest_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for root in &self.module_roots {
            if !root.is_dir() {
                continue;
            }
            let root_manifest_path = root.join("module.json");
            if root_manifest_path.is_file() {
                paths.push(root_manifest_path);
            }
            for entry in fs::read_dir(root)
                .with_context(|| format!("failed to read integration root {}", root.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("module.json");
                if !manifest_path.is_file() {
                    continue;
                }
                paths.push(manifest_path);
            }
        }
        Ok(paths)
    }

    fn read_manifest_at(&self, manifest_path: &Path) -> Result<Option<IntegrationModuleManifest>> {
        let manifest_text = fs::read_to_string(manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let mut manifest: IntegrationModuleManifest = serde_json::from_str(&manifest_text)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
        manifest.base_dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        if manifest.schema != IntegrationModuleManifest::SCHEMA {
            return Err(anyhow!(
                "unsupported module manifest schema '{}' in {}",
                manifest.schema,
                manifest_path.display()
            ));
        }
        if module_source_name_is_reserved(&manifest.id)
            || module_source_name_is_reserved(&manifest.display_name)
        {
            return Err(anyhow!(
                "integration module '{}' uses reserved source name; 'core' and 'system' are reserved",
                manifest.id
            ));
        }
        Ok(Some(manifest))
    }

    fn manifest_for_id(&self, module_id: &str) -> Result<IntegrationModuleManifest> {
        let normalized = normalize_module_id(module_id);
        self.module_manifests()?
            .into_iter()
            .find(|manifest| normalize_module_id(&manifest.id) == normalized)
            .ok_or_else(|| anyhow!("integration module '{module_id}' is not installed"))
    }

    fn cached_manifest_entries(&self) -> Result<Vec<ModuleManifestCacheEntry>> {
        if let Some(entries) = self.manifest_cache.get() {
            return Ok(entries.clone());
        }

        let entries = self.read_manifest_entries()?;
        let _ = self.manifest_cache.set(entries.clone());
        Ok(entries)
    }

    fn read_manifest_entries(&self) -> Result<Vec<ModuleManifestCacheEntry>> {
        let mut entries = Vec::new();
        for manifest_path in self.module_manifest_paths()? {
            let manifest_path_display = manifest_path.display().to_string();
            match self.read_manifest_at(&manifest_path) {
                Ok(Some(manifest)) => entries.push(ModuleManifestCacheEntry::Valid(manifest)),
                Ok(None) => entries.push(ModuleManifestCacheEntry::Empty),
                Err(error) => entries.push(ModuleManifestCacheEntry::Error {
                    manifest_path,
                    error: format!("{manifest_path_display}: {error}"),
                }),
            }
        }
        Ok(entries)
    }
}

#[derive(Clone, Debug)]
enum ModuleManifestCacheEntry {
    Valid(IntegrationModuleManifest),
    Empty,
    Error {
        manifest_path: PathBuf,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleManifest {
    pub schema: String,
    pub id: String,
    pub display_name: String,
    pub tooltip: String,
    pub icon_label: String,
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
    pub header_actions: Vec<IntegrationModuleActionManifest>,
    #[serde(default)]
    pub ui_schema: Vec<IntegrationUiEntityManifest>,
    pub transport: IntegrationModuleTransport,
    #[serde(default)]
    pub library_paths: BTreeMap<String, PathBuf>,
    #[serde(skip)]
    base_dir: PathBuf,
}

impl IntegrationModuleManifest {
    const SCHEMA: &'static str = "netstitch.integration.module.v1";

    fn to_dto(&self, enabled: bool, locale: &ModuleLocale) -> Result<IntegrationModuleDto> {
        let icon_path = self.icon_path.as_ref().map(|path| self.asset_path(path));
        let icon_data_uri = self.icon_data_uri(icon_path.as_deref())?;
        Ok(IntegrationModuleDto {
            id: self.id.clone(),
            display_name: localized_manifest_text(
                locale,
                self.display_name_key.as_deref(),
                &self.display_name,
            ),
            tooltip: localized_manifest_text(locale, self.tooltip_key.as_deref(), &self.tooltip),
            icon_label: self.icon_label.clone(),
            enabled,
            display_name_key: self.display_name_key.clone(),
            tooltip_key: self.tooltip_key.clone(),
            provider_title: localized_manifest_optional_text(
                locale,
                self.provider_title_key.as_deref(),
                self.provider_title.as_deref(),
            ),
            provider_title_key: self.provider_title_key.clone(),
            export_title: localized_manifest_optional_text(
                locale,
                self.export_title_key.as_deref(),
                self.export_title.as_deref(),
            ),
            export_title_key: self.export_title_key.clone(),
            button_color: self.button_color.clone(),
            icon_svg: self.icon_svg.clone(),
            icon_path,
            icon_data_uri,
            header_actions: self
                .header_actions
                .iter()
                .map(|action| action.to_dto(self, locale))
                .collect::<Result<Vec<_>>>()?,
            ui_schema: self
                .ui_schema
                .iter()
                .map(|entity| entity.to_dto(locale))
                .collect(),
            background_active: false,
            background_subscriptions: Vec::new(),
            background_status: None,
        })
    }

    fn library_path(&self) -> Result<PathBuf> {
        let platform_keys = current_platform_library_keys();
        let path = platform_keys
            .iter()
            .find_map(|key| self.library_paths.get(key.as_str()))
            .ok_or_else(|| {
                anyhow!(
                    "integration module '{}' has no native library path for {}",
                    self.id,
                    platform_keys[0]
                )
            })?;
        Ok(if path.is_absolute() {
            path.clone()
        } else {
            self.base_dir.join(path)
        })
    }

    fn storage_dir(&self) -> PathBuf {
        self.base_dir.join("data")
    }

    fn asset_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }

    fn icon_data_uri(&self, path: Option<&Path>) -> Result<Option<String>> {
        let Some(path) = path else {
            return Ok(None);
        };
        if !path.is_file() {
            return Ok(None);
        }
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let Some(mime) = module_asset_mime_type(&extension) else {
            return Ok(None);
        };
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read module icon {}", path.display()))?;
        Ok(Some(format!(
            "data:{mime};base64,{}",
            STANDARD.encode(bytes)
        )))
    }

    fn load_error(&self) -> Option<String> {
        match self.transport {
            IntegrationModuleTransport::NativeLibrary => match self.library_path() {
                Ok(path) if path.is_file() => None,
                Ok(path) => Some(format!("integration library not found: {}", path.display())),
                Err(error) => Some(error.to_string()),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationModuleActionManifest {
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
    #[serde(default = "default_true")]
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

impl IntegrationModuleActionManifest {
    fn to_dto(
        &self,
        manifest: &IntegrationModuleManifest,
        locale: &ModuleLocale,
    ) -> Result<IntegrationModuleActionDto> {
        let icon_path = self
            .icon_path
            .as_ref()
            .map(|path| manifest.asset_path(path));
        Ok(IntegrationModuleActionDto {
            id: self.id.clone(),
            label: localized_manifest_text(locale, self.label_key.as_deref(), &self.label),
            label_key: self.label_key.clone(),
            tooltip: localized_manifest_optional_text(
                locale,
                self.tooltip_key.as_deref(),
                self.tooltip.as_deref(),
            ),
            tooltip_key: self.tooltip_key.clone(),
            icon_svg: self.icon_svg.clone(),
            icon_path: icon_path.clone(),
            icon_data_uri: manifest.icon_data_uri(icon_path.as_deref())?,
            enabled: self.enabled,
            style: self.style.clone(),
            align: self.align.clone(),
            pulse: self.pulse,
            pulse_when_background_active: self.pulse_when_background_active,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiEntityManifest {
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
    pub options: Vec<IntegrationUiOptionManifest>,
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
    pub progress_stages: Vec<IntegrationUiProgressStageManifest>,
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
    pub children: Vec<IntegrationUiEntityManifest>,
    #[serde(default)]
    pub actions: Vec<IntegrationModuleActionManifest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiOptionManifest {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub label_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationUiProgressStageManifest {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub percent: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub name_key: Option<String>,
}

fn default_ui_opacity() -> String {
    "100%".to_string()
}

impl IntegrationUiEntityManifest {
    fn to_dto(&self, locale: &ModuleLocale) -> IntegrationUiEntityDto {
        IntegrationUiEntityDto {
            id: self.id.clone(),
            entity_type: self.entity_type.clone(),
            page: self.page.clone(),
            hidden: self.hidden,
            visible: self.visible,
            disabled: self.disabled,
            title: localized_manifest_optional_text(
                locale,
                self.title_key.as_deref(),
                self.title.as_deref(),
            ),
            title_key: self.title_key.clone(),
            tooltip: localized_manifest_optional_text(
                locale,
                self.tooltip_key.as_deref(),
                self.tooltip.as_deref(),
            ),
            tooltip_key: self.tooltip_key.clone(),
            value: localized_manifest_optional_text(
                locale,
                self.value_key.as_deref(),
                self.value.as_deref(),
            ),
            value_key: self.value_key.clone(),
            placeholder: localized_manifest_optional_text(
                locale,
                self.placeholder_key.as_deref(),
                self.placeholder.as_deref(),
            ),
            placeholder_key: self.placeholder_key.clone(),
            options: self
                .options
                .iter()
                .map(|option| IntegrationUiOptionDto {
                    value: option.value.clone(),
                    label: localized_manifest_text(
                        locale,
                        option.label_key.as_deref(),
                        &option.label,
                    ),
                    label_key: option.label_key.clone(),
                })
                .collect(),
            checked: self.checked,
            readonly: self.readonly,
            clear_button: self.clear_button,
            commit_on_enter: self.commit_on_enter,
            compact: self.compact,
            hide_label: self.hide_label,
            hide_host_back_button: self.hide_host_back_button,
            progress_stages: self
                .progress_stages
                .iter()
                .map(|stage| IntegrationUiProgressStageDto {
                    color: stage.color.clone(),
                    percent: stage.percent.clone(),
                    name: localized_manifest_optional_text(
                        locale,
                        stage.name_key.as_deref(),
                        stage.name.as_deref(),
                    ),
                    name_key: stage.name_key.clone(),
                })
                .collect(),
            scroll: self.scroll.clone(),
            size: self.size.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
            min_width: self.min_width.clone(),
            min_height: self.min_height.clone(),
            max_width: self.max_width.clone(),
            max_height: self.max_height.clone(),
            align: self.align.clone(),
            justify: self.justify.clone(),
            button_layout: self.button_layout.clone(),
            margin: self.margin.clone(),
            padding: self.padding.clone(),
            columns: self.columns.clone(),
            table_columns: self.table_columns.clone(),
            rows: self.rows.clone(),
            gap: self.gap.clone(),
            grid_column: self.grid_column.clone(),
            grid_row: self.grid_row.clone(),
            opacity: self.opacity.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.to_dto(locale))
                .collect(),
            actions: self
                .actions
                .iter()
                .map(|action| IntegrationModuleActionDto {
                    id: action.id.clone(),
                    label: localized_manifest_text(
                        locale,
                        action.label_key.as_deref(),
                        &action.label,
                    ),
                    label_key: action.label_key.clone(),
                    tooltip: localized_manifest_optional_text(
                        locale,
                        action.tooltip_key.as_deref(),
                        action.tooltip.as_deref(),
                    ),
                    tooltip_key: action.tooltip_key.clone(),
                    icon_svg: action.icon_svg.clone(),
                    icon_path: action.icon_path.clone(),
                    icon_data_uri: None,
                    enabled: action.enabled,
                    style: action.style.clone(),
                    align: action.align.clone(),
                    pulse: action.pulse,
                    pulse_when_background_active: action.pulse_when_background_active,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationModuleTransport {
    NativeLibrary,
}

struct CallbackUserData<'a> {
    handler: &'a mut dyn FnMut(IntegrationHostEvent),
}

unsafe extern "C" fn module_event_callback(ptr: *const u8, len: usize, user_data: *mut c_void) {
    if ptr.is_null() || user_data.is_null() {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(event) = serde_json::from_slice::<IntegrationHostEvent>(bytes) else {
        return;
    };
    let data = unsafe { &mut *(user_data as *mut CallbackUserData<'_>) };
    (data.handler)(event);
}

unsafe fn read_module_response(
    code: i32,
    output: IntegrationAbiBuffer,
    free: netstitch_shared::models::IntegrationAbiFree,
) -> Result<serde_json::Value> {
    if output.ptr.is_null() {
        return Err(anyhow!("integration module returned an empty ABI buffer"));
    }
    let bytes = unsafe { std::slice::from_raw_parts(output.ptr, output.len) }.to_vec();
    unsafe {
        free(output.ptr, output.len);
    }
    let response: IntegrationHostResponse =
        serde_json::from_slice(&bytes).context("failed to decode integration ABI response")?;
    if code != 0 || !response.ok {
        return Err(anyhow!(
            "{}",
            response
                .error
                .unwrap_or_else(|| format!("integration module failed with code {code}"))
        ));
    }
    Ok(response.result.unwrap_or(serde_json::Value::Null))
}

fn default_module_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(explicit) = std::env::var("NETSTITCH__INTEGRATIONS_DIR") {
        let explicit = PathBuf::from(explicit);
        if explicit.is_dir() {
            return vec![explicit];
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.join("integrations"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join("integrations"));
    }
    roots.dedup();
    roots
}

#[derive(Clone, Debug, Default)]
struct ModuleLocale {
    strings: std::collections::BTreeMap<String, String>,
}

impl ModuleLocale {
    fn load(base_dir: &Path, language_code: Option<&str>) -> Self {
        let mut locale = Self::default();
        let requested = language_code
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());
        locale.merge_ini(&base_dir.join("locales").join("en-en.ini"));
        if let Some(code) = requested {
            if code != "en-en" {
                locale.merge_ini(&base_dir.join("locales").join(format!("{code}.ini")));
            }
        }
        locale
    }

    fn merge_ini(&mut self, path: &Path) {
        let Ok(content) = fs::read_to_string(path) else {
            return;
        };
        let mut section = String::new();
        for raw_line in content.lines() {
            let line = raw_line.trim().trim_start_matches('\u{feff}').trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let Some((raw_key, raw_value)) = line.split_once('=') else {
                continue;
            };
            if section != "strings" {
                continue;
            }
            let key = raw_key.trim();
            let value = strip_locale_quotes(raw_value.trim()).replace("\\n", "\n");
            self.strings.insert(key.to_string(), value);
        }
    }

    fn text(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(String::as_str)
    }
}

fn localized_manifest_text(locale: &ModuleLocale, key: Option<&str>, fallback: &str) -> String {
    key.and_then(|key| locale.text(key))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn localized_manifest_optional_text(
    locale: &ModuleLocale,
    key: Option<&str>,
    fallback: Option<&str>,
) -> Option<String> {
    key.and_then(|key| locale.text(key))
        .filter(|value| !value.trim().is_empty())
        .or(fallback)
        .map(str::to_string)
}

fn strip_locale_quotes(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn module_asset_mime_type(extension: &str) -> Option<&'static str> {
    match extension {
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "ico" => Some("image/x-icon"),
        _ => None,
    }
}

fn ensure_module_database(storage_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(storage_dir)
        .with_context(|| format!("failed to create module storage {}", storage_dir.display()))?;
    let database_path = storage_dir.join("module.sqlite3");
    let conn = rusqlite::Connection::open(&database_path)
        .with_context(|| format!("failed to open {}", database_path.display()))?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS module_settings (
            setting_key TEXT PRIMARY KEY,
            setting_value TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );
        "#,
    )
    .with_context(|| format!("failed to initialize {}", database_path.display()))?;
    Ok(database_path)
}

fn normalize_path_for_boundary(path: &Path) -> Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) => {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    return Ok(fs::canonicalize(parent)?
                        .join(path.file_name().ok_or_else(|| {
                            anyhow!("path has no file name: {}", path.display())
                        })?));
                }
            }
            Ok(path.to_path_buf())
        }
    }
}

fn default_true() -> bool {
    true
}

fn normalize_module_id(module_id: &str) -> String {
    module_id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect::<String>()
        .to_ascii_lowercase()
}

fn module_source_name_is_reserved(value: &str) -> bool {
    let normalized = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(normalized.as_str(), "core" | "system")
}

fn current_platform_library_keys() -> [String; 3] {
    [
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        std::env::consts::OS.to_string(),
        "default".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{IntegrationService, default_module_roots};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn explicit_integrations_dir_replaces_default_module_roots() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let root = unique_temp_dir("host-explicit-roots");
        fs::remove_dir_all(&root).ok();
        let explicit_root = root.join("user-integrations");
        fs::create_dir_all(&explicit_root).expect("explicit integrations root");

        let previous = std::env::var_os("NETSTITCH__INTEGRATIONS_DIR");
        unsafe {
            std::env::set_var("NETSTITCH__INTEGRATIONS_DIR", &explicit_root);
        }
        let roots = default_module_roots();
        match previous {
            Some(value) => unsafe {
                std::env::set_var("NETSTITCH__INTEGRATIONS_DIR", value);
            },
            None => unsafe {
                std::env::remove_var("NETSTITCH__INTEGRATIONS_DIR");
            },
        }

        assert_eq!(
            roots,
            vec![explicit_root],
            "explicit integration roots from installed packages must not be duplicated with /opt/cwd defaults"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_storage_dir_uses_manifest_data_directory() {
        let root = unique_temp_dir("host-module-storage");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let module_root = integrations_root.join("example");
        fs::create_dir_all(module_root.join("bin")).expect("module bin dir");
        fs::write(
            module_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "example",
              "display_name": "Example",
              "tooltip": "Example integration module",
              "icon_label": "E",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/example.dll"
              }
            }"#,
        )
        .expect("module manifest");

        let core_storage_root = root.join("storage").join("integrations");
        let service =
            IntegrationService::with_module_roots(core_storage_root.clone(), [integrations_root]);

        assert_eq!(
            service.module_storage_dir("example"),
            module_root.join("data"),
            "module runtime storage must be portable under integrations/<module>/data"
        );
        assert_ne!(
            service.module_storage_dir("example"),
            core_storage_root.join("example"),
            "module storage must not fall back into core storage when the manifest exists"
        );
        assert_eq!(
            service
                .ensure_module_storage("example")
                .expect("module storage should initialize"),
            module_root.join("data").join("module.sqlite3")
        );
        assert!(
            module_root.join("data").join("module.sqlite3").is_file(),
            "host should initialize the module-local SQLite database under integrations/<module>/data"
        );
        assert!(
            !core_storage_root
                .join("example")
                .join("module.sqlite3")
                .exists(),
            "host must not initialize module SQLite under core storage"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_runtime_statuses_report_manifest_errors_without_blocking_valid_modules() {
        let root = unique_temp_dir("host-module-diagnostics");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let good_root = integrations_root.join("good");
        let bad_root = integrations_root.join("bad");
        fs::create_dir_all(good_root.join("bin")).expect("good module bin dir");
        fs::create_dir_all(&bad_root).expect("bad module dir");
        fs::write(
            good_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "good",
              "display_name": "Good",
              "tooltip": "Good integration module",
              "icon_label": "G",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/missing.dll"
              }
            }"#,
        )
        .expect("good module manifest");
        fs::write(bad_root.join("module.json"), "{ not-json").expect("bad module manifest");

        let service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let modules = service
            .available_modules()
            .expect("valid modules should still be discoverable");
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].id, "good");
        assert!(!modules[0].enabled);

        let statuses = service
            .module_runtime_statuses_for_language(None)
            .expect("module diagnostics should load");
        assert_eq!(statuses.len(), 2);
        assert!(statuses.iter().any(|status| {
            status.id == "good"
                && !status.connected
                && status
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("integration library not found"))
        }));
        assert!(statuses.iter().any(|status| {
            status.id == "bad"
                && !status.connected
                && status
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("failed to parse"))
        }));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_manifest_rejects_reserved_source_names_case_insensitively() {
        let root = unique_temp_dir("host-module-reserved-source");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let module_root = integrations_root.join("reserved");
        fs::create_dir_all(module_root.join("bin")).expect("module bin dir");
        fs::write(
            module_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "CoRe",
              "display_name": "CoRe",
              "tooltip": "Bad integration module",
              "icon_label": "C",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/missing.dll"
              }
            }"#,
        )
        .expect("module manifest");

        let service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let statuses = service
            .module_runtime_statuses_for_language(None)
            .expect("module diagnostics should report reserved names");
        assert_eq!(statuses.len(), 1);
        assert!(
            statuses[0]
                .error
                .as_deref()
                .is_some_and(|error| error.contains("reserved source name")),
            "reserved core/system module names must be rejected"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_runtime_statuses_do_not_execute_native_libraries_during_bootstrap() {
        let root = unique_temp_dir("host-module-bootstrap-no-call");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let module_root = integrations_root.join("generic");
        fs::create_dir_all(module_root.join("bin")).expect("module bin dir");
        fs::write(module_root.join("bin").join("generic.dll"), []).expect("library marker");
        fs::write(
            module_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "generic",
              "display_name": "Generic",
              "tooltip": "Generic integration module",
              "icon_label": "G",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/generic.dll"
              }
            }"#,
        )
        .expect("module manifest");

        let service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let statuses = service
            .module_runtime_statuses_for_language(None)
            .expect("module diagnostics should not execute native library code");
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].id, "generic");
        assert!(statuses[0].connected);
        assert!(
            statuses[0].error.is_none(),
            "bootstrap diagnostics must only validate the manifest and library path"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_manifest_localizes_runtime_strings_from_locale_files() {
        let root = unique_temp_dir("host-module-locales");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let module_root = integrations_root.join("localized");
        fs::create_dir_all(module_root.join("bin")).expect("module bin dir");
        fs::create_dir_all(module_root.join("locales")).expect("module locales dir");
        fs::write(module_root.join("bin").join("localized.dll"), []).expect("library marker");
        fs::write(
            module_root.join("locales").join("en-en.ini"),
            r#"[strings]
sample_module.module.display_name=Localized sample
sample_module.module.tooltip=English tooltip
sample_module.action.notice.label=Notice
sample_module.entity.help.value=English help text
sample_module.entity.note.title=Note
sample_module.entity.note.placeholder=Optional text
sample_module.option.fast=Fast
"#,
        )
        .expect("en locale");
        fs::write(
            module_root.join("locales").join("ru-ru.ini"),
            r#"[strings]
sample_module.module.display_name=Локализованный пример
sample_module.module.tooltip=Русская подсказка
sample_module.action.notice.label=Заметка
sample_module.entity.help.value=Русский текст помощи
sample_module.entity.note.title=Заметка
sample_module.entity.note.placeholder=Необязательный текст
sample_module.option.fast=Быстро
"#,
        )
        .expect("ru locale");
        fs::write(
            module_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "localized",
              "display_name": "Fallback sample",
              "display_name_key": "sample_module.module.display_name",
              "tooltip": "Fallback tooltip",
              "tooltip_key": "sample_module.module.tooltip",
              "icon_label": "L",
              "header_actions": [
                {
                  "id": "notice",
                  "label": "Fallback notice",
                  "label_key": "sample_module.action.notice.label",
                  "enabled": true
                }
              ],
              "ui_schema": [
                {
                  "id": "help",
                  "entity_type": "help_text",
                  "value": "Fallback help",
                  "value_key": "sample_module.entity.help.value"
                },
                {
                  "id": "note",
                  "entity_type": "text_input",
                  "title": "Fallback title",
                  "title_key": "sample_module.entity.note.title",
                  "placeholder": "Fallback placeholder",
                  "placeholder_key": "sample_module.entity.note.placeholder"
                },
                {
                  "id": "mode",
                  "entity_type": "select",
                  "options": [
                    {
                      "value": "fast",
                      "label": "Fallback fast",
                      "label_key": "sample_module.option.fast"
                    }
                  ]
                }
              ],
              "transport": "native_library",
              "library_paths": {
                "default": "bin/localized.dll"
              }
            }"#,
        )
        .expect("module manifest");

        let service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let modules = service
            .available_modules_for_language(Some("ru-ru"))
            .expect("localized module should load");

        assert_eq!(modules.len(), 1);
        let module = &modules[0];
        assert_eq!(module.display_name, "Локализованный пример");
        assert_eq!(module.tooltip, "Русская подсказка");
        assert_eq!(module.header_actions[0].label, "Заметка");
        assert_eq!(
            module.ui_schema[0].value.as_deref(),
            Some("Русский текст помощи")
        );
        assert_eq!(module.ui_schema[1].title.as_deref(), Some("Заметка"));
        assert_eq!(
            module.ui_schema[1].placeholder.as_deref(),
            Some("Необязательный текст")
        );
        assert_eq!(module.ui_schema[2].options[0].label, "Быстро");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_discovery_is_cached_for_process_lifetime() {
        let root = unique_temp_dir("host-module-startup-cache");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let first_root = integrations_root.join("first");
        let second_root = integrations_root.join("second");
        fs::create_dir_all(first_root.join("bin")).expect("first module bin dir");
        fs::write(
            first_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "first",
              "display_name": "First",
              "tooltip": "First module",
              "icon_label": "F",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/missing.dll"
              }
            }"#,
        )
        .expect("first module manifest");

        let service = IntegrationService::with_module_roots(
            root.join("storage"),
            [integrations_root.clone()],
        );
        let initial = service
            .available_modules()
            .expect("initial module discovery should load");
        assert_eq!(initial.len(), 1);
        assert_eq!(initial[0].id, "first");

        fs::create_dir_all(second_root.join("bin")).expect("second module bin dir");
        fs::write(
            second_root.join("module.json"),
            r#"{
              "schema": "netstitch.integration.module.v1",
              "id": "second",
              "display_name": "Second",
              "tooltip": "Second module",
              "icon_label": "S",
              "transport": "native_library",
              "library_paths": {
                "default": "bin/missing.dll"
              }
            }"#,
        )
        .expect("second module manifest");

        let cached = service
            .available_modules()
            .expect("cached module discovery should load");
        assert_eq!(
            cached.len(),
            1,
            "running host should not rescan integration folders on UI refresh"
        );
        assert_eq!(cached[0].id, "first");

        let restarted_service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let after_restart = restarted_service
            .available_modules()
            .expect("fresh host should discover current module folder contents");
        assert_eq!(after_restart.len(), 2);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn module_manifest_supports_platform_specific_native_library_paths() {
        let root = unique_temp_dir("host-platform-library-paths");
        fs::remove_dir_all(&root).ok();
        let integrations_root = root.join("integrations");
        let module_root = integrations_root.join("native");
        let platform_key = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let library_relative = PathBuf::from("bin")
            .join(&platform_key)
            .join(if cfg!(windows) {
                "native.dll"
            } else if cfg!(target_os = "macos") {
                "libnative.dylib"
            } else {
                "libnative.so"
            });
        fs::create_dir_all(module_root.join(library_relative.parent().expect("library parent")))
            .expect("platform bin dir");
        fs::write(module_root.join(&library_relative), []).expect("platform library marker");
        let manifest = format!(
            r#"{{
              "schema": "netstitch.integration.module.v1",
              "id": "native",
              "display_name": "Native",
              "tooltip": "Native integration module",
              "icon_label": "N",
              "transport": "native_library",
              "library_paths": {{
                "{platform_key}": "{}"
              }}
            }}"#,
            library_relative.to_string_lossy().replace('\\', "\\\\")
        );
        fs::write(module_root.join("module.json"), manifest).expect("module manifest");

        let service =
            IntegrationService::with_module_roots(root.join("storage"), [integrations_root]);
        let manifest = service
            .manifest_for_id("native")
            .expect("manifest should load");
        assert_eq!(
            manifest.library_path().expect("platform library path"),
            module_root.join(library_relative)
        );

        fs::remove_dir_all(root).ok();
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("netstitch-integrations-{prefix}-{suffix}"))
    }
}
