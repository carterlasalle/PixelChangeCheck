//! The sharer: capture, diff, plan, encode once, fan out.
//!
//! Four properties this module exists to guarantee:
//!
//! * **the reference is what viewers hold.** Every diff is taken against
//!   the framebuffer an up-to-date viewer has installed, and it advances
//!   only over rectangles that actually shipped. That is what makes a
//!   sub-threshold change accumulate instead of being discarded forever.
//! * **one sequencer.** A viewer subscribes to the broadcast *before* it
//!   takes its snapshot, then ignores anything at or below that revision.
//!   It can never end up applying patches that predate its own base.
//! * **loss is recovery, not silence.** A viewer that falls behind gets a
//!   fresh snapshot; one that keeps falling behind is disconnected.
//! * **a viewer never receives a torn snapshot.** Snapshots are chunked
//!   with an explicit begin/commit, so an interrupted transfer cannot
//!   half-replace a working surface.

use crate::capture::CaptureSource;
use crate::encoder::SurfaceSnapshot;
use crate::network::{
    verify_token, Epoch, Message, MessageSink, MessageTransport, NetworkConfig, Rev, SessionToken,
    SNAPSHOT_CHUNK_BYTES,
};
use crate::pcc::types::{rgb_len, Frame};
use crate::pcc::{PlanLimits, Planner};
use crate::reach::Rung;
use crate::relay::{generate_session_code, RelayRole, RelayTransport};
use crate::server::renderer::{web, SharedSurface};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

/// Backstop convergence deadline.
///
/// Receipt: with correct sequencing a viewer only goes stale on a real
/// failure, and that failure triggers an immediate repair. This bounds the
/// *undetected* case. It only fires on a screen that is actually changing
/// (an idle screen has nothing to repair), so a busy screen pays one
/// snapshot per 30s on top of the updates it is already sending.
const REPAIR_INTERVAL: Duration = Duration::from_secs(30);

/// Broadcast depth per viewer: deep enough to absorb a scheduling hiccup,
/// shallow enough that a viewer which has genuinely stalled gets a
/// snapshot rather than unbounded latency.
const BROADCAST_DEPTH: usize = 256;

/// Largest encoded message that may go on a stream.
const SEND_BUDGET: usize = crate::network::MAX_MESSAGE_SIZE as usize;

/// Failed handshakes allowed from one address before it is refused.
const AUTH_FAILURES_ALLOWED: u32 = 8;
const AUTH_FAILURE_WINDOW: Duration = Duration::from_secs(60);

/// How long a viewer has to present its token before the connection is
/// dropped.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// A lagging viewer is repaired; one that lags again immediately after
/// cannot be keeping up at all, and resending full frames only makes it
/// worse.
const MAX_CONSECUTIVE_LAGS: u32 = 3;

#[derive(Clone)]
pub struct ShareArgs {
    /// `host:port` to listen on for direct QUIC viewer connections.
    pub listen: Option<String>,
    /// `host:port` of a relay, for viewers that cannot connect directly.
    pub relay: Option<String>,
    /// SHA-256 fingerprint of the relay's certificate, so the sharer can
    /// verify it is reaching the relay operator and not a hijacker.
    pub relay_pin: String,
    /// Relay session code. Auto-generated if not given.
    pub session: Option<String>,
    /// `host:port` to bind the browser viewer on.
    pub web: Option<String>,
    /// Optional PEM certificate and key for HTTPS/WSS on the web port.
    pub web_cert: Option<(String, String)>,
    /// Force the synthetic test pattern instead of capturing a real screen.
    pub synthetic: bool,
    pub fps: u32,
    pub max_fps: u32,
    pub quality: f32,
    pub token: SessionToken,
    pub repair_interval: Duration,
    /// How hard to try for a direct path before the relay.
    pub reach: crate::reach::ReachPolicy,
    /// Capture and send audio alongside the screen.
    pub audio: bool,
}

/// The authoritative surface, exactly as an up-to-date viewer sees it.
#[derive(Clone)]
struct Published {
    rev: Rev,
    epoch: Epoch,
    snapshot: Arc<SurfaceSnapshot>,
    /// Encoded snapshot, tagged with the revision it represents, produced
    /// on demand and then shared by every joiner and every repair.
    encoded: Option<(Rev, Arc<Vec<u8>>)>,
}

impl Published {
    fn encoded_for(&self, rev: Rev) -> Option<&Arc<Vec<u8>>> {
        self.encoded
            .as_ref()
            .filter(|(r, _)| *r == rev)
            .map(|(_, d)| d)
    }
}

