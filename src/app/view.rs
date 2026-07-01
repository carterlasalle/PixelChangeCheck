use crate::encoder::decode_jpeg;
use crate::network::{connect_direct, Message, MessageTransport, NetworkConfig, WireChange};
use crate::relay::{RelayRole, RelayTransport};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tracing::{info, warn};

pub struct ViewArgs {
    /// Connect directly to a sharer listening at this address.
    pub connect: Option<SocketAddr>,
    /// Connect through a relay instead (for NAT'd / internet peers).
    pub relay: Option<SocketAddr>,
    /// Relay session code, required when `relay` is set.
    pub session: Option<String>,
    /// Try to open a native window. Falls back to a headless status loop
    /// if no display is available (e.g. this is a headless server).
    pub show_window: bool,
}

type SharedDims = Arc<StdMutex<Option<(u32, u32)>>>;
type SharedFrame = Arc<StdMutex<Vec<u8>>>;

/// Entry point for `pcc view`. Runs the async network receive loop on a
/// background tokio task, and (optionally) the native window's event loop
/// on the calling thread, since GUI toolkits generally require the main
/// thread on macOS.
pub fn run_view(args: ViewArgs) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let dims: SharedDims = Arc::new(StdMutex::new(None));
    let snapshot: SharedFrame = Arc::new(StdMutex::new(Vec::new()));

    let dims_bg = dims.clone();
    let snapshot_bg = snapshot.clone();
    let show_window = args.show_window;
    rt.spawn(async move {
        if let Err(e) = receive_loop(args, dims_bg, snapshot_bg).await {
            warn!("Viewer session ended: {e}");
        }
    });

    if show_window {
        if let Err(e) = run_window_loop(dims.clone(), snapshot.clone()) {
            warn!("Falling back to headless mode (no window available): {e}");
            run_headless_loop(dims, snapshot);
        }
    } else {
        run_headless_loop(dims, snapshot);
    }

    Ok(())
}

async fn receive_loop(args: ViewArgs, dims: SharedDims, snapshot: SharedFrame) -> Result<()> {
    let mut transport: Box<dyn MessageTransport> = if let Some(addr) = args.connect {
        info!("Connecting directly to {addr}");
        let mut transport = connect_direct(&NetworkConfig::default(), addr).await?;
        // QUIC streams aren't observed by the remote peer until data
        // actually flows on them, so the sharer's `accept_bi()` would
        // otherwise never return: kick the stream open immediately.
        transport.send(&Message::KeepAlive).await?;
        Box::new(transport)
    } else if let Some(relay_addr) = args.relay {
        let session = args
            .session
            .clone()
            .context("--session <CODE> is required when using --relay")?;
        info!("Connecting via relay {relay_addr}, session '{session}'");
        Box::new(RelayTransport::connect(relay_addr, session, RelayRole::Viewer).await?)
    } else {
        anyhow::bail!("Specify either --connect <addr> or --relay <addr> --session <code>");
    };

    info!("Connected. Waiting for frames...");

    let mut recon: Vec<u8> = Vec::new();
    let mut width = 0u32;

    loop {
        let msg = transport.recv().await?;
        match msg {
            Message::FullFrame {
                width: w,
                height: h,
                jpeg_data,
                ..
            } => {
                let (dw, dh, rgb) = decode_jpeg(&jpeg_data)?;
                width = if dw > 0 { dw } else { w };
                let height = if dh > 0 { dh } else { h };
                recon = rgb;
                *dims.lock().unwrap() = Some((width, height));
                *snapshot.lock().unwrap() = recon.clone();
            }
            Message::PartialUpdate { changes, .. } => {
                if recon.is_empty() || width == 0 {
                    continue; // haven't received a keyframe yet
                }
                apply_wire_changes(&mut recon, width, &changes)?;
                *snapshot.lock().unwrap() = recon.clone();
            }
            Message::KeepAlive => {}
            Message::QualityConfig(cfg) => {
                info!(
                    "Sharer adjusted quality: fps={} quality={:.1}",
                    cfg.target_fps, cfg.quality
                );
            }
            Message::Error(e) => warn!("Sharer reported an error: {e}"),
            Message::Bye => {
                info!("Sharer ended the session");
                break;
            }
        }
    }

    Ok(())
}

fn apply_wire_changes(recon: &mut [u8], width: u32, changes: &[WireChange]) -> Result<()> {
    for change in changes {
        let data = crate::encoder::compression::decompress_frame(&change.compressed_data)?;
        let row_bytes = (change.width * 3) as usize;
        for y in 0..change.height {
            let frame_offset = (((change.y + y) * width + change.x) * 3) as usize;
            let update_offset = (y as usize) * row_bytes;
            if frame_offset + row_bytes > recon.len() || update_offset + row_bytes > data.len() {
                continue; // defensively skip malformed/out-of-bounds updates
            }
            recon[frame_offset..frame_offset + row_bytes]
                .copy_from_slice(&data[update_offset..update_offset + row_bytes]);
        }
    }
    Ok(())
}

fn run_window_loop(dims: SharedDims, snapshot: SharedFrame) -> Result<()> {
    use minifb::{Key, Window, WindowOptions};

    info!("Waiting for the first frame to size the window...");
    let (width, height) = loop {
        if let Some(d) = *dims.lock().unwrap() {
            break d;
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    let mut window = Window::new(
        "PixelChangeCheck Viewer",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .context("Failed to open a window (no display available?)")?;

    window.set_target_fps(60);

    let mut argb = vec![0u32; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            let rgb = snapshot.lock().unwrap();
            if rgb.len() == (width * height * 3) as usize {
                for (i, px) in rgb.chunks_exact(3).enumerate() {
                    argb[i] = ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32;
                }
            }
        }
        window.update_with_buffer(&argb, width as usize, height as usize)?;
    }

    Ok(())
}

fn run_headless_loop(dims: SharedDims, snapshot: SharedFrame) {
    info!("Running headless (no window). Press Ctrl+C to quit.");
    loop {
        std::thread::sleep(Duration::from_secs(2));
        if let Some((w, h)) = *dims.lock().unwrap() {
            let bytes = snapshot.lock().unwrap().len();
            info!("Receiving {}x{} frames ({} bytes buffered)", w, h, bytes);
        }
    }
}
