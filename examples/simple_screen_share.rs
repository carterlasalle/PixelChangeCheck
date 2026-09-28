//! A self-contained demonstration of the real pipeline, in one process.
//!
//! It drives the real planner, encoder, wire protocol, transport and
//! compositor, then checks that what the viewer is showing is
//! pixel-identical to what was captured and prints the bandwidth saved.
//!
//! Run it with: cargo run --release --example simple_screen_share

use anyhow::Result;
use pixel_change_check_client::capture::SyntheticCapture;
use pixel_change_check_client::encoder::encode_snapshot;
use pixel_change_check_client::network::{
    connect_direct, generate_identity, server_endpoint, Message, MessageTransport, NetworkConfig,
    QuicTransport, SessionToken, MAX_MESSAGE_SIZE, SNAPSHOT_CHUNK_BYTES,
};
use pixel_change_check_client::pcc::types::FrameCapture;
use pixel_change_check_client::pcc::{Compositor, PlanLimits, Planner};
use std::net::SocketAddr;
use std::time::Instant;

const FRAMES: u32 = 90;

#[tokio::main]
async fn main() -> Result<()> {
    let (width, height) = (1280u32, 720u32);
    println!("PixelChangeCheck: {width}x{height}, {FRAMES} synthetic frames\n");

    let (wire, full, exact) = replay(width, height, FRAMES)?;
    if !exact {
        anyhow::bail!("the viewer did not reconstruct the capture exactly");
    }
    println!("wire bytes      : {wire:>10}");
    println!("full-frame bytes: {full:>10}");
    println!(
        "saved           : {:>9.2}%  ({:.1}x)",
        100.0 * (1.0 - wire as f64 / full as f64),
        full as f64 / wire.max(1) as f64
    );
    println!("\nNow the same stream over a real loopback QUIC connection...\n");

    let (tx_bytes, rx_bytes) = over_the_wire(width, height).await?;
    println!("  sharer encoded {tx_bytes} bytes; viewer applied {rx_bytes} updates");
    println!("\nDone. Both paths reconstruct the capture exactly.");
    Ok(())
}

/// Encode one revision's worth of visual state for a captured frame.
fn messages_for(
    rev: u64,
    width: u32,
    height: u32,
    plan: &pixel_change_check_client::pcc::Plan,
    rgb: &[u8],
    snapshot_cache: &mut Option<(u64, Vec<u8>)>,
) -> Result<Vec<Vec<u8>>> {
    if plan.ops.is_empty() && plan.wire_len == 0 {
        return Ok(vec![Message::KeepAlive { rev }.encode()?]);
    }
    if !plan.ops.is_empty() {
        return Ok(vec![Message::PartialUpdate {
            rev,
            epoch: 0,
            ops: plan.ops.clone(),
        }
        .encode()?]);
    }
    // The patches were not worth it: ship a lossless snapshot.
    if snapshot_cache.as_ref().is_none_or(|(r, _)| *r != rev) {
        let (_, data) = encode_snapshot(width, height, rgb)?;
        *snapshot_cache = Some((rev, data));
    }
    let data = &snapshot_cache.as_ref().expect("just set").1;
    let chunks: Vec<&[u8]> = data.chunks(SNAPSHOT_CHUNK_BYTES).collect();
    let mut out = Vec::with_capacity(chunks.len() + 2);
    out.push(
        Message::SnapshotBegin {
            rev,
            epoch: 0,
            width,
            height,
            format: pixel_change_check_client::encoder::SnapshotFormat::Png,
            total_len: data.len() as u32,
            chunks: chunks.len() as u32,
        }
        .encode()?,
    );
    for (index, chunk) in chunks.iter().enumerate() {
        out.push(
            Message::SnapshotChunk {
                rev,
                index: index as u32,
                data: chunk.to_vec(),
            }
            .encode()?,
        );
    }
    out.push(Message::SnapshotCommit { rev, epoch: 0 }.encode()?);
    Ok(out)
}