type Shared = Arc<RwLock<Published>>;
type EncodedBroadcast = broadcast::Sender<Arc<Vec<u8>>>;

/// Per-viewer feedback, used for adaptation and for saying who is behind.
#[derive(Debug, Default)]
struct ViewerStats {
    lag_events: u32,
}

/// Adaptive pressure from real network feedback, 0.0 (healthy) upward.
#[derive(Debug)]
struct Pressure {
    value: f32,
    last_change: Instant,
}

impl Pressure {
    fn penalise(&mut self, weight: f32) {
        self.value = (self.value + weight).min(4.0);
    }

    /// Decays back toward healthy so the rate is restored once the link is.
    fn decay(&mut self, dt: Duration) {
        self.value = (self.value - dt.as_secs_f32() * 0.25).max(0.0);
    }

    fn is_pressured(&self) -> bool {
        self.value >= 1.0
    }
}

pub async fn run_share(args: ShareArgs, metrics: crate::telemetry::SharedMetrics) -> Result<()> {
    let capture = CaptureSource::open(args.synthetic)?;
    let (width, height) = (capture.width(), capture.height());
    let token = args.token.clone();
    info!("Sharing {width}x{height}");

    let identity = Arc::new(
        crate::network::generate_identity().context("Failed to create the sharer identity")?,
    );
    info!("Certificate fingerprint (sha256): {}", identity.fingerprint);

    let target_quality = args.quality.clamp(0.1, 1.0);
    let requested_fps = args.fps.max(1).min(args.max_fps.max(1));

    let surface = Arc::new(SharedSurface::new(width, height, target_quality));
    let (tx, keepalive) = broadcast::channel::<Arc<Vec<u8>>>(BROADCAST_DEPTH);
    // The web viewer is another consumer of the very same stream, so it
    // gets a receiver of its own rather than a second encode path.
    let web_updates = tx.subscribe();

    // The web server starts once the published surface exists, so a
    // joining browser can be caught up from it.

    // Capture the first frame *before* any listener exists. A viewer that
    // connected during that window would otherwise receive an empty
    // surface and, because the reference would already equal the first
    // captured frame, never be told what it is missing.
    let first = capture.capture_frame()?;
    let first_frame = Frame::new(0, first.width, first.height, first.data)?;
    let published: Shared = Arc::new(RwLock::new(Published {
        rev: 0,
        epoch: 0,
        snapshot: Arc::new(SurfaceSnapshot::new(
            first_frame.width,
            first_frame.height,
            first_frame.data.clone(),
        )?),
        encoded: None,
    }));
    surface.publish(&first_frame, target_quality);

    // The ladder is decided once at startup rather than per session:
    // a STUN round trip on every viewer join is latency nobody asked for,
    // and the answer does not change while the process runs.
    //
    // It also must not block startup. A STUN probe on a network that
    // blackholes UDP takes its full timeout, and holding the listeners
    // closed for that long would make the sharer look dead. So it runs on
    // its own task and reports when it has an answer.
    if args.reach != crate::reach::ReachPolicy::RelayOnly {
        let reach = args.reach;
        let relay_configured = args.relay.is_some();
        tokio::spawn(async move {
            let Some(rung) = resolve_ladder(reach).await else {
                return;
            };
            match (rung.is_direct(), relay_configured) {
                (true, _) => info!(
                    "Direct path available via {}; the relay stays a fallback",
                    rung.label()
                ),
                (false, true) => info!("No direct path found; using the relay"),
                (false, false) => warn!(
                    "No direct path found and no --relay was given, so viewers must be on \
                     this network or reach the port directly"
                ),
            }
        });
    }
    if args.reach == crate::reach::ReachPolicy::DirectOnly && args.relay.is_some() {
        anyhow::bail!("--reach direct was given but --relay was also set; pick one");
    }

    if let Some(web_addr) = &args.web {
        // Only the sharer owns the authoritative surface, so only it can
        // produce the snapshot a joining browser needs.
        let source = published.clone();
        let snapshot: web::SnapshotFn = Arc::new(move || {
            let source = source.clone();
            // The published state is already current; this runs on the web
            // server's own thread, so it reads what the capture loop last
            // published rather than reaching into its task.
            match try_current_snapshot(&source) {
                Ok(msgs) => msgs,
                Err(e) => {
                    warn!("Could not build a web snapshot: {e}");
                    Vec::new()
                }
            }
        });
        start_web(
            web_addr,
            args.web_cert,
            &surface,
            web_updates,
            &token,
            &identity,
            snapshot,
        )
        .await?;
    }

    let viewers: Arc<Mutex<HashMap<u64, ViewerStats>>> = Arc::new(Mutex::new(HashMap::new()));
    let next_viewer_id = Arc::new(AtomicU64::new(1));

    if let Some(listen_addr) = &args.listen {
        let addr = crate::network::resolve(listen_addr)
            .await
            .with_context(|| format!("Invalid --listen address '{listen_addr}'"))?;
        let endpoint = crate::network::server_endpoint(&NetworkConfig::default(), &identity, addr)?;
        info!("Direct viewers on {addr}");
        info!(
            "  pcc view --connect {addr} --token {} --pin {}",
            token.as_str(),
            identity.fingerprint
        );
        spawn_accept_loop(
            endpoint,
            tx.clone(),
            published.clone(),
            token.clone(),
            viewers.clone(),
            next_viewer_id.clone(),
        );
    }

    if let Some(relay_addr) = &args.relay {
        let addr = crate::network::resolve(relay_addr)
            .await
            .with_context(|| format!("Invalid --relay address '{relay_addr}'"))?;
        let pin = crate::network::hex_to_der(&args.relay_pin)?;
        let session = args.session.clone().unwrap_or_else(generate_session_code);
        info!(
            "Relay {relay_addr}, session '{session}', token {}",
            token.as_str()
        );
        spawn_relay_loop(
            addr,
            pin,
            session,
            token.clone(),
            tx.clone(),
            published.clone(),
            viewers.clone(),
            next_viewer_id.clone(),
        );
    }

    drop(keepalive);

    capture_loop(
        metrics,
        first_frame,
        capture,
        Planner::default(),
        published,
        tx,
        viewers,
        surface,
        target_quality,
        requested_fps,
        args.max_fps.max(1),
        if args.repair_interval.is_zero() {
            REPAIR_INTERVAL
        } else {
            args.repair_interval
        },
    )
    .await
}

