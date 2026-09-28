//! Port mapping on the sharer's own router.
//!
//! NAT-PMP (RFC 6886) is chosen over UPnP IGD deliberately: it is a
//! two-byte-opcode, text-free request over UDP 5351, so it needs no C
//! library and no `libminiupnpc` build step, and it is what Apple routers
//! and most consumer routers speak. UPnP IGD is SOAP over HTTP and is
//! broader but much larger for the same outcome.
//!
//! Mapping a port only helps the *sharer*; the viewer still has to be
//! reachable. That is the job of the rung below this one, and the
//! diagnosis says so rather than implying a mapping is a solution.

use anyhow::{Context, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

/// RFC 6886 puts both the control port and the gateway port here.
const GATEWAY_PORT: u16 = 5351;
/// RFC 6886 caps a mapping at 2^32 - 1 seconds; an hour is the conventional
/// ask and long enough that a shared session does not have to renew
/// constantly, short enough that a forgotten mapping expires.
const MAPPING_LIFETIME: u32 = 3600;
const TIMEOUT: Duration = Duration::from_secs(2);

/// A mapping this process asked for, and may want to release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Mapping {
    pub internal_port: u16,
    pub external_port: u16,
    pub lifetime: u32,
}

/// Ask the default gateway to forward `internal_port` to this host.
///
/// Returns `None` when there is no NAT-PMP responder, which is a normal
/// outcome on a machine that is already directly reachable.
pub async fn map_port(internal_port: u16) -> Result<Option<Mapping>> {
    let gateway = default_gateway().await?;
    let Some(gateway) = gateway else {
        return Ok(None);
    };
    let sock = tokio::net::UdpSocket::bind("0.0.0.0:0")
        .await
        .context("could not open a UDP socket for NAT-PMP")?;
    sock.connect(SocketAddr::from((gateway, GATEWAY_PORT)))
        .await
        .with_context(|| format!("could not reach the NAT-PMP responder at {gateway}"))?;

    // Ask for the external address first: RFC 6886 says a gateway may
    // reject a port request from a client it has no external address for.
    let external_request = build_external_request(internal_port);
    let reply = exchange(&sock, &external_request, "external address").await?;
    if reply.is_empty() {
        return Ok(None);
    }

    let port_request = build_port_request(internal_port, MAPPING_LIFETIME);
    let reply = exchange(&sock, &port_request, "port mapping").await?;
    let Some((result, internal_port, lifetime, external_port)) = parse_port_reply(&reply) else {
        // A short or empty answer means "no responder", not "broken".
        return Ok(None);
    };
    if result != 0 {
        // 1 = unsupported, 3 = internal port conflict, 4 = external port
        // conflict, 5 = mapping already exists, 6 = gateway resource
        // shortage. None of these is worth failing startup over.
        anyhow::bail!("NAT-PMP refused the mapping: result {result}");
    }
    Ok(Some(Mapping {
        internal_port,
        external_port,
        lifetime,
    }))
}

/// Release a mapping this process created.
pub async fn unmap_port(internal_port: u16) -> Result<()> {
    let Some(gateway) = default_gateway().await? else {
        return Ok(());
    };
    let Ok(sock) = tokio::net::UdpSocket::bind("0.0.0.0:0").await else {
        return Ok(());
    };
    if sock
        .connect(SocketAddr::from((gateway, GATEWAY_PORT)))
        .await
        .is_err()
    {
        return Ok(());
    }
    let mut request = build_port_request(internal_port, 0);
    // op 2 = DeleteOperation, big-endian at bytes 2..4.
    request[2] = 0;
    request[3] = 2;
    let _ = exchange(&sock, &request, "port delete").await;
    Ok(())
}

async fn exchange(sock: &tokio::net::UdpSocket, request: &[u8], what: &str) -> Result<Vec<u8>> {
    sock.send(request)
        .await
        .with_context(|| format!("could not send the NAT-PMP {what} request"))?;
    let mut buf = [0u8; 32];
    let n = tokio::time::timeout(TIMEOUT, sock.recv(&mut buf))
        .await
        .map_err(|_| anyhow::anyhow!("no answer to the NAT-PMP {what} request within {TIMEOUT:?}"))?
        .context("NAT-PMP read failed")?;
    Ok(buf[..n].to_vec())
}

/// The default gateway, found by asking the kernel which source address
/// it would use for a public destination. That is the same trick
/// `has_default_route` uses, and it needs no routing-table parsing.
pub(crate) async fn default_gateway() -> Result<Option<Ipv4Addr>> {
    let probe: SocketAddr = "198.51.100.1:9".parse().expect("literal");
    let Ok(sock) = tokio::net::UdpSocket::bind("0.0.0.0:0").await else {
        return Ok(None);
    };
    if sock.connect(probe).await.is_err() {
        return Ok(None);
    }
    // The probe is IPv4, so the local address is too.
    match sock.local_addr().ok().map(|a| a.ip()) {
        Some(IpAddr::V4(v4)) => Ok(Some(v4)),
        _ => Ok(None),
    }
}

