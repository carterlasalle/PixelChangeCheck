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

use crate::audio::transport::EncodedFrame as EncodedAudio;
use crate::capture::CaptureSource;
use crate::encoder::SurfaceSnapshot;
use crate::network::{
    verify_token, Epoch, Message, MessageSink, MessageSource, MessageTransport, NetworkConfig, Rev,
    SessionToken, SNAPSHOT_CHUNK_BYTES,
};
use crate::pcc::types::{rgb_len, Frame};
use crate::pcc::{PlanLimits, Planner};
use crate::reach::Rung;
use crate::relay::{generate_session_code, RelayFan, RelayRole, RelayTransport, RELAY_CONTROL_ID};
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

/// Changed-area fraction at or above which the loop also emits a
/// `MotionPreview` interim hint. 40% is "a fling is happening", not "a
/// dialog opened": below it the exact update lands fast enough that an
/// interim box would only flicker.
const MOTION_PREVIEW_FRACTION: f64 = 0.40;

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
    /// Relay server address(es), comma-separated `host:port`. The sharer
    /// registers on every one; viewers probe the list in order so
    /// geography is "the first reachable relay", not a config file.
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
    /// What to capture. Synthetic mode ignores it.
    pub capture_target: crate::capture::CaptureTarget,
    pub fps: u32,
    pub max_fps: u32,
    pub quality: f32,
    pub token: SessionToken,
    pub repair_interval: Duration,
    /// How hard to try for a direct path before the relay.
    pub reach: crate::reach::ReachPolicy,
    /// Ask on stdin before admitting each viewer. Default admits on a
    /// valid token; with approval the token gets them to the door and a
    /// human opens it. Refusals get the same Error-and-wait treatment as
    /// a bad token, so a refused viewer sees a reason, not "lost".
    pub approve: bool,
    /// What audio to capture. `None_` (the default) sends nothing;
    /// anything else starts the capture pump.
    pub audio_source: crate::audio::AudioSource,
    /// Direct viewers at or above this count are asked to move to the
    /// first `--relay` (same session code) via `Redirect`. Per-viewer
    /// QUIC send costs scale linearly; the relay fan splits the session
    /// per viewer instead. 0 disables.
    pub broadcast_above: usize,
}

/// The authoritative surface, exactly as an up-to-date viewer sees it.
///
/// Public so integration tests can build the exact share-side state the
/// capture loop builds (ring pushes included) and drive the resume path.
#[derive(Clone)]
pub struct Published {
    pub rev: Rev,
    pub epoch: Epoch,
    pub snapshot: Arc<SurfaceSnapshot>,
    /// Encoded snapshot, tagged with the revision it represents, produced
    /// on demand and then shared by every joiner and every repair.
    pub encoded: Option<(Rev, Arc<Vec<u8>>)>,
    /// Bounded replay of recent broadcast payloads. The capture loop pushes
    /// every message it broadcasts; `serve_viewer` reads it when a viewer
    /// lags or asks for a refresh. Same bytes as the broadcast, second
    /// copy, bounded — never a second source of truth.
    pub ring: RevisionRing,
}

impl Published {
    fn encoded_for(&self, rev: Rev) -> Option<&Arc<Vec<u8>>> {
        self.encoded
            .as_ref()
            .filter(|(r, _)| *r == rev)
            .map(|(_, d)| d)
    }
}

pub type Shared = Arc<RwLock<Published>>;
type EncodedBroadcast = broadcast::Sender<Arc<Vec<u8>>>;

/// Bounded replay of recent revisions, so a viewer that only missed a few
/// updates is caught up by replay instead of a whole snapshot.
///
/// Placement: the capture loop owns the ring and pushes every encoded
/// message the broadcast carries (updates, snapshots-as-sequences, and
/// keep-alives — the revision chain, not just the interesting parts).
/// `serve_viewer` borrows it read-only when a viewer lags or asks for a
/// refresh: contiguous coverage from the viewer's floor means replay,
/// anything else means snapshot. The ring never changes what the
/// broadcast carries; it is a second copy of the same bytes, bounded in
/// both entries and bytes so a busy screen cannot grow it without limit.
/// An epoch change clears it: rectangles from an older geometry are
/// meaningless, and replaying them would corrupt rather than repair.
#[derive(Debug, Default, Clone)]
pub struct RevisionRing {
    entries: std::collections::VecDeque<RingEntry>,
    bytes: usize,
}

/// One ring entry: the encoded broadcast payload plus the revision and
/// epoch it belongs to. Snapshots span several messages at one revision;
/// every message of the sequence is stored, so replay reproduces the
/// exact byte stream the broadcast carried.
#[derive(Debug, Clone)]
struct RingEntry {
    rev: Rev,
    epoch: Epoch,
    bytes: Arc<Vec<u8>>,
}

/// Keep roughly half a minute of revisions: long enough to ride out a
/// transient stall, short enough that replay stays cheaper than a
/// snapshot. The byte cap is the binding limit on a busy screen.
const RING_MAX_ENTRIES: usize = 512;
/// Bytes of recent revisions retained. A 1080p busy screen emits on the
/// order of a few hundred KB/s of deltas; 16 MiB is minutes of that, so
/// the entry cap binds first and this caps pathological snapshots.
const RING_MAX_BYTES: usize = 16 * 1024 * 1024;

impl RevisionRing {
    /// Record one broadcast payload. Snapshots arrive as several messages
    /// at one revision; each is stored, so replay is byte-identical.
    pub fn push(&mut self, rev: Rev, epoch: Epoch, bytes: Arc<Vec<u8>>) {
        // An epoch change invalidates every rectangle sent so far.
        if let Some(back) = self.entries.back() {
            if back.epoch != epoch {
                self.entries.clear();
                self.bytes = 0;
            }
        }
        self.bytes += bytes.len();
        self.entries.push_back(RingEntry { rev, epoch, bytes });
        while self.entries.len() > RING_MAX_ENTRIES || self.bytes > RING_MAX_BYTES {
            if let Some(old) = self.entries.pop_front() {
                self.bytes = self.bytes.saturating_sub(old.bytes.len());
            } else {
                break;
            }
        }
    }

    /// Record a keep-alive, unless the ring already ends at this revision.
    /// Idle frames all carry the current rev and add no information, so
    /// storing each one would churn real history out of the ring within
    /// seconds on an idle screen.
    pub fn push_keepalive(&mut self, rev: Rev, epoch: Epoch, bytes: Arc<Vec<u8>>) {
        if let Some(back) = self.entries.back() {
            if back.rev == rev && back.epoch == epoch {
                return;
            }
        }
        self.push(rev, epoch, bytes);
    }

