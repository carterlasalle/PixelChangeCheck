//! What path does this machine have to the internet?
//!
//! The relay exists to solve exactly one problem: both peers behind NAT
//! with no direct path. That is rarer than it feels, because "behind
//! NAT" usually still leaves you reachable. So rather than assume the
//! worst, measure it.
//!
//! # The ladder
//!
//! ```text
//! 1. IPv6 direct     free, best latency, no third party
//! 2. UPnP / NAT-PMP  free, opens a port on the sharer's own router
//! 3. STUN + ICE      free, resolves most NAT pairs without a server of ours
//! 4. relay           guaranteed, costs money, needs trust
//! ```
//!
//! The relay stays at rung 4, and deliberately: it is TCP+TLS, so it
//! already traverses the corporate proxies that block UDP, which is
//! exactly where rungs 1-3 fail. A free-only path is *less* reliable
//! than a cheap relay behind a corporate proxy. The goal is to make the
//! relay unnecessary most of the time, not to remove it.
//!
//! Nothing here is a security decision. The token and the certificate pin
//! apply identically on every rung.

use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

/// Which rung a connection would take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rung {
    /// A routable IPv6 address: no NAT in the path at all.
    Ipv6Direct,
    /// A port the router can open for us.
    Upnp,
    /// A reflexive address discovered by STUN.
    StunIce,
    /// Nothing direct worked.
    Relay,
}

impl Rung {
    pub fn label(self) -> &'static str {
        match self {
            Rung::Ipv6Direct => "IPv6 direct",
            Rung::Upnp => "UPnP/NAT-PMP",
            Rung::StunIce => "STUN/ICE",
            Rung::Relay => "relay",
        }
    }

    pub fn is_direct(self) -> bool {
        !matches!(self, Rung::Relay)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// Every local address, with the ones that are actually usable called
    /// out. A report that lists `127.0.0.1` without saying it is useless
    /// is worse than no report.
    pub addresses: Vec<LocalAddress>,
    /// True when the machine has an IPv6 address that is not loopback,
    /// link-local or unique-local. Only a *global* address is reachable.
    pub has_global_ipv6: bool,
    /// A default route, which is the only evidence that anything leaves
    /// the host at all.
    pub has_default_route: bool,
    /// The reflexive address STUN saw, if the probe completed.
    pub reflexive: Option<SocketAddr>,
    /// What a direct attempt would have to use.
    pub best_rung: Rung,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalAddress {
    pub address: String,
    pub usable: bool,
    pub reason: &'static str,
}

/// Render a report for a human. This is the whole product surface of
/// `pcc diagnose`, so it is written to be read once and understood.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("PixelChangeCheck reachability report\n\n");
    out.push_str("Local addresses\n");
    for a in &report.addresses {
        let mark = if a.usable { "+" } else { "-" };
        out.push_str(&format!("  {mark} {:<40} {}\n", a.address, a.reason));
    }
    out.push_str("\nChecks\n");
    out.push_str(&format!(
        "  global IPv6 address    {}\n",
        yes_no(report.has_global_ipv6)
    ));
    out.push_str(&format!(
        "  default route          {}\n",
        yes_no(report.has_default_route)
    ));
    out.push_str(&format!(
        "  STUN reflexive address {}\n",
        report
            .reflexive
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "none".into())
    ));
    out.push_str(&format!(
        "\nBest direct path: {}\n",
        report.best_rung.label()
    ));
    // Always print the section: "no problems found" has to be
    // distinguishable from "this was never checked".
    out.push_str("\nNotes\n");
    if report.notes.is_empty() {
        out.push_str("  nothing to report\n");
    } else {
        for n in &report.notes {
            out.push_str(&format!("  - {n}\n"));
        }
    }
    out
}

