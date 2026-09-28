use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

/// The MAPPED-ADDRESS attribute, and the two address families it can
/// carry. Named so the parser below is checked against the spec's numbers
/// rather than against literals.
pub const ATTR_MAPPED: u16 = 0x0001;
const FAMILY_V4: u8 = 0x01;
const FAMILY_V6: u8 = 0x02;

/// Ask a STUN server (RFC 5389) what this host looks like from outside.
///
/// A binding request is a 20-byte header plus attributes; a response
/// carries the reflexive address. Implemented directly rather than pulled
/// in as a dependency because it is genuinely small and because the
/// failure mode we care about -- "no answer" -- has to be distinguishable
/// from "wrong answer".
pub async fn reflexive_address(
    server: SocketAddr,
    timeout: Duration,
) -> Result<SocketAddr, String> {
    const BINDING_REQUEST: u8 = 0x0001;
    const MAGIC_COOKIE: u32 = 0x2112_A442;

    let mut request = Vec::with_capacity(20);
    request.extend_from_slice(&BINDING_REQUEST.to_be_bytes());
    request.extend_from_slice(&0u16.to_be_bytes()); // no attributes
    request.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    request.extend_from_slice(&[0u8; 12]);

    let bind_addr: SocketAddr = if server.is_ipv4() {
        "0.0.0.0:0".parse().expect("literal")
    } else {
        "[::]:0".parse().expect("literal")
    };
    let socket = tokio::net::UdpSocket::bind(bind_addr)
        .await
        .map_err(|e| format!("could not open a UDP socket: {e}"))?;
    socket
        .connect(server)
        .await
        .map_err(|e| format!("could not reach the STUN server {server}: {e}"))?;
    socket
        .send(&request)
        .await
        .map_err(|e| format!("could not send a binding request: {e}"))?;

    let deadline = Instant::now() + timeout;
    let mut buf = [0u8; 512];
    let n = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!(
                "no answer from {server} within {}s (UDP may be blocked on this network)",
                timeout.as_secs()
            ));
        }
        match tokio::time::timeout(remaining, socket.recv(&mut buf)).await {
            Ok(Ok(n)) => break n,
            Ok(Err(e)) => return Err(format!("STUN read failed: {e}")),
            Err(_) => continue,
        }
    };
    parse_binding(&buf[..n])
}

pub fn parse_binding(packet: &[u8]) -> Result<SocketAddr, String> {
    if packet.len() < 20 {
        return Err(format!(
            "STUN response is truncated: {} bytes",
            packet.len()
        ));
    }
    let message_type = u16::from_be_bytes([packet[0], packet[1]]);
    if message_type != 0x0101 {
        return Err(format!(
            "expected a STUN binding success response, got type 0x{message_type:04x}"
        ));
    }
    if u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]) != 0x2112_A442 {
        return Err("STUN response has the wrong magic cookie".into());
    }
    let length = u16::from_be_bytes([packet[2], packet[3]]) as usize;
    let end = (20 + length).min(packet.len());
    let mut at = 20usize;
    while at + 4 <= end {
        let attr = u16::from_be_bytes([packet[at], packet[at + 1]]);
        let len = u16::from_be_bytes([packet[at + 2], packet[at + 3]]) as usize;
        let value_at = at + 4;
        if value_at + len > packet.len() {
            break;
        }
        if attr == ATTR_MAPPED && len >= 8 {
            let family = packet[value_at + 1];
            let port = u16::from_be_bytes([packet[value_at + 2], packet[value_at + 3]]);
            let ip = match family {
                FAMILY_V4 => {
                    let o = &packet[value_at + 4..value_at + 8];
                    IpAddr::V4(Ipv4Addr::new(o[0], o[1], o[2], o[3]))
                }
                FAMILY_V6 => {
                    let mut octets = [0u8; 16];
                    octets.copy_from_slice(&packet[value_at + 4..value_at + 20]);
                    IpAddr::V6(Ipv6Addr::from(octets))
                }
                other => {
                    return Err(format!(
                        "STUN returned unknown address family 0x{other:02x}"
                    ))
                }
            };
            return Ok(SocketAddr::new(ip, port));
        }
        // Attributes are padded to a 4-byte boundary.
        at = value_at + len.div_ceil(4) * 4;
    }
    Err("STUN response carried no MAPPED-ADDRESS attribute".into())
}

/// Public STUN servers. Google and Cloudflare both run the free RFC 5389
/// service and neither requires a key for a plain binding request.
pub const DEFAULT_STUN: &[&str] = &["stun.l.google.com:19302", "stun.cloudflare.com:3478"];

