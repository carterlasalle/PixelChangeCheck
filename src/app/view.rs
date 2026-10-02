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
use crate::pcc::types::Frame;
#[cfg(feature = "native-viewer")]
use crate::pcc::types::BYTES_PER_PIXEL;
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
    /// `connect`; unused in iroh mode (self-certifying endpoint ids).
    pub pin: Option<String>,
    pub token: SessionToken,
    /// Transport for the session. Gated at startup; see share.
    pub transport: crate::network::TransportKind,
    /// Iroh ticket. Required in iroh mode; unused otherwise.
    pub ticket: Option<String>,
    /// WebRTC offer blob. Required in webrtc mode; unused otherwise.
    pub offer: Option<String>,
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
#[cfg(feature = "audio")]
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

/// Presentation-only overlay state. The compositor never sees it:
/// a cursor or preview can be wrong without corrupting the authoritative
/// surface, and the next exact frame simply draws under/over it.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct CursorState {
    pub x: u32,
    pub y: u32,
    pub visible: bool,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct PreviewState {
    pub rev: Rev,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub changed_fraction: f32,
}

#[derive(Default)]
struct Surface {
    frame: Option<Arc<Frame>>,
    cursor: CursorState,
    /// Latest interim hint; cleared when an exact frame at or past its
    /// revision lands, so a stale box can never outlive the real pixels.
    preview: Option<PreviewState>,
    terminal: Option<(String, Stop)>,
    /// Set when the sharer asks us to move (fanout/migration handoff).
    /// The reconnect loop reads it; a plain disconnect does not set it.
    redirect: Option<(String, String)>,
}

