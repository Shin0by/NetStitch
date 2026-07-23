use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

pub const CLOUD_TAG_MAX_LENGTH: usize = 16;
pub const CLOUD_TAGS_PER_OBSERVATION_LIMIT: usize = 32;
pub const CLOUD_TAGS_PER_USER_LIMIT: usize = 100;

pub fn normalize_cloud_tag(value: &str) -> Option<String> {
    let normalized = value.to_ascii_uppercase();
    let valid_length = (1..=CLOUD_TAG_MAX_LENGTH).contains(&normalized.chars().count());
    let valid_chars = normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    let has_alphanumeric = normalized.chars().any(|ch| ch.is_ascii_alphanumeric());
    (valid_length && valid_chars && has_alphanumeric).then_some(normalized)
}
use sha2::{Digest, Sha256};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{ConnectionState, Protocol, TimestampMillis};

pub const DEFAULT_CLOUD_QUOTA_LIMIT: u32 = 0;
pub const DEFAULT_CLOUD_QUOTA_WINDOW_MS: u64 = 6 * 60 * 60 * 1000;
pub const DEFAULT_CLOUD_OBSERVATION_RETENTION_MS: u64 = 365 * 24 * 60 * 60 * 1000;
pub const CLOUD_OBSERVATION_BATCH_LIMIT: usize = 65_535;
pub const CLOUD_CLIENT_IDENTIFIER_PREFIX: &str = "nsclid_v1_";
pub const WEB_ACCESS_KEY_PREFIX: &str = "nswkey_v1_";
pub const CLOUD_JWT_AUDIENCE: &str = "netstitch-cloud";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudRequestTrigger {
    UserAction,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudOperationKind {
    Upload,
    Download,
    QuotaCheck,
    AppList,
    UserAppList,
    HealthCheck,
    OAuthStart,
    OAuthPoll,
    CaptchaVerify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudQuotaSnapshot {
    pub operation: CloudOperationKind,
    pub limit_count: u32,
    pub used_count: u32,
    pub window_started_at_ms: TimestampMillis,
    pub window_ends_at_ms: TimestampMillis,
    pub synced_at_ms: TimestampMillis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudClientIdentifierInput {
    pub os_family: String,
    pub os_version: String,
    pub machine_signals: Vec<String>,
}

pub fn cloud_observation_ip_is_public(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => ipv4_is_public_for_cloud(address),
        IpAddr::V6(address) => ipv6_is_public_for_cloud(address),
    }
}

fn ipv4_is_public_for_cloud(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    if address.is_unspecified()
        || address.is_loopback()
        || address.is_private()
        || address.is_link_local()
        || address.is_multicast()
        || address.is_broadcast()
        || address.is_documentation()
    {
        return false;
    }
    if octets[0] == 0
        || octets[0] >= 240
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 198 && (18..=19).contains(&octets[1]))
    {
        return false;
    }
    true
}

fn ipv6_is_public_for_cloud(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    if address.is_unspecified()
        || address.is_loopback()
        || address.is_multicast()
        || ((segments[0] & 0xfe00) == 0xfc00)
        || ((segments[0] & 0xffc0) == 0xfe80)
        || (segments[0] == 0x2001 && segments[1] == 0x0)
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
        || segments[0] == 0x2002
    {
        return false;
    }
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudJwtClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub scope: String,
    pub iat: u64,
    pub nbf: u64,
    pub exp: u64,
    pub jti: String,
    pub body_sha256: String,
    pub client_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudGoogleAuthStartRequest {
    pub client_identifier: String,
    pub client_public_key_jwk: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudGoogleAuthStartResponse {
    pub state: String,
    pub poll_secret: String,
    pub browser_url: String,
    pub expires_at_ms: TimestampMillis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudGoogleAuthPollRequest {
    pub state: String,
    pub poll_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudUserSessionResponse {
    pub user_id: String,
    pub login: String,
    #[serde(default)]
    pub client_id: String,
    pub key_id: String,
    pub session_token: String,
    pub session_expires_at_ms: TimestampMillis,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudUserAppSummary {
    pub app_id: String,
    pub display_name: String,
    pub publisher_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(default)]
    pub visibility: CloudObservationVisibility,
    pub endpoint_count: u64,
    #[serde(default)]
    pub total_endpoint_count: u64,
    #[serde(default)]
    pub available_row_count: u64,
    pub last_uploaded_at_ms: Option<TimestampMillis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudObservationQueryScope {
    AppCommunity,
    AppByAuthor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudObservationQuery {
    pub app_id: String,
    pub scope: CloudObservationQueryScope,
    pub author_user_id: Option<String>,
    #[serde(default)]
    pub client_identifier: Option<String>,
    #[serde(default)]
    pub app_query: Option<String>,
    #[serde(default)]
    pub remote_ip_query: Option<String>,
    #[serde(default)]
    pub remote_port_query: Option<String>,
    #[serde(default)]
    pub protocol: Option<Protocol>,
    #[serde(default)]
    pub source_query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifiedAppMethod {
    Authenticode,
    StableFileIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifiedAppStatus {
    Verified,
    Unverified,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedAppProof {
    pub app_id: String,
    pub platform: String,
    pub method: VerifiedAppMethod,
    pub status: VerifiedAppStatus,
    pub signature_chain_valid: bool,
    pub leaf_spki_sha256: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_app_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StableAppIdentityInput {
    pub authenticode_leaf_spki_sha256: Option<String>,
    pub company_name: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableAppIdentity {
    pub signature_key: String,
    pub subject: String,
    pub source: String,
}

pub fn stable_app_identity(input: StableAppIdentityInput) -> Option<StableAppIdentity> {
    let authenticode_leaf = normalize_identity_component(
        input
            .authenticode_leaf_spki_sha256
            .as_deref()
            .unwrap_or_default(),
    );
    let company_name =
        normalize_identity_component(input.company_name.as_deref().unwrap_or_default());
    let product_name =
        normalize_identity_component(input.product_name.as_deref().unwrap_or_default());

    if authenticode_leaf.is_empty() && company_name.is_empty() && product_name.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    if !authenticode_leaf.is_empty() {
        parts.push(format!("authenticode_leaf_spki_sha256={authenticode_leaf}"));
    }
    if !company_name.is_empty() {
        parts.push(format!("version_info_company_name={company_name}"));
    }
    if !product_name.is_empty() {
        parts.push(format!("version_info_product_name={product_name}"));
    }
    parts.sort();

    let canonical = format!("netstitch_stable_app_identity_v1\n{}", parts.join("\n"));
    let digest = Sha256::digest(canonical.as_bytes());
    let signature_key = format!("appsig_v1_{}", URL_SAFE_NO_PAD.encode(digest));

    let mut subject_parts = Vec::new();
    if !company_name.is_empty() {
        subject_parts.push(format!("CompanyName={company_name}"));
    }
    if !product_name.is_empty() {
        subject_parts.push(format!("ProductName={product_name}"));
    }
    if !authenticode_leaf.is_empty() {
        subject_parts.push("Authenticode=present".to_string());
    }

    let source = if !authenticode_leaf.is_empty()
        && (!company_name.is_empty() || !product_name.is_empty())
    {
        "authenticode+version_info"
    } else if !authenticode_leaf.is_empty() {
        "authenticode"
    } else {
        "version_info"
    };

    Some(StableAppIdentity {
        signature_key,
        subject: subject_parts.join("; "),
        source: source.to_string(),
    })
}

pub fn derive_web_access_key(client_identifier: &str) -> Option<String> {
    let client_identifier = client_identifier.trim();
    if client_identifier.is_empty() {
        return None;
    }
    let canonical = format!("netstitch_web_access_key_v1\n{client_identifier}");
    let digest = Sha256::digest(canonical.as_bytes());
    Some(format!(
        "{}{}",
        WEB_ACCESS_KEY_PREFIX,
        URL_SAFE_NO_PAD.encode(&digest[..18])
    ))
}

fn normalize_identity_component(value: &str) -> String {
    value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| !ch.is_control())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_app_identity_lowercases_and_excludes_version_like_values() {
        let first = stable_app_identity(StableAppIdentityInput {
            authenticode_leaf_spki_sha256: None,
            company_name: Some(" Example  Games Corp. ".to_string()),
            product_name: Some("Demo Game".to_string()),
        })
        .expect("identity");
        let second = stable_app_identity(StableAppIdentityInput {
            authenticode_leaf_spki_sha256: None,
            company_name: Some("example games corp.".to_string()),
            product_name: Some("DEMO GAME".to_string()),
        })
        .expect("identity");

        assert_eq!(first.signature_key, second.signature_key);
        assert!(first.signature_key.starts_with("appsig_v1_"));
        assert_eq!(
            first.subject,
            "CompanyName=example games corp.; ProductName=demo game"
        );
        assert_eq!(first.source, "version_info");
    }

    #[test]
    fn stable_app_identity_keeps_signed_and_unsigned_keys_separate() {
        let unsigned = stable_app_identity(StableAppIdentityInput {
            authenticode_leaf_spki_sha256: None,
            company_name: Some("Example Corp".to_string()),
            product_name: Some("Example App".to_string()),
        })
        .expect("unsigned identity");
        let signed = stable_app_identity(StableAppIdentityInput {
            authenticode_leaf_spki_sha256: Some("abc".to_string()),
            company_name: Some("Example Corp".to_string()),
            product_name: Some("Example App".to_string()),
        })
        .expect("signed identity");

        assert_ne!(unsigned.signature_key, signed.signature_key);
        assert_eq!(signed.source, "authenticode+version_info");
    }

    #[test]
    fn web_access_key_is_derived_without_exposing_client_identifier() {
        let client_identifier = "nsclid_v1_example-client-identifier";
        let first = derive_web_access_key(client_identifier).expect("web key");
        let second = derive_web_access_key(client_identifier).expect("web key");

        assert_eq!(first, second);
        assert!(first.starts_with(WEB_ACCESS_KEY_PREFIX));
        assert!(!first.contains(client_identifier));
        assert!(derive_web_access_key("   ").is_none());
    }

    #[test]
    fn cloud_tags_are_normalized_and_reject_unsupported_values() {
        assert_eq!(
            normalize_cloud_tag("EU-West_1.").as_deref(),
            Some("EU-WEST_1.")
        );
        assert!(normalize_cloud_tag(" EU-West_1. ").is_none());
        assert!(normalize_cloud_tag("test cloud").is_none());
        assert!(normalize_cloud_tag("china/east").is_none());
        assert!(normalize_cloud_tag("test\tcloud").is_none());
        assert!(normalize_cloud_tag("___").is_none());
        assert!(normalize_cloud_tag("abcdefghijklmnopq").is_none());
    }

    #[test]
    fn cloud_observation_visibility_serializes_as_api_lowercase() {
        assert_eq!(
            serde_json::to_value(CloudObservationVisibility::Private).expect("visibility json"),
            serde_json::json!("private")
        );
        assert_eq!(
            serde_json::from_value::<CloudObservationVisibility>(serde_json::json!("public"))
                .expect("visibility from json"),
            CloudObservationVisibility::Public
        );
    }

    fn ip(value: &str) -> IpAddr {
        value.parse().expect("IP fixture")
    }

    #[test]
    fn cloud_upload_accepts_only_public_ip_addresses() {
        assert!(cloud_observation_ip_is_public(ip("8.8.8.8")));
        assert!(cloud_observation_ip_is_public(ip("2606:4700:4700::1111")));

        for value in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "100.64.0.1",
            "169.254.1.1",
            "192.0.2.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "2001:db8::1",
            "2002:c000:0201::1",
            "2001:0:14c9:d804:1cd3:27d:da69:4e2d",
        ] {
            assert!(
                !cloud_observation_ip_is_public(ip(value)),
                "{value} must not be uploaded to cloud"
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CloudSourceKind {
    VerifiedUpload,
    CloudImport,
    CloudImportUntrusted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudTrustLevel {
    VerifiedUploadCandidate,
    CommunityVerified,
    CloudImportUntrusted,
    BlockedOrSuspect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudDomainStatus {
    None,
    Verified,
    Mismatch,
    Unresolved,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudObservationVisibility {
    Public,
    Private,
}

impl Default for CloudObservationVisibility {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudObservationVisibilityScope {
    All,
    Public,
    Private,
}

impl Default for CloudObservationVisibilityScope {
    fn default() -> Self {
        Self::All
    }
}

impl CloudObservationVisibilityScope {
    pub fn as_query_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudObservationRow {
    pub app_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub connection_state: ConnectionState,
    pub requests: u64,
    pub first_seen_ms: TimestampMillis,
    pub last_seen_ms: TimestampMillis,
    pub failed_hits: u64,
    pub successful_hits: u64,
    pub domain_raw: Option<String>,
    pub domain_verified: Option<String>,
    pub domain_status: CloudDomainStatus,
    pub trust_level: CloudTrustLevel,
    pub source_kind: CloudSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_signature_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_signature_subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_signature_issuer: Option<String>,
    pub cloud_observation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudObservationAppendRequest {
    pub client_identifier: String,
    pub client_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    pub author_signature: String,
    #[serde(default)]
    pub visibility: CloudObservationVisibility,
    pub app_proof: VerifiedAppProof,
    pub rows: Vec<CloudObservationRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudObservationDownloadResponse {
    pub scope: String,
    pub app_id: String,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
    pub rows: Vec<CloudObservationRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudKnownApp {
    pub connector_id: &'static str,
    pub app_id: &'static str,
    pub display_name: &'static str,
    pub publisher_name: &'static str,
    pub short_app_id: &'static str,
}

pub const CLOUD_KNOWN_APPS: &[CloudKnownApp] = &[
    CloudKnownApp {
        connector_id: "chrome",
        app_id: "netstitch.app.chrome",
        display_name: "Google Chrome",
        publisher_name: "Google LLC",
        short_app_id: "chrome",
    },
    CloudKnownApp {
        connector_id: "discord",
        app_id: "netstitch.app.discord",
        display_name: "Discord",
        publisher_name: "Discord Inc.",
        short_app_id: "discord",
    },
    CloudKnownApp {
        connector_id: "firefox",
        app_id: "netstitch.app.firefox",
        display_name: "Mozilla Firefox",
        publisher_name: "Mozilla",
        short_app_id: "firefox",
    },
    CloudKnownApp {
        connector_id: "telegram",
        app_id: "netstitch.app.telegram",
        display_name: "Telegram",
        publisher_name: "Telegram",
        short_app_id: "telegram",
    },
    CloudKnownApp {
        connector_id: "whatsapp",
        app_id: "netstitch.app.whatsapp",
        display_name: "WhatsApp",
        publisher_name: "WhatsApp LLC",
        short_app_id: "whatsapp",
    },
    CloudKnownApp {
        connector_id: "yandex_browser",
        app_id: "netstitch.app.yandex-browser",
        display_name: "Yandex Browser",
        publisher_name: "Yandex",
        short_app_id: "yandex-browser",
    },
];

pub fn cloud_app_for_connector_id(connector_id: &str) -> Option<CloudKnownApp> {
    let normalized = connector_id.trim();
    CLOUD_KNOWN_APPS
        .iter()
        .copied()
        .find(|app| app.connector_id.eq_ignore_ascii_case(normalized))
}

pub fn cloud_connector_id_for_app_id(app_id: &str) -> Option<&'static str> {
    let normalized = app_id.trim();
    CLOUD_KNOWN_APPS
        .iter()
        .find(|app| app.app_id.eq_ignore_ascii_case(normalized))
        .map(|app| app.connector_id)
}
