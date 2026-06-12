use crate::models::Protocol;
use ring::{aead, hmac};

pub const DOMAIN_SOURCE_HTTP_HOST: &str = "http_host";
pub const DOMAIN_SOURCE_TLS_SNI: &str = "tls_sni";
pub const DOMAIN_SOURCE_QUIC_SNI: &str = "quic_sni";
pub const DOMAIN_SOURCE_CSV_IMPORT: &str = "csv_import";

const QUIC_V1_VERSION: u32 = 0x0000_0001;
const QUIC_V1_INITIAL_SALT: [u8; 20] = [
    0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c, 0xad,
    0xcc, 0xbb, 0x7f, 0x0a,
];
const QUIC_CRYPTO_BUFFER_LIMIT: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedDomainDetection {
    pub domain: String,
    pub source: &'static str,
}

pub fn detect_verified_domain(
    protocol: Protocol,
    remote_port: u16,
    payload: &[u8],
) -> Option<VerifiedDomainDetection> {
    match protocol {
        Protocol::Tcp => {
            if let Some(domain) = http_host_domain(payload) {
                return Some(VerifiedDomainDetection {
                    domain,
                    source: DOMAIN_SOURCE_HTTP_HOST,
                });
            }
            if let Some(domain) = tls_sni_domain(payload) {
                return Some(VerifiedDomainDetection {
                    domain,
                    source: DOMAIN_SOURCE_TLS_SNI,
                });
            }
        }
        Protocol::Udp if remote_port == 443 => {
            if let Some(domain) = quic_initial_sni_domain(payload) {
                return Some(VerifiedDomainDetection {
                    domain,
                    source: DOMAIN_SOURCE_QUIC_SNI,
                });
            }
        }
        _ => {}
    }

    None
}

pub fn http_host_domain(payload: &[u8]) -> Option<String> {
    let header_end = payload
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)?;
    let headers = std::str::from_utf8(&payload[..header_end]).ok()?;
    let mut lines = headers.lines();
    let request_line = lines.next()?.trim();
    if !is_http_request_line(request_line) {
        return None;
    }

    for line in lines {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("host") {
            return normalize_verified_domain(value.trim());
        }
    }
    None
}

pub fn tls_sni_domain(payload: &[u8]) -> Option<String> {
    if payload.len() < 9 || payload[0] != 0x16 {
        return None;
    }
    let record_len = u16::from_be_bytes([payload[3], payload[4]]) as usize;
    let record_end = 5usize.checked_add(record_len)?;
    if payload.len() < record_end || payload[5] != 0x01 {
        return None;
    }
    let handshake_len = read_u24(payload, 6)? as usize;
    let handshake_end = 9usize.checked_add(handshake_len)?.min(record_end);
    tls_client_hello_sni_domain(&payload[5..handshake_end])
}

fn tls_client_hello_sni_domain(handshake: &[u8]) -> Option<String> {
    if handshake.len() < 38 || handshake[0] != 0x01 {
        return None;
    }
    let handshake_len = read_u24(handshake, 1)? as usize;
    let handshake_end = 4usize.checked_add(handshake_len)?;
    if handshake.len() < handshake_end || handshake_end < 38 {
        return None;
    }

    let mut index = 38;
    let session_id_len = *handshake.get(index)? as usize;
    index = index.checked_add(1 + session_id_len)?;
    if index.checked_add(2)? > handshake_end {
        return None;
    }
    let cipher_suites_len = u16::from_be_bytes([handshake[index], handshake[index + 1]]) as usize;
    index = index.checked_add(2 + cipher_suites_len)?;
    if index >= handshake_end {
        return None;
    }
    let compression_len = *handshake.get(index)? as usize;
    index = index.checked_add(1 + compression_len)?;
    if index.checked_add(2)? > handshake_end {
        return None;
    }
    let extensions_len = u16::from_be_bytes([handshake[index], handshake[index + 1]]) as usize;
    index += 2;
    let extensions_end = index.checked_add(extensions_len)?.min(handshake_end);

    while index.checked_add(4)? <= extensions_end {
        let extension_type = u16::from_be_bytes([handshake[index], handshake[index + 1]]);
        let extension_len =
            u16::from_be_bytes([handshake[index + 2], handshake[index + 3]]) as usize;
        index += 4;
        let extension_end = index.checked_add(extension_len)?;
        if extension_end > extensions_end {
            return None;
        }
        if extension_type == 0 {
            return parse_sni_extension(&handshake[index..extension_end]);
        }
        index = extension_end;
    }

    None
}

