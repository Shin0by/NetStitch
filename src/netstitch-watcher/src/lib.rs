mod monitor;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use axum::extract::{Path as AxumPath, Query, RawQuery, State};
use axum::http::{
    HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode,
    header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, ORIGIN, REFERER, SET_COOKIE, USER_AGENT},
};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use monitor::MonitorController;
use netstitch_cloud::{
    CloudClientIdentifierInput, CloudObservationAppendRequest, CloudOperationKind,
    CloudQuotaSnapshot, CloudSignedJwtInput, CloudSourceKind, CloudTrustLevel,
    CloudUserSessionResponse, VerifiedAppMethod, VerifiedAppProof, VerifiedAppStatus,
    derive_client_identifier, operation_scope, sign_request_jwt,
};
use netstitch_core::NetstitchCore;
use netstitch_shared::ipc::{
    AddIgnoredAddressRequest, AddTrackedAppRequest, ConfirmEndpointsRequest,
    DeleteIgnoredAddressRequest, DeleteObservationRequest, DeleteObservationsRequest,
    DeleteTrackedAppRequest, DownloadIntegrationProviderRequest, ExportConfirmedRequest,
    MarkEndpointsExportedRequest, SetAllTrackedAppsEnabledRequest, SetAppSettingRequest,
    SetTrackedAppEnabledRequest,
};
use netstitch_shared::models::{
    CLIENT_HEADER_DESKTOP_UI, CLIENT_HEADER_NAME, EndpointProbeStatusDto, EndpointProbeTargetDto,
    ExportProfileAdvancedSettingsRequestDto, ExportProfileRequestDto,
    IntegrationDownloadProgressDto, IntegrationHostEvent,
    IntegrationModuleUiActionClientRequestDto, IntegrationModuleUiActionEventDto,
    IntegrationModuleUiActionEventsResponseDto, MonitorStatus, MonitoringCsvImportRequestDto,
    MonitoringImportSourceDto, ProfileExportUiStateDto, Protocol, SETTING_UI_MONITORING_PUBLIC_IP,
    SnapshotResponse, SystemEventRequestDto, TrackedApp, UiFiltersDto,
};
use netstitch_shared::{
    CloudObservationRow, CloudObservationVisibility, CloudObservationVisibilityScope,
    StableAppIdentityInput, derive_web_access_key, runtime_build_version, runtime_module_version,
    runtime_platform_label, stable_app_identity,
};
use rcgen::{CertificateParams, DnType, KeyPair, PKCS_RSA_SHA256, date_time_ymd};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{error, info};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::IsUserAnAdmin;

const DEFAULT_WATCHER_PORT: u16 = 46473;
const APP_ENV_WATCHER_ADDR: &str = "NETSTITCH__WATCHER_ADDR";
const APP_ENV_LANGUAGE_DIR: &str = "NETSTITCH__LANGUAGE_DIR";
const APP_ENV_ENDPOINT_PROBE_TARGETS: &str = "NETSTITCH__ENDPOINT_PROBE_TARGETS";
const APP_ENV_ENDPOINT_PROBE_TARGETS_FILE: &str = "NETSTITCH__ENDPOINT_PROBE_TARGETS_FILE";
const APP_ENV_WEB_SCHEME: &str = "NETSTITCH__WEB_SCHEME";
const CLOUD_BASE_URL_ENV: &str = "NETSTITCH__CLOUD_BASE_URL";
const WEB_CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'";
const DEFAULT_CLOUD_BASE_URL: &str = "https://netstitch-sync.warfactory.workers.dev";
const CLIENT_IDENTIFIER_SALT: &[u8] = b"netstitch-community-cloud-v1";
const CLOUD_REFRESH_HTTP_TIMEOUT: Duration = Duration::from_millis(9500);
const CLOUD_INTERACTIVE_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const SETTING_CLOUD_PROVIDER: &str = "cloud.auth.provider";
const SETTING_CLOUD_EMAIL_DPAPI: &str = "cloud.auth.email.dpapi.v1";
const SETTING_CLOUD_DISPLAY_NAME_DPAPI: &str = "cloud.auth.display_name.dpapi.v1";
const SETTING_CLOUD_USER_ID: &str = "cloud.auth.user_id";
const SETTING_CLOUD_CLIENT_ID: &str = "cloud.auth.client_id";
const SETTING_CLOUD_KEY_ID: &str = "cloud.auth.key_id";
const SETTING_CLOUD_SESSION_TOKEN_DPAPI: &str = "cloud.auth.session_token.dpapi.v1";
const SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI: &str = "cloud.auth.client_private_key.dpapi.v1";
const SETTING_CLOUD_SESSION_EXPIRES_AT_MS: &str = "cloud.auth.session_expires_at_ms";
const SETTING_CLOUD_UPLOAD_NICKNAME: &str = "cloud.upload.nickname";
#[cfg(windows)]
const CLOUD_SECRET_ENTROPY: &[u8] = b"NetStitch cloud local auth secret v1";
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
const ENDPOINT_PROBE_TARGETS_FILE: &str = "endpoint_probe_targets.txt";
const WEB_TLS_CERT_FILE: &str = "web-server.cert.pem";
const WEB_TLS_KEY_FILE: &str = "web-server.key.pem";
const WEB_AUTH_COOKIE: &str = "netstitch_web_auth";
const DEFAULT_LANGUAGE_CODE: &str = "en-en";
const SVG_CONTENT_TYPE: &str = "image/svg+xml; charset=utf-8";
const PNG_CONTENT_TYPE: &str = "image/png";
const ICO_CONTENT_TYPE: &str = "image/x-icon";
const CLOSE_TIMES_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/close-times-svgrepo-com.svg");
const MODULE_CLOSE_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/logout-svgrepo-com.svg");
const MODULE_STOP_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/stop-circle-cross.svg");
const MODULE_REORDER_LEFT_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/left-svgrepo-com.svg");
const COPY_ICON_SVG: &[u8] = include_bytes!("../../../resources/ui/icons/copy.svg");
const CONFIRM_FILTERED_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/filter-add-solid-svgrepo-com.svg");
const UNCONFIRM_FILTERED_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/filter-remove-solid-svgrepo-com.svg");
const IGNORE_ADDRESS_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/eye-close-solid-svgrepo-com.svg");
const HELP_CIRCLE_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/help-circle-solid-svgrepo-com.svg");
const TRASH_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/trash-solid-svgrepo-com.svg");
const EXPORT_CLOUD_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/upload-to-cloud-svgrepo-com.svg");
const IMPORT_FROM_CLOUD_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/import-from-cloud-svgrepo-com.svg");
const MY_PUBLICATIONS_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/torso-svgrepo-com.svg");
const IMPORT_CSV_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/csv-import-svgrepo-com.svg");
const EXPORT_CSV_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/csv-export-svgrepo-com.svg");
const MONITORING_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/open-eye-svgrepo-com.svg");
const SORT_IDLE_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/sort-svgrepo-com.svg");
const SORT_ASC_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/sort-up-svgrepo-com.svg");
const SORT_DESC_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/ui/icons/sort-down-svgrepo-com.svg");
const DEFAULT_APP_ICON_SVG: &[u8] =
    include_bytes!("../../../resources/connectors/icons/default.svg");
const BROWSER_UI_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NetStitch Web __BUILD_VERSION__ (__RUNTIME_PLATFORM__)</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #181818;
      --chrome: #2d2d2d;
      --panel: #252526;
      --list: #1e1e1e;
      --row: #2b2b2b;
      --row-on: #2b3338;
      --control: #1f1f1f;
      --control-hover: #2a2d2e;
      --border: #3c3c3c;
      --border-strong: #4a4a4a;
      --text: #cccccc;
      --strong: #f0f0f0;
      --muted: #9d9d9d;
      --accent: #007acc;
      --accent-hover: #0e86d4;
      --progress-accent: #007acc;
      --progress-success: #6a9955;
      --progress-warning: #cca700;
      --progress-danger: #f48771;
      --progress-rust: #d97922;
      --progress-muted: #5a5a5a;
      --success: #89d185;
      --success-bg: #123a2f;
      --danger: #f48771;
      --danger-bg: #4b1f24;
      --warning: #cca700;
      --row-hover: #262c31;
      --row-enabled-hover: #344048;
      --scrollbar-track: #1e1e1e;
      --scrollbar-thumb: #424242;
      --scrollbar-thumb-hover: #525252;
      --transparent: transparent;
      --radius: 3px;
      --radius-pill: 999px;
      --panel-gap: 8px;
      --panel-stack-gap: 8px;
      --scrollbar-size: 8px;
      --size-header-height: 56px;
      --size-footer-height: 34px;
      --size-compact-control: 30px;
      --size-panel-footer-height: 43px;
      --size-header-action-button: 48px;
      --size-switch-width: 46px;
      --size-switch-height: 22px;
      --size-close-button: 20px;
      --size-app-icon-slot: 40px;
      --size-control-label-offset: 0px;
      --tracked-app-visible-rows: 4;
      --tracked-app-list-height: calc((52px * var(--tracked-app-visible-rows)) + (5px * (var(--tracked-app-visible-rows) - 1)) + 10px + 2px);
      --tracked-apps-path-bottom-pad: 0px;
      --modules-panel-height: 78px;
      --module-button-size: 36px;
      --module-button-pulse-gutter: 3px;
      --module-button-slot: calc(var(--module-button-size) + (var(--module-button-pulse-gutter) * 2));
      --module-activity-pulse: rgba(255, 255, 255, 0.34);
      --ignored-address-row-height: 30px;
      --ignored-address-row-gap: 5px;
      --ignored-addresses-visible-rows: 5;
      --ignored-addresses-list-height: calc((var(--ignored-address-row-height) * var(--ignored-addresses-visible-rows)) + (var(--ignored-address-row-gap) * (var(--ignored-addresses-visible-rows) - 1)) + 10px + 2px);
      --ignored-address-ip-column: 230px;
      --size-integration-status-left-max: 520px;
    }
    * { box-sizing: border-box; }
    * {
      scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
      scrollbar-width: thin;
    }
    *::-webkit-scrollbar {
      width: var(--scrollbar-size);
      height: var(--scrollbar-size);
    }
    *::-webkit-scrollbar-track {
      background: var(--scrollbar-track);
    }
    *::-webkit-scrollbar-thumb {
      border-radius: var(--radius);
      background: var(--scrollbar-thumb);
    }
    *::-webkit-scrollbar-thumb:hover {
      background: var(--scrollbar-thumb-hover);
    }
    html {
      width: 100%;
      height: 100%;
      background: var(--bg);
      overflow: hidden;
    }
    body {
      margin: 0;
      width: 100%;
      height: 100%;
      min-height: 100vh;
      background: var(--bg);
      color: var(--text);
      font: 13px "Segoe UI", system-ui, sans-serif;
      overflow: hidden;
    }
    .shell {
      position: fixed;
      inset: 0;
      display: grid;
      grid-template-rows: var(--size-header-height) minmax(0, 1fr) var(--size-footer-height);
      width: 100%;
      min-width: 100%;
      min-height: 100%;
      height: 100vh;
      height: 100dvh;
      min-width: 0;
      overflow: hidden;
      background: var(--bg);
    }
    .shell--controls-disabled [data-ui-action],
    .shell--controls-disabled button,
    .shell--controls-disabled select,
    .shell--controls-disabled textarea,
    .shell--controls-disabled input:not([readonly]),
    .shell--controls-disabled [role="button"] {
      pointer-events: none;
      cursor: not-allowed;
      opacity: 0.55;
    }
    header,
    footer {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      padding: 8px 22px;
      background: var(--chrome);
      border-color: var(--border);
    }
    header { justify-content: flex-start; align-items: flex-end; min-height: var(--size-header-height); height: auto; padding: 4px var(--panel-gap); border-bottom: 1px solid var(--border); }
    footer { justify-content: flex-start; min-height: var(--size-footer-height); height: var(--size-footer-height); padding-block: 0; border-top: 1px solid var(--border); }
    footer { overflow: visible; }
    h1, h2, h3, p { margin: 0; }
    h1 { color: var(--strong); font-size: 24px; line-height: 28px; letter-spacing: .02em; }
    h2 { color: var(--strong); font-size: 17px; line-height: 18px; }
    h3 { color: var(--strong); font-size: 13px; line-height: 18px; }
    .small { font-size: 13px; }
    .content {
      display: flex;
      flex: 1 1 auto;
      flex-direction: column;
      width: 100%;
      height: 100%;
      min-height: 0;
      min-width: 0;
      overflow: hidden;
      padding: 8px;
    }
    .workspace {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      grid-template-rows: auto minmax(0, 1fr);
      gap: var(--panel-gap);
      align-items: start;
      flex: 1 1 auto;
      height: 100%;
      min-height: 0;
      min-width: 0;
      max-width: 100%;
    }
    [hidden] {
      display: none !important;
    }
    .workspace,
    .workspace > .column,
    .workspace section,
    .card,
    .card__header,
    .row,
    .list,
    .table-wrap,
    .integration-status,
    .integration-status-layout,
    .integration-status__fields,
    .integration-status__action,
    .app-main,
    .app-actions,
    .footer-pill {
      min-width: 0;
      max-width: 100%;
    }
    .column { display: flex; flex-direction: column; gap: var(--panel-gap); min-width: 0; max-width: 100%; }
    .workspace > .column:first-child {
      grid-column: 1;
      grid-row: 1;
      align-self: stretch;
      height: 100%;
      min-height: 0;
    }
    .workspace > .column:nth-child(2) {
      grid-column: 2;
      grid-row: 1;
      align-self: stretch;
      height: 100%;
      gap: var(--panel-stack-gap);
      min-height: 0;
    }
    .workspace > .observations-card {
      grid-column: 1 / -1;
      grid-row: 2;
      align-self: stretch;
      height: 100%;
      min-height: 0;
    }
    .workspace > .cloud-web-panel {
      grid-column: 1 / -1;
      grid-row: 1 / -1;
      align-self: stretch;
      height: 100%;
      min-height: 0;
      position: relative;
      z-index: 2;
    }
    .cloud-web-panel {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr) auto;
      gap: 5px;
      padding: 5px;
    }
    .cloud-web-panel[hidden] {
      display: none;
    }
    .cloud-web-panel h2 {
      margin: 0;
      color: var(--strong);
      font-size: 17px;
      line-height: 18px;
    }
    .cloud-web-panel-service {
      display: inline-flex;
      align-items: center;
      justify-content: flex-end;
      gap: 6px;
      min-width: 0;
      color: var(--muted);
    }
    .cloud-web-panel-service__text {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .cloud-web-panel-body {
      display: grid;
      gap: 8px;
      min-height: 0;
      overflow: hidden;
    }
    .cloud-web-import-layout {
      display: grid;
      grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
      gap: var(--panel-gap);
      flex: 1 1 auto;
      min-height: 0;
    }
    .cloud-web-export-body {
      grid-template-rows: auto minmax(0, 1fr);
      min-height: 0;
      overflow: hidden;
    }
    .cloud-web-export-auth-subpanel,
    .cloud-web-export-publications-subpanel {
      display: flex;
      flex-direction: column;
      gap: 7px;
      padding: 7px;
      background: var(--panel);
      min-height: 0;
      overflow: hidden;
    }
    .cloud-web-auth-row {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 8px;
      min-height: 30px;
    }
    .cloud-web-auth-user {
      min-width: 0;
      overflow: hidden;
      color: var(--strong);
      font-size: 13px;
      font-weight: 600;
      line-height: 17px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .cloud-web-nickname-row {
      display: grid;
      grid-template-columns: minmax(180px, 360px) auto minmax(80px, 1fr);
      gap: 8px;
      align-items: center;
    }
    .cloud-web-nickname-status {
      min-height: 16px;
      font-size: 12px;
      line-height: 16px;
      white-space: nowrap;
    }
    .cloud-web-nickname-status--invalid { color: var(--danger); }
    .cloud-web-nickname-status--taken { color: var(--warning); }
    .cloud-web-nickname-status--accepted { color: var(--success); }
    .cloud-web-publications-table {
      min-height: 0;
    }
    .cloud-sync-publications-title {
      color: var(--text);
      font-weight: 700;
      min-height: 20px;
    }
    .cloud-web-publications-data-table {
      table-layout: fixed;
      min-width: 0;
    }
    .cloud-web-publications-data-table th:nth-child(2),
    .cloud-web-publications-data-table td:nth-child(2),
    .cloud-web-publications-data-table th:nth-child(3),
    .cloud-web-publications-data-table td:nth-child(3),
    .cloud-web-publications-data-table th:nth-child(4),
    .cloud-web-publications-data-table td:nth-child(4) {
      width: 130px;
      text-align: left;
    }
    .state-label--success {
      color: var(--success);
      font-weight: 700;
    }
    .cloud-web-publications-data-table .state-label--success {
      color: var(--success);
      font-weight: 700;
    }
    .cloud-web-app-row {
      display: grid;
      gap: 2px;
      padding: 5px 6px;
      border-bottom: 1px solid var(--border);
    }
    .cloud-web-app-row strong {
      color: var(--strong);
      font-weight: 600;
      overflow-wrap: anywhere;
    }
    .cloud-web-app-row span {
      overflow-wrap: anywhere;
    }
    .cloud-web-filter-grid {
      display: grid;
      grid-template-columns: minmax(120px, 30fr) minmax(120px, 30fr) minmax(160px, 38fr) 108px var(--size-header-action-button);
      gap: 8px;
      align-self: stretch;
      width: 100%;
      max-width: 100%;
      padding: 5px;
      align-items: end;
    }
    .cloud-web-filter-block {
      display: inline-flex;
      flex-direction: column;
      justify-content: space-between;
      gap: 2px;
      height: var(--size-header-action-button);
      min-width: 0;
    }
    .cloud-web-filter-grid .path-input-shell,
    .cloud-web-filter-grid .input-box {
      width: 100%;
      min-width: 0;
      box-sizing: border-box;
    }
    .cloud-web-filter-grid .path-input-shell > .input {
      height: var(--size-compact-control);
      min-height: var(--size-compact-control);
      padding-right: 30px;
      font-size: 12px;
      line-height: 16px;
    }
    .cloud-web-my-publications-button {
      align-self: end;
      justify-self: end;
    }
    .button__icon--my-publications {
      width: 30px;
      height: 30px;
    }
    .cloud-web-panel-footer {
      display: flex;
      align-items: center;
      gap: 5px;
      height: var(--size-panel-footer-height);
      min-height: var(--size-panel-footer-height);
      max-height: var(--size-panel-footer-height);
      margin: 0 -5px -5px;
      padding: 6px 8px;
      overflow: hidden;
      border-top: 1px solid var(--border);
      border-radius: 0 0 var(--radius-control) var(--radius-control);
      background: var(--chrome);
      color: var(--muted);
      font-size: 12px;
    }
    .cloud-web-panel-footer__progress {
      flex: 0 1 240px;
      width: 240px;
      min-width: 0;
      height: 18px;
      align-self: center;
    }
    .cloud-web-panel-footer__progress .progress-bar--compact .progress-bar__track {
      height: 8px;
      border-radius: 4px;
    }
    .cloud-web-panel-footer__progress .progress-bar--compact .progress-bar__segment,
    .cloud-web-panel-footer__progress .progress-bar--compact .progress-bar__remaining {
      height: 8px;
      border-radius: 4px;
    }
    .cloud-web-panel-footer__spacer {
      flex: 1 1 auto;
    }
    .cloud-sync-footer-identifier,
    .cloud-sync-footer-auth {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      min-width: 0;
      color: var(--muted);
      white-space: nowrap;
    }
    .cloud-sync-footer-identifier {
      cursor: pointer;
    }
    .cloud-sync-footer-identifier__label {
      color: var(--muted);
    }
    .cloud-sync-footer-identifier__value {
      max-width: min(360px, 24vw);
      overflow: hidden;
      color: var(--strong);
      font-weight: 600;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .cloud-web-subpanel {
      display: flex;
      flex-direction: column;
      min-height: 0;
      min-width: 0;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      overflow: hidden;
    }
    .cloud-web-subpanel .table-wrap {
      flex: 1 1 auto;
      min-height: 0;
      border: 0;
    }
    .cloud-web-action-header,
    .cloud-web-action-cell {
      width: 32px;
      min-width: 32px;
      max-width: 32px;
      text-align: center;
    }
    .card {
      display: flex;
      flex-direction: column;
      gap: 5px;
      padding: 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--panel);
      min-width: 0;
    }
    .card.cloud-web-panel {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr) auto;
      align-items: stretch;
      height: 100%;
      min-height: 0;
      overflow: hidden;
    }
    .card.cloud-web-panel > .card__header,
    .card.cloud-web-panel > .cloud-web-panel-footer {
      flex: 0 0 auto;
    }
    .card.cloud-web-panel > .cloud-web-panel-body,
    .card.cloud-web-panel > .cloud-web-import-layout {
      min-height: 0;
      overflow: hidden;
    }
    .card__header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 5px;
      min-height: 18px;
      width: calc(100% + 10px);
      max-width: none;
      align-self: stretch;
      box-sizing: border-box;
      margin: -5px -5px 0;
      padding: 6px 8px;
      border: 0;
      border-bottom: 1px solid var(--border);
      border-radius: var(--radius-control) var(--radius-control) 0 0;
      background: var(--chrome);
    }
    .subtle { color: var(--muted); font-size: 12px; line-height: 16px; }
    .hero-labels {
      display: flex;
      flex-wrap: wrap;
      align-items: flex-end;
      width: 100%;
      gap: 8px;
      justify-content: space-between;
      align-content: flex-end;
      min-height: 0;
    }
    .module-header-content {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      width: 100%;
      min-width: 0;
    }
    .module-header-title {
      min-width: 0;
      overflow: hidden;
      color: var(--strong);
      font-size: 17px;
      font-weight: 700;
      line-height: 20px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .module-header-controls,
    .module-header-actions,
    .module-header-standard-actions {
      display: flex;
      align-items: center;
      gap: 5px;
      min-width: 0;
    }
    .module-header-controls {
      flex: 0 0 auto;
      margin-left: auto;
    }
    .module-header-standard-actions {
      flex: 0 0 auto;
    }
    .panel-header-meta {
      color: var(--muted);
      font-size: 12px;
      font-weight: 700;
      line-height: 18px;
      white-space: nowrap;
    }
    .panel-header-meta--success { color: var(--success); }
    .panel-header-meta--danger { color: var(--danger); }
    .monitoring-header-meta {
      display: inline-flex;
      align-items: center;
      justify-content: flex-end;
      gap: 10px;
      min-width: 0;
      margin-left: auto;
    }
    .monitoring-header-meta .panel-header-meta {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .monitoring-public-toggle {
      flex: 0 0 auto;
      min-height: var(--size-switch-height);
      align-items: flex-end;
    }
    .tracked-apps-header-meta {
      display: inline-flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      min-width: 0;
    }
    .header-switch-stack {
      display: flex;
      flex: 0 0 auto;
      flex-direction: column;
      align-items: flex-end;
      justify-content: flex-end;
      gap: 3px;
      margin-left: auto;
      align-self: flex-end;
      min-width: 0;
      overflow: hidden;
    }
    .header-monitor-controls {
      display: flex;
      flex-wrap: nowrap;
      align-items: flex-end;
      gap: 5px;
      flex: 1 1 auto;
      min-width: 0;
    }
    .header-filter-block {
      display: inline-flex;
      flex-direction: column;
      justify-content: space-between;
      gap: 2px;
      height: var(--size-header-action-button);
      min-width: 0;
    }
    .header-filter-field-shell {
      width: 100%;
      min-width: 0;
    }
    .header-filter-block__label {
      min-height: 16px;
      padding: 0 var(--size-control-label-offset);
      color: var(--muted);
      font-size: 11px;
      font-weight: 600;
      line-height: 16px;
      white-space: nowrap;
    }
    .header-filter-select,
    .header-filter-field,
    .header-search-field {
      width: 100%;
      height: var(--size-compact-control);
      min-height: var(--size-compact-control);
      min-width: 0;
      padding: 3px 8px;
      font-size: 12px;
      line-height: 16px;
    }
    .header-filter-block--app { flex: 0 1 170px; min-width: 132px; max-width: 170px; }
    .header-filter-block--ip { flex: 0 1 126px; min-width: 102px; max-width: 126px; }
    .header-filter-block--domain { flex: 0 1 126px; min-width: 102px; max-width: 126px; }
    .header-filter-block--port { flex: 0 1 66px; min-width: 66px; max-width: 66px; }
    .header-filter-block--protocol { flex: 0 1 68px; min-width: 64px; max-width: 68px; }
    .header-filter-block--state { flex: 0 1 94px; min-width: 74px; max-width: 94px; }
    .header-action-button { white-space: nowrap; }
    .header-action-separator {
      display: block;
      flex: 0 0 1px;
      align-self: center;
      width: 1px;
      min-width: 1px;
      height: 44px;
      margin: 0;
      background: var(--border-strong);
      opacity: 1;
    }
    .button--icon.header-action-button {
      width: var(--size-header-action-button);
      height: var(--size-header-action-button);
      min-width: var(--size-header-action-button);
      min-height: var(--size-header-action-button);
      max-width: var(--size-header-action-button);
      max-height: var(--size-header-action-button);
    }
    .header-action-button.button:not(.button--icon) { height: var(--size-header-action-button); padding-inline: 14px; }
    .header-label,
    .footer-pill,
    .app-footer-panel {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      min-height: 24px;
      padding: 3px 0;
      color: var(--text);
      white-space: nowrap;
    }
    .header-label { padding: 0; font-weight: 600; }
    .header-label--accent { color: var(--accent-border); }
    .header-label--success { color: var(--success); }
    .header-label--danger { color: var(--danger); }
    .header-switch-row {
      display: inline-flex;
      align-items: flex-end;
      gap: 4px;
      min-height: var(--size-header-action-button);
      padding: 0;
      border: 0;
      background: var(--transparent);
      color: var(--text);
      align-self: flex-end;
      white-space: nowrap;
    }
    .header-switch-stack .header-switch-row {
      min-height: var(--size-switch-height);
    }
    .header-switch-row__label {
      max-width: 132px;
      overflow: hidden;
      color: var(--text);
      font-size: 12px;
      font-weight: 600;
      line-height: var(--size-switch-height);
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .footer-pill,
    .app-footer-panel {
      border: 0;
      border-radius: 0;
      background: var(--transparent);
      color: inherit;
    }
    .footer-pill + .footer-pill,
    .app-footer-panel + .app-footer-panel {
      margin-left: 6px;
      padding-left: 12px;
      border-left: 1px solid var(--border);
    }
    .footer-pill .select {
      width: auto;
      min-width: 126px;
      min-height: 24px;
      padding: 2px 8px;
      appearance: none;
      cursor: pointer;
      text-align: center;
      text-align-last: center;
    }
    .app-footer-actions {
      display: inline-flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      margin-left: auto;
      white-space: nowrap;
    }
    .footer-message-panel,
    .app-footer-message-panel {
      position: relative;
      flex: 1 1 auto;
      justify-content: flex-start;
      min-width: 0;
      overflow: visible;
    }
    .footer-message-label { color: var(--text); flex: 0 0 auto; line-height: 18px; }
    .footer-message-text {
      position: relative;
      flex: 1 1 auto;
      min-width: 0;
      overflow: visible;
      line-height: 18px;
      white-space: nowrap;
    }
    .footer-message-text--success { color: var(--success); }
    .footer-message-text--warning { color: var(--warning); }
    .footer-message-text--error { color: var(--danger); }
    .button--copy-status {
      width: 20px;
      height: 20px;
      min-height: 20px;
      padding: 0;
      flex: 0 0 20px;
      background: var(--accent);
      border-color: var(--accent);
      color: #ffffff;
    }
    .button--copy-status:hover,
    .button--copy-status:focus {
      background: var(--accent-hover);
      border-color: var(--accent-hover);
    }
    button[data-tooltip],
    .footer-watcher-status,
    .footer-tool-status,
    .footer-web-server-status,
    .footer-network-status,
    .footer-message-text {
      position: relative;
      overflow: visible;
    }
    .footer-language-panel,
    .app-footer-language-panel {
      flex: 0 0 auto;
      margin-left: 6px;
      margin-inline-start: 6px;
      overflow: visible;
    }
    .language-select-shell {
      position: relative;
      display: inline-flex;
      overflow: visible;
      z-index: 2;
    }
    .language-select-control {
      position: relative;
      display: inline-grid;
      grid-template-columns: minmax(0, 1fr);
      align-items: center;
      justify-content: center;
      width: max-content;
      min-width: 126px;
      height: 24px;
      margin: 0;
      padding: 2px 8px;
      border-radius: var(--radius);
      appearance: none;
      font-size: 12px;
      font: inherit;
      line-height: 18px;
      cursor: pointer;
    }
    .language-select-control__label {
      min-width: 0;
      overflow: hidden;
      color: var(--text);
      text-align: center;
      text-overflow: ellipsis;
      white-space: nowrap;
      pointer-events: none;
    }
    .language-select-menu {
      position: absolute;
      right: 0;
      bottom: 100%;
      display: grid;
      width: max-content;
      min-width: 126px;
      max-width: min(240px, calc(100vw - 24px));
      max-height: 220px;
      overflow: auto;
      padding: 3px;
      border: 1px solid var(--border);
      border-radius: var(--radius) var(--radius) 0 0;
      background: var(--control);
      z-index: 10000;
    }
    .language-select-menu[hidden] { display: none; }
    .language-select-option {
      min-height: 24px;
      padding: 3px 8px;
      border: 0;
      border-top: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--transparent);
      color: var(--text);
      font: inherit;
      line-height: 18px;
      cursor: pointer;
      text-align: center;
      white-space: nowrap;
    }
    .language-select-option:first-child { border-top: 0; }
    .language-select-option:hover,
    .language-select-option:focus { background: var(--row-hover); color: var(--strong); }
    .language-select-option--selected {
      background: var(--row-on);
      color: var(--strong);
    }
    .language-select-option--selected:hover,
    .language-select-option--selected:focus {
      background: var(--row-enabled-hover);
      color: var(--strong);
    }
    .footer-watcher-status,
    .footer-tool-status,
    .footer-network-status { cursor: help; }
    .footer-web-server-status { cursor: pointer; user-select: none; }
    .footer-watcher-dot {
      width: 14px;
      height: 14px;
      border: 1px solid var(--border);
      border-radius: 999px;
      flex: 0 0 14px;
      transform: translateY(0);
    }
    .footer-watcher-dot--connected { background: var(--success); }
    .footer-watcher-dot--disconnected { background: var(--danger); }
    .footer-watcher-dot--inactive { background: #3a3a3a; }
    .footer-watcher-status[data-tooltip]::after,
    .footer-tool-status[data-tooltip]::after,
    .footer-web-server-status[data-tooltip]::after,
    .footer-network-status[data-tooltip]::after,
    button[data-tooltip]::after,
    .button--copy-status[data-tooltip]::after,
    .footer-message-text[data-tooltip]::after {
      display: none;
    }
    .NetStitch-floating-tooltip {
      position: fixed;
      top: 0;
      left: 0;
      width: max-content;
      max-width: none;
      padding: 6px 8px;
      border: 1px solid var(--border-strong);
      border-radius: var(--radius);
      background: var(--control-hover);
      color: var(--text);
      white-space: normal;
      opacity: 0;
      visibility: hidden;
      pointer-events: none;
      z-index: 10000;
    }
    .NetStitch-floating-tooltip--visible {
      opacity: 1;
      visibility: visible;
    }
    .NetStitch-floating-tooltip--pre {
      white-space: pre;
    }
    .footer-watcher-status[data-tooltip]:hover::after,
    .footer-watcher-status[data-tooltip]:focus::after,
    .footer-tool-status[data-tooltip]:hover::after,
    .footer-tool-status[data-tooltip]:focus::after,
    .footer-web-server-status[data-tooltip]:hover::after,
    .footer-web-server-status[data-tooltip]:focus::after,
    .footer-network-status[data-tooltip]:hover::after,
    .footer-network-status[data-tooltip]:focus::after,
    button[data-tooltip]:hover::after,
    button[data-tooltip]:focus::after,
    .button--copy-status[data-tooltip]:hover::after,
    .button--copy-status[data-tooltip]:focus::after,
    .footer-message-text[data-tooltip]:hover::after,
    .footer-message-text[data-tooltip]:focus::after { opacity: 1; }
    button,
    a,
    [role="button"],
    [data-ui-action] { cursor: pointer; }
    .input-box,
    .input,
    .select,
    .button,
    .switch {
      min-height: 30px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--control);
      color: var(--text);
      font: inherit;
    }
    .input,
    .select { width: 100%; padding: 6px 10px; outline: none; }
    .input:focus,
    .select:focus { border-color: #3794ff; }
    .input--apply-pulse {
      animation: input-apply-pulse 180ms ease-out 1;
    }
    @keyframes input-apply-pulse {
      0% {
        border-color: #3794ff;
        box-shadow: 0 0 0 0 rgba(55, 148, 255, 0);
      }
      45% {
        border-color: #3794ff;
        box-shadow: 0 0 0 2px rgba(55, 148, 255, 0.35);
      }
      100% {
        border-color: var(--border);
        box-shadow: 0 0 0 0 rgba(55, 148, 255, 0);
      }
    }
    .path-input-shell {
      position: relative;
      display: grid;
      grid-template-columns: minmax(0, 1fr) 20px;
      align-items: center;
      width: 100%;
      min-width: 0;
      overflow: visible;
    }
    .path-input-shell > .input {
      grid-column: 1 / -1;
      grid-row: 1;
      display: block;
      width: 100%;
      padding-right: 30px;
    }
    .path-input-shell > .header-search-field {
      grid-column: 1 / -1;
      grid-row: 1;
      display: block;
      width: 100%;
      padding-right: 30px;
    }
    .path-input-clear {
      grid-column: 2;
      grid-row: 1;
      justify-self: end;
      align-self: center;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 20px;
      height: 20px;
      min-width: 20px;
      min-height: 20px;
      place-items: center;
      border: 0;
      border-radius: var(--radius);
      background: transparent;
      padding: 0;
      cursor: pointer;
      opacity: 0.72;
      z-index: 1;
      margin-right: 5px;
    }
    .path-input-clear:hover,
    .path-input-clear:focus {
      background: var(--control-hover);
      opacity: 1;
      outline: none;
    }
    .path-input-clear:disabled {
      cursor: default;
      opacity: 0.32;
      pointer-events: none;
    }
    .path-input-clear__glyph {
      display: inline-block;
      color: var(--text);
      font-size: 18px;
      font-weight: 400;
      line-height: 1;
      transform: translateY(-1px);
    }
    .tracked-apps-toolbar-spacer {
      min-height: 0;
    }
    .button {
      padding: 6px 10px;
      font-weight: 600;
      cursor: pointer;
    }
    .button:hover,
    .switch:hover { background: var(--control-hover); border-color: var(--border-strong); }
    .button--primary { background: var(--accent); border-color: var(--accent); color: #ffffff; }
    .button--primary:hover { background: var(--accent-hover); border-color: var(--accent-hover); }
    .button--warning { background: #5a3d00; border-color: var(--warning); color: var(--strong); }
    .button--danger { background: var(--danger-bg); border-color: var(--danger); }
    .button:disabled {
      cursor: default;
      opacity: 0.55;
      background: var(--control);
      border-color: var(--border);
      color: var(--muted);
    }
    .button--monitoring {
      background: var(--accent);
      border-color: var(--accent);
      color: #ffffff;
    }
    .button--monitoring:hover,
    .button--monitoring:focus { background: var(--accent-hover); border-color: var(--accent-hover); }
    .button--monitoring-active,
    .button--monitoring-active:hover,
    .button--monitoring-active:focus {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
    }
    .button--cloud-active,
    .button--cloud-active:hover,
    .button--cloud-active:focus {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
    }
    .button--monitoring-active {
      animation: monitoring-button-pulse 1.6s ease-in-out infinite;
    }
    .button--monitoring-active .button__icon-shell--monitoring {
      transform-origin: center center;
      animation: monitoring-icon-squash-cycle 1.8s infinite;
    }
    .button--monitoring-active .button__icon-scale--monitoring {
      transform-origin: center center;
      animation: monitoring-icon-scale-cycle 1.8s infinite;
    }
    .button--monitoring-active .button__icon-rotor--monitoring {
      transform-origin: center center;
      animation: monitoring-icon-rotate-cycle 1.8s infinite;
    }
    .button--icon {
      width: var(--size-compact-control);
      height: var(--size-compact-control);
      min-width: var(--size-compact-control);
      min-height: var(--size-compact-control);
      padding: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--control);
      border-color: var(--border);
      border-radius: var(--radius);
      font-size: 20px;
      line-height: 0;
    }
    .button__plus {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 20px;
      height: 20px;
      color: var(--text);
      font-size: 22px;
      font-weight: 400;
      line-height: 1;
      pointer-events: none;
      transform: translateY(-2px);
    }
    .button--mini { min-height: 20px; padding: 2px 7px; font-size: 12px; line-height: 14px; }
    .button--square {
      width: var(--size-close-button);
      height: var(--size-close-button);
      min-width: var(--size-close-button);
      min-height: var(--size-close-button);
      max-width: var(--size-close-button);
      max-height: var(--size-close-button);
      flex: 0 0 var(--size-close-button);
      padding: 0;
      display: grid;
      place-items: center;
      border-radius: var(--radius);
      line-height: 1;
    }
    .button--close {
      background: var(--control);
      border-color: var(--border);
    }
    .button__icon {
      display: block;
      width: 10px;
      height: 10px;
      filter: brightness(0) invert(1);
      pointer-events: none;
    }
    .button__icon-shell,
    .button__icon-scale,
    .button__icon-rotor {
      display: grid;
      place-items: center;
      width: 28px;
      height: 28px;
      pointer-events: none;
    }
    .button__icon-shell--monitoring,
    .button__icon-scale--monitoring,
    .button__icon-rotor--monitoring,
    .button__icon--monitoring {
      width: 24px;
      height: 24px;
    }
    .button__icon--confirm-filtered,
    .button__icon--unconfirm-filtered,
    .button__icon--trash,
    .button__icon--cloud-import,
    .button__icon--cloud-sync,
    .button__icon--import-csv,
    .button__icon--export-csv {
      width: 28px;
      height: 28px;
    }
    .button__icon-fallback {
      display: grid;
      place-items: center;
      width: 28px;
      height: 28px;
      color: var(--text-strong);
      font-size: 18px;
      font-weight: 800;
      line-height: 1;
      pointer-events: none;
    }
    .button__icon--module-stop,
    .button__icon--module-close,
    .button__icon--module-action {
      width: 28px;
      height: 28px;
    }
    .button__icon--my-publications {
      width: 30px;
      height: 30px;
    }
    .button--icon.button--monitoring {
      background: var(--accent);
      border-color: var(--accent);
      color: #ffffff;
    }
    .button--icon.button--monitoring:hover,
    .button--icon.button--monitoring:focus {
      background: var(--accent-hover);
      border-color: var(--accent-hover);
    }
    .button--icon.button--monitoring.button--monitoring-active,
    .button--icon.button--monitoring.button--monitoring-active:hover,
    .button--icon.button--monitoring.button--monitoring-active:focus {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
    }
    .button--icon.button--cloud-active,
    .button--icon.button--cloud-active:hover,
    .button--icon.button--cloud-active:focus {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
    }
    .switch {
      position: relative;
      width: var(--size-switch-width);
      height: var(--size-switch-height);
      min-height: var(--size-switch-height);
      padding: 0;
      cursor: pointer;
    }
    .switch:hover { background: var(--control-hover); border-color: var(--border-strong); }
    .switch__knob {
      position: absolute;
      top: 2px;
      left: 2px;
      width: 16px;
      height: 16px;
      border-radius: var(--radius);
      background: var(--muted);
      transition: transform 140ms ease, background 140ms ease;
    }
    .switch--on { background: var(--accent); }
    .switch--on .switch__knob { transform: translateX(24px); background: #eeeeee; }
    .row,
    .list {
      display: grid;
      gap: 5px;
      padding: 5px;
      border-top: 1px solid var(--border);
      border-bottom: 1px solid var(--border);
      background: var(--list);
      overflow: auto;
    }
    .integration-status {
      display: grid;
      gap: 5px;
      min-width: 0;
      padding: 5px;
    }
    .integration-status--compact {
      padding: 0;
    }
    .modules-card {
      flex: 1 1 var(--modules-panel-height);
      height: auto;
      min-height: var(--modules-panel-height);
      max-height: none;
      overflow: visible;
    }
    .modules-card__header-actions {
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 6px;
      min-width: 0;
    }
    .modules-card__order-label {
      color: var(--muted);
      font-size: 11px;
      font-weight: 600;
      line-height: 16px;
      white-space: nowrap;
    }
    .integration-module-grid {
      display: flex;
      flex-wrap: nowrap;
      gap: 6px;
      align-items: center;
      min-width: 0;
      min-height: calc(var(--module-button-slot) + var(--scrollbar-size));
      overflow-x: auto;
      overflow-y: visible;
      scrollbar-gutter: stable;
      padding: var(--module-button-pulse-gutter) 6px calc(var(--module-button-pulse-gutter) + var(--scrollbar-size)) 0;
    }
    .integration-module-button-shell {
      position: relative;
      display: flex;
      flex: 0 0 auto;
      align-items: center;
      gap: 2px;
      padding: var(--module-button-pulse-gutter);
    }
    .integration-module-button {
      width: var(--module-button-size);
      height: var(--module-button-size);
      min-width: var(--module-button-size);
      min-height: var(--module-button-size);
      padding: 0;
      overflow: hidden;
    }
    .integration-module-button--background-active {
      animation: integration-module-background-pulse 1.6s ease-in-out infinite;
    }
    .integration-module-button-skeleton {
      margin: var(--module-button-pulse-gutter);
    }
    .module-action-button--pulse {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
      animation: module-action-button-pulse 1.6s ease-in-out infinite;
    }
    .module-action-button--pulse:hover,
    .module-action-button--pulse:focus {
      background: #c7771a;
      border-color: #dd8a26;
      color: #ffffff;
    }
    @keyframes integration-module-background-pulse {
      0%, 100% {
        box-shadow: 0 0 0 0 rgba(255, 255, 255, 0);
        filter: brightness(1);
      }
      50% {
        box-shadow: 0 0 0 3px var(--module-activity-pulse);
        filter: brightness(1.12);
      }
    }
    @keyframes module-action-button-pulse {
      0%,
      100% {
        box-shadow: 0 0 0 0 rgba(255, 255, 255, 0);
        opacity: 1;
        transform: scale(1);
      }
      50% {
        box-shadow: 0 0 0 3px var(--module-activity-pulse);
        opacity: 0.96;
        transform: scale(1.02);
      }
    }
    .module-host-dialog {
      width: min(520px, calc(100vw - 32px));
    }
    .module-host-dialog__body {
      display: grid;
      grid-template-columns: 42px minmax(0, 1fr);
      align-items: start;
      padding: 12px 10px;
    }
    .module-host-dialog__icon {
      width: 34px;
      height: 34px;
      display: grid;
      place-items: center;
      border: 1px solid var(--color-control-border);
      background: var(--color-control-bg);
    }
    .module-host-dialog__icon-image {
      width: 22px;
      height: 22px;
      object-fit: contain;
    }
    .module-host-dialog__icon-fallback {
      font-weight: 800;
      color: var(--color-text);
    }
    .module-host-dialog__message {
      min-width: 0;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      color: var(--color-text);
    }
    .integration-module-button__icon {
      display: grid;
      place-items: center;
      width: 100%;
      height: 100%;
      font-size: 13px;
      font-weight: 700;
      line-height: 1;
    }
    .integration-module-button__image {
      display: block;
      width: 80%;
      height: 80%;
      object-fit: contain;
      border-radius: inherit;
      pointer-events: none;
    }
    .integration-module-reorder-controls {
      display: flex;
      gap: 2px;
      align-items: center;
    }
    .integration-module-reorder-button {
      width: 18px;
      min-width: 18px;
      max-width: 18px;
      height: var(--size-compact-control);
      min-height: var(--size-compact-control);
      max-height: var(--size-compact-control);
      padding: 0;
      line-height: 1;
      background: var(--control);
      border-color: var(--border);
    }
    .integration-module-reorder-button--left {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
    .integration-module-reorder-button--right {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
    .integration-module-reorder-button__icon {
      width: 12px;
      height: 12px;
    }
    .integration-module-reorder-button__icon--right {
      transform: rotate(180deg);
    }
    .integration-module-dialog {
      width: fit-content;
      min-width: min(620px, calc(100vw - 48px));
      max-width: calc(100vw - 48px);
      max-height: calc(100dvh - 82px);
      overflow: hidden;
    }
    .integration-module-dialog--layout .integration-module-dialog__body {
      width: 100%;
    }
    .integration-module-dialog--size-fullscreen {
      width: calc(100vw - 48px);
      height: calc(100dvh - var(--size-header-height) - 82px);
      max-height: calc(100dvh - var(--size-header-height) - 82px);
    }
    .integration-module-dialog--align-left {
      justify-self: start;
    }
    .integration-module-dialog--align-center {
      justify-self: center;
    }
    .integration-module-dialog--align-right {
      justify-self: end;
    }
    .integration-module-dialog .modal__body {
      flex: 0 1 auto;
      align-content: start;
      min-height: 0;
      max-height: calc(100dvh - 158px);
      overflow-x: hidden;
      overflow-y: auto;
    }
    .integration-module-dialog--size-fullscreen .integration-module-dialog__body {
      flex: 1 1 auto;
      max-height: none;
    }
    .integration-status-layout--module-menu {
      align-self: start;
      align-items: start;
      width: min(760px, 100%);
      max-width: 100%;
      gap: 5px;
    }
    .integration-status-layout--module-menu .integration-status__fields {
      gap: 2px;
      align-content: start;
    }
    .integration-status-layout--module-menu .integration-status__action {
      justify-content: flex-end;
      gap: 3px;
    }
    .integration-status-layout--module-menu .integration-status__row {
      min-height: 18px;
    }
    .integration-status-layout--module-menu .integration-status__path,
    .integration-status-layout--module-menu .integration-status__provider,
    .integration-status-layout--module-menu .integration-status__value {
      height: 18px;
      line-height: 16px;
    }
    .integration-status-layout--module-menu .integration-status-text {
      line-height: 16px;
    }
    .module-ui-schema {
      display: grid;
      gap: 6px;
      width: min(760px, 100%);
      max-width: 100%;
      min-width: 0;
      min-height: 0;
      align-self: start;
      align-content: start;
      box-sizing: border-box;
    }
    .module-ui-schema--dialog-layout {
      width: 100%;
      align-self: stretch;
      justify-self: stretch;
    }
    .module-ui-schema[hidden] {
      display: none;
    }
    .module-ui-schema__panel,
    .module-ui-schema__nested-subpanel,
    .module-ui-schema__row,
    .module-ui-schema__actions,
    .module-ui-schema__progress {
      display: grid;
      gap: 4px;
      min-width: 0;
      min-height: 0;
      align-content: start;
      box-sizing: border-box;
    }
    .module-ui-schema__panel {
      padding: 8px;
      border: 1px solid var(--border);
      background: var(--panel);
    }
    .module-ui-schema__nested-subpanel {
      gap: 5px;
      padding: 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--list);
    }
    .module-ui-schema__nested-subpanel-inner {
      display: grid;
      gap: 4px;
      min-width: 0;
      min-height: 0;
      padding: 8px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
      box-sizing: border-box;
    }
    .module-ui-schema__grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 8px;
      width: 100%;
      min-width: 0;
      min-height: 0;
      align-items: start;
    }
    .module-ui-schema__grid > .module-ui-schema__title,
    .module-ui-schema__grid > .module-ui-schema__value,
    .module-ui-schema__grid > .module-ui-schema__actions,
    .module-ui-schema__grid > .module-ui-schema__button-row {
      grid-column: 1 / -1;
    }
    .module-ui-schema--size-auto {
      width: fit-content;
      max-width: 100%;
    }
    .module-ui-schema--size-stretch {
      width: 100%;
      align-self: stretch;
    }
    .module-ui-schema--size-fullscreen {
      width: calc(100vw - 72px);
      height: calc(100vh - var(--size-footer-height) - 148px);
      max-width: 100%;
      max-height: calc(100vh - var(--size-footer-height) - 148px);
    }
    .module-ui-schema--align-left {
      justify-self: start;
      text-align: left;
    }
    .module-ui-schema--align-center {
      justify-self: center;
      text-align: center;
    }
    .module-ui-schema--align-right {
      justify-self: end;
      text-align: right;
    }
    .module-ui-schema__actions.module-ui-schema--align-center {
      justify-content: center;
    }
    .module-ui-schema__actions.module-ui-schema--align-right {
      justify-content: flex-end;
    }
    .module-ui-schema--scroll-x {
      overflow-x: auto;
      overflow-y: hidden;
    }
    .module-ui-schema--scroll-y {
      overflow-x: hidden;
      overflow-y: auto;
      max-height: min(360px, 70vh);
    }
    .module-ui-schema--scroll-both {
      overflow: auto;
      max-height: min(360px, 70vh);
    }
    .module-ui-schema__table {
      display: grid;
      width: 100%;
      min-width: 0;
      min-height: 0;
      box-sizing: border-box;
      grid-template-rows: auto minmax(0, 1fr) auto;
      gap: 0;
      align-content: stretch;
      align-self: stretch;
      overflow: hidden;
    }
    .module-ui-schema__table-frame,
    .module-ui-schema__table-frame.table-wrap {
      display: flex;
      flex-direction: column;
      grid-row: 2;
      width: 100%;
      height: 100%;
      max-width: 100%;
      min-width: 0;
      min-height: 0;
      box-sizing: border-box;
      overflow: hidden;
      overscroll-behavior: contain;
    }
    .module-ui-schema__table-body {
      flex: 1 1 auto;
      min-height: 0;
    }
    .module-ui-schema__table-body.module-ui-schema--scroll-x {
      overflow-x: auto;
      overflow-y: hidden;
    }
    .module-ui-schema__table-body.module-ui-schema--scroll-y {
      overflow-x: hidden;
      overflow-y: auto;
    }
    .module-ui-schema__table-body.module-ui-schema--scroll-both {
      overflow: auto;
    }
    .module-ui-schema__table.module-ui-schema--size-fullscreen .module-ui-schema__table-frame {
      height: 100%;
      max-height: none;
    }
    .module-ui-schema__table .module-ui-schema__title {
      padding: 6px 8px 0;
    }
    .module-ui-schema__progress--compact-labeled {
      grid-template-columns: max-content minmax(120px, 1fr);
      align-items: center;
      column-gap: 6px;
      width: 100%;
    }
    .module-ui-schema__progress--compact-labeled > .module-ui-schema__title {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .module-ui-schema__progress--compact-labeled > .progress-bar--compact {
      min-width: 120px;
    }
    .module-ui-schema__footer {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
      width: 100%;
      min-width: 0;
      padding: 0;
      border-top: 0;
    }
    .module-ui-schema__footer-host {
      display: flex;
      align-items: center;
      gap: 8px;
      flex: 1 1 auto;
      min-width: 0;
      overflow: hidden;
    }
    .module-ui-schema__footer-value {
      min-width: 0;
      color: var(--text-muted);
      font-size: 12px;
    }
    .module-ui-schema__footer > .module-ui-schema__value {
      flex: 1 1 auto;
      min-width: 0;
    }
    .module-ui-schema__footer > .module-ui-schema__actions {
      flex: 0 0 auto;
      width: auto;
    }
    .module-ui-schema__row {
      display: grid;
      width: 100%;
      min-width: 0;
      grid-template-columns: minmax(120px, auto) minmax(0, 1fr) auto;
      align-items: center;
      min-height: var(--size-compact-control);
      column-gap: 6px;
      box-sizing: border-box;
    }
    .module-ui-schema__row--no-title {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .module-ui-schema__row--textarea {
      align-items: start;
      min-height: 0;
    }
    .module-ui-schema__title h3 {
      margin: 0;
      font-size: 12px;
      line-height: 16px;
    }
    .module-ui-schema__value,
    .module-ui-schema__status {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .module-ui-schema__path {
      width: 100%;
      height: 22px;
    }
    .module-ui-schema__input-shell {
      position: relative;
      display: grid;
      grid-template-columns: minmax(0, 1fr) 20px;
      align-items: center;
      width: 100%;
      min-width: 0;
      align-self: stretch;
      overflow: visible;
    }
    .module-ui-schema__input,
    .module-ui-schema__select {
      width: 100%;
      height: var(--size-compact-control);
      min-height: var(--size-compact-control);
      min-width: 0;
      padding: 3px 5px;
      font-size: 12px;
      line-height: 16px;
    }
    .module-ui-schema__input-shell > .input {
      grid-column: 1 / -1;
      grid-row: 1;
    }
    .module-ui-schema__input {
      padding-right: 30px;
    }
    .module-ui-schema__input-shell--textarea {
      align-items: start;
      grid-auto-rows: auto;
      height: auto;
      min-height: 0;
    }
    .module-ui-schema__textarea {
      display: block;
      width: 100%;
      max-width: 100%;
      box-sizing: border-box;
      min-height: 74px;
      overflow: auto;
      resize: vertical;
      line-height: 18px;
      padding: 4px 30px 4px 5px;
    }
    .module-ui-schema__clear {
      position: absolute;
      top: 50%;
      right: 5px;
      transform: translateY(-50%);
      display: grid;
      place-items: center;
      width: 20px;
      height: 20px;
      min-width: 20px;
      min-height: 20px;
      padding: 0;
    }
    .module-ui-schema__clear--textarea {
      top: 6px;
      transform: none;
    }
    .module-ui-schema__actions {
      display: flex;
      flex-wrap: wrap;
      gap: 4px;
      width: 100%;
      min-width: 0;
    }
    .module-ui-schema__action-slot {
      display: flex;
      min-width: 0;
      min-height: var(--size-compact-control);
      align-items: center;
    }
    .module-ui-schema__action {
      white-space: nowrap;
    }
    .module-ui-schema__button-row {
      display: grid;
      width: 100%;
      min-width: 0;
      min-height: var(--size-compact-control);
      margin: 8px 0 0;
      padding: 0;
      align-items: center;
      align-self: stretch;
      align-content: center;
      box-sizing: border-box;
      clear: both;
      overflow: visible;
    }
    .module-ui-schema__button-row-content {
      display: grid;
      width: 100%;
      min-width: 0;
      min-height: var(--size-compact-control);
      align-items: center;
      box-sizing: border-box;
      overflow: visible;
    }
    .module-ui-schema__button-row > .module-ui-schema__actions {
      min-height: var(--size-compact-control);
      align-items: center;
    }
    .module-ui-schema__button-row-content > .module-ui-schema__actions {
      min-height: var(--size-compact-control);
      align-items: center;
    }
    .module-ui-schema__panel > .module-ui-schema__actions,
    .module-ui-schema__tabs-body > .module-ui-schema__actions {
      min-height: var(--size-compact-control);
      margin-bottom: 0;
    }
    .module-ui-schema__panel > .module-ui-schema__button-row,
    .module-ui-schema__tabs-body > .module-ui-schema__button-row {
      min-height: var(--size-compact-control);
      margin-bottom: 0;
      padding-top: 0;
      padding-bottom: 0;
      overflow: visible;
    }
    .module-ui-schema__actions--column {
      flex-direction: column;
      align-items: flex-start;
      justify-content: flex-start;
      width: auto;
    }
    .module-ui-schema__actions--column.module-ui-schema--align-center {
      align-items: center;
      justify-content: flex-start;
      justify-self: center;
    }
    .module-ui-schema__actions--column.module-ui-schema--align-right {
      align-items: flex-end;
      justify-content: flex-start;
      justify-self: end;
    }
    .module-ui-schema__action--align-left {
      margin-inline-start: 0;
      margin-inline-end: 0;
    }
    .module-ui-schema__action-slot.module-ui-schema__action--align-left {
      justify-content: flex-start;
    }
    .module-ui-schema__action--align-center {
      margin-inline-start: 0;
      margin-inline-end: 0;
    }
    .module-ui-schema__action-slot.module-ui-schema__action--align-center {
      justify-content: center;
    }
    .module-ui-schema__action--align-right {
      margin-inline-start: 0;
      margin-inline-end: 0;
    }
    .module-ui-schema__action-slot.module-ui-schema__action--align-right {
      justify-content: flex-end;
    }
    .module-ui-schema__separator {
      height: 1px;
      background: var(--border);
      margin: 2px 0;
    }
    .module-ui-schema__help {
      margin: 0;
      color: var(--text-muted);
      font-size: 12px;
    }
    .ui-entity-table {
      width: 100%;
      min-width: 100%;
      max-width: none;
      border-collapse: collapse;
      background: var(--list);
      color: var(--text);
      font-size: 12px;
      table-layout: fixed;
    }
    .ui-entity-table th,
    .ui-entity-table td {
      border-bottom: 1px solid var(--border);
      text-align: left;
      vertical-align: middle;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .ui-entity-table th {
      padding: 1px 10px;
      background: var(--chrome);
      color: var(--strong);
      font-weight: 700;
      line-height: 14px;
      height: 18px;
      border-bottom: 1px solid var(--border);
      white-space: nowrap;
    }
    .ui-entity-table td {
      padding: 2px 10px;
      line-height: 16px;
    }
    .ui-entity-table td.module-ui-schema--align-center {
      text-align: center;
    }
    .ui-entity-table td.module-ui-schema--align-right {
      text-align: right;
    }
    .module-ui-schema__table-cell-field.path-field {
      display: block;
      width: 100%;
      max-width: 100%;
      min-width: 0;
      height: 20px;
      line-height: 16px;
      padding: 1px 5px;
      box-sizing: border-box;
      overflow-x: hidden;
      overflow-y: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .module-ui-schema__table-cell-field.path-field:focus {
      overflow-x: auto;
      text-overflow: clip;
    }
    .ui-entity-table tr:last-child td {
      border-bottom: 0;
    }
    .integration-status-layout {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 8px;
      align-items: stretch;
      min-width: 0;
    }
    .integration-status__fields {
      display: grid;
      width: 100%;
      max-width: min(100%, var(--size-integration-status-left-max));
      gap: 5px;
      min-width: 0;
    }
    .integration-status__action {
      display: flex;
      flex-direction: column;
      align-items: stretch;
      justify-content: flex-end;
      gap: 5px;
      min-width: 0;
    }
    .integration-status__action > .button {
      width: 100%;
      min-width: 0;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .integration-status__main {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 8px;
      min-width: 0;
    }
    .integration-status__main--with-path {
      display: grid;
      grid-template-columns: auto auto minmax(0, 1fr);
      justify-content: stretch;
    }
    .integration-status__row {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      gap: 8px;
      align-items: center;
      min-height: 22px;
      min-width: 0;
    }
    .integration-status__row--with-action {
      grid-template-columns: auto minmax(0, 1fr) auto;
    }
    .integration-status__label {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .integration-status__path,
    .integration-status__provider,
    .integration-status__value {
      min-width: 0;
      margin: 0;
      overflow: hidden;
      color: var(--text);
      font-size: 12px;
      line-height: 16px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .integration-status-text {
      color: var(--muted);
      font-size: 12px;
      font-weight: 700;
      line-height: 18px;
      white-space: nowrap;
    }
    .integration-status-text--success { color: var(--success); }
    .integration-status-text--danger { color: var(--danger); }
    .path-field {
      display: block;
      width: 100%;
      min-width: 0;
      height: 22px;
      appearance: none;
      -webkit-appearance: none;
      border: 0;
      border-radius: var(--radius);
      background-clip: padding-box;
      background: transparent;
      color: var(--muted);
      font: inherit;
      font-size: 12px;
      line-height: 18px;
      outline: none;
      overflow-x: hidden;
      overflow-y: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      scrollbar-width: none;
    }
    .path-field:focus {
      background: var(--control);
      color: var(--text);
      overflow-x: auto;
      text-overflow: clip;
    }
    .path-field::-webkit-scrollbar { width: 0; height: 0; }
    .path-field--clickable { cursor: pointer; }
    .path-field--content-width {
      width: min(100%, var(--path-field-content-width, 100%));
      max-width: 100%;
      justify-self: start;
      flex: 0 1 auto;
    }
    .path-field--clickable:hover {
      color: var(--text);
      text-decoration: underline;
    }
    .integration-status__path.path-field,
    .integration-dialog-path__input.path-field,
    .ignored-addresses__domain.path-field,
    .cloud-sync-app-field.path-field,
    .domain-field.path-field {
      color: var(--text);
    }
    .integration-dialog-status-list {
      display: grid;
      gap: 5px;
      padding: 5px;
      border-top: 1px solid var(--border);
      border-bottom: 1px solid var(--border);
      background: var(--list);
    }
    .integration-dialog-status-row {
      display: grid;
      grid-template-columns: minmax(110px, auto) minmax(0, 1fr);
      gap: 8px;
      align-items: center;
      min-height: 30px;
      padding: 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
    }
    .integration-dialog-status-row--progress {
      grid-template-columns: minmax(0, 1fr) auto;
      background: var(--row-on);
    }
    .integration-dialog-status-row__label {
      min-width: 0;
    }
    .integration-dialog-status-row__value,
    .integration-dialog-status-text,
    .integration-dialog-path__input {
      min-width: 0;
      overflow: hidden;
      color: var(--text);
      font-size: 12px;
      line-height: 18px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .integration-dialog-status-text--success { color: var(--strong); }
    .integration-dialog-status-text--warning { color: var(--warning); }
    .integration-dialog-status-text--danger { color: var(--danger); }
    .integration-dialog-progress-main { min-width: 0; }
    .integration-dialog-path__input {
      min-width: 0;
      height: 22px;
      border: 0;
      border-radius: var(--radius);
      background: transparent;
      color: var(--text);
      font: inherit;
      font-size: 12px;
      outline: none;
    }
    .integration-dialog-path__input:focus { background: var(--control); }
    .progress-bar { display: grid; gap: 5px; min-width: 0; }
    .progress-bar__header {
      display: flex;
      justify-content: space-between;
      gap: 8px;
      align-items: center;
      min-width: 0;
      color: var(--accent);
      font-size: 12px;
      font-weight: 700;
      line-height: 18px;
    }
    .progress-bar__label,
    .progress-bar__meta {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .progress-bar__track {
      position: relative;
      height: 6px;
      overflow: hidden;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--control);
    }
    .progress-bar__segments {
      position: absolute;
      inset: 0;
      display: flex;
    }
    .progress-bar__segment {
      height: 100%;
    }
    .progress-bar__segment--accent { background: var(--progress-accent); }
    .progress-bar__segment--success { background: var(--progress-success); }
    .progress-bar__segment--warning { background: var(--progress-warning); }
    .progress-bar__segment--danger { background: var(--progress-danger); }
    .progress-bar__segment--rust { background: var(--progress-rust); }
    .progress-bar__segment--muted { background: var(--progress-muted); }
    .progress-bar__remaining {
      position: absolute;
      top: 0;
      right: 0;
      height: 100%;
      background: var(--control);
    }
    .progress-bar__stages {
      display: grid;
      gap: 8px;
      min-width: 0;
      color: var(--muted);
      font-size: 11px;
      line-height: 14px;
    }
    .progress-bar__stages span {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .progress-bar__stages span:last-child {
      text-align: right;
    }
    .tracked-list {
      height: var(--tracked-app-list-height);
      max-height: var(--tracked-app-list-height);
      padding-right: 0;
      scrollbar-gutter: stable;
    }
    .tracked-app {
      display: grid;
      grid-template-columns: var(--size-app-icon-slot) minmax(130px, 1fr) var(--size-switch-width);
      gap: 12px;
      align-items: stretch;
      min-width: 0;
      height: 52px;
      min-height: 52px;
      max-height: 52px;
      padding: 5px 0 5px 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
      cursor: pointer;
      overflow: visible;
    }
    .tracked-app--enabled { background: var(--row-on); }
    .tracked-app-skeleton {
      min-height: 52px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
      animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
    }
    .integration-module-button-skeleton {
      flex: 0 0 auto;
      width: var(--module-button-size);
      height: var(--module-button-size);
      min-width: var(--module-button-size);
      min-height: var(--module-button-size);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
      animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
    }
    @keyframes tracked-app-skeleton-pulse {
      from { background: var(--row); }
      to { background: var(--control-hover); }
    }
    @keyframes monitoring-button-pulse {
      0%,
      100% { transform: scale(1); opacity: 1; }
      50% { transform: scale(1.04); opacity: 0.92; }
    }
    @keyframes monitoring-icon-squash-cycle {
      0% {
        transform: translateY(0) scaleY(1);
        animation-timing-function: ease-in-out;
      }
      11.111% {
        transform: translateY(0) scaleY(0.2);
        animation-timing-function: ease-in-out;
      }
      22.222% {
        transform: translateY(0) scaleY(1);
        animation-timing-function: linear;
      }
      38% {
        transform: translateY(0) scaleY(1);
      }
      100% {
        transform: translateY(0) scaleY(1);
      }
    }
    @keyframes monitoring-icon-rotate-cycle {
      0% {
        transform: rotate(0deg);
      }
      44.444% {
        transform: rotate(0deg);
        animation-timing-function: linear;
      }
      72.222% {
        transform: rotate(360deg);
      }
      100% {
        transform: rotate(360deg);
      }
    }
    @keyframes monitoring-icon-scale-cycle {
      0% {
        transform: scale(1);
      }
      44.444% {
        transform: scale(1);
        animation-timing-function: cubic-bezier(0.2, 0.78, 0.22, 1);
      }
      58.333% {
        transform: scale(1.2);
        animation-timing-function: cubic-bezier(0.45, 0, 0.2, 1);
      }
      72.222% {
        transform: scale(1);
      }
      100% {
        transform: scale(1);
      }
    }
    .app-icon {
      display: grid;
      place-items: center;
      width: 40px;
      height: 40px;
      border: 1px solid var(--border);
      border-radius: 999px;
      background: var(--control);
      overflow: hidden;
      color: var(--strong);
      font-weight: 800;
    }
    .app-icon img { width: 32px; height: 32px; object-fit: contain; }
    .app-main { min-width: 0; display: grid; gap: 2px; }
    .app-title { overflow: hidden; color: var(--strong); font-size: 15px; line-height: 20px; text-overflow: ellipsis; white-space: nowrap; }
    .app-path {
      display: block;
      padding: 0;
      text-align: left;
    }
    .app-actions {
      display: grid;
      grid-template-columns: 1fr;
      grid-template-rows: 20px 18px;
      justify-items: end;
      align-content: start;
      gap: 2px;
      min-width: var(--size-switch-width);
    }
    .app-actions .switch {
      grid-row: 2;
    }
    .title-with-help {
      display: inline-flex;
      align-items: center;
      gap: 5px;
      padding: 2px 4px;
    }
    .help-icon {
      display: inline-grid;
      place-items: center;
      position: relative;
      top: 2px;
      width: 18px;
      height: 18px;
      border-radius: var(--radius-pill);
      border: 1px solid var(--border);
      background: var(--control);
      color: var(--muted);
      font-size: 12px;
      font-weight: 800;
      line-height: 16px;
      cursor: help;
    }
    .help-icon__image {
      display: block;
      width: 18px;
      height: 18px;
      filter: none;
      pointer-events: none;
    }
    .stack { display: flex; flex-direction: column; gap: 12px; }
    .tracked-apps-card .stack {
      display: grid;
      gap: 5px;
    }
    .tracked-apps-card > .stack {
      grid-template-rows: auto minmax(0, 1fr) auto;
      min-height: 0;
    }
    .exe-path-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) var(--size-compact-control);
      gap: 5px;
      align-items: center;
    }
    .tracked-apps-card .exe-path-row {
      padding-bottom: var(--tracked-apps-path-bottom-pad);
    }
    .tracked-apps-bulk-row {
      display: grid;
      grid-template-columns: auto var(--size-switch-width) minmax(0, 1fr);
      align-items: center;
      gap: 5px;
      min-height: var(--size-switch-height);
    }
    .tracked-apps-bulk-label,
    .tracked-apps-count-label {
      font-size: 12px;
      line-height: var(--size-switch-height);
      color: var(--text);
    }
    .tracked-apps-count-label {
      justify-self: end;
      white-space: nowrap;
    }
    .field-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) var(--size-compact-control);
      gap: 5px;
      align-items: center;
    }
    .controls { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px; }
    .button-row { display: flex; flex-wrap: wrap; gap: 5px; }
    .ignored-addresses-card { flex: 0 0 auto; min-height: 0; align-self: stretch; }
    .ignored-addresses-section {
      display: grid;
      gap: 5px;
      min-width: 0;
      min-height: 0;
    }
    .value-label,
    .integration-status__label,
    .integration-dialog-status-row__label {
      color: var(--text);
      font-size: 12px;
      font-weight: 700;
      line-height: 16px;
    }
    .ignored-addresses {
      display: grid;
      grid-auto-rows: var(--ignored-address-row-height);
      align-content: start;
      gap: var(--ignored-address-row-gap);
      height: var(--ignored-addresses-list-height);
      max-height: var(--ignored-addresses-list-height);
      padding: 5px;
      padding-right: 0;
      border-top: 1px solid var(--border);
      border-bottom: 1px solid var(--border);
      background: var(--list);
      overflow-x: hidden;
      overflow-y: auto;
      scrollbar-gutter: stable;
    }
    .ignored-addresses__empty {
      min-height: var(--ignored-address-row-height);
      padding: 5px;
      color: var(--muted);
      font-size: 12px;
      line-height: 18px;
    }
    .ignored-addresses__skeleton {
      height: var(--ignored-address-row-height);
      min-height: var(--ignored-address-row-height);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
      animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
    }
    .observations-card { display: flex; flex-direction: column; min-height: 0; height: 100%; max-height: none; }
    .observations-card .table-wrap { flex: 1 1 auto; min-height: 0; max-height: none; height: 100%; }
    table { width: 100%; min-width: 860px; border-collapse: collapse; table-layout: fixed; }
    th, td { border-bottom: 1px solid var(--border); text-align: left; vertical-align: middle; }
    th { position: sticky; top: 0; padding: 6px 10px; background: var(--chrome); color: var(--strong); font-size: 12px; font-weight: 700; line-height: 16px; z-index: 1; }
    td { padding: 2px 10px; font-size: 12px; line-height: 16px; }
    tbody tr:hover td { background: var(--row-hover); }
    .observation-row--confirmed td { background: var(--row-on); }
    .observation-row--confirmed:hover td { background: var(--row-enabled-hover); }
    .observations-card > .table-wrap > table th:nth-child(2),
    .observations-card > .table-wrap > table td:nth-child(2) { width: 138px; }
    .observations-card > .table-wrap > table th:nth-child(4),
    .observations-card > .table-wrap > table td:nth-child(4) { width: 64px; }
    .observations-card > .table-wrap > table th:nth-child(5),
    .observations-card > .table-wrap > table td:nth-child(5) { width: 66px; }
    .observations-card > .table-wrap > table th:nth-child(6),
    .observations-card > .table-wrap > table td:nth-child(6) { width: 128px; }
    .observations-card > .table-wrap > table th:nth-child(7),
    .observations-card > .table-wrap > table td:nth-child(7) { width: 52px; }
    .observations-card > .table-wrap > table th:nth-child(8),
    .observations-card > .table-wrap > table td:nth-child(8) { width: 142px; }
    .observations-card > .table-wrap > table th:nth-child(9),
    .observations-card > .table-wrap > table td:nth-child(9) { width: 142px; }
    .observations-card > .table-wrap > table th:nth-child(10),
    .observations-card > .table-wrap > table td:nth-child(10) { width: 96px; }
    .observation-connection {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      min-width: 0;
    }
    .connection-metrics {
      display: inline-flex;
      align-items: center;
      gap: 0;
      color: var(--text);
      white-space: nowrap;
    }
    .connection-metrics__success { color: var(--success); }
    .connection-metrics__failure { color: var(--danger); }
    .observation-app-field.path-field,
    .observation-ip-field.path-field {
      width: 100%;
      min-width: 0;
      color: var(--text);
    }
    .table-wrap { height: 100%; max-height: none; overflow: auto; border: 1px solid var(--border); border-radius: var(--radius); background: var(--list); }
    .table-wrap::-webkit-scrollbar-track { background: var(--scrollbar-track); }
    .table-sortable { cursor: pointer; user-select: none; }
    .table-sortable__content { display: inline-flex; align-items: center; gap: 3px; min-width: 0; transform: translateX(-8px); }
    .table-sortable__label { min-width: 0; }
    .table-sortable__icon {
      position: relative;
      top: 0;
      width: 18px;
      height: 36px;
      flex: 0 0 18px;
      filter: brightness(0) invert(1);
      opacity: 0.28;
      transform: translateZ(0);
      transition: opacity 160ms ease;
    }
    .table-sortable__icon--asc { opacity: 0.85; }
    .table-sortable__icon--desc { opacity: 0.85; }
    .table-sortable__icon--idle { opacity: 0.28; }
    thead th { padding: 1px 10px; line-height: 14px; height: 18px; }
    .table-actions { display: flex; flex-wrap: nowrap; gap: 5px; }
    .table-action-button {
      flex: 0 0 20px;
      width: 20px;
      height: 20px;
      min-width: 20px;
      min-height: 20px;
      padding: 0;
    }
    .table-action-button__icon {
      width: 14px;
      height: 14px;
    }
    .cloud-web-subpanel table {
      width: 100%;
      table-layout: fixed;
      border-collapse: collapse;
    }
    .cloud-web-subpanel col.cloud-web-action-column {
      width: 32px;
    }
    .cloud-web-subpanel col.browser-table-col-ip {
      width: 138px;
    }
    .cloud-web-subpanel col.browser-table-col-port {
      width: 64px;
    }
    .cloud-web-subpanel col.browser-table-col-protocol {
      width: 66px;
    }
    .cloud-web-subpanel col.browser-table-col-connection {
      width: 128px;
    }
    .cloud-web-subpanel col.browser-table-col-count {
      width: 52px;
    }
    .cloud-web-subpanel col.browser-table-col-author {
      width: 142px;
    }
    .cloud-web-subpanel col.browser-table-col-privacy {
      width: 64px;
    }
    .cloud-web-subpanel thead th {
      height: 18px;
      padding: 1px 10px;
      line-height: 14px;
      background: var(--chrome);
      color: var(--strong);
      font-weight: 700;
    }
    .cloud-web-subpanel tbody td {
      padding: 2px 10px;
      line-height: 16px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      box-sizing: border-box;
      background: transparent;
      border-bottom: 1px solid var(--border);
      color: var(--text);
    }
    .cloud-web-subpanel tbody tr:hover td {
      background: var(--row-hover);
    }
    .cloud-web-subpanel tbody tr.observation-row--confirmed td {
      background: var(--row-on);
    }
    .cloud-web-subpanel tbody tr.observation-row--confirmed:hover td {
      background: var(--row-enabled-hover);
    }
    .cloud-web-subpanel th:nth-child(n),
    .cloud-web-subpanel td:nth-child(n) {
      width: auto;
    }
    .cloud-web-subpanel th.cloud-web-action-header,
    .cloud-web-subpanel td.cloud-web-action-cell {
      width: 32px;
      min-width: 32px;
      max-width: 32px;
      padding-left: 6px;
      padding-right: 6px;
      text-align: center;
      box-sizing: border-box;
    }
    .cloud-web-subpanel tbody tr.observation-row {
      cursor: default;
    }
    .cloud-web-subpanel tbody tr.observation-row .table-action-button {
      cursor: pointer;
    }
    .observations-card .table-wrap .path-field,
    .cloud-web-panel .table-wrap .path-field {
      background: transparent;
      border-color: transparent;
      color: var(--text);
      box-shadow: none;
    }
    .observations-card .table-wrap .path-field:focus,
    .observations-card .table-wrap .observation-row:hover .path-field,
    .observations-card .table-wrap .observation-row--confirmed .path-field,
    .observations-card .table-wrap .observation-row--confirmed:hover .path-field,
    .cloud-web-panel .table-wrap .path-field:focus,
    .cloud-web-panel .table-wrap .observation-row:hover .path-field,
    .cloud-web-panel .table-wrap .observation-row--confirmed .path-field,
    .cloud-web-panel .table-wrap .observation-row--confirmed:hover .path-field {
      background: transparent;
      border-color: transparent;
      color: var(--text);
      box-shadow: none;
    }
    .ignored-row {
      display: grid;
      grid-template-columns: var(--ignored-address-ip-column) 18px minmax(80px, 1fr) var(--size-close-button);
      gap: 6px;
      align-items: center;
      min-width: 0;
      height: var(--ignored-address-row-height);
      min-height: var(--ignored-address-row-height);
      padding: 3px 0 3px 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--row);
    }
    .ignored-row code {
      display: block;
      width: 100%;
      min-width: var(--ignored-address-ip-column);
      max-width: var(--ignored-address-ip-column);
      justify-self: stretch;
      text-align: left;
    }
    .ignored-row .ignored-addresses__address {
      width: 100%;
      min-width: var(--ignored-address-ip-column);
      max-width: var(--ignored-address-ip-column);
      grid-column: 1;
      grid-row: 1;
      justify-self: stretch;
      color: var(--text);
      direction: ltr;
      text-align: left;
      unicode-bidi: plaintext;
    }
    .ignored-row .help-icon {
      grid-column: 2;
      grid-row: 1;
      justify-self: center;
    }
    #ignored-addresses-panel-help {
      position: relative;
      top: -2px;
    }
    .ignored-row .domain-field {
      grid-column: 3;
      grid-row: 1;
    }
    .ignored-row .button {
      grid-column: 4;
      grid-row: 1;
      justify-self: end;
    }
    .ip-cell {
      display: flex;
      align-items: center;
      gap: 6px;
      min-width: 0;
    }
    .ip-cell .help-icon { top: 0; }
    .domain-field {
      width: min(260px, 100%);
      min-width: 120px;
      min-height: 24px;
      padding: 2px 6px;
      color: var(--text);
    }
    code { overflow: hidden; color: var(--muted); text-overflow: ellipsis; white-space: nowrap; }
    details pre {
      max-height: 360px;
      overflow: auto;
      margin: 0;
      padding: 10px;
      border: 1px solid var(--border);
      background: var(--list);
      white-space: pre-wrap;
    }
    .modal-backdrop {
      position: fixed;
      inset: 0 0 34px 0;
      z-index: 30;
      display: grid;
      place-items: center;
      padding: 24px;
      background: var(--bg);
    }
    .modal-backdrop.module-overlay-backdrop {
      inset: var(--size-header-height) 0 34px 0;
    }
    .modal-backdrop[hidden] { display: none; }
    .disabled-browser-overlay {
      position: fixed;
      inset: 0 0 34px 0;
      z-index: 40;
      display: grid;
      place-items: center;
      padding: 24px;
      background: rgba(24, 24, 24, 0.96);
    }
    .disabled-browser-overlay[hidden] { display: none; }
    .disabled-browser-overlay__panel {
      display: grid;
      gap: 8px;
      width: min(520px, 100%);
      padding: 22px 24px;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--panel);
    }
    .host-unavailable-overlay {
      z-index: 42;
    }
    .host-unavailable-overlay .disabled-browser-overlay__panel {
      border-color: var(--warning);
    }
    .disabled-browser-overlay__title {
      margin: 0;
      color: var(--strong);
      font-size: 20px;
      line-height: 1.2;
    }
    .disabled-browser-overlay__text {
      margin: 0;
      color: var(--text);
      line-height: 1.5;
    }
    .disabled-browser-overlay__note { color: var(--muted); }
    .web-auth-overlay__form {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 8px;
      align-items: center;
      margin-top: 4px;
    }
    .web-auth-overlay__status {
      min-height: 16px;
      color: var(--warning);
      font-size: 12px;
      line-height: 16px;
    }
    .modal {
      display: grid;
      gap: 12px;
      width: min(560px, 100%);
      padding: 14px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--panel);
    }
    .modal--panel,
    .modal--compact {
      display: flex;
      flex-direction: column;
      gap: 5px;
      padding: 5px;
    }
    .modal--panel .modal__header,
    .modal--compact .modal__header {
      min-height: 18px;
      margin: -5px -5px 0;
      padding: 6px 8px;
      border-bottom: 1px solid var(--border);
      background: var(--chrome);
    }
    .modal--panel .modal__body,
    .modal--compact .modal__body { gap: 5px; }
    .modal--panel .modal__footer,
    .modal--compact .modal__footer {
      flex: 0 0 var(--size-panel-footer-height);
      height: var(--size-panel-footer-height);
      min-height: var(--size-panel-footer-height);
      max-height: var(--size-panel-footer-height);
      align-items: center;
      gap: 5px;
      margin: 0 -5px -5px;
      padding: 6px 8px;
      overflow: hidden;
      border-top: 1px solid var(--border);
      background: var(--chrome);
    }
    .modal--panel .modal__header h2,
    .modal--compact .modal__header h2 {
      margin: 0;
      font-size: 18px;
      line-height: 22px;
    }
    .modal__header,
    .modal__actions,
    .modal__footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 12px;
    }
    .modal__body { display: grid; gap: 8px; }
    .modal__actions,
    .modal__footer { justify-content: flex-end; }
    .modal__footer { margin-top: 16px; }
    .modal--panel .modal__footer,
    .modal--compact .modal__footer { margin-top: 0; }
    .integration-module-dialog > .modal__footer[data-ui-entity="panel-footer"] {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: center;
    }
    .integration-module-dialog
      > .modal__footer[data-ui-entity="panel-footer"]
      > .module-ui-schema__footer-host {
      grid-column: 1;
    }
    .integration-module-dialog
      > .modal__footer[data-ui-entity="panel-footer"]
      > .module-ui-schema__footer-nav {
      grid-column: 2;
      justify-self: end;
    }
    [data-ui-entity="panel-footer"] {
      display: flex;
      align-items: center;
      gap: 6px;
      flex: 0 0 var(--size-panel-footer-height);
      height: var(--size-panel-footer-height);
      min-height: var(--size-panel-footer-height);
      max-height: var(--size-panel-footer-height);
      overflow: hidden;
    }
    .panel-footer {
      margin: 0 -5px -5px;
      padding: 6px 8px;
      border-top: 1px solid var(--border);
      border-radius: 0 0 var(--radius-control) var(--radius-control);
      background: var(--chrome);
      color: var(--muted);
    }
    .panel-footer-meta {
      display: inline-flex;
      align-items: center;
      gap: 0;
      min-width: 0;
      overflow: hidden;
      color: var(--muted);
      font-size: 12px;
      line-height: 16px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .panel-footer-meta__item {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .panel-footer-meta__item + .panel-footer-meta__item {
      margin-left: 6px;
      padding-left: 12px;
      border-left: 1px solid var(--border);
    }
    .monitoring-panel-footer {
      justify-content: flex-start;
    }
    .modal-footer-separator {
      width: 1px;
      align-self: stretch;
      min-height: 30px;
      margin: 0 5px;
      background: var(--border);
    }
    .integration-folder-path-help {
      margin: 0;
      padding-left: var(--size-control-label-offset);
      color: var(--muted);
      font-size: 12px;
      line-height: 16px;
    }
    .help-text {
      color: var(--muted);
      font-size: 13px;
      line-height: 1.5;
    }
    .note-box {
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--list);
      padding: 12px 14px;
    }
    .integration-download-actions { justify-content: center; }
    .integration-folder-field { display: grid; gap: 5px; margin-top: 20px; }
    .integration-folder-field-row {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 5px;
      margin-top: 0;
    }
    .integration-folder-field-row > .button {
      height: 30px;
      min-height: 30px;
    }
    .server-file-picker-dialog {
      display: grid;
      width: min(900px, calc(100vw - 64px));
      height: min(640px, calc(100vh - 84px));
      grid-template-rows: auto minmax(0, 1fr) auto;
    }
    .server-file-picker-dialog .modal__body {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr) auto;
      gap: 8px;
      min-height: 0;
    }
    .server-file-picker-path-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 6px;
      align-items: end;
    }
    .server-file-picker-roots {
      display: flex;
      flex-wrap: wrap;
      gap: 5px;
      min-width: 0;
    }
    .server-file-picker-list {
      overflow: auto;
      display: grid;
      align-content: start;
      gap: 5px;
      min-height: 0;
      padding: 5px;
      border: 1px solid var(--border);
      border-radius: 3px;
      background: var(--panel);
      scrollbar-gutter: stable;
    }
    .server-file-picker-row {
      display: grid;
      grid-template-columns: 28px minmax(0, 1fr) auto;
      gap: 8px;
      align-items: center;
      min-height: 32px;
      padding: 5px 7px;
      border: 1px solid var(--border);
      border-radius: 3px;
      background: var(--row);
      color: var(--strong);
      cursor: pointer;
      text-align: left;
    }
    .server-file-picker-row:hover,
    .server-file-picker-row--selected {
      border-color: var(--accent);
      background: var(--row-on);
    }
    .server-file-picker-row[disabled] {
      cursor: not-allowed;
      opacity: 0.55;
    }
    .server-file-picker-row__name,
    .server-file-picker-row__path {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      min-width: 0;
    }
    .server-file-picker-row__path {
      color: var(--muted);
      font-size: 11px;
    }
    .server-file-picker-save-row {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      gap: 8px;
      align-items: center;
    }
    .server-file-picker-feedback {
      min-height: 18px;
      color: var(--danger);
      font-size: 12px;
    }
    .profile-export-dialog {
      width: min(1080px, calc(100vw - 32px));
      height: min(720px, calc(100vh - 32px));
      grid-template-rows: auto minmax(0, 1fr) auto;
    }
    .profile-export-advanced-dialog {
      display: grid;
      width: min(940px, calc(100vw - 48px));
      height: min(650px, calc(100vh - 48px));
      grid-template-rows: auto minmax(0, 1fr) auto;
    }
    .profile-export-dialog .modal__body {
      flex: 1 1 auto;
      min-height: 0;
      overflow: hidden;
    }
    .profile-export-advanced-dialog .modal__body {
      height: 100%;
      min-height: 0;
      overflow: hidden;
    }
    .profile-export-advanced-body {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto auto minmax(0, 1fr);
      gap: 10px;
      height: 100%;
      min-width: 0;
      min-height: 0;
    }
    .profile-export-advanced-intro {
      grid-column: 1 / -1;
      padding-left: var(--size-control-label-offset);
    }
    .profile-export-advanced-panel {
      display: grid;
      gap: 8px;
      align-content: start;
      min-width: 0;
      min-height: 0;
      padding: 8px;
      border: 1px solid var(--border);
      border-radius: var(--radius-control);
      background: var(--list);
    }
    .profile-export-advanced-panel--settings {
      grid-row: 2;
      grid-template-rows: auto auto auto auto minmax(0, 1fr);
      align-content: stretch;
      overflow: hidden;
    }
    .profile-export-advanced-panel--uncovered {
      grid-row: 3;
      grid-template-rows: auto minmax(0, 1fr);
      align-content: stretch;
      overflow: hidden;
    }
    .profile-export-advanced-panel--uncovered > .profile-export-list {
      min-height: 0;
      margin: 0;
      overflow: auto;
    }
    .profile-export-advanced-panel--uncovered > #profile-export-advanced-wizard-uncovered {
      min-height: 0;
      overflow: auto;
    }
    .profile-export-advanced-option {
      display: grid;
      gap: 4px;
      min-width: 0;
      padding: 6px;
      border: 1px solid var(--border);
      border-radius: var(--radius-control);
      background: var(--row);
    }
    .profile-export-advanced-option > .profile-export-switch-row {
      justify-self: stretch;
      justify-content: space-between;
    }
    .profile-export-advanced-option > .control-help-text {
      padding-left: var(--size-control-label-offset);
    }
    .profile-export-field--domains {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr);
      min-height: 0;
    }
    .profile-export-field--template {
      display: grid;
      gap: 4px;
      min-width: 0;
    }
    .profile-export-field--manual-domains {
      min-height: 0;
    }
    .profile-export-domains-input {
      width: 100%;
      height: 100%;
      min-height: 0;
      line-height: 18px;
      overflow: auto;
      resize: none;
    }
    .profile-export-domain-field-header {
      display: flex;
      align-items: end;
      justify-content: space-between;
      gap: 5px;
      min-width: 0;
    }
    .profile-export-domain-field-header > .value-label {
      padding-left: var(--size-control-label-offset);
      color: var(--muted);
      font-size: 11px;
      font-weight: 600;
      line-height: 16px;
    }
    .path-input-shell--textarea {
      height: 100%;
    }
    .path-input-shell--textarea > .profile-export-domains-input {
      padding-right: 30px;
    }
    .profile-export-section {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr);
      gap: 6px;
      height: 100%;
      min-height: 0;
      overflow: hidden;
    }
    .tabs {
      display: grid;
      gap: 0;
      min-width: 0;
    }
    .tabs__list {
      display: flex;
      flex-wrap: wrap;
      gap: 0;
      align-items: flex-end;
      min-width: 0;
      width: max-content;
      max-width: 100%;
      justify-self: start;
      border: 1px solid var(--border);
      border-bottom: 0;
      border-radius: var(--radius-control) var(--radius-control) 0 0;
      background: var(--chrome);
    }
    .tabs__tab {
      position: relative;
      appearance: none;
      min-height: 32px;
      padding: 6px 12px 8px;
      border: 0;
      border-right: 1px solid var(--border);
      border-radius: 0;
      background: var(--chrome);
      color: var(--text);
      font-size: 12px;
      font-weight: 700;
      line-height: 16px;
      text-align: left;
      cursor: pointer;
    }
    .tabs__tab:hover {
      background: var(--control-hover);
    }
    .tabs__tab[aria-selected="true"] {
      background: var(--chrome);
      color: var(--strong);
    }
    .tabs__tab[aria-selected="true"]::after {
      content: "";
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      height: 3px;
      background: var(--accent);
    }
    .tabs__body {
      display: grid;
      gap: 8px;
      min-width: 0;
      height: auto;
      min-height: 0;
      max-height: none;
      overflow: visible;
      padding: 8px;
      border: 1px solid var(--border);
      border-radius: 0 0 var(--radius-control) var(--radius-control);
      background: var(--chrome);
    }
    .tabs__panel {
      display: grid;
      gap: 8px;
      min-width: 0;
      align-content: start;
    }
    .module-ui-schema__tabs {
      display: flex;
      flex-direction: column;
      gap: 8px;
      min-width: 0;
      min-height: 0;
      box-sizing: border-box;
      overflow: hidden;
    }
    .module-ui-schema__tabs-shell {
      display: flex;
      flex: 1 1 auto;
      flex-direction: column;
      min-width: 0;
      min-height: 0;
      box-sizing: border-box;
      overflow: hidden;
    }
    .module-ui-schema__tabs-body {
      display: grid;
      align-content: start;
      gap: 8px;
      flex: 1 1 auto;
      min-width: 0;
      min-height: 0;
      box-sizing: border-box;
      overflow: hidden;
    }
    .module-ui-schema__tabs-body.module-ui-schema--scroll-x {
      overflow-x: auto;
      overflow-y: hidden;
      max-height: min(360px, 70vh);
    }
    .module-ui-schema__tabs-body.module-ui-schema--scroll-y {
      overflow-x: hidden;
      overflow-y: auto;
      max-height: min(360px, 70vh);
    }
    .module-ui-schema__tabs-body.module-ui-schema--scroll-both {
      overflow: auto;
      max-height: min(360px, 70vh);
    }
    .module-ui-schema__tabs-body.module-ui-schema--scroll-y,
    .module-ui-schema__tabs-body.module-ui-schema--scroll-both {
      padding-bottom: calc(10px + var(--size-compact-control));
      scroll-padding-bottom: calc(10px + var(--size-compact-control));
    }
    .profile-export-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(180px, 0.45fr);
      column-gap: 5px;
      row-gap: 4px;
    }
    .profile-export-grid--attach > .profile-export-field:first-child {
      grid-column: 1 / -1;
    }
    .profile-export-grid--attach > .profile-export-field:nth-child(2) {
      grid-column: 1;
    }
    .profile-export-grid--attach > .profile-export-field:nth-child(3) {
      grid-column: 2;
    }
    .profile-export-grid--attach > .profile-export-tab-help {
      margin-top: 4px;
    }
    .profile-export-path-row {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 5px;
    }
    .profile-export-path-row .path-input-shell > .input {
      min-height: 30px;
      height: 30px;
      border-radius: var(--radius);
      padding: 4px 30px 4px 5px;
    }
    .profile-export-path-row > .button,
    .profile-export-field > .input-box.input,
    .profile-export-field > .input-box.select {
      height: 30px;
      min-height: 30px;
    }
    .profile-export-field--domains > .input-box.input.profile-export-domains-input {
      height: 100%;
      min-height: 0;
    }
    .profile-export-grid--single { grid-template-columns: minmax(0, 1fr); }
    .profile-export-field { display: grid; gap: 4px; min-width: 0; }
    .profile-export-field > .value-label {
      padding-left: var(--size-control-label-offset);
    }
    .profile-export-field label {
      color: var(--muted);
      font-size: 11px;
      font-weight: 600;
      line-height: 16px;
    }
    .profile-export-switch-row {
      display: flex;
      align-items: center;
      gap: 8px;
      justify-self: start;
      max-width: 100%;
      min-height: 30px;
      min-width: 0;
    }
    .profile-export-switch-row__label {
      color: var(--muted);
      font-size: 11px;
      font-weight: 600;
      line-height: var(--size-switch-height);
    }
    .control-help-text {
      min-width: 0;
      margin: 0;
      padding: 0;
      background: transparent;
      color: var(--muted);
      font-size: 12px;
      font-weight: 400;
      line-height: 16px;
    }
    .profile-export-tab-help {
      grid-column: 1 / -1;
      align-self: end;
      justify-self: stretch;
      padding-left: var(--size-control-label-offset);
    }
    .profile-export-preview-panel {
      display: grid;
      grid-template-rows: auto minmax(0, 1fr);
      gap: 8px;
      min-width: 0;
      min-height: 0;
      padding: 8px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--chrome);
      overflow: hidden;
    }
    .profile-export-preview-actions {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
      justify-content: flex-start;
      min-width: 0;
    }
    .profile-export-preview-actions__right {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
      margin-left: auto;
    }
    .profile-export-footer {
      justify-content: space-between;
    }
    .profile-export-preview {
      display: grid;
      gap: 5px;
      height: 100%;
      min-height: 0;
      max-height: none;
      padding: 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--list);
      overflow: auto;
      align-content: start;
      justify-items: stretch;
    }
    .profile-export-preview__group {
      display: grid;
      gap: 3px;
      width: 100%;
      max-width: 100%;
      justify-items: stretch;
    }
    .profile-export-preview__section {
      display: grid;
      gap: 2px;
      justify-items: start;
    }
    .profile-export-preview__section + .profile-export-preview__section {
      margin-top: 10px;
    }
    .profile-export-preview__title {
      color: var(--strong);
      font-size: 12px;
      font-weight: 700;
      line-height: 16px;
    }
    .profile-export-preview__row {
      display: grid;
      grid-template-columns: 230px minmax(0, 1fr);
      gap: 6px;
      align-items: center;
      min-height: 24px;
      padding: 3px 5px;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--surface);
      color: var(--text);
      font-size: 12px;
      line-height: 16px;
      width: 100%;
      max-width: 100%;
      box-sizing: border-box;
    }
    .profile-export-preview__metric {
      min-height: 18px;
      padding: 0;
      border: 0;
      background: transparent;
    }
    .profile-export-preview__metric-value {
      color: var(--text);
      font-weight: 700;
      white-space: nowrap;
    }
    .profile-export-preview__metric-value--ready {
      color: var(--success);
    }
    .profile-export-preview__metric-value--excluded {
      color: var(--accent-border);
    }
    .profile-export-preview__metric .profile-export-preview__muted {
      color: var(--text);
      white-space: nowrap;
    }
    .profile-export-preview__muted {
      color: var(--muted);
    }
    .profile-export-preview__note {
      margin: 0;
      color: var(--muted);
      font-size: 12px;
      line-height: 16px;
    }
    .profile-export-preview__group--warning > .profile-export-preview__title,
    .profile-export-preview__title--warning {
      color: var(--warning);
    }
    .profile-export-list {
      display: grid;
      gap: 4px;
      margin: 0;
      padding-left: 18px;
      color: var(--text);
      font-size: 12px;
      line-height: 18px;
    }
    .profile-export-list--plain {
      color: var(--text);
    }
    .profile-export-feedback {
      color: var(--danger);
      font-size: 12px;
      line-height: 16px;
      min-height: 0;
      display: none;
    }
    .right { margin-left: auto; justify-self: end; }
    footer > .right { margin-left: 6px; }
    @media (max-width: 900px) {
      .controls { grid-template-columns: 1fr; }
      .integration-status-layout--module-menu { grid-template-columns: minmax(0, 1fr); }
      header { align-items: flex-start; flex-direction: column; height: auto; }
      .shell { grid-template-rows: auto minmax(0, 1fr) 34px; }
    }
  </style>
</head>
<body>
  <main class="shell shell--controls-disabled" id="app-shell" data-ui-disabled="true">
    <header id="browser-header">
      <div class="hero-labels" id="main-header-content">
      <div class="header-monitor-controls header-system-tools" data-ui-alias="system-tools">
        <button class="button button--icon button--monitoring header-action-button" id="toggle-monitoring-button" type="button" onclick="toggleMonitoring()" data-tooltip="Start monitoring" data-tooltip-align="end" aria-label="Start monitoring"><span class="button__icon-shell button__icon-shell--monitoring"><span class="button__icon-scale button__icon-scale--monitoring"><span class="button__icon-rotor button__icon-rotor--monitoring"><img class="button__icon button__icon--monitoring" src="/v1/assets/monitoring.svg" alt=""></span></span></span></button>
        <div class="header-action-separator" aria-hidden="true"></div>
        <div class="header-filter-block header-filter-block--app">
          <span class="header-filter-block__label" id="app-filter-label">App</span>
          <select class="input-box select header-filter-select" id="app-filter" onchange="setAppFilter(this.value)">
            <option value="">All</option>
          </select>
        </div>
        <div class="header-filter-block header-filter-block--ip">
          <span class="header-filter-block__label" id="ip-filter-label">IP</span>
          <div class="path-input-shell header-filter-field-shell">
            <input class="input-box input header-search-field" id="ip-filter" placeholder="Search by IP" onkeydown="applyIpFilterOnEnter(event)" data-clear-button="true">
            <button class="path-input-clear" id="clear-ip-filter-button" type="button" onclick="clearIpFilter()" data-clear-target="ip-filter" data-clear-button="true" data-ui-action="clear-ip-filter" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
          </div>
        </div>
        <div class="header-filter-block header-filter-block--domain">
          <span class="header-filter-block__label" id="domain-filter-label">Domain</span>
          <div class="path-input-shell header-filter-field-shell">
            <input class="input-box input header-search-field header-search-field--domain" id="domain-filter" placeholder="Domain" onkeydown="applyDomainFilterOnEnter(event)" data-clear-button="true" data-tooltip="Domain filter. Use * as any number of characters, for example *example.com or example*.com." data-tooltip-align="end">
            <button class="path-input-clear" id="clear-domain-filter-button" type="button" onclick="clearDomainFilter()" data-clear-target="domain-filter" data-clear-button="true" data-ui-action="clear-observation-domain-search" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
          </div>
        </div>
        <div class="header-filter-block header-filter-block--port">
          <span class="header-filter-block__label" id="port-filter-label">Port</span>
          <div class="path-input-shell header-filter-field-shell">
            <input class="input-box input header-search-field" id="port-filter" placeholder="Port" onkeydown="applyPortFilterOnEnter(event)" data-clear-button="true">
            <button class="path-input-clear" id="clear-port-filter-button" type="button" onclick="clearPortFilter()" data-clear-target="port-filter" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
          </div>
        </div>
        <div class="header-filter-block header-filter-block--protocol">
          <span class="header-filter-block__label" id="protocol-filter-label">Protocol</span>
          <select class="input-box select header-filter-select" id="protocol-filter" onchange="setProtocolFilter(this.value)">
            <option value="">All</option>
          </select>
        </div>
        <div class="header-filter-block header-filter-block--state">
          <span class="header-filter-block__label" id="observation-filter-label">State</span>
          <select class="input-box select header-filter-select" id="observation-filter" onchange="setObservationFilter(this.value)">
            <option value="All">All</option>
            <option value="Unconfirmed">Unconfirmed</option>
            <option value="Confirmed">Confirmed</option>
            <option value="Success">Success</option>
            <option value="Failed">Failure</option>
          </select>
        </div>
        <button class="button button--icon header-action-button" id="confirm-filtered-button" type="button" onclick="confirmFiltered()" data-tooltip="Monitoring: Add all visible rows for export" data-tooltip-align="end" aria-label="Monitoring: Add all visible rows for export"><img class="button__icon button__icon--confirm-filtered" src="/v1/assets/confirm-filtered.svg" alt=""></button>
        <button class="button button--icon header-action-button" id="unconfirm-filtered-button" type="button" onclick="unconfirmFiltered()" data-tooltip="Monitoring: Exclude all rows from export" data-tooltip-align="end" aria-label="Monitoring: Exclude all rows from export"><img class="button__icon button__icon--unconfirm-filtered" src="/v1/assets/unconfirm-filtered.svg" alt=""></button>
        <button class="button button--icon button--danger header-action-button" id="clear-monitoring-button" type="button" onclick="clearMonitoring()" data-tooltip="Monitoring: Clear monitoring" data-tooltip-align="end" aria-label="Monitoring: Clear monitoring"><img class="button__icon button__icon--trash" src="/v1/assets/trash.svg" alt=""></button>
        <div class="header-action-separator" aria-hidden="true"></div>
        <button class="button button--icon header-action-button" id="cloud-import-button" type="button" onclick="toggleCloudPanel('import')" data-tooltip="Cloud import" data-tooltip-align="end" aria-label="Cloud import"><img class="button__icon button__icon--cloud-import" src="/v1/assets/import-cloud.svg" alt=""></button>
        <button class="button button--icon header-action-button" id="cloud-export-button" type="button" onclick="toggleCloudPanel('export')" data-tooltip="Cloud export" data-tooltip-align="end" aria-label="Cloud export"><img class="button__icon button__icon--cloud-sync" src="/v1/assets/export-cloud.svg" alt=""></button>
        <button class="button button--icon header-action-button" id="import-csv-button" type="button" onclick="openCsvImportPicker()" data-tooltip="Monitoring: Import CSV" data-tooltip-align="end" aria-label="Monitoring: Import CSV"><img class="button__icon button__icon--import-csv" src="/v1/assets/import-csv.svg" alt=""></button>
        <button class="button button--icon header-action-button" id="export-csv-button" type="button" onclick="openCsvExportPicker()" data-tooltip="Monitoring: Export selected to CSV" data-tooltip-align="end" aria-label="Monitoring: Export selected to CSV"><img class="button__icon button__icon--export-csv" src="/v1/assets/export-csv.svg" alt=""></button>
        <div class="header-action-separator" aria-hidden="true"></div>
      </div>
      <div class="header-switch-stack" id="header-labels"></div>
      </div>
      <div class="module-header-content" id="module-header-content" data-ui-entity="module-header" hidden>
        <div class="module-header-title" id="module-header-title">Modules</div>
        <div class="module-header-controls">
          <div class="module-header-actions" id="module-header-actions" data-ui-entity="module-header-actions"></div>
          <div class="header-action-separator" aria-hidden="true"></div>
          <div class="module-header-standard-actions">
            <button class="button button--icon header-action-button" id="module-header-stop-button" type="button" data-ui-action="stop-module-background" onclick="stopModuleBackgroundAndExit()" data-tooltip="Stop" data-tooltip-align="end" aria-label="Stop"><img class="button__icon button__icon--module-stop" src="/v1/assets/module-stop.svg" alt=""></button>
            <button class="button button--icon header-action-button" id="module-header-close-button" type="button" data-ui-action="close-module-overlays" onclick="closeModuleOverlays()" data-tooltip="Close" data-tooltip-align="end" aria-label="Close"><img class="button__icon button__icon--module-close" src="/v1/assets/module-close.svg" alt=""></button>
          </div>
        </div>
      </div>
    </header>
    <div class="content">
      <div class="workspace" id="browser-workspace">
        <div class="column">
          <section class="card tracked-apps-card" id="tracked-apps-panel">
            <div class="card__header tracked-apps-header">
              <div class="title-with-help">
                <h2 id="tracked-apps-title">Tracked apps</h2>
                <span class="help-icon" id="tracked-apps-help" data-tooltip="Add executable paths or toggle apps in the shell state." data-tooltip-align="start" aria-label="Add executable paths or toggle apps in the shell state." tabindex="0">?</span>
              </div>
              <div class="tracked-apps-header-meta">
                <span class="panel-header-meta panel-header-meta--danger" id="tracked-enabled-count">On: 0</span>
              </div>
            </div>
            <div class="stack">
              <div class="exe-path-row">
                <div class="path-input-shell">
                  <input class="input-box input" id="exe-path" placeholder="Path" data-commit-on-enter="true" data-clear-button="true" onkeydown="addTrackedAppOnEnter(event)">
                  <button class="path-input-clear" id="clear-exe-path-button" type="button" onclick="clearExePath()" data-clear-target="exe-path" data-clear-button="true" data-ui-action="clear-exe-path" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                </div>
                <button class="button button--icon" id="add-exe-button" type="button" onclick="addTrackedApp()" data-tooltip="Add execution file" data-tooltip-align="end" aria-label="Add execution file"><span class="button__plus" aria-hidden="true">+</span></button>
              </div>
              <div class="tracked-apps-toolbar-spacer" aria-hidden="true"></div>
              <div class="tracked-apps-bulk-row">
                <label class="tracked-apps-bulk-label" id="enable-all-label">Enable all</label>
                <button id="enable-all-switch" class="input-box switch" type="button" onclick="toggleEnableAll()" aria-label="Enable all"><span class="switch__knob"></span></button>
                <span class="tracked-apps-count-label" id="tracked-count-inline">0 items</span>
              </div>
            </div>
            <div class="list tracked-list" id="tracked-apps">
              <div class="tracked-app-skeleton"></div>
              <div class="tracked-app-skeleton"></div>
              <div class="tracked-app-skeleton"></div>
              <div class="tracked-app-skeleton"></div>
              <div class="tracked-app-skeleton"></div>
            </div>
          </section>
        </div>
        <div class="column">
          <section class="card modules-card" id="integration-panel">
            <div class="card__header">
              <div class="title-with-help">
                <h2 id="integration-title">Modules</h2>
                <span class="help-icon" id="integration-help" data-tooltip="External modules available to the local workspace." data-tooltip-align="start" aria-label="External modules available to the local workspace." tabindex="0">?</span>
              </div>
              <div class="modules-card__header-actions">
                <span class="modules-card__order-label" id="modules-order-label">Display order</span>
                <button id="modules-order-switch" class="input-box switch" type="button" onclick="toggleModuleOrderEditing()" aria-label="Display order"><span class="switch__knob"></span></button>
                <span class="panel-header-meta" id="integration-ready"></span>
              </div>
            </div>
            <div class="integration-status integration-status--compact" id="integration-info">
              <div class="integration-module-grid" id="integration-module-actions"></div>
            </div>
          </section>
          <section class="card ignored-addresses-card" id="ignored-addresses-panel">
            <div class="card__header">
              <div class="title-with-help">
                <h2 id="ignored-addresses-panel-title">Ignored addresses</h2>
                <span class="help-icon" id="ignored-addresses-panel-help" data-tooltip="Addresses in this list are hidden from monitoring views and filtered actions." data-tooltip-align="start" aria-label="Addresses in this list are hidden from monitoring views and filtered actions." tabindex="0">?</span>
              </div>
            </div>
            <div class="ignored-addresses-section">
              <div class="ignored-addresses" id="ignored-addresses"></div>
            </div>
          </section>
        </div>
        <section class="card observations-card" id="browser-observations-panel">
          <div class="card__header">
            <div class="title-with-help">
              <h2 id="observations-title">Monitoring</h2>
              <span class="help-icon" id="observations-help" data-tooltip="Snapshot-shaped rows keep the client close to the real API." data-tooltip-align="start" aria-label="Snapshot-shaped rows keep the client close to the real API." tabindex="0">?</span>
            </div>
            <div class="monitoring-header-meta">
              <label class="header-switch-row header-switch-row--filter monitoring-public-toggle" id="public-ip-label" data-tooltip="On shows public IPs, off shows non-public IPs." data-tooltip-align="end"><span class="header-switch-row__label">Public</span>
                <button class="input-box switch" id="public-ip-filter" type="button" role="switch" aria-checked="false" onclick="togglePublicIpFilter()" aria-label="Public"><span class="switch__knob"></span></button>
              </label>
            </div>
          </div>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th id="table-app" class="table-sortable" onclick="setObservationSort('app')"><span class="table-sortable__content"><img id="table-app-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-app-label">App</span></span></th><th id="table-ip" class="table-sortable" onclick="setObservationSort('ip')"><span class="table-sortable__content"><img id="table-ip-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-ip-label">IP</span></span></th><th id="table-domain" class="table-sortable" onclick="setObservationSort('domain')"><span class="table-sortable__content"><img id="table-domain-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-domain-label">Domain</span></span></th><th id="table-port" class="table-sortable" onclick="setObservationSort('port')"><span class="table-sortable__content"><img id="table-port-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-port-label">Port</span></span></th><th id="table-proto" class="table-sortable" onclick="setObservationSort('protocol')"><span class="table-sortable__content"><img id="table-proto-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-proto-label">Proto</span></span></th><th id="table-conn" class="table-sortable" onclick="setObservationSort('connection')"><span class="table-sortable__content"><img id="table-conn-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-conn-label">Conn</span></span></th>
                  <th id="table-hits" class="table-sortable" onclick="setObservationSort('hits')"><span class="table-sortable__content"><img id="table-hits-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-hits-label">Hits</span></span></th><th id="table-first-seen" class="table-sortable" onclick="setObservationSort('first_seen')"><span class="table-sortable__content"><img id="table-first-seen-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-first-seen-label">First seen</span></span></th><th id="table-last-seen" class="table-sortable" onclick="setObservationSort('last_seen')"><span class="table-sortable__content"><img id="table-last-seen-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label" id="table-last-seen-label">Last seen</span></span></th><th id="table-action"><span id="table-action-label">Action</span></th>
                </tr>
              </thead>
              <tbody id="observations"></tbody>
            </table>
          </div>
          <div class="panel-footer monitoring-panel-footer" data-ui-entity="panel-footer">
            <span class="panel-footer-meta" id="observations-count"><span class="panel-footer-meta__item">Total rows: 0</span><span class="panel-footer-meta__item">Displayed rows: 0</span><span class="panel-footer-meta__item">Selected rows: 0</span></span>
          </div>
        </section>
        <section class="card cloud-web-panel cloud-web-panel--import" id="browser-cloud-import-panel" hidden>
          <div class="card__header">
            <div class="title-with-help">
              <h2 id="cloud-import-panel-title">Import Cloud</h2>
              <span class="help-icon" id="cloud-import-panel-help" data-tooltip="Cloud import panel." data-tooltip-align="start" aria-label="Cloud import panel." tabindex="0">?</span>
            </div>
            <span class="panel-header-meta cloud-web-panel-service" id="cloud-import-panel-status" data-tooltip="" data-tooltip-align="end"><span class="cloud-web-panel-service__text">Service online</span><span class="footer-watcher-dot footer-watcher-dot--connected" aria-hidden="true"></span></span>
          </div>
          <div class="cloud-web-import-layout">
            <div class="cloud-web-subpanel">
              <div class="cloud-web-filter-grid" id="cloud-import-app-filters">
                <div class="cloud-web-filter-block">
                  <span class="header-filter-block__label" id="cloud-import-app-filter-label">App</span>
                  <div class="path-input-shell">
                    <input class="input-box input" id="cloud-import-app-filter" type="text" autocomplete="off" placeholder="Search by app name" onkeydown="applyCloudTextFilterOnEnter(event)" data-clear-button="true">
                    <button class="path-input-clear" type="button" onclick="clearCloudImportFilter('cloud-import-app-filter')" data-clear-target="cloud-import-app-filter" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                  </div>
                </div>
                <div class="cloud-web-filter-block">
                  <span class="header-filter-block__label" id="cloud-import-company-filter-label">Company</span>
                  <div class="path-input-shell">
                    <input class="input-box input" id="cloud-import-company-filter" type="text" autocomplete="off" placeholder="Search by company" onkeydown="applyCloudTextFilterOnEnter(event)" data-clear-button="true">
                    <button class="path-input-clear" type="button" onclick="clearCloudImportFilter('cloud-import-company-filter')" data-clear-target="cloud-import-company-filter" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                  </div>
                </div>
                <div class="cloud-web-filter-block">
                  <span class="header-filter-block__label" id="cloud-import-author-filter-label">Author</span>
                  <div class="path-input-shell">
                    <input class="input-box input" id="cloud-import-author-filter" type="text" autocomplete="off" placeholder="Search by author" onkeydown="applyCloudTextFilterOnEnter(event)" data-clear-button="true">
                    <button class="path-input-clear" type="button" onclick="clearCloudImportFilter('cloud-import-author-filter')" data-clear-target="cloud-import-author-filter" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                  </div>
                </div>
                <div class="cloud-web-filter-block cloud-web-filter-block--visibility">
                  <span class="header-filter-block__label" id="cloud-import-visibility-filter-label">Private</span>
                  <select class="input-box select header-filter-select" id="cloud-import-visibility-filter" onchange="setCloudImportVisibilityScope(this.value)">
                    <option value="all">All</option>
                    <option value="public">Public</option>
                    <option value="private">Private</option>
                  </select>
                </div>
                <button class="button button--icon header-action-button cloud-web-my-publications-button" id="cloud-import-scope-mine" type="button" onclick="toggleCloudMineFilter()" data-tooltip="My publications" data-tooltip-align="end" aria-label="My publications"><img class="button__icon button__icon--my-publications" src="/v1/assets/my-publications.svg" alt=""></button>
              </div>
              <div class="table-wrap">
                <table>
                  <colgroup>
                    <col>
                    <col>
                    <col class="browser-table-col-count">
                    <col class="browser-table-col-author">
                    <col class="cloud-web-action-column">
                  </colgroup>
                  <thead>
                    <tr>
                      <th id="cloud-import-table-app" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">App</span></span></th>
                      <th id="cloud-import-table-company" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Company</span></span></th>
                      <th id="cloud-import-table-rows" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Rows</span></span></th>
                      <th id="cloud-import-table-authors" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Authors</span></span></th>
                      <th class="cloud-web-action-header" aria-hidden="true"></th>
                    </tr>
                  </thead>
                  <tbody id="cloud-import-apps">
                    <tr><td colspan="5" id="cloud-import-placeholder">Application list has not been loaded yet.</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
            <div class="cloud-web-subpanel">
              <div class="table-wrap">
                <table>
                  <colgroup>
                    <col>
                    <col class="browser-table-col-ip">
                    <col>
                    <col class="browser-table-col-port">
                    <col class="browser-table-col-protocol">
                    <col class="browser-table-col-connection">
                    <col class="browser-table-col-count">
                    <col class="browser-table-col-author">
                    <col class="browser-table-col-privacy">
                    <col class="cloud-web-action-column">
                  </colgroup>
                  <thead>
                    <tr>
                      <th id="cloud-import-row-table-app" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">App</span></span></th>
                      <th id="cloud-import-row-table-ip" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">IP</span></span></th>
                      <th id="cloud-import-row-table-domain" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Domain</span></span></th>
                      <th id="cloud-import-row-table-port" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Port</span></span></th>
                      <th id="cloud-import-row-table-protocol" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Proto</span></span></th>
                      <th id="cloud-import-row-table-connection" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Conn</span></span></th>
                      <th id="cloud-import-row-table-hits" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Cnt.</span></span></th>
                      <th id="cloud-import-row-table-author" class="table-sortable"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Author</span></span></th>
                      <th id="cloud-import-row-table-privacy" data-tooltip="true means private; false means public." data-tooltip-align="end">Private</th>
                      <th class="cloud-web-action-header" aria-hidden="true"></th>
                    </tr>
                  </thead>
                  <tbody id="cloud-import-rows">
                    <tr><td colspan="10" id="cloud-import-rows-placeholder">Choose an application and download cloud data.</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
          <div class="cloud-web-panel-footer">
            <span class="panel-footer-meta" id="cloud-import-row-summary"><span class="panel-footer-meta__item">Total rows: 0</span><span class="panel-footer-meta__item">Selected rows: 0</span><span class="panel-footer-meta__item" id="cloud-import-quota">Download: 0/0</span></span>
            <span class="cloud-web-panel-footer__progress" id="cloud-import-progress" data-ui-entity="progress-bar" data-ui-key="cloud-download-footer-progress" hidden></span>
            <span class="cloud-web-panel-footer__spacer" aria-hidden="true"></span>
            <button class="input-box button button--primary" id="cloud-import-add-monitoring-button" type="button" onclick="addSelectedCloudRowsToMonitoring()">Add to monitoring</button>
            <button class="input-box button button--primary" id="cloud-import-export-csv-button" type="button" onclick="exportSelectedCloudRowsCsv()">Export CSV</button>
          </div>
        </section>
        <section class="card cloud-web-panel cloud-web-panel--export" id="browser-cloud-export-panel" hidden>
          <div class="card__header">
            <div class="title-with-help">
              <h2 id="cloud-export-panel-title">Export Cloud</h2>
              <span class="help-icon" id="cloud-export-panel-help" data-tooltip="Cloud export panel." data-tooltip-align="start" aria-label="Cloud export panel." tabindex="0">?</span>
            </div>
            <span class="panel-header-meta cloud-web-panel-service" id="cloud-export-panel-status" data-tooltip="" data-tooltip-align="end"><span class="cloud-web-panel-service__text">Service online</span><span class="footer-watcher-dot footer-watcher-dot--connected" aria-hidden="true"></span></span>
          </div>
          <div class="cloud-web-panel-body cloud-web-export-body">
            <div class="cloud-web-subpanel cloud-web-export-auth-subpanel">
              <div class="cloud-web-auth-row">
                <button class="input-box button button--secondary" id="cloud-export-auth-button" type="button" disabled>Authorization</button>
                <button class="input-box button button--secondary" id="cloud-export-sign-out-button" type="button" onclick="signOutCloudWeb()">Sign out</button>
                <span class="cloud-web-auth-user" id="cloud-export-auth-user">-</span>
              </div>
              <div class="cloud-web-nickname-row">
                <input class="input-box input" id="cloud-export-nickname" type="text" maxlength="32" autocomplete="off" placeholder="Nickname" data-commit-on-enter="true" onchange="updateCloudExportNicknameDraft()" onkeydown="applyCloudExportNicknameOnEnter(event)">
                <label class="cloud-web-nickname-status" id="cloud-export-private-label" data-tooltip="Only your signed-in account can download this upload." data-tooltip-align="end">
                  <input id="cloud-export-private-upload" type="checkbox" onchange="toggleCloudPrivateUpload(event)">
                  <span id="cloud-export-private-text">Private upload</span>
                </label>
                <span class="cloud-web-nickname-status" id="cloud-export-nickname-status"></span>
              </div>
            </div>
            <div class="cloud-web-subpanel cloud-web-export-publications-subpanel">
              <div class="cloud-sync-publications-title" id="cloud-export-publications-title">Publications</div>
              <div class="table-wrap cloud-web-publications-table">
                <div class="table-header-wrap">
                  <div class="table-header-scroll">
                    <table class="observations-table observations-table--header cloud-web-publications-data-table cloud-web-publications-data-table--header">
                      <thead>
                        <tr>
                          <th id="cloud-export-publications-app" class="table-sortable" onclick="setCloudPublicationSort('app')"><span class="table-sortable__content"><img id="cloud-export-publications-app-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">App</span></span></th>
                          <th id="cloud-export-publications-new-rows" class="table-sortable" onclick="setCloudPublicationSort('new_rows')"><span class="table-sortable__content"><img id="cloud-export-publications-new-rows-sort-icon" class="table-sortable__icon table-sortable__icon--desc" src="/v1/assets/sort-desc.svg" alt=""><span class="table-sortable__label">New data</span></span></th>
                          <th id="cloud-export-publications-author-rows" class="table-sortable" onclick="setCloudPublicationSort('author_rows')"><span class="table-sortable__content"><img id="cloud-export-publications-author-rows-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Author rows</span></span></th>
                          <th id="cloud-export-publications-total-rows" class="table-sortable" onclick="setCloudPublicationSort('total_rows')"><span class="table-sortable__content"><img id="cloud-export-publications-total-rows-sort-icon" class="table-sortable__icon table-sortable__icon--idle" src="/v1/assets/sort-idle.svg" alt=""><span class="table-sortable__label">Total rows</span></span></th>
                        </tr>
                      </thead>
                    </table>
                  </div>
                  <div class="table-header-scrollbar-fill" aria-hidden="true"></div>
                </div>
                <div class="table-body-wrap">
                  <table class="observations-table observations-table--body cloud-web-publications-data-table cloud-web-publications-data-table--body">
                    <tbody id="cloud-export-publications-rows">
                      <tr><td colspan="4" id="cloud-export-publications-placeholder">Sign in to see applications you published.</td></tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
          <div class="cloud-web-panel-footer">
            <span id="cloud-export-quota">Upload: 0/0</span>
            <span class="cloud-web-panel-footer__progress" id="cloud-export-progress" data-ui-entity="progress-bar" data-ui-key="cloud-upload-footer-progress" hidden></span>
            <span class="cloud-sync-footer-identifier" id="cloud-export-identifier" data-tooltip="" data-tooltip-align="start" role="button" tabindex="0" onclick="copyCloudIdentifier(event)">
              <span class="cloud-sync-footer-identifier__label" id="cloud-export-identifier-label">Identifier:</span>
              <span class="cloud-sync-footer-identifier__value" id="cloud-export-identifier-value">-</span>
            </span>
            <span class="cloud-sync-footer-auth" id="cloud-export-footer-auth" data-tooltip="" data-tooltip-align="end">
              <span class="cloud-sync-footer-identifier__label" id="cloud-export-footer-auth-label">Authorization:</span>
              <span class="cloud-sync-footer-identifier__value" id="cloud-export-footer-auth-value">-</span>
            </span>
            <span class="cloud-web-panel-footer__spacer" aria-hidden="true"></span>
            <button class="input-box button button--primary" id="cloud-export-send-button" type="button" onclick="requestCloudExportFromNative()">Send</button>
          </div>
        </section>
      </div>
    </div>
    <footer id="browser-footer">
      <span class="app-footer-panel" id="footer-apps">Apps: 0/0</span>
      <span class="app-footer-panel footer-watcher-status" id="footer-watcher-status" data-tooltip="Watcher unavailable (embedded watcher)." tabindex="0">
        <span id="footer-watcher-label">Watcher</span>
        <span class="footer-watcher-dot footer-watcher-dot--inactive" id="footer-watcher-dot" aria-hidden="true"></span>
      </span>
      <span class="app-footer-panel footer-tool-status" id="footer-tool-status" data-tooltip="Tool unavailable (netstitch-tool native library)." tabindex="0">
        <span id="footer-tool-label">Tool</span>
        <span class="footer-watcher-dot footer-watcher-dot--inactive" id="footer-tool-dot" aria-hidden="true"></span>
      </span>
      <span class="app-footer-panel footer-web-server-status" id="footer-web-server-status" data-tooltip="Local browser web server is disabled." onclick="copyWebServerUrl(event)" tabindex="0">
        <span id="footer-web-server-label">Web server</span>
        <span class="footer-watcher-dot footer-watcher-dot--inactive" id="footer-web-server-dot" aria-hidden="true"></span>
      </span>
      <span class="app-footer-panel footer-network-status" id="footer-network-status" data-tooltip="endpoint probes are checking." tabindex="0">
        <span id="footer-network-label">DNS</span>
        <span class="footer-watcher-dot footer-watcher-dot--inactive" id="footer-network-dot" aria-hidden="true"></span>
      </span>
      <span class="app-footer-panel app-footer-message-panel" id="footer-message-panel">
        <button class="input-box button button--square button--copy-status" id="footer-status-copy" type="button" onclick="copyStatusHistory(event)" data-tooltip="Copy the last 100 status lines" data-tooltip-align="start" aria-label="Copy the last 100 status lines">
          <img class="button__icon" src="/v1/assets/copy.svg" alt="">
        </button>
        <span class="footer-message-label" id="footer-message-label">Message:</span>
        <input class="footer-message-text path-field" id="footer-message-text" type="text" readonly value="" aria-label="Message" data-tooltip="" data-tooltip-align="end">
      </span>
      <div class="app-footer-panel app-footer-actions app-footer-language-panel">
        <span id="language-label">Language:</span>
        <div class="language-select-shell">
          <button class="language-select-control input-box" id="language-select" type="button" onclick="toggleLanguageMenu(event)" aria-haspopup="listbox" aria-expanded="false">
            <span class="language-select-control__label" id="language-select-label">English (EN)</span>
          </button>
          <div class="language-select-menu" id="language-select-menu" role="listbox" hidden></div>
        </div>
      </div>
    </footer>
  </main>
  <div class="modal-backdrop module-overlay-backdrop" id="integration-module-modal" hidden>
    <section class="modal modal--panel integration-module-dialog" role="dialog" aria-modal="true" aria-labelledby="integration-module-modal-title" data-ui-entity="netstitch-ui-integration-module-dialog">
      <div class="modal__header" data-ui-entity="panel-header">
        <div class="title-with-help">
          <h2 id="integration-module-modal-title">Integration</h2>
          <span class="help-icon" id="integration-module-modal-help" data-tooltip="External module." data-tooltip-align="start" aria-label="External module." tabindex="0">?</span>
        </div>
      </div>
      <div class="modal__body integration-module-dialog__body" id="integration-module-modal-body" data-ui-entity="subpanel">
        <div class="module-ui-schema" id="integration-module-ui-schema" data-ui-entity="module-ui-schema"></div>
      </div>
      <div class="modal__footer" data-ui-entity="panel-footer">
        <div class="module-ui-schema__footer-host" id="integration-module-modal-footer-actions"></div>
        <span class="module-ui-schema__footer-value" id="integration-module-modal-footer-value"></span>
        <button class="button module-ui-schema__footer-nav" id="integration-module-modal-close-button" data-ui-action="back-integration-module" onclick="backIntegrationModulePanel()">Back</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop module-host-dialog-backdrop" id="module-host-dialog-modal" hidden>
    <section class="modal modal--compact module-host-dialog" role="dialog" aria-modal="true" aria-labelledby="module-host-dialog-title" data-ui-entity="module-host-dialog">
      <div class="modal__header" data-ui-entity="panel-header">
        <div class="title-with-help">
          <h2 id="module-host-dialog-title">Module</h2>
        </div>
      </div>
      <div class="modal__body module-host-dialog__body">
        <div class="module-host-dialog__icon" id="module-host-dialog-icon" aria-hidden="true"></div>
        <div class="module-host-dialog__message" id="module-host-dialog-message"></div>
      </div>
      <div class="modal__footer" data-ui-entity="panel-footer">
        <button class="input-box button button--secondary" id="module-host-dialog-cancel" type="button" onclick="closeModuleHostDialog('cancel')">Cancel</button>
        <button class="input-box button button--primary" id="module-host-dialog-ok" type="button" onclick="closeModuleHostDialog('ok')">OK</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="integration-modal" hidden>
    <section class="modal modal--panel" role="dialog" aria-modal="true" aria-labelledby="integration-modal-title">
      <div class="modal__header">
        <div>
          <div class="title-with-help">
            <h2 id="integration-modal-title">Module</h2>
            <span class="help-icon" id="integration-modal-help" data-tooltip="Prepare the module for later data export." data-tooltip-align="start" aria-label="Prepare the module for later data export." tabindex="0">?</span>
          </div>
        </div>
      </div>
      <div class="modal__body">
        <div class="integration-dialog-status-list" id="integration-modal-status"></div>
        <div class="button-row integration-download-actions" id="integration-download-actions"></div>
        <div class="integration-folder-field">
          <label class="integration-folder-path-help" for="integration-dir" id="integration-folder-path-help">Module path for later data export.</label>
          <div class="field-row integration-folder-field-row">
            <div class="path-input-shell">
              <input class="input-box input" id="integration-dir" placeholder="Path" onkeydown="applyIntegrationPathOnEnter(event)" data-clear-button="true">
              <button class="path-input-clear" id="clear-integration-folder-button" type="button" onclick="clearIntegrationPath()" data-clear-target="integration-dir" data-clear-button="true" data-ui-action="clear-integration-folder" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
            </div>
            <button class="input-box button" id="browse-integration-folder-button" type="button" data-ui-action="browse-integration-folder" onclick="browseIntegrationFolder()">Browse</button>
          </div>
        </div>
      </div>
      <div class="modal__footer">
        <button class="button" id="integration-modal-cancel-button" onclick="closeIntegrationDialog()">Cancel</button>
        <button class="button button--primary" id="integration-modal-use-button" onclick="useIntegrationFolder()">Assign folder</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="export-profile-modal" hidden>
    <section class="modal modal--panel profile-export-dialog" role="dialog" aria-modal="true" aria-labelledby="export-profile-modal-title" data-ui-entity="netstitch-ui-profile-export-dialog">
      <div class="modal__header">
        <div>
          <div class="title-with-help">
            <h2 id="export-profile-modal-title">Profile export</h2>
            <span class="help-icon" id="export-profile-modal-help" data-tooltip="Preview and apply profile-aware export changes." data-tooltip-align="start" aria-label="Preview and apply profile-aware export changes." tabindex="0">?</span>
          </div>
        </div>
      </div>
      <div class="modal__body">
        <div class="profile-export-section">
          <div class="tabs profile-export-tabs" data-ui-entity="tabs">
            <div class="tabs__list" id="profile-export-tabs" role="tablist">
              <button class="tabs__tab" id="profile-export-mode-attach-button" role="tab" data-ui-action="select-profile-export-mode" data-profile-export-mode="attach_netstitch_lists" onclick="selectProfileExportMode('attach_netstitch_lists')">Attach</button>
              <button class="tabs__tab" id="profile-export-mode-patch-button" role="tab" data-ui-action="select-profile-export-mode" data-profile-export-mode="patch_selected_profile" onclick="selectProfileExportMode('patch_selected_profile')">Patch</button>
              <button class="tabs__tab" id="profile-export-mode-merge-button" role="tab" data-ui-action="select-profile-export-mode" data-profile-export-mode="merge_into_existing_lists" onclick="selectProfileExportMode('merge_into_existing_lists')">Merge</button>
            </div>
            <div class="tabs__body profile-export-tabs-body" id="profile-export-tabs-body" data-ui-entity="tabs-body">
              <div class="tabs__panel" id="profile-export-attach-panel" role="tabpanel" data-profile-export-panel="attach_netstitch_lists">
                <div class="profile-export-grid profile-export-grid--attach">
                  <div class="profile-export-field">
                    <label for="profile-export-attach-profile-input" id="profile-export-attach-profile-label">Selected profile path</label>
                    <div class="field-row profile-export-path-row">
                      <div class="path-input-shell">
                        <input class="input-box input" id="profile-export-attach-profile-input" data-ui-entity="netstitch-ui-profile-export-profile-input" placeholder="Path" autocomplete="off" data-commit-on-enter="true" data-clear-button="true" onchange="updateProfileExportDraft()" onkeydown="applyProfileExportInputOnEnter(event)">
                        <button class="path-input-clear" id="clear-profile-export-attach-profile-button" type="button" data-ui-action="clear-profile-export-profile" onclick="clearProfileExportPath()" data-clear-target="profile-export-attach-profile-input" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                      </div>
                      <button class="input-box button" id="browse-profile-export-attach-profile-button" type="button" data-ui-action="browse-profile-export-profile" onclick="browseProfileExportProfile()">Browse</button>
                    </div>
                  </div>
                  <div class="profile-export-field">
                    <label for="profile-export-attach-profile-select" id="profile-export-attach-profile-select-label">Available profiles</label>
                    <select class="input-box select" id="profile-export-attach-profile-select" data-ui-entity="netstitch-ui-profile-export-profile-select" onchange="selectProfileExportProfile(this.value)"></select>
                  </div>
                  <div class="profile-export-field">
                    <label for="profile-export-attach-generated-name-input" id="profile-export-attach-generated-name-label">Generated profile name</label>
                    <input class="input-box input" id="profile-export-attach-generated-name-input" data-ui-entity="netstitch-ui-profile-export-generated-name-input" value="NetStitch" autocomplete="off" data-commit-on-enter="true" onchange="updateProfileExportDraft()" onkeydown="applyProfileExportInputOnEnter(event)">
                  </div>
                  <div class="control-help-text profile-export-tab-help" id="profile-export-attach-note" data-ui-entity="help-text"></div>
                </div>
              </div>
              <div class="tabs__panel" id="profile-export-patch-panel" role="tabpanel" data-profile-export-panel="patch_selected_profile" hidden>
                <div class="profile-export-grid profile-export-grid--single">
                  <div class="profile-export-field">
                    <label for="profile-export-patch-profile-input" id="profile-export-patch-profile-label">Selected profile path</label>
                    <div class="field-row profile-export-path-row">
                      <div class="path-input-shell">
                        <input class="input-box input" id="profile-export-patch-profile-input" data-ui-entity="netstitch-ui-profile-export-profile-input" placeholder="Path" autocomplete="off" data-commit-on-enter="true" data-clear-button="true" onchange="updateProfileExportDraft()" onkeydown="applyProfileExportInputOnEnter(event)">
                        <button class="path-input-clear" id="clear-profile-export-patch-profile-button" type="button" data-ui-action="clear-profile-export-profile" onclick="clearProfileExportPath()" data-clear-target="profile-export-patch-profile-input" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                      </div>
                      <button class="input-box button" id="browse-profile-export-patch-profile-button" type="button" data-ui-action="browse-profile-export-profile" onclick="browseProfileExportProfile()">Browse</button>
                    </div>
                  </div>
                  <div class="profile-export-field">
                    <label for="profile-export-patch-profile-select" id="profile-export-patch-profile-select-label">Available profiles</label>
                    <select class="input-box select" id="profile-export-patch-profile-select" data-ui-entity="netstitch-ui-profile-export-profile-select" onchange="selectProfileExportProfile(this.value)"></select>
                  </div>
                  <div class="control-help-text profile-export-tab-help" id="profile-export-patch-note" data-ui-entity="help-text"></div>
                </div>
              </div>
              <div class="tabs__panel" id="profile-export-merge-panel" role="tabpanel" data-profile-export-panel="merge_into_existing_lists" hidden>
                <div class="profile-export-grid profile-export-grid--single">
                  <div class="profile-export-field">
                    <label for="profile-export-merge-profile-input" id="profile-export-merge-profile-label">Selected profile path</label>
                    <div class="field-row profile-export-path-row">
                      <div class="path-input-shell">
                        <input class="input-box input" id="profile-export-merge-profile-input" data-ui-entity="netstitch-ui-profile-export-profile-input" placeholder="Path" autocomplete="off" data-commit-on-enter="true" data-clear-button="true" onchange="updateProfileExportDraft()" onkeydown="applyProfileExportInputOnEnter(event)">
                        <button class="path-input-clear" id="clear-profile-export-merge-profile-button" type="button" data-ui-action="clear-profile-export-profile" onclick="clearProfileExportPath()" data-clear-target="profile-export-merge-profile-input" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
                      </div>
                      <button class="input-box button" id="browse-profile-export-merge-profile-button" type="button" data-ui-action="browse-profile-export-profile" onclick="browseProfileExportProfile()">Browse</button>
                    </div>
                  </div>
                  <div class="profile-export-switch-row" id="profile-export-dangerous-row">
                    <span class="profile-export-switch-row__label" id="profile-export-dangerous-text">Allow dangerous merge</span>
                    <button class="input-box switch" id="profile-export-dangerous-switch" type="button" role="switch" aria-checked="false" data-ui-action="toggle-profile-export-dangerous" onclick="toggleProfileExportDangerous()"><span class="switch__knob"></span></button>
                  </div>
                  <div class="control-help-text profile-export-tab-help" id="profile-export-merge-note" data-ui-entity="help-text"></div>
                </div>
              </div>
            </div>
          </div>
          <div class="profile-export-feedback" id="profile-export-feedback"></div>
          <div class="profile-export-preview-panel" id="profile-export-preview-panel">
            <div class="profile-export-preview-actions">
              <button class="button" id="profile-export-analyze-button" data-ui-action="analyze-profile-export" onclick="analyzeProfileExport()">Analyze</button>
              <button class="button" id="profile-export-advanced-wizard-button" data-ui-action="open-profile-export-advanced-wizard" onclick="openProfileExportAdvancedWizard()">Additional settings</button>
              <div class="profile-export-preview-actions__right">
                <button class="button" id="profile-export-backup-button" data-ui-action="backup-profile-export" onclick="backupProfileExport()">Create backup</button>
                <button class="button" id="profile-export-revert-button" data-ui-action="revert-profile-export" onclick="revertProfileExport()">Revert changes</button>
              </div>
            </div>
            <div class="profile-export-preview" id="profile-export-preview"></div>
          </div>
        </div>
      </div>
      <div class="modal__footer profile-export-footer">
        <button class="button button--primary" id="profile-export-apply-button" data-ui-action="apply-profile-export" onclick="applyProfileExport()">Apply</button>
        <button class="button" id="profile-export-cancel-button" data-ui-action="cancel-profile-export" onclick="closeProfileExportDialog()">Cancel</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="profile-export-advanced-wizard-modal" hidden>
    <section class="modal modal--panel profile-export-advanced-dialog" role="dialog" aria-modal="true" aria-labelledby="profile-export-advanced-wizard-title" data-ui-entity="netstitch-ui-profile-export-advanced-wizard-dialog">
      <div class="modal__header">
        <div>
          <h2 id="profile-export-advanced-wizard-title">Additional settings</h2>
        </div>
      </div>
      <div class="modal__body">
        <div class="profile-export-advanced-body">
        <p class="control-help-text profile-export-advanced-intro" id="profile-export-advanced-wizard-help">Use this mode only when you want to extend the profile manually for new protocol/port traffic.</p>
        <div class="profile-export-advanced-panel profile-export-advanced-panel--uncovered">
          <div class="profile-export-preview__title profile-export-preview__title--warning" id="profile-export-advanced-wizard-uncovered-title">Not covered by profile rules</div>
          <div id="profile-export-advanced-wizard-uncovered"></div>
        </div>
        <div class="profile-export-advanced-panel profile-export-advanced-panel--settings">
          <div class="profile-export-advanced-option">
          <div class="profile-export-switch-row">
            <span class="profile-export-switch-row__label" id="profile-export-advanced-create-rules-label">Create rules for uncovered ports</span>
            <button class="input-box switch" id="profile-export-advanced-create-rules-switch" type="button" role="switch" aria-checked="false" onclick="toggleProfileExportAdvancedSetting('advancedCreateRules')"><span class="switch__knob"></span></button>
          </div>
          <div class="control-help-text" id="profile-export-advanced-create-rules-help"></div>
          <div class="profile-export-field profile-export-field--template">
            <label class="value-label" for="profile-export-advanced-template-select" id="profile-export-advanced-template-label">Rule template</label>
            <select class="input-box select" id="profile-export-advanced-template-select" onchange="updateProfileExportAdvancedSettings()"></select>
          </div>
          </div>
          <div class="profile-export-advanced-option">
          <div class="profile-export-switch-row">
            <span class="profile-export-switch-row__label" id="profile-export-advanced-exclude-ips-label">Add selected IPs to exclusions</span>
            <button class="input-box switch" id="profile-export-advanced-exclude-ips-switch" type="button" role="switch" aria-checked="false" onclick="toggleProfileExportAdvancedSetting('advancedExcludeIps')"><span class="switch__knob"></span></button>
          </div>
          <div class="control-help-text" id="profile-export-advanced-exclude-ips-help"></div>
          </div>
          <div class="profile-export-advanced-option">
          <div class="profile-export-switch-row">
            <span class="profile-export-switch-row__label" id="profile-export-advanced-whois-ranges-label">Use ranges instead of IPs</span>
            <button class="input-box switch" id="profile-export-advanced-whois-ranges-switch" type="button" role="switch" aria-checked="false" onclick="toggleProfileExportAdvancedSetting('advancedWhoisRanges')"><span class="switch__knob"></span></button>
          </div>
          <div class="control-help-text" id="profile-export-advanced-whois-ranges-help"></div>
          </div>
          <div class="profile-export-advanced-option">
          <div class="profile-export-switch-row">
            <span class="profile-export-switch-row__label" id="profile-export-advanced-domains-label">Add domains to domain list</span>
            <button class="input-box switch" id="profile-export-advanced-domains-switch" type="button" role="switch" aria-checked="false" onclick="toggleProfileExportAdvancedSetting('advancedDomains')"><span class="switch__knob"></span></button>
          </div>
          <div class="control-help-text" id="profile-export-advanced-domains-help"></div>
          </div>
          <div class="profile-export-field profile-export-field--domains profile-export-field--manual-domains">
            <div class="profile-export-domain-field-header">
              <label class="value-label" for="profile-export-advanced-manual-domains-input" id="profile-export-advanced-manual-domains-label">Manual domains</label>
              <button class="input-box button button--secondary" id="profile-export-advanced-add-domains-button" type="button" onclick="addProfileExportSelectedDomains()">Add</button>
            </div>
            <div class="path-input-shell path-input-shell--textarea">
              <textarea class="input-box input profile-export-domains-input" id="profile-export-advanced-manual-domains-input" rows="8" data-commit-on-enter="true" data-clear-button="true" onchange="updateProfileExportAdvancedSettings(true)"></textarea>
              <button class="path-input-clear" id="clear-profile-export-advanced-manual-domains-button" type="button" onclick="clearProfileExportAdvancedManualDomains()" data-clear-target="profile-export-advanced-manual-domains-input" data-clear-button="true" data-ui-action="clear-profile-export-advanced-manual-domains" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
            </div>
          </div>
        </div>
        </div>
      </div>
      <div class="modal__footer">
        <button class="button button--primary" id="profile-export-advanced-wizard-apply-button" data-ui-action="apply-profile-export" onclick="applyProfileExportAdvancedSettings()">Apply</button>
        <button class="button" id="profile-export-advanced-wizard-cancel-button" data-ui-action="cancel-profile-export-advanced-settings" onclick="cancelProfileExportAdvancedSettings()">Cancel</button>
        <div class="modal-footer-separator" aria-hidden="true"></div>
        <button class="button" id="profile-export-advanced-wizard-close-button" data-ui-action="close-profile-export-advanced-wizard" onclick="closeProfileExportAdvancedWizard()">Close</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="ignored-delete-modal" hidden>
    <section class="modal modal--compact" role="dialog" aria-modal="true" aria-labelledby="ignored-delete-modal-title">
      <div class="modal__header">
        <div>
          <h2 id="ignored-delete-modal-title">Delete ignored address?</h2>
        </div>
      </div>
      <div class="modal__body">
        <p class="help-text" id="ignored-delete-modal-help">This rule will be removed from the filter only. Existing observations stay in local storage.</p>
        <div class="note-box" id="ignored-delete-modal-body"></div>
      </div>
      <div class="modal__footer">
        <button class="button" id="ignored-delete-modal-cancel-button" data-ui-action="cancel-delete-ignored-address" onclick="cancelDeleteIgnoredAddress()">No</button>
        <button class="button button--danger" id="ignored-delete-modal-confirm-button" data-ui-action="confirm-delete-ignored-address" onclick="confirmDeleteIgnoredAddress()">Yes, delete</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="observation-delete-modal" hidden>
    <section class="modal modal--compact" role="dialog" aria-modal="true" aria-labelledby="observation-delete-modal-title">
      <div class="modal__header">
        <div>
          <h2 id="observation-delete-modal-title">Delete monitoring row?</h2>
        </div>
      </div>
      <div class="modal__body">
        <p class="help-text" id="observation-delete-modal-help">This removes only the selected monitoring row from local storage. The tracked app and other observations stay intact.</p>
        <div class="note-box" id="observation-delete-modal-body"></div>
      </div>
      <div class="modal__footer">
        <button class="button" id="observation-delete-modal-cancel-button" data-ui-action="cancel-delete-observation" onclick="cancelDeleteObservation()">No</button>
        <button class="button button--danger" id="observation-delete-modal-confirm-button" data-ui-action="confirm-delete-observation" onclick="confirmDeleteObservation()">Yes, delete</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="clear-monitoring-modal" hidden>
    <section class="modal modal--compact" role="dialog" aria-modal="true" aria-labelledby="clear-monitoring-modal-title">
      <div class="modal__header">
        <div>
          <h2 id="clear-monitoring-modal-title">Clear monitoring?</h2>
        </div>
      </div>
      <div class="modal__body">
        <p class="help-text" id="clear-monitoring-modal-help">This removes all visible monitoring rows from local storage. Tracked apps and rows hidden by filters stay intact.</p>
        <div class="note-box" id="clear-monitoring-modal-body"></div>
      </div>
      <div class="modal__footer">
        <button class="button" id="clear-monitoring-modal-cancel-button" data-ui-action="cancel-clear-monitoring" onclick="cancelClearMonitoring()">No</button>
        <button class="button button--danger" id="clear-monitoring-modal-confirm-button" data-ui-action="confirm-clear-monitoring" onclick="confirmClearMonitoring()">Yes, clear</button>
      </div>
    </section>
  </div>
  <div class="modal-backdrop" id="server-file-picker-modal" hidden>
    <section class="modal modal--panel server-file-picker-dialog" role="dialog" aria-modal="true" aria-labelledby="server-file-picker-title">
      <div class="modal__header">
        <div>
          <h2 id="server-file-picker-title">Choose file</h2>
          <p class="help-text" id="server-file-picker-help">Files are shown from the machine where NetStitch is running.</p>
        </div>
      </div>
      <div class="modal__body">
        <div class="server-file-picker-path-row">
          <div class="path-input-shell">
            <input class="input-box input" id="server-file-picker-path" placeholder="Path" data-clear-button="true">
            <button class="path-input-clear" id="server-file-picker-clear-path-button" type="button" onclick="clearServerFilePickerPath()" data-clear-target="server-file-picker-path" data-clear-button="true" data-tooltip="Clear field" data-tooltip-align="end" aria-label="Clear field"><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>
          </div>
          <button class="button" id="server-file-picker-open-path-button" type="button" onclick="openServerFilePickerTypedPath()">Open</button>
        </div>
        <div class="server-file-picker-list" id="server-file-picker-list"></div>
        <div>
          <div class="server-file-picker-roots" id="server-file-picker-roots"></div>
          <div class="server-file-picker-save-row" id="server-file-picker-save-row" hidden>
            <label class="value-label" id="server-file-picker-save-name-label" for="server-file-picker-save-name">File name</label>
            <input class="input-box input" id="server-file-picker-save-name" placeholder="NetStitch-monitoring-selected.csv">
          </div>
          <div class="server-file-picker-feedback" id="server-file-picker-feedback"></div>
        </div>
      </div>
      <div class="modal__footer">
        <button class="button" id="server-file-picker-cancel-button" type="button" onclick="closeServerFilePicker()">Cancel</button>
        <button class="button button--primary" id="server-file-picker-confirm-button" type="button" onclick="confirmServerFilePicker()">Choose</button>
      </div>
    </section>
  </div>
  <div class="disabled-browser-overlay" id="browser-disabled-overlay" hidden>
    <section class="disabled-browser-overlay__panel" role="status" aria-live="polite" aria-labelledby="browser-disabled-overlay-title">
      <h2 class="disabled-browser-overlay__title" id="browser-disabled-overlay-title">Web access is disabled</h2>
      <p class="disabled-browser-overlay__text" id="browser-disabled-overlay-text">Enable Web access in the desktop header to use the browser interface.</p>
      <p class="disabled-browser-overlay__text disabled-browser-overlay__note" id="browser-disabled-overlay-note">This page will start working again after desktop re-enables the web server.</p>
    </section>
  </div>
  <div class="disabled-browser-overlay host-unavailable-overlay" id="host-unavailable-overlay" hidden>
    <section class="disabled-browser-overlay__panel" role="status" aria-live="polite" aria-labelledby="host-unavailable-overlay-title">
      <h2 class="disabled-browser-overlay__title" id="host-unavailable-overlay-title">NetStitch host is unavailable</h2>
      <p class="disabled-browser-overlay__text" id="host-unavailable-overlay-text">The main NetStitch program is not responding, or its embedded web server was switched off.</p>
      <p class="disabled-browser-overlay__text disabled-browser-overlay__note" id="host-unavailable-overlay-note">The browser page will reconnect automatically after the host answers again.</p>
    </section>
  </div>
  <div class="disabled-browser-overlay" id="web-auth-overlay" hidden>
    <section class="disabled-browser-overlay__panel" role="dialog" aria-modal="true" aria-labelledby="web-auth-overlay-title">
      <h2 class="disabled-browser-overlay__title" id="web-auth-overlay-title">Web access key</h2>
      <p class="disabled-browser-overlay__text" id="web-auth-overlay-text">Paste the key from the NetStitch desktop footer, or open the copied web-server link with the key included.</p>
      <div class="web-auth-overlay__form">
        <input class="input-box input" id="web-auth-key-input" type="password" autocomplete="off" onkeydown="submitWebAuthKey(event)" placeholder="Access key">
        <button class="input-box button button--primary" id="web-auth-submit-button" type="button" onclick="submitWebAuthKey()">Open</button>
      </div>
      <p class="web-auth-overlay__status" id="web-auth-status"></p>
    </section>
  </div>
  <script>
    (() => {
      if (window.__netstitchTooltipLayerInstalled) return;
      window.__netstitchTooltipLayerInstalled = true;
      const tooltip = document.createElement('div');
      tooltip.className = 'NetStitch-floating-tooltip';
      tooltip.setAttribute('role', 'tooltip');
      document.body.appendChild(tooltip);
      let activeTarget = null;
      const textFor = (target) => target?.getAttribute('data-tooltip') || '';
      function hideTooltip() {
        activeTarget = null;
        tooltip.classList.remove('NetStitch-floating-tooltip--visible');
      }
      function positionTooltip() {
        if (!activeTarget || !textFor(activeTarget)) return hideTooltip();
        const rect = activeTarget.getBoundingClientRect();
        const gap = 6;
        const padding = 8;
        const tooltipRect = tooltip.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;
        const align = activeTarget.getAttribute('data-tooltip-align') || 'center';
        let left = align === 'start'
          ? rect.left
          : align === 'end'
            ? rect.right - tooltipRect.width
            : rect.left + rect.width / 2 - tooltipRect.width / 2;
        if (tooltipRect.width < viewportWidth - padding * 2) {
          left = Math.max(padding, Math.min(left, viewportWidth - tooltipRect.width - padding));
        } else {
          left = padding;
        }
        let top = rect.top - tooltipRect.height - gap;
        if (top < padding) top = rect.bottom + gap;
        if (top + tooltipRect.height > viewportHeight - padding) {
          top = Math.max(padding, viewportHeight - tooltipRect.height - padding);
        }
        tooltip.style.left = Math.round(left) + 'px';
        tooltip.style.top = Math.round(top) + 'px';
      }
      function showTooltip(target) {
        const text = textFor(target);
        if (!text) return hideTooltip();
        activeTarget = target;
        tooltip.textContent = text;
        tooltip.classList.toggle(
          'NetStitch-floating-tooltip--pre',
          target.classList.contains('footer-message-text') || text.includes('\n')
        );
        tooltip.classList.add('NetStitch-floating-tooltip--visible');
        positionTooltip();
      }
      document.addEventListener('pointerover', (event) => {
        const target = event.target.closest('[data-tooltip]');
        if (target) showTooltip(target);
      });
      document.addEventListener('pointerout', (event) => {
        if (activeTarget && !activeTarget.contains(event.relatedTarget)) hideTooltip();
      });
      document.addEventListener('focusin', (event) => {
        const target = event.target.closest('[data-tooltip]');
        if (target) showTooltip(target);
      });
      document.addEventListener('focusout', (event) => {
        if (activeTarget && activeTarget === event.target) hideTooltip();
      });
      window.addEventListener('scroll', positionTooltip, true);
      window.addEventListener('resize', positionTooltip);
    })();

    const CLOSE_ICON_SRC = '/v1/assets/close-times.svg';
    const MODULE_STOP_ICON_SRC = '/v1/assets/module-stop.svg';
    const MODULE_CLOSE_ICON_SRC = '/v1/assets/module-close.svg';
    const MODULE_REORDER_LEFT_ICON_SRC = '/v1/assets/module-reorder-left.svg';
    const COPY_ICON_SRC = '/v1/assets/copy.svg';
    const CONFIRM_FILTERED_ICON_SRC = '/v1/assets/confirm-filtered.svg';
    const UNCONFIRM_FILTERED_ICON_SRC = '/v1/assets/unconfirm-filtered.svg';
    const IGNORE_ADDRESS_ICON_SRC = '/v1/assets/ignore-address.svg';
    const TRASH_ICON_SRC = '/v1/assets/trash.svg';
    const IMPORT_CLOUD_ICON_SRC = '/v1/assets/import-cloud.svg';
    const EXPORT_CLOUD_ICON_SRC = '/v1/assets/export-cloud.svg';
    const IMPORT_CSV_ICON_SRC = '/v1/assets/import-csv.svg';
    const EXPORT_CSV_ICON_SRC = '/v1/assets/export-csv.svg';
    const MONITORING_ICON_SRC = '/v1/assets/monitoring.svg';
    const MY_PUBLICATIONS_ICON_SRC = '/v1/assets/my-publications.svg';
    const SORT_IDLE_ICON_SRC = '/v1/assets/sort-idle.svg';
    const SORT_ASC_ICON_SRC = '/v1/assets/sort-asc.svg';
    const SORT_DESC_ICON_SRC = '/v1/assets/sort-desc.svg';
    const WATCHER_MODULE_NAME = 'embedded watcher';
    const TOOL_MODULE_NAME = 'netstitch-tool native library';
    const state = {
      snapshot: null,
      languageCatalog: null,
      webUrl: null,
      endpointProbeTargets: null,
      snapshotLoadInFlight: false,
      backgroundSnapshotTimer: null,
      watcherConnected: false,
      browserAccessDisabledOverlay: false,
      webAuthRequired: false,
      webAuthMessage: '',
      hostUnavailableFailures: 0,
      hostUnavailableVisible: false,
      hostAvailabilityTimer: null,
      observationSortKey: 'last_seen',
      observationSortDescending: true,
      lastSelectedObservationId: null,
      observationSelectionInFlightSelect: new Set(),
      observationSelectionInFlightDeselect: new Set(),
      integrationDir: '',
      selectedIntegrationModuleId: '',
      moduleUiPage: 'main',
      moduleUiValues: {},
      moduleUiActionGeneration: 0,
      moduleUiActionPollTokens: {},
      moduleUiTableSort: {},
      moduleOrderEditing: false,
      moduleOrder: [],
      integrationChildReturnToModule: false,
      profileExportReturnToModule: false,
      moduleDialog: null,
      lastServiceStatusEvents: {},
      initialConnectorEventsSeeded: false,
      moduleStatusEventKeys: new Set(),
      lastDomainCaptureError: '',
      pendingIgnoredAddressDelete: null,
      pendingObservationDelete: null,
      pendingClearMonitoringCount: null,
      pendingClearMonitoringScope: 'monitoring',
      filePicker: { open: false, intent: '', mode: 'any', title: '', help: '', currentPath: '', parentPath: '', entries: [], roots: [], selectedPath: '', save: false, defaultFileName: '', saveFileName: '', defaultExtension: '', extensions: [], overwritePolicy: 'prompt', confirmLabel: '', moduleValueTarget: '', moduleStatusTarget: '', selectedStatus: '', feedback: '' },
      integrationDownload: { active: false, visible: false, cancelled: false, generation: 0, stage: 'idle', percent: null, downloadedBytes: null, totalBytes: null, extractedEntries: null, totalEntries: null, message: null, repoRoot: null },
      profileExport: { open: false, repoRootKey: '', mode: 'attach_netstitch_lists', selectedProfilePaths: { attach_netstitch_lists: '', patch_selected_profile: '', merge_into_existing_lists: '' }, generatedProfileName: 'NetStitch', dangerousConfirmed: false, advancedCreateRules: false, advancedExcludeIps: false, advancedWhoisRanges: false, advancedDomains: false, advancedManualDomains: '', advancedTemplateRule: '', advancedReadyForExport: false, advancedDomainCleanupRequested: false, preview: null, feedback: '', previewRequestId: 0 },
      cloudPanel: null,
      cloud: {
        apps: [],
        myApps: [],
        rows: [],
        selectedRows: new Set(),
        lastSelectedRowId: '',
        selectedAppId: '',
        selectedAppName: '',
        loadingAppId: '',
        quota: null,
        serviceOnline: false,
        serviceMessage: '',
        rowFilters: defaultFilters(),
        appFilters: { app: '', publisher: '', source: '' },
        scopeMine: false,
        importVisibilityScope: 'all',
        exportAuthor: '',
        uploadPrivate: false,
        auth: null,
        nicknameStatus: 'idle',
        publicationSortKey: 'new_rows',
        publicationSortDescending: true
      },
      statusHistory: []
    };
    function defaultFilters() {
      return {
        app_search: '',
        ip_search: '',
        domain_search: '',
        port_search: '',
        protocol: 'All',
        public_ip: true,
        observation_filter: 'All'
      };
    }

    function currentFilters() {
      const filters = {
        ...defaultFilters(),
        ...(state.snapshot?.filters || {})
      };
      if (!['All', 'Unconfirmed', 'Confirmed', 'Success', 'Failed'].includes(text(filters.observation_filter))) {
        filters.observation_filter = 'All';
      }
      return filters;
    }

    function text(value, fallback = '') {
      return value === null || value === undefined || value === '' ? fallback : String(value);
    }

    function html(value) {
      return text(value)
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
    }

    async function api(path, options = {}) {
      const requestOptions = { ...options };
      requestOptions.headers = {
        'content-type': 'application/json',
        ...(options.headers || {})
      };
      let response;
      try {
        response = await fetch(path, requestOptions);
        markHostAvailable();
      } catch (error) {
        throw error;
      }
      if (!response.ok) {
        const message = await response.text();
        const error = new Error(message);
        error.status = response.status;
        throw error;
      }
      const contentType = response.headers.get('content-type') || '';
      return contentType.includes('application/json') ? response.json() : response.text();
    }

    async function post(path, body, options = {}) {
      try {
        const result = await api(path, {
          method: 'POST',
          body: body === undefined ? undefined : JSON.stringify(body)
        });
        if (options.skipSnapshotRefresh !== true) {
          await loadSnapshot();
        }
        return result;
      } catch (error) {
        setStatus(text(error?.message || error), true);
        throw error;
      }
    }

    async function updateFilters(patch) {
      const next = {
        ...defaultFilters(),
        ...currentFilters(),
        ...patch
      };
      const previousSnapshot = state.snapshot ? { ...state.snapshot, filters: currentFilters() } : null;
      if (state.snapshot) {
        state.snapshot = { ...state.snapshot, filters: next };
        applySnapshotSyncDelta(
          snapshotSyncDelta(previousSnapshot, state.snapshot),
          state.snapshot
        );
      }
      await post('/v1/filters', next, { skipSnapshotRefresh: true });
      await loadSnapshot({ background: true });
    }

    async function loadLanguages() {
      state.languageCatalog = await api('/v1/languages');
      renderLanguageSelect();
    }

    async function loadWebUrl() {
      try {
        state.webUrl = await api('/v1/web-url');
      } catch (_error) {
        state.webUrl = null;
      }
    }

    function webKeyFromHash() {
      const hash = String(window.location.hash || '');
      if (!hash.startsWith('#')) return '';
      const params = new URLSearchParams(hash.slice(1));
      return text(params.get('web_key')).trim();
    }

    async function authenticateWebAccessKey(key) {
      const value = text(key).trim();
      if (!value) {
        throw new Error(t('web.auth.empty_key', 'Enter the web access key.'));
      }
      await api('/v1/web-auth', {
        method: 'POST',
        body: JSON.stringify({ key: value })
      });
      state.webAuthRequired = false;
      state.webAuthMessage = '';
      renderWebAuthOverlay(false);
    }

    async function authenticateWebAccessFromHash() {
      const key = webKeyFromHash();
      if (!key) return false;
      await authenticateWebAccessKey(key);
      history.replaceState(null, '', window.location.pathname + window.location.search);
      return true;
    }

    async function loadEndpointProbeTargets() {
      try {
        const response = await api('/v1/endpoint-probe-targets');
        state.endpointProbeTargets = Array.isArray(response.targets) ? response.targets : [];
      } catch (_error) {
        state.endpointProbeTargets = [];
      }
    }

    function endpointProbeRequestSpec(rawTarget) {
      const value = text(rawTarget).trim();
      if (!value) {
        return { target: '', protocol: 'tcp' };
      }
      if (value.startsWith('tcp://')) {
        return { target: value.slice(6), protocol: 'tcp' };
      }
      if (value.startsWith('udp://')) {
        return { target: value.slice(6), protocol: 'udp' };
      }
      const port = value.split(':').pop();
      return { target: value, protocol: port === '53' ? 'udp' : 'tcp' };
    }

    function stableJson(value) {
      try {
        return JSON.stringify(value ?? null);
      } catch (_error) {
        return '';
      }
    }

    function snapshotSectionSignatures(snapshot) {
      return {
        language: text(snapshot?.app_settings?.ui_language_code),
        filters: stableJson(snapshot?.filters),
        trackedApps: stableJson(snapshot?.tracked_apps),
        enableAllOverlay: stableJson(snapshot?.app_settings?.ui_enable_all_overlay),
        monitorStatus: stableJson(snapshot?.monitor_status),
        ignoredAddresses: stableJson(snapshot?.ignored_addresses),
        observations: stableJson(snapshot?.observed_endpoints),
        integrationIntegration: stableJson(snapshot?.integration_status),
        integrationModules: stableJson(snapshot?.integration_modules),
        integrationProviders: stableJson(snapshot?.integration_providers),
        webAccessLocalhost: stableJson(snapshot?.app_settings?.web_access_localhost),
        domainCaptureEnabled: stableJson(snapshot?.app_settings?.domain_capture_enabled),
        domainCaptureError: text(snapshot?.runtime_status?.domain_capture?.backend_error),
        isElevated: stableJson(snapshot?.runtime_status?.is_elevated),
        toolAvailable: stableJson(snapshot?.runtime_status?.tool_available),
        endpointProbe: stableJson(snapshot?.runtime_status?.endpoint_probe)
      };
    }

    function snapshotSyncDelta(previousSnapshot, snapshot) {
      const previous = snapshotSectionSignatures(previousSnapshot);
      const next = snapshotSectionSignatures(snapshot);
      return {
        languageChanged: previous.language !== next.language,
        headerChanged:
          previous.filters !== next.filters
          || previous.trackedApps !== next.trackedApps
          || previous.enableAllOverlay !== next.enableAllOverlay
          || previous.webAccessLocalhost !== next.webAccessLocalhost
          || previous.domainCaptureEnabled !== next.domainCaptureEnabled
          || previous.isElevated !== next.isElevated,
        trackedAppsChanged:
          previous.trackedApps !== next.trackedApps
          || previous.enableAllOverlay !== next.enableAllOverlay,
        monitorChanged:
          previous.monitorStatus !== next.monitorStatus
          || previous.ignoredAddresses !== next.ignoredAddresses,
        observationsChanged:
          previous.filters !== next.filters
          || previous.observations !== next.observations
          || previous.trackedApps !== next.trackedApps,
        integrationChanged:
          previous.integrationIntegration !== next.integrationIntegration
          || previous.integrationModules !== next.integrationModules
          || previous.integrationProviders !== next.integrationProviders,
        footerToolChanged: previous.toolAvailable !== next.toolAvailable,
        footerWebChanged: previous.webAccessLocalhost !== next.webAccessLocalhost,
        footerNetworkChanged: previous.endpointProbe !== next.endpointProbe,
        domainCaptureErrorChanged:
          previous.domainCaptureEnabled !== next.domainCaptureEnabled
          || previous.domainCaptureError !== next.domainCaptureError
          || previous.isElevated !== next.isElevated
      };
    }

    function maybeReportDomainCaptureError(snapshot) {
      const enabled = Boolean(snapshot?.app_settings?.domain_capture_enabled);
      const error = text(snapshot?.runtime_status?.domain_capture?.backend_error).trim();
      if (!enabled || !error) {
        state.lastDomainCaptureError = '';
        return;
      }
      if (state.lastDomainCaptureError === error) return;
      state.lastDomainCaptureError = error;
      const adminHint = Boolean(snapshot?.runtime_status?.is_elevated)
        ? ''
        : '; ' + t('footer.event.domain_capture_admin_required', 'Run NetStitch as administrator to use advanced monitoring');
      pushStatusLine(t('footer.event.domain_capture_failed', 'Advanced monitoring is unavailable') + ': ' + error + adminHint, true);
    }

    function applySnapshotSyncDelta(delta, snapshot) {
      renderFooterWatcher(state.watcherConnected);

      if (delta.headerChanged) {
        renderHeader(snapshot);
      }

      if (delta.trackedAppsChanged) {
        renderTrackedApps(snapshot);
      }

      if (delta.monitorChanged) {
        renderMonitor(snapshot);
      }

      if (delta.observationsChanged) {
        renderObservations(snapshot);
        renderIntegrationModulePanel(snapshot);
        if (state.cloudPanel === 'export') {
          renderCloudExportRows();
        }
      }

      if (delta.integrationChanged) {
        renderIntegration(snapshot);
        renderIntegrationModulePanel(snapshot);
      }

      if (delta.footerToolChanged) {
        renderFooterTool(snapshot);
      }

      if (delta.footerWebChanged) {
        renderFooterWebServer(snapshot);
      }

      if (delta.footerNetworkChanged) {
        renderFooterNetwork(snapshot);
      }

      if (delta.domainCaptureErrorChanged) {
        maybeReportDomainCaptureError(snapshot);
      }
    }

    function renderBackgroundSnapshotSync(previousSnapshot, snapshot) {
      const delta = snapshotSyncDelta(previousSnapshot, snapshot);

      if (delta.languageChanged) {
        render();
        return;
      }

      applySnapshotSyncDelta(delta, snapshot);
    }

    async function loadSnapshot(options = {}) {
      if (state.snapshotLoadInFlight) {
        return state.snapshot;
      }
      state.snapshotLoadInFlight = true;
      try {
        if (!state.languageCatalog) {
          await loadLanguages();
        }
        if (!state.webUrl) {
          await loadWebUrl();
        }
        await loadEndpointProbeTargets();
        const integrationInput = document.getElementById('integration-dir');
        if (integrationInput) {
          state.integrationDir = integrationInput.value.trim();
        }
        const params = new URLSearchParams();
        if (state.integrationDir) params.set('integration_dir', state.integrationDir);
        state.endpointProbeTargets.forEach((rawTarget) => {
          const spec = endpointProbeRequestSpec(rawTarget);
          if (!spec.target) return;
          params.append('endpoint_probe_target', spec.target);
          params.append('endpoint_probe_protocol', spec.protocol);
        });
        const query = '?' + params.toString();
        const previousSnapshot = state.snapshot;
        state.snapshot = await api('/v1/snapshot' + query);
        await loadFooterMessages(10);
        syncModuleOrderFromSnapshot(state.snapshot);
        reconcileObservationSelectionOverlay(state.snapshot);
        maybeReportDomainCaptureError(state.snapshot);
        reportRuntimeServiceEvents(state.snapshot);
        resetProfileExportControlsWhenIntegrationChanged();
        if (!state.watcherConnected) {
          pushStatusLine(t('footer.event.watcher_connected', 'Watcher connection established'));
        }
        state.watcherConnected = true;
        if (options.background === true) {
          renderBackgroundSnapshotSync(previousSnapshot, state.snapshot);
        } else {
          render();
        }
      } catch (error) {
        if (Number(error?.status) === 401) {
          state.webAuthRequired = true;
          state.webAuthMessage = '';
          state.watcherConnected = false;
          renderShellDisabled(true);
          renderWebAuthOverlay(true);
          return state.snapshot;
        }
        if (state.watcherConnected) {
          pushStatusLine(t('footer.event.watcher_disconnected', 'Watcher connection lost'));
        }
        state.watcherConnected = false;
        if (options.background === true) {
          renderFooterWatcher(false);
          renderFooterWebServer(state.snapshot);
          renderFooterNetwork(state.snapshot);
          renderFooterMessage();
        } else {
          renderShellDisabled(!state.snapshot);
          renderFooterWatcher(false);
          renderFooterWebServer(state.snapshot);
        }
        setStatus(String(error), true);
      } finally {
        state.snapshotLoadInFlight = false;
      }
    }

    async function refreshSnapshotInBackground() {
      if (state.browserAccessDisabledOverlay || state.webAuthRequired || state.hostUnavailableVisible || document.hidden) {
        scheduleBackgroundSnapshotRefresh(state.watcherConnected ? 1200 : 4000);
        return;
      }
      try {
        await loadSnapshot({ background: true });
        scheduleBackgroundSnapshotRefresh(1200);
      } catch (_error) {
        // loadSnapshot already updates footer status on failure.
        scheduleBackgroundSnapshotRefresh(4000);
      }
    }

    function scheduleBackgroundSnapshotRefresh(delayMs) {
      if (state.backgroundSnapshotTimer !== null) {
        window.clearTimeout(state.backgroundSnapshotTimer);
      }
      state.backgroundSnapshotTimer = window.setTimeout(refreshSnapshotInBackground, delayMs);
    }

    function render() {
      const snapshot = state.snapshot;
      renderShellDisabled(!snapshot || state.browserAccessDisabledOverlay || state.webAuthRequired || state.hostUnavailableVisible);
      renderBrowserDisabledOverlay(state.browserAccessDisabledOverlay);
      renderWebAuthOverlay(state.webAuthRequired);
      renderHostUnavailableOverlay(state.hostUnavailableVisible);
      if (!snapshot) return;
      renderHeader(snapshot);
      renderModuleHeader();
      renderTrackedApps(snapshot);
      renderMonitor(snapshot);
      renderObservations(snapshot);
      renderIntegration(snapshot);
      renderCloudPanel();
      renderLanguageSelect();
      renderStaticText();
      seedConnectorEvents(snapshot);
      seedModuleEvents(snapshot);
      renderFooterMessage();
      syncClearButtonStates();
      scheduleVisualDebugRects();
    }

    let visualDebugRectFrame = null;

    function stampVisualDebugRect(id) {
      const node = document.getElementById(id);
      if (!node) return;
      const rect = node.getBoundingClientRect();
      const left = Math.round(rect.left);
      const top = Math.round(rect.top);
      const width = Math.round(rect.width);
      const height = Math.round(rect.height);
      node.setAttribute('data-debug-rect', [left, top, width, height].join(','));
    }

    function stampVisualDebugRects() {
      for (const id of ['app-shell', 'browser-header', 'browser-workspace', 'browser-observations-panel', 'browser-footer']) {
        stampVisualDebugRect(id);
      }
    }

    function scheduleVisualDebugRects() {
      if (visualDebugRectFrame !== null) {
        cancelAnimationFrame(visualDebugRectFrame);
      }
      visualDebugRectFrame = requestAnimationFrame(() => {
        visualDebugRectFrame = null;
        stampVisualDebugRects();
      });
    }

    function renderShellDisabled(disabled) {
      const shell = document.getElementById('app-shell');
      if (!shell) return;
      shell.classList.toggle('shell--controls-disabled', Boolean(disabled));
      shell.dataset.uiDisabled = String(Boolean(disabled));
      shell.setAttribute('aria-busy', String(Boolean(disabled)));
      scheduleVisualDebugRects();
    }

    function renderBrowserDisabledOverlay(visible) {
      const overlay = document.getElementById('browser-disabled-overlay');
      if (!overlay) return;
      overlay.hidden = !visible;
    }

    function renderWebAuthOverlay(visible, message) {
      const overlay = document.getElementById('web-auth-overlay');
      if (!overlay) return;
      overlay.hidden = !visible;
      const title = document.getElementById('web-auth-overlay-title');
      const textNode = document.getElementById('web-auth-overlay-text');
      const input = document.getElementById('web-auth-key-input');
      const button = document.getElementById('web-auth-submit-button');
      const status = document.getElementById('web-auth-status');
      if (title) title.textContent = t('web.auth.title', 'Web access key');
      if (textNode) textNode.textContent = t('web.auth.help', 'Paste the key from the NetStitch desktop footer, or open the copied web-server link with the key included.');
      if (input) input.placeholder = t('web.auth.placeholder', 'Access key');
      if (button) button.textContent = t('web.auth.submit', 'Open');
      if (status) status.textContent = text(message, state.webAuthMessage);
      if (visible && input) {
        window.setTimeout(() => input.focus(), 0);
      }
    }

    function renderHostUnavailableOverlay(visible) {
      const overlay = document.getElementById('host-unavailable-overlay');
      if (!overlay) return;
      overlay.hidden = !visible;
      const title = document.getElementById('host-unavailable-overlay-title');
      const textNode = document.getElementById('host-unavailable-overlay-text');
      const note = document.getElementById('host-unavailable-overlay-note');
      if (title) title.textContent = t('web.host_unavailable.title', 'NetStitch host is unavailable');
      if (textNode) textNode.textContent = t('web.host_unavailable.text', 'The main NetStitch program is not responding, or its embedded web server was switched off.');
      if (note) note.textContent = t('web.host_unavailable.note', 'The browser page will reconnect automatically after the host answers again.');
    }

    function markHostAvailable() {
      state.hostUnavailableFailures = 0;
      if (!state.hostUnavailableVisible) return;
      state.hostUnavailableVisible = false;
      renderHostUnavailableOverlay(false);
      renderShellDisabled(!state.snapshot || state.browserAccessDisabledOverlay || state.webAuthRequired);
    }

    function recordHostAvailabilityFailure() {
      state.hostUnavailableFailures += 1;
      if (state.hostUnavailableFailures < 3 || state.hostUnavailableVisible) return;
      state.hostUnavailableVisible = true;
      renderHostUnavailableOverlay(true);
      renderShellDisabled(true);
    }

    function scheduleHostAvailabilityCheck() {
      if (state.hostAvailabilityTimer !== null) {
        window.clearTimeout(state.hostAvailabilityTimer);
      }
      state.hostAvailabilityTimer = window.setTimeout(checkHostAvailability, 1000);
    }

    async function checkHostAvailability() {
      try {
        await fetch('/health', { cache: 'no-store' });
        markHostAvailable();
      } catch (_error) {
        recordHostAvailabilityFailure();
      } finally {
        scheduleHostAvailabilityCheck();
      }
    }

    async function submitWebAuthKey(event) {
      if (event && event.key && event.key !== 'Enter') return;
      if (event) event.preventDefault();
      const input = document.getElementById('web-auth-key-input');
      const key = text(input?.value).trim();
      try {
        await authenticateWebAccessKey(key);
        if (input) input.value = '';
        await loadSnapshot();
        scheduleBackgroundSnapshotRefresh(1200);
      } catch (error) {
        state.webAuthRequired = true;
        state.webAuthMessage = Number(error?.status) === 401
          ? t('web.auth.invalid_key', 'The web access key is invalid.')
          : text(error?.message || error);
        renderShellDisabled(true);
        renderWebAuthOverlay(true, state.webAuthMessage);
      }
    }

    function currentLanguageCode() {
      return state.snapshot?.app_settings?.ui_language_code
        || state.languageCatalog?.fallback_code
        || 'en-en';
    }

    function languageBundle(code) {
      return (state.languageCatalog?.languages || [])
        .find((language) => String(language.code).toLowerCase() === String(code).toLowerCase());
    }

    function t(key, fallback) {
      const selected = languageBundle(currentLanguageCode());
      const fallbackBundle = languageBundle(state.languageCatalog?.fallback_code || 'en-en');
      return selected?.strings?.[key]
        || fallbackBundle?.strings?.[key]
        || fallback
        || key;
    }

    function template(key, fallback, values = {}) {
      return Object.entries(values).reduce(
        (message, [name, value]) => message.replaceAll('{' + name + '}', text(value)),
        t(key, fallback)
      );
    }

    function cloudDownloadMessage(rows) {
      return template('dialog.cloud_sync.download_done', 'Cloud data loaded: {count} rows', { count: rows });
    }

    function cloudUploadMessage(result) {
      let message = template('dialog.cloud_sync.upload_done', 'Cloud upload finished: {rows} rows, requests: {requests}', {
        rows: Number(result?.accepted_rows || 0),
        requests: Number(result?.request_count || 0)
      });
      const skipped = Number(result?.skipped_non_public_rows || 0);
      if (skipped > 0) {
        message += ', ' + t('dialog.cloud_sync.skipped_non_public', 'skipped non-public') + ': ' + skipped;
      }
      const visibility = text(result?.visibility).trim();
      return visibility ? message + ', visibility: ' + visibility : message;
    }

    function cloudImportMessage(result) {
      return template('dialog.cloud_sync.import_done_detail', 'Added to monitoring: {imported}/{requested} rows, skipped: {skipped}', {
        imported: Number(result?.imported_count || result?.updated || result?.rows || 0),
        requested: Number(result?.requested_count || result?.imported_count || result?.rows || 0),
        skipped: Number(result?.skipped_count || 0)
      });
    }

    function cloudReadableMessage(value) {
      let payload = value;
      if (typeof payload === 'string') {
        const trimmed = payload.trim();
        if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) return payload;
        try {
          payload = JSON.parse(trimmed);
        } catch (_) {
          return payload;
        }
      }
      if (!payload || typeof payload !== 'object') return text(payload);
      if (payload.error) {
        const code = text(payload.error.code).trim();
        const message = text(payload.error.message || code).trim();
        return code && message && code !== message ? message + ' (' + code + ')' : (message || code);
      }
      if (payload.download?.rows !== undefined) return cloudDownloadMessage(payload.download.rows);
      if (payload.cloud_upload) return cloudUploadMessage(payload.cloud_upload);
      if (payload.accepted_rows !== undefined) return cloudUploadMessage(payload);
      if (payload.cloud_import) return cloudImportMessage(payload.cloud_import);
      if (payload.imported_count !== undefined || payload.updated !== undefined) return cloudImportMessage(payload);
      if (payload.health || payload.quota || payload.apps || payload.my_apps) {
        return t('dialog.cloud_sync.refresh_done', 'Cloud refreshed');
      }
      return JSON.stringify(payload);
    }

    function contentWidthCh(value, min = 12, max = 120) {
      return Math.max(min, Math.min(text(value).length + 1, max));
    }

    function pathFieldContentWidthStyle(value, min, max) {
      return '--path-field-content-width: ' + contentWidthCh(value, min, max) + 'ch;';
    }

    function renderText(id, key, fallback) {
      const target = document.getElementById(id);
      if (target) target.textContent = t(key, fallback);
    }

    function renderSortableHeaderText(id, key, fallback) {
      const target = document.getElementById(id);
      const label = target?.querySelector('.table-sortable__label');
      if (label) {
        label.textContent = t(key, fallback);
      } else if (target) {
        target.textContent = t(key, fallback);
      }
    }

    function renderPlaceholder(id, key, fallback) {
      const target = document.getElementById(id);
      if (target) target.placeholder = t(key, fallback);
    }

    function renderLabelWithColon(id, key, fallback) {
      const target = document.getElementById(id);
      if (target) target.textContent = t(key, fallback) + ':';
    }

    function renderHelp(id, key, fallback) {
      const target = document.getElementById(id);
      if (!target) return;
      const tooltip = t(key, fallback);
      target.dataset.tooltip = tooltip;
      target.setAttribute('aria-label', tooltip);
    }

    function syncClearButtonStates() {
      for (const button of document.querySelectorAll('[data-clear-target][data-clear-button="true"]')) {
        const targetId = button.getAttribute('data-clear-target') || '';
        const target = targetId ? document.getElementById(targetId) : null;
        if (!target || target.dataset.clearButton !== 'true') continue;
        const value = target && 'value' in target ? String(target.value || '') : '';
        button.disabled = value.length === 0;
      }
    }

    function localizedClearMonitoringLabel() {
      return currentLanguageCode().startsWith('ru') ? 'Очистить мониторинг' : 'Clear monitoring';
    }

    function localizedConfirmationFilterLabel() {
      return t('table.state', 'State');
    }

    function renderStaticText() {
      renderText('tracked-apps-title', 'tracked_apps.title', 'Tracked apps');
      renderHelp('tracked-apps-help', 'tracked_apps.help', 'Add executable paths or toggle apps in the shell state.');
      renderText('enable-all-label', 'tracked_apps.enable_all', 'Enable all');
      renderText('app-filter-label', 'filter.app', 'App');
      renderText('ip-filter-label', 'table.ip', 'IP');
      renderText('domain-filter-label', 'table.domain', 'Domain');
      renderText('port-filter-label', 'filter.port', 'Port');
      renderText('protocol-filter-label', 'filter.protocol', 'Protocol');
      const observationFilterLabel = document.getElementById('observation-filter-label');
      if (observationFilterLabel) {
        observationFilterLabel.textContent = localizedConfirmationFilterLabel();
      }
      renderText('ignored-addresses-panel-title', 'ignored_addresses.title', 'Ignored addresses');
      renderHelp('ignored-addresses-panel-help', 'ignored_addresses.help', 'Addresses in this list are hidden from monitoring views and filtered actions.');
      const toggleMonitoringButton = document.getElementById('toggle-monitoring-button');
      if (toggleMonitoringButton) {
        const label = ['Running', 'Starting'].includes(state.snapshot?.monitor_status)
          ? t('action.stop_monitoring', 'Stop monitoring')
          : t('action.start_monitoring', 'Start monitoring');
        toggleMonitoringButton.setAttribute('aria-label', label);
        toggleMonitoringButton.setAttribute('data-tooltip', label);
        toggleMonitoringButton.setAttribute('data-tooltip-align', 'end');
        toggleMonitoringButton.innerHTML = '<span class="button__icon-shell button__icon-shell--monitoring"><span class="button__icon-scale button__icon-scale--monitoring"><span class="button__icon-rotor button__icon-rotor--monitoring"><img class="button__icon button__icon--monitoring" src="' + MONITORING_ICON_SRC + '" alt=""></span></span></span>';
      }
      const confirmFilteredButton = document.getElementById('confirm-filtered-button');
      if (confirmFilteredButton) {
        const label = t('action.confirm_filtered', 'Confirm all visible');
        confirmFilteredButton.setAttribute('aria-label', label);
        confirmFilteredButton.setAttribute('data-tooltip', label);
        confirmFilteredButton.setAttribute('data-tooltip-align', 'end');
        confirmFilteredButton.innerHTML = '<img class="button__icon button__icon--confirm-filtered" src="' + CONFIRM_FILTERED_ICON_SRC + '" alt="">';
      }
      const unconfirmFilteredButton = document.getElementById('unconfirm-filtered-button');
      if (unconfirmFilteredButton) {
        const label = t('action.unconfirm_filtered', 'Unconfirm all visible');
        unconfirmFilteredButton.setAttribute('aria-label', label);
        unconfirmFilteredButton.setAttribute('data-tooltip', label);
        unconfirmFilteredButton.setAttribute('data-tooltip-align', 'end');
        unconfirmFilteredButton.innerHTML = '<img class="button__icon button__icon--unconfirm-filtered" src="' + UNCONFIRM_FILTERED_ICON_SRC + '" alt="">';
      }
      const clearMonitoringButton = document.getElementById('clear-monitoring-button');
      if (clearMonitoringButton) {
        const label = localizedClearMonitoringLabel();
        clearMonitoringButton.setAttribute('aria-label', label);
        clearMonitoringButton.setAttribute('data-tooltip', label);
        clearMonitoringButton.setAttribute('data-tooltip-align', 'end');
        clearMonitoringButton.innerHTML = '<img class="button__icon button__icon--trash" src="' + TRASH_ICON_SRC + '" alt="">';
      }
      const cloudImportButton = document.getElementById('cloud-import-button');
      if (cloudImportButton) {
        const label = t('action.cloud_import', 'Cloud import');
        cloudImportButton.setAttribute('aria-label', label);
        cloudImportButton.setAttribute('data-tooltip', label);
        cloudImportButton.setAttribute('data-tooltip-align', 'end');
        cloudImportButton.innerHTML = '<img class="button__icon button__icon--cloud-import" src="' + IMPORT_CLOUD_ICON_SRC + '" alt="">';
      }
      const cloudExportButton = document.getElementById('cloud-export-button');
      if (cloudExportButton) {
        const label = t('action.cloud_export', 'Cloud export');
        cloudExportButton.setAttribute('aria-label', label);
        cloudExportButton.setAttribute('data-tooltip', label);
        cloudExportButton.setAttribute('data-tooltip-align', 'end');
        cloudExportButton.innerHTML = '<img class="button__icon button__icon--cloud-sync" src="' + EXPORT_CLOUD_ICON_SRC + '" alt="">';
      }
      renderCloudPanel();
      const importCsvButton = document.getElementById('import-csv-button');
      if (importCsvButton) {
        const label = t('action.import_csv', 'Monitoring: Import CSV');
        importCsvButton.setAttribute('aria-label', label);
        importCsvButton.setAttribute('data-tooltip', label);
        importCsvButton.setAttribute('data-tooltip-align', 'end');
        importCsvButton.innerHTML = '<img class="button__icon button__icon--import-csv" src="' + IMPORT_CSV_ICON_SRC + '" alt="">';
      }
      const exportCsvButton = document.getElementById('export-csv-button');
      if (exportCsvButton) {
        const label = t('action.export_csv', 'Monitoring: Export selected to CSV');
        exportCsvButton.setAttribute('aria-label', label);
        exportCsvButton.setAttribute('data-tooltip', label);
        exportCsvButton.setAttribute('data-tooltip-align', 'end');
        exportCsvButton.innerHTML = '<img class="button__icon button__icon--export-csv" src="' + EXPORT_CSV_ICON_SRC + '" alt="">';
      }
      renderText('observations-title', 'observations.title', 'Monitoring');
      renderHelp('observations-help', 'observations.subtitle', 'Snapshot-shaped rows keep the client close to the real API.');
      renderText('integration-title', 'integration.title', 'Modules');
      renderHelp('integration-help', 'integration.subtitle', 'External modules available to the local workspace.');
      renderText('modules-order-label', 'modules.order', 'Display order');
      renderText('integration-module-modal-close-button', 'dialog.back', 'Back');
      renderModuleHeader();
      renderText('integration-modal-title', 'dialog.integration.title', 'Module');
      renderHelp('integration-modal-help', 'dialog.integration.help', 'Prepare the module for later data export.');
      renderText('integration-folder-path-help', 'dialog.integration.path_help', 'Module path for later data export.');
      renderText('browse-integration-folder-button', 'dialog.integration.browse', 'Browse');
      renderText('integration-modal-cancel-button', 'dialog.cancel', 'Cancel');
      renderText('integration-modal-use-button', 'dialog.integration.use_folder', 'Assign folder');
      for (const button of document.querySelectorAll('[data-clear-target][data-clear-button="true"]')) {
        const tooltip = t('input.clear', 'Clear field');
        button.dataset.tooltip = tooltip;
        button.setAttribute('aria-label', tooltip);
      }
      renderText('ignored-delete-modal-title', 'dialog.delete_ignored.title', 'Delete ignored address?');
      renderText('ignored-delete-modal-help', 'dialog.delete_ignored.help', 'This rule will be removed from the filter only. Existing observations stay in local storage.');
      renderText('ignored-delete-modal-cancel-button', 'dialog.no', 'No');
      renderText('ignored-delete-modal-confirm-button', 'dialog.yes_delete', 'Yes, delete');
      renderText('observation-delete-modal-title', 'dialog.delete_observation.title', 'Delete monitoring row?');
      renderText('observation-delete-modal-help', 'dialog.delete_observation.help', 'This removes only the selected monitoring row from local storage. The tracked app and other observations stay intact.');
      renderText('observation-delete-modal-cancel-button', 'dialog.no', 'No');
      renderText('observation-delete-modal-confirm-button', 'dialog.yes_delete', 'Yes, delete');
      renderText('clear-monitoring-modal-title', 'dialog.clear_monitoring.title', 'Clear monitoring?');
      renderText('clear-monitoring-modal-help', 'dialog.clear_monitoring.help', 'This removes all visible monitoring rows from local storage. Tracked apps and rows hidden by filters stay intact.');
      renderText('clear-monitoring-modal-cancel-button', 'dialog.no', 'No');
      renderText('clear-monitoring-modal-confirm-button', 'dialog.clear_monitoring.confirm', 'Yes, clear');
      renderText('server-file-picker-open-path-button', 'dialog.file_picker.open', 'Open');
      renderText('server-file-picker-cancel-button', 'dialog.cancel', 'Cancel');
      renderText('server-file-picker-confirm-button', 'dialog.file_picker.choose', 'Choose');
      renderText('server-file-picker-save-name-label', 'dialog.file_picker.file_name', 'File name');
      const pickerPath = document.getElementById('server-file-picker-path');
      if (pickerPath) pickerPath.placeholder = t('tracked_apps.path_placeholder', 'Path');
      const pickerSaveName = document.getElementById('server-file-picker-save-name');
      if (pickerSaveName) pickerSaveName.placeholder = 'NetStitch-monitoring-selected.csv';
      renderText('export-profile-modal-title', 'dialog.profile_export.title', 'Profile export');
      renderHelp('export-profile-modal-help', 'dialog.profile_export.help', 'Preview and apply profile-aware export changes.');
      renderProfileExportModeCopy();
      for (const id of ['profile-export-attach-profile-label', 'profile-export-patch-profile-label', 'profile-export-merge-profile-label']) {
        renderText(id, 'dialog.profile_export.selected_profile_path', 'Selected profile path');
      }
      for (const id of ['profile-export-attach-profile-select-label', 'profile-export-patch-profile-select-label']) {
        renderText(id, 'dialog.profile_export.available_profiles', 'Available profiles');
      }
      renderText('profile-export-attach-generated-name-label', 'dialog.profile_export.generated_profile_name', 'Generated profile name');
      for (const id of ['browse-profile-export-attach-profile-button', 'browse-profile-export-patch-profile-button', 'browse-profile-export-merge-profile-button']) {
        renderText(id, 'dialog.integration.browse', 'Browse');
      }
      renderText('profile-export-dangerous-text', 'dialog.profile_export.dangerous_confirm', 'Allow dangerous merge');
      renderText('profile-export-cancel-button', 'dialog.cancel', 'Cancel');
      renderText('profile-export-analyze-button', 'dialog.profile_export.analyze', 'Analyze');
      renderText('profile-export-backup-button', 'dialog.profile_export.backup', 'Create backup');
      renderText('profile-export-apply-button', 'dialog.profile_export.apply', 'Export');
      renderText('profile-export-advanced-wizard-button', 'dialog.profile_export.advanced_wizard', 'Additional settings');
      renderText('profile-export-revert-button', 'dialog.profile_export.revert', 'Revert changes');
      renderText('profile-export-advanced-wizard-title', 'dialog.profile_export.advanced_wizard_title', 'Additional settings');
      renderText('profile-export-advanced-wizard-help', 'dialog.profile_export.advanced_wizard_help', 'Use this mode when you need to extend the profile manually for new protocol/port traffic.');
      renderText('profile-export-advanced-wizard-uncovered-title', 'dialog.profile_export.uncovered_title', 'Not covered by profile rules');
      renderText('profile-export-advanced-template-label', 'dialog.profile_export.advanced_template_rule', 'Rule template');
      renderText('profile-export-advanced-create-rules-label', 'dialog.profile_export.advanced_create_rules', 'Create rules for uncovered ports');
      renderText('profile-export-advanced-create-rules-help', 'dialog.profile_export.advanced_create_rules_help', 'Adds new rules to the selected NetStitch profile using the template rule.');
      renderText('profile-export-advanced-exclude-ips-label', 'dialog.profile_export.advanced_exclude_ips', 'Add selected IPs to exclusions');
      renderText('profile-export-advanced-exclude-ips-help', 'dialog.profile_export.advanced_exclude_ips_help', 'Explicitly excludes selected IPs from bypass through a NetStitch-owned exclude list.');
      renderText('profile-export-advanced-whois-ranges-label', 'dialog.profile_export.advanced_whois_ranges', 'Use ranges instead of IPs');
      renderText('profile-export-advanced-whois-ranges-help', 'dialog.profile_export.advanced_whois_ranges_help', 'Uses ranges for export when they are available.');
      renderText('profile-export-advanced-domains-label', 'dialog.profile_export.advanced_domains', 'Add domains to domain list');
      renderText('profile-export-advanced-domains-help', 'dialog.profile_export.advanced_domains_help', 'Adds detected and manually entered domains to a module-owned domain list.');
      renderText('profile-export-advanced-manual-domains-label', 'dialog.profile_export.advanced_manual_domains', 'Domains');
      renderText('profile-export-advanced-add-domains-button', 'dialog.profile_export.advanced_add_domains', 'Add');
      renderText('profile-export-advanced-wizard-apply-button', 'dialog.profile_export.advanced_apply', 'Apply');
      renderText('profile-export-advanced-wizard-cancel-button', 'dialog.profile_export.advanced_cancel', 'Reset');
      renderText('profile-export-advanced-wizard-close-button', 'dialog.profile_export.close', 'Close');
      const manualDomains = document.getElementById('profile-export-advanced-manual-domains-input');
      if (manualDomains) manualDomains.placeholder = t('dialog.profile_export.advanced_manual_domains_placeholder', 'example.com, cdn.example.net');
      renderLabelWithColon('language-label', 'footer.language.label', 'Language');
      renderLabelWithColon('footer-message-label', 'footer.message.label', 'Message');
      const footerMessage = document.getElementById('footer-message-text');
      if (footerMessage) {
        footerMessage.setAttribute('aria-label', t('footer.message.label', 'Message'));
      }
      const copyStatus = document.getElementById('footer-status-copy');
      if (copyStatus) {
        const tooltip = t('footer.status.copy_tooltip', 'Copy the last 100 status lines');
        copyStatus.dataset.tooltip = tooltip;
        copyStatus.setAttribute('aria-label', tooltip);
      }
      renderFooterWatcher(state.watcherConnected);
      renderFooterWebServer(state.snapshot);
      renderFooterNetwork(state.snapshot);
      renderText('table-app-label', 'table.app', 'App');
      renderText('table-ip-label', 'table.ip', 'IP');
      renderText('table-domain-label', 'table.domain', 'Domain');
      renderText('table-port-label', 'table.port', 'Port');
      renderText('table-proto-label', 'table.proto', 'Proto');
      renderText('table-conn-label', 'table.conn', 'Conn');
      renderText('table-hits-label', 'table.hits', 'Hits');
      renderText('table-first-seen-label', 'table.first_seen', 'First seen');
      renderText('table-last-seen-label', 'table.last_seen', 'Last seen');
      renderText('table-action-label', 'table.action', 'Action');
      setHeaderTooltip('table-app', t('table.app', 'App'));
      setHeaderTooltip('table-ip', t('table.ip', 'IP'));
      setHeaderTooltip('table-domain', t('table.domain_tooltip', 'Domain is shown only when verified from HTTP Host, TCP TLS SNI, or QUIC Initial SNI'));
      setHeaderTooltip('table-port', t('table.port', 'Port'));
      setHeaderTooltip('table-proto', t('filter.protocol', 'Protocol'));
      setHeaderTooltip('table-conn', t('table.conn_tooltip', 'Current state and counters in Established (success/fail) format'));
      setHeaderTooltip('table-hits', t('table.hits_tooltip', 'Request count'));
      setHeaderTooltip('table-first-seen', t('table.first_seen', 'First seen'));
      setHeaderTooltip('table-last-seen', t('table.last_seen', 'Last seen'));
      setHeaderTooltip('table-action', t('table.action', 'Action'));
      setHeaderTooltip('table-app', t('table.app', 'App'));
      setHeaderTooltip('table-ip', t('table.ip', 'IP'));
      setHeaderTooltip('table-domain', t('table.domain_tooltip', 'Domain is shown only when verified from HTTP Host, TCP TLS SNI, or QUIC Initial SNI'));
      setHeaderTooltip('table-port', t('table.port', 'Port'));
      setHeaderTooltip('table-proto', t('filter.protocol', 'Protocol'));
      setHeaderTooltip('table-conn', t('table.conn_tooltip', 'Current state and counters in Established (success/fail) format'));
      setHeaderTooltip('table-hits', t('table.hits_tooltip', 'Request count'));
      setHeaderTooltip('table-first-seen', t('table.first_seen', 'First seen'));
      setHeaderTooltip('table-last-seen', t('table.last_seen', 'Last seen'));
      setHeaderTooltip('table-action', t('table.action', 'Action'));
      const exePath = document.getElementById('exe-path');
      if (exePath) exePath.placeholder = t('tracked_apps.path_placeholder', 'Path');
      const addExe = document.getElementById('add-exe-button');
      if (addExe) addExe.dataset.tooltip = t('tracked_apps.add_exe', 'Add execution file');
      const integrationDir = document.getElementById('integration-dir');
      if (integrationDir) integrationDir.placeholder = t('tracked_apps.path_placeholder', 'Path');
      for (const id of ['profile-export-attach-profile-input', 'profile-export-patch-profile-input', 'profile-export-merge-profile-input']) {
        const profilePath = document.getElementById(id);
        if (profilePath) profilePath.placeholder = t('tracked_apps.path_placeholder', 'Path');
      }
      renderIntegrationModulePanel(state.snapshot);
      renderIntegrationModal(state.snapshot);
      renderProfileExportDialog();
      renderIgnoredDeleteModal();
      renderObservationDeleteModal();
      renderHeaderFilters(state.snapshot);
    }

    function renderLanguageSelect() {
      const button = document.getElementById('language-select');
      const label = document.getElementById('language-select-label');
      const menu = document.getElementById('language-select-menu');
      if (!button || !label || !menu || !state.languageCatalog) return;
      const current = currentLanguageCode();
      const languages = state.languageCatalog.languages || [];
      const selected = languages.find((language) => String(language.code).toLowerCase() === String(current).toLowerCase());
      label.textContent = selected?.label || current;
      button.setAttribute('aria-expanded', String(!menu.hidden));
      menu.innerHTML = languages
        .map((language) => {
          const selectedClass = String(language.code).toLowerCase() === String(current).toLowerCase()
            ? ' language-select-option--selected'
            : '';
          const codeLiteral = JSON.stringify(String(language.code)).replaceAll('"', '&quot;');
          return '<button class="language-select-option' + selectedClass + '" type="button" role="option" aria-selected="' + (selectedClass ? 'true' : 'false') + '" onclick="setLanguage(' + codeLiteral + ')">' + html(language.label) + '</button>';
        })
        .join('');
    }

    function renderHeaderFilters(snapshot) {
      const cloudImportActive = state.cloudPanel === 'import';
      const filters = cloudImportActive ? state.cloud.rowFilters : currentFilters();
      const observationFilter = document.getElementById('observation-filter');
      if (observationFilter) {
        observationFilter.innerHTML = [
          ['All', t('filter.all', 'All')],
          ['Unconfirmed', t('filter.unconfirmed', 'Not selected')],
          ['Confirmed', t('filter.confirmed', 'Selected')],
          ['Success', t('filter.success', 'Success')],
          ['Failed', t('filter.failed', 'Failure')]
        ].map(([value, label]) => '<option value="' + value + '">' + html(label) + '</option>').join('');
        const currentObservationFilter = ['All', 'Unconfirmed', 'Confirmed', 'Success', 'Failed'].includes(text(filters.observation_filter))
          ? text(filters.observation_filter)
          : 'All';
        observationFilter.value = currentObservationFilter;
        observationFilter.setAttribute('aria-label', localizedConfirmationFilterLabel());
      }
      const appFilter = document.getElementById('app-filter');
      if (appFilter) {
        const apps = cloudImportActive
          ? uniqueSorted(state.cloud.rows.map((row) => row.application))
          : uniqueSorted(
              filteredObservations(snapshot, { ignoreAppFilter: true }).map((row) => appName(snapshot, row.tracked_app_id, row))
            );
        appFilter.innerHTML = [
          '<option value="">' + html(t('filter.all', 'All')) + '</option>'
        ].concat(apps.map((value) => '<option value="' + html(value) + '">' + html(value) + '</option>')).join('');
        appFilter.value = filters.app_search && apps.includes(filters.app_search) ? filters.app_search : '';
        appFilter.setAttribute('aria-label', t('filter.app', 'App'));
      }
      const portFilter = document.getElementById('port-filter');
      if (portFilter) {
        portFilter.placeholder = t('filter.port_placeholder', 'Port');
        if (document.activeElement !== portFilter) portFilter.value = text(filters.port_search);
        portFilter.setAttribute('aria-label', t('filter.port', 'Port'));
      }
      const protocolFilter = document.getElementById('protocol-filter');
      if (protocolFilter) {
        const protocols = cloudImportActive
          ? uniqueSorted(state.cloud.rows.map((row) => text(row.protocol)).filter(Boolean))
          : uniqueSorted(
              filteredObservations(snapshot, { ignoreProtocolFilter: true }).map((row) => text(row.protocol)).filter(Boolean)
            );
        protocolFilter.innerHTML = [
          '<option value="">' + html(t('filter.protocol.all', 'All')) + '</option>'
        ].concat(protocols.map((value) => '<option value="' + html(value) + '">' + html(value) + '</option>')).join('');
        if (filters.protocol && protocols.some((value) => value.toLowerCase() === text(filters.protocol).toLowerCase())) {
          const current = protocols.find((value) => value.toLowerCase() === text(filters.protocol).toLowerCase()) || '';
          protocolFilter.value = current;
        } else {
          protocolFilter.value = '';
        }
        protocolFilter.setAttribute('aria-label', t('filter.protocol', 'Protocol'));
      }
      const ipFilter = document.getElementById('ip-filter');
      if (ipFilter) {
        ipFilter.placeholder = t('filter.search_placeholder', 'Search by IP');
        if (document.activeElement !== ipFilter) ipFilter.value = text(filters.ip_search);
        ipFilter.setAttribute('aria-label', t('table.ip', 'IP'));
      }
      const domainFilter = document.getElementById('domain-filter');
      if (domainFilter) {
        domainFilter.placeholder = t('filter.domain_placeholder', 'Domain');
        if (document.activeElement !== domainFilter) domainFilter.value = text(filters.domain_search);
        const domainTooltip = t('filter.domain_tooltip', 'Domain filter. Use * as any number of characters, for example *example.com or example*.com.');
        domainFilter.setAttribute('aria-label', domainTooltip);
        domainFilter.dataset.tooltip = domainTooltip;
      }
      const publicIpFilter = document.getElementById('public-ip-filter');
      if (publicIpFilter) {
        publicIpFilter.setAttribute('aria-checked', Boolean(filters.public_ip) ? 'true' : 'false');
        publicIpFilter.classList.toggle('switch--on', Boolean(filters.public_ip));
        publicIpFilter.setAttribute('aria-label', t('observations.public_ip', 'Public'));
      }
      const publicIpLabel = document.getElementById('public-ip-label');
      if (publicIpLabel) {
        const publicIpLabelText = publicIpLabel.querySelector('.header-switch-row__label');
        if (publicIpLabelText) publicIpLabelText.textContent = t('observations.public_ip', 'Public');
        publicIpLabel.dataset.tooltip = t('observations.public_ip_tooltip', 'On shows public IPs; off shows non-public IPs.');
      }
      syncClearButtonStates();
    }

    function uniqueSorted(values, numeric) {
      const deduped = Array.from(new Set((values || []).map((value) => text(value).trim()).filter(Boolean)));
      return deduped.sort((left, right) => numeric
        ? Number(left) - Number(right) || left.localeCompare(right)
        : left.localeCompare(right));
    }

    function renderHeader(snapshot) {
      const tracked = snapshot.tracked_apps || [];
      const enabled = tracked.filter((app) => app.enabled).length;
      renderHeaderFilters(snapshot);
      document.getElementById('header-labels').innerHTML = [
        domainCaptureSwitch(snapshot.app_settings?.domain_capture_enabled),
        webSwitch(snapshot.app_settings?.web_access_localhost)
      ].join('');
      document.getElementById('footer-apps').textContent = t('footer.apps', 'Apps') + ': ' + enabled + '/' + tracked.length;
      renderFooterWatcher(state.watcherConnected);
      renderFooterTool(snapshot);
      renderFooterWebServer(snapshot);
      renderFooterNetwork(snapshot);
      renderCloudPanel();
    }

    function renderFooterWatcher(connected) {
      const container = document.getElementById('footer-watcher-status');
      const label = document.getElementById('footer-watcher-label');
      const dot = document.getElementById('footer-watcher-dot');
      if (!container || !label || !dot) return;
      label.textContent = t('footer.watcher.label', 'Watcher');
      dot.className = 'footer-watcher-dot ' + (!state.snapshot ? 'footer-watcher-dot--inactive' : connected ? 'footer-watcher-dot--connected' : 'footer-watcher-dot--disconnected');
      container.dataset.tooltip = availabilityStatusLine(
        t('status.watcher', 'Watcher status'),
        connected,
        browserUiLogUrl()
      );
    }

    function renderFooterTool(snapshot) {
      const container = document.getElementById('footer-tool-status');
      const label = document.getElementById('footer-tool-label');
      const dot = document.getElementById('footer-tool-dot');
      if (!container || !label || !dot) return;
      const available = Boolean(snapshot?.runtime_status?.tool_available);
      label.textContent = t('footer.tool.label', 'Tool');
      dot.className = 'footer-watcher-dot ' + (!snapshot ? 'footer-watcher-dot--inactive' : available ? 'footer-watcher-dot--connected' : 'footer-watcher-dot--disconnected');
      container.dataset.tooltip = availabilityStatusLine(
        t('status.tool', 'Tool status'),
        available,
        TOOL_MODULE_NAME
      );
    }

    function renderFooterWebServer(snapshot) {
      const container = document.getElementById('footer-web-server-status');
      const label = document.getElementById('footer-web-server-label');
      const dot = document.getElementById('footer-web-server-dot');
      if (!container || !label || !dot) return;
      const enabled = Boolean(snapshot?.app_settings?.web_access_localhost);
      const available = Boolean(enabled && state.watcherConnected);
      label.textContent = t('footer.web_server.label', 'Web server');
      dot.className = 'footer-watcher-dot '
        + (available ? 'footer-watcher-dot--connected' : enabled ? 'footer-watcher-dot--disconnected' : 'footer-watcher-dot--inactive');
      container.dataset.tooltip = webServerStatusLine(snapshot);
    }

    function renderFooterNetwork(snapshot) {
      const container = document.getElementById('footer-network-status');
      const label = document.getElementById('footer-network-label');
      const dot = document.getElementById('footer-network-dot');
      if (!container || !label || !dot) return;
      const probe = snapshot?.runtime_status?.endpoint_probe || { is_checking: true, probes: [] };
      const available = Boolean(probe.first_successful_target);
      const checking = Boolean(!available && (probe.is_checking || !state.watcherConnected));
      label.textContent = t('footer.network.label', 'DNS');
      dot.className = 'footer-watcher-dot ' + (!snapshot ? 'footer-watcher-dot--inactive' : available ? 'footer-watcher-dot--connected' : checking ? 'footer-watcher-dot--inactive' : 'footer-watcher-dot--disconnected');
      container.dataset.tooltip = endpointProbeTooltip(probe);
    }

    function endpointProbeTooltip(probe) {
      const lines = [];
      if (probe.first_successful_target) {
        lines.push(t('footer.network.available_prefix', 'First reachable DNS:') + ' ' + probe.first_successful_target);
      } else if (!state.watcherConnected) {
        lines.push(t('status.waiting_for_watcher', 'Waiting for watcher'));
      } else if (probe.is_checking) {
        lines.push(t('footer.network.checking', 'endpoint probes are checking'));
      } else if ((probe.probes || []).length) {
        const item = probe.probes[0];
        const target = String(item?.target || '').trim();
        const error = String(item?.error || '').trim();
        const detail = target && error ? (target + ': ' + error) : (target || error);
        lines.push(detail
          ? t('footer.network.unavailable', 'All endpoint probes are unavailable') + ': ' + detail
          : t('footer.network.unavailable', 'All endpoint probes are unavailable'));
      } else {
        lines.push(t('footer.network.unavailable', 'All endpoint probes are unavailable'));
      }
      for (const item of probe.probes || []) {
        const state = item.available === true
          ? t('footer.network.true', 'True')
          : item.available === false
            ? t('footer.network.false', 'False')
            : t('footer.network.unknown', 'Checking');
        lines.push(item.target + ': ' + state + (item.error ? ' (' + item.error + ')' : ''));
      }
      return lines.join('\n');
    }

    function browserUiUrl() {
      return state.webUrl?.url || window.location.origin + '/';
    }

    function browserUiLogUrl() {
      return browserUiUrl().replace(/\/+$/u, '');
    }

    function webServerUrlFallbackLine() {
      return state.webUrl?.used_localhost_fallback
        ? t('footer.web_server.localhost_fallback', 'No LAN address was detected; localhost URL is used')
        : '';
    }

    async function copyTextToClipboard(text) {
      try {
        if (navigator.clipboard && window.isSecureContext) {
          await navigator.clipboard.writeText(text);
          return;
        }
      } catch (_) {}
      const area = document.createElement('textarea');
      area.value = text;
      area.setAttribute('readonly', '');
      area.style.position = 'fixed';
      area.style.left = '-9999px';
      document.body.appendChild(area);
      area.select();
      try {
        document.execCommand('copy');
      } catch (_) {}
      document.body.removeChild(area);
    }

    function normalizeStatusSeverity(severityOrFailed, line = '') {
      if (severityOrFailed === true) return 'error';
      const raw = text(severityOrFailed).trim().toLowerCase();
      if (['success', 'warning', 'error', 'info', 'debug'].includes(raw)) return raw;
      const lower = text(line).trim().toLowerCase();
      if (lower.startsWith(t('status.error', 'Error').toLowerCase() + ':')) return 'error';
      return 'info';
    }

    function statusHistoryEntry(line, severityOrFailed) {
      const value = normalizeLogLine(line);
      if (!value) return null;
      return { message: value, severity: normalizeStatusSeverity(severityOrFailed, value) };
    }

    function recordFooterMessageEvent(entry) {
      if (!entry?.message) return;
      api('/v1/system-events', {
        method: 'POST',
        body: JSON.stringify({
          source: 'core',
          component: 'ui',
          action_type: 'message',
          severity: entry.severity || 'info',
          entity_type: 'footer',
          payload: { message: entry.message }
        })
      }).catch(() => {});
    }

    function pushStatusLine(line, severityOrFailed = false) {
      const entry = statusHistoryEntry(line, severityOrFailed);
      if (!entry) return;
      recordFooterMessageEvent(entry);
      const history = state.statusHistory;
      const last = history[history.length - 1];
      if (last?.message === entry.message && last?.severity === entry.severity) return;
      history.push(entry);
      if (history.length > 10) {
        history.splice(0, history.length - 10);
      }
    }

    function normalizeLogLine(line) {
      return text(line).trim().replace(/\.+$/u, '');
    }

    function availabilityStatusLine(label, available, detail) {
      return label + ': ' + t(available ? 'status.available' : 'status.unavailable', available ? 'Available' : 'Unavailable') + ' (' + detail + ')';
    }

    function webServerStatusLine(snapshot, watcherModule = 'embedded watcher v' + text('__WATCHER_VERSION__')) {
      const enabled = Boolean(snapshot?.app_settings?.web_access_localhost);
      const available = Boolean(enabled && state.watcherConnected);
      const stateLabel = enabled
        ? (available ? t('status.available', 'Available') : t('status.unavailable', 'Unavailable'))
        : t('status.disabled', 'Disabled');
      return t('status.web_server', 'Web server status') + ': ' + stateLabel + ' (' + browserUiLogUrl() + '; ' + watcherModule + ')';
    }

    function dnsStatusLine(probe, toolModule) {
      const current = probe || { is_checking: true, probes: [] };
      let stateLabel = t('status.unavailable', 'Unavailable');
      if (current.first_successful_target) {
        stateLabel = t('status.available', 'Available') + ' (' + current.first_successful_target + '; ' + toolModule + ')';
      } else if (!state.watcherConnected) {
        stateLabel = t('status.waiting_for_watcher', 'Waiting for watcher') + ' (' + toolModule + ')';
      } else if (current.is_checking) {
        stateLabel = t('status.checking', 'Checking') + ' (' + toolModule + ')';
      } else if ((current.probes || []).length) {
        const item = current.probes[0];
        const target = String(item?.target || '').trim();
        const error = String(item?.error || '').trim();
        const detail = target && error ? (target + ': ' + error) : (target || error);
        stateLabel = t('status.unavailable', 'Unavailable') + ' (' + (detail ? detail + '; ' : '') + toolModule + ')';
      } else {
        stateLabel = t('status.unavailable', 'Unavailable') + ' (' + toolModule + ')';
      }
      return t('status.dns', 'DNS status') + ': ' + stateLabel;
    }

    function pushServiceStatusEvent(key, line) {
      const value = normalizeLogLine(line);
      if (!value) return;
      if (state.lastServiceStatusEvents[key] === value) return;
      state.lastServiceStatusEvents[key] = value;
      pushStatusLine(value);
    }

    function trackedAppsStatusLine(snapshot) {
      const apps = snapshot?.tracked_apps || [];
      const enableAll = Boolean(snapshot?.app_settings?.ui_enable_all_overlay);
      const enabled = apps.filter((app) => enableAll || Boolean(app?.enabled)).length;
      return enabled + '\\' + apps.length;
    }

    function reportRuntimeServiceEvents(snapshot) {
      if (!snapshot) return;
      const watcherModule = 'embedded watcher v' + text('__WATCHER_VERSION__');
      pushServiceStatusEvent(
        'build',
        t('status.build_version', 'Build version') + ': NetStitch v' + text('__BUILD_VERSION__')
      );
      pushServiceStatusEvent(
        'current',
        t('status.current', 'Current status') + ': ' + text(snapshot?.ui?.status_text)
      );
      pushServiceStatusEvent(
        'app',
        t('status.current_app', 'App status') + ': ' + trackedAppsStatusLine(snapshot)
      );
      pushServiceStatusEvent(
        'watcher',
        availabilityStatusLine(
          t('status.watcher', 'Watcher status'),
          Boolean(snapshot?.ui?.watcher_connected),
          window.location.origin + '; ' + watcherModule
        )
      );
      pushServiceStatusEvent(
        'tool',
        availabilityStatusLine(
          t('status.tool', 'Tool status'),
          Boolean(snapshot?.runtime_status?.tool_available),
          'netstitch-tool native v' + text('__TOOL_VERSION__')
        )
      );
      pushServiceStatusEvent('web', webServerStatusLine(snapshot, watcherModule));
      pushServiceStatusEvent(
        'dns',
        dnsStatusLine(
          snapshot?.runtime_status?.endpoint_probe,
          'netstitch-tool native v' + text('__TOOL_VERSION__')
        )
      );
    }

    function systemEventFooterEntry(event) {
      const message = normalizeLogLine(event?.payload?.message);
      if (!message) return null;
      return { message, severity: normalizeStatusSeverity(event?.severity, message) };
    }

    async function loadFooterMessages(limit = 10) {
      try {
        const response = await api('/v1/system-events?limit=' + encodeURIComponent(String(limit)));
        const entries = (Array.isArray(response?.events) ? response.events : [])
          .filter((event) => text(event?.component) === 'ui' && text(event?.action_type) === 'message')
          .map(systemEventFooterEntry)
          .filter(Boolean)
          .reverse();
        if (limit <= 10) {
          state.statusHistory = entries.slice(-10);
          renderFooterMessage();
        }
        return entries;
      } catch (_error) {
        return state.statusHistory.slice(-limit);
      }
    }

    function activeFooterMessageLines() {
      return state.statusHistory;
    }

    function statusHistoryTooltip(lines = activeFooterMessageLines()) {
      if (!lines.length) return '';
      const keep = Math.min(6, lines.length);
      const hidden = Math.max(0, lines.length - keep);
      const visible = hidden > 0 ? ['...' + hidden] : [];
      return visible.concat(lines.slice(lines.length - keep).map((entry) => entry.message)).join('\n');
    }

    function renderFooterMessage() {
      const target = document.getElementById('footer-message-text');
      if (!target) return;
      const lines = activeFooterMessageLines();
      const latest = lines[lines.length - 1] || null;
      target.value = latest?.message || '';
      target.dataset.tooltip = statusHistoryTooltip(lines);
      target.classList.toggle('footer-message-text--success', latest?.severity === 'success');
      target.classList.toggle('footer-message-text--warning', latest?.severity === 'warning');
      target.classList.toggle('footer-message-text--error', latest?.severity === 'error');
    }

    function cloudProgressStages(totalRows) {
      const chunkCount = Math.max(1, Math.ceil(Math.max(1, Number(totalRows) || 1) / 500));
      return Array.from({ length: chunkCount }, () => ({
        label: '',
        width: 1,
        className: 'progress-bar__segment--accent'
      }));
    }

    function setCloudFooterProgress(panel, label, percent, meta = '', totalRows = 0) {
      const target = document.getElementById(panel === 'export' ? 'cloud-export-progress' : 'cloud-import-progress');
      if (!target) return;
      const bounded = Math.min(100, Math.max(1, Number(percent) || 1));
      const rows = Number(totalRows || meta || 0);
      target.hidden = false;
      target.innerHTML = progressBarEntityHtml(
        text(label, 'Progress'),
        text(meta),
        bounded,
        cloudProgressStages(rows),
        true,
        true
      );
    }

    function clearCloudFooterProgress(panel) {
      const target = document.getElementById(panel === 'export' ? 'cloud-export-progress' : 'cloud-import-progress');
      if (!target) return;
      target.hidden = true;
      target.innerHTML = '';
    }

    function seedConnectorEvents(snapshot) {
      if (state.initialConnectorEventsSeeded || !state.watcherConnected) return;
      if (!(snapshot?.tracked_apps || []).length) return;
      const runtime = snapshot?.runtime_status || {};
      const count = Number(runtime.connector_apps_detected || 0);
      pushStatusLine(count > 0
        ? t('footer.event.connectors_loaded_prefix', 'Application connectors loaded:') + ' ' + count + ' ' + t('footer.event.connector_apps_suffix', 'detected apps')
        : t('footer.event.connectors_loaded_empty', 'Application connectors loaded: no detected apps'));
      (runtime.unavailable_tracked_apps || []).forEach((app) => {
        pushStatusLine(trackedAppAvailabilityLine(
          t('footer.event.tracked_app_unavailable_prefix', 'Tracked app unavailable:'),
          app.display_name || app.exe_path,
          app.exe_path
        ));
      });
      state.initialConnectorEventsSeeded = true;
    }

    function seedModuleEvents(snapshot) {
      if (!state.watcherConnected) return;
      const modules = snapshot?.runtime_status?.integration_modules || [];
      if (!modules.length) return;
      modules.forEach((module) => {
        const key = integrationModuleStatusKey(module);
        if (state.moduleStatusEventKeys.has(key)) return;
        pushStatusLine(integrationModuleStatusLine(module));
        state.moduleStatusEventKeys.add(key);
      });
    }

    function integrationModuleStatusKey(module) {
      return [
        text(module?.id),
        text(module?.manifest_path),
        module?.connected ? '1' : '0',
        text(module?.error),
      ].join('|');
    }

    function integrationModuleStatusLine(module) {
      const name = text(module?.display_name || module?.id, 'Module');
      if (module?.connected) {
        return t('footer.event.module_connected_prefix', 'Module connected:') + ' ' + name;
      }
      return t('footer.event.module_failed_prefix', 'Module failed:') + ' ' + name + ' - ' + text(module?.error, 'unknown error');
    }

    function trackedAppAvailabilityLine(prefix, displayName, exePath) {
      return prefix + ' ' + text(displayName, exePath) + ' - ' + text(exePath);
    }

    function setFooterMessage(message, failed = false) {
      pushStatusLine(cloudReadableMessage(message), failed);
      renderFooterMessage();
    }

    async function copyWebServerUrl(event) {
      if (event) event.stopPropagation();
      const url = browserUiLogUrl();
      await copyTextToClipboard(url);
      const fallback = webServerUrlFallbackLine();
      if (fallback) pushStatusLine(fallback);
      setFooterMessage(t('footer.web_server.copied_prefix', 'Web server address copied to clipboard:') + ' ' + url);
    }

    async function copyStatusHistory(event) {
      if (event) event.stopPropagation();
      const lines = await loadFooterMessages(100);
      await copyTextToClipboard(lines.slice(-100).map((entry) => entry.message).join('\n'));
      setFooterMessage(t('footer.status.copied', 'Status history copied to clipboard'));
    }

    function toggleLanguageMenu(event) {
      if (event) event.stopPropagation();
      const menu = document.getElementById('language-select-menu');
      const button = document.getElementById('language-select');
      if (!menu || !button) return;
      menu.hidden = !menu.hidden;
      button.setAttribute('aria-expanded', String(!menu.hidden));
    }

    function closeLanguageMenu() {
      const menu = document.getElementById('language-select-menu');
      const button = document.getElementById('language-select');
      if (!menu || !button) return;
      menu.hidden = true;
      button.setAttribute('aria-expanded', 'false');
    }

    function headerLabel(label, extra) {
      return '<span class="header-label ' + extra + '">' + html(label) + '</span>';
    }

    function localizedHeaderWebAccessLabel() {
      const localized = text(t('header.web_access.localhost', 'Web server')).trim();
      if (!localized || localized.toLowerCase() === 'web localhost') {
        return currentLanguageCode().startsWith('ru') ? 'Web сервер' : 'Web server';
      }
      return localized;
    }

    function webSwitch(enabled) {
      const label = localizedHeaderWebAccessLabel();
      return '<span class="header-switch-row header-switch-row--server"><span class="header-switch-row__label">' + html(label) + '</span> <button id="web-server-switch" class="input-box switch ' + (enabled ? 'switch--on' : '') + '" type="button" onclick="toggleWebAccess()" aria-label="' + html(label) + '"><span class="switch__knob"></span></button></span>';
    }

    function domainCaptureSwitch(enabled) {
      const label = t('header.domain_capture', 'Advanced mon.');
      return '<span class="header-switch-row header-switch-row--domain-capture"><span class="header-switch-row__label">' + html(label) + '</span> <button id="domain-capture-switch" class="input-box switch ' + (enabled ? 'switch--on' : '') + '" type="button" onclick="toggleDomainCapture()" aria-label="' + html(label) + '"><span class="switch__knob"></span></button></span>';
    }

    function renderTrackedApps(snapshot) {
      const apps = snapshot.tracked_apps || [];
      const enableAll = Boolean(snapshot.app_settings?.ui_enable_all_overlay);
      const enabledCount = apps.filter((app) => Boolean(app.enabled)).length;
      const trackedEnabledCount = document.getElementById('tracked-enabled-count');
      trackedEnabledCount.textContent = t('tracked_apps.enabled_status', 'On') + ': ' + enabledCount;
      trackedEnabledCount.className = 'panel-header-meta ' + (enabledCount > 0 ? 'panel-header-meta--success' : 'panel-header-meta--danger');
      document.getElementById('tracked-count-inline').textContent = apps.length + ' ' + t('tracked_apps.items', 'items');
      document.getElementById('enable-all-switch').className = 'input-box switch' + (enableAll ? ' switch--on' : '');
      document.getElementById('tracked-apps').innerHTML = apps.map((app) => {
        const name = trackedAppDisplayName(app);
        const icon = app.id !== null && app.id !== undefined
          ? '<img src="' + iconUrl(app.id) + '" alt="">'
          : html((name || '?').slice(0, 1).toUpperCase());
        const deleteButton = app.icon_key === 'manual'
          ? '<button class="button button--danger button--square button--close" type="button" onclick="deleteTrackedApp(event, ' + app.id + ')" aria-label="Delete tracked app"><img class="button__icon" src="' + CLOSE_ICON_SRC + '" alt=""></button>'
          : '';
        return '<div class="tracked-app ' + (app.enabled ? 'tracked-app--enabled' : '') + '" onclick="toggleTrackedApp(' + app.id + ')">'
          + '<div class="app-icon">' + icon + '</div>'
          + '<div class="app-main"><div class="app-title">' + html(name) + '</div>'
          + '<input class="app-path path-field path-field--content-width" type="text" readonly value="' + html(app.exe_path) + '" style="' + pathFieldContentWidthStyle(app.exe_path, 18, 96) + '" onclick="event.stopPropagation()">'
          + '</div>'
          + '<div class="app-actions">' + deleteButton + '<button class="input-box switch ' + (app.enabled ? 'switch--on' : '') + '" type="button" aria-label="Toggle tracked app" onclick="event.stopPropagation(); toggleTrackedApp(' + app.id + ')"><span class="switch__knob"></span></button></div>'
          + '</div>';
      }).join('') || '<span class="subtle">No tracked apps yet.</span>';
    }

    function renderMonitor(snapshot) {
      const status = snapshot.monitor_status || 'Stopped';
      const monitoring = ['Running', 'Starting'].includes(status);
      const monitor = document.getElementById('monitor-state');
      if (monitor) {
        monitor.textContent = monitoring
          ? t('monitor.running', 'Running')
          : t('monitor.stopped', 'Stopped');
        monitor.className = 'panel-header-meta ' + (monitoring ? 'panel-header-meta--success' : 'panel-header-meta--danger');
      }
      const toggleButton = document.getElementById('toggle-monitoring-button');
      if (toggleButton) {
        const label = monitoring
          ? t('action.stop_monitoring', 'Stop monitoring')
          : t('action.start_monitoring', 'Start monitoring');
        toggleButton.className = monitoring
          ? 'button button--icon button--monitoring button--monitoring-active header-action-button'
          : 'button button--icon button--monitoring header-action-button';
        toggleButton.setAttribute('aria-label', label);
        toggleButton.setAttribute('data-tooltip', label);
        toggleButton.setAttribute('data-tooltip-align', 'end');
        toggleButton.innerHTML = '<span class="button__icon-shell button__icon-shell--monitoring"><span class="button__icon-scale button__icon-scale--monitoring"><span class="button__icon-rotor button__icon-rotor--monitoring"><img class="button__icon button__icon--monitoring" src="' + MONITORING_ICON_SRC + '" alt=""></span></span></span>';
      }
      const ignored = snapshot.ignored_addresses || [];
      if (!state.watcherConnected && ignored.length === 0) {
        document.getElementById('ignored-addresses').innerHTML = Array.from({ length: 5 }, () =>
          '<div class="ignored-addresses__skeleton"></div>'
        ).join('');
        return;
      }
      document.getElementById('ignored-addresses').innerHTML =
        ignored.map((rule) =>
          '<div class="ignored-row"><input class="path-field ignored-addresses__address" type="text" readonly value="' + html(rule.address_pattern) + '" aria-label="' + html(rule.address_pattern) + '">'
          + ignoredAddressHelpHtml(rule, 'end')
          + '<input class="path-field domain-field" type="text" readonly value="' + html(ignoredAddressDomainText(rule.enrichment)) + '" aria-label="' + html(t('table.domain', 'Domain')) + '">'
          + '<button class="button button--danger button--square button--close" data-ui-action="delete-ignored-address" onclick="requestDeleteIgnoredAddress(event, ' + rule.id + ')" aria-label="' + html(t('ignored_addresses.remove', 'Remove ignored address')) + '" data-tooltip="' + html(t('ignored_addresses.remove', 'Remove ignored address')) + '" data-tooltip-align="end"><img class="button__icon" src="' + CLOSE_ICON_SRC + '" alt=""></button></div>'
        ).join('') || '<span class="ignored-addresses__empty">' + html(t('ignored_addresses.empty', 'No ignored addresses')) + '</span>';
    }

    function renderObservations(snapshot) {
      const rows = filteredObservations(snapshot);
      const sourceRows = snapshot?.observed_endpoints || [];
      const selectedCount = sourceRows.filter((row) => observationRowSelected(row)).length;
      document.getElementById('observations-count').innerHTML =
        '<span class="panel-footer-meta__item">' + html(t('observations.total', 'Total rows')) + ': ' + html(sourceRows.length) + '</span>'
        + '<span class="panel-footer-meta__item">' + html(t('observations.displayed', 'Displayed rows')) + ': ' + html(rows.length) + '</span>'
        + '<span class="panel-footer-meta__item">' + html(t('observations.selected', 'Selected rows')) + ': ' + html(selectedCount) + '</span>';
      if (!snapshot && !rows.length) return;
      if (!state.watcherConnected && !rows.length) {
        document.getElementById('observations').innerHTML = '<tr><td colspan="10"><div class="observations-empty-state"></div></td></tr>';
        return;
      }
      document.getElementById('observations').innerHTML = rows.map((row) => {
        const exportReady = observationRowSelected(row);
        const rowClass = exportReady ? 'observation-row observation-row--confirmed' : 'observation-row';
        const domainText = enrichmentDomainText(row.enrichment);
        const domainTooltip = domainText
          ? ' data-tooltip="' + html(domainText) + '" data-tooltip-align="start"'
          : '';
        return '<tr class="' + rowClass + '" onclick="handleObservationRowClick(event, ' + row.id + ', ' + exportReady + ')">'
          + '<td><input class="path-field observation-app-field" type="text" readonly value="' + html(appName(snapshot, row.tracked_app_id, row)) + '" aria-label="' + html(appName(snapshot, row.tracked_app_id, row)) + '" data-tooltip="' + html(appName(snapshot, row.tracked_app_id, row)) + '" data-tooltip-align="start"></td>'
          + '<td><span class="ip-cell"><input class="path-field observation-ip-field" type="text" readonly value="' + html(row.remote_ip) + '" aria-label="' + html(row.remote_ip) + '" data-tooltip="' + html(row.remote_ip) + '" data-tooltip-align="start">' + enrichmentHelpHtml(row.enrichment, 'start') + '</span></td>'
          + '<td><input class="path-field domain-field" type="text" readonly value="' + html(domainText) + '" aria-label="' + html(t('table.domain', 'Domain')) + '"' + domainTooltip + '></td>'
          + '<td>' + html(row.remote_port) + '</td>'
          + '<td>' + html(row.protocol) + '</td>'
          + '<td><div class="observation-connection"><span class="' + html(connectionStateClass(row.connection_state)) + '">' + html(row.connection_state) + '</span><span class="connection-metrics">(<span class="connection-metrics__success">' + html(row.successful_hits || 0) + '</span>/<span class="connection-metrics__failure">' + html(row.failed_hits || 0) + '</span>)</span></div></td>'
          + '<td>' + html(row.hits) + '</td>'
          + '<td>' + formatTime(row.first_seen_ms) + '</td>'
          + '<td>' + formatTime(row.last_seen_ms) + '</td>'
          + '<td><div class="table-actions">'
          + '<button class="button button--icon button--square table-action-button" onclick="handleObservationRowAction(event, ' + row.id + ', ' + exportReady + ')" aria-label="' + html(exportReady ? t('action.unconfirm', 'Exclude from export') : t('action.confirm', 'Add for export')) + '" data-tooltip="' + html(exportReady ? t('action.unconfirm', 'Exclude from export') : t('action.confirm', 'Add for export')) + '" data-tooltip-align="end"><img class="button__icon ' + (exportReady ? 'button__icon--unconfirm-filtered' : 'button__icon--confirm-filtered') + ' table-action-button__icon" src="' + (exportReady ? UNCONFIRM_FILTERED_ICON_SRC : CONFIRM_FILTERED_ICON_SRC) + '" alt=""></button>'
          + '<button class="button button--icon button--square table-action-button" onclick="event.stopPropagation(); ignoreAddress(\'' + html(row.remote_ip) + '\')" aria-label="' + html(t('action.ignore_address', 'Ignore IP')) + '" data-tooltip="' + html(t('action.ignore_address', 'Ignore IP')) + '" data-tooltip-align="end"><img class="button__icon button__icon--unconfirm-filtered table-action-button__icon" src="' + IGNORE_ADDRESS_ICON_SRC + '" alt=""></button>'
          + '<button class="button button--danger button--square button--close" data-ui-action="delete-observation" data-ui-key="' + html(row.id) + '" onclick="requestDeleteObservation(event, ' + row.id + ')" aria-label="' + html(t('action.delete_observation', 'Delete row')) + '" data-tooltip="' + html(t('action.delete_observation', 'Delete row')) + '" data-tooltip-align="end"><img class="button__icon" src="' + CLOSE_ICON_SRC + '" alt=""></button>'
          + '</div></td>'
          + '</tr>';
      }).join('') || '<tr><td colspan="10" class="subtle">No observations match the current filters.</td></tr>';
    }

    function enrichmentDomainText(enrichment) {
      return text(enrichment?.domain_name, '').trim();
    }

    function connectionStateClass(state) {
      if (['Attempting', 'Failed'].includes(text(state))) return 'state-label state-label--danger';
      if (['Established', 'Closing'].includes(text(state))) return 'state-label state-label--success';
      return 'state-label';
    }

    function setHeaderTooltip(id, tooltip) {
      const node = document.getElementById(id);
      if (!node) return;
      node.dataset.tooltip = text(tooltip);
      node.dataset.tooltipAlign = 'start';
    }

    function ignoredAddressDomainText(enrichment) {
      return text(enrichment?.domain_name, '-');
    }

    function ignoredAddressHelpHtml(rule, align) {
      const tooltip = ignoredAddressTooltip(rule);
      if (!tooltip) return '';
      return '<span class="help-icon" data-tooltip="' + html(tooltip) + '" data-tooltip-align="' + html(align || 'start') + '" aria-label="' + html(tooltip) + '" tabindex="0">?</span>';
    }

    function ignoredAddressTooltip(rule) {
      const lines = [];
      if (ignoredRuleIsLoopback(rule.address_pattern)) {
        lines.push(t('enrichment.localhost_rule', 'This is a localhost address rule.'));
      } else if (ignoredRuleIsLocalMachineCandidate(rule.address_pattern)) {
        lines.push(t('enrichment.local_ip_rule', 'This is a local machine IP address rule.'));
      }
      const enrichment = enrichmentTooltip(rule.enrichment);
      if (enrichment) lines.push(enrichment);
      return lines.join('\n');
    }

    function enrichmentHelpHtml(enrichment, align) {
      const tooltip = enrichmentTooltip(enrichment);
      if (!tooltip) return '';
      return '<span class="help-icon" data-tooltip="' + html(tooltip) + '" data-tooltip-align="' + html(align || 'start') + '" aria-label="' + html(tooltip) + '" tabindex="0">?</span>';
    }

    function enrichmentTooltip(enrichment) {
      if (!enrichment) return '';
      return [
        [t('enrichment.domain', 'Domain'), enrichment.domain_name],
        [t('enrichment.owner', 'Owner'), enrichment.owner_name],
        [t('enrichment.range', 'Range'), enrichment.owner_range],
        [t('enrichment.registry', 'Registry'), enrichment.registry],
        [t('enrichment.source', 'Source'), enrichment.source]
      ].map(([label, value]) => label + ': ' + text(value, t('enrichment.unknown', 'Unknown'))).join('\n');
    }

    function safeHexColor(value) {
      return /^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(text(value).trim());
    }

    function moduleButtonStyle(module) {
      const color = text(module?.button_color).trim();
      return safeHexColor(color) ? ' style="background: ' + html(color) + '; border-color: ' + html(color) + ';"' : '';
    }

    function moduleIconSrc(module) {
      if (module?.icon_data_uri) return text(module.icon_data_uri);
      if (module?.icon_svg) return 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(text(module.icon_svg));
      if (module?.icon_path && module?.id) return '/v1/integrations/' + encodeURIComponent(text(module.id)) + '/icon';
      return '';
    }

    function moduleActionIconSrc(action) {
      if (action?.icon_data_uri) return text(action.icon_data_uri);
      if (action?.icon_svg) return 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(text(action.icon_svg));
      return '';
    }

    function moduleDialogFromCommand(command, module) {
      if (text(command?.command_type) !== 'show_dialog') return null;
      const payload = command?.payload || {};
      const message = text(payload.message || payload.text).trim();
      if (!message) return null;
      const dialogId = text(payload.dialog_id).trim() || text(module?.id, 'module') + '-' + Date.now();
      const buttons = text(payload.buttons, 'ok').trim().toLowerCase();
      return {
        module_id: text(module?.id),
        dialog_id: dialogId,
        title: text(module?.display_name, text(module?.id, 'Module')),
        message,
        show_cancel: buttons === 'ok_cancel',
        icon_src: moduleIconSrc(module),
        icon_fallback: moduleActionFallbackLabel(text(module?.icon_label, text(module?.display_name, 'M')))
      };
    }

    function handleModuleHostCommands(module, commands) {
      if (!Array.isArray(commands)) return false;
      let changed = false;
      commands.forEach((command) => {
        if (text(command?.command_type) === 'browse_window') {
          openModuleBrowseWindow(module, command);
          return;
        }
        if (text(command?.command_type) === 'set_module_page') {
          const page = text(command?.payload?.page, 'main').trim();
          state.moduleUiPage = page || 'main';
          changed = true;
          return;
        }
        if (text(command?.command_type) === 'set_ui_values') {
          const values = command?.payload?.values && typeof command.payload.values === 'object'
            ? command.payload.values
            : command?.payload;
          if (values && typeof values === 'object') {
            const nextValues = { ...(state.moduleUiValues || {}) };
            Object.entries(values).forEach(([key, value]) => {
              const normalizedKey = text(key).trim().slice(0, 120);
              if (!normalizedKey) return;
              if (value === null || value === undefined) {
                delete nextValues[normalizedKey];
              } else {
                nextValues[normalizedKey] = value;
              }
            });
            state.moduleUiValues = nextValues;
            changed = true;
          }
          return;
        }
        const dialog = moduleDialogFromCommand(command, module);
        if (!dialog) return;
        if (!state.moduleDialog || text(state.moduleDialog.module_id) === text(dialog.module_id)) {
          state.moduleDialog = dialog;
          changed = true;
        }
      });
      if (changed) {
        renderIntegrationModulePanel(state.snapshot);
        renderModuleHostDialog();
      }
      return changed;
    }

    function moduleUiActionToken(moduleId, actionId) {
      return text(moduleId, 'module') + ':' + text(actionId, 'action') + ':' + Date.now().toString(36) + ':' + Math.random().toString(36).slice(2);
    }

    function handleModuleUiActionEvents(module, events) {
      if (!Array.isArray(events) || events.length === 0) return 0;
      let lastSeq = 0;
      const commands = [];
      events.forEach((event) => {
        const seq = Number(event?.seq || 0);
        if (Number.isFinite(seq) && seq > lastSeq) lastSeq = seq;
        if (text(event?.event_type) === 'ui_values') {
          commands.push({ command_type: 'set_ui_values', payload: event?.payload || {} });
        }
      });
      if (commands.length > 0) handleModuleHostCommands(module, commands);
      return lastSeq;
    }

    async function fetchModuleUiActionEvents(module, actionToken, generation, after) {
      const moduleId = text(module?.id);
      const response = await api('/v1/integrations/ui-action-events?module_id=' + encodeURIComponent(moduleId) + '&ui_action_token=' + encodeURIComponent(actionToken) + '&after=' + encodeURIComponent(String(after || 0)));
      if (state.moduleUiActionGeneration !== generation) return after || 0;
      const lastSeq = handleModuleUiActionEvents(module, response?.events);
      return lastSeq > (after || 0) ? lastSeq : (after || 0);
    }

    async function pollModuleUiActionEvents(module, actionToken, generation) {
      let after = 0;
      while (state.moduleUiActionGeneration === generation && Boolean((state.moduleUiActionPollTokens || {})[actionToken])) {
        await delay(150);
        if (state.moduleUiActionGeneration !== generation || !Boolean((state.moduleUiActionPollTokens || {})[actionToken])) return;
        try {
          after = await fetchModuleUiActionEvents(module, actionToken, generation, after);
        } catch (_error) {
          return;
        }
      }
    }

    function moduleBrowseWindowMode(value) {
      const mode = text(value, 'file_open').trim();
      if (mode === 'folder' || mode === 'file_open' || mode === 'file_save') return mode;
      return 'file_open';
    }

    function moduleBrowseWindowExtensions(payload) {
      const values = [];
      const pushExtension = (extension) => {
        const normalized = text(extension).trim().replace(/^\./, '').toLowerCase();
        if (!normalized || !/^[a-z0-9_-]{1,32}$/.test(normalized)) return;
        if (!values.includes(normalized)) values.push(normalized);
      };
      (Array.isArray(payload?.filters) ? payload.filters : []).forEach((filter) => {
        (Array.isArray(filter?.extensions) ? filter.extensions : []).forEach(pushExtension);
      });
      pushExtension(payload?.default_extension);
      return values;
    }

    function moduleBrowseWindowDefaultTitle(mode) {
      if (mode === 'folder') return t('dialog.file_picker.folder_title', 'Choose folder');
      if (mode === 'file_save') return t('dialog.file_picker.save_title', 'Save file');
      return t('dialog.file_picker.title', 'Choose file');
    }

    function moduleBrowseWindowConfirmLabel(mode, value) {
      const label = text(value).trim();
      if (label) return label;
      if (mode === 'file_save') return t('dialog.file_picker.save', 'Save');
      return t('dialog.file_picker.choose', 'Choose');
    }

    function openModuleBrowseWindow(module, command) {
      const payload = command?.payload || {};
      const target = text(payload.target).trim().slice(0, 120);
      if (!target) return;
      const mode = moduleBrowseWindowMode(payload.mode);
      const currentValue = text((state.moduleUiValues || {})[target]);
      openServerFilePicker({
        intent: 'module_browse_window',
        mode,
        title: text(payload.title).trim() || moduleBrowseWindowDefaultTitle(mode),
        help: text(payload.help).trim() || t('dialog.file_picker.help', 'Files are shown from the machine where NetStitch is running.'),
        initialPath: text(payload.start_dir).trim() || pickerInitialDirectory(currentValue),
        save: mode === 'file_save',
        defaultFileName: text(payload.default_name).trim(),
        defaultExtension: text(payload.default_extension).trim(),
        extensions: moduleBrowseWindowExtensions(payload),
        overwritePolicy: text(payload.overwrite_policy, 'prompt').trim() || 'prompt',
        confirmLabel: moduleBrowseWindowConfirmLabel(mode, payload.confirm_label),
        moduleValueTarget: target,
        moduleStatusTarget: text(payload.status_target).trim().slice(0, 120),
        selectedStatus: text(payload.selected_status).trim()
      });
    }

    function renderModuleHostDialog() {
      const modal = document.getElementById('module-host-dialog-modal');
      if (!modal) return;
      const dialog = state.moduleDialog;
      modal.hidden = !dialog;
      if (!dialog) return;
      const title = document.getElementById('module-host-dialog-title');
      const message = document.getElementById('module-host-dialog-message');
      const icon = document.getElementById('module-host-dialog-icon');
      const cancel = document.getElementById('module-host-dialog-cancel');
      const ok = document.getElementById('module-host-dialog-ok');
      if (title) title.textContent = text(dialog.title, 'Module');
      if (message) message.textContent = text(dialog.message);
      if (icon) {
        const iconSrc = text(dialog.icon_src);
        icon.innerHTML = iconSrc
          ? '<img class="module-host-dialog__icon-image" src="' + html(iconSrc) + '" alt="">'
          : '<span class="module-host-dialog__icon-fallback">' + html(text(dialog.icon_fallback, '*')) + '</span>';
      }
      if (cancel) {
        cancel.hidden = !dialog.show_cancel;
        cancel.textContent = t('dialog.cancel', 'Cancel');
      }
      if (ok) ok.textContent = t('dialog.ok', 'OK');
    }

    function moduleDialogOwner(dialog) {
      const selected = selectedIntegrationModule(state.snapshot);
      if (selected && text(selected.id) === text(dialog.module_id)) return selected;
      return {
        id: text(dialog.module_id),
        display_name: text(dialog.title, 'Module'),
        icon_data_uri: text(dialog.icon_src),
        icon_label: text(dialog.icon_fallback, '*')
      };
    }

    async function closeModuleHostDialog(result) {
      const dialog = state.moduleDialog;
      if (!dialog) return;
      state.moduleDialog = null;
      renderModuleHostDialog();
      try {
        const response = await post('/v1/integrations/dialog-result', {
          module_id: text(dialog.module_id),
          dialog_id: text(dialog.dialog_id),
          result: result === 'cancel' ? 'cancel' : 'ok'
        });
        handleModuleHostCommands(moduleDialogOwner(dialog), response?.commands);
        pushStatusLine(text(dialog.title, 'Module') + ': dialog ' + (result === 'cancel' ? 'cancel' : 'ok'));
      } catch (error) {
        pushStatusLine(text(error?.message, 'Dialog result failed'), true);
      }
    }

    function orderedIntegrationModules(modules) {
      const order = Array.isArray(state.moduleOrder) ? state.moduleOrder : [];
      return modules.slice().sort((left, right) => {
        const leftIndex = order.indexOf(text(left.id));
        const rightIndex = order.indexOf(text(right.id));
        const normalizedLeft = leftIndex < 0 ? Number.MAX_SAFE_INTEGER : leftIndex;
        const normalizedRight = rightIndex < 0 ? Number.MAX_SAFE_INTEGER : rightIndex;
        if (normalizedLeft !== normalizedRight) return normalizedLeft - normalizedRight;
        return text(left.display_name).localeCompare(text(right.display_name));
      });
    }

    function normalizedModuleOrder(order) {
      const normalized = [];
      (Array.isArray(order) ? order : []).forEach((item) => {
        const id = text(item).trim();
        if (id && !normalized.includes(id)) normalized.push(id);
      });
      return normalized;
    }

    function syncModuleOrderFromSnapshot(snapshot) {
      const saved = normalizedModuleOrder(snapshot?.app_settings?.ui_module_order);
      if (!saved.length || JSON.stringify(saved) === JSON.stringify(state.moduleOrder || [])) return;
      state.moduleOrder = saved;
    }

    function moveModuleInOrder(moduleId, offset) {
      const modules = orderedIntegrationModules(enabledIntegrationModules(state.snapshot));
      const ids = [];
      for (const id of state.moduleOrder || []) {
        if (modules.some((module) => text(module.id) === text(id))) ids.push(text(id));
      }
      for (const module of modules) {
        if (!ids.includes(text(module.id))) ids.push(text(module.id));
      }
      const index = ids.indexOf(text(moduleId));
      if (index < 0) return;
      const target = Math.max(0, Math.min(ids.length - 1, index + offset));
      const [item] = ids.splice(index, 1);
      ids.splice(target, 0, item);
      state.moduleOrder = ids;
      if (state.snapshot?.app_settings) state.snapshot.app_settings.ui_module_order = ids.slice();
      post('/v1/settings', { key: 'ui.modules.order', value: JSON.stringify(ids) }, { skipSnapshotRefresh: true })
        .catch((error) => pushStatusLine(text(error?.message || error)));
      renderIntegration(state.snapshot);
    }

    function toggleModuleOrderEditing() {
      state.moduleOrderEditing = !state.moduleOrderEditing;
      renderIntegration(state.snapshot);
    }

    function renderIntegration(snapshot) {
      const modules = snapshot
        ? orderedIntegrationModules((snapshot.integration_modules || []).filter((module) => module && module.enabled !== false))
        : [];
      const ready = modules.length > 0;
      const headerStatus = document.getElementById('integration-ready');
      if (headerStatus) {
        headerStatus.textContent = ready
          ? template('integration.loaded', 'Loaded: {count}', { count: modules.length })
          : '';
        headerStatus.className = ready ? 'panel-header-meta panel-header-meta--success' : 'panel-header-meta';
      }
      const orderLabel = document.getElementById('modules-order-label');
      if (orderLabel) orderLabel.textContent = t('modules.order', 'Display order');
      const orderSwitch = document.getElementById('modules-order-switch');
      if (orderSwitch) {
        const label = t('modules.order', 'Display order');
        orderSwitch.className = 'input-box switch' + (state.moduleOrderEditing ? ' switch--on' : '');
        orderSwitch.setAttribute('aria-label', label);
        orderSwitch.setAttribute('data-tooltip', label);
        orderSwitch.setAttribute('data-tooltip-align', 'end');
      }
      const target = document.getElementById('integration-module-actions') || document.getElementById('integration-info');
      if (!target) return;
      if (!modules.length) {
        target.innerHTML = snapshot
          ? ''
          : Array.from({ length: 10 }, () => '<div class="integration-module-button-skeleton" aria-hidden="true"></div>').join('');
        renderIntegrationModulePanel(snapshot);
        return;
      }
      target.innerHTML = modules.map((module) => {
        const id = text(module.id);
        const name = text(module.display_name, id || 'Integration');
        let tooltip = text(module.tooltip, name);
        if (module.background_active) {
          tooltip = tooltip + ' - ' + t('modules.background_running', 'Running in background');
        }
        const icon = (text(module.icon_label).trim() || name.slice(0, 1) || '*').slice(0, 4);
        const iconSrc = moduleIconSrc(module);
        const iconHtml = iconSrc
          ? '<img class="integration-module-button__image" src="' + html(iconSrc) + '" alt="">'
          : '<span class="integration-module-button__icon">' + html(icon) + '</span>';
        const reorderLeft = state.moduleOrderEditing
          ? '<button class="button button--mini integration-module-reorder-button integration-module-reorder-button--left" type="button" onclick="moveModuleInOrder(' + html(JSON.stringify(id)) + ', -1)" data-tooltip="' + html(t('modules.order.move_left', 'Move left')) + '" aria-label="' + html(t('modules.order.move_left', 'Move left')) + '"><img class="button__icon integration-module-reorder-button__icon" src="' + MODULE_REORDER_LEFT_ICON_SRC + '" alt=""></button>'
          : '';
        const reorderRight = state.moduleOrderEditing
          ? '<button class="button button--mini integration-module-reorder-button integration-module-reorder-button--right" type="button" onclick="moveModuleInOrder(' + html(JSON.stringify(id)) + ', 1)" data-tooltip="' + html(t('modules.order.move_right', 'Move right')) + '" aria-label="' + html(t('modules.order.move_right', 'Move right')) + '"><img class="button__icon integration-module-reorder-button__icon integration-module-reorder-button__icon--right" src="' + MODULE_REORDER_LEFT_ICON_SRC + '" alt=""></button>'
          : '';
        const buttonClass = module.background_active
          ? 'button button--icon integration-module-button integration-module-button--background-active'
          : 'button button--icon integration-module-button';
        return '<div class="integration-module-button-shell">' + reorderLeft + '<button class="' + buttonClass + '" id="browser-integration-module-' + html(id) + '" data-ui-action="open-integration-module" data-ui-key="' + html(id) + '" onclick="openIntegrationModulePanel(' + html(JSON.stringify(id)) + ')" aria-label="' + html(name) + '" data-tooltip="' + html(tooltip) + '" data-tooltip-align="start"' + moduleButtonStyle(module) + '>' + iconHtml + '</button>' + reorderRight + '</div>';
      }).join('');
      renderIntegrationModulePanel(snapshot);
    }

    function enabledIntegrationModules(snapshot) {
      return (snapshot?.integration_modules || []).filter((module) => module && module.enabled !== false);
    }

    function selectedIntegrationModule(snapshot) {
      const modules = orderedIntegrationModules(enabledIntegrationModules(snapshot));
      if (!modules.length) return null;
      const selected = modules.find((module) => text(module.id) === text(state.selectedIntegrationModuleId));
      return selected || modules[0];
    }

    function selectedIntegrationModuleIdForRequest(snapshot = state.snapshot) {
      const module = selectedIntegrationModule(snapshot);
      const moduleId = text(module?.id || state.selectedIntegrationModuleId).trim();
      return moduleId || null;
    }

    function moduleOpenActionId(module) {
      const action = moduleActions(module)
        .find((item) => ['module.open', 'open'].includes(text(item?.id)) && item?.enabled !== false);
      return action ? text(action.id) : '';
    }

    function moduleActions(module) {
      const directActions = Array.isArray(module?.header_actions) ? module.header_actions : [];
      const schemaActions = [];
      function collectEntityActions(entity) {
        if (!entity || typeof entity !== 'object') return;
        if (Array.isArray(entity.actions)) schemaActions.push(...entity.actions);
        if (Array.isArray(entity.children)) entity.children.forEach(collectEntityActions);
      }
      if (Array.isArray(module?.ui_schema)) module.ui_schema.forEach(collectEntityActions);
      return [...directActions, ...schemaActions];
    }

    function scopedProfileExportPayload(request) {
      return {
        module_id: selectedIntegrationModuleIdForRequest(),
        request
      };
    }

    function moduleOverlayActive() {
      const moduleModal = document.getElementById('integration-module-modal');
      const integrationModal = document.getElementById('integration-modal');
      const advancedModal = document.getElementById('profile-export-advanced-wizard-modal');
      return Boolean(moduleModal && !moduleModal.hidden)
        || Boolean(integrationModal && !integrationModal.hidden && state.integrationChildReturnToModule)
        || Boolean(state.profileExport.open && state.profileExportReturnToModule)
        || Boolean(advancedModal && !advancedModal.hidden && state.profileExportReturnToModule);
    }

    function renderModuleHeader() {
      const mainHeader = document.getElementById('main-header-content');
      const moduleHeader = document.getElementById('module-header-content');
      if (!mainHeader || !moduleHeader) return;
      const active = moduleOverlayActive();
      mainHeader.hidden = active;
      moduleHeader.hidden = !active;
      if (!active) return;
      const module = selectedIntegrationModule(state.snapshot);
      const title = document.getElementById('module-header-title');
      if (title) title.textContent = text(module?.display_name, t('integration.title', 'Integration'));
      const actionsTarget = document.getElementById('module-header-actions');
      if (actionsTarget) {
        const actions = Array.isArray(module?.header_actions) ? module.header_actions : [];
      const displayedRows = filteredObservations(state.snapshot);
      const allRows = snapshotMonitoringRows(state.snapshot);
      const moduleBackgroundActive = Boolean(module?.background_active);
      const latestSourceRows = moduleBackgroundActive
        ? [...allRows].sort((left, right) => Number(right?.id || 0) - Number(left?.id || 0))
        : [];
      const schemaContext = {
        page: state.moduleUiPage || 'main',
        selectedRows: displayedRows.filter((row) => observationRowSelected(row)).length,
        displayedRows: displayedRows.length,
        totalRows: allRows.length,
        moduleBackgroundActive,
        lastRow: latestSourceRows.length ? moduleMonitoringRowLabel(latestSourceRows[0]) : '-',
        latestRows: moduleBackgroundActive ? latestMonitoringRowsLabel(allRows, 5) : latestMonitoringRowsLabel([], 5),
        latestRowsCount: moduleBackgroundActive ? Math.min(5, allRows.length) : 0
      };
        actionsTarget.innerHTML = actions.map((action) => {
          const id = text(action.id);
          const label = moduleUiContextValue(text(action.label, id || 'Action'), schemaContext);
          const tooltip = moduleUiContextValue(text(action.tooltip, label), schemaContext);
          const iconSrc = moduleActionIconSrc(action);
          const iconHtml = iconSrc
            ? '<img class="button__icon button__icon--module-action" src="' + html(iconSrc) + '" alt="">'
            : '<span class="button__icon-fallback button__icon-fallback--module-action">' + html(moduleActionFallbackLabel(label)) + '</span>';
          const pulseClass = moduleActionPulseClass(action, module?.background_active);
          return '<button class="button button--icon header-action-button' + pulseClass + '" type="button" data-ui-action="module-header-action" data-ui-key="' + html(id) + '" onclick="moduleHeaderAction(' + html(JSON.stringify(id)) + ')" data-tooltip="' + html(tooltip) + '" data-tooltip-align="end" aria-label="' + html(tooltip) + '"' + (action.enabled === false ? ' disabled' : '') + '>' + iconHtml + '</button>';
        }).join('');
      }
      const closeButton = document.getElementById('module-header-close-button');
      const closeLabel = t('dialog.close', 'Close');
      if (closeButton) {
        closeButton.setAttribute('aria-label', closeLabel);
        closeButton.setAttribute('data-tooltip', closeLabel);
        closeButton.setAttribute('data-tooltip-align', 'end');
      }
      const stopButton = document.getElementById('module-header-stop-button');
      const stopLabel = t('modules.stop', 'Stop');
      const stopTooltip = t('modules.stop_tooltip', 'Stops the module and its background processes');
      if (stopButton) {
        stopButton.setAttribute('aria-label', stopLabel);
        stopButton.setAttribute('data-tooltip', stopTooltip);
        stopButton.setAttribute('data-tooltip-align', 'end');
      }
    }

    async function moduleHeaderAction(actionId) {
      const module = selectedIntegrationModule(state.snapshot);
      const action = moduleActions(module).find((item) => text(item.id) === text(actionId));
      const label = text(action?.label, text(actionId, 'Action'));
      const moduleId = text(module?.id);
      if (!moduleId) {
        pushStatusLine(text(module?.display_name, t('integration.title', 'Integration')) + ': ' + label);
        return;
      }
      const displayedRows = filteredObservations(state.snapshot);
      const selectedIds = displayedRows
        .filter((row) => observationRowSelected(row))
        .map((row) => Number(row.id))
        .filter((id) => Number.isFinite(id));
      const displayedIds = displayedRows
        .map((row) => Number(row.id))
        .filter((id) => Number.isFinite(id));
      const generation = state.moduleUiActionGeneration;
      const actionToken = moduleUiActionToken(moduleId, actionId);
      state.moduleUiActionPollTokens = { ...(state.moduleUiActionPollTokens || {}), [actionToken]: true };
      const eventPolling = pollModuleUiActionEvents(module, actionToken, generation);
      try {
        const response = await post('/v1/integrations/ui-action', {
          module_id: moduleId,
          action_id: text(actionId),
          ui_action_token: actionToken,
          selected_monitoring_row_ids: selectedIds,
          displayed_monitoring_row_ids: displayedIds,
          filters: currentFilters(),
          payload: moduleUiPayload(module)
        });
        if (state.moduleUiActionGeneration !== generation) return;
        try {
          await fetchModuleUiActionEvents(module, actionToken, generation, 0);
        } catch (_error) {}
        state.moduleUiActionPollTokens = { ...(state.moduleUiActionPollTokens || {}), [actionToken]: false };
        try {
          await eventPolling;
        } catch (_error) {}
        handleModuleHostCommands(module, response?.commands);
        if (response?.message) pushStatusLine(text(response.message));
        else pushStatusLine(text(module?.display_name, t('integration.title', 'Integration')) + ': ' + label);
        if (response?.refresh || (Array.isArray(response?.commands) && response.commands.length > 0)) {
          await loadSnapshot({ background: true });
        }
      } catch (error) {
        pushStatusLine(text(module?.display_name, t('integration.title', 'Integration')) + ': ' + text(error?.message, 'Action failed'));
      } finally {
        state.moduleUiActionPollTokens = { ...(state.moduleUiActionPollTokens || {}), [actionToken]: false };
      }
    }

    function moduleActionFallbackLabel(label) {
      const value = text(label).trim();
      return value ? value[0].toUpperCase() : '*';
    }

    function moduleUiContextValue(value, context) {
      return text(value)
        .replaceAll('{context.tables.monitoring.selected_rows}', String(context?.selectedRows ?? 0))
        .replaceAll('{context.tables.monitoring.selected_total}', String(context?.selectedRows ?? 0) + '/' + String(context?.totalRows ?? 0))
        .replaceAll('{context.tables.monitoring.displayed_rows}', String(context?.displayedRows ?? 0))
        .replaceAll('{context.tables.monitoring.total_rows}', String(context?.totalRows ?? 0))
        .replaceAll('{context.module.background_active}', context?.moduleBackgroundActive ? 'true' : 'false')
        .replaceAll('{context.module.background_action_label}', context?.moduleBackgroundActive ? 'Stop' : 'Start')
        .replaceAll('{context.module.background_status}', context?.moduleBackgroundActive ? 'Running in background' : 'Stopped')
        .replaceAll('{context.monitoring.last_row}', text(context?.lastRow, '-'))
        .replaceAll('{context.monitoring.latest_rows}', text(context?.latestRows, '-'))
        .replaceAll('{context.monitoring.latest_rows_count}', String(context?.latestRowsCount ?? 0));
    }

    function moduleActionStyleClass(style) {
      const value = text(style).trim().toLowerCase();
      if (value === 'primary' || value === 'blue' || value === 'accent') return ' button--primary';
      return ' button--secondary';
    }

    function moduleActionAlignClass(align) {
      const value = text(align).trim().toLowerCase();
      if (value === 'center' || value === 'centre') return ' module-ui-schema__action--align-center';
      if (value === 'right' || value === 'end') return ' module-ui-schema__action--align-right';
      return ' module-ui-schema__action--align-left';
    }

    function moduleActionPulseClass(action, moduleBackgroundActive) {
      if (Boolean(action?.pulse) || (Boolean(action?.pulse_when_background_active) && Boolean(moduleBackgroundActive))) {
        return ' module-action-button--pulse';
      }
      return '';
    }

    function moduleUiScrollClass(scroll) {
      const value = text(scroll).trim().toLowerCase();
      if (value === 'x' || value === 'horizontal') return ' module-ui-schema--scroll-x';
      if (value === 'y' || value === 'vertical') return ' module-ui-schema--scroll-y';
      if (value === 'both' || value === 'xy' || value === 'x-y') return ' module-ui-schema--scroll-both';
      return '';
    }

    function moduleUiSizeClass(entity) {
      const value = text(entity?.size).trim().toLowerCase();
      if (value === 'auto' || value === 'content' || value === 'fit' || value === 'fit-content') return ' module-ui-schema--size-auto';
      if (value === 'stretch' || value === 'fill' || value === 'wide') return ' module-ui-schema--size-stretch';
      if (value === 'fullscreen' || value === 'full-screen' || value === 'full') return ' module-ui-schema--size-fullscreen';
      return '';
    }

    function moduleUiAlignClass(entity) {
      const value = text(entity?.align).trim().toLowerCase();
      if (value === 'left' || value === 'start') return ' module-ui-schema--align-left';
      if (value === 'center' || value === 'centre' || value === 'middle') return ' module-ui-schema--align-center';
      if (value === 'right' || value === 'end') return ' module-ui-schema--align-right';
      return ' module-ui-schema--align-left';
    }

    function moduleUiSanitizedDimension(value) {
      const candidate = text(value).trim();
      if (!candidate || candidate.length > 96) return '';
      if (/[;{}:]/.test(candidate)) return '';
      if (!/^[a-zA-Z0-9\s.%(),+\-*\/]+$/.test(candidate)) return '';
      return candidate;
    }

    function moduleUiEntityStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['height', 'height'],
        ['min_width', 'min-width'],
        ['min_height', 'min-height'],
        ['max_width', 'max-width'],
        ['max_height', 'max-height'],
        ['margin', 'margin'],
        ['padding', 'padding'],
        ['grid_column', 'grid-column'],
        ['grid_row', 'grid-row']
      ], true);
    }

    function moduleUiTextareaRowStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['min_width', 'min-width'],
        ['max_width', 'max-width'],
        ['margin', 'margin'],
        ['padding', 'padding'],
        ['grid_column', 'grid-column'],
        ['grid_row', 'grid-row']
      ], true);
    }

    function moduleUiTextareaControlStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['height', 'height'],
        ['min_height', 'min-height'],
        ['max_height', 'max-height']
      ], false);
    }

    function moduleUiGridStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['height', 'height'],
        ['min_width', 'min-width'],
        ['min_height', 'min-height'],
        ['max_width', 'max-width'],
        ['max_height', 'max-height'],
        ['margin', 'margin'],
        ['padding', 'padding'],
        ['columns', 'grid-template-columns'],
        ['rows', 'grid-template-rows'],
        ['gap', 'gap'],
        ['grid_column', 'grid-column'],
        ['grid_row', 'grid-row']
      ], true);
    }

    function moduleUiTableShellStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['height', 'height'],
        ['min_width', 'min-width'],
        ['min_height', 'min-height'],
        ['max_width', 'max-width'],
        ['max_height', 'max-height'],
        ['margin', 'margin'],
        ['padding', 'padding'],
        ['grid_column', 'grid-column'],
        ['grid_row', 'grid-row']
      ], true);
    }

    function moduleUiTableViewportStyle(entity) {
      return '';
    }

    function moduleUiButtonRowStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['height', 'height'],
        ['min_width', 'min-width'],
        ['min_height', 'min-height'],
        ['max_width', 'max-width'],
        ['max_height', 'max-height'],
        ['margin', 'margin'],
        ['grid_column', 'grid-column'],
        ['grid_row', 'grid-row']
      ], true);
    }

    function moduleUiButtonContentStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['padding', 'padding']
      ], false);
    }

    function moduleUiStyleFromFields(entity, fields, includeOpacity) {
      const parts = [];
      for (const [field, cssName] of fields) {
        const value = moduleUiSanitizedDimension(entity?.[field]);
        if (value) parts.push(cssName + ': ' + value);
      }
      if (includeOpacity) {
        const opacity = moduleUiSanitizedOpacity(entity?.opacity);
        if (opacity) parts.push('opacity: ' + opacity);
      }
      return parts.join('; ');
    }

    function moduleUiSanitizedOpacity(value) {
      const candidate = text(value).trim();
      if (!candidate.endsWith('%')) return '';
      const percent = Number(candidate.slice(0, -1).trim());
      if (!Number.isFinite(percent) || percent < 0 || percent > 100) return '';
      return Number.isInteger(percent) ? String(percent) + '%' : percent.toFixed(2) + '%';
    }

    function moduleUiStyleAttr(entity) {
      const style = moduleUiEntityStyle(entity);
      return style ? ' style="' + html(style) + '"' : '';
    }

    function moduleUiTableShellStyleAttr(entity) {
      const style = moduleUiTableShellStyle(entity);
      return style ? ' style="' + html(style) + '"' : '';
    }

    function moduleUiTableViewportStyleAttr(entity) {
      const style = moduleUiTableViewportStyle(entity);
      return style ? ' style="' + html(style) + '"' : '';
    }

    function moduleUiButtonRowStyleAttr(entity) {
      const style = moduleUiButtonRowStyle(entity);
      return style ? ' style="' + html(style) + '"' : '';
    }

    function moduleUiButtonContentStyleAttr(entity) {
      const style = moduleUiButtonContentStyle(entity);
      return style ? ' style="' + html(style) + '"' : '';
    }

    function moduleUiEntitySizeClass(entity) {
      return moduleUiSizeClass(entity);
    }

    function moduleUiEntityLayoutClass(entity) {
      return moduleUiEntitySizeClass(entity) + moduleUiAlignClass(entity);
    }

    function moduleUiEntityWithLayoutDefaults(entity) {
      const next = { ...(entity || {}) };
      const type = text(next.entity_type, 'row').replace(/_/g, '-').toLowerCase();
      const setDefault = (field, value) => {
        if (!text(next[field]).trim()) next[field] = value;
      };
      setDefault('size', 'stretch');
      setDefault('width', '100%');
      setDefault('min_width', '0');
      setDefault('opacity', '100%');
      if (['panel', 'subpanel', 'nested-subpanel', 'grid', 'layout-grid', 'tabs', 'tab-view'].includes(type)) {
        setDefault('height', 'auto');
        setDefault('min_height', '0');
        setDefault('scroll', 'off');
      } else if (['button', 'action-button'].includes(type)) {
        setDefault('margin', '8px 0 0');
        setDefault('padding', '0');
      } else if (['separator', 'help-text', 'help', 'table', 'progress', 'footer'].includes(type)) {
        setDefault('min_height', '0');
      }
      if (type === 'grid' || type === 'layout-grid') {
        setDefault('columns', 'repeat(auto-fit, minmax(180px, 1fr))');
        setDefault('gap', '8px');
      }
      setDefault('align', 'left');
      return next;
    }

    function moduleUiDialogLayoutForPage(entities, context = {}) {
      for (const entity of Array.isArray(entities) ? entities : []) {
        if (!moduleUiEntityVisibleForContext(entity, context)) continue;
        if (moduleUiEntityType(entity) === 'footer') continue;
        const layout = moduleUiDialogLayoutFromRootEntity(entity);
        if (layout) return layout;
      }
      return null;
    }

    function moduleUiDialogLayoutFromRootEntity(entity) {
      if (!moduleUiEntityRequestsDialogLayout(entity)) return null;
      const classes = [
        'integration-module-dialog--layout',
        moduleUiDialogSizeClass(entity?.size),
        moduleUiDialogAlignClass(entity?.align)
      ].filter(Boolean).join(' ');
      return {
        className: classes,
        style: moduleUiDialogStyle(entity)
      };
    }

    function moduleUiEntityRequestsDialogLayout(entity) {
      if (moduleUiSizeRequestsDialogLayout(entity?.size)) return true;
      return [
        entity?.width,
        entity?.height,
        entity?.min_width,
        entity?.min_height,
        entity?.max_width,
        entity?.max_height
      ].some(moduleUiDimensionUsesViewportUnit);
    }

    function moduleUiSizeRequestsDialogLayout(size) {
      const value = text(size).trim().toLowerCase();
      return value === 'fullscreen' || value === 'full-screen' || value === 'full';
    }

    function moduleUiDialogSizeClass(size) {
      return moduleUiSizeRequestsDialogLayout(size) ? 'integration-module-dialog--size-fullscreen' : '';
    }

    function moduleUiDialogAlignClass(align) {
      const value = text(align).trim().toLowerCase();
      if (value === 'left' || value === 'start') return 'integration-module-dialog--align-left';
      if (value === 'right' || value === 'end') return 'integration-module-dialog--align-right';
      return 'integration-module-dialog--align-center';
    }

    function moduleUiDialogStyle(entity) {
      return moduleUiStyleFromFields(entity, [
        ['width', 'width'],
        ['height', 'height'],
        ['min_width', 'min-width'],
        ['min_height', 'min-height'],
        ['max_width', 'max-width'],
        ['max_height', 'max-height'],
        ['margin', 'margin'],
        ['padding', 'padding']
      ], true);
    }

    function moduleUiDimensionUsesViewportUnit(value) {
      const candidate = text(value).toLowerCase();
      return candidate.includes('vw')
        || candidate.includes('vh')
        || candidate.includes('vmin')
        || candidate.includes('vmax');
    }

    function moduleUiPayload(module) {
      return {
        ui_values: { ...(state.moduleUiValues || {}) },
        module_background_active: Boolean(module?.background_active),
        module: {
          background_active: Boolean(module?.background_active)
        }
      };
    }

    function snapshotMonitoringRows(snapshot) {
      return Array.isArray(snapshot?.observed_endpoints) ? snapshot.observed_endpoints : [];
    }

    function moduleUiCurrentValue(entity, defaultValue) {
      const id = text(entity?.id);
      if (id && Object.prototype.hasOwnProperty.call(state.moduleUiValues || {}, id)) {
        return text(state.moduleUiValues[id]);
      }
      return text(defaultValue);
    }

    function moduleUiCurrentChecked(entity, defaultValue) {
      const id = text(entity?.id);
      if (id && Object.prototype.hasOwnProperty.call(state.moduleUiValues || {}, id)) {
        return Boolean(state.moduleUiValues[id]);
      }
      return Boolean(defaultValue);
    }

    function moduleUiSetValue(id, value) {
      const key = text(id);
      if (!key) return;
      state.moduleUiValues = { ...(state.moduleUiValues || {}), [key]: text(value) };
    }

    function moduleUiSetChecked(id, checked) {
      const key = text(id);
      if (!key) return;
      state.moduleUiValues = { ...(state.moduleUiValues || {}), [key]: Boolean(checked) };
    }

    function moduleUiEntityChecked(entity, value) {
      if (typeof entity?.checked === 'boolean') return entity.checked;
      const normalized = text(value).trim().toLowerCase();
      return normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on';
    }

    function renderIntegrationUiEntity(entity, context = {}) {
      if (!entity || entity.hidden === true || entity.visible === false) return '';
      entity = moduleUiEntityWithLayoutDefaults(entity);
      const entityPage = text(entity.page).trim();
      if (entityPage && entityPage !== text(context?.page, 'main')) return '';
      const type = text(entity.entity_type, 'row').replace(/_/g, '-').toLowerCase();
      const id = text(entity.id, 'entity');
      const title = moduleUiContextValue(entity.title, context);
      const value = moduleUiContextValue(entity.value, context);
      const tooltip = moduleUiContextValue(entity.tooltip, context);
      const placeholder = moduleUiContextValue(entity.placeholder, context);
      const disabled = Boolean(entity.disabled);
      const readonly = Boolean(entity.readonly);
      const children = Array.isArray(entity.children) ? entity.children : [];
      const actions = Array.isArray(entity.actions) ? entity.actions : [];
      const alignClass = moduleUiAlignClass(entity);
      const sizeClass = moduleUiEntitySizeClass(entity) + alignClass;
      const styleAttr = moduleUiStyleAttr(entity);
      const childHtml = children.map((child) => renderIntegrationUiEntity(child, context)).join('');
      const actionHtml = actions.map((action) => {
        const actionId = text(action.id);
        const label = moduleUiContextValue(text(action.label, actionId || 'Action'), context);
        const actionTooltip = moduleUiContextValue(text(action.tooltip, label), context);
        const iconSrc = moduleActionIconSrc(action);
        const iconHtml = iconSrc
          ? '<img class="button__icon button__icon--module-action" src="' + html(iconSrc) + '" alt="">'
          : html(label);
        const pulseClass = moduleActionPulseClass(action, context?.moduleBackgroundActive);
        const actionAlignClass = moduleActionAlignClass(action.align);
        return '<div class="module-ui-schema__action-slot' + actionAlignClass + '"><button class="button module-ui-schema__action' + moduleActionStyleClass(action.style) + actionAlignClass + pulseClass + '" type="button" data-ui-action="module-ui-action" data-ui-key="' + html(actionId) + '" data-tooltip="' + html(actionTooltip) + '" onclick="moduleHeaderAction(' + html(JSON.stringify(actionId)) + ')"' + (disabled || action.enabled === false ? ' disabled' : '') + '>' + iconHtml + '</button></div>';
      }).join('');
      const help = tooltip
        ? '<span class="help-icon" data-tooltip="' + html(tooltip) + '" data-tooltip-align="start" aria-label="' + html(tooltip) + '" tabindex="0">?</span>'
        : '';
      const titleHtml = title
        ? '<div class="title-with-help module-ui-schema__title"><h3>' + html(title) + '</h3>' + help + '</div>'
        : '';
      const rowClass = 'module-ui-schema__row' + (title ? '' : ' module-ui-schema__row--no-title') + sizeClass;
      const displayValue = moduleUiCurrentValue(entity, value);
      const valueHtml = displayValue
        ? '<span class="module-ui-schema__value">' + html(displayValue) + '</span>'
        : '';

      if (type === 'separator') {
        return '<div class="module-ui-schema__separator' + sizeClass + '"' + styleAttr + ' data-ui-entity="separator" data-ui-key="' + html(id) + '"></div>';
      }
      if (type === 'help-text') {
        return '<p class="module-ui-schema__help' + sizeClass + '"' + styleAttr + ' data-ui-entity="help-text" data-ui-key="' + html(id) + '">' + html(value || title) + '</p>';
      }
      if (type === 'path-field') {
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="path-field" data-ui-key="' + html(id) + '">' + titleHtml + '<input class="path-field module-ui-schema__path" type="text" readonly value="' + html(displayValue) + '">' + actionHtml + childHtml + '</div>';
      }
      if (type === 'input' || type === 'text-input' || type === 'text-field') {
        const controlValue = moduleUiCurrentValue(entity, value);
        const idJson = JSON.stringify(id);
        const clearDisabled = !controlValue || disabled || readonly;
        const clearEnabled = entity.clear_button === true;
        const clearAttr = clearEnabled ? ' data-clear-button="true"' : '';
        const commitAttr = entity.commit_on_enter === true ? ' data-commit-on-enter="true"' : '';
        const clearHtml = clearEnabled ? '<button class="path-input-clear module-ui-schema__clear" type="button" aria-label="Clear" data-clear-button="true" data-tooltip="Clear" data-tooltip-align="end" onclick="moduleUiSetValue(' + html(idJson) + ', \'\'); renderIntegrationModulePanel(state.snapshot)"' + (clearDisabled ? ' disabled' : '') + '><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>' : '';
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="text-input" data-ui-key="' + html(id) + '">' + titleHtml + '<div class="path-input-shell module-ui-schema__input-shell"><input class="input-box input module-ui-schema__input" type="text" value="' + html(controlValue) + '" placeholder="' + html(placeholder) + '"' + clearAttr + commitAttr + (disabled ? ' disabled' : '') + (readonly ? ' readonly' : '') + ' oninput="moduleUiSetValue(' + html(idJson) + ', this.value)">' + clearHtml + '</div>' + actionHtml + childHtml + '</div>';
      }
      if (type === 'textarea' || type === 'text-area') {
        const controlValue = moduleUiCurrentValue(entity, value);
        const idJson = JSON.stringify(id);
        const clearDisabled = !controlValue || disabled || readonly;
        const clearEnabled = entity.clear_button === true;
        const clearAttr = clearEnabled ? ' data-clear-button="true"' : '';
        const commitAttr = entity.commit_on_enter === true ? ' data-commit-on-enter="true"' : '';
        const textareaRowStyle = moduleUiTextareaRowStyle(entity);
        const textareaControlStyle = moduleUiTextareaControlStyle(entity);
        const textareaRowStyleAttr = textareaRowStyle ? ' style="' + html(textareaRowStyle) + '"' : '';
        const textareaControlStyleAttr = textareaControlStyle ? ' style="' + html(textareaControlStyle) + '"' : '';
        const clearHtml = clearEnabled ? '<button class="path-input-clear module-ui-schema__clear module-ui-schema__clear--textarea" type="button" aria-label="Clear" data-clear-button="true" data-tooltip="Clear" data-tooltip-align="end" onclick="moduleUiSetValue(' + html(idJson) + ', \'\'); renderIntegrationModulePanel(state.snapshot)"' + (clearDisabled ? ' disabled' : '') + '><span class="path-input-clear__glyph" aria-hidden="true">×</span></button>' : '';
        return '<div class="' + rowClass + ' module-ui-schema__row--textarea"' + textareaRowStyleAttr + ' data-ui-entity="textarea" data-ui-key="' + html(id) + '">' + titleHtml + '<div class="path-input-shell module-ui-schema__input-shell module-ui-schema__input-shell--textarea"><textarea class="input-box input module-ui-schema__textarea"' + textareaControlStyleAttr + ' placeholder="' + html(placeholder) + '"' + clearAttr + commitAttr + (disabled ? ' disabled' : '') + (readonly ? ' readonly' : '') + ' oninput="moduleUiSetValue(' + html(idJson) + ', this.value)">' + html(controlValue) + '</textarea>' + clearHtml + '</div>' + actionHtml + childHtml + '</div>';
      }
      if (type === 'select' || type === 'dropdown' || type === 'combo-box') {
        const options = Array.isArray(entity.options) ? entity.options : [];
        const controlValue = moduleUiCurrentValue(entity, value);
        const idJson = JSON.stringify(id);
        const optionsHtml = options.map((option) => {
          const optionValue = text(option.value);
          const optionLabel = moduleUiContextValue(text(option.label, optionValue), context);
          return '<option value="' + html(optionValue) + '"' + (optionValue === controlValue ? ' selected' : '') + '>' + html(optionLabel) + '</option>';
        }).join('');
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="select" data-ui-key="' + html(id) + '">' + titleHtml + '<select class="input-box select module-ui-schema__select"' + (disabled ? ' disabled' : '') + ' onchange="moduleUiSetValue(' + html(idJson) + ', this.value)">' + optionsHtml + '</select>' + actionHtml + childHtml + '</div>';
      }
      if (type === 'switch' || type === 'toggle') {
        const checked = moduleUiCurrentChecked(entity, moduleUiEntityChecked(entity, value));
        const idJson = JSON.stringify(id);
        const onclick = disabled ? '' : ' onclick="moduleUiSetChecked(' + html(idJson) + ', ' + (checked ? 'false' : 'true') + '); renderIntegrationModulePanel(state.snapshot)"';
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="switch" data-ui-key="' + html(id) + '">' + titleHtml + '<button class="input-box switch' + (checked ? ' switch--on' : '') + '" type="button" aria-pressed="' + (checked ? 'true' : 'false') + '" aria-label="' + html(title || id) + '"' + (disabled ? ' disabled' : '') + onclick + '><span class="switch__knob"></span></button>' + actionHtml + childHtml + '</div>';
      }
      if (type === 'tabs' || type === 'tab-view') {
        const options = Array.isArray(entity.options) ? entity.options : [];
        const requestedTab = moduleUiCurrentValue(entity, value).trim();
        const activeTab = options.some((option) => text(option.value).trim() === requestedTab)
          ? requestedTab
          : text(value || options[0]?.value);
        const activeChildren = children.filter((child) => {
          const childPage = text(child?.page).trim();
          return !childPage || childPage === activeTab;
        });
        const tabButtons = options.map((option) => {
          const optionValue = text(option.value);
          const optionLabel = moduleUiContextValue(text(option.label, optionValue), context);
          const selected = optionValue === activeTab;
          return '<button class="tabs__tab' + (selected ? ' tabs__tab--active' : '') + '" type="button" role="tab" aria-selected="' + (selected ? 'true' : 'false') + '"' + (disabled ? ' disabled' : '') + ' onclick="moduleUiSetValue(' + html(JSON.stringify(id)) + ', ' + html(JSON.stringify(optionValue)) + '); renderIntegrationModulePanel(state.snapshot)">' + html(optionLabel) + '</button>';
        }).join('');
        const tabContext = { ...context, page: activeTab };
        const tabBody = activeChildren.map((child) => renderIntegrationUiEntity(child, tabContext)).join('');
        return '<div class="module-ui-schema__tabs' + sizeClass + '"' + styleAttr + ' data-ui-entity="tabs" data-ui-key="' + html(id) + '">' + titleHtml + '<div class="tabs module-ui-schema__tabs-shell"><div class="tabs__list" role="tablist">' + tabButtons + '</div><div class="tabs__body module-ui-schema__tabs-body' + moduleUiScrollClass(entity.scroll) + '">' + tabBody + '</div></div>' + actionHtml + '</div>';
      }
      if (type === 'status' || type === 'status-label') {
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="status-label" data-ui-key="' + html(id) + '">' + titleHtml + '<span class="state-label module-ui-schema__status">' + html(displayValue || title) + '</span>' + actionHtml + childHtml + '</div>';
      }
      if (type === 'value' || type === 'value-label') {
        return '<div class="' + rowClass + '"' + styleAttr + ' data-ui-entity="value-label" data-ui-key="' + html(id) + '">' + titleHtml + valueHtml + actionHtml + childHtml + '</div>';
      }
      if (type === 'progress') {
        const percent = parseProgressPercent(moduleUiCurrentValue(entity, value));
        const compact = entity.compact === true;
        const hideLabel = entity.hide_label === true;
        const label = title || 'Progress';
        const compactLabelClass = compact && !hideLabel ? ' module-ui-schema__progress--compact-labeled' : '';
        const compactTitleHtml = compact && !hideLabel ? titleHtml : '';
        const stages = moduleUiProgressStages(entity);
        const normalized = normalizeProgressStages(stages);
        const meta = progressBarCurrentText(percent, progressBarCurrentStageLabel(normalized, percent));
        return '<div class="module-ui-schema__progress' + sizeClass + compactLabelClass + '"' + styleAttr + ' data-ui-entity="progress-bar" data-ui-key="' + html(id) + '">' + compactTitleHtml + progressBarEntityHtml(label, meta, percent, stages, compact, hideLabel) + actionHtml + childHtml + '</div>';
      }
      if (type === 'button' || type === 'action-button') {
        const actionsLayoutClass = moduleUiActionsLayoutClass(entity);
        return '<div class="module-ui-schema__button-row' + sizeClass + '"' + moduleUiButtonRowStyleAttr(entity) + ' data-ui-entity="button" data-ui-key="' + html(id) + '"><div class="module-ui-schema__button-row-content"' + moduleUiButtonContentStyleAttr(entity) + '><div class="module-ui-schema__actions' + alignClass + actionsLayoutClass + '">' + actionHtml + '</div>' + childHtml + '</div></div>';
      }
      if (type === 'footer') {
        return '<div class="module-ui-schema__footer' + sizeClass + '"' + styleAttr + ' data-ui-entity="footer" data-ui-key="' + html(id) + '">' + valueHtml + actionHtml + childHtml + '</div>';
      }
      if (type === 'table') {
        return '<div class="module-ui-schema__table' + sizeClass + '"' + moduleUiTableShellStyleAttr(entity) + ' data-ui-entity="table" data-ui-key="' + html(id) + '">' + titleHtml + moduleUiTableHtml(entity, value, id, moduleUiScrollClass(entity.scroll), moduleUiTableViewportStyleAttr(entity)) + actionHtml + childHtml + '</div>';
      }
      if (type === 'grid' || type === 'layout-grid') {
        const gridStyle = moduleUiGridStyle(entity);
        const gridStyleAttr = gridStyle ? ' style="' + html(gridStyle) + '"' : '';
        return '<div class="module-ui-schema__panel module-ui-schema__grid' + moduleUiScrollClass(entity.scroll) + sizeClass + '"' + gridStyleAttr + ' data-ui-entity="grid" data-ui-key="' + html(id) + '">' + titleHtml + valueHtml + actionHtml + childHtml + '</div>';
      }
      if (type === 'nested-subpanel') {
        return '<div class="module-ui-schema__nested-subpanel' + moduleUiScrollClass(entity.scroll) + sizeClass + '"' + styleAttr + ' data-ui-entity="nested-subpanel" data-ui-key="' + html(id) + '">' + titleHtml + '<div class="module-ui-schema__nested-subpanel-inner">' + valueHtml + actionHtml + childHtml + '</div></div>';
      }
      const entityClass = type === 'panel' || type === 'subpanel'
        ? 'module-ui-schema__panel' + moduleUiScrollClass(entity.scroll) + sizeClass
        : rowClass;
      return '<div class="' + entityClass + '"' + styleAttr + ' data-ui-entity="' + html(type) + '" data-ui-key="' + html(id) + '">' + titleHtml + valueHtml + actionHtml + childHtml + '</div>';
    }

    function moduleUiEntityType(entity) {
      return text(entity?.entity_type, 'row').replace(/_/g, '-').toLowerCase();
    }

    function moduleUiActionsLayoutClass(entity) {
      const layout = text(entity?.button_layout).trim().toLowerCase();
      if (!layout || layout === 'row') return '';
      if (layout === 'column') return ' module-ui-schema__actions--column';
      return '';
    }

    function moduleUiEntitiesWithoutFooters(entities) {
      return (Array.isArray(entities) ? entities : [])
        .filter((entity) => moduleUiEntityType(entity) !== 'footer')
        .map((entity) => ({
          ...entity,
          children: moduleUiEntitiesWithoutFooters(entity.children)
        }));
    }

    function moduleUiFooterEntities(entities) {
      const footers = [];
      const collect = (items) => {
        (Array.isArray(items) ? items : []).forEach((entity) => {
          if (moduleUiEntityType(entity) === 'footer') {
            footers.push(entity);
          } else {
            collect(entity?.children);
          }
        });
      };
      collect(entities);
      return footers;
    }

    function moduleUiEntityVisibleForContext(entity, context = {}) {
      if (!entity || entity.hidden === true || entity.visible === false) return false;
      const entityPage = text(entity.page).trim();
      return !entityPage || entityPage === text(context?.page, 'main');
    }

    function moduleUiFooterHidesHostBack(entities, context = {}) {
      const visit = (items) => (Array.isArray(items) ? items : []).some((entity) => {
        if (!moduleUiEntityVisibleForContext(entity, context)) return false;
        if (moduleUiEntityType(entity) === 'footer') return entity.hide_host_back_button === true;
        return visit(entity?.children);
      });
      return visit(entities);
    }

    function moduleUiFooterValue(entities, context = {}) {
      for (const entity of Array.isArray(entities) ? entities : []) {
        if (moduleUiEntityType(entity) === 'footer') {
          const value = moduleUiContextValue(entity.value, context).trim();
          if (value) return value;
        }
        const childValue = moduleUiFooterValue(entity.children, context);
        if (childValue) return childValue;
      }
      return '';
    }

    function moduleMonitoringRowLabel(row) {
      if (!row) return '-';
      const enrichment = row.enrichment || {};
      const domain = text(enrichment.domain_name, '-');
      return text(row.process_name, '-')
        + '\t' + text(row.remote_ip, '-')
        + '\t' + domain
        + '\t' + text(row.remote_port, '-')
        + '\t' + text(row.protocol, '-')
        + '\t' + text(row.connection_state, '-') + ' (' + text(row.successful_hits, '0') + '/' + text(row.failed_hits, '0') + ')';
    }

    function latestMonitoringRowsLabel(rows, limit) {
      const sorted = Array.isArray(rows) ? [...rows] : [];
      sorted.sort((left, right) => Number(right?.id || 0) - Number(left?.id || 0));
      const values = sorted.slice(0, limit).map((row) => moduleMonitoringRowLabel(row));
      return values.length
        ? ['App\tIP\tDomain\tPort\tProtocol\tConn'].concat(values).join('\n')
        : 'App\tIP\tDomain\tPort\tProtocol\tConn';
    }

    function moduleUiTableSortState(id) {
      const key = text(id);
      const current = state.moduleUiTableSort && state.moduleUiTableSort[key];
      if (!current || typeof current !== 'object') return { column: null, descending: false };
      const column = Number.isInteger(current.column) ? current.column : null;
      return { column, descending: Boolean(current.descending) };
    }

    function moduleUiTableSortIndicator(id, column) {
      const current = moduleUiTableSortState(id);
      if (current.column !== column) return 'idle';
      return current.descending ? 'desc' : 'asc';
    }

    function setModuleUiTableSort(id, column) {
      const key = text(id);
      const numericColumn = Number(column);
      if (!key || !Number.isInteger(numericColumn)) return;
      const current = moduleUiTableSortState(key);
      let next = { column: numericColumn, descending: false };
      if (current.column === numericColumn && current.descending === false) {
        next = { column: numericColumn, descending: true };
      } else if (current.column === numericColumn && current.descending === true) {
        next = { column: null, descending: false };
      }
      state.moduleUiTableSort = { ...(state.moduleUiTableSort || {}), [key]: next };
      renderIntegrationModulePanel(state.snapshot);
    }

    function moduleUiCompareTableCells(left, right) {
      const leftText = text(left);
      const rightText = text(right);
      const leftNumber = Number(leftText.replace(',', '.'));
      const rightNumber = Number(rightText.replace(',', '.'));
      if (leftText.trim() && rightText.trim() && Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
        return leftNumber - rightNumber;
      }
      return leftText.toLocaleLowerCase().localeCompare(rightText.toLocaleLowerCase());
    }

    function moduleUiSortedTableRows(rows, id) {
      const current = moduleUiTableSortState(id);
      if (current.column === null) return rows;
      return [...rows].sort((left, right) => {
        const result = moduleUiCompareTableCells(left[current.column] || '', right[current.column] || '');
        return current.descending ? -result : result;
      });
    }

    function moduleUiTableColumn(entity, index) {
      const columns = Array.isArray(entity?.table_columns) ? entity.table_columns : [];
      return columns.find((column) => Number(column?.index) === index) || null;
    }

    function moduleUiTableColumnStyle(entity, index) {
      const column = moduleUiTableColumn(entity, index);
      if (!column) return '';
      return moduleUiStyleFromFields(column, [
        ['width', 'width'],
        ['min_width', 'min-width'],
        ['max_width', 'max-width']
      ], false);
    }

    function moduleUiTableColumnAlignClass(entity, index) {
      const value = text(moduleUiTableColumn(entity, index)?.align).trim().toLowerCase();
      if (value === 'center' || value === 'centre' || value === 'middle') return ' module-ui-schema--align-center';
      if (value === 'right' || value === 'end') return ' module-ui-schema--align-right';
      return ' module-ui-schema--align-left';
    }

    function moduleUiTableColumnTextField(entity, index) {
      return Boolean(moduleUiTableColumn(entity, index)?.text_field);
    }

    function moduleUiTableColGroupHtml(entity, count) {
      let columns = '';
      for (let index = 0; index < count; index += 1) {
        const style = moduleUiTableColumnStyle(entity, index);
        columns += '<col' + (style ? ' style="' + html(style) + '"' : '') + '>';
      }
      return columns ? '<colgroup>' + columns + '</colgroup>' : '';
    }

    function moduleUiTableCellHtml(entity, index, cell) {
      const style = moduleUiTableColumnStyle(entity, index);
      const className = moduleUiTableColumnAlignClass(entity, index).trim();
      const content = moduleUiTableColumnTextField(entity, index)
        ? '<input class="path-field module-ui-schema__table-cell-field" type="text" readonly tabindex="0" value="' + html(cell) + '" aria-label="' + html(cell) + '">'
        : html(cell);
      return '<td class="' + html(className) + '"' + (style ? ' style="' + html(style) + '"' : '') + '>' + content + '</td>';
    }

    function moduleUiTableHtml(entity, value, id, scrollClass = '', styleAttr = '') {
      const rows = text(value)
        .split('\n')
        .map((line) => line.trim())
        .filter(Boolean)
        .map((line) => line.split('\t').map((cell) => cell.trim()));
      if (!rows.length) {
        return '<div class="module-ui-schema__table-frame table-wrap"' + styleAttr + '><div class="table-body-wrap module-ui-schema__table-body' + scrollClass + '"><table class="ui-entity-table ui-entity-table--body"><tbody><tr><td>-</td></tr></tbody></table></div></div>';
      }
      const header = rows[0];
      const body = moduleUiSortedTableRows(rows.slice(1), id);
      const colGroupHtml = moduleUiTableColGroupHtml(entity, header.length);
      const headHtml = '<thead><tr>' + header.map((cell, index) => {
        const indicator = moduleUiTableSortIndicator(id, index);
        const icon = indicator === 'asc' ? SORT_ASC_ICON_SRC : (indicator === 'desc' ? SORT_DESC_ICON_SRC : SORT_IDLE_ICON_SRC);
        return '<th class="table-sortable" onclick="setModuleUiTableSort(' + html(JSON.stringify(id)) + ', ' + html(String(index)) + ')" data-sort-state="' + html(indicator) + '"><span class="table-sortable__content"><img class="table-sortable__icon table-sortable__icon--' + html(indicator) + '" src="' + html(icon) + '" alt=""><span class="table-sortable__label">' + html(cell) + '</span></span></th>';
      }).join('') + '</tr></thead>';
      const bodyHtml = '<tbody>' + (body.length
        ? body.map((row) => '<tr>' + row.map((cell, index) => moduleUiTableCellHtml(entity, index, cell)).join('') + '</tr>').join('')
        : '<tr><td colspan="' + html(String(Math.max(1, header.length))) + '">-</td></tr>') + '</tbody>';
      return '<div class="module-ui-schema__table-frame table-wrap"' + styleAttr + '><div class="table-header-wrap"><div class="table-header-scroll"><table class="ui-entity-table ui-entity-table--header">' + colGroupHtml + headHtml + '</table></div><div class="table-header-scrollbar-fill"></div></div><div class="table-body-wrap module-ui-schema__table-body' + scrollClass + '"><table class="ui-entity-table ui-entity-table--body">' + colGroupHtml + bodyHtml + '</table></div></div>';
    }

    function closeModuleOverlays() {
      state.moduleUiActionGeneration += 1;
      const moduleModal = document.getElementById('integration-module-modal');
      const integrationModal = document.getElementById('integration-modal');
      if (moduleModal) moduleModal.hidden = true;
      if (integrationModal) integrationModal.hidden = true;
      state.profileExport.open = false;
      state.moduleDialog = null;
      state.integrationChildReturnToModule = false;
      state.profileExportReturnToModule = false;
      state.selectedIntegrationModuleId = '';
      state.moduleUiPage = 'main';
      state.moduleUiValues = {};
      closeProfileExportAdvancedWizard();
      renderProfileExportDialog();
      renderModuleHostDialog();
      renderModuleHeader();
    }

    async function stopModuleBackgroundAndExit() {
      state.moduleUiActionGeneration += 1;
      const module = selectedIntegrationModule(state.snapshot);
      const moduleId = text(module?.id);
      if (moduleId) {
        try {
          const result = await post('/v1/integrations/background/stop', { module_id: moduleId });
          const stopped = Boolean(result?.stopped);
          pushStatusLine(text(module?.display_name, moduleId) + ': ' + (stopped ? t('modules.stop', 'Stop') : 'Listener already stopped'));
          await loadSnapshot({ background: true });
        } catch (error) {
          pushStatusLine(text(module?.display_name, moduleId) + ': ' + text(error?.message, 'Failed to stop module listener'));
        }
      }
      closeModuleOverlays();
    }

    function renderIntegrationModulePanel(snapshot) {
      const modal = document.getElementById('integration-module-modal');
      if (!modal || !snapshot) return;
      const dialog = modal.querySelector('.integration-module-dialog');
      const schemaTarget = document.getElementById('integration-module-ui-schema');
      const module = selectedIntegrationModule(snapshot);
      if (!module) {
        modal.hidden = true;
        if (dialog) {
          dialog.className = 'modal modal--panel integration-module-dialog';
          dialog.removeAttribute('style');
        }
        if (schemaTarget) schemaTarget.className = 'module-ui-schema';
        state.selectedIntegrationModuleId = '';
        renderModuleHeader();
        return;
      }
      const title = document.getElementById('integration-module-modal-title');
      const help = document.getElementById('integration-module-modal-help');
      const footerActionsTarget = document.getElementById('integration-module-modal-footer-actions');
      const footerValueTarget = document.getElementById('integration-module-modal-footer-value');
      const moduleName = text(module.display_name, text(module.id, t('integration.title', 'Integration')));
      const moduleHelp = text(module.tooltip, moduleName);
      if (title) title.textContent = moduleName;
      if (help) {
        help.dataset.tooltip = moduleHelp;
        help.setAttribute('aria-label', moduleHelp);
      }
      if (schemaTarget) {
        const schema = Array.isArray(module.ui_schema) ? module.ui_schema : [];
        const displayedRows = filteredObservations(snapshot);
        const selectedRows = displayedRows.filter((row) => observationRowSelected(row)).length;
        const allRows = snapshotMonitoringRows(snapshot);
        const moduleBackgroundActive = Boolean(module.background_active);
        const latestSourceRows = moduleBackgroundActive
          ? [...allRows].sort((left, right) => Number(right?.id || 0) - Number(left?.id || 0))
          : [];
        const latestRows = moduleBackgroundActive ? latestMonitoringRowsLabel(allRows, 5) : latestMonitoringRowsLabel([], 5);
        const lastRow = latestSourceRows.length ? moduleMonitoringRowLabel(latestSourceRows[0]) : '-';
        const schemaContext = {
          page: state.moduleUiPage || 'main',
          selectedRows,
          displayedRows: displayedRows.length,
          totalRows: allRows.length,
          moduleBackgroundActive,
          lastRow,
          latestRows,
          latestRowsCount: moduleBackgroundActive ? Math.min(5, allRows.length) : 0
        };
        const bodySchema = moduleUiEntitiesWithoutFooters(schema);
        const dialogLayout = moduleUiDialogLayoutForPage(schema, schemaContext);
        if (dialog) {
          dialog.className = 'modal modal--panel integration-module-dialog' + (dialogLayout ? ' ' + dialogLayout.className : '');
          if (dialogLayout && dialogLayout.style) {
            dialog.setAttribute('style', dialogLayout.style);
          } else {
            dialog.removeAttribute('style');
          }
        }
        schemaTarget.className = 'module-ui-schema' + (dialogLayout ? ' module-ui-schema--dialog-layout' : '');
        schemaTarget.innerHTML = bodySchema.map((entity) => renderIntegrationUiEntity(entity, schemaContext)).join('');
        schemaTarget.hidden = bodySchema.length === 0;
        if (footerActionsTarget) {
          const footers = moduleUiFooterEntities(schema);
          footerActionsTarget.innerHTML = footers
            .map((entity) => renderIntegrationUiEntity(entity, schemaContext))
            .join('');
          footerActionsTarget.hidden = footers.length === 0;
        }
        const backButton = document.getElementById('integration-module-modal-close-button');
        if (backButton) backButton.hidden = moduleUiFooterHidesHostBack(schema, schemaContext);
        if (footerValueTarget) {
          footerValueTarget.textContent = '';
          footerValueTarget.hidden = true;
        }
      }
      renderModuleHeader();
    }

    function backIntegrationModulePanel() {
      if (text(state.moduleUiPage || 'main') !== 'main') {
        state.moduleUiPage = 'main';
        renderIntegrationModulePanel(state.snapshot);
        renderModuleHeader();
        return;
      }
      closeIntegrationModulePanel();
    }

    function renderIntegrationModal(snapshot) {
      const target = document.getElementById('integration-modal-status');
      const modal = document.getElementById('integration-modal');
      if (modal) modal.classList.toggle('module-overlay-backdrop', Boolean(state.integrationChildReturnToModule));
      const title = document.getElementById('integration-modal-title');
      if (title) {
        const module = selectedIntegrationModule(snapshot);
        title.textContent = state.integrationChildReturnToModule
          ? text(module?.provider_title, t('dialog.integration.title', 'Module'))
          : t('dialog.integration.title', 'Module');
      }
      if (!target || !snapshot) return;
      renderIntegrationProviderButtons(snapshot);
      const inputPath = text(state.integrationDir).trim();
      const preview = integrationDialogPreview(snapshot.integration_status, inputPath);
      const provider = '<div class="integration-dialog-status-row"><span class="integration-dialog-status-row__label">' + html(t('integration.provider', 'Integration provider')) + '</span><span class="integration-dialog-status-row__value">' + html(preview.providerName) + '</span></div>';
      const progress = state.integrationDownload.visible
        ? '<div class="integration-dialog-status-row integration-dialog-status-row--progress"><div class="integration-dialog-progress-main">' + progressBarHtml() + '</div><button class="button" id="integration-download-cancel-button" data-ui-action="cancel-integration-download" onclick="cancelIntegrationDownload()" ' + (state.integrationDownload.active ? '' : 'disabled') + '>' + html(t('dialog.integration.cancel_download', 'Cancel')) + '</button></div>'
        : '';
      target.innerHTML =
        '<div class="integration-dialog-status-row"><span class="integration-dialog-status-row__label">' + html(t('dialog.integration.status', 'Status')) + '</span><span class="' + preview.statusClass + '">' + html(preview.statusText) + '</span></div>'
        + provider
        + '<div class="integration-dialog-status-row"><span class="integration-dialog-status-row__label">' + html(t('dialog.integration.primary_path', 'Primary path') + ':') + '</span><input class="integration-dialog-path__input path-field" type="text" readonly value="' + html(preview.repoPath) + '"></div>'
        + '<div class="integration-dialog-status-row"><span class="integration-dialog-status-row__label">' + html(t('dialog.integration.export_path', 'Export path') + ':') + '</span><input class="integration-dialog-path__input path-field" type="text" readonly value="' + html(preview.exportPath) + '"></div>'
        + progress;
      syncClearButtonStates();
    }

    function renderIntegrationProviderButtons(snapshot) {
      const target = document.getElementById('integration-download-actions');
      if (!target) return;
      const providers = snapshot?.integration_providers || [];
      target.innerHTML = providers.map((provider) => {
        const id = text(provider.id);
        const name = text(provider.display_name, id || 'Provider');
        const tooltip = name;
        return '<button class="button" id="integration-download-provider-' + html(id) + '" onclick="downloadIntegration(\'' + html(id) + '\')" data-ui-action="download-integration-provider" data-ui-key="' + html(id) + '" data-tooltip="' + html(tooltip) + '" data-tooltip-align="start" aria-label="' + html(tooltip) + '">' + html(name) + '</button>';
      }).join('');
    }

    function integrationDialogPreview(integration, inputPath) {
      const ready = Boolean(integration?.is_available);
      if (!inputPath) {
        return {
          statusText: ready ? t('dialog.integration.status_configured', 'Module configured') : t('dialog.integration.status_missing', 'Module not specified'),
          statusClass: ready ? 'integration-dialog-status-text integration-dialog-status-text--success' : 'integration-dialog-status-text integration-dialog-status-text--warning',
          repoPath: integration?.repo_root || 'Not configured',
          exportPath: integration?.export_path || 'Configure an integration module before exporting',
          providerName: integration?.provider_name || t('dialog.integration.provider_unset', 'Not specified')
        };
      }
      const samePath = ready && (
        integrationPreviewPathsMatch(inputPath, integration?.repo_root || '')
        || integrationPreviewPathsMatch(inputPath, state.integrationDir || '')
      );
      return {
        statusText: samePath && ready ? t('dialog.integration.status_configured', 'Module configured') : t('dialog.integration.status_entered', 'Path entered'),
        statusClass: samePath && ready ? 'integration-dialog-status-text integration-dialog-status-text--success' : 'integration-dialog-status-text integration-dialog-status-text--warning',
        repoPath: samePath && integration?.repo_root ? integration.repo_root : inputPath,
        exportPath: samePath && integration?.export_path ? integration.export_path : t('dialog.integration.status_entered', 'Path entered'),
        providerName: samePath ? integration?.provider_name || t('dialog.integration.provider_unset', 'Not specified') : t('dialog.integration.provider_unset', 'Not specified')
      };
    }

    function integrationPreviewPathsMatch(left, right) {
      const normalize = (value) => text(value)
        .trim()
        .replace(/[\\/]+$/g, '')
        .replace(/\//g, '\\')
        .toLowerCase();
      const normalizedLeft = normalize(left);
      return Boolean(normalizedLeft) && normalizedLeft === normalize(right);
    }

    function progressBarHtml() {
      const download = state.integrationDownload;
      const percent = download.percent === null || download.percent === undefined
        ? integrationVisualPercent(download.stage || 'idle', null)
        : integrationVisualPercent(download.stage || 'idle', Number(download.percent));
      const meta = integrationProgressMeta(download, percent);
      return progressBarEntityHtml(
        t('dialog.integration.download_progress', 'Module download'),
        meta,
        percent,
        integrationProgressStages()
      );
    }

    function progressBarEntityHtml(label, meta, percent, stages, compact = false, hideLabel = false) {
      const bounded = Math.min(100, Math.max(0, Number(percent) || 0));
      const normalized = normalizeProgressStages(stages);
      const columns = normalized.map((stage) => Math.max(1, Number(stage.width) || 1) + 'fr').join(' ');
      const visualStages = compact ? mergeAdjacentProgressStages(normalized) : normalized;
      const segments = visualStages.map((stage) =>
        '<div class="progress-bar__segment ' + html(stage.className || 'progress-bar__segment--accent') + '" style="' + html(progressSegmentStyle(stage)) + '"></div>'
      ).join('');
      const labels = normalized.map((stage) => '<span>' + html(stage.label) + '</span>').join('');
      const className = compact ? 'progress-bar progress-bar--compact' : 'progress-bar';
      const showLabels = !compact && !hideLabel;
      const currentText = progressBarCurrentText(bounded, progressBarCurrentStageLabel(normalized, bounded));
      const displayMeta = text(meta, currentText);
      const tooltip = displayMeta.includes(currentText) ? displayMeta : displayMeta + ' | ' + currentText;
      const header = showLabels ? '<div class="progress-bar__header"><span class="progress-bar__label">' + html(label) + '</span><span class="progress-bar__meta">' + html(displayMeta) + '</span></div>' : '';
      const stageLabels = showLabels ? '<div class="progress-bar__stages" style="grid-template-columns: ' + html(columns) + ';">' + labels + '</div>' : '';
      return '<div class="' + className + '" data-ui-entity="progress-bar" aria-label="' + html(label) + '" title="' + html(tooltip) + '">' + header + '<div class="progress-bar__track"><div class="progress-bar__segments">' + segments + '</div><div class="progress-bar__remaining" style="width: ' + (100 - bounded) + '%;"></div></div>' + stageLabels + '</div>';
    }

    function integrationProgressStages() {
      return [
        { label: t('dialog.integration.stage_downloading', 'Downloading'), width: 75, className: 'progress-bar__segment--accent' },
        { label: t('dialog.integration.stage_extracting', 'Extracting'), width: 25, className: 'progress-bar__segment--success' }
      ];
    }

    function normalizeProgressStages(stages) {
      const source = Array.isArray(stages) && stages.length
        ? stages
        : [{ label: 'Progress', width: 100, className: 'progress-bar__segment--accent' }];
      const total = source.reduce((sum, stage) => sum + Math.max(1, Number(stage.width) || 1), 0) || 1;
      let used = 0;
      const normalized = source.map((stage) => {
        const width = Math.max(1, Math.floor((Math.max(1, Number(stage.width) || 1) * 100) / total));
        used += width;
        return { label: text(stage.label), width, className: stage.className || 'progress-bar__segment--accent', style: text(stage.style) };
      });
      normalized[normalized.length - 1].width = Math.max(1, normalized[normalized.length - 1].width + 100 - used);
      return normalized;
    }

    function progressSegmentStyle(stage) {
      const width = Math.max(1, Number(stage.width) || 1);
      const stageStyle = text(stage.style).trim();
      return 'width: ' + width + '%;' + (stageStyle ? ' ' + stageStyle : '');
    }

    function mergeAdjacentProgressStages(stages) {
      const merged = [];
      for (const stage of stages) {
        const previous = merged[merged.length - 1];
        if (previous && previous.className === stage.className && text(previous.style) === text(stage.style)) {
          previous.width = Math.min(100, Math.max(1, Number(previous.width) || 1) + Math.max(1, Number(stage.width) || 1));
          if (!text(previous.label)) previous.label = text(stage.label);
        } else {
          merged.push({ ...stage });
        }
      }
      return merged;
    }

    function progressBarCurrentStageLabel(stages, percent) {
      let boundary = 0;
      for (const stage of stages) {
        boundary = Math.min(100, boundary + Math.max(1, Number(stage.width) || 1));
        if (Number(percent) <= boundary) return text(stage.label).trim();
      }
      return text(stages[stages.length - 1]?.label).trim();
    }

    function progressBarCurrentText(percent, stageLabel) {
      const bounded = Math.min(100, Math.max(0, Number(percent) || 0));
      const stage = text(stageLabel).trim();
      return stage ? bounded + '% | ' + stage : bounded + '%';
    }

    function parseProgressPercent(value) {
      const parsed = Number(text(value).trim().replace(/%$/, '').trim());
      if (!Number.isFinite(parsed)) return 0;
      return Math.round(Math.min(100, Math.max(0, parsed)));
    }

    function moduleUiProgressStages(entity) {
      const source = Array.isArray(entity?.progress_stages) ? entity.progress_stages : [];
      if (!source.length) {
        return [{ label: 'Progress', width: 100, className: 'progress-bar__segment--accent' }];
      }
      const stages = [];
      let previous = 0;
      let lastClass = 'progress-bar__segment--accent';
      let lastStyle = '';
      for (const stage of source) {
        const boundary = progressStageBoundary(stage?.percent);
        if (boundary === null || boundary <= previous) continue;
        const width = Math.min(100, boundary) - previous;
        if (width <= 0) continue;
        const color = text(stage?.color);
        const nextStage = {
          label: text(stage?.name),
          width,
          className: progressStageClass(color),
          style: progressStageStyle(color)
        };
        lastClass = nextStage.className;
        lastStyle = nextStage.style;
        stages.push(nextStage);
        previous = Math.min(100, boundary);
        if (previous >= 100) break;
      }
      if (!stages.length) {
        return [{ label: 'Progress', width: 100, className: 'progress-bar__segment--accent' }];
      }
      if (previous < 100) {
        stages.push({ label: '', width: 100 - previous, className: lastClass, style: lastStyle });
      }
      return stages;
    }

    function progressStageBoundary(value) {
      if (value === null || value === undefined) return null;
      const parsed = Number(text(value).trim().replace(/%$/, '').trim());
      if (!Number.isFinite(parsed)) return null;
      return Math.round(Math.min(100, Math.max(0, parsed)));
    }

    function progressStageClass(color) {
      switch (text(color).trim().toLowerCase()) {
        case 'success':
        case 'green':
          return 'progress-bar__segment--success';
        case 'warning':
        case 'yellow':
          return 'progress-bar__segment--warning';
        case 'danger':
        case 'error':
        case 'red':
          return 'progress-bar__segment--danger';
        case 'rust':
        case 'orange':
          return 'progress-bar__segment--rust';
        case 'muted':
        case 'gray':
        case 'grey':
          return 'progress-bar__segment--muted';
        default:
          return 'progress-bar__segment--accent';
      }
    }

    function progressStageStyle(color) {
      const value = text(color).trim();
      return isSafeHexColor(value) ? 'background: ' + value + ';' : '';
    }

    function isSafeHexColor(value) {
      return /^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(text(value).trim());
    }

    function integrationVisualPercent(stage, percent) {
      if (percent !== null && percent !== undefined && !Number.isNaN(percent)) {
        const clamped = Math.min(100, Math.max(0, percent));
        if (stage === 'downloading') return Math.floor((clamped * 75) / 100);
        if (stage === 'extracting') return 75 + Math.floor((clamped * 25) / 100);
        return clamped;
      }
      return {
        preparing: 0,
        downloading: 0,
        extracting: 75,
        detecting: 100,
        installing: 100,
        complete: 100
      }[stage] || 0;
    }

    function integrationProgressMeta(download, percent) {
      const stage = integrationStageLabel(download.stage || 'idle');
      if (download.percent === null || download.percent === undefined) {
        if (download.downloadedBytes !== null && download.downloadedBytes !== undefined) {
          return stage + ' | ' + Math.floor(Number(download.downloadedBytes) / 1024) + ' ' + t('dialog.integration.progress_kb', 'KB');
        }
        return stage;
      }
      if (download.stage === 'extracting' && download.extractedEntries !== null && download.totalEntries !== null) {
        return percent + '% | ' + download.extractedEntries + '/' + download.totalEntries + ' ' + t('dialog.integration.progress_entries', 'entries');
      }
      if (download.downloadedBytes !== null && download.downloadedBytes !== undefined) {
        const downloadedKb = Math.floor(Number(download.downloadedBytes) / 1024);
        if (download.totalBytes !== null && download.totalBytes !== undefined) {
          return percent + '% | ' + downloadedKb + '/' + Math.floor(Number(download.totalBytes) / 1024) + ' ' + t('dialog.integration.progress_kb', 'KB');
        }
        return percent + '% | ' + downloadedKb + ' ' + t('dialog.integration.progress_kb', 'KB');
      }
      return percent + '% | ' + stage;
    }

    function integrationStageLabel(stage) {
      const labels = {
        preparing: t('dialog.integration.stage_preparing', 'Preparing'),
        downloading: t('dialog.integration.stage_downloading', 'Downloading'),
        extracting: t('dialog.integration.stage_extracting', 'Extracting'),
        detecting: t('dialog.integration.stage_detecting', 'Detecting'),
        installing: t('dialog.integration.stage_installing', 'Installing'),
        complete: t('dialog.integration.stage_complete', 'Complete'),
        failed: t('dialog.integration.stage_failed', 'Failed'),
        idle: t('dialog.integration.stage_idle', 'Idle')
      };
      return labels[stage] || text(stage);
    }

    function cancelIntegrationDownload() {
      if (!state.integrationDownload.active) return;
      state.integrationDownload.active = false;
      state.integrationDownload.visible = true;
      state.integrationDownload.cancelled = true;
      state.integrationDownload.generation = (state.integrationDownload.generation || 0) + 1;
      renderIntegrationModal(state.snapshot);
    }

    function delay(ms) {
      return new Promise((resolve) => setTimeout(resolve, ms));
    }

    function applyIntegrationDownloadProgress(progress, generation) {
      const previousStage = state.integrationDownload.stage;
      state.integrationDownload = {
        active: Boolean(progress.active),
        visible: Boolean(progress.active) || (progress.stage || 'idle') !== 'idle',
        cancelled: false,
        generation,
        stage: progress.stage || 'idle',
        percent: progress.percent ?? null,
        downloadedBytes: progress.downloaded_bytes ?? null,
        totalBytes: progress.total_bytes ?? null,
        extractedEntries: progress.extracted_entries ?? null,
        totalEntries: progress.total_entries ?? null,
        message: progress.message || null,
        repoRoot: progress.repo_root || null
      };
      if (state.integrationDownload.stage !== previousStage) {
        pushStatusLine(integrationProgressFooterLine(state.integrationDownload));
      }
      renderIntegrationModal(state.snapshot);
    }

    function integrationProgressFooterLine(download) {
      const label = t('dialog.integration.download_progress', 'Module download');
      const stage = integrationStageLabel(download.stage || 'idle');
      if (download.message) return label + ': ' + stage + ' - ' + download.message;
      if (download.stage === 'complete' && download.repoRoot) return label + ': ' + stage + ' - ' + download.repoRoot;
      return label + ': ' + stage;
    }

    async function pollIntegrationDownloadProgress(generation) {
      while (state.integrationDownload.generation === generation && !state.integrationDownload.cancelled) {
        try {
          const progress = await api('/v1/integrations/download-progress');
          if (state.integrationDownload.generation !== generation || state.integrationDownload.cancelled) return;
          applyIntegrationDownloadProgress(progress, generation);
          if (!progress.active && ['complete', 'failed'].includes(progress.stage)) return;
        } catch (_error) {
          return;
        }
        await delay(200);
      }
    }

    async function openIntegrationModulePanel(moduleId) {
      if (state.moduleOrderEditing) return;
      const modal = document.getElementById('integration-module-modal');
      const module = enabledIntegrationModules(state.snapshot)
        .find((candidate) => text(candidate.id) === text(moduleId));
      if (!modal || !module) return;
      state.selectedIntegrationModuleId = text(module.id);
      state.moduleUiPage = 'main';
      state.moduleUiValues = {};
      if (!state.integrationDir && state.snapshot?.integration_status?.repo_root) {
        state.integrationDir = state.snapshot.integration_status.repo_root;
      }
      modal.hidden = false;
      renderIntegrationModulePanel(state.snapshot);
      renderModuleHeader();
      const openActionId = moduleOpenActionId(module);
      if (openActionId) {
        await moduleHeaderAction(openActionId);
      }
    }

    function closeIntegrationModulePanel() {
      const modal = document.getElementById('integration-module-modal');
      if (modal) modal.hidden = true;
      state.selectedIntegrationModuleId = '';
      state.moduleUiPage = 'main';
      state.moduleUiValues = {};
      state.integrationChildReturnToModule = false;
      state.profileExportReturnToModule = false;
      renderModuleHeader();
    }

    function returnToIntegrationModulePanel() {
      const modal = document.getElementById('integration-module-modal');
      if (modal) modal.hidden = false;
      renderIntegrationModulePanel(state.snapshot);
      renderModuleHeader();
    }

    function openIntegrationDialog() {
      const modal = document.getElementById('integration-modal');
      const input = document.getElementById('integration-dir');
      state.integrationDownload = { active: false, visible: false, cancelled: false, generation: state.integrationDownload.generation || 0, stage: 'idle', percent: null, downloadedBytes: null, totalBytes: null, extractedEntries: null, totalEntries: null, message: null, repoRoot: null };
      if (input && !input.value && state.snapshot?.integration_status?.repo_root) {
        input.value = state.snapshot.integration_status.repo_root;
      }
      state.integrationDir = input?.value.trim() || '';
      if (modal) modal.hidden = false;
      renderIntegrationModal(state.snapshot);
      renderModuleHeader();
      if (input) input.focus();
    }

    function clearExePath() {
      const input = document.getElementById('exe-path');
      if (input) {
        input.value = '';
        input.focus();
      }
      syncClearButtonStates();
    }

    function clearIntegrationPath() {
      const input = document.getElementById('integration-dir');
      if (input) {
        input.value = '';
        state.integrationDir = '';
        pulseInput(input);
        renderIntegrationModal(state.snapshot);
        input.focus();
      }
      syncClearButtonStates();
    }

    function browseIntegrationFolder() {
      const input = document.getElementById('integration-dir');
      const currentPath = input?.value.trim()
        || state.integrationDir
        || state.snapshot?.integration_status?.repo_root
        || '';
      openServerFilePicker({
        intent: 'integration_folder',
        mode: 'any',
        title: t('dialog.integration.title', 'Module'),
        help: t('dialog.integration.path_help', 'Module path for later data export.'),
        initialPath: currentPath
      });
    }

    function closeIntegrationDialog() {
      const modal = document.getElementById('integration-modal');
      if (modal) modal.hidden = true;
      const shouldReturnToModule = Boolean(state.integrationChildReturnToModule);
      state.integrationChildReturnToModule = false;
      if (shouldReturnToModule) returnToIntegrationModulePanel();
      renderModuleHeader();
    }

    async function useIntegrationFolder() {
      const input = document.getElementById('integration-dir');
      state.integrationDir = input?.value.trim() || '';
      pulseInput(input);
      renderIntegrationModal(state.snapshot);
      const shouldReturnToModule = Boolean(state.integrationChildReturnToModule);
      await post('/v1/integrations/configure', {
        module_id: selectedIntegrationModuleIdForRequest(),
        repo_root: state.integrationDir || null
      });
      state.integrationChildReturnToModule = false;
      const modal = document.getElementById('integration-modal');
      if (modal) modal.hidden = true;
      await loadSnapshot();
      if (shouldReturnToModule) returnToIntegrationModulePanel();
      renderModuleHeader();
    }

    function applyIntegrationPathOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      const input = event.currentTarget || document.getElementById('integration-dir');
      state.integrationDir = input?.value.trim() || '';
      pulseInput(input);
      renderIntegrationModal(state.snapshot);
    }

    function observationMatchesFilters(snapshot, row, options = {}) {
      const filters = currentFilters();
      const ignored = snapshot?.ignored_addresses || [];
      if (ignored.some((rule) => addressIgnored(row.remote_ip, rule.address_pattern))) return false;
      if (!options.ignoreObservationFilter) {
        const selected = observationRowSelected(row);
        if (filters.observation_filter === 'Unconfirmed' && selected) return false;
        if (filters.observation_filter === 'Confirmed' && !selected) return false;
        if (filters.observation_filter === 'Success' && !(row.successful_hits > 0 || ['Established', 'Closing'].includes(row.connection_state))) return false;
        if (filters.observation_filter === 'Exported') return false;
        if (filters.observation_filter === 'Failed' && !(row.failed_hits > 0 || ['Attempting', 'Failed'].includes(row.connection_state))) return false;
      }
      if (!options.ignoreAppFilter) {
        const appSearch = text(filters.app_search).trim().toLowerCase();
        if (appSearch) {
          const appText = [appName(snapshot, row.tracked_app_id, row), row.process_name]
            .map((value) => text(value).toLowerCase())
            .join(' ');
          if (!appText.includes(appSearch)) return false;
        }
      }
      if (!options.ignorePortFilter) {
        const portSearch = text(filters.port_search).trim().toLowerCase();
        if (portSearch && !text(row.remote_port).toLowerCase().includes(portSearch)) return false;
      }
      if (!options.ignoreProtocolFilter) {
        const protocolSearch = text(filters.protocol).trim().toLowerCase();
        if (protocolSearch && protocolSearch !== 'all' && text(row.protocol).toLowerCase() !== protocolSearch) return false;
      }
      if (!options.ignoreIpFilter) {
        const ipSearch = text(filters.ip_search).trim().toLowerCase();
        if (ipSearch && !text(row.remote_ip).toLowerCase().includes(ipSearch)) return false;
      }
      if (!options.ignoreDomainFilter) {
        const domainSearch = text(filters.domain_search).trim().toLowerCase();
        if (!domainFilterMatches(enrichmentDomainText(row.enrichment), domainSearch)) return false;
      }
      if (!options.ignorePublicFilter && observationIpIsPublic(row) !== Boolean(filters.public_ip)) return false;
      return true;
    }

    function observationIpIsPublic(row) {
      return ipAddressIsPublic(text(row?.remote_ip));
    }

    function observationIsSuccessful(row) {
      return Number(row?.successful_hits || 0) > 0 || ['Established', 'Closing'].includes(text(row?.connection_state));
    }

    function observationIsFailed(row) {
      return Number(row?.failed_hits || 0) > 0 || ['Attempting', 'Failed'].includes(text(row?.connection_state));
    }

    function domainFilterMatches(value, pattern) {
      const current = text(value).trim().toLowerCase();
      const query = text(pattern).trim().toLowerCase();
      if (!query) return true;
      if (!query.includes('*')) return current.includes(query);
      return wildcardPatternMatches(current, query);
    }

    function wildcardPatternMatches(value, pattern) {
      if (pattern === '*') return true;
      const parts = pattern.split('*');
      const anchoredStart = !pattern.startsWith('*');
      const anchoredEnd = !pattern.endsWith('*');
      let position = 0;
      for (let index = 0; index < parts.length; index += 1) {
        const part = parts[index];
        if (!part) continue;
        if (index === 0 && anchoredStart) {
          if (!value.slice(position).startsWith(part)) return false;
          position += part.length;
          continue;
        }
        const found = value.slice(position).indexOf(part);
        if (found < 0) return false;
        position += found + part.length;
      }
      if (anchoredEnd) {
        const last = [...parts].reverse().find((part) => part);
        if (last && !value.endsWith(last)) return false;
      }
      return true;
    }

    function ipAddressIsPublic(value) {
      const ip = text(value).trim();
      if (!ip) return false;
      if (ip.includes(':')) return ipv6AddressIsPublic(ip);
      return ipv4AddressIsPublic(ip);
    }

    function ipv4AddressIsPublic(value) {
      const parts = value.split('.').map((part) => Number(part));
      if (parts.length !== 4 || parts.some((part) => !Number.isInteger(part) || part < 0 || part > 255)) return false;
      const [a, b] = parts;
      if (a === 0 || a === 10 || a === 127 || a >= 224) return false;
      if (a === 100 && b >= 64 && b <= 127) return false;
      if (a === 169 && b === 254) return false;
      if (a === 172 && b >= 16 && b <= 31) return false;
      if (a === 192 && b === 0) return false;
      if (a === 192 && b === 168) return false;
      if (a === 198 && (b === 18 || b === 19)) return false;
      if (a >= 240) return false;
      return true;
    }

    function ipv6AddressIsPublic(value) {
      const normalized = value.toLowerCase();
      if (normalized === '::' || normalized === '::1') return false;
      if (normalized.startsWith('fc') || normalized.startsWith('fd')) return false;
      if (normalized.startsWith('fe8') || normalized.startsWith('fe9') || normalized.startsWith('fea') || normalized.startsWith('feb')) return false;
      if (normalized.startsWith('ff')) return false;
      if (normalized.startsWith('2001:db8')) return false;
      return true;
    }

    function filteredObservations(snapshot, options = {}) {
      const current = snapshot || state.snapshot;
      const rows = (current?.observed_endpoints || []).filter((row) => observationMatchesFilters(current, row, options));
      return sortObservations(current, rows);
    }

    function observationStateRank(row) {
      if (row.is_exported) return 2;
      if (observationRowSelected(row)) return 1;
      return 0;
    }

    function sortObservations(snapshot, rows) {
      const key = state.observationSortKey || 'last_seen';
      const sorted = rows.slice().sort((left, right) => {
        let ordering = 0;
        if (key === 'app') ordering = text(appName(snapshot, left.tracked_app_id, left)).localeCompare(text(appName(snapshot, right.tracked_app_id, right)));
        else if (key === 'ip') ordering = text(left.remote_ip).localeCompare(text(right.remote_ip));
        else if (key === 'domain') ordering = text(enrichmentDomainText(left.enrichment)).localeCompare(text(enrichmentDomainText(right.enrichment)));
        else if (key === 'port') ordering = Number(left.remote_port || 0) - Number(right.remote_port || 0);
        else if (key === 'protocol') ordering = text(left.protocol).localeCompare(text(right.protocol));
        else if (key === 'connection') ordering = text(left.connection_state).localeCompare(text(right.connection_state));
        else if (key === 'first_seen') ordering = Number(left.first_seen_ms || 0) - Number(right.first_seen_ms || 0);
        else if (key === 'last_seen') ordering = Number(left.last_seen_ms || 0) - Number(right.last_seen_ms || 0);
        else if (key === 'hits') ordering = Number(left.hits || 0) - Number(right.hits || 0);
        else if (key === 'state') ordering = observationStateRank(left) - observationStateRank(right);
        if (ordering === 0) ordering = Number(left.id || 0) - Number(right.id || 0);
        return state.observationSortDescending ? -ordering : ordering;
      });
      return sorted;
    }

    function updateObservationSortIndicators() {
      const ids = ['app', 'ip', 'domain', 'port', 'proto', 'conn', 'hits', 'first-seen', 'last-seen'];
      ids.forEach((id) => {
        const node = document.getElementById('table-' + id + '-sort-icon');
        if (!node) return;
        node.className = 'table-sortable__icon table-sortable__icon--idle';
        node.src = SORT_IDLE_ICON_SRC;
      });
      const current = state.observationSortKey;
      if (!current) return;
      const map = {
        app: 'app',
        ip: 'ip',
        domain: 'domain',
        port: 'port',
        protocol: 'proto',
        connection: 'conn',
        hits: 'hits',
        first_seen: 'first-seen',
        last_seen: 'last-seen'
      };
      const node = document.getElementById('table-' + map[current] + '-sort-icon');
      if (!node) return;
      node.className = 'table-sortable__icon table-sortable__icon--' + (state.observationSortDescending ? 'desc' : 'asc');
      node.src = state.observationSortDescending ? SORT_DESC_ICON_SRC : SORT_ASC_ICON_SRC;
    }

    function applyObservationFilters() {
      renderHeaderFilters(state.snapshot);
      renderObservations(state.snapshot);
      updateObservationSortIndicators();
    }

    function addressIgnored(address, pattern) {
      const current = text(address).toLowerCase();
      const rule = text(pattern).trim().toLowerCase();
      if (current === rule) return true;
      if (rule === '127.0.0.0/8' && current.startsWith('127.')) return true;
      if (rule === '::1/128' && current === '::1') return true;
      if (rule.endsWith('/32')) return current === rule.slice(0, -3);
      if (rule.endsWith('/128')) return current === rule.slice(0, -4);
      return false;
    }

    function appName(snapshot, id, row) {
      const app = (snapshot.tracked_apps || []).find((item) => item.id === id);
      if (app) return trackedAppDisplayName(app);
      return normalizeDisplayName(row?.process_name || 'Imported app');
    }

    function trackedAppDisplayName(app) {
      return normalizeDisplayName(app?.display_name || app?.process_name || 'Tracked app');
    }

    function normalizeDisplayName(value) {
      const trimmed = text(value).trim();
      if (!trimmed) return 'Tracked app';
      const withoutExe = trimmed.toLowerCase().endsWith('.exe') ? trimmed.slice(0, -4) : trimmed;
      if (withoutExe.toLowerCase() === 'chrome') return 'Chrome';
      return withoutExe.charAt(0).toUpperCase() + withoutExe.slice(1);
    }

    function iconUrl(id) {
      return '/v1/tracked-apps/' + encodeURIComponent(id) + '/icon';
    }

    function formatTime(ms) {
      return ms ? new Date(ms).toLocaleString() : 'n/a';
    }

    async function setObservationFilter(value) {
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.observation_filter = value || 'All';
        renderCloudRows();
        return;
      }
      await updateFilters({ observation_filter: value || 'All' });
    }

    async function setAppFilter(value) {
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.app_search = value || '';
        renderCloudRows();
        return;
      }
      await updateFilters({ app_search: value || '' });
    }

    async function setPortFilter(value) {
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.port_search = value || '';
        renderCloudRows();
        return;
      }
      await updateFilters({ port_search: value || '' });
    }

    async function setDomainFilter(value) {
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.domain_search = value || '';
        renderCloudRows();
        return;
      }
      await updateFilters({ domain_search: value || '' });
    }

    async function togglePublicIpFilter() {
      const current = Boolean(state.snapshot?.filters?.public_ip);
      await updateFilters({ public_ip: !current });
    }

    async function applyPortFilterOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      const input = event.currentTarget || document.getElementById('port-filter');
      pulseInput(input);
      await setPortFilter(input ? input.value || '' : '');
    }

    async function setProtocolFilter(value) {
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.protocol = value || 'All';
        renderCloudRows();
        return;
      }
      await updateFilters({ protocol: value || 'All' });
    }

    function pulseInput(input) {
      if (!input) return;
      input.classList.remove('input--apply-pulse');
      void input.offsetWidth;
      input.classList.add('input--apply-pulse');
      window.setTimeout(() => input.classList.remove('input--apply-pulse'), 220);
    }

    async function applyIpFilterOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      const input = event.currentTarget || document.getElementById('ip-filter');
      pulseInput(input);
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.ip_search = input ? input.value || '' : '';
        renderCloudRows();
        return;
      }
      await updateFilters({ ip_search: input ? input.value || '' : '' });
    }

    async function applyDomainFilterOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      const input = event.currentTarget || document.getElementById('domain-filter');
      pulseInput(input);
      await setDomainFilter(input ? input.value || '' : '');
    }

    async function applyCloudTextFilterOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      pulseInput(event.currentTarget);
      state.cloud.appFilters = readCloudAppFilterInputs();
      await refreshCloudApps();
    }

    async function clearCloudImportFilter(id) {
      const input = document.getElementById(id);
      if (!input) return;
      input.value = '';
      pulseInput(input);
      syncClearButtonStates();
      state.cloud.appFilters = readCloudAppFilterInputs();
      await refreshCloudApps();
    }

    async function setCloudImportVisibilityScope(value) {
      const normalized = text(value).toLowerCase();
      const scope = ['all', 'public', 'private'].includes(normalized) ? normalized : 'all';
      state.cloud.importVisibilityScope = scope;
      state.cloud.selectedAppId = '';
      state.cloud.selectedAppName = '';
      state.cloud.rows = [];
      state.cloud.selectedRows = new Set();
      renderCloudRows();
      await refreshCloudApps();
    }

    function setCloudMineFilterActive(active) {
      state.cloud.scopeMine = Boolean(active);
      const button = document.getElementById('cloud-import-scope-mine');
      if (button) {
        button.classList.toggle('button--cloud-active', state.cloud.scopeMine);
        button.setAttribute('aria-pressed', state.cloud.scopeMine ? 'true' : 'false');
      }
    }

    async function toggleCloudMineFilter() {
      if (!state.cloud.auth?.authenticated) {
        setCloudMineFilterActive(false);
        return;
      }
      const active = !state.cloud.scopeMine;
      if (active) {
        try {
          await loadCloudMyApps();
        } catch (error) {
          setStatus(text(error?.message || error), true);
          setCloudMineFilterActive(false);
          await refreshCloudApps();
          return;
        }
      }
      setCloudMineFilterActive(active);
      refreshCloudApps();
    }

    function readCloudAppFilterInputs() {
      return {
        app: text(document.getElementById('cloud-import-app-filter')?.value).trim(),
        publisher: text(document.getElementById('cloud-import-company-filter')?.value).trim(),
        source: text(document.getElementById('cloud-import-author-filter')?.value).trim()
      };
    }

    function cloudAppSearchParams() {
      const params = new URLSearchParams();
      const filters = state.cloud.appFilters || {};
      const app = text(filters.app).trim();
      const publisher = text(filters.publisher).trim();
      const source = text(filters.source).trim();
      if (app.length >= 2) params.set('query', app);
      if (publisher.length >= 2) params.set('publisher', publisher);
      if (source.length >= 2) params.set('source', source);
      return params;
    }

    async function refreshCloudApps() {
      const params = cloudAppSearchParams();
      const mineActive = Boolean(state.cloud.scopeMine);
      state.cloud.selectedAppId = '';
      state.cloud.selectedAppName = '';
      state.cloud.rows = [];
      state.cloud.selectedRows = new Set();
      if ([...params.keys()].length === 0 && !mineActive) {
        state.cloud.apps = [];
        renderCloudApps();
        renderCloudRows();
        return;
      }
      if ([...params.keys()].length === 0 && mineActive) {
        state.cloud.apps = [];
        try {
          await loadCloudMyApps();
        } catch (_error) {
          state.cloud.myApps = [];
        }
        renderCloudApps();
        renderCloudRows();
        return;
      }
      try {
        const response = await api('/v1/cloud/apps?' + params.toString());
        state.cloud.apps = Array.isArray(response.items) ? response.items : [];
        try {
          await loadCloudMyApps();
        } catch (_error) {
          state.cloud.myApps = [];
        }
        renderCloudApps();
        renderCloudRows();
      } catch (error) {
        state.cloud.apps = [];
        renderCloudApps();
        setStatus(text(error?.message || error), true);
      }
    }

    async function refreshCloudQuota() {
      try {
        const response = await api('/v1/cloud/quota');
        state.cloud.quota = response || null;
        state.cloud.serviceOnline = true;
        state.cloud.serviceMessage = '';
      } catch (error) {
        state.cloud.serviceOnline = false;
        state.cloud.serviceMessage = text(error?.message || error);
        setStatus(state.cloud.serviceMessage, true);
      }
      renderCloudPanel();
    }

    async function refreshCloudAuthState() {
      try {
        const response = await api('/v1/cloud/auth-state');
        state.cloud.auth = response || null;
        state.cloud.exportAuthor = text(response?.nickname);
        state.cloud.nicknameStatus = state.cloud.exportAuthor ? 'accepted' : 'idle';
      } catch (_error) {
        state.cloud.auth = null;
        state.cloud.nicknameStatus = 'idle';
      }
      renderCloudExportAuth();
      await refreshCloudMyApps();
    }

    async function loadCloudMyApps() {
      try {
        const auth = await api('/v1/cloud/auth-state');
        state.cloud.auth = auth || null;
      } catch (_error) {
        state.cloud.auth = null;
      }
      if (!state.cloud.auth?.authenticated) {
        state.cloud.myApps = [];
        return;
      }
      const response = await api('/v1/cloud/my-apps');
      state.cloud.myApps = Array.isArray(response.items) ? response.items : [];
    }

    async function refreshCloudMyApps() {
      try {
        await loadCloudMyApps();
      } catch (error) {
        state.cloud.myApps = [];
        setStatus(text(error?.message || error), true);
      }
      renderCloudApps();
      renderCloudExportRows();
    }

    function cloudExportNicknameStatusText() {
      if (state.cloud.nicknameStatus === 'invalid') {
        return t('dialog.cloud_sync.nickname_invalid', 'Nickname is not allowed');
      }
      if (state.cloud.nicknameStatus === 'taken') {
        return t('dialog.cloud_sync.nickname_taken', 'Nickname is already taken, or sign in to confirm it');
      }
      if (state.cloud.nicknameStatus === 'accepted') {
        return t('dialog.cloud_sync.nickname_accepted', 'Nickname accepted');
      }
      return '';
    }

    function cloudExportNicknameStatusClass() {
      const base = 'cloud-web-nickname-status';
      if (state.cloud.nicknameStatus === 'invalid') return base + ' cloud-web-nickname-status--invalid';
      if (state.cloud.nicknameStatus === 'taken') return base + ' cloud-web-nickname-status--taken';
      if (state.cloud.nicknameStatus === 'accepted') return base + ' cloud-web-nickname-status--accepted';
      return base;
    }

    function renderCloudExportAuth() {
      const auth = state.cloud.auth || {};
      const authenticated = Boolean(auth.authenticated);
      const authButton = document.getElementById('cloud-export-auth-button');
      const signOutButton = document.getElementById('cloud-export-sign-out-button');
      const authUser = document.getElementById('cloud-export-auth-user');
      const nickname = document.getElementById('cloud-export-nickname');
      const nicknameStatus = document.getElementById('cloud-export-nickname-status');
      const identifier = document.getElementById('cloud-export-identifier');
      const identifierLabel = document.getElementById('cloud-export-identifier-label');
      const identifierValue = document.getElementById('cloud-export-identifier-value');
      const footerAuth = document.getElementById('cloud-export-footer-auth');
      const footerAuthLabel = document.getElementById('cloud-export-footer-auth-label');
      const footerAuthValue = document.getElementById('cloud-export-footer-auth-value');
      const authLabel = authenticated
        ? text(auth.authorization_label, t('dialog.cloud_sync.authorization_active', 'Session active'))
        : '-';
      if (authButton) {
        authButton.textContent = t('dialog.cloud_sync.authorization', 'Authorization');
        authButton.disabled = true;
        authButton.dataset.tooltip = t('dialog.cloud_sync.web_auth_desktop_only', 'Use the desktop application to sign in to Google.');
      }
      if (signOutButton) {
        signOutButton.textContent = t('dialog.cloud_sync.sign_out', 'Sign out');
        signOutButton.disabled = !authenticated;
      }
      if (authUser) {
        authUser.textContent = authLabel;
        authUser.dataset.tooltip = t('dialog.cloud_sync.authorization_tooltip', 'Your name and email are not stored in the app or in the cloud.');
      }
      if (identifierLabel) identifierLabel.textContent = t('dialog.cloud_sync.identifier', 'Identifier') + ':';
      if (identifierValue) identifierValue.textContent = text(auth.client_identifier, '-');
      if (identifier) {
        identifier.dataset.tooltip = text(auth.client_identifier)
          ? t('dialog.cloud_sync.identifier_copy_tooltip', 'Click to copy identifier')
          : t('dialog.cloud_sync.identifier_pending', 'Identifier is not ready yet');
      }
      if (footerAuthLabel) footerAuthLabel.textContent = t('dialog.cloud_sync.authorization', 'Authorization') + ':';
      if (footerAuthValue) footerAuthValue.textContent = authLabel;
      if (footerAuth) footerAuth.dataset.tooltip = t('dialog.cloud_sync.authorization_tooltip', 'Your name and email are not stored in the app or in the cloud.');
      if (nickname && document.activeElement !== nickname) {
        nickname.value = state.cloud.exportAuthor || text(auth.nickname);
      }
      const privateUpload = document.getElementById('cloud-export-private-upload');
      const privateLabel = document.getElementById('cloud-export-private-label');
      const privateText = document.getElementById('cloud-export-private-text');
      if (privateUpload) privateUpload.checked = Boolean(state.cloud.uploadPrivate);
      if (privateLabel) privateLabel.dataset.tooltip = t('dialog.cloud_sync.private_upload_tooltip', 'Only your signed-in account can download this upload.');
      if (privateText) privateText.textContent = t('dialog.cloud_sync.private_upload', 'Private upload');
      if (nicknameStatus) {
        nicknameStatus.className = cloudExportNicknameStatusClass();
        nicknameStatus.textContent = cloudExportNicknameStatusText();
      }
      renderCloudExportRows();
    }

    function toggleCloudPrivateUpload(event) {
      state.cloud.uploadPrivate = Boolean(event?.currentTarget?.checked);
      renderCloudExportAuth();
    }

    async function copyCloudIdentifier(event) {
      event?.stopPropagation();
      const value = text(state.cloud.auth?.client_identifier);
      if (!value) return;
      await copyTextToClipboard(value);
      setFooterMessage(t('dialog.cloud_sync.identifier_copied', 'Identifier copied'));
    }

    function updateCloudExportNicknameDraft() {
      const input = document.getElementById('cloud-export-nickname');
      state.cloud.exportAuthor = text(input?.value);
      state.cloud.nicknameStatus = state.cloud.exportAuthor.trim() ? 'idle' : 'invalid';
      renderCloudExportAuth();
    }

    async function applyCloudExportNicknameOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      const input = event.currentTarget || document.getElementById('cloud-export-nickname');
      pulseInput(input);
      const nickname = text(input?.value).trim();
      try {
        const response = await post('/v1/cloud/nickname', { nickname }, { skipSnapshotRefresh: true });
        state.cloud.exportAuthor = text(response?.nickname, nickname);
        state.cloud.nicknameStatus = text(response?.status) === 'accepted' ? 'accepted' : 'idle';
      } catch (error) {
        const message = text(error?.message || error);
        state.cloud.nicknameStatus = message.includes('nickname_taken') ? 'taken' : 'invalid';
      }
      renderCloudExportAuth();
    }

    async function signOutCloudWeb() {
      try {
        await post('/v1/cloud/sign-out', {}, { skipSnapshotRefresh: true });
        state.cloud.auth = { authenticated: false, authorization_label: null, nickname: state.cloud.exportAuthor };
        setFooterMessage(t('dialog.cloud_sync.signed_out', 'Signed out of the cloud account'));
      } catch (error) {
        setStatus(text(error?.message || error), true);
      }
      renderCloudExportAuth();
      renderCloudExportRows();
    }

    function cloudAppAuthorsLabel(app) {
      const authors = Array.isArray(app?.authors) ? app.authors.filter(Boolean) : [];
      return authors.length ? authors.join(', ') : '-';
    }

    function cloudAppTextField(value) {
      const textValue = text(value, '-');
      const escapedValue = html(textValue);
      return '<input class="path-field cloud-sync-app-field" type="text" readonly value="' + escapedValue + '" aria-label="' + escapedValue + '" data-tooltip="' + escapedValue + '" data-tooltip-align="start">';
    }

    function cloudVisibleApps() {
      const scope = text(state.cloud.importVisibilityScope, 'all').toLowerCase();
      const mineActive = Boolean(state.cloud.scopeMine);
      const filters = state.cloud.appFilters || {};
      const appQuery = text(filters.app).trim().toLowerCase();
      const publisherQuery = text(filters.publisher).trim().toLowerCase();
      const sourceQuery = text(filters.source).trim().toLowerCase();
      if (!mineActive && appQuery.length < 2 && publisherQuery.length < 2 && sourceQuery.length < 2) return [];
      if (scope === 'public' && !mineActive) return state.cloud.apps.slice();
      const appsById = new Map();
      if (!mineActive && (scope === 'all' || scope === 'public')) {
        for (const app of state.cloud.apps) appsById.set(text(app.app_id), { ...app });
      }
      for (const item of state.cloud.myApps || []) {
        const itemVisibility = text(item.visibility, 'public').toLowerCase();
        const includeForScope = scope === 'all' || scope === itemVisibility;
        if (!includeForScope) continue;
        if (!mineActive && itemVisibility !== 'private') continue;
        const appId = text(item.app_id);
        if (!appId) continue;
        if (!appsById.has(appId)) {
          appsById.set(appId, {
            app_id: appId,
            display_name: text(item.display_name),
            publisher_name: text(item.publisher_name),
            authors: text(state.cloud.auth?.nickname || state.cloud.exportAuthor)
              ? [text(state.cloud.auth?.nickname || state.cloud.exportAuthor)]
              : [],
            endpoint_count: 0,
            available_row_count: 0,
            author_count: 0
          });
        }
        const app = appsById.get(appId);
        app.endpoint_count = Number(app.endpoint_count || 0) + Number(item.endpoint_count || 0);
        app.available_row_count = Number(app.available_row_count || 0) + Math.max(Number(item.available_row_count || 0), Number(item.endpoint_count || 0));
      }
      return Array.from(appsById.values()).filter((app) => {
        if (appQuery.length >= 2 && !text(app.display_name).toLowerCase().includes(appQuery)) return false;
        if (publisherQuery.length >= 2 && !text(app.publisher_name).toLowerCase().includes(publisherQuery)) return false;
        if (sourceQuery.length >= 2 && !cloudAppAuthorsLabel(app).toLowerCase().includes(sourceQuery)) return false;
        return true;
      });
    }

    function renderCloudApps() {
      const tbody = document.getElementById('cloud-import-apps');
      if (!tbody) return;
      const visibleApps = cloudVisibleApps();
      if (!visibleApps.length) {
        tbody.innerHTML = '<tr><td colspan="5">' + html(t('dialog.cloud_sync.apps_empty', 'No cloud applications match the current filters.')) + '</td></tr>';
        return;
      }
      tbody.innerHTML = visibleApps.map((app) => {
        const selected = text(app.app_id) === state.cloud.selectedAppId;
        const loading = text(app.app_id) === state.cloud.loadingAppId;
        const rowClass = ' class="observation-row' + (selected ? ' observation-row--confirmed' : '') + (loading ? ' cloud-web-row--loading' : '') + '"';
        const appNameLiteral = JSON.stringify(text(app.display_name)).replaceAll('"', '&quot;');
        const appIdLiteral = JSON.stringify(text(app.app_id)).replaceAll('"', '&quot;');
        const expectedTotalLiteral = JSON.stringify(Number(app.available_row_count || app.endpoint_count || 0)).replaceAll('"', '&quot;');
        const label = html(t('dialog.cloud_sync.download_action', 'Download'));
        return '<tr' + rowClass + '>'
          + '<td>' + cloudAppTextField(app.display_name) + '</td>'
          + '<td>' + cloudAppTextField(app.publisher_name) + '</td>'
          + '<td>' + cloudAppTextField(app.available_row_count || app.endpoint_count || 0) + '</td>'
          + '<td>' + cloudAppTextField(cloudAppAuthorsLabel(app)) + '</td>'
          + '<td class="cloud-web-action-cell"><button class="input-box button button--icon button--square table-action-button" type="button" onclick="downloadCloudApp(' + appIdLiteral + ',' + appNameLiteral + ',' + expectedTotalLiteral + ')" data-tooltip="' + label + '" aria-label="' + label + '"><img class="button__icon button__icon--cloud-import table-action-button__icon" src="' + IMPORT_CLOUD_ICON_SRC + '" alt=""></button></td>'
          + '</tr>';
      }).join('');
    }

    function cloudObservationParams(appId, appName, expectedTotal) {
      const params = new URLSearchParams();
      params.set('app_id', appId);
      params.set('app_name', appName);
      params.set('visibility', text(state.cloud.importVisibilityScope, 'all').toLowerCase());
      if (state.cloud.scopeMine) params.set('own_scope', 'true');
      if (Number(expectedTotal || 0) > 0) params.set('expected_total', String(Number(expectedTotal || 0)));
      const source = text(document.getElementById('cloud-import-author-filter')?.value).trim();
      if (source.length >= 2) params.set('source', source);
      return params;
    }

    async function downloadCloudApp(appId, appName, expectedTotal) {
      state.cloud.loadingAppId = appId;
      state.cloud.selectedAppId = appId;
      state.cloud.selectedAppName = appName;
      state.cloud.rows = [];
      state.cloud.selectedRows = new Set();
      state.cloud.lastSelectedRowId = '';
      setCloudFooterProgress('import', t('dialog.cloud_sync.download_action', 'Download'), 12, appName, expectedTotal);
      renderCloudApps();
      renderCloudRows();
      try {
        const response = await api('/v1/cloud/observations?' + cloudObservationParams(appId, appName, expectedTotal).toString());
        state.cloud.rows = Array.isArray(response.rows) ? response.rows : [];
        state.cloud.quota = response.quota || state.cloud.quota;
        state.cloud.selectedRows = new Set();
        state.cloud.lastSelectedRowId = '';
        renderCloudApps();
        renderCloudRows();
        renderHeaderFilters(state.snapshot);
        setCloudFooterProgress('import', t('dialog.cloud_sync.download_action', 'Download'), 100, appName, state.cloud.rows.length || expectedTotal);
        setFooterMessage(cloudDownloadMessage(state.cloud.rows.length));
        window.setTimeout(() => clearCloudFooterProgress('import'), 350);
      } catch (error) {
        clearCloudFooterProgress('import');
        setStatus(text(error?.message || error), true);
      } finally {
        state.cloud.loadingAppId = '';
        renderCloudApps();
      }
    }

    function cloudRowMatchesFilters(row) {
      const filters = state.cloud.rowFilters;
      if (filters.app_search && text(row.application) !== text(filters.app_search)) return false;
      if (filters.ip_search && !text(row.ip).toLowerCase().includes(text(filters.ip_search).toLowerCase())) return false;
      if (!domainFilterMatches(row.domain, filters.domain_search)) return false;
      if (filters.port_search && text(row.port) !== text(filters.port_search)) return false;
      if (filters.protocol && !['', 'All'].includes(text(filters.protocol)) && text(row.protocol).toLowerCase() !== text(filters.protocol).toLowerCase()) return false;
      const stateFilter = text(filters.observation_filter, 'All');
      if (stateFilter === 'Confirmed' && !state.cloud.selectedRows.has(text(row.row_id))) return false;
      if (stateFilter === 'Unconfirmed' && state.cloud.selectedRows.has(text(row.row_id))) return false;
      if (stateFilter === 'Success' && Number(row.successful_hits || 0) <= 0) return false;
      if (stateFilter === 'Failed' && Number(row.failed_hits || 0) <= 0) return false;
      return true;
    }

    function visibleCloudRows() {
      return state.cloud.rows.filter(cloudRowMatchesFilters);
    }

    function toggleCloudRow(rowId) {
      const id = text(rowId);
      if (state.cloud.selectedRows.has(id)) {
        state.cloud.selectedRows.delete(id);
      } else {
        state.cloud.selectedRows.add(id);
      }
      state.cloud.lastSelectedRowId = id;
      renderCloudRows();
      renderHeaderFilters(state.snapshot);
    }

    function handleCloudRowClick(event, rowId) {
      const id = text(rowId);
      if (!id) return;
      if (!event?.shiftKey && !event?.ctrlKey && !event?.metaKey) return;
      event.preventDefault();
      const additiveSelection = Boolean(event?.ctrlKey || event?.metaKey);
      const rangeSelection = Boolean(event?.shiftKey && state.cloud.lastSelectedRowId);
      if (rangeSelection) {
        const visibleIds = visibleCloudRows().map((row) => text(row.row_id));
        const start = visibleIds.indexOf(state.cloud.lastSelectedRowId);
        const end = visibleIds.indexOf(id);
        if (start >= 0 && end >= 0) {
          const from = Math.min(start, end);
          const to = Math.max(start, end);
          for (const rangeId of visibleIds.slice(from, to + 1)) {
            if (state.cloud.selectedRows.has(rangeId)) {
              state.cloud.selectedRows.delete(rangeId);
            } else {
              state.cloud.selectedRows.add(rangeId);
            }
          }
        } else {
          toggleCloudRow(id);
          return;
        }
      } else if (additiveSelection || !event?.shiftKey || !state.cloud.lastSelectedRowId) {
        if (state.cloud.selectedRows.has(id)) {
          state.cloud.selectedRows.delete(id);
        } else {
          state.cloud.selectedRows.add(id);
        }
      }
      state.cloud.lastSelectedRowId = id;
      renderCloudRows();
      renderHeaderFilters(state.snapshot);
    }

    function handleCloudRowAction(event, rowId) {
      event?.stopPropagation();
      if (event?.shiftKey || event?.ctrlKey || event?.metaKey) {
        handleCloudRowClick(event, rowId);
      } else {
        toggleCloudRow(rowId);
      }
    }

    function renderCloudRows() {
      const tbody = document.getElementById('cloud-import-rows');
      if (!tbody) return;
      const rows = visibleCloudRows();
      const total = state.cloud.rows.length;
      const selectedCount = selectedCloudRows().length;
      const summary = document.getElementById('cloud-import-row-summary');
      const download = state.cloud.quota?.download;
      const quotaText = t('dialog.cloud_sync.download', 'Download') + ': ' + (download ? (download.used_count + '/' + download.limit_count) : '0/0');
      if (summary) {
        summary.innerHTML =
          '<span class="panel-footer-meta__item">' + html(t('observations.total', 'Total rows')) + ': ' + html(total) + '</span>'
          + '<span class="panel-footer-meta__item">' + html(t('observations.selected', 'Selected rows')) + ': ' + html(selectedCount) + '</span>'
          + '<span class="panel-footer-meta__item" id="cloud-import-quota">' + html(quotaText) + '</span>';
      }
      const addButton = document.getElementById('cloud-import-add-monitoring-button');
      const exportButton = document.getElementById('cloud-import-export-csv-button');
      const hasSelected = rows.some((row) => state.cloud.selectedRows.has(text(row.row_id)));
      if (addButton) addButton.disabled = !hasSelected;
      if (exportButton) exportButton.disabled = !hasSelected;
      if (!state.cloud.selectedAppId) {
        tbody.innerHTML = '<tr><td colspan="10">' + html(t('dialog.cloud_sync.apps_not_loaded', 'Choose an application and download data from the cloud.')) + '</td></tr>';
        return;
      }
      if (!rows.length) {
        tbody.innerHTML = '<tr><td colspan="10">' + html(total ? t('observations.empty_filtered', 'No rows match the current filters.') : t('dialog.cloud_sync.no_downloaded_rows', 'No rows were downloaded.')) + '</td></tr>';
        return;
      }
      tbody.innerHTML = rows.map((row) => {
        const selected = state.cloud.selectedRows.has(text(row.row_id));
        const rowClass = ' class="observation-row' + (selected ? ' observation-row--confirmed' : '') + '"';
        const rowIdLiteral = JSON.stringify(text(row.row_id)).replaceAll('"', '&quot;');
        const actionLabel = html(selected ? t('dialog.cloud_sync.unselect_row', 'Unselect row') : t('dialog.cloud_sync.select_row', 'Select row'));
        return '<tr' + rowClass + ' onclick="handleCloudRowClick(event, ' + rowIdLiteral + ')">'
          + '<td>' + html(text(row.application)) + '</td>'
          + '<td>' + html(text(row.ip)) + '</td>'
          + '<td>' + html(text(row.domain)) + '</td>'
          + '<td>' + html(text(row.port)) + '</td>'
          + '<td>' + html(text(row.protocol)) + '</td>'
          + '<td>' + html(text(row.connection)) + '</td>'
          + '<td>' + html(text(row.hits)) + '</td>'
          + '<td>' + html(text(row.author, '-')) + '</td>'
          + '<td data-tooltip="' + html(t('dialog.cloud_sync.privacy_column_tooltip', 'true means private; false means public.')) + '" data-tooltip-align="end">' + html(text(row.privacy, 'false')) + '</td>'
          + '<td class="cloud-web-action-cell"><button class="input-box button button--icon button--square table-action-button" type="button" onclick="handleCloudRowAction(event, ' + rowIdLiteral + ')" data-tooltip="' + actionLabel + '" aria-label="' + actionLabel + '"><img class="button__icon ' + (selected ? 'button__icon--unconfirm-filtered' : 'button__icon--confirm-filtered') + ' table-action-button__icon" src="' + (selected ? UNCONFIRM_FILTERED_ICON_SRC : CONFIRM_FILTERED_ICON_SRC) + '" alt=""></button></td>'
          + '</tr>';
      }).join('');
    }

    function cloudExportCandidateRows() {
      return (state.snapshot?.observed_endpoints || [])
        .filter((row) => observationRowSelected(row));
    }

    function cloudDisplayNameForAppId(appId) {
      const normalized = text(appId).trim().toLowerCase();
      if (!normalized) return '';
      const myApp = (state.cloud.myApps || []).find((item) => text(item.app_id).trim().toLowerCase() === normalized);
      if (myApp) return text(myApp.display_name);
      const catalogApp = (state.cloud.apps || []).find((item) => text(item.app_id).trim().toLowerCase() === normalized);
      return catalogApp ? text(catalogApp.display_name) : '';
    }

    function cloudExportAppPreviewForRow(row) {
      const directCloudAppId = text(row.cloud_app_id).trim();
      if (directCloudAppId) {
        return {
          app_id: directCloudAppId,
          display_name: cloudDisplayNameForAppId(directCloudAppId) || directCloudAppId
        };
      }
      const trackedAppId = row.tracked_app_id;
      const app = (state.snapshot?.tracked_apps || []).find((item) => String(item.id) === String(trackedAppId));
      const appCloudAppId = text(app?.cloud_app_id).trim();
      if (appCloudAppId) {
        return {
          app_id: appCloudAppId,
          display_name: cloudDisplayNameForAppId(appCloudAppId) || appCloudAppId
        };
      }
      const fallbackId = 'tracked:' + text(trackedAppId, 'unknown');
      return {
        app_id: fallbackId,
        display_name: fallbackId
      };
    }

    function buildCloudExportPublicationRows() {
      const rowsByKey = new Map();
      (state.cloud.myApps || []).forEach((item) => {
        const appId = text(item.app_id).trim();
        const key = appId.toLowerCase() || text(item.display_name).trim().toLowerCase();
        if (!key) return;
        if (!rowsByKey.has(key)) {
          rowsByKey.set(key, {
            key,
            display_name: text(item.display_name, appId),
            new_rows: 0,
            author_rows: 0,
            total_rows: 0
          });
        }
        const row = rowsByKey.get(key);
        row.author_rows += Number(item.author_rows || 0);
        row.total_rows += Number(item.total_rows || 0);
      });
      cloudExportCandidateRows().forEach((row) => {
        const preview = cloudExportAppPreviewForRow(row);
        const key = text(preview.app_id).trim().toLowerCase();
        if (!rowsByKey.has(key)) {
          rowsByKey.set(key, {
            key,
            display_name: text(preview.display_name, preview.app_id),
            new_rows: 0,
            author_rows: 0,
            total_rows: 0
          });
        }
        rowsByKey.get(key).new_rows += 1;
      });
      return sortCloudExportPublicationRows(Array.from(rowsByKey.values()));
    }

    function sortCloudExportPublicationRows(rows) {
      const key = state.cloud.publicationSortKey || 'new_rows';
      return rows.slice().sort((left, right) => {
        let ordering = 0;
        if (key === 'new_rows') ordering = Number(left.new_rows || 0) - Number(right.new_rows || 0);
        else if (key === 'author_rows') ordering = Number(left.author_rows || 0) - Number(right.author_rows || 0);
        else if (key === 'total_rows') ordering = Number(left.total_rows || 0) - Number(right.total_rows || 0);
        else ordering = text(left.display_name).localeCompare(text(right.display_name), undefined, { sensitivity: 'base' });
        if (ordering === 0) ordering = text(left.display_name).localeCompare(text(right.display_name), undefined, { sensitivity: 'base' });
        if (ordering === 0) ordering = text(left.key).localeCompare(text(right.key), undefined, { sensitivity: 'base' });
        return state.cloud.publicationSortDescending ? -ordering : ordering;
      });
    }

    function setCloudPublicationSort(key) {
      if (state.cloud.publicationSortKey === key) {
        if (state.cloud.publicationSortDescending) {
          state.cloud.publicationSortKey = 'new_rows';
          state.cloud.publicationSortDescending = true;
        } else {
          state.cloud.publicationSortDescending = true;
        }
      } else {
        state.cloud.publicationSortKey = key;
        state.cloud.publicationSortDescending = false;
      }
      renderCloudExportPublications();
    }

    function updateCloudPublicationSortIndicators() {
      ['app', 'new-rows', 'author-rows', 'total-rows'].forEach((id) => {
        const node = document.getElementById('cloud-export-publications-' + id + '-sort-icon');
        if (!node) return;
        node.className = 'table-sortable__icon table-sortable__icon--idle';
        node.src = SORT_IDLE_ICON_SRC;
      });
      const map = {
        app: 'app',
        new_rows: 'new-rows',
        author_rows: 'author-rows',
        total_rows: 'total-rows'
      };
      const node = document.getElementById('cloud-export-publications-' + map[state.cloud.publicationSortKey || 'new_rows'] + '-sort-icon');
      if (!node) return;
      node.className = 'table-sortable__icon table-sortable__icon--' + (state.cloud.publicationSortDescending ? 'desc' : 'asc');
      node.src = state.cloud.publicationSortDescending ? SORT_DESC_ICON_SRC : SORT_ASC_ICON_SRC;
    }

    function renderCloudExportPublications() {
      const tbody = document.getElementById('cloud-export-publications-rows');
      const title = document.getElementById('cloud-export-publications-title');
      const appHeader = document.getElementById('cloud-export-publications-app');
      const newRowsHeader = document.getElementById('cloud-export-publications-new-rows');
      const authorRowsHeader = document.getElementById('cloud-export-publications-author-rows');
      const totalRowsHeader = document.getElementById('cloud-export-publications-total-rows');
      if (title) title.textContent = t('dialog.cloud_sync.publications', 'Publications');
      if (appHeader) appHeader.querySelector('.table-sortable__label').textContent = t('table.app', 'Application');
      if (newRowsHeader) newRowsHeader.querySelector('.table-sortable__label').textContent = t('dialog.cloud_sync.new_rows', 'New data');
      if (authorRowsHeader) authorRowsHeader.querySelector('.table-sortable__label').textContent = t('dialog.cloud_sync.author_rows', 'Author rows');
      if (totalRowsHeader) totalRowsHeader.querySelector('.table-sortable__label').textContent = t('dialog.cloud_sync.total_rows', 'Total rows');
      updateCloudPublicationSortIndicators();
      if (!tbody) return;
      if (!state.cloud.auth?.authenticated) {
        tbody.innerHTML = '<tr><td class="cloud-sync-empty-cell" colspan="4">' + html(t('dialog.cloud_sync.my_apps_not_loaded', 'Sign in to see applications you published.')) + '</td></tr>';
        return;
      }
      const rows = buildCloudExportPublicationRows();
      if (!rows.length) {
        tbody.innerHTML = '<tr><td class="cloud-sync-empty-cell" colspan="4">' + html(t('dialog.cloud_sync.apps_empty', 'No cloud applications match the current filters.')) + '</td></tr>';
        return;
      }
      tbody.innerHTML = rows.map((item) => {
        const newRows = Number(item.new_rows || 0);
        return '<tr class="observation-row cloud-sync-publication-row">'
          + '<td>' + html(text(item.display_name)) + '</td>'
          + '<td>' + (newRows > 0 ? '<span class="state-label state-label--success">' + html(text(newRows)) + '</span>' : '') + '</td>'
          + '<td>' + html(text(item.author_rows, '0')) + '</td>'
          + '<td>' + html(text(item.total_rows, '0')) + '</td>'
          + '</tr>';
      }).join('');
    }

    function renderCloudExportRows() {
      const rows = cloudExportCandidateRows();
      renderCloudExportPublications();
      const quota = document.getElementById('cloud-export-quota');
      if (quota) {
        const upload = state.cloud.quota?.upload;
        quota.textContent = t('dialog.cloud_sync.upload', 'Upload') + ': ' + (upload ? (upload.used_count + '/' + upload.limit_count) : '0/0');
      }
      const sendButton = document.getElementById('cloud-export-send-button');
      const authenticated = Boolean(state.cloud.auth?.authenticated);
      const nickname = text(document.getElementById('cloud-export-nickname')?.value || state.cloud.exportAuthor).trim();
      const nicknameReady = nickname.length > 0 && state.cloud.nicknameStatus !== 'invalid' && state.cloud.nicknameStatus !== 'taken';
      if (sendButton) sendButton.disabled = !rows.length || !authenticated || !nicknameReady;
    }

    async function requestCloudExportFromNative() {
      const rows = cloudExportCandidateRows();
      if (!rows.length) {
        setFooterMessage(t('dialog.cloud_sync.no_upload_rows', 'No rows are ready for cloud export.'), true);
        return;
      }
      const nickname = text(document.getElementById('cloud-export-nickname')?.value || state.cloud.exportAuthor).trim();
      if (!nickname) {
        state.cloud.nicknameStatus = 'invalid';
        renderCloudExportAuth();
        setFooterMessage(t('dialog.cloud_sync.nickname_invalid', 'Nickname is not allowed'), true);
        return;
      }
      try {
        await post('/v1/cloud/nickname', { nickname }, { skipSnapshotRefresh: true });
        state.cloud.exportAuthor = nickname;
        state.cloud.nicknameStatus = 'accepted';
        setCloudFooterProgress('export', t('dialog.cloud_sync.upload_action', 'Send'), 20, String(rows.length), rows.length);
        const result = await post('/v1/cloud/upload', {
          author_signature: nickname,
          visibility: state.cloud.uploadPrivate ? 'private' : 'public'
        });
        if (result?.quota) state.cloud.quota = result.quota;
        setCloudFooterProgress('export', t('dialog.cloud_sync.upload_action', 'Send'), 100, String(result?.accepted_rows || rows.length), rows.length);
        setFooterMessage(cloudUploadMessage(result));
        renderCloudExportRows();
        window.setTimeout(() => clearCloudFooterProgress('export'), 350);
      } catch (error) {
        clearCloudFooterProgress('export');
        const message = text(error?.message || error);
        if (message.includes('nickname_taken')) {
          state.cloud.nicknameStatus = 'taken';
          renderCloudExportAuth();
        } else if (message.includes('invalid_author_signature') || message.includes('nickname_blocked')) {
          state.cloud.nicknameStatus = 'invalid';
          renderCloudExportAuth();
        }
        setStatus(text(error?.message || error), true);
      }
    }

    function selectedCloudRows() {
      return (state.cloud.rows || []).filter((row) => state.cloud.selectedRows.has(text(row.row_id)));
    }

    function cloudRowToMonitoringImportRow(row) {
      const raw = row.row || {};
      const connectionRaw = text(raw.connection_state || row.connection, 'Established').split(' ')[0].toLowerCase();
      const connectionState = connectionRaw === 'attempting'
        ? 'Attempting'
        : connectionRaw === 'closing'
          ? 'Closing'
          : connectionRaw === 'failed'
            ? 'Failed'
            : connectionRaw === 'unknown'
              ? 'Unknown'
              : 'Established';
      return {
        application: text(row.application),
        app_connector_id: null,
        cloud_app_id: text(raw.app_id) || null,
        app_signature_key: raw.app_signature_key || null,
        app_signature_subject: raw.app_signature_subject || null,
        app_signature_issuer: raw.app_signature_issuer || null,
        remote_ip: text(row.ip),
        domain: text(row.domain) || null,
        remote_port: Number(row.port || 0),
        protocol: text(row.protocol, 'TCP').toLowerCase(),
        connection_state: connectionState,
        first_seen_ms: Number(row.first_seen_ms || Date.now()),
        last_seen_ms: Number(row.last_seen_ms || Date.now()),
        hits: Number(row.hits || 0),
        failed_hits: Number(row.failed_hits || 0),
        successful_hits: Number(row.successful_hits || 0)
      };
    }

    async function addSelectedCloudRowsToMonitoring() {
      const rows = selectedCloudRows();
      if (!rows.length) {
        setFooterMessage(t('dialog.cloud_sync.no_selected_rows', 'Select cloud rows first'), true);
        return;
      }
      try {
        const result = await post('/v1/observations/import-csv', {
          import_source: 'cloud_download',
          rows: rows.map(cloudRowToMonitoringImportRow)
        });
        setFooterMessage(cloudImportMessage(result));
      } catch (error) {
        setStatus(text(error?.message || error), true);
      }
    }

    function cloudCsvRows(rows) {
      const header = [
        'application',
        'app_connector_id',
        'cloud_app_id',
        'app_signature_key',
        'app_signature_subject',
        'app_signature_issuer',
        'ip',
        'domain',
        'port',
        'protocol',
        'connection',
        'requests',
        'first_seen',
        'last_seen'
      ].join(',');
      const body = rows.map((row) => [
        row.application,
        '',
        row.row?.app_id || '',
        row.row?.app_signature_key || '',
        row.row?.app_signature_subject || '',
        row.row?.app_signature_issuer || '',
        row.ip,
        row.domain,
        row.port,
        row.protocol,
        row.connection,
        row.hits,
        formatTime(row.first_seen_ms),
        formatTime(row.last_seen_ms)
      ].map(csvEscape).join(','));
      return [header].concat(body).join('\r\n') + '\r\n';
    }

    function exportSelectedCloudRowsCsv() {
      const rows = selectedCloudRows();
      if (!rows.length) {
        setFooterMessage(t('footer.event.csv_export_empty', 'No selected rows for CSV'), true);
        return;
      }
      const blob = new Blob([cloudCsvRows(rows)], { type: 'text/csv;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'NetStitch-cloud-import-selected.csv';
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
      setFooterMessage(t('footer.event.csv_export_completed', 'CSV exported') + ': ' + link.download);
    }

    async function clearIpFilter() {
      const input = document.getElementById('ip-filter');
      if (input) input.value = '';
      syncClearButtonStates();
      if (state.cloudPanel === 'import') {
        state.cloud.rowFilters.ip_search = '';
        renderCloudRows();
        return;
      }
      await updateFilters({ ip_search: '' });
    }

    async function clearDomainFilter() {
      const input = document.getElementById('domain-filter');
      if (input) input.value = '';
      pulseInput(input);
      syncClearButtonStates();
      await setDomainFilter('');
    }

    async function clearPortFilter() {
      const input = document.getElementById('port-filter');
      if (input) input.value = '';
      pulseInput(input);
      syncClearButtonStates();
      await setPortFilter('');
    }

    function toggleCloudPanel(panel) {
      state.cloudPanel = state.cloudPanel === panel ? null : panel;
      renderCloudPanel();
      if (state.cloudPanel) {
        refreshCloudQuota();
        refreshCloudAuthState();
      }
    }

    function syncCloudHeaderControlAvailability(active) {
      const disabledIds = [
        'toggle-monitoring-button',
        'import-csv-button',
        'export-csv-button'
      ];
      for (const id of disabledIds) {
        const node = document.getElementById(id);
        if (node) node.disabled = Boolean(active);
      }
      for (const id of ['domain-capture-switch', 'web-server-switch']) {
        const node = document.getElementById(id);
        if (node) node.disabled = Boolean(active);
      }
    }

    function renderCloudPanel() {
      const active = state.cloudPanel;
      syncCloudHeaderControlAvailability(active);
      for (const id of ['tracked-apps-panel', 'integration-panel', 'ignored-addresses-panel', 'browser-observations-panel']) {
        const node = document.getElementById(id);
        if (node) node.hidden = Boolean(active);
      }
      const importPanel = document.getElementById('browser-cloud-import-panel');
      const exportPanel = document.getElementById('browser-cloud-export-panel');
      if (importPanel) importPanel.hidden = active !== 'import';
      if (exportPanel) exportPanel.hidden = active !== 'export';
      const importButton = document.getElementById('cloud-import-button');
      const exportButton = document.getElementById('cloud-export-button');
      if (importButton) importButton.classList.toggle('button--cloud-active', active === 'import');
      if (exportButton) exportButton.classList.toggle('button--cloud-active', active === 'export');
      const confirmFilteredButton = document.getElementById('confirm-filtered-button');
      const unconfirmFilteredButton = document.getElementById('unconfirm-filtered-button');
      const clearMonitoringButton = document.getElementById('clear-monitoring-button');
      if (confirmFilteredButton && active === 'import') {
        const label = t('dialog.cloud_sync.select_visible', 'Cloud import: select visible rows');
        confirmFilteredButton.disabled = false;
        confirmFilteredButton.setAttribute('aria-label', label);
        confirmFilteredButton.setAttribute('data-tooltip', label);
      }
      if (unconfirmFilteredButton && active === 'import') {
        const label = t('dialog.cloud_sync.unselect_visible', 'Cloud import: unselect visible rows');
        unconfirmFilteredButton.disabled = false;
        unconfirmFilteredButton.setAttribute('aria-label', label);
        unconfirmFilteredButton.setAttribute('data-tooltip', label);
      }
      if (clearMonitoringButton && active === 'import') {
        const label = t('dialog.cloud_sync.delete_selected', 'Cloud import: delete selected rows');
        clearMonitoringButton.disabled = false;
        clearMonitoringButton.setAttribute('aria-label', label);
        clearMonitoringButton.setAttribute('data-tooltip', label);
      }
      if (active !== 'import') {
        if (confirmFilteredButton) {
          const label = t('action.confirm_filtered', 'Confirm all visible');
          confirmFilteredButton.setAttribute('aria-label', label);
          confirmFilteredButton.setAttribute('data-tooltip', label);
        }
        if (unconfirmFilteredButton) {
          const label = t('action.unconfirm_filtered', 'Unconfirm all visible');
          unconfirmFilteredButton.setAttribute('aria-label', label);
          unconfirmFilteredButton.setAttribute('data-tooltip', label);
        }
        if (clearMonitoringButton) {
          const label = localizedClearMonitoringLabel();
          clearMonitoringButton.setAttribute('aria-label', label);
          clearMonitoringButton.setAttribute('data-tooltip', label);
        }
      }
      renderText('cloud-import-panel-title', 'dialog.cloud_sync.download_title', 'Cloud: data download');
      renderHelp('cloud-import-panel-help', 'dialog.cloud_sync.import_help', 'Search applications through the local NetStitch service and stage downloaded rows locally.');
      renderText('cloud-export-panel-title', 'dialog.cloud_sync.upload_title', 'Cloud: data upload');
      renderHelp('cloud-export-panel-help', 'dialog.cloud_sync.export_help', 'Cloud export is served by the local NetStitch service.');
      const statusKey = state.cloud.serviceOnline ? 'dialog.cloud_sync.service_online' : 'dialog.cloud_sync.service_unknown';
      const statusFallback = state.cloud.serviceOnline ? 'Service online' : 'Service status unknown';
      const importStatus = document.getElementById('cloud-import-panel-status');
      const exportStatus = document.getElementById('cloud-export-panel-status');
      for (const statusNode of [importStatus, exportStatus]) {
        if (!statusNode) continue;
        statusNode.className = 'panel-header-meta cloud-web-panel-service ' + (state.cloud.serviceOnline ? 'panel-header-meta--success' : 'panel-header-meta--danger');
        const statusText = statusNode.querySelector('.cloud-web-panel-service__text');
        if (statusText) statusText.textContent = t(statusKey, statusFallback);
        const statusDot = statusNode.querySelector('.footer-watcher-dot');
        if (statusDot) statusDot.className = 'footer-watcher-dot ' + (state.cloud.serviceOnline ? 'footer-watcher-dot--connected' : 'footer-watcher-dot--disconnected');
        if (state.cloud.serviceMessage) statusNode.dataset.tooltip = state.cloud.serviceMessage;
      }
      renderCloudExportAuth();
      renderText('cloud-import-app-filter-label', 'table.app', 'App');
      renderText('cloud-import-company-filter-label', 'dialog.cloud_sync.company', 'Company');
      renderText('cloud-import-author-filter-label', 'dialog.cloud_sync.source', 'Author');
      renderText('cloud-import-visibility-filter-label', 'dialog.cloud_sync.visibility_scope', 'Private');
      renderPlaceholder('cloud-import-app-filter', 'dialog.cloud_sync.app_search', 'Search by app name');
      renderPlaceholder('cloud-import-company-filter', 'dialog.cloud_sync.publisher_search', 'Search by company');
      renderPlaceholder('cloud-import-author-filter', 'dialog.cloud_sync.source_search', 'Search by author');
      const visibilitySelect = document.getElementById('cloud-import-visibility-filter');
      if (visibilitySelect) {
        visibilitySelect.value = text(state.cloud.importVisibilityScope, 'all').toLowerCase();
        const labels = [
          ['all', 'dialog.cloud_sync.visibility_all', 'All'],
          ['public', 'dialog.cloud_sync.visibility_public', 'Public'],
          ['private', 'dialog.cloud_sync.visibility_private', 'Private']
        ];
        for (const [value, key, fallback] of labels) {
          const option = visibilitySelect.querySelector('option[value="' + value + '"]');
          if (option) option.textContent = t(key, fallback);
        }
      }
      const mineButton = document.getElementById('cloud-import-scope-mine');
      if (mineButton) {
        const mineLabel = t('dialog.cloud_sync.scope_mine', 'My publications');
        const mineEnabled = Boolean(state.cloud.auth?.authenticated);
        mineButton.setAttribute('aria-label', mineLabel);
        mineButton.setAttribute('data-tooltip', mineLabel);
        mineButton.disabled = !mineEnabled;
        if (!mineEnabled) state.cloud.scopeMine = false;
        mineButton.setAttribute('aria-pressed', state.cloud.scopeMine && mineEnabled ? 'true' : 'false');
        mineButton.classList.toggle('button--cloud-active', Boolean(state.cloud.scopeMine && mineEnabled));
        mineButton.innerHTML = '<img class="button__icon button__icon--my-publications" src="' + MY_PUBLICATIONS_ICON_SRC + '" alt="">';
      }
      renderSortableHeaderText('cloud-import-table-app', 'filter.app', 'App');
      renderSortableHeaderText('cloud-import-table-company', 'dialog.cloud_sync.company', 'Company');
      renderSortableHeaderText('cloud-import-table-rows', 'dialog.cloud_sync.available_rows', 'Rows');
      renderSortableHeaderText('cloud-import-table-authors', 'dialog.cloud_sync.authors', 'Authors');
      renderSortableHeaderText('cloud-import-row-table-app', 'filter.app', 'App');
      renderSortableHeaderText('cloud-import-row-table-ip', 'table.ip', 'IP');
      renderSortableHeaderText('cloud-import-row-table-domain', 'table.domain', 'Domain');
      renderSortableHeaderText('cloud-import-row-table-port', 'filter.port', 'Port');
      renderSortableHeaderText('cloud-import-row-table-protocol', 'filter.protocol', 'Protocol');
      renderSortableHeaderText('cloud-import-row-table-connection', 'table.conn', 'Conn');
      renderSortableHeaderText('cloud-import-row-table-hits', 'table.hits', 'Cnt.');
      renderSortableHeaderText('cloud-import-row-table-author', 'dialog.cloud_sync.source', 'Author');
      const privacyHeader = document.getElementById('cloud-import-row-table-privacy');
      if (privacyHeader) {
        privacyHeader.textContent = t('dialog.cloud_sync.privacy_column', 'Private');
        privacyHeader.setAttribute('data-tooltip', t('dialog.cloud_sync.privacy_column_tooltip', 'true means private; false means public.'));
      }
      renderText('cloud-import-quota', 'dialog.cloud_sync.download_quota', 'Download: 0/0');
      renderText('cloud-import-add-monitoring-button', 'dialog.cloud_sync.add_to_monitoring', 'Add to monitoring');
      renderText('cloud-import-export-csv-button', 'dialog.cloud_sync.export_selected', 'Export CSV');
      renderText('cloud-export-send-button', 'dialog.cloud_sync.upload_action', 'Send');
      renderText('cloud-export-quota', 'dialog.cloud_sync.upload_quota', 'Upload: 0/0');
      renderText('cloud-import-placeholder', 'dialog.cloud_sync.apps_not_loaded', 'Enter a filter and press Enter.');
      renderText('cloud-import-rows-placeholder', 'dialog.cloud_sync.no_downloaded_rows', 'Choose an application and download cloud data.');
      renderCloudApps();
      renderCloudRows();
      renderCloudExportRows();
      syncClearButtonStates();
    }

    function setObservationSort(key) {
      if (state.observationSortKey === key) {
        if (state.observationSortDescending) {
          state.observationSortKey = '';
          state.observationSortDescending = false;
        } else {
          state.observationSortDescending = true;
        }
      } else {
        state.observationSortKey = key;
        state.observationSortDescending = false;
      }
      applyObservationFilters();
    }

    function pickerInitialDirectory(pathValue) {
      const value = text(pathValue).trim();
      if (!value) return '';
      const normalized = value.replaceAll('/', '\\');
      const slash = Math.max(normalized.lastIndexOf('\\'), normalized.lastIndexOf('/'));
      if (slash > 2) return value.slice(0, slash);
      return value;
    }

    async function openServerFilePicker(options) {
      state.filePicker = {
        open: true,
        intent: options.intent || '',
        mode: options.mode || 'any',
        title: options.title || t('dialog.file_picker.title', 'Choose file'),
        help: options.help || t('dialog.file_picker.help', 'Files are shown from the machine where NetStitch is running.'),
        currentPath: '',
        parentPath: '',
        entries: [],
        roots: [],
        selectedPath: '',
        save: Boolean(options.save),
        defaultFileName: options.defaultFileName || '',
        saveFileName: options.defaultFileName || '',
        defaultExtension: options.defaultExtension || '',
        extensions: Array.isArray(options.extensions) ? options.extensions : [],
        overwritePolicy: options.overwritePolicy || 'prompt',
        confirmLabel: options.confirmLabel || '',
        moduleValueTarget: options.moduleValueTarget || '',
        moduleStatusTarget: options.moduleStatusTarget || '',
        selectedStatus: options.selectedStatus || '',
        feedback: ''
      };
      renderServerFilePicker();
      await loadServerFilePicker(options.initialPath || '');
    }

    async function loadServerFilePicker(pathValue) {
      const picker = state.filePicker;
      if (!picker.open) return;
      try {
        const query = new URLSearchParams({
          mode: picker.mode,
          path: text(pathValue)
        });
        const extensions = Array.isArray(picker.extensions) ? picker.extensions.filter(Boolean).join(',') : '';
        if (extensions) query.set('extensions', extensions);
        const response = await api('/v1/filesystem/browse?' + query.toString());
        state.filePicker.currentPath = response.current_path || '';
        state.filePicker.parentPath = response.parent_path || '';
        state.filePicker.entries = Array.isArray(response.entries) ? response.entries : [];
        state.filePicker.roots = Array.isArray(response.roots) ? response.roots : [];
        state.filePicker.selectedPath = '';
        state.filePicker.feedback = '';
      } catch (error) {
        state.filePicker.feedback = text(error?.message || error);
      }
      renderServerFilePicker();
    }

    function renderServerFilePicker() {
      const modal = document.getElementById('server-file-picker-modal');
      if (!modal) return;
      const picker = state.filePicker || {};
      modal.hidden = !picker.open;
      if (!picker.open) return;
      const title = document.getElementById('server-file-picker-title');
      if (title) title.textContent = picker.title || t('dialog.file_picker.title', 'Choose file');
      const help = document.getElementById('server-file-picker-help');
      if (help) help.textContent = picker.help || t('dialog.file_picker.help', 'Files are shown from the machine where NetStitch is running.');
      const pathInput = document.getElementById('server-file-picker-path');
      if (pathInput) pathInput.value = text(picker.currentPath);
      const saveRow = document.getElementById('server-file-picker-save-row');
      if (saveRow) saveRow.hidden = !picker.save;
      const saveName = document.getElementById('server-file-picker-save-name');
      if (saveName) saveName.value = text(picker.saveFileName, picker.defaultFileName);
      const feedback = document.getElementById('server-file-picker-feedback');
      if (feedback) feedback.textContent = text(picker.feedback);
      const confirm = document.getElementById('server-file-picker-confirm-button');
      if (confirm) confirm.textContent = picker.confirmLabel || t('dialog.file_picker.choose', 'Choose');
      const roots = document.getElementById('server-file-picker-roots');
      if (roots) {
        roots.innerHTML = (picker.roots || []).map((root) =>
          '<button class="button button--secondary" type="button" data-picker-path="' + html(root) + '" onclick="serverFilePickerOpenDatasetPath(event)">' + html(root) + '</button>'
        ).join('');
      }
      const rows = [];
      if (picker.parentPath) {
        rows.push('<button class="server-file-picker-row" type="button" data-picker-path="' + html(picker.parentPath) + '" onclick="serverFilePickerOpenDatasetPath(event)"><span>↥</span><span><span class="server-file-picker-row__name">..</span><span class="server-file-picker-row__path">' + html(picker.parentPath) + '</span></span><span>' + html(t('dialog.file_picker.folder', 'Folder')) + '</span></button>');
      }
      for (const entry of picker.entries || []) {
        const selected = picker.selectedPath === entry.path ? ' server-file-picker-row--selected' : '';
        const icon = entry.is_dir ? '▸' : '•';
        const kind = entry.is_dir ? t('dialog.file_picker.folder', 'Folder') : t('dialog.file_picker.file', 'File');
        const action = entry.is_dir ? 'serverFilePickerOpenDatasetPath(event)' : 'serverFilePickerSelectDatasetPath(event)';
        const disabled = !entry.is_dir && !entry.selectable ? ' disabled' : '';
        rows.push('<button class="server-file-picker-row' + selected + '" type="button" data-picker-path="' + html(entry.path) + '" onclick="' + action + '"' + disabled + '><span>' + icon + '</span><span><span class="server-file-picker-row__name">' + html(entry.name) + '</span><span class="server-file-picker-row__path">' + html(entry.path) + '</span></span><span>' + html(kind) + '</span></button>');
      }
      const list = document.getElementById('server-file-picker-list');
      if (list) {
        list.innerHTML = rows.join('') || '<div class="subtle">' + html(t('dialog.file_picker.empty', 'No files in this folder.')) + '</div>';
      }
      syncClearButtonStates();
    }

    function serverFilePickerOpenDatasetPath(event) {
      const pathValue = event?.currentTarget?.dataset?.pickerPath || '';
      loadServerFilePicker(pathValue);
    }

    function serverFilePickerSelectDatasetPath(event) {
      const pathValue = event?.currentTarget?.dataset?.pickerPath || '';
      if (!pathValue) return;
      state.filePicker.selectedPath = pathValue;
      if (state.filePicker.save) {
        const parts = pathValue.split(/[\\/]/);
        state.filePicker.saveFileName = parts[parts.length - 1] || state.filePicker.saveFileName;
      }
      state.filePicker.feedback = '';
      renderServerFilePicker();
    }

    function openServerFilePickerTypedPath() {
      const pathValue = document.getElementById('server-file-picker-path')?.value || '';
      loadServerFilePicker(pathValue);
    }

    function clearServerFilePickerPath() {
      const input = document.getElementById('server-file-picker-path');
      if (input) input.value = '';
      syncClearButtonStates();
    }

    function closeServerFilePicker() {
      state.filePicker.open = false;
      renderServerFilePicker();
    }

    function serverFilePickerJoinPath(directory, fileName) {
      const dir = text(directory).trim();
      const name = text(fileName).trim();
      if (!dir) return name;
      if (!name) return dir;
      if (/[\\/]$/.test(dir)) return dir + name;
      return dir + (dir.includes('\\') ? '\\' : '/') + name;
    }

    function serverFilePickerEnsureExtension(pathValue, extension) {
      const path = text(pathValue).trim();
      const normalizedExtension = text(extension).trim().replace(/^\./, '');
      if (!path || !normalizedExtension) return path;
      const fileName = path.split(/[\\/]/).pop() || path;
      if (/\.[^\\/.\s]+$/.test(fileName)) return path;
      return path + '.' + normalizedExtension;
    }

    function serverFilePickerPathExists(pathValue) {
      const path = text(pathValue).trim();
      if (!path) return false;
      return (state.filePicker?.entries || []).some((entry) => text(entry?.path).trim() === path);
    }

    async function confirmServerFilePicker() {
      const picker = state.filePicker || {};
      const saveName = document.getElementById('server-file-picker-save-name')?.value || picker.saveFileName;
      let selectedPath = picker.save
        ? serverFilePickerJoinPath(picker.currentPath, saveName || picker.defaultFileName)
        : picker.selectedPath;
      if (picker.intent === 'integration_folder' && !selectedPath) {
        selectedPath = picker.currentPath;
      }
      if (picker.intent === 'module_browse_window' && picker.mode === 'folder' && !selectedPath) {
        selectedPath = picker.currentPath;
      }
      if (picker.intent === 'module_browse_window' && picker.mode === 'file_save') {
        selectedPath = serverFilePickerEnsureExtension(selectedPath, picker.defaultExtension);
        if (serverFilePickerPathExists(selectedPath)) {
          const overwritePolicy = text(picker.overwritePolicy, 'prompt').trim();
          if (overwritePolicy === 'deny') {
            state.filePicker.feedback = t('dialog.file_picker.exists_denied', 'Selected file already exists.');
            return renderServerFilePicker();
          }
          if (overwritePolicy === 'prompt' && !window.confirm(t('dialog.file_picker.overwrite_prompt', 'Replace the existing file?'))) {
            return renderServerFilePicker();
          }
        }
      }
      if (!selectedPath) {
        state.filePicker.feedback = t('dialog.file_picker.select_required', 'Select a file first.');
        return renderServerFilePicker();
      }
      try {
        if (picker.intent === 'add_executable') {
          await post('/v1/tracked-apps', { exe_path: selectedPath, enabled: false });
          const input = document.getElementById('exe-path');
          if (input) input.value = '';
        } else if (picker.intent === 'profile_export') {
          selectProfileExportProfile(selectedPath);
        } else if (picker.intent === 'integration_folder') {
          const input = document.getElementById('integration-dir');
          if (input) {
            input.value = selectedPath;
            state.integrationDir = selectedPath;
            pulseInput(input);
          }
          renderIntegrationModal(state.snapshot);
        } else if (picker.intent === 'csv_import') {
          await importCsvFromServerPath(selectedPath);
        } else if (picker.intent === 'csv_export') {
          await exportCsvToServerPath(selectedPath);
        } else if (picker.intent === 'module_browse_window') {
          const target = text(picker.moduleValueTarget).trim().slice(0, 120);
          const statusTarget = text(picker.moduleStatusTarget).trim().slice(0, 120);
          const selectedStatus = text(picker.selectedStatus).trim();
          if (target) {
            const nextValues = { ...(state.moduleUiValues || {}), [target]: selectedPath };
            if (statusTarget && selectedStatus) nextValues[statusTarget] = selectedStatus;
            state.moduleUiValues = nextValues;
            renderIntegrationModulePanel(state.snapshot);
          }
        }
        closeServerFilePicker();
      } catch (error) {
        state.filePicker.feedback = text(error?.message || error);
        renderServerFilePicker();
      }
    }

    async function addTrackedApp() {
      const input = document.getElementById('exe-path');
      const exePath = input.value.trim();
      if (!exePath) {
        return openServerFilePicker({
          intent: 'add_executable',
          mode: 'executable',
          title: t('dialog.file_picker.executable_title', 'Choose executable file'),
          help: t('dialog.file_picker.executable_help', 'Select an executable file on the machine where NetStitch is running.'),
          initialPath: ''
        });
      }
      await post('/v1/tracked-apps', { exe_path: exePath, enabled: false });
      input.value = '';
    }

    async function addTrackedAppOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      event.preventDefault();
      pulseInput(event.currentTarget);
      await addTrackedApp();
    }

    function openCsvImportPicker() {
      openServerFilePicker({
        intent: 'csv_import',
        mode: 'csv_open',
        title: t('dialog.file_picker.csv_import_title', 'Import monitoring CSV'),
        help: t('dialog.file_picker.csv_import_help', 'Select a CSV file from the machine where NetStitch is running.'),
        initialPath: ''
      });
    }

    function openCsvExportPicker() {
      const rows = selectedCsvExportRows(state.snapshot);
      if (!rows.length) {
        return setFooterMessage(t('footer.event.csv_export_empty', 'No selected rows for CSV'), true);
      }
      openServerFilePicker({
        intent: 'csv_export',
        mode: 'csv_save',
        title: t('dialog.file_picker.csv_export_title', 'Export monitoring CSV'),
        help: t('dialog.file_picker.csv_export_help', 'Choose where to save the CSV on the machine where NetStitch is running.'),
        save: true,
        defaultFileName: 'NetStitch-monitoring-selected.csv',
        initialPath: ''
      });
    }

    async function importCsvFromServerPath(pathValue) {
      const response = await post('/v1/filesystem/read-text', { path: pathValue }, { skipSnapshotRefresh: true });
      const request = parseCsvImportRequest(response.content || '');
      if (!request.rows.length) {
        setFooterMessage(t('footer.event.csv_import_empty', 'CSV does not contain monitoring rows to import'), true);
        return;
      }
      const result = await post('/v1/observations/import-csv', request);
      setFooterMessage(csvImportStatusLine(t('footer.event.csv_import_completed', 'CSV imported'), pathValue, result));
    }

    async function exportCsvToServerPath(pathValue) {
      const rows = selectedCsvExportRows(state.snapshot);
      if (!rows.length) {
        setFooterMessage(t('footer.event.csv_export_empty', 'No selected rows for CSV'), true);
        return;
      }
      const targetPath = /\.csv$/i.test(text(pathValue).trim()) ? pathValue : text(pathValue).trim() + '.csv';
      const content = renderCsvExportRows(rows);
      const response = await post('/v1/filesystem/write-text', { path: targetPath, content }, { skipSnapshotRefresh: true });
      setFooterMessage(t('footer.event.csv_export_completed', 'CSV exported') + ': ' + text(response.path, targetPath));
    }

    function selectedCsvExportRows(snapshot) {
      return (snapshot?.observed_endpoints || [])
        .filter((observation) => observationRowSelected(observation))
        .filter((observation) => text(observation.remote_ip).trim())
        .map((observation) => ({
          application: appName(snapshot, observation.tracked_app_id, observation),
          ip: text(observation.remote_ip).trim(),
          domain: csvExportDomainText(observation.enrichment),
          port: text(observation.remote_port),
          protocol: text(observation.protocol),
          connection: csvConnectionValue(observation),
          requests: text(observation.hits),
          first_seen: formatTime(observation.first_seen_ms),
          last_seen: formatTime(observation.last_seen_ms)
        }));
    }

    function csvExportDomainText(enrichment) {
      const source = text(enrichment?.domain_source);
      if (!['http_host', 'tls_sni', 'quic_sni', 'csv_import'].includes(source)) return '';
      return text(enrichment?.domain_name).trim();
    }

    function renderCsvExportRows(rows) {
      const header = ['application', 'ip', 'domain', 'port', 'protocol', 'connection', 'requests', 'first_seen', 'last_seen'];
      const lines = [header.join(',')];
      for (const row of rows) {
        lines.push([
          row.application,
          row.ip,
          row.domain,
          row.port,
          row.protocol,
          row.connection,
          row.requests,
          row.first_seen,
          row.last_seen
        ].map(csvEscape).join(','));
      }
      return lines.join('\r\n') + '\r\n';
    }

    function csvConnectionValue(observation) {
      return text(observation.connection_state) + ' (' + (observation.successful_hits || 0) + '/' + (observation.failed_hits || 0) + '/' + (observation.hits || 0) + ')';
    }

    function parseCsvImportRequest(content) {
      const records = parseCsvRecords(content);
      if (!records.length) return { rows: [] };
      const columns = {};
      records[0].forEach((value, index) => { columns[normalizeCsvHeader(value)] = index; });
      const rows = [];
      for (const record of records.slice(1)) {
        if (record.every((value) => !text(value).trim())) continue;
        const ip = csvColumn(record, columns, ['ip']);
        const port = csvColumn(record, columns, ['port']);
        if (!ip || !port) continue;
        const remotePort = Number.parseInt(text(port).trim(), 10);
        if (!Number.isFinite(remotePort) || remotePort < 1 || remotePort > 65535) continue;
        const protocolRaw = text(csvColumn(record, columns, ['protocol']), 'tcp').trim().toLowerCase();
        const protocol = protocolRaw === 'udp' ? 'Udp' : protocolRaw === 'icmp' ? 'Icmp' : 'Tcp';
        const metrics = parseCsvConnectionValue(
          csvColumn(record, columns, ['connection']) || '',
          csvColumn(record, columns, ['requests']) || ''
        );
        rows.push({
          application: text(csvColumn(record, columns, ['application'])).trim() || null,
          remote_ip: text(ip).trim(),
          remote_port: remotePort,
          protocol,
          domain: text(csvColumn(record, columns, ['domain'])).trim() || null,
          successful_hits: metrics.successful_hits,
          failed_hits: metrics.failed_hits,
          hits: metrics.hits,
          first_seen_ms: parseCsvTimestampMs(csvColumn(record, columns, ['first_seen'])),
          last_seen_ms: parseCsvTimestampMs(csvColumn(record, columns, ['last_seen']))
        });
      }
      return { import_source: 'csv', rows };
    }

    function parseCsvRecords(content) {
      const records = [];
      let record = [];
      let field = '';
      let quoted = false;
      const source = text(content).replace(/\r\n/g, '\n').replace(/\r/g, '\n');
      for (let index = 0; index < source.length; index += 1) {
        const char = source[index];
        if (quoted) {
          if (char === '"' && source[index + 1] === '"') {
            field += '"';
            index += 1;
          } else if (char === '"') {
            quoted = false;
          } else {
            field += char;
          }
        } else if (char === '"') {
          quoted = true;
        } else if (char === ',') {
          record.push(field);
          field = '';
        } else if (char === '\n') {
          record.push(field);
          records.push(record);
          record = [];
          field = '';
        } else {
          field += char;
        }
      }
      record.push(field);
      if (record.some((value) => text(value).trim())) records.push(record);
      return records;
    }

    function csvColumn(record, columns, aliases) {
      for (const alias of aliases) {
        const index = columns[normalizeCsvHeader(alias)];
        if (index !== undefined) return record[index] || '';
      }
      return '';
    }

    function normalizeCsvHeader(value) {
      return text(value).trim().toLowerCase().replace(/[\s-]+/g, '_');
    }

    function parseCsvConnectionValue(connection, requests) {
      const raw = text(connection);
      const match = raw.match(/\((\d+)\/(\d+)(?:\/(\d+))?\)/);
      if (match) {
        const successful = Number.parseInt(match[1], 10) || 0;
        const failed = Number.parseInt(match[2], 10) || 0;
        const total = match[3] ? Number.parseInt(match[3], 10) || 0 : successful + failed;
        return { successful_hits: successful, failed_hits: failed, hits: Math.max(total, successful + failed) };
      }
      const total = Number.parseInt(text(requests), 10) || 1;
      return { successful_hits: Math.max(total, 1), failed_hits: 0, hits: Math.max(total, 1) };
    }

    function parseCsvTimestampMs(value) {
      const textValue = text(value).trim();
      if (!textValue) return null;
      const parsed = Date.parse(textValue);
      return Number.isFinite(parsed) ? parsed : null;
    }

    function csvImportStatusLine(successMessage, pathValue, result) {
      return successMessage + ': ' + pathValue + ' (' + (result.imported_count || 0) + '/' + (result.requested_count || result.imported_count || 0) + ' imported, ' + (result.skipped_count || 0) + ' skipped)';
    }

    function csvEscape(value) {
      const textValue = text(value);
      if (/[",\r\n]/.test(textValue)) {
        return '"' + textValue.replaceAll('"', '""') + '"';
      }
      return textValue;
    }

    async function toggleTrackedApp(id) {
      const snapshot = state.snapshot;
      if (snapshot.app_settings?.ui_enable_all_overlay) return setStatus('Disable Enable all before toggling one app.', true);
      const app = (snapshot.tracked_apps || []).find((item) => item.id === id);
      if (!app) return;
      await post('/v1/tracked-apps/toggle', { tracked_app_id: app.id, enabled: !app.enabled });
    }

    async function deleteTrackedApp(event, id) {
      event.stopPropagation();
      if (!confirm('Remove this app from tracked apps? Captured observations stay in local storage.')) return;
      await post('/v1/tracked-apps/delete', { tracked_app_id: id });
    }

    async function toggleEnableAll() {
      const enabled = !Boolean(state.snapshot?.app_settings?.ui_enable_all_overlay);
      await post('/v1/settings', { key: 'ui.enable_all_overlay', value: String(enabled) });
    }

    async function toggleWebAccess() {
      const enabled = !Boolean(state.snapshot?.app_settings?.web_access_localhost);
      await post('/v1/settings', { key: 'web.localhost_enabled', value: String(enabled) }, { skipSnapshotRefresh: !enabled });
      if (state.snapshot?.app_settings) {
        state.snapshot.app_settings.web_access_localhost = enabled;
      }
      if (enabled) {
        state.browserAccessDisabledOverlay = false;
        await loadWebUrl();
        await loadSnapshot();
      } else {
        state.browserAccessDisabledOverlay = true;
        render();
      }
    }

    async function toggleDomainCapture() {
      const enabled = !Boolean(state.snapshot?.app_settings?.domain_capture_enabled);
      if (enabled && !Boolean(state.snapshot?.runtime_status?.is_elevated)) {
        setFooterMessage(t('footer.event.domain_capture_admin_required', 'Run NetStitch as administrator to use advanced monitoring'), true);
        return;
      }
      await post('/v1/settings', { key: 'domain_capture.enabled', value: String(enabled) });
      if (state.snapshot?.app_settings) {
        state.snapshot.app_settings.domain_capture_enabled = enabled;
      }
      setFooterMessage(enabled
        ? t('footer.event.domain_capture_enabled', 'Advanced monitoring enabled')
        : t('footer.event.domain_capture_disabled', 'Advanced monitoring disabled'));
      render();
    }

    async function setLanguage(languageCode) {
      await post('/v1/settings', { key: 'ui.language', value: languageCode });
      closeLanguageMenu();
    }

    document.addEventListener('click', (event) => {
      if (!event.target.closest('.language-select-shell')) {
        closeLanguageMenu();
      }
    });

    document.addEventListener('input', (event) => {
      if (event.target && event.target.id && event.target.dataset?.clearButton === 'true') {
        syncClearButtonStates();
      }
    });

    document.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter') return;
      const target = event.target;
      if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) return;
      if (target.dataset?.commitOnEnter !== 'true') return;
      if (target.getAttribute('onkeydown')) return;
      pulseInput(target);
      target.dispatchEvent(new Event('change', { bubbles: true }));
    });

    window.addEventListener('error', (event) => {
      if (!event) return;
      const message = text(event.message || event.error?.message || '');
      if (message) setStatus(message, true);
    });

    window.addEventListener('unhandledrejection', (event) => {
      const message = text(event?.reason?.message || event?.reason || '');
      if (message) setStatus(message, true);
    });

    async function confirmObservation(id, confirmed) {
      await setObservationRowsSelection(confirmed ? [id] : [], confirmed ? [] : [id]);
    }

    function observationRowSelected(row) {
      if (row?.is_exported) return false;
      const id = Number(row?.id);
      if (Number.isFinite(id)) {
        if (state.observationSelectionInFlightSelect.has(id)) return true;
        if (state.observationSelectionInFlightDeselect.has(id)) return false;
      }
      return Boolean(row?.is_confirmed && !row?.is_exported);
    }

    function queueObservationSelectionOverlay(confirmIds, unconfirmIds) {
      for (const rawId of confirmIds || []) {
        const id = Number(rawId);
        if (!Number.isFinite(id)) continue;
        state.observationSelectionInFlightDeselect.delete(id);
        state.observationSelectionInFlightSelect.add(id);
      }
      for (const rawId of unconfirmIds || []) {
        const id = Number(rawId);
        if (!Number.isFinite(id)) continue;
        state.observationSelectionInFlightSelect.delete(id);
        state.observationSelectionInFlightDeselect.add(id);
      }
    }

    function reconcileObservationSelectionOverlay(snapshot) {
      const rowsById = new Map((snapshot?.observed_endpoints || []).map((row) => [Number(row.id), row]));
      for (const id of Array.from(state.observationSelectionInFlightSelect)) {
        const row = rowsById.get(Number(id));
        if (!row || Boolean(row.is_confirmed && !row.is_exported)) {
          state.observationSelectionInFlightSelect.delete(id);
        }
      }
      for (const id of Array.from(state.observationSelectionInFlightDeselect)) {
        const row = rowsById.get(Number(id));
        if (!row || !Boolean(row.is_confirmed && !row.is_exported)) {
          state.observationSelectionInFlightDeselect.delete(id);
        }
      }
    }

    async function setObservationRowsSelection(confirmIds, unconfirmIds) {
      const confirmList = Array.from(new Set((confirmIds || []).filter((id) => id !== null && id !== undefined)));
      const unconfirmList = Array.from(new Set((unconfirmIds || []).filter((id) => id !== null && id !== undefined)));
      queueObservationSelectionOverlay(confirmList, unconfirmList);
      renderObservations(state.snapshot);
      try {
        if (confirmList.length) {
          await post('/v1/observations/confirm', { endpoint_ids: confirmList, confirmed: true }, { skipSnapshotRefresh: true });
        }
        if (unconfirmList.length) {
          await post('/v1/observations/confirm', { endpoint_ids: unconfirmList, confirmed: false }, { skipSnapshotRefresh: true });
        }
        if (confirmList.length || unconfirmList.length) {
          await loadSnapshot();
        }
      } catch (error) {
        for (const id of confirmList) state.observationSelectionInFlightSelect.delete(Number(id));
        for (const id of unconfirmList) state.observationSelectionInFlightDeselect.delete(Number(id));
        renderObservations(state.snapshot);
        throw error;
      }
    }

    async function applyObservationRowSelection(event, rowId, selected) {
      const id = Number(rowId);
      if (!Number.isFinite(id)) return;
      const rows = filteredObservations(state.snapshot);
      const confirmIds = [];
      const unconfirmIds = [];
      const rangeSelection = Boolean(event?.shiftKey && state.lastSelectedObservationId !== null && state.lastSelectedObservationId !== undefined);
      if (rangeSelection) {
        const ids = rows.map((row) => row.id);
        const start = ids.indexOf(state.lastSelectedObservationId);
        const end = ids.indexOf(id);
        if (start >= 0 && end >= 0) {
          const from = Math.min(start, end);
          const to = Math.max(start, end);
          for (const row of rows.slice(from, to + 1)) {
            if (observationRowSelected(row)) {
              unconfirmIds.push(row.id);
            } else {
              confirmIds.push(row.id);
            }
          }
        } else if (selected) {
          unconfirmIds.push(id);
        } else {
          confirmIds.push(id);
        }
      } else if (selected) {
        unconfirmIds.push(id);
      } else {
        confirmIds.push(id);
      }
      state.lastSelectedObservationId = id;
      await setObservationRowsSelection(confirmIds, unconfirmIds);
    }

    async function handleObservationRowClick(event, rowId, selected) {
      if (!event?.shiftKey && !event?.ctrlKey && !event?.metaKey) return;
      event.preventDefault();
      await applyObservationRowSelection(event, rowId, selected);
    }

    async function handleObservationRowAction(event, rowId, selected) {
      event?.stopPropagation();
      if (event?.shiftKey || event?.ctrlKey || event?.metaKey) {
        event.preventDefault();
        await applyObservationRowSelection(event, rowId, selected);
        return;
      }
      const id = Number(rowId);
      if (!Number.isFinite(id)) return;
      state.lastSelectedObservationId = id;
      await setObservationRowsSelection(selected ? [] : [id], selected ? [id] : []);
    }

    function requestDeleteObservation(event, id) {
      event.stopPropagation();
      const observation = (state.snapshot?.observed_endpoints || []).find((item) => item.id === id);
      if (!observation) return;
      state.pendingObservationDelete = observation;
      renderObservationDeleteModal();
    }

    function renderObservationDeleteModal() {
      const observation = state.pendingObservationDelete;
      if (!observation) return renderDeleteConfirmModal('observation-delete-modal', 'observation-delete-modal-body', null);
      const endpoint = observation.remote_ip + ':' + observation.remote_port + ' ' + observation.protocol;
      const connectionState = observation.connection_state + ' (' + (observation.failed_hits || 0) + '/' + (observation.successful_hits || 0) + ')';
      renderDeleteConfirmModal('observation-delete-modal', 'observation-delete-modal-body', [
        { text: t('dialog.delete_observation.app', 'App') + ': ' + appName(state.snapshot, observation.tracked_app_id, observation), muted: false },
        { text: t('dialog.delete_observation.endpoint', 'Endpoint') + ': ' + endpoint, muted: false },
        { text: t('dialog.delete_observation.state', 'State') + ': ' + connectionState, muted: true }
      ]);
    }

    function cancelDeleteObservation() {
      state.pendingObservationDelete = null;
      renderObservationDeleteModal();
    }

    async function confirmDeleteObservation() {
      const observation = state.pendingObservationDelete;
      if (!observation) return;
      await post('/v1/observations/delete', { endpoint_id: observation.id });
      state.pendingObservationDelete = null;
      renderObservationDeleteModal();
      await loadSnapshot();
    }

    function requestClearMonitoring() {
      if (state.cloudPanel === 'import') {
        requestClearCloudRows();
        return;
      }
      const count = (state.snapshot?.observed_endpoints || []).length;
      if (!count) {
        setFooterMessage(localizedClearMonitoringLabel() + ': 0');
        return;
      }
      state.pendingClearMonitoringScope = 'monitoring';
      state.pendingClearMonitoringCount = count;
      renderClearMonitoringModal();
    }

    function requestClearCloudRows() {
      const ids = selectedCloudRows().map((row) => text(row.row_id));
      if (!ids.length) {
        setFooterMessage(t('dialog.cloud_sync.delete_selected', 'Delete selected cloud rows') + ': 0');
        return;
      }
      state.pendingClearMonitoringScope = 'cloud-import';
      state.pendingClearMonitoringCount = ids.length;
      renderClearMonitoringModal();
    }

    function renderClearMonitoringModal() {
      const count = state.pendingClearMonitoringCount;
      if (!count) return renderDeleteConfirmModal('clear-monitoring-modal', 'clear-monitoring-modal-body', null);
      renderDeleteConfirmModal('clear-monitoring-modal', 'clear-monitoring-modal-body', [
        { text: t('dialog.clear_monitoring.visible', 'Total rows') + ': ' + count, muted: false }
      ]);
    }

    function cancelClearMonitoring() {
      state.pendingClearMonitoringCount = null;
      state.pendingClearMonitoringScope = 'monitoring';
      renderClearMonitoringModal();
    }

    async function confirmClearMonitoring() {
      if (state.pendingClearMonitoringScope === 'cloud-import') {
        const selected = new Set(selectedCloudRows().map((row) => text(row.row_id)));
        if (!selected.size) {
          state.pendingClearMonitoringCount = null;
          state.pendingClearMonitoringScope = 'monitoring';
          renderClearMonitoringModal();
          setFooterMessage(t('dialog.cloud_sync.delete_selected', 'Delete selected cloud rows') + ': 0');
          return;
        }
        state.cloud.rows = state.cloud.rows.filter((row) => !selected.has(text(row.row_id)));
        for (const id of selected) state.cloud.selectedRows.delete(id);
        state.pendingClearMonitoringCount = null;
        state.pendingClearMonitoringScope = 'monitoring';
        renderClearMonitoringModal();
        renderCloudRows();
        renderHeaderFilters(state.snapshot);
        setFooterMessage(t('dialog.cloud_sync.delete_selected', 'Delete selected cloud rows') + ': ' + selected.size);
        return;
      }
      const ids = (state.snapshot?.observed_endpoints || []).map((row) => row.id);
      if (!ids.length) {
        state.pendingClearMonitoringCount = null;
        state.pendingClearMonitoringScope = 'monitoring';
        renderClearMonitoringModal();
        setFooterMessage(localizedClearMonitoringLabel() + ': 0');
        return;
      }
      const result = await post('/v1/observations/delete-batch', { endpoint_ids: ids });
      state.pendingClearMonitoringCount = null;
      state.pendingClearMonitoringScope = 'monitoring';
      renderClearMonitoringModal();
      setFooterMessage(localizedClearMonitoringLabel() + ': ' + text(result?.deleted ?? 0));
      await loadSnapshot();
    }

    async function confirmFiltered() {
      if (state.cloudPanel === 'import') {
        const ids = visibleCloudRows().map((row) => text(row.row_id));
        state.cloud.selectedRows = new Set([...state.cloud.selectedRows, ...ids]);
        state.cloud.lastSelectedRowId = ids.length ? ids[ids.length - 1] : '';
        renderCloudRows();
        renderHeaderFilters(state.snapshot);
        setFooterMessage(t('action.confirm_filtered', 'Confirm all visible') + ': ' + ids.length);
        return;
      }
      const ids = filteredObservations(state.snapshot).map((row) => row.id);
      await setObservationRowsSelection(ids, []);
      setFooterMessage(t('footer.event.confirmed_filtered_prefix', 'Confirmed filtered observations:') + ' ' + ids.length);
    }

    async function unconfirmFiltered() {
      if (state.cloudPanel === 'import') {
        const ids = new Set(visibleCloudRows().map((row) => text(row.row_id)));
        for (const id of ids) state.cloud.selectedRows.delete(id);
        state.cloud.lastSelectedRowId = '';
        renderCloudRows();
        renderHeaderFilters(state.snapshot);
        setFooterMessage(t('action.unconfirm_filtered', 'Unconfirm all visible') + ': ' + ids.size);
        return;
      }
      const ids = (state.snapshot?.observed_endpoints || [])
        .filter((row) => Boolean(row.is_confirmed || row.is_exported))
        .map((row) => row.id);
      if (!ids.length) {
        setFooterMessage(t('action.unconfirm_filtered', 'Unconfirm all visible') + ': 0');
        return;
      }
      await setObservationRowsSelection([], ids);
      state.profileExport.advancedDomains = false;
      state.profileExport.advancedManualDomains = '';
      state.profileExport.advancedReadyForExport = false;
      state.profileExport.advancedDomainCleanupRequested = false;
      persistProfileExportUiState();
      setFooterMessage(t('action.unconfirm_filtered', 'Unconfirm all visible') + ': ' + ids.length);
    }

    async function clearMonitoring() {
      requestClearMonitoring();
    }

    async function ignoreAddress(address) {
      await post('/v1/ignored-addresses', { address_pattern: address });
    }

    function requestDeleteIgnoredAddress(event, id) {
      event.stopPropagation();
      const rule = (state.snapshot?.ignored_addresses || []).find((item) => item.id === id);
      if (!rule) return;
      state.pendingIgnoredAddressDelete = rule;
      renderIgnoredDeleteModal();
    }

    function renderIgnoredDeleteModal() {
      const rule = state.pendingIgnoredAddressDelete;
      if (!rule) return renderDeleteConfirmModal('ignored-delete-modal', 'ignored-delete-modal-body', null);
      const tooltip = ignoredAddressTooltip(rule);
      renderDeleteConfirmModal('ignored-delete-modal', 'ignored-delete-modal-body', [
        { text: t('dialog.delete_ignored.address', 'Address') + ': ' + rule.address_pattern, muted: false },
        ...text(tooltip).split('\n').filter(Boolean).map((line) => ({ text: line, muted: true }))
      ]);
    }

    function ignoredAddressDeleteNote(pattern) {
      if (ignoredRuleIsLoopback(pattern)) {
        return t('dialog.delete_ignored.localhost_note', 'This is a localhost rule. Removing it can show local loopback traffic such as 127.0.0.1 or ::1.');
      }
      if (ignoredRuleIsLocalMachineCandidate(pattern)) {
        return t('dialog.delete_ignored.local_ip_note', 'This is a local machine IP rule. Removing it can show traffic to your own LAN address.');
      }
      return '';
    }

    function renderDeleteConfirmModal(modalId, bodyId, lines) {
      const modal = document.getElementById(modalId);
      const body = document.getElementById(bodyId);
      if (!modal || !body) return;
      modal.hidden = !lines;
      body.innerHTML = (lines || []).map((line) =>
        '<p class="' + (line.muted ? 'subtle' : 'small') + '">' + html(line.text) + '</p>'
      ).join('');
    }

    function ignoredRuleIpAndPrefix(pattern) {
      const trimmed = text(pattern).trim();
      const parts = trimmed.split('/');
      if (parts.length === 2) {
        const prefix = Number(parts[1]);
        if (Number.isNaN(prefix)) return null;
        return { ip: parts[0].toLowerCase(), prefix };
      }
      return { ip: trimmed.toLowerCase(), prefix: null };
    }

    function ignoredRuleIsLoopback(pattern) {
      const parsed = ignoredRuleIpAndPrefix(pattern);
      if (!parsed) return false;
      return parsed.ip === '::1' || parsed.ip.startsWith('127.');
    }

    function ignoredRuleIsLocalMachineCandidate(pattern) {
      const parsed = ignoredRuleIpAndPrefix(pattern);
      if (!parsed || ignoredRuleIsLoopback(pattern) || parsed.ip === '0.0.0.0' || parsed.ip === '::') return false;
      const isExactHostRule = parsed.prefix === 32 || parsed.prefix === 128 || parsed.prefix === null;
      if (!isExactHostRule) return false;
      return parsed.ip === localMachineIpForWebUrl();
    }

    function localMachineIpForWebUrl() {
      try {
        const url = new URL(state.webUrl?.url || '', window.location.origin);
        return url.hostname.toLowerCase().replace(/^\[/, '').replace(/\]$/, '');
      } catch (_error) {
        return '';
      }
    }

    function cancelDeleteIgnoredAddress() {
      state.pendingIgnoredAddressDelete = null;
      renderIgnoredDeleteModal();
    }

    async function confirmDeleteIgnoredAddress() {
      const rule = state.pendingIgnoredAddressDelete;
      if (!rule) return;
      await post('/v1/ignored-addresses/delete', { ignored_address_id: rule.id });
      state.pendingIgnoredAddressDelete = null;
      renderIgnoredDeleteModal();
    }

    async function downloadIntegration(providerId) {
      const generation = (state.integrationDownload.generation || 0) + 1;
      state.integrationDownload = {
        active: true,
        visible: true,
        cancelled: false,
        generation,
        stage: 'preparing',
        percent: null,
        downloadedBytes: null,
        totalBytes: null,
        extractedEntries: null,
        totalEntries: null,
        message: null,
        repoRoot: null
      };
      pushStatusLine(integrationProgressFooterLine(state.integrationDownload));
      renderIntegrationModal(state.snapshot);

      try {
        const accepted = await post('/v1/integrations/download/start', {
          module_id: selectedIntegrationModuleIdForRequest(),
          provider_id: providerId
        });
        if (state.integrationDownload.generation !== generation || state.integrationDownload.cancelled) return;
        applyIntegrationDownloadProgress(accepted, generation);
        await pollIntegrationDownloadProgress(generation);
        if (state.integrationDownload.generation !== generation || state.integrationDownload.cancelled) return;
        const integrationDir = document.getElementById('integration-dir');
        if (integrationDir && state.integrationDownload.repoRoot) {
          integrationDir.value = state.integrationDownload.repoRoot;
        }
        await loadSnapshot();
        renderIntegrationModal(state.snapshot);
      } catch (error) {
        if (state.integrationDownload.generation === generation && !state.integrationDownload.cancelled) {
          state.integrationDownload.active = false;
          state.integrationDownload.visible = true;
          state.integrationDownload.stage = 'failed';
          state.integrationDownload.message = text(error?.message || error);
          pushStatusLine(integrationProgressFooterLine(state.integrationDownload));
          renderIntegrationModal(state.snapshot);
        }
      }

      if (state.integrationDownload.generation === generation && !state.integrationDownload.cancelled) {
        state.integrationDownload.active = false;
        state.integrationDownload.visible = true;
        renderIntegrationModal(state.snapshot);
      }
    }

    function defaultProfileExportPath() {
      const integration = state.snapshot?.integration_status || {};
      const profilePaths = Array.isArray(integration.profile_paths) ? integration.profile_paths : [];
      const firstProfile = profilePaths.map(text).find(Boolean);
      return text(firstProfile || integration.export_path || integration.repo_root || '');
    }

    function profileExportProfileOptions() {
      const integration = state.snapshot?.integration_status || {};
      const options = (Array.isArray(integration.profile_paths) ? integration.profile_paths : [])
        .map(text)
        .filter(Boolean);
      const fallback = defaultProfileExportPath();
      if (!options.length && fallback) options.push(fallback);
      return Array.from(new Set(options)).sort((left, right) => {
        if (left === fallback) return -1;
        if (right === fallback) return 1;
        return profileExportProfileOptionLabel(left).localeCompare(profileExportProfileOptionLabel(right));
      });
    }

    function profileExportProfileOptionLabel(path) {
      const value = text(path);
      const normalized = value.replaceAll('/', '\\');
      const parts = normalized.split('\\').filter(Boolean);
      return parts.pop() || value;
    }

    function profileExportRepoRootKey() {
      const repoRoot = text(state.snapshot?.integration_status?.repo_root).trim();
      if (!repoRoot || repoRoot.toLowerCase() === 'not configured' || repoRoot.toLowerCase() === 'not found') {
        return '';
      }
      return repoRoot.replaceAll('/', '\\').replace(/[\\]+$/g, '').toLowerCase();
    }

    function defaultProfileExportUiState(repoRootKey, defaultPath) {
      return {
        repo_root_key: repoRootKey,
        mode: 'attach_netstitch_lists',
        attach_profile_path: defaultPath,
        patch_profile_path: defaultPath,
        merge_profile_path: defaultPath,
        generated_profile_name: 'NetStitch',
        dangerous_confirmed: false,
        advanced_create_rules_for_uncovered: false,
        advanced_add_ips_to_exclude: false,
        advanced_use_whois_ranges_for_export: false,
        advanced_add_detected_domains: false,
        advanced_manual_domains: '',
        advanced_template_rule_source: '',
        advanced_ready_for_export: false,
        advanced_domain_cleanup_requested: false
      };
    }

    function applyProfileExportUiState(saved) {
      state.profileExport.repoRootKey = text(saved?.repo_root_key);
      state.profileExport.mode = text(saved?.mode, 'attach_netstitch_lists');
      state.profileExport.selectedProfilePaths = {
        attach_netstitch_lists: text(saved?.attach_profile_path),
        patch_selected_profile: text(saved?.patch_profile_path),
        merge_into_existing_lists: text(saved?.merge_profile_path)
      };
      state.profileExport.generatedProfileName = text(saved?.generated_profile_name);
      state.profileExport.dangerousConfirmed = Boolean(saved?.dangerous_confirmed);
      state.profileExport.advancedCreateRules = Boolean(saved?.advanced_create_rules_for_uncovered);
      state.profileExport.advancedExcludeIps = Boolean(saved?.advanced_add_ips_to_exclude);
      state.profileExport.advancedWhoisRanges = Boolean(saved?.advanced_use_whois_ranges_for_export);
      state.profileExport.advancedDomains = Boolean(saved?.advanced_add_detected_domains);
      state.profileExport.advancedManualDomains = text(saved?.advanced_manual_domains);
      state.profileExport.advancedTemplateRule = text(saved?.advanced_template_rule_source);
      state.profileExport.advancedReadyForExport = Boolean(saved?.advanced_ready_for_export);
      state.profileExport.advancedDomainCleanupRequested = Boolean(saved?.advanced_domain_cleanup_requested);
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
    }

    function currentProfileExportUiState() {
      syncProfileExportInputs();
      return {
        repo_root_key: text(state.profileExport.repoRootKey),
        mode: state.profileExport.mode || 'attach_netstitch_lists',
        attach_profile_path: profileExportSelectedProfilePath('attach_netstitch_lists'),
        patch_profile_path: profileExportSelectedProfilePath('patch_selected_profile'),
        merge_profile_path: profileExportSelectedProfilePath('merge_into_existing_lists'),
        generated_profile_name: text(state.profileExport.generatedProfileName),
        dangerous_confirmed: Boolean(state.profileExport.dangerousConfirmed),
        advanced_create_rules_for_uncovered: Boolean(state.profileExport.advancedCreateRules),
        advanced_add_ips_to_exclude: Boolean(state.profileExport.advancedExcludeIps),
        advanced_use_whois_ranges_for_export: Boolean(state.profileExport.advancedWhoisRanges),
        advanced_add_detected_domains: Boolean(state.profileExport.advancedDomains),
        advanced_manual_domains: text(state.profileExport.advancedManualDomains),
        advanced_template_rule_source: text(state.profileExport.advancedTemplateRule),
        advanced_ready_for_export: Boolean(state.profileExport.advancedReadyForExport),
        advanced_domain_cleanup_requested: Boolean(state.profileExport.advancedDomainCleanupRequested)
      };
    }

    function persistProfileExportUiState() {
      const value = currentProfileExportUiState();
      if (!value.repo_root_key) return;
      if (state.snapshot?.app_settings) {
        state.snapshot.app_settings.profile_export_ui_state = value;
      }
      post('/v1/settings', { key: 'profile_export.ui_state', value: JSON.stringify(value) }, { skipSnapshotRefresh: true }).catch(() => {});
    }

    function resetProfileExportControlsWhenIntegrationChanged() {
      const repoRootKey = profileExportRepoRootKey();
      if (repoRootKey === state.profileExport.repoRootKey) return;
      const defaultPath = defaultProfileExportPath();
      const saved = state.snapshot?.app_settings?.profile_export_ui_state;
      if (saved?.repo_root_key === repoRootKey) {
        applyProfileExportUiState(saved);
      } else {
        applyProfileExportUiState(defaultProfileExportUiState(repoRootKey, defaultPath));
        persistProfileExportUiState();
      }
    }

    function profileExportSelectedProfilePath(mode = state.profileExport.mode || 'attach_netstitch_lists') {
      const paths = state.profileExport.selectedProfilePaths || {};
      return text(paths[mode], text(state.profileExport.selectedProfilePath));
    }

    function setProfileExportSelectedProfilePath(mode, value) {
      state.profileExport.selectedProfilePaths = {
        attach_netstitch_lists: profileExportSelectedProfilePath('attach_netstitch_lists'),
        patch_selected_profile: profileExportSelectedProfilePath('patch_selected_profile'),
        merge_into_existing_lists: profileExportSelectedProfilePath('merge_into_existing_lists'),
        [mode]: text(value)
      };
      state.profileExport.selectedProfilePath = state.profileExport.selectedProfilePaths[state.profileExport.mode || 'attach_netstitch_lists'];
    }

    function openProfileExportDialog() {
      resetProfileExportControlsWhenIntegrationChanged();
      if (!state.snapshot?.integration_status?.is_available) {
        state.profileExportReturnToModule = false;
        return false;
      }
      state.profileExport.open = true;
      state.profileExport.feedback = '';
      state.profileExport.preview = null;
      pushStatusLine(t('footer.event.export_requested', 'Export requested for confirmed IPs'));
      renderProfileExportDialog();
      renderModuleHeader();
      refreshProfileExportPreview();
      return true;
    }

    function closeProfileExportDialog() {
      const shouldReturnToModule = Boolean(state.profileExportReturnToModule);
      state.profileExportReturnToModule = false;
      state.profileExport.open = false;
      closeProfileExportAdvancedWizard();
      renderProfileExportDialog();
      if (shouldReturnToModule) returnToIntegrationModulePanel();
      renderModuleHeader();
    }

    function openProfileExportAdvancedWizard() {
      const modal = document.getElementById('profile-export-advanced-wizard-modal');
      if (modal) modal.classList.toggle('module-overlay-backdrop', Boolean(state.profileExportReturnToModule));
      if (modal) modal.hidden = false;
      renderProfileExportAdvancedWizard();
      renderModuleHeader();
    }

    function closeProfileExportAdvancedWizard() {
      const modal = document.getElementById('profile-export-advanced-wizard-modal');
      if (modal) modal.hidden = true;
      renderModuleHeader();
    }

    function selectProfileExportMode(mode) {
      state.profileExport.mode = mode;
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
      persistProfileExportUiState();
      renderProfileExportDialog();
      refreshProfileExportPreview();
    }

    function toggleProfileExportDangerous(checked) {
      state.profileExport.dangerousConfirmed = checked === undefined
        ? !state.profileExport.dangerousConfirmed
        : Boolean(checked);
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
      persistProfileExportUiState();
      renderProfileExportDialog();
      refreshProfileExportPreview();
    }

    function profileExportSelectedInput() {
      const mode = state.profileExport.mode || 'attach_netstitch_lists';
      if (mode === 'patch_selected_profile') return document.getElementById('profile-export-patch-profile-input');
      if (mode === 'merge_into_existing_lists') return document.getElementById('profile-export-merge-profile-input');
      return document.getElementById('profile-export-attach-profile-input');
    }

    function syncProfileExportInputs() {
      const inputByMode = {
        attach_netstitch_lists: document.getElementById('profile-export-attach-profile-input'),
        patch_selected_profile: document.getElementById('profile-export-patch-profile-input'),
        merge_into_existing_lists: document.getElementById('profile-export-merge-profile-input')
      };
      const generatedInput = document.getElementById('profile-export-attach-generated-name-input');
      for (const [mode, input] of Object.entries(inputByMode)) {
        if (input) setProfileExportSelectedProfilePath(mode, input.value);
      }
      if (generatedInput) state.profileExport.generatedProfileName = generatedInput.value;
    }

    function updateProfileExportDraft() {
      syncProfileExportInputs();
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
    }

    function updateProfileExportInput() {
      syncProfileExportInputs();
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
      persistProfileExportUiState();
      renderProfileExportDialog();
      refreshProfileExportPreview();
    }

    function applyProfileExportInputOnEnter(event) {
      if (!event || event.key !== 'Enter') return;
      pulseInput(event.currentTarget);
      updateProfileExportInput();
    }

    function clearProfileExportPath() {
      const selectedInput = profileExportSelectedInput();
      setProfileExportSelectedProfilePath(state.profileExport.mode || 'attach_netstitch_lists', '');
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
      if (selectedInput) selectedInput.value = '';
      persistProfileExportUiState();
      renderProfileExportDialog();
      syncClearButtonStates();
      refreshProfileExportPreview();
    }

    function browseProfileExportProfile() {
      state.profileExport.feedback = '';
      state.profileExport.preview = null;
      renderProfileExportDialog();
      const currentPath = profileExportSelectedProfilePath(state.profileExport.mode || 'attach_netstitch_lists');
      openServerFilePicker({
        intent: 'profile_export',
        mode: 'profile',
        title: t('dialog.file_picker.profile_title', 'Choose integration profile'),
        help: t('dialog.file_picker.profile_help', 'Select a .bat, .cmd, .conf, or .txt profile on the machine where NetStitch is running.'),
        initialPath: pickerInitialDirectory(currentPath)
      });
    }

    function selectProfileExportProfile(value) {
      const selected = text(value);
      if (!selected) return;
      setProfileExportSelectedProfilePath(state.profileExport.mode || 'attach_netstitch_lists', selected);
      const input = profileExportSelectedInput();
      if (input) input.value = selected;
      state.profileExport.preview = null;
      state.profileExport.feedback = '';
      persistProfileExportUiState();
      renderProfileExportDialog();
      refreshProfileExportPreview();
    }

    function currentProfileExportProviderId() {
      const integration = state.snapshot?.integration_status || {};
      const providerId = text(integration.provider_id);
      if (providerId) return providerId;
      return 'default';
    }

    function renderProfileExportModeCopy() {
      renderText(
        'profile-export-mode-attach-button',
        'dialog.profile_export.mode_attach',
        'Attach'
      );
      renderText(
        'profile-export-mode-patch-button',
        'dialog.profile_export.mode_patch',
        'Patch'
      );
      renderText(
        'profile-export-mode-merge-button',
        'dialog.profile_export.mode_merge',
        'Merge'
      );
      renderText(
        'profile-export-attach-note',
        'dialog.profile_export.note_attach',
        'Creates module-owned export files without changing the selected source profile.'
      );
      renderText(
        'profile-export-patch-note',
        'dialog.profile_export.note_patch',
        'Links module-owned lists from the selected profile; existing rules are not rewritten.'
      );
      renderText(
        'profile-export-merge-note',
        'dialog.profile_export.note_merge',
        'Writes selected data into existing lists only after explicit confirmation.'
      );
    }

    function profileExportRequest() {
      syncProfileExportInputs();
      const integration = state.snapshot?.integration_status || {};
      const repoRoot = text(integration.repo_root).trim();
      const providerId = currentProfileExportProviderId();
      const selectedProfilePath = text(profileExportSelectedProfilePath()).trim();
      const generatedProfileName = text(state.profileExport.generatedProfileName).trim();
      return {
        provider_id: providerId,
        repo_root: repoRoot,
        selected_profile_path: selectedProfilePath || null,
        generated_profile_name: generatedProfileName || null,
        mode: state.profileExport.mode || 'attach_netstitch_lists',
        dangerous_confirmed: Boolean(state.profileExport.dangerousConfirmed)
      };
    }

    function renderProfileExportDialog() {
      const modal = document.getElementById('export-profile-modal');
      if (modal) {
        modal.hidden = !state.profileExport.open;
        modal.classList.toggle('module-overlay-backdrop', Boolean(state.profileExportReturnToModule));
      }
      renderModuleHeader();
      if (!modal || !state.profileExport.open) return;
      const title = document.getElementById('export-profile-modal-title');
      if (title) {
        const module = selectedIntegrationModule(state.snapshot);
        title.textContent = state.profileExportReturnToModule
          ? text(module?.export_title, t('dialog.profile_export.title', 'Profile export'))
          : t('dialog.profile_export.title', 'Profile export');
      }

      for (const panel of document.querySelectorAll('[data-profile-export-panel]')) {
        panel.hidden = panel.getAttribute('data-profile-export-panel') !== state.profileExport.mode;
      }
      for (const input of document.querySelectorAll('[data-ui-entity="netstitch-ui-profile-export-profile-input"]')) {
        const mode = input.id.includes('patch')
          ? 'patch_selected_profile'
          : input.id.includes('merge')
            ? 'merge_into_existing_lists'
            : 'attach_netstitch_lists';
        if (document.activeElement !== input) input.value = text(profileExportSelectedProfilePath(mode));
      }
      const generatedInput = document.getElementById('profile-export-attach-generated-name-input');
      const dangerous = document.getElementById('profile-export-dangerous-switch');
      if (generatedInput && document.activeElement !== generatedInput) generatedInput.value = text(state.profileExport.generatedProfileName, 'NetStitch');
      if (dangerous) {
        const enabled = Boolean(state.profileExport.dangerousConfirmed);
        dangerous.className = 'input-box switch' + (enabled ? ' switch--on' : '');
        dangerous.setAttribute('aria-checked', String(enabled));
      }
      renderProfileExportProfileSelects();
      renderProfileExportModeCopy();

      for (const button of document.querySelectorAll('[data-profile-export-mode]')) {
        const active = button.getAttribute('data-profile-export-mode') === state.profileExport.mode;
        button.setAttribute('aria-pressed', String(active));
        button.setAttribute('aria-selected', String(active));
      }

      const feedback = document.getElementById('profile-export-feedback');
      if (feedback) feedback.textContent = text(state.profileExport.feedback);

      const applyButton = document.getElementById('profile-export-apply-button');
      const analyzeButton = document.getElementById('profile-export-analyze-button');
      const dangerousBlocked = state.profileExport.mode === 'merge_into_existing_lists' && !state.profileExport.dangerousConfirmed;
      if (analyzeButton) analyzeButton.disabled = dangerousBlocked;
      if (applyButton) applyButton.disabled = dangerousBlocked;
      syncClearButtonStates();

      const preview = document.getElementById('profile-export-preview');
      if (!preview) return;
      preview.innerHTML = dangerousBlocked ? '' : profileExportPreviewHtml(state.profileExport.preview);
      renderProfileExportAdvancedWizard();
    }

    function renderProfileExportProfileSelects() {
      const options = profileExportProfileOptions();
      for (const select of document.querySelectorAll('[data-ui-entity="netstitch-ui-profile-export-profile-select"]')) {
        const noProfiles = !options.length;
        const mode = select.id.includes('patch')
          ? 'patch_selected_profile'
          : select.id.includes('merge')
            ? 'merge_into_existing_lists'
            : 'attach_netstitch_lists';
        const selected = text(profileExportSelectedProfilePath(mode) || defaultProfileExportPath());
        select.innerHTML = noProfiles
          ? '<option value="">' + html(t('dialog.profile_export.no_profiles', 'No profiles detected')) + '</option>'
          : options.map((path) => '<option value="' + html(path) + '">' + html(profileExportProfileOptionLabel(path)) + '</option>').join('');
        select.value = options.includes(selected) ? selected : '';
        select.disabled = noProfiles;
      }
    }

    function profileExportPreviewHtml(plan) {
      if (!plan) {
        return '';
      }
      return [
        profileExportSummaryHtml(plan),
        profileExportUncoveredTargetsHtml(plan.uncovered_targets || []),
        profileExportFileChangesHtml([...(plan.file_changes || []), ...(plan.advanced_file_changes || [])]),
        profileExportWarningsHtml(profileExportVisibleWarnings(plan.warnings || [])),
        profileExportMatchedRulesHtml(plan.matched_rules || [])
      ].join('');
    }

    function profileExportSummaryHtml(plan) {
      const addressSkipped = Number((plan.existing_ips || []).length || 0)
        + Number(plan.skipped_unconfirmed || 0)
        + Number(plan.skipped_exported || 0)
        + Number(plan.skipped_duplicates || 0);
      return '<div class="profile-export-preview__group" id="profile-export-summary">'
        + profileExportSummarySectionHtml(t('dialog.profile_export.group_addresses', 'Addresses'), [
          [t('dialog.profile_export.selected', 'Selected'), text(plan.exported_count, '0'), 'total'],
          [t('dialog.profile_export.will_add', 'Will be added'), String((plan.new_ips || []).length), 'ready'],
          [t('dialog.profile_export.skipped', 'Skipped'), String(addressSkipped), 'excluded']
        ])
        + profileExportSummarySectionHtml(t('dialog.profile_export.group_domains', 'Domains'), [
          [t('dialog.profile_export.selected', 'Selected'), text(plan.advanced_domain_count, '0'), 'total'],
          [t('dialog.profile_export.will_add', 'Will be added'), text(plan.advanced_new_domain_count, '0'), 'ready'],
          [t('dialog.profile_export.skipped', 'Skipped'), text(plan.advanced_existing_domain_count, '0'), 'excluded']
        ])
        + profileExportSummarySectionHtml(t('dialog.profile_export.group_ranges', 'Ranges'), [
          [t('dialog.profile_export.selected', 'Selected'), text(plan.covered_range_count, '0'), 'total'],
          [t('dialog.profile_export.will_add', 'Will be added'), text(plan.new_range_count, '0'), 'ready'],
          [t('dialog.profile_export.skipped', 'Skipped'), text(plan.existing_range_count, '0'), 'excluded']
        ])
        + '</div>';
    }

    function profileExportSummarySectionHtml(title, rows) {
      return '<div class="profile-export-preview__section">'
        + '<div class="profile-export-preview__title">' + html(title) + ':</div>'
        + rows.map((row) => profileExportMetricRowHtml(row[0], row[1], row[2])).join('')
        + '</div>';
    }

    function profileExportUncoveredTargetsHtml(targets) {
      if (!targets.length) return '';
      const rows = '<ul class="profile-export-list profile-export-list--plain">'
        + targets.map((target) => '<li>' + html(text(target.protocol) + ':' + text(target.port) + ' - ' + text(target.ip_count, '0') + ' IP') + '</li>').join('')
        + '</ul>';
      return '<div class="profile-export-preview__group profile-export-preview__group--warning" id="profile-export-uncovered-targets" data-ui-entity="profile-export-uncovered-targets">'
        + '<div class="profile-export-preview__title">' + html(t('dialog.profile_export.uncovered_title', 'Not covered by profile rules')) + ':</div>'
        + rows
        + '<div class="profile-export-preview__note">' + html(t('dialog.profile_export.uncovered_help', 'These selected addresses will not be written by ordinary export because the selected profile has no rule for the protocol and port.')) + '</div>'
        + '</div>';
    }

    function profileExportFileChangesHtml(changes) {
      const rows = changes.length
        ? changes.map((change) => profileExportRowHtml(text(change.operation, 'update'), text(change.path, '-'))).join('')
        : '<div class="profile-export-preview__muted">' + html(t('dialog.profile_export.no_file_changes', 'No file changes.')) + '</div>';
      return '<div class="profile-export-preview__group" id="profile-export-file-changes" data-ui-entity="profile-export-file-changes"><div class="profile-export-preview__title">' + html(t('dialog.profile_export.file_changes', 'File changes')) + ':</div>' + rows + '</div>';
    }

    function profileExportWarningsHtml(warnings) {
      if (!warnings.length) return '';
      const rows = warnings.length
        ? warnings.map((warning) => profileExportRowHtml(text(warning.code, 'warning'), text(warning.message, ''))).join('')
        : '<div class="profile-export-preview__muted">' + html(t('dialog.profile_export.no_warnings', 'No warnings.')) + '</div>';
      return '<div class="profile-export-preview__group" id="profile-export-warnings" data-ui-entity="profile-export-warnings"><div class="profile-export-preview__title">' + html(t('dialog.profile_export.warnings', 'Warnings')) + ':</div>' + rows + '</div>';
    }

    function profileExportMatchedRulesHtml(rules) {
      const rows = rules.length
        ? rules.map((rule) => {
          const detail = text(rule.protocol) + ' ' + profileExportPortsLabel(rule) + ' -> ' + (rule.list_refs || []).join(', ');
          return profileExportRowHtml(text(rule.source, 'rule'), detail);
        }).join('')
        : '<div class="profile-export-preview__muted">' + html(t('dialog.profile_export.no_matched_rules', 'No matched rules.')) + '</div>';
      return '<div class="profile-export-preview__group" id="profile-export-matched-rules" data-ui-entity="profile-export-matched-rules"><div class="profile-export-preview__title">' + html(t('dialog.profile_export.matched_rules', 'Matched rules')) + ':</div>' + rows + '</div>';
    }

    function renderProfileExportAdvancedWizard() {
      const plan = state.profileExport.preview;
      const uncovered = document.getElementById('profile-export-advanced-wizard-uncovered');
      if (uncovered) {
        const targets = plan?.uncovered_targets || [];
        uncovered.innerHTML = targets.length
          ? targets.map((target) => profileExportRowHtml(text(target.protocol) + ':' + text(target.port), text(target.ip_count, '0') + ' IP')).join('')
          : '<div class="profile-export-preview__muted">' + html(t('dialog.profile_export.advanced_wizard_empty', 'There are no uncovered protocol/port targets.')) + '</div>';
      }
      const templateSelect = document.getElementById('profile-export-advanced-template-select');
      if (templateSelect) {
        const matched = plan?.matched_rules || [];
        templateSelect.innerHTML = '<option value="">' + html(t('dialog.profile_export.advanced_wizard_step_template', 'Choose an existing rule as the bypass-method template.')) + '</option>'
          + matched.map((rule) => '<option value="' + html(text(rule.source)) + '">' + html(profileExportRuleLabel(rule)) + '</option>').join('');
        templateSelect.value = matched.some((rule) => text(rule.source) === text(state.profileExport.advancedTemplateRule))
          ? text(state.profileExport.advancedTemplateRule)
          : '';
      }
      syncAdvancedSwitch('profile-export-advanced-create-rules-switch', state.profileExport.advancedCreateRules);
      syncAdvancedSwitch('profile-export-advanced-exclude-ips-switch', state.profileExport.advancedExcludeIps);
      syncAdvancedSwitch('profile-export-advanced-whois-ranges-switch', state.profileExport.advancedWhoisRanges);
      syncAdvancedSwitch('profile-export-advanced-domains-switch', state.profileExport.advancedDomains);
      const manualDomains = document.getElementById('profile-export-advanced-manual-domains-input');
      if (manualDomains && document.activeElement !== manualDomains) {
        manualDomains.value = text(state.profileExport.advancedManualDomains);
      }
      syncClearButtonStates();
    }

    function profileExportRuleLabel(rule) {
      return text(rule.protocol) + ' ' + profileExportPortsLabel(rule) + ' -> ' + (rule.list_refs || []).join(', ');
    }

    function profileExportPortsLabel(rule) {
      const explicit = text(rule.ports_display, '');
      if (explicit) return explicit;
      const ports = Array.isArray(rule.ports) ? rule.ports : [];
      if (!ports.length) return '*';

      const compact = [];
      let start = ports[0];
      let previous = ports[0];
      for (let index = 1; index < ports.length; index += 1) {
        const port = ports[index];
        if (port === previous + 1) {
          previous = port;
          continue;
        }
        compact.push(start === previous ? String(start) : String(start) + '-' + String(previous));
        start = port;
        previous = port;
      }
      compact.push(start === previous ? String(start) : String(start) + '-' + String(previous));
      return compact.join(', ');
    }

    function syncAdvancedSwitch(id, enabled) {
      const button = document.getElementById(id);
      if (!button) return;
      button.className = 'input-box switch' + (enabled ? ' switch--on' : '');
      button.setAttribute('aria-checked', String(Boolean(enabled)));
    }

    function toggleProfileExportAdvancedSetting(key) {
      state.profileExport[key] = !Boolean(state.profileExport[key]);
      state.profileExport.advancedReadyForExport = false;
      persistProfileExportUiState();
      renderProfileExportAdvancedWizard();
    }

    function updateProfileExportAdvancedSettings(domainEdited) {
      const templateSelect = document.getElementById('profile-export-advanced-template-select');
      const manualDomains = document.getElementById('profile-export-advanced-manual-domains-input');
      if (templateSelect) state.profileExport.advancedTemplateRule = templateSelect.value;
      if (manualDomains) {
        state.profileExport.advancedManualDomains = manualDomains.value;
        state.profileExport.advancedDomains = Boolean(splitManualDomains(manualDomains.value).length);
        if (domainEdited) {
          state.profileExport.advancedDomainCleanupRequested = !state.profileExport.advancedDomains;
        }
      }
      state.profileExport.advancedReadyForExport = false;
      persistProfileExportUiState();
    }

    function addProfileExportSelectedDomains() {
      updateProfileExportAdvancedSettings();
      state.profileExport.advancedManualDomains = mergeManualDomains(
        state.profileExport.advancedManualDomains,
        selectedProfileExportDomains()
      );
      state.profileExport.advancedDomains = Boolean(splitManualDomains(state.profileExport.advancedManualDomains).length);
      state.profileExport.advancedReadyForExport = false;
      state.profileExport.advancedDomainCleanupRequested = false;
      persistProfileExportUiState();
      renderProfileExportAdvancedWizard();
    }

    function clearProfileExportAdvancedManualDomains() {
      state.profileExport.advancedManualDomains = '';
      state.profileExport.advancedDomains = false;
      state.profileExport.advancedReadyForExport = false;
      state.profileExport.advancedDomainCleanupRequested = true;
      const manualDomains = document.getElementById('profile-export-advanced-manual-domains-input');
      if (manualDomains) manualDomains.value = '';
      persistProfileExportUiState();
      renderProfileExportAdvancedWizard();
      syncClearButtonStates();
    }

    function cancelProfileExportAdvancedSettings() {
      state.profileExport.advancedCreateRules = false;
      state.profileExport.advancedExcludeIps = false;
      state.profileExport.advancedWhoisRanges = false;
      state.profileExport.advancedDomains = false;
      state.profileExport.advancedManualDomains = '';
      state.profileExport.advancedTemplateRule = '';
      state.profileExport.advancedReadyForExport = false;
      state.profileExport.advancedDomainCleanupRequested = false;
      persistProfileExportUiState();
      renderProfileExportAdvancedWizard();
    }

    function selectedProfileExportDomains() {
      const seen = new Set();
      const domains = [];
      for (const row of (state.snapshot?.observed_endpoints || [])) {
        if (!row?.is_confirmed || row?.is_exported) continue;
        const enrichment = row.enrichment || {};
        const source = text(enrichment.domain_source);
        if (!['http_host', 'tls_sni', 'quic_sni', 'csv_import'].includes(source)) continue;
        const domain = text(enrichment.domain_name).trim();
        const key = domain.toLowerCase();
        if (!key || seen.has(key)) continue;
        seen.add(key);
        domains.push(domain);
      }
      return domains;
    }

    function mergeManualDomains(current, additions) {
      const seen = new Set();
      const domains = [];
      for (const domain of [...splitManualDomains(current), ...(additions || [])]) {
        const value = text(domain).trim();
        const key = value.toLowerCase();
        if (!key || seen.has(key)) continue;
        seen.add(key);
        domains.push(value);
      }
      return domains.join('\n');
    }

    function splitManualDomains(value) {
      return text(value)
        .split(/\r?\n/)
        .map((item) => item.trim())
        .filter(Boolean);
    }

    function profileExportRowHtml(label, value) {
      return '<div class="profile-export-preview__row"><span class="profile-export-preview__muted">' + html(label) + '</span><input class="path-field" type="text" readonly value="' + html(value) + '"></div>';
    }

    function profileExportMetricRowHtml(label, value, tone) {
      const safeTone = ['total', 'ready', 'excluded'].includes(tone) ? tone : 'total';
      return '<div class="profile-export-preview__row profile-export-preview__metric"><span class="profile-export-preview__muted">' + html(label) + '</span><span class="profile-export-preview__metric-value profile-export-preview__metric-value--' + safeTone + '">' + html(value) + '</span></div>';
    }

    function profileExportVisibleWarnings(warnings) {
      return (warnings || []).filter((warning) => {
        const code = text(warning.code);
        return code !== 'uncovered_protocol_port' && code !== 'profile_merge_target_skipped';
      });
    }

    function profileExportChangedFiles(plan) {
      return [...(plan?.file_changes || []), ...(plan?.advanced_file_changes || [])]
        .filter((change) => text(change.operation) !== 'skip')
        .map((change) => text(change.path))
        .filter(Boolean);
    }

    async function refreshProfileExportPreview() {
      if (!state.profileExport.open) return;
      if (state.profileExport.mode === 'merge_into_existing_lists' && !state.profileExport.dangerousConfirmed) {
        state.profileExport.previewRequestId += 1;
        state.profileExport.preview = null;
        state.profileExport.feedback = '';
        renderProfileExportDialog();
        return;
      }
      const requestId = ++state.profileExport.previewRequestId;
      try {
        const plan = await post('/v1/export/profile/preview', scopedProfileExportPayload(profileExportRequest()), { skipSnapshotRefresh: true });
        if (requestId !== state.profileExport.previewRequestId || !state.profileExport.open) return;
        state.profileExport.preview = plan;
        state.profileExport.feedback = '';
      } catch (error) {
        if (requestId !== state.profileExport.previewRequestId || !state.profileExport.open) return;
        state.profileExport.preview = null;
        state.profileExport.feedback = '';
      }
      renderProfileExportDialog();
    }

    async function analyzeProfileExport() {
      state.profileExport.previewRequestId += 1;
      const request = profileExportRequest();
      try {
        const plan = profileExportAdvancedSettingsActive()
          ? await post('/v1/export/profile/advanced-settings/analyze', scopedProfileExportPayload(profileExportAdvancedSettingsRequest(request)), { skipSnapshotRefresh: true })
          : await post('/v1/export/profile/analyze', scopedProfileExportPayload(request), { skipSnapshotRefresh: true });
        state.profileExport.preview = plan;
        state.profileExport.feedback = '';
        setFooterMessage(t('footer.event.profile_export_analyzed', 'Profile export analysis ready'));
      } catch (error) {
        state.profileExport.feedback = '';
      }
      renderProfileExportDialog();
    }

    function profileExportAdvancedSettingsActive() {
      const hasDomainSettings = Boolean(state.profileExport.advancedDomains && splitManualDomains(state.profileExport.advancedManualDomains).length);
      return Boolean(
        state.profileExport.advancedReadyForExport
        || state.profileExport.advancedCreateRules
        || state.profileExport.advancedExcludeIps
        || state.profileExport.advancedWhoisRanges
        || hasDomainSettings
        || state.profileExport.advancedDomainCleanupRequested
      );
    }

    async function applyProfileExport() {
      state.profileExport.previewRequestId += 1;
      const request = profileExportRequest();
      try {
        const applyPlan = state.profileExport.advancedReadyForExport
          ? await post('/v1/export/profile/advanced-settings/apply', scopedProfileExportPayload(profileExportAdvancedSettingsRequest(request)))
          : await post('/v1/export/profile/apply', scopedProfileExportPayload(request));
        const changed = profileExportChangedFiles(applyPlan);
        state.profileExport.preview = applyPlan;
        state.profileExport.feedback = '';
        const suffix = changed.length ? ': ' + changed.join(', ') : '';
        setFooterMessage(t('footer.event.profile_export_applied', 'Profile export applied') + suffix);
      } catch (error) {
        state.profileExport.feedback = '';
      }
      renderProfileExportDialog();
    }

    async function applyProfileExportAdvancedSettings() {
      updateProfileExportAdvancedSettings();
      const hasDomainSettings = Boolean(state.profileExport.advancedDomains && splitManualDomains(state.profileExport.advancedManualDomains).length);
      state.profileExport.advancedReadyForExport = Boolean(
        state.profileExport.advancedCreateRules
        || state.profileExport.advancedExcludeIps
        || state.profileExport.advancedWhoisRanges
        || hasDomainSettings
        || state.profileExport.advancedDomainCleanupRequested
      );
      persistProfileExportUiState();
      state.profileExport.feedback = '';
      closeProfileExportAdvancedWizard();
      setFooterMessage(t('dialog.profile_export.advanced_apply', 'Apply'));
      renderProfileExportDialog();
    }

    function profileExportAdvancedSettingsRequest(exportRequest) {
      return {
        export: exportRequest || profileExportRequest(),
        settings: {
          create_rules_for_uncovered: Boolean(state.profileExport.advancedCreateRules),
          add_ips_to_exclude: Boolean(state.profileExport.advancedExcludeIps),
          use_whois_ranges_for_export: Boolean(state.profileExport.advancedWhoisRanges),
          add_detected_domains: Boolean(state.profileExport.advancedDomains),
          remove_detected_domains: Boolean(state.profileExport.advancedDomainCleanupRequested),
          manual_domains: splitManualDomains(state.profileExport.advancedManualDomains),
          template_rule_source: text(state.profileExport.advancedTemplateRule) || null
        }
      };
    }

    async function backupProfileExport() {
      state.profileExport.previewRequestId += 1;
      try {
        const plan = await post('/v1/export/profile/backup', scopedProfileExportPayload(profileExportRequest()), { skipSnapshotRefresh: true });
        state.profileExport.preview = plan;
        state.profileExport.feedback = '';
        const changed = profileExportChangedFiles(plan);
        const suffix = changed.length ? ': ' + changed.join(', ') : '';
        setFooterMessage(t('footer.event.profile_export_backup_created', 'Profile export backup created') + suffix);
      } catch (error) {
        state.profileExport.feedback = '';
      }
      renderProfileExportDialog();
    }

    async function revertProfileExport() {
      state.profileExport.previewRequestId += 1;
      try {
        const plan = await post('/v1/export/profile/revert', scopedProfileExportPayload(profileExportRequest()));
        state.profileExport.preview = plan;
        state.profileExport.feedback = '';
        const changed = profileExportChangedFiles(plan);
        const suffix = changed.length ? ': ' + changed.join(', ') : '';
        setFooterMessage(t('footer.event.profile_export_reverted', 'Profile export changes reverted') + suffix);
      } catch (error) {
        state.profileExport.feedback = '';
      }
      renderProfileExportDialog();
    }

    async function startMonitoring() {
      const response = await post('/v1/monitor/start');
      if (state.snapshot) {
        state.snapshot.monitor_status = response?.status || 'Starting';
      }
      setFooterMessage(t('footer.event.monitoring_started', 'Monitoring started'));
      renderMonitor(state.snapshot);
      renderFooterWatcher(state.watcherConnected);
      renderFooterTool(state.snapshot);
      renderFooterWebServer(state.snapshot);
      renderFooterNetwork(state.snapshot);
      renderFooterMessage();
    }

    async function stopMonitoring() {
      const response = await post('/v1/monitor/stop');
      if (state.snapshot) {
        state.snapshot.monitor_status = response?.status || 'Stopped';
      }
      setFooterMessage(t('footer.event.monitoring_stopped', 'Monitoring stopped'));
      renderMonitor(state.snapshot);
      renderFooterWatcher(state.watcherConnected);
      renderFooterTool(state.snapshot);
      renderFooterWebServer(state.snapshot);
      renderFooterNetwork(state.snapshot);
      renderFooterMessage();
    }

    async function toggleMonitoring() {
      const status = state.snapshot?.monitor_status || 'Stopped';
      if (['Running', 'Starting'].includes(status)) {
        await stopMonitoring();
      } else {
        await startMonitoring();
      }
      await loadSnapshot();
    }

    function setStatus(message, failed = false) {
      const readable = cloudReadableMessage(message);
      const value = failed
        ? t('status.error', 'Error') + ': ' + readable
        : readable;
      setFooterMessage(value);
    }

    window.addEventListener('load', scheduleVisualDebugRects);
    window.addEventListener('resize', scheduleVisualDebugRects);
    scheduleVisualDebugRects();
    async function bootstrapBrowserUi() {
      try {
        await loadLanguages();
      } catch (_error) {}
      try {
        await authenticateWebAccessFromHash();
      } catch (error) {
        state.webAuthRequired = true;
        state.webAuthMessage = Number(error?.status) === 401
          ? t('web.auth.invalid_key', 'The web access key is invalid.')
          : text(error?.message || error);
        renderShellDisabled(true);
        renderWebAuthOverlay(true, state.webAuthMessage);
        return;
      }
      await loadSnapshot();
      if (!state.webAuthRequired) {
        scheduleBackgroundSnapshotRefresh(1200);
      }
    }

    scheduleHostAvailabilityCheck();
    bootstrapBrowserUi();
  </script>
</body>
</html>"#;

#[derive(Clone)]
struct AppState {
    core: NetstitchCore,
    monitor: MonitorController,
    shutdown: ShutdownSignal,
    ui_filters: Arc<Mutex<UiFiltersDto>>,
    integration_download_progress: Arc<Mutex<IntegrationDownloadProgressDto>>,
    module_ui_action_events: Arc<Mutex<ModuleUiActionEventStore>>,
    endpoint_probe_status: Arc<Mutex<EndpointProbeStatusDto>>,
    endpoint_probe_targets: Arc<Mutex<Vec<String>>>,
    bind_addr: SocketAddr,
}

#[derive(Default)]
struct ModuleUiActionEventStore {
    sessions: HashMap<String, ModuleUiActionEventSession>,
}

struct ModuleUiActionEventSession {
    next_seq: u64,
    updated_at: Instant,
    events: Vec<IntegrationModuleUiActionEventDto>,
}

impl ModuleUiActionEventStore {
    fn register(&mut self, module_id: &str, token: &str) {
        if token.trim().is_empty() {
            return;
        }
        self.prune_stale();
        self.sessions.insert(
            module_ui_action_event_key(module_id, token),
            ModuleUiActionEventSession {
                next_seq: 1,
                updated_at: Instant::now(),
                events: Vec::new(),
            },
        );
    }

    fn push(&mut self, module_id: &str, token: &str, event_type: &str, payload: serde_json::Value) {
        if token.trim().is_empty() {
            return;
        }
        self.prune_stale();
        let key = module_ui_action_event_key(module_id, token);
        let session = self
            .sessions
            .entry(key)
            .or_insert_with(|| ModuleUiActionEventSession {
                next_seq: 1,
                updated_at: Instant::now(),
                events: Vec::new(),
            });
        let seq = session.next_seq;
        session.next_seq = session.next_seq.saturating_add(1);
        session.updated_at = Instant::now();
        session.events.push(IntegrationModuleUiActionEventDto {
            seq,
            module_id: module_id.to_string(),
            ui_action_token: token.to_string(),
            event_type: event_type.to_string(),
            payload,
        });
        if session.events.len() > 256 {
            let excess = session.events.len() - 256;
            session.events.drain(0..excess);
        }
    }

    fn events_after(
        &mut self,
        module_id: &str,
        token: &str,
        after: u64,
    ) -> Vec<IntegrationModuleUiActionEventDto> {
        self.prune_stale();
        self.sessions
            .get(&module_ui_action_event_key(module_id, token))
            .map(|session| {
                session
                    .events
                    .iter()
                    .filter(|event| event.seq > after)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn prune_stale(&mut self) {
        let now = Instant::now();
        self.sessions
            .retain(|_, session| now.duration_since(session.updated_at) < Duration::from_secs(600));
    }
}

fn module_ui_action_event_key(module_id: &str, token: &str) -> String {
    format!("{}:{}", module_id.trim(), token.trim())
}

#[derive(Deserialize)]
struct SnapshotQuery {
    integration_dir: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    endpoint_probe_target: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    endpoint_probe_protocol: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SystemEventsQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct IntegrationDialogResultRequest {
    module_id: String,
    dialog_id: String,
    result: String,
}

#[derive(Debug, Deserialize)]
struct IntegrationModuleUiActionEventsQuery {
    module_id: String,
    ui_action_token: String,
    after: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CloudAppsQuery {
    query: Option<String>,
    publisher: Option<String>,
    source: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CloudCatalogAppDto {
    app_id: String,
    display_name: String,
    publisher_name: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    endpoint_count: u64,
    #[serde(default)]
    available_row_count: u64,
    #[serde(default)]
    author_count: u64,
    #[serde(default)]
    last_seen_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CloudAppsResponseDto {
    items: Vec<CloudCatalogAppDto>,
}

#[derive(Debug, Serialize)]
struct CloudWebAppsResponse {
    items: Vec<CloudCatalogAppDto>,
}

#[derive(Clone, Debug, Deserialize)]
struct CloudUserAppSummaryDto {
    app_id: String,
    display_name: String,
    #[serde(default)]
    publisher_name: Option<String>,
    #[serde(default)]
    author_signature: Option<String>,
    #[serde(default)]
    visibility: CloudObservationVisibility,
    #[serde(default)]
    endpoint_count: u64,
    #[serde(default)]
    available_row_count: u64,
    #[serde(default)]
    total_endpoint_count: u64,
}

#[derive(Debug, Deserialize)]
struct CloudUserAppsResponseDto {
    items: Vec<CloudUserAppSummaryDto>,
}

#[derive(Debug, Serialize)]
struct CloudWebUserAppSummary {
    app_id: String,
    display_name: String,
    publisher_name: Option<String>,
    author_signature: Option<String>,
    visibility: CloudObservationVisibility,
    endpoint_count: u64,
    available_row_count: u64,
    author_rows: u64,
    total_rows: Option<u64>,
}

#[derive(Debug, Serialize)]
struct CloudWebUserAppsResponse {
    items: Vec<CloudWebUserAppSummary>,
}

#[derive(Debug, Deserialize)]
struct CloudQuotaResponseDto {
    upload: CloudQuotaSnapshot,
    download: CloudQuotaSnapshot,
}

#[derive(Debug, Serialize)]
struct CloudWebQuotaResponse {
    upload: CloudQuotaSnapshot,
    download: CloudQuotaSnapshot,
}

#[derive(Debug, Serialize)]
struct CloudLocalAuthorResponse {
    nickname: Option<String>,
}

#[derive(Debug, Serialize)]
struct CloudWebAuthStateResponse {
    authenticated: bool,
    authorization_label: Option<String>,
    nickname: Option<String>,
    client_identifier: String,
}

#[derive(Debug, Deserialize)]
struct CloudNicknameRequest {
    nickname: String,
}

#[derive(Debug, Serialize)]
struct CloudNicknameResponse {
    nickname: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct CloudUploadRequest {
    #[serde(default)]
    author_signature: Option<String>,
    #[serde(default)]
    visibility: Option<CloudObservationVisibility>,
}

#[derive(Debug, Serialize)]
struct CloudUploadResponse {
    accepted_rows: usize,
    request_count: usize,
    skipped_non_public_rows: usize,
    responses: Vec<serde_json::Value>,
    quota: Option<CloudWebQuotaResponse>,
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

#[derive(Debug, Deserialize)]
struct CloudObservationsQuery {
    app_id: String,
    app_name: Option<String>,
    expected_total: Option<usize>,
    visibility: Option<CloudObservationVisibilityScope>,
    #[serde(default)]
    own_scope: bool,
    ip: Option<String>,
    domain: Option<String>,
    port: Option<String>,
    protocol: Option<String>,
    source: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct CloudWebObservationRow {
    row_id: String,
    application: String,
    author: String,
    ip: String,
    domain: String,
    port: u16,
    protocol: String,
    connection: String,
    hits: u64,
    privacy: String,
    failed_hits: u64,
    successful_hits: u64,
    first_seen_ms: u64,
    last_seen_ms: u64,
    row: CloudObservationRow,
}

#[derive(Debug, Serialize)]
struct CloudWebObservationsResponse {
    rows: Vec<CloudWebObservationRow>,
    quota: Option<CloudWebQuotaResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CloudObservationDownloadPage {
    #[serde(default)]
    total: u64,
    rows: Vec<CloudObservationDownloadPageRow>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CloudObservationDownloadPageRow {
    #[serde(default)]
    visibility: Option<CloudObservationVisibility>,
    #[serde(flatten)]
    row: CloudObservationRow,
}

struct CloudCollectedObservationRow {
    is_private: bool,
    row: CloudObservationRow,
}

#[derive(Debug, Deserialize)]
struct ConfigureIntegrationRequest {
    #[serde(default)]
    module_id: Option<String>,
    repo_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct StopIntegrationModuleBackgroundRequest {
    module_id: String,
}

#[derive(Debug, Deserialize)]
struct ScopedProfileExportRequest<T> {
    #[serde(default)]
    module_id: Option<String>,
    request: T,
}

impl<T> ScopedProfileExportRequest<T> {
    fn into_parts(self) -> (Option<String>, T) {
        (self.module_id, self.request)
    }
}

#[derive(Debug, Deserialize)]
struct BrowseFilesystemQuery {
    path: Option<PathBuf>,
    mode: Option<FilePickerMode>,
    extensions: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FilePickerMode {
    Any,
    Executable,
    Folder,
    FileOpen,
    FileSave,
    Profile,
    CsvOpen,
    CsvSave,
}

impl Default for FilePickerMode {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Serialize)]
struct FilesystemBrowseDto {
    current_path: String,
    parent_path: Option<String>,
    roots: Vec<String>,
    entries: Vec<FilesystemEntryDto>,
}

#[derive(Debug, Serialize)]
struct FilesystemEntryDto {
    name: String,
    path: String,
    is_dir: bool,
    is_file: bool,
    selectable: bool,
}

#[derive(Debug, Deserialize)]
struct ReadTextFileRequest {
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct ReadTextFileResponse {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct WriteTextFileRequest {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Serialize)]
struct WriteTextFileResponse {
    path: String,
    bytes_written: usize,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    Ok(match Option::<OneOrMany>::deserialize(deserializer)? {
        Some(OneOrMany::One(value)) => vec![value],
        Some(OneOrMany::Many(values)) => values,
        None => Vec::new(),
    })
}

#[derive(Debug, Clone, Serialize)]
struct LanguageCatalogDto {
    fallback_code: String,
    languages: Vec<LanguageDto>,
}

#[derive(Debug, Clone, Serialize)]
struct LanguageDto {
    code: String,
    label: String,
    order: i32,
    strings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct WebUrlDto {
    url: String,
    used_localhost_fallback: bool,
}

#[derive(Debug, Deserialize)]
struct WebAuthRequest {
    key: String,
}

#[derive(Debug, Serialize)]
struct WebAuthResponse {
    ok: bool,
}

#[derive(Debug, Clone, Serialize)]
struct EndpointProbeTargetsDto {
    targets: Vec<String>,
}

type ShutdownSignal = Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("NetStitch=info,netstitch_watcher=info")
        .compact()
        .try_init();
}

pub async fn serve_watcher(addr: Option<String>) -> Result<()> {
    init_tracing();
    install_rustls_crypto_provider();
    let core = NetstitchCore::bootstrap().context("failed to bootstrap NetStitch core")?;
    let monitor = MonitorController::new(core.clone());
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let bind_addr = watcher_addr(addr.as_deref())?;
    let state = build_runtime_state(core, monitor, shutdown_tx, bind_addr);
    spawn_runtime_tool_loops(&state);
    let app = Router::new()
        .route("/", get(root_web_ui))
        .route("/health", get(health))
        .route("/v1/web-auth", post(web_auth))
        .route("/v1/web-url", get(web_url))
        .route(
            "/v1/system-events",
            get(system_events).post(record_system_event),
        )
        .route("/v1/endpoint-probe-targets", get(endpoint_probe_targets))
        .route("/v1/cloud/apps", get(cloud_apps))
        .route("/v1/cloud/observations", get(cloud_observations))
        .route("/v1/cloud/quota", get(cloud_quota))
        .route("/v1/cloud/local-author", get(cloud_local_author))
        .route("/v1/cloud/auth-state", get(cloud_auth_state))
        .route("/v1/cloud/my-apps", get(cloud_my_apps))
        .route("/v1/cloud/nickname", post(set_cloud_nickname))
        .route("/v1/cloud/sign-out", post(cloud_sign_out))
        .route("/v1/cloud/upload", post(cloud_upload))
        .route("/v1/assets/close-times.svg", get(close_times_icon))
        .route("/v1/assets/module-stop.svg", get(module_stop_icon))
        .route("/v1/assets/module-close.svg", get(module_close_icon))
        .route(
            "/v1/assets/module-reorder-left.svg",
            get(module_reorder_left_icon),
        )
        .route("/v1/assets/copy.svg", get(copy_icon))
        .route("/v1/assets/help-circle.svg", get(help_circle_icon))
        .route(
            "/v1/assets/confirm-filtered.svg",
            get(confirm_filtered_icon),
        )
        .route("/v1/assets/trash.svg", get(trash_icon))
        .route("/v1/assets/import-cloud.svg", get(import_cloud_icon))
        .route("/v1/assets/export-cloud.svg", get(export_cloud_icon))
        .route("/v1/assets/my-publications.svg", get(my_publications_icon))
        .route("/v1/assets/import-csv.svg", get(import_csv_icon))
        .route("/v1/assets/export-csv.svg", get(export_csv_icon))
        .route(
            "/v1/assets/unconfirm-filtered.svg",
            get(unconfirm_filtered_icon),
        )
        .route("/v1/assets/ignore-address.svg", get(ignore_address_icon))
        .route("/v1/assets/monitoring.svg", get(monitoring_icon))
        .route("/v1/assets/sort-idle.svg", get(sort_idle_icon))
        .route("/v1/assets/sort-asc.svg", get(sort_asc_icon))
        .route("/v1/assets/sort-desc.svg", get(sort_desc_icon))
        .route("/v1/languages", get(languages))
        .route("/v1/filesystem/browse", get(browse_filesystem))
        .route("/v1/filesystem/read-text", post(read_text_file))
        .route("/v1/filesystem/write-text", post(write_text_file))
        .route("/v1/snapshot", get(snapshot))
        .route("/v1/filters", post(set_ui_filters))
        .route(
            "/v1/tracked-apps/{tracked_app_id}/icon",
            get(tracked_app_icon),
        )
        .route(
            "/v1/integrations/{module_id}/icon",
            get(integration_module_icon),
        )
        .route("/v1/tracked-apps", post(add_tracked_app))
        .route("/v1/tracked-apps/toggle", post(set_tracked_app_enabled))
        .route("/v1/tracked-apps/delete", post(delete_tracked_app))
        .route(
            "/v1/tracked-apps/enabled",
            post(set_all_tracked_apps_enabled),
        )
        .route("/v1/monitor/start", post(start_monitoring))
        .route("/v1/monitor/stop", post(stop_monitoring))
        .route("/v1/watcher/shutdown", post(shutdown_watcher))
        .route("/v1/observations/confirm", post(confirm_observations))
        .route(
            "/v1/observations/mark-exported",
            post(mark_observations_exported),
        )
        .route("/v1/observations/delete", post(delete_observation))
        .route("/v1/observations/delete-batch", post(delete_observations))
        .route("/v1/observations/import-csv", post(import_monitoring_csv))
        .route("/v1/ignored-addresses", post(add_ignored_address))
        .route("/v1/ignored-addresses/delete", post(delete_ignored_address))
        .route("/v1/settings", post(set_app_setting))
        .route("/v1/integrations/configure", post(configure_integration))
        .route("/v1/integrations/ui-action", post(integration_ui_action))
        .route(
            "/v1/integrations/ui-action-events",
            get(integration_ui_action_events),
        )
        .route(
            "/v1/integrations/background/stop",
            post(stop_integration_module_background),
        )
        .route(
            "/v1/integrations/dialog-result",
            post(integration_dialog_result),
        )
        .route(
            "/v1/integrations/profile-export-ui-state",
            post(set_profile_export_ui_state),
        )
        .route(
            "/v1/integrations/download/start",
            post(start_integration_download),
        )
        .route("/v1/integrations/download", post(download_integration))
        .route(
            "/v1/integrations/download-progress",
            get(integration_download_progress),
        )
        .route("/v1/integrations/providers", get(integration_providers))
        .route("/v1/export", post(export_confirmed))
        .route("/v1/export/profile/preview", post(preview_profile_export))
        .route("/v1/export/profile/analyze", post(analyze_profile_export))
        .route("/v1/export/profile/backup", post(backup_profile_export))
        .route("/v1/export/profile/apply", post(apply_profile_export))
        .route(
            "/v1/export/profile/advanced-settings/analyze",
            post(analyze_profile_export_advanced_settings),
        )
        .route(
            "/v1/export/profile/advanced-settings/apply",
            post(apply_profile_export_advanced_settings),
        )
        .route("/v1/export/profile/revert", post(revert_profile_export))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            browser_http_access_gate,
        ));

    if web_scheme_is_https() {
        let tls_config = web_tls_config(&state)
            .await
            .context("failed to prepare local HTTPS certificate")?;
        let shutdown_handle = axum_server::Handle::new();
        let server_handle = shutdown_handle.clone();
        tokio::spawn(async move {
            let _ = shutdown_rx.await;
            shutdown_handle.graceful_shutdown(None);
        });
        info!("netstitch-watcher listening on https://{}", bind_addr);
        axum_server::bind_rustls(bind_addr, tls_config)
            .handle(server_handle)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(bind_addr).await?;
        info!("netstitch-watcher listening on http://{}", bind_addr);
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await?;
    }
    Ok(())
}

fn install_rustls_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

pub fn spawn_embedded_watcher(addr: Option<String>) -> std::thread::JoinHandle<anyhow::Result<()>> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("netstitch-watcher-runtime")
            .build()
            .context("failed to build embedded watcher runtime")?;
        runtime.block_on(serve_watcher(addr))
    })
}

fn build_runtime_state(
    core: NetstitchCore,
    monitor: MonitorController,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    bind_addr: SocketAddr,
) -> AppState {
    let mut ui_filters = UiFiltersDto::default();
    ui_filters.public_ip = core.monitoring_public_ip_filter_enabled().unwrap_or(true);
    AppState {
        core,
        monitor,
        shutdown: Arc::new(tokio::sync::Mutex::new(Some(shutdown_tx))),
        ui_filters: Arc::new(Mutex::new(ui_filters)),
        integration_download_progress: Arc::new(Mutex::new(IntegrationDownloadProgressDto::idle())),
        module_ui_action_events: Arc::new(Mutex::new(ModuleUiActionEventStore::default())),
        endpoint_probe_status: Arc::new(Mutex::new(initial_endpoint_probe_status())),
        endpoint_probe_targets: Arc::new(Mutex::new(load_endpoint_probe_targets())),
        bind_addr,
    }
}

fn current_ui_filters(state: &AppState) -> UiFiltersDto {
    state
        .ui_filters
        .lock()
        .expect("ui filters lock poisoned")
        .clone()
}

fn background_subscriptions_from_payload(
    payload: &serde_json::Value,
) -> WatcherResult<Vec<String>> {
    let value = payload
        .get("subscriptions")
        .cloned()
        .unwrap_or_else(|| payload.clone());
    if value.is_null() {
        return Ok(Vec::new());
    }
    serde_json::from_value::<Vec<String>>(value).map_err(|error| {
        WatcherError::bad_request(format!("invalid background subscriptions: {error}"))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModuleLogEventCommand {
    severity: String,
    message: String,
}

fn module_log_event_from_payload(
    module_id: &str,
    payload: &serde_json::Value,
) -> WatcherResult<ModuleLogEventCommand> {
    let object = payload.as_object().ok_or_else(|| {
        WatcherError::bad_request("log_event payload must be an object with message and severity")
    })?;
    let message = object
        .get("message")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| WatcherError::bad_request("log_event message is required"))?;
    let severity = object
        .get("severity")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("info");
    let severity = match severity {
        "info" | "success" | "warning" | "error" => severity.to_string(),
        _ => {
            return Err(WatcherError::bad_request(
                "log_event severity must be info, success, warning, or error",
            ));
        }
    };
    let mut clipped = String::new();
    for ch in message.chars().take(500) {
        clipped.push(ch);
    }
    Ok(ModuleLogEventCommand {
        severity,
        message: format!("[{module_id}] {clipped}"),
    })
}

fn spawn_runtime_tool_loops(state: &AppState) {
    let tool_runtime_lock = Arc::new(tokio::sync::Mutex::new(()));
    spawn_ip_enrichment_tool_loop(state.core.clone(), tool_runtime_lock.clone());
    spawn_endpoint_probe_tool_loop(
        state.core.clone(),
        state.endpoint_probe_status.clone(),
        state.endpoint_probe_targets.clone(),
        tool_runtime_lock,
    );
}

fn spawn_ip_enrichment_tool_loop(
    core: NetstitchCore,
    tool_runtime_lock: Arc<tokio::sync::Mutex<()>>,
) {
    tokio::spawn(async move {
        loop {
            let enabled = core.ip_enrichment_enabled().unwrap_or(true);
            if enabled {
                let _tool_runtime_guard = tool_runtime_lock.lock().await;
                let result = tokio::time::timeout(
                    Duration::from_secs(60),
                    netstitch_tool::enrich_pending_once(&core, 16, 60 * 60 * 1000),
                )
                .await;
                match result {
                    Ok(Err(error)) => {
                        tracing::debug!("failed to run native network tool module: {error:#}");
                    }
                    Err(_) => {
                        tracing::debug!("native network tool module timed out");
                    }
                    Ok(Ok(_)) => {}
                }
            }
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}

fn spawn_endpoint_probe_tool_loop(
    core: NetstitchCore,
    endpoint_probe_status: Arc<Mutex<EndpointProbeStatusDto>>,
    endpoint_probe_targets: Arc<Mutex<Vec<String>>>,
    tool_runtime_lock: Arc<tokio::sync::Mutex<()>>,
) {
    tokio::spawn(async move {
        let mut last_available = None::<bool>;
        loop {
            mark_endpoint_probe_checking(&endpoint_probe_status);
            let targets = current_endpoint_probe_targets(&endpoint_probe_targets);
            let next_status =
                match run_endpoint_probe_tool(&targets, tool_runtime_lock.clone()).await {
                    Ok(status) => status,
                    Err(error) => {
                        tracing::debug!("failed to run endpoint probe native tool: {error}");
                        EndpointProbeStatusDto {
                            is_checking: false,
                            first_successful_target: None,
                            probes: vec![EndpointProbeTargetDto {
                                target: targets.first().cloned().unwrap_or_default(),
                                available: Some(false),
                                error: Some(error.to_string()),
                            }],
                        }
                    }
                };
            let available = next_status.first_successful_target.is_some();
            if last_available != Some(available) {
                append_runtime_event(
                    &core,
                    "dns",
                    if available {
                        "probe_available"
                    } else {
                        "probe_unavailable"
                    },
                    if available { "info" } else { "warning" },
                    None,
                    None,
                    serde_json::json!({
                        "first_successful_target": next_status.first_successful_target.clone(),
                        "probe_count": next_status.probes.len(),
                    }),
                );
                last_available = Some(available);
            }
            set_endpoint_probe_status(&endpoint_probe_status, next_status);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

async fn run_endpoint_probe_tool(
    targets: &[String],
    tool_runtime_lock: Arc<tokio::sync::Mutex<()>>,
) -> Result<EndpointProbeStatusDto> {
    if targets.is_empty() {
        anyhow::bail!("endpoint probe targets are not configured");
    }
    let _tool_runtime_guard = tool_runtime_lock.lock().await;
    tokio::time::timeout(
        Duration::from_secs(180),
        netstitch_tool::probe_endpoint_target_strings(targets),
    )
    .await
    .context("endpoint probe native tool timed out")?
    .context("failed to run endpoint probe native tool")
}

fn initial_endpoint_probe_status() -> EndpointProbeStatusDto {
    EndpointProbeStatusDto {
        is_checking: true,
        first_successful_target: None,
        probes: Vec::new(),
    }
}

fn mark_endpoint_probe_checking(status: &Arc<Mutex<EndpointProbeStatusDto>>) {
    if let Ok(mut guard) = status.lock() {
        guard.is_checking = true;
    }
}

fn set_endpoint_probe_status(
    status: &Arc<Mutex<EndpointProbeStatusDto>>,
    next_status: EndpointProbeStatusDto,
) {
    if let Ok(mut guard) = status.lock() {
        *guard = next_status;
    }
}

fn current_endpoint_probe_status(state: &AppState) -> EndpointProbeStatusDto {
    state
        .endpoint_probe_status
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| initial_endpoint_probe_status())
}

fn current_tool_available() -> bool {
    true
}

fn append_runtime_event(
    core: &NetstitchCore,
    component: &str,
    action_type: &str,
    severity: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    payload: serde_json::Value,
) {
    append_runtime_event_with_source(
        core,
        "core",
        component,
        action_type,
        severity,
        entity_type,
        entity_id,
        payload,
    );
}

fn append_runtime_event_with_source(
    core: &NetstitchCore,
    source: &str,
    component: &str,
    action_type: &str,
    severity: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    payload: serde_json::Value,
) {
    let core = core.clone();
    let request = SystemEventRequestDto {
        source: Some(source.to_string()),
        component: component.to_string(),
        action_type: action_type.to_string(),
        severity: severity.to_string(),
        entity_type: entity_type.map(ToOwned::to_owned),
        entity_id: entity_id.map(ToOwned::to_owned),
        payload,
    };
    tokio::task::spawn_blocking(move || {
        if let Err(error) = core.append_system_event(request) {
            tracing::debug!("failed to append system event: {error:#}");
        }
    });
}

fn integration_module_source_name(core: &NetstitchCore, module_id: &str) -> String {
    core.integration_module_display_name(module_id)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| module_id.trim().to_string())
}

fn enforce_domain_capture_admin_gate(core: &NetstitchCore) -> anyhow::Result<bool> {
    if !current_process_is_elevated() && core.domain_capture_enabled()? {
        core.set_app_setting(
            netstitch_shared::models::SETTING_DOMAIN_CAPTURE_ENABLED,
            "false",
        )?;
        return Ok(true);
    }
    Ok(false)
}

#[cfg(target_os = "windows")]
fn current_process_is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

#[cfg(not(target_os = "windows"))]
fn current_process_is_elevated() -> bool {
    true
}

fn setting_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn update_endpoint_probe_targets(state: &AppState, targets: Vec<String>, protocols: Vec<String>) {
    let mut targets = targets
        .into_iter()
        .enumerate()
        .map(|(index, target)| {
            canonical_endpoint_probe_target(&target, protocols.get(index).map(String::as_str))
        })
        .filter(|target| !target.is_empty())
        .collect::<Vec<_>>();
    targets.dedup();
    if targets.is_empty() {
        return;
    }
    if let Ok(mut guard) = state.endpoint_probe_targets.lock() {
        *guard = targets;
    }
}

fn current_endpoint_probe_targets(targets: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    let configured = targets
        .lock()
        .map(|targets| targets.clone())
        .unwrap_or_default();
    if configured.is_empty() {
        load_endpoint_probe_targets()
    } else {
        configured
    }
}

fn canonical_endpoint_probe_target(target: &str, protocol_override: Option<&str>) -> String {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let (prefixed_protocol, normalized_target) = if let Some(value) = trimmed.strip_prefix("tcp://")
    {
        (Protocol::Tcp, value)
    } else if let Some(value) = trimmed.strip_prefix("udp://") {
        (Protocol::Udp, value)
    } else {
        (default_probe_protocol(trimmed), trimmed)
    };

    let protocol = protocol_override
        .map(Protocol::from_name)
        .filter(|value| *value != Protocol::Other)
        .unwrap_or(prefixed_protocol);

    let normalized_target = normalized_target.trim();
    if normalized_target.is_empty() {
        return String::new();
    }

    format!("{}://{}", protocol.as_str(), normalized_target)
}

fn default_probe_protocol(target: &str) -> Protocol {
    match target
        .rsplit(':')
        .next()
        .and_then(|port| port.parse::<u16>().ok())
    {
        Some(53) => Protocol::Udp,
        _ => Protocol::Tcp,
    }
}

fn cloud_base_url() -> String {
    env::var(CLOUD_BASE_URL_ENV)
        .unwrap_or_else(|_| DEFAULT_CLOUD_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn cloud_http_client(timeout: Duration) -> WatcherResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| {
            WatcherError::with_status(
                StatusCode::BAD_GATEWAY,
                anyhow::anyhow!("failed to create cloud HTTP client: {error}"),
            )
        })
}

async fn cloud_get_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    bearer: Option<&str>,
) -> WatcherResult<T> {
    let mut request = client.get(url);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(|error| {
        WatcherError::with_status(
            StatusCode::BAD_GATEWAY,
            anyhow::anyhow!("cloud request failed: {error}"),
        )
    })?;
    let status = response.status();
    let response_text = response
        .text()
        .await
        .unwrap_or_else(|_| "cloud request failed".to_string());
    if !status.is_success() {
        return Err(WatcherError::with_status(
            status,
            anyhow::anyhow!(compact_json_text(&response_text)),
        ));
    }
    serde_json::from_str::<T>(&response_text).map_err(|error| {
        WatcherError::with_status(
            StatusCode::BAD_GATEWAY,
            anyhow::anyhow!("cloud response is invalid: {error}"),
        )
    })
}

async fn cloud_quota_snapshot() -> WatcherResult<CloudWebQuotaResponse> {
    let client_identifier = local_cloud_client_identifier().map_err(WatcherError::bad_request)?;
    let client = cloud_http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let url = format!(
        "{}/v1/client/quota?client_identifier={}",
        cloud_base_url(),
        url_component(&client_identifier)
    );
    let response = cloud_get_json::<CloudQuotaResponseDto>(&client, &url, None).await?;
    Ok(CloudWebQuotaResponse {
        upload: response.upload,
        download: response.download,
    })
}

fn local_cloud_client_identifier() -> Result<String, String> {
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

fn effective_cloud_query_text(value: Option<&str>) -> Option<String> {
    let trimmed = value.unwrap_or_default().trim();
    if trimmed.chars().count() < 2 {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn effective_cloud_protocol(value: Option<&str>) -> Option<String> {
    let trimmed = value.unwrap_or_default().trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn cloud_web_observation_row(
    index: usize,
    is_private: bool,
    row: CloudObservationRow,
    app_name: &str,
) -> CloudWebObservationRow {
    let author = row.author_signature.clone().unwrap_or_default();
    let row_id = format!(
        "{}:{}:{}:{}:{}:{}",
        index,
        row.app_id,
        row.remote_ip,
        row.remote_port,
        row.protocol.as_str(),
        author
    );
    let protocol = match row.protocol {
        Protocol::Tcp => "TCP",
        Protocol::Udp => "UDP",
        Protocol::Other => "Other",
    }
    .to_string();
    let connection = format!(
        "{} ({}/{})",
        row.connection_state.as_str(),
        row.successful_hits,
        row.failed_hits
    );
    CloudWebObservationRow {
        row_id,
        application: app_name.to_string(),
        author,
        ip: row.remote_ip.to_string(),
        domain: row.domain_raw.clone().unwrap_or_default(),
        port: row.remote_port,
        protocol,
        connection,
        hits: row.requests,
        privacy: is_private.to_string(),
        failed_hits: row.failed_hits,
        successful_hits: row.successful_hits,
        first_seen_ms: row.first_seen_ms,
        last_seen_ms: row.last_seen_ms,
        row,
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

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

fn cloud_visibility_value(visibility: CloudObservationVisibility) -> &'static str {
    match visibility {
        CloudObservationVisibility::Public => "public",
        CloudObservationVisibility::Private => "private",
    }
}

fn cloud_operation_id(prefix: &str, client_identifier: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

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

fn validate_author_signature(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 32 {
        return Err(
            "{\"error\":{\"code\":\"invalid_author_signature\",\"message\":\"Nickname must be 1..32 characters\"}}"
                .to_string(),
        );
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "!.@#$%^&*()_+=-~".contains(ch))
    {
        return Err(
            "{\"error\":{\"code\":\"invalid_author_signature\",\"message\":\"Nickname contains unsupported characters\"}}"
                .to_string(),
        );
    }
    if RESERVED_AUTHOR_SIGNATURES.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return Err(
            "{\"error\":{\"code\":\"nickname_blocked\",\"message\":\"Nickname is reserved\"}}"
                .to_string(),
        );
    }
    Ok(trimmed.to_string())
}

#[cfg(windows)]
fn unprotect_cloud_secret(protected: &str) -> Result<String, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
    };

    let mut input = URL_SAFE_NO_PAD
        .decode(protected.trim())
        .map_err(|_| "stored cloud auth secret is not valid base64url".to_string())?;
    let mut entropy = CLOUD_SECRET_ENTROPY.to_vec();
    let mut input_blob = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let entropy_blob = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_mut_ptr(),
    };
    let mut output_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &mut input_blob,
            std::ptr::null_mut(),
            &entropy_blob,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        )
    };
    if ok == 0 {
        return Err("failed to unprotect cloud auth secret with Windows DPAPI".to_string());
    }

    let plaintext = unsafe {
        std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec()
    };
    if !output_blob.pbData.is_null() {
        unsafe {
            windows_sys::Win32::Foundation::LocalFree(output_blob.pbData.cast());
        }
    }
    String::from_utf8(plaintext)
        .map_err(|_| "stored cloud auth secret is not valid UTF-8".to_string())
}

#[cfg(not(windows))]
fn unprotect_cloud_secret(_protected: &str) -> Result<String, String> {
    Err("secure local cloud auth storage is available only on Windows".to_string())
}

fn read_cloud_session(core: &NetstitchCore) -> Result<CloudUserSessionResponse, String> {
    let get = |key: &str| -> Result<Option<String>, String> {
        core.get_app_setting(key).map_err(|error| error.to_string())
    };
    let user_id = get(SETTING_CLOUD_USER_ID)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud upload requires Google sign-in\"}}".to_string())?;
    let client_id = get(SETTING_CLOUD_CLIENT_ID)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud client id is missing\"}}".to_string())?;
    let key_id = get(SETTING_CLOUD_KEY_ID)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud key id is missing\"}}"
                .to_string()
        })?;
    let protected_token = get(SETTING_CLOUD_SESSION_TOKEN_DPAPI)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Cloud session token is missing\"}}".to_string())?;
    let session_token = unprotect_cloud_secret(&protected_token)?;
    let session_expires_at_ms = get(SETTING_CLOUD_SESSION_EXPIRES_AT_MS)?
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if session_expires_at_ms > 0 && session_expires_at_ms <= current_timestamp_ms() {
        return Err(
            "{\"error\":{\"code\":\"session_expired\",\"message\":\"Cloud session expired; sign in again\"}}"
                .to_string(),
        );
    }
    let display_name = get(SETTING_CLOUD_DISPLAY_NAME_DPAPI)?
        .and_then(|value| unprotect_cloud_secret(&value).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let email = get(SETTING_CLOUD_EMAIL_DPAPI)?
        .and_then(|value| unprotect_cloud_secret(&value).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let provider = get(SETTING_CLOUD_PROVIDER)?;
    Ok(CloudUserSessionResponse {
        user_id,
        login: "Google".to_string(),
        client_id,
        key_id,
        session_token,
        session_expires_at_ms,
        provider,
        display_name,
        email,
    })
}

fn read_cloud_private_key(core: &NetstitchCore) -> Result<Vec<u8>, String> {
    let protected = core
        .get_app_setting(SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI)
        .map_err(|error| error.to_string())?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"missing_client_key\",\"message\":\"Cloud client key is missing\"}}".to_string())?;
    let key_b64 = unprotect_cloud_secret(&protected)?;
    URL_SAFE_NO_PAD
        .decode(key_b64.trim())
        .map_err(|_| "{\"error\":{\"code\":\"missing_client_key\",\"message\":\"Stored cloud client key is invalid\"}}".to_string())
}

fn default_cloud_author_signature(core: &NetstitchCore) -> Result<String, String> {
    core.get_app_setting(SETTING_CLOUD_UPLOAD_NICKNAME)
        .map_err(|error| error.to_string())?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "{\"error\":{\"code\":\"missing_author_signature\",\"message\":\"Nickname is required before cloud upload\"}}".to_string())
}

fn imported_cloud_app_id_for_endpoint(
    endpoint: &netstitch_shared::models::ObservedEndpoint,
    app: Option<&TrackedApp>,
) -> Option<String> {
    endpoint
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

fn connector_cloud_app_for_tracked_app(
    app: Option<&TrackedApp>,
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

fn build_local_app_proof_for_tracked_app(
    app: &TrackedApp,
    preferred_app_id: Option<&str>,
    preferred_display_name: Option<&str>,
) -> Option<VerifiedAppProof> {
    if app.exe_path.as_os_str().is_empty()
        || app
            .exe_path
            .to_string_lossy()
            .starts_with("netstitch-csv-import://")
    {
        return None;
    }
    let probe = probe_authenticode_signature(&app.exe_path).ok()?;
    let stable_identity = stable_app_identity(StableAppIdentityInput {
        authenticode_leaf_spki_sha256: probe
            .status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("Valid"))
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
    let display_name = probe
        .product_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            preferred_display_name
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .or_else(|| {
            app.exe_path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "Unknown application".to_string());
    Some(VerifiedAppProof {
        app_id,
        platform: env::consts::OS.to_string(),
        method: VerifiedAppMethod::StableFileIdentity,
        status: VerifiedAppStatus::Verified,
        signature_chain_valid: true,
        leaf_spki_sha256: Some(stable_identity.signature_key.clone()),
        subject: Some(stable_identity.subject),
        issuer: probe.issuer,
        display_name: Some(display_name.clone()),
        publisher_name: probe.company_name.filter(|value| !value.trim().is_empty()),
        short_app_id: Some(short_app_id_from_display(
            &display_name,
            &stable_identity.signature_key,
        )),
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
        method: VerifiedAppMethod::StableFileIdentity,
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

fn endpoint_to_cloud_row(
    endpoint: &netstitch_shared::models::ObservedEndpoint,
    app_id: &str,
    source_kind: CloudSourceKind,
) -> CloudObservationRow {
    let domain_raw = endpoint
        .enrichment
        .as_ref()
        .and_then(|enrichment| enrichment.domain_name.clone())
        .filter(|value| !value.trim().is_empty());
    CloudObservationRow {
        app_id: app_id.to_string(),
        author_signature: None,
        remote_ip: endpoint.remote_ip,
        remote_port: endpoint.remote_port,
        protocol: endpoint.protocol,
        connection_state: endpoint.connection_state,
        requests: endpoint.hits,
        first_seen_ms: endpoint.first_seen_ms,
        last_seen_ms: endpoint.last_seen_ms.max(endpoint.first_seen_ms),
        failed_hits: endpoint.failed_hits,
        successful_hits: endpoint.successful_hits,
        domain_raw,
        domain_verified: None,
        domain_status: netstitch_shared::CloudDomainStatus::None,
        trust_level: if source_kind == CloudSourceKind::VerifiedUpload {
            CloudTrustLevel::VerifiedUploadCandidate
        } else {
            CloudTrustLevel::CloudImportUntrusted
        },
        source_kind,
        app_signature_key: endpoint.app_signature_key.clone(),
        app_signature_subject: endpoint.app_signature_subject.clone(),
        app_signature_issuer: endpoint.app_signature_issuer.clone(),
        cloud_observation_id: None,
    }
}

fn build_cloud_upload_groups(snapshot: &SnapshotResponse) -> UploadBuildResult {
    let apps_by_id = snapshot
        .tracked_apps
        .iter()
        .filter_map(|app| app.id.map(|id| (id, app)))
        .collect::<BTreeMap<_, _>>();
    let mut local_proofs = BTreeMap::<u64, Option<VerifiedAppProof>>::new();
    let mut groups = BTreeMap::<UploadGroupKey, UploadGroup>::new();
    let mut skipped_non_public_rows = 0usize;

    for endpoint in snapshot
        .observed_endpoints
        .iter()
        .filter(|item| item.is_confirmed)
    {
        if !netstitch_shared::cloud_observation_ip_is_public(endpoint.remote_ip) {
            skipped_non_public_rows += 1;
            continue;
        }
        let app = apps_by_id.get(&endpoint.tracked_app_id).copied();
        let imported_app_id = imported_cloud_app_id_for_endpoint(endpoint, app);
        let connector_cloud_app = connector_cloud_app_for_tracked_app(app);
        let imported_signature = endpoint
            .app_signature_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let local_proof = if let Some(app) = app {
            if !local_proofs.contains_key(&endpoint.tracked_app_id) {
                local_proofs.insert(
                    endpoint.tracked_app_id,
                    build_local_app_proof_for_tracked_app(
                        app,
                        connector_cloud_app.map(|known| known.app_id),
                        connector_cloud_app.map(|known| known.display_name),
                    ),
                );
            }
            local_proofs
                .get(&endpoint.tracked_app_id)
                .cloned()
                .flatten()
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
                endpoint.app_signature_subject.clone(),
                endpoint.app_signature_issuer.clone(),
                local_proof
                    .as_ref()
                    .and_then(|proof| proof.display_name.clone()),
                local_proof
                    .as_ref()
                    .and_then(|proof| proof.publisher_name.clone()),
            )
        };
        let key = UploadGroupKey {
            app_id: app_id.clone(),
            source_kind,
            signature_key,
        };
        groups
            .entry(key)
            .or_insert_with(|| UploadGroup {
                app_proof,
                rows: Vec::new(),
            })
            .rows
            .push(endpoint_to_cloud_row(endpoint, &app_id, source_kind));
    }

    UploadBuildResult {
        groups,
        skipped_non_public_rows,
    }
}

fn compact_json_text(value: &str) -> String {
    serde_json::from_str::<serde_json::Value>(value)
        .and_then(|parsed| serde_json::to_string(&parsed))
        .unwrap_or_else(|_| value.to_string())
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

#[derive(Debug, Deserialize)]
struct AuthenticodeProbe {
    status: Option<String>,
    leaf_spki_sha256: Option<String>,
    issuer: Option<String>,
    company_name: Option<String>,
    product_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CloudSignatureBackfillAppReport {
    tracked_app_id: u64,
    display_name: String,
    exe_path: String,
    cloud_app_id: Option<String>,
    status: String,
    signature_key: Option<String>,
    endpoints_updated: usize,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct CloudSignatureBackfillReport {
    dry_run: bool,
    apps_seen: usize,
    apps_signed: usize,
    endpoints_updated: usize,
    app_reports: Vec<CloudSignatureBackfillAppReport>,
}

pub fn backfill_cloud_signatures(json: bool, dry_run: bool) -> Result<()> {
    let core = NetstitchCore::bootstrap().context("failed to bootstrap NetStitch core")?;
    let endpoint_counts = core
        .list_endpoints()
        .context("failed to load observed endpoints")?
        .into_iter()
        .fold(BTreeMap::<u64, usize>::new(), |mut counts, endpoint| {
            *counts.entry(endpoint.tracked_app_id).or_default() += 1;
            counts
        });

    let mut report = CloudSignatureBackfillReport {
        dry_run,
        apps_seen: 0,
        apps_signed: 0,
        endpoints_updated: 0,
        app_reports: Vec::new(),
    };

    for app in core
        .list_all_tracked_apps()
        .context("failed to load tracked apps")?
    {
        let Some(tracked_app_id) = app.id else {
            continue;
        };
        let endpoint_count = endpoint_counts.get(&tracked_app_id).copied().unwrap_or(0);
        if endpoint_count == 0 {
            continue;
        }
        report.apps_seen += 1;

        let display_name = app
            .display_name
            .clone()
            .or(app.process_name.clone())
            .unwrap_or_else(|| app.exe_path.display().to_string());
        let cloud_app_id = app.cloud_app_id.clone().or_else(|| {
            app.connector_id
                .as_deref()
                .and_then(netstitch_shared::cloud_app_for_connector_id)
                .map(|item| item.app_id.to_string())
        });

        let Some(cloud_app_id_value) = cloud_app_id.as_deref() else {
            report.app_reports.push(CloudSignatureBackfillAppReport {
                tracked_app_id,
                display_name,
                exe_path: app.exe_path.display().to_string(),
                cloud_app_id,
                status: "skipped_no_cloud_app_id".to_string(),
                signature_key: None,
                endpoints_updated: 0,
                message: Some("tracked app has no known cloud app id".to_string()),
            });
            continue;
        };

        if !app.exe_path.is_file() {
            report.app_reports.push(CloudSignatureBackfillAppReport {
                tracked_app_id,
                display_name,
                exe_path: app.exe_path.display().to_string(),
                cloud_app_id,
                status: "skipped_missing_executable".to_string(),
                signature_key: None,
                endpoints_updated: 0,
                message: Some("executable path is not available on this machine".to_string()),
            });
            continue;
        }

        let probe = match probe_authenticode_signature(&app.exe_path) {
            Ok(probe) => probe,
            Err(error) => {
                report.app_reports.push(CloudSignatureBackfillAppReport {
                    tracked_app_id,
                    display_name,
                    exe_path: app.exe_path.display().to_string(),
                    cloud_app_id,
                    status: "skipped_probe_error".to_string(),
                    signature_key: None,
                    endpoints_updated: 0,
                    message: Some(error.to_string()),
                });
                continue;
            }
        };

        let authenticode_key = probe.leaf_spki_sha256.as_deref().unwrap_or("").trim();
        let authenticode_valid = probe
            .status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("Valid"));
        let Some(stable_identity) = stable_app_identity(StableAppIdentityInput {
            authenticode_leaf_spki_sha256: if authenticode_valid && !authenticode_key.is_empty() {
                Some(authenticode_key.to_string())
            } else {
                None
            },
            company_name: probe.company_name.clone(),
            product_name: probe.product_name.clone(),
        }) else {
            report.app_reports.push(CloudSignatureBackfillAppReport {
                tracked_app_id,
                display_name,
                exe_path: app.exe_path.display().to_string(),
                cloud_app_id,
                status: "skipped_no_stable_identity".to_string(),
                signature_key: probe.leaf_spki_sha256,
                endpoints_updated: 0,
                message: Some(format!(
                    "No Authenticode key or stable VersionInfo identity; Authenticode status is {}",
                    probe.status.unwrap_or_else(|| "unknown".to_string())
                )),
            });
            continue;
        };

        let endpoints_updated = core
            .backfill_observation_cloud_signature(
                tracked_app_id,
                cloud_app_id_value,
                &stable_identity.signature_key,
                Some(stable_identity.subject.as_str()),
                probe.issuer.as_deref(),
                &format!("local_app_backfill:{}", stable_identity.source),
                dry_run,
            )
            .with_context(|| format!("failed to backfill observations for {display_name}"))?;
        report.apps_signed += 1;
        report.endpoints_updated += endpoints_updated;
        report.app_reports.push(CloudSignatureBackfillAppReport {
            tracked_app_id,
            display_name,
            exe_path: app.exe_path.display().to_string(),
            cloud_app_id,
            status: if dry_run {
                "would_update".to_string()
            } else {
                "updated".to_string()
            },
            signature_key: Some(stable_identity.signature_key),
            endpoints_updated,
            message: None,
        });
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Cloud signature backfill: apps seen {}, signed {}, endpoints {}{}",
            report.apps_seen,
            report.apps_signed,
            report.endpoints_updated,
            if dry_run { " (dry-run)" } else { "" }
        );
        for item in &report.app_reports {
            println!(
                "- #{} {}: {} endpoints={} cloud_app_id={} signature={}",
                item.tracked_app_id,
                item.display_name,
                item.status,
                item.endpoints_updated,
                item.cloud_app_id.as_deref().unwrap_or("-"),
                item.signature_key.as_deref().unwrap_or("-")
            );
            if let Some(message) = item.message.as_deref() {
                println!("  {message}");
            }
        }
    }
    Ok(())
}

fn probe_authenticode_signature(exe_path: &Path) -> Result<AuthenticodeProbe> {
    #[cfg(windows)]
    {
        native_windows_authenticode_probe(exe_path)
    }
    #[cfg(not(windows))]
    {
        let _ = exe_path;
        Err(anyhow::anyhow!(
            "Authenticode proof is only available on Windows"
        ))
    }
}

#[cfg(windows)]
#[cfg(windows)]
struct WindowsSignatureInfo {
    status: String,
    leaf_spki_sha256: Option<String>,
    issuer: Option<String>,
}

#[cfg(windows)]
fn native_windows_authenticode_probe(exe_path: &Path) -> Result<AuthenticodeProbe> {
    let version = netstitch_connectors::app_file_metadata(exe_path);
    let signature =
        windows_authenticode_signature(exe_path).unwrap_or_else(|_| WindowsSignatureInfo {
            status: "NotSigned".to_string(),
            leaf_spki_sha256: None,
            issuer: None,
        });

    Ok(AuthenticodeProbe {
        status: Some(signature.status),
        leaf_spki_sha256: signature.leaf_spki_sha256,
        issuer: signature.issuer,
        company_name: version.company_name,
        product_name: version.product_name,
    })
}

#[cfg(windows)]
fn windows_authenticode_signature(exe_path: &Path) -> Result<WindowsSignatureInfo> {
    let wide_path = path_wide_null(exe_path);
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
fn signer_certificate_identity(wide_path: &[u16]) -> Result<SignerCertificateIdentity> {
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
        return Err(anyhow::anyhow!("embedded signer certificate was not found"));
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
        return Err(anyhow::anyhow!("signer info was not found"));
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
        return Err(anyhow::anyhow!("signer info could not be read"));
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
        return Err(anyhow::anyhow!("signer certificate was not found in store"));
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
fn path_wide_null(value: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    value
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.monitor.status(&state.core).await;
    Json(serde_json::json!({
        "ok": true,
        "monitor_status": status,
        "monitor_details": state.monitor.details(&state.core).await,
        "database_path": state.core.paths().database_path,
    }))
}

async fn root_web_ui(State(state): State<AppState>) -> axum::response::Response {
    if browser_access_disabled(&state) {
        return disabled_browser_ui_response();
    }
    browser_ui_response(&state).unwrap_or_else(IntoResponse::into_response)
}

fn browser_ui_response(state: &AppState) -> WatcherResult<axum::response::Response> {
    Ok((
        [(
            CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, max-age=0",
        )],
        Html(browser_ui_html(state)),
    )
        .into_response())
}

fn browser_ui_html(_state: &AppState) -> String {
    BROWSER_UI_HTML
        .replace("__BUILD_VERSION__", &runtime_build_version())
        .replace("__RUNTIME_PLATFORM__", runtime_platform_label())
        .replace("__WATCHER_VERSION__", &runtime_module_version("watcher"))
        .replace("__TOOL_VERSION__", &runtime_module_version("tool"))
}

fn browser_access_disabled(state: &AppState) -> bool {
    state
        .core
        .web_access_localhost_enabled()
        .map(|enabled| !enabled)
        .unwrap_or(true)
}

fn is_desktop_client(headers: &HeaderMap) -> bool {
    headers
        .get(CLIENT_HEADER_NAME)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case(CLIENT_HEADER_DESKTOP_UI))
}

fn browser_access_disabled_error() -> WatcherError {
    WatcherError::with_status(
        StatusCode::FORBIDDEN,
        anyhow::anyhow!("browser access is disabled; enable Web access in the desktop header"),
    )
}

fn disabled_browser_ui_response() -> axum::response::Response {
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NetStitch Web {} ({})</title>
  <style>
    :root {{
      color-scheme: dark;
      --bg: #181818;
      --panel: #252526;
      --border: #3c3c3c;
      --text: #cccccc;
      --strong: #f0f0f0;
      --muted: #9d9d9d;
    }}
    * {{ box-sizing: border-box; }}
    html, body {{ margin: 0; width: 100%; height: 100%; background: var(--bg); color: var(--text); font: 14px \"Segoe UI\", system-ui, sans-serif; }}
    body {{ display: grid; place-items: center; padding: 24px; }}
    main {{ width: min(520px, 100%); padding: 22px 24px; border: 1px solid var(--border); border-radius: 6px; background: var(--panel); }}
    h1 {{ margin: 0 0 10px; color: var(--strong); font-size: 20px; }}
    p {{ margin: 0; line-height: 1.5; }}
    p + p {{ margin-top: 8px; color: var(--muted); }}
  </style>
</head>
<body>
  <main>
    <h1>Web access is disabled</h1>
    <p>Enable Web access in the desktop header to use the browser interface.</p>
    <p>This page will start working again after desktop re-enables the web server.</p>
  </main>
</body>
</html>"#,
        runtime_build_version(),
        runtime_platform_label()
    );
    (
        [(
            CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, max-age=0",
        )],
        Html(html),
    )
        .into_response()
}

async fn browser_http_access_gate(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if is_desktop_client(request.headers()) {
        return add_browser_security_headers(next.run(request).await);
    }

    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let origin = header_to_string(request.headers(), ORIGIN);
    let user_agent = header_to_string(request.headers(), USER_AGENT);

    if request.uri().path() == "/" {
        return add_browser_security_headers(next.run(request).await);
    }

    if request.uri().path() == "/health" {
        return add_browser_security_headers(next.run(request).await);
    }

    if browser_access_disabled(&state) {
        audit_web_security_event(
            "web_access_disabled",
            method.as_str(),
            &path,
            origin.as_deref(),
            user_agent.as_deref(),
        );
        return add_browser_security_headers(browser_access_disabled_error().into_response());
    }

    if !browser_same_origin_allowed(&state, request.headers()) {
        audit_web_security_event(
            "web_origin_rejected",
            method.as_str(),
            &path,
            origin.as_deref(),
            user_agent.as_deref(),
        );
        return add_browser_security_headers(
            WatcherError::with_status(
                StatusCode::FORBIDDEN,
                anyhow::anyhow!("browser request origin is not allowed"),
            )
            .into_response(),
        );
    }

    if browser_public_get_route(&method, &path) || browser_public_post_route(&method, &path) {
        return add_browser_security_headers(next.run(request).await);
    }

    if !browser_web_auth_valid(&state, request.headers()) {
        audit_web_security_event(
            "web_key_required",
            method.as_str(),
            &path,
            origin.as_deref(),
            user_agent.as_deref(),
        );
        return add_browser_security_headers(
            WatcherError::with_status(
                StatusCode::UNAUTHORIZED,
                anyhow::anyhow!("web access key is required"),
            )
            .into_response(),
        );
    }

    if method != Method::GET && method != Method::HEAD {
        audit_web_security_event(
            "web_remote_action",
            method.as_str(),
            &path,
            origin.as_deref(),
            user_agent.as_deref(),
        );
    }

    add_browser_security_headers(next.run(request).await)
}

fn browser_public_get_route(method: &Method, path: &str) -> bool {
    if method != Method::GET && method != Method::HEAD {
        return false;
    }
    path.starts_with("/v1/assets/")
        || (path.starts_with("/v1/tracked-apps/") && path.ends_with("/icon"))
        || (path.starts_with("/v1/integrations/") && path.ends_with("/icon"))
        || path == "/v1/languages"
}

fn browser_public_post_route(method: &Method, path: &str) -> bool {
    method == Method::POST && path == "/v1/web-auth"
}

fn browser_web_auth_valid(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(cookie) = headers.get(COOKIE).and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let Some(expected_key) = web_access_key(state).ok() else {
        return false;
    };
    let expected_cookie = web_auth_cookie_value(&expected_key);
    cookie.split(';').any(|part| {
        let Some((name, value)) = part.trim().split_once('=') else {
            return false;
        };
        name == WEB_AUTH_COOKIE && value == expected_cookie
    })
}

fn web_access_key(_state: &AppState) -> WatcherResult<String> {
    let client_identifier = local_cloud_client_identifier().map_err(WatcherError::bad_request)?;
    derive_web_access_key(&client_identifier).ok_or_else(|| {
        WatcherError::with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!("failed to derive web access key"),
        )
    })
}

fn web_auth_cookie_value(web_key: &str) -> String {
    let digest = Sha256::digest(format!("netstitch_web_auth_cookie_v1\n{web_key}").as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn browser_same_origin_allowed(state: &AppState, headers: &HeaderMap) -> bool {
    let allowed_origins = web_allowed_origins(state);
    for header in [ORIGIN, REFERER] {
        let Some(value) = header_to_string(headers, header) else {
            continue;
        };
        if !allowed_origins
            .iter()
            .any(|origin| browser_origin_header_matches(&value, origin))
        {
            return false;
        }
    }
    true
}

fn browser_origin_header_matches(value: &str, origin: &str) -> bool {
    let value = value.trim();
    if value == origin {
        return true;
    }
    value.strip_prefix(origin).is_some_and(|rest| {
        rest.starts_with('/') || rest.starts_with('?') || rest.starts_with('#') || rest.is_empty()
    })
}

fn web_allowed_origins(state: &AppState) -> Vec<String> {
    let url = web_url_dto(state).url;
    let mut origins = Vec::new();
    if let Some(origin) = web_origin_from_url(&url) {
        origins.push(origin);
    }
    let port = state.bind_addr.port();
    for host in ["127.0.0.1", "localhost", "[::1]"] {
        origins.push(format!("{}://{host}:{port}", web_scheme()));
    }
    origins.sort();
    origins.dedup();
    origins
}

fn web_origin_from_url(url: &str) -> Option<String> {
    let (scheme, rest) = if let Some(rest) = url.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        ("http", rest)
    } else {
        return None;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    Some(format!("{scheme}://{authority}"))
}

fn header_to_string(headers: &HeaderMap, name: HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn add_browser_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("same-origin"),
    );
    if let Ok(value) = HeaderValue::from_str(WEB_CSP) {
        headers.insert(HeaderName::from_static("content-security-policy"), value);
    }
    response
}

fn audit_web_security_event(
    event_id: &'static str,
    method: &str,
    path: &str,
    origin: Option<&str>,
    user_agent: Option<&str>,
) {
    info!(
        event_id,
        entity_type = "browser_shell",
        action_type = method,
        path,
        origin = origin.unwrap_or(""),
        user_agent = user_agent.unwrap_or("")
    );
}

async fn languages() -> impl IntoResponse {
    Json(load_language_catalog())
}

async fn web_auth(
    State(state): State<AppState>,
    Json(request): Json<WebAuthRequest>,
) -> WatcherResult<impl IntoResponse> {
    let expected = web_access_key(&state)?;
    if request.key.trim() != expected {
        audit_web_security_event("web_key_rejected", "POST", "/v1/web-auth", None, None);
        return Err(WatcherError::with_status(
            StatusCode::UNAUTHORIZED,
            anyhow::anyhow!("web access key is invalid"),
        ));
    }

    let mut headers = HeaderMap::new();
    let secure = if web_scheme_is_https() {
        "; Secure"
    } else {
        ""
    };
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{}={}; Path=/; Max-Age=31536000; HttpOnly; SameSite=Strict{}",
            WEB_AUTH_COOKIE,
            web_auth_cookie_value(&expected),
            secure
        ))
        .map_err(|error| {
            WatcherError::with_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow::anyhow!("failed to prepare web auth cookie: {error}"),
            )
        })?,
    );
    Ok((headers, Json(WebAuthResponse { ok: true })))
}

async fn web_url(State(state): State<AppState>) -> impl IntoResponse {
    Json(web_url_dto(&state))
}

async fn system_events(
    State(state): State<AppState>,
    Query(query): Query<SystemEventsQuery>,
) -> WatcherResult<impl IntoResponse> {
    let events = state
        .core
        .list_system_events(query.limit.unwrap_or(250))
        .context("failed to list system events")?;
    Ok(Json(serde_json::json!({ "events": events })))
}

async fn record_system_event(
    State(state): State<AppState>,
    Json(request): Json<SystemEventRequestDto>,
) -> WatcherResult<impl IntoResponse> {
    let event = state
        .core
        .append_system_event(request)
        .context("failed to record system event")?;
    Ok(Json(event))
}

async fn endpoint_probe_targets() -> impl IntoResponse {
    Json(EndpointProbeTargetsDto {
        targets: load_endpoint_probe_targets(),
    })
}

async fn cloud_apps(
    State(state): State<AppState>,
    Query(query): Query<CloudAppsQuery>,
) -> WatcherResult<impl IntoResponse> {
    append_runtime_event(
        &state.core,
        "cloud",
        "apps_request",
        "info",
        None,
        None,
        serde_json::json!({
            "query_present": query.query.as_deref().is_some_and(|value| !value.trim().is_empty()),
            "publisher_present": query.publisher.as_deref().is_some_and(|value| !value.trim().is_empty()),
            "source_present": query.source.as_deref().is_some_and(|value| !value.trim().is_empty()),
        }),
    );
    let mut params = Vec::new();
    if let Some(value) = effective_cloud_query_text(query.query.as_deref()) {
        params.push(format!("query={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_query_text(query.publisher.as_deref()) {
        params.push(format!("publisher={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_query_text(query.source.as_deref()) {
        params.push(format!("source={}", url_component(&value)));
    }
    if params.is_empty() {
        append_runtime_event(
            &state.core,
            "cloud",
            "apps_response",
            "info",
            None,
            None,
            serde_json::json!({ "items": 0, "skipped": "empty_query" }),
        );
        return Ok(Json(CloudWebAppsResponse { items: Vec::new() }));
    }

    let client = cloud_http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let url = format!("{}/v1/apps?{}", cloud_base_url(), params.join("&"));
    let response = match cloud_get_json::<CloudAppsResponseDto>(&client, &url, None).await {
        Ok(response) => response,
        Err(error) => {
            append_runtime_event(
                &state.core,
                "cloud",
                "apps_response",
                "error",
                None,
                None,
                serde_json::json!({ "error": error.error.to_string() }),
            );
            return Err(error);
        }
    };
    append_runtime_event(
        &state.core,
        "cloud",
        "apps_response",
        "info",
        None,
        None,
        serde_json::json!({ "items": response.items.len() }),
    );
    Ok(Json(CloudWebAppsResponse {
        items: response.items,
    }))
}

async fn cloud_quota(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    append_runtime_event(
        &state.core,
        "cloud",
        "quota_request",
        "info",
        None,
        None,
        serde_json::json!({}),
    );
    let quota = match cloud_quota_snapshot().await {
        Ok(quota) => quota,
        Err(error) => {
            append_runtime_event(
                &state.core,
                "cloud",
                "quota_response",
                "error",
                None,
                None,
                serde_json::json!({ "error": error.error.to_string() }),
            );
            return Err(error);
        }
    };
    append_runtime_event(
        &state.core,
        "cloud",
        "quota_response",
        "info",
        None,
        None,
        serde_json::json!({
            "upload_remaining": quota.upload.limit_count.saturating_sub(quota.upload.used_count),
            "download_remaining": quota.download.limit_count.saturating_sub(quota.download.used_count),
        }),
    );
    Ok(Json(quota))
}

async fn cloud_local_author(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let nickname = state
        .core
        .get_app_setting(SETTING_CLOUD_UPLOAD_NICKNAME)
        .map_err(WatcherError::from)?
        .map(|value| value.trim().to_string())
        .filter(|value| validate_author_signature(value).is_ok());
    Ok(Json(CloudLocalAuthorResponse { nickname }))
}

async fn cloud_auth_state(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let nickname = state
        .core
        .get_app_setting(SETTING_CLOUD_UPLOAD_NICKNAME)
        .map_err(WatcherError::from)?
        .map(|value| value.trim().to_string())
        .filter(|value| validate_author_signature(value).is_ok());
    let session = read_cloud_session(&state.core).ok();
    let authorization_label = session.as_ref().and_then(|session| {
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
    });
    let client_identifier = local_cloud_client_identifier().unwrap_or_default();
    Ok(Json(CloudWebAuthStateResponse {
        authenticated: session.is_some(),
        authorization_label,
        nickname,
        client_identifier,
    }))
}

async fn cloud_my_apps(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let Ok(session) = read_cloud_session(&state.core) else {
        return Ok(Json(CloudWebUserAppsResponse { items: Vec::new() }));
    };
    let client = cloud_http_client(CLOUD_REFRESH_HTTP_TIMEOUT)?;
    let url = format!("{}/v1/users/me/apps", cloud_base_url());
    let response =
        cloud_get_json::<CloudUserAppsResponseDto>(&client, &url, Some(&session.session_token))
            .await?;
    let mut items = response
        .items
        .into_iter()
        .map(|item| CloudWebUserAppSummary {
            app_id: item.app_id,
            display_name: item.display_name,
            publisher_name: item.publisher_name,
            author_signature: item.author_signature,
            visibility: item.visibility,
            endpoint_count: item.endpoint_count,
            available_row_count: item.available_row_count.max(item.endpoint_count),
            author_rows: item.endpoint_count,
            total_rows: (item.total_endpoint_count > 0).then_some(item.total_endpoint_count),
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(Json(CloudWebUserAppsResponse { items }))
}

async fn set_cloud_nickname(
    State(state): State<AppState>,
    Json(request): Json<CloudNicknameRequest>,
) -> WatcherResult<impl IntoResponse> {
    let nickname =
        validate_author_signature(&request.nickname).map_err(WatcherError::bad_request)?;
    state
        .core
        .set_app_setting(SETTING_CLOUD_UPLOAD_NICKNAME, &nickname)
        .map_err(WatcherError::from)?;
    Ok(Json(CloudNicknameResponse {
        nickname,
        status: "accepted".to_string(),
    }))
}

async fn cloud_sign_out(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    for key in [
        SETTING_CLOUD_PROVIDER,
        SETTING_CLOUD_EMAIL_DPAPI,
        SETTING_CLOUD_DISPLAY_NAME_DPAPI,
        SETTING_CLOUD_USER_ID,
        SETTING_CLOUD_CLIENT_ID,
        SETTING_CLOUD_KEY_ID,
        SETTING_CLOUD_SESSION_TOKEN_DPAPI,
        SETTING_CLOUD_CLIENT_PRIVATE_KEY_DPAPI,
        SETTING_CLOUD_SESSION_EXPIRES_AT_MS,
    ] {
        state
            .core
            .set_app_setting(key, "")
            .map_err(WatcherError::from)?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn cloud_upload(
    State(state): State<AppState>,
    Json(request): Json<CloudUploadRequest>,
) -> WatcherResult<impl IntoResponse> {
    append_runtime_event(
        &state.core,
        "cloud",
        "upload_request",
        "info",
        None,
        None,
        serde_json::json!({
            "visibility": cloud_visibility_value(request.visibility.unwrap_or_default()),
            "author_signature_present": request.author_signature.as_deref().is_some_and(|value| !value.trim().is_empty()),
        }),
    );
    let author_signature = request
        .author_signature
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .map(Ok)
        .unwrap_or_else(|| default_cloud_author_signature(&state.core))
        .map_err(WatcherError::bad_request)
        .and_then(|value| validate_author_signature(&value).map_err(WatcherError::bad_request))?;
    let visibility = request.visibility.unwrap_or_default();
    let client_identifier = local_cloud_client_identifier().map_err(WatcherError::bad_request)?;
    let session = read_cloud_session(&state.core).map_err(WatcherError::bad_request)?;
    let private_key = read_cloud_private_key(&state.core).map_err(WatcherError::bad_request)?;
    let monitor_status = state.monitor.status(&state.core).await;
    let snapshot = state
        .core
        .snapshot(monitor_status, None)
        .context("failed to load cloud upload candidates")?;
    let mut upload_build = build_cloud_upload_groups(&snapshot);
    if upload_build.groups.is_empty() {
        let message = if upload_build.skipped_non_public_rows > 0 {
            "{\"error\":{\"code\":\"nothing_to_upload\",\"message\":\"Only private, local, documentation or other non-public IP ranges were selected for cloud upload\"}}"
        } else {
            "{\"error\":{\"code\":\"nothing_to_upload\",\"message\":\"No confirmed rows with cloud application identifiers\"}}"
        };
        return Err(WatcherError::bad_request(message));
    }

    let request_count = upload_build.groups.len();
    let client = cloud_http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    let base_url = cloud_base_url();
    let client_version = runtime_build_version();
    let upload_operation_id = cloud_upload_operation_id(&client_identifier);
    let mut accepted_rows = 0usize;
    let mut responses = Vec::<serde_json::Value>::new();

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
        let body_json =
            serde_json::to_vec(&body).context("failed to serialize cloud upload body")?;
        let jwt = sign_request_jwt(CloudSignedJwtInput {
            key_id: &session.key_id,
            private_key_pkcs8_der: &private_key,
            client_id: &session.client_id,
            scope: operation_scope(CloudOperationKind::Upload),
            body: &body_json,
            client_version: &client_version,
            now_seconds: current_timestamp_ms() / 1000,
            ttl_seconds: 5 * 60,
        })
        .map_err(|error| {
            WatcherError::bad_request(format!("cloud upload JWT signing failed: {error}"))
        })?;

        let response = client
            .post(format!("{base_url}/v1/observations/append"))
            .bearer_auth(&session.session_token)
            .header("x-netstitch-jwt", jwt)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await
            .map_err(|error| WatcherError::from(anyhow::anyhow!("cloud upload failed: {error}")))?;
        let is_success = response.status().is_success();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "cloud upload failed".to_string());
        if !is_success {
            return Err(WatcherError::bad_request(compact_json_text(&response_text)));
        }
        let value = serde_json::from_str::<serde_json::Value>(&response_text)
            .unwrap_or_else(|_| serde_json::json!({ "raw": response_text }));
        accepted_rows += value
            .get("accepted_rows")
            .and_then(|item| item.as_u64())
            .unwrap_or(0) as usize;
        responses.push(value);
    }

    let quota = cloud_quota_snapshot().await.ok();
    append_runtime_event(
        &state.core,
        "cloud",
        "upload_response",
        "info",
        None,
        None,
        serde_json::json!({
            "accepted_rows": accepted_rows,
            "request_count": request_count,
            "skipped_non_public_rows": upload_build.skipped_non_public_rows,
            "upload_remaining": quota.as_ref().map(|value| value.upload.limit_count.saturating_sub(value.upload.used_count)),
            "download_remaining": quota.as_ref().map(|value| value.download.limit_count.saturating_sub(value.download.used_count)),
        }),
    );
    Ok(Json(CloudUploadResponse {
        accepted_rows,
        request_count,
        skipped_non_public_rows: upload_build.skipped_non_public_rows,
        responses,
        quota,
    }))
}

async fn cloud_observations(
    State(state): State<AppState>,
    Query(query): Query<CloudObservationsQuery>,
) -> WatcherResult<impl IntoResponse> {
    append_runtime_event(
        &state.core,
        "cloud",
        "download_request",
        "info",
        Some("cloud_app"),
        Some(query.app_id.trim()),
        serde_json::json!({
            "app_id_present": !query.app_id.trim().is_empty(),
            "expected_total": query.expected_total,
            "own_scope": query.own_scope,
            "visibility": query.visibility.unwrap_or_default().as_query_value(),
        }),
    );
    let session = read_cloud_session(&state.core).ok();
    let rows = match cloud_observations_rows(&query, &cloud_base_url(), session).await {
        Ok(rows) => rows,
        Err(error) => {
            append_runtime_event(
                &state.core,
                "cloud",
                "download_response",
                "error",
                Some("cloud_app"),
                Some(query.app_id.trim()),
                serde_json::json!({ "error": error.error.to_string() }),
            );
            return Err(error);
        }
    };
    let quota = cloud_quota_snapshot().await.ok();
    append_runtime_event(
        &state.core,
        "cloud",
        "download_response",
        "info",
        Some("cloud_app"),
        Some(query.app_id.trim()),
        serde_json::json!({
            "rows": rows.len(),
            "upload_remaining": quota.as_ref().map(|value| value.upload.limit_count.saturating_sub(value.upload.used_count)),
            "download_remaining": quota.as_ref().map(|value| value.download.limit_count.saturating_sub(value.download.used_count)),
        }),
    );
    Ok(Json(CloudWebObservationsResponse { rows, quota }))
}

async fn cloud_observations_rows(
    query: &CloudObservationsQuery,
    base_url: &str,
    session: Option<CloudUserSessionResponse>,
) -> WatcherResult<Vec<CloudWebObservationRow>> {
    let app_id = query.app_id.trim();
    if app_id.is_empty() {
        return Err(WatcherError::bad_request(
            "{\"error\":{\"code\":\"cloud_app_required\",\"message\":\"Choose an application before downloading\"}}",
        ));
    }

    let client_identifier = local_cloud_client_identifier().map_err(WatcherError::bad_request)?;
    let download_operation_id = cloud_download_operation_id(&client_identifier);
    let mut base_params = vec![
        format!("client_identifier={}", url_component(&client_identifier)),
        format!("app_id={}", url_component(app_id)),
        format!("operation_id={}", url_component(&download_operation_id)),
    ];
    if let Some(value) = effective_cloud_query_text(query.ip.as_deref()) {
        base_params.push(format!("ip={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_query_text(query.domain.as_deref()) {
        base_params.push(format!("domain={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_query_text(query.port.as_deref()) {
        base_params.push(format!("port={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_query_text(query.source.as_deref()) {
        base_params.push(format!("source={}", url_component(&value)));
    }
    if let Some(value) = effective_cloud_protocol(query.protocol.as_deref()) {
        base_params.push(format!("protocol={}", url_component(&value)));
    }
    let visibility_scope = query.visibility.unwrap_or_default();
    base_params.push(format!("visibility={}", visibility_scope.as_query_value()));
    let bearer = if query.own_scope {
        let session = session
            .as_ref()
            .ok_or_else(|| WatcherError::bad_request("{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Author scoped download requires Google sign-in\"}}"))?;
        base_params.push(format!(
            "author_user_id={}",
            url_component(&session.user_id)
        ));
        Some(session.session_token.clone())
    } else {
        match visibility_scope {
            CloudObservationVisibilityScope::Private => Some(
                session
                    .as_ref()
                    .ok_or_else(|| WatcherError::bad_request("{\"error\":{\"code\":\"not_authenticated\",\"message\":\"Private cloud download requires Google sign-in\"}}"))?
                    .session_token
                    .clone(),
            ),
            CloudObservationVisibilityScope::All => {
                session.as_ref().map(|item| item.session_token.clone())
            }
            CloudObservationVisibilityScope::Public => None,
        }
    };

    let client = cloud_http_client(CLOUD_INTERACTIVE_HTTP_TIMEOUT)?;
    const PAGE_SIZE: usize = 500;
    let mut all_rows = Vec::new();
    let mut seen_rows = BTreeSet::new();
    let mut total_rows = query.expected_total.filter(|value| *value > 0);
    let mut offset = 0usize;
    loop {
        let mut params = base_params.clone();
        params.push(format!("limit={PAGE_SIZE}"));
        params.push(format!("offset={offset}"));
        let url = format!("{base_url}/v1/observations?{}", params.join("&"));
        let response =
            cloud_get_json::<CloudObservationDownloadPage>(&client, &url, bearer.as_deref())
                .await?;
        let page_len = response.rows.len();
        if let Ok(response_total) = usize::try_from(response.total) {
            if response_total > 0 {
                total_rows =
                    Some(total_rows.map_or(response_total, |total| total.max(response_total)));
            }
        }
        let before_rows = all_rows.len();
        for item in response.rows {
            let is_private = item.visibility == Some(CloudObservationVisibility::Private);
            if seen_rows.insert(cloud_observation_dedupe_key(&item.row, is_private)) {
                all_rows.push(CloudCollectedObservationRow {
                    is_private,
                    row: item.row,
                });
            }
        }
        let added_rows = all_rows.len().saturating_sub(before_rows);
        if !all_rows.is_empty() {
            total_rows = Some(total_rows.map_or(all_rows.len(), |total| total.max(all_rows.len())));
        }
        offset = offset.saturating_add(page_len);
        if page_len < PAGE_SIZE || added_rows == 0 {
            break;
        }
    }
    let app_name = query
        .app_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(app_id)
        .to_string();
    let rows = all_rows
        .into_iter()
        .enumerate()
        .map(|(index, item)| cloud_web_observation_row(index, item.is_private, item.row, &app_name))
        .collect();
    Ok(rows)
}

fn cloud_observation_dedupe_key(row: &CloudObservationRow, is_private: bool) -> String {
    let visibility = if is_private { "private" } else { "public" };
    match row.cloud_observation_id.as_deref() {
        Some(row_id) => format!("{visibility}|{row_id}"),
        None => format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{}",
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

fn web_url_dto(state: &AppState) -> WebUrlDto {
    let port = state.bind_addr.port();
    let bind_ip = state.bind_addr.ip();
    let host = if bind_ip.is_unspecified() {
        local_machine_ip_for_web_url().unwrap_or_else(|| "127.0.0.1".to_string())
    } else {
        bind_ip.to_string()
    };
    let used_localhost_fallback = host == "127.0.0.1";
    let web_key = web_access_key(state).ok();
    let fragment = web_key
        .as_deref()
        .map(|key| format!("#web_key={key}"))
        .unwrap_or_default();
    WebUrlDto {
        url: format!("{}://{host}:{port}/{fragment}", web_scheme()),
        used_localhost_fallback,
    }
}

fn local_machine_ip_for_web_url() -> Option<String> {
    for target in load_endpoint_probe_targets() {
        let probe_target = target
            .strip_prefix("tcp://")
            .or_else(|| target.strip_prefix("udp://"))
            .unwrap_or(target.as_str());
        let Ok(remote) = probe_target.parse::<SocketAddr>() else {
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
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
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

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for path in paths {
        if !deduped.contains(&path) {
            deduped.push(path);
        }
    }
    deduped
}

async fn close_times_icon() -> impl IntoResponse {
    icon_bytes_response(CLOSE_TIMES_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn module_stop_icon() -> impl IntoResponse {
    icon_bytes_response(MODULE_STOP_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn module_close_icon() -> impl IntoResponse {
    icon_bytes_response(MODULE_CLOSE_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn module_reorder_left_icon() -> impl IntoResponse {
    icon_bytes_response(MODULE_REORDER_LEFT_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn copy_icon() -> impl IntoResponse {
    icon_bytes_response(COPY_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn help_circle_icon() -> impl IntoResponse {
    icon_bytes_response(HELP_CIRCLE_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn confirm_filtered_icon() -> impl IntoResponse {
    icon_bytes_response(CONFIRM_FILTERED_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn trash_icon() -> impl IntoResponse {
    icon_bytes_response(TRASH_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn unconfirm_filtered_icon() -> impl IntoResponse {
    icon_bytes_response(UNCONFIRM_FILTERED_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn ignore_address_icon() -> impl IntoResponse {
    icon_bytes_response(IGNORE_ADDRESS_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn export_cloud_icon() -> impl IntoResponse {
    icon_bytes_response(EXPORT_CLOUD_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn import_cloud_icon() -> impl IntoResponse {
    icon_bytes_response(IMPORT_FROM_CLOUD_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn my_publications_icon() -> impl IntoResponse {
    icon_bytes_response(MY_PUBLICATIONS_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn import_csv_icon() -> impl IntoResponse {
    icon_bytes_response(IMPORT_CSV_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn export_csv_icon() -> impl IntoResponse {
    icon_bytes_response(EXPORT_CSV_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn monitoring_icon() -> impl IntoResponse {
    icon_bytes_response(MONITORING_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn sort_idle_icon() -> impl IntoResponse {
    icon_bytes_response(SORT_IDLE_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn sort_asc_icon() -> impl IntoResponse {
    icon_bytes_response(SORT_ASC_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn sort_desc_icon() -> impl IntoResponse {
    icon_bytes_response(SORT_DESC_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

async fn tracked_app_icon(
    State(state): State<AppState>,
    AxumPath(tracked_app_id): AxumPath<u64>,
) -> WatcherResult<axum::response::Response> {
    let monitor_status = state.monitor.status(&state.core).await;
    let snapshot = state
        .core
        .snapshot(monitor_status, None)
        .context("failed to load tracked app icons")?;
    let icon_path = snapshot
        .tracked_apps
        .iter()
        .find(|app| app.id == Some(tracked_app_id))
        .and_then(|app| app.icon_path.as_deref());

    Ok(icon_file_response(
        icon_path,
        &[
            state.core.paths().icon_cache_dir.as_path(),
            state.core.paths().connector_icon_dir.as_path(),
        ],
    ))
}

async fn integration_module_icon(
    State(state): State<AppState>,
    AxumPath(module_id): AxumPath<String>,
) -> WatcherResult<axum::response::Response> {
    let monitor_status = state.monitor.status(&state.core).await;
    let snapshot = state
        .core
        .snapshot(monitor_status, None)
        .context("failed to load integration module icon")?;
    let icon_path = snapshot
        .integration_modules
        .iter()
        .find(|module| module.id == module_id)
        .and_then(|module| module.icon_path.as_deref());
    let allowed_dirs = integration_module_icon_allowed_dirs(&state);
    let allowed_dir_refs = allowed_dirs
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();

    Ok(icon_file_response(icon_path, &allowed_dir_refs))
}

async fn snapshot(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> WatcherResult<impl IntoResponse> {
    let query = parse_snapshot_query(raw_query.as_deref());
    update_endpoint_probe_targets(
        &state,
        query.endpoint_probe_target,
        query.endpoint_probe_protocol,
    );
    let domain_capture_admin_disabled = enforce_domain_capture_admin_gate(&state.core)
        .context("failed to enforce advanced monitoring permissions")?;
    let monitor_status = state.monitor.status(&state.core).await;
    let integration_dir = query.integration_dir.as_ref().map(PathBuf::from);
    let mut snapshot = state
        .core
        .snapshot(monitor_status, integration_dir.as_deref())
        .context("failed to load snapshot")?;
    snapshot.filters = current_ui_filters(&state);
    snapshot.runtime_status.endpoint_probe = current_endpoint_probe_status(&state);
    snapshot.runtime_status.tool_available = current_tool_available();
    snapshot.runtime_status.domain_capture = state.monitor.domain_capture_status();
    snapshot.runtime_status.flow_capture = state.monitor.flow_capture_status();
    snapshot.runtime_status.domain_capture_admin_disabled = domain_capture_admin_disabled;
    snapshot.runtime_status.is_elevated = current_process_is_elevated();
    Ok(Json(snapshot))
}

async fn set_ui_filters(
    State(state): State<AppState>,
    Json(request): Json<UiFiltersDto>,
) -> WatcherResult<impl IntoResponse> {
    state
        .core
        .set_app_setting(
            SETTING_UI_MONITORING_PUBLIC_IP,
            if request.public_ip { "true" } else { "false" },
        )
        .context("failed to persist public IP monitoring filter")?;
    *state.ui_filters.lock().expect("ui filters lock poisoned") = request.clone();
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        request.clone(),
        "filters.changed",
        serde_json::json!({ "source": "ui" }),
    );
    Ok(Json(request))
}

fn parse_snapshot_query(raw_query: Option<&str>) -> SnapshotQuery {
    let mut query = SnapshotQuery {
        integration_dir: None,
        endpoint_probe_target: Vec::new(),
        endpoint_probe_protocol: Vec::new(),
    };

    let Some(raw_query) = raw_query else {
        return query;
    };

    for pair in raw_query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_query_component(raw_key);
        let value = decode_query_component(raw_value);
        match key.as_str() {
            "integration_dir" => query.integration_dir = Some(value),
            "endpoint_probe_target" => query.endpoint_probe_target.push(value),
            "endpoint_probe_protocol" => query.endpoint_probe_protocol.push(value),
            _ => {}
        }
    }

    query
}

fn decode_query_component(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    decoded.push(byte);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_string())
}

fn icon_file_response(
    icon_path: Option<&Path>,
    allowed_icon_dirs: &[&Path],
) -> axum::response::Response {
    if let Some(path) = icon_path {
        if icon_path_is_inside_allowed_dir(path, allowed_icon_dirs) {
            if let Some(content_type) = icon_content_type(path) {
                if let Ok(bytes) = std::fs::read(path) {
                    return icon_bytes_response(bytes, content_type);
                }
            }
        }
    }

    icon_bytes_response(DEFAULT_APP_ICON_SVG.to_vec(), SVG_CONTENT_TYPE)
}

fn integration_module_icon_allowed_dirs(state: &AppState) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(state.core.paths().integrations_dir.clone());
    if let Some(explicit) = env::var_os("NETSTITCH__INTEGRATIONS_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        dirs.push(explicit);
    }
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        dirs.push(parent.join("integrations"));
    }
    if let Ok(cwd) = env::current_dir() {
        dirs.push(cwd.join("integrations"));
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

fn icon_bytes_response(bytes: Vec<u8>, content_type: &'static str) -> axum::response::Response {
    (
        [
            (CONTENT_TYPE, content_type),
            (
                CACHE_CONTROL,
                "no-store, no-cache, must-revalidate, max-age=0",
            ),
        ],
        bytes,
    )
        .into_response()
}

fn icon_path_is_inside_allowed_dir(icon_path: &Path, allowed_icon_dirs: &[&Path]) -> bool {
    let Ok(icon_path) = icon_path.canonicalize() else {
        return false;
    };
    allowed_icon_dirs.iter().any(|icon_dir| {
        icon_dir
            .canonicalize()
            .is_ok_and(|allowed_dir| icon_path.starts_with(allowed_dir))
    })
}

fn icon_content_type(icon_path: &Path) -> Option<&'static str> {
    match icon_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("svg") => Some(SVG_CONTENT_TYPE),
        Some("png") => Some(PNG_CONTENT_TYPE),
        Some("ico") => Some(ICO_CONTENT_TYPE),
        _ => None,
    }
}

fn load_language_catalog() -> LanguageCatalogDto {
    let mut languages = BTreeMap::<String, LanguageDto>::new();
    for dir in candidate_language_dirs() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("ini") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if let Some(language) = parse_language_ini(&path, &content) {
                languages.insert(language.code.to_ascii_lowercase(), language);
            }
        }
    }

    if languages.is_empty() {
        languages.insert(
            DEFAULT_LANGUAGE_CODE.to_string(),
            LanguageDto {
                code: DEFAULT_LANGUAGE_CODE.to_string(),
                label: "English (EN)".to_string(),
                order: 10,
                strings: fallback_language_strings(),
            },
        );
    }

    let mut languages = languages.into_values().collect::<Vec<_>>();
    languages.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.label.cmp(&right.label))
    });

    LanguageCatalogDto {
        fallback_code: DEFAULT_LANGUAGE_CODE.to_string(),
        languages,
    }
}

fn candidate_language_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(explicit) = env::var(APP_ENV_LANGUAGE_DIR) {
        dirs.push(PathBuf::from(explicit));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("language"));
        }
    }
    if let Ok(current_dir) = env::current_dir() {
        dirs.push(current_dir.join("language"));
        dirs.push(current_dir.join("resources").join("language"));
    }
    dirs
}

fn parse_language_ini(path: &Path, content: &str) -> Option<LanguageDto> {
    let mut section = String::new();
    let mut code = path.file_stem()?.to_string_lossy().to_ascii_lowercase();
    let mut native_name = code.clone();
    let mut order = 100;
    let mut strings = BTreeMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_ascii_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match section.as_str() {
            "language" if key.eq_ignore_ascii_case("code") && !value.is_empty() => {
                code = value.to_ascii_lowercase();
            }
            "language" if key.eq_ignore_ascii_case("name") && !value.is_empty() => {
                native_name = value.to_string();
            }
            "language" if key.eq_ignore_ascii_case("order") => {
                if let Ok(parsed) = value.parse::<i32>() {
                    order = parsed;
                }
            }
            "strings" if !key.is_empty() => {
                strings.insert(key.to_string(), value.to_string());
            }
            _ => {}
        }
    }

    if strings.is_empty() {
        return None;
    }

    Some(LanguageDto {
        label: format!("{} ({})", native_name, display_language_region(&code)),
        code,
        order,
        strings,
    })
}

fn fallback_language_strings() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.title".to_string(), "NetStitch MVP".to_string()),
        (
            "status.build_version".to_string(),
            "Build version".to_string(),
        ),
        (
            "integration.loaded".to_string(),
            "Loaded: {count}".to_string(),
        ),
        ("observations.total".to_string(), "Total rows".to_string()),
        (
            "observations.displayed".to_string(),
            "Displayed rows".to_string(),
        ),
        (
            "observations.selected".to_string(),
            "Selected rows".to_string(),
        ),
        ("footer.language.label".to_string(), "Language".to_string()),
        (
            "header.domain_capture".to_string(),
            "Advanced mon.".to_string(),
        ),
        (
            "header.domain_capture.enabled_tooltip".to_string(),
            "Advanced monitoring is enabled".to_string(),
        ),
        (
            "header.domain_capture.disabled_tooltip".to_string(),
            "Advanced monitoring is disabled".to_string(),
        ),
        ("footer.tool.label".to_string(), "Tool".to_string()),
        (
            "footer.tool.available_tooltip".to_string(),
            "netstitch-tool native library is available for lookup and endpoint checks."
                .to_string(),
        ),
        (
            "footer.tool.unavailable_tooltip".to_string(),
            "netstitch-tool native library is unavailable in the watcher runtime.".to_string(),
        ),
        ("footer.network.label".to_string(), "DNS".to_string()),
        (
            "footer.network.available_prefix".to_string(),
            "First reachable DNS:".to_string(),
        ),
        (
            "footer.network.unavailable".to_string(),
            "All endpoint probes are unavailable".to_string(),
        ),
        (
            "footer.network.checking".to_string(),
            "endpoint probes are checking".to_string(),
        ),
        ("footer.network.true".to_string(), "True".to_string()),
        ("footer.network.false".to_string(), "False".to_string()),
        ("footer.network.unknown".to_string(), "Checking".to_string()),
        (
            "status.waiting_for_watcher".to_string(),
            "Waiting for watcher".to_string(),
        ),
        (
            "footer.event.domain_capture_enabled".to_string(),
            "Advanced monitoring enabled".to_string(),
        ),
        (
            "footer.event.domain_capture_disabled".to_string(),
            "Advanced monitoring disabled".to_string(),
        ),
        (
            "footer.event.domain_capture_admin_required".to_string(),
            "Run NetStitch as administrator to use advanced monitoring".to_string(),
        ),
        (
            "footer.event.domain_capture_failed".to_string(),
            "Advanced monitoring is unavailable".to_string(),
        ),
        (
            "web.host_unavailable.title".to_string(),
            "NetStitch host is unavailable".to_string(),
        ),
        (
            "web.host_unavailable.text".to_string(),
            "The main NetStitch program is not responding, or its embedded web server was switched off."
                .to_string(),
        ),
        (
            "web.host_unavailable.note".to_string(),
            "The browser page will reconnect automatically after the host answers again.".to_string(),
        ),
    ])
}

fn display_language_region(code: &str) -> String {
    code.rsplit_once('-')
        .map(|(_, region)| region)
        .unwrap_or(code)
        .to_ascii_uppercase()
}

async fn add_tracked_app(
    State(state): State<AppState>,
    Json(request): Json<AddTrackedAppRequest>,
) -> WatcherResult<impl IntoResponse> {
    let tracked_app = state
        .core
        .add_tracked_app(request)
        .context("failed to add tracked app")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "tracked_apps.changed",
        serde_json::json!({ "source": "ui", "action": "add", "tracked_app_id": tracked_app.id }),
    );
    Ok((StatusCode::CREATED, Json(tracked_app)))
}

async fn set_tracked_app_enabled(
    State(state): State<AppState>,
    Json(request): Json<SetTrackedAppEnabledRequest>,
) -> WatcherResult<impl IntoResponse> {
    let updated = state
        .core
        .set_tracked_app_enabled(request)
        .context("failed to update tracked app")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "tracked_apps.changed",
        serde_json::json!({ "source": "ui", "action": "set_enabled", "updated": updated }),
    );
    Ok(Json(serde_json::json!({ "updated": updated })))
}

async fn delete_tracked_app(
    State(state): State<AppState>,
    Json(request): Json<DeleteTrackedAppRequest>,
) -> WatcherResult<impl IntoResponse> {
    let deleted = state
        .core
        .delete_tracked_app(request.tracked_app_id)
        .context("failed to delete tracked app")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "tracked_apps.changed",
        serde_json::json!({ "source": "ui", "action": "delete", "deleted": deleted }),
    );
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn set_all_tracked_apps_enabled(
    State(state): State<AppState>,
    Json(request): Json<SetAllTrackedAppsEnabledRequest>,
) -> WatcherResult<impl IntoResponse> {
    let updated = state
        .core
        .set_all_tracked_apps_enabled(request.enabled)
        .context("failed to update tracked apps")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "tracked_apps.changed",
        serde_json::json!({ "source": "ui", "action": "set_all_enabled", "updated": updated }),
    );
    Ok(Json(serde_json::json!({ "updated": updated })))
}

async fn start_monitoring(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let response = state.monitor.start(state.core.clone()).await;
    append_runtime_event(
        &state.core,
        "monitoring",
        "start",
        "info",
        None,
        None,
        serde_json::to_value(&response).unwrap_or_else(|_| serde_json::json!({})),
    );
    state.core.dispatch_integration_module_event(
        MonitorStatus::Running,
        current_ui_filters(&state),
        "monitoring.started",
        serde_json::json!({ "source": "ui" }),
    );
    Ok(Json(response))
}

async fn stop_monitoring(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let response = state.monitor.stop(&state.core).await;
    append_runtime_event(
        &state.core,
        "monitoring",
        "stop",
        "info",
        None,
        None,
        serde_json::to_value(&response).unwrap_or_else(|_| serde_json::json!({})),
    );
    state.core.dispatch_integration_module_event(
        MonitorStatus::Stopped,
        current_ui_filters(&state),
        "monitoring.stopped",
        serde_json::json!({ "source": "ui" }),
    );
    Ok(Json(response))
}

async fn shutdown_watcher(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    let monitor_response = state.monitor.stop(&state.core).await;
    if let Some(sender) = state.shutdown.lock().await.take() {
        let _ = sender.send(());
    }

    Ok(Json(serde_json::json!({
        "shutdown": "scheduled",
        "monitor": monitor_response,
    })))
}

async fn confirm_observations(
    State(state): State<AppState>,
    Json(request): Json<ConfirmEndpointsRequest>,
) -> WatcherResult<impl IntoResponse> {
    let updated = state
        .core
        .confirm_endpoints(request)
        .context("failed to confirm endpoints")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "monitoring.rows_changed",
        serde_json::json!({ "source": "ui", "action": "confirm", "updated": updated }),
    );
    Ok(Json(serde_json::json!({ "updated": updated })))
}

async fn mark_observations_exported(
    State(state): State<AppState>,
    Json(request): Json<MarkEndpointsExportedRequest>,
) -> WatcherResult<impl IntoResponse> {
    let updated = state
        .core
        .mark_endpoints_exported(request)
        .context("failed to mark endpoints exported")?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}

async fn delete_observation(
    State(state): State<AppState>,
    Json(request): Json<DeleteObservationRequest>,
) -> WatcherResult<impl IntoResponse> {
    let deleted = state
        .core
        .delete_observation(request)
        .context("failed to delete observation")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "monitoring.rows_deleted",
        serde_json::json!({ "source": "ui", "deleted": deleted }),
    );
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn delete_observations(
    State(state): State<AppState>,
    Json(request): Json<DeleteObservationsRequest>,
) -> WatcherResult<impl IntoResponse> {
    let deleted = state
        .core
        .delete_observations(request)
        .context("failed to delete observations")?;
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "monitoring.rows_deleted",
        serde_json::json!({ "source": "ui", "deleted": deleted }),
    );
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn import_monitoring_csv(
    State(state): State<AppState>,
    Json(request): Json<MonitoringCsvImportRequestDto>,
) -> WatcherResult<impl IntoResponse> {
    let requested_rows = request.rows.len();
    let import_source = request.import_source.clone();
    let (component, action_type, error_context) = match import_source {
        MonitoringImportSourceDto::Csv => {
            ("csv", "import_response", "failed to import monitoring CSV")
        }
        MonitoringImportSourceDto::CloudDownload => (
            "cloud",
            "add_to_monitoring_response",
            "failed to add cloud rows to monitoring",
        ),
    };
    let result = state
        .core
        .import_monitoring_csv(request)
        .context(error_context)?;
    append_runtime_event(
        &state.core,
        component,
        action_type,
        "info",
        None,
        None,
        serde_json::json!({
            "source": import_source,
            "requested_rows": requested_rows,
            "imported_count": result.imported_count,
            "skipped_count": result.skipped_count,
        }),
    );
    let monitor_status = state.monitor.status(&state.core).await;
    state.core.dispatch_integration_module_event(
        monitor_status,
        current_ui_filters(&state),
        "monitoring.rows_added",
        serde_json::json!({
            "source": import_source,
            "requested_rows": requested_rows,
            "imported_count": result.imported_count,
            "skipped_count": result.skipped_count,
        }),
    );
    Ok(Json(result))
}

async fn set_app_setting(
    State(state): State<AppState>,
    Json(request): Json<SetAppSettingRequest>,
) -> WatcherResult<impl IntoResponse> {
    if request.key == netstitch_shared::models::SETTING_DOMAIN_CAPTURE_ENABLED
        && setting_truthy(&request.value)
        && !current_process_is_elevated()
    {
        return Err(WatcherError::bad_request(
            "Advanced monitoring requires administrator rights. Run NetStitch as administrator.",
        ));
    }
    state
        .core
        .set_app_setting(&request.key, &request.value)
        .context("failed to persist app setting")?;
    let setting_key = request.key.clone();
    let setting_value = request.value.clone();
    let component = if request.key == netstitch_shared::models::SETTING_DOMAIN_CAPTURE_ENABLED {
        "dns"
    } else if request.key == netstitch_shared::models::SETTING_WEB_ACCESS_LOCALHOST {
        "web"
    } else {
        "settings"
    };
    let redacted_setting = request.key.contains("token")
        || request.key.contains("secret")
        || request.key.contains("private_key");
    let logged_setting_value = if redacted_setting {
        "[redacted]".to_string()
    } else {
        setting_value
    };
    append_runtime_event(
        &state.core,
        component,
        "setting_changed",
        "info",
        Some("app_setting"),
        Some(&setting_key),
        serde_json::json!({
            "key": setting_key,
            "value": logged_setting_value,
        }),
    );
    let settings = state
        .core
        .app_settings()
        .context("failed to reload app settings")?;
    Ok(Json(settings))
}

async fn configure_integration(
    State(state): State<AppState>,
    Json(request): Json<ConfigureIntegrationRequest>,
) -> WatcherResult<impl IntoResponse> {
    let status = state
        .core
        .configure_integration_root(request.module_id.as_deref(), request.repo_root.as_deref())
        .context("failed to persist integration root")?;
    Ok(Json(status))
}

async fn integration_ui_action(
    State(state): State<AppState>,
    Json(request): Json<IntegrationModuleUiActionClientRequestDto>,
) -> WatcherResult<impl IntoResponse> {
    let monitor_status = state.monitor.status(&state.core).await;
    let module_id = request.module_id.clone();
    let ui_action_token = request.ui_action_token.trim().to_string();
    let action_id = request.action_id.clone();
    let selected_count = request.selected_monitoring_row_ids.len();
    let core = state.core.clone();
    if !ui_action_token.is_empty() {
        if let Ok(mut events) = state.module_ui_action_events.lock() {
            events.register(&module_id, &ui_action_token);
        }
    }
    let event_module_id = module_id.clone();
    let event_token = ui_action_token.clone();
    let event_store = state.module_ui_action_events.clone();
    let result = tokio::task::spawn_blocking(move || {
        core.run_integration_module_ui_action_with_events(monitor_status, request, move |event| {
            let Some((event_type, payload)) = module_ui_action_event_from_host_event(event) else {
                return;
            };
            if let Ok(mut events) = event_store.lock() {
                events.push(&event_module_id, &event_token, event_type, payload);
            }
        })
    })
    .await
    .map_err(|error| {
        WatcherError::with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!("integration module UI action task failed: {error}"),
        )
    })?
    .context("failed to run integration module UI action")?;
    execute_integration_host_commands(&state, &module_id, &result.commands).await?;
    append_runtime_event(
        &state.core,
        "integration",
        "ui_action",
        result.severity.as_str(),
        Some("module"),
        Some(&module_id),
        serde_json::json!({
            "module_id": module_id,
            "action_id": action_id,
            "selected_monitoring_rows": selected_count,
            "message": result.message.clone(),
            "refresh": result.refresh,
            "commands": result.commands.len(),
        }),
    );
    Ok(Json(result))
}

async fn integration_ui_action_events(
    State(state): State<AppState>,
    Query(query): Query<IntegrationModuleUiActionEventsQuery>,
) -> WatcherResult<impl IntoResponse> {
    let module_id = query.module_id.trim();
    let ui_action_token = query.ui_action_token.trim();
    if module_id.is_empty() || ui_action_token.is_empty() {
        return Err(WatcherError::bad_request(
            "module_id and ui_action_token are required",
        ));
    }
    let events = state
        .module_ui_action_events
        .lock()
        .map(|mut store| store.events_after(module_id, ui_action_token, query.after.unwrap_or(0)))
        .unwrap_or_default();
    Ok(Json(IntegrationModuleUiActionEventsResponseDto { events }))
}

async fn stop_integration_module_background(
    State(state): State<AppState>,
    Json(request): Json<StopIntegrationModuleBackgroundRequest>,
) -> WatcherResult<impl IntoResponse> {
    let module_id = request.module_id.trim().to_string();
    if module_id.is_empty() {
        return Err(WatcherError::bad_request("module_id is required"));
    }
    let stopped = state
        .core
        .stop_integration_module_background(&module_id)
        .context("failed to stop integration module background task")?;
    append_runtime_event(
        &state.core,
        "integration",
        "background_stop",
        "info",
        Some("module"),
        Some(&module_id),
        serde_json::json!({
            "module_id": module_id,
            "stopped": stopped,
            "source": "module_header",
        }),
    );
    Ok(Json(serde_json::json!({ "stopped": stopped })))
}

async fn integration_dialog_result(
    State(state): State<AppState>,
    Json(request): Json<IntegrationDialogResultRequest>,
) -> WatcherResult<impl IntoResponse> {
    let module_id = request.module_id.trim().to_string();
    if module_id.is_empty() {
        return Err(WatcherError::bad_request("module_id is required"));
    }
    let dialog_id = request.dialog_id.trim().to_string();
    if dialog_id.is_empty() {
        return Err(WatcherError::bad_request("dialog_id is required"));
    }
    let result = match request.result.trim().to_ascii_lowercase().as_str() {
        "ok" => "ok",
        "cancel" => "cancel",
        _ => {
            return Err(WatcherError::bad_request(
                "dialog result must be ok or cancel",
            ));
        }
    }
    .to_string();
    let module_source = integration_module_source_name(&state.core, &module_id);
    let core = state.core.clone();
    let module_id_for_call = module_id.clone();
    let dialog_id_for_call = dialog_id.clone();
    let result_for_call = result.clone();
    let response = tokio::task::spawn_blocking(move || {
        core.run_integration_module_dialog_result(
            &module_id_for_call,
            &dialog_id_for_call,
            &result_for_call,
        )
    })
    .await
    .map_err(|error| {
        WatcherError::with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!("dialog result task failed: {error}"),
        )
    })?
    .context("failed to deliver integration module dialog result")?;
    execute_integration_host_commands(&state, &module_id, &response.commands).await?;

    append_runtime_event_with_source(
        &state.core,
        &module_source,
        "integration",
        "module_dialog_result",
        "info",
        Some("module_dialog"),
        Some(&dialog_id),
        serde_json::json!({
            "module_id": module_id,
            "dialog_id": dialog_id,
            "result": result,
        }),
    );
    Ok(Json(response))
}

async fn execute_integration_host_commands(
    state: &AppState,
    module_id: &str,
    commands: &[netstitch_shared::models::IntegrationModuleHostCommandDto],
) -> WatcherResult<()> {
    for command in commands {
        match command.command_type.as_str() {
            "set_filters" => {
                let filters = serde_json::from_value::<UiFiltersDto>(command.payload.clone())
                    .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state
                    .core
                    .set_app_setting(
                        SETTING_UI_MONITORING_PUBLIC_IP,
                        if filters.public_ip { "true" } else { "false" },
                    )
                    .context("failed to persist module-requested public IP filter")?;
                *state.ui_filters.lock().expect("ui filters lock poisoned") = filters;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "filters.changed",
                    serde_json::json!({ "source": "module_command" }),
                );
            }
            "start_monitoring" => {
                let _ = state.monitor.start(state.core.clone()).await;
                state.core.dispatch_integration_module_event(
                    MonitorStatus::Running,
                    current_ui_filters(state),
                    "monitoring.started",
                    serde_json::json!({ "source": "module_command" }),
                );
            }
            "stop_monitoring" => {
                let _ = state.monitor.stop(&state.core).await;
                state.core.dispatch_integration_module_event(
                    MonitorStatus::Stopped,
                    current_ui_filters(state),
                    "monitoring.stopped",
                    serde_json::json!({ "source": "module_command" }),
                );
            }
            "add_tracked_app" => {
                let request =
                    serde_json::from_value::<AddTrackedAppRequest>(command.payload.clone())
                        .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.add_tracked_app(request)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "tracked_apps.changed",
                    serde_json::json!({ "source": "module_command", "action": "add" }),
                );
            }
            "set_tracked_app_enabled" => {
                let request =
                    serde_json::from_value::<SetTrackedAppEnabledRequest>(command.payload.clone())
                        .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.set_tracked_app_enabled(request)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "tracked_apps.changed",
                    serde_json::json!({ "source": "module_command", "action": "set_enabled" }),
                );
            }
            "set_all_tracked_apps_enabled" => {
                let request = serde_json::from_value::<SetAllTrackedAppsEnabledRequest>(
                    command.payload.clone(),
                )
                .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.set_all_tracked_apps_enabled(request.enabled)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "tracked_apps.changed",
                    serde_json::json!({ "source": "module_command", "action": "set_all_enabled" }),
                );
            }
            "delete_tracked_app" => {
                let request =
                    serde_json::from_value::<DeleteTrackedAppRequest>(command.payload.clone())
                        .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.delete_tracked_app(request.tracked_app_id)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "tracked_apps.changed",
                    serde_json::json!({ "source": "module_command", "action": "delete" }),
                );
            }
            "confirm_monitoring_rows" => {
                let request =
                    serde_json::from_value::<ConfirmEndpointsRequest>(command.payload.clone())
                        .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.confirm_endpoints(request)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "monitoring.rows_changed",
                    serde_json::json!({ "source": "module_command", "action": "confirm" }),
                );
            }
            "delete_monitoring_rows" => {
                let request =
                    serde_json::from_value::<DeleteObservationsRequest>(command.payload.clone())
                        .map_err(|error| WatcherError::bad_request(error.to_string()))?;
                state.core.delete_observations(request)?;
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "monitoring.rows_deleted",
                    serde_json::json!({ "source": "module_command" }),
                );
            }
            "select_monitoring_rows" => {
                let monitor_status = state.monitor.status(&state.core).await;
                state.core.dispatch_integration_module_event(
                    monitor_status,
                    current_ui_filters(state),
                    "monitoring.selection_changed",
                    command.payload.clone(),
                );
            }
            "start_background" => {
                let subscriptions = background_subscriptions_from_payload(&command.payload)?;
                state
                    .core
                    .start_integration_module_background(module_id, subscriptions)
                    .context("failed to start integration module background task")?;
            }
            "stop_background" => {
                state
                    .core
                    .stop_integration_module_background(module_id)
                    .context("failed to stop integration module background task")?;
            }
            "set_module_page" => {
                let page = command
                    .payload
                    .get("page")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        WatcherError::bad_request("set_module_page payload must contain page")
                    })?;
                if page.len() > 80 {
                    return Err(WatcherError::bad_request(
                        "set_module_page page is too long",
                    ));
                }
            }
            "set_ui_values" => {
                validate_module_ui_values_payload(&command.payload, "set_ui_values")?;
            }
            "browse_window" => {
                validate_module_browse_window_payload(&command.payload)?;
            }
            "show_dialog" => {
                validate_module_dialog_payload(&command.payload)?;
            }
            "log_event" => {
                let event = module_log_event_from_payload(module_id, &command.payload)?;
                let module_source = integration_module_source_name(&state.core, module_id);
                append_runtime_event_with_source(
                    &state.core,
                    &module_source,
                    "integration",
                    "module_message",
                    &event.severity,
                    Some("module"),
                    Some(module_id),
                    serde_json::json!({
                        "module_id": module_id,
                        "message": event.message,
                    }),
                );
            }
            unsupported => {
                return Err(WatcherError::bad_request(format!(
                    "unsupported integration host command: {unsupported}"
                )));
            }
        }
    }
    Ok(())
}

fn module_ui_action_event_from_host_event(
    event: IntegrationHostEvent,
) -> Option<(&'static str, serde_json::Value)> {
    if event.event != "ui_values" {
        return None;
    }
    module_ui_values_event_payload(&event.payload)
        .ok()
        .map(|payload| ("ui_values", payload))
}

fn module_ui_values_event_payload(payload: &serde_json::Value) -> WatcherResult<serde_json::Value> {
    let values = validate_module_ui_values_payload(payload, "ui_values event")?;
    Ok(serde_json::json!({ "values": values.clone() }))
}

fn validate_module_ui_values_payload<'a>(
    payload: &'a serde_json::Value,
    context: &str,
) -> WatcherResult<&'a serde_json::Map<String, serde_json::Value>> {
    let values = payload
        .get("values")
        .unwrap_or(payload)
        .as_object()
        .ok_or_else(|| WatcherError::bad_request(format!("{context} payload must be an object")))?;
    if values.len() > 64 {
        return Err(WatcherError::bad_request(format!(
            "{context} payload contains too many values"
        )));
    }
    for key in values.keys() {
        if key.trim().is_empty() || key.len() > 120 {
            return Err(WatcherError::bad_request(format!(
                "{context} keys must be non-empty and at most 120 characters"
            )));
        }
    }
    Ok(values)
}

fn validate_module_dialog_payload(payload: &serde_json::Value) -> WatcherResult<()> {
    let object = payload.as_object().ok_or_else(|| {
        WatcherError::bad_request("show_dialog payload must be an object with message and buttons")
    })?;
    object
        .get("message")
        .or_else(|| object.get("text"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| WatcherError::bad_request("show_dialog message is required"))?;
    let buttons = object
        .get("buttons")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("ok");
    match buttons {
        "ok" | "ok_cancel" => Ok(()),
        _ => Err(WatcherError::bad_request(
            "show_dialog buttons must be ok or ok_cancel",
        )),
    }
}

fn validate_module_browse_window_payload(payload: &serde_json::Value) -> WatcherResult<()> {
    let object = payload
        .as_object()
        .ok_or_else(|| WatcherError::bad_request("browse_window payload must be an object"))?;
    let target = object
        .get("target")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| WatcherError::bad_request("browse_window target is required"))?;
    if target.len() > 120 {
        return Err(WatcherError::bad_request(
            "browse_window target must be at most 120 characters",
        ));
    }

    let mode = object
        .get("mode")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("file_open");
    if !matches!(mode, "folder" | "file_open" | "file_save") {
        return Err(WatcherError::bad_request(
            "browse_window mode must be folder, file_open or file_save",
        ));
    }

    for field in [
        ("title", 200usize),
        ("start_dir", 4096usize),
        ("default_name", 255usize),
        ("default_extension", 32usize),
        ("confirm_label", 80usize),
        ("status_target", 120usize),
        ("selected_status", 240usize),
    ] {
        validate_optional_module_string_field(object, field.0, field.1, "browse_window")?;
    }

    let overwrite_policy = object
        .get("overwrite_policy")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("prompt");
    if !matches!(overwrite_policy, "prompt" | "allow" | "deny") {
        return Err(WatcherError::bad_request(
            "browse_window overwrite_policy must be prompt, allow or deny",
        ));
    }
    if object
        .get("can_create_directories")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err(WatcherError::bad_request(
            "browse_window can_create_directories must be boolean",
        ));
    }
    validate_module_browse_window_filters(object)?;
    Ok(())
}

fn validate_optional_module_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    max_len: usize,
    command: &str,
) -> WatcherResult<()> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    let Some(text) = value.as_str() else {
        return Err(WatcherError::bad_request(format!(
            "{command} {field} must be a string"
        )));
    };
    if text.trim().len() > max_len {
        return Err(WatcherError::bad_request(format!(
            "{command} {field} must be at most {max_len} characters"
        )));
    }
    Ok(())
}

fn validate_module_browse_window_filters(
    object: &serde_json::Map<String, serde_json::Value>,
) -> WatcherResult<()> {
    let Some(filters) = object.get("filters") else {
        return Ok(());
    };
    let filters = filters
        .as_array()
        .ok_or_else(|| WatcherError::bad_request("browse_window filters must be an array"))?;
    if filters.len() > 16 {
        return Err(WatcherError::bad_request(
            "browse_window filters must contain at most 16 items",
        ));
    }
    for filter in filters {
        let filter = filter
            .as_object()
            .ok_or_else(|| WatcherError::bad_request("browse_window filter must be an object"))?;
        validate_optional_module_string_field(filter, "name", 80, "browse_window filter")?;
        let extensions = filter
            .get("extensions")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                WatcherError::bad_request("browse_window filter extensions must be an array")
            })?;
        if extensions.is_empty() || extensions.len() > 32 {
            return Err(WatcherError::bad_request(
                "browse_window filter extensions must contain 1 to 32 items",
            ));
        }
        for extension in extensions {
            validate_module_file_extension(extension.as_str().ok_or_else(|| {
                WatcherError::bad_request("browse_window filter extension must be a string")
            })?)?;
        }
    }
    if let Some(default_extension) = object.get("default_extension") {
        validate_module_file_extension(default_extension.as_str().unwrap_or_default())?;
    }
    Ok(())
}

fn validate_module_file_extension(extension: &str) -> WatcherResult<()> {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() || extension.len() > 32 {
        return Err(WatcherError::bad_request(
            "browse_window extensions must be non-empty and at most 32 characters",
        ));
    }
    if !extension
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(WatcherError::bad_request(
            "browse_window extensions may contain only ASCII letters, digits, underscore or dash",
        ));
    }
    Ok(())
}

async fn set_profile_export_ui_state(
    State(state): State<AppState>,
    Json(request): Json<ProfileExportUiStateDto>,
) -> WatcherResult<impl IntoResponse> {
    state.core.set_profile_export_ui_state(request)?;
    Ok(Json(serde_json::json!({ "saved": true })))
}

async fn browse_filesystem(
    Query(query): Query<BrowseFilesystemQuery>,
) -> WatcherResult<impl IntoResponse> {
    let mode = query.mode.unwrap_or_default();
    let extensions = normalized_file_picker_extensions(query.extensions.as_deref());
    let current_path = resolve_picker_directory(query.path.as_deref())?;
    let roots = filesystem_roots()
        .into_iter()
        .map(|path| path_to_display_string(&path))
        .collect::<Vec<_>>();
    let mut entries = Vec::new();

    for entry in fs::read_dir(&current_path).with_context(|| {
        format!(
            "failed to read directory {}",
            path_to_display_string(&current_path)
        )
    })? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let is_dir = metadata.is_dir();
        let is_file = metadata.is_file();
        let name = entry.file_name().to_string_lossy().to_string();

        entries.push(FilesystemEntryDto {
            name,
            path: path_to_display_string(&path),
            is_dir,
            is_file,
            selectable: picker_path_is_selectable(&path, is_file, mode, &extensions),
        });
    }

    entries.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(Json(FilesystemBrowseDto {
        current_path: path_to_display_string(&current_path),
        parent_path: current_path
            .parent()
            .map(path_to_display_string)
            .filter(|parent| parent != &path_to_display_string(&current_path)),
        roots,
        entries,
    }))
}

async fn read_text_file(
    State(state): State<AppState>,
    Json(request): Json<ReadTextFileRequest>,
) -> WatcherResult<impl IntoResponse> {
    const MAX_TEXT_FILE_BYTES: u64 = 50 * 1024 * 1024;

    let path = canonical_existing_path(&request.path)?;
    let metadata = fs::metadata(&path)
        .with_context(|| format!("failed to inspect {}", path_to_display_string(&path)))?;
    if !metadata.is_file() {
        return Err(WatcherError::bad_request("selected path is not a file"));
    }
    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(WatcherError::bad_request(format!(
            "selected file is too large: {} bytes",
            metadata.len()
        )));
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path_to_display_string(&path)))?;
    append_runtime_event(
        &state.core,
        "csv",
        "read_file",
        "info",
        Some("file"),
        Some(&path_to_display_string(&path)),
        serde_json::json!({
            "path": path_to_display_string(&path),
            "bytes": metadata.len(),
        }),
    );
    Ok(Json(ReadTextFileResponse {
        path: path_to_display_string(&path),
        content,
    }))
}

async fn write_text_file(
    State(state): State<AppState>,
    Json(request): Json<WriteTextFileRequest>,
) -> WatcherResult<impl IntoResponse> {
    if request.path.as_os_str().is_empty() {
        return Err(WatcherError::bad_request("file path is empty"));
    }
    if request.path.is_dir() {
        return Err(WatcherError::bad_request("selected path is a directory"));
    }
    let parent = request
        .path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| WatcherError::bad_request("file parent directory is empty"))?;
    let parent = canonical_existing_path(parent)?;
    if !parent.is_dir() {
        return Err(WatcherError::bad_request(
            "file parent path is not a directory",
        ));
    }
    let file_name = request
        .path
        .file_name()
        .ok_or_else(|| WatcherError::bad_request("file name is empty"))?;
    let path = parent.join(file_name);
    if path.is_dir() {
        return Err(WatcherError::bad_request("selected path is a directory"));
    }

    fs::write(&path, request.content.as_bytes())
        .with_context(|| format!("failed to write {}", path_to_display_string(&path)))?;
    append_runtime_event(
        &state.core,
        "csv",
        "write_file",
        "info",
        Some("file"),
        Some(&path_to_display_string(&path)),
        serde_json::json!({
            "path": path_to_display_string(&path),
            "bytes": request.content.len(),
        }),
    );
    Ok(Json(WriteTextFileResponse {
        path: path_to_display_string(&path),
        bytes_written: request.content.len(),
    }))
}

fn resolve_picker_directory(path: Option<&Path>) -> WatcherResult<PathBuf> {
    let Some(path) = path.filter(|path| !path.as_os_str().is_empty()) else {
        return default_picker_directory();
    };

    if path.exists() {
        let metadata = fs::metadata(path)
            .with_context(|| format!("failed to inspect {}", path_to_display_string(path)))?;
        let directory = if metadata.is_dir() {
            path
        } else {
            path.parent()
                .ok_or_else(|| WatcherError::bad_request("file parent directory is empty"))?
        };
        return canonical_existing_path(directory);
    }

    if let Some(parent) = path.parent().filter(|parent| parent.exists()) {
        return canonical_existing_path(parent);
    }

    Err(WatcherError::bad_request(format!(
        "path does not exist: {}",
        path_to_display_string(path)
    )))
}

fn default_picker_directory() -> WatcherResult<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            if parent.exists() {
                return canonical_existing_path(parent);
            }
        }
    }
    canonical_existing_path(&env::current_dir().context("failed to resolve current directory")?)
}

fn canonical_existing_path(path: &Path) -> WatcherResult<PathBuf> {
    path.canonicalize().map_err(|error| {
        WatcherError::bad_request(format!(
            "failed to resolve {}: {error}",
            path_to_display_string(path)
        ))
    })
}

fn filesystem_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            push_unique_existing_root(&mut roots, parent);
        }
    }
    if let Ok(current_dir) = env::current_dir() {
        push_unique_existing_root(&mut roots, &current_dir);
    }

    #[cfg(windows)]
    {
        for letter in b'A'..=b'Z' {
            let root = PathBuf::from(format!("{}:\\", letter as char));
            push_unique_existing_root(&mut roots, &root);
        }
    }

    #[cfg(not(windows))]
    {
        push_unique_existing_root(&mut roots, Path::new("/"));
    }

    roots
}

fn push_unique_existing_root(roots: &mut Vec<PathBuf>, path: &Path) {
    if !path.exists() {
        return;
    }
    let candidate = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !roots.iter().any(|root| root == &candidate) {
        roots.push(candidate);
    }
}

fn picker_path_is_selectable(
    path: &Path,
    is_file: bool,
    mode: FilePickerMode,
    extensions: &[String],
) -> bool {
    if !is_file {
        return false;
    }

    if !extensions.is_empty() {
        return path_extension_is_one_of_owned(path, extensions);
    }

    match mode {
        FilePickerMode::Any | FilePickerMode::FileOpen | FilePickerMode::FileSave => true,
        FilePickerMode::Folder => false,
        FilePickerMode::Executable => {
            #[cfg(windows)]
            {
                path_extension_is_one_of(path, &["exe", "lnk"])
            }
            #[cfg(not(windows))]
            {
                true
            }
        }
        FilePickerMode::Profile => path_extension_is_one_of(path, &["bat", "cmd", "conf", "txt"]),
        FilePickerMode::CsvOpen | FilePickerMode::CsvSave => {
            path_extension_is_one_of(path, &["csv"])
        }
    }
}

fn normalized_file_picker_extensions(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split([',', ';', ' ', '\n', '\r', '\t'])
        .map(|value| value.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 32
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        })
        .take(64)
        .fold(Vec::<String>::new(), |mut acc, value| {
            if !acc.iter().any(|existing| existing == &value) {
                acc.push(value);
            }
            acc
        })
}

fn path_extension_is_one_of_owned(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extensions
                .iter()
                .any(|expected| extension.eq_ignore_ascii_case(expected))
        })
        .unwrap_or(false)
}

fn path_extension_is_one_of(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extensions
                .iter()
                .any(|expected| extension.eq_ignore_ascii_case(expected))
        })
        .unwrap_or(false)
}

fn path_to_display_string(path: &Path) -> String {
    path.display().to_string()
}

async fn add_ignored_address(
    State(state): State<AppState>,
    Json(request): Json<AddIgnoredAddressRequest>,
) -> WatcherResult<impl IntoResponse> {
    let rule = state
        .core
        .add_ignored_address(request)
        .context("failed to add ignored address")?;
    Ok((StatusCode::CREATED, Json(rule)))
}

async fn delete_ignored_address(
    State(state): State<AppState>,
    Json(request): Json<DeleteIgnoredAddressRequest>,
) -> WatcherResult<impl IntoResponse> {
    let deleted = state
        .core
        .delete_ignored_address(request)
        .context("failed to delete ignored address")?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn export_confirmed(
    State(state): State<AppState>,
    Json(request): Json<ExportConfirmedRequest>,
) -> WatcherResult<impl IntoResponse> {
    let result = state
        .core
        .export_confirmed(request)
        .context("failed to export confirmed endpoints")?;
    Ok(Json(result))
}

async fn preview_profile_export(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .preview_profile_export(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn analyze_profile_export(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .analyze_profile_export(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn apply_profile_export(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .apply_profile_export(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn analyze_profile_export_advanced_settings(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileAdvancedSettingsRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .analyze_profile_export_advanced_settings(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn apply_profile_export_advanced_settings(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileAdvancedSettingsRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .apply_profile_export_advanced_settings(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn backup_profile_export(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .backup_profile_export(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn revert_profile_export(
    State(state): State<AppState>,
    Json(request): Json<ScopedProfileExportRequest<ExportProfileRequestDto>>,
) -> WatcherResult<impl IntoResponse> {
    let (module_id, request) = request.into_parts();
    let result = state
        .core
        .revert_profile_export(module_id.as_deref(), request)?;
    Ok(Json(result))
}

async fn integration_providers(State(state): State<AppState>) -> WatcherResult<impl IntoResponse> {
    Ok(Json(state.core.integration_providers()?))
}

async fn integration_download_progress(
    State(state): State<AppState>,
) -> WatcherResult<impl IntoResponse> {
    Ok(Json(current_integration_download_progress(&state)))
}

async fn start_integration_download(
    State(state): State<AppState>,
    Json(request): Json<DownloadIntegrationProviderRequest>,
) -> WatcherResult<impl IntoResponse> {
    let module_id = request.module_id.clone();
    let provider_id = request.provider_id.clone();
    set_integration_download_progress(
        &state,
        IntegrationDownloadProgressDto::stage(provider_id.clone(), "preparing"),
    );

    let job_state = state.clone();
    tokio::task::spawn_blocking(move || {
        let progress_store = job_state.integration_download_progress.clone();
        let result = job_state.core.download_integration_provider_with_progress(
            module_id.as_deref(),
            &provider_id,
            move |progress| {
                if let Ok(mut current) = progress_store.lock() {
                    *current = progress;
                }
            },
        );

        if let Err(error) = result {
            set_integration_download_progress(
                &job_state,
                IntegrationDownloadProgressDto {
                    active: false,
                    provider_id: Some(provider_id),
                    stage: "failed".to_string(),
                    percent: None,
                    downloaded_bytes: None,
                    total_bytes: None,
                    extracted_entries: None,
                    total_entries: None,
                    message: Some(error.to_string()),
                    repo_root: None,
                },
            );
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(current_integration_download_progress(&state)),
    ))
}

async fn download_integration(
    State(state): State<AppState>,
    Json(request): Json<DownloadIntegrationProviderRequest>,
) -> WatcherResult<impl IntoResponse> {
    let module_id = request.module_id.clone();
    let provider_id = request.provider_id.clone();
    set_integration_download_progress(
        &state,
        IntegrationDownloadProgressDto::stage(provider_id.clone(), "preparing"),
    );

    let job_state = state.clone();
    let provider_id_for_job = provider_id.clone();
    let download_result = tokio::task::spawn_blocking(move || {
        let progress_store = job_state.integration_download_progress.clone();
        job_state.core.download_integration_provider_with_progress(
            module_id.as_deref(),
            &provider_id_for_job,
            move |progress| {
                if let Ok(mut current) = progress_store.lock() {
                    *current = progress;
                }
            },
        )
    })
    .await
    .context("integration download worker failed")?;

    let status = match download_result {
        Ok(status) => status,
        Err(error) => {
            set_integration_download_progress(
                &state,
                IntegrationDownloadProgressDto {
                    active: false,
                    provider_id: Some(provider_id),
                    stage: "failed".to_string(),
                    percent: None,
                    downloaded_bytes: None,
                    total_bytes: None,
                    extracted_entries: None,
                    total_entries: None,
                    message: Some(error.to_string()),
                    repo_root: None,
                },
            );
            return Err(error
                .context("failed to download integration provider")
                .into());
        }
    };
    Ok(Json(status))
}

fn set_integration_download_progress(state: &AppState, progress: IntegrationDownloadProgressDto) {
    if let Ok(mut current) = state.integration_download_progress.lock() {
        *current = progress;
    }
}

fn current_integration_download_progress(state: &AppState) -> IntegrationDownloadProgressDto {
    state
        .integration_download_progress
        .lock()
        .map(|current| current.clone())
        .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
}

fn web_scheme_is_https() -> bool {
    !env::var(APP_ENV_WEB_SCHEME)
        .ok()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("http"))
}

fn web_scheme() -> &'static str {
    if web_scheme_is_https() {
        "https"
    } else {
        "http"
    }
}

async fn web_tls_config(state: &AppState) -> Result<RustlsConfig> {
    let cert_path = state.core.paths().data_dir.join(WEB_TLS_CERT_FILE);
    let key_path = state.core.paths().data_dir.join(WEB_TLS_KEY_FILE);
    ensure_web_tls_files(state, &cert_path, &key_path)?;
    RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .with_context(|| {
            format!(
                "failed to load web TLS files {} and {}",
                cert_path.display(),
                key_path.display()
            )
        })
}

fn ensure_web_tls_files(state: &AppState, cert_path: &Path, key_path: &Path) -> Result<()> {
    if cert_path.is_file() && key_path.is_file() {
        return Ok(());
    }
    if let Some(parent) = cert_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let signing_key = KeyPair::generate_for(&PKCS_RSA_SHA256)
        .context("failed to generate web RSA private key")?;
    let mut params = CertificateParams::new(web_certificate_subject_names(state))
        .context("failed to prepare web self-signed certificate parameters")?;
    params.not_before = date_time_ymd(2024, 1, 1);
    params.not_after = date_time_ymd(2036, 1, 1);
    params
        .distinguished_name
        .push(DnType::CommonName, "NetStitch local web");
    let cert = params
        .self_signed(&signing_key)
        .context("failed to generate web self-signed certificate")?;
    fs::write(cert_path, cert.pem())
        .with_context(|| format!("failed to write {}", cert_path.display()))?;
    fs::write(key_path, signing_key.serialize_pem())
        .with_context(|| format!("failed to write {}", key_path.display()))?;
    restrict_private_key_file(key_path)?;
    Ok(())
}

fn web_certificate_subject_names(state: &AppState) -> Vec<String> {
    let mut names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    if let Some(local_ip) = local_machine_ip_for_web_url() {
        names.push(local_ip);
    }
    match state.bind_addr.ip() {
        IpAddr::V4(ip) if !ip.is_unspecified() => names.push(ip.to_string()),
        IpAddr::V6(ip) if !ip.is_unspecified() => names.push(ip.to_string()),
        _ => {}
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(unix)]
fn restrict_private_key_file(key_path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(key_path)
        .with_context(|| format!("failed to read {}", key_path.display()))?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(key_path, permissions)
        .with_context(|| format!("failed to restrict {}", key_path.display()))
}

#[cfg(not(unix))]
fn restrict_private_key_file(_key_path: &Path) -> Result<()> {
    Ok(())
}

fn watcher_addr(override_addr: Option<&str>) -> Result<SocketAddr> {
    let raw = override_addr
        .map(ToOwned::to_owned)
        .or_else(|| env::var(APP_ENV_WATCHER_ADDR).ok())
        .unwrap_or_else(default_watcher_addr);
    let raw_addr = raw
        .trim()
        .strip_prefix("https://")
        .or_else(|| raw.trim().strip_prefix("http://"))
        .unwrap_or(raw.trim())
        .split('/')
        .next()
        .unwrap_or(raw.trim());
    SocketAddr::from_str(raw_addr).with_context(|| format!("invalid watcher socket address: {raw}"))
}

fn default_watcher_addr() -> String {
    format!("0.0.0.0:{DEFAULT_WATCHER_PORT}")
}

struct WatcherError {
    status: StatusCode,
    error: anyhow::Error,
}

type WatcherResult<T> = Result<T, WatcherError>;

impl WatcherError {
    fn with_status(status: StatusCode, error: impl Into<anyhow::Error>) -> Self {
        Self {
            status,
            error: error.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::with_status(StatusCode::BAD_REQUEST, anyhow::anyhow!(message.into()))
    }
}

impl<E> From<E> for WatcherError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self::with_status(StatusCode::INTERNAL_SERVER_ERROR, value)
    }
}

impl IntoResponse for WatcherError {
    fn into_response(self) -> axum::response::Response {
        error!("{:#}", self.error);
        (
            self.status,
            Json(serde_json::json!({
                "error": self.error.to_string(),
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        APP_ENV_ENDPOINT_PROBE_TARGETS, APP_ENV_ENDPOINT_PROBE_TARGETS_FILE, BROWSER_UI_HTML,
        CloudObservationDownloadPage, CloudObservationDownloadPageRow, CloudObservationsQuery,
        SnapshotQuery, canonical_endpoint_probe_target, cloud_observations_rows,
        current_endpoint_probe_targets, parse_endpoint_probe_targets, parse_snapshot_query,
    };
    use netstitch_cloud::CloudObservationRow;
    use netstitch_shared::{
        CloudObservationVisibility, CloudObservationVisibilityScope, CloudSourceKind,
        CloudTrustLevel, IntegrationHostEvent,
    };
    use std::collections::BTreeMap;
    use std::net::IpAddr;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    const DESKTOP_APP_RS: &str = include_str!("../../netstitch-ui/src/app.rs");
    const DESKTOP_THEME_RS: &str = include_str!("../../netstitch-ui/src/theme.rs");
    const DESKTOP_UI_ENTITIES_RS: &str = include_str!("../../netstitch-ui/src/ui_entities.rs");
    const WATCHER_MAIN_RS: &str = include_str!("lib.rs");

    #[derive(Clone, Copy)]
    struct MainWindowElementMapEntry {
        region: &'static str,
        name: &'static str,
        desktop_tokens: &'static [&'static str],
        browser_tokens: &'static [&'static str],
    }

    fn desktop_main_window_sources() -> [&'static str; 3] {
        [DESKTOP_APP_RS, DESKTOP_THEME_RS, DESKTOP_UI_ENTITIES_RS]
    }

    fn sources_contain_any(sources: &[&str], token: &str) -> bool {
        sources.iter().any(|source| source.contains(token))
    }

    fn assert_main_window_element_map(entries: &[MainWindowElementMapEntry]) {
        let desktop_sources = desktop_main_window_sources();
        let mut failures = Vec::new();

        for entry in entries {
            for token in entry.desktop_tokens {
                if !sources_contain_any(&desktop_sources, token) {
                    failures.push(format!(
                        "[desktop][{}][{}] missing token: {}",
                        entry.region, entry.name, token
                    ));
                }
            }
            for token in entry.browser_tokens {
                if !BROWSER_UI_HTML.contains(token) {
                    failures.push(format!(
                        "[browser][{}][{}] missing token: {}",
                        entry.region, entry.name, token
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "main window element map drifted between desktop and browser:\n{}",
            failures.join("\n")
        );
    }

    fn token_index(haystack: &str, needle: &str) -> usize {
        haystack
            .find(needle)
            .unwrap_or_else(|| panic!("missing token in ordered layout assertion: {needle}"))
    }

    #[test]
    fn browser_ui_duplicates_main_desktop_panels() {
        for expected in [
            "Tracked apps",
            "Monitoring",
            "Integration",
            "header-monitor-controls",
            "header-switch-stack",
            "header-label",
            "tracked-apps-help",
            "tracked-enabled-count",
            "tracked_apps.enabled_status",
            "clear-exe-path-button",
            "clear-integration-folder-button",
            "input.clear",
            "ignored-addresses-panel-title",
            "ignored-addresses-panel-help",
            "observations-help",
            "integration-help",
            "renderHelp('tracked-apps-help', 'tracked_apps.help'",
            "renderHelp('ignored-addresses-panel-help', 'ignored_addresses.help'",
            "renderHelp('observations-help', 'observations.subtitle'",
            "renderHelp('integration-help', 'integration.subtitle'",
            "modal__footer",
            "path-field",
            "class=\"path-field ignored-addresses__address\"",
            "ignored-addresses-section",
            "ignored-addresses__empty",
            "ignored-addresses__skeleton",
            "ignored-delete-modal",
            "requestDeleteIgnoredAddress",
            "renderIgnoredDeleteModal",
            "renderDeleteConfirmModal",
            "confirmDeleteIgnoredAddress",
            "cancel-delete-ignored-address",
            "confirm-delete-ignored-address",
            "dialog.delete_ignored.localhost_note",
            "dialog.delete_ignored.local_ip_note",
            "panel-header-meta",
            "tracked-apps-header-meta",
            "integration-status-text",
            "integration-module-grid",
            "integration-module-actions",
            "integration-module-modal",
            "integration-module-modal-title",
            "integration-module-modal-body",
            "integration-module-modal-close-button",
            "renderIntegrationModulePanel",
            "latestSourceRows.length ? moduleMonitoringRowLabel(latestSourceRows[0]) : '-'",
            "function snapshotMonitoringRows(snapshot)",
            "return Array.isArray(snapshot?.observed_endpoints) ? snapshot.observed_endpoints : [];",
            "moduleUiTableHtml(entity, value, id, moduleUiScrollClass(entity.scroll), moduleUiTableViewportStyleAttr(entity))",
            "function moduleUiTableColumn(entity, index)",
            "function moduleUiTableColumnTextField(entity, index)",
            "module-ui-schema__table-cell-field",
            "ui-entity-table",
            "openIntegrationModulePanel",
            "closeIntegrationModulePanel",
            "open-integration-module",
            "openIntegrationDialog",
            "useIntegrationFolder",
            "downloadIntegration",
            "cancelIntegrationDownload",
            "/v1/integrations/download/start",
            "/v1/integrations/download-progress",
            "integration-download-actions",
            "download-integration-provider",
            "integration-modal-status",
            "dialog.integration.path_help",
            "dialog.integration.status_configured",
            "dialog.integration.download_progress",
            "dialog.integration.stage_downloading",
            "dialog.integration.stage_extracting",
            "data-ui-entity=\"progress-bar\"",
            "data-ui-action=\"cancel-integration-download\"",
            "toggleWebAccess",
            "toggleTrackedApp",
            "confirmFiltered",
            "unconfirmFiltered",
            "clearMonitoring",
            "requestDeleteObservation",
            "renderObservationDeleteModal",
            "confirmDeleteObservation",
            "cancel-delete-observation",
            "confirm-delete-observation",
            "dialog.delete_observation.title",
            "delete-observation",
            "openProfileExportDialog",
            "renderProfileExportDialog",
            "profileExportRequest",
            "refreshProfileExportPreview",
            "analyzeProfileExport",
            "applyProfileExport",
            "revertProfileExport",
            "export-profile-modal",
            "data-ui-entity=\"tabs\"",
            "profile-export-tabs",
            "profile-export-tabs-body",
            "data-ui-entity=\"tabs-body\"",
            "data-ui-entity=\"help-text\"",
            "class=\"tabs__tab\"",
            "profile-export-mode-attach-button",
            "profile-export-attach-profile-input",
            "profile-export-attach-profile-select",
            "profile-export-attach-generated-name-input",
            "profile-export-attach-note",
            "browse-profile-export-attach-profile-button",
            "clear-profile-export-attach-profile-button",
            "profile-export-patch-profile-input",
            "profile-export-patch-profile-select",
            "profile-export-patch-note",
            "browse-profile-export-patch-profile-button",
            "profile-export-merge-profile-input",
            "profile-export-merge-note",
            "profile-export-dangerous-switch",
            "profile-export-preview-panel",
            "profile-export-analyze-button",
            "profile-export-revert-button",
            "data-ui-action=\"analyze-profile-export\"",
            "data-ui-action=\"revert-profile-export\"",
            "repoRootKey",
            "selectedProfilePaths",
            "resetProfileExportControlsWhenIntegrationChanged",
            "profileExportSelectedProfilePath",
            "setProfileExportSelectedProfilePath",
            "dialog.profile_export.available_profiles",
            "dialog.profile_export.mode_attach",
            "dialog.profile_export.note_attach",
            "profileExportProfileOptions",
            "selectProfileExportProfile",
            "/v1/export/profile/preview",
            "/v1/export/profile/apply",
            "/v1/export/profile/revert",
            "dialog.profile_export.title",
            "footer.event.profile_export_applied",
            "/v1/observations/delete",
            "/v1/integrations/",
            "/v1/tracked-apps/",
            "/icon",
            "/v1/assets/close-times.svg",
            "/v1/assets/module-reorder-left.svg",
            "/v1/assets/copy.svg",
            "/v1/assets/confirm-filtered.svg",
            "/v1/assets/unconfirm-filtered.svg",
            "/v1/assets/ignore-address.svg",
            "/v1/assets/trash.svg",
            "/v1/assets/import-cloud.svg",
            "/v1/assets/export-cloud.svg",
            "/v1/assets/my-publications.svg",
            "/v1/assets/monitoring.svg",
            "/v1/assets/sort-idle.svg",
            "/v1/assets/sort-asc.svg",
            "/v1/assets/sort-desc.svg",
            "/v1/languages",
            "/v1/web-url",
            "/v1/endpoint-probe-targets",
            "language-select",
            "setLanguage",
            "ui.language",
            "webUrl",
            "loadWebUrl",
            "endpointProbeTargets",
            "loadEndpointProbeTargets",
            "endpointProbeRequestSpec",
            "endpoint_probe_protocol",
            "app-filter",
            "port-filter",
            "protocol-filter",
            "ip-filter",
            "action.unconfirm_filtered",
            "clear-monitoring-button",
            "confirm-clear-monitoring",
            "localizedClearMonitoringLabel",
            "filter.protocol.all",
            "header-switch-row--server",
            "app-shell",
            "shell--controls-disabled",
            "data-ui-disabled",
            "renderShellDisabled",
            "host-unavailable-overlay",
            "scheduleHostAvailabilityCheck",
            "state.hostUnavailableFailures < 3",
            "web.host_unavailable.title",
            "web.host_unavailable.text",
            "web.host_unavailable.note",
            "button button--icon",
            "button__plus",
            "button--square button--close",
            "button__icon",
            "footer-watcher-status",
            "footer-tool-status",
            "footer-web-server-status",
            "footer-network-status",
            "embedded watcher",
            "TOOL_MODULE_NAME",
            "footer-tool-dot",
            "footer-web-server-dot",
            "footer-network-dot",
            "footer-message-panel",
            "footer-message-text",
            "class=\"footer-message-text path-field\"",
            "footer-watcher-dot--connected",
            "footer-watcher-dot--disconnected",
            "footer-watcher-dot--inactive",
            "availabilityStatusLine",
            "status.waiting_for_watcher",
            "footer.web_server.copied_prefix",
            "footer.network.available_prefix",
            "endpointProbeTooltip",
            "renderFooterNetwork",
            "renderFooterTool",
            "footer.message.label",
            "footer.status.copy_tooltip",
            "footer.status.copied",
            "copyWebServerUrl",
            "copyStatusHistory",
            "recordFooterMessageEvent",
            "loadFooterMessages(100)",
            "footer-message-text--success",
            "footer-message-text--warning",
            "footer-message-text--error",
            "statusHistoryTooltip",
            "activeFooterMessageLines",
            "browserUiLogUrl",
            "status.web_server",
            "status.dns",
            "footer.event.monitoring_started",
            "footer.event.monitoring_stopped",
            "path-field--content-width",
            "pathFieldContentWidthStyle",
            "hidden > 0 ? ['...' + hidden] : []",
            "renderLabelWithColon('language-label', 'footer.language.label', 'Language');",
            "renderLabelWithColon('footer-message-label', 'footer.message.label', 'Message');",
            "toggleLanguageMenu",
            "renderFooterWebServer",
            "toggle-monitoring-button",
            "toggleMonitoring",
            "button--monitoring-active",
            "integration-dialog-path__input",
            "dialog.integration.primary_path",
            "dialog.integration.export_path",
            "integration-download-actions",
            "integration-folder-field-row",
            "integrationVisualPercent",
            "integrationPreviewPathsMatch",
            "progressBarEntityHtml",
            "normalizeProgressStages",
            "integrationProgressStages",
            "moduleUiProgressStages",
            "mergeAdjacentProgressStages",
            "progressBarCurrentText",
            "parseProgressPercent",
            "progress_stages",
            "applyIntegrationDownloadProgress",
            "pollIntegrationDownloadProgress",
            "progress-bar__segments",
            "progress-bar__segment--accent",
            "progress-bar__segment--success",
            "progress-bar__segment--rust",
            "progress-bar__remaining",
            "progress-bar__stages",
            "width: 75",
            "width: 25",
            "footer.web_server.localhost_fallback",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser UI should include {expected}"
            );
        }

        assert!(
            !BROWSER_UI_HTML.contains("file:///"),
            "browser UI must use local watcher icon routes instead of file URLs"
        );
        assert!(
            !BROWSER_UI_HTML.contains("Language: English</span>"),
            "browser UI should render a real language selector, not a static language label"
        );
        assert!(
            !BROWSER_UI_HTML.contains("footer-watcher-state"),
            "browser footer should use only the watcher indicator, not a Connected/Disconnected text label"
        );
        assert!(
            !BROWSER_UI_HTML.contains("status-line"),
            "browser status/errors panel should be removed in favor of footer Message"
        );
        assert!(
            !BROWSER_UI_HTML.contains("footer-message-text__value"),
            "browser footer Message should use the shared read-only path-field directly"
        );
        assert!(
            !BROWSER_UI_HTML.contains("app-subtitle"),
            "browser header should not render the removed subtitle block"
        );
        assert!(
            !BROWSER_UI_HTML.contains("id=\"app-title\"")
                && !BROWSER_UI_HTML.contains("renderText('app-title'"),
            "browser header should not render the application title"
        );
        assert!(
            !BROWSER_UI_HTML.contains("id=\"monitor-subtitle\""),
            "browser panel subtitles should be moved into help icons"
        );
        assert!(
            !BROWSER_UI_HTML.contains(
                "<h2 id=\"ignored-delete-modal-title\">Delete ignored address?</h2>\n          <p class=\"help-text\""
            ),
            "browser confirm-overlay help text should be in body, not panel header"
        );
        assert!(
            BROWSER_UI_HTML.contains(
                "<div class=\"modal__body\">\n        <p class=\"help-text\" id=\"ignored-delete-modal-help\""
            ),
            "browser confirm-overlay body should contain the descriptive help text"
        );
        assert!(
            !BROWSER_UI_HTML.contains("class=\"badge\" id=\"monitor-state\""),
            "browser monitor header status should not use a badge surface"
        );
        assert!(
            !BROWSER_UI_HTML.contains("class=\"badge")
                && !BROWSER_UI_HTML.contains("class=\"chip")
                && !BROWSER_UI_HTML.contains("badge--")
                && !BROWSER_UI_HTML.contains("chip--"),
            "browser UI should use label classes instead of badge/chip surfaces"
        );
        assert!(
            !BROWSER_UI_HTML.contains("id=\"start-monitoring-button\"")
                && !BROWSER_UI_HTML.contains("id=\"stop-monitoring-button\""),
            "browser monitor controls should expose one toggle monitoring button"
        );
        assert!(
            !BROWSER_UI_HTML.contains("class=\"badge\" id=\"observations-count\""),
            "browser observations header counter should not use a badge surface"
        );
        assert!(
            !BROWSER_UI_HTML.contains("class=\"badge\" id=\"integration-ready\""),
            "browser integration header status should not use a badge surface"
        );
        assert!(
            !BROWSER_UI_HTML.contains("<h3>' + html(t('ignored_addresses.title'"),
            "browser ignored-address title should be outside the scrollable list"
        );
        assert!(BROWSER_UI_HTML.contains("ignoredAddressDomainText(rule.enrichment)"));
        assert!(BROWSER_UI_HTML.contains("ignoredAddressHelpHtml(rule, 'end')"));
        assert!(BROWSER_UI_HTML.contains("enrichment.localhost_rule"));
        assert!(BROWSER_UI_HTML.contains("enrichment.local_ip_rule"));
        assert!(BROWSER_UI_HTML.contains(".ip-cell .help-icon { top: 0; }"));
        assert!(
            BROWSER_UI_HTML.contains("<span class=\"help-icon\""),
            "browser help affordances should render the shared help icon surface"
        );
        assert!(
            BROWSER_UI_HTML.contains(">?</span>"),
            "browser help affordances should render the restored question mark glyph"
        );
        assert!(
            !BROWSER_UI_HTML.contains("badge badge--success' : 'integration-dialog-status-text"),
            "browser integration modal status should not use badge classes"
        );
        assert!(
            !BROWSER_UI_HTML.contains("currentFooterStatusLines"),
            "browser footer messages must come from system_events instead of snapshot fallback lines"
        );
        assert!(
            !BROWSER_UI_HTML.contains("appendStatusLines"),
            "browser footer should not append snapshot status lines on each render"
        );
        assert!(
            !BROWSER_UI_HTML.contains("integration_status?.details"),
            "browser footer should not show the old integration note fallback"
        );
        assert!(
            !BROWSER_UI_HTML.contains("[[24, 256]"),
            "browser integration progress should use backend stages instead of synthetic timer steps"
        );
        assert!(
            !BROWSER_UI_HTML.contains("progress-bar__fill"),
            "browser progress bar should use segmented stages plus a remaining mask, not a single fill"
        );
        assert!(
            !BROWSER_UI_HTML.contains("UI filter"),
            "browser delete modal should not expose implementation wording"
        );
        assert!(
            !BROWSER_UI_HTML.contains("Integration folder path"),
            "browser integration modal should use the shared Path placeholder"
        );
        assert!(
            !BROWSER_UI_HTML.contains("/NetStitch"),
            "browser UI must not publish or link the old app-name URL path"
        );
    }

    #[test]
    fn browser_modules_panel_uses_bootstrap_skeletons_instead_of_empty_copy() {
        assert!(BROWSER_UI_HTML.contains("integration-module-button-skeleton"));
        assert!(BROWSER_UI_HTML.contains("Array.from({ length: 10 }"));
        assert!(BROWSER_UI_HTML.contains("target.innerHTML = snapshot"));
        assert!(BROWSER_UI_HTML.contains(
            "template('integration.loaded', 'Loaded: {count}', { count: modules.length })"
        ));
        let removed_key = ["integration", "missing"].join(".");
        let removed_en = ["No", " modules"].concat();
        let removed_ru = ["Модулей", "нет"].join(" ");
        assert!(!BROWSER_UI_HTML.contains(&removed_key));
        assert!(!BROWSER_UI_HTML.contains(&removed_en));
        assert!(!BROWSER_UI_HTML.contains(&removed_ru));
    }

    #[test]
    fn browser_endpoint_probe_targets_are_external_to_embedded_html() {
        assert!(BROWSER_UI_HTML.contains("/v1/endpoint-probe-targets"));
        for compiled_target in ["8.8.8.8:53", "1.1.1.1:53", "2001:4860:4860::8888"] {
            assert!(
                !BROWSER_UI_HTML.contains(compiled_target),
                "browser shell must load endpoint probe target {compiled_target} from config/API"
            );
        }
    }

    #[test]
    fn browser_module_ui_uses_web_snapshot_rows_and_desktop_sort_headers() {
        for token in [
            "const allRows = snapshotMonitoringRows(state.snapshot);",
            "const allRows = snapshotMonitoringRows(snapshot);",
            "<span class=\"table-sortable__content\"><img class=\"table-sortable__icon table-sortable__icon--",
            "<span class=\"table-sortable__label\">' + html(cell) + '</span></span></th>",
            ".ui-entity-table {\n      width: 100%;\n      min-width: 100%;\n      max-width: none;\n      border-collapse: collapse;\n      background: var(--list);",
            ".ui-entity-table th {\n      padding: 1px 10px;\n      background: var(--chrome);\n      color: var(--strong);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser module UI should keep desktop-compatible table/background token {token}"
            );
        }

        for obsolete in [
            "state.snapshot?.observations",
            "snapshot.observations",
            "<span class=\"table-sortable__label\">' + html(cell) + '</span><img",
            "background: var(--surface-strong);",
            "background: var(--surface-muted);",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser module UI must not keep obsolete token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_profile_export_overlay_uses_profile_endpoints_not_obsolete_export() {
        assert!(!BROWSER_UI_HTML.contains("id=\"export-confirmed-button\""));
        assert!(!BROWSER_UI_HTML.contains("function exportConfirmed()"));
        assert!(!BROWSER_UI_HTML.contains("/v1/assets/export-confirmed.svg"));
        assert!(!BROWSER_UI_HTML.contains("function exportIntegrationModule()"));
        assert!(!BROWSER_UI_HTML.contains("function configureIntegrationModule()"));
        assert!(!BROWSER_UI_HTML.contains("id=\"integration-export-button\""));
        assert!(!BROWSER_UI_HTML.contains("id=\"integration-configure-button\""));
        assert!(BROWSER_UI_HTML.contains("function returnToIntegrationModulePanel()"));
        assert!(BROWSER_UI_HTML.contains("state.profileExportReturnToModule = false;"));
        assert!(BROWSER_UI_HTML.contains("id=\"export-profile-modal\""));
        assert!(BROWSER_UI_HTML.contains("role=\"tablist\""));
        assert!(BROWSER_UI_HTML.contains("data-ui-entity=\"tabs\""));
        assert!(BROWSER_UI_HTML.contains("class=\"tabs__body profile-export-tabs-body\""));
        assert!(BROWSER_UI_HTML.contains("data-ui-entity=\"tabs-body\""));
        assert!(BROWSER_UI_HTML.contains(".tabs__tab[aria-selected=\"true\"]::after"));
        assert!(BROWSER_UI_HTML.contains("height: 3px;"));
        assert!(BROWSER_UI_HTML.contains("data-profile-export-panel=\"attach_netstitch_lists\""));
        assert!(BROWSER_UI_HTML.contains("data-profile-export-panel=\"patch_selected_profile\""));
        assert!(
            BROWSER_UI_HTML.contains("data-profile-export-panel=\"merge_into_existing_lists\"")
        );
        assert!(BROWSER_UI_HTML.contains("browseProfileExportProfile()"));
        assert!(BROWSER_UI_HTML.contains("clearProfileExportPath()"));
        assert!(BROWSER_UI_HTML.contains("id=\"profile-export-dangerous-switch\""));
        assert!(BROWSER_UI_HTML.contains("class=\"profile-export-switch-row\""));
        assert!(!BROWSER_UI_HTML.contains("id=\"profile-export-dangerous-checkbox\""));
        assert!(BROWSER_UI_HTML.contains("const exportReady = observationRowSelected(row);"));
        assert!(BROWSER_UI_HTML.contains("handleObservationRowClick(event, "));
        assert!(BROWSER_UI_HTML.contains(", ' + exportReady + ')"));
        assert!(!BROWSER_UI_HTML.contains("data-ui-action=\"preview-profile-export\""));
        assert!(!BROWSER_UI_HTML.contains("id=\"profile-export-preview-button\""));
        assert!(BROWSER_UI_HTML.contains("refreshProfileExportPreview();"));
        assert!(BROWSER_UI_HTML.contains("profileExportMetricRowHtml("));
        assert!(BROWSER_UI_HTML.contains("profileExportVisibleWarnings("));
        assert!(BROWSER_UI_HTML.contains("code !== 'profile_merge_target_skipped'"));
        assert!(
            BROWSER_UI_HTML
                .contains("profile-export-preview__metric .profile-export-preview__muted")
        );
        assert!(BROWSER_UI_HTML.contains("profile-export-preview__note"));
        assert!(BROWSER_UI_HTML.contains("justify-items: stretch"));
        assert!(BROWSER_UI_HTML.contains("box-sizing: border-box"));
        assert!(BROWSER_UI_HTML.contains("Selected"));
        assert!(BROWSER_UI_HTML.contains("Will be added"));
        assert!(BROWSER_UI_HTML.contains("Skipped"));
        assert!(!BROWSER_UI_HTML.contains("Exported IPs"));
        assert!(BROWSER_UI_HTML.contains("data-ui-action=\"analyze-profile-export\""));
        assert!(BROWSER_UI_HTML.contains("data-ui-action=\"backup-profile-export\""));
        assert!(BROWSER_UI_HTML.contains("data-ui-action=\"apply-profile-export\""));
        assert!(BROWSER_UI_HTML.contains("data-ui-action=\"revert-profile-export\""));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-manual-domains-input"));
        assert!(BROWSER_UI_HTML.contains("rows=\"8\""));
        assert!(BROWSER_UI_HTML.contains(".split(/\\r?\\n/)"));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-body"));
        assert!(
            BROWSER_UI_HTML.contains(".profile-export-advanced-dialog {\n      display: grid;")
        );
        assert!(BROWSER_UI_HTML.contains("grid-template-rows: auto minmax(0, 1fr) auto;"));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-panel"));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-option"));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-panel--settings"));
        assert!(BROWSER_UI_HTML.contains("profile-export-advanced-panel--uncovered"));
        assert!(BROWSER_UI_HTML.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(BROWSER_UI_HTML.contains("grid-template-rows: auto auto minmax(0, 1fr);"));
        assert!(BROWSER_UI_HTML.contains("profile-export-field--domains"));
        assert!(BROWSER_UI_HTML.contains(
            ".profile-export-field--domains > .input-box.input.profile-export-domains-input"
        ));
        assert!(BROWSER_UI_HTML.contains("height: 100%;"));
        assert!(BROWSER_UI_HTML.contains("resize: none;"));
        assert!(BROWSER_UI_HTML.contains("width: min(940px, calc(100vw - 48px));"));
        assert!(
            BROWSER_UI_HTML
                .contains("splitManualDomains(state.profileExport.advancedManualDomains)")
        );
        assert!(BROWSER_UI_HTML.contains("post('/v1/export/profile/advanced-settings/apply'"));
        assert!(BROWSER_UI_HTML.contains("advanced_manual_domains"));
        assert!(BROWSER_UI_HTML.contains("post('/v1/export/profile/preview'"));
        assert!(BROWSER_UI_HTML.contains("post('/v1/export/profile/analyze'"));
        assert!(BROWSER_UI_HTML.contains("post('/v1/export/profile/backup'"));
        assert!(BROWSER_UI_HTML.contains("post('/v1/export/profile/apply'"));
        assert!(
            BROWSER_UI_HTML
                .contains("dangerous_confirmed: Boolean(state.profileExport.dangerousConfirmed)")
        );
        assert!(BROWSER_UI_HTML.contains("preview.innerHTML = dangerousBlocked ? '' : profileExportPreviewHtml(state.profileExport.preview);"));
        assert!(BROWSER_UI_HTML.contains("state.profileExport.previewRequestId += 1;"));
        assert!(!BROWSER_UI_HTML.contains("onclick=\"exportConfirmed()\""));
    }

    #[test]
    fn endpoint_probe_targets_parse_external_config_format() {
        let parsed = parse_endpoint_probe_targets(
            "udp://1.1.1.1:53,8.8.8.8:53\n# comment\n[2606:4700:4700::1111]:53;9.9.9.9:53 # inline",
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
    fn endpoint_probe_targets_carry_protocol_from_snapshot_requests() {
        assert_eq!(
            canonical_endpoint_probe_target("1.1.1.1:53", Some("udp")),
            "udp://1.1.1.1:53"
        );
        assert_eq!(
            canonical_endpoint_probe_target("udp://1.1.1.1:53", Some("tcp")),
            "tcp://1.1.1.1:53"
        );
        assert_eq!(
            canonical_endpoint_probe_target("example.com:443", None),
            "tcp://example.com:443"
        );
    }

    #[test]
    fn current_endpoint_probe_targets_falls_back_to_external_config_when_runtime_state_is_empty() {
        let temp_file = std::env::temp_dir().join(format!(
            "NetStitch-endpoint-probe-targets-{}.txt",
            std::process::id()
        ));
        std::fs::write(&temp_file, "udp://94.140.14.14:53\nudp://77.88.8.8:53\n")
            .expect("temp endpoint probe target file writes");

        let previous_targets = std::env::var(APP_ENV_ENDPOINT_PROBE_TARGETS).ok();
        let previous_targets_file = std::env::var(APP_ENV_ENDPOINT_PROBE_TARGETS_FILE).ok();
        unsafe {
            std::env::remove_var(APP_ENV_ENDPOINT_PROBE_TARGETS);
            std::env::set_var(APP_ENV_ENDPOINT_PROBE_TARGETS_FILE, &temp_file);
        }

        let loaded = current_endpoint_probe_targets(&Arc::new(Mutex::new(Vec::new())));

        if let Some(value) = previous_targets {
            unsafe { std::env::set_var(APP_ENV_ENDPOINT_PROBE_TARGETS, value) };
        } else {
            unsafe { std::env::remove_var(APP_ENV_ENDPOINT_PROBE_TARGETS) };
        }
        if let Some(value) = previous_targets_file {
            unsafe { std::env::set_var(APP_ENV_ENDPOINT_PROBE_TARGETS_FILE, value) };
        } else {
            unsafe { std::env::remove_var(APP_ENV_ENDPOINT_PROBE_TARGETS_FILE) };
        }
        let _ = std::fs::remove_file(temp_file);

        assert_eq!(loaded, vec!["udp://94.140.14.14:53", "udp://77.88.8.8:53"]);
    }

    #[test]
    fn snapshot_query_accepts_single_endpoint_probe_target_value() {
        let parsed: SnapshotQuery = parse_snapshot_query(Some(
            "endpoint_probe_target=one.one.one.one:443&endpoint_probe_protocol=tcp",
        ));

        assert_eq!(parsed.endpoint_probe_target, vec!["one.one.one.one:443"]);
        assert_eq!(parsed.endpoint_probe_protocol, vec!["tcp"]);
    }

    #[test]
    fn snapshot_query_accepts_duplicate_endpoint_probe_target_fields() {
        let parsed: SnapshotQuery = parse_snapshot_query(Some(
            "endpoint_probe_target=one.one.one.one:443&endpoint_probe_target=1.1.1.1:53&endpoint_probe_protocol=tcp&endpoint_probe_protocol=udp",
        ));

        assert_eq!(
            parsed.endpoint_probe_target,
            vec!["one.one.one.one:443", "1.1.1.1:53"]
        );
        assert_eq!(parsed.endpoint_probe_protocol, vec!["tcp", "udp"]);
    }

    #[test]
    fn browser_add_exe_control_matches_desktop_icon_button_contract() {
        assert!(
            DESKTOP_APP_RS.contains("id: ui::id::ADD_EXE_BUTTON"),
            "desktop UI should expose a stable Add exe control"
        );
        assert!(
            DESKTOP_APP_RS.contains("class: \"input-box button button--icon\""),
            "desktop Add exe control should use the shared icon-button class"
        );
        assert!(
            DESKTOP_APP_RS.contains("\"+\""),
            "desktop Add exe control should render a visible plus icon"
        );
        assert!(
            BROWSER_UI_HTML.contains("id=\"add-exe-button\""),
            "browser UI should expose a stable Add exe control"
        );
        assert!(
            BROWSER_UI_HTML.contains("class=\"button button--icon\""),
            "browser Add exe control should use the same icon-button subtype"
        );
        assert!(
            BROWSER_UI_HTML.contains("<span class=\"button__plus\" aria-hidden=\"true\">+</span>"),
            "browser Add exe control should render a visible plus icon"
        );
        assert!(
            BROWSER_UI_HTML.contains(".button--icon"),
            "browser UI should define icon-button styles instead of relying on a narrow generic button"
        );
    }

    #[test]
    fn browser_and_desktop_share_core_visual_contracts() {
        for (name, color) in [
            ("workbench background", "#1e1e1e"),
            ("panel header", "#2d2d2d"),
            ("panel background", "#252526"),
            ("list background", "#181818"),
            ("row background", "#2b2b2b"),
            ("enabled row background", "#2b3338"),
            ("control background", "#1f1f1f"),
            ("border", "#3c3c3c"),
            ("accent", "#007acc"),
            ("text", "#cccccc"),
            ("strong text", "#f0f0f0"),
            ("scrollbar track", "#1e1e1e"),
            ("scrollbar thumb", "#424242"),
            ("scrollbar thumb hover", "#525252"),
        ] {
            assert!(
                DESKTOP_THEME_RS.contains(color),
                "desktop theme should keep {name} color {color}"
            );
            assert!(
                BROWSER_UI_HTML.contains(color),
                "browser theme should keep {name} color {color}"
            );
        }

        for (name, desktop_token, browser_token) in [
            ("control radius", "--radius-control: 3px;", "--radius: 3px;"),
            (
                "tracked app list height",
                "--size-tracked-app-list-height: calc((var(--size-tracked-app-height) * var(--size-tracked-app-visible-rows)) + (5px * (var(--size-tracked-app-visible-rows) - 1)) + 10px + 2px);",
                "--tracked-app-list-height: calc((52px * var(--tracked-app-visible-rows)) + (5px * (var(--tracked-app-visible-rows) - 1)) + 10px + 2px);",
            ),
            (
                "tracked app visible rows",
                "--size-tracked-app-visible-rows: 4;",
                "--tracked-app-visible-rows: 4;",
            ),
            ("panel gap", "--size-panel-gap: 8px;", "--panel-gap: 8px;"),
            (
                "panel stack gap",
                "--size-panel-stack-gap: 8px;",
                "--panel-stack-gap: 8px;",
            ),
            (
                "modules panel compact minimum height",
                "--size-modules-panel-height: 78px;",
                "--modules-panel-height: 78px;",
            ),
            (
                "modules grid compact padding",
                ".integration-module-grid {\n  display: flex;",
                ".integration-module-grid {\n      display: flex;",
            ),
            (
                "module button size token",
                "--size-module-button: 36px;",
                "--module-button-size: 36px;",
            ),
            (
                "module button slot uses module size",
                "--size-module-button-slot: calc(var(--size-module-button) + (var(--size-module-button-pulse-gutter) * 2));",
                "--module-button-slot: calc(var(--module-button-size) + (var(--module-button-pulse-gutter) * 2));",
            ),
            (
                "modules grid vertical space",
                "padding: var(--size-module-button-pulse-gutter) 6px calc(var(--size-module-button-pulse-gutter) + var(--size-scrollbar)) 0;",
                "padding: var(--module-button-pulse-gutter) 6px calc(var(--module-button-pulse-gutter) + var(--scrollbar-size)) 0;",
            ),
            (
                "module activity pulse is system white",
                "--color-module-activity-pulse: rgba(255, 255, 255, 0.34);",
                "--module-activity-pulse: rgba(255, 255, 255, 0.34);",
            ),
            (
                "module header pulse uses dedicated white outline animation",
                "animation: module-action-button-pulse 1.6s ease-in-out infinite;",
                "animation: module-action-button-pulse 1.6s ease-in-out infinite;",
            ),
            (
                "tracked app path bottom alignment pad",
                "--size-tracked-apps-path-bottom-pad: 0px;",
                "--tracked-apps-path-bottom-pad: 0px;",
            ),
            (
                "control label starts at control edge",
                "--size-control-label-offset: 0px;",
                "--size-control-label-offset: 0px;",
            ),
            (
                "profile export labels use header-like muted style",
                ".profile-export-field > .value-label {\n  padding-left: var(--size-control-label-offset);\n  color: var(--color-text-muted);\n  font-size: 11px;\n  font-weight: 600;",
                ".profile-export-field label {\n      color: var(--muted);\n      font-size: 11px;\n      font-weight: 600;",
            ),
            (
                "app icon slot",
                "--size-app-icon-slot: 40px;",
                "width: 40px;",
            ),
            (
                "app icon image",
                "--size-app-icon-image: 32px;",
                ".app-icon img { width: 32px; height: 32px; object-fit: contain; }",
            ),
            (
                "compact icon button",
                "--size-compact-control: 30px;",
                ".button--icon {\n      width: var(--size-compact-control);",
            ),
            (
                "module header icons",
                ".button__icon--module-stop,\n.button__icon--module-close,\n.button__icon--module-action {\n  width: 28px;\n  height: 28px;",
                ".button__icon--module-stop,\n    .button__icon--module-close,\n    .button__icon--module-action {\n      width: 28px;\n      height: 28px;",
            ),
            (
                "module button image",
                ".integration-module-button__image {\n  display: block;\n  width: 80%;\n  height: 80%;\n  object-fit: contain;",
                ".integration-module-button__image {\n      display: block;\n      width: 80%;\n      height: 80%;\n      object-fit: contain;",
            ),
            (
                "panel footer fixed height token",
                "--size-panel-footer-height: 43px;",
                "--size-panel-footer-height: 43px;",
            ),
            (
                "panel footer fixed flex basis",
                "flex: 0 0 var(--size-panel-footer-height);",
                "flex: 0 0 var(--size-panel-footer-height);",
            ),
            (
                "close button",
                "--size-close-button: 20px;",
                ".button--square {\n      width: var(--size-close-button);",
            ),
            (
                "footer watcher indicator",
                ".footer-watcher-status__indicator {\n  width: 14px;",
                ".footer-watcher-dot {\n      width: 14px;",
            ),
            (
                "footer language selector center alignment",
                ".language-select-control__label {\n  min-width: 0;",
                ".footer-pill .select {\n      width: auto;",
            ),
            (
                "footer language selector without native arrow",
                ".language-select-control {\n  position: relative;\n  display: inline-grid;\n  grid-template-columns: minmax(0, 1fr);",
                "appearance: none;",
            ),
            (
                "footer language dropdown anchored to selector",
                ".language-select-menu {\n  position: absolute;\n  right: 0;\n  bottom: 100%;",
                ".language-select-menu {\n      position: absolute;\n      right: 0;\n      bottom: 100%;",
            ),
            (
                "workspace row sizing",
                "grid-template-rows: auto minmax(0, 1fr);",
                "grid-template-rows: auto minmax(0, 1fr);",
            ),
            (
                "ignored addresses card keeps five-row intrinsic height",
                ".ignored-addresses-card {\n  flex: 0 0 auto;",
                ".ignored-addresses-card { flex: 0 0 auto;",
            ),
            (
                "modules panel absorbs top-column leftover height",
                ".modules-card {\n  flex: 1 1 var(--size-modules-panel-height);",
                ".modules-card {\n      flex: 1 1 var(--modules-panel-height);",
            ),
            (
                "ignored addresses section",
                ".ignored-addresses-section",
                ".ignored-addresses-section",
            ),
            (
                "labelled values use primary text color",
                ".integration-status__path,\n.integration-status__provider,\n.integration-status__value {\n  min-width: 0;\n  margin: 0;\n  overflow: hidden;\n  color: var(--color-text);",
                ".integration-status__path,\n    .integration-status__provider,\n    .integration-status__value {\n      min-width: 0;\n      margin: 0;\n      overflow: hidden;\n      color: var(--text);",
            ),
            (
                "module menu auto-sized panel",
                ".integration-module-dialog {\n  width: fit-content;",
                ".integration-module-dialog {\n      width: fit-content;",
            ),
            (
                "module menu compact fields",
                ".integration-status-layout--module-menu .integration-status__fields {\n  gap: 2px;",
                ".integration-status-layout--module-menu .integration-status__fields {\n      gap: 2px;",
            ),
            (
                "module menu compact rows",
                ".integration-status-layout--module-menu .integration-status__row {\n  min-height: 18px;",
                ".integration-status-layout--module-menu .integration-status__row {\n      min-height: 18px;",
            ),
            (
                "module menu viewport bounded vertical scroll",
                "overflow-x: hidden;\n  overflow-y: auto;",
                "overflow-x: hidden;\n      overflow-y: auto;",
            ),
            (
                "domain field uses primary text color",
                ".domain-field {\n  width: min(260px, 100%);\n  min-width: 120px;\n  color: var(--color-text);",
                ".domain-field {\n      width: min(260px, 100%);\n      min-width: 120px;\n      min-height: 24px;\n      padding: 2px 6px;\n      color: var(--text);",
            ),
            (
                "ignored address tuned row height",
                "--size-ignored-address-row-height: 30px;",
                "--ignored-address-row-height: 30px;",
            ),
            (
                "ignored address visible row estimate",
                "--size-ignored-addresses-visible-rows: 5;",
                "--ignored-addresses-visible-rows: 5;",
            ),
            (
                "ignored address exact five-row list height",
                "--size-ignored-addresses-list-height: calc((var(--size-ignored-address-row-height) * var(--size-ignored-addresses-visible-rows)) + (var(--size-ignored-address-row-gap) * (var(--size-ignored-addresses-visible-rows) - 1)) + 10px + 2px);",
                "--ignored-addresses-list-height: calc((var(--ignored-address-row-height) * var(--ignored-addresses-visible-rows)) + (var(--ignored-address-row-gap) * (var(--ignored-addresses-visible-rows) - 1)) + 10px + 2px);",
            ),
            (
                "ignored address fixed auto rows",
                "grid-auto-rows: var(--size-ignored-address-row-height);",
                "grid-auto-rows: var(--ignored-address-row-height);",
            ),
            (
                "ignored address list fixed to five rows",
                "height: var(--size-ignored-addresses-list-height);",
                "height: var(--ignored-addresses-list-height);",
            ),
            (
                "list scrollbar gutter reserve",
                "scrollbar-gutter: stable;",
                "scrollbar-gutter: stable;",
            ),
            (
                "ignored address no vertical stretch",
                "align-content: start;",
                "align-content: start;",
            ),
            (
                "ignored address panel help raised",
                "#ignored-addresses-panel-help {\n  position: relative;\n  top: -2px;",
                "#ignored-addresses-panel-help {\n      position: relative;\n      top: -2px;",
            ),
            (
                "ignored address fixed ip column",
                "--size-ignored-address-ip-column: 230px;",
                "--ignored-address-ip-column: 230px;",
            ),
            (
                "ignored address row columns",
                "grid-template-columns: var(--size-ignored-address-ip-column) 18px minmax(80px, 1fr) var(--size-close-button);",
                "grid-template-columns: var(--ignored-address-ip-column) 18px minmax(80px, 1fr) var(--size-close-button);",
            ),
            (
                "shared left value label",
                ".value-label,\n.integration-status__label,\n.integration-dialog-status-row__label",
                ".value-label,\n    .integration-status__label,\n    .integration-dialog-status-row__label",
            ),
            (
                "clearable path input shell",
                ".path-input-shell {\n  position: relative;",
                ".path-input-shell {\n      position: relative;",
            ),
            (
                "clearable path input button",
                ".path-input-clear {\n  position: absolute;",
                ".path-input-clear {\n      grid-column: 2;\n      grid-row: 1;",
            ),
            (
                "integration status two-column body",
                ".integration-status-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) auto;",
                ".integration-status-layout {\n      display: grid;\n      grid-template-columns: minmax(0, 1fr) auto;",
            ),
            (
                "integration status left width limit",
                "--size-integration-status-left-max: 520px;",
                "--size-integration-status-left-max: 520px;",
            ),
            (
                "integration status fields width cap",
                ".integration-status__fields {\n  display: grid;\n  width: 100%;\n  max-width: min(100%, var(--size-integration-status-left-max));",
                ".integration-status__fields {\n      display: grid;\n      width: 100%;\n      max-width: min(100%, var(--size-integration-status-left-max));",
            ),
            (
                "integration status right action",
                ".integration-status__action {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  justify-content: flex-end;\n  gap: 5px;\n  min-width: 0;",
                ".integration-status__action {\n      display: flex;\n      flex-direction: column;\n      align-items: stretch;\n      justify-content: flex-end;\n      gap: 5px;\n      min-width: 0;",
            ),
            (
                "tracked apps toolbar spacer",
                ".tracked-apps-toolbar-spacer {\n  min-height: 0;",
                ".tracked-apps-toolbar-spacer {\n      min-height: 0;",
            ),
            (
                "ignored address delete column",
                ".ignored-addresses__row .button {\n  grid-column: 4;",
                ".ignored-row .button {\n      grid-column: 4;",
            ),
            (
                "footer message fills available space",
                ".app-footer-message-panel {\n  flex: 1 1 auto;",
                ".footer-message-panel,\n    .app-footer-message-panel {\n      position: relative;\n      flex: 1 1 auto;",
            ),
            ("switch width", "--size-switch-width: 46px;", "width: 46px;"),
            (
                "switch height",
                "--size-switch-height: 22px;",
                "height: 22px;",
            ),
            (
                "global scrollbar width",
                "--size-scrollbar: 8px;",
                "--scrollbar-size: 8px;",
            ),
            (
                "global scrollbar firefox style",
                "scrollbar-width: thin;",
                "scrollbar-width: thin;",
            ),
            (
                "global scrollbar webkit style",
                "*::-webkit-scrollbar {\n  width: var(--size-scrollbar);",
                "*::-webkit-scrollbar {\n      width: var(--scrollbar-size);",
            ),
        ] {
            assert!(
                DESKTOP_THEME_RS.contains(desktop_token),
                "desktop theme should keep {name} token"
            );
            assert!(
                BROWSER_UI_HTML.contains(browser_token),
                "browser theme should mirror desktop {name} token"
            );
        }
        let desktop_theme_css = DESKTOP_THEME_RS
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(DESKTOP_THEME_RS);
        for removed in [
            "integration-dialog-status-row--paths",
            "integration-dialog-paths",
        ] {
            assert!(
                !desktop_theme_css.contains(removed),
                "desktop theme should not keep nested integration path group token {removed}"
            );
            assert!(
                !BROWSER_UI_HTML.contains(removed),
                "browser theme should not keep nested integration path group token {removed}"
            );
        }
    }

    #[test]
    fn browser_and_desktop_keep_stable_panel_layout_order() {
        for expected in [
            ".workspace {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  grid-template-rows: auto minmax(0, 1fr);\n  gap: var(--size-panel-gap);",
            ".workspace > .column:first-child {\n  grid-column: 1;\n  grid-row: 1;\n  align-self: stretch;\n  height: 100%;\n  min-height: 0;",
            ".workspace > .column:nth-child(2) {\n  grid-column: 2;\n  grid-row: 1;\n  align-self: stretch;\n  height: 100%;\n  gap: var(--size-panel-stack-gap);\n  min-height: 0;",
            ".workspace > .observations-card {\n  grid-column: 1 / -1;\n  grid-row: 2;\n  align-self: stretch;\n  height: 100%;\n  min-height: 0;",
        ] {
            assert!(
                DESKTOP_THEME_RS.contains(expected),
                "desktop layout should keep fixed top/right/full-width panel placement: {expected}"
            );
        }

        for expected in [
            ".workspace {\n      display: grid;\n      grid-template-columns: repeat(2, minmax(0, 1fr));\n      grid-template-rows: auto minmax(0, 1fr);\n      gap: var(--panel-gap);",
            ".workspace > .column:first-child {\n      grid-column: 1;\n      grid-row: 1;\n      align-self: stretch;\n      height: 100%;\n      min-height: 0;",
            ".workspace > .column:nth-child(2) {\n      grid-column: 2;\n      grid-row: 1;\n      align-self: stretch;\n      height: 100%;\n      gap: var(--panel-stack-gap);\n      min-height: 0;",
            ".workspace > .observations-card {\n      grid-column: 1 / -1;\n      grid-row: 2;\n      align-self: stretch;\n      height: 100%;\n      min-height: 0;",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser layout should mirror fixed panel placement: {expected}"
            );
        }

        assert!(
            !BROWSER_UI_HTML.contains(".workspace { grid-template-columns: 1fr; }"),
            "browser layout should not collapse the panel structure via media query"
        );
    }

    #[test]
    fn browser_and_desktop_share_icon_and_control_surface_contracts() {
        for (name, desktop_token, browser_token) in [
            (
                "app icon image containment",
                ".app-icon__image",
                ".app-icon img { width: 32px; height: 32px; object-fit: contain; }",
            ),
            (
                "close icon class",
                "class: \"button__icon\"",
                "class=\"button__icon\"",
            ),
            (
                "close icon asset",
                "CLOSE_TIMES_ICON_SVG",
                "/v1/assets/close-times.svg",
            ),
            (
                "monitoring radar asset",
                "MONITORING_ICON_SVG",
                "/v1/assets/monitoring.svg",
            ),
            (
                "confirm filtered asset",
                "CONFIRM_FILTERED_ICON_SVG",
                "/v1/assets/confirm-filtered.svg",
            ),
            (
                "sort idle asset",
                "SORT_IDLE_ICON_SVG",
                "/v1/assets/sort-idle.svg",
            ),
            (
                "sort asc asset",
                "SORT_ASC_ICON_SVG",
                "/v1/assets/sort-asc.svg",
            ),
            (
                "sort desc asset",
                "SORT_DESC_ICON_SVG",
                "/v1/assets/sort-desc.svg",
            ),
            (
                "ignore address asset",
                "IGNORE_ADDRESS_ICON_SVG",
                "/v1/assets/ignore-address.svg",
            ),
            (
                "cloud import asset",
                "resources/ui/icons/import-from-cloud-svgrepo-com.svg",
                "/v1/assets/import-cloud.svg",
            ),
            (
                "cloud export asset",
                "resources/ui/icons/upload-to-cloud-svgrepo-com.svg",
                "/v1/assets/export-cloud.svg",
            ),
            (
                "my publications asset",
                "resources/ui/icons/torso-svgrepo-com.svg",
                "/v1/assets/my-publications.svg",
            ),
            (
                "manual delete close button",
                "button--danger button--square button--close",
                "button--danger button--square button--close",
            ),
            (
                "enabled switch accent",
                ".switch--on {\n  background: var(--color-accent);",
                ".switch--on { background: var(--accent); }",
            ),
            (
                "switch knob travel",
                "transform: translateX(24px);",
                "transform: translateX(24px);",
            ),
            (
                "tracked app enabled row",
                ".tracked-app--enabled",
                ".tracked-app--enabled",
            ),
            (
                "web server inactive indicator",
                "footer-watcher-status__indicator--inactive",
                "footer-watcher-dot--inactive",
            ),
            (
                "footer message shared path field",
                "class: \"{footer_status_text_class}\"",
                "class=\"footer-message-text path-field\"",
            ),
        ] {
            assert!(
                DESKTOP_APP_RS.contains(desktop_token) || DESKTOP_THEME_RS.contains(desktop_token),
                "desktop UI should keep {name} contract"
            );
            assert!(
                BROWSER_UI_HTML.contains(browser_token),
                "browser UI should mirror desktop {name} contract"
            );
        }
    }

    #[test]
    fn browser_main_window_element_map_keeps_primary_regions_in_sync() {
        let entries = [
            MainWindowElementMapEntry {
                region: "header",
                name: "application filter",
                desktop_tokens: &[
                    "ui::id::OBSERVATION_APP_FILTER_INPUT",
                    "ui::control::OBSERVATION_APP_FILTER_INPUT",
                ],
                browser_tokens: &["id=\"app-filter\"", "setAppFilter(this.value)"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "ip filter",
                desktop_tokens: &[
                    "ui::id::OBSERVATION_SEARCH_INPUT",
                    "ui::control::OBSERVATION_SEARCH_INPUT",
                ],
                browser_tokens: &["id=\"ip-filter\"", "applyIpFilterOnEnter(event)"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "clear ip filter action",
                desktop_tokens: &[
                    "ui::id::CLEAR_OBSERVATION_SEARCH_BUTTON",
                    "ui::action::CLEAR_OBSERVATION_SEARCH",
                ],
                browser_tokens: &[
                    "id=\"clear-ip-filter-button\"",
                    "data-ui-action=\"clear-ip-filter\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "port filter",
                desktop_tokens: &[
                    "ui::id::OBSERVATION_PORT_FILTER_INPUT",
                    "ui::control::OBSERVATION_PORT_FILTER_INPUT",
                ],
                browser_tokens: &["id=\"port-filter\"", "applyPortFilterOnEnter(event)"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "clear port filter action",
                desktop_tokens: &["ui::action::CLEAR_OBSERVATION_PORT_SEARCH"],
                browser_tokens: &["clearPortFilter()"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "protocol filter",
                desktop_tokens: &[
                    "ui::id::OBSERVATION_PROTOCOL_FILTER_SELECT",
                    "ui::control::OBSERVATION_PROTOCOL_FILTER_SELECT",
                ],
                browser_tokens: &["id=\"protocol-filter\"", "setProtocolFilter(this.value)"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "status filter",
                desktop_tokens: &[
                    "ui::id::OBSERVATION_FILTER_SELECT",
                    "ui::control::OBSERVATION_FILTER_SELECT",
                ],
                browser_tokens: &[
                    "id=\"observation-filter\"",
                    "setObservationFilter(this.value)",
                ],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "toggle monitoring button",
                desktop_tokens: &[
                    "ui::id::TOGGLE_MONITORING_BUTTON",
                    "ui::action::TOGGLE_MONITORING",
                ],
                browser_tokens: &["id=\"toggle-monitoring-button\"", "toggleMonitoring()"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "confirm filtered button",
                desktop_tokens: &[
                    "ui::id::CONFIRM_FILTERED_BUTTON",
                    "ui::action::CONFIRM_FILTERED",
                ],
                browser_tokens: &["id=\"confirm-filtered-button\"", "confirmFiltered()"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "unconfirm filtered button",
                desktop_tokens: &[
                    "ui::id::UNCONFIRM_FILTERED_BUTTON",
                    "ui::action::UNCONFIRM_FILTERED",
                ],
                browser_tokens: &["id=\"unconfirm-filtered-button\"", "unconfirmFiltered()"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "clear monitoring button",
                desktop_tokens: &[
                    "ui::id::CLEAR_MONITORING_BUTTON",
                    "ui::action::CLEAR_MONITORING",
                ],
                browser_tokens: &["id=\"clear-monitoring-button\"", "clearMonitoring()"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "cloud import button",
                desktop_tokens: &[
                    "ui::id::OPEN_CLOUD_IMPORT_BUTTON",
                    "ui::action::OPEN_CLOUD_IMPORT",
                ],
                browser_tokens: &["id=\"cloud-import-button\"", "toggleCloudPanel('import')"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "cloud export button",
                desktop_tokens: &[
                    "ui::id::OPEN_CLOUD_EXPORT_BUTTON",
                    "ui::action::OPEN_CLOUD_EXPORT",
                ],
                browser_tokens: &["id=\"cloud-export-button\"", "toggleCloudPanel('export')"],
            },
            MainWindowElementMapEntry {
                region: "header",
                name: "web server switch",
                desktop_tokens: &[
                    "ui::id::WEB_ACCESS_LOCALHOST_BUTTON",
                    "ui::action::TOGGLE_WEB_ACCESS_LOCALHOST",
                ],
                browser_tokens: &["header-switch-row--server", "toggleWebAccess()"],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "panel shell",
                desktop_tokens: &[
                    "ui::entity::TRACKED_APPS_PANEL",
                    "ui::id::TRACKED_APPS_PANEL",
                ],
                browser_tokens: &[
                    "id=\"tracked-apps-panel\"",
                    "class=\"card tracked-apps-card\"",
                    "tracked-apps-header-meta",
                ],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "help and header status",
                desktop_tokens: &["ui::entity::TRACKED_APPS_HEADER", "HelpIcon {"],
                browser_tokens: &[
                    "id=\"tracked-apps-help\"",
                    "id=\"tracked-enabled-count\"",
                    "id=\"tracked-count-inline\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "exe path input",
                desktop_tokens: &["ui::id::EXE_PATH_INPUT", "ui::control::EXE_PATH_INPUT"],
                browser_tokens: &["id=\"exe-path\"", "path-input-shell"],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "clear exe path action",
                desktop_tokens: &[
                    "ui::id::CLEAR_EXE_PATH_BUTTON",
                    "ui::action::CLEAR_EXE_PATH",
                ],
                browser_tokens: &[
                    "id=\"clear-exe-path-button\"",
                    "data-ui-action=\"clear-exe-path\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "add exe action",
                desktop_tokens: &["ui::id::ADD_EXE_BUTTON", "ui::action::ADD_EXE"],
                browser_tokens: &["id=\"add-exe-button\"", "addTrackedApp()"],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "bulk enable row",
                desktop_tokens: &[
                    "ui::action::TOGGLE_ALL_APPS",
                    "ui::id::TOGGLE_ALL_APPS_BUTTON",
                ],
                browser_tokens: &["id=\"enable-all-switch\"", "toggleEnableAll()"],
            },
            MainWindowElementMapEntry {
                region: "tracked-apps",
                name: "tracked apps list",
                desktop_tokens: &[
                    "ui::entity::TRACKED_APP_LIST",
                    "ui::id::TRACKED_APP_LIST",
                    "ui::entity::TRACKED_APP_ITEM",
                ],
                browser_tokens: &["id=\"tracked-apps\"", "toggleTrackedApp("],
            },
            MainWindowElementMapEntry {
                region: "integration",
                name: "integration panel shell",
                desktop_tokens: &["ui::entity::INTEGRATION_PANEL", "ui::id::INTEGRATION_PANEL"],
                browser_tokens: &[
                    "id=\"integration-help\"",
                    "id=\"integration-info\"",
                    "integration-module-actions",
                ],
            },
            MainWindowElementMapEntry {
                region: "integration",
                name: "module actions",
                desktop_tokens: &[
                    "ui::action::OPEN_INTEGRATION_MODULE",
                    "integration-module-button",
                ],
                browser_tokens: &[
                    "open-integration-module",
                    "integration-module-button",
                    "openIntegrationModulePanel(",
                ],
            },
            MainWindowElementMapEntry {
                region: "ignored-addresses",
                name: "panel shell and list",
                desktop_tokens: &[
                    "ui::entity::IGNORED_ADDRESSES_PANEL",
                    "ignored-addresses-section",
                ],
                browser_tokens: &[
                    "id=\"ignored-addresses-panel-title\"",
                    "id=\"ignored-addresses-panel-help\"",
                    "id=\"ignored-addresses\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "observations",
                name: "panel shell",
                desktop_tokens: &[
                    "ui::entity::OBSERVATIONS_PANEL",
                    "ui::id::OBSERVATIONS_PANEL",
                    "ui::entity::OBSERVATIONS_TABLE",
                ],
                browser_tokens: &[
                    "id=\"observations-title\"",
                    "id=\"observations-help\"",
                    "id=\"observations-count\"",
                    "id=\"observations\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "footer",
                name: "status panels",
                desktop_tokens: &[
                    "ui::entity::APP_FOOTER_APPS_PANEL",
                    "ui::entity::APP_FOOTER_WATCHER_STATUS",
                    "ui::entity::APP_FOOTER_TOOL_STATUS",
                    "ui::entity::APP_FOOTER_WEB_SERVER_STATUS",
                    "ui::entity::APP_FOOTER_NETWORK_STATUS",
                ],
                browser_tokens: &[
                    "id=\"footer-apps\"",
                    "id=\"footer-watcher-status\"",
                    "id=\"footer-tool-status\"",
                    "id=\"footer-web-server-status\"",
                    "id=\"footer-network-status\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "footer",
                name: "message and language panels",
                desktop_tokens: &[
                    "ui::entity::APP_FOOTER_MESSAGE_PANEL",
                    "ui::entity::APP_FOOTER_LANGUAGE_PANEL",
                    "ui::entity::APP_FOOTER_LANGUAGE_SELECT",
                ],
                browser_tokens: &[
                    "id=\"footer-message-panel\"",
                    "id=\"footer-message-text\"",
                    "id=\"language-select\"",
                    "id=\"language-select-menu\"",
                ],
            },
        ];

        assert_main_window_element_map(&entries);
    }

    #[test]
    fn browser_main_window_element_map_keeps_table_and_dialog_regions_in_sync() {
        let entries = [
            MainWindowElementMapEntry {
                region: "observations-table",
                name: "sortable headers",
                desktop_tokens: &[
                    "SortHeaderCell { label: table_app.clone()",
                    "SortHeaderCell { label: table_ip.clone()",
                    "SortHeaderCell { label: table_domain.clone()",
                    "SortHeaderCell { label: table_port.clone()",
                    "SortHeaderCell { label: table_proto.clone()",
                    "SortHeaderCell { label: table_conn.clone()",
                    "SortHeaderCell { label: table_hits.clone()",
                    "SortHeaderCell { label: table_first_seen.clone()",
                    "SortHeaderCell { label: table_last_seen.clone()",
                    "th { \"data-tooltip\": \"{table_action}\"",
                ],
                browser_tokens: &[
                    "id=\"table-app\"",
                    "id=\"table-ip\"",
                    "id=\"table-domain\"",
                    "id=\"table-port\"",
                    "id=\"table-proto\"",
                    "id=\"table-conn\"",
                    "id=\"table-hits\"",
                    "id=\"table-first-seen\"",
                    "id=\"table-last-seen\"",
                    "id=\"table-action\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "observations-table",
                name: "row action buttons",
                desktop_tokens: &[
                    "ui::action::TOGGLE_OBSERVATION_CONFIRMED",
                    "ui::action::IGNORE_ADDRESS",
                    "ui::action::DELETE_OBSERVATION",
                ],
                browser_tokens: &[
                    "confirmObservation(",
                    "ignoreAddress(",
                    "data-ui-action=\"delete-observation\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "dialogs",
                name: "integration module menu modal",
                desktop_tokens: &[
                    "ui::entity::INTEGRATION_MODULE_DIALOG",
                    "ui::id::INTEGRATION_MODULE_DIALOG",
                    "ui::action::CLOSE_INTEGRATION_MODULE",
                ],
                browser_tokens: &[
                    "id=\"integration-module-modal\"",
                    "id=\"integration-module-modal-title\"",
                    "id=\"integration-module-modal-close-button\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "dialogs",
                name: "integration configure modal",
                desktop_tokens: &[
                    "ui::entity::INTEGRATION_ROOT_DIALOG",
                    "ui::id::INTEGRATION_ROOT_DIALOG",
                    "ui::action::USE_INTEGRATION_ROOT",
                ],
                browser_tokens: &[
                    "id=\"integration-modal\"",
                    "id=\"integration-modal-title\"",
                    "id=\"integration-modal-use-button\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "dialogs",
                name: "ignored address delete modal",
                desktop_tokens: &[
                    "ui::entity::DELETE_IGNORED_ADDRESS_DIALOG",
                    "ui::id::DELETE_IGNORED_ADDRESS_DIALOG",
                    "ui::action::CONFIRM_DELETE_IGNORED_ADDRESS",
                ],
                browser_tokens: &[
                    "id=\"ignored-delete-modal\"",
                    "id=\"ignored-delete-modal-title\"",
                    "id=\"ignored-delete-modal-confirm-button\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "dialogs",
                name: "observation delete modal",
                desktop_tokens: &[
                    "ui::entity::DELETE_OBSERVATION_DIALOG",
                    "ui::id::DELETE_OBSERVATION_DIALOG",
                    "ui::action::CONFIRM_DELETE_OBSERVATION",
                ],
                browser_tokens: &[
                    "id=\"observation-delete-modal\"",
                    "id=\"observation-delete-modal-title\"",
                    "id=\"observation-delete-modal-confirm-button\"",
                ],
            },
            MainWindowElementMapEntry {
                region: "dialogs",
                name: "clear monitoring modal",
                desktop_tokens: &[
                    "ui::entity::CLEAR_MONITORING_DIALOG",
                    "ui::id::CLEAR_MONITORING_DIALOG",
                    "ui::action::CONFIRM_CLEAR_MONITORING",
                ],
                browser_tokens: &[
                    "id=\"clear-monitoring-modal\"",
                    "id=\"clear-monitoring-modal-title\"",
                    "id=\"clear-monitoring-modal-confirm-button\"",
                ],
            },
        ];

        assert_main_window_element_map(&entries);
    }

    #[test]
    fn browser_monitoring_default_sort_uses_latest_last_seen_first() {
        let source = include_str!("lib.rs").replace('\r', "");
        assert!(
            source.contains("observationSortKey: 'last_seen'"),
            "browser monitoring table should initially sort by Last seen"
        );
        assert!(
            source.contains("const key = state.observationSortKey || 'last_seen';"),
            "browser monitoring table should fall back to Last seen after sort reset"
        );
        assert!(
            source.contains("observationSortDescending: true"),
            "browser monitoring default sort should show newest rows first"
        );
    }

    #[test]
    fn browser_main_window_element_map_covers_every_primary_region() {
        let entries = [
            ("header", 11usize),
            ("tracked-apps", 6usize),
            ("integration", 2usize),
            ("ignored-addresses", 1usize),
            ("observations", 1usize),
            ("observations-table", 2usize),
            ("footer", 2usize),
            ("dialogs", 5usize),
        ];
        let mut actual = BTreeMap::new();
        for region in [
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "header",
            "tracked-apps",
            "tracked-apps",
            "tracked-apps",
            "tracked-apps",
            "tracked-apps",
            "tracked-apps",
            "integration",
            "integration",
            "ignored-addresses",
            "observations",
            "observations-table",
            "observations-table",
            "footer",
            "footer",
            "dialogs",
            "dialogs",
            "dialogs",
            "dialogs",
            "dialogs",
        ] {
            *actual.entry(region).or_insert(0usize) += 1;
        }

        let expected = entries.into_iter().collect::<BTreeMap<_, _>>();
        assert_eq!(
            actual, expected,
            "main window element map should keep stable coverage for each primary region"
        );
    }

    #[test]
    fn browser_main_window_keeps_header_controls_in_desktop_order_without_extra_actions() {
        let ordered_tokens = [
            "id=\"toggle-monitoring-button\"",
            "class=\"header-action-separator\"",
            "id=\"app-filter\"",
            "id=\"ip-filter\"",
            "id=\"port-filter\"",
            "id=\"protocol-filter\"",
            "id=\"observation-filter\"",
            "id=\"confirm-filtered-button\"",
            "id=\"unconfirm-filtered-button\"",
            "id=\"clear-monitoring-button\"",
        ];

        let mut previous = 0usize;
        for (index, token) in ordered_tokens.into_iter().enumerate() {
            let current = token_index(BROWSER_UI_HTML, token);
            if index > 0 {
                assert!(
                    current > previous,
                    "browser header control order drifted around token {token}"
                );
            }
            previous = current;
        }

        assert!(
            BROWSER_UI_HTML.contains("localizedHeaderWebAccessLabel()"),
            "browser header should keep a dedicated web-server label renderer"
        );
        assert!(
            BROWSER_UI_HTML.contains(
                "return '<span class=\"header-switch-row header-switch-row--server\"><span class=\"header-switch-row__label\">'"
            ),
            "browser header should keep the web-server switch rendered as a trailing header toggle block"
        );
        assert!(
            BROWSER_UI_HTML.contains(
                "<header id=\"browser-header\">\n      <div class=\"hero-labels\" id=\"main-header-content\">"
            ),
            "browser header should mirror the desktop hero-labels wrapper"
        );
        for module_header_token in [
            "id=\"module-header-content\"",
            "id=\"module-header-close-button\"",
            "class=\"button__icon button__icon--module-close\"",
            "src=\"/v1/assets/module-close.svg\"",
            "data-ui-action=\"close-module-overlays\"",
            "id=\"module-header-stop-button\"",
            "class=\"button__icon button__icon--module-stop\"",
            "src=\"/v1/assets/module-stop.svg\"",
            "data-ui-action=\"stop-module-background\"",
            "id=\"module-header-actions\"",
            "button__icon--module-action",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(module_header_token),
                "browser header should expose the generic module header token {module_header_token}"
            );
        }

        for removed in [
            "id=\"refresh-button\"",
            "Проверить",
            "Check</button>",
            "id=\"module-header-back-button\"",
            "data-ui-action=\"back-module-overlay\"",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(removed),
                "browser header should not keep extra desktop-divergent action {removed}"
            );
        }
    }

    #[test]
    fn browser_main_window_clear_buttons_stay_nested_inside_clearable_field_shells() {
        for shell in [
            "<div class=\"path-input-shell header-filter-field-shell\">\n            <input class=\"input-box input header-search-field\" id=\"ip-filter\"",
            "<div class=\"path-input-shell\">\n                  <input class=\"input-box input\" id=\"exe-path\"",
            "<div class=\"path-input-shell\">\n              <input class=\"input-box input\" id=\"integration-dir\"",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(shell),
                "browser clearable control shell should keep nested field markup: {shell}"
            );
        }

        for action in [
            "id=\"clear-ip-filter-button\"",
            "id=\"clear-port-filter-button\"",
            "id=\"clear-exe-path-button\"",
            "id=\"clear-integration-folder-button\"",
            "class=\"path-input-clear__glyph\" aria-hidden=\"true\">\u{00d7}</span>",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(action),
                "browser clear button should exist inside its clearable field shell: {action}"
            );
        }

        for contract in [
            "data-clear-target=\"ip-filter\"",
            "data-clear-target=\"domain-filter\"",
            "data-clear-target=\"port-filter\"",
            "data-clear-target=\"exe-path\"",
            "data-clear-target=\"integration-dir\"",
            "data-clear-target=\"profile-export-attach-profile-input\"",
            "data-clear-target=\"profile-export-patch-profile-input\"",
            "data-clear-target=\"profile-export-merge-profile-input\"",
            "data-clear-target=\"profile-export-advanced-manual-domains-input\"",
            "data-clear-target=\"server-file-picker-path\"",
            "data-clear-button=\"true\"",
            "function syncClearButtonStates()",
            "document.querySelectorAll('[data-clear-target][data-clear-button=\"true\"]')",
            "target.dataset.clearButton !== 'true'",
            "button.disabled = value.length === 0;",
            "event.target.dataset?.clearButton === 'true'",
            "document.addEventListener('input', (event) =>",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(contract),
                "browser clear buttons should keep disabled-state contract token {contract}"
            );
        }

        for token in [
            ".path-input-shell {\n      position: relative;",
            "grid-template-columns: minmax(0, 1fr) 20px;",
            ".path-input-shell > .input {\n      grid-column: 1 / -1;\n      grid-row: 1;",
            ".path-input-shell > .header-search-field {\n      grid-column: 1 / -1;\n      grid-row: 1;",
            ".path-input-clear {\n      grid-column: 2;\n      grid-row: 1;",
            "justify-self: end;",
            "margin-right: 5px;",
            "z-index: 1;",
            ".path-input-clear__glyph {\n      display: inline-block;",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser clear-button containment contract should keep token {token}"
            );
        }
    }

    #[test]
    fn browser_shell_title_and_http_responses_expose_runtime_version_and_disable_cache() {
        for token in [
            "<title>NetStitch Web __BUILD_VERSION__ (__RUNTIME_PLATFORM__)</title>",
            ".replace(\"__BUILD_VERSION__\", &runtime_build_version())",
            ".replace(\"__RUNTIME_PLATFORM__\", runtime_platform_label())",
            "CACHE_CONTROL",
            "no-store, no-cache, must-revalidate, max-age=0",
            "(CONTENT_TYPE, content_type)",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token)
                    || WATCHER_MAIN_RS.contains(token)
                    || DESKTOP_APP_RS.contains(token),
                "browser runtime-version/no-cache contract should keep token {token}"
            );
        }
    }

    #[test]
    fn browser_main_window_tracked_apps_toolbar_reuses_desktop_shell_contract() {
        for token in [
            "class=\"card tracked-apps-card\" id=\"tracked-apps-panel\"",
            "class=\"card__header tracked-apps-header\"",
            "<div class=\"stack\">",
            "<div class=\"exe-path-row\">",
            "<input class=\"input-box input\" id=\"exe-path\" placeholder=\"Path\">",
            "<div class=\"tracked-apps-toolbar-spacer\" aria-hidden=\"true\"></div>",
            "<div class=\"tracked-apps-bulk-row\">",
            "class=\"tracked-apps-bulk-label\" id=\"enable-all-label\"",
            "class=\"tracked-apps-count-label\" id=\"tracked-count-inline\"",
            ".button__plus {\n      display: inline-flex;\n      align-items: center;\n      justify-content: center;",
            ".tracked-apps-card .stack {\n      display: grid;",
            ".tracked-apps-card > .stack {\n      grid-template-rows: auto minmax(0, 1fr) auto;",
            ".exe-path-row {\n      display: grid;\n      grid-template-columns: minmax(0, 1fr) var(--size-compact-control);",
            ".tracked-apps-bulk-row {\n      display: grid;\n      grid-template-columns: auto var(--size-switch-width) minmax(0, 1fr);",
            ".tracked-apps-bulk-label,\n    .tracked-apps-count-label {",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser tracked-apps toolbar should keep desktop token {token}"
            );
        }

        for obsolete in [
            "class=\"card observations-card\">\n            <div class=\"card__header\">\n              <div class=\"title-with-help\">\n                <h2 id=\"tracked-apps-title\"",
            "class=\"toolbar\"",
            "class=\"form-row\"",
            "class=\"bulk-row\"",
            "header-toggle__label",
            "header-server-switch",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser tracked-apps/header markup should not keep obsolete token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_main_window_primary_region_shells_reuse_desktop_card_contracts() {
        for token in [
            "id=\"tracked-apps-panel\"",
            "class=\"card tracked-apps-card\"",
            "class=\"card__header tracked-apps-header\"",
            "id=\"tracked-apps-title\"",
            "class=\"card modules-card\" id=\"integration-panel\"",
            "id=\"integration-title\"",
            "class=\"card ignored-addresses-card\"",
            "id=\"ignored-addresses-panel-title\"",
            "class=\"card observations-card\" id=\"browser-observations-panel\"",
            "id=\"observations-title\"",
            "class=\"card__header\"",
            "class=\"title-with-help\"",
            "class=\"panel-header-meta\"",
            "class=\"table-wrap\"",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser primary region shell should keep desktop card token {token}"
            );
        }

        for obsolete in [
            "tracked-panel",
            "monitor-panel",
            "ignored-panel",
            "panel-title-row",
            "summary-card",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser shell should not keep non-desktop primary region token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_main_window_header_filters_reuse_desktop_block_contracts() {
        for token in [
            "class=\"header-filter-block header-filter-block--app\"",
            "class=\"header-filter-block header-filter-block--ip\"",
            "class=\"header-filter-block header-filter-block--domain\"",
            "class=\"header-filter-block header-filter-block--port\"",
            "class=\"header-filter-block header-filter-block--protocol\"",
            "class=\"header-filter-block header-filter-block--state\"",
            "class=\"input-box select header-filter-select\" id=\"app-filter\"",
            "class=\"input-box input header-search-field\" id=\"ip-filter\"",
            "class=\"input-box input header-search-field header-search-field--domain\" id=\"domain-filter\"",
            "data-tooltip=\"Domain filter. Use * as any number of characters, for example *example.com or example*.com.\"",
            "const domainTooltip = t('filter.domain_tooltip'",
            "function domainFilterMatches(value, pattern) {",
            "function wildcardPatternMatches(value, pattern) {",
            "onkeydown=\"addTrackedAppOnEnter(event)\"",
            "async function addTrackedAppOnEnter(event) {",
            "class=\"input-box input header-search-field\" id=\"port-filter\"",
            "class=\"input-box select header-filter-select\" id=\"protocol-filter\"",
            "class=\"input-box select header-filter-select\" id=\"observation-filter\"",
            "class=\"header-filter-block__label\" id=\"app-filter-label\"",
            "class=\"header-filter-block__label\" id=\"ip-filter-label\"",
            "class=\"header-filter-block__label\" id=\"domain-filter-label\"",
            "class=\"header-filter-block__label\" id=\"port-filter-label\"",
            "class=\"header-filter-block__label\" id=\"protocol-filter-label\"",
            "class=\"header-filter-block__label\" id=\"observation-filter-label\"",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser header filters should reuse desktop block token {token}"
            );
        }

        for obsolete in ["header-filter-group", "header-filter-label"] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser header filters should not keep obsolete token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_main_window_switches_reuse_desktop_knob_markup() {
        for token in [
            "<button id=\"enable-all-switch\" class=\"input-box switch\" type=\"button\" onclick=\"toggleEnableAll()\" aria-label=\"Enable all\"><span class=\"switch__knob\"></span></button>",
            "<div class=\"header-switch-stack\" id=\"header-labels\"></div>",
            ".header-switch-stack {\n      display: flex;",
            ".header-switch-stack .header-switch-row {\n      min-height: var(--size-switch-height);",
            "return '<span class=\"header-switch-row header-switch-row--server\"><span class=\"header-switch-row__label\">'",
            "<button id=\"web-server-switch\" class=\"input-box switch ' + (enabled ? 'switch--on' : '') + '\" type=\"button\" onclick=\"toggleWebAccess()\" aria-label=\"'",
            "<button id=\"domain-capture-switch\" class=\"input-box switch ' + (enabled ? 'switch--on' : '') + '\" type=\"button\" onclick=\"toggleDomainCapture()\" aria-label=\"'",
            "<span class=\"switch__knob\"></span></button></span>';",
            "document.getElementById('enable-all-switch').className = 'input-box switch' + (enableAll ? ' switch--on' : '');",
            "<button class=\"input-box switch ' + (app.enabled ? 'switch--on' : '') + '\" type=\"button\" aria-label=\"Toggle tracked app\"",
            "<span class=\"switch__knob\"></span></button></div>'",
            ".app-actions {\n      display: grid;\n      grid-template-columns: 1fr;\n      grid-template-rows: 20px 18px;",
            ".app-actions .switch {\n      grid-row: 2;",
            ".switch__knob {",
            ".switch--on .switch__knob { transform: translateX(24px); background: #eeeeee; }",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser switches should keep desktop knob token {token}"
            );
        }

        for obsolete in [".switch::before {", ".switch--on::before {"] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser switches should not keep obsolete pseudo-knob token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_cloud_export_panel_keeps_desktop_auth_and_panel_header_contract() {
        for token in [
            "class=\"panel-header-meta cloud-web-panel-service\" id=\"cloud-import-panel-status\"",
            "class=\"panel-header-meta cloud-web-panel-service\" id=\"cloud-export-panel-status\"",
            "class=\"cloud-web-panel-service__text\"",
            "class=\"footer-watcher-dot footer-watcher-dot--connected\"",
            "class=\"cloud-web-panel-body cloud-web-export-body\"",
            "id=\"cloud-export-auth-button\"",
            "id=\"cloud-export-sign-out-button\"",
            "id=\"cloud-export-auth-user\"",
            "id=\"cloud-export-nickname\"",
            "id=\"cloud-export-private-upload\"",
            "id=\"cloud-export-private-text\"",
            "id=\"cloud-export-nickname-status\"",
            "class=\"card cloud-web-panel cloud-web-panel--export\"",
            ".card.cloud-web-panel {",
            "id=\"cloud-export-publications-rows\"",
            "publicationSortKey: 'new_rows'",
            "publicationSortDescending: true",
            "id=\"cloud-export-publications-new-rows-sort-icon\" class=\"table-sortable__icon table-sortable__icon--desc\" src=\"/v1/assets/sort-desc.svg\"",
            ".cloud-web-publications-data-table .state-label--success {\n      color: var(--success);",
            "'<span class=\"state-label state-label--success\">' + html(text(newRows)) + '</span>'",
            "api('/v1/cloud/auth-state')",
            "api('/v1/cloud/my-apps')",
            "post('/v1/cloud/nickname'",
            "post('/v1/cloud/sign-out'",
            "visibility: state.cloud.uploadPrivate ? 'private' : 'public'",
            ".route(\"/v1/cloud/auth-state\", get(cloud_auth_state))",
            ".route(\"/v1/cloud/my-apps\", get(cloud_my_apps))",
            ".route(\"/v1/cloud/nickname\", post(set_cloud_nickname))",
            ".route(\"/v1/cloud/sign-out\", post(cloud_sign_out))",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token) || WATCHER_MAIN_RS.contains(token),
                "browser cloud export should keep desktop-like auth/header token {token}"
            );
        }
    }

    #[test]
    fn browser_main_window_footer_regions_keep_desktop_panel_contracts() {
        for token in [
            "class=\"app-footer-panel\" id=\"footer-apps\"",
            "class=\"app-footer-panel footer-watcher-status\"",
            "class=\"app-footer-panel footer-tool-status\"",
            "class=\"app-footer-panel footer-web-server-status\"",
            "class=\"app-footer-panel footer-network-status\"",
            "class=\"app-footer-panel app-footer-message-panel\" id=\"footer-message-panel\"",
            "class=\"input-box button button--square button--copy-status\" id=\"footer-status-copy\"",
            "class=\"footer-message-text path-field\"",
            "class=\"app-footer-panel app-footer-actions app-footer-language-panel\"",
            "class=\"language-select-shell\"",
            "class=\"language-select-control input-box\" id=\"language-select\"",
            "class=\"language-select-control__label\" id=\"language-select-label\"",
            "id=\"language-select-menu\"",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser footer should keep shared panel token {token}"
            );
        }

        assert!(
            !BROWSER_UI_HTML.contains("class=\"app-footer-panel\" id=\"footer-lines\""),
            "browser footer should not keep the removed lines counter in the main app footer"
        );
        assert!(
            BROWSER_UI_HTML.contains("class=\"monitoring-header-meta\"")
                && BROWSER_UI_HTML.contains("id=\"public-ip-filter\"")
                && !BROWSER_UI_HTML.contains("id=\"table-ip-public\""),
            "browser monitoring header should expose the public IP switch without a table column"
        );
        assert!(
            BROWSER_UI_HTML.contains(
                "class=\"panel-footer monitoring-panel-footer\" data-ui-entity=\"panel-footer\""
            ) && BROWSER_UI_HTML.contains("id=\"observations-count\"")
                && BROWSER_UI_HTML
                    .contains("<span class=\"panel-footer-meta__item\">Total rows: 0</span>")
                && BROWSER_UI_HTML
                    .contains("<span class=\"panel-footer-meta__item\">Displayed rows: 0</span>")
                && BROWSER_UI_HTML
                    .contains("<span class=\"panel-footer-meta__item\">Selected rows: 0</span>"),
            "browser monitoring counters should live in the panel footer"
        );
        assert!(
            BROWSER_UI_HTML.contains("id=\"cloud-import-row-summary\"")
                && BROWSER_UI_HTML
                    .contains("<span class=\"panel-footer-meta__item\">Total rows: 0</span>")
                && BROWSER_UI_HTML
                    .contains("<span class=\"panel-footer-meta__item\">Selected rows: 0</span>")
                && BROWSER_UI_HTML.contains("<span class=\"panel-footer-meta__item\" id=\"cloud-import-quota\">Download: 0/0</span>"),
            "browser cloud import footer should expose row and selection counters"
        );

        for obsolete in [
            "footer-watcher-state",
            "status-line",
            "footer-message-text__value",
            "language-dropdown",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(obsolete),
                "browser footer should not keep obsolete token {obsolete}"
            );
        }
    }

    #[test]
    fn browser_main_window_help_icons_keep_desktop_circle_geometry() {
        for (desktop_token, browser_tokens) in [
            (
                ".help-icon {\n  display: inline-grid;\n  place-items: center;\n  width: 18px;\n  height: 18px;\n  border-radius: var(--radius-pill);",
                &[
                    ".help-icon {",
                    "display: inline-grid;",
                    "place-items: center;",
                    "width: 18px;",
                    "height: 18px;",
                    "border-radius: var(--radius-pill);",
                ][..],
            ),
            (
                "background: var(--color-control-bg);",
                &["background: var(--control);"][..],
            ),
            (
                "color: var(--color-text-muted);",
                &["color: var(--muted);"][..],
            ),
            ("line-height: 16px;", &["line-height: 16px;"][..]),
        ] {
            assert!(
                DESKTOP_THEME_RS.contains(desktop_token),
                "desktop theme should keep help icon token {desktop_token}"
            );
            for browser_token in browser_tokens {
                assert!(
                    BROWSER_UI_HTML.contains(browser_token),
                    "browser help icon should mirror desktop circle geometry token {browser_token}"
                );
            }
        }
    }

    #[test]
    fn browser_main_window_keeps_footer_and_observations_contained_inside_shell() {
        for token in [
            ".shell {\n      position: fixed;\n      inset: 0;\n      display: grid;\n      grid-template-rows: var(--size-header-height) minmax(0, 1fr) var(--size-footer-height);",
            ".content {\n      display: flex;\n      flex: 1 1 auto;\n      flex-direction: column;\n      width: 100%;\n      height: 100%;\n      min-height: 0;\n      min-width: 0;\n      overflow: hidden;",
            ".workspace > .observations-card {\n      grid-column: 1 / -1;\n      grid-row: 2;\n      align-self: stretch;\n      height: 100%;\n      min-height: 0;",
            ".observations-card { display: flex; flex-direction: column; min-height: 0; height: 100%; max-height: none; }",
            ".observations-card .table-wrap { flex: 1 1 auto; min-height: 0; max-height: none; height: 100%; }",
            ".table-wrap { height: 100%; max-height: none; overflow: auto;",
            "footer { justify-content: flex-start; min-height: var(--size-footer-height); height: var(--size-footer-height);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(token),
                "browser shell should keep footer/table containment token {token}"
            );
        }
    }

    #[test]
    fn browser_main_window_primary_panels_keep_desktop_markup_order() {
        let ordered_tokens = [
            "id=\"tracked-apps-title\"",
            "id=\"integration-help\"",
            "id=\"ignored-addresses-panel-title\"",
            "id=\"observations-title\"",
            "id=\"footer-apps\"",
        ];

        let mut previous = 0usize;
        for (index, token) in ordered_tokens.into_iter().enumerate() {
            let current = token_index(BROWSER_UI_HTML, token);
            if index > 0 {
                assert!(
                    current > previous,
                    "browser primary panel order drifted around token {token}"
                );
            }
            previous = current;
        }
    }

    fn css_rule_block<'a>(source: &'a str, selector: &str) -> &'a str {
        let start = source
            .find(selector)
            .unwrap_or_else(|| panic!("missing css selector: {selector}"));
        let rule = &source[start..];
        let brace = rule
            .find('{')
            .unwrap_or_else(|| panic!("selector has no opening brace: {selector}"));
        let body = &rule[brace + 1..];
        let end = body
            .find('}')
            .unwrap_or_else(|| panic!("selector has no closing brace: {selector}"));
        &body[..end]
    }

    fn css_property_value<'a>(block: &'a str, property: &str) -> &'a str {
        let needle = format!("{property}:");
        let start = block
            .find(&needle)
            .unwrap_or_else(|| panic!("missing css property {property} in block:\n{block}"));
        let value = &block[start + needle.len()..];
        value
            .split(';')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| panic!("css property {property} has empty value in block:\n{block}"))
    }

    #[test]
    fn browser_and_desktop_share_header_dimension_property_map() {
        for (desktop_selector, browser_selector, property) in [
            (
                ".header-monitor-controls",
                ".header-monitor-controls",
                "display",
            ),
            (
                ".header-monitor-controls",
                ".header-monitor-controls",
                "flex-wrap",
            ),
            (
                ".header-monitor-controls",
                ".header-monitor-controls",
                "align-items",
            ),
            (
                ".header-monitor-controls",
                ".header-monitor-controls",
                "gap",
            ),
            (
                "\n.header-filter-block {\n",
                "\n    .header-filter-block {\n",
                "height",
            ),
            (
                ".header-filter-block__label",
                ".header-filter-block__label",
                "font-size",
            ),
            (
                ".header-filter-block__label",
                ".header-filter-block__label",
                "font-weight",
            ),
            (
                ".header-filter-select,\n.header-search-field",
                ".header-filter-select,\n    .header-filter-field,\n    .header-search-field",
                "height",
            ),
            (
                ".header-filter-select,\n.header-search-field",
                ".header-filter-select,\n    .header-filter-field,\n    .header-search-field",
                "min-height",
            ),
            (
                ".header-filter-select,\n.header-search-field",
                ".header-filter-select,\n    .header-filter-field,\n    .header-search-field",
                "font-size",
            ),
            (
                ".header-filter-select,\n.header-search-field",
                ".header-filter-select,\n    .header-filter-field,\n    .header-search-field",
                "line-height",
            ),
            (
                ".button--icon.header-action-button",
                ".button--icon.header-action-button",
                "width",
            ),
            (
                ".button--icon.header-action-button",
                ".button--icon.header-action-button",
                "height",
            ),
            (".header-switch-row", ".header-switch-row", "gap"),
            (
                ".header-switch-row__label",
                ".header-switch-row__label",
                "font-size",
            ),
            (
                ".header-switch-row__label",
                ".header-switch-row__label",
                "font-weight",
            ),
            (
                ".card__header {\n  display: flex;",
                ".card__header {\n      display: flex;",
                "gap",
            ),
            (
                ".card__header {\n  display: flex;",
                ".card__header {\n      display: flex;",
                "border-radius",
            ),
        ] {
            let desktop_block = css_rule_block(DESKTOP_THEME_RS, desktop_selector);
            let browser_block = css_rule_block(BROWSER_UI_HTML, browser_selector);
            assert_eq!(
                css_property_value(desktop_block, property),
                css_property_value(browser_block, property),
                "browser css property map drifted for selector pair {desktop_selector} -> {browser_selector} property {property}"
            );
        }
    }

    #[test]
    fn browser_and_desktop_share_header_filter_width_map() {
        for (desktop_selector, browser_selector, widths) in [
            (
                ".header-monitor-controls .header-filter-select--app,\n.header-monitor-controls .header-filter-block--app",
                ".header-filter-block--app",
                [
                    ("flex", "0 1 170px"),
                    ("min-width", "132px"),
                    ("max-width", "170px"),
                ]
                .as_slice(),
            ),
            (
                ".header-monitor-controls .header-filter-block--ip",
                ".header-filter-block--ip",
                [
                    ("flex", "0 1 126px"),
                    ("min-width", "102px"),
                    ("max-width", "126px"),
                ]
                .as_slice(),
            ),
            (
                ".header-monitor-controls .header-filter-block--domain",
                ".header-filter-block--domain",
                [
                    ("flex", "0 1 126px"),
                    ("min-width", "102px"),
                    ("max-width", "126px"),
                ]
                .as_slice(),
            ),
            (
                ".header-monitor-controls .header-filter-select--port,\n.header-monitor-controls .header-filter-block--port",
                ".header-filter-block--port",
                [
                    ("flex", "0 1 66px"),
                    ("min-width", "66px"),
                    ("max-width", "66px"),
                ]
                .as_slice(),
            ),
            (
                ".header-monitor-controls .header-filter-select--protocol,\n.header-monitor-controls .header-filter-block--protocol",
                ".header-filter-block--protocol",
                [
                    ("flex", "0 1 68px"),
                    ("min-width", "64px"),
                    ("max-width", "68px"),
                ]
                .as_slice(),
            ),
            (
                ".header-monitor-controls .header-filter-select--state,\n.header-monitor-controls .header-filter-block--state",
                ".header-filter-block--state",
                [
                    ("flex", "0 1 94px"),
                    ("min-width", "74px"),
                    ("max-width", "94px"),
                ]
                .as_slice(),
            ),
        ] {
            let desktop_block = css_rule_block(DESKTOP_THEME_RS, desktop_selector);
            let browser_block = css_rule_block(BROWSER_UI_HTML, browser_selector);
            for (browser_property, expected_value) in widths {
                assert_eq!(
                    css_property_value(browser_block, browser_property),
                    *expected_value,
                    "browser header width map drifted for {browser_selector} property {browser_property}"
                );
            }
            if desktop_selector.contains("app") {
                assert_eq!(
                    css_property_value(desktop_block, "width"),
                    "min(170px, 16vw)"
                );
            }
            if desktop_selector.contains("ip") {
                assert_eq!(
                    css_property_value(desktop_block, "width"),
                    "min(126px, 10vw)"
                );
            }
            if desktop_selector.contains("domain") {
                assert_eq!(
                    css_property_value(desktop_block, "width"),
                    "min(126px, 10vw)"
                );
            }
            if desktop_selector.contains("port") {
                assert_eq!(css_property_value(desktop_block, "width"), "66px");
            }
            if desktop_selector.contains("protocol") {
                assert_eq!(css_property_value(desktop_block, "width"), "min(68px, 7vw)");
            }
            if desktop_selector.contains("state") {
                assert_eq!(css_property_value(desktop_block, "width"), "min(94px, 9vw)");
            }
        }
    }

    #[test]
    fn browser_and_desktop_share_monitoring_button_icon_geometry() {
        for (desktop_selector, browser_selector, property, expected) in [
            (
                ".button__icon-shell--monitoring,\n.button__icon-scale--monitoring,\n.button__icon-rotor--monitoring,\n.button__icon--monitoring",
                ".button__icon-shell--monitoring,\n    .button__icon-scale--monitoring,\n    .button__icon-rotor--monitoring,\n    .button__icon--monitoring",
                "width",
                "24px",
            ),
            (
                ".button__icon-shell--monitoring,\n.button__icon-scale--monitoring,\n.button__icon-rotor--monitoring,\n.button__icon--monitoring",
                ".button__icon-shell--monitoring,\n    .button__icon-scale--monitoring,\n    .button__icon-rotor--monitoring,\n    .button__icon--monitoring",
                "height",
                "24px",
            ),
            (
                ".button__icon--import-csv",
                ".button__icon--confirm-filtered,\n    .button__icon--unconfirm-filtered,\n    .button__icon--trash,\n    .button__icon--cloud-import,\n    .button__icon--cloud-sync,\n    .button__icon--import-csv,\n    .button__icon--export-csv",
                "width",
                "28px",
            ),
            (
                ".button__icon--export-csv",
                ".button__icon--confirm-filtered,\n    .button__icon--unconfirm-filtered,\n    .button__icon--trash,\n    .button__icon--cloud-import,\n    .button__icon--cloud-sync,\n    .button__icon--import-csv,\n    .button__icon--export-csv",
                "height",
                "28px",
            ),
            (".table-sortable__icon", ".table-sortable__icon", "top", "0"),
            (
                ".table-sortable__content",
                ".table-sortable__content",
                "gap",
                "3px",
            ),
            (
                ".table-sortable__content",
                ".table-sortable__content",
                "transform",
                "translateX(-8px)",
            ),
        ] {
            let desktop_block = css_rule_block(DESKTOP_THEME_RS, desktop_selector);
            let browser_block = css_rule_block(BROWSER_UI_HTML, browser_selector);
            assert_eq!(css_property_value(desktop_block, property), expected);
            assert_eq!(css_property_value(browser_block, property), expected);
        }
        for (desktop_selector, browser_selector, property, expected) in [
            (
                "th:nth-child(2), td:nth-child(2)",
                ".observations-card > .table-wrap > table th:nth-child(2),\n    .observations-card > .table-wrap > table td:nth-child(2)",
                "width",
                "138px",
            ),
            (
                "th:nth-child(4), td:nth-child(4)",
                ".observations-card > .table-wrap > table th:nth-child(4),\n    .observations-card > .table-wrap > table td:nth-child(4)",
                "width",
                "64px",
            ),
            (
                "th:nth-child(5), td:nth-child(5)",
                ".observations-card > .table-wrap > table th:nth-child(5),\n    .observations-card > .table-wrap > table td:nth-child(5)",
                "width",
                "66px",
            ),
            (
                "th:nth-child(6), td:nth-child(6)",
                ".observations-card > .table-wrap > table th:nth-child(6),\n    .observations-card > .table-wrap > table td:nth-child(6)",
                "width",
                "128px",
            ),
            (
                "th:nth-child(7), td:nth-child(7)",
                ".observations-card > .table-wrap > table th:nth-child(7),\n    .observations-card > .table-wrap > table td:nth-child(7)",
                "width",
                "52px",
            ),
        ] {
            let desktop_block = css_rule_block(DESKTOP_THEME_RS, desktop_selector);
            let browser_block = css_rule_block(BROWSER_UI_HTML, browser_selector);
            assert_eq!(css_property_value(desktop_block, property), expected);
            assert_eq!(css_property_value(browser_block, property), expected);
        }
        assert!(
            !BROWSER_UI_HTML.contains("th:nth-child(3), td:nth-child(3) { width: 58px;"),
            "browser monitoring table must not keep the obsolete public-column width map"
        );
    }

    #[test]
    fn browser_background_snapshot_poll_keeps_monitoring_state_in_sync() {
        for expected in [
            "snapshotLoadInFlight: false,",
            "backgroundSnapshotTimer: null,",
            "function snapshotSectionSignatures(snapshot) {",
            "function snapshotSyncDelta(previousSnapshot, snapshot) {",
            "function applySnapshotSyncDelta(delta, snapshot) {",
            "function renderBackgroundSnapshotSync(previousSnapshot, snapshot) {",
            "trackedApps: stableJson(snapshot?.tracked_apps),",
            "filters: stableJson(snapshot?.filters),",
            "observations: stableJson(snapshot?.observed_endpoints),",
            "headerChanged:",
            "observationsChanged:",
            "previous.filters !== next.filters",
            "renderTrackedApps(snapshot);",
            "renderObservations(snapshot);",
            "renderIntegration(snapshot);",
            "async function refreshSnapshotInBackground() {",
            "if (state.browserAccessDisabledOverlay || document.hidden) {",
            "scheduleBackgroundSnapshotRefresh(state.watcherConnected ? 1200 : 4000);",
            "await loadSnapshot({ background: true });",
            "scheduleBackgroundSnapshotRefresh(1200);",
            "scheduleBackgroundSnapshotRefresh(4000);",
            "function scheduleBackgroundSnapshotRefresh(delayMs) {",
            "window.setTimeout(refreshSnapshotInBackground, delayMs);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser monitoring parity loop should include {expected}"
            );
        }
    }

    #[test]
    fn browser_cloud_uses_only_local_watcher_routes() {
        for expected in [
            "api('/v1/cloud/apps?'",
            "api('/v1/cloud/quota')",
            "api('/v1/cloud/my-apps')",
            "api('/v1/cloud/observations?'",
            "post('/v1/cloud/upload'",
            ".route(\"/v1/cloud/apps\", get(cloud_apps))",
            ".route(\"/v1/cloud/observations\", get(cloud_observations))",
            ".route(\"/v1/cloud/quota\", get(cloud_quota))",
            ".route(\"/v1/cloud/local-author\", get(cloud_local_author))",
            ".route(\"/v1/cloud/my-apps\", get(cloud_my_apps))",
            ".route(\"/v1/cloud/upload\", post(cloud_upload))",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected) || WATCHER_MAIN_RS.contains(expected),
                "browser cloud/native watcher contract should include {expected}"
            );
        }
        for forbidden in [
            "fetch('https://",
            "fetch(\"https://",
            "netstitch-sync",
            "workers.dev",
            "cloudflare",
            "XMLHttpRequest",
            "new WebSocket(",
            "EventSource(",
            "sendBeacon(",
            "window.open(",
            "location.href",
            "location.assign(",
            "location.replace(",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(forbidden),
                "browser cloud UI must not use external/browser-owned network path {forbidden}"
            );
        }
    }

    #[test]
    fn browser_cloud_tables_keep_desktop_row_action_and_selection_contract() {
        for expected in [
            ".button__icon--my-publications {\n      width: 30px;\n      height: 30px;",
            "<colgroup>\n                    <col>\n                    <col>\n                    <col class=\"browser-table-col-count\">\n                    <col class=\"browser-table-col-author\">\n                    <col class=\"cloud-web-action-column\">",
            "scopeMine: false,",
            "function setCloudMineFilterActive(active) {",
            "state.cloud.scopeMine = Boolean(active);",
            "const active = !state.cloud.scopeMine;",
            "await loadCloudMyApps();",
            "const mineActive = Boolean(state.cloud.scopeMine);",
            "if ([...params.keys()].length === 0 && mineActive) {",
            "if (!mineActive && appQuery.length < 2 && publisherQuery.length < 2 && sourceQuery.length < 2) return [];",
            "if (!mineActive && (scope === 'all' || scope === 'public')) {",
            "if (!mineActive && itemVisibility !== 'private') continue;",
            "if (state.cloud.scopeMine) params.set('own_scope', 'true');",
            "grid-template-columns: minmax(120px, 30fr) minmax(120px, 30fr) minmax(160px, 38fr) 108px var(--size-header-action-button);",
            "align-self: stretch;",
            "width: 100%;",
            "mineButton.disabled = !mineEnabled;",
            "mineButton.classList.toggle('button--cloud-active', Boolean(state.cloud.scopeMine && mineEnabled));",
            "<col class=\"cloud-web-action-column\">",
            ".cloud-web-subpanel col.cloud-web-action-column {\n      width: 32px;",
            ".cloud-web-subpanel col.browser-table-col-ip {\n      width: 138px;",
            ".cloud-web-subpanel col.browser-table-col-port {\n      width: 64px;",
            ".cloud-web-subpanel col.browser-table-col-protocol {\n      width: 66px;",
            ".cloud-web-subpanel col.browser-table-col-connection {\n      width: 128px;",
            ".cloud-web-subpanel col.browser-table-col-count {\n      width: 52px;",
            ".cloud-web-subpanel col.browser-table-col-author {\n      width: 142px;",
            ".cloud-web-subpanel col.browser-table-col-privacy {\n      width: 64px;",
            "<col class=\"browser-table-col-ip\">\n                    <col>\n                    <col class=\"browser-table-col-port\">\n                    <col class=\"browser-table-col-protocol\">\n                    <col class=\"browser-table-col-connection\">\n                    <col class=\"browser-table-col-count\">\n                    <col class=\"browser-table-col-author\">\n                    <col class=\"browser-table-col-privacy\">\n                    <col class=\"cloud-web-action-column\">",
            ".cloud-web-subpanel tbody td {\n      padding: 2px 10px;\n      line-height: 16px;\n      white-space: nowrap;\n      overflow: hidden;\n      text-overflow: ellipsis;\n      box-sizing: border-box;\n      background: transparent;",
            "<button class=\"input-box button button--icon button--square table-action-button\" type=\"button\" onclick=\"downloadCloudApp(",
            "<img class=\"button__icon button__icon--cloud-import table-action-button__icon\" src=\"' + IMPORT_CLOUD_ICON_SRC + '\" alt=\"\">",
            "function handleCloudRowClick(event, rowId) {",
            "if (!event?.shiftKey && !event?.ctrlKey && !event?.metaKey) return;",
            "event.preventDefault();",
            "event?.shiftKey && state.cloud.lastSelectedRowId",
            "event?.ctrlKey || event?.metaKey",
            "function handleCloudRowAction(event, rowId) {",
            "event?.stopPropagation();",
            "toggleCloudRow(rowId);",
            "<tr' + rowClass + ' onclick=\"handleCloudRowClick(event, ' + rowIdLiteral + ')\">",
            "<button class=\"input-box button button--icon button--square table-action-button\" type=\"button\" onclick=\"handleCloudRowAction(event, ' + rowIdLiteral + ')\"",
            ".cloud-web-subpanel th.cloud-web-action-header,\n    .cloud-web-subpanel td.cloud-web-action-cell {\n      width: 32px;",
            ".cloud-web-subpanel tbody tr.observation-row--confirmed td {\n      background: var(--row-on);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser cloud table should keep desktop action/selection contract token {expected}"
            );
        }
        assert!(
            !BROWSER_UI_HTML.contains(
                "<colgroup>\n                    <col>\n                    <col>\n                    <col>\n                    <col>\n                    <col>\n                    <col class=\"cloud-web-action-column\">"
            ),
            "browser cloud app table must not keep an extra spacer column before the action column"
        );
        assert!(
            !BROWSER_UI_HTML.contains("background: var(--row);\n      border-bottom: 1px solid var(--border);\n      color: var(--text);\n    }\n    .cloud-web-subpanel tbody tr:hover td"),
            "browser cloud table body cells must not force the gray base row background"
        );
    }

    #[test]
    fn browser_cloud_download_does_not_auto_select_imported_rows() {
        let download_pos = BROWSER_UI_HTML
            .find("async function downloadCloudApp(appId, appName, expectedTotal) {")
            .expect("downloadCloudApp");
        let download_source = &BROWSER_UI_HTML[download_pos
            ..BROWSER_UI_HTML[download_pos..]
                .find("function cloudRowMatchesFilters")
                .map(|end| download_pos + end)
                .expect("cloud row filters")];

        assert!(
            download_source.contains("state.cloud.selectedRows = new Set();"),
            "cloud import download should reset selection to empty"
        );
        assert!(
            !download_source.contains("state.cloud.rows.map((row) => text(row.row_id))"),
            "cloud import download must not auto-select downloaded rows"
        );
    }

    #[test]
    fn browser_cloud_runtime_download_uses_paged_observation_loading() {
        let source = include_str!("lib.rs");
        let route_pos = source
            .find("async fn cloud_observations(")
            .expect("cloud_observations route");
        let route_source = &source[route_pos
            ..source[route_pos..]
                .find("fn web_url_dto")
                .map(|end| route_pos + end)
                .expect("web url dto")];

        for expected in [
            "const PAGE_SIZE: usize = 500;",
            "let mut offset = 0usize;",
            "loop {",
            "params.push(format!(\"offset={offset}\"));",
            "if query.own_scope {",
            "author_user_id={}",
            "seen_rows.insert(cloud_observation_dedupe_key(&item.row, is_private))",
            "all_rows.push(CloudCollectedObservationRow",
            "offset = offset.saturating_add(page_len);",
            "added_rows == 0",
        ] {
            assert!(
                route_source.contains(expected),
                "browser cloud route must keep paged loading token {expected}"
            );
        }
        assert!(
            !route_source.contains("response.offset"),
            "browser cloud route must not stop pagination because the Worker omitted offset echo"
        );
        assert!(
            !route_source.contains("format!(\"offset={}\", all_rows.len())"),
            "browser cloud route must not page by unique row count"
        );
        assert!(
            !route_source.contains("offset >= total"),
            "browser cloud route must not treat a reported total of 500 as a hard stop"
        );
    }

    #[tokio::test]
    async fn browser_cloud_runtime_download_fetches_all_http_pages() {
        let (base_url, server) = spawn_cloud_observations_server();
        let query = CloudObservationsQuery {
            app_id: "app".to_string(),
            app_name: Some("Demo App".to_string()),
            expected_total: Some(602),
            ip: None,
            domain: None,
            port: None,
            protocol: None,
            source: None,
            visibility: Some(CloudObservationVisibilityScope::Public),
            own_scope: false,
        };

        let rows = match cloud_observations_rows(&query, &base_url, None).await {
            Ok(rows) => rows,
            Err(error) => panic!(
                "browser cloud route should fetch all pages: {}",
                error.error
            ),
        };
        let requests = server.join().expect("server should finish");

        assert_eq!(rows.len(), 602);
        assert_eq!(rows[0].application, "Demo App");
        assert_eq!(rows[501].application, "Demo App");
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
    fn browser_delete_monitoring_uses_backend_deleted_count() {
        let delete_call =
            "const result = await post('/v1/observations/delete-batch', { endpoint_ids: ids });";
        let delete_index = token_index(BROWSER_UI_HTML, delete_call);
        let after_delete = &BROWSER_UI_HTML[delete_index..];

        assert!(
            after_delete.contains("result?.deleted ?? 0"),
            "browser delete footer must use backend deleted count"
        );
        assert!(
            !after_delete.contains("localizedClearMonitoringLabel() + ': ' + ids.length"),
            "browser delete footer must not report requested ids as deleted"
        );
    }

    #[test]
    fn browser_cloud_import_add_to_monitoring_requires_selection() {
        for expected in [
            "return (state.cloud.rows || []).filter((row) => state.cloud.selectedRows.has(text(row.row_id)));",
            "const rows = selectedCloudRows();",
            "if (addButton) addButton.disabled = !hasSelected;",
            "import_source: 'cloud_download'",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser cloud import should require selected rows token {expected}"
            );
        }
        assert!(
            !BROWSER_UI_HTML.contains(
                "const rows = selectedRows.length ? selectedRows : (state.cloud.rows || []);"
            ),
            "browser cloud import must not fall back to all rows when selection is empty"
        );
    }

    #[test]
    fn browser_cloud_progress_uses_500_row_chunks() {
        for expected in [
            "function cloudProgressStages(totalRows)",
            "Math.ceil(Math.max(1, Number(totalRows) || 1) / 500)",
            "cloudProgressStages(rows)",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser cloud progress should expose 500-row chunk token {expected}"
            );
        }
    }

    fn spawn_cloud_observations_server() -> (String, std::thread::JoinHandle<Vec<String>>) {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("test cloud server bind");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("test cloud server address")
        );
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for stream in listener.incoming().take(2) {
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
                    serde_json::to_string(&cloud_observations_page(0, 500, 500))
                        .expect("first cloud page json")
                } else if path.starts_with("/v1/observations?") && path.contains("offset=500") {
                    serde_json::to_string(&cloud_observations_page(500, 102, 500))
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

    fn cloud_observations_page(
        start: usize,
        count: usize,
        total: u64,
    ) -> CloudObservationDownloadPage {
        CloudObservationDownloadPage {
            total,
            rows: (start..start + count)
                .map(|index| CloudObservationDownloadPageRow {
                    visibility: Some(CloudObservationVisibility::Public),
                    row: cloud_observations_row(index),
                })
                .collect(),
        }
    }

    fn cloud_observations_row(index: usize) -> CloudObservationRow {
        CloudObservationRow {
            app_id: "app".to_string(),
            author_signature: Some("Author".to_string()),
            remote_ip: IpAddr::from([10, 1, (index / 255) as u8, (index % 255) as u8]),
            remote_port: 443,
            protocol: netstitch_shared::Protocol::Tcp,
            connection_state: netstitch_shared::ConnectionState::Established,
            requests: index as u64,
            first_seen_ms: index as u64,
            last_seen_ms: 10_000 + index as u64,
            failed_hits: 0,
            successful_hits: 1,
            domain_raw: Some(format!("web-cdn-{index}.example")),
            domain_verified: None,
            domain_status: netstitch_shared::CloudDomainStatus::None,
            trust_level: CloudTrustLevel::CommunityVerified,
            source_kind: CloudSourceKind::VerifiedUpload,
            app_signature_key: Some("appsig".to_string()),
            app_signature_subject: None,
            app_signature_issuer: None,
            cloud_observation_id: Some(format!("web-row-{index}")),
        }
    }

    #[test]
    fn browser_monitoring_table_keeps_row_selection_modifier_contract() {
        for expected in [
            "lastSelectedObservationId: null,",
            "observationSelectionInFlightSelect: new Set(),",
            "observationSelectionInFlightDeselect: new Set(),",
            "function queueObservationSelectionOverlay(confirmIds, unconfirmIds) {",
            "function reconcileObservationSelectionOverlay(snapshot) {",
            "reconcileObservationSelectionOverlay(state.snapshot);",
            "const exportReady = observationRowSelected(row);",
            "function applyObservationRowSelection(event, rowId, selected) {",
            "function handleObservationRowClick(event, rowId, selected) {",
            "if (!event?.shiftKey && !event?.ctrlKey && !event?.metaKey) return;",
            "function handleObservationRowAction(event, rowId, selected) {",
            "event?.shiftKey && state.lastSelectedObservationId",
            "event?.ctrlKey || event?.metaKey",
            "queueObservationSelectionOverlay(confirmList, unconfirmList);",
            "await setObservationRowsSelection(confirmIds, unconfirmIds);",
            "await setObservationRowsSelection(ids, []);",
            "await setObservationRowsSelection([], ids);",
            "<tr class=\"' + rowClass + '\" onclick=\"handleObservationRowClick(event, ' + row.id + ', ' + exportReady + ')\">",
            "onclick=\"handleObservationRowAction(event, ' + row.id + ', ' + exportReady + ')\"",
            "onclick=\"event.stopPropagation(); ignoreAddress(",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser monitoring table should keep row selection modifier token {expected}"
            );
        }
    }

    #[test]
    fn browser_table_path_fields_inherit_hover_and_selected_row_style() {
        for expected in [
            ".observations-card .table-wrap .path-field,\n    .cloud-web-panel .table-wrap .path-field {\n      background: transparent;",
            ".observations-card .table-wrap .observation-row:hover .path-field,",
            ".observations-card .table-wrap .observation-row--confirmed .path-field,",
            ".cloud-web-panel .table-wrap .observation-row:hover .path-field,",
            ".cloud-web-panel .table-wrap .observation-row--confirmed .path-field,",
            "box-shadow: none;",
            "function cloudAppTextField(value) {",
            "'<td>' + cloudAppTextField(app.display_name) + '</td>'",
            "'<td>' + cloudAppTextField(app.publisher_name) + '</td>'",
            "'<td>' + cloudAppTextField(app.available_row_count || app.endpoint_count || 0) + '</td>'",
            "'<td>' + cloudAppTextField(cloudAppAuthorsLabel(app)) + '</td>'",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser table path-field should inherit row visual state token {expected}"
            );
        }
        let app_pos = BROWSER_UI_HTML
            .find("'<td>' + cloudAppTextField(app.display_name) + '</td>'")
            .expect("app cell");
        let company_pos = BROWSER_UI_HTML
            .find("'<td>' + cloudAppTextField(app.publisher_name) + '</td>'")
            .expect("company cell");
        let rows_pos = BROWSER_UI_HTML
            .find("'<td>' + cloudAppTextField(app.available_row_count || app.endpoint_count || 0) + '</td>'")
            .expect("rows cell");
        let authors_pos = BROWSER_UI_HTML
            .find("'<td>' + cloudAppTextField(cloudAppAuthorsLabel(app)) + '</td>'")
            .expect("authors cell");
        assert!(
            app_pos < company_pos && company_pos < rows_pos && rows_pos < authors_pos,
            "browser cloud app row cells should match header order: App, Company, Rows, Authors"
        );
    }

    #[test]
    fn browser_filter_updates_use_snapshot_delta_without_dropping_rows() {
        for expected in [
            "const previousSnapshot = state.snapshot ? { ...state.snapshot, filters: currentFilters() } : null;",
            "state.snapshot = { ...state.snapshot, filters: next };",
            "applySnapshotSyncDelta(",
            "await post('/v1/filters', next, { skipSnapshotRefresh: true });",
            "await loadSnapshot({ background: true });",
            "const rows = (current?.observed_endpoints || []).filter((row) => observationMatchesFilters(current, row, options));",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser filter sync should keep canonical rows and render from snapshot delta: {expected}"
            );
        }
        assert!(
            !BROWSER_UI_HTML.contains("state.snapshot.observed_endpoints = filteredObservations"),
            "browser filters must not replace canonical observed_endpoints with filtered rows"
        );
    }

    #[test]
    fn browser_tracked_app_toggle_preserves_connector_identity_route() {
        let source = include_str!("lib.rs");
        for expected in [
            ".route(\"/v1/tracked-apps/toggle\", post(set_tracked_app_enabled))",
            "async fn set_tracked_app_enabled(",
            "SetTrackedAppEnabledRequest",
            "await post('/v1/tracked-apps/toggle', { tracked_app_id: app.id, enabled: !app.enabled });",
        ] {
            assert!(
                source.contains(expected) || BROWSER_UI_HTML.contains(expected),
                "tracked-app toggle must update enabled state by id without re-adding the app: {expected}"
            );
        }
        assert!(
            !BROWSER_UI_HTML.contains(
                "await post('/v1/tracked-apps', { exe_path: app.exe_path, enabled: !app.enabled });"
            ),
            "browser toggle must not call add-app route because that can manualize connector rows"
        );
    }

    #[test]
    fn watcher_exposes_monitoring_csv_import_route() {
        let source = include_str!("lib.rs");
        for expected in [
            ".route(\"/v1/observations/import-csv\", post(import_monitoring_csv))",
            "MonitoringCsvImportRequestDto",
        ] {
            assert!(
                source.contains(expected),
                "watcher must expose cumulative monitoring CSV import contract: {expected}"
            );
        }
    }

    #[test]
    fn browser_file_picker_uses_watcher_filesystem_routes() {
        let source = include_str!("lib.rs");
        for expected in [
            ".route(\"/v1/filesystem/browse\", get(browse_filesystem))",
            ".route(\"/v1/filesystem/read-text\", post(read_text_file))",
            ".route(\"/v1/filesystem/write-text\", post(write_text_file))",
            "async fn browse_filesystem(",
            "async fn read_text_file(",
            "async fn write_text_file(",
        ] {
            assert!(
                source.contains(expected),
                "watcher must expose server-side filesystem picker contract: {expected}"
            );
        }

        for expected in [
            "id=\"server-file-picker-modal\"",
            "function openServerFilePicker(options)",
            "await api('/v1/filesystem/browse?'",
            "post('/v1/filesystem/read-text'",
            "post('/v1/filesystem/write-text'",
            "openCsvImportPicker()",
            "openCsvExportPicker()",
            "imported_count",
            "skipped_count",
            "selectProfileExportProfile(selectedPath)",
            "mode: 'executable'",
            "mode: 'profile'",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser shell must use watcher-host file picker flow: {expected}"
            );
        }

        assert!(
            !BROWSER_UI_HTML.contains("browser_browse_unavailable"),
            "profile export browse should no longer tell users that browser file selection is unavailable"
        );
    }

    #[test]
    fn server_file_picker_selectability_matches_export_modes() {
        assert!(super::picker_path_is_selectable(
            &PathBuf::from("profile.bat"),
            true,
            super::FilePickerMode::Profile,
            &[]
        ));
        assert!(super::picker_path_is_selectable(
            &PathBuf::from("monitoring.csv"),
            true,
            super::FilePickerMode::CsvOpen,
            &[]
        ));
        assert!(!super::picker_path_is_selectable(
            &PathBuf::from("monitoring.txt"),
            true,
            super::FilePickerMode::CsvSave,
            &[]
        ));
        assert!(!super::picker_path_is_selectable(
            &PathBuf::from("folder"),
            false,
            super::FilePickerMode::Any,
            &[]
        ));
        let module_extensions = vec!["conf".to_string(), "txt".to_string()];
        assert!(super::picker_path_is_selectable(
            &PathBuf::from("module.conf"),
            true,
            super::FilePickerMode::FileOpen,
            &module_extensions
        ));
        assert!(!super::picker_path_is_selectable(
            &PathBuf::from("module.csv"),
            true,
            super::FilePickerMode::FileOpen,
            &module_extensions
        ));
    }

    #[test]
    fn browser_display_names_normalize_chrome_like_desktop() {
        for expected in [
            "function trackedAppDisplayName(app) {",
            "function normalizeDisplayName(value) {",
            "if (withoutExe.toLowerCase() === 'chrome') return 'Chrome';",
            "if (app) return trackedAppDisplayName(app);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser app names should share desktop-style capitalization: {expected}"
            );
        }
    }

    #[test]
    fn browser_footer_tooltips_use_standardized_availability_copy() {
        for expected in [
            "availabilityStatusLine(",
            "t('status.watcher', 'Watcher status')",
            "browserUiLogUrl()",
            "t('status.tool', 'Tool status')",
            "webServerStatusLine(snapshot)",
            "t('status.waiting_for_watcher', 'Waiting for watcher')",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser footer tooltip contract should include {expected}"
            );
        }
        for removed in [
            "availabilityTooltip(",
            "footer.watcher.available_browser_tooltip",
            "footer.tool.available_browser_tooltip",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(removed),
                "browser footer tooltip contract should not include {removed}"
            );
        }
    }

    #[test]
    fn browser_footer_web_server_renderer_uses_snapshot_contract_consistently() {
        for expected in [
            "renderFooterWebServer(state.snapshot);",
            "renderFooterWebServer(snapshot);",
            "function renderFooterWebServer(snapshot) {",
            "const enabled = Boolean(snapshot?.app_settings?.web_access_localhost);",
            "container.dataset.tooltip = webServerStatusLine(snapshot);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser web server footer renderer should include {expected}"
            );
        }
        assert!(
            !BROWSER_UI_HTML.contains("function renderFooterWebServer(enabled) {"),
            "browser web server footer renderer should not keep the stale enabled-only signature"
        );
    }

    #[test]
    fn browser_layout_and_tracked_app_paths_stay_shrink_safe() {
        for expected in [
            "html {\n      width: 100%;\n      height: 100%;\n      background: var(--bg);\n      overflow: hidden;",
            "body {\n      margin: 0;\n      width: 100%;\n      height: 100%;\n      min-height: 100vh;\n      background: var(--bg);\n      color: var(--text);\n      font: 13px \"Segoe UI\", system-ui, sans-serif;\n      overflow: hidden;",
            ".shell {\n      position: fixed;\n      inset: 0;\n      display: grid;\n      grid-template-rows: var(--size-header-height) minmax(0, 1fr) var(--size-footer-height);\n      width: 100%;\n      min-width: 100%;\n      min-height: 100%;\n      height: 100vh;\n      height: 100dvh;\n      min-width: 0;",
            ".workspace,\n    .workspace > .column,\n    .workspace section,\n    .card,\n    .card__header,\n    .row,\n    .list,\n    .table-wrap,\n    .integration-status,\n    .integration-status-layout,\n    .integration-status__fields,\n    .integration-status__action,\n    .app-main,\n    .app-actions,\n    .footer-pill {\n      min-width: 0;\n      max-width: 100%;",
            ".path-field--content-width {\n      width: min(100%, var(--path-field-content-width, 100%));\n      max-width: 100%;\n      justify-self: start;\n      flex: 0 1 auto;",
            "style=\"' + pathFieldContentWidthStyle(app.exe_path, 18, 96) + '\"",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser shell should keep shrink-safe layout token {expected}"
            );
        }
    }

    #[test]
    fn browser_ui_is_published_on_unique_port_root_only() {
        let source = include_str!("lib.rs");

        assert!(
            source.contains(".route(\"/\", get(root_web_ui))"),
            "browser UI should be published on the unique-port root path"
        );
        assert!(
            !source.contains(".route(\"/{web_app_name}\""),
            "browser UI must not keep a named app path when the port is already unique"
        );
        assert!(
            !source.contains(".route(\"/web\""),
            "browser UI must not publish /web"
        );
    }

    #[test]
    fn browser_disabled_root_uses_html_shell_instead_of_json_error() {
        let response = super::disabled_browser_ui_response();
        let rendered = format!("{response:?}");
        assert!(
            rendered.contains("text/html"),
            "disabled browser root should render HTML instead of JSON"
        );
    }

    #[test]
    fn browser_access_guard_uses_desktop_client_header_contract() {
        let source = include_str!("lib.rs");
        for expected in [
            "CLIENT_HEADER_NAME",
            "CLIENT_HEADER_DESKTOP_UI",
            "fn is_desktop_client(headers: &HeaderMap) -> bool",
            ".eq_ignore_ascii_case(CLIENT_HEADER_DESKTOP_UI)",
            "browser access is disabled; enable Web access in the desktop header",
            "WEB_AUTH_COOKIE",
            ".route(\"/v1/web-auth\", post(web_auth))",
            "fn browser_web_auth_valid(state: &AppState, headers: &HeaderMap) -> bool",
            "fn web_access_key(state: &AppState) -> WatcherResult<String>",
            "derive_web_access_key(&client_identifier)",
            "fn browser_same_origin_allowed(state: &AppState, headers: &HeaderMap) -> bool",
            "x-content-type-options",
            "content-security-policy",
            "web_key_required",
            "web_key_rejected",
            "web_remote_action",
        ] {
            assert!(
                source.contains(expected),
                "browser access gate should include {expected}"
            );
        }
        for expected in [
            "function webKeyFromHash()",
            "await api('/v1/web-auth'",
            "history.replaceState(null, '', window.location.pathname + window.location.search);",
            "id=\"web-auth-overlay\"",
            "web.auth.invalid_key",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser shell should implement web key auth: {expected}"
            );
        }
        let forbidden_cors_header = ["access", "control", "allow", "origin"].join("-");
        assert!(
            !source.contains(&forbidden_cors_header),
            "browser shell must not publish permissive CORS headers"
        );
    }

    #[test]
    fn browser_web_url_carries_access_key_fragment() {
        let source = include_str!("lib.rs");
        for expected in [
            "format!(\"#web_key={key}\")",
            "format!(\"{}://{host}:{port}/{fragment}\", web_scheme())",
            "fn web_tls_config(state: &AppState) -> Result<RustlsConfig>",
            "web-server.cert.pem",
            "web-server.key.pem",
        ] {
            assert!(
                source.contains(expected),
                "browser web key URL contract should include {expected}"
            );
        }
    }

    #[test]
    fn browser_web_access_toggle_shows_local_disabled_overlay_after_successful_shutdown() {
        for expected in [
            "browserAccessDisabledOverlay: false",
            "id=\"browser-disabled-overlay\"",
            "id=\"browser-disabled-overlay-title\">Web access is disabled</h2>",
            "Enable Web access in the desktop header to use the browser interface.",
            "await post('/v1/settings', { key: 'web.localhost_enabled', value: String(enabled) }, { skipSnapshotRefresh: !enabled });",
            "state.browserAccessDisabledOverlay = true;",
            "renderBrowserDisabledOverlay(state.browserAccessDisabledOverlay);",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser web-access shutdown UX should include {expected}"
            );
        }
    }

    #[test]
    fn browser_language_menu_and_observation_rows_keep_hover_and_selected_states() {
        for expected in [
            "--row-hover: #262c31;",
            ".language-select-option:hover,",
            ".language-select-option--selected {",
            "background: var(--row-on);",
            ".language-select-option--selected:hover,",
            "background: var(--row-enabled-hover);",
            "tbody tr:hover td { background: var(--row-hover); }",
            ".observation-row--confirmed:hover td { background: var(--row-enabled-hover); }",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser visual state contract should include {expected}"
            );
        }
    }

    #[test]
    fn browser_table_headers_keep_sort_icons_when_localized() {
        for expected in [
            "id=\"table-app-sort-icon\"",
            "id=\"table-ip-sort-icon\"",
            "id=\"table-domain-sort-icon\"",
            "id=\"table-app-label\"",
            "id=\"table-ip-label\"",
            "id=\"table-domain-label\"",
            "renderText('table-app-label', 'table.app', 'App');",
            "renderText('table-ip-label', 'table.ip', 'IP');",
            "renderText('table-domain-label', 'table.domain', 'Domain');",
            "renderText('table-action-label', 'table.action', 'Action');",
        ] {
            assert!(
                BROWSER_UI_HTML.contains(expected),
                "browser sortable headers should preserve icon markup via {expected}"
            );
        }
        for removed in [
            "renderText('table-app', 'table.app', 'App');",
            "renderText('table-ip', 'table.ip', 'IP');",
            "renderText('table-domain', 'table.domain', 'Domain');",
        ] {
            assert!(
                !BROWSER_UI_HTML.contains(removed),
                "browser sortable headers should not overwrite header shell via {removed}"
            );
        }
    }

    #[test]
    fn integration_ui_action_route_and_payload_contract_are_present() {
        assert!(
            WATCHER_MAIN_RS.contains(r#".route("/v1/integrations/ui-action""#),
            "watcher must expose the generic module UI action route"
        );
        assert!(
            WATCHER_MAIN_RS.contains(r#".route("/v1/integrations/ui-action-events""#)
                && WATCHER_MAIN_RS.contains("IntegrationModuleUiActionEventsResponseDto")
                && WATCHER_MAIN_RS.contains("module_ui_action_event_from_host_event"),
            "watcher must expose live module UI action events without routing through a final response only"
        );
        assert!(
            WATCHER_MAIN_RS.contains(r#".route("/v1/integrations/{module_id}/icon""#)
                && BROWSER_UI_HTML.contains("module?.icon_path && module?.id")
                && BROWSER_UI_HTML.contains("'/v1/integrations/'"),
            "browser module icons must support module-owned icon_path assets"
        );
        assert!(
            DESKTOP_APP_RS.contains("selected_monitoring_row_ids")
                && DESKTOP_APP_RS.contains("displayed_monitoring_row_ids"),
            "desktop module actions must send selected and displayed monitoring rows"
        );
        assert!(
            BROWSER_UI_HTML.contains("selected_monitoring_row_ids: selectedIds")
                && BROWSER_UI_HTML.contains("displayed_monitoring_row_ids: displayedIds"),
            "browser module actions must send selected and displayed monitoring rows"
        );
        assert!(
            DESKTOP_APP_RS.contains("request.ui_action_token = ui_action_token.clone();")
                && DESKTOP_APP_RS.contains("poll_module_ui_action_events(")
                && DESKTOP_APP_RS
                    .contains("&ui_action_token,\n                    last_event_seq,")
                && DESKTOP_APP_RS.contains("event.event_type == \"ui_values\""),
            "desktop module actions must poll live ui_values events by action token"
        );
        assert!(
            BROWSER_UI_HTML.contains("ui_action_token: actionToken")
                && BROWSER_UI_HTML.contains("pollModuleUiActionEvents")
                && BROWSER_UI_HTML.contains("text(event?.event_type) === 'ui_values'"),
            "browser module actions must poll live ui_values events by action token"
        );
    }

    #[test]
    fn module_ui_action_events_accept_only_explicit_ui_values() {
        let Some((event_type, payload)) =
            super::module_ui_action_event_from_host_event(IntegrationHostEvent {
                event: "ui_values".to_string(),
                payload: serde_json::json!({
                    "values": {
                        "download-progress": 42,
                        "download-status": "Downloading"
                    }
                }),
            })
        else {
            panic!("ui_values event should be accepted");
        };

        assert_eq!(event_type, "ui_values");
        assert_eq!(
            payload
                .get("values")
                .and_then(|values| values.get("download-progress")),
            Some(&serde_json::json!(42))
        );

        for event_name in ["download_progress", "set_ui_values"] {
            assert!(
                super::module_ui_action_event_from_host_event(IntegrationHostEvent {
                    event: event_name.to_string(),
                    payload: serde_json::json!({
                        "values": {
                            "download-progress": 42
                        }
                    }),
                })
                .is_none(),
                "{event_name} must not become a module UI live-event alias"
            );
        }

        assert!(
            super::module_ui_action_event_from_host_event(IntegrationHostEvent {
                event: "ui_values".to_string(),
                payload: serde_json::json!({ "values": ["not", "an", "object"] }),
            })
            .is_none(),
            "ui_values live event must keep the same object payload contract as set_ui_values"
        );
    }

    #[test]
    fn module_ui_action_event_store_is_token_scoped() {
        let mut store = super::ModuleUiActionEventStore::default();
        store.register("module-a", "token-a");
        store.push(
            "module-a",
            "token-a",
            "ui_values",
            serde_json::json!({ "values": { "progress": 10 } }),
        );
        store.push(
            "module-a",
            "token-b",
            "ui_values",
            serde_json::json!({ "values": { "progress": 99 } }),
        );

        let events = store.events_after("module-a", "token-a", 0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[0].ui_action_token, "token-a");
        assert_eq!(
            events[0]
                .payload
                .get("values")
                .and_then(|values| values.get("progress")),
            Some(&serde_json::json!(10))
        );
        assert!(store.events_after("module-a", "token-a", 1).is_empty());
        assert!(
            store
                .events_after("module-a", "missing-token", 0)
                .is_empty(),
            "late or unrelated module UI events must not leak across action tokens"
        );
    }

    #[test]
    fn integration_host_commands_do_not_automate_cloud_or_csv_actions() {
        let command_block = WATCHER_MAIN_RS
            .split("async fn execute_integration_host_commands")
            .nth(1)
            .and_then(|tail| tail.split("async fn set_profile_export_ui_state").next())
            .expect("execute_integration_host_commands block should exist");
        for allowed in [
            "\"set_filters\"",
            "\"start_monitoring\"",
            "\"stop_monitoring\"",
            "\"add_tracked_app\"",
            "\"set_tracked_app_enabled\"",
            "\"set_all_tracked_apps_enabled\"",
            "\"delete_tracked_app\"",
            "\"confirm_monitoring_rows\"",
            "\"delete_monitoring_rows\"",
            "\"select_monitoring_rows\"",
            "\"browse_window\"",
        ] {
            assert!(
                command_block.contains(allowed),
                "missing allowed integration host command {allowed}"
            );
        }
        for forbidden in [
            "cloud",
            "csv",
            "upload",
            "download",
            "export_csv",
            "import_csv",
        ] {
            assert!(
                !command_block.to_ascii_lowercase().contains(forbidden),
                "module host commands must not automate {forbidden}"
            );
        }
    }

    #[test]
    fn default_watcher_addr_uses_loopback() {
        assert_eq!(super::default_watcher_addr(), "0.0.0.0:46473");
    }
}
