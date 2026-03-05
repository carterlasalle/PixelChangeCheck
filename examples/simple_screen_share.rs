use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::FrameEncoder,
    network::{NetworkConfig, ResilienceConfig},
    pcc::{PCCDetector, QualityConfig, FrameCapture, PixelChangeDetector},
    server::renderer::Renderer,
};
use std::time::Duration;
use tokio::time;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting PCC Screen Share Example");

    // Initialize screen capture
    let capture = ScreenCapture::new()?;
    let width = capture.width();
    let height = capture.height();

    info!("Screen resolution: {}x{}", width, height);

    // Create encoder
    let encoder = FrameEncoder::new(width, height, QualityConfig::default())?;

    // Create PCC detector
    let detector = PCCDetector::default();

    // Set up network config
    let _network_config = NetworkConfig::default();
    let _resilience_config = ResilienceConfig::default();

    // Start renderer (server side)
    info!("Starting renderer...");
    let renderer = Renderer::new(width, height, 30).await?;

    // Main screen sharing loop
    let mut previous_frame = None;
    let mut frame_count = 0u64;
    let start_time = std::time::Instant::now();

    info!("Starting capture loop...");

    loop {
        // Capture frame
        let frame = capture.capture_frame()?;

        // Detect changes
        if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;
            if !changes.is_empty() {
                info!("Detected {} changed regions", changes.len());

                // Encode frame
                let encoded_data = encoder.encode_frame(&frame.data).await?;
                info!(
                    "Encoded frame: {} bytes (ratio: {:.1}:1)",
                    encoded_data.len(),
                    frame.data.len() as f32 / encoded_data.len() as f32
                );

                // Push to renderer buffer (simulating network transfer for localhost)
                renderer.buffer.push_frame(frame.clone()).await?;
            } else {
                info!("No changes detected - skipping frame (PCC optimization)");
            }
        } else {
            // First frame - always send it
            info!("Sending initial frame");
            renderer.buffer.push_frame(frame.clone()).await?;
        }

        previous_frame = Some(frame);
        frame_count += 1;

        // Print statistics and exit after 5 seconds
        let elapsed = start_time.elapsed();
        if elapsed >= Duration::from_secs(5) {
            let fps = frame_count as f32 / elapsed.as_secs_f32();
            info!("Average FPS: {:.2} ({} frames in {:.1}s)", fps, frame_count, elapsed.as_secs_f32());
            break;
        }

        // Maintain target framerate (~30fps)
        time::sleep(Duration::from_millis(33)).await;
    }

    // Clean shutdown
    renderer.shutdown().await?;
    info!("Screen share session ended.");

    Ok(())
} 