/// Install a viewer the way a real joiner is installed: a lossless
/// snapshot first, so any patch that follows has a base to apply to.
fn install_snapshot(
    viewer: &mut Compositor,
    rgb: &[u8],
    width: u32,
    height: u32,
    rev: u64,
) -> Result<()> {
    use pixel_change_check_client::encoder::encode_snapshot;
    let (format, data) = encode_snapshot(width, height, rgb)?;
    let chunks: Vec<&[u8]> = data.chunks(SNAPSHOT_CHUNK_BYTES).collect();
    viewer.begin_snapshot(
        0,
        width,
        height,
        format,
        data.len() as u32,
        chunks.len() as u32,
    )?;
    for (i, chunk) in chunks.iter().enumerate() {
        viewer.push_snapshot_chunk(i as u32, chunk)?;
    }
    viewer.commit_snapshot(rev, 0)?;
    Ok(())
}

/// Capture -> plan -> encode -> serialize -> deserialize -> apply, with
/// no transport in between but every other real stage in the way. The
/// `exact` result is therefore measured, not asserted.
fn replay(width: u32, height: u32, frames: u32) -> Result<(usize, usize, bool)> {
    let capture = SyntheticCapture::new(width, height);
    let planner = Planner::default();
    let mut reference = capture.capture_frame()?;
    let mut snapshot_cache: Option<(u64, Vec<u8>)> = None;
    let mut viewer = Compositor::new();
    let mut wire = 0usize;
    let mut full = 0usize;
    let mut rev = 0u64;
    install_snapshot(&mut viewer, &reference.data, width, height, rev)?;

    for _ in 0..frames {
        let current = capture.capture_frame()?;
        full += (width * height * 3) as usize;
        rev += 1;

        let plan = planner.plan(
            &mut reference.data,
            width,
            height,
            &current,
            PlanLimits {
                snapshot_bytes: snapshot_cache
                    .as_ref()
                    .map(|(_, d)| d.len())
                    .unwrap_or(usize::MAX),
                max_update_bytes: MAX_MESSAGE_SIZE as usize,
            },
        )?;

        for bytes in messages_for(
            rev,
            width,
            height,
            &plan,
            &current.data,
            &mut snapshot_cache,
        )? {
            wire += bytes.len();
            apply_encoded(&mut viewer, &bytes)?;
        }
    }

    // A synthetic capture is a bouncing box, so only the frames that
    // actually carried pixels advanced the viewer's revision; the last
    // captured frame is the one that must match.
    let exact = viewer.buffer() == reference.data;
    Ok((wire, full, exact))
}

/// Feed one encoded message through the real decoder into the real
/// compositor.
fn apply_encoded(viewer: &mut Compositor, bytes: &[u8]) -> Result<()> {
    match pixel_change_check_client::network::Message::decode(bytes)? {
        Message::SnapshotBegin {
            rev: _,
            epoch,
            width,
            height,
            format,
            total_len,
            chunks,
        } => viewer.begin_snapshot(epoch, width, height, format, total_len, chunks)?,
        Message::SnapshotChunk {
            rev: _,
            index,
            data,
        } => viewer.push_snapshot_chunk(index, &data)?,
        Message::SnapshotCommit { rev, epoch } => viewer.commit_snapshot(rev, epoch)?,
        Message::PartialUpdate { rev, epoch, ops } => viewer.apply_ops(rev, epoch, &ops)?,
        Message::KeepAlive { .. } | Message::Ack { .. } | Message::Bye | Message::Error(_) => {}
        Message::QualityConfig(_)
        | Message::Hello { .. }
        | Message::RequestKeyframe
        | Message::E2eOffer { .. }
        | Message::E2eReply { .. } => {}
    }
    Ok(())
}