pub fn quic_initial_sni_domain(datagram: &[u8]) -> Option<String> {
    let packet = parse_quic_v1_initial(datagram)?;
    let keys = QuicInitialKeys::derive(packet.dcid)?;
    let mask = quic_header_mask(&keys.header_protection_key, packet.sample)?;
    let first_unprotected = packet.first ^ (mask[0] & 0x0f);
    let packet_number_len = usize::from(first_unprotected & 0x03) + 1;
    if packet.pn_offset.checked_add(packet_number_len)? > packet.packet_end {
        return None;
    }

    let mut header = datagram[..packet.pn_offset + packet_number_len].to_vec();
    header[0] = first_unprotected;
    let mut packet_number = 0u64;
    for index in 0..packet_number_len {
        let value = datagram[packet.pn_offset + index] ^ mask[index + 1];
        header[packet.pn_offset + index] = value;
        packet_number = (packet_number << 8) | u64::from(value);
    }

    let mut ciphertext = datagram[packet.pn_offset + packet_number_len..packet.packet_end].to_vec();
    let plaintext = quic_decrypt_initial_payload(&keys, packet_number, &header, &mut ciphertext)?;
    tls_sni_from_quic_crypto_frames(plaintext)
}

struct ParsedQuicInitial<'a> {
    first: u8,
    dcid: &'a [u8],
    pn_offset: usize,
    packet_end: usize,
    sample: &'a [u8],
}

fn parse_quic_v1_initial(datagram: &[u8]) -> Option<ParsedQuicInitial<'_>> {
    let first = *datagram.first()?;
    if first & 0x80 == 0 || first & 0x40 == 0 || first & 0x30 != 0 {
        return None;
    }
    let version = u32::from_be_bytes(datagram.get(1..5)?.try_into().ok()?);
    if version != QUIC_V1_VERSION {
        return None;
    }

    let mut index = 5usize;
    let dcid_len = usize::from(*datagram.get(index)?);
    index += 1;
    let dcid = datagram.get(index..index.checked_add(dcid_len)?)?;
    index += dcid_len;
    let scid_len = usize::from(*datagram.get(index)?);
    index += 1 + scid_len;
    if index > datagram.len() {
        return None;
    }
    let token_len = read_quic_varint(datagram, &mut index)? as usize;
    index = index.checked_add(token_len)?;
    if index > datagram.len() {
        return None;
    }
    let protected_len = read_quic_varint(datagram, &mut index)? as usize;
    let pn_offset = index;
    let packet_end = pn_offset.checked_add(protected_len)?;
    if packet_end > datagram.len() || pn_offset.checked_add(4 + 16)? > packet_end {
        return None;
    }

    Some(ParsedQuicInitial {
        first,
        dcid,
        pn_offset,
        packet_end,
        sample: &datagram[pn_offset + 4..pn_offset + 4 + 16],
    })
}

struct QuicInitialKeys {
    packet_key: Vec<u8>,
    packet_iv: [u8; 12],
    header_protection_key: Vec<u8>,
}

impl QuicInitialKeys {
    fn derive(dcid: &[u8]) -> Option<Self> {
        let initial_secret = hkdf_extract(&QUIC_V1_INITIAL_SALT, dcid);
        let client_secret = hkdf_expand_label(&initial_secret, b"client in", 32)?;
        let packet_key = hkdf_expand_label(&client_secret, b"quic key", 16)?;
        let packet_iv =
            <[u8; 12]>::try_from(hkdf_expand_label(&client_secret, b"quic iv", 12)?).ok()?;
        let header_protection_key = hkdf_expand_label(&client_secret, b"quic hp", 16)?;
        Some(Self {
            packet_key,
            packet_iv,
            header_protection_key,
        })
    }
}

fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, salt), ikm)
        .as_ref()
        .to_vec()
}

fn hkdf_expand_label(secret: &[u8], label: &[u8], output_len: usize) -> Option<Vec<u8>> {
    let full_label_len = b"tls13 ".len().checked_add(label.len())?;
    if output_len > u16::MAX as usize || full_label_len > u8::MAX as usize {
        return None;
    }
    let mut info = Vec::with_capacity(2 + 1 + full_label_len + 1);
    info.extend_from_slice(&(output_len as u16).to_be_bytes());
    info.push(full_label_len as u8);
    info.extend_from_slice(b"tls13 ");
    info.extend_from_slice(label);
    info.push(0);
    hkdf_expand(secret, &info, output_len)
}

