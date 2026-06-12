use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use netstitch_core::{IpDomainCacheRecord, NetstitchCore, WhoisRangeCacheRecord};
use netstitch_shared::models::{
    EndpointProbeStatusDto, EndpointProbeTargetDto, IntegrationAbiBuffer, IntegrationHostRequest,
    IntegrationHostResponse, Protocol,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket, lookup_host};
use tokio::time::timeout;

const DEFAULT_LIMIT: usize = 16;
const DEFAULT_RETRY_AFTER_MS: u64 = 60 * 60 * 1000;
const DNS_TIMEOUT: Duration = Duration::from_secs(4);
const ENDPOINT_PROBE_TIMEOUT: Duration = Duration::from_secs(4);
const WHOIS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProbeTargetSpec {
    target: String,
    protocol: Protocol,
}

#[derive(Debug, Clone, Serialize)]
struct LookupSummary {
    ip: IpAddr,
    domain_name: Option<String>,
    owner_name: Option<String>,
    owner_range: Option<String>,
    registry: Option<String>,
    country: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DomainLookupSummary {
    domain: String,
    resolved_ips: Vec<IpAddr>,
    addresses: Vec<LookupSummary>,
}

#[derive(Debug, Deserialize)]
struct ToolOncePayload {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default = "default_retry_after_ms")]
    retry_after_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ToolLookupPayload {
    ip: IpAddr,
}

#[derive(Debug, Deserialize)]
struct ToolDomainLookupPayload {
    domain: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ToolProbeEndpointPayload {
    Targets { targets: Vec<String> },
    TargetList(Vec<String>),
}

fn default_limit() -> usize {
    DEFAULT_LIMIT
}

fn default_retry_after_ms() -> u64 {
    DEFAULT_RETRY_AFTER_MS
}

pub async fn enrich_pending_once(
    core: &NetstitchCore,
    limit: usize,
    retry_after_ms: u64,
) -> Result<usize> {
    let client = build_http_client()?;
    enrich_once(core, &client, limit, retry_after_ms).await
}

pub async fn probe_endpoint_target_strings(targets: &[String]) -> Result<EndpointProbeStatusDto> {
    if targets.is_empty() {
        return Err(anyhow!(
            "endpoint probe requires at least one host:port target supplied by the caller"
        ));
    }
    let parsed = targets
        .iter()
        .map(|target| parse_probe_target_spec(target))
        .collect::<Result<Vec<_>>>()?;
    Ok(probe_endpoint_targets(&parsed, ENDPOINT_PROBE_TIMEOUT).await)
}

fn build_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(DNS_TIMEOUT)
        .user_agent("NetStitch network tool module")
        .build()
        .context("failed to build DNS HTTP client")
}

async fn enrich_once(
    core: &NetstitchCore,
    client: &reqwest::Client,
    limit: usize,
    retry_after_ms: u64,
) -> Result<usize> {
    let targets = core.pending_ip_enrichment_targets(limit.max(1), retry_after_ms)?;
    let mut enriched = 0;
    for ip in targets {
        enrich_ip(core, client, ip).await?;
        enriched += 1;
    }
    Ok(enriched)
}

async fn enrich_ip(
    core: &NetstitchCore,
    _client: &reqwest::Client,
    ip: IpAddr,
) -> Result<LookupSummary> {
    let now = current_timestamp_ms();
    core.upsert_ip_domain_cache(IpDomainCacheRecord {
        remote_ip: ip,
        lookup_status: "requires_payload".to_string(),
        error_text: None,
        last_attempt_ms: now,
        updated_at_ms: None,
    })?;

    let whois = team_cymru_lookup(ip).await.ok().flatten();
    if let Some(record) = whois.as_ref() {
        core.upsert_whois_range_cache(record.clone())?;
    }

    Ok(LookupSummary {
        ip,
        domain_name: None,
        owner_name: whois.as_ref().and_then(|record| record.owner_name.clone()),
        owner_range: whois.as_ref().map(|record| record.cidr.clone()),
        registry: whois.as_ref().and_then(|record| record.registry.clone()),
        country: whois.as_ref().and_then(|record| record.country.clone()),
        source: whois.and_then(|record| record.source),
    })
}

