use std::collections::{BTreeSet, HashMap};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::{Context, Result};
use netstitch_core::{NetstitchCore, ObservationRecord};
use netstitch_shared::ipc::MonitorCommandResponse;
use netstitch_shared::models::{
    CapabilityReport, ConnectionState, DomainCaptureStatusDto, FlowCaptureStatusDto, MonitorStatus,
    TrackedApp, TransportProtocol, UiFiltersDto,
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tokio::sync::{Mutex, watch};
use tracing::{debug, error};

#[cfg(any(target_os = "windows", target_os = "linux"))]
use anyhow::anyhow;
#[cfg(target_os = "windows")]
use std::ffi::{CString, c_void};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::sync::mpsc::{self, Receiver};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, GetLastError, NO_ERROR};
#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6TABLE_OWNER_PID, MIB_TCPTABLE_OWNER_PID,
    MIB_UDP6TABLE_OWNER_PID, MIB_UDPTABLE_OWNER_PID, TCP_TABLE_CLASS, TCP_TABLE_OWNER_PID_ALL,
    UDP_TABLE_CLASS, UDP_TABLE_OWNER_PID,
};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
#[cfg(target_os = "windows")]
use windows::core::{PCSTR, PCWSTR};

// Keep the owner-table pass responsive enough to catch short-lived game endpoints.
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const REEMIT_INTERVAL: Duration = Duration::from_secs(15);
const SEEN_RETENTION: Duration = Duration::from_secs(300);
const DOMAIN_PACKET_MATCH_RETENTION: Duration = Duration::from_secs(15);
const TCP_OWNER_MATCH_RETENTION: Duration = Duration::from_secs(30);
#[cfg(target_os = "windows")]
const WINDIVERT_LAYER_FLOW: i32 = 2;
#[cfg(target_os = "windows")]
const WINDIVERT_LAYER_NETWORK: i32 = 0;
#[cfg(target_os = "windows")]
const WINDIVERT_FLAG_SNIFF: u64 = 1;
#[cfg(target_os = "windows")]
const WINDIVERT_FLAG_RECV_ONLY: u64 = 4;
#[cfg(target_os = "windows")]
const WINDIVERT_EVENT_FLOW_ESTABLISHED: u8 = 1;
#[cfg(target_os = "windows")]
const WINDIVERT_EVENT_FLOW_DELETED: u8 = 2;
#[cfg(target_os = "windows")]
const WINDIVERT_SHUTDOWN_RECV: u32 = 1;
const IPPROTO_TCP: u8 = 6;
const IPPROTO_UDP: u8 = 17;
const DOMAIN_PACKET_BUFFER_LIMIT: usize = 16 * 1024;
#[cfg(target_os = "linux")]
const LINUX_ETH_P_ALL: u16 = 0x0003;
#[cfg(target_os = "linux")]
const LINUX_ETH_P_IP: u16 = 0x0800;
#[cfg(target_os = "linux")]
const LINUX_ETH_P_IPV6: u16 = 0x86DD;
#[cfg(target_os = "linux")]
const LINUX_ETH_P_8021Q: u16 = 0x8100;
#[cfg(target_os = "linux")]
const LINUX_ETH_P_8021AD: u16 = 0x88A8;

#[derive(Clone)]
pub struct MonitorController {
    inner: Arc<Mutex<MonitorRuntime>>,
    domain_capture: Arc<DomainCaptureMetrics>,
    flow_capture: Arc<FlowCaptureMetrics>,
}

struct MonitorRuntime {
    running: bool,
    backend_name: String,
    last_error: Option<String>,
    stop_signal: Option<watch::Sender<bool>>,
}

#[derive(Default)]
struct DomainCaptureMetrics {
    backend_started: AtomicBool,
    packets_seen: AtomicU64,
    tcp_payload_packets: AtomicU64,
    domains_detected: AtomicU64,
    domains_matched: AtomicU64,
    pending_packets: AtomicU64,
    dropped_no_owner: AtomicU64,
    dropped_no_process: AtomicU64,
    dropped_untracked_process: AtomicU64,
    backend_error: StdMutex<Option<String>>,
}

#[derive(Default)]
struct FlowCaptureMetrics {
    flow_backend_started: AtomicBool,
    packet_backend_started: AtomicBool,
    flow_events: AtomicU64,
    udp_flow_events: AtomicU64,
    packet_events: AtomicU64,
    udp_packet_events: AtomicU64,
    observations_emitted: AtomicU64,
    matched_tracked_app: AtomicU64,
    dropped_no_owner: AtomicU64,
    dropped_no_process: AtomicU64,
    dropped_untracked_process: AtomicU64,
    backend_error: StdMutex<Option<String>>,
}

impl FlowCaptureMetrics {
    fn reset(&self) {
        self.flow_backend_started.store(false, Ordering::Relaxed);
        self.packet_backend_started.store(false, Ordering::Relaxed);
        self.flow_events.store(0, Ordering::Relaxed);
        self.udp_flow_events.store(0, Ordering::Relaxed);
        self.packet_events.store(0, Ordering::Relaxed);
        self.udp_packet_events.store(0, Ordering::Relaxed);
        self.observations_emitted.store(0, Ordering::Relaxed);
        self.matched_tracked_app.store(0, Ordering::Relaxed);
        self.dropped_no_owner.store(0, Ordering::Relaxed);
        self.dropped_no_process.store(0, Ordering::Relaxed);
        self.dropped_untracked_process.store(0, Ordering::Relaxed);
        if let Ok(mut error) = self.backend_error.lock() {
            *error = None;
        }
    }

    fn snapshot(&self) -> FlowCaptureStatusDto {
        FlowCaptureStatusDto {
            flow_backend_started: self.flow_backend_started.load(Ordering::Relaxed),
            packet_backend_started: self.packet_backend_started.load(Ordering::Relaxed),
            backend_error: self
                .backend_error
                .lock()
                .ok()
                .and_then(|error| error.clone()),
            flow_events: self.flow_events.load(Ordering::Relaxed),
            udp_flow_events: self.udp_flow_events.load(Ordering::Relaxed),
            packet_events: self.packet_events.load(Ordering::Relaxed),
            udp_packet_events: self.udp_packet_events.load(Ordering::Relaxed),
            observations_emitted: self.observations_emitted.load(Ordering::Relaxed),
            matched_tracked_app: self.matched_tracked_app.load(Ordering::Relaxed),
            dropped_no_owner: self.dropped_no_owner.load(Ordering::Relaxed),
            dropped_no_process: self.dropped_no_process.load(Ordering::Relaxed),
            dropped_untracked_process: self.dropped_untracked_process.load(Ordering::Relaxed),
        }
    }

    fn set_backend_error(&self, error: String) {
        if let Ok(mut backend_error) = self.backend_error.lock() {
            match backend_error.as_mut() {
                Some(existing) if !existing.contains(&error) => {
                    existing.push_str("; ");
                    existing.push_str(&error);
                }
                Some(_) => {}
                None => *backend_error = Some(error),
            }
        }
    }
}

impl DomainCaptureMetrics {
    fn reset(&self) {
        self.backend_started.store(false, Ordering::Relaxed);
        self.packets_seen.store(0, Ordering::Relaxed);
        self.tcp_payload_packets.store(0, Ordering::Relaxed);
        self.domains_detected.store(0, Ordering::Relaxed);
        self.domains_matched.store(0, Ordering::Relaxed);
        self.pending_packets.store(0, Ordering::Relaxed);
        self.dropped_no_owner.store(0, Ordering::Relaxed);
        self.dropped_no_process.store(0, Ordering::Relaxed);
        self.dropped_untracked_process.store(0, Ordering::Relaxed);
        if let Ok(mut error) = self.backend_error.lock() {
            *error = None;
        }
    }

    fn snapshot(&self) -> DomainCaptureStatusDto {
        DomainCaptureStatusDto {
            backend_started: self.backend_started.load(Ordering::Relaxed),
            backend_error: self
                .backend_error
                .lock()
                .ok()
                .and_then(|error| error.clone()),
            packets_seen: self.packets_seen.load(Ordering::Relaxed),
            tcp_payload_packets: self.tcp_payload_packets.load(Ordering::Relaxed),
            domains_detected: self.domains_detected.load(Ordering::Relaxed),
            domains_matched: self.domains_matched.load(Ordering::Relaxed),
            pending_packets: self.pending_packets.load(Ordering::Relaxed),
            dropped_no_owner: self.dropped_no_owner.load(Ordering::Relaxed),
            dropped_no_process: self.dropped_no_process.load(Ordering::Relaxed),
            dropped_untracked_process: self.dropped_untracked_process.load(Ordering::Relaxed),
        }
    }

    fn set_backend_error(&self, error: String) {
        if let Ok(mut backend_error) = self.backend_error.lock() {
            *backend_error = Some(error);
        }
    }
}