fn hkdf_expand(prk: &[u8], info: &[u8], output_len: usize) -> Option<Vec<u8>> {
    if output_len > 255 * 32 {
        return None;
    }
    let key = hmac::Key::new(hmac::HMAC_SHA256, prk);
    let mut output = Vec::with_capacity(output_len);
    let mut previous = Vec::<u8>::new();
    let mut counter = 1u8;
    while output.len() < output_len {
        let mut context = hmac::Context::with_key(&key);
        context.update(&previous);
        context.update(info);
        context.update(&[counter]);
        previous = context.sign().as_ref().to_vec();
        output.extend_from_slice(&previous);
        counter = counter.checked_add(1)?;
    }
    output.truncate(output_len);
    Some(output)
}

fn quic_header_mask(header_protection_key: &[u8], sample: &[u8]) -> Option<[u8; 5]> {
    let key =
        aead::quic::HeaderProtectionKey::new(&aead::quic::AES_128, header_protection_key).ok()?;
    key.new_mask(sample).ok()
}

fn quic_decrypt_initial_payload<'a>(
    keys: &QuicInitialKeys,
    packet_number: u64,
    header: &[u8],
    ciphertext: &'a mut [u8],
) -> Option<&'a [u8]> {
    let key =
        aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_128_GCM, &keys.packet_key).ok()?);
    let nonce = quic_packet_nonce(&keys.packet_iv, packet_number);
    let plaintext = key
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(header),
            ciphertext,
        )
        .ok()?;
    Some(plaintext)
}

fn quic_packet_nonce(iv: &[u8; 12], packet_number: u64) -> [u8; 12] {
    let mut nonce = *iv;
    let packet_number_bytes = packet_number.to_be_bytes();
    for index in 0..8 {
        nonce[4 + index] ^= packet_number_bytes[index];
    }
    nonce
}

fn tls_sni_from_quic_crypto_frames(payload: &[u8]) -> Option<String> {
    let mut index = 0usize;
    let mut chunks: Vec<(usize, &[u8])> = Vec::new();
    while index < payload.len() {
        let frame_type = read_quic_varint(payload, &mut index)?;
        match frame_type {
            0x00 | 0x01 => {}
            0x02 | 0x03 => skip_quic_ack_frame(payload, &mut index)?,
            0x06 => {
                let offset = read_quic_varint(payload, &mut index)? as usize;
                let len = read_quic_varint(payload, &mut index)? as usize;
                let end = index.checked_add(len)?;
                let data = payload.get(index..end)?;
                chunks.push((offset, data));
                index = end;
            }
            0x1c | 0x1d => break,
            _ => break,
        }
    }

    chunks.sort_by_key(|(offset, _)| *offset);
    let mut crypto_stream = Vec::new();
    for (offset, data) in chunks {
        if offset != crypto_stream.len() {
            return None;
        }
        if crypto_stream.len().saturating_add(data.len()) > QUIC_CRYPTO_BUFFER_LIMIT {
            return None;
        }
        crypto_stream.extend_from_slice(data);
        if let Some(domain) = tls_client_hello_sni_domain(&crypto_stream) {
            return Some(domain);
        }
    }
    None
}

fn skip_quic_ack_frame(payload: &[u8], index: &mut usize) -> Option<()> {
    let _largest_acknowledged = read_quic_varint(payload, index)?;
    let _ack_delay = read_quic_varint(payload, index)?;
    let ack_range_count = read_quic_varint(payload, index)?;
    let _first_ack_range = read_quic_varint(payload, index)?;
    for _ in 0..ack_range_count {
        let _gap = read_quic_varint(payload, index)?;
        let _ack_range = read_quic_varint(payload, index)?;
    }
    Some(())
}

fn read_quic_varint(payload: &[u8], index: &mut usize) -> Option<u64> {
    let first = *payload.get(*index)?;
    let len = 1usize << usize::from(first >> 6);
    let end = (*index).checked_add(len)?;
    let bytes = payload.get(*index..end)?;
    *index = end;
    let mut value = u64::from(first & 0x3f);
    for byte in &bytes[1..] {
        value = (value << 8) | u64::from(*byte);
    }
    Some(value)
}

