use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::net::IpAddr;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::app_watcher::read_persisted_cloud_session;
use crate::watcher_api::{
    ConnectionStateDto, ObservationDto, ProtocolDto, SnapshotResponse, TrackedAppDto,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::TimeZone;
use netstitch_cloud::{
    CloudClientIdentifierInput, CloudGoogleAuthPollRequest, CloudGoogleAuthStartRequest,
    CloudGoogleAuthStartResponse, CloudObservationAppendRequest, CloudObservationRow,
    CloudOperationKind, CloudQuotaSnapshot, CloudSignedJwtInput, CloudSourceKind, CloudTrustLevel,
    CloudUserSessionResponse, VerifiedAppMethod, VerifiedAppProof, VerifiedAppStatus,
    derive_client_identifier, operation_scope, sign_request_jwt,
};
use netstitch_shared::{
    CloudObservationMatchCandidate, CloudObservationMatchItem, CloudObservationMatchRequest,
    CloudObservationMatchResponse, CloudObservationVisibility, CloudObservationVisibilityScope,
    StableAppIdentityInput, stable_app_identity,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CLOUD_BASE_URL_ENV: &str = "NETSTITCH__CLOUD_BASE_URL";
const DEFAULT_CLOUD_BASE_URL: &str = "https://netstitch-sync.warfactory.workers.dev";
const CLIENT_IDENTIFIER_SALT: &[u8] = b"netstitch-community-cloud-v1";
const CLOUD_REFRESH_HTTP_TIMEOUT: Duration = Duration::from_millis(9500);
const CLOUD_INTERACTIVE_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const RESERVED_AUTHOR_SIGNATURES: &[&str] = &[
    "admin",
    "administrator",
    "anon",
    "anonymous",
    "cloud",
    "cloudflare",
    "google",
    "mod",
    "moderator",
    "netstitch",
    "netstitchcloud",
    "netstitchsupport",
    "official",
    "owner",
    "root",
    "support",
    "system",
];

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CloudSyncUiState {
    pub client_identifier: String,
    pub service_available: Option<bool>,
    pub service_message: String,
    pub upload_quota: Option<CloudQuotaSnapshot>,
    pub download_quota: Option<CloudQuotaSnapshot>,
    pub apps: Vec<CloudCatalogApp>,
    pub my_apps: Vec<CloudUserAppSummary>,
    pub tags: Vec<CloudTagSummary>,
    pub own_tag_count: usize,
    pub tag_limit: usize,
    pub downloaded_rows: Vec<CloudDownloadedObservation>,
    pub selected_download_row_ids: BTreeSet<String>,
    pub uploaded_observation_ids: BTreeSet<u64>,
    pub matched_observations: BTreeMap<u64, CloudObservationMatchItem>,
    pub session: Option<CloudUserSessionResponse>,
    pub client_private_key_pkcs8_der: Option<Vec<u8>>,
    pub auth_generation: u64,
    pub refresh_generation: u64,
    pub refresh_in_progress: bool,
    pub last_error: Option<String>,
    pub last_response_json: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct CloudTagSummary {
    pub tag: String,
    #[serde(default)]
    pub user_count: u64,
    #[serde(default)]
    pub is_own: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct CloudCatalogApp {
    pub app_id: String,
    pub display_name: String,
    pub publisher_name: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub endpoint_count: u64,
    #[serde(default)]
    pub available_row_count: u64,
    #[serde(default)]
    pub author_count: u64,
    #[serde(default)]
    pub last_seen_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct CloudUserAppSummary {
    pub app_id: String,
    pub display_name: String,
    #[serde(default)]
    pub publisher_name: Option<String>,
    #[serde(default)]
    pub author_signature: Option<String>,
    #[serde(default)]
    pub visibility: CloudObservationVisibility,
    #[serde(default)]
    pub endpoint_count: u64,
    #[serde(default)]
    pub total_endpoint_count: u64,
    #[serde(default)]
    pub available_row_count: u64,
    #[serde(default)]
    pub tag_groups: Vec<CloudUserTagGroupSummary>,
    #[serde(default)]
    pub last_seen_ms: Option<u64>,
    #[serde(default)]
    pub last_uploaded_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub(crate) struct CloudUserTagGroupSummary {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub endpoint_count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CloudNicknameCheckStatus {
    #[default]
    Idle,
    Checking,
    Invalid,
    Taken,
    Accepted,
    Error,
}

impl CloudNicknameCheckStatus {
    pub(crate) fn blocks_upload(self) -> bool {
        matches!(
            self,
            CloudNicknameCheckStatus::Checking
                | CloudNicknameCheckStatus::Invalid
                | CloudNicknameCheckStatus::Taken
        )
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct CloudNicknameCheckResponse {
    status: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CloudSearchFilters {
    pub app_query: String,
    pub publisher_query: String,
    pub selected_app_id: Option<String>,
    pub remote_ip_query: String,
    pub remote_domain_query: String,
    pub remote_port_query: String,
    pub protocol: String,
    pub source_query: String,
    pub own_scope: bool,
    pub visibility_scope: CloudObservationVisibilityScope,
}

impl CloudSearchFilters {
    fn effective_text(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.chars().count() < 2 {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn effective_protocol(&self) -> Option<&str> {
        let protocol = self.protocol.trim();
        if protocol.is_empty() || protocol.eq_ignore_ascii_case("all") {
            None
        } else {
            Some(protocol)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct CloudDownloadedObservation {
    pub row_id: String,
    pub app_display_name: String,
    pub privacy_label: String,
    pub source_label: String,
    pub protocol_label: String,
    pub connection_label: String,
    pub requests_label: String,
    pub first_seen_label: String,
    pub last_seen_label: String,
    pub domain_label: String,
    pub row: CloudObservationRow,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct HealthResponse {
    ok: bool,
    service: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct AppsResponse {
    items: Vec<CloudCatalogApp>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct UserAppsResponse {
    items: Vec<CloudUserAppSummary>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    items: Vec<CloudTagSummary>,
    #[serde(default)]
    own_count: usize,
    #[serde(default = "default_cloud_tag_limit")]
    limit: usize,
}

fn default_cloud_tag_limit() -> usize {
    netstitch_shared::CLOUD_TAGS_PER_USER_LIMIT
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct QuotaResponse {
    upload: CloudQuotaSnapshot,
    download: CloudQuotaSnapshot,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct CloudObservationDownloadResponse {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    limit: u64,
    #[serde(default)]
    offset: u64,
    rows: Vec<CloudObservationDownloadRow>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct CloudObservationDownloadRow {
    #[serde(default)]
    visibility: Option<CloudObservationVisibility>,
    #[serde(flatten)]
    row: CloudObservationRow,
}

#[derive(Clone, Debug, PartialEq)]
struct CollectedCloudObservation {
    is_private: bool,
    row: CloudObservationRow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CloudUploadSummary {
    pub accepted_rows: usize,
    pub request_count: usize,
    pub skipped_non_public_rows: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CloudUploadAppPreview {
    pub app_id: String,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UploadGroupKey {
    app_id: String,
    source_kind: CloudSourceKind,
    signature_key: Option<String>,
}

struct UploadGroup {
    app_proof: VerifiedAppProof,
    rows: Vec<CloudObservationRow>,
}

struct UploadBuildResult {
    groups: BTreeMap<UploadGroupKey, UploadGroup>,
    skipped_non_public_rows: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthenticodeProbe {
    status: String,
    leaf_spki_sha256: Option<String>,
    issuer: Option<String>,
    company_name: Option<String>,
    product_name: Option<String>,
}

pub(crate) fn default_cloud_base_url() -> String {
    env::var(CLOUD_BASE_URL_ENV)
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CLOUD_BASE_URL.to_string())
}

pub(crate) fn upload_confirmed_observations(
    state: &mut CloudSyncUiState,
    snapshot: &SnapshotResponse,
    client_version: &str,
    author_signature: &str,
    visibility: CloudObservationVisibility,
) -> Result<CloudUploadSummary, String> {
    let author_signature = validate_author_signature(author_signature)?;
    let client_identifier = ensure_identifier(state)?;
    if state
        .upload_quota
        .as_ref()
        .is_some_and(netstitch_cloud::quota_is_exhausted)
    {
        return Err("{\"error\":{\"code\":\"quota_exhausted\",\"message\":\"Cloud upload quota is exhausted\"}}".to_string());
    }
    let session = state
        .session
        .as_ref()
        .ok_or_else(|| "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud upload requires Google sign-in\"}}".to_string())?
        .clone();
    let private_key = state
        .client_private_key_pkcs8_der
        .as_deref()
        .ok_or_else(|| "{\"error\":{\"code\":\"missing_client_key\",\"message\":\"Cloud client key is missing\"}}".to_string())?
        .to_vec();

    let mut upload_build = build_upload_groups(snapshot)?;
    if upload_build.groups.is_empty() {
        return Err("{\"error\":{\"code\":\"nothing_to_upload\",\"message\":\"No confirmed rows with cloud application identifiers\"}}".to_string());
    }

    let client = http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let base_url = default_cloud_base_url();
    let upload_operation_id = cloud_upload_operation_id(&client_identifier);
    let mut accepted_rows = 0usize;
    let mut responses = Vec::<Value>::new();

    for group in upload_build.groups.values_mut() {
        let body = CloudObservationAppendRequest {
            client_identifier: client_identifier.clone(),
            client_version: client_version.to_string(),
            operation_id: Some(upload_operation_id.clone()),
            author_signature: author_signature.clone(),
            visibility,
            app_proof: group.app_proof.clone(),
            rows: std::mem::take(&mut group.rows),
        };
        let body_json = serde_json::to_vec(&body)
            .map_err(|error| format!("cloud upload body serialization failed: {error}"))?;
        let jwt = sign_request_jwt(CloudSignedJwtInput {
            key_id: &session.key_id,
            private_key_pkcs8_der: &private_key,
            client_id: &session.client_id,
            scope: operation_scope(CloudOperationKind::Upload),
            body: &body_json,
            client_version,
            now_seconds: current_timestamp_ms() / 1000,
            ttl_seconds: 5 * 60,
        })
        .map_err(|error| format!("cloud upload JWT signing failed: {error}"))?;

        let response = client
            .post(format!("{base_url}/v1/observations/append"))
            .bearer_auth(&session.session_token)
            .header("x-netstitch-jwt", jwt)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .map_err(|error| format!("cloud upload failed: {error}"))?;
        let is_success = response.status().is_success();
        let response_text = response
            .text()
            .unwrap_or_else(|_| "cloud upload failed".to_string());
        let response_json = compact_json_text(&response_text);
        if !is_success {
            state.last_error = Some(response_json.clone());
            state.last_response_json = Some(response_json.clone());
            return Err(response_json);
        }
        let value = parse_json_value(&response_text);
        accepted_rows += value
            .get("accepted_rows")
            .and_then(|item| item.as_u64())
            .unwrap_or(0) as usize;
        responses.push(value);
    }

    refresh_cloud_state(state, &CloudSearchFilters::default())?;
    state.last_error = None;
    state.last_response_json = Some(compact_json_value(&json!({
        "upload": {
            "accepted_rows": accepted_rows,
            "request_count": upload_build.groups.len(),
            "skipped_non_public_rows": upload_build.skipped_non_public_rows,
            "visibility": match visibility {
                CloudObservationVisibility::Public => "public",
                CloudObservationVisibility::Private => "private",
            },
            "responses": responses,
        }
    })));
    Ok(CloudUploadSummary {
        accepted_rows,
        request_count: upload_build.groups.len(),
        skipped_non_public_rows: upload_build.skipped_non_public_rows,
    })
}

pub(crate) fn cloud_upload_app_preview_for_observation(
    snapshot: &SnapshotResponse,
    observation: &ObservationDto,
    local_apps: &mut BTreeMap<u64, Option<CloudUploadAppPreview>>,
) -> Option<CloudUploadAppPreview> {
    let apps_by_id = snapshot
        .tracked_apps
        .iter()
        .map(|app| (app.id, app))
        .collect::<BTreeMap<_, _>>();
    let app = upload_app_for_observation(observation, &apps_by_id, &snapshot.tracked_apps);
    let imported_app = imported_cloud_app_id_for_observation(observation, app).map(|app_id| {
        let display_name = connector_cloud_app_for_app(app)
            .filter(|known| known.app_id.eq_ignore_ascii_case(&app_id))
            .map(|known| known.display_name.to_string());
        CloudUploadAppPreview {
            app_id,
            display_name,
        }
    });
    let local_app = app.and_then(|app| {
        if !local_apps.contains_key(&app.id) {
            let connector_cloud_app = connector_cloud_app_for_app(Some(app));
            local_apps.insert(
                app.id,
                build_local_app_proof(
                    app,
                    connector_cloud_app.map(|known| known.app_id),
                    connector_cloud_app.map(|known| known.display_name),
                )
                .and_then(|proof| {
                    let app_id = proof.app_id.trim().to_string();
                    (!app_id.is_empty()).then(|| CloudUploadAppPreview {
                        app_id,
                        display_name: proof
                            .display_name
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty()),
                    })
                }),
            );
        }
        local_apps.get(&app.id).cloned().flatten()
    });

    local_app.or(imported_app)
}

fn build_upload_groups(snapshot: &SnapshotResponse) -> Result<UploadBuildResult, String> {
    let apps_by_id = snapshot
        .tracked_apps
        .iter()
        .map(|app| (app.id, app))
        .collect::<BTreeMap<_, _>>();
    let mut local_proofs = BTreeMap::<u64, Option<VerifiedAppProof>>::new();
    let mut groups = BTreeMap::<UploadGroupKey, UploadGroup>::new();
    let mut skipped_non_public_rows = 0usize;

    for observation in snapshot
        .observations
        .iter()
        .filter(|item| item.is_confirmed)
    {
        let remote_ip = observation
            .remote_ip
            .parse::<IpAddr>()
            .map_err(|error| format!("invalid observation IP for cloud upload: {error}"))?;
        if !netstitch_shared::cloud_observation_ip_is_public(remote_ip) {
            skipped_non_public_rows += 1;
            continue;
        }
        let app = upload_app_for_observation(observation, &apps_by_id, &snapshot.tracked_apps);
        let imported_app_id = imported_cloud_app_id_for_observation(observation, app);
        let connector_cloud_app = connector_cloud_app_for_app(app);

        let imported_signature = observation
            .app_signature_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let local_proof = if let Some(app) = app {
            if !local_proofs.contains_key(&app.id) {
                local_proofs.insert(
                    app.id,
                    build_local_app_proof(
                        app,
                        connector_cloud_app.map(|known| known.app_id),
                        connector_cloud_app.map(|known| known.display_name),
                    ),
                );
            }
            local_proofs.get(&app.id).cloned().flatten()
        } else {
            None
        };
        let local_signature = local_proof
            .as_ref()
            .and_then(|proof| proof.leaf_spki_sha256.clone());
        let verified_by_local = local_proof
            .as_ref()
            .is_some_and(|proof| proof.status == VerifiedAppStatus::Verified)
            && imported_signature
                .as_ref()
                .map(|imported| local_signature.as_deref() == Some(imported.as_str()))
                .unwrap_or(true);
        let app_id = if verified_by_local {
            local_proof
                .as_ref()
                .map(|proof| proof.app_id.trim().to_string())
                .filter(|value| !value.is_empty())
        } else {
            imported_app_id.or_else(|| {
                local_proof
                    .as_ref()
                    .map(|proof| proof.app_id.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
        };
        let Some(app_id) = app_id else {
            continue;
        };

        let source_kind = if verified_by_local {
            CloudSourceKind::VerifiedUpload
        } else {
            CloudSourceKind::CloudImportUntrusted
        };
        let signature_key = if verified_by_local {
            local_signature
        } else {
            imported_signature.clone()
        };
        let app_proof = if verified_by_local {
            local_proof.expect("verified_by_local requires local proof")
        } else {
            untrusted_import_proof(
                &app_id,
                signature_key.clone(),
                observation.app_signature_subject.clone(),
                observation.app_signature_issuer.clone(),
                local_proof
                    .as_ref()
                    .and_then(|proof| proof.display_name.clone()),
                local_proof
                    .as_ref()
                    .and_then(|proof| proof.publisher_name.clone()),
            )
        };
        let row = observation_to_cloud_row(observation, &app_id, source_kind)?;
        let key = UploadGroupKey {
            app_id: app_id.clone(),
            source_kind,
            signature_key,
        };
        groups
            .entry(key.clone())
            .or_insert_with(|| UploadGroup {
                app_proof,
                rows: Vec::new(),
            })
            .rows
            .push(row);
    }

    if groups.is_empty() && skipped_non_public_rows > 0 {
        return Err("{\"error\":{\"code\":\"nothing_to_upload\",\"message\":\"Only private, local, documentation or other non-public IP ranges were selected for cloud upload\"}}".to_string());
    }

    Ok(UploadBuildResult {
        groups,
        skipped_non_public_rows,
    })
}

fn imported_cloud_app_id_for_observation(
    observation: &ObservationDto,
    app: Option<&TrackedAppDto>,
) -> Option<String> {
    observation
        .cloud_app_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(normalize_cloud_app_id)
        .or_else(|| {
            app.and_then(|app| {
                app.cloud_app_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .and_then(normalize_cloud_app_id)
            })
        })
}

fn upload_app_for_observation<'a>(
    observation: &ObservationDto,
    apps_by_id: &BTreeMap<u64, &'a TrackedAppDto>,
    apps: &'a [TrackedAppDto],
) -> Option<&'a TrackedAppDto> {
    apps_by_id
        .get(&observation.tracked_app_id)
        .copied()
        .or_else(|| inferred_upload_app_for_orphan_observation(observation, apps))
}

fn inferred_upload_app_for_orphan_observation<'a>(
    observation: &ObservationDto,
    apps: &'a [TrackedAppDto],
) -> Option<&'a TrackedAppDto> {
    if observation.tracked_app_id != 0 {
        return None;
    }
    let observation_key = upload_app_match_key(&observation.process_name)?;
    let matches = apps
        .iter()
        .filter(|app| tracked_app_upload_match_keys(app).contains(&observation_key))
        .take(2)
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

fn tracked_app_upload_match_keys(app: &TrackedAppDto) -> BTreeSet<String> {
    [
        upload_app_match_key(&app.display_name),
        upload_app_match_key(&exe_file_stem(&app.exe_path).unwrap_or_default()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn upload_app_match_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let stem = trimmed
        .strip_suffix(".exe")
        .or_else(|| trimmed.strip_suffix(".EXE"))
        .unwrap_or(trimmed);
    let key = stem
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>();
    (!key.is_empty()).then_some(key)
}

#[cfg(test)]
fn connector_cloud_app_id_for_app(app: Option<&TrackedAppDto>) -> Option<String> {
    connector_cloud_app_for_app(app).map(|item| item.app_id.to_string())
}

fn connector_cloud_app_for_app(
    app: Option<&TrackedAppDto>,
) -> Option<netstitch_shared::CloudKnownApp> {
    app.and_then(|app| {
        app.connector_id
            .as_deref()
            .and_then(netstitch_shared::cloud_app_for_connector_id)
    })
}

fn normalize_cloud_app_id(value: &str) -> Option<String> {
    let normalized = value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn observation_to_cloud_row(
    observation: &ObservationDto,
    app_id: &str,
    source_kind: CloudSourceKind,
) -> Result<CloudObservationRow, String> {
    let remote_ip = observation
        .remote_ip
        .parse::<IpAddr>()
        .map_err(|error| format!("invalid observation IP for cloud upload: {error}"))?;
    let domain_raw = observation
        .enrichment
        .as_ref()
        .and_then(|enrichment| enrichment.domain_name.clone())
        .filter(|value| !value.trim().is_empty());

    Ok(CloudObservationRow {
        app_id: app_id.to_string(),
        author_signature: None,
        tags: observation.tags.clone(),
        remote_ip,
        remote_port: observation.remote_port,
        protocol: match observation.protocol {
            ProtocolDto::Tcp => netstitch_shared::Protocol::Tcp,
            ProtocolDto::Udp => netstitch_shared::Protocol::Udp,
            ProtocolDto::Other => netstitch_shared::Protocol::Other,
        },
        connection_state: match observation.connection_state {
            ConnectionStateDto::Unknown => netstitch_shared::ConnectionState::Unknown,
            ConnectionStateDto::Attempting => netstitch_shared::ConnectionState::Attempting,
            ConnectionStateDto::Established => netstitch_shared::ConnectionState::Established,
            ConnectionStateDto::Closing => netstitch_shared::ConnectionState::Closing,
            ConnectionStateDto::Failed => netstitch_shared::ConnectionState::Failed,
        },
        requests: u64::from(observation.hits),
        first_seen_ms: observation.first_seen_ms,
        last_seen_ms: observation.last_seen_ms.max(observation.first_seen_ms),
        failed_hits: u64::from(observation.failed_hits),
        successful_hits: u64::from(observation.successful_hits),
        domain_raw,
        domain_verified: None,
        domain_status: netstitch_shared::CloudDomainStatus::None,
        trust_level: if source_kind == CloudSourceKind::VerifiedUpload {
            CloudTrustLevel::VerifiedUploadCandidate
        } else {
            CloudTrustLevel::CloudImportUntrusted
        },
        source_kind,
        app_signature_key: observation.app_signature_key.clone(),
        app_signature_subject: observation.app_signature_subject.clone(),
        app_signature_issuer: observation.app_signature_issuer.clone(),
        cloud_observation_id: None,
    })
}

fn build_local_app_proof(
    app: &TrackedAppDto,
    preferred_app_id: Option<&str>,
    preferred_display_name: Option<&str>,
) -> Option<VerifiedAppProof> {
    let exe_path = app.exe_path.trim();
    if exe_path.is_empty() || exe_path.starts_with("netstitch-csv-import://") {
        return None;
    }
    let probe = probe_authenticode_signature(exe_path).ok()?;
    let stable_identity = stable_app_identity(StableAppIdentityInput {
        authenticode_leaf_spki_sha256: probe
            .status
            .eq_ignore_ascii_case("Valid")
            .then(|| probe.leaf_spki_sha256.clone())
            .flatten(),
        company_name: probe.company_name.clone(),
        product_name: probe.product_name.clone(),
    })?;
    let app_id = preferred_app_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| cloud_app_id_from_signature_key(&stable_identity.signature_key));
    let display_name = app_display_name(app, &probe, preferred_display_name);
    let short_app_id = short_app_id_from_display(&display_name, &stable_identity.signature_key);
    Some(VerifiedAppProof {
        app_id,
        platform: env::consts::OS.to_string(),
        method: VerifiedAppMethod::StableFileIdentity,
        status: VerifiedAppStatus::Verified,
        signature_chain_valid: true,
        leaf_spki_sha256: Some(stable_identity.signature_key),
        subject: Some(stable_identity.subject),
        issuer: probe.issuer,
        display_name: Some(display_name),
        publisher_name: probe.company_name.filter(|value| !value.trim().is_empty()),
        short_app_id: Some(short_app_id),
    })
}

fn untrusted_import_proof(
    app_id: &str,
    signature_key: Option<String>,
    subject: Option<String>,
    issuer: Option<String>,
    display_name: Option<String>,
    publisher_name: Option<String>,
) -> VerifiedAppProof {
    VerifiedAppProof {
        app_id: app_id.to_string(),
        platform: env::consts::OS.to_string(),
        method: VerifiedAppMethod::Authenticode,
        status: VerifiedAppStatus::Unknown,
        signature_chain_valid: false,
        leaf_spki_sha256: signature_key,
        subject,
        issuer,
        display_name,
        publisher_name,
        short_app_id: None,
    }
}

fn cloud_app_id_from_signature_key(signature_key: &str) -> String {
    let suffix = signature_key
        .trim()
        .strip_prefix("appsig_v1_")
        .unwrap_or(signature_key.trim());
    let slug: String = suffix
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .take(32)
        .collect::<String>()
        .to_ascii_lowercase();
    if slug.is_empty() {
        "netstitch.app.local.unknown".to_string()
    } else {
        format!("netstitch.app.local.{slug}")
    }
}

fn app_display_name(
    app: &TrackedAppDto,
    probe: &AuthenticodeProbe,
    preferred_display_name: Option<&str>,
) -> String {
    probe
        .product_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            let value = preferred_display_name.unwrap_or_default().trim();
            (!value.is_empty()).then(|| value.to_string())
        })
        .or_else(|| exe_file_stem(&app.exe_path))
        .unwrap_or_else(|| "Unknown application".to_string())
}

fn exe_file_stem(path: &str) -> Option<String> {
    let file_name = path
        .trim()
        .rsplit(['\\', '/'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
        .trim();
    (!stem.is_empty()).then(|| stem.to_string())
}

fn short_app_id_from_display(display_name: &str, signature_key: &str) -> String {
    let base: String = display_name
        .trim()
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '.') {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let suffix = signature_key
        .trim()
        .strip_prefix("appsig_v1_")
        .unwrap_or(signature_key.trim())
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase();
    match (base.is_empty(), suffix.is_empty()) {
        (false, false) => format!("{base}-{suffix}"),
        (false, true) => base,
        (true, false) => format!("app-{suffix}"),
        (true, true) => "app".to_string(),
    }
}

fn probe_authenticode_signature(exe_path: &str) -> Result<AuthenticodeProbe, String> {
    #[cfg(windows)]
    {
        native_windows_authenticode_probe(exe_path)
    }
    #[cfg(not(windows))]
    {
        let _ = exe_path;
        Err("Authenticode proof is only available on Windows".to_string())
    }
}

#[cfg(windows)]
#[derive(Default)]
struct WindowsVersionInfo {
    company_name: Option<String>,
    product_name: Option<String>,
}

#[cfg(windows)]
fn native_windows_authenticode_probe(exe_path: &str) -> Result<AuthenticodeProbe, String> {
    let version = windows_file_version_info(exe_path).unwrap_or_default();
    let signature =
        windows_authenticode_signature(exe_path).unwrap_or_else(|_| WindowsSignatureInfo {
            status: "NotSigned".to_string(),
            leaf_spki_sha256: None,
            issuer: None,
        });

    Ok(AuthenticodeProbe {
        status: signature.status,
        leaf_spki_sha256: signature.leaf_spki_sha256,
        issuer: signature.issuer,
        company_name: version.company_name,
        product_name: version.product_name,
    })
}

#[cfg(windows)]
struct WindowsSignatureInfo {
    status: String,
    leaf_spki_sha256: Option<String>,
    issuer: Option<String>,
}

#[cfg(windows)]
fn windows_file_version_info(exe_path: &str) -> Result<WindowsVersionInfo, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW};

    let wide_path = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let mut handle = 0u32;
    let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
    if size == 0 {
        return Ok(WindowsVersionInfo::default());
    }
    let mut data = vec![0u8; size as usize];
    let ok = unsafe {
        GetFileVersionInfoW(
            wide_path.as_ptr(),
            0,
            size,
            data.as_mut_ptr().cast::<std::ffi::c_void>(),
        )
    };
    if ok == 0 {
        return Ok(WindowsVersionInfo::default());
    }

    let translations = version_translations(&data);
    let mut candidates = translations
        .into_iter()
        .chain(["040904b0".to_string(), "040904e4".to_string()]);
    for lang_codepage in candidates.by_ref() {
        let company_name = version_string(&data, &lang_codepage, "CompanyName");
        let product_name = version_string(&data, &lang_codepage, "ProductName");
        if company_name.is_some() || product_name.is_some() {
            return Ok(WindowsVersionInfo {
                company_name,
                product_name,
            });
        }
    }

    let _ = null_mut::<std::ffi::c_void>();
    Ok(WindowsVersionInfo::default())
}

#[cfg(windows)]
fn version_translations(data: &[u8]) -> Vec<String> {
    use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

    let mut buffer = std::ptr::null_mut::<std::ffi::c_void>();
    let mut len = 0u32;
    let sub_block = wide_null("\\VarFileInfo\\Translation");
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast::<std::ffi::c_void>(),
            sub_block.as_ptr(),
            &mut buffer,
            &mut len,
        )
    };
    if ok == 0 || buffer.is_null() || len < 4 {
        return Vec::new();
    }
    let words = unsafe { std::slice::from_raw_parts(buffer.cast::<u16>(), (len as usize) / 2) };
    words
        .chunks_exact(2)
        .map(|pair| format!("{:04x}{:04x}", pair[0], pair[1]))
        .collect()
}

#[cfg(windows)]
fn version_string(data: &[u8], lang_codepage: &str, key: &str) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

    let sub_block = wide_null(&format!("\\StringFileInfo\\{lang_codepage}\\{key}"));
    let mut buffer = std::ptr::null_mut::<std::ffi::c_void>();
    let mut len = 0u32;
    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast::<std::ffi::c_void>(),
            sub_block.as_ptr(),
            &mut buffer,
            &mut len,
        )
    };
    if ok == 0 || buffer.is_null() || len == 0 {
        return None;
    }
    let raw = unsafe { std::slice::from_raw_parts(buffer.cast::<u16>(), len as usize) };
    let end = raw
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(raw.len());
    let value = String::from_utf16_lossy(&raw[..end]).trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(windows)]
fn windows_authenticode_signature(exe_path: &str) -> Result<WindowsSignatureInfo, String> {
    let wide_path = wide_null(exe_path);
    let valid = verify_wintrust_file(&wide_path);
    let (leaf_spki_sha256, issuer) = signer_certificate_identity(&wide_path)
        .map(|identity| (Some(identity.leaf_spki_sha256), identity.issuer))
        .unwrap_or((None, None));

    Ok(WindowsSignatureInfo {
        status: if valid { "Valid" } else { "NotTrusted" }.to_string(),
        leaf_spki_sha256,
        issuer,
    })
}

#[cfg(windows)]
struct SignerCertificateIdentity {
    leaf_spki_sha256: String,
    issuer: Option<String>,
}

#[cfg(windows)]
fn verify_wintrust_file(wide_path: &[u16]) -> bool {
    use std::mem::size_of;
    use std::ptr::null_mut;
    use windows_sys::Win32::Security::WinTrust::{
        WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO,
        WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE, WTD_REVOCATION_CHECK_NONE, WTD_REVOKE_NONE,
        WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE, WTD_UICONTEXT_EXECUTE,
        WinVerifyTrustEx,
    };

    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: wide_path.as_ptr(),
        hFile: null_mut(),
        pgKnownSubject: null_mut(),
    };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        pPolicyCallbackData: null_mut(),
        pSIPClientData: null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &mut file_info,
        },
        dwStateAction: WTD_STATEACTION_VERIFY,
        hWVTStateData: null_mut(),
        pwszURLReference: null_mut(),
        dwProvFlags: WTD_REVOCATION_CHECK_NONE | WTD_CACHE_ONLY_URL_RETRIEVAL,
        dwUIContext: WTD_UICONTEXT_EXECUTE,
        pSignatureSettings: null_mut(),
    };
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    let status = unsafe { WinVerifyTrustEx(null_mut(), &mut action, &mut data) };
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    let _ = unsafe { WinVerifyTrustEx(null_mut(), &mut action, &mut data) };
    status == 0
}

#[cfg(windows)]
fn signer_certificate_identity(wide_path: &[u16]) -> Result<SignerCertificateIdentity, String> {
    use std::mem::zeroed;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Security::Cryptography::{
        CERT_FIND_SUBJECT_CERT, CERT_INFO, CERT_NAME_ISSUER_FLAG, CERT_NAME_SIMPLE_DISPLAY_TYPE,
        CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED, CERT_QUERY_FORMAT_FLAG_BINARY,
        CERT_QUERY_OBJECT_FILE, CMSG_SIGNER_INFO, CMSG_SIGNER_INFO_PARAM, CertCloseStore,
        CertFindCertificateInStore, CertFreeCertificateContext, CertGetNameStringW, CryptMsgClose,
        CryptMsgGetParam, CryptQueryObject, HCERTSTORE, PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
    };

    let mut encoding = 0u32;
    let mut content_type = 0u32;
    let mut format_type = 0u32;
    let mut store: HCERTSTORE = null_mut();
    let mut message = null_mut::<std::ffi::c_void>();
    let ok = unsafe {
        CryptQueryObject(
            CERT_QUERY_OBJECT_FILE,
            wide_path.as_ptr().cast::<std::ffi::c_void>(),
            CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
            CERT_QUERY_FORMAT_FLAG_BINARY,
            0,
            &mut encoding,
            &mut content_type,
            &mut format_type,
            &mut store,
            &mut message,
            null_mut(),
        )
    };
    if ok == 0 || store.is_null() || message.is_null() {
        return Err("embedded signer certificate was not found".to_string());
    }

    let mut signer_size = 0u32;
    let ok = unsafe {
        CryptMsgGetParam(
            message,
            CMSG_SIGNER_INFO_PARAM,
            0,
            null_mut(),
            &mut signer_size,
        )
    };
    if ok == 0 || signer_size == 0 {
        unsafe {
            CryptMsgClose(message);
            CertCloseStore(store, 0);
        }
        return Err("signer info was not found".to_string());
    }

    let mut signer_bytes = vec![0u8; signer_size as usize];
    let ok = unsafe {
        CryptMsgGetParam(
            message,
            CMSG_SIGNER_INFO_PARAM,
            0,
            signer_bytes.as_mut_ptr().cast::<std::ffi::c_void>(),
            &mut signer_size,
        )
    };
    if ok == 0 {
        unsafe {
            CryptMsgClose(message);
            CertCloseStore(store, 0);
        }
        return Err("signer info could not be read".to_string());
    }

    let signer = unsafe { &*(signer_bytes.as_ptr().cast::<CMSG_SIGNER_INFO>()) };
    let mut cert_info: CERT_INFO = unsafe { zeroed() };
    cert_info.Issuer = signer.Issuer;
    cert_info.SerialNumber = signer.SerialNumber;
    let cert = unsafe {
        CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            0,
            CERT_FIND_SUBJECT_CERT,
            (&cert_info as *const CERT_INFO).cast::<std::ffi::c_void>(),
            null(),
        )
    };
    if cert.is_null() {
        unsafe {
            CryptMsgClose(message);
            CertCloseStore(store, 0);
        }
        return Err("signer certificate was not found in store".to_string());
    }

    let cert_info = unsafe { (*cert).pCertInfo };
    let public_key = unsafe { (*cert_info).SubjectPublicKeyInfo.PublicKey };
    let public_key_bytes = if public_key.pbData.is_null() || public_key.cbData == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(public_key.pbData, public_key.cbData as usize) }
    };
    let leaf_spki_sha256 = URL_SAFE_NO_PAD.encode(Sha256::digest(public_key_bytes));

    let issuer = cert_name_string(
        cert,
        CERT_NAME_SIMPLE_DISPLAY_TYPE,
        CERT_NAME_ISSUER_FLAG,
        CertGetNameStringW,
    );

    unsafe {
        CertFreeCertificateContext(cert);
        CryptMsgClose(message);
        CertCloseStore(store, 0);
    }

    Ok(SignerCertificateIdentity {
        leaf_spki_sha256,
        issuer,
    })
}

#[cfg(windows)]
fn cert_name_string(
    cert: *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT,
    name_type: u32,
    flags: u32,
    getter: unsafe extern "system" fn(
        *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT,
        u32,
        u32,
        *const std::ffi::c_void,
        windows_sys::core::PWSTR,
        u32,
    ) -> u32,
) -> Option<String> {
    let len = unsafe {
        getter(
            cert,
            name_type,
            flags,
            std::ptr::null(),
            std::ptr::null_mut(),
            0,
        )
    };
    if len <= 1 {
        return None;
    }
    let mut buffer = vec![0u16; len as usize];
    let written = unsafe {
        getter(
            cert,
            name_type,
            flags,
            std::ptr::null(),
            buffer.as_mut_ptr(),
            len,
        )
    };
    if written <= 1 {
        return None;
    }
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    let value = String::from_utf16_lossy(&buffer[..end]).trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub(crate) fn local_client_identifier() -> Result<String, String> {
    let mut machine_signals = Vec::new();
    for key in [
        "COMPUTERNAME",
        "PROCESSOR_IDENTIFIER",
        "PROCESSOR_ARCHITECTURE",
        "USERDOMAIN",
    ] {
        if let Ok(value) = env::var(key) {
            machine_signals.push(format!("{key}={value}"));
        }
    }
    if machine_signals.is_empty() {
        machine_signals.push("machine=local".to_string());
    }

    derive_client_identifier(
        &CloudClientIdentifierInput {
            os_family: env::consts::OS.to_string(),
            os_version: env::var("OS").unwrap_or_else(|_| "unknown".to_string()),
            machine_signals,
        },
        CLIENT_IDENTIFIER_SALT,
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn refresh_cloud_state(
    state: &mut CloudSyncUiState,
    filters: &CloudSearchFilters,
) -> Result<(), String> {
    let client_identifier = ensure_identifier(state)?;
    let client = http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let base_url = default_cloud_base_url();

    let (health, health_json) =
        get_json::<HealthResponse>(&client, &format!("{base_url}/v1/health"), None)?;
    state.service_available = Some(health.ok);
    state.service_message = health.service;

    let quota_url = format!(
        "{base_url}/v1/client/quota?client_identifier={}",
        url_component(&client_identifier)
    );
    let (quota, quota_json) = get_json::<QuotaResponse>(&client, &quota_url, None)?;
    state.upload_quota = Some(quota.upload);
    state.download_quota = Some(quota.download);

    let app_params = cloud_app_catalog_params(filters);
    let apps_json = if app_params.is_empty() {
        state.apps.clear();
        json!({ "items": [] })
    } else {
        let apps_url = format!("{base_url}/v1/apps?{}", app_params.join("&"));
        let (apps, apps_response_json) = get_json::<AppsResponse>(&client, &apps_url, None)?;
        state.apps = apps.items;
        parse_json_value(&apps_response_json)
    };
    let my_apps_json = if let Some(session) =
        state.session.clone().or_else(read_persisted_cloud_session)
    {
        state.session = Some(session.clone());
        let my_apps_url = format!("{base_url}/v1/users/me/apps");
        match get_json::<UserAppsResponse>(&client, &my_apps_url, Some(&session.session_token)) {
            Ok((my_apps, response_json)) => {
                state.my_apps = my_apps.items;
                parse_json_value(&response_json)
            }
            Err(error) => {
                state.my_apps.clear();
                json!({ "items": [], "error": error })
            }
        }
    } else {
        state.my_apps.clear();
        json!({ "items": [] })
    };
    let tags_url = format!("{base_url}/v1/tags");
    let tag_bearer = state
        .session
        .as_ref()
        .map(|session| session.session_token.as_str());
    let tags_json = match get_json::<TagsResponse>(&client, &tags_url, tag_bearer) {
        Ok((response, response_json)) => {
            state.tags = response.items;
            state.own_tag_count = response.own_count;
            state.tag_limit = response.limit;
            parse_json_value(&response_json)
        }
        Err(error) => {
            state.tags.clear();
            json!({ "items": [], "error": error })
        }
    };

    state.last_error = None;
    state.last_response_json = Some(compact_json_value(&json!({
        "health": parse_json_value(&health_json),
        "quota": parse_json_value(&quota_json),
        "apps": apps_json,
        "my_apps": my_apps_json,
        "tags": tags_json,
    })));
    Ok(())
}

pub(crate) fn refresh_cloud_observation_matches(
    state: &mut CloudSyncUiState,
    candidates: Vec<CloudObservationMatchCandidate>,
) -> Result<(), String> {
    if candidates.is_empty() {
        state.matched_observations.clear();
        return Ok(());
    }
    let Some(session) = state.session.clone().or_else(read_persisted_cloud_session) else {
        state.matched_observations.clear();
        return Ok(());
    };
    state.session = Some(session.clone());
    let client = http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let mut items = Vec::new();
    for rows in candidates.chunks(5_000) {
        let response = post_json::<CloudObservationMatchResponse>(
            &client,
            &format!(
                "{}/v1/users/me/observations/match",
                default_cloud_base_url()
            ),
            Some(&session.session_token),
            &serde_json::to_value(CloudObservationMatchRequest {
                rows: rows.to_vec(),
            })
            .map_err(|error| {
                format!("failed to encode cloud observation match request: {error}")
            })?,
        )?;
        items.extend(response.items);
    }
    state.matched_observations = items
        .into_iter()
        .filter_map(|item| {
            item.client_row_id
                .parse::<u64>()
                .ok()
                .map(|observation_id| (observation_id, item))
        })
        .collect();
    Ok(())
}

fn cloud_app_catalog_params(filters: &CloudSearchFilters) -> Vec<String> {
    let mut app_params = Vec::<String>::new();
    if let Some(query) = CloudSearchFilters::effective_text(&filters.app_query) {
        app_params.push(format!("query={}", url_component(&query)));
    }
    if let Some(publisher) = CloudSearchFilters::effective_text(&filters.publisher_query) {
        app_params.push(format!("publisher={}", url_component(&publisher)));
    }
    if let Some(source) = CloudSearchFilters::effective_text(&filters.source_query) {
        app_params.push(format!("source={}", url_component(&source)));
    }
    app_params
}

pub(crate) fn download_observations(
    state: &mut CloudSyncUiState,
    filters: &CloudSearchFilters,
    on_progress: impl FnMut(usize, Option<usize>),
) -> Result<usize, String> {
    let app_id = filters
        .selected_app_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"cloud_app_required\",\"message\":\"Choose an application before downloading\"}}".to_string())?;
    let client = http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let base_url = default_cloud_base_url();
    download_observations_with_client(state, filters, app_id, &client, &base_url, on_progress)
}

fn download_observations_with_client(
    state: &mut CloudSyncUiState,
    filters: &CloudSearchFilters,
    app_id: &str,
    client: &Client,
    base_url: &str,
    mut on_progress: impl FnMut(usize, Option<usize>),
) -> Result<usize, String> {
    let client_identifier = ensure_identifier(state)?;
    let download_operation_id = cloud_download_operation_id(&client_identifier);
    let mut base_params = vec![
        format!("client_identifier={}", url_component(&client_identifier)),
        format!("app_id={}", url_component(app_id)),
        format!("operation_id={}", url_component(&download_operation_id)),
    ];
    if let Some(value) = CloudSearchFilters::effective_text(&filters.remote_ip_query) {
        base_params.push(format!("ip={}", url_component(&value)));
    }
    if let Some(value) = CloudSearchFilters::effective_text(&filters.remote_domain_query) {
        base_params.push(format!("domain={}", url_component(&value)));
    }
    if let Some(value) = CloudSearchFilters::effective_text(&filters.remote_port_query) {
        base_params.push(format!("port={}", url_component(&value)));
    }
    if let Some(value) = CloudSearchFilters::effective_text(&filters.source_query) {
        base_params.push(format!("source={}", url_component(&value)));
    }
    if let Some(value) = filters.effective_protocol() {
        base_params.push(format!("protocol={}", url_component(value)));
    }
    base_params.push(format!(
        "visibility={}",
        filters.visibility_scope.as_query_value()
    ));

    let mut bearer = None;
    if filters.own_scope || filters.visibility_scope == CloudObservationVisibilityScope::Private {
        let session = state
            .session
            .as_ref()
            .ok_or_else(|| "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Author scoped download requires Google sign-in\"}}".to_string())?;
        if filters.own_scope {
            base_params.push(format!(
                "author_user_id={}",
                url_component(&session.user_id)
            ));
        }
        bearer = Some(session.session_token.as_str());
    } else if filters.visibility_scope == CloudObservationVisibilityScope::All {
        bearer = state
            .session
            .as_ref()
            .map(|session| session.session_token.as_str());
    }

    const PAGE_SIZE: usize = 500;
    let expected_total_rows =
        expected_cloud_download_total(state, app_id, filters.visibility_scope, filters.own_scope);
    let mut pages = CloudObservationPageCollector::new(expected_total_rows);
    loop {
        let mut params = base_params.clone();
        params.push(format!("limit={PAGE_SIZE}"));
        params.push(format!("offset={}", pages.offset()));
        let url = format!("{base_url}/v1/observations?{}", params.join("&"));
        let (response, _) = get_json::<CloudObservationDownloadResponse>(&client, &url, bearer)?;
        let should_continue = pages.push_response(response, PAGE_SIZE);
        on_progress(pages.progress_rows(), pages.total_rows());
        if !should_continue {
            break;
        }
    }
    let total_rows = pages.total_rows();

    state.downloaded_rows = pages
        .into_rows()
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let row = item.row;
            let row_id = downloaded_row_id(index, &row);
            let app_display_name = cloud_app_display_name_for_id(state, &row.app_id);
            let privacy_label = item.is_private.to_string();
            let source_label = row.author_signature.clone().unwrap_or_default();
            let protocol_label = cloud_protocol_display_label(row.protocol);
            let connection_label = format!(
                "{} ({}/{})",
                row.connection_state.as_str(),
                row.successful_hits,
                row.failed_hits
            );
            let requests_label = row.requests.to_string();
            let first_seen_label = format_cloud_timestamp(row.first_seen_ms);
            let last_seen_label = format_cloud_timestamp(row.last_seen_ms);
            let domain_label = row.domain_raw.clone().unwrap_or_default();
            CloudDownloadedObservation {
                row_id,
                app_display_name,
                privacy_label,
                source_label,
                protocol_label,
                connection_label,
                requests_label,
                first_seen_label,
                last_seen_label,
                domain_label,
                row,
            }
        })
        .collect();
    state.selected_download_row_ids.clear();

    let quota_url = format!(
        "{base_url}/v1/client/quota?client_identifier={}",
        url_component(&client_identifier)
    );
    if let Ok((quota, _)) = get_json::<QuotaResponse>(&client, &quota_url, None) {
        state.upload_quota = Some(quota.upload);
        state.download_quota = Some(quota.download);
    }

    state.last_error = None;
    state.last_response_json = Some(compact_json_value(&json!({
        "download": {
            "rows": state.downloaded_rows.len(),
            "total": total_rows.unwrap_or(state.downloaded_rows.len()),
        }
    })));
    Ok(state.downloaded_rows.len())
}

struct CloudObservationPageCollector {
    rows: Vec<CollectedCloudObservation>,
    seen_rows: BTreeSet<String>,
    total_rows: Option<usize>,
    offset: usize,
}

impl CloudObservationPageCollector {
    fn new(expected_total_rows: Option<usize>) -> Self {
        Self {
            rows: Vec::new(),
            seen_rows: BTreeSet::new(),
            total_rows: expected_total_rows,
            offset: 0,
        }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn total_rows(&self) -> Option<usize> {
        self.total_rows
    }

    fn progress_rows(&self) -> usize {
        self.total_rows
            .map_or(self.rows.len(), |total| self.rows.len().min(total))
    }

    fn push_response(
        &mut self,
        response: CloudObservationDownloadResponse,
        page_size: usize,
    ) -> bool {
        let page_len = response.rows.len();
        let response_total = usize::try_from(response.total)
            .ok()
            .filter(|value| *value > 0);
        if let Some(response_total) = response_total {
            self.total_rows = Some(
                self.total_rows
                    .map_or(response_total, |total| total.max(response_total)),
            );
        }
        let before_rows = self.rows.len();
        for item in response.rows {
            let is_private = item.visibility == Some(CloudObservationVisibility::Private);
            if self
                .seen_rows
                .insert(cloud_observation_dedupe_key(&item.row, is_private))
            {
                self.rows.push(CollectedCloudObservation {
                    is_private,
                    row: item.row,
                });
            }
        }
        let added_rows = self.rows.len().saturating_sub(before_rows);
        if !self.rows.is_empty() {
            self.total_rows = Some(
                self.total_rows
                    .map_or(self.rows.len(), |total| total.max(self.rows.len())),
            );
        }
        self.offset = self.offset.saturating_add(page_len);
        page_len >= page_size && added_rows > 0
    }

    fn into_rows(self) -> Vec<CollectedCloudObservation> {
        self.rows
    }
}

fn cloud_observation_dedupe_key(row: &CloudObservationRow, is_private: bool) -> String {
    let visibility = if is_private { "private" } else { "public" };
    match row.cloud_observation_id.as_deref() {
        Some(row_id) => format!("{visibility}|{row_id}"),
        None => format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}",
            visibility,
            row.app_id,
            row.author_signature.as_deref().unwrap_or_default(),
            row.remote_ip,
            row.remote_port,
            row.protocol.as_str(),
            row.connection_state.as_str(),
            row.first_seen_ms,
            row.last_seen_ms,
            row.requests,
            row.domain_raw.as_deref().unwrap_or_default(),
            row.source_kind,
            row.app_signature_key.as_deref().unwrap_or_default()
        ),
    }
}

fn expected_cloud_download_total(
    state: &CloudSyncUiState,
    app_id: &str,
    scope: CloudObservationVisibilityScope,
    own_scope: bool,
) -> Option<usize> {
    let public_total = if own_scope {
        state
            .my_apps
            .iter()
            .filter(|app| {
                app.app_id == app_id && app.visibility == CloudObservationVisibility::Public
            })
            .map(|app| app.available_row_count.max(app.endpoint_count))
            .sum()
    } else {
        state
            .apps
            .iter()
            .find(|app| app.app_id == app_id)
            .map(|app| app.available_row_count.max(app.endpoint_count))
            .unwrap_or(0)
    };
    let private_total = if scope == CloudObservationVisibilityScope::Public {
        0
    } else {
        state
            .my_apps
            .iter()
            .filter(|app| {
                app.app_id == app_id && app.visibility == CloudObservationVisibility::Private
            })
            .map(|app| app.available_row_count.max(app.endpoint_count))
            .sum()
    };
    let total = match scope {
        CloudObservationVisibilityScope::All => public_total.saturating_add(private_total),
        CloudObservationVisibilityScope::Public => public_total,
        CloudObservationVisibilityScope::Private => private_total,
    };
    (total > 0)
        .then_some(total)
        .and_then(|value| usize::try_from(value).ok())
}

fn cloud_protocol_display_label(protocol: netstitch_shared::models::Protocol) -> String {
    let protocol = match protocol {
        netstitch_shared::models::Protocol::Tcp => ProtocolDto::Tcp,
        netstitch_shared::models::Protocol::Udp => ProtocolDto::Udp,
        netstitch_shared::models::Protocol::Other => ProtocolDto::Other,
    }
    .as_str();
    protocol.to_string()
}

fn cloud_app_display_name_for_id(state: &CloudSyncUiState, app_id: &str) -> String {
    state
        .apps
        .iter()
        .find(|app| app.app_id == app_id)
        .map(|app| app.display_name.clone())
        .or_else(|| {
            state
                .my_apps
                .iter()
                .find(|app| app.app_id == app_id)
                .map(|app| app.display_name.clone())
        })
        .unwrap_or_else(|| app_id.to_string())
}

fn format_cloud_timestamp(timestamp_ms: u64) -> String {
    chrono::Local
        .timestamp_millis_opt(timestamp_ms as i64)
        .single()
        .map(|date_time| date_time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

pub(crate) fn validate_author_signature(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 32 {
        return Err("{\"error\":{\"code\":\"invalid_author_signature\",\"message\":\"Nickname must be 1..32 characters\"}}".to_string());
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "!.@#$%^&*()_+=-~".contains(ch))
    {
        return Err("{\"error\":{\"code\":\"invalid_author_signature\",\"message\":\"Nickname contains unsupported characters\"}}".to_string());
    }
    if RESERVED_AUTHOR_SIGNATURES.contains(&reserved_author_signature_norm(trimmed).as_str()) {
        return Err(
            "{\"error\":{\"code\":\"nickname_blocked\",\"message\":\"Nickname is reserved\"}}"
                .to_string(),
        );
    }
    Ok(trimmed.to_string())
}

pub(crate) fn validate_cloud_tag(value: &str) -> Result<String, String> {
    netstitch_shared::normalize_cloud_tag(value).ok_or_else(|| {
        format!(
            "{{\"error\":{{\"code\":\"invalid_tag\",\"message\":\"Tag must be 1..{} ASCII letters, digits, '-', '_' or '.'\"}}}}",
            netstitch_shared::CLOUD_TAG_MAX_LENGTH
        )
    })
}

pub(crate) fn refresh_cloud_tags(state: &mut CloudSyncUiState, query: &str) -> Result<(), String> {
    let client = http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let url = if query.trim().is_empty() {
        format!("{}/v1/tags", default_cloud_base_url())
    } else {
        format!(
            "{}/v1/tags?query={}",
            default_cloud_base_url(),
            url_component(query.trim())
        )
    };
    let bearer = state
        .session
        .as_ref()
        .map(|session| session.session_token.as_str());
    let (response, _) = get_json::<TagsResponse>(&client, &url, bearer)?;
    state.tags = response.items;
    state.own_tag_count = response.own_count;
    state.tag_limit = response.limit;
    Ok(())
}

pub(crate) fn delete_cloud_tag(state: &mut CloudSyncUiState, value: &str) -> Result<(), String> {
    let tag = validate_cloud_tag(value)?;
    let session = state
        .session
        .as_ref()
        .ok_or_else(|| {
            "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud tag deletion requires Google sign-in\"}}".to_string()
        })?;
    let client = http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let _ = post_json::<serde_json::Value>(
        &client,
        &format!("{}/v1/tags/delete", default_cloud_base_url()),
        Some(&session.session_token),
        &json!({ "tag": tag }),
    )?;
    refresh_cloud_tags(state, "")
}

pub(crate) fn check_author_signature_availability(
    state: &CloudSyncUiState,
    value: &str,
) -> Result<CloudNicknameCheckStatus, String> {
    let nickname = match validate_author_signature(value) {
        Ok(value) => value,
        Err(_) => return Ok(CloudNicknameCheckStatus::Invalid),
    };
    let client = http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let base_url = default_cloud_base_url();
    let url = format!(
        "{base_url}/v1/authors/nickname?nickname={}",
        url_component(&nickname)
    );
    let bearer = state
        .session
        .as_ref()
        .map(|session| session.session_token.as_str());
    let (response, _) = get_json::<CloudNicknameCheckResponse>(&client, &url, bearer)?;
    match response.status.as_str() {
        "accepted" => Ok(CloudNicknameCheckStatus::Accepted),
        "taken" => Ok(CloudNicknameCheckStatus::Taken),
        "invalid" | "blocked" => Ok(CloudNicknameCheckStatus::Invalid),
        _ => Ok(CloudNicknameCheckStatus::Error),
    }
}

fn reserved_author_signature_norm(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn downloaded_row_id(index: usize, row: &CloudObservationRow) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        index,
        row.app_id,
        row.remote_ip,
        row.remote_port,
        row.protocol.as_str(),
        row.author_signature.as_deref().unwrap_or_default()
    )
}

pub(crate) fn login_cloud_user_with_google(
    state: &mut CloudSyncUiState,
    ui_language: &str,
) -> Result<(), String> {
    let client_identifier = ensure_identifier(state)?;
    let key_material = netstitch_cloud::generate_es256_client_key(format!(
        "netstitch-ui-{}",
        current_timestamp_ms()
    ))
    .map_err(|error| error.to_string())?;
    let client = http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let base_url = default_cloud_base_url();
    let response = client
        .post(format!("{base_url}/v1/auth/google/start"))
        .json(&CloudGoogleAuthStartRequest {
            client_identifier,
            client_public_key_jwk: key_material.public_key_jwk.clone(),
            ui_language: Some(ui_language.to_string()),
        })
        .send()
        .map_err(|error| format!("cloud google auth start failed: {error}"))?;
    let is_success = response.status().is_success();
    let response_text = response
        .text()
        .unwrap_or_else(|_| "cloud google auth start failed".to_string());
    let response_json = compact_json_text(&response_text);
    if !is_success {
        return Err(response_json);
    }
    let start = serde_json::from_str::<CloudGoogleAuthStartResponse>(&response_text)
        .map_err(|error| format!("cloud google auth start response is invalid: {error}"))?;
    state.last_response_json = Some(compact_json_value(&json!({
        "auth": {
            "browser_url": start.browser_url.clone(),
            "state": start.state.clone(),
            "expires_at_ms": start.expires_at_ms,
        }
    })));

    open_browser_url(&start.browser_url)?;

    for _ in 0..150 {
        thread::sleep(std::time::Duration::from_secs(2));
        let response = client
            .post(format!("{base_url}/v1/auth/session/poll"))
            .json(&CloudGoogleAuthPollRequest {
                state: start.state.clone(),
                poll_secret: start.poll_secret.clone(),
            })
            .send()
            .map_err(|error| format!("cloud google auth poll failed: {error}"))?;
        let is_success = response.status().is_success();
        let response_text = response
            .text()
            .unwrap_or_else(|_| "cloud google auth poll failed".to_string());
        let response_json = compact_json_text(&response_text);
        if !is_success {
            return Err(response_json);
        }
        let value = parse_json_value(&response_text);
        if value.get("status").and_then(|status| status.as_str()) == Some("pending") {
            continue;
        }
        let session = serde_json::from_value::<CloudUserSessionResponse>(value)
            .map_err(|error| format!("cloud google auth session is invalid: {error}"))?;
        state.session = Some(session);
        state.client_private_key_pkcs8_der = Some(key_material.private_key_pkcs8_der);
        state.last_error = None;
        state.last_response_json = Some(response_json);
        return Ok(());
    }

    Err("{\"error\":{\"code\":\"auth_timeout\",\"message\":\"Google authentication was not completed in time\"}}".to_string())
}

pub(crate) fn open_browser_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let shell_result = open_browser_url_with_windows_shell(url);
        if shell_result.is_ok() {
            return Ok(());
        }

        let cmd_result = Command::new("cmd").args(["/C", "start", "", url]).spawn();
        if cmd_result.is_ok() {
            return Ok(());
        }

        Err(format!(
            "failed to open browser: ShellExecuteW: {}; cmd start: {}",
            shell_result
                .err()
                .unwrap_or_else(|| "unknown error".to_string()),
            cmd_result
                .err()
                .map(|error| error.to_string())
                .unwrap_or_else(|| "unknown error".to_string()),
        ))
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open browser: {error}"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open browser: {error}"))
    }
}

#[cfg(windows)]
fn open_browser_url_with_windows_shell(url: &str) -> Result<(), String> {
    use std::ffi::c_void;
    use std::ptr;

    unsafe extern "system" {
        fn ShellExecuteW(
            hwnd: *mut c_void,
            lpoperation: *const u16,
            lpfile: *const u16,
            lpparameters: *const u16,
            lpdirectory: *const u16,
            nshowcmd: i32,
        ) -> isize;
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let operation = wide("open");
    let file = wide(url);
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            1,
        )
    };

    if result > 32 {
        Ok(())
    } else {
        Err(format!("ShellExecuteW returned {result}"))
    }
}

fn ensure_identifier(state: &mut CloudSyncUiState) -> Result<String, String> {
    if state.client_identifier.is_empty() {
        state.client_identifier = local_client_identifier()?;
    }
    Ok(state.client_identifier.clone())
}

fn http_client(timeout: Duration) -> Result<Client, String> {
    Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| format!("failed to create cloud HTTP client: {error}"))
}

fn get_json<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
    bearer: Option<&str>,
) -> Result<(T, String), String> {
    let mut request = client.get(url);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|error| format!("cloud request failed: {error}"))?;
    let is_success = response.status().is_success();
    let response_text = response
        .text()
        .unwrap_or_else(|_| "cloud request failed".to_string());
    let response_json = compact_json_text(&response_text);
    if !is_success {
        return Err(response_json);
    }
    let parsed = serde_json::from_str::<T>(&response_text)
        .map_err(|error| format!("cloud response is invalid: {error}"))?;
    Ok((parsed, response_json))
}

fn post_json<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
    bearer: Option<&str>,
    body: &Value,
) -> Result<T, String> {
    let mut request = client.post(url).json(body);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|error| format!("cloud request failed: {error}"))?;
    let is_success = response.status().is_success();
    let response_text = response
        .text()
        .unwrap_or_else(|_| "cloud request failed".to_string());
    if !is_success {
        return Err(compact_json_text(&response_text));
    }
    serde_json::from_str::<T>(&response_text)
        .map_err(|error| format!("cloud response is invalid: {error}"))
}

fn compact_json_text(value: &str) -> String {
    serde_json::from_str::<Value>(value)
        .map(|json| compact_json_value(&json))
        .unwrap_or_else(|_| value.trim().to_string())
}

fn compact_json_value(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
}

fn parse_json_value(value: &str) -> Value {
    serde_json::from_str::<Value>(value).unwrap_or_else(|_| json!(value))
}

fn url_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect::<Vec<_>>(),
        })
        .collect()
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn cloud_upload_operation_id(client_identifier: &str) -> String {
    cloud_operation_id("upl", client_identifier)
}

