use anyhow::Result;
use std::sync::Arc;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::FrameEncoder,
    network::{NetworkConfig, QUICTransport, ResilienceConfig},
    pcc::{PCCDetector, QualityConfig},
    server::renderer::Renderer,
};
use std::time::Duration;
use tokio::time;
use tracing::{info, Level};
use rustls;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting PCC Screen Share Example");

    // Initialize components
    let capture = ScreenCapture::new()?;
    
    // Get primary display info using available API
    let display_info = display_info::DisplayInfo::from_point(0, 0)?;
    let width = display_info.width();
    let height = display_info.height();

    info!("Screen resolution: {}x{}", width, height);

    // Create encoder
    let encoder = FrameEncoder::new(
        width,
        height,
        QualityConfig::default(),
    )?;

    // Create PCC detector
    let detector = PCCDetector::default();

    // Set up network transport
    let network_config = NetworkConfig::default();
    let resilience_config = ResilienceConfig::default();
    
    // Start server first
    info!("Starting server...");
    let renderer = Renderer::new(width, height, 30).await?;
    
    // Create client transport
    info!("Starting client...");
    let mut transport = QUICTransport::new(
        create_client_endpoint().await?,
        network_config,
    );

    // Connect to server
    transport.connect().await?;
    info!("Connected to server");

    // Main screen sharing loop
    let mut previous_frame = None;
    let mut frame_count = 0;
    let start_time = std::time::Instant::now();

    loop {
        // Capture frame
        let frame = capture.capture_frame()?;
        
        // Detect changes
        if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;
            if !changes.is_empty() {
                info!("Detected {} changed regions", changes.len());
                
                // Encode and send changes
                let encoded = encoder.encode_frame(&frame.data).await?;
                transport.send_frame(&encoded).await?;
            }
        }
        
        previous_frame = Some(frame);
        frame_count += 1;

        // Print statistics every 5 seconds
        if start_time.elapsed() >= Duration::from_secs(5) {
            let fps = frame_count as f32 / 5.0;
            info!("Average FPS: {:.2}", fps);
            break;
        }

        // Maintain target framerate
        time::sleep(Duration::from_millis(33)).await; // ~30fps
    }

    Ok(())
}

async fn create_client_endpoint() -> Result<quinn::Endpoint> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth();
        
    let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
    let mut transport_config = quinn::TransportConfig::default();
    client_config.transport_config(Arc::new(transport_config));

    let addr = "0.0.0.0:0".parse()?;
    let endpoint = quinn::Endpoint::client(addr)?;
    endpoint.set_default_client_config(client_config);

    Ok(endpoint)
} 