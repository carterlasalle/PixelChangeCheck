use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::FrameEncoder,
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
    let full_frame_bytes = (width * height * 3) as usize;

    info!("Capture resolution: {}x{} ({:.1} MB per raw frame)",
        width, height, full_frame_bytes as f64 / 1_048_576.0);

    // Create encoder (for initial keyframe only)
    let encoder = FrameEncoder::new(width, height, QualityConfig::default())?;

    // Create PCC detector
    let detector = PCCDetector::default();

    // Start renderer (server side)
    info!("Starting renderer...");
    let renderer = Renderer::new(width, height, 30).await?;

    // Bandwidth tracking
    let mut previous_frame = None;
    let mut frame_count = 0u64;
    let mut frames_skipped = 0u64;
    let mut total_bytes_without_pcc = 0u64;
    let mut total_bytes_with_pcc = 0u64;
    let start_time = std::time::Instant::now();

    info!("Starting capture loop...");

    loop {
        // Capture frame
        let frame = capture.capture_frame()?;

        // Detect changes
        if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;

            // Without PCC we'd send the full frame every time
            total_bytes_without_pcc += full_frame_bytes as u64;

            if !changes.is_empty() {
                // Calculate actual bytes in changed regions (what PCC sends)
                let region_bytes: usize = changes.iter().map(|c| c.data.len()).sum();
                let changed_pixels: u32 = changes.iter().map(|c| c.width * c.height).sum();
                let total_pixels = width * height;
                let pct_changed = 100.0 * changed_pixels as f64 / total_pixels as f64;
                let pct_saved = 100.0 * (1.0 - region_bytes as f64 / full_frame_bytes as f64);

                total_bytes_with_pcc += region_bytes as u64;

                info!(
                    "Frame {}: {} regions ({:.2}% of screen) — {:.1} KB sent vs {:.1} MB full frame ({:.1}% saved)",
                    frame_count,
                    changes.len(),
                    pct_changed,
                    region_bytes as f64 / 1024.0,
                    full_frame_bytes as f64 / 1_048_576.0,
                    pct_saved,
                );

                // Apply only the changed regions to the renderer (not the full frame)
                renderer.buffer.apply_updates(changes).await?;
            } else {
                info!("Frame {}: No changes — frame skipped entirely (100% saved)", frame_count);
                frames_skipped += 1;
                // With PCC: 0 bytes sent. Without PCC: full frame would still be sent.
            }
        } else {
            // First frame — send as full keyframe
            let encoded = encoder.encode_frame(&frame.data).await?;
            total_bytes_without_pcc += full_frame_bytes as u64;
            total_bytes_with_pcc += encoded.len() as u64;

            info!(
                "Keyframe sent: {:.1} KB JPEG (from {:.1} MB raw, {:.1}:1 compression)",
                encoded.len() as f64 / 1024.0,
                full_frame_bytes as f64 / 1_048_576.0,
                full_frame_bytes as f32 / encoded.len() as f32,
            );

            renderer.buffer.push_frame(frame.clone()).await?;
            // Advance to current_frame so apply_updates works on subsequent frames
            renderer.buffer.next_frame().await?;
        }

        previous_frame = Some(frame);
        frame_count += 1;

        // Print statistics and exit after 5 seconds
        let elapsed = start_time.elapsed();
        if elapsed >= Duration::from_secs(5) {
            let fps = frame_count as f32 / elapsed.as_secs_f32();
            info!("=== PCC Screen Share Summary ===");
            info!("Duration: {:.1}s | Frames: {} | FPS: {:.1}", elapsed.as_secs_f32(), frame_count, fps);
            info!("Frames skipped (no change): {}/{} ({:.0}%)",
                frames_skipped, frame_count,
                100.0 * frames_skipped as f64 / frame_count as f64);
            info!("Without PCC: {:.2} MB would have been sent",
                total_bytes_without_pcc as f64 / 1_048_576.0);
            info!("With PCC:    {:.2} MB actually sent",
                total_bytes_with_pcc as f64 / 1_048_576.0);
            if total_bytes_without_pcc > 0 {
                info!("Bandwidth saved: {:.1}%",
                    100.0 * (1.0 - total_bytes_with_pcc as f64 / total_bytes_without_pcc as f64));
            }
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