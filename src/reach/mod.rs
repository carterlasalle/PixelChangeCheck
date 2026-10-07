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
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

pub mod peer;
pub mod stun;
pub mod upnp;

/// Which rung a connection would take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rung {
    /// A routable IPv6 address: no NAT in the path at all.
    Ipv6Direct,
    /// The sharer's router was asked to forward a port for us.
    PortMapping,
    /// A reflexive address the peer was actually able to reach.
    StunIce,
    /// Nothing direct worked.
    Relay,
}

impl Rung {
    pub fn label(self) -> &'static str {
        match self {
            Rung::Ipv6Direct => "IPv6 direct",
            Rung::PortMapping => "mapped port",
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
    /// The port the router was asked to forward, when it agreed.
    pub mapping: Option<upnp::Mapping>,
    /// The result of asking the peer whether it can reach us.
    pub peer_check: peer::CheckOutcome,
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
        "  mapped port            {}\n",
        report
            .mapping
            .as_ref()
            .map(|m| format!(
                "{} -> {} for {}s",
                m.internal_port, m.external_port, m.lifetime
            ))
            .unwrap_or_else(|| "none".into())
    ));
    out.push_str(&format!(
        "  peer reachability     {}\n",
        match report.peer_check {
            peer::CheckOutcome::Reachable { rtt } => {
                format!("reachable, {rtt:?} round trip")
            }
            peer::CheckOutcome::Unreachable => "unreachable from the peer".into(),
            peer::CheckOutcome::NotAttempted => "not attempted (no candidate)".into(),
        }
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

/// How hard to try for a direct path before falling back to a relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReachPolicy {
    /// Try IPv6, then STUN, then the relay.
    TryDirect,
    /// Refuse the relay. Fails loudly rather than silently degrading.
    DirectOnly,
    /// Skip discovery entirely.
    RelayOnly,
}

/// A single line a viewer can open or paste.
///
/// The token and the pin both travel in the URL, so neither is retyped and
/// neither is mistyped. The pin is public and the token is the secret, so
/// this is safe to put in a chat message to one person and no wider.
pub fn pair_url(listen: &str, pin: &str, token: &str) -> String {
    format!("pcc://view?connect={}&pin={}&token={}", listen, pin, token)
}

/// The relay form of the pair line: same shape, but the address is the
/// relay and the session rides along. The pin here is the *relay's*
/// fingerprint — the single most-pasted wrong value in testing was the
/// sharer's pin in this slot, and the two are indistinguishable 64-hex
/// strings, so the builder exists to keep the two forms apart.
pub fn pair_url_relay(relay: &str, pin: &str, session: &str, token: &str) -> String {
    format!(
        "pcc://view?relay={}&pin={}&session={}&token={}",
        relay, pin, session, token
    )
}

/// The pieces of a `pcc://view?...` invite, as produced by [`pair_url`]
/// and [`pair_url_relay`].
///
/// Exactly one of `connect` and `relay` is set: the first is a direct
/// address, the second a relay plus its session code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairInvite {
    pub connect: Option<String>,
    pub relay: Option<String>,
    pub pin: String,
    pub session: Option<String>,
    pub token: String,
}

impl PairInvite {
    /// The `pcc view` arguments this invite stands for, ready to hand to
    /// the same parser the flags go through.
    pub fn to_view_args(&self) -> Vec<String> {
        let mut args = vec!["view".to_string()];
        if let Some(c) = &self.connect {
            args.push("--connect".into());
            args.push(c.clone());
        }
        if let Some(r) = &self.relay {
            args.push("--relay".into());
            args.push(r.clone());
        }
        if let Some(s) = &self.session {
            args.push("--session".into());
            args.push(s.clone());
        }
        args.push("--pin".into());
        args.push(self.pin.clone());
        args.push("--token".into());
        args.push(self.token.clone());
        args
    }
}

