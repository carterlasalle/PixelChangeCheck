use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod capture;
mod encoder;
mod network;
mod pcc;
mod server;

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

    // Initialize screen capture
    let capture = match capture::ScreenCapture::new() {
        Ok(c) => {
            info!("Screen capture initialized: {}x{}", c.width(), c.height());
            c
        }
        Err(e) => {
            info!("Screen capture not available (headless?): {}", e);
            return Ok(());
        }
    };

    // Initialize PCC detector
    let detector = pcc::PCCDetector::default();
    info!("PCC detector initialized");

    // Initialize encoder
    let encoder = encoder::FrameEncoder::new(
        capture.width(),
        capture.height(),
        pcc::QualityConfig::default(),
    )?;
    info!("Frame encoder initialized");

    info!("All components initialized successfully.");
    info!("Use the example `simple_screen_share` for a full demo.");

    Ok(())
}
