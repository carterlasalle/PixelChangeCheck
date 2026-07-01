//! A minimal relay server for when a sharer and viewer can't reach each
//! other directly (e.g. both are behind NAT with no port forwarding).
//!
//! Both sides make *outbound* TCP connections to a relay host and register
//! with the same session code. The relay pairs a "host" connection with any
//! number of "viewer" connections and forwards raw framed `Message` bytes
//! between them. It never inspects frame contents; it just pipes bytes.

use crate::network::{Message, MessageTransport};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

/// Sent once, immediately after connecting to a relay, to identify which
/// session to join and in which role.
#[derive(Debug, Serialize, Deserialize)]
pub struct RelayRegister {
    pub session: String,
    pub role: RelayRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayRole {
    Host,
    Viewer,
}

async fn write_registration<W: AsyncWrite + Unpin>(w: &mut W, reg: &RelayRegister) -> Result<()> {
    let bytes = bincode::serialize(reg)?;
    w.write_all(&(bytes.len() as u32).to_le_bytes()).await?;
    w.write_all(&bytes).await?;
    w.flush().await?;
    Ok(())
}

async fn read_registration<R: AsyncRead + Unpin>(r: &mut R) -> Result<RelayRegister> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > 4096 {
        anyhow::bail!("Relay registration message too large");
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
}

/// Generate a short, human-typeable session code (e.g. for reading over the
/// phone), like "7F3K9A".
pub fn generate_session_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no ambiguous chars
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

struct Session {
    host: Option<mpsc::Sender<Vec<u8>>>,
    viewers: Vec<mpsc::Sender<Vec<u8>>>,
    /// Raw framed bytes of the most recent `Message::FullFrame` the host
    /// sent, so a viewer who joins mid-session doesn't sit on a black
    /// screen waiting for the next periodic keyframe.
    last_keyframe: Option<Vec<u8>>,
}

type Sessions = Arc<Mutex<HashMap<String, Session>>>;

/// Run a relay server, forwarding framed bytes between a host and its
/// viewers for each registered session. Blocks until the listener errors.
pub async fn run_relay_server(bind_addr: std::net::SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("Failed to bind relay on {bind_addr}"))?;
    info!("Relay server listening on {bind_addr}");

    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, peer) = listener.accept().await?;
        let sessions = sessions.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_relay_client(stream, sessions).await {
                warn!("Relay client {peer} disconnected: {e}");
            }
        });
    }
}

async fn handle_relay_client(mut stream: TcpStream, sessions: Sessions) -> Result<()> {
    let reg = read_registration(&mut stream).await?;
    info!("Relay: {:?} joined session '{}'", reg.role, reg.session);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(64);

    {
        let mut sessions = sessions.lock().await;
        let session = sessions.entry(reg.session.clone()).or_insert_with(|| Session {
            host: None,
            viewers: Vec::new(),
            last_keyframe: None,
        });
        match reg.role {
            RelayRole::Host => session.host = Some(tx.clone()),
            RelayRole::Viewer => {
                session.viewers.push(tx.clone());
                if let Some(keyframe) = session.last_keyframe.clone() {
                    let _ = tx.send(keyframe).await;
                }
            }
        }
    }

    let (mut read_half, mut write_half) = stream.into_split();
    let session_id = reg.session.clone();
    let role = reg.role;
    let sessions_for_forward = sessions.clone();

    // Task: bytes arriving from this peer get forwarded to its counterpart(s).
    let forward_task = tokio::spawn(async move {
        loop {
            let mut len_buf = [0u8; 4];
            if read_half.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut payload = vec![0u8; len];
            if read_half.read_exact(&mut payload).await.is_err() {
                break;
            }

            let mut framed = Vec::with_capacity(4 + len);
            framed.extend_from_slice(&len_buf);
            framed.extend_from_slice(&payload);

            let mut sessions = sessions_for_forward.lock().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                match role {
                    // Host data fans out to every connected viewer.
                    RelayRole::Host => {
                        if matches!(Message::deserialize(&payload), Ok(Message::FullFrame { .. })) {
                            session.last_keyframe = Some(framed.clone());
                        }
                        for viewer in &session.viewers {
                            let _ = viewer.send(framed.clone()).await;
                        }
                    }
                    // Viewer data (rare: control/ack messages) goes to the host.
                    RelayRole::Viewer => {
                        if let Some(host) = &session.host {
                            let _ = host.send(framed.clone()).await;
                        }
                    }
                }
            }
        }
    });

    // Task: bytes destined for this peer get written out to its socket.
    while let Some(bytes) = rx.recv().await {
        if write_half.write_all(&bytes).await.is_err() {
            break;
        }
    }

    forward_task.abort();

    // Clean up registration on disconnect.
    let mut sessions = sessions.lock().await;
    if let Some(session) = sessions.get_mut(&reg.session) {
        match reg.role {
            RelayRole::Host => session.host = None,
            RelayRole::Viewer => session.viewers.retain(|v| !v.same_channel(&tx)),
        }
        if session.host.is_none() && session.viewers.is_empty() {
            sessions.remove(&reg.session);
        }
    }

    Ok(())
}

/// A TCP connection to a relay server, already registered for a session,
/// implementing the same `MessageTransport` as a direct QUIC connection.
pub struct RelayTransport {
    stream: TcpStream,
}

impl RelayTransport {
    pub async fn connect(
        relay_addr: std::net::SocketAddr,
        session: String,
        role: RelayRole,
    ) -> Result<Self> {
        let mut stream = TcpStream::connect(relay_addr)
            .await
            .with_context(|| format!("Failed to connect to relay at {relay_addr}"))?;
        write_registration(&mut stream, &RelayRegister { session, role }).await?;
        Ok(Self { stream })
    }
}

#[async_trait]
impl MessageTransport for RelayTransport {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        msg.write_framed(&mut self.stream).await
    }

    async fn recv(&mut self) -> Result<Message> {
        Message::read_framed(&mut self.stream).await
    }
}
