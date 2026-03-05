//! PCC Screen Receiver — connects to a sender and reconstructs the shared screen.
//!
//! Usage:
//!   cargo run --example screen_receiver -- <host> [port]
//!
//! Default port: 5800. Saves periodic snapshots to received_screen.png.

use anyhow::{Context, Result};
use pixel_change_check_client::network::{recv_message, ScreenShareMessage};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tracing::{info, warn, Level};

const DEFAULT_PORT: u16 = 5800;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let host = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: screen_receiver <host> [port]");
        eprintln!("Example: cargo run --example screen_receiver -- 192.168.1.5");
        std::process::exit(1);
    });

    let port: u16 = std::env::args()
        .nth(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    info!("=== PCC Screen Receiver ===");

    let addr = format!("{}:{}", host, port);
    info!("Connecting to {}...", addr);

    let stream = TcpStream::connect(&addr)
        .await
        .with_context(|| format!("Failed to connect to {}", addr))?;

    info!("Connected to {}", addr);
    let (mut reader, _writer) = stream.into_split();

    // Receive Hello
    let msg = recv_message(&mut reader).await?;
    let (width, height) = match msg {
        ScreenShareMessage::Hello { width, height } => (width, height),
        _ => anyhow::bail!("Expected Hello message as first message from sender"),
    };

    let full_frame_bytes = (width * height * 3) as usize;
    info!(
        "Remote screen: {}x{} ({:.1} MB/frame)",
        width,
        height,
        full_frame_bytes as f64 / 1_048_576.0
    );

    let mut frame_data = vec![0u8; full_frame_bytes];
    let mut frame_count = 0u64;
    let mut total_wire_bytes = 0u64;
    let mut last_snapshot = Instant::now();
    let start = Instant::now();

    info!("Receiving frames... (snapshots saved to received_screen.png)");

    loop {
        let msg = match recv_message(&mut reader).await {
            Ok(msg) => msg,
            Err(e) => {
                info!("Connection closed: {}", e);
                break;
            }
        };

        match msg {
            ScreenShareMessage::Keyframe {
                id,
                width: fw,
                height: fh,
                data,
            } => {
                if data.len() != full_frame_bytes {
                    warn!(
                        "Keyframe size mismatch: expected {} bytes, got {} ({}x{} vs {}x{})",
                        full_frame_bytes, data.len(), width, height, fw, fh
                    );
                }
                let copy_len = data.len().min(frame_data.len());
                frame_data[..copy_len].copy_from_slice(&data[..copy_len]);
                total_wire_bytes += data.len() as u64;
                frame_count += 1;
                info!("Keyframe {}: {}x{}", id, fw, fh);
            }
            ScreenShareMessage::Delta {
                frame_id,
                changes,
            } => {
                let num_regions = changes.len();
                let mut delta_bytes = 0usize;

                for change in &changes {
                    for y in 0..change.height {
                        let dst = ((change.y + y) * width + change.x) as usize * 3;
                        let src = (y * change.width) as usize * 3;
                        let len = change.width as usize * 3;

                        if dst + len <= frame_data.len() && src + len <= change.data.len() {
                            frame_data[dst..dst + len]
                                .copy_from_slice(&change.data[src..src + len]);
                        }
                    }
                    delta_bytes += change.data.len();
                }

                total_wire_bytes += delta_bytes as u64;
                frame_count += 1;

                if frame_count % 30 == 0 {
                    let elapsed = start.elapsed().as_secs_f64();
                    let fps = frame_count as f64 / elapsed;
                    info!(
                        "Frame {}: {} regions, {:.1} KB | {:.1} FPS | {:.2} MB total",
                        frame_id,
                        num_regions,
                        delta_bytes as f64 / 1024.0,
                        fps,
                        total_wire_bytes as f64 / 1_048_576.0
                    );
                }
            }
            ScreenShareMessage::Hello { .. } => {
                warn!("Unexpected Hello message mid-stream");
            }
        }

        // Save snapshot every 5 seconds
        if last_snapshot.elapsed() >= Duration::from_secs(5) {
            if let Err(e) = save_snapshot(&frame_data, width, height) {
                warn!("Failed to save snapshot: {}", e);
            }
            last_snapshot = Instant::now();
        }
    }

    // Final snapshot and summary
    let _ = save_snapshot(&frame_data, width, height);
    let elapsed = start.elapsed().as_secs_f64();
    info!("=== Session Summary ===");
    info!(
        "Duration: {:.1}s | Frames: {} | Avg FPS: {:.1}",
        elapsed,
        frame_count,
        if elapsed > 0.0 {
            frame_count as f64 / elapsed
        } else {
            0.0
        }
    );
    info!(
        "Total received: {:.2} MB",
        total_wire_bytes as f64 / 1_048_576.0
    );

    Ok(())
}

fn save_snapshot(data: &[u8], width: u32, height: u32) -> Result<()> {
    use image::{ImageBuffer, Rgb};
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .context("Failed to create image buffer")?;
    img.save("received_screen.png")?;
    info!("Snapshot saved: received_screen.png ({}x{})", width, height);
    Ok(())
}
