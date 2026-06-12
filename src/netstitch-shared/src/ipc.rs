use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{
    AggregatedIpDto, ExportProfilePlanDto, ExportProfileRequestDto, ExportResultDto,
    IntegrationProviderDto, IntegrationStatusDto, MonitorStatus, ObservedEndpoint,
    ObservedEndpointId, SnapshotResponse, TrackedApp, TrackedAppId, UserExportTarget,
};

pub use crate::models::RequestId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcMessage<T> {
    pub request_id: RequestId,
    pub body: T,
}

impl<T> IpcMessage<T> {
    pub fn new(request_id: RequestId, body: T) -> Self {
        Self { request_id, body }
    }
}

pub type IpcRequestMessage = IpcMessage<IpcRequest>;
pub type IpcResponseMessage = IpcMessage<IpcResponse>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddTrackedAppRequest {
    pub exe_path: PathBuf,
    pub enabled: bool,
}

impl AddTrackedAppRequest {
    pub fn new(exe_path: impl Into<PathBuf>) -> Self {
        Self {
            exe_path: exe_path.into(),
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTrackedAppEnabledRequest {
    pub tracked_app_id: TrackedAppId,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetAllTrackedAppsEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteTrackedAppRequest {
    pub tracked_app_id: TrackedAppId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmEndpointsRequest {
    pub endpoint_ids: Vec<ObservedEndpointId>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkEndpointsExportedRequest {
    pub endpoint_ids: Vec<ObservedEndpointId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteObservationRequest {
    pub endpoint_id: ObservedEndpointId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteObservationsRequest {
    pub endpoint_ids: Vec<ObservedEndpointId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportConfirmedRequest {
    pub target: UserExportTarget,
    pub integration_root: Option<PathBuf>,
    pub generate_overlay_script: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetAppSettingRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddIgnoredAddressRequest {
    pub address_pattern: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteIgnoredAddressRequest {
    pub ignored_address_id: crate::models::IgnoredAddressId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadIntegrationProviderRequest {
    #[serde(default)]
    pub module_id: Option<String>,
    pub provider_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorCommandResponse {
    pub accepted: bool,
    pub status: MonitorStatus,
    pub message: Option<String>,
}

impl MonitorCommandResponse {
    pub fn new(status: MonitorStatus) -> Self {
        Self {
            accepted: true,
            status,
            message: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcRequest {
    Ping,
    AddTrackedApp(AddTrackedAppRequest),
    DeleteTrackedApp(DeleteTrackedAppRequest),
    SetTrackedAppEnabled {
        tracked_app_id: TrackedAppId,
        enabled: bool,
    },
    StartMonitoring {
        tracked_app_ids: Vec<TrackedAppId>,
    },
    StopMonitoring {
        tracked_app_ids: Vec<TrackedAppId>,
    },
    ListTrackedApps,
    ListObservedEndpoints {
        tracked_app_id: Option<TrackedAppId>,
        confirmed_only: bool,
        exported_only: bool,
    },
    AggregateObservedIps {
        tracked_app_id: Option<TrackedAppId>,
        confirmed_only: bool,
    },
    ConfirmEndpoints(ConfirmEndpointsRequest),
    DeleteObservation(DeleteObservationRequest),
    DeleteObservations(DeleteObservationsRequest),
    SetAppSetting(SetAppSettingRequest),
    AddIgnoredAddress(AddIgnoredAddressRequest),
    DeleteIgnoredAddress(DeleteIgnoredAddressRequest),
    ExportConfirmed(ExportConfirmedRequest),
    PreviewProfileExport(ExportProfileRequestDto),
    AnalyzeProfileExport(ExportProfileRequestDto),
    BackupProfileExport(ExportProfileRequestDto),
    ApplyProfileExport(ExportProfileRequestDto),
    DownloadIntegrationProvider(DownloadIntegrationProviderRequest),
    ResolveIntegrationOverlay {
        repo_root: PathBuf,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcResponse {
    Pong,
    Ack,
    MonitorCommand(MonitorCommandResponse),
    TrackedApps(Vec<TrackedApp>),
    ObservedEndpoints(Vec<ObservedEndpoint>),
    AggregatedIps(Vec<AggregatedIpDto>),
    Snapshot(SnapshotResponse),
    ExportResult(ExportResultDto),
    ExportProfilePlan(ExportProfilePlanDto),
    IntegrationStatus(IntegrationStatusDto),
    IntegrationProviders(Vec<IntegrationProviderDto>),
    Error(IpcError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcErrorCode {
    InvalidRequest,
    NotFound,
    PermissionDenied,
    StorageUnavailable,
    IntegrationUnavailable,
    ExportFailed,
    ShutdownRejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcError {
    pub code: IpcErrorCode,
    pub message: String,
    pub details: Option<String>,
}

impl IpcError {
    pub fn new(code: IpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        code: IpcErrorCode,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(details.into()),
        }
    }
}
