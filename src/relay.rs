//! A relay for when a sharer and a viewer can't reach each other directly
//! (both behind NAT, no port forwarding).
//!
//! Both sides make *outbound* TLS connections to a relay host and register
//! with the same session code and the same viewer token. The relay pairs a
//! host with any number of viewers and forwards framed `Message` bytes. It
//! never inspects frame contents, so a future end-to-end encryption layer
//! needs no changes here.
//!
//! What it does enforce, because it faces untrusted networks:
//!
//! * every length prefix is checked against the frame budget *before* the
//!   buffer is allocated;
//! * the session map is never held across an `await`, so one slow viewer
//!   cannot stall another session;
//! * each viewer has its own byte budget, and a viewer that stops draining
//!   is disconnected rather than allowed to grow a queue forever;
//! * registrations are generational, so a reconnecting host's cleanup
//!   cannot erase its own replacement;
//! * reader and writer are supervised together, so EOF on either side
//!   tears the whole connection down.

use crate::network::{
    read_len_prefix, read_len_prefix_with_slack, relay_credential, write_encoded, Message,
    MessageSink, MessageSource, MessageTransport, NetworkConfig, ServerIdentity, SessionToken,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_rustls::client::TlsStream;
use tracing::{info, warn};

/// Sent once, immediately after connecting, to identify the session, the
/// role, and to authorize the connection. Carries the client's protocol
/// version so a relay can tell "old client" from "wrong token" — a
/// mismatch that previously cost a long debugging session on a correct
/// configuration (0.1.1/0.1.2 clients rejected as `bad credential`).
#[derive(Debug, Serialize, Deserialize)]
pub struct RelayRegister {
    pub session: String,
    pub role: RelayRole,
    pub token: String,
    /// The client's `PROTOCOL_VERSION`. Absent (0) on pre-0.1.5
    /// registrations, which predate the field.
    pub protocol_version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayRole {
    Host,
    Viewer,
}

/// Ceiling on a registration envelope.
const MAX_REGISTRATION_BYTES: usize = 4096;

/// Queued bytes one peer may fall behind by before it is disconnected.
///
/// Receipt: a 4K snapshot is at most a few MB. 8 MiB is roughly two
/// worst-case snapshots, so a viewer on a real link never trips this, while
/// a viewer that has stopped reading cannot pin an unbounded amount of
/// memory in the relay.
const MAX_PEER_QUEUE_BYTES: usize = 8 * 1024 * 1024;

/// Queued messages per peer, so many tiny messages cannot be cheaper to
/// buffer than a few large ones.
const MAX_PEER_QUEUE_MESSAGES: usize = 64;

/// Maximum concurrent sessions the relay will host.
const MAX_SESSIONS: usize = 1024;

/// Constant-time string equality, for a credential compared against a
/// remote peer on every connection. The loop runs over the longer of the
/// two so a length mismatch does not leak through timing.
fn same_credential(expected: &str, presented: &str) -> bool {
    let a = expected.as_bytes();
    let b = presented.as_bytes();
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

/// Maximum viewers per session.
const MAX_VIEWERS_PER_SESSION: usize = 64;

/// Failed registrations allowed from one peer address inside the window.
const AUTH_FAILURES_ALLOWED: u32 = 8;
const AUTH_FAILURE_WINDOW: Duration = Duration::from_secs(60);

/// A session with no traffic at all is forgotten.
const SESSION_IDLE_TTL: Duration = Duration::from_secs(3600);

/// Short rendezvous code used to find a session. Not a secret on its own --
/// the token is what authorizes.
pub fn generate_session_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no ambiguous chars
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

#[derive(Clone)]
struct Peer {
    /// Monotonic per connection. Cleanup only removes the registration it
    /// created, so a reconnecting host's predecessor cannot delete it.
    gen: u64,
    /// Relay-assigned viewer id, session-scoped and never reused. 0 is the
    /// host; viewers start at 1. The host leg carries this inside the
    /// outer length so every frame routes to exactly one viewer.
    id: u32,
    tx: mpsc::Sender<Vec<u8>>,
}

struct Session {
    host: Option<Peer>,
    viewers: Vec<Peer>,
    /// Next viewer id for this session. Starts at 1, never reused, so a
    /// replacement host sees the same ids its predecessor saw.
    next_viewer_id: u32,
    last_seen: Instant,
}

type Sessions = Arc<Mutex<HashMap<String, Session>>>;

/// A TLS-wrapped TCP connection to a relay, already registered and
/// authorized.
pub struct RelayTransport {
    stream: TlsStream<TcpStream>,
}

impl RelayTransport {
    /// Dial a relay, present the token, and wait for it to accept.
    pub async fn connect(
        relay_addr: std::net::SocketAddr,
        server_name: &str,
        pin: &[u8],
        session: String,
        token: SessionToken,
        role: RelayRole,
    ) -> Result<Self> {
        let config = NetworkConfig::client_tls_config(pin)?;
        let server_name = rustls::pki_types::ServerName::try_from(server_name.to_owned())
            .map_err(|e| anyhow::anyhow!("Invalid relay server name '{server_name}': {e}"))?;
        let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
        let tcp = TcpStream::connect(relay_addr)
            .await
            .with_context(|| format!("Failed to connect to the relay at {relay_addr}"))?;
        let mut stream = connector
            .connect(server_name, tcp)
            .await
            .with_context(|| format!("TLS handshake with the relay at {relay_addr} failed"))?;

        let reg = RelayRegister {
            session,
            role,
            token: relay_credential(&token),
            protocol_version: crate::network::PROTOCOL_VERSION,
        };
        let bytes = bincode::serialize(&reg)?;
        stream
            .write_all(&(bytes.len() as u32).to_le_bytes())
            .await?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;

        // The relay acknowledges a good registration with the protocol
        // version byte, so "connected" means "authorized" rather than
        // merely "TCP accepted".
        let mut ack = [0u8; 1];
        tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut ack))
            .await
            .context("Timed out waiting for the relay to accept the registration")?
            .context("The relay closed the connection during registration")?;
        if ack[0] != crate::network::PROTOCOL_VERSION {
            anyhow::bail!(
                "The relay rejected this viewer: check --session and --token \
                 (relay speaks protocol {}, offered {})",
                ack[0],
                crate::network::PROTOCOL_VERSION
            );
        }

        Ok(Self { stream })
    }
}

/// Bytes of peer id carried inside the outer length on the host leg.
pub const RELAY_ID_BYTES: usize = 4;
/// Peer id the relay uses for its own control frames. A host never emits
/// it; a viewer never sees it.
pub const RELAY_CONTROL_ID: u32 = 0xFFFF_FFFF;
/// Relay -> host: a viewer with this id is registered for the session.
const K_CONTROL_VIEWER_HERE: u8 = 0xEF;
/// Relay -> host: the viewer with this id disconnected.
const K_CONTROL_VIEWER_LEFT: u8 = 0xEE;

/// One host relay connection, multiplexed into per-viewer sessions.
///
/// `into_fan` splits the TLS stream and spawns a reader task (host-leg
/// framing -> `(peer_id, envelope)` on `rx`) and a writer task
/// (`(peer_id, envelope)` on `tx` -> host-leg framing). Both tasks end
/// when the relay connection dies; `rx` closing is the signal the caller
/// uses to tear its sessions down.
pub struct RelayFan {
    /// `(peer_id, envelope)`. `peer_id == RELAY_CONTROL_ID` means a relay
    /// control frame: `[kind u8][viewer_id u32 LE]`.
    pub rx: mpsc::Receiver<(u32, Vec<u8>)>,
    /// `(peer_id, envelope)`; the writer tags and length-fixes each one.
    pub tx: mpsc::Sender<(u32, Vec<u8>)>,
}

/// Read one host-leg frame: `[len][id][env]` or a relay control frame.
/// Returns `(id, envelope)`; control frames arrive as
/// `(RELAY_CONTROL_ID, [kind][viewer_id])`. `None` at EOF.
pub async fn read_host_frame<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<(u32, Vec<u8>)>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    // The id rides inside the outer length, so the cap needs its 4 bytes.
    let len = read_len_prefix_with_slack(&len_buf, RELAY_ID_BYTES)?;
    if len < RELAY_ID_BYTES {
        anyhow::bail!("host-leg frame too short for a peer id: {len} bytes");
    }
    let mut rest = vec![0u8; len];
    reader.read_exact(&mut rest).await?;
    let id = u32::from_le_bytes(rest[0..4].try_into().unwrap());
    Ok(Some((id, rest[4..].to_vec())))
}

