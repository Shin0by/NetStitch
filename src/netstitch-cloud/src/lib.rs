use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
pub use netstitch_shared::{
    CLOUD_CLIENT_IDENTIFIER_PREFIX as CLIENT_IDENTIFIER_PREFIX, CLOUD_JWT_AUDIENCE as JWT_AUDIENCE,
    CLOUD_OBSERVATION_BATCH_LIMIT, CloudClientIdentifierInput, CloudDomainStatus,
    CloudGoogleAuthPollRequest, CloudGoogleAuthStartRequest, CloudGoogleAuthStartResponse,
    CloudJwtClaims, CloudObservationAppendRequest, CloudObservationQuery,
    CloudObservationQueryScope, CloudObservationRow, CloudObservationVisibility,
    CloudObservationVisibilityScope, CloudOperationKind, CloudQuotaSnapshot, CloudRequestTrigger,
    CloudSourceKind, CloudTrustLevel, CloudUserAppSummary, CloudUserSessionResponse,
    DEFAULT_CLOUD_OBSERVATION_RETENTION_MS as DEFAULT_OBSERVATION_RETENTION_MS,
    DEFAULT_CLOUD_QUOTA_LIMIT as DEFAULT_QUOTA_LIMIT,
    DEFAULT_CLOUD_QUOTA_WINDOW_MS as DEFAULT_QUOTA_WINDOW_MS, VerifiedAppMethod, VerifiedAppProof,
    VerifiedAppStatus,
};
use ring::signature::KeyPair;
use ring::{digest, hmac, rand, signature};
use uuid::Uuid;

const JWT_CLOCK_SKEW_SECONDS: u64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudContractError {
    EmptyClientSignals,
    EmptyIdentifierSalt,
    BackgroundCloudRequest,
    QuotaExhausted,
    InvalidJwtTtl,
    KeyGenerationFailed,
    InvalidPrivateKey,
    JwtSigningFailed,
    JwtSerializationFailed,
}

impl fmt::Display for CloudContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyClientSignals => f.write_str("client identifier signals are empty"),
            Self::EmptyIdentifierSalt => f.write_str("client identifier salt is empty"),
            Self::BackgroundCloudRequest => {
                f.write_str("cloud requests must be started by an explicit user action")
            }
            Self::QuotaExhausted => f.write_str("cloud request quota is exhausted"),
            Self::InvalidJwtTtl => f.write_str("JWT TTL is invalid"),
            Self::KeyGenerationFailed => f.write_str("failed to generate client signing key"),
            Self::InvalidPrivateKey => f.write_str("client signing key is invalid"),
            Self::JwtSigningFailed => f.write_str("failed to sign JWT"),
            Self::JwtSerializationFailed => f.write_str("failed to serialize JWT"),
        }
    }
}

impl std::error::Error for CloudContractError {}

pub fn ensure_user_action(trigger: CloudRequestTrigger) -> Result<(), CloudContractError> {
    match trigger {
        CloudRequestTrigger::UserAction => Ok(()),
        CloudRequestTrigger::Background => Err(CloudContractError::BackgroundCloudRequest),
    }
}

pub fn operation_scope(operation: CloudOperationKind) -> &'static str {
    match operation {
        CloudOperationKind::Upload => "observations:append",
        CloudOperationKind::Download => "observations:read",
        CloudOperationKind::QuotaCheck => "quota:read",
        CloudOperationKind::AppList => "apps:read",
        CloudOperationKind::UserAppList => "apps:read:self",
        CloudOperationKind::HealthCheck => "health:read",
        CloudOperationKind::OAuthStart => "auth:oauth:start",
        CloudOperationKind::OAuthPoll => "auth:oauth:poll",
        CloudOperationKind::CaptchaVerify => "auth:captcha:verify",
    }
}

pub fn quota_bucket(operation: CloudOperationKind) -> Option<&'static str> {
    match operation {
        CloudOperationKind::Upload => Some("upload"),
        CloudOperationKind::Download => Some("download"),
        CloudOperationKind::QuotaCheck
        | CloudOperationKind::AppList
        | CloudOperationKind::UserAppList
        | CloudOperationKind::HealthCheck
        | CloudOperationKind::OAuthStart
        | CloudOperationKind::OAuthPoll
        | CloudOperationKind::CaptchaVerify => None,
    }
}

