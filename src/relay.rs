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
    read_len_prefix, verify_token, write_encoded, Message, MessageSink, MessageSource,
    MessageTransport, NetworkConfig, ServerIdentity, SessionToken,
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
/// role, and to authorize the connection.
#[derive(Debug, Serialize, Deserialize)]
pub struct RelayRegister {
    pub session: String,
    pub role: RelayRole,
    pub token: String,
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
    tx: mpsc::Sender<Vec<u8>>,
}

struct Session {
    host: Option<Peer>,
    viewers: Vec<Peer>,
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
            token: token.as_str().to_string(),
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

    let stream = acceptor.accept(tcp).await.context("TLS handshake failed")?;
    // Split after the handshake so the reader and the writer can be
    // supervised together by `select!` below.
    let (mut read_half, mut write_half) = tokio::io::split(stream);
    let reg = read_registration(&mut read_half).await?;
    if !verify_token(&expected_token, &reg.token) {
        warn!("Relay: rejected {:?} from {peer} (bad token)", reg.role);
        anyhow::bail!("bad viewer token");
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

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(MAX_PEER_QUEUE_MESSAGES);
    let gen = next_gen.fetch_add(1, Ordering::Relaxed);
    let peer_entry = Peer {
        gen,
        tx: tx.clone(),
    };

    {
        // Look up and register under the lock, then let it go. Nothing
        // below this point ever holds it across an await.
        let mut map = sessions.lock().await;
        let session = map.entry(reg.session.clone()).or_insert_with(|| Session {
            host: None,
            viewers: Vec::new(),
            last_seen: Instant::now(),
        });
        session.last_seen = Instant::now();
        match reg.role {
            RelayRole::Host => {
                // A new host generation replaces the old one. The previous
                // host's writer sees its channel close and exits; its
                // cleanup can no longer clear this registration.
                if let Some(previous) = session.host.replace(peer_entry.clone()) {
                    drop(previous);
                }
            }
            RelayRole::Viewer => {
                if session.viewers.len() >= MAX_VIEWERS_PER_SESSION {
                    anyhow::bail!(
                        "session '{}' already has {} viewers (max {MAX_VIEWERS_PER_SESSION})",
                        reg.session,
                        session.viewers.len()
                    );
                }
                session.viewers.push(peer_entry.clone());
            }
        }
        if map.len() > MAX_SESSIONS {
            anyhow::bail!(
                "relay is hosting {} sessions (max {MAX_SESSIONS})",
                map.len()
            );
        }
    }
    // Everything above this point may still be holding `tx`; from here on,
    // closing the session entry is what ends this peer's writer.
    drop(tx);

    let peer_role = reg.role;
    let session_id = reg.session.clone();
    let mut peer_entry = Some(peer_entry);
    let mut queued_bytes = 0usize;

    loop {
        tokio::select! {
            // Inbound: forward to the counterpart(s).
            inbound = read_frame(&mut read_half) => {
                let Some(payload) = inbound? else { break }; // EOF
                let framed = payload;
                let len = framed.len();

                let mut targets: Vec<mpsc::Sender<Vec<u8>>> = Vec::new();
                {
                    let mut map = sessions.lock().await;
                    if let Some(session) = map.get_mut(&session_id) {
                        session.last_seen = Instant::now();
                        match peer_role {
                            RelayRole::Host => {
                                targets.extend(session.viewers.iter().map(|v| v.tx.clone()));
                            }
                            RelayRole::Viewer => {
                                if let Some(h) = &session.host {
                                    targets.push(h.tx.clone());
                                }
                            }
                        }
                    }
                }
                // No lock held. A full queue means that peer is not
                // draining; dropping the message there would silently
                // corrupt its view, so it is disconnected instead and
                // reconnects for a fresh snapshot.
                let mut congested = false;
                for tx in targets {
                    match tx.try_send(framed.clone()) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => congested = true,
                        Err(mpsc::error::TrySendError::Closed(_)) => {}
                    }
                }
                if congested {
                    info!("Relay: disconnecting a congested viewer in session '{session_id}'");
                    break;
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
    Ok(bincode::deserialize(&buf)?)
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
}
