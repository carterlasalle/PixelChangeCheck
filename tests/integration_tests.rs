use anyhow::Result;
use pixel_change_check_client::{
    encoder::FrameEncoder,
    network::{NetworkConfig, ResilienceConfig, NetworkResilience},
    pcc::{PCCDetector, QualityConfig, Frame, PixelChangeDetector},
    server::renderer::FrameBuffer,
};
use std::time::Duration;
use tokio::time;

// Test configurations
const TEST_WIDTH: u32 = 1920;
const TEST_HEIGHT: u32 = 1080;

/// Helper to create a test frame with given id
fn create_test_frame(id: u64) -> Frame {
    Frame {
        id,
        timestamp: std::time::SystemTime::now(),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: vec![0; (TEST_WIDTH * TEST_HEIGHT * 3) as usize],
    }
}

#[tokio::test]
async fn test_pcc_detection_pipeline() -> Result<()> {
    let encoder = FrameEncoder::new(TEST_WIDTH, TEST_HEIGHT, QualityConfig::default())?;
    let detector = PCCDetector::default();

    // Create two frames with some differences
    let frame1 = create_test_frame(1);
    let mut frame2 = create_test_frame(2);
    // Modify some pixels in frame2
    for i in 0..300 {
        frame2.data[i] = 255;
    }

    // Detect changes
    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert!(!changes.is_empty(), "Should detect changes between frames");
    assert!(changes.len() <= (TEST_WIDTH * TEST_HEIGHT) as usize);

    // Encode a frame
    let encoded = encoder.encode_frame(&frame1.data).await?;
    assert!(!encoded.is_empty(), "Encoded frame should not be empty");

    Ok(())
}

#[tokio::test]
async fn test_pcc_no_changes() -> Result<()> {
    let detector = PCCDetector::default();

    let frame1 = create_test_frame(1);
    let frame2 = Frame {
        id: 2,
        timestamp: std::time::SystemTime::now(),
        ..frame1.clone()
    };

    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert!(changes.is_empty(), "Identical frames should have no changes");

    Ok(())
}

#[tokio::test]
async fn test_network_resilience() -> Result<()> {
    let config = ResilienceConfig {
        max_retries: 5,
        retry_delay: Duration::from_millis(50),
        jitter_buffer_size: 5,
        error_correction_enabled: true,
    };

    let resilience = NetworkResilience::new(config);

    // Test retry logic with a counter
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = resilience
        .with_retry(move || {
            let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count < 2 {
                Err(anyhow::anyhow!("Simulated failure"))
            } else {
                Ok(())
            }
        })
        .await;

    assert!(result.is_ok(), "Should succeed after retries");
    assert!(counter.load(std::sync::atomic::Ordering::SeqCst) >= 3);

    // Test health check
    assert!(resilience.is_healthy().await, "Should be healthy after success");

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
    }

    // Verify encoding still works after reconfigure
    let frame = create_test_frame(1);
    let encoded = encoder.encode_frame(&frame.data).await?;
    assert!(!encoded.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_frame_buffer() -> Result<()> {
    let buffer = FrameBuffer::new(TEST_WIDTH, TEST_HEIGHT);

    // Create test frame
    let frame = create_test_frame(1);

    // Test frame management
    buffer.push_frame(frame.clone()).await?;
    let next = buffer.next_frame().await?;
    assert!(next.is_some(), "Should have a frame available");

    let next = next.unwrap();
    assert_eq!(next.id, 1);
    assert_eq!(next.width, TEST_WIDTH);
    assert_eq!(next.height, TEST_HEIGHT);

    // Test updates
    let update = pixel_change_check_client::pcc::PixelChange {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        data: vec![255; 100 * 100 * 3],
    };

    buffer.apply_updates(vec![update]).await?;

    // Verify the current frame was updated
    let current = buffer.current_frame().await;
    assert!(current.is_some(), "Should have a current frame after update");

    Ok(())
}

#[tokio::test]
async fn test_frame_encode_decode() -> Result<()> {
    let frame = create_test_frame(42);

    let encoded = frame.encode()?;
    assert!(!encoded.is_empty());

    let decoded = Frame::decode(&encoded)?;
    assert_eq!(decoded.id, 42);
    assert_eq!(decoded.width, TEST_WIDTH);
    assert_eq!(decoded.height, TEST_HEIGHT);
    assert_eq!(decoded.data.len(), frame.data.len());

    Ok(())
}

#[tokio::test]
async fn test_compression() -> Result<()> {
    let frame_data = vec![0u8; (TEST_WIDTH * TEST_HEIGHT * 3) as usize];

    let compressed =
        pixel_change_check_client::encoder::compression::compress_frame(&frame_data, 0.8)?;
    assert!(
        compressed.len() < frame_data.len(),
        "Compressed data should be smaller"
    );

    let decompressed =
        pixel_change_check_client::encoder::compression::decompress_frame(&compressed)?;
    assert_eq!(
        decompressed, frame_data,
        "Decompressed data should match original"
    );

    Ok(())
}

#[tokio::test]
async fn test_renderer_creation() -> Result<()> {
    let renderer = pixel_change_check_client::server::renderer::Renderer::new(
        TEST_WIDTH, TEST_HEIGHT, 30,
    )
    .await?;

    // Push a frame and render it
    let frame = create_test_frame(1);
    renderer.buffer.push_frame(frame).await?;

    if let Some(buffered) = renderer.buffer.next_frame().await? {
        assert_eq!(buffered.width, TEST_WIDTH);
        assert_eq!(buffered.height, TEST_HEIGHT);
    }

    renderer.shutdown().await?;
    Ok(())
} 