fn build_external_request(internal_port: u16) -> Vec<u8> {
    let mut v = Vec::with_capacity(12);
    v.extend_from_slice(&0u16.to_be_bytes()); // version 0
    v.extend_from_slice(&0u16.to_be_bytes()); // op 0 = external address
    v.extend_from_slice(&[0u8; 4]); // reserved
    v.extend_from_slice(&internal_port.to_be_bytes());
    v.extend_from_slice(&0u16.to_be_bytes()); // suggested port
    v
}

/// A 16-byte port-mapping request, per RFC 6886 section 3.2:
///
/// ```text
/// 0..2 version   2..4 op=1 (map UDP)   4..8 reserved
/// 8..10 internal port   10..12 requested external port   12..16 lifetime
/// ```
fn build_port_request(internal_port: u16, lifetime: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.extend_from_slice(&0u16.to_be_bytes());
    v.extend_from_slice(&1u16.to_be_bytes()); // op 1 = map UDP
    v.extend_from_slice(&[0u8; 4]); // reserved
    v.extend_from_slice(&internal_port.to_be_bytes());
    // Asking for the same port keeps the advertised URL simple; a gateway
    // that cannot honour it answers with a different external port.
    v.extend_from_slice(&internal_port.to_be_bytes());
    v.extend_from_slice(&lifetime.to_be_bytes());
    v
}

/// Read an 18-byte port-mapping response:
///
/// ```text
/// 0..2 version   2..4 op   4..6 result   6..10 epoch
/// 10..12 internal port   12..14 external port   14..18 lifetime
/// ```
fn parse_port_reply(reply: &[u8]) -> Option<(u16, u16, u32, u16)> {
    if reply.len() < 18 {
        return None;
    }
    Some((
        u16::from_be_bytes([reply[4], reply[5]]),   // result
        u16::from_be_bytes([reply[10], reply[11]]), // internal
        u32::from_be_bytes([reply[14], reply[15], reply[16], reply[17]]), // lifetime
        u16::from_be_bytes([reply[12], reply[13]]), // external
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_external_request_is_twelve_bytes_and_well_formed() {
        let r = build_external_request(5800);
        assert_eq!(r.len(), 12);
        assert_eq!(&r[0..2], &[0, 0], "version 0");
        assert_eq!(&r[2..4], &[0, 0], "external address request");
        assert_eq!(&r[4..8], &[0, 0, 0, 0], "reserved");
        assert_eq!(&r[8..10], &5800u16.to_be_bytes(), "internal port");
        assert_eq!(&r[10..12], &[0, 0], "suggested port");
    }

    #[test]
    fn the_port_request_carries_the_lifetime_in_seconds() {
        let r = build_port_request(5800, 3600);
        assert_eq!(r.len(), 16, "RFC 6886 section 3.2");
        assert_eq!(&r[0..2], &[0, 0], "version 0");
        assert_eq!(&r[2..4], &[0, 1], "op 1 = map UDP");
        assert_eq!(&r[4..8], &[0, 0, 0, 0], "reserved");
        assert_eq!(&r[8..10], &5800u16.to_be_bytes(), "internal port");
        assert_eq!(&r[10..12], &5800u16.to_be_bytes(), "external port");
        assert_eq!(&r[12..16], &3600u32.to_be_bytes(), "lifetime in seconds");
    }

    #[test]
    fn a_delete_request_is_the_map_request_with_op_two() {
        let mut r = build_port_request(5800, 0);
        r[2] = 0;
        r[3] = 2;
        assert_eq!(&r[2..4], &[0, 2], "op 2 = delete");
        assert_eq!(&r[8..10], &5800u16.to_be_bytes());
        assert_eq!(&r[12..16], &[0, 0, 0, 0], "lifetime 0 releases it");
    }

    #[test]
    fn a_port_reply_is_read_at_the_right_offsets() {
        let mut reply = vec![0u8; 18];
        reply[0..2].copy_from_slice(&0u16.to_be_bytes()); // version
        reply[2..4].copy_from_slice(&129u16.to_be_bytes()); // op = 128 + 1
        reply[4..6].copy_from_slice(&0u16.to_be_bytes()); // result = success
        reply[6..10].copy_from_slice(&7u32.to_be_bytes()); // epoch
        reply[10..12].copy_from_slice(&5800u16.to_be_bytes());
        reply[12..14].copy_from_slice(&5801u16.to_be_bytes());
        reply[14..18].copy_from_slice(&3600u32.to_be_bytes());
        let (result, internal, lifetime, external) = parse_port_reply(&reply).unwrap();
        assert_eq!(
            (result, internal, lifetime, external),
            (0, 5800, 3600, 5801)
        );
    }

    #[test]
    fn a_short_port_reply_is_refused() {
        assert!(parse_port_reply(&[0u8; 12]).is_none());
    }
}