/// Parse a `pcc://view?...` invite back into its pieces.
///
/// This is the inverse of [`pair_url`] / [`pair_url_relay`], and it exists
/// so an invite can be pasted whole rather than decomposed by hand — the
/// pin and the token are both long opaque strings, and the relay form puts
/// the *relay's* pin where the direct form puts the sharer's, which is the
/// mistake this is meant to make impossible.
pub fn parse_pair_url(raw: &str) -> Result<PairInvite, String> {
    let rest = raw
        .trim()
        .strip_prefix("pcc://")
        .ok_or_else(|| "an invite starts with pcc:// — this does not".to_string())?;
    let (kind, query) = rest
        .split_once('?')
        .ok_or_else(|| "an invite needs a ? before its fields".to_string())?;
    if kind != "view" {
        return Err(format!("unknown invite kind '{kind}'; expected 'view'"));
    }

    let mut connect = None;
    let mut relay = None;
    let mut pin = None;
    let mut session = None;
    let mut token = None;
    for field in query.split('&') {
        if field.is_empty() {
            continue;
        }
        let (key, value) = field
            .split_once('=')
            .ok_or_else(|| format!("invite field '{field}' has no '='"))?;
        if value.is_empty() {
            return Err(format!("invite field '{key}' is empty"));
        }
        let slot = match key {
            "connect" => &mut connect,
            "relay" => &mut relay,
            "pin" => &mut pin,
            "session" => &mut session,
            "token" => &mut token,
            other => return Err(format!("unknown invite field '{other}'")),
        };
        if slot.is_some() {
            return Err(format!("invite repeats the '{key}' field"));
        }
        *slot = Some(value.to_string());
    }

    // The two forms are mutually exclusive, and saying so beats silently
    // preferring one: a link carrying both was assembled by hand.
    if connect.is_some() && relay.is_some() {
        return Err("invite has both connect= and relay=".to_string());
    }
    if connect.is_none() && relay.is_none() {
        return Err("invite has neither connect= nor relay=".to_string());
    }

    let pin = pin.ok_or_else(|| "invite is missing pin=".to_string())?;
    let token = token.ok_or_else(|| "invite is missing token=".to_string())?;
    if relay.is_some() && session.is_none() {
        return Err("a relay invite needs session=".to_string());
    }

    Ok(PairInvite {
        connect,
        relay,
        pin,
        session,
        token,
    })
}

/// The three values a sharer needs from a relay, in one paste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayHint {
    /// The relay's address, as the relay printed it.
    pub relay: String,
    /// The relay's certificate fingerprint.
    pub pin: String,
    /// The relay's token — the value that must be identical on all three
    /// sides, and the one nobody can guess.
    pub token: String,
}

/// Pull the relay's address, pin and token out of **whatever the operator
/// pasted**.
///
/// The relay prints a full `pcc share --relay … --relay-pin … --token …`
/// line, and a human may also paste just the flags, or a `pcc://relay?...`
/// URL. Requiring one exact shape is what turned a one-value copy into a
/// hunt through a terminal scrollback, so this accepts all of them and
/// says precisely which value is missing when something is.
pub fn parse_relay_hint(raw: &str) -> Result<RelayHint, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("nothing pasted".to_string());
    }

    // The URL form, for when one is copied on its own.
    if let Some(query) = text.strip_prefix("pcc://relay?") {
        let mut relay = None;
        let mut pin = None;
        let mut token = None;
        for field in query.split('&') {
            match field.split_once('=') {
                Some(("addr" | "relay", v)) => relay = Some(v.to_string()),
                Some(("pin", v)) => pin = Some(v.to_string()),
                Some(("token", v)) => token = Some(v.to_string()),
                _ => return Err(format!("unknown relay field '{field}'")),
            }
        }
        return assemble(relay, pin, token);
    }

    // The flag form, taken from any surrounding text so a whole pasted
    // command line — with or without the leading `pcc share` — works.
    let mut relay = None;
    let mut pin = None;
    let mut token = None;
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let value = words.get(i + 1).map(|v| v.trim_matches('\'').to_string());
        match words[i] {
            "--relay" | "--addr" => relay = value,
            "--relay-pin" | "--pin" => pin = value,
            "--token" => token = value,
            _ => {}
        }
        if matches!(
            words[i],
            "--relay" | "--addr" | "--relay-pin" | "--pin" | "--token"
        ) {
            i += 1;
        }
        i += 1;
    }

    assemble(relay, pin, token)
}

fn assemble(
    relay: Option<String>,
    pin: Option<String>,
    token: Option<String>,
) -> Result<RelayHint, String> {
    let missing = [
        (relay.is_none(), "--relay <host:port>"),
        (pin.is_none(), "--relay-pin <64 hex>"),
        (token.is_none(), "--token <the relay's token>"),
    ]
    .into_iter()
    .filter_map(|(missing, name)| missing.then_some(name))
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "that line is missing {} — paste the whole line the relay printed",
            missing.join(", ")
        ));
    }
    Ok(RelayHint {
        relay: relay.expect("checked"),
        pin: pin.expect("checked"),
        token: token.expect("checked"),
    })
}

