//! The native viewer: connect, authenticate, reconstruct, present.
//!
//! The state machine is `pcc::Compositor`, so this file holds only the
//! parts that are genuinely about presenting: choosing a transport,
//! telling the user what is happening, and drawing. Two rules the UI
//! follows that the old code got wrong:
//!
//! * a failed session ends the window loop instead of leaving it spinning
//!   on a screen that will never change;
//! * a resize re-reads the surface dimensions, so a sharer that changes
//!   geometry does not leave a viewer rendering into a stale buffer.

use crate::network::{
    connect_direct, Message, MessageSink, MessageSource, MessageTransport, NetworkConfig, Rev,
    SessionToken,
};
use crate::pcc::types::{Frame, BYTES_PER_PIXEL};
use crate::pcc::{ApplyError, Compositor};
use crate::relay::{RelayRole, RelayTransport};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ViewArgs {
    /// `host:port` of a sharer listening for direct connections.
    pub connect: Option<String>,
    /// `host:port` of a relay, for NAT'd peers.
    pub relay: Option<String>,
    /// Relay session code, required with `relay`.
    pub session: Option<String>,
    /// SHA-256 fingerprint of the sharer's certificate. Required with
    /// `connect`: this is what makes the connection authenticated.
    pub pin: String,
    pub token: SessionToken,
    /// Open a native window; otherwise print periodic status.
    pub show_window: bool,
    /// Reconnect after a failure instead of exiting.
    pub reconnect: bool,
}

/// How many video frames the playout clock may hold while waiting for
/// audio time to reach them.
///
/// Receipt: at 30 fps this is a second of video, far more than any
/// healthy session needs, and still bounded -- past it the oldest frame is
/// dropped, because a stale frame is worth less than a fresh one.
const PLAYOUT_DEPTH: usize = 30;

/// What the presentation layer needs to know, shared with the receive
/// task.
/// Why a session stopped, so a failed connection exits non-zero rather
/// than looking like a clean run to whatever is driving the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// The sharer ended the session normally.
    Clean,
    /// The session could not be established or was lost.
    Failed,
}

#[derive(Default)]
struct Surface {
    frame: Option<Arc<Frame>>,
    terminal: Option<(String, Stop)>,
}

/// Entry point for `pcc view`. The network loop runs on a background
/// tokio task; the window loop stays on the calling thread because GUI
/// toolkits generally require the main thread on macOS.
pub fn run_view(args: ViewArgs, metrics: crate::telemetry::SharedMetrics) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let surface: Arc<Mutex<Surface>> = Arc::new(Mutex::new(Surface::default()));
    let bg_surface = surface.clone();
    let metrics_bg = metrics.clone();
    let attempts = Arc::new(Mutex::new(0usize));
    let bg_attempts = attempts.clone();

    let view_args = args.clone();
    rt.spawn(async move {
        // Reconnect as a new synchronisation epoch: every attempt starts
        // from a clean compositor, so a half-received surface from the
        // previous attempt can never be mistaken for a valid one.
        loop {
            {
                let mut s = bg_surface.lock();
                s.frame = None;
                s.terminal = None;
            }
            // The connection is rebuilt per attempt, so it is captured
            // here rather than hoisted out of the reconnect loop.
            let outcome = receive_once(&view_args, &bg_surface, &metrics_bg).await;
            *bg_attempts.lock() += 1;
            // Every exit path must release the presentation loop, or a
            // cleanly-ended session leaves it spinning forever.
            let (message, stop) = match outcome {
                Ok(()) => {
                    let n = bg_attempts.lock();
                    let m = format!("the sharer ended the session after {n} attempt(s)");
                    info!("Viewer session ended: {m}");
                    (m, Stop::Clean)
                }
                Err(e) => {
                    // The whole chain: a bare "handshake failed" is not
                    // something anyone can act on.
                    let message = format!("{e:#}");
                    warn!("Viewer session ended: {message}");
                    (message, Stop::Failed)
                }
            };
            bg_surface.lock().terminal = Some((message, stop));
            if !view_args.reconnect {
                break;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        // Wake the presentation loop even if it was waiting on a frame
        // that will never arrive.
        if bg_surface.lock().terminal.is_none() {
            bg_surface.lock().terminal = Some(("session ended".into(), Stop::Clean));
        }
    });

    if args.show_window {
        if let Err(e) = run_window_loop(surface.clone()) {
            warn!("Falling back to headless mode (no window available): {e}");
            run_headless_loop(surface.clone());
        }
    } else {
        run_headless_loop(surface.clone());
    }

    // A session that never connected is a failure, and a script driving
    // this must be able to tell.
    let outcome = surface.lock().terminal.clone();
    match outcome {
        Some((message, Stop::Failed)) if !args.reconnect => Err(anyhow::anyhow!(message)),
        _ => Ok(()),
    }
}

/// A sealed session presented as one transport.
///
/// It cannot be split a second time: the two directions share nothing,
/// but each already owns its own counter, and splitting would hand the
/// caller halves that are individually useless.
struct SealedSession {
    sink: Box<dyn MessageSink>,
    source: Box<dyn MessageSource>,
}

#[async_trait::async_trait]
impl MessageTransport for SealedSession {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        self.sink.send(msg).await
    }

    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        self.sink.send_encoded(bytes).await
    }

    async fn recv(&mut self) -> Result<Message> {
        self.source.recv().await
    }

    fn split(self: Box<Self>) -> (Box<dyn MessageSink>, Box<dyn MessageSource>) {
        unreachable!("a sealed session is used whole")
    }
}

