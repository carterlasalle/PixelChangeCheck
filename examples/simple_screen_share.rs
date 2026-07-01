//! A self-contained local smoke test: runs a sharer and a viewer in the
//! same process, talking over a real (loopback) QUIC connection, and
//! prints out what PCC actually saved you versus sending every frame in
//! full. Works with or without a real display -- it automatically falls
//! back to a synthetic animated test pattern if no screen is available
//! (e.g. in CI or a headless container).
//!
//! Run it with: `cargo run --example simple_screen_share`

use anyhow::Result;
use pixel_change_check_client::capture::CaptureSource;
use pixel_change_check_client::encoder::FrameEncoder;
use pixel_change_check_client::network::{connect_direct, server_endpoint, Message, MessageTransport, NetworkConfig, QuicTransport};
use pixel_change_check_client::pcc::{PCCDetector, PixelChangeDetector, QualityConfig};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time;
use tracing::{info, Level};

const BIND_ADDR: &str = "127.0.0.1:19850";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting PCC local demo (sharer + viewer over real QUIC on {BIND_ADDR})");

    let addr: SocketAddr = BIND_ADDR.parse()?;
    let net_config = NetworkConfig::default();
    let endpoint = server_endpoint(&net_config, addr)?;

    let sharer = tokio::spawn(run_sharer(endpoint));

    // Give the listener a moment to come up.
    time::sleep(Duration::from_millis(50)).await;
    let viewer = tokio::spawn(run_viewer(addr, net_config));

    let (sent_bytes, keyframes, partials, keepalives) = sharer.await??;
    let received_bytes = viewer.await??;

    info!("--- Demo complete ---");
    info!("Sharer sent {} messages: {} keyframes, {} partial updates, {} keep-alives",
        keyframes + partials + keepalives, keyframes, partials, keepalives);
    info!("Total bytes sent by sharer: {sent_bytes}");
    info!("Total bytes received by viewer: {received_bytes}");
    info!("Keep-alives cost almost nothing on the wire -- that's PCC skipping unchanged frames.");

    Ok(())
}

async fn run_sharer(endpoint: quinn::Endpoint) -> Result<(usize, u32, u32, u32)> {
    let capture = CaptureSource::open(false)?; // real screen if available, else synthetic
    let width = capture.width();
    let height = capture.height();
    info!("Sharer capturing {}x{}", width, height);

    let detector = PCCDetector::default();
    let encoder = FrameEncoder::new(width, height, QualityConfig::default())?;

    let connecting = endpoint.accept().await.ok_or_else(|| anyhow::anyhow!("no connection"))?;
    let connection = connecting.await?;
    let (send, recv) = connection.accept_bi().await?;
    let mut transport = QuicTransport::new(send, recv);
    let _ = transport.recv().await; // drain the viewer's handshake kick

    let mut previous_frame = None;
    let mut sent_bytes = 0usize;
    let (mut keyframes, mut partials, mut keepalives) = (0u32, 0u32, 0u32);

    for i in 0..30u64 {
        let frame = capture.capture_frame()?;

        let message = if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;
            if changes.is_empty() {
                keepalives += 1;
                Message::KeepAlive
            } else {
                partials += 1;
                let mut wire_changes = Vec::with_capacity(changes.len());
                for change in &changes {
                    let compressed = pixel_change_check_client::encoder::compression::compress_frame(&change.data, 0.8)?;
                    wire_changes.push(pixel_change_check_client::network::WireChange {
                        x: change.x,
                        y: change.y,
                        width: change.width,
                        height: change.height,
                        compressed_data: compressed,
                    });
                }
                Message::PartialUpdate { frame_id: i, changes: wire_changes }
            }
        } else {
            keyframes += 1;
            let jpeg = encoder.encode_frame(&frame.data).await?;
            Message::FullFrame { frame_id: i, width, height, jpeg_data: jpeg }
        };

        sent_bytes += message.serialize()?.len();
        transport.send(&message).await?;

        previous_frame = Some(frame);
        time::sleep(Duration::from_millis(33)).await;
    }

    transport.send(&Message::Bye).await?;
    // Dropping the connection immediately after sending can race with the
    // peer actually reading the last message; give it a moment to land.
    time::sleep(Duration::from_millis(200)).await;
    Ok((sent_bytes, keyframes, partials, keepalives))
}

async fn run_viewer(addr: SocketAddr, net_config: NetworkConfig) -> Result<usize> {
    let mut transport = connect_direct(&net_config, addr).await?;
    // Kick the QUIC stream open on the wire; see app::view for why.
    transport.send(&Message::KeepAlive).await?;

    let mut received_bytes = 0usize;
    let mut frame_count = 0u32;

    loop {
        let msg = transport.recv().await?;
        received_bytes += msg.serialize()?.len();
        match msg {
            Message::FullFrame { width, height, .. } => {
                frame_count += 1;
                info!("Viewer received keyframe #{frame_count} ({width}x{height})");
            }
            Message::PartialUpdate { changes, .. } => {
                info!("Viewer received partial update: {} changed region(s)", changes.len());
            }
            Message::KeepAlive => {
                info!("Viewer received keep-alive (no pixels changed)");
            }
            Message::Bye => {
                info!("Sharer ended the session");
                break;
            }
            _ => {}
        }
    }

    Ok(received_bytes)
}