    /// Messages after `floor` at `epoch`, in broadcast order, when the
    /// ring covers them contiguously. `None` means replay is impossible:
    /// the floor predates the ring, the epoch moved on, or a revision in
    /// between is missing (a snapshot sequence that was truncated by the
    /// bounds, never a partial hole — pushes are in order).
    pub fn replay_from(&self, floor: Rev, epoch: Epoch) -> Option<Vec<Arc<Vec<u8>>>> {
        if self.entries.is_empty() {
            return None;
        }
        // The ring must still hold the floor itself to prove contiguity:
        // replay starts *after* it, but a gap between the floor and the
        // oldest entry means unknown history.
        let mut start = None;
        let mut prev_rev: Option<Rev> = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.epoch != epoch {
                return None;
            }
            if entry.rev <= floor {
                // A snapshot sequence shares one revision across several
                // messages; the floor is covered if any of them is at or
                // below it, and replay starts after the whole sequence.
                if entry.rev == floor {
                    start = Some(i + 1);
                } else {
                    start = None;
                }
                prev_rev = Some(entry.rev);
                continue;
            }
            // Past the floor: revisions must advance by exactly one per
            // sequence. Several messages may share a revision (one
            // snapshot), but a jump of two or more is a hole.
            match prev_rev {
                Some(prev) if entry.rev == prev || entry.rev == prev + 1 => {}
                _ => return None,
            }
            // The floor itself is gone: entries after it prove nothing
            // about what the viewer missed in between.
            start?;
            prev_rev = Some(entry.rev);
        }
        let start = start?;
        if start >= self.entries.len() {
            return Some(Vec::new());
        }
        Some(
            self.entries
                .range(start..)
                .map(|e| e.bytes.clone())
                .collect(),
        )
    }
}

#[cfg(test)]
mod revision_ring_tests {
    use super::*;

    fn msg(rev: Rev, _epoch: Epoch) -> Arc<Vec<u8>> {
        // The ring stores the epoch alongside the bytes (KeepAlive carries
        // none), so the bytes stay valid envelopes `peek_rev` can read.
        Arc::new(Message::KeepAlive { rev }.encode().unwrap())
    }

    #[test]
    fn replay_returns_everything_after_the_floor_in_order() {
        let mut ring = RevisionRing::default();
        for rev in 1..=5 {
            ring.push(rev, 0, msg(rev, 0));
        }
        let out = ring.replay_from(2, 0).expect("contiguous coverage replays");
        assert_eq!(out.len(), 3);
        assert_eq!(crate::network::peek_rev(&out[0]), Some(3));
        assert_eq!(crate::network::peek_rev(&out[2]), Some(5));
    }

    #[test]
    fn a_floor_that_predates_the_ring_falls_back_to_snapshot() {
        let mut ring = RevisionRing::default();
        for rev in 10..=12 {
            ring.push(rev, 0, msg(rev, 0));
        }
        assert!(
            ring.replay_from(3, 0).is_none(),
            "unknown history must not replay"
        );
    }

    #[test]
    fn a_hole_in_the_middle_falls_back_to_snapshot() {
        let mut ring = RevisionRing::default();
        for rev in [1u64, 2, 4, 5] {
            ring.push(rev, 0, msg(rev, 0));
        }
        assert!(
            ring.replay_from(1, 0).is_none(),
            "a missing revision is not replayable"
        );
    }

    #[test]
    fn an_epoch_change_clears_the_ring() {
        let mut ring = RevisionRing::default();
        for rev in 1..=3 {
            ring.push(rev, 0, msg(rev, 0));
        }
        ring.push(4, 1, msg(4, 1));
        assert!(
            ring.replay_from(2, 0).is_none(),
            "old-epoch rectangles would corrupt"
        );
        // The clear dropped everything before the epoch change. A viewer
        // still on the old epoch gets a snapshot (tested by the None
        // above); once it holds rev 4 there is nothing after it to replay.
        let out = ring.replay_from(4, 1).expect("caught-up floor replays");
        assert!(out.is_empty());
    }

    #[test]
    fn the_bounds_evict_oldest_first() {
        let mut ring = RevisionRing::default();
        for rev in 1..=(RING_MAX_ENTRIES as u64 + 10) {
            ring.push(rev, 0, msg(rev, 0));
        }
        assert_eq!(ring.entries.len(), RING_MAX_ENTRIES);
        assert!(
            ring.replay_from(1, 0).is_none(),
            "evicted history must not replay"
        );
        let floor = RING_MAX_ENTRIES as u64 + 9;
        assert!(
            ring.replay_from(floor, 0).is_some(),
            "retained tail must replay"
        );
    }

    #[test]
    fn idle_keepalives_do_not_churn_real_history() {
        let mut ring = RevisionRing::default();
        for rev in 1..=3 {
            ring.push(rev, 0, msg(rev, 0));
        }
        // An idle screen emits the same rev hundreds of times. Only the
        // first is informative; the rest must not evict real history.
        for _ in 0..(RING_MAX_ENTRIES + 10) {
            ring.push_keepalive(3, 0, msg(3, 0));
        }
        // revs 1, 2, 3 plus the one deduped keep-alive at rev 3.
        assert_eq!(ring.entries.len(), 3);
        let out = ring.replay_from(2, 0).expect("history must survive idling");
        assert_eq!(out.len(), 1);
        assert_eq!(crate::network::peek_rev(&out[0]), Some(3));
    }

    #[test]
    fn an_idle_floor_with_nothing_after_replays_empty() {
        let mut ring = RevisionRing::default();
        for rev in 1..=3 {
            ring.push(rev, 0, msg(rev, 0));
        }
        let out = ring.replay_from(3, 0).expect("caught up replays empty");
        assert!(out.is_empty());
    }
}

/// The capture loop and the datagram pump share one frame shape instead
/// of two parallel structs with a `From` between them.
type AudioBroadcast = broadcast::Sender<Arc<EncodedAudio>>;

/// How many audio frames may queue per viewer. 256 frames is five
/// seconds at 20 ms, which is far more than any healthy viewer needs and
/// still bounded: past it the viewer is not keeping up with audio, and a
/// gap is a click rather than a stall.
const AUDIO_QUEUE: usize = 256;

/// Per-viewer feedback, used for adaptation and for saying who is behind.
#[derive(Debug, Default, Clone, Copy)]
struct ViewerStats {
    lag_events: u32,
    /// Newest revision the viewer confirmed it applied. Starts at the
    /// snapshot floor and only moves forward; the pressure loop reads it
    /// to tell a merely-slow viewer from one that has stalled entirely.
    acked_rev: Rev,
    /// Latest measured round-trip time on this viewer's path, sampled
    /// from the QUIC stack. `None` on relay/fan sessions, which expose no
    /// connection handle and therefore no transport measurements.
    rtt_ms: Option<u64>,
    /// Cumulative lost packets on this viewer's path, same source.
    /// Monotonic: the pressure loop diffs it per frame.
    lost_packets: u64,
    /// Current congestion window in bytes, same source. A collapsing
    /// window means the path — not the viewer — is the bottleneck.
    cwnd_bytes: u64,
    /// True for viewers on the direct QUIC listener; false for relay/fan
    /// sessions. Broadcast mode only ever redirects direct viewers — a
    /// relayed viewer is already where it should be.
    direct: bool,
    /// Set when a `Redirect` was queued for this viewer. The serve task
    /// sends it once, then clears it; the flag stops the loop queuing a
    /// second one while the first is in flight.
    redirect_queued: bool,
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
    let capture = CaptureSource::open_target(args.synthetic, &args.capture_target)?;
    let (width, height) = (capture.width(), capture.height());
    let token = args.token.clone();
    info!("Sharing {width}x{height}");

    let identity = Arc::new(
        crate::network::generate_identity().context("Failed to create the sharer identity")?,
    );
    info!("Certificate fingerprint (sha256): {}", identity.fingerprint);

    let target_quality = args.quality.clamp(0.1, 1.0);
    let requested_fps = args.fps.max(1).min(args.max_fps.max(1));

