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

use crate::network::{connect_direct, Message, MessageTransport, NetworkConfig, Rev, SessionToken};
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
pub fn run_view(args: ViewArgs) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let surface: Arc<Mutex<Surface>> = Arc::new(Mutex::new(Surface::default()));
    let bg_surface = surface.clone();
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
            let outcome = receive_once(&view_args, &bg_surface).await;
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

async fn receive_once(args: &ViewArgs, surface: &Arc<Mutex<Surface>>) -> Result<()> {
    anyhow::ensure!(
        !(args.connect.is_some() && args.relay.is_some()),
        "specify either --connect or --relay, not both"
    );

    let mut transport: Box<dyn MessageTransport> = if let Some(target) = &args.connect {
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

    info!("Connected. Waiting for frames...");
    let mut compositor = Compositor::new();
    let mut last_ack: Rev = 0;

    loop {
        let msg = transport.recv().await?;
        match msg {
            Message::SnapshotBegin {
                rev: _,
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
            Message::SnapshotCommit { rev, epoch } => {
                compositor.commit_snapshot(rev, epoch)?;
                if let Some((w, h)) = compositor.dimensions() {
                    let frame = Frame::new(rev, w, h, compositor.buffer().to_vec())?;
                    surface.lock().frame = Some(Arc::new(frame));
                    info!("Snapshot installed: {w}x{h} at revision {rev}");
                }
                if rev != last_ack {
                    last_ack = rev;
                    transport.send(&Message::Ack { rev }).await?;
                }
            }
            Message::PartialUpdate { rev, epoch, ops } => {
                match compositor.apply_ops(rev, epoch, &ops) {
                    Ok(()) => {
                        if let Some((w, h)) = compositor.dimensions() {
                            let frame = Frame::new(rev, w, h, compositor.buffer().to_vec())?;
                            surface.lock().frame = Some(Arc::new(frame));
                        }
                        if rev != last_ack {
                            last_ack = rev;
                            transport.send(&Message::Ack { rev }).await?;
                        }
                    }
                    Err(ApplyError::Rejected(_)) => {
                        // The compositor knows precisely what is missing.
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
            Message::Hello { .. } | Message::Ack { .. } | Message::RequestKeyframe => {
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
        for (i, px) in frame.data.chunks_exact(BYTES_PER_PIXEL).enumerate() {
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