async fn start_web(
    web_addr: &str,
    cert: Option<(String, String)>,
    surface: &Arc<SharedSurface>,
    updates: broadcast::Receiver<Arc<Vec<u8>>>,
    token: &SessionToken,
    identity: &crate::network::ServerIdentity,
    snapshot: web::SnapshotFn,
) -> Result<()> {
    let addr = crate::network::resolve(web_addr)
        .await
        .with_context(|| format!("Invalid --web address '{web_addr}'"))?;
    let tls = match cert {
        Some((cert_path, key_path)) => {
            let certs = std::fs::read(&cert_path)
                .with_context(|| format!("Failed to read the certificate at {cert_path}"))?;
            let key = std::fs::read(&key_path)
                .with_context(|| format!("Failed to read the private key at {key_path}"))?;
            let parsed = rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(vec![rustls::Certificate(certs)], rustls::PrivateKey(key))
                .map_err(|e| anyhow::anyhow!("Invalid web certificate/key pair: {e}"))?;
            Some(Arc::new(parsed))
        }
        None => None,
    };

    let surface = surface.clone();
    let token = token.clone();
    let fingerprint = identity.fingerprint.clone();
    let snapshot_for_thread = snapshot.clone();
    let scheme = if tls.is_some() { "https" } else { "http" };
    // The web server owns its own current-thread runtime and blocks, so
    // bind here (to report the address and to fail fast) and hand the
    // bound listener over.
    let listener = std::net::TcpListener::bind(addr)
        .with_context(|| format!("Failed to bind the web viewer on {addr}"))?;
    listener
        .set_nonblocking(true)
        .context("Failed to put the web listener into non-blocking mode")?;
    info!(
        "Browser viewer: {scheme}://{addr}/?token={}",
        token.as_str()
    );
    std::thread::spawn(move || {
        if let Err(e) = web::run_web_server(
            listener,
            surface,
            updates,
            token,
            fingerprint,
            tls,
            snapshot_for_thread,
        ) {
            error!("Web viewer stopped: {e}");
        }
    });
    Ok(())
}

// ------------------------------------------------------------- accepting

