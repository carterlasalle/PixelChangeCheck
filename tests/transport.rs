//! End-to-end tests over real sockets.
//!
//! These drive the actual `share`/`relay` plumbing: TLS, the token
//! handshake, framed messages, and a real compositor on the viewer side.
//! Nothing here is a mock, and no assertion is about a byte that was
//! never produced by an encoder.

use anyhow::Result;
use pixel_change_check_client::capture::SyntheticCapture;
use pixel_change_check_client::encoder::{encode_snapshot, SnapshotFormat};
use pixel_change_check_client::network::{
    connect_direct, generate_identity, hex_to_der, server_endpoint, verify_token, Message,
    MessageTransport, NetworkConfig, QuicTransport, SessionToken, MAX_MESSAGE_SIZE,
    SNAPSHOT_CHUNK_BYTES,
};
use pixel_change_check_client::pcc::types::{Frame, FrameCapture};
use pixel_change_check_client::pcc::{Compositor, PlanLimits, Planner};
use pixel_change_check_client::relay::{self, RelayRole, RelayTransport};
use std::net::SocketAddr;
use std::time::Duration;

const W: u32 = 320;
const H: u32 = 240;

fn capture() -> SyntheticCapture {
    SyntheticCapture::new(W, H)
}

/// Drive the real capture -> plan -> encode path, and hand each encoded
/// message to `sink` as the sharer would.
/// Returns the encoded messages and the sharer's final reference, so a
/// test can assert the viewer is pixel-identical to it rather than merely
/// "a snapshot arrived".
fn produce(sink: &mut Vec<Vec<u8>>, frames: u32) -> Result<Frame> {
    let capture = capture();
    let planner = Planner::default();
    let mut reference = capture.capture_frame()?;
    let mut snapshot_cache: Option<(u64, Vec<u8>)> = None;
    let mut rev = 0u64;

    // A joiner is always caught up first.
    push_snapshot(sink, &reference.data, W, H, rev)?;

    for _ in 0..frames {
        let current = capture.capture_frame()?;
        rev += 1;
        let plan = planner.plan(
            &mut reference.data,
            W,
            H,
            &current,
            PlanLimits {
                snapshot_bytes: snapshot_cache
                    .as_ref()
                    .map(|(_, d)| d.len())
                    .unwrap_or(usize::MAX),
                max_update_bytes: MAX_MESSAGE_SIZE as usize,
            },
        )?;
        if plan.ops.is_empty() && plan.wire_len == 0 {
            sink.push(Message::KeepAlive { rev }.encode()?);
        } else if plan.ops.is_empty() {
            // A snapshot conveys the whole current frame, so the
            // reference becomes it -- exactly as the sharer does.
            reference = current.clone();
            push_snapshot(sink, &current.data, W, H, rev)?;
            let (_, data) = encode_snapshot(W, H, &current.data)?;
            snapshot_cache = Some((rev, data));
        } else {
            sink.push(
                Message::PartialUpdate {
                    rev,
                    pts_us: 0,
                    epoch: 0,
                    ops: plan.ops,
                }
                .encode()?,
            );
        }
    }
    Ok(reference)
}

fn push_snapshot(
    sink: &mut Vec<Vec<u8>>,
    rgb: &[u8],
    width: u32,
    height: u32,
    rev: u64,
) -> Result<()> {
    let (_, data) = encode_snapshot(width, height, rgb)?;
    let chunks: Vec<&[u8]> = data.chunks(SNAPSHOT_CHUNK_BYTES).collect();
    sink.push(
        Message::SnapshotBegin {
            rev,
            pts_us: 0,
            epoch: 0,
            width,
            height,
            format: SnapshotFormat::Png,
            total_len: data.len() as u32,
            chunks: chunks.len() as u32,
        }
        .encode()?,
    );
    for (i, chunk) in chunks.iter().enumerate() {
        sink.push(
            Message::SnapshotChunk {
                rev,
                index: i as u32,
                data: chunk.to_vec(),
            }
            .encode()?,
        );
    }
    sink.push(
        Message::SnapshotCommit {
            rev,
            pts_us: 0,
            epoch: 0,
        }
        .encode()?,
    );
    Ok(())
}

