use anyhow::Result;
use pixel_change_check_client::{
    encoder::FrameEncoder,
    network::{ResilienceConfig, NetworkResilience, ScreenShareMessage, send_message, recv_message},
    pcc::{PCCDetector, QualityConfig, Frame, PixelChangeDetector},
    server::renderer::FrameBuffer,
};
use std::time::Duration;

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
async fn test_pcc_rgb_data_correctness() -> Result<()> {
    let detector = PCCDetector::default();

    // Use a small frame that spans multiple blocks (block_size = 32)
    let w = 64u32;
    let h = 64u32;

    let frame1 = Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: w,
        height: h,
        data: vec![0u8; (w * h * 3) as usize],
    };

    // Modify a single pixel at (10, 5) — set RGB to white
    let mut frame2 = frame1.clone();
    frame2.id = 2;
    let pixel_offset = (5 * w + 10) as usize * 3;
    frame2.data[pixel_offset] = 255;     // R
    frame2.data[pixel_offset + 1] = 255; // G
    frame2.data[pixel_offset + 2] = 255; // B

    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert!(!changes.is_empty(), "Should detect the changed pixel");

    // Should detect exactly one changed region (single pixel in one block)
    assert_eq!(changes.len(), 1, "Should detect exactly one changed region");

    let change = &changes[0];
    // Exact bounds: single pixel at (10, 5)
    assert_eq!(change.x, 10, "Change x should be 10");
    assert_eq!(change.y, 5, "Change y should be 5");
    assert_eq!(change.width, 1, "Change width should be 1 pixel");
    assert_eq!(change.height, 1, "Change height should be 1 pixel");

    // Verify data has correct RGB format (3 bytes per pixel)
    assert_eq!(
        change.data.len(),
        (change.width * change.height * 3) as usize,
        "Change data should have 3 bytes per pixel"
    );
    assert_eq!(change.data, vec![255, 255, 255], "Changed pixel data should be white");

    Ok(())
}