#[derive(Clone, Debug)]
struct ProcessMetadata {
    exe_path: Option<PathBuf>,
    process_name: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ObservationKey {
    tracked_app_id: u64,
    remote_ip: String,
    remote_port: u16,
    protocol: TransportProtocol,
    connection_state: ConnectionState,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct EndpointKey {
    remote_ip: String,
    remote_port: u16,
    protocol: TransportProtocol,
}

#[derive(Clone, Copy, Debug)]
struct RecentTrackedEndpoint {
    tracked_app_id: Option<u64>,
    seen_at_ms: u64,
}

#[derive(Clone, Debug)]
struct RawObservation {
    pid: u32,
    local_ip: Option<String>,
    local_port: Option<u16>,
    remote_ip: String,
    remote_port: u16,
    protocol: TransportProtocol,
    connection_state: ConnectionState,
    origin: ObservationOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservationOrigin {
    TcpTable,
    #[cfg(any(target_os = "windows", test))]
    WinDivertFlow,
    UdpPacket,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TcpOwnerKey {
    local_ip: String,
    local_port: u16,
    remote_ip: String,
    remote_port: u16,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct UdpOwnerKey {
    local_ip: String,
    local_port: u16,
}

#[derive(Clone, Debug)]
struct UdpOwnerSocket {
    local_ip: String,
    local_port: u16,
    pid: u32,
}

#[derive(Default)]
struct UdpOwnerIndex {
    exact: HashMap<UdpOwnerKey, u32>,
    port_only: HashMap<u16, Option<u32>>,
}

#[derive(Clone, Debug)]
struct UdpEndpointPacket {
    local_ip: String,
    local_port: u16,
    remote_ip: String,
    remote_port: u16,
}

#[derive(Clone, Debug)]
struct VerifiedDomainPacket {
    local_ip: String,
    local_port: u16,
    remote_ip: String,
    remote_port: u16,
    protocol: TransportProtocol,
    domain_name: String,
    domain_source: String,
}

#[derive(Clone, Debug)]
struct PendingVerifiedDomainPacket {
    packet: VerifiedDomainPacket,
    first_seen_ms: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PacketFlowKey {
    protocol: TransportProtocol,
    local_ip: IpAddr,
    local_port: u16,
    remote_ip: IpAddr,
    remote_port: u16,
}

#[derive(Default)]
struct DomainPacketAccumulator {
    pending: HashMap<PacketFlowKey, Vec<u8>>,
}

#[derive(Default)]
struct DomainPacketIngestResult {
    tcp_payload_seen: bool,
    packet: Option<VerifiedDomainPacket>,
}

#[derive(Clone)]
struct TrackedMatcher {
    tracked_app: TrackedApp,
    normalized_path: String,
    executable_name: String,
    process_names: BTreeSet<String>,
}

impl MonitorController {
    pub fn new(_core: NetstitchCore) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MonitorRuntime {
                running: false,
                backend_name: monitor_backend_name().to_string(),
                last_error: None,
                stop_signal: None,
            })),
            domain_capture: Arc::new(DomainCaptureMetrics::default()),
            flow_capture: Arc::new(FlowCaptureMetrics::default()),
        }
    }

    pub async fn status(&self, core: &NetstitchCore) -> MonitorStatus {
        let runtime = self.inner.lock().await;
        if runtime.running {
            MonitorStatus::Running
        } else if runtime.last_error.is_some() {
            MonitorStatus::Unavailable
        } else if !monitoring_supported() {
            MonitorStatus::Unavailable
        } else if core
            .list_effective_tracked_apps()
            .map(|apps| apps.into_iter().any(|app| app.enabled))
            .unwrap_or(false)
        {
            MonitorStatus::Stopped
        } else {
            MonitorStatus::Stopped
        }
    }

    pub async fn details(&self, core: &NetstitchCore) -> serde_json::Value {
        let runtime = self.inner.lock().await;
        let active_targets = core
            .list_effective_tracked_apps()
            .map(|apps| apps.into_iter().filter(|app| app.enabled).count())
            .unwrap_or_default();
        serde_json::json!({
            "backend_name": runtime.backend_name,
            "last_error": runtime.last_error,
            "active_targets": active_targets,
            "capability": monitor_capability(),
            "domain_capture": self.domain_capture.snapshot(),
            "flow_capture": self.flow_capture.snapshot(),
        })
    }

    pub fn domain_capture_status(&self) -> DomainCaptureStatusDto {
        self.domain_capture.snapshot()
    }

    pub fn flow_capture_status(&self) -> FlowCaptureStatusDto {
        self.flow_capture.snapshot()
    }

    pub async fn start(&self, core: NetstitchCore) -> MonitorCommandResponse {
        if !monitoring_supported() {
            return MonitorCommandResponse {
                accepted: false,
                message: Some(
                    "Monitoring is currently available only on Windows and Linux builds."
                        .to_string(),
                ),
                status: self.status(&core).await,
            };
        }

        let mut runtime = self.inner.lock().await;
        if runtime.running {
            drop(runtime);
            return MonitorCommandResponse {
                accepted: true,
                message: Some("Monitoring is already running".to_string()),
                status: self.status(&core).await,
            };
        }

        let (stop_tx, stop_rx) = watch::channel(false);
        runtime.running = true;
        runtime.last_error = None;
        runtime.stop_signal = Some(stop_tx);
        self.domain_capture.reset();
        self.flow_capture.reset();
        drop(runtime);

        let monitor = self.clone();
        let core_for_task = core.clone();
        let domain_capture = self.domain_capture.clone();
        let flow_capture = self.flow_capture.clone();
        tokio::spawn(async move {
            if let Err(err) = run_monitor_loop(
                monitor.clone(),
                core_for_task,
                stop_rx,
                domain_capture,
                flow_capture,
            )
            .await
            {
                error!("{err:#}");
                monitor.set_error(err.to_string()).await;
            }
            monitor.finish().await;
        });

        MonitorCommandResponse {
            accepted: true,
            message: Some("Monitoring started".to_string()),
            status: self.status(&core).await,
        }
    }

    pub async fn stop(&self, core: &NetstitchCore) -> MonitorCommandResponse {
        let mut runtime = self.inner.lock().await;
        if let Some(stop_signal) = runtime.stop_signal.take() {
            let _ = stop_signal.send(true);
        }
        runtime.running = false;
        drop(runtime);

        MonitorCommandResponse {
            accepted: true,
            message: Some("Monitoring stop requested".to_string()),
            status: self.status(core).await,
        }
    }

    async fn set_error(&self, message: String) {
        let mut runtime = self.inner.lock().await;
        runtime.last_error = Some(message);
    }

    async fn finish(&self) {
        let mut runtime = self.inner.lock().await;
        runtime.running = false;
        runtime.stop_signal = None;
    }
}

async fn run_monitor_loop(
    monitor: MonitorController,
    core: NetstitchCore,
    mut stop_rx: watch::Receiver<bool>,
    domain_capture: Arc<DomainCaptureMetrics>,
    flow_capture: Arc<FlowCaptureMetrics>,
) -> Result<()> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    let mut recently_seen: HashMap<ObservationKey, u64> = HashMap::new();
    let mut pending_domain_packets = Vec::<PendingVerifiedDomainPacket>::new();
    let mut recent_tcp_owner_index = HashMap::<TcpOwnerKey, (u32, u64)>::new();
    let mut recent_tracked_endpoint_index = HashMap::<EndpointKey, RecentTrackedEndpoint>::new();
    let flow_receiver = start_flow_receiver(flow_capture.clone());
    let udp_packet_receiver = start_udp_endpoint_packet_receiver(flow_capture.clone());
    let mut domain_packet_receiver = None::<DomainPacketReceiver>;
    let mut previous_domain_capture_enabled = false;

    loop {
        if *stop_rx.borrow() {
            break;
        }

        let tracked_apps = core
            .list_effective_tracked_apps()
            .context("failed to list tracked apps for monitoring loop")?;
        let domain_capture_enabled = core.domain_capture_enabled().unwrap_or(false);
        if domain_capture_enabled && !previous_domain_capture_enabled {
            domain_capture.reset();
        }
        if domain_capture_enabled {
            if domain_packet_receiver.is_none() {
                domain_packet_receiver = start_domain_packet_receiver(domain_capture.clone());
            }
        } else if previous_domain_capture_enabled || domain_packet_receiver.is_some() {
            domain_packet_receiver = None;
            pending_domain_packets.clear();
            domain_capture
                .backend_started
                .store(false, Ordering::Relaxed);
            domain_capture.pending_packets.store(0, Ordering::Relaxed);
        }
        previous_domain_capture_enabled = domain_capture_enabled;
        let enabled: Vec<_> = tracked_apps.into_iter().filter(|app| app.enabled).collect();
        if enabled.is_empty() {
            tokio::select! {
                _ = stop_rx.changed() => continue,
                _ = tokio::time::sleep(POLL_INTERVAL) => continue,
            }
        }

        system.refresh_processes(ProcessesToUpdate::All, true);
        let now = current_timestamp_ms();
        let process_map = snapshot_processes(&system);
        let mut raw_observations = collect_raw_observations()?;
        let udp_owner_index = match collect_udp_owner_sockets() {
            Ok(sockets) => build_udp_owner_index(&sockets),
            Err(error) => {
                flow_capture.set_backend_error(format!("UDP owner table unavailable: {error:#}"));
                UdpOwnerIndex::default()
            }
        };
        update_recent_tcp_owner_index(&mut recent_tcp_owner_index, &raw_observations, now);
        let mut tcp_owner_index = build_tcp_owner_index(&raw_observations);
        if let Some(receiver) = flow_receiver.as_ref() {
            let flow_observations = receiver.drain();
            update_recent_tcp_owner_index(&mut recent_tcp_owner_index, &flow_observations, now);
            raw_observations.extend(flow_observations);
        }
        if let Some(receiver) = udp_packet_receiver.as_ref() {
            for packet in receiver.drain() {
                let Some(pid) = owner_pid_for_udp_packet(&packet, &udp_owner_index) else {
                    flow_capture
                        .dropped_no_owner
                        .fetch_add(1, Ordering::Relaxed);
                    continue;
                };
                raw_observations.push(RawObservation {
                    pid,
                    local_ip: Some(packet.local_ip),
                    local_port: Some(packet.local_port),
                    remote_ip: packet.remote_ip,
                    remote_port: packet.remote_port,
                    protocol: TransportProtocol::Udp,
                    connection_state: ConnectionState::Attempting,
                    origin: ObservationOrigin::UdpPacket,
                });
            }
        }
        recent_tcp_owner_index.retain(|_, (_, seen_at)| {
            now.saturating_sub(*seen_at) < TCP_OWNER_MATCH_RETENTION.as_millis() as u64
        });
        for (key, (pid, _)) in &recent_tcp_owner_index {
            tcp_owner_index.entry(key.clone()).or_insert(*pid);
        }
        let matchers = build_matchers(&enabled);

        for observation in raw_observations {
            let counted_flow_capture = observation.origin != ObservationOrigin::TcpTable;
            if let Some(process) = process_map.get(&observation.pid) {
                if let Some(tracked) = match_tracked_app(&matchers, process) {
                    if counted_flow_capture {
                        flow_capture
                            .matched_tracked_app
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    let tracked_app_id = tracked.id.unwrap_or_default();
                    let key = ObservationKey {
                        tracked_app_id,
                        remote_ip: observation.remote_ip.clone(),
                        remote_port: observation.remote_port,
                        protocol: observation.protocol.clone(),
                        connection_state: observation.connection_state,
                    };
                    update_recent_tracked_endpoint_index(
                        &mut recent_tracked_endpoint_index,
                        EndpointKey {
                            remote_ip: observation.remote_ip.clone(),
                            remote_port: observation.remote_port,
                            protocol: observation.protocol.clone(),
                        },
                        tracked_app_id,
                        now,
                    );

                    if should_emit(&recently_seen, &key, now) {
                        let endpoint = core.ingest_observation(ObservationRecord {
                            tracked_app_id,
                            process_id: Some(observation.pid),
                            process_name: process.process_name.clone(),
                            remote_ip: observation
                                .remote_ip
                                .parse()
                                .context("failed to parse remote ip")?,
                            remote_port: observation.remote_port,
                            protocol: observation.protocol.clone(),
                            connection_state: observation.connection_state,
                            observed_at_ms: now,
                        })?;
                        core.dispatch_integration_module_event(
                            MonitorStatus::Running,
                            UiFiltersDto::default(),
                            "monitoring.rows_added",
                            serde_json::json!({
                                "source": "monitoring_loop",
                                "observed_endpoint_id": endpoint.id,
                                "tracked_app_id": tracked_app_id,
                            }),
                        );
                        if counted_flow_capture {
                            flow_capture
                                .observations_emitted
                                .fetch_add(1, Ordering::Relaxed);
                        }
                        recently_seen.insert(key, now);
                    }
                } else if counted_flow_capture {
                    flow_capture
                        .dropped_untracked_process
                        .fetch_add(1, Ordering::Relaxed);
                }
            } else if counted_flow_capture {
                flow_capture
                    .dropped_no_process
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(receiver) = domain_packet_receiver.as_ref() {
            pending_domain_packets.extend(receiver.drain().into_iter().map(|packet| {
                PendingVerifiedDomainPacket {
                    packet,
                    first_seen_ms: now,
                }
            }));
        }
        domain_capture
            .pending_packets
            .store(pending_domain_packets.len() as u64, Ordering::Relaxed);
        if !pending_domain_packets.is_empty() {
            let mut still_pending = Vec::new();
            for pending in pending_domain_packets.drain(..) {
                match try_store_verified_domain_packet(
                    &core,
                    &pending.packet,
                    &tcp_owner_index,
                    &recent_tracked_endpoint_index,
                    &process_map,
                    &matchers,
                    now,
                )? {
                    DomainPacketStoreResult::Stored => {
                        domain_capture
                            .domains_matched
                            .fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    DomainPacketStoreResult::NoOwner => {
                        if now.saturating_sub(pending.first_seen_ms)
                            >= DOMAIN_PACKET_MATCH_RETENTION.as_millis() as u64
                        {
                            domain_capture
                                .dropped_no_owner
                                .fetch_add(1, Ordering::Relaxed);
                            continue;
                        }
                    }
                    DomainPacketStoreResult::NoProcess => {
                        domain_capture
                            .dropped_no_process
                            .fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    DomainPacketStoreResult::UntrackedProcess => {
                        domain_capture
                            .dropped_untracked_process
                            .fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                }
                if now.saturating_sub(pending.first_seen_ms)
                    < DOMAIN_PACKET_MATCH_RETENTION.as_millis() as u64
                {
                    still_pending.push(pending);
                }
            }
            pending_domain_packets = still_pending;
            domain_capture
                .pending_packets
                .store(pending_domain_packets.len() as u64, Ordering::Relaxed);
        }

        recently_seen
            .retain(|_, seen_at| now.saturating_sub(*seen_at) < SEEN_RETENTION.as_millis() as u64);
        recent_tracked_endpoint_index.retain(|_, endpoint| {
            now.saturating_sub(endpoint.seen_at_ms) < TCP_OWNER_MATCH_RETENTION.as_millis() as u64
        });

        tokio::select! {
            _ = stop_rx.changed() => continue,
            _ = tokio::time::sleep(POLL_INTERVAL) => {},
        }
    }

    debug!("monitor loop exited");
    monitor.finish().await;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DomainPacketStoreResult {
    Stored,
    NoOwner,
    NoProcess,
    UntrackedProcess,
}

fn try_store_verified_domain_packet(
    core: &NetstitchCore,
    packet: &VerifiedDomainPacket,
    tcp_owner_index: &HashMap<TcpOwnerKey, u32>,
    tracked_endpoint_index: &HashMap<EndpointKey, RecentTrackedEndpoint>,
    process_map: &HashMap<u32, ProcessMetadata>,
    matchers: &[TrackedMatcher],
    now: u64,
) -> Result<DomainPacketStoreResult> {
    let endpoint_key = EndpointKey {
        remote_ip: packet.remote_ip.clone(),
        remote_port: packet.remote_port,
        protocol: packet.protocol.clone(),
    };
    let fallback_tracked_app_id = tracked_endpoint_index
        .get(&endpoint_key)
        .and_then(|endpoint| endpoint.tracked_app_id);
    let owner_key = TcpOwnerKey {
        local_ip: packet.local_ip.clone(),
        local_port: packet.local_port,
        remote_ip: packet.remote_ip.clone(),
        remote_port: packet.remote_port,
    };
    let Some(pid) = tcp_owner_index.get(&owner_key).copied() else {
        if let Some(tracked_app_id) = fallback_tracked_app_id {
            store_verified_domain_for_tracked_app(core, tracked_app_id, packet, now)?;
            return Ok(DomainPacketStoreResult::Stored);
        }
        return Ok(DomainPacketStoreResult::NoOwner);
    };
    let Some(process) = process_map.get(&pid) else {
        if let Some(tracked_app_id) = fallback_tracked_app_id {
            store_verified_domain_for_tracked_app(core, tracked_app_id, packet, now)?;
            return Ok(DomainPacketStoreResult::Stored);
        }
        return Ok(DomainPacketStoreResult::NoProcess);
    };
    let Some(tracked) = match_tracked_app(matchers, process) else {
        if let Some(tracked_app_id) = fallback_tracked_app_id {
            store_verified_domain_for_tracked_app(core, tracked_app_id, packet, now)?;
            return Ok(DomainPacketStoreResult::Stored);
        }
        return Ok(DomainPacketStoreResult::UntrackedProcess);
    };
    let tracked_app_id = tracked.id.unwrap_or_default();
    store_verified_domain_for_tracked_app(core, tracked_app_id, packet, now)?;
    Ok(DomainPacketStoreResult::Stored)
}

fn store_verified_domain_for_tracked_app(
    core: &NetstitchCore,
    tracked_app_id: u64,
    packet: &VerifiedDomainPacket,
    now: u64,
) -> Result<()> {
    let remote_ip = packet
        .remote_ip
        .parse()
        .context("failed to parse verified domain remote ip")?;
    core.store_verified_domain(
        tracked_app_id,
        remote_ip,
        packet.remote_port,
        packet.protocol,
        packet.domain_name.clone(),
        packet.domain_source.clone(),
        now,
    )?;
    Ok(())
}

fn build_matchers(tracked_apps: &[TrackedApp]) -> Vec<TrackedMatcher> {
    let connector_process_names = connector_process_names_by_id();
    build_matchers_with_connector_process_names(tracked_apps, &connector_process_names)
}

fn build_matchers_with_connector_process_names(
    tracked_apps: &[TrackedApp],
    connector_process_names: &HashMap<String, BTreeSet<String>>,
) -> Vec<TrackedMatcher> {
    tracked_apps
        .iter()
        .cloned()
        .map(|tracked_app| {
            let normalized_path = normalize_path(&tracked_app.exe_path.to_string_lossy());
            let executable_name = Path::new(&tracked_app.exe_path)
                .file_name()
                .and_then(|part| part.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let mut process_names = BTreeSet::new();
            insert_process_name(&mut process_names, &executable_name);
            if let Some(process_name) = tracked_app.process_name.as_deref() {
                insert_process_name(&mut process_names, process_name);
            }
            if let Some(connector_id) = tracked_app.connector_id.as_deref()
                && let Some(names) = connector_process_names.get(connector_id)
            {
                process_names.extend(names.iter().cloned());
            }
            TrackedMatcher {
                tracked_app,
                normalized_path,
                executable_name,
                process_names,
            }
        })
        .collect()
}

fn connector_process_names_by_id() -> HashMap<String, BTreeSet<String>> {
    let current_os = netstitch_connectors::TargetOs::current();
    let mut by_id: HashMap<String, BTreeSet<String>> = HashMap::new();
    for manifest in netstitch_connectors::manifests() {
        let names = by_id.entry(manifest.id.to_string()).or_default();
        for process_name in manifest.process_names {
            insert_process_name(names, process_name);
        }
        for alias in manifest.process_aliases {
            if current_os.is_none() || Some(alias.os) == current_os {
                insert_process_name(names, alias.name);
            }
        }
    }
    by_id
}

fn insert_process_name(names: &mut BTreeSet<String>, process_name: &str) {
    let trimmed = process_name.trim();
    if !trimmed.is_empty() {
        names.insert(trimmed.to_ascii_lowercase());
    }
}

fn match_tracked_app<'a>(
    matchers: &'a [TrackedMatcher],
    process: &ProcessMetadata,
) -> Option<&'a TrackedApp> {
    let normalized_process_path = process
        .exe_path
        .as_ref()
        .map(|path| normalize_path(&path.to_string_lossy()))
        .unwrap_or_default();
    let process_name = process.process_name.to_ascii_lowercase();

    matchers
        .iter()
        .find(|matcher| {
            (!normalized_process_path.is_empty()
                && matcher.normalized_path == normalized_process_path)
                || matcher.executable_name == process_name
                || matcher.process_names.contains(&process_name)
        })
        .map(|matcher| &matcher.tracked_app)
}

fn snapshot_processes(system: &System) -> HashMap<u32, ProcessMetadata> {
    system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let exe_path = process.exe().map(|path| path.to_path_buf());
            let process_name = process.name().to_string_lossy().to_string();
            (
                pid.as_u32(),
                ProcessMetadata {
                    exe_path,
                    process_name,
                },
            )
        })
        .collect()
}

fn should_emit(seen: &HashMap<ObservationKey, u64>, key: &ObservationKey, now: u64) -> bool {
    match seen.get(key) {
        Some(previous) => now.saturating_sub(*previous) >= REEMIT_INTERVAL.as_millis() as u64,
        None => true,
    }
}

fn update_recent_tracked_endpoint_index(
    index: &mut HashMap<EndpointKey, RecentTrackedEndpoint>,
    key: EndpointKey,
    tracked_app_id: u64,
    now: u64,
) {
    match index.get_mut(&key) {
        Some(existing) => {
            if existing.tracked_app_id != Some(tracked_app_id) {
                existing.tracked_app_id = None;
            }
            existing.seen_at_ms = now;
        }
        None => {
            index.insert(
                key,
                RecentTrackedEndpoint {
                    tracked_app_id: Some(tracked_app_id),
                    seen_at_ms: now,
                },
            );
        }
    }
}

fn build_tcp_owner_index(observations: &[RawObservation]) -> HashMap<TcpOwnerKey, u32> {
    observations
        .iter()
        .filter(|observation| observation.protocol == TransportProtocol::Tcp)
        .filter_map(|observation| {
            Some((
                tcp_owner_key_from_observation(observation)?,
                observation.pid,
            ))
        })
        .collect()
}

fn update_recent_tcp_owner_index(
    index: &mut HashMap<TcpOwnerKey, (u32, u64)>,
    observations: &[RawObservation],
    now: u64,
) {
    for observation in observations
        .iter()
        .filter(|observation| observation.protocol == TransportProtocol::Tcp)
    {
        if let Some(key) = tcp_owner_key_from_observation(observation) {
            index.insert(key, (observation.pid, now));
        }
    }
}

fn tcp_owner_key_from_observation(observation: &RawObservation) -> Option<TcpOwnerKey> {
    Some(TcpOwnerKey {
        local_ip: observation.local_ip.clone()?,
        local_port: observation.local_port?,
        remote_ip: observation.remote_ip.clone(),
        remote_port: observation.remote_port,
    })
}

fn monitor_backend_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows-tcp-table+windivert-flow+windivert-packet-domain"
    } else if cfg!(target_os = "linux") {
        "linux-proc-net"
    } else {
        "unsupported"
    }
}

fn monitoring_supported() -> bool {
    cfg!(any(target_os = "windows", target_os = "linux"))
}

fn monitor_capability() -> CapabilityReport {
    CapabilityReport {
        monitoring_supported: monitoring_supported(),
        export_supported: true,
        external_integrations_supported: true,
        requires_admin: cfg!(target_os = "windows"),
        ipc_supported: true,
    }
}

#[cfg(target_os = "windows")]
fn normalize_path(value: &str) -> String {
    value.trim().replace('/', "\\").to_ascii_lowercase()
}

#[cfg(not(target_os = "windows"))]
fn normalize_path(value: &str) -> String {
    value.trim().to_string()
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(any(target_os = "windows", test))]
fn tcp_state_to_connection_state(raw_state: u32) -> ConnectionState {
    match raw_state {
        3 | 4 => ConnectionState::Attempting,
        5 => ConnectionState::Established,
        6 | 7 | 8 | 9 | 10 | 11 => ConnectionState::Closing,
        1 | 12 => ConnectionState::Failed,
        _ => ConnectionState::Unknown,
    }
}

#[cfg(target_os = "windows")]
struct FlowReceiver {
    receiver: Receiver<RawObservation>,
    shutdown: Arc<WinDivertRuntime>,
}

#[cfg(target_os = "windows")]
impl FlowReceiver {
    fn drain(&self) -> Vec<RawObservation> {
        self.receiver.try_iter().collect()
    }
}

#[cfg(target_os = "windows")]
impl Drop for FlowReceiver {
    fn drop(&mut self) {
        self.shutdown.close();
    }
}

#[cfg(target_os = "windows")]
fn start_flow_receiver(metrics: Arc<FlowCaptureMetrics>) -> Option<FlowReceiver> {
    match start_windivert_flow_receiver(metrics.clone()) {
        Ok(receiver) => Some(receiver),
        Err(error) => {
            metrics.set_backend_error(format!("{error:#}"));
            debug!("WinDivert FLOW backend unavailable: {error:#}");
            None
        }
    }
}

#[cfg(target_os = "windows")]
struct UdpPacketReceiver {
    receiver: Receiver<UdpEndpointPacket>,
    shutdown: Arc<WinDivertRuntime>,
}

#[cfg(target_os = "windows")]
impl UdpPacketReceiver {
    fn drain(&self) -> Vec<UdpEndpointPacket> {
        self.receiver.try_iter().collect()
    }
}

#[cfg(target_os = "windows")]
impl Drop for UdpPacketReceiver {
    fn drop(&mut self) {
        self.shutdown.close();
    }
}

#[cfg(target_os = "windows")]
fn start_udp_endpoint_packet_receiver(
    metrics: Arc<FlowCaptureMetrics>,
) -> Option<UdpPacketReceiver> {
    match start_windivert_udp_endpoint_packet_receiver(metrics.clone()) {
        Ok(receiver) => Some(receiver),
        Err(error) => {
            metrics.set_backend_error(format!("{error:#}"));
            debug!("WinDivert UDP endpoint backend unavailable: {error:#}");
            None
        }
    }
}

#[cfg(target_os = "windows")]
struct DomainPacketReceiver {
    receiver: Receiver<VerifiedDomainPacket>,
    shutdown: Arc<WinDivertRuntime>,
}

#[cfg(target_os = "windows")]
impl DomainPacketReceiver {
    fn drain(&self) -> Vec<VerifiedDomainPacket> {
        self.receiver.try_iter().collect()
    }
}

#[cfg(target_os = "windows")]
impl Drop for DomainPacketReceiver {
    fn drop(&mut self) {
        self.shutdown.close();
    }
}

#[cfg(target_os = "windows")]
fn start_domain_packet_receiver(
    metrics: Arc<DomainCaptureMetrics>,
) -> Option<DomainPacketReceiver> {
    match start_windivert_domain_packet_receiver(metrics.clone()) {
        Ok(receiver) => Some(receiver),
        Err(error) => {
            metrics.set_backend_error(format!("{error:#}"));
            debug!("WinDivert packet domain backend unavailable: {error:#}");
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
struct FlowReceiver;

#[cfg(not(target_os = "windows"))]
impl FlowReceiver {
    fn drain(&self) -> Vec<RawObservation> {
        Vec::new()
    }
}

#[cfg(not(target_os = "windows"))]
fn start_flow_receiver(_metrics: Arc<FlowCaptureMetrics>) -> Option<FlowReceiver> {
    None
}

#[cfg(not(target_os = "windows"))]
struct UdpPacketReceiver;

#[cfg(not(target_os = "windows"))]
impl UdpPacketReceiver {
    fn drain(&self) -> Vec<UdpEndpointPacket> {
        Vec::new()
    }
}

#[cfg(not(target_os = "windows"))]
fn start_udp_endpoint_packet_receiver(
    _metrics: Arc<FlowCaptureMetrics>,
) -> Option<UdpPacketReceiver> {
    None
}

#[cfg(target_os = "linux")]
struct DomainPacketReceiver {
    receiver: Receiver<VerifiedDomainPacket>,
    shutdown: Arc<LinuxRawPacketRuntime>,
}

#[cfg(target_os = "linux")]
impl DomainPacketReceiver {
    fn drain(&self) -> Vec<VerifiedDomainPacket> {
        self.receiver.try_iter().collect()
    }
}

#[cfg(target_os = "linux")]
impl Drop for DomainPacketReceiver {
    fn drop(&mut self) {
        self.shutdown.close();
    }
}

#[cfg(target_os = "linux")]
struct LinuxRawPacketRuntime {
    socket: OwnedFd,
    closed: AtomicBool,
}

#[cfg(target_os = "linux")]
impl LinuxRawPacketRuntime {
    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

#[cfg(target_os = "linux")]
fn start_domain_packet_receiver(
    metrics: Arc<DomainCaptureMetrics>,
) -> Option<DomainPacketReceiver> {
    match start_linux_domain_packet_receiver(metrics.clone()) {
        Ok(receiver) => Some(receiver),
        Err(error) => {
            metrics.set_backend_error(format!("{error:#}"));
            debug!("Linux packet domain backend unavailable: {error:#}");
            None
        }
    }
}

#[cfg(target_os = "linux")]
fn start_linux_domain_packet_receiver(
    metrics: Arc<DomainCaptureMetrics>,
) -> Result<DomainPacketReceiver> {
    let protocol = i32::from(LINUX_ETH_P_ALL.to_be());
    let fd = unsafe {
        libc::socket(
            libc::AF_PACKET,
            libc::SOCK_RAW | libc::SOCK_NONBLOCK,
            protocol,
        )
    };
    if fd < 0 {
        return Err(anyhow!(
            "Linux packet-domain backend requires CAP_NET_RAW/root access: {}",
            std::io::Error::last_os_error()
        ));
    }

    let runtime = Arc::new(LinuxRawPacketRuntime {
        socket: unsafe { OwnedFd::from_raw_fd(fd) },
        closed: AtomicBool::new(false),
    });
    let (sender, receiver) = mpsc::channel();
    let thread_runtime = runtime.clone();
    metrics.backend_started.store(true, Ordering::Relaxed);
    let thread_metrics = metrics.clone();
    std::thread::Builder::new()
        .name("netstitch-linux-packet-domain".to_string())
        .spawn(move || {
            receive_linux_domain_packets(thread_runtime, sender, thread_metrics);
        })
        .context("failed to spawn Linux packet-domain receiver thread")?;

    Ok(DomainPacketReceiver {
        receiver,
        shutdown: runtime,
    })
}

#[cfg(target_os = "linux")]
fn receive_linux_domain_packets(
    runtime: Arc<LinuxRawPacketRuntime>,
    sender: mpsc::Sender<VerifiedDomainPacket>,
    metrics: Arc<DomainCaptureMetrics>,
) {
    let mut frame = vec![0u8; 0xFFFF];
    let mut accumulator = DomainPacketAccumulator::default();
    loop {
        if runtime.closed.load(Ordering::SeqCst) {
            break;
        }

        let received_len = unsafe {
            libc::recv(
                runtime.socket.as_raw_fd(),
                frame.as_mut_ptr().cast(),
                frame.len(),
                libc::MSG_DONTWAIT,
            )
        };
        if received_len < 0 {
            let error = std::io::Error::last_os_error();
            let code = error.raw_os_error();
            if code == Some(libc::EAGAIN) || code == Some(libc::EWOULDBLOCK) {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            metrics.set_backend_error(format!("Linux packet-domain receive failed: {error}"));
            break;
        }

        let frame_len = received_len as usize;
        if frame_len > frame.len() {
            continue;
        }
        let Some(packet) = linux_ip_packet_from_link_frame(&frame[..frame_len]) else {
            continue;
        };

        metrics.packets_seen.fetch_add(1, Ordering::Relaxed);
        let ingest_result = accumulator.ingest(packet);
        if ingest_result.tcp_payload_seen {
            metrics.tcp_payload_packets.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(domain_packet) = ingest_result.packet {
            metrics.domains_detected.fetch_add(1, Ordering::Relaxed);
            if sender.send(domain_packet).is_err() {
                break;
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_ip_packet_from_link_frame(frame: &[u8]) -> Option<&[u8]> {
    if let Some(packet) = linux_ip_packet_from_ethernet_frame(frame) {
        return Some(packet);
    }
    if let Some(packet) = linux_ip_packet_from_cooked_frame(frame) {
        return Some(packet);
    }
    linux_ip_packet_candidate(frame)
}

#[cfg(target_os = "linux")]
fn linux_ip_packet_from_ethernet_frame(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 14 {
        return None;
    }
    let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
    let (payload_offset, payload_type) = match ethertype {
        LINUX_ETH_P_IP | LINUX_ETH_P_IPV6 => (14, ethertype),
        LINUX_ETH_P_8021Q | LINUX_ETH_P_8021AD if frame.len() >= 18 => {
            (18, u16::from_be_bytes([frame[16], frame[17]]))
        }
        _ => return None,
    };
    linux_typed_ip_payload(frame, payload_offset, payload_type)
}

#[cfg(target_os = "linux")]
fn linux_ip_packet_from_cooked_frame(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 16 {
        return None;
    }
    let protocol = u16::from_be_bytes([frame[14], frame[15]]);
    linux_typed_ip_payload(frame, 16, protocol)
}

#[cfg(target_os = "linux")]
fn linux_typed_ip_payload(frame: &[u8], offset: usize, protocol: u16) -> Option<&[u8]> {
    match protocol {
        LINUX_ETH_P_IP | LINUX_ETH_P_IPV6 => linux_ip_packet_candidate(frame.get(offset..)?),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn linux_ip_packet_candidate(packet: &[u8]) -> Option<&[u8]> {
    let version = packet.first()? >> 4;
    match version {
        4 => {
            if packet.len() < 20 {
                return None;
            }
            let ihl = usize::from(packet[0] & 0x0f) * 4;
            let total_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
            if ihl < 20 || total_len < ihl || packet.len() < total_len {
                return None;
            }
            Some(&packet[..total_len])
        }
        6 => {
            if packet.len() < 40 {
                return None;
            }
            let payload_len = u16::from_be_bytes([packet[4], packet[5]]) as usize;
            let total_len = 40usize.checked_add(payload_len)?;
            if packet.len() < total_len {
                return None;
            }
            Some(&packet[..total_len])
        }
        _ => None,
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
struct DomainPacketReceiver;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
impl DomainPacketReceiver {
    fn drain(&self) -> Vec<VerifiedDomainPacket> {
        Vec::new()
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn start_domain_packet_receiver(
    _metrics: Arc<DomainCaptureMetrics>,
) -> Option<DomainPacketReceiver> {
    None
}

#[cfg(target_os = "windows")]
type DivertHandle = isize;

#[cfg(target_os = "windows")]
type WinDivertOpenFn = unsafe extern "system" fn(*const i8, i32, i16, u64) -> DivertHandle;
#[cfg(target_os = "windows")]
type WinDivertRecvFn = unsafe extern "system" fn(
    DivertHandle,
    *mut c_void,
    u32,
    *mut u32,
    *mut WinDivertAddress,
) -> i32;
#[cfg(target_os = "windows")]
type WinDivertCloseFn = unsafe extern "system" fn(DivertHandle) -> i32;
#[cfg(target_os = "windows")]
type WinDivertShutdownFn = unsafe extern "system" fn(DivertHandle, u32) -> i32;
#[cfg(target_os = "windows")]
type WinDivertFormatIpv6Fn = unsafe extern "system" fn(*const u32, *mut i8, u32) -> i32;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WinDivertAddress {
    timestamp: i64,
    bitfield: u64,
    flow: WinDivertFlowData,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WinDivertFlowData {
    endpoint_id: u64,
    parent_endpoint_id: u64,
    process_id: u32,
    local_addr: [u32; 4],
    remote_addr: [u32; 4],
    local_port: u16,
    remote_port: u16,
    protocol: u8,
}

#[cfg(target_os = "windows")]
struct WinDivertApi {
    open: WinDivertOpenFn,
    recv: WinDivertRecvFn,
    close: WinDivertCloseFn,
    shutdown: Option<WinDivertShutdownFn>,
    format_ipv6: Option<WinDivertFormatIpv6Fn>,
}

#[cfg(target_os = "windows")]
struct WinDivertRuntime {
    api: Arc<WinDivertApi>,
    handle: DivertHandle,
    closed: AtomicBool,
}

#[cfg(target_os = "windows")]
impl WinDivertRuntime {
    fn close(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }

        if let Some(shutdown) = self.api.shutdown {
            unsafe {
                let _ = shutdown(self.handle, WINDIVERT_SHUTDOWN_RECV);
            }
        }
        unsafe {
            let _ = (self.api.close)(self.handle);
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for WinDivertRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(target_os = "windows")]
fn start_windivert_flow_receiver(metrics: Arc<FlowCaptureMetrics>) -> Result<FlowReceiver> {
    let api = Arc::new(load_windivert_api()?);
    let filter =
        CString::new("(tcp or udp) and !loopback").context("failed to build WinDivert filter")?;
    let handle = unsafe { (api.open)(filter.as_ptr(), WINDIVERT_LAYER_FLOW, 0, 0) };
    if handle == 0 || handle == -1 {
        return Err(anyhow!(
            "WinDivertOpen failed for FLOW layer: {}; ensure administrator rights and WinDivert DLL/SYS availability",
            last_windows_error_detail()
        ));
    }

    let runtime = Arc::new(WinDivertRuntime {
        api,
        handle,
        closed: AtomicBool::new(false),
    });
    let (sender, receiver) = mpsc::channel();
    let thread_runtime = runtime.clone();
    metrics.flow_backend_started.store(true, Ordering::Relaxed);
    let thread_metrics = metrics.clone();
    std::thread::Builder::new()
        .name("netstitch-windivert-flow".to_string())
        .spawn(move || {
            receive_windivert_flow_events(thread_runtime, sender, thread_metrics);
        })
        .context("failed to spawn WinDivert FLOW receiver thread")?;

    Ok(FlowReceiver {
        receiver,
        shutdown: runtime,
    })
}

#[cfg(target_os = "windows")]
fn start_windivert_udp_endpoint_packet_receiver(
    metrics: Arc<FlowCaptureMetrics>,
) -> Result<UdpPacketReceiver> {
    let api = Arc::new(load_windivert_api()?);
    let filter = CString::new("outbound and udp and udp.PayloadLength > 0 and !loopback")
        .context("failed to build WinDivert UDP endpoint filter")?;
    let (handle, _open_detail) = open_windivert_network_sniff(&api, &filter)?;

    let runtime = Arc::new(WinDivertRuntime {
        api,
        handle,
        closed: AtomicBool::new(false),
    });
    let (sender, receiver) = mpsc::channel();
    let thread_runtime = runtime.clone();
    metrics
        .packet_backend_started
        .store(true, Ordering::Relaxed);
    let thread_metrics = metrics.clone();
    std::thread::Builder::new()
        .name("netstitch-windivert-udp-endpoints".to_string())
        .spawn(move || {
            receive_windivert_udp_endpoint_packets(thread_runtime, sender, thread_metrics);
        })
        .context("failed to spawn WinDivert UDP endpoint receiver thread")?;

    Ok(UdpPacketReceiver {
        receiver,
        shutdown: runtime,
    })
}

#[cfg(target_os = "windows")]
fn start_windivert_domain_packet_receiver(
    metrics: Arc<DomainCaptureMetrics>,
) -> Result<DomainPacketReceiver> {
    let api = Arc::new(load_windivert_api()?);
    let filter = CString::new(
        "outbound and ((tcp and tcp.PayloadLength > 0) or (udp and udp.DstPort == 443 and udp.PayloadLength > 0)) and !loopback",
    )
    .context("failed to build WinDivert packet-domain filter")?;
    let (handle, _open_detail) = open_windivert_network_sniff(&api, &filter)?;

    let runtime = Arc::new(WinDivertRuntime {
        api,
        handle,
        closed: AtomicBool::new(false),
    });
    let (sender, receiver) = mpsc::channel();
    let thread_runtime = runtime.clone();
    metrics.backend_started.store(true, Ordering::Relaxed);
    let thread_metrics = metrics.clone();
    std::thread::Builder::new()
        .name("netstitch-windivert-packet-domain".to_string())
        .spawn(move || {
            receive_windivert_domain_packets(thread_runtime, sender, thread_metrics);
        })
        .context("failed to spawn WinDivert packet-domain receiver thread")?;

    Ok(DomainPacketReceiver {
        receiver,
        shutdown: runtime,
    })
}

#[cfg(target_os = "windows")]
fn open_windivert_network_sniff(
    api: &WinDivertApi,
    filter: &CString,
) -> Result<(DivertHandle, String)> {
    let attempts = [
        ("sniff priority 0", 0, WINDIVERT_FLAG_SNIFF),
        (
            "sniff+recv-only priority 0",
            0,
            WINDIVERT_FLAG_SNIFF | WINDIVERT_FLAG_RECV_ONLY,
        ),
        ("sniff priority 100", 100, WINDIVERT_FLAG_SNIFF),
        ("sniff priority -100", -100, WINDIVERT_FLAG_SNIFF),
    ];
    let mut failures = Vec::new();
    for (label, priority, flags) in attempts {
        let handle =
            unsafe { (api.open)(filter.as_ptr(), WINDIVERT_LAYER_NETWORK, priority, flags) };
        if handle != 0 && handle != -1 {
            return Ok((handle, label.to_string()));
        }
        failures.push(format!("{label}: {}", last_windows_error_detail()));
    }

    Err(anyhow!(
        "WinDivertOpen failed for NETWORK sniff layer after {} safe attempts: {}; ensure administrator rights and WinDivert DLL/SYS availability",
        attempts.len(),
        failures.join("; ")
    ))
}

#[cfg(target_os = "windows")]
fn last_windows_error_detail() -> String {
    let error = unsafe { GetLastError() };
    format!("Win32 error {}", error.0)
}

#[cfg(target_os = "windows")]
fn receive_windivert_flow_events(
    runtime: Arc<WinDivertRuntime>,
    sender: mpsc::Sender<RawObservation>,
    metrics: Arc<FlowCaptureMetrics>,
) {
    let mut packet = [0u8; 1];
    loop {
        if runtime.closed.load(Ordering::SeqCst) {
            break;
        }

        let mut received_len = 0u32;
        let mut address = WinDivertAddress::default();
        let ok = unsafe {
            (runtime.api.recv)(
                runtime.handle,
                packet.as_mut_ptr().cast(),
                packet.len() as u32,
                &mut received_len,
                &mut address,
            )
        };
        if ok == 0 {
            break;
        }
        metrics.flow_events.fetch_add(1, Ordering::Relaxed);
        if address.flow.protocol == IPPROTO_UDP {
            metrics.udp_flow_events.fetch_add(1, Ordering::Relaxed);
        }

        if let Some(observation) = raw_observation_from_flow(&runtime.api, &address) {
            if sender.send(observation).is_err() {
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn receive_windivert_udp_endpoint_packets(
    runtime: Arc<WinDivertRuntime>,
    sender: mpsc::Sender<UdpEndpointPacket>,
    metrics: Arc<FlowCaptureMetrics>,
) {
    let mut packet = vec![0u8; 0xFFFF];
    loop {
        if runtime.closed.load(Ordering::SeqCst) {
            break;
        }

        let mut received_len = 0u32;
        let mut address = WinDivertAddress::default();
        let ok = unsafe {
            (runtime.api.recv)(
                runtime.handle,
                packet.as_mut_ptr().cast(),
                packet.len() as u32,
                &mut received_len,
                &mut address,
            )
        };
        if ok == 0 {
            break;
        }
        metrics.packet_events.fetch_add(1, Ordering::Relaxed);

        let packet_len = received_len as usize;
        if packet_len > packet.len() {
            continue;
        }
        if let Some(endpoint) = udp_endpoint_packet_from_network_packet(&packet[..packet_len]) {
            metrics.udp_packet_events.fetch_add(1, Ordering::Relaxed);
            if sender.send(endpoint).is_err() {
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn receive_windivert_domain_packets(
    runtime: Arc<WinDivertRuntime>,
    sender: mpsc::Sender<VerifiedDomainPacket>,
    metrics: Arc<DomainCaptureMetrics>,
) {
    let mut packet = vec![0u8; 0xFFFF];
    let mut accumulator = DomainPacketAccumulator::default();
    loop {
        if runtime.closed.load(Ordering::SeqCst) {
            break;
        }

        let mut received_len = 0u32;
        let mut address = WinDivertAddress::default();
        let ok = unsafe {
            (runtime.api.recv)(
                runtime.handle,
                packet.as_mut_ptr().cast(),
                packet.len() as u32,
                &mut received_len,
                &mut address,
            )
        };
        if ok == 0 {
            break;
        }
        metrics.packets_seen.fetch_add(1, Ordering::Relaxed);

        let packet_len = received_len as usize;
        if packet_len > packet.len() {
            continue;
        }
        let ingest_result = accumulator.ingest(&packet[..packet_len]);
        if ingest_result.tcp_payload_seen {
            metrics.tcp_payload_packets.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(domain_packet) = ingest_result.packet {
            metrics.domains_detected.fetch_add(1, Ordering::Relaxed);
            if sender.send(domain_packet).is_err() {
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn raw_observation_from_flow(
    api: &WinDivertApi,
    address: &WinDivertAddress,
) -> Option<RawObservation> {
    let protocol = windivert_protocol_to_transport(address.flow.protocol)?;
    let event = windivert_event(address.bitfield);
    let connection_state = flow_event_to_connection_state(address.flow.protocol, event)?;
    let remote_ip = format_windivert_remote_ip(api, address)?;
    if is_unspecified_ip(&remote_ip) {
        return None;
    }

    Some(RawObservation {
        pid: address.flow.process_id,
        local_ip: format_windivert_local_ip(api, address),
        local_port: Some(address.flow.local_port),
        remote_ip,
        remote_port: address.flow.remote_port,
        protocol,
        connection_state,
        origin: ObservationOrigin::WinDivertFlow,
    })
}

#[cfg(target_os = "windows")]
fn windivert_protocol_to_transport(protocol: u8) -> Option<TransportProtocol> {
    match protocol {
        IPPROTO_TCP => Some(TransportProtocol::Tcp),
        IPPROTO_UDP => Some(TransportProtocol::Udp),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn flow_event_to_connection_state(protocol: u8, event: u8) -> Option<ConnectionState> {
    match (protocol, event) {
        (IPPROTO_TCP, WINDIVERT_EVENT_FLOW_ESTABLISHED) => Some(ConnectionState::Established),
        (IPPROTO_TCP, WINDIVERT_EVENT_FLOW_DELETED) => Some(ConnectionState::Closing),
        (IPPROTO_UDP, WINDIVERT_EVENT_FLOW_ESTABLISHED) => Some(ConnectionState::Attempting),
        (IPPROTO_UDP, WINDIVERT_EVENT_FLOW_DELETED) => Some(ConnectionState::Closing),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn windivert_event(bitfield: u64) -> u8 {
    ((bitfield >> 8) & 0xff) as u8
}

#[cfg(target_os = "windows")]
fn format_windivert_remote_ip(api: &WinDivertApi, address: &WinDivertAddress) -> Option<String> {
    format_windivert_ipv6(api, &address.flow.remote_addr).and_then(normalize_mapped_ip)
}

#[cfg(target_os = "windows")]
fn format_windivert_local_ip(api: &WinDivertApi, address: &WinDivertAddress) -> Option<String> {
    format_windivert_ipv6(api, &address.flow.local_addr).and_then(normalize_mapped_ip)
}

#[cfg(target_os = "windows")]
fn format_windivert_ipv6(api: &WinDivertApi, raw_addr: &[u32; 4]) -> Option<String> {
    if let Some(format_ipv6) = api.format_ipv6 {
        let mut buffer = [0i8; 128];
        let ok =
            unsafe { format_ipv6(raw_addr.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        if ok != 0 {
            return c_buffer_to_string(&buffer);
        }
    }

    let bytes = raw_addr
        .iter()
        .flat_map(|part| part.to_be_bytes())
        .collect::<Vec<_>>();
    let bytes: [u8; 16] = bytes.try_into().ok()?;
    Some(Ipv6Addr::from(bytes).to_string())
}

#[cfg(target_os = "windows")]
fn normalize_mapped_ip(value: String) -> Option<String> {
    match value.parse::<std::net::IpAddr>().ok()? {
        std::net::IpAddr::V4(ip) => Some(ip.to_string()),
        std::net::IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(|mapped| mapped.to_string())
            .or_else(|| Some(ip.to_string())),
    }
}

#[cfg(target_os = "windows")]
fn c_buffer_to_string(buffer: &[i8]) -> Option<String> {
    let end = buffer.iter().position(|value| *value == 0)?;
    let bytes = buffer[..end]
        .iter()
        .map(|value| *value as u8)
        .collect::<Vec<_>>();
    String::from_utf8(bytes).ok()
}

#[cfg(target_os = "windows")]
fn is_unspecified_ip(value: &str) -> bool {
    value == Ipv4Addr::UNSPECIFIED.to_string() || value == Ipv6Addr::UNSPECIFIED.to_string()
}

#[cfg(test)]
fn verified_domain_packet_from_network_packet(packet: &[u8]) -> Option<VerifiedDomainPacket> {
    let parsed = parse_transport_packet(packet)?;
    let detection = netstitch_shared::detect_verified_domain(
        parsed.protocol,
        parsed.remote_port,
        parsed.payload,
    )?;
    Some(VerifiedDomainPacket {
        local_ip: parsed.local_ip.to_string(),
        local_port: parsed.local_port,
        remote_ip: parsed.remote_ip.to_string(),
        remote_port: parsed.remote_port,
        protocol: parsed.protocol,
        domain_name: detection.domain,
        domain_source: detection.source.to_string(),
    })
}

impl DomainPacketAccumulator {
    fn ingest(&mut self, packet: &[u8]) -> DomainPacketIngestResult {
        let Some(parsed) = parse_transport_packet(packet) else {
            return DomainPacketIngestResult::default();
        };
        if parsed.protocol == TransportProtocol::Udp {
            let detection = netstitch_shared::detect_verified_domain(
                parsed.protocol,
                parsed.remote_port,
                parsed.payload,
            );
            return DomainPacketIngestResult {
                tcp_payload_seen: true,
                packet: detection.map(|detection| VerifiedDomainPacket {
                    local_ip: parsed.local_ip.to_string(),
                    local_port: parsed.local_port,
                    remote_ip: parsed.remote_ip.to_string(),
                    remote_port: parsed.remote_port,
                    protocol: parsed.protocol,
                    domain_name: detection.domain,
                    domain_source: detection.source.to_string(),
                }),
            };
        }
        let key = PacketFlowKey {
            protocol: parsed.protocol,
            local_ip: parsed.local_ip,
            local_port: parsed.local_port,
            remote_ip: parsed.remote_ip,
            remote_port: parsed.remote_port,
        };

        let payload = self.pending.entry(key.clone()).or_default();
        if payload.len().saturating_add(parsed.payload.len()) > DOMAIN_PACKET_BUFFER_LIMIT {
            self.pending.remove(&key);
            return DomainPacketIngestResult {
                tcp_payload_seen: true,
                packet: None,
            };
        }
        payload.extend_from_slice(parsed.payload);

        let detection =
            netstitch_shared::detect_verified_domain(parsed.protocol, parsed.remote_port, payload);
        if let Some(detection) = detection {
            self.pending.remove(&key);
            return DomainPacketIngestResult {
                tcp_payload_seen: true,
                packet: Some(VerifiedDomainPacket {
                    local_ip: parsed.local_ip.to_string(),
                    local_port: parsed.local_port,
                    remote_ip: parsed.remote_ip.to_string(),
                    remote_port: parsed.remote_port,
                    protocol: parsed.protocol,
                    domain_name: detection.domain,
                    domain_source: detection.source.to_string(),
                }),
            };
        }

        if http_headers_finished_without_host(parsed.remote_port, payload) {
            self.pending.remove(&key);
        }
        DomainPacketIngestResult {
            tcp_payload_seen: true,
            packet: None,
        }
    }
}

fn http_headers_finished_without_host(remote_port: u16, payload: &[u8]) -> bool {
    remote_port == 80 && payload.windows(4).any(|window| window == b"\r\n\r\n")
}

struct ParsedTransportPacket<'a> {
    local_ip: IpAddr,
    local_port: u16,
    remote_ip: IpAddr,
    remote_port: u16,
    protocol: TransportProtocol,
    payload: &'a [u8],
}

fn parse_transport_packet(packet: &[u8]) -> Option<ParsedTransportPacket<'_>> {
    let version = packet.first()? >> 4;
    match version {
        4 => parse_ipv4_transport_packet(packet),
        6 => parse_ipv6_transport_packet(packet),
        _ => None,
    }
}

fn parse_ipv4_transport_packet(packet: &[u8]) -> Option<ParsedTransportPacket<'_>> {
    if packet.len() < 20 {
        return None;
    }
    let ihl = usize::from(packet[0] & 0x0f) * 4;
    if ihl < 20 {
        return None;
    }
    let total_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
    let packet_end = total_len.min(packet.len());
    if packet_end <= ihl {
        return None;
    }
    let local_ip = IpAddr::V4(Ipv4Addr::new(
        packet[12], packet[13], packet[14], packet[15],
    ));
    let remote_ip = IpAddr::V4(Ipv4Addr::new(
        packet[16], packet[17], packet[18], packet[19],
    ));
    parse_transport_segment(packet[9], local_ip, remote_ip, &packet[ihl..packet_end])
}

fn parse_ipv6_transport_packet(packet: &[u8]) -> Option<ParsedTransportPacket<'_>> {
    if packet.len() < 40 {
        return None;
    }
    let payload_len = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    let packet_end = 40usize.checked_add(payload_len)?.min(packet.len());
    if packet_end <= 40 {
        return None;
    }
    let local_ip = IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&packet[8..24]).ok()?));
    let remote_ip = IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&packet[24..40]).ok()?));
    parse_transport_segment(packet[6], local_ip, remote_ip, &packet[40..packet_end])
}

fn parse_transport_segment(
    protocol: u8,
    local_ip: IpAddr,
    remote_ip: IpAddr,
    segment: &[u8],
) -> Option<ParsedTransportPacket<'_>> {
    match protocol {
        IPPROTO_TCP => parse_tcp_segment(local_ip, remote_ip, segment),
        IPPROTO_UDP => parse_udp_datagram(local_ip, remote_ip, segment),
        _ => None,
    }
}

fn parse_tcp_segment(
    local_ip: IpAddr,
    remote_ip: IpAddr,
    segment: &[u8],
) -> Option<ParsedTransportPacket<'_>> {
    if segment.len() < 20 {
        return None;
    }
    let local_port = u16::from_be_bytes([segment[0], segment[1]]);
    let remote_port = u16::from_be_bytes([segment[2], segment[3]]);
    let data_offset = usize::from(segment[12] >> 4) * 4;
    if data_offset < 20 || segment.len() < data_offset {
        return None;
    }
    let payload = &segment[data_offset..];
    if payload.is_empty() {
        return None;
    }
    Some(ParsedTransportPacket {
        local_ip,
        local_port,
        remote_ip,
        remote_port,
        protocol: TransportProtocol::Tcp,
        payload,
    })
}

fn parse_udp_datagram(
    local_ip: IpAddr,
    remote_ip: IpAddr,
    datagram: &[u8],
) -> Option<ParsedTransportPacket<'_>> {
    if datagram.len() < 8 {
        return None;
    }
    let local_port = u16::from_be_bytes([datagram[0], datagram[1]]);
    let remote_port = u16::from_be_bytes([datagram[2], datagram[3]]);
    if remote_port != 443 {
        return None;
    }
    let datagram_len = u16::from_be_bytes([datagram[4], datagram[5]]) as usize;
    if datagram_len < 8 || datagram.len() < datagram_len {
        return None;
    }
    let payload = &datagram[8..datagram_len];
    if payload.is_empty() {
        return None;
    }
    Some(ParsedTransportPacket {
        local_ip,
        local_port,
        remote_ip,
        remote_port,
        protocol: TransportProtocol::Udp,
        payload,
    })
}

#[cfg(any(target_os = "windows", test))]
fn udp_endpoint_packet_from_network_packet(packet: &[u8]) -> Option<UdpEndpointPacket> {
    let version = packet.first()? >> 4;
    match version {
        4 => udp_endpoint_from_ipv4_packet(packet),
        6 => udp_endpoint_from_ipv6_packet(packet),
        _ => None,
    }
}

#[cfg(any(target_os = "windows", test))]
fn udp_endpoint_from_ipv4_packet(packet: &[u8]) -> Option<UdpEndpointPacket> {
    if packet.len() < 20 {
        return None;
    }
    let ihl = usize::from(packet[0] & 0x0f) * 4;
    if ihl < 20 || packet[9] != IPPROTO_UDP {
        return None;
    }
    let total_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
    let packet_end = total_len.min(packet.len());
    if packet_end < ihl + 8 {
        return None;
    }
    let local_ip = IpAddr::V4(Ipv4Addr::new(
        packet[12], packet[13], packet[14], packet[15],
    ));
    let remote_ip = IpAddr::V4(Ipv4Addr::new(
        packet[16], packet[17], packet[18], packet[19],
    ));
    udp_endpoint_from_datagram(local_ip, remote_ip, &packet[ihl..packet_end])
}

#[cfg(any(target_os = "windows", test))]
fn udp_endpoint_from_ipv6_packet(packet: &[u8]) -> Option<UdpEndpointPacket> {
    if packet.len() < 40 || packet[6] != IPPROTO_UDP {
        return None;
    }
    let payload_len = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    let packet_end = 40usize.checked_add(payload_len)?.min(packet.len());
    if packet_end < 48 {
        return None;
    }
    let local_ip = IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&packet[8..24]).ok()?));
    let remote_ip = IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&packet[24..40]).ok()?));
    udp_endpoint_from_datagram(local_ip, remote_ip, &packet[40..packet_end])
}

#[cfg(any(target_os = "windows", test))]
fn udp_endpoint_from_datagram(
    local_ip: IpAddr,
    remote_ip: IpAddr,
    datagram: &[u8],
) -> Option<UdpEndpointPacket> {
    if datagram.len() < 8 || remote_ip.is_unspecified() {
        return None;
    }
    let local_port = u16::from_be_bytes([datagram[0], datagram[1]]);
    let remote_port = u16::from_be_bytes([datagram[2], datagram[3]]);
    let datagram_len = u16::from_be_bytes([datagram[4], datagram[5]]) as usize;
    if datagram_len < 8 || datagram.len() < datagram_len {
        return None;
    }
    Some(UdpEndpointPacket {
        local_ip: local_ip.to_string(),
        local_port,
        remote_ip: remote_ip.to_string(),
        remote_port,
    })
}

#[cfg(target_os = "windows")]
fn load_windivert_api() -> Result<WinDivertApi> {
    let module = load_windivert_library()?;
    Ok(WinDivertApi {
        open: load_required_symbol(module, "WinDivertOpen")?,
        recv: load_required_symbol(module, "WinDivertRecv")?,
        close: load_required_symbol(module, "WinDivertClose")?,
        shutdown: load_optional_symbol(module, "WinDivertShutdown"),
        format_ipv6: load_optional_symbol(module, "WinDivertHelperFormatIPv6Address"),
    })
}

#[cfg(target_os = "windows")]
fn load_windivert_library() -> Result<windows::Win32::Foundation::HMODULE> {
    let mut candidates = Vec::new();
    if let Ok(explicit) = std::env::var("NETSTITCH__WINDIVERT_DLL") {
        if !explicit.trim().is_empty() {
            candidates.push(explicit);
        }
    }
    candidates.push("WinDivert.dll".to_string());
    candidates.push("WinDivert64.dll".to_string());

    let mut last_error = None;
    for candidate in candidates {
        let wide = candidate
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        match unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) } {
            Ok(module) => return Ok(module),
            Err(error) => last_error = Some(error),
        }
    }

    Err(anyhow!(
        "failed to load WinDivert.dll/WinDivert64.dll: {}",
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "no candidate loaded".to_string())
    ))
}

#[cfg(target_os = "windows")]
fn load_required_symbol<T>(module: windows::Win32::Foundation::HMODULE, name: &str) -> Result<T>
where
    T: Copy,
{
    load_optional_symbol(module, name)
        .ok_or_else(|| anyhow!("required WinDivert symbol is missing: {name}"))
}

#[cfg(target_os = "windows")]
fn load_optional_symbol<T>(module: windows::Win32::Foundation::HMODULE, name: &str) -> Option<T>
where
    T: Copy,
{
    let symbol_name = CString::new(name).ok()?;
    let proc = unsafe { GetProcAddress(module, PCSTR(symbol_name.as_ptr().cast()))? };
    Some(unsafe { std::mem::transmute_copy(&proc) })
}

#[cfg(target_os = "windows")]
fn collect_raw_observations() -> Result<Vec<RawObservation>> {
    let mut observations = collect_tcp_v4()?;
    observations.extend(collect_tcp_v6()?);
    Ok(observations)
}

#[cfg(target_os = "linux")]
fn collect_raw_observations() -> Result<Vec<RawObservation>> {
    let inode_pids = linux_socket_inode_pid_map();
    let mut observations = collect_linux_proc_net(
        Path::new("/proc/net/tcp"),
        TransportProtocol::Tcp,
        false,
        &inode_pids,
    )?;
    observations.extend(collect_linux_proc_net(
        Path::new("/proc/net/tcp6"),
        TransportProtocol::Tcp,
        true,
        &inode_pids,
    )?);
    observations.extend(collect_linux_proc_net(
        Path::new("/proc/net/udp"),
        TransportProtocol::Udp,
        false,
        &inode_pids,
    )?);
    observations.extend(collect_linux_proc_net(
        Path::new("/proc/net/udp6"),
        TransportProtocol::Udp,
        true,
        &inode_pids,
    )?);
    Ok(observations)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn collect_raw_observations() -> Result<Vec<RawObservation>> {
    Ok(Vec::new())
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct LinuxProcSocketRow {
    local_ip: String,
    local_port: u16,
    remote_ip: String,
    remote_port: u16,
    state: String,
    inode: u64,
}

#[cfg(target_os = "linux")]
fn collect_linux_proc_net(
    path: &Path,
    protocol: TransportProtocol,
    ipv6: bool,
    inode_pids: &HashMap<u64, u32>,
) -> Result<Vec<RawObservation>> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let rows = parse_linux_proc_net_rows(&contents, ipv6);
    let mut observations = Vec::new();
    for row in rows {
        if linux_remote_is_unspecified(&row.remote_ip, row.remote_port) {
            continue;
        }
        let Some(pid) = inode_pids.get(&row.inode).copied() else {
            continue;
        };
        let connection_state = match protocol {
            TransportProtocol::Tcp => linux_tcp_state_to_connection_state(&row.state),
            TransportProtocol::Udp => ConnectionState::Attempting,
            TransportProtocol::Other => ConnectionState::Unknown,
        };
        observations.push(RawObservation {
            pid,
            local_ip: Some(row.local_ip),
            local_port: Some(row.local_port),
            remote_ip: row.remote_ip,
            remote_port: row.remote_port,
            protocol: protocol.clone(),
            connection_state,
            origin: ObservationOrigin::TcpTable,
        });
    }
    Ok(observations)
}

#[cfg(target_os = "linux")]
fn parse_linux_proc_net_rows(contents: &str, ipv6: bool) -> Vec<LinuxProcSocketRow> {
    contents
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(1)?;
            let remote = parts.get(2)?;
            let state = parts.get(3)?.to_string();
            let inode = parts.get(9)?.parse::<u64>().ok()?;
            let (local_ip, local_port) = parse_linux_proc_endpoint(local, ipv6)?;
            let (remote_ip, remote_port) = parse_linux_proc_endpoint(remote, ipv6)?;
            Some(LinuxProcSocketRow {
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                state,
                inode,
            })
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn parse_linux_proc_endpoint(value: &str, ipv6: bool) -> Option<(String, u16)> {
    let (address_hex, port_hex) = value.split_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let address = if ipv6 {
        parse_linux_proc_ipv6(address_hex)?.to_string()
    } else {
        let raw = u32::from_str_radix(address_hex, 16).ok()?;
        Ipv4Addr::from(raw.to_le_bytes()).to_string()
    };
    Some((address, port))
}

#[cfg(target_os = "linux")]
fn parse_linux_proc_ipv6(value: &str) -> Option<Ipv6Addr> {
    if value.len() != 32 {
        return None;
    }
    let mut bytes = [0_u8; 16];
    for index in 0..4 {
        let start = index * 8;
        let word = u32::from_str_radix(&value[start..start + 8], 16).ok()?;
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    Some(Ipv6Addr::from(bytes))
}

#[cfg(target_os = "linux")]
fn linux_remote_is_unspecified(ip: &str, port: u16) -> bool {
    port == 0 || ip == Ipv4Addr::UNSPECIFIED.to_string() || ip == Ipv6Addr::UNSPECIFIED.to_string()
}

#[cfg(target_os = "linux")]
fn linux_tcp_state_to_connection_state(state_hex: &str) -> ConnectionState {
    match u8::from_str_radix(state_hex, 16).unwrap_or_default() {
        0x01 => ConnectionState::Established,
        0x02 | 0x03 => ConnectionState::Attempting,
        0x04 | 0x05 | 0x06 | 0x08 | 0x09 | 0x0B => ConnectionState::Closing,
        0x07 => ConnectionState::Failed,
        _ => ConnectionState::Unknown,
    }
}

#[cfg(target_os = "linux")]
fn linux_socket_inode_pid_map() -> HashMap<u64, u32> {
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return map;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let fd_dir = entry.path().join("fd");
        let Ok(fds) = fs::read_dir(fd_dir) else {
            continue;
        };
        for fd in fds.flatten() {
            let Ok(target) = fs::read_link(fd.path()) else {
                continue;
            };
            let Some(inode) = linux_socket_inode_from_link(&target.to_string_lossy()) else {
                continue;
            };
            map.entry(inode).or_insert(pid);
        }
    }
    map
}

#[cfg(target_os = "linux")]
fn linux_socket_inode_from_link(value: &str) -> Option<u64> {
    value
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

#[cfg(target_os = "windows")]
fn collect_tcp_v4() -> Result<Vec<RawObservation>> {
    unsafe {
        let buffer = read_tcp_table(AF_INET.0 as u32, TCP_TABLE_OWNER_PID_ALL)?;
        let table = &*(buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        let mut observations = Vec::new();

        for row in rows {
            let local_ip = Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string();
            let remote_ip = Ipv4Addr::from(u32::from_be(row.dwRemoteAddr)).to_string();
            if remote_ip == Ipv4Addr::UNSPECIFIED.to_string() {
                continue;
            }

            observations.push(RawObservation {
                pid: row.dwOwningPid,
                local_ip: Some(local_ip),
                local_port: Some(decode_port(row.dwLocalPort)),
                remote_ip,
                remote_port: decode_port(row.dwRemotePort),
                protocol: TransportProtocol::Tcp,
                connection_state: tcp_state_to_connection_state(row.dwState),
                origin: ObservationOrigin::TcpTable,
            });
        }

        Ok(observations)
    }
}

#[cfg(target_os = "windows")]
fn collect_tcp_v6() -> Result<Vec<RawObservation>> {
    unsafe {
        let buffer = read_tcp_table(AF_INET6.0 as u32, TCP_TABLE_OWNER_PID_ALL)?;
        let table = &*(buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        let mut observations = Vec::new();

        for row in rows {
            let local_ip = Ipv6Addr::from(row.ucLocalAddr).to_string();
            let remote_ip = Ipv6Addr::from(row.ucRemoteAddr).to_string();
            if remote_ip == Ipv6Addr::UNSPECIFIED.to_string() {
                continue;
            }

            observations.push(RawObservation {
                pid: row.dwOwningPid,
                local_ip: Some(local_ip),
                local_port: Some(decode_port(row.dwLocalPort)),
                remote_ip,
                remote_port: decode_port(row.dwRemotePort),
                protocol: TransportProtocol::Tcp,
                connection_state: tcp_state_to_connection_state(row.dwState),
                origin: ObservationOrigin::TcpTable,
            });
        }

        Ok(observations)
    }
}

#[cfg(target_os = "windows")]
fn collect_udp_owner_sockets() -> Result<Vec<UdpOwnerSocket>> {
    let mut sockets = collect_udp_v4_owner_sockets()?;
    sockets.extend(collect_udp_v6_owner_sockets()?);
    Ok(sockets)
}

#[cfg(target_os = "linux")]
fn collect_udp_owner_sockets() -> Result<Vec<UdpOwnerSocket>> {
    let inode_pids = linux_socket_inode_pid_map();
    let mut sockets =
        collect_linux_udp_owner_sockets(Path::new("/proc/net/udp"), false, &inode_pids)?;
    sockets.extend(collect_linux_udp_owner_sockets(
        Path::new("/proc/net/udp6"),
        true,
        &inode_pids,
    )?);
    Ok(sockets)
}

#[cfg(target_os = "linux")]
fn collect_linux_udp_owner_sockets(
    path: &Path,
    ipv6: bool,
    inode_pids: &HashMap<u64, u32>,
) -> Result<Vec<UdpOwnerSocket>> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(parse_linux_proc_net_rows(&contents, ipv6)
        .into_iter()
        .filter_map(|row| {
            Some(UdpOwnerSocket {
                local_ip: row.local_ip,
                local_port: row.local_port,
                pid: inode_pids.get(&row.inode).copied()?,
            })
        })
        .collect())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn collect_udp_owner_sockets() -> Result<Vec<UdpOwnerSocket>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
fn collect_udp_v4_owner_sockets() -> Result<Vec<UdpOwnerSocket>> {
    unsafe {
        let buffer = read_udp_table(AF_INET.0 as u32, UDP_TABLE_OWNER_PID)?;
        let table = &*(buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        let mut sockets = Vec::new();

        for row in rows {
            sockets.push(UdpOwnerSocket {
                pid: row.dwOwningPid,
                local_ip: Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string(),
                local_port: decode_port(row.dwLocalPort),
            });
        }

        Ok(sockets)
    }
}

#[cfg(target_os = "windows")]
fn collect_udp_v6_owner_sockets() -> Result<Vec<UdpOwnerSocket>> {
    unsafe {
        let buffer = read_udp_table(AF_INET6.0 as u32, UDP_TABLE_OWNER_PID)?;
        let table = &*(buffer.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        let mut sockets = Vec::new();

        for row in rows {
            sockets.push(UdpOwnerSocket {
                pid: row.dwOwningPid,
                local_ip: Ipv6Addr::from(row.ucLocalAddr).to_string(),
                local_port: decode_port(row.dwLocalPort),
            });
        }

        Ok(sockets)
    }
}

#[cfg(target_os = "windows")]
unsafe fn read_tcp_table(family: u32, class: TCP_TABLE_CLASS) -> Result<Vec<u8>> {
    let mut size = 0u32;
    let first = unsafe { GetExtendedTcpTable(None, &mut size, false, family, class, 0) };
    if first != ERROR_INSUFFICIENT_BUFFER.0 {
        return Err(anyhow!(
            "GetExtendedTcpTable initial call failed: {first:?}"
        ));
    }

    let mut buffer = vec![0u8; size as usize];
    let second = unsafe {
        GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            family,
            class,
            0,
        )
    };
    if second != NO_ERROR.0 {
        return Err(anyhow!("GetExtendedTcpTable failed: {second:?}"));
    }
    Ok(buffer)
}

#[cfg(target_os = "windows")]
unsafe fn read_udp_table(family: u32, class: UDP_TABLE_CLASS) -> Result<Vec<u8>> {
    let mut size = 0u32;
    let first = unsafe { GetExtendedUdpTable(None, &mut size, false, family, class, 0) };
    if first != ERROR_INSUFFICIENT_BUFFER.0 {
        return Err(anyhow!(
            "GetExtendedUdpTable initial call failed: {first:?}"
        ));
    }

    let mut buffer = vec![0u8; size as usize];
    let second = unsafe {
        GetExtendedUdpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            false,
            family,
            class,
            0,
        )
    };
    if second != NO_ERROR.0 {
        return Err(anyhow!("GetExtendedUdpTable failed: {second:?}"));
    }
    Ok(buffer)
}

fn build_udp_owner_index(sockets: &[UdpOwnerSocket]) -> UdpOwnerIndex {
    let mut index = UdpOwnerIndex::default();
    for socket in sockets {
        index.exact.insert(
            UdpOwnerKey {
                local_ip: socket.local_ip.clone(),
                local_port: socket.local_port,
            },
            socket.pid,
        );
        index
            .port_only
            .entry(socket.local_port)
            .and_modify(|pid| {
                if *pid != Some(socket.pid) {
                    *pid = None;
                }
            })
            .or_insert(Some(socket.pid));
    }
    index
}

fn owner_pid_for_udp_packet(packet: &UdpEndpointPacket, index: &UdpOwnerIndex) -> Option<u32> {
    let exact = UdpOwnerKey {
        local_ip: packet.local_ip.clone(),
        local_port: packet.local_port,
    };
    if let Some(pid) = index.exact.get(&exact) {
        return Some(*pid);
    }

    let wildcard = if packet.local_ip.contains(':') {
        "::".to_string()
    } else {
        Ipv4Addr::UNSPECIFIED.to_string()
    };
    let wildcard_key = UdpOwnerKey {
        local_ip: wildcard,
        local_port: packet.local_port,
    };
    if let Some(pid) = index.exact.get(&wildcard_key) {
        return Some(*pid);
    }

    index.port_only.get(&packet.local_port).and_then(|pid| *pid)
}

#[cfg(target_os = "windows")]
fn decode_port(raw_port: u32) -> u16 {
    u16::from_be((raw_port & 0xFFFF) as u16)
}

#[cfg(test)]
mod matcher_tests {
    use super::{ProcessMetadata, build_matchers_with_connector_process_names, match_tracked_app};
    use netstitch_shared::models::TrackedApp;
    use std::collections::{BTreeSet, HashMap};
    use std::path::PathBuf;

    #[test]
    fn connector_aliases_match_linux_chrome_runtime_process() {
        let tracked_app = TrackedApp {
            id: Some(7),
            exe_path: PathBuf::from("/opt/google/chrome/google-chrome"),
            connector_id: Some("chrome".to_string()),
            cloud_app_id: None,
            process_name: Some("google-chrome".to_string()),
            display_name: Some("Google Chrome".to_string()),
            icon_key: Some("chrome".to_string()),
            icon_path: None,
            current_tag: None,
            enabled: true,
            created_at_ms: 1,
        };
        let connector_process_names = HashMap::from([(
            "chrome".to_string(),
            BTreeSet::from(["chrome".to_string(), "google-chrome".to_string()]),
        )]);
        let matchers =
            build_matchers_with_connector_process_names(&[tracked_app], &connector_process_names);
        let process = ProcessMetadata {
            exe_path: Some(PathBuf::from("/opt/google/chrome/chrome")),
            process_name: "chrome".to_string(),
        };

        let matched = match_tracked_app(&matchers, &process).expect("chrome process matches");

        assert_eq!(matched.id, Some(7));
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{
        IPPROTO_TCP, IPPROTO_UDP, ObservationOrigin, RawObservation, TcpOwnerKey,
        UdpEndpointPacket, UdpOwnerSocket, WINDIVERT_EVENT_FLOW_DELETED,
        WINDIVERT_EVENT_FLOW_ESTABLISHED, build_udp_owner_index, flow_event_to_connection_state,
        owner_pid_for_udp_packet, tcp_state_to_connection_state, update_recent_tcp_owner_index,
        windivert_event, windivert_protocol_to_transport,
    };
    use netstitch_shared::models::{ConnectionState, TransportProtocol};
    use std::collections::HashMap;

    #[test]
    fn tcp_state_mapping_marks_non_established_attempts_as_failure_like() {
        assert_eq!(
            tcp_state_to_connection_state(3),
            ConnectionState::Attempting
        );
        assert_eq!(tcp_state_to_connection_state(12), ConnectionState::Failed);
        assert_eq!(
            tcp_state_to_connection_state(5),
            ConnectionState::Established
        );
    }

    #[test]
    fn windivert_flow_mapping_treats_udp_as_attempt_candidates() {
        assert_eq!(
            windivert_protocol_to_transport(IPPROTO_UDP),
            Some(TransportProtocol::Udp)
        );
        assert_eq!(
            flow_event_to_connection_state(IPPROTO_UDP, WINDIVERT_EVENT_FLOW_ESTABLISHED),
            Some(ConnectionState::Attempting)
        );
        assert_eq!(
            flow_event_to_connection_state(IPPROTO_UDP, WINDIVERT_EVENT_FLOW_DELETED),
            Some(ConnectionState::Closing)
        );
    }

    #[test]
    fn windivert_flow_mapping_keeps_tcp_established_success_like() {
        assert_eq!(
            windivert_protocol_to_transport(IPPROTO_TCP),
            Some(TransportProtocol::Tcp)
        );
        assert_eq!(
            flow_event_to_connection_state(IPPROTO_TCP, WINDIVERT_EVENT_FLOW_ESTABLISHED),
            Some(ConnectionState::Established)
        );
        assert_eq!(
            flow_event_to_connection_state(IPPROTO_TCP, WINDIVERT_EVENT_FLOW_DELETED),
            Some(ConnectionState::Closing)
        );
    }

    #[test]
    fn recent_tcp_owner_index_keeps_flow_keys_for_later_domain_matching() {
        let observation = RawObservation {
            pid: 42,
            local_ip: Some("192.168.1.10".to_string()),
            local_port: Some(50000),
            remote_ip: "203.0.113.20".to_string(),
            remote_port: 443,
            protocol: TransportProtocol::Tcp,
            connection_state: ConnectionState::Established,
            origin: ObservationOrigin::WinDivertFlow,
        };
        let mut index = HashMap::new();

        update_recent_tcp_owner_index(&mut index, &[observation], 12_345);

        assert_eq!(
            index.get(&TcpOwnerKey {
                local_ip: "192.168.1.10".to_string(),
                local_port: 50000,
                remote_ip: "203.0.113.20".to_string(),
                remote_port: 443,
            }),
            Some(&(42, 12_345))
        );
    }

    #[test]
    fn windivert_address_bitfield_helper_decodes_event() {
        let bitfield = (u64::from(WINDIVERT_EVENT_FLOW_ESTABLISHED) << 8) | (1 << 20);
        assert_eq!(windivert_event(bitfield), WINDIVERT_EVENT_FLOW_ESTABLISHED);
    }

    #[test]
    fn udp_owner_index_matches_exact_and_wildcard_local_sockets() {
        let exact_packet = UdpEndpointPacket {
            local_ip: "192.168.69.15".to_string(),
            local_port: 64090,
            remote_ip: "34.79.201.95".to_string(),
            remote_port: 64337,
        };
        let wildcard_packet = UdpEndpointPacket {
            local_ip: "192.168.69.15".to_string(),
            local_port: 64091,
            remote_ip: "34.79.201.95".to_string(),
            remote_port: 64337,
        };
        let index = build_udp_owner_index(&[
            UdpOwnerSocket {
                local_ip: "192.168.69.15".to_string(),
                local_port: 64090,
                pid: 100,
            },
            UdpOwnerSocket {
                local_ip: "0.0.0.0".to_string(),
                local_port: 64091,
                pid: 101,
            },
        ]);

        assert_eq!(owner_pid_for_udp_packet(&exact_packet, &index), Some(100));
        assert_eq!(
            owner_pid_for_udp_packet(&wildcard_packet, &index),
            Some(101)
        );
    }

    #[test]
    fn udp_owner_index_rejects_ambiguous_port_only_match() {
        let packet = UdpEndpointPacket {
            local_ip: "192.168.69.15".to_string(),
            local_port: 64090,
            remote_ip: "34.79.201.95".to_string(),
            remote_port: 64337,
        };
        let index = build_udp_owner_index(&[
            UdpOwnerSocket {
                local_ip: "192.168.69.16".to_string(),
                local_port: 64090,
                pid: 100,
            },
            UdpOwnerSocket {
                local_ip: "192.168.69.17".to_string(),
                local_port: 64090,
                pid: 101,
            },
        ]);

        assert_eq!(owner_pid_for_udp_packet(&packet, &index), None);
    }

    #[test]
    fn mapped_ipv6_addresses_are_normalized_to_ipv4() {
        assert_eq!(
            super::normalize_mapped_ip("::ffff:203.0.113.45".to_string()),
            Some("203.0.113.45".to_string())
        );
        assert_eq!(
            super::normalize_mapped_ip("2001:db8::1".to_string()),
            Some("2001:db8::1".to_string())
        );
    }
}

#[cfg(all(test, target_os = "linux"))]
mod linux_proc_tests {
    use super::{
        LINUX_ETH_P_8021Q, LINUX_ETH_P_IP, linux_ip_packet_from_link_frame,
        linux_socket_inode_from_link, linux_tcp_state_to_connection_state,
        parse_linux_proc_endpoint, parse_linux_proc_net_rows,
    };
    use netstitch_shared::models::ConnectionState;

    #[test]
    fn proc_net_ipv4_endpoint_is_little_endian() {
        let (ip, port) =
            parse_linux_proc_endpoint("0100007F:1F90", false).expect("endpoint parses");

        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn proc_net_ipv6_endpoint_is_word_little_endian() {
        let (ip, port) = parse_linux_proc_endpoint("00000000000000000000000001000000:01BB", true)
            .expect("endpoint parses");

        assert_eq!(ip, "::1");
        assert_eq!(port, 443);
    }

    #[test]
    fn proc_net_rows_extract_inode_and_state() {
        let rows = parse_linux_proc_net_rows(
            "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n\
             0: 0100007F:9C40 0200007F:01BB 01 00000000:00000000 00:00000000 00000000 1000 0 424242 1 0000000000000000 100 0 0 10 0\n",
            false,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].local_ip, "127.0.0.1");
        assert_eq!(rows[0].remote_ip, "127.0.0.2");
        assert_eq!(rows[0].local_port, 40000);
        assert_eq!(rows[0].remote_port, 443);
        assert_eq!(rows[0].state, "01");
        assert_eq!(rows[0].inode, 424242);
    }

    #[test]
    fn linux_tcp_states_map_to_shared_connection_states() {
        assert_eq!(
            linux_tcp_state_to_connection_state("01"),
            ConnectionState::Established
        );
        assert_eq!(
            linux_tcp_state_to_connection_state("02"),
            ConnectionState::Attempting
        );
        assert_eq!(
            linux_tcp_state_to_connection_state("06"),
            ConnectionState::Closing
        );
        assert_eq!(
            linux_tcp_state_to_connection_state("07"),
            ConnectionState::Failed
        );
    }

    #[test]
    fn socket_inode_link_is_parsed() {
        assert_eq!(
            linux_socket_inode_from_link("socket:[123456]"),
            Some(123456)
        );
        assert_eq!(linux_socket_inode_from_link("pipe:[123456]"), None);
    }

    #[test]
    fn linux_packet_domain_unwraps_ethernet_ipv4_payload() {
        let ip_packet = minimal_ipv4_packet();
        let mut frame = vec![0_u8; 14];
        frame[12..14].copy_from_slice(&LINUX_ETH_P_IP.to_be_bytes());
        frame.extend_from_slice(&ip_packet);

        assert_eq!(
            linux_ip_packet_from_link_frame(&frame).expect("ip payload"),
            ip_packet.as_slice()
        );
    }

    #[test]
    fn linux_packet_domain_unwraps_vlan_ipv4_payload() {
        let ip_packet = minimal_ipv4_packet();
        let mut frame = vec![0_u8; 18];
        frame[12..14].copy_from_slice(&LINUX_ETH_P_8021Q.to_be_bytes());
        frame[16..18].copy_from_slice(&LINUX_ETH_P_IP.to_be_bytes());
        frame.extend_from_slice(&ip_packet);

        assert_eq!(
            linux_ip_packet_from_link_frame(&frame).expect("vlan ip payload"),
            ip_packet.as_slice()
        );
    }

    fn minimal_ipv4_packet() -> Vec<u8> {
        let mut packet = vec![0_u8; 20];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&(20_u16).to_be_bytes());
        packet[8] = 64;
        packet[9] = super::IPPROTO_TCP;
        packet[12..16].copy_from_slice(&[192, 168, 1, 10]);
        packet[16..20].copy_from_slice(&[203, 0, 113, 20]);
        packet
    }
}

#[cfg(test)]
mod packet_domain_tests {
    use super::*;

    #[test]
    fn outbound_ipv4_http_host_packet_yields_verified_domain() {
        let payload = b"GET / HTTP/1.1\r\nHost: Game.Example:8080\r\n\r\n";
        let packet = ipv4_tcp_packet([192, 168, 1, 10], [203, 0, 113, 20], 49152, 8080, payload);

        let detected =
            verified_domain_packet_from_network_packet(&packet).expect("domain should parse");

        assert_eq!(detected.local_ip, "192.168.1.10");
        assert_eq!(detected.local_port, 49152);
        assert_eq!(detected.remote_ip, "203.0.113.20");
        assert_eq!(detected.remote_port, 8080);
        assert_eq!(detected.protocol, TransportProtocol::Tcp);
        assert_eq!(detected.domain_name, "game.example");
        assert_eq!(
            detected.domain_source,
            netstitch_shared::DOMAIN_SOURCE_HTTP_HOST
        );
    }

    #[test]
    fn outbound_ipv4_udp_quic_initial_packet_yields_verified_domain() {
        let payload = netstitch_shared::test_quic_initial_datagram("Quic.Game.Example");
        let packet = ipv4_udp_packet([192, 168, 1, 10], [203, 0, 113, 20], 49152, 443, &payload);

        let detected =
            verified_domain_packet_from_network_packet(&packet).expect("QUIC SNI should parse");

        assert_eq!(detected.local_ip, "192.168.1.10");
        assert_eq!(detected.local_port, 49152);
        assert_eq!(detected.remote_ip, "203.0.113.20");
        assert_eq!(detected.remote_port, 443);
        assert_eq!(detected.protocol, TransportProtocol::Udp);
        assert_eq!(detected.domain_name, "quic.game.example");
        assert_eq!(
            detected.domain_source,
            netstitch_shared::DOMAIN_SOURCE_QUIC_SNI
        );
    }

    #[test]
    fn outbound_ipv4_udp_packet_yields_endpoint_for_any_remote_port() {
        let packet = ipv4_udp_packet(
            [192, 168, 69, 15],
            [34, 79, 201, 95],
            64090,
            64337,
            b"game payload",
        );

        let endpoint =
            udp_endpoint_packet_from_network_packet(&packet).expect("UDP endpoint should parse");

        assert_eq!(endpoint.local_ip, "192.168.69.15");
        assert_eq!(endpoint.local_port, 64090);
        assert_eq!(endpoint.remote_ip, "34.79.201.95");
        assert_eq!(endpoint.remote_port, 64337);
    }

    #[test]
    fn packet_without_verified_domain_is_ignored() {
        let packet = ipv4_tcp_packet([192, 168, 1, 10], [203, 0, 113, 20], 49152, 443, b"\x17");

        assert!(verified_domain_packet_from_network_packet(&packet).is_none());
    }

    #[test]
    fn split_http_host_payload_is_detected_after_accumulation() {
        let mut accumulator = DomainPacketAccumulator::default();
        let first = ipv4_tcp_packet(
            [192, 168, 1, 10],
            [203, 0, 113, 20],
            49152,
            80,
            b"GET / HTTP/1.1\r\nHo",
        );
        let second = ipv4_tcp_packet(
            [192, 168, 1, 10],
            [203, 0, 113, 20],
            49152,
            80,
            b"st: Split.Example\r\n\r\n",
        );

        let first_result = accumulator.ingest(&first);
        assert!(first_result.tcp_payload_seen);
        assert!(first_result.packet.is_none());
        let detected = accumulator
            .ingest(&second)
            .packet
            .expect("split Host header should parse");

        assert_eq!(detected.domain_name, "split.example");
        assert_eq!(
            detected.domain_source,
            netstitch_shared::DOMAIN_SOURCE_HTTP_HOST
        );
    }

    fn ipv4_udp_packet(
        src: [u8; 4],
        dst: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let udp_len = 8 + payload.len();
        let total_len = 20 + udp_len;
        let mut packet = vec![0u8; total_len];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&(total_len as u16).to_be_bytes());
        packet[8] = 64;
        packet[9] = IPPROTO_UDP;
        packet[12..16].copy_from_slice(&src);
        packet[16..20].copy_from_slice(&dst);
        let udp = 20;
        packet[udp..udp + 2].copy_from_slice(&src_port.to_be_bytes());
        packet[udp + 2..udp + 4].copy_from_slice(&dst_port.to_be_bytes());
        packet[udp + 4..udp + 6].copy_from_slice(&(udp_len as u16).to_be_bytes());
        packet[udp + 8..].copy_from_slice(payload);
        packet
    }

    fn ipv4_tcp_packet(
        src: [u8; 4],
        dst: [u8; 4],
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let total_len = 20 + 20 + payload.len();
        let mut packet = vec![0u8; total_len];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&(total_len as u16).to_be_bytes());
        packet[8] = 64;
        packet[9] = IPPROTO_TCP;
        packet[12..16].copy_from_slice(&src);
        packet[16..20].copy_from_slice(&dst);
        let tcp = 20;
        packet[tcp..tcp + 2].copy_from_slice(&src_port.to_be_bytes());
        packet[tcp + 2..tcp + 4].copy_from_slice(&dst_port.to_be_bytes());
        packet[tcp + 12] = 5 << 4;
        packet[tcp + 20..].copy_from_slice(payload);
        packet
    }
}