fn spawn_accept_loop(
    endpoint: quinn::Endpoint,
    tx: EncodedBroadcast,
    published: Shared,
    token: SessionToken,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_id: Arc<AtomicU64>,
) {
    tokio::spawn(async move {
        let failures: Arc<Mutex<HashMap<SocketAddr, (u32, Instant)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        while let Some(connecting) = endpoint.accept().await {
            let peer = connecting.remote_address();
            if rate_limited(&failures, peer) {
                warn!("Refusing {peer}: too many failed handshakes");
                continue;
            }
            match connecting.await {
                Ok(connection) => {
                    let (tx, published, token, viewers, next_id) = (
                        tx.clone(),
                        published.clone(),
                        token.clone(),
                        viewers.clone(),
                        next_id.clone(),
                    );
                    tokio::spawn(async move {
                        match connection.accept_bi().await {
                            Ok((send, recv)) => {
                                let transport =
                                    crate::network::QuicTransport::new(send, recv, connection);
                                serve_viewer(
                                    Box::new(transport),
                                    peer,
                                    "QUIC",
                                    tx,
                                    published,
                                    token,
                                    viewers,
                                    next_id,
                                )
                                .await;
                            }
                            Err(e) => warn!("Viewer {peer} never opened a stream: {e}"),
                        }
                    });
                }
                Err(e) => warn!("Failed to accept connection from {peer}: {e}"),
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn spawn_relay_loop(
    addr: SocketAddr,
    pin: Vec<u8>,
    session: String,
    token: SessionToken,
    tx: EncodedBroadcast,
    published: Shared,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_id: Arc<AtomicU64>,
) {
    tokio::spawn(async move {
        // Reconnect forever: a sharer is long-lived, and a relay restart is
        // a normal event rather than a reason to stop sharing.
        let resilience = crate::network::NetworkResilience::new(crate::network::ResilienceConfig {
            max_retries: 0,
            retry_delay: Duration::from_secs(3),
        });
        loop {
            match RelayTransport::connect(
                addr,
                "pcc-relay",
                &pin,
                session.clone(),
                token.clone(),
                RelayRole::Host,
            )
            .await
            {
                Ok(transport) => {
                    info!("Relay connected (session '{session}')");
                    serve_viewer(
                        Box::new(transport),
                        addr,
                        "relay",
                        tx.clone(),
                        published.clone(),
                        token.clone(),
                        viewers.clone(),
                        next_id.clone(),
                    )
                    .await;
                    warn!("Relay connection lost; reconnecting");
                }
                Err(e) => warn!("Failed to reach the relay: {e}"),
            }
            resilience.note_failure().await;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

// ------------------------------------------------------------ one viewer

#[allow(clippy::too_many_arguments)]
async fn serve_viewer(
    transport: Box<dyn MessageTransport>,
    peer: SocketAddr,
    path: &str,
    tx: EncodedBroadcast,
    published: Shared,
    token: SessionToken,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_viewer_id: Arc<AtomicU64>,
) {
    // Subscribe *before* anything else, so no update can slip between the
    // snapshot we are about to take and the stream we are about to start.
    let mut rx = tx.subscribe();
    let (mut sink, mut source) = transport.split();
    let label = format!("{path} viewer {peer}");

    let handshake = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
        match source.recv().await {
            Ok(Message::Hello { token }) => Some(token),
            Ok(other) => {
                let _ = sink
                    .send(&Message::Error(format!(
                        "Send Hello with your viewer token first (got a message with rev {:?})",
                        other.rev()
                    )))
                    .await;
                None
            }
            Err(e) => {
                warn!("{label} handshake read failed: {e}");
                None
            }
        }
    })
    .await;

    let presented = match handshake {
        Ok(Some(presented)) if verify_token(&token, &presented) => presented,
        _ => {
            let _ = sink
                .send(&Message::Error(
                    "Unauthorized. Check the --token this sharer printed.".into(),
                ))
                .await;
            // Give the viewer a moment to read the reason; dropping the
            // connection immediately would replace an explanation with
            // "connection lost".
            let _ = tokio::time::timeout(Duration::from_secs(2), source.recv()).await;
            warn!("{label} rejected: bad or missing viewer token");
            return;
        }
    };
    let _ = presented;
    let id = next_viewer_id.fetch_add(1, Ordering::Relaxed);
    viewers.lock().insert(id, ViewerStats::default());
    info!("{label} authorized");

    // End-to-end encryption. The token has already been checked, so the
    // offer/reply exchange only has to prove both sides derived the same
    // keys; after it every frame is sealed and the relay sees nothing but
    // sizes and timing.
    let host_keys = crate::network::e2e::KeyPair::generate();
    let mut sink: Box<dyn crate::network::MessageSink> = sink;
    let mut source: Box<dyn crate::network::MessageSource> = source;
    let session = match crate::network::e2e::host_handshake(
        &mut sink,
        &mut source,
        host_keys,
        token.as_str(),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!("{label} encryption handshake failed: {e}");
            return;
        }
    };
    let mut sink: Box<dyn crate::network::MessageSink> = Box::new(crate::network::SealedSink::new(
        sink,
        session.host_to_viewer,
    ));
    let mut source: Box<dyn crate::network::MessageSource> = Box::new(
        crate::network::SealedSource::new(source, session.viewer_to_host),
    );

    let mut floor = match send_snapshot(&mut sink, &published).await {
        Ok(rev) => rev,
        Err(e) => {
            warn!("{label} could not be caught up: {e}");
            return;
        }
    };

    // Viewer control messages run on their own task so a refresh request is
    // not stuck behind a slow write.
    let (control_tx, mut control_rx) = tokio::sync::mpsc::channel::<Message>(8);
    tokio::spawn(async move {
        while let Ok(msg) = source.recv().await {
            if control_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut consecutive_lags = 0u32;
    loop {
        tokio::select! {
            update = rx.recv() => match update {
                Ok(bytes) => {
                    let Some(rev) = peek_rev(&bytes) else { continue };
                    if rev <= floor {
                        continue; // already contained in the snapshot we sent
                    }
                    if let Err(e) = sink.send_encoded(&bytes).await {
                        if let Some(v) = viewers.lock().get_mut(&id) {
                            v.lag_events += 1;
                        }
                        warn!("{label} write failed: {e}");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    consecutive_lags += 1;
                    if let Some(v) = viewers.lock().get_mut(&id) {
                            v.lag_events += 1;
                        }
                    if consecutive_lags > MAX_CONSECUTIVE_LAGS {
                        warn!("{label} fell behind {consecutive_lags} times; disconnecting");
                        break;
                    }
                    warn!("{label} fell behind by {missed} updates; repairing");
                    match send_snapshot(&mut sink, &published).await {
                        Ok(rev) => {
                            floor = rev;
                            consecutive_lags = 0;
                        }
                        Err(e) => {
                            warn!("{label} repair failed: {e}");
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            control = control_rx.recv() => {
                let Some(msg) = control else { break };
                match msg {
                    Message::RequestKeyframe => {
                        match send_snapshot(&mut sink, &published).await {
                            Ok(rev) => floor = rev,
                            Err(e) => { warn!("{label} refresh failed: {e}"); break; }
                        }
                    }
                    Message::Ack { .. } => {}
                    Message::Bye => break,
                    Message::Hello { .. } => {
                        let _ = sink.send(&Message::Error("Already authorized.".into())).await;
                    }
                    Message::Error(e) => warn!("{label} reported: {e}"),
                    _ => {}
                }
            }
        }
    }
    viewers.lock().remove(&id);
    info!("{label} disconnected");
}

/// Take (or reuse) the current snapshot and send it whole. Returns the
/// revision it represents, which becomes the viewer's floor.
async fn send_snapshot(sink: &mut Box<dyn MessageSink>, published: &Shared) -> Result<Rev> {
    let (rev, epoch, width, height, data) = {
        let mut p = published.write().await;
        let rev = p.rev;
        if p.encoded_for(rev).is_none() {
            p.encoded = Some((
                rev,
                Arc::new(
                    crate::encoder::encode_snapshot(
                        p.snapshot.width,
                        p.snapshot.height,
                        &p.snapshot.rgb,
                    )?
                    .1,
                ),
            ));
        }
        let encoded = p.encoded_for(rev).expect("just produced");
        (
            rev,
            p.epoch,
            p.snapshot.width,
            p.snapshot.height,
            encoded.clone(),
        )
    };
    for msg in snapshot_messages(width, height, &data, rev, epoch)? {
        sink.send_encoded(&msg).await?;
    }
    Ok(rev)
}

/// Split an encoded snapshot into the messages that carry it: one begin,
/// `ceil(len / chunk)` chunks, and one commit. Nothing is ever half-sent
/// in a way a viewer could mistake for a complete surface.
fn snapshot_messages(
    width: u32,
    height: u32,
    data: &[u8],
    rev: Rev,
    epoch: Epoch,
) -> Result<Vec<Vec<u8>>> {
    let chunks: Vec<&[u8]> = data.chunks(SNAPSHOT_CHUNK_BYTES).collect();
    if chunks.is_empty() || chunks.len() as u32 > crate::network::MAX_SNAPSHOT_CHUNKS {
        anyhow::bail!(
            "snapshot of {} bytes needs {} chunks (max {})",
            data.len(),
            chunks.len().max(1),
            crate::network::MAX_SNAPSHOT_CHUNKS
        );
    }
    let mut out = Vec::with_capacity(chunks.len() + 2);
    out.push(
        Message::SnapshotBegin {
            rev,
            epoch,
            width,
            height,
            format: crate::encoder::SnapshotFormat::Png,
            total_len: data.len() as u32,
            chunks: chunks.len() as u32,
        }
        .encode()?,
    );
    for (index, chunk) in chunks.iter().enumerate() {
        out.push(
            Message::SnapshotChunk {
                rev,
                index: index as u32,
                data: chunk.to_vec(),
            }
            .encode()?,
        );
    }
    out.push(Message::SnapshotCommit { rev, epoch }.encode()?);
    Ok(out)
}

/// Read a message's revision without decoding its payload.
pub(crate) fn peek_rev(bytes: &[u8]) -> Option<Rev> {
    if bytes.len() < 5 || bytes[0] != crate::network::PROTOCOL_VERSION {
        return None;
    }
    let body = &bytes[5..];
    if body.is_empty() {
        return None;
    }
    // SnapshotBegin, SnapshotChunk, PartialUpdate, KeepAlive and
    // SnapshotCommit all lead with their revision.
    match body[0] {
        0x04 | 0x05 | 0x06 | 0x07 | 0x0B if body.len() >= 9 => {
            Some(u64::from_le_bytes(body[1..9].try_into().ok()?))
        }
        _ => None,
    }
}

// ---------------------------------------------------------- capture loop

#[allow(clippy::too_many_arguments)]
async fn capture_loop(
    metrics: crate::telemetry::SharedMetrics,
    mut first: Frame,
    capture: CaptureSource,
    planner: Planner,
    published: Shared,
    tx: EncodedBroadcast,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    surface: Arc<SharedSurface>,
    target_quality: f32,
    requested_fps: u32,
    max_fps: u32,
    repair_interval: Duration,
) -> Result<()> {
    // The authoritative surface, shared. `Arc::make_mut` hands the loop a
    // mutable view for free and copies only when a joining viewer is
    // actually holding the same allocation.
    let mut reference: Arc<Vec<u8>> = Arc::new(std::mem::take(&mut first.data));
    let mut surface_size = (first.width, first.height);
    let mut epoch: Epoch = 0;
    let mut rev: Rev = 0;
    // A snapshot is owed before any patch makes sense: at startup, and
    // again whenever the geometry change invalidates every rectangle sent
    // so far.
    let mut pending_snapshot = true;
    let mut effective_fps = requested_fps;
    let mut last_repair = Instant::now();
    let mut quality = target_quality;
    let mut pressure = Pressure {
        value: 0.0,
        last_change: Instant::now(),
    };

    loop {
        let loop_start = Instant::now();
        let frame = capture.capture_frame()?;

        // A resize, rotation or display switch invalidates every rectangle
        // we have sent. Start a new epoch rather than ending the share or
        // diffing mismatched geometry.
        if surface_size != (frame.width, frame.height) {
            epoch += 1;
            metrics.epoch_bumps.incr();
            warn!(
                "Capture geometry changed to {}x{}; starting epoch {epoch}",
                frame.width, frame.height
            );
            reference = Arc::new(vec![0u8; rgb_len(frame.width, frame.height)?]);
            surface_size = (frame.width, frame.height);
            pending_snapshot = true;
        }
        if pending_snapshot && surface_size == (0, 0) {
            reference = Arc::new(frame.data.clone());
        }

        // A patch set is only worth sending if it beats the snapshot we
        // most recently produced.
        let snapshot_bytes = published
            .read()
            .await
            .encoded
            .as_ref()
            .map(|(_, d)| d.len())
            .unwrap_or(usize::MAX);
        let (width, height) = surface_size;
        // A mutable view of the shared surface; this copies only if a
        // joining viewer is holding the same allocation right now.
        let rgb: &mut Vec<u8> = Arc::make_mut(&mut reference);

        let detect_start = Instant::now();
        let plan = planner.plan(
            rgb,
            width,
            height,
            &frame,
            PlanLimits {
                snapshot_bytes,
                max_update_bytes: SEND_BUDGET,
            },
        )?;
        metrics.detect.record_duration(detect_start.elapsed());
        metrics.plan.record_duration(detect_start.elapsed());
        metrics.frames_total.incr();
        let total_pixels = (width as f64) * (height as f64);
        metrics.changed_area_fraction.set(if total_pixels > 0.0 {
            (plan.changed_pixels as f64 / total_pixels).min(1.0)
        } else {
            0.0
        });

        let idle = plan.ops.is_empty() && plan.wire_len == 0;
        let wants_snapshot = !idle && plan.ops.is_empty();
        let repair_due = last_repair.elapsed() >= repair_interval;

        if pending_snapshot || wants_snapshot || (!idle && repair_due && snapshot_bytes > 0) {
            // A snapshot conveys the whole current frame, so the
            // reference becomes that frame. Without this the next diff
            // would still be measured against a surface no viewer holds,
            // and its patches would be applied to the wrong base.
            if wants_snapshot {
                reference = Arc::new(frame.data.clone());
            }
            rev += 1;
            last_repair = Instant::now();
            pending_snapshot = false;
            metrics.frames_snapshot_fallback.incr();
            let encode_start = Instant::now();
            for msg in publish_snapshot(&published, &reference, width, height, rev, epoch).await? {
                metrics.observe_message_bytes(msg.len());
                metrics.bytes_snapshot.add(msg.len() as u64);
                let _ = tx.send(Arc::new(msg));
            }
            metrics.encode.record_duration(encode_start.elapsed());
        } else if !idle {
            rev += 1;
            for op in &plan.ops {
                let n = op.wire_len() as u64;
                match op {
                    crate::network::WireOp::Fill { .. } => metrics.bytes_fill.add(n),
                    crate::network::WireOp::Copy { .. } => metrics.bytes_copy.add(n),
                    crate::network::WireOp::Rect { .. } => metrics.bytes_patch.add(n),
                }
            }
            let serialize_start = Instant::now();
            let bytes = Message::PartialUpdate {
                rev,
                epoch,
                ops: plan.ops,
            }
            .encode();
            match bytes {
                Ok(bytes) => {
                    metrics.observe_message_bytes(bytes.len());
                    metrics.serialize.record_duration(serialize_start.elapsed());
                    let _ = tx.send(Arc::new(bytes));
                }
                Err(e) => {
                    // Oversized: replace with a snapshot rather than
                    // dropping updates and letting viewers go stale.
                    warn!("Update rejected ({e}); sending a snapshot instead");
                    reference = Arc::new(frame.data.clone());
                    rev += 1;
                    last_repair = Instant::now();
                    for msg in
                        publish_snapshot(&published, &reference, width, height, rev, epoch).await?
                    {
                        let _ = tx.send(Arc::new(msg));
                    }
                }
            }
        } else {
            // Nothing changed. A keep-alive is nine bytes and it keeps NAT
            // mappings open; there is no reason to leave a viewer in the
            // dark about liveness.
            metrics.frames_idle.incr();
            if let Ok(bytes) = (Message::KeepAlive { rev }).encode() {
                metrics.bytes_keepalive.add(bytes.len() as u64);
                let _ = tx.send(Arc::new(bytes));
            }
        }

        {
            // Publish the surface itself, not a copy taken earlier. A
            // late joiner must see what is on screen now, which is the
            // whole point of handing over the shared buffer.
            let mut p = published.write().await;
            p.rev = rev;
            p.epoch = epoch;
            p.snapshot = Arc::new(SurfaceSnapshot {
                width,
                height,
                rgb: reference.clone(),
            });
        }
        surface.publish(&frame, quality);

        // Real feedback-driven adaptation: viewers that fall behind are
        // the signal, not local CPU overrun.
        pressure.decay(loop_start.elapsed());
        let lagging = viewers.lock().values().filter(|v| v.lag_events > 0).count();
        if lagging > 0 {
            pressure.penalise(lagging as f32);
        }
        if pressure.is_pressured() && effective_fps > 10 {
            effective_fps = (effective_fps * 3 / 4).max(10);
            info!("{lagging} viewer(s) falling behind; dropping to {effective_fps} fps");
            pressure.last_change = Instant::now();
        } else if !pressure.is_pressured()
            && effective_fps < requested_fps
            && pressure.last_change.elapsed() > Duration::from_secs(5)
        {
            effective_fps = (effective_fps * 4 / 3).min(requested_fps);
            pressure.last_change = Instant::now();
        }

        // The local loop overrunning its own budget is a real signal too,
        // but it only affects the *lossy* preview quality, because that is
        // the only thing quality still controls.
        let frame_interval = Duration::from_secs(1) / effective_fps.max(1);
        if loop_start.elapsed() > frame_interval * 2 && quality > 0.3 {
            quality = (quality - 0.1).max(0.3);
            let cfg = crate::pcc::QualityConfig {
                target_fps: effective_fps,
                max_fps,
                quality,
            };
            if let Ok(bytes) = Message::QualityConfig(cfg).encode() {
                let _ = tx.send(Arc::new(bytes));
            }
        }

        let elapsed = loop_start.elapsed();
        if elapsed > frame_interval {
            metrics.loop_overruns.incr();
        }
        metrics.captured_frames.incr();
        metrics
            .effective_fps
            .set(1.0 / frame_interval.as_secs_f64());
        if elapsed < frame_interval {
            tokio::time::sleep(frame_interval - elapsed).await;
        }
    }
}

/// Work out which rung of the ladder applies here.
async fn resolve_ladder(policy: crate::reach::ReachPolicy) -> Option<Rung> {
    if policy == crate::reach::ReachPolicy::RelayOnly {
        return None;
    }
    let report = crate::reach::diagnose(None).await;
    // A STUN probe that answers means NAT is not symmetric for UDP, which
    // is the single most useful thing to know before reaching for a relay.
    let rung = if report.has_global_ipv6 {
        Rung::Ipv6Direct
    } else if report.reflexive.is_some() {
        Rung::StunIce
    } else {
        Rung::Relay
    };
    for note in &report.notes {
        info!("reach: {note}");
    }
    Some(rung)
}

/// The message sequence that brings a receiver up to the published
/// surface, without blocking the caller.
fn try_current_snapshot(published: &Shared) -> Result<Vec<Vec<u8>>> {
    // `try_write` so this can run on the web server's thread: blocking that
    // thread would stall every stream it is serving.
    let mut state = published
        .try_write()
        .map_err(|_| anyhow::anyhow!("the capture loop holds the surface; try again"))?;
    let width = state.snapshot.width;
    let height = state.snapshot.height;
    let rev = state.rev;
    let epoch = state.epoch;
    let rgb = state.snapshot.rgb.clone();
    if state.encoded_for(rev).is_none() {
        state.encoded = Some((
            rev,
            Arc::new(crate::encoder::encode_snapshot(width, height, &rgb)?.1),
        ));
    }
    let data = state.encoded_for(rev).expect("just produced").clone();
    snapshot_messages(width, height, &data, rev, epoch)
}

/// Publish the shared surface and build the message sequence for it. The
/// encoded bytes are cached, so the next joiner does not pay for a second
/// encode of an unchanged screen.
async fn publish_snapshot(
    published: &Shared,
    reference: &Arc<Vec<u8>>,
    width: u32,
    height: u32,
    rev: Rev,
    epoch: Epoch,
) -> Result<Vec<Vec<u8>>> {
    let data = {
        let mut p = published.write().await;
        // Hand out the very same allocation the capture loop holds.
        p.snapshot = Arc::new(SurfaceSnapshot {
            width,
            height,
            rgb: reference.clone(),
        });
        if p.encoded_for(rev).is_none() {
            p.encoded = Some((
                rev,
                Arc::new(crate::encoder::encode_snapshot(width, height, reference)?.1),
            ));
        }
        p.encoded_for(rev).expect("just produced").clone()
    };
    snapshot_messages(width, height, &data, rev, epoch)
}

fn rate_limited(
    failures: &Arc<Mutex<HashMap<SocketAddr, (u32, Instant)>>>,
    peer: SocketAddr,
) -> bool {
    let mut map = failures.lock();
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
    use crate::network::PROTOCOL_VERSION;

    #[test]
    fn peek_rev_reads_a_partial_update_revision() {
        let bytes = Message::PartialUpdate {
            rev: 42,
            epoch: 1,
            ops: vec![],
        }
        .encode()
        .unwrap();
        assert_eq!(peek_rev(&bytes), Some(42));
    }

    #[test]
    fn peek_rev_reads_a_keepalive_revision() {
        let bytes = Message::KeepAlive { rev: 7 }.encode().unwrap();
        assert_eq!(peek_rev(&bytes), Some(7));
    }

    #[test]
    fn peek_rev_ignores_messages_without_a_revision() {
        let bytes = Message::Bye.encode().unwrap();
        assert_eq!(peek_rev(&bytes), None);
        assert_eq!(peek_rev(b""), None);
        assert_eq!(peek_rev(&[PROTOCOL_VERSION, 0, 0, 0, 0]), None);
    }
}