/// Entry point for `pcc view`. The network loop runs on a background
/// tokio task; the window loop stays on the calling thread because GUI
/// toolkits generally require the main thread on macOS.
pub fn run_view(args: ViewArgs, metrics: crate::telemetry::SharedMetrics) -> Result<()> {
    args.transport.require_implemented()?;
    let rt = tokio::runtime::Runtime::new()?;

    let surface: Arc<Mutex<Surface>> = Arc::new(Mutex::new(Surface::default()));
    let bg_surface = surface.clone();
    let metrics_bg = metrics.clone();
    let attempts = Arc::new(Mutex::new(0usize));
    let bg_attempts = attempts.clone();

    let view_args = args.clone();
    rt.spawn(async move {
        // Reconnect keeps the last applied (epoch, rev) and offers it in
        // the next Hello, so the sharer can replay the ring instead of
        // snapshotting. The presented frame stays on screen across the
        // gap: the compositor restarts clean each attempt and the replay
        // re-applies onto it, or a snapshot replaces it when the ring no
        // longer covers the floor. Crypto is fresh per attempt (new
        // KeyPair inside receive_once), so no counter or nonce crosses
        // the reconnect.
        // The 30s silence watchdog inside receive_once ends attempts
        // whose session produces nothing (Host gone, wrong code); the
        // loop then decides: without --reconnect the viewer exits
        // non-zero with the cause, with it the operator opted into
        // waiting and retries continue. Parked viewers are the normal
        // case this serves: the Host usually follows within seconds.
        let mut resume: Option<(crate::network::Epoch, crate::network::Rev)> = None;
        loop {
            // The connection is rebuilt per attempt, so it is captured
            // here rather than hoisted out of the reconnect loop.
            // `receive_once` returns what it last applied: a transport
            // failure keeps it for the next Hello, a clean end clears it.
            let outcome = receive_once(&view_args, &bg_surface, &metrics_bg, resume).await;
            *bg_attempts.lock() += 1;
            // Every exit path must release the presentation loop, or a
            // cleanly-ended session leaves it spinning forever.
            // A redirect is a handoff, not an ending: dial the named
            // relay session with the current resume point, then keep
            // going. The surface keeps showing the last exact frame
            // across the gap.
            let redirect = { bg_surface.lock().redirect.take() };
            if let Some((relay, session)) = redirect {
                {
                    info!("Following redirect to relay {relay} session {session}");
                    let mut args = view_args.clone();
                    args.connect = None;
                    args.relay = Some(relay);
                    args.session = Some(session);
                    // One race-free attempt on the new path; the loop
                    // around it still reconnects on failure.
                    let outcome2 = receive_once(&args, &bg_surface, &metrics_bg, resume).await;
                    *bg_attempts.lock() += 1;
                    match outcome2 {
                        Ok(_) => {
                            let m = "the sharer ended the session after redirect";
                            bg_surface.lock().terminal = Some((m.into(), Stop::Clean));
                            break;
                        }
                        Err(e) => {
                            let message = format!("{e:#}");
                            warn!("Viewer session ended: {message}");
                            resume = resume_point_from_error(&e);
                            bg_surface.lock().terminal = Some((message, Stop::Failed));
                            if !view_args.reconnect {
                                break;
                            }
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                }
            }
            let (message, stop, next_resume) = match outcome {
                Ok(_applied) => {
                    let n = bg_attempts.lock();
                    let m = format!("the sharer ended the session after {n} attempt(s)");
                    info!("Viewer session ended: {m}");
                    (m, Stop::Clean, None)
                }
                Err(e) => {
                    // The whole chain: a bare "handshake failed" is not
                    // something anyone can act on.
                    let message = format!("{e:#}");
                    warn!("Viewer session ended: {message}");
                    (message, Stop::Failed, resume_point_from_error(&e))
                }
            };
            // A failed attempt resumes only when the compositor actually
            // holds pixels; otherwise the next attempt snapshots scratch.
            resume = next_resume;
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

    #[cfg(feature = "native-viewer")]
    if args.show_window {
        if let Err(e) = run_window_loop(surface.clone()) {
            warn!("Falling back to headless mode (no window available): {e}");
            run_headless_loop(surface.clone());
        }
    }
    #[cfg(feature = "native-viewer")]
    if !args.show_window {
        run_headless_loop(surface.clone());
    }
    // Without the `native-viewer` feature there is no window to open:
    // every session is headless, exactly as if `--no-window` were passed.
    #[cfg(not(feature = "native-viewer"))]
    {
        if args.show_window {
            warn!("This build has no native window; running headless");
        }
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

/// Dial one direct address through Hello. Returns the transport plus
/// the QUIC connection handle (needed for audio datagrams).
async fn dial_direct(
    args: &ViewArgs,
    target: &str,
    resume: Option<(crate::network::Epoch, crate::network::Rev)>,
) -> Result<(crate::network::QuicTransport, quinn::Connection)> {
    let addr = crate::network::resolve(target)
        .await
        .with_context(|| format!("Could not resolve --connect {target}"))?;
    let pin = crate::network::hex_to_der(
        args.pin
            .as_deref()
            .context("--pin is required with --connect")?,
    )?;
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
            resume,
        })
        .await?;
    // Retain the handle before boxing the single stream: datagrams are
    // a connection property and cannot be reached from `MessageSink`.
    let conn = transport.connection();
    Ok((transport, conn))
}

/// Dial the relay list in probe order through Hello. No QUIC handle:
/// the relay exposes no connection, so relayed viewers get no datagram
/// audio — the existing deliberate omission, unchanged.
///
/// A session with no Host accepts the registration and then never sends
/// a frame, so without a deadline this waits forever with no
/// diagnostic. The whole probe gets one budget: when it expires the
/// viewer names the session and asks whether the sharer is running,
/// instead of hanging on silence.
async fn dial_relays(
    args: &ViewArgs,
    targets: &str,
    resume: Option<(crate::network::Epoch, crate::network::Rev)>,
) -> Result<(Box<dyn MessageTransport>, Option<quinn::Connection>)> {
    let session = args
        .session
        .clone()
        .context("--session <CODE> is required when using --relay")?;
    // Geography: probe each named relay in order, first handshake
    // wins. A dead relay costs one TCP timeout, not the session.
    let mut transport = None;
    let mut last_err = anyhow::anyhow!("no --relay addresses to probe");
    for target in split_relays(targets) {
        let addr = match crate::network::resolve(&target).await {
            Ok(a) => a,
            Err(e) => {
                warn!("Skipping relay '{target}': {e:#}");
                last_err = e;
                continue;
            }
        };
        let pin = crate::network::hex_to_der(
            args.pin
                .as_deref()
                .context("--pin is required with --relay")?,
        )?;
        info!("Connecting via relay {addr}, session '{session}'");
        match RelayTransport::connect(
            addr,
            server_name_of(&target),
            &pin,
            session.clone(),
            args.token.clone(),
            RelayRole::Viewer,
        )
        .await
        {
            Ok(tr) => {
                transport = Some(tr);
                break;
            }
            Err(e) => {
                warn!("Relay {addr} refused: {e:#}; trying the next");
                last_err = e;
            }
        }
    }
    let mut transport =
        transport.ok_or_else(|| last_err.context("no relay in --relay answered"))?;
    // The relay accepts a viewer into a session with no Host and then
    // sends nothing, so this Hello is the only thing standing between
    // the viewer and a silent infinite wait. Ten seconds matches the
    // sharer's handshake timeout; on expiry the viewer says the session
    // has no Host and asks whether the sharer is still running.
    tokio::time::timeout(
        Duration::from_secs(10),
        transport.send(&Message::Hello {
            token: args.token.as_str().to_string(),
            resume,
        }),
    )
    .await
    .with_context(|| {
        format!("no host in session '{session}' answered within 10s — is the sharer still running?")
    })??;
    Ok((Box::new(transport), None))
}

/// Race direct against every named relay; the first completed Hello wins
/// and the loser is dropped. Both attempts carry the same resume point,
/// so the winner replays from where the last session stopped.
async fn race_paths(
    args: &ViewArgs,
    resume: Option<(crate::network::Epoch, crate::network::Rev)>,
) -> Result<(Box<dyn MessageTransport>, Option<quinn::Connection>)> {
    let direct_target = args.connect.clone().expect("--connect for a race");
    let relay_targets = args.relay.clone().expect("--relay for a race");
    let direct_args = args.clone();
    let mut direct =
        tokio::spawn(async move { dial_direct(&direct_args, &direct_target, resume).await });
    let relay_args = args.clone();
    let mut relay =
        tokio::spawn(async move { dial_relays(&relay_args, &relay_targets, resume).await });
    // The loser is awaited, not abandoned: a slow winner that then fails
    // its E2E handshake must not strand a healthy loser.
    tokio::select! {
        d = (&mut direct) => match d {
            Ok(Ok((transport, conn))) => {
                Ok((Box::new(transport) as Box<dyn MessageTransport>, Some(conn)))
            }
            Ok(Err(e)) => {
                warn!("Direct path lost the race: {e:#}; waiting for the relay");
                let (transport, conn) = relay.await.map_err(|e| anyhow::anyhow!("{e}"))??;
                Ok((transport, conn))
            }
            Err(e) => anyhow::bail!("direct race task failed: {e}"),
        },
        r = (&mut relay) => match r {
            Ok(Ok(pair)) => Ok(pair),
            Ok(Err(e)) => {
                warn!("Relay path lost the race: {e:#}; waiting for direct");
                let (transport, conn) = direct.await.map_err(|e| anyhow::anyhow!("{e}"))??;
                let (transport, conn): (
                    crate::network::QuicTransport,
                    quinn::Connection,
                ) = (transport, conn);
                Ok((Box::new(transport) as Box<dyn MessageTransport>, Some(conn)))
            }
            Err(e) => anyhow::bail!("relay race task failed: {e}"),
        },
    }
}

async fn receive_once(
    args: &ViewArgs,
    surface: &Arc<Mutex<Surface>>,
    metrics: &crate::telemetry::SharedMetrics,
    resume: Option<(crate::network::Epoch, crate::network::Rev)>,
) -> Result<Option<(crate::network::Epoch, crate::network::Rev)>> {
    // Datagrams need the QUIC `Connection`, not just one stream. Keep it
    // alongside the transport; losing it loses audio only on the native
    // path, because the relay exposes no connection handle at all.
    let mut quic: Option<quinn::Connection> = None;
    // Iroh mode ignores connect/relay/pin: the ticket carries the
    // endpoint id plus discovered addresses, and the iroh relay (not
    // ours) bridges whatever NAT is left.
    if args.transport == crate::network::TransportKind::WebRtc {
        let offer_blob = args
            .offer
            .clone()
            .context("--offer is required with --transport webrtc")?;
        let mut pending = crate::network::webrtc_join(&offer_blob).await?;
        info!("WebRTC answer (paste it back to the sharer):");
        info!("  ANSWER: {}", pending.answer_blob);
        // Full trickle both ways over stdin/stdout, matching the sharer:
        // print our candidates as they arrive; read the sharer's (one
        // JSON blob per line) until an empty line. The loopback test
        // proved host-only signalling connects on loopback/LAN, but on a
        // real network the srflx/host candidates only arrive via trickle
        // — one blob each way was the "connection never opened" failure.
        if let Some(mut rx) = pending.candidate_out.take() {
            tokio::spawn(async move {
                while let Some(c) = rx.recv().await {
                    if let Ok(json) = serde_json::to_string(&c) {
                        println!("CANDIDATE: {json}");
                    }
                }
            });
        }
        let cand_in = pending.candidate_in.clone();
        tokio::task::spawn_blocking(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(std::io::stdin().lock()).lines() {
                let line = line.unwrap_or_default();
                if line.trim().is_empty() {
                    break;
                }
                // CANDIDATE: prefix optional; plain JSON also accepted.
                let body = line
                    .trim()
                    .strip_prefix("CANDIDATE:")
                    .map(str::trim)
                    .unwrap_or(line.trim());
                if let Ok(c) = serde_json::from_str(body) {
                    let _ = cand_in.try_send(c);
                }
            }
        });
        // The connection forms only after the sharer applies the answer;
        // finish the join (channel OnOpen) before the E2E handshake.
        if let Some(open_wait) = pending.open_wait.take() {
            open_wait
                .await
                .context("webrtc open task failed")?
                .context("webrtc connection never opened")?;
        }
        let mut transport = crate::network::webrtc_join_open(pending).await?;
        transport
            .send(&Message::Hello {
                token: args.token.as_str().to_string(),
                resume,
            })
            .await?;
        return receive_on_transport(Box::new(transport), args, surface, metrics, resume, None)
            .await;
    }
    if args.transport == crate::network::TransportKind::Iroh {
        let ticket = args
            .ticket
            .clone()
            .context("--ticket is required with --transport iroh")?;
        let mut transport = crate::network::iroh_dial(&ticket).await?;
        transport
            .send(&Message::Hello {
                token: args.token.as_str().to_string(),
                resume,
            })
            .await?;
        return receive_on_transport(Box::new(transport), args, surface, metrics, resume, None)
            .await;
    }
    let transport: Box<dyn MessageTransport> = match (&args.connect, &args.relay) {
        // Path migration at connect time: race direct against the relay
        // list, first session wins. A viewer behind the same NAT as the
        // sharer gets the LAN path; one outside gets the relay — with
        // one command, no `--reach` guessing.
        (Some(_), Some(_)) => {
            let (transport, conn) = race_paths(args, resume).await?;
            quic = conn;
            transport
        }
        (Some(target), None) => {
            let (transport, conn) = dial_direct(args, target, resume).await?;
            quic = Some(conn);
            Box::new(transport)
        }
        (None, Some(targets)) => {
            let (transport, _) = dial_relays(args, targets, resume).await?;
            transport
        }
        (None, None) => {
            anyhow::bail!(
                "Specify --connect <host:port> and/or --relay <host:port> --session <code>"
            );
        }
    };
    receive_on_transport(transport, args, surface, metrics, resume, quic).await
}

/// The shared tail of every viewer session: E2E handshake, audio setup,
/// and the apply loop. Dial paths differ per transport; everything
/// after Hello is identical, so it lives here once.
async fn receive_on_transport(
    transport: Box<dyn MessageTransport>,
    args: &ViewArgs,
    surface: &std::sync::Arc<parking_lot::Mutex<Surface>>,
    metrics: &crate::telemetry::SharedMetrics,
    _resume: Option<(crate::network::Epoch, crate::network::Rev)>,
    #[cfg_attr(not(feature = "audio"), allow(unused_variables))] quic: Option<quinn::Connection>,
) -> Result<Option<(crate::network::Epoch, crate::network::Rev)>> {
    // End-to-end encryption, before anything else is exchanged. The token
    // is the pre-shared key, so there is no second credential to manage
    // and a wrong token fails here rather than after a snapshot has been
    // decoded.
    let keys = crate::network::e2e::KeyPair::generate();
    let (mut sink, mut source) = transport.split();
    // The handshake waits on the sharer twice (offer, then reply). A
    // parked session produces neither, so this is the second place a
    // Host-less session hangs — same 30s budget as the frame loop, same
    // diagnostic.
    let session = match tokio::time::timeout(
        Duration::from_secs(30),
        crate::network::e2e::viewer_handshake(&mut sink, &mut source, keys, args.token.as_str()),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "no handshake reply for 30s — the sharer may have left, or the session code is wrong"
        )
    })? {
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
    #[cfg(feature = "audio")]
    let mut sync = crate::audio::sync::Estimator::new();
    #[cfg(feature = "audio")]
    let mut sharer_origin: Option<std::time::Instant> = None;
    // The output device lives on its own thread, so nothing here holds a
    // `!Send` stream across an await.
    //
    // Without the `audio` feature none of this exists: the viewer still
    // runs, it just never has an audio clock and shows every frame on
    // arrival, exactly as if the sharer had sent silence forever.
    #[cfg(feature = "audio")]
    let mut audio_out: Option<crate::audio::AudioOutput> = None;
    #[cfg(feature = "audio")]
    let mut audio_played: Option<crate::audio::output::PlayedReceiver<u64>> = None;
    #[cfg(feature = "audio")]
    let mut audio: Option<crate::audio::AudioReceiver> = None;
    #[cfg(feature = "audio")]
    let mut audio_connection: Option<quinn::Connection> = None;
    // Adaptive hold for late frames, plus the arrival/pts history that
    // feeds it. Gaps are concealed in pts order when the next frame
    // arrives; the hold bounds how many frames one gap may produce.
    #[cfg(feature = "audio")]
    let mut audio_jitter = crate::audio::Jitter::new();
    #[cfg(feature = "audio")]
    let mut last_arrival: Option<std::time::Instant> = None;
    #[cfg(feature = "audio")]
    let mut last_audio_pts: Option<u64> = None;
    #[cfg(feature = "audio")]
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
    // One failed sealed frame requests a fresh snapshot instead of
    // exiting; the second consecutive one exits, so a truly broken peer
    // cannot spin here forever. Reset on every good message.
    let mut resyncing = false;
    // With audio, a frame waits for the audio clock rather than being
    // shown the instant it decodes. Without audio there is no clock, and
    // the frame is shown immediately, so there is nothing to hold.
    #[cfg(feature = "audio")]
    let mut pending: Option<Frame> = None;

    loop {
        // Drain whatever audio has arrived, then take one visual message.
        // A blocked visual stream must not stop audio, and vice versa.
        #[cfg(feature = "audio")]
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
                Ok(Ok(datagram)) => {
                    let now = std::time::Instant::now();
                    if let Some(prev) = last_arrival {
                        audio_jitter.observe_gap_us(
                            now.duration_since(prev)
                                .as_micros()
                                .min(u128::from(u64::MAX)) as u64,
                        );
                    }
                    last_arrival = Some(now);
                    // Push one decoded frame, filling any pts gap ahead of
                    // it in order. The arriving packet's FEC repairs the
                    // first missing frame when the encoder emitted it; the
                    // rest get concealment. The hold bounds the run so a
                    // dead link cannot fill the device with synthesis.
                    let mut push = |frame: &crate::audio::DecodedAudio| {
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
                    };
                    match receiver.decode_datagram(&datagram) {
                        Ok(frame) => {
                            if let Some(prev) = last_audio_pts {
                                let missing_us = frame
                                    .pts_us
                                    .saturating_sub(prev)
                                    .saturating_sub(crate::audio::FRAME_MS * 1_000);
                                let missing = (missing_us / (crate::audio::FRAME_MS * 1_000))
                                    .min(u64::from(audio_jitter.max_concealed_frames()))
                                    as usize;
                                for i in 1..=missing {
                                    let gap_pts = prev + i as u64 * crate::audio::FRAME_MS * 1_000;
                                    if gap_pts >= frame.pts_us {
                                        break;
                                    }
                                    let filled = if i == 1 {
                                        receiver
                                            .recover_missing_fec(gap_pts, &datagram)
                                            .or_else(|_| receiver.conceal_missing(gap_pts))
                                    } else {
                                        receiver.conceal_missing(gap_pts)
                                    };
                                    match filled {
                                        Ok(gap_frame) => push(&gap_frame),
                                        Err(e) => {
                                            warn!("Audio gap fill refused: {e}");
                                            break;
                                        }
                                    }
                                }
                            }
                            last_audio_pts = Some(frame.pts_us);
                            push(&frame);
                        }
                        Err(e) => warn!("Audio datagram refused: {e}"),
                    }
                }
                Ok(Err(e)) => warn!("Audio datagrams unavailable: {e}"),
                Err(_) => {}
            }
        }
        // Release whatever the audio clock says is due.
        #[cfg(feature = "audio")]
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
        // Silence is a symptom too: a Host that left (or a wrong
        // session code) yields no frames and no error, only this wait.
        // 30s without progress ends the attempt with the session named,
        // so the operator learns "sharer gone", not "still connecting".
        let msg = match tokio::time::timeout(Duration::from_secs(30), transport.recv()).await {
            Ok(Ok(msg)) => {
                resyncing = false;
                msg
            }
            Ok(Err(e)) => {
                // A sealed frame that fails authentication means the
                // byte stream is no longer aligned with the surface:
                // treat it as a lost sync point, not a fatal error.
                // Request a fresh snapshot and keep going; a second
                // consecutive failure still exits below, so a truly
                // broken peer cannot spin here forever.
                let context = format!("{e:#}");
                if (context.contains("sealed frame") || context.contains("authentication"))
                    && !resyncing
                {
                    warn!(
                        "Frame failed authentication at applied {}:{}; resyncing with a fresh snapshot",
                        compositor.epoch(),
                        compositor.rev()
                    );
                    metrics.repairs.incr();
                    let _ = transport.send(&Message::RequestKeyframe).await;
                    resyncing = true;
                    continue;
                }
                return Err(e).with_context(|| {
                    format!(
                        "transport failed at applied {}:{}",
                        compositor.epoch(),
                        compositor.rev()
                    )
                });
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "no frame for 30s — the sharer may have left, or the session code is wrong"
                ));
            }
        };
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
                    // With audio, hold the frame for the audio clock;
                    // without it there is no clock, so present immediately.
                    // One if/else: the frame moves exactly once, and without
                    // audio the hold arm (and `pending` itself) do not exist.
                    #[cfg(feature = "audio")]
                    if sync.convergence() && audio_played.is_some() {
                        // Hold it: the audio clock decides when it is due.
                        pending = Some(frame);
                    } else {
                        present_frame(surface, frame, rev);
                    }
                    #[cfg(not(feature = "audio"))]
                    present_frame(surface, frame, rev);
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
                            // Hold for the audio clock when there is
                            // one; otherwise present immediately. One
                            // if/else under audio, one bare call without:
                            // the frame moves exactly once either way.
                            #[cfg(feature = "audio")]
                            if sync.convergence() && audio_played.is_some() {
                                // Hold it: the audio clock decides when
                                // it is due, never the arrival.
                                pending = Some(frame);
                            } else {
                                present_frame(surface, frame, rev);
                            }
                            #[cfg(not(feature = "audio"))]
                            present_frame(surface, frame, rev);
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
            Message::CursorMove { x, y } => {
                surface.lock().cursor = CursorState {
                    x,
                    y,
                    visible: true,
                };
            }
            Message::CursorHide => {
                surface.lock().cursor.visible = false;
            }
            Message::MotionPreview {
                rev,
                x,
                y,
                width,
                height,
                changed_fraction,
            } => {
                // Interim only: never touch the frame. A preview from an
                // old revision (reordered past the exact update) is
                // dropped — showing it would regress what is on screen.
                if rev > last_ack {
                    surface.lock().preview = Some(PreviewState {
                        rev,
                        x,
                        y,
                        width,
                        height,
                        changed_fraction,
                    });
                }
            }
            Message::Redirect { relay, session } => {
                // Handoff, not failure: the reconnect loop dials the new
                // session with the current resume point.
                info!("Sharer redirected to relay {relay} session {session}");
                surface.lock().redirect = Some((relay, session));
                return Ok(applied_point(&compositor));
            }
            Message::Error(e) => return Err(anyhow::anyhow!("{e}")),
            Message::Bye => {
                info!("Sharer ended the session");
                return Ok(applied_point(&compositor));
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

/// Extract the (epoch, rev) the viewer last applied from a transport
/// failure's context. `receive_once` attaches it as
/// "transport failed at applied {epoch}:{rev}"; anything else (handshake
/// refusal, corrupt frame) carries no resume point and snapshots fresh.
fn resume_point_from_error(
    e: &anyhow::Error,
) -> Option<(crate::network::Epoch, crate::network::Rev)> {
    let chain = format!("{e:#}");
    let marker = "transport failed at applied ";
    let at = chain.find(marker)? + marker.len();
    let rest = chain[at..].split_whitespace().next()?;
    let (epoch, rev) = rest.split_once(':')?;
    let epoch: crate::network::Epoch = epoch.parse().ok()?;
    let rev: crate::network::Rev = rev.parse().ok()?;
    if rev == 0 {
        return None;
    }
    Some((epoch, rev))
}

/// What the compositor holds now, for the next Hello. `None` before the
/// first snapshot commits: there is nothing to resume from.
fn applied_point(c: &Compositor) -> Option<(crate::network::Epoch, crate::network::Rev)> {
    c.is_fresh().then(|| (c.epoch(), c.rev()))
}

fn server_name_of(target: &str) -> &str {
    // The certificate is validated by fingerprint, so the SNI only has to
    // be syntactically valid. The host part is the natural choice.
    match target.rsplit_once(':') {
        Some((host, _port)) if !host.is_empty() => host,
        _ => "pcc",
    }
}

#[cfg(feature = "native-viewer")]
fn run_window_loop(surface: Arc<Mutex<Surface>>) -> Result<()> {
    use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

    info!("Waiting for the first frame to size the window...");
    let first = wait_for_frame(&surface, None)?;
    let (width, height) = (first.width, first.height);

    // HiDPI: a 1512x982@2x share is a 3024x1964 surface, and a window
    // sized in physical pixels overflows a display measured in logical
    // points. `Scale::FitScreen` asks minifb to fit the buffer to the
    // screen instead of opening 1:1 — the pixels stay exact, the
    // window fits. Resize stays on so a geometry change re-fits.
    let mut window = Window::new(
        "PixelChangeCheck Viewer",
        width as usize,
        height as usize,
        WindowOptions {
            resize: true,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .context("Failed to open a window (no display available?)")?;
    window.set_target_fps(60);

    // The window can be resized; the surface is re-read every time its
    // dimensions change, so a sharer that changes geometry does not leave
    // this loop drawing into a stale buffer.
    let mut current = (width, height);
    let mut argb = vec![0u32; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (frame, cursor, terminal) = {
            let s = surface.lock();
            (s.frame.clone(), s.cursor, s.terminal.clone())
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
        // The remote cursor, drawn over the exact frame. Presentation
        // only: the compositor buffer underneath is untouched.
        if cursor.visible {
            let (w, h) = (current.0 as usize, current.1 as usize);
            draw_cursor(&mut argb, w, h, cursor.x as usize, cursor.y as usize);
        }
        window.update_with_buffer(&argb, current.0 as usize, current.1 as usize)?;
    }
    Ok(())
}

/// A 12x16 arrow drawn into an ARGB buffer, clipped at the edges.
/// White fill, black outline: visible on any content, and trivially
/// cheap — a bounding-box walk, no allocation.
#[cfg(feature = "native-viewer")]
fn draw_cursor(argb: &mut [u32], w: usize, h: usize, cx: usize, cy: usize) {
    // (dx, dy, fill?) rows of the arrow shape.
    const ROWS: [&str; 16] = [
        "##..............",
        "###.............",
        "####............",
        "#####...........",
        "######..........",
        "#######.........",
        "########........",
        "#####...........",
        "###.............",
        "####............",
        "##.##...........",
        "...###..........",
        "....###.........",
        ".....###........",
        "......###.......",
        "...............",
    ];
    for (dy, row) in ROWS.iter().enumerate() {
        for (dx, cell) in row.bytes().enumerate() {
            if cell == b'.' {
                continue;
            }
            let (x, y) = (cx + dx, cy + dy);
            if x >= w || y >= h {
                continue;
            }
            argb[y * w + x] = if cell == b'#' { 0x00FFFFFF } else { 0x00000000 };
        }
    }
}

/// Publish one exact frame to the presentation surface, clearing any
/// interim preview it supersedes. The two call sites (snapshot commit and
/// partial update) shared this block verbatim; the helper exists so the
/// `audio`/`native-viewer` feature matrix gates it once, not twice.
fn present_frame(surface: &Arc<Mutex<Surface>>, frame: Frame, rev: Rev) {
    let mut s = surface.lock();
    s.frame = Some(Arc::new(frame));
    // The exact pixels decide: any interim hint at or below this revision
    // is now history.
    if s.preview.is_some_and(|p| p.rev <= rev) {
        s.preview = None;
    }
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

/// Wait for a frame, optionally giving up after a deadline. A session
/// with no Host never produces a frame, so waiting forever is a hang
/// with no diagnostic — the caller names the session, this names the
/// wait.
#[cfg(feature = "native-viewer")]
fn wait_for_frame(
    surface: &Arc<Mutex<Surface>>,
    deadline: Option<Duration>,
) -> Result<std::sync::Arc<Frame>> {
    let start = std::time::Instant::now();
    loop {
        let s = surface.lock();
        if let Some((reason, _)) = &s.terminal {
            anyhow::bail!("{reason}");
        }
        if let Some(frame) = &s.frame {
            return Ok(frame.clone());
        }
        if deadline.is_some_and(|d| start.elapsed() >= d) {
            anyhow::bail!(
                "no frame arrived within {}s — is the sharer still running?",
                deadline.map(|d| d.as_secs()).unwrap_or(0)
            );
        }
        drop(s);
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Split a comma-separated `--relay` list, dropping empties. Same
/// order-in, order-out contract as the sharer's copy: the list order is
/// the probe preference order.
fn split_relays(spec: &str) -> Vec<String> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// The address a viewer is pointed at, resolved once so the headless log
/// and error messages can name something concrete.
pub fn describe(target: &str) -> String {
    match target.parse::<SocketAddr>() {
        Ok(a) => a.to_string(),
        Err(_) => target.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_list_matches_the_sharer_contract() {
        assert_eq!(split_relays("a:1, b:2"), vec!["a:1", "b:2"]);
        assert!(split_relays("").is_empty());
    }

    // The arrow lives behind `native-viewer` with the window loop
    // that is its only caller.
    #[cfg(feature = "native-viewer")]
    #[test]
    fn cursor_arrow_paints_inside_and_clips_at_edges() {
        let (w, h) = (16usize, 16usize);
        let mut argb = vec![0x00123456u32; w * h];
        draw_cursor(&mut argb, w, h, 2, 2);
        assert!(argb.contains(&0x00FFFFFF), "arrow must paint");
        assert!(argb.contains(&0x00123456), "background must survive");
        // Bottom-right corner: must not panic or write out of bounds.
        let mut edge = vec![0u32; w * h];
        draw_cursor(&mut edge, w, h, 15, 15);
        assert_eq!(edge[15 * w + 15], 0x00FFFFFF);
    }

    #[test]
    fn stale_preview_and_cursor_state_behave() {
        // Preview clears when exact pixels at/past it land.
        let mut s = Surface {
            preview: Some(PreviewState {
                rev: 7,
                x: 0,
                y: 0,
                width: 8,
                height: 8,
                changed_fraction: 0.5,
            }),
            ..Default::default()
        };
        if s.preview.is_some_and(|p| p.rev <= 8) {
            s.preview = None;
        }
        assert!(s.preview.is_none());
        // A hide after a move leaves the last position but invisible.
        s.cursor = CursorState {
            x: 3,
            y: 4,
            visible: true,
        };
        s.cursor.visible = false;
        assert!(!s.cursor.visible);
        assert_eq!((s.cursor.x, s.cursor.y), (3, 4));
    }
}
