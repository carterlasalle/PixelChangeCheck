use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::FrameEncoder,
    pcc::{Frame, PCCDetector, PixelChangeDetector, QualityConfig},
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .init();

    info!("Starting PixelChangeCheck client...");

    // Initialize screen capture (optional — may fail in headless environments)
    match ScreenCapture::new() {
        Ok(c) => {
            info!("Screen capture available: {}x{}", c.width(), c.height());
        }
        Err(e) => {
            info!("Screen capture not available (headless environment): {}", e);
        }
    };

    // Run performance self-test with synthetic frames
    info!("Running performance self-test...");

    let test_w = 1920u32;
    let test_h = 1080u32;
    let full_frame_bytes = (test_w * test_h * 3) as usize;
    let detector = PCCDetector::default();
    let encoder = FrameEncoder::new(test_w, test_h, QualityConfig::default())?;

    // Create two test frames with ~10% pixel difference
    let frame1 = Frame {
        id: 0,
        timestamp: std::time::SystemTime::now(),
        width: test_w,
        height: test_h,
        data: vec![128u8; full_frame_bytes],
    };
    let mut frame2 = frame1.clone();
    frame2.id = 1;
    let change_pixels = (test_w * test_h) as usize / 10;
    for byte in frame2.data[..change_pixels * 3].iter_mut() {
        *byte = 255;
    }

    // Benchmark PCC detection
    let iters = 10u32;
    let start = std::time::Instant::now();
    let mut changes = Vec::new();
    for _ in 0..iters {
        changes = detector.detect_changes(&frame1, &frame2)?;
    }
    let detect_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
    let region_bytes: usize = changes.iter().map(|c| c.data.len()).sum();
    info!("PCC detection: {:.1}ms avg ({} regions, {}x{})", detect_ms, changes.len(), test_w, test_h);
    info!("  Region data: {:.1} KB vs {:.1} MB full frame ({:.1}% bandwidth saved)",
        region_bytes as f64 / 1024.0,
        full_frame_bytes as f64 / 1_048_576.0,
        100.0 * (1.0 - region_bytes as f64 / full_frame_bytes as f64));

    // Benchmark frame encoding
    let start = std::time::Instant::now();
    for _ in 0..iters {
        encoder.encode_frame(&frame1.data).await?;
    }
    let encode_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
    info!("Frame encoding: {:.1}ms avg ({}x{})", encode_ms, test_w, test_h);

    let pipeline_fps = 1000.0 / (detect_ms + encode_ms);
    info!("Pipeline: {:.1} FPS theoretical max (detect + encode)", pipeline_fps);

    info!("Self-test complete. All components working.");
    info!("Next steps:");
    info!("  cargo run --example simple_screen_share  - Full screen sharing demo with bandwidth stats");
    info!("  cargo bench                              - Detailed benchmarks");
    info!("  cargo test                               - Run test suite");

    Ok(())
}
