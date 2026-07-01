use anyhow::Result;
use pixel_change_check_client::{
    encoder::FrameEncoder,
    network::{connect_direct, server_endpoint, Message, MessageTransport, NetworkConfig, QuicTransport, ResilienceConfig, NetworkResilience, WireChange},
    pcc::{PCCDetector, QualityConfig, Frame, PixelChangeDetector},
    relay::{self, RelayRole, RelayTransport},
    server::renderer::FrameBuffer,
};
use std::net::SocketAddr;
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

/// Regression test for a bug where the PCC detector treated RGB frame data
/// (3 bytes/pixel) as if it were 1 byte/pixel, corrupting both the
/// comparison and the extracted change coordinates/data for any frame with
/// real color data. A tiny frame makes the expected geometry easy to check
/// by hand.
#[tokio::test]
async fn test_pcc_detector_respects_rgb_stride() -> Result<()> {
    const W: u32 = 8;
    const H: u32 = 8;

    let frame1 = Frame {
        id: 1,
        timestamp: std::time::SystemTime::now(),
        width: W,
        height: H,
        data: vec![0u8; (W * H * 3) as usize],
    };

    // Change exactly one pixel at (3, 2): its green channel.
    let mut frame2 = frame1.clone();
    let idx = (((2 * W) + 3) * 3 + 1) as usize;
    frame2.data[idx] = 200;

    let detector = PCCDetector::default();
    let changes = detector.detect_changes(&frame1, &frame2)?;

    assert_eq!(changes.len(), 1, "exactly one changed pixel should yield one region");
    let change = &changes[0];
    assert_eq!((change.x, change.y), (3, 2), "changed pixel coordinates must be exact");
    assert_eq!((change.width, change.height), (1, 1));
    assert_eq!(change.data.len(), 3, "a 1x1 RGB region is 3 bytes, not 1");
    assert_eq!(change.data, vec![0, 200, 0]);

    Ok(())
}

/// End-to-end test over a real (loopback) QUIC connection: a sharer sends
/// a full keyframe and then a partial update, and a viewer reconstructs
/// the exact same pixels PixelChangeCheck detected, over the actual
/// network stack (not just in-process function calls).
#[tokio::test]
async fn test_direct_quic_end_to_end() -> Result<()> {
    let bind_addr: SocketAddr = "127.0.0.1:19801".parse()?;
    let net_config = NetworkConfig::default();
    let endpoint = server_endpoint(&net_config, bind_addr)?;

    // Dropping a quinn `Connection` tears it down immediately, which can
    // truncate in-flight stream data. Have the viewer explicitly signal
    // once it has read everything before the server side hangs up.
    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        let connecting = endpoint.accept().await.expect("no incoming connection");
        let connection = connecting.await.expect("failed to establish connection");
        let (send, recv) = connection.accept_bi().await.expect("failed to accept stream");
        let mut transport = QuicTransport::new(send, recv);

        // Drain the viewer's handshake kick before sending our frames.
        let _ = transport.recv().await;

        let full_frame = Message::FullFrame {
            frame_id: 1,
            width: 4,
            height: 4,
            jpeg_data: vec![9, 9, 9], // content doesn't matter for transport-layer test
        };
        transport.send(&full_frame).await.expect("failed to send full frame");

        let partial = Message::PartialUpdate {
            frame_id: 2,
            changes: vec![WireChange {
                x: 1,
                y: 1,
                width: 1,
                height: 1,
                compressed_data: vec![7, 7, 7],
            }],
        };
        transport.send(&partial).await.expect("failed to send partial update");

        let _ = done_rx.await;
    });

    let mut viewer = connect_direct(&net_config, bind_addr).await?;
    // Kick the QUIC stream open on the wire (see app::view for why this is needed).
    viewer.send(&Message::KeepAlive).await?;

    let received_full = viewer.recv().await?;
    match received_full {
        Message::FullFrame { frame_id, width, height, jpeg_data } => {
            assert_eq!(frame_id, 1);
            assert_eq!((width, height), (4, 4));
            assert_eq!(jpeg_data, vec![9, 9, 9]);
        }
        other => panic!("expected FullFrame, got {other:?}"),
    }

    let received_partial = viewer.recv().await?;
    match received_partial {
        Message::PartialUpdate { frame_id, changes } => {
            assert_eq!(frame_id, 2);
            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].compressed_data, vec![7, 7, 7]);
        }
        other => panic!("expected PartialUpdate, got {other:?}"),
    }

    let _ = done_tx.send(());
    server_task.await?;
    Ok(())
}

/// End-to-end test of the relay path: a "host" and a "viewer" both dial
/// out to a relay server (simulating two peers that can't reach each
/// other directly) and the relay forwards frames between them.
#[tokio::test]
async fn test_relay_end_to_end() -> Result<()> {
    let relay_addr: SocketAddr = "127.0.0.1:19900".parse()?;
    tokio::spawn(async move {
        let _ = relay::run_relay_server(relay_addr).await;
    });
    // Give the relay a moment to start listening.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut host = RelayTransport::connect(relay_addr, "TEST-SESSION".into(), RelayRole::Host).await?;
    let mut viewer =
        RelayTransport::connect(relay_addr, "TEST-SESSION".into(), RelayRole::Viewer).await?;

    let full_frame = Message::FullFrame {
        frame_id: 1,
        width: 2,
        height: 2,
        jpeg_data: vec![1, 2, 3],
    };
    host.send(&full_frame).await?;

    let received = viewer.recv().await?;
    match received {
        Message::FullFrame { frame_id, jpeg_data, .. } => {
            assert_eq!(frame_id, 1);
            assert_eq!(jpeg_data, vec![1, 2, 3]);
        }
        other => panic!("expected FullFrame, got {other:?}"),
    }

    // A viewer that joins *after* a keyframe was sent should immediately
    // receive that cached keyframe rather than waiting for the next one.
    let mut late_viewer =
        RelayTransport::connect(relay_addr, "TEST-SESSION".into(), RelayRole::Viewer).await?;
    let replayed = late_viewer.recv().await?;
    match replayed {
        Message::FullFrame { frame_id, .. } => assert_eq!(frame_id, 1),
        other => panic!("expected replayed FullFrame, got {other:?}"),
    }

    Ok(())
} 