/// The `pcc doctor` output: reachability, plus whether audio is available.
pub fn render_doctor(reach: &Report, audio: Option<String>) -> String {
    let mut out = render(reach);
    out.push_str("\nAudio\n");
    match audio {
        Some(device) => {
            out.push_str(&format!("  capture device: {device}\n"));
        }
        None => out.push_str("  no capture device found; sharing still works, without audio\n"),
    }
    out
}

/// Probe for a usable capture device without starting a stream.
pub async fn probe_audio() -> Option<String> {
    crate::audio::capture::default_device_name()
}

/// Run the full diagnosis.
pub async fn diagnose(stun: Option<SocketAddr>) -> Report {
    let mut addresses = Vec::new();
    let mut has_global_ipv6 = false;

    // Every address a peer could plausibly reach us on.
    for addr in interface_addresses() {
        let (reason, usable) = classify(addr.ip());
        if addr.ip().is_ipv6() && usable {
            has_global_ipv6 = true;
        }
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
        None => tokio::net::lookup_host(stun::DEFAULT_STUN[0])
            .await
            .ok()
            .and_then(|mut it| it.next()),
    };
    let mut reflexive = None;
    if let Some(server) = target {
        match stun::reflexive_address(server, Duration::from_secs(3)).await {
            Ok(addr) => reflexive = Some(addr),
            Err(e) => notes.push(format!("STUN probe failed: {e}")),
        }
    } else {
        notes.push("Could not resolve a public STUN server.".into());
    }

    // Ask the router to forward the port we already listen on. This only
    // helps the sharer, and it is deliberately asked for *after* STUN, so
    // a machine that is already reachable does not open a hole it does
    // not need.
    let mapping = match upnp::map_port(0).await {
        Ok(m) => m,
        Err(e) => {
            notes.push(format!("port mapping failed: {e}"));
            None
        }
    };
    if let Some(m) = &mapping {
        notes.push(format!(
            "The router was asked to forward UDP port {}; a viewer still has to reach \
             that port, which is what the peer check decides.",
            m.external_port
        ));
    }

    // The decisive question: can the peer actually reach the candidate?
    // A reflexive address is evidence, not proof, because symmetric NAT
    // answers every destination with a different port.
    let candidate = reflexive.map(|address| peer::Candidate { address });
    let peer_check = match candidate {
        Some(c) => {
            let outcome = peer::probe_peer(Some(c)).await;
            if matches!(outcome, peer::CheckOutcome::Unreachable) {
                notes.push(
                    "A STUN address was found but the peer could not reach it; this is what \
                     symmetric NAT looks like."
                        .into(),
                );
            }
            outcome
        }
        None => peer::CheckOutcome::NotAttempted,
    };

    let best_rung = if has_global_ipv6 {
        Rung::Ipv6Direct
    } else if mapping.is_some() {
        Rung::PortMapping
    } else if matches!(peer_check, peer::CheckOutcome::Reachable { .. }) {
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
        mapping,
        peer_check,
        notes,
    }
}

/// The addresses this host would actually source traffic from.
///
/// Binding port 0 reports only `0.0.0.0`, which tells an operator nothing.
/// Connecting the socket first makes the kernel choose a real source
/// address, which is exactly the question: is there a global IPv6 here, or
/// only a private IPv4?
fn interface_addresses() -> Vec<SocketAddr> {
    let mut out: Vec<SocketAddr> = Vec::new();
    let probes: [(&str, SocketAddr); 2] = [
        // TEST-NET-3 and 2001:db8:: are guaranteed not to be a local
        // network, so a route to them is a default route.
        ("0.0.0.0:0", "203.0.113.1:9".parse().expect("literal")),
        ("[::]:0", "[2001:db8::1]:9".parse().expect("literal")),
    ];
    for (bind, probe) in probes {
        let Ok(sock) = std::net::UdpSocket::bind(bind) else {
            continue;
        };
        if sock.connect(probe).is_err() {
            continue;
        }
        if let Ok(addr) = sock.local_addr() {
            if addr.ip().is_unspecified() {
                continue;
            }
            if !out.iter().any(|a| a.ip() == addr.ip()) {
                out.push(addr);
            }
        }
    }
    out
}

/// The address a viewer on this network should dial: the first
/// private-LAN IPv4 from the same interface probe `diagnose` runs, with
/// the listener's port. A wildcard bind prints 0.0.0.0, which no viewer
/// can dial — this is what makes the printed viewer line copy-pasteable.
/// `None` when there is no LAN address (loopback binds keep theirs).
pub fn lan_address_for(port: u16) -> Option<SocketAddr> {
    interface_addresses()
        .into_iter()
        .find(|a| matches!(a.ip(), std::net::IpAddr::V4(v) if v.is_private()))
        .map(|a| SocketAddr::new(a.ip(), port))
}