fn parse_sni_extension(extension: &[u8]) -> Option<String> {
    if extension.len() < 5 {
        return None;
    }
    let list_len = u16::from_be_bytes([extension[0], extension[1]]) as usize;
    let mut index = 2usize;
    let list_end = index.checked_add(list_len)?.min(extension.len());
    while index.checked_add(3)? <= list_end {
        let name_type = extension[index];
        let name_len = u16::from_be_bytes([extension[index + 1], extension[index + 2]]) as usize;
        index += 3;
        let name_end = index.checked_add(name_len)?;
        if name_end > list_end {
            return None;
        }
        if name_type == 0 {
            let value = std::str::from_utf8(&extension[index..name_end]).ok()?;
            return normalize_verified_domain(value);
        }
        index = name_end;
    }
    None
}

fn read_u24(payload: &[u8], index: usize) -> Option<u32> {
    Some(
        ((u32::from(*payload.get(index)?)) << 16)
            | ((u32::from(*payload.get(index + 1)?)) << 8)
            | u32::from(*payload.get(index + 2)?),
    )
}

fn is_http_request_line(value: &str) -> bool {
    const METHODS: [&str; 9] = [
        "GET", "POST", "HEAD", "PUT", "DELETE", "PATCH", "OPTIONS", "TRACE", "CONNECT",
    ];
    let mut parts = value.split_ascii_whitespace();
    let Some(method) = parts.next() else {
        return false;
    };
    let _target = parts.next();
    let Some(version) = parts.next() else {
        return false;
    };
    METHODS.contains(&method) && version.starts_with("HTTP/")
}

pub fn normalize_verified_domain(value: &str) -> Option<String> {
    let mut host = value.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.starts_with('[') {
        return None;
    }
    if let Some((candidate, port)) = host.rsplit_once(':') {
        if !candidate.contains(':') && port.parse::<u16>().is_ok() {
            host = candidate.to_string();
        }
    }
    if is_verified_domain_name(&host) {
        Some(host)
    } else {
        None
    }
}

fn is_verified_domain_name(value: &str) -> bool {
    if value.len() > 253 || !value.contains('.') || value.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }
    value.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|item| item.is_ascii_alphanumeric() || item == b'-')
    })
}

#[cfg(test)]
pub fn test_tls_client_hello_record(domain: &str) -> Vec<u8> {
    let handshake = test_tls_client_hello_handshake(domain);
    let mut payload = Vec::new();
    payload.push(0x16);
    payload.extend_from_slice(&[0x03, 0x01]);
    payload.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    payload.extend_from_slice(&handshake);
    payload
}

#[doc(hidden)]
pub fn test_quic_initial_datagram(domain: &str) -> Vec<u8> {
    let dcid = [0x83, 0x94, 0xc8, 0xf0, 0x3e, 0x51, 0x57, 0x08];
    let keys = QuicInitialKeys::derive(&dcid).expect("test QUIC keys should derive");
    let mut crypto = test_tls_client_hello_handshake(domain);
    let mut plaintext = Vec::new();
    write_quic_varint(0x06, &mut plaintext);
    write_quic_varint(0, &mut plaintext);
    write_quic_varint(crypto.len() as u64, &mut plaintext);
    plaintext.append(&mut crypto);
    while plaintext.len() < 32 {
        plaintext.push(0);
    }

    let packet_number = 0u64;
    let packet_number_len = 4usize;
    let mut header = Vec::new();
    header.push(0xc0 | (packet_number_len as u8 - 1));
    header.extend_from_slice(&QUIC_V1_VERSION.to_be_bytes());
    header.push(dcid.len() as u8);
    header.extend_from_slice(&dcid);
    header.push(0);
    write_quic_varint(0, &mut header);
    write_quic_varint(
        (packet_number_len + plaintext.len() + 16) as u64,
        &mut header,
    );
    let pn_offset = header.len();
    header.extend_from_slice(&[0, 0, 0, 0]);

    let key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::AES_128_GCM, &keys.packet_key)
            .expect("test packet key should initialize"),
    );
    let mut ciphertext = plaintext;
    key.seal_in_place_append_tag(
        aead::Nonce::assume_unique_for_key(quic_packet_nonce(&keys.packet_iv, packet_number)),
        aead::Aad::from(&header),
        &mut ciphertext,
    )
    .expect("test QUIC Initial should encrypt");

    let mut packet = header;
    packet.extend_from_slice(&ciphertext);
    let sample = &packet[pn_offset + 4..pn_offset + 4 + 16];
    let mask = quic_header_mask(&keys.header_protection_key, sample)
        .expect("test header protection mask should generate");
    packet[0] ^= mask[0] & 0x0f;
    for index in 0..packet_number_len {
        packet[pn_offset + index] ^= mask[index + 1];
    }
    packet
}

