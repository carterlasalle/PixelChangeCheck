use anyhow::Result;
use pixel_change_check_client::{
    capture::ScreenCapture,
    encoder::FrameEncoder,
    network::{NetworkConfig, QUICTransport, ResilienceConfig},
    pcc::{PCCDetector, QualityConfig},
    server::renderer::Renderer,
};
use std::time::Duration;
use tokio::time;

// Test configurations
const TEST_WIDTH: u32 = 1920;
const TEST_HEIGHT: u32 = 1080;
const TEST_FPS: u32 = 30;

#[tokio::test]
async fn test_full_pipeline() -> Result<()> {
    // Initialize components
    let capture = ScreenCapture::new()?;
    let encoder = FrameEncoder::new(TEST_WIDTH, TEST_HEIGHT, QualityConfig::default())?;
    let detector = PCCDetector::default();
    
    // Set up network
    let network_config = NetworkConfig::default();
    let resilience_config = ResilienceConfig::default();
    let transport = QUICTransport::new(
        create_test_endpoint().await?,
        network_config,
    );
    
    // Set up renderer
    let renderer = Renderer::new(TEST_WIDTH, TEST_HEIGHT, TEST_FPS).await?;
    
    // Capture and process a few frames
    let mut previous_frame = None;
    for _ in 0..3 {
        // Capture frame
        let frame = capture.capture_frame()?;
        
        // Detect changes
        if let Some(prev) = previous_frame {
            let changes = detector.detect_changes(&prev, &frame)?;
            assert!(changes.len() <= (TEST_WIDTH * TEST_HEIGHT) as usize);
        }
        
        // Encode frame
        let encoded = encoder.encode_frame(&frame.into()).await?;
        assert!(!encoded.is_empty());
        
        previous_frame = Some(frame);
        time::sleep(Duration::from_millis(33)).await; // ~30fps
    }
    
    Ok(())
}

#[tokio::test]
async fn test_network_resilience() -> Result<()> {
    let config = ResilienceConfig {
        max_retries: 3,
        base_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(1),
        connection_timeout: Duration::from_secs(5),
    };
    
    let resilience = pixel_change_check_client::network::NetworkResilience::new(config);
    
    // Test retry logic
    let mut fail_count = 0;
    let result = resilience.with_retry(|| {
        fail_count += 1;
        if fail_count < 3 {
            Err(anyhow::anyhow!("Simulated failure"))
        } else {
            Ok(())
        }
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(fail_count, 3);
    
    Ok(())
}

#[tokio::test]
async fn test_quality_adaptation() -> Result<()> {
    let mut encoder = FrameEncoder::new(TEST_WIDTH, TEST_HEIGHT, QualityConfig::default())?;
    
    // Test quality adjustment
    let configs = [
        QualityConfig {
            target_fps: 30,
            max_fps: 60,
            quality: 0.8,
            compression_level: 6,
        },
        QualityConfig {
            target_fps: 15,
            max_fps: 30,
            quality: 0.5,
            compression_level: 8,
        },
    ];
    
    for config in configs.iter() {
        encoder.reconfigure(*config).await?;
        // Verify configuration was applied
        // (would need getter methods to properly test)
    }
    
    Ok(())
}

#[tokio::test]
async fn test_frame_buffer() -> Result<()> {
    let buffer = pixel_change_check_client::server::renderer::FrameBuffer::new(
        TEST_WIDTH,
        TEST_HEIGHT,
    );
    
    // Create test frame
    let frame = pixel_change_check_client::pcc::Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: vec![0; (TEST_WIDTH * TEST_HEIGHT * 3) as usize],
    };
    
    // Test frame management
    buffer.push_frame(frame.clone()).await?;
    let next = buffer.next_frame().await?;
    assert!(next.is_some());
    
    // Test updates
    let update = pixel_change_check_client::pcc::PixelChange {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        data: vec![255; 100 * 100 * 3],
    };
    
    buffer.apply_updates(vec![update]).await?;
    
    Ok(())
}

// Helper function to create test endpoint
async fn create_test_endpoint() -> Result<quinn::Endpoint> {
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_idle_timeout(Some(Duration::from_secs(10).try_into()?));
    
    let mut server_config = quinn::ServerConfig::default();
    server_config.transport = Arc::new(transport_config);
    
    let addr = "127.0.0.1:0".parse()?;
    let endpoint = quinn::Endpoint::server(server_config, addr)?;
    
    Ok(endpoint)
} 