/// Per-viewer queue on the host side of the fan.
const FAN_SESSION_QUEUE: usize = 64;

/// Tag an envelope for one viewer: `[len+4][id][env]`, fixing the outer
/// length in the same step. The relay is length-preserving, so the id
/// must live *inside* the length it forwards.
fn tag_host_frame(id: u32, envelope: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + envelope.len());
    out.extend_from_slice(&((envelope.len() + 4) as u32).to_le_bytes());
    out.extend_from_slice(&id.to_le_bytes());
    out.extend_from_slice(envelope);
    out
}

struct RelaySink {
    stream: tokio::io::WriteHalf<TlsStream<TcpStream>>,
}

#[async_trait]
impl MessageSink for RelaySink {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        self.send_encoded(&msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.stream, bytes).await
    }
}

struct RelaySource {
    stream: tokio::io::ReadHalf<TlsStream<TcpStream>>,
}

#[async_trait]
impl MessageSource for RelaySource {
    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.stream).await
    }

    /// Read one envelope without decoding it, which is what a sealed
    /// session needs: the ciphertext is not a `Message` yet.
    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        Message::read_envelope(&mut self.stream).await
    }
}

/// One host relay connection, multiplexed into per-viewer sessions.
///
/// The host learns each viewer's relay-assigned id from the tagged frames
/// (or `ViewerHere` replay) and runs one `serve_viewer` per id, each with
/// its own E2E keys. That is the whole fix: viewer A's ciphertext is
/// sealed under A's keys, never fanned to B.
impl RelayTransport {
    /// Split the host connection into per-viewer sessions. Host role only:
    /// the framing differs by role and only the host speaks host-leg.
    pub fn into_fan(self: Box<Self>) -> RelayFan {
        let (fan_tx_in, fan_rx_in) = mpsc::channel::<(u32, Vec<u8>)>(FAN_SESSION_QUEUE);
        let (fan_tx_out, mut fan_rx_out) = mpsc::channel::<(u32, Vec<u8>)>(FAN_SESSION_QUEUE);
        let (mut read_half, mut write_half) = tokio::io::split(self.stream);
        // Reader: host-leg framing -> (peer_id, envelope).
        tokio::spawn(async move {
            loop {
                match read_host_frame(&mut read_half).await {
                    Ok(Some((id, env))) => {
                        if fan_tx_in.send((id, env)).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });
        // Writer: (peer_id, envelope) -> host-leg framing, fixing the
        // outer length in the same step that inserts the id.
        tokio::spawn(async move {
            while let Some((id, env)) = fan_rx_out.recv().await {
                if write_half
                    .write_all(&tag_host_frame(id, &env))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        RelayFan {
            rx: fan_rx_in,
            tx: fan_tx_out,
        }
    }
}

#[async_trait]
impl MessageTransport for RelayTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        write_encoded(&mut self.stream, &msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        write_encoded(&mut self.stream, bytes).await
    }

    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.stream).await
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        let (read, write) = tokio::io::split(self.stream);
        (
            Box::new(RelaySink { stream: write }),
            Box::new(RelaySource { stream: read }),
        )
    }
}

/// Run a relay server until the listener errors.
///
/// The listener is supplied by the caller so the bound address is known
/// before the server starts: binding here would let the caller print an
/// address that turned out to be unavailable.
pub async fn run_relay_server(
    listener: TcpListener,
    identity: Arc<ServerIdentity>,
    token: SessionToken,
) -> Result<()> {
    let acceptor =
        tokio_rustls::TlsAcceptor::from(Arc::new(NetworkConfig::server_crypto_config(&identity)?));
    info!("Relay server listening on {} (TLS)", listener.local_addr()?);
    info!(
        "Relay certificate fingerprint (sha256): {}",
        identity.fingerprint
    );

    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
    let next_gen = Arc::new(AtomicU64::new(1));
    let failures: Arc<Mutex<HashMap<std::net::SocketAddr, (u32, Instant)>>> =
        Arc::new(Mutex::new(HashMap::new()));

    {
        let sessions = sessions.clone();
        let failures = failures.clone();
        tokio::spawn(async move { sweep(sessions, failures).await });
    }

    loop {
        let (tcp, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let sessions = sessions.clone();
        let next_gen = next_gen.clone();
        let token = token.clone();
        let failures = failures.clone();
        tokio::spawn(async move {
            if let Err(e) =
                handle_client(tcp, peer, acceptor, sessions, next_gen, token, failures).await
            {
                warn!("Relay peer {peer}: {e}");
            }
        });
    }
}

/// Retire sessions that have gone quiet, and forget auth-failure counters
/// that have aged out.
async fn sweep(
    sessions: Sessions,
    failures: Arc<tokio::sync::Mutex<HashMap<std::net::SocketAddr, (u32, Instant)>>>,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        {
            let mut map = sessions.lock().await;
            map.retain(|_, s| s.last_seen.elapsed() < SESSION_IDLE_TTL);
        }
        {
            let mut map = failures.lock().await;
            map.retain(|_, (_, at)| at.elapsed() < AUTH_FAILURE_WINDOW);
        }
    }
}

async fn handle_client(
    tcp: TcpStream,
    peer: std::net::SocketAddr,
    acceptor: tokio_rustls::TlsAcceptor,
    sessions: Sessions,
    next_gen: Arc<AtomicU64>,
    expected_token: SessionToken,
    failures: Arc<tokio::sync::Mutex<HashMap<std::net::SocketAddr, (u32, Instant)>>>,
) -> Result<()> {
    if record_and_check_rate(&failures, peer).await {
        anyhow::bail!(
            "too many failed registrations (max {AUTH_FAILURES_ALLOWED} per {}s)",
            AUTH_FAILURE_WINDOW.as_secs()
        );
    }

    let stream = match acceptor.accept(tcp).await {
        Ok(s) => s,
        Err(e) => {
            // A bare TCP connect (health check, port scan) fails the
            // handshake with no ClientHello to learn from. That is
            // noise, not an attack: debug, not warn, so a 2-minute
            // probe does not bury the journal in WARN lines.
            tracing::debug!("Relay peer {peer}: TLS handshake failed (bare connect?): {e}");
            return Ok(());
        }
    };
    // Split after the handshake so the reader and the writer can be
    // supervised together by `select!` below.
    let (mut read_half, mut write_half) = tokio::io::split(stream);
    let reg = read_registration(&mut read_half).await?;
    // Compared against a value derived from the session token, never
    // against the token itself: a relay that learns the session secret
    // could forge a viewer's encryption proof, and "the relay cannot read
    // the screen" is exactly the claim that has to hold against the
    // operator running it, not only against someone sniffing the wire.
    if !same_credential(relay_credential(&expected_token).as_str(), &reg.token) {
        // A pre-0.1.5 client sends the raw token where the relay expects
        // the HMAC credential, so it can never match — but the operator's
        // token is correct and the fix is a version bump, not a retype.
        // Say exactly that, with both versions named.
        if reg.protocol_version == 0 || reg.protocol_version != crate::network::PROTOCOL_VERSION {
            let client = if reg.protocol_version == 0 {
                "pre-0.1.5 (no version in registration)".to_string()
            } else {
                format!("protocol {}", reg.protocol_version)
            };
            warn!(
                "Relay: rejected {:?} from {peer}: client speaks {client}, this relay speaks protocol {} — upgrade the client, the token is not the problem",
                reg.role,
                crate::network::PROTOCOL_VERSION
            );
            anyhow::bail!(
                "client speaks {client}, this relay speaks protocol {}: upgrade the client",
                crate::network::PROTOCOL_VERSION
            );
        }
        warn!(
            "Relay: rejected {:?} from {peer} (bad credential)",
            reg.role
        );
        anyhow::bail!("bad {:?} credential", reg.role);
    }
    // The registration was good; clear the peer's failure count.
    failures.lock().await.remove(&peer);
    // Accept the connection, so the client can distinguish "authorized"
    // from "TCP accepted".
    write_half
        .write_all(&[crate::network::PROTOCOL_VERSION])
        .await?;
    write_half.flush().await?;

    info!(
        "Relay: {:?} joined session '{}' from {peer}",
        reg.role, reg.session
    );

    // The host speaks host-leg framing (peer id inside the outer length);
    // viewers speak the unchanged viewer leg. The role decides the parse.
    let host_channel = reg.role == RelayRole::Host;
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(if host_channel {
        MAX_PEER_QUEUE_MESSAGES + MAX_VIEWERS_PER_SESSION
    } else {
        MAX_PEER_QUEUE_MESSAGES
    });
    let gen = next_gen.fetch_add(1, Ordering::Relaxed);
    // Viewers learn their id at registration; the host learns it from the
    // tagged frames (or ViewerHere replay) that follow.
    let mut my_id: u32 = 0;
    let mut here_replay: Vec<u32> = Vec::new();
    let peer_entry = {
        let mut map = sessions.lock().await;
        let session = map.entry(reg.session.clone()).or_insert_with(|| Session {
            host: None,
            viewers: Vec::new(),
            next_viewer_id: 1,
            last_seen: Instant::now(),
        });
        session.last_seen = Instant::now();
        let entry = match reg.role {
            RelayRole::Host => {
                // A new host generation replaces the old one. The previous
                // host's writer sees its channel close and exits; its
                // cleanup can no longer clear this registration.
                let entry = Peer {
                    gen,
                    id: 0,
                    tx: tx.clone(),
                };
                if let Some(previous) = session.host.replace(entry.clone()) {
                    drop(previous);
                }
                // An idle viewer never speaks, so a host that arrives after
                // its viewers would never learn their ids without this.
                here_replay = session.viewers.iter().map(|v| v.id).collect();
                entry
            }
            RelayRole::Viewer => {
                if session.viewers.len() >= MAX_VIEWERS_PER_SESSION {
                    anyhow::bail!(
                        "session '{}' already has {} viewers (max {MAX_VIEWERS_PER_SESSION})",
                        reg.session,
                        session.viewers.len()
                    );
                }
                // Ids are a session property, never reused, so a replacement
                // host sees the same ids its predecessor saw.
                let id = session.next_viewer_id;
                session.next_viewer_id = session.next_viewer_id.saturating_add(1).max(1);
                if id == RELAY_CONTROL_ID {
                    anyhow::bail!("session '{}' exhausted its viewer ids", reg.session);
                }
                my_id = id;
                let entry = Peer {
                    gen,
                    id,
                    tx: tx.clone(),
                };
                session.viewers.push(entry.clone());
                entry
            }
        };
        if map.len() > MAX_SESSIONS {
            anyhow::bail!(
                "relay is hosting {} sessions (max {MAX_SESSIONS})",
                map.len()
            );
        }
        entry
    };
    // Everything above this point may still be holding `tx`; from here on,
    // closing the session entry is what ends this peer's writer.
    drop(tx);

    let peer_role = reg.role;
    let session_id = reg.session.clone();
    let mut peer_entry = Some(peer_entry);
    let mut queued_bytes = 0usize;

    // A host that arrives after its viewers learns their ids here: an
    // idle viewer never speaks, so without the replay the host would never
    // spawn a session for it.
    for id in here_replay {
        let mut frame = Vec::with_capacity(9);
        frame.extend_from_slice(&9u32.to_le_bytes());
        frame.extend_from_slice(&RELAY_CONTROL_ID.to_le_bytes());
        frame.push(K_CONTROL_VIEWER_HERE);
        frame.extend_from_slice(&id.to_le_bytes());
        if write_half.write_all(&frame).await.is_err() {
            break;
        }
    }
    loop {
        tokio::select! {
            // Inbound: forward to the counterpart(s).
            inbound = async {
                if host_channel {
                    // Split `[len][id][env]` into its parts; the unicast
                    // arm below reattaches the length it forwards.
                    read_host_frame(&mut read_half).await.map(|o| {
                        o.map(|(id, env)| {
                            let mut framed = Vec::with_capacity(8 + env.len());
                            framed.extend_from_slice(&id.to_le_bytes());
                            framed.extend_from_slice(&env);
                            framed
                        })
                    })
                } else {
                    read_frame(&mut read_half).await
                }
            } => {
                let Some(framed) = inbound? else { break }; // EOF
                let len = framed.len();

                if host_channel {
                    // `[id][env]` (prefix already consumed): unicast to
                    // exactly that viewer, reattaching the length in the
                    // same step. Unknown ids are drop-and-log, never fatal.
                    let id = u32::from_le_bytes(framed[0..4].try_into().unwrap());
                    if id == RELAY_CONTROL_ID {
                        warn!("Relay: host sent a control frame; dropping it");
                        continue;
                    }
                    let target = {
                        let map = sessions.lock().await;
                        map.get(&session_id).and_then(|session| {
                            session.viewers.iter().find(|v| v.id == id).map(|v| v.tx.clone())
                        })
                    };
                    let Some(target) = target else {
                        warn!("Relay: frame for unknown viewer id {id}; dropping it");
                        continue;
                    };
                    let env = framed.get(4..).unwrap_or(&[]);
                    let mut out = Vec::with_capacity(4 + env.len());
                    out.extend_from_slice(&(env.len() as u32).to_le_bytes());
                    out.extend_from_slice(env);
                    match target.try_send(out) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            warn!("Relay: dropping host: viewer {id} not draining");
                            break;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {}
                    }
                    // Touch liveness without holding the lock across writes.
                    if let Some(session) = sessions.lock().await.get_mut(&session_id) {
                        session.last_seen = Instant::now();
                    }
                } else {
                    // `[len][env]`: tag with this viewer's id and queue to
                    // the host, fixing the length in the same step. The
                    // incoming prefix covers the envelope only, so the
                    // tagged length grows by the 4 id bytes.
                    let framed = {
                        let env = framed.get(4..).unwrap_or(&[]);
                        let mut tagged = Vec::with_capacity(8 + env.len());
                        tagged.extend_from_slice(
                            &((env.len() + 4) as u32).to_le_bytes(),
                        );
                        tagged.extend_from_slice(&my_id.to_le_bytes());
                        tagged.extend_from_slice(env);
                        tagged
                    };
                    let target = {
                        let mut map = sessions.lock().await;
                        map.get_mut(&session_id).and_then(|session| {
                            session.last_seen = Instant::now();
                            session.host.as_ref().map(|h| h.tx.clone())
                        })
                    };
                    let Some(target) = target else { continue };
                    match target.try_send(framed) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            info!("Relay: disconnecting a congested viewer in session '{session_id}'");
                            break;
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {}
                    }
                }
                queued_bytes += len;
                if queued_bytes > MAX_PEER_QUEUE_BYTES {
                    warn!("Relay: peer {peer} queued {queued_bytes} bytes without draining; dropping it");
                    break;
                }
            }
            // Outbound: write to the socket.
            outbound = rx.recv() => {
                let Some(bytes) = outbound else { break };
                queued_bytes = queued_bytes.saturating_sub(bytes.len());
                if write_half.write_all(&bytes).await.is_err() {
                    break;
                }
            }
        }
    }

    // Unregister exactly the generation we created.
    {
        let mut map = sessions.lock().await;
        if let Some(session) = map.get_mut(&session_id) {
            if let Some(entry) = peer_entry.as_mut() {
                match peer_role {
                    RelayRole::Host => {
                        if session.host.as_ref().is_some_and(|h| h.gen == entry.gen) {
                            session.host = None;
                        }
                    }
                    RelayRole::Viewer => {
                        session.viewers.retain(|v| v.gen != entry.gen);
                    }
                }
            }
            // A viewer departure is told to the host so it can drop that
            // session; without it the host would seal frames into the void.
            let left_id = if peer_role == RelayRole::Viewer {
                Some(peer_entry.as_ref().map(|e| e.id).unwrap_or(0))
            } else {
                None
            };
            if let Some(host) = session.host.as_ref() {
                if let Some(id) = left_id {
                    let mut frame = Vec::with_capacity(9);
                    frame.extend_from_slice(&9u32.to_le_bytes());
                    frame.extend_from_slice(&RELAY_CONTROL_ID.to_le_bytes());
                    frame.push(K_CONTROL_VIEWER_LEFT);
                    frame.extend_from_slice(&id.to_le_bytes());
                    let _ = host.tx.try_send(frame);
                }
            }
            // A host's keys die with its connection and an established
            // viewer never re-offers, so viewers left behind would strand.
            // Bounce them: their connections close and they rejoin fresh.
            if session.host.is_none() && peer_role == RelayRole::Host {
                session.viewers.clear();
            }
            if session.host.is_none() && session.viewers.is_empty() {
                map.remove(&session_id);
            }
        }
    }
    info!("Relay: {:?} left session '{session_id}'", peer_role);
    Ok(())
}

/// One inbound frame, already length-checked, or `None` at EOF.
async fn read_frame<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    // The cap is applied here, before the allocation it authorises.
    let len = read_len_prefix(&len_buf)?;
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    let mut framed = Vec::with_capacity(4 + len);
    framed.extend_from_slice(&len_buf);
    framed.extend_from_slice(&payload);
    Ok(Some(framed))
}

