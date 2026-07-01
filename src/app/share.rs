use crate::capture::CaptureSource;
use crate::encoder::{compression, FrameEncoder};
use crate::network::{server_endpoint, Message, MessageTransport, NetworkConfig, QuicTransport, WireChange};
use crate::pcc::{PCCDetector, PixelChangeDetector, QualityConfig};
use crate::relay::{generate_session_code, RelayRole, RelayTransport};
use crate::server::renderer::{web, Renderer};
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex as TokioMutex};
use tracing::{error, info, warn};

pub struct ShareArgs {
    /// Address to listen on for direct QUIC viewer connections.
    pub listen: Option<SocketAddr>,
    /// Relay server address, for viewers that can't connect directly.
    pub relay: Option<SocketAddr>,
    /// Relay session code. Auto-generated if not given.
    pub session: Option<String>,
    /// Address to bind the browser-friendly MJPEG viewer on.
    pub web: Option<SocketAddr>,
    /// Force the synthetic test pattern instead of capturing a real screen.
    pub synthetic: bool,
    pub fps: u32,
    pub quality: f32,
}

type SharedKeyframe = Arc<TokioMutex<Option<Arc<Message>>>>;

pub async fn run_share(args: ShareArgs) -> Result<()> {
    let capture = CaptureSource::open(args.synthetic)?;
    let width = capture.width();
    let height = capture.height();
    info!("Sharing {}x{}", width, height);

    let detector = PCCDetector::default();
    let target_quality = args.quality.clamp(0.1, 1.0);
    let mut quality_config = QualityConfig {
        target_fps: args.fps,
        max_fps: 60,
        quality: target_quality,
        compression_level: 6,
    };
    let mut encoder = FrameEncoder::new(width, height, quality_config)?;

    let renderer = Arc::new(Renderer::new(width, height, args.fps).await?);

    if let Some(web_addr) = args.web {
        let snapshot = renderer.snapshot_handle();
        let quality = quality_config.quality;
        let fps = args.fps;
        std::thread::spawn(move || {
            if let Err(e) = web::run_web_server(web_addr, snapshot, width, height, quality, fps) {
                error!("Web viewer failed: {e}");
            }
        });
        info!("Browser viewer (works from any device, incl. iPhone Safari): http://{web_addr}/");
    }

    let (tx, _rx) = broadcast::channel::<Arc<Message>>(32);
    let last_keyframe: SharedKeyframe = Arc::new(TokioMutex::new(None));

    if let Some(listen_addr) = args.listen {
        let net_config = NetworkConfig::default();
        let endpoint = server_endpoint(&net_config, listen_addr)?;
        info!("Listening for direct viewer connections on {listen_addr}");
        let tx = tx.clone();
        let last_keyframe = last_keyframe.clone();
        tokio::spawn(async move {
            while let Some(connecting) = endpoint.accept().await {
                match connecting.await {
                    Ok(connection) => {
                        let tx = tx.clone();
                        let last_keyframe = last_keyframe.clone();
                        tokio::spawn(async move {
                            let remote = connection.remote_address();
                            info!("Viewer connected from {remote}");
                            match connection.accept_bi().await {
                                Ok((send, recv)) => {
                                    let transport = QuicTransport::new(send, recv);
                                    if let Err(e) =
                                        forward_to_viewer(transport, tx, last_keyframe).await
                                    {
                                        warn!("Viewer {remote} disconnected: {e}");
                                    }
                                }
                                Err(e) => warn!("Failed to open stream to {remote}: {e}"),
                            }
                        });
                    }
                    Err(e) => warn!("Failed to accept connection: {e}"),
                }
            }
        });
    }

    if let Some(relay_addr) = args.relay {
        let session = args.session.clone().unwrap_or_else(generate_session_code);
        info!("Relay: connecting to {relay_addr}, session code '{session}' (share this with your viewer)");
        let tx = tx.clone();
        let last_keyframe = last_keyframe.clone();
        tokio::spawn(async move {
            loop {
                match RelayTransport::connect(relay_addr, session.clone(), RelayRole::Host).await {
                    Ok(transport) => {
                        info!("Relay connected (session '{session}')");
                        if let Err(e) =
                            forward_to_viewer(transport, tx.clone(), last_keyframe.clone()).await
                        {
                            warn!("Relay connection lost: {e}");
                        }
                    }
                    Err(e) => warn!("Failed to connect to relay: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });
    }

    capture_loop(
        capture,
        detector,
        &mut encoder,
        &mut quality_config,
        target_quality,
        width,
        height,
        args.fps,
        renderer,
        tx,
        last_keyframe,
    )
    .await
}

/// Sends the cached keyframe (if any) to catch a viewer up immediately,
/// then relays every subsequently broadcast message until the transport
/// errors out (peer disconnected).
async fn forward_to_viewer<T: MessageTransport>(
    mut transport: T,
    tx: broadcast::Sender<Arc<Message>>,
    last_keyframe: SharedKeyframe,
) -> Result<()> {
    let mut rx = tx.subscribe();

    if let Some(keyframe) = last_keyframe.lock().await.clone() {
        transport.send(&keyframe).await?;
    }

    loop {
        match rx.recv().await {
            Ok(msg) => transport.send(&msg).await?,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn capture_loop(
    capture: CaptureSource,
    detector: PCCDetector,
    encoder: &mut FrameEncoder,
    quality_config: &mut QualityConfig,
    target_quality: f32,
    width: u32,
    height: u32,
    fps: u32,
    renderer: Arc<Renderer>,
    tx: broadcast::Sender<Arc<Message>>,
    last_keyframe: SharedKeyframe,
) -> Result<()> {
    let frame_interval = Duration::from_secs(1) / fps.max(1);
    // Send a full keyframe periodically so late joiners over direct QUIC
    // (which don't benefit from the relay's cache) still catch up quickly.
    let keyframe_interval = (fps.max(1) as u64) * 5;

    let mut previous_frame: Option<crate::pcc::Frame> = None;
    let mut overrun_streak = 0u32;
    let mut healthy_streak = 0u32;

    loop {
        let loop_start = std::time::Instant::now();
        let frame = capture.capture_frame()?;
        renderer.publish_frame(frame.data.clone()).await;

        let message = if let Some(prev) = &previous_frame {
            let changes = detector.detect_changes(prev, &frame)?;
            if changes.is_empty() {
                Message::KeepAlive
            } else if frame.id % keyframe_interval == 0 {
                let jpeg = encoder.encode_frame(&frame.data).await?;
                Message::FullFrame {
                    frame_id: frame.id,
                    width,
                    height,
                    jpeg_data: jpeg,
                }
            } else {
                let mut wire_changes = Vec::with_capacity(changes.len());
                for change in &changes {
                    let compressed =
                        compression::compress_frame(&change.data, quality_config.quality)?;
                    wire_changes.push(WireChange {
                        x: change.x,
                        y: change.y,
                        width: change.width,
                        height: change.height,
                        compressed_data: compressed,
                    });
                }
                Message::PartialUpdate {
                    frame_id: frame.id,
                    changes: wire_changes,
                }
            }
        } else {
            let jpeg = encoder.encode_frame(&frame.data).await?;
            Message::FullFrame {
                frame_id: frame.id,
                width,
                height,
                jpeg_data: jpeg,
            }
        };

        let msg_arc = Arc::new(message);
        if matches!(*msg_arc, Message::FullFrame { .. }) {
            *last_keyframe.lock().await = Some(msg_arc.clone());
        }
        let _ = tx.send(msg_arc);

        previous_frame = Some(frame);

        let elapsed = loop_start.elapsed();
        if elapsed > frame_interval {
            overrun_streak += 1;
            healthy_streak = 0;
        } else {
            healthy_streak += 1;
            overrun_streak = 0;
        }

        if overrun_streak >= 10 && quality_config.quality > 0.3 {
            quality_config.quality = (quality_config.quality - 0.1).max(0.3);
            encoder.reconfigure(*quality_config).await?;
            let _ = tx.send(Arc::new(Message::QualityConfig(*quality_config)));
            info!(
                "Network/CPU can't keep up; reducing quality to {:.1}",
                quality_config.quality
            );
            overrun_streak = 0;
        } else if healthy_streak >= 150 && quality_config.quality < target_quality {
            quality_config.quality = (quality_config.quality + 0.1).min(target_quality);
            encoder.reconfigure(*quality_config).await?;
            let _ = tx.send(Arc::new(Message::QualityConfig(*quality_config)));
            info!("Conditions improved; raising quality to {:.1}", quality_config.quality);
            healthy_streak = 0;
        }

        if elapsed < frame_interval {
            tokio::time::sleep(frame_interval - elapsed).await;
        }
    }
}