#[async_trait::async_trait]
impl MessageSink for Box<dyn MessageSink> {
    async fn send(&mut self, msg: &Message) -> Result<()> {
        (**self).send(msg).await
    }
    async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
        (**self).send_encoded(bytes).await
    }
}

#[async_trait::async_trait]
impl MessageSource for Box<dyn MessageSource> {
    async fn recv(&mut self) -> Result<Message> {
        (**self).recv().await
    }
    async fn recv_raw(&mut self) -> Result<Vec<u8>> {
        (**self).recv_raw().await
    }
}

async fn receive_once(
    args: &ViewArgs,
    surface: &Arc<Mutex<Surface>>,
    metrics: &crate::telemetry::SharedMetrics,
) -> Result<()> {
    anyhow::ensure!(
        !(args.connect.is_some() && args.relay.is_some()),
        "specify either --connect or --relay, not both"
    );

    // Datagrams need the QUIC `Connection`, not just one stream. Keep it
    // alongside the transport; losing it loses audio only on the native
    // path, because the relay exposes no connection handle at all.
    let mut quic: Option<quinn::Connection> = None;
    let transport: Box<dyn MessageTransport> = if let Some(target) = &args.connect {
        let addr = crate::network::resolve(target)
            .await
            .with_context(|| format!("Could not resolve --connect {target}"))?;
        let pin = crate::network::hex_to_der(&args.pin)?;
        info!("Connecting directly to {addr}");
        let mut transport = connect_direct(
            &NetworkConfig::default(),
            &pin,
            target,
            server_name_of(target),
        )
        .await?;
        // QUIC streams are not visible to the peer until data flows, so
        // the handshake kick is what makes `accept_bi` return. Sending
        // Hello does double duty: it authenticates and it opens the
        // stream.
        transport
            .send(&Message::Hello {
                token: args.token.as_str().to_string(),
            })
            .await?;
        // Retain the handle before boxing the single stream: datagrams are
        // a connection property and cannot be reached from `MessageSink`.
        quic = Some(transport.connection());
        Box::new(transport)
    } else if let Some(target) = &args.relay {
        let session = args
            .session
            .clone()
            .context("--session <CODE> is required when using --relay")?;
        let addr = crate::network::resolve(target)
            .await
            .with_context(|| format!("Could not resolve --relay {target}"))?;
        let pin = crate::network::hex_to_der(&args.pin)?;
        info!("Connecting via relay {addr}, session '{session}'");
        let mut transport = RelayTransport::connect(
            addr,
            server_name_of(target),
            &pin,
            session,
            args.token.clone(),
            RelayRole::Viewer,
        )
        .await?;
        transport
            .send(&Message::Hello {
                token: args.token.as_str().to_string(),
            })
            .await?;
        Box::new(transport)
    } else {
        anyhow::bail!(
            "Specify either --connect <host:port> or --relay <host:port> --session <code>"
        );
    };

    // End-to-end encryption, before anything else is exchanged. The token
    // is the pre-shared key, so there is no second credential to manage
    // and a wrong token fails here rather than after a snapshot has been
    // decoded.
    let keys = crate::network::e2e::KeyPair::generate();
    let (mut sink, mut source) = transport.split();
    let session = match crate::network::e2e::viewer_handshake(
        &mut sink,
        &mut source,
        keys,
        args.token.as_str(),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            // A dropped stream loses the sharer's reason, because a
            // QUIC send stream that is not finished is reset rather
            // than flushed. The viewer knows what it just sent, so it
            // says the useful thing: the token was refused.
            let detail = e.to_string();
            anyhow::bail!(
                "the sharer refused this session (token, or a different build?): {detail}"
            );
        }
    };
    let mut transport: Box<dyn MessageTransport> = Box::new(SealedSession {
        sink: Box::new(crate::network::SealedSink::new(
            sink,
            session.viewer_to_host,
        )),
        // The viewer *reads* what the host seals, so the source opens
        // with the host-to-viewer direction. Getting this backwards fails
        // authentication on the first frame, which is at least a loud
        // failure rather than a silent downgrade.
        source: Box::new(crate::network::SealedSource::new(
            source,
            session.host_to_viewer,
        )),
    });

    // Audio arrives as QUIC datagrams on the same connection, so a
    // multi-megabyte snapshot can never delay a 20 ms frame. No magic
    // header is needed: unlike `accept_uni`, a datagram cannot be mistaken
    // for ordinary picture traffic, and a lost packet is dropped rather
    // than retransmitted.
    // The two clocks, and the bridge between them. Until the estimator
    // converges, video plays as soon as it arrives rather than being
    // scheduled against a plausible-but-wrong offset.
    let mut sync = crate::audio::sync::Estimator::new();
    let mut sharer_origin: Option<std::time::Instant> = None;
    // The output device lives on its own thread, so nothing here holds a
    // `!Send` stream across an await.
    let mut audio_out: Option<crate::audio::AudioOutput> = None;
    let mut audio_played: Option<crate::audio::output::PlayedReceiver<u64>> = None;
    let mut audio: Option<crate::audio::AudioReceiver> = None;
    let mut audio_connection: Option<quinn::Connection> = None;
    if let Some(connection) = quic {
        // Datagrams exist only on QUIC direct connections. Relay
        // transports do not expose the underlying connection, so viewers
        // there intentionally receive no audio until that path is built.
        if connection.max_datagram_size().is_some() {
            match crate::audio::AudioReceiver::new() {
                Ok(receiver) => {
                    // Video is released against the audio clock,
                    // never the other way round: the audio device
                    // has a buffer and a latency nobody controls,
                    // so its time defines "now".
                    match crate::audio::AudioOutput::start::<u64>(PLAYOUT_DEPTH) {
                        Ok((out, played)) => {
                            info!("Audio playout started");
                            audio_out = Some(out);
                            audio_played = Some(played);
                        }
                        Err(e) => warn!("Audio output unavailable: {e}"),
                    }
                    audio = Some(receiver);
                    audio_connection = Some(connection);
                }
                Err(e) => warn!("Audio decoder unavailable: {e}"),
            }
        } else {
            warn!("Audio datagrams unavailable: QUIC peer did not negotiate datagram support");
        }
    }

    info!("Connected. Waiting for frames...");
    let joined_at = std::time::Instant::now();
    let mut compositor = Compositor::new();
    let mut last_ack: Rev = 0;
    // With audio, a frame waits for the audio clock rather than being
    // shown the instant it decodes. Without audio there is no clock, and
    // the frame is shown immediately.
    let mut pending: Option<Frame> = None;

    loop {
        // Drain whatever audio has arrived, then take one visual message.
        // A blocked visual stream must not stop audio, and vice versa.
        if let (Some(receiver), Some(connection), Some(out)) = (
            audio.as_mut(),
            audio_connection.as_ref(),
            audio_out.as_ref(),
        ) {
            // Datagrams are polled, not streamed: each read is at most one
            // frame, and a missing datagram is simply never delivered. The
            // timeout keeps one slow transport from holding up snapshots.
            match tokio::time::timeout(
                std::time::Duration::from_millis(1),
                connection.read_datagram(),
            )
            .await
            {
                Ok(Ok(datagram)) => match receiver.decode_datagram(&datagram) {
                    Ok(frame) => {
                        // The first audio frame pins the sharer's clock to
                        // this process's clock; every later one refines it.
                        let origin = *sharer_origin.get_or_insert_with(std::time::Instant::now);
                        sync.observe(crate::audio::sync::OffsetSample::new(
                            (origin + frame.pts()).elapsed(),
                            // No RTT measurement of our own here; the
                            // estimator's clamp keeps a bad sample from
                            // poisoning the playout.
                            std::time::Duration::from_millis(0),
                        ));
                        out.push(&frame.pcm);
                    }
                    Err(e) => warn!("Audio datagram refused: {e}"),
                },
                Ok(Err(e)) => warn!("Audio datagrams unavailable: {e}"),
                Err(_) => {}
            }
        }
        // Release whatever the audio clock says is due.
        if let Some(rx) = audio_played.as_ref() {
            while let Ok(played) = rx.try_recv() {
                if played
                    .revisions
                    .contains(&pending.as_ref().map(|f| f.id).unwrap_or(u64::MAX))
                    && !played.revisions.is_empty()
                {
                    if let Some(frame) = pending.take() {
                        surface.lock().frame = Some(Arc::new(frame));
                    }
                }
            }
        }
        let msg = transport.recv().await?;
        match msg {
            Message::SnapshotBegin {
                rev: _,
                pts_us: _,
                epoch,
                width,
                height,
                format,
                total_len,
                chunks,
            } => {
                compositor.begin_snapshot(epoch, width, height, format, total_len, chunks)?;
            }
            Message::SnapshotChunk {
                rev: _,
                index,
                data,
            } => {
                compositor.push_snapshot_chunk(index, &data)?;
            }
            Message::SnapshotCommit { rev, pts_us, epoch } => {
                let apply_start = std::time::Instant::now();
                compositor.commit_snapshot(rev, epoch)?;
                metrics.apply.record_duration(apply_start.elapsed());
                if let Some((w, h)) = compositor.dimensions() {
                    let frame = Frame::with_pts(rev, w, h, compositor.buffer().to_vec(), pts_us)?;
                    if sync.convergence() && audio_played.is_some() {
                        // Hold it: the audio clock decides when it is due.
                        pending = Some(frame);
                    } else {
                        surface.lock().frame = Some(Arc::new(frame));
                    }
                    // The gap between joining and this frame is the single
                    // number a new user cares about most.
                    metrics
                        .first_exact_image
                        .record_duration(joined_at.elapsed());
                    metrics.first_paint.record_duration(joined_at.elapsed());
                    info!("Snapshot installed: {w}x{h} at revision {rev}");
                }
                if rev != last_ack {
                    last_ack = rev;
                    transport.send(&Message::Ack { rev }).await?;
                }
            }
            Message::PartialUpdate {
                rev,
                pts_us,
                epoch,
                ops,
            } => {
                let apply_start = std::time::Instant::now();
                match compositor.apply_ops(rev, epoch, &ops) {
                    Ok(()) => {
                        metrics.apply.record_duration(apply_start.elapsed());
                        if let Some((w, h)) = compositor.dimensions() {
                            let frame =
                                Frame::with_pts(rev, w, h, compositor.buffer().to_vec(), pts_us)?;
                            if sync.convergence() && audio_played.is_some() {
                                // Hold it: the audio clock decides when
                                // it is due, never the arrival.
                                pending = Some(frame);
                            } else {
                                surface.lock().frame = Some(Arc::new(frame));
                            }
                        }
                        if rev != last_ack {
                            last_ack = rev;
                            transport.send(&Message::Ack { rev }).await?;
                        }
                    }
                    Err(ApplyError::Rejected(_)) => {
                        // The compositor knows precisely what is missing.
                        metrics.repairs.incr();
                        transport.send(&Message::RequestKeyframe).await?;
                    }
                    Err(ApplyError::Invalid(e)) => {
                        return Err(e.context("The sharer sent an update this viewer cannot apply"));
                    }
                }
            }
            Message::KeepAlive { rev } => {
                if rev > last_ack && last_ack == 0 {
                    // We have a snapshot but the sharer says it is
                    // further ahead; the next update will carry it.
                }
            }
            Message::QualityConfig(cfg) => {
                info!(
                    "Sharer adjusted quality: target_fps={} quality={:.1}",
                    cfg.target_fps, cfg.quality
                );
            }
            Message::Error(e) => return Err(anyhow::anyhow!("{e}")),
            Message::Bye => {
                info!("Sharer ended the session");
                return Ok(());
            }
            Message::Hello { .. }
            | Message::Ack { .. }
            | Message::RequestKeyframe
            | Message::E2eOffer { .. }
            | Message::E2eReply { .. } => {
                // Viewer-to-sharer messages; a sharer sending one back is
                // a protocol error, not something to act on.
                warn!("Ignoring a viewer-only message from the sharer");
            }
        }
    }
}