#[tokio::test]
async fn test_pcc_multiblock_changes() -> Result<()> {
    let detector = PCCDetector::default();

    let w = 128u32;
    let h = 64u32;

    let frame1 = Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: w,
        height: h,
        data: vec![0u8; (w * h * 3) as usize],
    };

    // Modify pixels in two separate blocks:
    // Pixel (5, 5) is in block (0, 0)
    // Pixel (40, 5) is in block (32, 0)
    let mut frame2 = frame1.clone();
    frame2.id = 2;

    let px1 = ((5 * w + 5) * 3) as usize;
    frame2.data[px1] = 200;
    frame2.data[px1 + 1] = 200;
    frame2.data[px1 + 2] = 200;

    let px2 = ((5 * w + 40) * 3) as usize;
    frame2.data[px2] = 100;
    frame2.data[px2 + 1] = 100;
    frame2.data[px2 + 2] = 100;

    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert_eq!(changes.len(), 2, "Should detect changes in two separate blocks");

    // Verify each change has correct RGB data size
    for change in &changes {
        assert_eq!(
            change.data.len(),
            (change.width * change.height * 3) as usize,
            "Each change region should have 3 bytes per pixel"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_renderer_creation() -> Result<()> {
    let renderer = pixel_change_check_client::server::renderer::Renderer::new(
        TEST_WIDTH, TEST_HEIGHT, 30,
    )
    .await?;

    // Push a frame with known data and verify rendering
    let mut frame = create_test_frame(1);
    frame.data = vec![42; (TEST_WIDTH * TEST_HEIGHT * 3) as usize];
    renderer.buffer.push_frame(frame).await?;

    if let Some(buffered) = renderer.buffer.next_frame().await? {
        assert_eq!(buffered.width, TEST_WIDTH);
        assert_eq!(buffered.height, TEST_HEIGHT);
    }

    // Verify the current frame was stored correctly
    let current = renderer.buffer.current_frame().await;
    assert!(current.is_some(), "Should have a current frame");
    let current = current.unwrap();
    assert_eq!(current.data[0], 42, "Frame data should match what was pushed");

    renderer.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_pcc_bandwidth_savings() -> Result<()> {
    let detector = PCCDetector::default();

    let w = 1920u32;
    let h = 1080u32;
    let full_frame_bytes = (w * h * 3) as usize;

    let frame1 = Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: w,
        height: h,
        data: vec![0u8; full_frame_bytes],
    };

    // Change ~1% of the frame (a 192x108 region at top-left)
    let mut frame2 = frame1.clone();
    frame2.id = 2;
    let change_region_w = 192u32;
    let change_region_h = 108u32;
    for y in 0..change_region_h {
        for x in 0..change_region_w {
            let offset = ((y * w + x) * 3) as usize;
            frame2.data[offset] = 255;
            frame2.data[offset + 1] = 255;
            frame2.data[offset + 2] = 255;
        }
    }

    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert!(!changes.is_empty(), "Should detect changes");

    // Calculate bandwidth: only the changed region data should be sent
    let region_bytes: usize = changes.iter().map(|c| c.data.len()).sum();

    // The changed region is ~1% of the frame, so region data should be much smaller
    assert!(
        region_bytes < full_frame_bytes / 10,
        "Region data ({} bytes) should be <10% of full frame ({} bytes)",
        region_bytes,
        full_frame_bytes,
    );

    // Verify savings are significant (>90%)
    let savings_pct = 100.0 * (1.0 - region_bytes as f64 / full_frame_bytes as f64);
    assert!(
        savings_pct > 90.0,
        "Bandwidth savings should be >90%, got {:.1}%",
        savings_pct,
    );

    Ok(())
}

#[tokio::test]
async fn test_apply_updates_reconstructs_frame() -> Result<()> {
    let detector = PCCDetector::default();

    let w = 128u32;
    let h = 64u32;
    let buffer = FrameBuffer::new(w, h);

    // Push initial keyframe (all black)
    let frame1 = Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: w,
        height: h,
        data: vec![0u8; (w * h * 3) as usize],
    };
    buffer.push_frame(frame1.clone()).await?;
    buffer.next_frame().await?; // move to current_frame

    // Create frame2 with a white pixel at (10, 5)
    let mut frame2 = frame1.clone();
    frame2.id = 2;
    let pixel_offset = ((5 * w + 10) * 3) as usize;
    frame2.data[pixel_offset] = 255;
    frame2.data[pixel_offset + 1] = 255;
    frame2.data[pixel_offset + 2] = 255;

    // Detect changes and apply as updates
    let changes = detector.detect_changes(&frame1, &frame2)?;
    assert!(!changes.is_empty(), "Should detect changes");

    buffer.apply_updates(changes).await?;

    // Verify the pixel was updated in the buffer's current frame
    let current = buffer.current_frame().await;
    assert!(current.is_some(), "Should have current frame after updates");
    let current = current.unwrap();
    assert_eq!(current.data[pixel_offset], 255, "R channel should be updated");
    assert_eq!(current.data[pixel_offset + 1], 255, "G channel should be updated");
    assert_eq!(current.data[pixel_offset + 2], 255, "B channel should be updated");

    // Verify unchanged pixels remain black
    assert_eq!(current.data[0], 0, "Unchanged pixels should remain 0");
    assert_eq!(current.data[1], 0, "Unchanged pixels should remain 0");
    assert_eq!(current.data[2], 0, "Unchanged pixels should remain 0");

    Ok(())
}

#[tokio::test]
async fn test_screen_share_protocol_roundtrip() -> Result<()> {
    // Test all three message types through send_message/recv_message

    // Create a duplex stream (in-memory bidirectional pipe)
    let (mut client, mut server) = tokio::io::duplex(4 * 1024 * 1024);

    // Test Hello message
    let msg = ScreenShareMessage::Hello {
        width: 1920,
        height: 1080,
    };
    send_message(&mut client, &msg).await?;
    let received = recv_message(&mut server).await?;
    match received {
        ScreenShareMessage::Hello { width, height } => {
            assert_eq!(width, 1920);
            assert_eq!(height, 1080);
        }
        _ => panic!("Expected Hello message"),
    }

    // Test Delta message with pixel changes
    let changes = vec![pixel_change_check_client::pcc::PixelChange {
        x: 10,
        y: 20,
        width: 32,
        height: 32,
        data: vec![255; 32 * 32 * 3],
    }];
    let msg = ScreenShareMessage::Delta {
        frame_id: 42,
        changes,
    };
    send_message(&mut client, &msg).await?;
    let received = recv_message(&mut server).await?;
    match received {
        ScreenShareMessage::Delta {
            frame_id,
            changes,
        } => {
            assert_eq!(frame_id, 42);
            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].x, 10);
            assert_eq!(changes[0].y, 20);
            assert_eq!(changes[0].width, 32);
            assert_eq!(changes[0].height, 32);
            assert_eq!(changes[0].data.len(), 32 * 32 * 3);
        }
        _ => panic!("Expected Delta message"),
    }

    // Test Keyframe message (large payload)
    let data = vec![128u8; 640 * 480 * 3];
    let msg = ScreenShareMessage::Keyframe {
        id: 1,
        width: 640,
        height: 480,
        data: data.clone(),
    };
    send_message(&mut client, &msg).await?;
    let received = recv_message(&mut server).await?;
    match received {
        ScreenShareMessage::Keyframe {
            id,
            width,
            height,
            data: received_data,
        } => {
            assert_eq!(id, 1);
            assert_eq!(width, 640);
            assert_eq!(height, 480);
            assert_eq!(received_data, data);
        }
        _ => panic!("Expected Keyframe message"),
    }

    Ok(())
}