fn cloud_download_operation_id(client_identifier: &str) -> String {
    cloud_operation_id("dwn", client_identifier)
}

fn cloud_operation_id(prefix: &str, client_identifier: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(client_identifier.as_bytes());
    hasher.update(timestamp.to_string().as_bytes());
    let digest = hasher.finalize();
    let suffix = URL_SAFE_NO_PAD.encode(&digest[..12]);
    format!("{prefix}_{timestamp}_{suffix}")
}

pub(crate) fn quota_label(quota: Option<&CloudQuotaSnapshot>) -> String {
    quota
        .map(|quota| {
            format!(
                "{}/{}",
                quota.used_count.min(quota.limit_count),
                quota.limit_count
            )
        })
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn quota_is_exhausted(quota: Option<&CloudQuotaSnapshot>) -> bool {
    quota
        .map(|quota| quota.limit_count != 0 && quota.used_count >= quota.limit_count)
        .unwrap_or(false)
}

pub(crate) fn quota_window_label(quota: Option<&CloudQuotaSnapshot>, hours_suffix: &str) -> String {
    quota
        .map(|quota| {
            let window_ms = quota
                .window_ends_at_ms
                .saturating_sub(quota.window_started_at_ms);
            let hours = (window_ms / (60 * 60 * 1000)).max(1);
            format!("{hours}{hours_suffix}")
        })
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_refresh_http_timeout_is_not_shorter_than_overlay_refresh_interval_margin() {
        assert!(CLOUD_REFRESH_HTTP_TIMEOUT >= Duration::from_secs(8));
    }

    #[test]
    fn cloud_interactive_http_timeout_keeps_auth_and_upload_slower_than_refresh() {
        assert!(CLOUD_INTERACTIVE_HTTP_TIMEOUT > CLOUD_REFRESH_HTTP_TIMEOUT);
    }

    #[test]
    fn app_catalog_refresh_requires_user_search_text() {
        assert!(cloud_app_catalog_params(&CloudSearchFilters::default()).is_empty());
        assert!(
            cloud_app_catalog_params(&CloudSearchFilters {
                app_query: "a".to_string(),
                ..CloudSearchFilters::default()
            })
            .is_empty()
        );
        assert_eq!(
            cloud_app_catalog_params(&CloudSearchFilters {
                app_query: "ab".to_string(),
                ..CloudSearchFilters::default()
            }),
            vec!["query=ab".to_string()]
        );
        assert_eq!(
            cloud_app_catalog_params(&CloudSearchFilters {
                publisher_query: "co".to_string(),
                ..CloudSearchFilters::default()
            }),
            vec!["publisher=co".to_string()]
        );
    }

    #[test]
    fn cloud_upload_identity_keeps_connector_id_separate_from_imported_display_id() {
        let app = TrackedAppDto {
            id: 1,
            connector_id: Some("chrome".to_string()),
            cloud_app_id: Some("Demo Game".to_string()),
            display_name: "User label".to_string(),
            icon_key: "default".to_string(),
            icon_path: None,
            current_tag: None,
            exe_path: "C:\\Games\\DemoGame.exe".to_string(),
            enabled: true,
            created_at: String::new(),
        };
        let observation = ObservationDto {
            id: 10,
            tracked_app_id: 1,
            cloud_app_id: Some(" IMPORTED   Demo ".to_string()),
            app_signature_key: None,
            app_signature_subject: None,
            app_signature_issuer: None,
            app_signature_source: None,
            process_name: "DemoGame.exe".to_string(),
            remote_ip: "8.8.8.8".to_string(),
            remote_port: 443,
            protocol: ProtocolDto::Tcp,
            first_seen_ms: 1,
            first_seen: String::new(),
            last_seen_ms: 2,
            last_seen: String::new(),
            hits: 1,
            connection_state: ConnectionStateDto::Established,
            failed_hits: 0,
            successful_hits: 1,
            is_confirmed: true,
            is_exported: false,
            tags: Vec::new(),
            cloud_tags: Vec::new(),
            enrichment: None,
        };

        assert_eq!(
            imported_cloud_app_id_for_observation(&observation, Some(&app)).as_deref(),
            Some("imported demo")
        );
        assert_eq!(
            connector_cloud_app_id_for_app(Some(&app)).as_deref(),
            Some("netstitch.app.chrome")
        );
    }

    #[test]
    fn cloud_upload_infers_orphan_observation_app_from_exe_name() {
        let app = TrackedAppDto {
            id: 42,
            connector_id: None,
            cloud_app_id: None,
            display_name: "Demo".to_string(),
            icon_key: "default".to_string(),
            icon_path: None,
            current_tag: None,
            exe_path: "C:\\Games\\DemoApp.exe".to_string(),
            enabled: true,
            created_at: String::new(),
        };
        let observation = ObservationDto {
            id: 10,
            tracked_app_id: 0,
            cloud_app_id: None,
            app_signature_key: None,
            app_signature_subject: None,
            app_signature_issuer: None,
            app_signature_source: None,
            process_name: "Demo App".to_string(),
            remote_ip: "8.8.4.4".to_string(),
            remote_port: 443,
            protocol: ProtocolDto::Tcp,
            first_seen_ms: 1,
            first_seen: String::new(),
            last_seen_ms: 2,
            last_seen: String::new(),
            hits: 1,
            connection_state: ConnectionStateDto::Established,
            failed_hits: 0,
            successful_hits: 1,
            is_confirmed: true,
            is_exported: false,
            tags: Vec::new(),
            cloud_tags: Vec::new(),
            enrichment: None,
        };
        let apps = vec![app];
        let apps_by_id = apps
            .iter()
            .map(|app| (app.id, app))
            .collect::<BTreeMap<_, _>>();

        let inferred = upload_app_for_observation(&observation, &apps_by_id, &apps)
            .expect("orphan observation should infer local app");

        assert_eq!(inferred.id, 42);
    }

    #[test]
    fn zero_quota_limit_does_not_disable_cloud_actions() {
        let quota = CloudQuotaSnapshot {
            operation: CloudOperationKind::Upload,
            limit_count: 0,
            used_count: 0,
            window_started_at_ms: 10,
            window_ends_at_ms: 20,
            synced_at_ms: 11,
        };

        assert_eq!(quota_label(Some(&quota)), "0/0");
        assert!(!quota_is_exhausted(Some(&quota)));
    }

    #[test]
    fn author_signature_validation_is_required_and_case_agnostic_ready() {
        assert_eq!(validate_author_signature("Nick_01-~").unwrap(), "Nick_01-~");
        assert!(validate_author_signature("").is_err());
        assert!(validate_author_signature("кириллица").is_err());
        assert!(validate_author_signature("abcdefghijklmnopqrstuvwxyz1234567").is_err());
        assert!(validate_author_signature("Root").is_err());
        assert!(validate_author_signature("ROOT").is_err());
        assert_eq!(validate_author_signature("r.o.o.t").unwrap(), "r.o.o.t");
        assert_eq!(
            validate_author_signature("Net-Stitch").unwrap(),
            "Net-Stitch"
        );
        assert!(validate_author_signature("NETSTITCH").is_err());
    }

    #[test]
    fn cloud_tag_validation_uses_uppercase_ascii_without_spaces() {
        assert_eq!(validate_cloud_tag("eu-west_1").unwrap(), "EU-WEST_1");
        assert!(validate_cloud_tag(" eu-west_1 ").is_err());
        assert!(validate_cloud_tag("test cloud").is_err());
        assert!(validate_cloud_tag("test\tcloud").is_err());
        assert!(validate_cloud_tag("тест").is_err());
        assert!(validate_cloud_tag("   ").is_err());
    }

    #[test]
    fn cloud_download_does_not_auto_select_imported_rows() {
        let source = include_str!("cloud_sync.rs");
        let download_pos = source
            .find("pub(crate) fn download_observations")
            .expect("download_observations");
        let download_source = &source[download_pos
            ..source[download_pos..]
                .find("fn cloud_protocol_display_label")
                .map(|end| download_pos + end)
                .expect("cloud protocol helper")];

        assert!(
            download_source.contains("state.selected_download_row_ids.clear();"),
            "cloud download should reset import selection to empty"
        );
        assert!(
            !download_source.contains(".map(|row| row.row_id.clone())"),
            "cloud download must not auto-select all downloaded rows"
        );
    }

    #[test]
    fn cloud_download_uses_paged_observation_loading() {
        let source = include_str!("cloud_sync.rs");
        let download_pos = source
            .find("pub(crate) fn download_observations")
            .expect("download_observations");
        let download_source = &source[download_pos
            ..source[download_pos..]
                .find("fn cloud_protocol_display_label")
                .map(|end| download_pos + end)
                .expect("cloud protocol helper")];

        for expected in [
            "const PAGE_SIZE: usize = 500;",
            "loop {",
            "params.push(format!(\"offset={}\", pages.offset()));",
            "CloudObservationPageCollector::new(expected_total_rows)",
            "pages.push_response(response, PAGE_SIZE)",
            "on_progress(pages.progress_rows(), pages.total_rows());",
            ".into_rows()",
        ] {
            assert!(
                download_source.contains(expected),
                "desktop cloud download must keep paged loading token {expected}"
            );
        }
        assert!(
            !download_source.contains("response.offset"),
            "desktop cloud download must not stop pagination because the Worker omitted offset echo"
        );
        assert!(
            !download_source.contains("offset = all_rows.len()"),
            "desktop cloud download must not page by unique row count"
        );
        assert!(
            download_source.contains("added_rows > 0"),
            "desktop cloud download must stop gracefully if the Worker repeats a full page"
        );
        assert!(
            !download_source.contains("offset >= total"),
            "desktop cloud download must not treat a reported total of 500 as a hard stop"
        );
    }

    #[test]
    fn cloud_download_page_collector_advances_by_raw_page_size() {
        let mut collector = CloudObservationPageCollector::new(Some(1_102));

        assert!(collector.push_response(cloud_download_page(0, 500, 1_102), 500));
        assert_eq!(collector.offset(), 500);
        assert_eq!(collector.progress_rows(), 500);

        assert!(collector.push_response(cloud_download_page(500, 500, 1_102), 500));
        assert_eq!(collector.offset(), 1_000);
        assert_eq!(collector.progress_rows(), 1_000);

        assert!(!collector.push_response(cloud_download_page(1_000, 102, 1_102), 500));
        assert_eq!(collector.offset(), 1_102);
        assert_eq!(collector.progress_rows(), 1_102);
        assert_eq!(collector.into_rows().len(), 1_102);
    }

    #[test]
    fn cloud_download_page_collector_ignores_underreported_total() {
        let mut collector = CloudObservationPageCollector::new(Some(602));

        assert!(collector.push_response(cloud_download_page(0, 500, 500), 500));
        assert_eq!(collector.offset(), 500);
        assert_eq!(collector.progress_rows(), 500);
        assert_eq!(collector.total_rows(), Some(602));

        assert!(!collector.push_response(cloud_download_page(500, 102, 500), 500));
        assert_eq!(collector.offset(), 602);
        assert_eq!(collector.progress_rows(), 602);
        assert_eq!(collector.total_rows(), Some(602));
        assert_eq!(collector.into_rows().len(), 602);
    }

    #[test]
    fn cloud_download_page_collector_does_not_need_catalog_total() {
        let mut collector = CloudObservationPageCollector::new(None);

        assert!(collector.push_response(cloud_download_page(0, 500, 500), 500));
        assert_eq!(collector.offset(), 500);
        assert_eq!(collector.progress_rows(), 500);
        assert_eq!(collector.total_rows(), Some(500));

        assert!(!collector.push_response(cloud_download_page(500, 102, 500), 500));
        assert_eq!(collector.offset(), 602);
        assert_eq!(collector.progress_rows(), 602);
        assert_eq!(collector.total_rows(), Some(602));
        assert_eq!(collector.into_rows().len(), 602);
    }

    #[test]
    fn cloud_download_page_collector_stops_gracefully_on_duplicate_page() {
        let mut collector = CloudObservationPageCollector::new(Some(1_000));
        let first_page = cloud_download_page(0, 500, 1_000);
        let duplicate_page = first_page.clone();

        assert!(collector.push_response(first_page, 500));
        assert!(!collector.push_response(duplicate_page, 500));
        assert_eq!(collector.offset(), 1_000);
        assert_eq!(collector.progress_rows(), 500);
        assert_eq!(collector.into_rows().len(), 500);
    }

    #[test]
    fn cloud_download_page_collector_keeps_public_and_private_duplicates() {
        let mut public_row = cloud_observation_row(1);
        public_row.visibility = Some(CloudObservationVisibility::Public);
        let mut private_row = public_row.clone();
        private_row.visibility = Some(CloudObservationVisibility::Private);

        let mut collector = CloudObservationPageCollector::new(Some(2));
        assert!(!collector.push_response(
            CloudObservationDownloadResponse {
                total: 2,
                limit: 500,
                offset: 0,
                rows: vec![public_row, private_row],
            },
            500,
        ));
        let rows = collector.into_rows();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows.iter().filter(|row| row.is_private).count(), 1);
    }

    #[test]
    fn cloud_download_expected_total_sums_public_and_private_for_all_scope() {
        let state = CloudSyncUiState {
            apps: vec![CloudCatalogApp {
                app_id: "app".to_string(),
                display_name: "Demo App".to_string(),
                publisher_name: "Demo Publisher".to_string(),
                authors: vec!["Author".to_string()],
                endpoint_count: 2,
                available_row_count: 2,
                author_count: 1,
                last_seen_ms: None,
            }],
            my_apps: vec![CloudUserAppSummary {
                app_id: "app".to_string(),
                display_name: "Demo App".to_string(),
                publisher_name: Some("Demo Publisher".to_string()),
                author_signature: Some("Author".to_string()),
                visibility: CloudObservationVisibility::Private,
                endpoint_count: 2,
                total_endpoint_count: 2,
                available_row_count: 2,
                tag_groups: Vec::new(),
                last_seen_ms: None,
                last_uploaded_at_ms: None,
            }],
            ..CloudSyncUiState::default()
        };

        assert_eq!(
            expected_cloud_download_total(
                &state,
                "app",
                CloudObservationVisibilityScope::All,
                false
            ),
            Some(4)
        );
        assert_eq!(
            expected_cloud_download_total(
                &state,
                "app",
                CloudObservationVisibilityScope::Public,
                false
            ),
            Some(2)
        );
        assert_eq!(
            expected_cloud_download_total(
                &state,
                "app",
                CloudObservationVisibilityScope::Private,
                false
            ),
            Some(2)
        );
    }

    #[test]
    fn cloud_download_expected_total_uses_author_rows_for_own_scope() {
        let state = CloudSyncUiState {
            apps: vec![CloudCatalogApp {
                app_id: "app".to_string(),
                display_name: "Demo App".to_string(),
                publisher_name: "Demo Publisher".to_string(),
                authors: vec!["Author".to_string()],
                endpoint_count: 602,
                available_row_count: 602,
                author_count: 1,
                last_seen_ms: None,
            }],
            my_apps: vec![
                CloudUserAppSummary {
                    app_id: "app".to_string(),
                    display_name: "Demo App".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Author".to_string()),
                    visibility: CloudObservationVisibility::Public,
                    endpoint_count: 602,
                    total_endpoint_count: 602,
                    available_row_count: 602,
                    tag_groups: Vec::new(),
                    last_seen_ms: None,
                    last_uploaded_at_ms: None,
                },
                CloudUserAppSummary {
                    app_id: "app".to_string(),
                    display_name: "Demo App".to_string(),
                    publisher_name: Some("Demo Publisher".to_string()),
                    author_signature: Some("Author".to_string()),
                    visibility: CloudObservationVisibility::Private,
                    endpoint_count: 3,
                    total_endpoint_count: 3,
                    available_row_count: 3,
                    tag_groups: Vec::new(),
                    last_seen_ms: None,
                    last_uploaded_at_ms: None,
                },
            ],
            ..CloudSyncUiState::default()
        };

        assert_eq!(
            expected_cloud_download_total(
                &state,
                "app",
                CloudObservationVisibilityScope::All,
                true
            ),
            Some(605)
        );
        assert_eq!(
            expected_cloud_download_total(
                &state,
                "app",
                CloudObservationVisibilityScope::Public,
                true
            ),
            Some(602)
        );
    }

    #[test]
    fn cloud_download_observations_fetches_all_http_pages() {
        let (base_url, server) = spawn_cloud_download_server();
        let client = http_client(Duration::from_secs(5)).expect("test HTTP client");
        let mut state = CloudSyncUiState {
            client_identifier: "test-client".to_string(),
            apps: vec![CloudCatalogApp {
                app_id: "app".to_string(),
                display_name: "Demo App".to_string(),
                publisher_name: "Demo Publisher".to_string(),
                authors: vec!["Author".to_string()],
                endpoint_count: 602,
                available_row_count: 602,
                author_count: 1,
                last_seen_ms: None,
            }],
            ..CloudSyncUiState::default()
        };
        let filters = CloudSearchFilters {
            selected_app_id: Some("app".to_string()),
            ..CloudSearchFilters::default()
        };
        let mut progress = Vec::new();

        let count = download_observations_with_client(
            &mut state,
            &filters,
            "app",
            &client,
            &base_url,
            |loaded, total| progress.push((loaded, total)),
        )
        .expect("cloud download should fetch every page");

        let requests = server.join().expect("server should finish");
        assert_eq!(count, 602);
        assert_eq!(state.downloaded_rows.len(), 602);
        assert_eq!(state.downloaded_rows[0].row.tags, vec!["TAG-0".to_string()]);
        assert_eq!(
            state.downloaded_rows[501].row.tags,
            vec!["TAG-501".to_string()]
        );
        assert_eq!(
            state.last_response_json.as_deref(),
            Some(r#"{"download":{"rows":602,"total":602}}"#)
        );
        assert!(state.selected_download_row_ids.is_empty());
        assert_eq!(progress, vec![(500, Some(602)), (602, Some(602))]);
        assert!(
            requests.iter().any(
                |request| request.contains("/v1/observations?") && request.contains("offset=0")
            ),
            "first request should fetch offset=0: {requests:?}"
        );
        assert!(
            requests
                .iter()
                .any(|request| request.contains("/v1/observations?")
                    && request.contains("offset=500")),
            "second request should fetch offset=500: {requests:?}"
        );
        assert!(
            !requests
                .iter()
                .any(|request| request.contains("offset=602")),
            "short second page should stop without a third observation request: {requests:?}"
        );
    }

    #[test]
    fn generated_cloud_app_id_is_stable_and_local() {
        assert_eq!(
            cloud_app_id_from_signature_key("appsig_v1_ABCdef1234567890_-extra"),
            "netstitch.app.local.abcdef1234567890_-extra"
        );
    }

    fn cloud_download_page(
        start: usize,
        count: usize,
        total: u64,
    ) -> CloudObservationDownloadResponse {
        CloudObservationDownloadResponse {
            total,
            limit: 500,
            offset: start as u64,
            rows: (start..start + count).map(cloud_observation_row).collect(),
        }
    }

    fn spawn_cloud_download_server() -> (String, std::thread::JoinHandle<Vec<String>>) {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("test cloud server bind");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("test cloud server address")
        );
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for stream in listener.incoming().take(3) {
                let mut stream = stream.expect("test cloud server connection");
                let mut buffer = [0u8; 8192];
                let read = stream.read(&mut buffer).expect("test cloud request read");
                let request = String::from_utf8_lossy(&buffer[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/")
                    .to_string();
                requests.push(path.clone());
                let body = if path.starts_with("/v1/observations?") && path.contains("offset=0") {
                    serde_json::to_string(&cloud_download_page(0, 500, 500))
                        .expect("first cloud page json")
                } else if path.starts_with("/v1/observations?") && path.contains("offset=500") {
                    serde_json::to_string(&cloud_download_page(500, 102, 500))
                        .expect("second cloud page json")
                } else {
                    "{}".to_string()
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("test cloud response write");
            }
            requests
        });
        (base_url, server)
    }

    fn cloud_observation_row(index: usize) -> CloudObservationDownloadRow {
        CloudObservationDownloadRow {
            visibility: Some(CloudObservationVisibility::Public),
            row: CloudObservationRow {
                app_id: "app".to_string(),
                author_signature: Some("Author".to_string()),
                remote_ip: IpAddr::from([10, 0, (index / 255) as u8, (index % 255) as u8]),
                remote_port: 443,
                protocol: netstitch_shared::Protocol::Tcp,
                connection_state: netstitch_shared::ConnectionState::Established,
                requests: index as u64,
                first_seen_ms: index as u64,
                last_seen_ms: 10_000 + index as u64,
                failed_hits: 0,
                successful_hits: 1,
                domain_raw: Some(format!("cdn-{index}.example")),
                domain_verified: None,
                domain_status: netstitch_cloud::CloudDomainStatus::None,
                trust_level: CloudTrustLevel::CommunityVerified,
                source_kind: CloudSourceKind::VerifiedUpload,
                app_signature_key: Some("appsig".to_string()),
                app_signature_subject: None,
                app_signature_issuer: None,
                tags: vec![format!("TAG-{index}")],
                cloud_observation_id: Some(format!("row-{index}")),
            },
        }
    }

    #[test]
    fn display_slug_keeps_duplicate_app_names_distinguishable() {
        assert_eq!(
            short_app_id_from_display("Same App", "appsig_v1_ABCDEF123456"),
            "same-app-abcdef12"
        );
        assert_eq!(
            short_app_id_from_display("Same App", "appsig_v1_987654321"),
            "same-app-98765432"
        );
    }
}