fn server_name_of(target: &str) -> &str {
    // The certificate is validated by fingerprint, so the SNI only has to
    // be syntactically valid. The host part is the natural choice.
    match target.rsplit_once(':') {
        Some((host, _port)) if !host.is_empty() => host,
        _ => "pcc",
    }
}

fn run_window_loop(surface: Arc<Mutex<Surface>>) -> Result<()> {
    use minifb::{Key, Window, WindowOptions};

    info!("Waiting for the first frame to size the window...");
    let first = wait_for_frame(&surface, None)?;
    let (width, height) = (first.width, first.height);

    let mut window = Window::new(
        "PixelChangeCheck Viewer",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .context("Failed to open a window (no display available?)")?;
    window.set_target_fps(60);

    // The window can be resized; the surface is re-read every time its
    // dimensions change, so a sharer that changes geometry does not leave
    // this loop drawing into a stale buffer.
    let mut current = (width, height);
    let mut argb = vec![0u32; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (frame, terminal) = {
            let s = surface.lock();
            (s.frame.clone(), s.terminal.clone())
        };
        if let Some((reason, _)) = terminal {
            warn!("Viewer stopped: {reason}");
            break;
        }
        let Some(frame) = frame else { continue };
        if (frame.width, frame.height) != current {
            argb = vec![0u32; (frame.width * frame.height) as usize];
            current = (frame.width, frame.height);
            info!("Surface resized to {}x{}", current.0, current.1);
        }
        let expected = (current.0 * current.1) as usize * BYTES_PER_PIXEL;
        if frame.data.len() != expected {
            continue;
        }
        for (i, px) in frame
            .data
            .as_chunks::<BYTES_PER_PIXEL>()
            .0
            .iter()
            .enumerate()
        {
            argb[i] = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
        }
        window.update_with_buffer(&argb, current.0 as usize, current.1 as usize)?;
    }
    Ok(())
}

fn run_headless_loop(surface: Arc<Mutex<Surface>>) {
    info!("Running headless (no window). Press Ctrl+C to quit.");
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let s = surface.lock();
        if let Some((reason, _)) = &s.terminal {
            info!("Viewer stopped: {reason}");
            return;
        }
        match &s.frame {
            Some(f) => info!(
                "Receiving {}x{} ({} bytes)",
                f.width,
                f.height,
                f.data.len()
            ),
            None => info!("Connected, waiting for the first snapshot…"),
        }
    }
}

/// Wait for a frame, optionally giving up after a deadline. The old code
/// looped here forever, so a failed connection left the app hung with no
/// way out and no explanation.
fn wait_for_frame(
    surface: &Arc<Mutex<Surface>>,
    _deadline: Option<Duration>,
) -> Result<std::sync::Arc<Frame>> {
    loop {
        let s = surface.lock();
        if let Some((reason, _)) = &s.terminal {
            anyhow::bail!("{reason}");
        }
        if let Some(frame) = &s.frame {
            return Ok(frame.clone());
        }
        drop(s);
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The address a viewer is pointed at, resolved once so the headless log
/// and error messages can name something concrete.
pub fn describe(target: &str) -> String {
    match target.parse::<SocketAddr>() {
        Ok(a) => a.to_string(),
        Err(_) => target.to_string(),
    }
}