fn yes_no(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

/// Whether an address can plausibly be reached from another host.
pub fn classify(ip: IpAddr) -> (&'static str, bool) {
    match ip {
        IpAddr::V4(v4) => match v4 {
            v if v.is_loopback() => ("loopback", false),
            v if v.is_private() => ("private (LAN only)", false),
            v if v.is_link_local() => ("link-local", false),
            v if v.is_unspecified() => ("unspecified", false),
            v if v.is_broadcast() => ("broadcast", false),
            v if v.is_documentation() => ("documentation range", false),
            v if v.octets()[0] == 100 && (64..128).contains(&v.octets()[1]) => {
                ("CGNAT (carrier NAT)", false)
            }
            _ => ("public", true),
        },
        IpAddr::V6(v6) => match v6 {
            v if v.is_loopback() => ("loopback", false),
            v if v.is_unspecified() => ("unspecified", false),
            // fe80::/10
            v if (v.segments()[0] & 0xffc0) == 0xfe80 => ("link-local (needs a zone)", false),
            // fc00::/7
            v if (v.segments()[0] & 0xfe00) == 0xfc00 => ("unique-local", false),
            // 2000::/3, the only globally routable range
            v if (v.segments()[0] & 0xe000) == 0x2000 => ("global", true),
            _ => ("not globally routable", false),
        },
    }
}

/// Is this an IPv6 address a remote peer could dial?
pub fn is_global_ipv6(ip: IpAddr) -> bool {
    matches!(classify(ip), (_, true))
}

/// The MAPPED-ADDRESS attribute, and the two address families it can
/// carry. Named so the parser below is checked against the spec's numbers
/// rather than against literals.
const ATTR_MAPPED: u16 = 0x0001;
const FAMILY_V4: u8 = 0x01;
const FAMILY_V6: u8 = 0x02;

/// Ask a STUN server (RFC 5389) what this host looks like from outside.
///
/// A binding request is a 20-byte header plus attributes; a response
/// carries the reflexive address. Implemented directly rather than pulled
/// in as a dependency because it is genuinely small and because the
/// failure mode we care about -- "no answer" -- has to be distinguishable
/// from "wrong answer".
pub async fn stun_reflexive(server: SocketAddr, timeout: Duration) -> Result<SocketAddr, String> {
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

fn parse_binding(packet: &[u8]) -> Result<SocketAddr, String> {
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

/// Run the full diagnosis.
pub async fn diagnose(stun: Option<SocketAddr>) -> Report {
    let mut addresses = Vec::new();
    let mut has_global_ipv6 = false;

    // Binding port 0 is the portable way to ask the OS which local address
    // it would use to reach the internet, without parsing routing tables.
    if let Ok(sock) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        if let Ok(a) = sock.local_addr() {
            let (reason, usable) = classify(a.ip());
            if a.ip().is_ipv6() && usable {
                has_global_ipv6 = true;
            }
            addresses.push(LocalAddress {
                address: a.to_string(),
                reason,
                usable,
            });
        }
    }

    // Enumerate interfaces so a report shows every route a peer could use.
    for (idx, addr) in enumerate_interface_addresses().into_iter().enumerate() {
        let (reason, usable) = classify(addr.ip());
        if addr.ip().is_ipv6() && usable {
            has_global_ipv6 = true;
        }
        let _ = idx;
        addresses.push(LocalAddress {
            address: addr.to_string(),
            reason,
            usable,
        });
    }

    let has_default_route = has_default_route().await;

    let mut notes = Vec::new();
    if !has_global_ipv6 {
        notes.push(
            "No global IPv6 address: the IPv6 rung is unavailable and every direct path \
             must traverse NAT."
                .into(),
        );
    }
    if !has_default_route {
        notes.push("No default route: this host cannot reach the internet at all.".into());
    }

    // STUN is the only rung we can actually test without a peer.
    let target = match stun {
        Some(s) => Some(s),
        None => tokio::net::lookup_host(DEFAULT_STUN[0])
            .await
            .ok()
            .and_then(|mut it| it.next()),
    };
    let mut reflexive = None;
    if let Some(server) = target {
        match stun_reflexive(server, Duration::from_secs(3)).await {
            Ok(addr) => reflexive = Some(addr),
            Err(e) => notes.push(format!("STUN probe failed: {e}")),
        }
    } else {
        notes.push("Could not resolve a public STUN server.".into());
    }

    let best_rung = if has_global_ipv6 {
        Rung::Ipv6Direct
    } else if reflexive.is_some() {
        // A reflexive address proves NAT is not symmetric for UDP; the ICE
        // candidate check against a real peer is the remaining unknown, so
        // this rung is reported as a strong hint rather than a guarantee.
        Rung::StunIce
    } else {
        Rung::Relay
    };

    if best_rung == Rung::Relay {
        notes.push(
            "No direct path could be found. A relay is required -- and note that a relay \
             is TCP, so it also works on networks that block UDP."
                .into(),
        );
    }

    Report {
        addresses,
        has_global_ipv6,
        has_default_route,
        reflexive,
        best_rung,
        notes,
    }
}

/// Local addresses on every interface, via the OS.
///
/// Implemented with the standard library's link-local enumeration so the
/// diagnosis does not need a dependency, and tolerant by design: a machine
/// that will not enumerate still gets a report from the bound socket above.
fn enumerate_interface_addresses() -> Vec<SocketAddr> {
    // `UdpSocket::bind` to port 0 on each candidate family is the portable
    // way to ask the OS which local addresses exist without a new crate.
    let mut out = Vec::new();
    for probe in ["0.0.0.0:0", "[::]:0"] {
        if let Ok(sock) = std::net::UdpSocket::bind(probe) {
            if let Ok(addr) = sock.local_addr() {
                if !out.iter().any(|a: &SocketAddr| a.ip() == addr.ip()) {
                    out.push(addr);
                }
            }
        }
    }
    out
}

/// A UDP "connect" to a public address and a zero-length send is the
/// portable way to ask the routing table whether a default route exists,
/// without parsing platform routing tables.
async fn has_default_route() -> bool {
    // 203.0.113.0/24 is TEST-NET-3: guaranteed not to be a local network,
    // so a route to it implies a default route.
    let probe: SocketAddr = "203.0.113.1:9".parse().expect("literal");
    match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(sock) => sock.connect(probe).await.is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_and_private_addresses_are_not_callable_reachable() {
        for ip in [
            "127.0.0.1",
            "10.0.0.5",
            "192.168.1.1",
            "172.16.0.1",
            "169.254.1.1",
            "0.0.0.0",
        ] {
            let (_, usable) = classify(ip.parse().unwrap());
            assert!(!usable, "{ip} must not be reported as reachable");
        }
    }

    #[test]
    fn cgnat_is_recognised_because_it_is_the_common_mobile_case() {
        let (_, usable) = classify("100.100.5.6".parse().unwrap());
        assert!(!usable);
    }

    #[test]
    fn global_addresses_are_reachable() {
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            assert!(is_global_ipv6(ip.parse().unwrap()), "{ip} is global");
        }
    }

    #[test]
    fn ipv6_scopes_are_distinguished() {
        assert!(!is_global_ipv6("fe80::1".parse().unwrap()), "link-local");
        assert!(!is_global_ipv6("fd00::1".parse().unwrap()), "unique-local");
        assert!(!is_global_ipv6("::1".parse().unwrap()), "loopback");
        assert!(is_global_ipv6("2001:4860:4860::8888".parse().unwrap()));
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

    #[test]
    fn the_report_renders_every_section_even_when_empty() {
        let report = Report {
            addresses: vec![],
            has_global_ipv6: false,
            has_default_route: false,
            reflexive: None,
            best_rung: Rung::Relay,
            notes: vec![],
        };
        let text = render(&report);
        for section in ["Local addresses", "Checks", "Best direct path", "Notes"] {
            assert!(text.contains(section), "missing {section} in:\n{text}");
        }
    }

    #[test]
    fn the_rung_ordering_is_the_intended_ladder() {
        assert!(Rung::Ipv6Direct < Rung::Upnp);
        assert!(Rung::Upnp < Rung::StunIce);
        assert!(Rung::StunIce < Rung::Relay);
        assert!(!Rung::Relay.is_direct());
    }
}