fn apply(viewer: &mut Compositor, bytes: &[u8]) -> Result<()> {
    match Message::decode(bytes)? {
        Message::SnapshotBegin {
            rev: _,
            pts_us: _,
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
        Message::SnapshotCommit {
            rev,
            pts_us: 0,
            epoch,
        } => viewer.commit_snapshot(rev, epoch)?,
        Message::PartialUpdate {
            rev,
            pts_us: 0,
            epoch,
            ops,
        } => viewer.apply_ops(rev, epoch, &ops)?,
        _ => {}
    }
    Ok(())
}

async fn serve_once(
    listener: SocketAddr,
    identity: pixel_change_check_client::network::ServerIdentity,
    token: SessionToken,
    script: Vec<Vec<u8>>,
) {
    let endpoint = server_endpoint(&NetworkConfig::default(), &identity, listener).unwrap();
    tokio::spawn(async move {
        let connecting = endpoint.accept().await.expect("no connection");
        let connection = connecting.await.expect("handshake failed");
        let (send, recv) = connection.accept_bi().await.expect("no stream");
        let mut transport = QuicTransport::new(send, recv, connection);

        // The first thing a viewer sends is its token.
        match transport.recv().await {
            Ok(Message::Hello { token: presented }) => {
                if !verify_token(&token, &presented) {
                    let _ = transport
                        .send(&Message::Error("Unauthorized.".into()))
                        .await;
                    let _ = transport.recv().await;
                    return;
                }
            }
            other => {
                let _ = transport
                    .send(&Message::Error(format!("expected Hello, got {other:?}")))
                    .await;
                return;
            }
        }

        for bytes in script {
            if transport.send_encoded(&bytes).await.is_err() {
                return;
            }
        }
        // Hold the stream open until the viewer acknowledges the last one.
        for _ in 0..20 {
            match transport.recv().await {
                Ok(Message::Ack { .. }) => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });
}

/// A viewer that presents a valid token receives real frames and ends up
/// pixel-identical to the capture.
#[tokio::test]
async fn a_direct_viewer_reconstructs_the_capture_exactly() -> Result<()> {
    let listener: SocketAddr = "127.0.0.1:19811".parse()?;
    let identity = generate_identity()?;
    let token = SessionToken::parse("TESTTOKEN1234")?;
    let mut script = Vec::new();
    let final_reference = produce(&mut script, 25)?;

    let pin = hex_to_der(&identity.fingerprint)?;
    serve_once(listener, identity.clone(), token.clone(), script.clone()).await;

    let mut transport = connect_direct(
        &NetworkConfig::default(),
        &pin,
        &listener.to_string(),
        "pcc",
    )
    .await?;
    transport
        .send(&Message::Hello {
            token: token.as_str().to_string(),
        })
        .await?;

    let mut viewer = Compositor::new();
    let mut seen = 0usize;
    for _ in 0..script.len() {
        let msg = transport.recv().await?;
        apply(&mut viewer, &msg.encode()?)?;
        seen += 1;
    }
    assert_eq!(seen, script.len());

    assert!(
        viewer.is_fresh(),
        "the viewer must have installed a snapshot"
    );
    assert_eq!(viewer.dimensions(), Some((W, H)));
    assert_eq!(
        viewer.buffer(),
        &final_reference.data,
        "the viewer must be pixel-identical to the sharer's reference"
    );
    Ok(())
}

/// An unauthorized viewer gets nothing but an error.
#[tokio::test]
async fn a_viewer_with_the_wrong_token_is_refused() -> Result<()> {
    let listener: SocketAddr = "127.0.0.1:19812".parse()?;
    let identity = generate_identity()?;
    let token = SessionToken::parse("RIGHTTOKEN123")?;
    let pin = hex_to_der(&identity.fingerprint)?;
    serve_once(listener, identity.clone(), token, Vec::new()).await;

    let mut transport = connect_direct(
        &NetworkConfig::default(),
        &pin,
        &listener.to_string(),
        "pcc",
    )
    .await?;
    transport
        .send(&Message::Hello {
            token: "WRONGTOKEN999".into(),
        })
        .await?;

    match transport.recv().await? {
        Message::Error(e) => assert!(e.contains("Unauthorized"), "unhelpful: {e}"),
        other => panic!("expected an error, got {other:?}"),
    }
    Ok(())
}

/// A viewer pinned to the wrong certificate cannot connect at all.
#[tokio::test]
async fn a_wrong_certificate_pin_fails_the_handshake() -> Result<()> {
    let listener: SocketAddr = "127.0.0.1:19813".parse()?;
    let identity = generate_identity()?;
    let other = generate_identity()?;
    let token = SessionToken::parse("PINTOKEN12345")?;
    serve_once(listener, identity, token, Vec::new()).await;

    let result = connect_direct(
        &NetworkConfig::default(),
        &other.certificate,
        &listener.to_string(),
        "pcc",
    )
    .await;
    let err = result
        .err()
        .expect("a mismatched pin must fail")
        .to_string();
    assert!(
        err.contains("handshake") || err.contains("fingerprint"),
        "unhelpful: {err}"
    );
    Ok(())
}

/// The relay path, over TLS, with the token enforced on both ends.
#[tokio::test]
async fn the_relay_forwards_real_frames_to_an_authorized_viewer() -> Result<()> {
    let relay_identity = generate_identity()?;
    let token = SessionToken::parse("RELAYTOKEN123")?;
    let session = "TEST-SESSION".to_string();
    // Bind an ephemeral port, so a leftover process can never make this
    // test fail for the wrong reason.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let relay_addr: SocketAddr = listener.local_addr()?;
    let server_identity = std::sync::Arc::new(relay_identity.clone());
    let server_token = token.clone();
    tokio::spawn(async move {
        if let Err(e) = relay::run_relay_server(listener, server_identity, server_token).await {
            eprintln!("relay failed: {e}");
        }
    });

    let pin = hex_to_der(&relay_identity.fingerprint)?;
    // The viewer registers before the host sends anything: a viewer that
    // joins mid-stream is caught up by a fresh snapshot, which is what the
    // sharer does and what this script starts with.
    let mut viewer = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        SessionToken::parse("RELAYTOKEN123")?,
        RelayRole::Viewer,
    )
    .await?;

    let mut host = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        SessionToken::parse("RELAYTOKEN123")?,
        RelayRole::Host,
    )
    .await?;
    let mut script = Vec::new();
    let final_reference = produce(&mut script, 15)?;
    for bytes in &script {
        host.send_encoded(bytes).await?;
    }

    let mut compositor = Compositor::new();
    for _ in 0..script.len() {
        let msg = tokio::time::timeout(Duration::from_secs(10), viewer.recv())
            .await
            .expect("the relay should forward without stalling")?;
        apply(&mut compositor, &msg.encode()?)?;
    }
    assert!(compositor.is_fresh());
    assert_eq!(compositor.dimensions(), Some((W, H)));
    assert_eq!(
        compositor.buffer(),
        &final_reference.data,
        "a relayed viewer must be pixel-identical to the sharer's reference"
    );
    Ok(())
}

/// The relay refuses a registration whose token is wrong, and says so.
#[tokio::test]
async fn the_relay_refuses_a_wrong_token() -> Result<()> {
    let relay_identity = generate_identity()?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let relay_addr: SocketAddr = listener.local_addr()?;
    let server_identity = std::sync::Arc::new(relay_identity.clone());
    tokio::spawn(async move {
        let _ = relay::run_relay_server(
            listener,
            server_identity,
            SessionToken::parse("GOODTOKEN1234").unwrap(),
        )
        .await;
    });

    let pin = hex_to_der(&relay_identity.fingerprint)?;
    let result = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        "S".into(),
        SessionToken::parse("BADTOKEN12345")?,
        RelayRole::Viewer,
    )
    .await;
    let err = result
        .err()
        .expect("a bad token must be refused")
        .to_string();
    assert!(
        err.contains("rejected") || err.contains("closed"),
        "unhelpful: {err}"
    );
    Ok(())
}

/// A second host taking over a session must not have its registration
/// erased by the first host's cleanup.
#[tokio::test]
async fn a_reconnecting_host_keeps_its_registration() -> Result<()> {
    let relay_identity = generate_identity()?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let relay_addr: SocketAddr = listener.local_addr()?;
    let server_identity = std::sync::Arc::new(relay_identity.clone());
    tokio::spawn(async move {
        let _ = relay::run_relay_server(
            listener,
            server_identity,
            SessionToken::parse("HOSTTOKEN1234").unwrap(),
        )
        .await;
    });

    let pin = hex_to_der(&relay_identity.fingerprint)?;
    let token = SessionToken::parse("HOSTTOKEN1234")?;
    let session = "HANDOVER".to_string();

    let first = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    // The replacement registers before the original disconnects.
    let mut second = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    drop(first);

    // The replacement host must still reach a viewer.
    let mut viewer = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session,
        token,
        RelayRole::Viewer,
    )
    .await?;
    let msg = Message::KeepAlive { rev: 1 };
    second.send(&msg).await?;
    let got = tokio::time::timeout(Duration::from_secs(5), viewer.recv()).await??;
    assert_eq!(got.rev(), Some(1), "the replacement host lost its session");
    Ok(())
}