async fn lookup_domain(
    core: &NetstitchCore,
    client: &reqwest::Client,
    domain: String,
) -> Result<DomainLookupSummary> {
    let resolved_ips = resolve_domain_ips(&domain).await?;
    let mut addresses = Vec::with_capacity(resolved_ips.len());
    for ip in resolved_ips.iter().copied() {
        addresses.push(enrich_ip(core, client, ip).await?);
    }
    Ok(DomainLookupSummary {
        domain,
        resolved_ips,
        addresses,
    })
}

async fn resolve_domain_ips(domain: &str) -> Result<Vec<IpAddr>> {
    let domain = domain.trim();
    if domain.is_empty() {
        return Err(anyhow!("domain lookup requires a non-empty domain"));
    }

    let addresses = timeout(DNS_TIMEOUT, lookup_host((domain, 0)))
        .await
        .context("domain DNS lookup timed out")?
        .context("domain DNS lookup failed")?;
    Ok(addresses
        .map(|address| address.ip())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

async fn probe_endpoint_targets(
    targets: &[ProbeTargetSpec],
    timeout_per_target: Duration,
) -> EndpointProbeStatusDto {
    let mut tasks = Vec::with_capacity(targets.len());
    for (index, target) in targets.iter().cloned().enumerate() {
        let formatted_target = format_probe_target(&target);
        let handle =
            tokio::spawn(async move { probe_endpoint_target(&target, timeout_per_target).await });
        tasks.push((index, formatted_target, handle));
    }

    let mut status = EndpointProbeStatusDto {
        is_checking: false,
        first_successful_target: None,
        probes: Vec::with_capacity(targets.len()),
    };
    let mut probes = vec![None; targets.len()];
    for (index, formatted_target, handle) in tasks {
        let probe = match handle.await {
            Ok(probe) => probe,
            Err(error) => EndpointProbeTargetDto {
                target: formatted_target,
                available: Some(false),
                error: Some(format!("endpoint probe task failed: {error}")),
            },
        };
        probes[index] = Some(probe);
    }

    for probe in probes.into_iter().flatten() {
        if status.first_successful_target.is_none() && probe.available == Some(true) {
            status.first_successful_target = Some(probe.target.clone());
        }
        status.probes.push(probe);
    }
    status
}

async fn probe_endpoint_target(
    target: &ProbeTargetSpec,
    timeout_per_target: Duration,
) -> EndpointProbeTargetDto {
    let result = timeout(timeout_per_target, probe_endpoint_availability(target))
        .await
        .map_err(|_| "connection timed out".to_string())
        .and_then(|result| result);
    match result {
        Ok(()) => EndpointProbeTargetDto {
            target: format_probe_target(target),
            available: Some(true),
            error: None,
        },
        Err(error) => EndpointProbeTargetDto {
            target: format_probe_target(target),
            available: Some(false),
            error: Some(error),
        },
    }
}

async fn probe_endpoint_availability(target: &ProbeTargetSpec) -> Result<(), String> {
    if should_use_https_probe(target) {
        return probe_https_endpoint(target).await;
    }
    let addresses = resolve_endpoint_addresses(&target.target).await?;
    match target.protocol {
        Protocol::Tcp => connect_first_available(addresses).await.map(|stream| {
            drop(stream);
        }),
        Protocol::Udp => probe_udp_endpoint(target, addresses).await,
        Protocol::Other => Err("unsupported endpoint probe protocol".to_string()),
    }
}

async fn probe_udp_endpoint(
    _target: &ProbeTargetSpec,
    addresses: Vec<SocketAddr>,
) -> Result<(), String> {
    probe_dns_endpoint_addresses(addresses).await
}

async fn probe_https_endpoint(target: &ProbeTargetSpec) -> Result<(), String> {
    let host =
        endpoint_host(&target.target).ok_or_else(|| "invalid https probe target".to_string())?;
    let url = format!("https://{host}/");
    let client = reqwest::Client::builder()
        .timeout(ENDPOINT_PROBE_TIMEOUT)
        .build()
        .map_err(|error| format!("https client build failed: {error}"))?;
    client
        .head(&url)
        .send()
        .await
        .map(|_| ())
        .map_err(|error| format!("https probe failed: {error}"))
}

async fn resolve_endpoint_addresses(target: &str) -> Result<Vec<SocketAddr>, String> {
    let localhost_fallbacks = localhost_fallback_addresses(target);
    let resolved_addresses = match lookup_host(target).await {
        Ok(addresses) => addresses.collect::<Vec<_>>(),
        Err(error) => {
            if let Some(fallbacks) = localhost_fallbacks {
                return Ok(fallbacks.to_vec());
            } else {
                return Err(format!("target resolution failed: {error}"));
            }
        }
    };
    let mut addresses = localhost_fallbacks
        .map(|fallbacks| fallbacks.to_vec())
        .unwrap_or_default();
    for address in resolved_addresses {
        if !addresses.contains(&address) {
            addresses.push(address);
        }
    }
    if addresses.is_empty() {
        Err("target did not resolve to any address".to_string())
    } else {
        Ok(addresses)
    }
}

async fn connect_first_available(addresses: Vec<SocketAddr>) -> Result<TcpStream, String> {
    let mut last_error = None;
    for address in addresses {
        match TcpStream::connect(address).await {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(format!("{address}: {error}")),
        }
    }
    Err(last_error.unwrap_or_else(|| "target did not resolve to any address".to_string()))
}

async fn probe_dns_endpoint_addresses(addresses: Vec<SocketAddr>) -> Result<(), String> {
    let query = build_dns_probe_query();
    let mut last_error = None;
    for address in addresses {
        match probe_dns_endpoint_address(address, &query).await {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(format!("{address}: {error}")),
        }
    }
    Err(last_error.unwrap_or_else(|| "target did not resolve to any address".to_string()))
}

async fn probe_dns_endpoint_address(address: SocketAddr, query: &[u8]) -> Result<(), String> {
    let bind_addr = if address.is_ipv4() {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    };
    let socket = UdpSocket::bind(bind_addr)
        .await
        .map_err(|error| format!("udp bind failed: {error}"))?;
    socket
        .connect(address)
        .await
        .map_err(|error| format!("udp connect failed: {error}"))?;
    socket
        .send(query)
        .await
        .map_err(|error| format!("dns query send failed: {error}"))?;

    let mut buffer = [0_u8; 512];
    let received = socket
        .recv(&mut buffer)
        .await
        .map_err(|error| format!("dns response receive failed: {error}"))?;
    if received == 0 {
        Err("dns response was empty".to_string())
    } else {
        Ok(())
    }
}

fn build_dns_probe_query() -> Vec<u8> {
    let mut query = vec![
        0x13, 0x37, // id
        0x01, 0x00, // standard query + recursion desired
        0x00, 0x01, // qdcount
        0x00, 0x00, // ancount
        0x00, 0x00, // nscount
        0x00, 0x00, // arcount
    ];
    for label in ["example", "net"] {
        query.push(label.len() as u8);
        query.extend_from_slice(label.as_bytes());
    }
    query.push(0x00); // root
    query.extend_from_slice(&[0x00, 0x01]); // QTYPE A
    query.extend_from_slice(&[0x00, 0x01]); // QCLASS IN
    query
}

fn localhost_fallback_addresses(target: &str) -> Option<[SocketAddr; 2]> {
    let port = target.strip_prefix("localhost:")?.parse::<u16>().ok()?;
    Some([
        SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        SocketAddr::from((Ipv6Addr::LOCALHOST, port)),
    ])
}

async fn team_cymru_lookup(ip: IpAddr) -> Result<Option<WhoisRangeCacheRecord>> {
    let mut stream = timeout(WHOIS_TIMEOUT, TcpStream::connect(("whois.cymru.com", 43)))
        .await
        .context("whois connect timed out")?
        .context("whois connect failed")?;
    let query = format!(" -v {ip}\r\n");
    timeout(WHOIS_TIMEOUT, stream.write_all(query.as_bytes()))
        .await
        .context("whois write timed out")?
        .context("whois write failed")?;

    let mut bytes = Vec::new();
    timeout(WHOIS_TIMEOUT, stream.read_to_end(&mut bytes))
        .await
        .context("whois read timed out")?
        .context("whois read failed")?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(parse_team_cymru_response(&text, current_timestamp_ms()))
}

fn parse_team_cymru_response(text: &str, updated_at_ms: u64) -> Option<WhoisRangeCacheRecord> {
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("AS") {
            continue;
        }
        let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
        if fields.len() < 7 {
            continue;
        }
        let cidr = fields.get(2)?.to_string();
        if !cidr.contains('/') {
            continue;
        }
        return Some(WhoisRangeCacheRecord {
            cidr,
            country: non_empty_field(fields.get(3).copied()),
            registry: non_empty_field(fields.get(4).copied()),
            owner_name: non_empty_field(fields.get(6).copied()),
            source: Some("Team Cymru".to_string()),
            updated_at_ms,
        });
    }
    None
}

fn non_empty_field(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "NA")
        .map(ToOwned::to_owned)
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn parse_probe_target_spec(value: &str) -> Result<ProbeTargetSpec> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("probe-endpoint target must not be empty"));
    }

    let (protocol, target) = if let Some(target) = trimmed.strip_prefix("tcp://") {
        (Protocol::Tcp, target)
    } else if let Some(target) = trimmed.strip_prefix("udp://") {
        (Protocol::Udp, target)
    } else {
        (default_probe_protocol(trimmed), trimmed)
    };

    let normalized_target = target.trim();
    if normalized_target.is_empty() {
        return Err(anyhow!("probe-endpoint target must not be empty"));
    }

    Ok(ProbeTargetSpec {
        target: normalized_target.to_string(),
        protocol,
    })
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