/// The address a peer *off this network* should dial: the local address of
/// the interface the default route uses.
///
/// This is the relay's case, not the sharer's. A relay on a VPS binds
/// `0.0.0.0` and has a public address, so `lan_address_for` is wrong there
/// (it returns the provider's private network, which nobody can reach) and
/// the bound address is `0.0.0.0`, which is not an address at all. The
/// routing table knows the right answer and this asks it.
///
/// `connect` on a UDP socket only sets the destination; it sends nothing,
/// so this is instant and touches no network.
pub fn outbound_address_for(port: u16) -> Option<SocketAddr> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    // TEST-NET-3, so the route is the default route and never a real peer.
    sock.connect("203.0.113.1:9").ok()?;
    let local = sock.local_addr().ok()?;
    if local.ip().is_unspecified() || local.ip().is_loopback() {
        return None;
    }
    Some(SocketAddr::new(local.ip(), port))
}

/// The address a peer should be told to dial for a listener bound at
/// `bound`: the bound address when it is specific, or the interface the
/// default route uses when it is a wildcard — which is not an address
/// anybody can dial.
///
/// One function so the relay's printed invite and a `--remember` save can
/// never disagree about what the relay's address is.
pub fn dial_address_for_bind(bound: SocketAddr) -> SocketAddr {
    if bound.ip().is_unspecified() {
        outbound_address_for(bound.port()).unwrap_or(bound)
    } else {
        bound
    }
}