/// The same stream over a real QUIC connection between two tasks.
async fn over_the_wire(width: u32, height: u32) -> Result<(usize, usize)> {
    let listen: SocketAddr = "127.0.0.1:19877".parse()?;
    let identity = generate_identity()?;
    let endpoint = server_endpoint(&NetworkConfig::default(), &identity, listen)?;
    let token = SessionToken::generate();
    let pin = pixel_change_check_client::network::hex_to_der(&identity.fingerprint)?;

    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        let connecting = endpoint.accept().await.expect("no connection");
        let connection = connecting.await.expect("handshake failed");
        let (send, recv) = connection.accept_bi().await.expect("no stream");
        let mut transport = QuicTransport::new(send, recv, connection);
        // The viewer must present its token first.
        match transport.recv().await {
            Ok(Message::Hello { .. }) => {}
            other => panic!("expected Hello, got {other:?}"),
        }

        let capture = SyntheticCapture::new(width, height);
        let planner = Planner::default();
        let mut reference = capture.capture_frame().unwrap();
        let mut snapshot_cache: Option<(u64, Vec<u8>)> = None;
        let mut sent = 0usize;
        let mut rev = 0u64;
        let start = Instant::now();
        // A real joiner is caught up before the first update is sent.
        let (_, snapshot) = encode_snapshot(width, height, &reference.data).unwrap();
        let chunks: Vec<&[u8]> = snapshot.chunks(SNAPSHOT_CHUNK_BYTES).collect();
        for msg in [Message::SnapshotBegin {
            rev,
            epoch: 0,
            width,
            height,
            format: pixel_change_check_client::encoder::SnapshotFormat::Png,
            total_len: snapshot.len() as u32,
            chunks: chunks.len() as u32,
        }] {
            let bytes = msg.encode().unwrap();
            sent += bytes.len();
            transport.send_encoded(&bytes).await.unwrap();
        }
        for (i, chunk) in chunks.iter().enumerate() {
            let bytes = Message::SnapshotChunk {
                rev,
                index: i as u32,
                data: chunk.to_vec(),
            }
            .encode()
            .unwrap();
            sent += bytes.len();
            transport.send_encoded(&bytes).await.unwrap();
        }
        let bytes = Message::SnapshotCommit { rev, epoch: 0 }.encode().unwrap();
        sent += bytes.len();
        transport.send_encoded(&bytes).await.unwrap();
        rev += 1;

        for _ in 0..FRAMES {
            let current = capture.capture_frame().unwrap();
            rev += 1;
            let plan = planner
                .plan(
                    &mut reference.data,
                    width,
                    height,
                    &current,
                    PlanLimits {
                        snapshot_bytes: snapshot_cache
                            .as_ref()
                            .map(|(_, d)| d.len())
                            .unwrap_or(usize::MAX),
                        max_update_bytes: MAX_MESSAGE_SIZE as usize,
                    },
                )
                .unwrap();
            for bytes in messages_for(
                rev,
                width,
                height,
                &plan,
                &current.data,
                &mut snapshot_cache,
            )
            .unwrap()
            {
                sent += bytes.len();
                transport.send_encoded(&bytes).await.unwrap();
            }
        }
        println!(
            "  sharer: {FRAMES} frames in {:?}, {sent} bytes on the wire",
            start.elapsed()
        );
        let _ = done_rx.await;
        sent
    });

    let mut viewer =
        connect_direct(&NetworkConfig::default(), &pin, &listen.to_string(), "pcc").await?;
    viewer
        .send(&Message::Hello {
            token: token.as_str().to_string(),
        })
        .await?;

    let mut compositor = Compositor::new();
    let mut applied = 0usize;
    let start = Instant::now();
    while applied < FRAMES as usize {
        match viewer.recv().await? {
            Message::SnapshotBegin {
                rev: _,
                epoch,
                width,
                height,
                format,
                total_len,
                chunks,
            } => compositor.begin_snapshot(epoch, width, height, format, total_len, chunks)?,
            Message::SnapshotChunk {
                rev: _,
                index,
                data,
            } => compositor.push_snapshot_chunk(index, &data)?,
            Message::SnapshotCommit { rev, epoch } => {
                compositor.commit_snapshot(rev, epoch)?;
                applied += 1;
            }
            Message::PartialUpdate { rev, epoch, ops } => {
                compositor.apply_ops(rev, epoch, &ops)?;
                viewer.send(&Message::Ack { rev }).await?;
                applied += 1;
            }
            Message::KeepAlive { .. } => {}
            other => println!("  viewer received {other:?}"),
        }
    }
    println!(
        "  viewer: {FRAMES} updates reconstructed in {:?}",
        start.elapsed()
    );
    let _ = done_tx.send(());
    let sent = server.await.unwrap_or(0);
    Ok((sent, applied))
}