fn format_probe_target(target: &ProbeTargetSpec) -> String {
    format!("{}://{}", target.protocol.as_str(), target.target)
}

fn should_use_https_probe(target: &ProbeTargetSpec) -> bool {
    target.protocol == Protocol::Tcp
        && endpoint_port(&target.target) == Some(443)
        && endpoint_host(&target.target).is_some_and(|host| {
            !host.eq_ignore_ascii_case("localhost") && host.parse::<IpAddr>().is_err()
        })
}

fn endpoint_host(target: &str) -> Option<&str> {
    if let Some(rest) = target.strip_prefix('[') {
        let end = rest.find(']')?;
        return Some(&rest[..end]);
    }
    target.rsplit_once(':').map(|(host, _)| host)
}

fn endpoint_port(target: &str) -> Option<u16> {
    if let Some(rest) = target.strip_prefix('[') {
        let end = rest.find(']')?;
        let port = rest.get(end + 2..)?;
        return port.parse::<u16>().ok();
    }
    target
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
}

fn handle_native_tool_request(request: IntegrationHostRequest) -> Result<serde_json::Value> {
    if request.abi_version != netstitch_shared::models::INTEGRATION_ABI_VERSION {
        return Err(anyhow!(
            "unsupported integration ABI version {}",
            request.abi_version
        ));
    }

    match request.action.as_str() {
        "health" => Ok(serde_json::json!({
            "module_id": request.module_id,
            "kind": "network_tool",
            "transport": "native_library",
            "supported_os": ["windows", "linux"],
        })),
        "once" => {
            let payload = serde_json::from_value::<ToolOncePayload>(request.payload).unwrap_or(
                ToolOncePayload {
                    limit: DEFAULT_LIMIT,
                    retry_after_ms: DEFAULT_RETRY_AFTER_MS,
                },
            );
            let runtime = build_current_thread_runtime()?;
            let core = NetstitchCore::bootstrap().context("failed to bootstrap netstitch core")?;
            let count = runtime.block_on(enrich_pending_once(
                &core,
                payload.limit,
                payload.retry_after_ms,
            ))?;
            Ok(serde_json::json!({ "enriched": count }))
        }
        "lookup" => {
            let payload = serde_json::from_value::<ToolLookupPayload>(request.payload)
                .context("invalid lookup payload")?;
            let runtime = build_current_thread_runtime()?;
            let core = NetstitchCore::bootstrap().context("failed to bootstrap netstitch core")?;
            let client = build_http_client()?;
            serde_json::to_value(runtime.block_on(enrich_ip(&core, &client, payload.ip))?)
                .context("failed to encode lookup result")
        }
        "lookup-domain" => {
            let payload = serde_json::from_value::<ToolDomainLookupPayload>(request.payload)
                .context("invalid lookup-domain payload")?;
            let runtime = build_current_thread_runtime()?;
            let core = NetstitchCore::bootstrap().context("failed to bootstrap netstitch core")?;
            let client = build_http_client()?;
            serde_json::to_value(runtime.block_on(lookup_domain(&core, &client, payload.domain))?)
                .context("failed to encode domain lookup result")
        }
        "probe-endpoint" => {
            let payload = serde_json::from_value::<ToolProbeEndpointPayload>(request.payload)
                .context("invalid probe-endpoint payload")?;
            let targets = match payload {
                ToolProbeEndpointPayload::Targets { targets } => targets,
                ToolProbeEndpointPayload::TargetList(targets) => targets,
            };
            let runtime = build_current_thread_runtime()?;
            serde_json::to_value(runtime.block_on(probe_endpoint_target_strings(&targets))?)
                .context("failed to encode endpoint probe result")
        }
        unknown => Err(anyhow!("unknown native network tool action '{unknown}'")),
    }
}