async fn read_registration<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<RelayRegister> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len == 0 || len > MAX_REGISTRATION_BYTES {
        anyhow::bail!("relay registration size {len} is outside 1..={MAX_REGISTRATION_BYTES}");
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    // Pre-0.1.5 clients serialize without the trailing version byte.
    // bincode is fixed-order, so a legacy body parses as the same
    // prefix: try the current shape first, then the legacy one with
    // version 0 (meaning "older than versioning").
    if let Ok(reg) = bincode::deserialize::<RelayRegister>(&buf) {
        return Ok(reg);
    }
    #[derive(serde::Deserialize)]
    struct LegacyRegister {
        session: String,
        role: RelayRole,
        token: String,
    }
    let legacy: LegacyRegister = bincode::deserialize(&buf)?;
    Ok(RelayRegister {
        session: legacy.session,
        role: legacy.role,
        token: legacy.token,
        protocol_version: 0,
    })
}

/// Per-address failure counter. Returns true when the peer is over budget.
async fn record_and_check_rate(
    failures: &Arc<Mutex<HashMap<std::net::SocketAddr, (u32, Instant)>>>,
    peer: std::net::SocketAddr,
) -> bool {
    let mut map = failures.lock().await;
    let entry = map.entry(peer).or_insert((0, Instant::now()));
    if entry.1.elapsed() > AUTH_FAILURE_WINDOW {
        *entry = (0, Instant::now());
    }
    entry.0 += 1;
    entry.0 > AUTH_FAILURES_ALLOWED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_codes_are_six_unambiguous_symbols() {
        let code = generate_session_code();
        assert_eq!(code.len(), 6);
        assert!(code
            .chars()
            .all(|c| "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(c)));
    }

    #[tokio::test]
    async fn a_hostile_registration_length_is_refused_without_allocating() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        let mut cursor: &[u8] = &bytes;
        let err = read_registration(&mut cursor)
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("max_message_size") || err.contains("outside"),
            "unhelpful: {err}"
        );
    }

    #[tokio::test]
    async fn a_hostile_frame_length_is_refused_without_allocating() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        let mut cursor: &[u8] = &bytes;
        let err = read_frame(&mut cursor).await.unwrap_err().to_string();
        assert!(err.contains("max_message_size"), "unhelpful: {err}");
    }

    #[tokio::test]
    async fn host_leg_frames_roundtrip_through_the_relay_transform() {
        // A viewer-leg frame, tagged for one viewer and stripped back,
        // must come back byte-identical: the id lives inside the outer
        // length, and the prefix is fixed in the same step.
        let envelope = crate::network::Message::KeepAlive { rev: 7 }
            .encode()
            .unwrap();
        let mut viewer_leg = Vec::new();
        viewer_leg.extend_from_slice(&(envelope.len() as u32).to_le_bytes());
        viewer_leg.extend_from_slice(&envelope);
        let tagged = tag_host_frame(3, &envelope);
        let mut cursor: &[u8] = &tagged;
        let (id, env) = read_host_frame(&mut cursor)
            .await
            .unwrap()
            .expect("a tagged frame must parse");
        assert_eq!(id, 3);
        assert_eq!(env, envelope);
        // The viewer-bound strip restores the exact viewer-leg bytes.
        let mut out = Vec::with_capacity(4 + env.len());
        out.extend_from_slice(&(env.len() as u32).to_le_bytes());
        out.extend_from_slice(&env);
        assert_eq!(out, viewer_leg);
    }

    #[tokio::test]
    async fn a_host_leg_frame_too_short_for_an_id_is_refused() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let mut cursor: &[u8] = &bytes;
        let err = read_host_frame(&mut cursor).await.unwrap_err().to_string();
        assert!(err.contains("peer id"), "unhelpful: {err}");
    }

    #[tokio::test]
    async fn rate_limiting_trips_after_the_configured_failures() {
        let failures: Arc<Mutex<HashMap<std::net::SocketAddr, (u32, Instant)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let peer: std::net::SocketAddr = "10.0.0.1:5000".parse().unwrap();
        for i in 0..AUTH_FAILURES_ALLOWED {
            assert!(
                !record_and_check_rate(&failures, peer).await,
                "allowed {i} of {AUTH_FAILURES_ALLOWED} attempts"
            );
        }
        assert!(
            record_and_check_rate(&failures, peer).await,
            "the limit must trip"
        );
    }

    #[tokio::test]
    async fn legacy_registration_decodes_with_version_zero() {
        // A pre-0.1.5 client serializes session/role/token with no
        // trailing version byte. The relay must read it as version 0
        // (older than versioning), not reject the bytes.
        #[derive(serde::Serialize)]
        struct Legacy {
            session: String,
            role: RelayRole,
            token: String,
        }
        let body = bincode::serialize(&Legacy {
            session: "ABC123".into(),
            role: RelayRole::Host,
            token: "tok".into(),
        })
        .unwrap();
        let mut framed = (body.len() as u32).to_le_bytes().to_vec();
        framed.extend_from_slice(&body);
        let mut cursor: &[u8] = &framed;
        let reg = read_registration(&mut cursor).await.unwrap();
        assert_eq!(reg.protocol_version, 0);
        assert_eq!(reg.role, RelayRole::Host);
    }

    #[tokio::test]
    async fn current_registration_carries_the_protocol_version() {
        let reg = RelayRegister {
            session: "ABC123".into(),
            role: RelayRole::Viewer,
            token: "tok".into(),
            protocol_version: crate::network::PROTOCOL_VERSION,
        };
        let body = bincode::serialize(&reg).unwrap();
        let mut framed = (body.len() as u32).to_le_bytes().to_vec();
        framed.extend_from_slice(&body);
        let mut cursor: &[u8] = &framed;
        let back: RelayRegister = read_registration(&mut cursor).await.unwrap();
        assert_eq!(back.protocol_version, crate::network::PROTOCOL_VERSION);
    }
}