/// A 20-byte binding request with no attributes, per RFC 5389.
pub fn binding_request() -> Vec<u8> {
    let mut v = Vec::with_capacity(20);
    v.extend_from_slice(&0x0001u16.to_be_bytes()); // BindingRequest
    v.extend_from_slice(&0u16.to_be_bytes()); // no attributes
    v.extend_from_slice(&0x2112_A442u32.to_be_bytes()); // magic cookie
    v.extend_from_slice(&[0u8; 12]);
    v
}

/// A binding success response echoing the request's transaction id, with
/// the responder's own address as MAPPED-ADDRESS.
pub fn success_response(request: &[u8], responder: Option<SocketAddr>) -> Option<Vec<u8>> {
    if request.len() < 20 {
        return None;
    }
    let local = responder?;
    let value = match local.ip() {
        // reserved(1) + family(1) + port(2) + address.
        IpAddr::V4(v4) => {
            let mut v = vec![0u8, FAMILY_V4];
            v.extend_from_slice(&local.port().to_be_bytes());
            v.extend_from_slice(&v4.octets());
            v
        }
        IpAddr::V6(v6) => {
            let mut v = vec![0u8, FAMILY_V6];
            v.extend_from_slice(&local.port().to_be_bytes());
            v.extend_from_slice(&v6.octets());
            v
        }
    };
    let mut out = Vec::with_capacity(20 + 4 + value.len());
    out.extend_from_slice(&0x0101u16.to_be_bytes()); // BindingSuccessResponse
    out.extend_from_slice(&((4 + value.len()) as u16).to_be_bytes());
    out.extend_from_slice(&0x2112_A442u32.to_be_bytes());
    out.extend_from_slice(&request[8..20]); // transaction id
    out.extend_from_slice(&ATTR_MAPPED.to_be_bytes());
    out.extend_from_slice(&(value.len() as u16).to_be_bytes());
    out.extend_from_slice(&value);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_binding_request_is_twenty_bytes_with_the_right_magic() {
        let r = binding_request();
        assert_eq!(r.len(), 20);
        assert_eq!(u16::from_be_bytes([r[0], r[1]]), 0x0001);
        assert_eq!(u32::from_be_bytes([r[4], r[5], r[6], r[7]]), 0x2112_A442);
    }

    #[test]
    fn a_success_response_round_trips_the_transaction_id() {
        let req = binding_request();
        let local: SocketAddr = "203.0.113.7:9999".parse().unwrap();
        let resp = success_response(&req, Some(local)).unwrap();
        assert_eq!(u16::from_be_bytes([resp[0], resp[1]]), 0x0101);
        assert_eq!(&resp[8..20], &req[8..20], "transaction id must match");
        // And it must parse back to the responder's own address.
        assert_eq!(parse_binding(&resp).unwrap(), local);
    }

    #[test]
    fn a_success_response_with_no_local_address_is_refused() {
        let req = binding_request();
        assert!(success_response(&req, None).is_none());
    }
}

#[test]
fn a_short_response_is_refused_rather_than_read_past() {
    assert!(parse_binding(&[0u8; 4]).is_err());
}

#[test]
fn a_response_with_the_wrong_type_or_cookie_is_refused() {
    let mut packet = vec![0u8; 20];
    packet[0] = 0x01;
    packet[1] = 0x02; // not a binding success
    assert!(parse_binding(&packet).is_err());

    let mut packet = vec![0u8; 20];
    packet[0] = 0x01;
    packet[1] = 0x01;
    packet[4] = 0xde; // wrong cookie
    let err = parse_binding(&packet).unwrap_err();
    assert!(err.contains("magic cookie"), "unhelpful: {err}");
}

#[test]
fn a_well_formed_response_yields_the_reflexive_address() {
    let mut packet = vec![0u8; 20];
    packet[0] = 0x01;
    packet[1] = 0x01;
    packet[2..4].copy_from_slice(&8u16.to_be_bytes());
    packet[4..8].copy_from_slice(&0x2112_A442u32.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x01, 0x00, 0x08]); // MAPPED-ADDRESS
                                                         // reserved(1) + family(1) + port(2) + address(4) = 8 bytes.
    packet.extend_from_slice(&[0x00, 0x01]); // reserved, family IPv4
    packet.extend_from_slice(&3478u16.to_be_bytes());
    packet.extend_from_slice(&[203, 0, 113, 9]);
    assert_eq!(
        parse_binding(&packet).unwrap(),
        "203.0.113.9:3478".parse::<SocketAddr>().unwrap()
    );
}