fn build_current_thread_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build native network tool runtime")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_call(
    request_ptr: *const u8,
    request_len: usize,
    _event_callback: netstitch_shared::models::IntegrationAbiEventCallback,
    _event_user_data: *mut std::ffi::c_void,
    output: *mut IntegrationAbiBuffer,
) -> i32 {
    if request_ptr.is_null() || output.is_null() {
        return 1;
    }
    let request_bytes = unsafe { std::slice::from_raw_parts(request_ptr, request_len) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let request: IntegrationHostRequest = serde_json::from_slice(request_bytes)?;
        handle_native_tool_request(request)
    }));

    let response = match result {
        Ok(Ok(value)) => IntegrationHostResponse {
            ok: true,
            result: Some(value),
            error: None,
        },
        Ok(Err(error)) => IntegrationHostResponse {
            ok: false,
            result: None,
            error: Some(format!("{error:#}")),
        },
        Err(_) => IntegrationHostResponse {
            ok: false,
            result: None,
            error: Some("native network tool module panicked".to_string()),
        },
    };

    match write_abi_response(response, output) {
        Ok(()) => 0,
        Err(_) => 2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

fn write_abi_response(
    response: IntegrationHostResponse,
    output: *mut IntegrationAbiBuffer,
) -> Result<()> {
    let mut bytes = serde_json::to_vec(&response)?.into_boxed_slice();
    let len = bytes.len();
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    unsafe {
        (*output).ptr = ptr;
        (*output).len = len;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_team_cymru_prefix_owner() {
        let text = "AS | IP | BGP Prefix | CC | Registry | Allocated | AS Name\n15169 | 8.8.8.8 | 8.8.8.0/24 | US | arin | 1992-12-01 | GOOGLE, US";
        let record = parse_team_cymru_response(text, 42).expect("record parsed");
        assert_eq!(record.cidr, "8.8.8.0/24");
        assert_eq!(record.owner_name.as_deref(), Some("GOOGLE, US"));
        assert_eq!(record.registry.as_deref(), Some("arin"));
        assert_eq!(record.country.as_deref(), Some("US"));
    }

    #[test]
    fn native_tool_abi_health_reports_windows_linux_support() {
        let result = handle_native_tool_request(IntegrationHostRequest {
            abi_version: netstitch_shared::models::INTEGRATION_ABI_VERSION,
            module_id: "netstitch-tool".to_string(),
            storage_dir: std::path::PathBuf::from("data"),
            action: "health".to_string(),
            payload: serde_json::Value::Null,
        })
        .expect("health request should succeed");

        assert_eq!(result["kind"], "network_tool");
        assert_eq!(result["transport"], "native_library");
        assert!(
            result["supported_os"]
                .as_array()
                .expect("supported os")
                .iter()
                .any(|value| value == "linux")
        );
    }

    #[test]
    fn probe_target_prefixes_override_default_protocol() {
        assert_eq!(
            parse_probe_target_spec("tcp://1.1.1.1:53").expect("tcp target parses"),
            ProbeTargetSpec {
                target: "1.1.1.1:53".to_string(),
                protocol: Protocol::Tcp,
            }
        );
        assert_eq!(
            parse_probe_target_spec("udp://example.com:443").expect("udp target parses"),
            ProbeTargetSpec {
                target: "example.com:443".to_string(),
                protocol: Protocol::Udp,
            }
        );
    }

    #[tokio::test]
    async fn endpoint_probe_api_requires_caller_supplied_targets() {
        assert!(
            probe_endpoint_target_strings(&[])
                .await
                .expect_err("empty targets should fail")
                .to_string()
                .contains("requires at least one host:port target")
        );
    }

    #[test]
    fn tcp_domain_443_targets_use_https_probe_path() {
        assert!(should_use_https_probe(&ProbeTargetSpec {
            target: "dns.google:443".to_string(),
            protocol: Protocol::Tcp,
        }));
        assert!(!should_use_https_probe(&ProbeTargetSpec {
            target: "1.1.1.1:443".to_string(),
            protocol: Protocol::Tcp,
        }));
    }

    #[tokio::test]
    async fn endpoint_probe_collects_all_targets_and_marks_first_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener should bind");
        let listener_addr = listener
            .local_addr()
            .expect("listener addr should load")
            .to_string();
        let domain_target = format!(
            "localhost:{}",
            listener
                .local_addr()
                .expect("listener addr should load")
                .port()
        );
        let accept_task = tokio::spawn(async move {
            let _ = listener.accept().await;
        });
        let tcp_target = ProbeTargetSpec {
            target: domain_target.clone(),
            protocol: Protocol::Tcp,
        };
        let first_target = ProbeTargetSpec {
            target: "not-a-socket-address-without-port".to_string(),
            protocol: Protocol::Tcp,
        };
        let third_target = ProbeTargetSpec {
            target: "also-not-a-socket-address-without-port".to_string(),
            protocol: Protocol::Tcp,
        };

        let status = probe_endpoint_targets(
            &[first_target, tcp_target.clone(), third_target],
            Duration::from_secs(1),
        )
        .await;
        accept_task.abort();

        assert!(!status.is_checking);
        assert_eq!(status.probes.len(), 3);
        assert_eq!(
            status.first_successful_target.as_deref(),
            Some(format_probe_target(&tcp_target).as_str())
        );
        assert_eq!(status.probes[0].available, Some(false));
        assert_eq!(status.probes[1].available, Some(true));
        assert_eq!(status.probes[2].available, Some(false));
        assert_ne!(
            status.first_successful_target.as_deref(),
            Some(format!("tcp://{listener_addr}").as_str())
        );
    }

    #[tokio::test]
    async fn dns_endpoint_probe_uses_udp_for_port_53_targets() {
        let socket = UdpSocket::bind("127.0.0.1:0")
            .await
            .expect("udp listener should bind");
        let port = socket.local_addr().expect("udp addr").port();
        let target = ProbeTargetSpec {
            target: format!("localhost:{port}"),
            protocol: Protocol::Udp,
        };
        let responder = tokio::spawn(async move {
            let mut buffer = [0_u8; 512];
            let (received, peer) = socket.recv_from(&mut buffer).await.expect("udp receive");
            assert!(received > 0, "dns probe should send a datagram");
            socket
                .send_to(&buffer[..received.min(12)], peer)
                .await
                .expect("udp response should send");
        });

        let status = probe_endpoint_targets(&[target.clone()], Duration::from_secs(1)).await;
        responder.await.expect("udp responder should finish");

        assert_eq!(status.probes.len(), 1);
        assert_eq!(status.probes[0].available, Some(true));
        assert_eq!(
            status.first_successful_target.as_deref(),
            Some(format_probe_target(&target).as_str())
        );
    }
}
