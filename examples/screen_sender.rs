//! PCC Screen Sender — captures your screen and streams it to a connected viewer.
//!
//! Usage:
//!   cargo run --example screen_sender [-- [port]]
//!
//! Default port: 5800. The sender listens for incoming TCP connections
//! and streams screen data using PCC delta compression.

use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    network::{send_message, ScreenShareMessage},
    pcc::{FrameCapture, PCCDetector, PixelChangeDetector},
};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tracing::{info, Level};

const DEFAULT_PORT: u16 = 5800;
const KEYFRAME_INTERVAL: u64 = 300; // Full resync every ~10s at 30fps

/// Best-effort detection of the local LAN IP address.
fn get_local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // Initialize screen capture
    let capture = ScreenCapture::new()?;
    let width = capture.width();
    let height = capture.height();
    let full_frame_bytes = (width * height * 3) as usize;

    info!("=== PCC Screen Sender ===");
    info!(
        "Screen: {}x{} ({:.1} MB/frame raw)",
        width,
        height,
        full_frame_bytes as f64 / 1_048_576.0
    );

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let local_ip = get_local_ip();
    info!("Listening on 0.0.0.0:{}", port);
    info!("Connect from another device with:");
    info!(
        "  cargo run --example screen_receiver -- {} {}",
        local_ip, port
    );
    info!("Press Ctrl+C to stop.\n");

    loop {
        info!("Waiting for a viewer to connect...");
        let (stream, addr) = listener.accept().await?;
        info!("Viewer connected from {}", addr);

        match stream_to_viewer(stream, &capture, width, height, full_frame_bytes).await {
            Ok(()) => info!("Viewer disconnected normally."),
            Err(e) => info!("Viewer session ended: {}", e),
        }
        info!("Ready for next viewer.\n");
    }
}

async fn stream_to_viewer(
    stream: tokio::net::TcpStream,
    capture: &ScreenCapture,
    width: u32,
    height: u32,
    full_frame_bytes: usize,
) -> Result<()> {
    let (_reader, mut writer) = stream.into_split();
    let detector = PCCDetector::default();

    // Send Hello with screen dimensions
    send_message(&mut writer, &ScreenShareMessage::Hello { width, height }).await?;

    let mut previous_frame = None;
    let mut frame_count = 0u64;
    let mut total_wire_bytes = 0u64;
    let mut total_raw_bytes = 0u64;
    let start = Instant::now();

    loop {
        let frame = capture.capture_frame()?;
        let is_keyframe = previous_frame.is_none() || frame_count % KEYFRAME_INTERVAL == 0;

        if is_keyframe {
            let msg = ScreenShareMessage::Keyframe {
                id: frame.id,
                width: frame.width,
                height: frame.height,
                data: frame.data.clone(),
            };
            let wire = send_message(&mut writer, &msg).await?;
            total_wire_bytes += wire as u64;
            total_raw_bytes += full_frame_bytes as u64;
            info!(
                "Keyframe {}: {:.1} KB on wire (from {:.1} MB raw)",
                frame_count,
                wire as f64 / 1024.0,
                full_frame_bytes as f64 / 1_048_576.0
            );
        } else if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;
            total_raw_bytes += full_frame_bytes as u64;

            if !changes.is_empty() {
                let region_bytes: usize = changes.iter().map(|c| c.data.len()).sum();
                let num_regions = changes.len();
                let msg = ScreenShareMessage::Delta {
                    frame_id: frame.id,
                    changes,
                };
                let wire = send_message(&mut writer, &msg).await?;
                total_wire_bytes += wire as u64;

                // Print stats every ~1 second
                if frame_count % 30 == 0 {
                    let elapsed = start.elapsed().as_secs_f64();
                    let fps = frame_count as f64 / elapsed;
                    let savings = if total_raw_bytes > 0 {
                        100.0 * (1.0 - total_wire_bytes as f64 / total_raw_bytes as f64)
                    } else {
                        0.0
                    };
                    info!(
                        "Frame {}: {} regions, {:.1} KB wire ({:.1} KB regions) | {:.1} FPS | {:.0}% saved",
                        frame_count,
                        num_regions,
                        wire as f64 / 1024.0,
                        region_bytes as f64 / 1024.0,
                        fps,
                        savings
                    );
                }
            } else {
                // No changes detected — frame skipped entirely
            }
        }

        previous_frame = Some(frame);
        frame_count += 1;

        // Target ~30fps
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}