    // The MJPEG preview lives only for browsers: without --web there is
    // nobody to read it, and the per-frame compare inside `publish` is a
    // 6 MB memcmp on every capture for nothing.
    let surface = args
        .web
        .as_ref()
        .map(|_| Arc::new(SharedSurface::new(width, height, target_quality)));
    let (tx, keepalive) = broadcast::channel::<Arc<Vec<u8>>>(BROADCAST_DEPTH);
    // Audio is encoded once here and shared, exactly as visual messages
    // are, so N viewers do not mean N Opus encodes.
    let (audio_tx, audio_rx) = tokio::sync::broadcast::channel::<Arc<EncodedAudio>>(AUDIO_QUEUE);
    if args.audio_source != crate::audio::AudioSource::None_ {
        start_audio_capture(audio_tx.clone(), args.audio_source);
    }

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
        ring: RevisionRing::default(),
    }));
    if let Some(surface) = &surface {
        surface.publish(&first_frame, target_quality);
    }

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

    if let (Some(web_addr), Some(surface)) = (&args.web, &surface) {
        // The web viewer is another consumer of the very same stream, so
        // it gets a receiver of its own rather than a second encode path.
        let web_updates = tx.subscribe();
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
            args.web_cert.clone(),
            surface,
            web_updates,
            &token,
            &identity,
            snapshot,
        )
        .await?;
    }

    let viewers: Arc<Mutex<HashMap<u64, ViewerStats>>> = Arc::new(Mutex::new(HashMap::new()));
    let next_viewer_id = Arc::new(AtomicU64::new(1));
    // Broadcast handoff destination, fixed for the life of the share:
    // the first relay plus the session code every relay loop registers
    // with. Computed once so the loop and the spawners cannot disagree.
    let redirect_to = redirect_target(&args);

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
            Some((audio_tx.clone(), audio_rx.resubscribe())),
            metrics.clone(),
            args.approve,
            redirect_to.clone(),
        );
    }

    // Geography: every named relay gets its own registration loop.
    // The first reachable relay wins per viewer — the sharer does not
    // need to know which one that is, because it serves whoever dials in
    // on any of them. A bad address fails its own loop, not the share.
    if let Some(relays) = &args.relay {
        let pin = crate::network::hex_to_der(&args.relay_pin)?;
        let session = args.session.clone().unwrap_or_else(generate_session_code);
        for relay_addr in split_relays(relays) {
            let addr = match crate::network::resolve(&relay_addr).await {
                Ok(a) => a,
                Err(e) => {
                    warn!("Skipping relay '{relay_addr}': {e:#}");
                    continue;
                }
            };
            info!(
                "Relay {relay_addr}, session '{session}', token {}",
                token.as_str()
            );
            spawn_relay_loop(
                addr,
                pin.clone(),
                session.clone(),
                token.clone(),
                tx.clone(),
                published.clone(),
                viewers.clone(),
                next_viewer_id.clone(),
                Some((audio_tx.clone(), audio_rx.resubscribe())),
                metrics.clone(),
                args.approve,
                None,
            );
        }
    }

    drop(keepalive);

    // No platform cursor API is linked: real shares have no sampler
    // and viewers hide the overlay; `--synthetic` sweeps a scripted
    // pointer so the plane is exercised end to end.
    let cursor_sampler: Box<dyn crate::capture::CursorSampler> = if args.synthetic {
        Box::new(crate::capture::SweepSampler::new())
    } else {
        Box::new(crate::capture::NoCursorSampler)
    };
    capture_loop(
        metrics,
        first_frame,
        capture,
        cursor_sampler,
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
        args.broadcast_above,
        redirect_to,
    )
    .await
}