pub fn observation_expires_at_ms(last_seen_ms: u64) -> u64 {
    last_seen_ms.saturating_add(DEFAULT_OBSERVATION_RETENTION_MS)
}

pub fn observation_is_expired(last_seen_ms: u64, now_ms: u64) -> bool {
    observation_expires_at_ms(last_seen_ms) <= now_ms
}

pub fn observation_query_is_valid(query: &CloudObservationQuery) -> bool {
    if query.app_id.trim().is_empty() {
        return false;
    }

    match query.scope {
        CloudObservationQueryScope::AppCommunity => query.author_user_id.is_none(),
        CloudObservationQueryScope::AppByAuthor => query
            .author_user_id
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
    }
}

pub fn quota_remaining_count(quota: &CloudQuotaSnapshot) -> u32 {
    if quota.limit_count == 0 {
        return u32::MAX;
    }
    quota.limit_count.saturating_sub(quota.used_count)
}

pub fn quota_is_exhausted(quota: &CloudQuotaSnapshot) -> bool {
    quota_remaining_count(quota) == 0
}

pub fn try_spend_local_quota_mirror(
    quota: &mut CloudQuotaSnapshot,
) -> Result<(), CloudContractError> {
    if quota.limit_count == 0 {
        quota.used_count = 0;
        return Ok(());
    }
    if quota_is_exhausted(quota) {
        return Err(CloudContractError::QuotaExhausted);
    }
    quota.used_count = quota.used_count.saturating_add(1);
    Ok(())
}

pub fn derive_client_identifier(
    input: &CloudClientIdentifierInput,
    salt: &[u8],
) -> Result<String, CloudContractError> {
    if salt.is_empty() {
        return Err(CloudContractError::EmptyIdentifierSalt);
    }

    let mut signals = input
        .machine_signals
        .iter()
        .map(|signal| normalize_identifier_component(signal))
        .filter(|signal| !signal.is_empty())
        .collect::<Vec<_>>();
    signals.sort();
    signals.dedup();

    if signals.is_empty() {
        return Err(CloudContractError::EmptyClientSignals);
    }

    let canonical = format!(
        "v1\nos_family={}\nos_version={}\nsignals={}",
        normalize_identifier_component(&input.os_family),
        normalize_identifier_component(&input.os_version),
        signals.join("|")
    );

    let key = hmac::Key::new(hmac::HMAC_SHA256, salt);
    let tag = hmac::sign(&key, canonical.as_bytes());
    Ok(format!(
        "{}{}",
        CLIENT_IDENTIFIER_PREFIX,
        URL_SAFE_NO_PAD.encode(tag.as_ref())
    ))
}