fn test_tls_client_hello_handshake(domain: &str) -> Vec<u8> {
    let domain = domain.as_bytes();
    let mut extensions = Vec::new();
    extensions.extend_from_slice(&0u16.to_be_bytes());
    let mut sni = Vec::new();
    let list_len = 1 + 2 + domain.len();
    sni.extend_from_slice(&(list_len as u16).to_be_bytes());
    sni.push(0);
    sni.extend_from_slice(&(domain.len() as u16).to_be_bytes());
    sni.extend_from_slice(domain);
    extensions.extend_from_slice(&(sni.len() as u16).to_be_bytes());
    extensions.extend_from_slice(&sni);

    let mut hello = Vec::new();
    hello.extend_from_slice(&[0x03, 0x03]);
    hello.extend_from_slice(&[0x11; 32]);
    hello.push(0);
    hello.extend_from_slice(&2u16.to_be_bytes());
    hello.extend_from_slice(&0x1301u16.to_be_bytes());
    hello.push(1);
    hello.push(0);
    hello.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
    hello.extend_from_slice(&extensions);

    let mut payload = Vec::new();
    payload.push(0x01);
    let hello_len = hello.len() as u32;
    payload.push(((hello_len >> 16) & 0xff) as u8);
    payload.push(((hello_len >> 8) & 0xff) as u8);
    payload.push((hello_len & 0xff) as u8);
    payload.extend_from_slice(&hello);
    payload
}

fn write_quic_varint(value: u64, output: &mut Vec<u8>) {
    if value < 64 {
        output.push(value as u8);
    } else if value < 16_384 {
        let encoded = (value as u16) | 0x4000;
        output.extend_from_slice(&encoded.to_be_bytes());
    } else if value < 1_073_741_824 {
        let encoded = (value as u32) | 0x8000_0000;
        output.extend_from_slice(&encoded.to_be_bytes());
    } else {
        let encoded = value | 0xc000_0000_0000_0000;
        output.extend_from_slice(&encoded.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_host_extracts_domain_from_plain_request() {
        let payload = b"GET / HTTP/1.1\r\nHost: Example.NET:80\r\nUser-Agent: test\r\n\r\n";

        assert_eq!(http_host_domain(payload).as_deref(), Some("example.net"));
    }

    #[test]
    fn http_host_rejects_ip_hosts() {
        let payload = b"GET / HTTP/1.1\r\nHost: 203.0.113.10\r\n\r\n";

        assert_eq!(http_host_domain(payload), None);
    }

    #[test]
    fn tls_sni_extracts_domain_from_client_hello() {
        let payload = test_tls_client_hello_record("Game.Example");

        assert_eq!(tls_sni_domain(&payload).as_deref(), Some("game.example"));
    }

    #[test]
    fn verified_detection_uses_tcp_payload_sources_on_any_port() {
        let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

        assert_eq!(
            detect_verified_domain(Protocol::Udp, 80, payload),
            None,
            "UDP DNS is not accepted as a verified source"
        );
        assert_eq!(
            detect_verified_domain(Protocol::Tcp, 8080, payload)
                .expect("HTTP Host should be detected")
                .source,
            DOMAIN_SOURCE_HTTP_HOST
        );
    }

    #[test]
    fn quic_initial_extracts_domain_from_client_hello_sni() {
        let datagram = test_quic_initial_datagram("Quic.Game.Example");

        assert_eq!(
            quic_initial_sni_domain(&datagram).as_deref(),
            Some("quic.game.example")
        );
        let detection = detect_verified_domain(Protocol::Udp, 443, &datagram)
            .expect("QUIC Initial SNI should be detected");
        assert_eq!(detection.source, DOMAIN_SOURCE_QUIC_SNI);
    }
}