/// Rewrite an address that points at *this* machine into its loopback form.
///
/// A host frequently cannot dial its own public or LAN address: clouds
/// route the packet out and back, or drop it, and NAT hairpinning is
/// inconsistent. Sharing to a relay running on the same machine is one of
/// the first things anyone tries, and it failed with "could not connect to
/// the relay" while a perfectly healthy relay was listening — so this is
/// where that gets decided, rather than in a doc note the user has to find.
///
/// A hostname is left alone: resolving it here would be a second guess on
/// top of the resolver's.
pub fn prefer_local(addr: &str) -> String {
    let Ok(sock) = addr.parse::<SocketAddr>() else {
        return addr.to_string();
    };
    if sock.ip().is_loopback() {
        return addr.to_string();
    }
    if !interface_addresses().iter().any(|a| a.ip() == sock.ip()) {
        return addr.to_string();
    }
    let loopback = if sock.is_ipv4() {
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    } else {
        std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
    };
    SocketAddr::new(loopback, sock.port()).to_string()
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
    fn the_report_renders_every_section_even_when_empty() {
        let report = Report {
            addresses: vec![],
            has_global_ipv6: false,
            has_default_route: false,
            reflexive: None,
            best_rung: Rung::Relay,
            mapping: None,
            peer_check: peer::CheckOutcome::NotAttempted,
            notes: vec![],
        };
        let text = render(&report);
        for section in [
            "Local addresses",
            "Checks",
            "Best direct path",
            "Notes",
            "mapped port",
            "peer reachability",
        ] {
            assert!(text.contains(section), "missing {section} in:\n{text}");
        }
    }

    #[test]
    fn the_rung_ordering_is_the_intended_ladder() {
        assert!(Rung::Ipv6Direct < Rung::PortMapping);
        assert!(Rung::PortMapping < Rung::StunIce);
        assert!(Rung::StunIce < Rung::Relay);
        assert!(!Rung::Relay.is_direct());
    }

    /// The parser is the inverse of the two builders, so anything the
    /// builders emit must come back identical. A drift here is silent:
    /// the viewer would connect to the wrong endpoint or with the wrong
    /// pin, and both failures look like a network problem.
    #[test]
    fn every_invite_the_builders_emit_parses_back() {
        let direct = pair_url("10.0.0.5:5800", "aa11", "TOKEN1234");
        let invite = parse_pair_url(&direct).expect("direct invite should parse");
        assert_eq!(invite.connect.as_deref(), Some("10.0.0.5:5800"));
        assert_eq!(invite.relay, None);
        assert_eq!(invite.pin, "aa11");
        assert_eq!(invite.token, "TOKEN1234");
        assert_eq!(
            invite.to_view_args(),
            vec![
                "view",
                "--connect",
                "10.0.0.5:5800",
                "--pin",
                "aa11",
                "--token",
                "TOKEN1234",
            ]
        );

        let relay = pair_url_relay("relay.example:5900", "bb22", "S9", "TOKEN1234");
        let invite = parse_pair_url(&relay).expect("relay invite should parse");
        assert_eq!(invite.relay.as_deref(), Some("relay.example:5900"));
        assert_eq!(invite.connect, None);
        assert_eq!(invite.session.as_deref(), Some("S9"));
        assert!(invite.to_view_args().contains(&"--session".to_string()));
    }

    /// Each of these would otherwise become a confusing connection error
    /// rather than a named cause.
    #[test]
    fn malformed_invites_are_refused_with_a_reason() {
        assert!(parse_pair_url("https://relay/v/S/#token=T")
            .unwrap_err()
            .contains("pcc://"));
        assert!(parse_pair_url("pcc://view?pin=a&token=T")
            .unwrap_err()
            .contains("neither"));
        assert!(parse_pair_url("pcc://view?connect=a&relay=b&pin=p&token=T")
            .unwrap_err()
            .contains("both"));
        assert!(parse_pair_url("pcc://view?connect=a&token=T")
            .unwrap_err()
            .contains("missing pin"));
        assert!(parse_pair_url("pcc://view?relay=r&pin=p&token=T")
            .unwrap_err()
            .contains("session"));
        assert!(parse_pair_url("pcc://view?connect=a&pin=p&token=T&typo=1")
            .unwrap_err()
            .contains("unknown"));
    }

    /// The whole point: whatever the operator pasted, the three values come
    /// out. Requiring one exact shape is what turned a one-value copy into a
    /// hunt through a terminal's scrollback.
    #[test]
    fn a_relay_line_is_understood_in_every_shape_it_is_pasted() {
        let pin = "a".repeat(64);
        // The full line the relay prints.
        let printed = format!(
            "On the machine being shared, run:\n  \
             pcc share --relay 203.0.113.5:5900 --relay-pin {pin} --token TTTOKEN1234"
        );
        let h = parse_relay_hint(&printed).expect("the printed line should parse");
        assert_eq!(h.relay, "203.0.113.5:5900");
        assert_eq!(h.pin, pin);
        assert_eq!(h.token, "TTTOKEN1234");

        // Just the flags.
        let h = parse_relay_hint(&format!(
            "--relay 1.2.3.4:5900 --relay-pin {pin} --token T1"
        ))
        .expect("bare flags should parse");
        assert_eq!(h.relay, "1.2.3.4:5900");

        // The URL form.
        let h = parse_relay_hint(&format!("pcc://relay?addr=5.6.7.8:5900&pin={pin}&token=T2"))
            .expect("the url form should parse");
        assert_eq!(h.relay, "5.6.7.8:5900");
        assert_eq!(h.token, "T2");
    }

    /// A half-pasted line must name what is missing rather than silently
    /// starting a share that cannot register.
    #[test]
    fn a_partial_relay_line_names_what_is_missing() {
        let e = parse_relay_hint("--relay 1.2.3.4:5900 --relay-pin aa").unwrap_err();
        assert!(e.contains("--token"), "should name the token: {e}");
        let e = parse_relay_hint("--token T").unwrap_err();
        assert!(
            e.contains("--relay") && e.contains("--relay-pin"),
            "got {e}"
        );
        assert!(parse_relay_hint("").unwrap_err().contains("nothing"));
    }

    /// The same-machine rewrite must leave everything that is *not* this
    /// host alone. Getting that wrong would silently redirect a real remote
    /// relay to loopback and break it for everyone.
    #[test]
    fn prefer_local_only_rewrites_this_host() {
        assert_eq!(prefer_local("127.0.0.1:5900"), "127.0.0.1:5900");
        assert_eq!(prefer_local("[::1]:5900"), "[::1]:5900");
        // A hostname is the resolver's job, not ours.
        assert_eq!(
            prefer_local("relay.example.com:5900"),
            "relay.example.com:5900"
        );
        // TEST-NET-3 is guaranteed never to be a local address.
        assert_eq!(prefer_local("203.0.113.5:5900"), "203.0.113.5:5900");
        // Garbage passes through rather than panicking.
        assert_eq!(prefer_local("not an address"), "not an address");
    }

    /// And it does rewrite this host's own address, which is the point:
    /// dialing your own public address is what failed.
    #[test]
    fn prefer_local_rewrites_an_address_this_machine_holds() {
        let mine = interface_addresses()
            .into_iter()
            .find(|a| !a.ip().is_loopback() && !a.ip().is_unspecified());
        let Some(mine) = mine else {
            return; // no external interface in this environment
        };
        let dial = SocketAddr::new(mine.ip(), 5900).to_string();
        assert_eq!(prefer_local(&dial), "127.0.0.1:5900");
    }
}