pub fn payload_sha256_base64url(payload: &[u8]) -> String {
    let digest = digest::digest(&digest::SHA256, payload);
    URL_SAFE_NO_PAD.encode(digest.as_ref())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudClientKeyMaterial {
    pub key_id: String,
    pub private_key_pkcs8_der: Vec<u8>,
    pub public_key_jwk: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudSignedJwtInput<'a> {
    pub key_id: &'a str,
    pub private_key_pkcs8_der: &'a [u8],
    pub client_id: &'a str,
    pub scope: &'a str,
    pub body: &'a [u8],
    pub client_version: &'a str,
    pub now_seconds: u64,
    pub ttl_seconds: u64,
}

pub fn generate_es256_client_key(
    key_id: impl Into<String>,
) -> Result<CloudClientKeyMaterial, CloudContractError> {
    let rng = rand::SystemRandom::new();
    let private_key_pkcs8_der =
        signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .map_err(|_| CloudContractError::KeyGenerationFailed)?
            .as_ref()
            .to_vec();
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(
        &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
        &private_key_pkcs8_der,
        &rng,
    )
    .map_err(|_| CloudContractError::InvalidPrivateKey)?;
    let public_key_jwk = public_key_jwk_from_uncompressed_point(key_pair.public_key().as_ref())?;

    Ok(CloudClientKeyMaterial {
        key_id: key_id.into(),
        private_key_pkcs8_der,
        public_key_jwk,
    })
}

pub fn sign_request_jwt(input: CloudSignedJwtInput<'_>) -> Result<String, CloudContractError> {
    if input.ttl_seconds == 0 || input.ttl_seconds > 15 * 60 {
        return Err(CloudContractError::InvalidJwtTtl);
    }

    let rng = rand::SystemRandom::new();
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(
        &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
        input.private_key_pkcs8_der,
        &rng,
    )
    .map_err(|_| CloudContractError::InvalidPrivateKey)?;

    let header = serde_json::json!({
        "alg": "ES256",
        "kid": input.key_id,
        "typ": "JWT",
    });
    let skew_seconds = JWT_CLOCK_SKEW_SECONDS.min((15 * 60u64).saturating_sub(input.ttl_seconds));
    let valid_from = input.now_seconds.saturating_sub(skew_seconds);
    let claims = CloudJwtClaims {
        iss: format!("netstitch-client:{}", input.client_id),
        sub: input.client_id.to_string(),
        aud: JWT_AUDIENCE.to_string(),
        scope: input.scope.to_string(),
        iat: valid_from,
        nbf: valid_from,
        exp: input.now_seconds + input.ttl_seconds,
        jti: Uuid::now_v7().to_string(),
        body_sha256: payload_sha256_base64url(input.body),
        client_version: input.client_version.to_string(),
    };

    let header = base64url_json(&header)?;
    let claims = base64url_json(&claims)?;
    let signing_input = format!("{header}.{claims}");
    let signature = key_pair
        .sign(&rng, signing_input.as_bytes())
        .map_err(|_| CloudContractError::JwtSigningFailed)?;

    Ok(format!(
        "{}.{}",
        signing_input,
        URL_SAFE_NO_PAD.encode(signature.as_ref())
    ))
}

fn public_key_jwk_from_uncompressed_point(point: &[u8]) -> Result<String, CloudContractError> {
    if point.len() != 65 || point.first() != Some(&0x04) {
        return Err(CloudContractError::InvalidPrivateKey);
    }

    serde_json::to_string(&serde_json::json!({
        "kty": "EC",
        "crv": "P-256",
        "x": URL_SAFE_NO_PAD.encode(&point[1..33]),
        "y": URL_SAFE_NO_PAD.encode(&point[33..65]),
    }))
    .map_err(|_| CloudContractError::JwtSerializationFailed)
}

fn base64url_json<T: serde::Serialize>(value: &T) -> Result<String, CloudContractError> {
    serde_json::to_vec(value)
        .map(|json| URL_SAFE_NO_PAD.encode(json))
        .map_err(|_| CloudContractError::JwtSerializationFailed)
}

pub fn app_proof_allows_cloud_upload(app_proof: &VerifiedAppProof) -> bool {
    app_proof.status == VerifiedAppStatus::Verified
        && app_proof.signature_chain_valid
        && app_proof
            .leaf_spki_sha256
            .as_deref()
            .is_some_and(|value| !value.is_empty())
}

pub fn append_request_is_valid_for_upload(request: &CloudObservationAppendRequest) -> bool {
    app_proof_allows_cloud_upload(&request.app_proof)
        && !request.client_identifier.is_empty()
        && !request.rows.is_empty()
        && request.rows.len() <= CLOUD_OBSERVATION_BATCH_LIMIT
        && request.rows.iter().all(|row| {
            row.app_id == request.app_proof.app_id
                && row.source_kind == CloudSourceKind::VerifiedUpload
        })
}

fn normalize_identifier_component(value: &str) -> String {
    value
        .trim()
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| !ch.is_control())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use netstitch_shared::{CloudObservationVisibility, ConnectionState, Protocol};

    #[test]
    fn cloud_requests_require_user_action() {
        assert!(ensure_user_action(CloudRequestTrigger::UserAction).is_ok());
        assert_eq!(
            ensure_user_action(CloudRequestTrigger::Background),
            Err(CloudContractError::BackgroundCloudRequest)
        );
    }

    #[test]
    fn quota_spend_uses_local_mirror_without_exceeding_limit() {
        let mut quota = CloudQuotaSnapshot {
            operation: CloudOperationKind::Upload,
            limit_count: 3,
            used_count: 2,
            window_started_at_ms: 10,
            window_ends_at_ms: 20,
            synced_at_ms: 11,
        };

        assert_eq!(quota_remaining_count(&quota), 1);
        try_spend_local_quota_mirror(&mut quota).unwrap();
        assert_eq!(quota.used_count, 3);
        assert!(quota_is_exhausted(&quota));
        assert_eq!(
            try_spend_local_quota_mirror(&mut quota),
            Err(CloudContractError::QuotaExhausted)
        );
    }

    #[test]
    fn zero_quota_limit_means_unlimited_for_testing() {
        let mut quota = CloudQuotaSnapshot {
            operation: CloudOperationKind::Upload,
            limit_count: 0,
            used_count: 99,
            window_started_at_ms: 10,
            window_ends_at_ms: 20,
            synced_at_ms: 11,
        };

        assert!(!quota_is_exhausted(&quota));
        assert_eq!(quota_remaining_count(&quota), u32::MAX);
        try_spend_local_quota_mirror(&mut quota).unwrap();
        assert_eq!(quota.used_count, 0);
    }

    #[test]
    fn operation_scopes_cover_overlay_bootstrap_refresh() {
        assert_eq!(
            operation_scope(CloudOperationKind::QuotaCheck),
            "quota:read"
        );
        assert_eq!(operation_scope(CloudOperationKind::AppList), "apps:read");
        assert_eq!(
            operation_scope(CloudOperationKind::UserAppList),
            "apps:read:self"
        );
        assert_eq!(
            operation_scope(CloudOperationKind::HealthCheck),
            "health:read"
        );
        assert_eq!(
            operation_scope(CloudOperationKind::OAuthStart),
            "auth:oauth:start"
        );
        assert_eq!(
            operation_scope(CloudOperationKind::OAuthPoll),
            "auth:oauth:poll"
        );
        assert_eq!(quota_bucket(CloudOperationKind::AppList), None);
        assert_eq!(quota_bucket(CloudOperationKind::UserAppList), None);
        assert_eq!(quota_bucket(CloudOperationKind::HealthCheck), None);
        assert_eq!(quota_bucket(CloudOperationKind::OAuthStart), None);
    }

    #[test]
    fn observation_query_supports_public_app_slice_and_author_scope() {
        let public_query = CloudObservationQuery {
            app_id: "discord".to_string(),
            scope: CloudObservationQueryScope::AppCommunity,
            author_user_id: None,
            client_identifier: Some("nsclid_v1_test".to_string()),
            app_query: None,
            remote_ip_query: None,
            remote_port_query: None,
            protocol: None,
            source_query: None,
        };
        let author_query = CloudObservationQuery {
            app_id: "discord".to_string(),
            scope: CloudObservationQueryScope::AppByAuthor,
            author_user_id: Some("user_1".to_string()),
            client_identifier: Some("nsclid_v1_test".to_string()),
            app_query: None,
            remote_ip_query: None,
            remote_port_query: None,
            protocol: None,
            source_query: None,
        };
        let invalid_author_query = CloudObservationQuery {
            app_id: "discord".to_string(),
            scope: CloudObservationQueryScope::AppByAuthor,
            author_user_id: None,
            client_identifier: Some("nsclid_v1_test".to_string()),
            app_query: None,
            remote_ip_query: None,
            remote_port_query: None,
            protocol: None,
            source_query: None,
        };

        assert!(observation_query_is_valid(&public_query));
        assert!(observation_query_is_valid(&author_query));
        assert!(!observation_query_is_valid(&invalid_author_query));
    }

    #[test]
    fn observation_retention_expires_rows_after_one_year() {
        let last_seen_ms = 1_000;
        let expires_at_ms = observation_expires_at_ms(last_seen_ms);

        assert_eq!(
            expires_at_ms,
            last_seen_ms + DEFAULT_OBSERVATION_RETENTION_MS
        );
        assert!(!observation_is_expired(last_seen_ms, expires_at_ms - 1));
        assert!(observation_is_expired(last_seen_ms, expires_at_ms));
    }

    #[test]
    fn generated_es256_key_exports_public_jwk_and_signs_upload_jwt() {
        let key = generate_es256_client_key("key_test").unwrap();
        let public_jwk: serde_json::Value = serde_json::from_str(&key.public_key_jwk).unwrap();
        assert_eq!(public_jwk["kty"], "EC");
        assert_eq!(public_jwk["crv"], "P-256");

        let body = br#"{"rows":[]}"#;
        let token = sign_request_jwt(CloudSignedJwtInput {
            key_id: &key.key_id,
            private_key_pkcs8_der: &key.private_key_pkcs8_der,
            client_id: "cl_test",
            scope: operation_scope(CloudOperationKind::Upload),
            body,
            client_version: "1.1.0",
            now_seconds: 100,
            ttl_seconds: 60,
        })
        .unwrap();
        let parts = token.split('.').collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);

        let header: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[0]).unwrap()).unwrap();
        let claims: CloudJwtClaims =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();

        assert_eq!(header["alg"], "ES256");
        assert_eq!(header["typ"], "JWT");
        assert_eq!(header["kid"], "key_test");
        assert_eq!(claims.iss, "netstitch-client:cl_test");
        assert_eq!(claims.sub, "cl_test");
        assert_eq!(claims.aud, JWT_AUDIENCE);
        assert_eq!(claims.scope, "observations:append");
        assert_eq!(claims.iat, 70);
        assert_eq!(claims.nbf, 70);
        assert_eq!(claims.exp, 160);
        assert_eq!(claims.body_sha256, payload_sha256_base64url(body));
    }

    #[test]
    fn request_jwt_rejects_too_long_ttl() {
        let key = generate_es256_client_key("key_test").unwrap();
        assert_eq!(
            sign_request_jwt(CloudSignedJwtInput {
                key_id: &key.key_id,
                private_key_pkcs8_der: &key.private_key_pkcs8_der,
                client_id: "cl_test",
                scope: operation_scope(CloudOperationKind::Upload),
                body: b"{}",
                client_version: "1.1.0",
                now_seconds: 100,
                ttl_seconds: 901,
            }),
            Err(CloudContractError::InvalidJwtTtl)
        );
    }

    #[test]
    fn client_identifier_is_stable_for_reordered_signals() {
        let salt = b"netstitch-test-salt";
        let first = CloudClientIdentifierInput {
            os_family: "Windows".to_string(),
            os_version: "11".to_string(),
            machine_signals: vec!["Board-A".to_string(), "Machine-B".to_string()],
        };
        let second = CloudClientIdentifierInput {
            os_family: " windows ".to_string(),
            os_version: "11".to_string(),
            machine_signals: vec!["machine-b".to_string(), " board-a ".to_string()],
        };

        assert_eq!(
            derive_client_identifier(&first, salt).unwrap(),
            derive_client_identifier(&second, salt).unwrap()
        );
    }

    #[test]
    fn client_identifier_rejects_empty_inputs() {
        let input = CloudClientIdentifierInput {
            os_family: "windows".to_string(),
            os_version: "11".to_string(),
            machine_signals: vec![" ".to_string()],
        };

        assert_eq!(
            derive_client_identifier(&input, b"salt"),
            Err(CloudContractError::EmptyClientSignals)
        );
        assert_eq!(
            derive_client_identifier(&input, b""),
            Err(CloudContractError::EmptyIdentifierSalt)
        );
    }

    #[test]
    fn append_request_requires_verified_app_and_matching_rows() {
        let app_proof = VerifiedAppProof {
            app_id: "discord".to_string(),
            platform: "windows".to_string(),
            method: VerifiedAppMethod::StableFileIdentity,
            status: VerifiedAppStatus::Verified,
            signature_chain_valid: true,
            leaf_spki_sha256: Some("abc".to_string()),
            subject: None,
            issuer: None,
            display_name: Some("Discord".to_string()),
            publisher_name: None,
            short_app_id: None,
        };
        let row = CloudObservationRow {
            app_id: "discord".to_string(),
            author_signature: None,
            remote_ip: "203.0.113.10".parse().unwrap(),
            remote_port: 443,
            protocol: Protocol::Tcp,
            connection_state: ConnectionState::Established,
            requests: 2,
            first_seen_ms: 1,
            last_seen_ms: 2,
            failed_hits: 0,
            successful_hits: 2,
            domain_raw: None,
            domain_verified: None,
            domain_status: CloudDomainStatus::None,
            trust_level: CloudTrustLevel::VerifiedUploadCandidate,
            source_kind: CloudSourceKind::VerifiedUpload,
            app_signature_key: Some("abc".to_string()),
            app_signature_subject: None,
            app_signature_issuer: None,
            cloud_observation_id: None,
        };

        let request = CloudObservationAppendRequest {
            client_identifier: "nsclid_v1_test".to_string(),
            client_version: "1.1.0".to_string(),
            author_signature: "Publisher1".to_string(),
            visibility: CloudObservationVisibility::Public,
            operation_id: None,
            app_proof,
            rows: vec![row.clone()],
        };

        assert!(append_request_is_valid_for_upload(&request));

        let mut too_large_request = request;
        too_large_request.rows = vec![row; CLOUD_OBSERVATION_BATCH_LIMIT + 1];
        assert!(!append_request_is_valid_for_upload(&too_large_request));
    }
}