/// Start capture and the encode pump.
///
/// The pump runs on its own thread because the capture callback must
/// never wait on the share loop, and the share loop must never wait on
/// audio. Each frame is encoded **once** here and broadcast, so N viewers
/// do not mean N Opus encodes.
fn start_audio_capture(audio_tx: AudioBroadcast, which: crate::audio::AudioSource) {
    let source = match crate::audio::capture::open_source(which) {
        Ok(s) => s,
        Err(e) => {
            warn!("No audio capture device: {e}");
            return;
        }
    };
    let name = source.device_name().to_string();
    let mut source = source;
    match source.start() {
        Ok(rx) => {
            let start = Instant::now();
            std::thread::spawn(move || {
                let Ok(mut encoder) = crate::audio::AudioSender::new() else {
                    return;
                };
                while let Ok(frame) = rx.recv() {
                    let Ok(opus) = encoder.encode_frame(&frame.pcm) else {
                        continue;
                    };
                    let _ = audio_tx.send(Arc::new(EncodedAudio {
                        pts_us: frame.capture_time.duration_since(start).as_micros() as u64,
                        pcm_len: frame.pcm.len() as u32,
                        opus: Arc::new(opus),
                    }));
                }
            });
            info!("Audio capture started from {name}");
        }
        Err(e) => warn!("Audio capture could not start: {e}"),
    }
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
    // Remote browser mode requires HTTPS. Without TLS the viewer
    // JavaScript itself travels in the clear, so anyone who can modify
    // network traffic can replace it and steal the session secret or the
    // screen after decryption — application-layer sealing cannot save a
    // compromised application. Loopback stays plaintext (dev convenience,
    // no network attacker); anything else refuses to serve insecurely.
    if cert.is_none() && !addr.ip().is_loopback() {
        anyhow::bail!(
            "refusing to serve the browser viewer on non-loopback {addr} without TLS: pass --web-cert/--web-key, or bind a loopback --web address"
        );
    }
    let tls = match cert {
        Some((cert_path, key_path)) => {
            let certs = std::fs::read(&cert_path)
                .with_context(|| format!("Failed to read the certificate at {cert_path}"))?;
            let key = std::fs::read(&key_path)
                .with_context(|| format!("Failed to read the private key at {key_path}"))?;
            let parsed = rustls::ServerConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_protocol_versions(rustls::ALL_VERSIONS)
            .expect("the ring provider supports these versions")
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::pki_types::CertificateDer::from(certs)],
                rustls::pki_types::PrivatePkcs8KeyDer::from(key).into(),
            )
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
    // The secret goes after the fragment, never in the query. A query
    // string is sent in the request line, kept in browser history, and
    // can leak through referrers, proxy logs and screenshots of the URL.
    // Everything after `#` is handled by the browser and never leaves it.
    info!(
        "Browser viewer: {scheme}://{addr}/#token={}",
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

#[allow(clippy::too_many_arguments)]
fn spawn_accept_loop(
    endpoint: quinn::Endpoint,
    tx: EncodedBroadcast,
    published: Shared,
    token: SessionToken,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_id: Arc<AtomicU64>,
    audio: Option<(AudioBroadcast, broadcast::Receiver<Arc<EncodedAudio>>)>,
    metrics: crate::telemetry::SharedMetrics,
    approve: bool,
    redirect_to: Option<(String, String)>,
) {
    tokio::spawn(async move {
        let redirect_to = redirect_to;
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
                    let (tx, published, token, viewers, next_id, audio, metrics, redirect_to) = (
                        tx.clone(),
                        published.clone(),
                        token.clone(),
                        viewers.clone(),
                        next_id.clone(),
                        audio.as_ref().map(|(t, r)| (t.clone(), r.resubscribe())),
                        metrics.clone(),
                        redirect_to.clone(),
                    );
                    tokio::spawn(async move {
                        match connection.accept_bi().await {
                            Ok((send, recv)) => {
                                let transport = crate::network::QuicTransport::new(
                                    send,
                                    recv,
                                    connection.clone(),
                                );
                                serve_viewer(
                                    Box::new(transport),
                                    peer,
                                    "QUIC",
                                    tx,
                                    published,
                                    token,
                                    viewers,
                                    next_id,
                                    audio,
                                    Some(connection),
                                    metrics,
                                    approve,
                                    redirect_to,
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
    audio: Option<(AudioBroadcast, broadcast::Receiver<Arc<EncodedAudio>>)>,
    metrics: crate::telemetry::SharedMetrics,
    approve: bool,
    // Relayed viewers are already where broadcast mode wants them, so the
    // relay path never redirects. Kept in the signature so both spawners
    // match and a future per-relay handoff does not reshuffle args.
    _redirect_to: Option<(String, String)>,
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
                    serve_relay_fan(
                        transport,
                        addr,
                        tx.clone(),
                        published.clone(),
                        token.clone(),
                        viewers.clone(),
                        next_id.clone(),
                        audio.as_ref().map(|(t, r)| (t.clone(), r.resubscribe())),
                        metrics.clone(),
                        approve,
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

// ------------------------------------------------- relay fan demux
//
// One host relay connection carries one logical session per viewer. The
// relay tags each viewer frame with its id; the demux below routes by id
// and spawns one `serve_viewer` per viewer, each with its own E2E keys.
// Without this the first viewer to handshake would set the keys and every
// other viewer would receive ciphertext it cannot open.

/// One viewer's logical transport over the shared relay connection.
struct FanSession {
    id: u32,
    tx: tokio::sync::mpsc::Sender<(u32, Vec<u8>)>,
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

impl FanSession {
    fn new(
        id: u32,
        tx: tokio::sync::mpsc::Sender<(u32, Vec<u8>)>,
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) -> Self {
        Self { id, tx, rx }
    }
}

struct FanSink {
    id: u32,
    tx: tokio::sync::mpsc::Sender<(u32, Vec<u8>)>,
}

struct FanSource {
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

#[async_trait::async_trait]
impl MessageSink for FanSink {
    async fn send(&mut self, msg: &Message) -> anyhow::Result<()> {
        self.send_encoded(&msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.tx
            .send((self.id, bytes.to_vec()))
            .await
            .map_err(|_| anyhow::anyhow!("relay fan is gone"))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MessageSource for FanSource {
    async fn recv(&mut self) -> anyhow::Result<Message> {
        let bytes = self
            .rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("relay fan session closed"))?;
        Ok(Message::decode(&bytes)?)
    }

    async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("relay fan session closed"))
    }
}

#[async_trait::async_trait]
impl MessageTransport for FanSession {
    async fn send(&mut self, msg: &Message) -> anyhow::Result<()> {
        self.send_encoded(&msg.encode()?).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.tx
            .send((self.id, bytes.to_vec()))
            .await
            .map_err(|_| anyhow::anyhow!("relay fan is gone"))?;
        Ok(())
    }

    async fn recv(&mut self) -> anyhow::Result<Message> {
        let bytes = self
            .rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("relay fan session closed"))?;
        Ok(Message::decode(&bytes)?)
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        let FanSession { id, tx, rx } = *self;
        (Box::new(FanSink { id, tx }), Box::new(FanSource { rx }))
    }
}

/// Per-viewer queue between the demux and one `serve_viewer`.
const FAN_SESSION_QUEUE: usize = 64;

/// Run one relay connection as a fan: route tagged frames by peer id and
/// spawn one `serve_viewer` per viewer. Returns when the connection dies,
/// after tearing every session down.
#[allow(clippy::too_many_arguments)]
async fn serve_relay_fan(
    transport: RelayTransport,
    peer: SocketAddr,
    tx: EncodedBroadcast,
    published: Shared,
    token: SessionToken,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_viewer_id: Arc<AtomicU64>,
    audio: Option<(AudioBroadcast, broadcast::Receiver<Arc<EncodedAudio>>)>,
    metrics: crate::telemetry::SharedMetrics,
    approve: bool,
) {
    let fan: RelayFan = Box::new(transport).into_fan();
    let mut fan = fan;
    // One task owns the session map, so no lock is needed. Session death
    // is visible through `exit_rx`, so a dead session's queue cannot fill
    // and stall the demux.
    let mut sessions: HashMap<u32, tokio::sync::mpsc::Sender<Vec<u8>>> = HashMap::new();
    let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel::<u32>(64);
    loop {
        tokio::select! {
            frame = fan.rx.recv() => {
                let Some((id, bytes)) = frame else { break }; // relay connection lost
                if id == RELAY_CONTROL_ID {
                    // `[kind][viewer_id]`: ViewerLeft tears the session
                    // down; ViewerHere spawns one for an idle viewer the
                    // host would otherwise never hear from.
                    if bytes.len() == 5 {
                        let viewer = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
                        if bytes[0] == 0xEE {
                            sessions.remove(&viewer);
                        } else if bytes[0] == 0xEF && !sessions.contains_key(&viewer) {
                            sessions.insert(
                                viewer,
                                spawn_fan_session(
                                    viewer, peer, &fan.tx, &tx, &published, &token,
                                    &viewers, &next_viewer_id, &audio, &metrics,
                                    approve, &exit_tx,
                                ),
                            );
                        }
                    }
                    continue;
                }
                // Implicit join: the first frame from an unknown id spawns
                // the session. A non-Hello first frame is rejected by
                // `serve_viewer`'s existing handshake path.
                let send = match sessions.get(&id) {
                    Some(send) => send.clone(),
                    None => {
                        let send = spawn_fan_session(
                            id, peer, &fan.tx, &tx, &published, &token,
                            &viewers, &next_viewer_id, &audio, &metrics,
                            approve, &exit_tx,
                        );
                        sessions.insert(id, send.clone());
                        send
                    }
                };
                // Awaited: backpressure, not a silent drop. A dead session
                // surfaces its error here and is reaped below.
                if send.send(bytes).await.is_err() {
                    sessions.remove(&id);
                }
            }
            exited = exit_rx.recv() => {
                let Some(id) = exited else { break };
                sessions.remove(&id);
            }
        }
    }
    // Connection lost: drop every session sender so each `serve_viewer`
    // exits through its closed source.
    sessions.clear();
}

#[allow(clippy::too_many_arguments)]
fn spawn_fan_session(
    id: u32,
    peer: SocketAddr,
    fan_tx: &tokio::sync::mpsc::Sender<(u32, Vec<u8>)>,
    tx: &EncodedBroadcast,
    published: &Shared,
    token: &SessionToken,
    viewers: &Arc<Mutex<HashMap<u64, ViewerStats>>>,
    next_viewer_id: &Arc<AtomicU64>,
    audio: &Option<(AudioBroadcast, broadcast::Receiver<Arc<EncodedAudio>>)>,
    metrics: &crate::telemetry::SharedMetrics,
    approve: bool,
    exit_tx: &tokio::sync::mpsc::Sender<u32>,
) -> tokio::sync::mpsc::Sender<Vec<u8>> {
    let (session_tx, session_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(FAN_SESSION_QUEUE);
    let (fan_tx, tx, published, token, viewers, next_id, audio, metrics, approve, exit_tx) = (
        fan_tx.clone(),
        tx.clone(),
        published.clone(),
        token.clone(),
        viewers.clone(),
        next_viewer_id.clone(),
        audio.as_ref().map(|(t, r)| (t.clone(), r.resubscribe())),
        metrics.clone(),
        approve,
        exit_tx.clone(),
    );
    tokio::spawn(async move {
        serve_viewer(
            Box::new(FanSession::new(id, fan_tx, session_rx)),
            peer,
            &format!("relay id={id}"),
            tx,
            published,
            token,
            viewers,
            next_id,
            audio,
            None,
            metrics,
            approve,
            None,
        )
        .await;
        let _ = exit_tx.send(id).await;
    });
    session_tx
}

// ------------------------------------------------------------ one viewer

#[allow(clippy::too_many_arguments)]
/// Split a comma-separated `--relay` list, dropping empties. One
/// address stays one address: no whitespace games, no dedup — the order
/// is the preference order viewers probe in.
fn split_relays(spec: &str) -> Vec<String> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// How many direct viewers are currently registered.
fn direct_viewer_count(viewers: &Arc<Mutex<HashMap<u64, ViewerStats>>>) -> usize {
    viewers.lock().values().filter(|v| v.direct).count()
}

/// Where over-threshold direct viewers are sent: the first named relay
/// plus the session code they must rejoin with. `None` when no relay is
/// configured — failing viewers off the share would be worse than serving
/// them slowly, so broadcast mode simply stays off.
fn redirect_target(args: &ShareArgs) -> Option<(String, String)> {
    let relay = split_relays(args.relay.as_deref()?).first()?.clone();
    let session = args.session.clone().unwrap_or_else(generate_session_code);
    Some((relay, session))
}

/// `true` when stdin is closed (EOF immediately): tests, daemons,
/// pipes. Approval prompts must not block forever there.
fn is_stdin_closed(stdin: &std::io::Stdin) -> bool {
    use std::io::BufRead;
    let mut buf = [0u8; 1];
    let mut locked = stdin.lock();
    // Peek without consuming: a closed stdin reads 0 bytes now and on
    // every later read, so treating it as closed is stable.
    matches!(locked.fill_buf(), Ok(b) if b.is_empty())
        && matches!(std::io::Read::read(&mut locked, &mut buf), Ok(0))
}

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
    audio: Option<(AudioBroadcast, broadcast::Receiver<Arc<EncodedAudio>>)>,
    quic: Option<quinn::Connection>,
    metrics: crate::telemetry::SharedMetrics,
    approve: bool,
    redirect_to: Option<(String, String)>,
) {
    // Subscribe *before* anything else, so no update can slip between the
    // snapshot we are about to take and the stream we are about to start.
    let mut rx = tx.subscribe();
    let (mut sink, mut source) = transport.split();
    let label = format!("{path} viewer {peer}");

    let handshake = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
        match source.recv().await {
            Ok(Message::Hello { token, resume }) => Some((token, resume)),
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

    let (presented, resume) = match handshake {
        Ok(Some((presented, resume))) if verify_token(&token, &presented) => (presented, resume),
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
    if approve {
        // A human at the keyboard decides. The prompt names the peer and
        // the session; anything but an explicit `yes` refuses. stdin is
        // read on a blocking thread so the async runtime never stalls —
        // and when stdin is closed (tests, daemons) that reads as refusal
        // would hang, so a closed stdin admits instead of deadlocking:
        // approval is a convenience gate, the token stays the secret.
        let approved = tokio::task::spawn_blocking(move || {
            use std::io::{BufRead, Write};
            let stdin = std::io::stdin();
            if is_stdin_closed(&stdin) {
                return true;
            }
            let mut out = std::io::stdout();
            let _ = writeln!(out, "Viewer {peer} requests the screen. Admit? [y/N]");
            let _ = out.flush();
            let mut line = String::new();
            std::io::BufReader::new(stdin.lock())
                .read_line(&mut line)
                .is_ok()
                && matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
        })
        .await
        .unwrap_or(false);
        if !approved {
            let _ = sink
                .send(&Message::Error("The sharer declined this viewer.".into()))
                .await;
            let _ = tokio::time::timeout(Duration::from_secs(2), source.recv()).await;
            warn!("{label} declined by approval prompt");
            return;
        }
    }
    let id = next_viewer_id.fetch_add(1, Ordering::Relaxed);
    // serve_viewer does not know its own path; the spawner marks it
    // right after. Default here is relay (false): a direct session that
    // never gets marked is never redirected, which is the safe side.
    {
        let mut guard = viewers.lock();
        // The QUIC handle only exists on direct sessions; the fan passes
        // None. Marking here (not at the spawner) keeps the two call sites
        // from disagreeing about what "direct" means.
        guard.insert(
            id,
            ViewerStats {
                direct: quic.is_some(),
                ..Default::default()
            },
        );
    }
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

    // Audio travels separately from visual messages, and loss must not
    // turn into a stall. A QUIC uni stream would retransmit a dropped
    // packet and playback would wait, turning one lost radio frame into
    // an audible pause. Unreliable QUIC datagrams arrive latest-only: a
    // lost packet is simply gone and the next frame carries on.
    // Datagrams do not reach relayed viewers or browsers, because those
    // paths do not expose the QUIC connection. That is a deliberate
    // omission, not an accident: the alternative would serialize several
    // loss-recovery protocols without measuring how anyone uses one.
    if let (Some((_, _)), Some(mut arx), Some(connection)) = (
        audio.as_ref(),
        audio.as_ref().map(|(_, r)| r.resubscribe()),
        quic.as_ref(),
    ) {
        let audio_label = label.clone();
        let send_conn = connection.clone();
        let audio_metrics = metrics.clone();
        tokio::spawn(async move {
            loop {
                let frame = match arx.recv().await {
                    Ok(f) => f,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // A viewer that cannot keep up with audio loses a
                        // click, not a session.
                        warn!("{audio_label} audio fell behind by {n} frames");
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                let payload = match crate::audio::transport::AudioSender::datagram_payload(&frame) {
                    Ok(payload) => payload,
                    Err(e) => {
                        warn!("{audio_label} oversize audio frame dropped: {e:#}");
                        audio_metrics.audio_frames_dropped.incr();
                        continue;
                    }
                };
                if let Err(e) = send_conn.send_datagram(bytes::Bytes::from(payload)) {
                    // Local backpressure means the transport has not yet
                    // handed the frame to UDP; drop it there instead of
                    // hiding the loss in an unbounded queue. Audio's job
                    // is current speech, not a replay.
                    tracing::debug!("{audio_label} audio datagram not queued: {e}");
                    audio_metrics.audio_frames_dropped.incr();
                    continue;
                }
                audio_metrics.audio_frames_sent.incr();
            }
        });
        info!("{label} audio over QUIC datagrams enabled");
    }

    // A reconnecting viewer names what it last applied. When the ring
    // still covers it at the same epoch, replay catches it up with no
    // snapshot; anything else snapshots exactly as a fresh join does.
    // Fresh crypto either way: the E2E handshake above already derived new
    // session keys, so no counter or nonce crosses the reconnect.
    let mut floor = match resume {
        Some((epoch, rev)) => {
            let replay = {
                let p = published.read().await;
                if epoch == p.epoch {
                    p.ring.replay_from(rev, epoch)
                } else {
                    None
                }
            };
            match replay {
                Some(msgs) => {
                    let mut caught = rev;
                    for bytes in &msgs {
                        if let Err(e) = sink.send_encoded(bytes).await {
                            warn!("{label} resume replay failed: {e}");
                            return;
                        }
                        if let Some(r) = crate::network::peek_rev(bytes) {
                            caught = caught.max(r);
                        }
                    }
                    info!(
                        "{label} resumed from rev {rev} with {} replayed messages",
                        msgs.len()
                    );
                    caught
                }
                None => match send_snapshot(&mut sink, &published).await {
                    Ok(rev) => rev,
                    Err(e) => {
                        warn!("{label} could not be caught up: {e}");
                        return;
                    }
                },
            }
        }
        None => match send_snapshot(&mut sink, &published).await {
            Ok(rev) => rev,
            Err(e) => {
                warn!("{label} could not be caught up: {e}");
                return;
            }
        },
    };
    // The floor is applied by definition: the viewer holds it, whether it
    // came from a snapshot or a replay of contiguous history.
    if let Some(v) = viewers.lock().get_mut(&id) {
        v.acked_rev = floor;
    }

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

    // Transport measurements, sampled per send on direct QUIC sessions.
    // Relay/fan sessions pass `None` and keep the last sample: no handle,
    // no measurement, no guessing.
    let quic_stats = quic.clone();
    let mut last_lost: Option<u64> = None;
    let mut consecutive_lags = 0u32;
    // Sample at most every 250 ms: RTT/loss/cwnd move slowly, and the QUIC
    // stats lock is not free.
    let mut last_sample = Instant::now() - Duration::from_secs(1);
    loop {
        // Broadcast-mode handoff: one Redirect, then this session ends.
        // The viewer reconnects through the relay with its resume point,
        // so the move loses no pixels — only the direct socket.
        if viewers.lock().get(&id).is_some_and(|v| v.redirect_queued) {
            if let Some((relay, session)) = redirect_to.clone() {
                let _ = sink.send(&Message::Redirect { relay, session }).await;
                info!("{label} redirected to relay (broadcast mode)");
            }
            break;
        }
        tokio::select! {
            update = rx.recv() => match update {
                Ok(bytes) => {
                    let Some(rev) = crate::network::peek_rev(&bytes) else {
                        continue;
                    };
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
                    // Fresh sample with every delivered update, throttled.
                    if last_sample.elapsed() >= Duration::from_millis(250) {
                        if let Some(conn) = quic_stats.as_ref() {
                            let stats = conn.stats();
                            let lost = stats.path.lost_packets;
                            let is_new_loss = last_lost.is_some_and(|prev| lost > prev);
                            last_lost = Some(lost);
                            if let Some(v) = viewers.lock().get_mut(&id) {
                                v.rtt_ms = Some(conn.rtt().as_millis().min(u128::from(u64::MAX)) as u64);
                                v.lost_packets = lost;
                                v.cwnd_bytes = stats.path.cwnd;
                                if is_new_loss {
                                    v.lag_events += 1;
                                }
                            }
                            last_sample = Instant::now();
                        }
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
                    match repair_viewer(&mut sink, &published, floor).await {
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
                    Message::Ack { rev } => {
                        // Cumulative applied watermark, monotonic: an old
                        // duplicate says nothing new.
                        if let Some(v) = viewers.lock().get_mut(&id) {
                            v.acked_rev = v.acked_rev.max(rev);
                        }
                    }
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

/// Catch a lagging viewer up: replay the revision ring when it covers
/// the viewer's floor, else fall back to a fresh snapshot. Returns the new
/// floor. A viewer that only missed a few updates gets those exact bytes
/// replayed; one that is far behind, predates the ring, or crossed an
/// epoch gets a snapshot. A `RequestKeyframe` still means snapshot — the
/// viewer declared its state unusable, so replaying against it would be wrong.
pub async fn repair_viewer(
    sink: &mut Box<dyn MessageSink>,
    published: &Shared,
    floor: Rev,
) -> Result<Rev> {
    let replay = {
        let p = published.read().await;
        p.ring.replay_from(floor, p.epoch)
    };
    let Some(msgs) = replay else {
        return send_snapshot(sink, published).await;
    };
    let mut new_floor = floor;
    for bytes in &msgs {
        sink.send_encoded(bytes).await?;
        if let Some(rev) = crate::network::peek_rev(bytes) {
            new_floor = new_floor.max(rev);
        }
    }
    Ok(new_floor)
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
            pts_us: 0,
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
    out.push(
        Message::SnapshotCommit {
            rev,
            pts_us: 0,
            epoch,
        }
        .encode()?,
    );
    Ok(out)
}

// ---------------------------------------------------------- capture loop

#[allow(clippy::too_many_arguments)]
async fn capture_loop(
    metrics: crate::telemetry::SharedMetrics,
    mut first: Frame,
    capture: CaptureSource,
    mut cursor_sampler: Box<dyn crate::capture::CursorSampler>,
    planner: Planner,
    published: Shared,
    tx: EncodedBroadcast,
    viewers: Arc<Mutex<HashMap<u64, ViewerStats>>>,
    surface: Option<Arc<SharedSurface>>,
    target_quality: f32,
    requested_fps: u32,
    max_fps: u32,
    repair_interval: Duration,
    broadcast_above: usize,
    redirect_to: Option<(String, String)>,
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
    let mut last_total_lost: u64 = 0;
    let mut last_repair = Instant::now();
    let mut quality = target_quality;
    let mut pressure = Pressure {
        value: 0.0,
        last_change: Instant::now(),
    };
    // Last cursor state sent: `None` means the viewers currently hide it.
    // A message goes out only on change, so an idle pointer costs nothing.
    let mut last_cursor: Option<(u32, u32)> = None;
    let mut cursor_hidden = false;

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
        // A mutable view of the shared surface.
        //
        // The comment here used to claim this "copies only if a joining
        // viewer is holding the same allocation right now". That was
        // wrong: the loop publishes `reference.clone()` below, which is
        // an Arc clone, so the second strong count is always there and
        // this deep-copies the whole framebuffer every frame.
        //
        // Measured on an M-series Mac at 1920x1080: 0.236 ms/frame, about
        // 1.4% of a 60 fps budget. Real traffic, small wall clock -- the
        // copy engine moves 5.9 MiB in a quarter of a millisecond.
        //
        // It is left in place deliberately. Publishing the surface every
        // frame is what makes a late joiner see what is on screen *now*,
        // and that is a bug this code shipped once already. Trading 1.4%
        // of a frame budget for a chance to reintroduce it is a bad deal.
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
        // Two phases, two numbers. Recording the same span twice made
        // pcc_detect_seconds and pcc_plan_seconds identical, so neither
        // said which phase was actually expensive.
        let total = detect_start.elapsed();
        metrics.detect.record_duration(plan.detect);
        metrics
            .plan
            .record_duration(total.saturating_sub(plan.detect));
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
        // Preview inputs, computed before `plan.ops` moves into the
        // update below. No second scan: the bounds walk is one pass over
        // the (already materialised) op list.
        let changed_fraction = plan.changed_fraction(width, height);
        let changed_bounds = plan.changed_bounds();

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
                let shared = Arc::new(msg);
                published
                    .write()
                    .await
                    .ring
                    .push(rev, epoch, shared.clone());
                let _ = tx.send(shared);
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
                pts_us: 0,
                epoch,
                ops: plan.ops,
            }
            .encode();
            match bytes {
                Ok(bytes) => {
                    metrics.observe_message_bytes(bytes.len());
                    metrics.serialize.record_duration(serialize_start.elapsed());
                    let shared = Arc::new(bytes);
                    published
                        .write()
                        .await
                        .ring
                        .push(rev, epoch, shared.clone());
                    let _ = tx.send(shared);
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
                        let shared = Arc::new(msg);
                        published
                            .write()
                            .await
                            .ring
                            .push(rev, epoch, shared.clone());
                        let _ = tx.send(shared);
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
                let shared = Arc::new(bytes);
                published
                    .write()
                    .await
                    .ring
                    .push_keepalive(rev, epoch, shared.clone());
                let _ = tx.send(shared);
            }
        }

        // The cursor plane: sample once per frame, send only on
        // change. A move under 2px is jitter, not intent — the platform
        // sampler reports raw positions and the viewer would shimmer.
        match cursor_sampler.sample(width, height) {
            Some(c)
                if last_cursor.is_none_or(|(x, y)| x.abs_diff(c.x).max(y.abs_diff(c.y)) >= 2) =>
            {
                last_cursor = Some((c.x, c.y));
                cursor_hidden = false;
                if let Ok(bytes) = (Message::CursorMove { x: c.x, y: c.y }).encode() {
                    let shared = Arc::new(bytes);
                    published
                        .write()
                        .await
                        .ring
                        .push_keepalive(rev, epoch, shared.clone());
                    let _ = tx.send(shared);
                }
            }
            None if !cursor_hidden => {
                cursor_hidden = true;
                last_cursor = None;
                if let Ok(bytes) = Message::CursorHide.encode() {
                    let shared = Arc::new(bytes);
                    published
                        .write()
                        .await
                        .ring
                        .push_keepalive(rev, epoch, shared.clone());
                    let _ = tx.send(shared);
                }
            }
            _ => {}
        }
        // The motion lane: when the changed fraction says "fling", send an
        // interim rectangle before the exact update lands, so a viewer on
        // a slow path shows *something* instead of a frozen frame. It is
        // never authoritative — the exact frame still decides the pixels.
        if changed_fraction >= MOTION_PREVIEW_FRACTION {
            if let Some((x, y, w, h)) = changed_bounds {
                let preview = Message::MotionPreview {
                    rev,
                    x,
                    y,
                    width: w,
                    height: h,
                    changed_fraction: changed_fraction as f32,
                };
                if let Ok(bytes) = preview.encode() {
                    metrics.bytes_preview.add(bytes.len() as u64);
                    let _ = tx.send(Arc::new(bytes));
                }
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
        if let Some(surface) = &surface {
            surface.publish(&frame, quality);
        }

        // Real feedback-driven adaptation: the network is the signal,
        // not local CPU overrun. Lag events say a viewer missed the
        // broadcast; the ack watermark says how far behind it actually is;
        // RTT/loss/cwnd say whether the path itself is degraded. A viewer
        // acking every revision over a clean path is merely slow; one on a
        // lossy, high-RTT, collapsing-window path is congested.
        pressure.decay(loop_start.elapsed());
        let current = rev;
        let stats: Vec<ViewerStats> = viewers.lock().values().cloned().collect();
        let lagging = stats.iter().filter(|v| v.lag_events > 0).count();
        let stalled = stats
            .iter()
            .filter(|v| v.lag_events > 0 && current.saturating_sub(v.acked_rev) > 30)
            .count();
        // Worst measured path across direct viewers: relay sessions expose
        // no handle and do not vote here.
        let worst_rtt_ms = stats.iter().filter_map(|v| v.rtt_ms).max().unwrap_or(0);
        let total_lost: u64 = stats.iter().map(|v| v.lost_packets).sum();
        let min_cwnd = stats.iter().map(|v| v.cwnd_bytes).filter(|c| *c > 0).min();
        let cwnd_collapsed = min_cwnd.is_some_and(|c| c < 64 * 1024);
        // The path is degraded even if nobody lagged yet: back off before
        // the queues fill rather than after. `last_total_lost` diffs the
        // monotonic counter, so only *new* loss penalises.
        let new_loss = total_lost > last_total_lost;
        last_total_lost = total_lost;
        if lagging > 0 {
            // Stalled viewers weigh double: they are not catching up.
            pressure.penalise(lagging as f32 + stalled as f32);
        }
        if worst_rtt_ms >= 250 || new_loss {
            pressure.penalise(1.0);
        }
        // The step itself is `decide_congestion_step` below (unit-tested);
        // the loop only adds the pressure gate and the 5 s recovery delay.
        let step = decide_congestion_step(
            effective_fps,
            requested_fps,
            worst_rtt_ms,
            new_loss,
            cwnd_collapsed,
            loop_start.elapsed() > frame_interval_at(effective_fps) * 2,
        );
        if pressure.is_pressured() && step.fps != effective_fps {
            effective_fps = step.fps;
            info!("{lagging} viewer(s) falling behind; dropping to {effective_fps} fps");
            pressure.last_change = Instant::now();
        } else if !pressure.is_pressured()
            && step.fps != effective_fps
            && pressure.last_change.elapsed() > Duration::from_secs(5)
        {
            effective_fps = step.fps;
            pressure.last_change = Instant::now();
        }

        // Broadcast mode: past the threshold, the newest direct viewer
        // is asked to reconnect through the first relay. One per frame
        // at most, so a flash crowd drains steadily instead of all at
        // once — and each redirect is a resume, not a rejoin, so nobody
        // loses pixels. No relay configured means no redirect: failing
        // viewers off the share would be worse than serving them slowly.
        if broadcast_above > 0
            && redirect_to.is_some()
            && direct_viewer_count(&viewers) > broadcast_above
        {
            // One per frame at most: a flash crowd drains steadily, and
            // each redirect is a resume (same session code on the relay),
            // so nobody loses pixels in the move.
            let mut guard = viewers.lock();
            if let Some((_, v)) = guard
                .iter_mut()
                .filter(|(_, v)| v.direct && !v.redirect_queued)
                .max_by_key(|(id, _)| **id)
            {
                v.redirect_queued = true;
            }
        }
        // Publish the measured path state: worst RTT across direct
        // viewers, deepest ack gap as pending depth, oldest unapplied age.
        // These are the numbers the fps/quality decisions above actually
        // read, so the dashboard shows the cause, not just the effect.
        metrics.viewer_rtt_ms.set_raw(worst_rtt_ms);
        let max_gap = stats
            .iter()
            .map(|v| current.saturating_sub(v.acked_rev))
            .max()
            .unwrap_or(0);
        metrics.viewer_queue_depth.set_raw(max_gap);
        // One rev is roughly one frame interval old, so the gap converts
        // to an age without tracking per-message timestamps.
        metrics
            .oldest_pending_ms
            .set_raw(max_gap.saturating_mul(1_000 / u64::from(effective_fps.max(1))));

        // Quality follows the same signal one step further down: a
        // congested path or an overrunning loop both degrade the *lossy*
        // preview first, because that is the only thing quality controls.
        // A collapsing window halves it outright — the bottleneck is the
        // path, not the encoder.
        let frame_interval = frame_interval_at(effective_fps);
        if step.degrade_quality && quality > 0.3 {
            quality = if cwnd_collapsed {
                (quality - 0.2).max(0.3)
            } else {
                (quality - 0.1).max(0.3)
            };
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

/// Frame pacing for an fps value. One place, so the loop and the
/// congestion step cannot disagree about what "one frame" means.
fn frame_interval_at(fps: u32) -> Duration {
    Duration::from_secs(1) / fps.max(1)
}

/// Pure decision core of the network-driven fps step: given one frame's
/// measured path state, what fps follows and does quality degrade. The
/// capture loop calls this inline; the unit tests below pin the
/// thresholds without running a loop.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CongestionDecision {
    fps: u32,
    degrade_quality: bool,
}

fn decide_congestion_step(
    fps: u32,
    requested_fps: u32,
    worst_rtt_ms: u64,
    new_loss: bool,
    cwnd_collapsed: bool,
    loop_overran: bool,
) -> CongestionDecision {
    let congested_path = worst_rtt_ms >= 250 || new_loss;
    let mut fps = fps;
    if congested_path && fps > 10 {
        fps = (fps * 3 / 4).max(10);
    } else if !congested_path && fps < requested_fps && worst_rtt_ms < 100 {
        fps = (fps * 4 / 3).min(requested_fps);
    }
    let degrade_quality = loop_overran || congested_path || cwnd_collapsed;
    CongestionDecision {
        fps,
        degrade_quality,
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
    fn broadcast_redirect_needs_a_relay() {
        let mut args = ShareArgs {
            listen: None,
            relay: None,
            relay_pin: String::new(),
            session: Some("ABC123".into()),
            web: None,
            web_cert: None,
            synthetic: true,
            capture_target: crate::capture::CaptureTarget::Display(0),
            fps: 30,
            max_fps: 60,
            quality: 0.8,
            token: crate::network::SessionToken::parse("TOKENTOKENTOKENTOKENTOKENTOKENTOK1")
                .unwrap(),
            repair_interval: std::time::Duration::from_secs(30),
            reach: crate::reach::ReachPolicy::TryDirect,
            approve: false,
            broadcast_above: 2,
            audio_source: crate::audio::AudioSource::None_,
        };
        assert!(redirect_target(&args).is_none());
        args.relay = Some("r1:1, r2:2".into());
        assert_eq!(
            redirect_target(&args),
            Some(("r1:1".to_string(), "ABC123".to_string()))
        );
    }

    #[test]
    fn redirect_flags_only_unflagged_direct_viewers() {
        let viewers: Arc<Mutex<HashMap<u64, ViewerStats>>> = Arc::new(Mutex::new(HashMap::new()));
        assert_eq!(direct_viewer_count(&viewers), 0);
        for (id, direct) in [(1u64, true), (2, true), (3, false)] {
            viewers.lock().insert(
                id,
                ViewerStats {
                    direct,
                    ..Default::default()
                },
            );
        }
        assert_eq!(direct_viewer_count(&viewers), 2);
    }

    #[test]
    fn relay_list_splits_and_skips_empties() {
        assert_eq!(split_relays("a:1"), vec!["a:1"]);
        assert_eq!(split_relays("a:1, b:2 ,c:3"), vec!["a:1", "b:2", "c:3"]);
        assert!(split_relays(" , ,").is_empty());
        // Order is preference: never sorted, never deduped.
        assert_eq!(split_relays("b:2,a:1,b:2"), vec!["b:2", "a:1", "b:2"]);
    }

    #[test]
    fn closed_stdin_counts_as_closed() {
        // /dev/null reads EOF immediately: the worst case the approval
        // prompter must survive without hanging.
        use std::io::BufRead;
        let f = std::fs::File::open("/dev/null").unwrap();
        let mut locked = std::io::BufReader::new(f);
        let mut buf = Vec::new();
        assert_eq!(locked.read_until(b'x', &mut buf).unwrap(), 0);
    }

    #[test]
    fn high_rtt_backs_off_before_queues_fill() {
        let d = decide_congestion_step(30, 30, 300, false, false, false);
        assert_eq!(d.fps, 22, "a 300 ms path must shed fps, got {}", d.fps);
        assert!(d.degrade_quality);
    }

    #[test]
    fn new_loss_backs_off_before_queues_fill() {
        let d = decide_congestion_step(30, 30, 40, true, false, false);
        assert_eq!(d.fps, 22);
        assert!(d.degrade_quality);
    }

    #[test]
    fn healthy_path_recovers_and_holds_quality() {
        let d = decide_congestion_step(22, 30, 20, false, false, false);
        assert_eq!(d.fps, 29, "recovery multiplies by 4/3, got {}", d.fps);
        assert!(!d.degrade_quality);
    }

    #[test]
    fn recovery_does_not_ramp_into_a_bad_path() {
        let d = decide_congestion_step(22, 30, 300, false, false, false);
        assert_eq!(
            d.fps, 16,
            "congested step wins over recovery, got {}",
            d.fps
        );
    }

    #[test]
    fn collapsed_window_degrades_quality_without_touching_fps() {
        let d = decide_congestion_step(30, 30, 40, false, true, false);
        assert_eq!(d.fps, 30);
        assert!(d.degrade_quality);
    }

    /// The TLS gate in `start_web`, factored for tests: remote browser
    /// mode requires HTTPS, loopback stays plaintext for dev convenience.
    /// `true` means serve.
    fn web_tls_gate(addr: &str, has_cert: bool) -> bool {
        let addr: SocketAddr = addr.parse().unwrap();
        has_cert || addr.ip().is_loopback()
    }

    #[test]
    fn loopback_without_tls_is_allowed() {
        assert!(web_tls_gate("127.0.0.1:8080", false));
    }

    #[test]
    fn remote_without_tls_is_refused() {
        assert!(!web_tls_gate("0.0.0.0:8080", false));
        assert!(!web_tls_gate("192.168.1.20:8080", false));
    }

    #[test]
    fn remote_with_tls_is_allowed() {
        assert!(web_tls_gate("0.0.0.0:8080", true));
    }

    #[test]
    fn peek_rev_reads_a_partial_update_revision() {
        let bytes = Message::PartialUpdate {
            rev: 42,
            pts_us: 0,
            epoch: 1,
            ops: vec![],
        }
        .encode()
        .unwrap();
        assert_eq!(crate::network::peek_rev(&bytes), Some(42));
    }

    #[test]
    fn peek_rev_reads_a_keepalive_revision() {
        let bytes = Message::KeepAlive { rev: 7 }.encode().unwrap();
        assert_eq!(crate::network::peek_rev(&bytes), Some(7));
    }

    #[test]
    fn peek_rev_ignores_messages_without_a_revision() {
        let bytes = Message::Bye.encode().unwrap();
        assert_eq!(crate::network::peek_rev(&bytes), None);
        assert_eq!(crate::network::peek_rev(b""), None);
        assert_eq!(
            crate::network::peek_rev(&[PROTOCOL_VERSION, 0, 0, 0, 0]),
            None
        );
    }
}
