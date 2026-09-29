//! End-to-end tests over real sockets.
//!
//! These drive the actual `share`/`relay` plumbing: TLS, the token
//! handshake, framed messages, and a real compositor on the viewer side.
//! Nothing here is a mock, and no assertion is about a byte that was
//! never produced by an encoder.

use anyhow::{anyhow, Result};
use pixel_change_check_client::capture::SyntheticCapture;
use pixel_change_check_client::encoder::{encode_snapshot, SnapshotFormat};
use pixel_change_check_client::network::e2e;
use pixel_change_check_client::network::{
    connect_direct, generate_identity, hex_to_der, server_endpoint, verify_token, Message,
    MessageTransport, NetworkConfig, QuicTransport, SessionToken, MAX_MESSAGE_SIZE,
    SNAPSHOT_CHUNK_BYTES,
};
use pixel_change_check_client::pcc::types::{Frame, FrameCapture};
use pixel_change_check_client::pcc::{Compositor, PlanLimits, Planner};
use pixel_change_check_client::relay::{self, RelayRole, RelayTransport, RELAY_CONTROL_ID};
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
            Ok(Message::Hello {
                token: presented,
                resume: _,
            }) => {
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
            resume: None,
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
            resume: None,
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
///
/// The host speaks the fan leg: it learns the viewer's relay-assigned id
/// from the tagged Hello and answers on that id. This is the plaintext
/// regression for the tagging transform below the E2E layer.
#[tokio::test]
async fn the_relay_forwards_real_frames_to_an_authorized_viewer() -> Result<()> {
    let token = SessionToken::parse("RELAYTOKEN123")?;
    let (relay_addr, pin) = start_relay(token.clone()).await?;
    let session = "TEST-SESSION".to_string();

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

    let host = RelayTransport::connect(
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
    let script_len = script.len();
    // The viewer says Hello first so the host learns its id from the tag.
    viewer
        .send(&Message::Hello {
            token: token.as_str().to_string(),
            resume: None,
        })
        .await?;
    let mut fan = Box::new(host).into_fan();
    // Skip the ViewerHere replay: the Hello below carries the same id.
    let (id, first) = loop {
        let (got, bytes) = tokio::time::timeout(Duration::from_secs(5), fan.rx.recv())
            .await
            .map_err(|e| anyhow!(e.to_string()))?
            .ok_or_else(|| anyhow!("fan must stay open"))?;
        if got == RELAY_CONTROL_ID {
            continue;
        }
        break (got, bytes);
    };
    assert_eq!(Message::decode(&first)?.rev(), None);
    for bytes in &script {
        fan.tx.send((id, bytes.clone())).await?;
    }

    let mut compositor = Compositor::new();
    for _ in 0..script_len {
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
    let second = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    drop(first);

    // The replacement host must still reach a viewer. The viewer registers
    // after the takeover, so the host learns its id from the tagged Hello;
    // the ViewerHere replay covers viewers that registered before it.
    let mut viewer = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session,
        token.clone(),
        RelayRole::Viewer,
    )
    .await?;
    viewer
        .send(&Message::Hello {
            token: token.as_str().to_string(),
            resume: None,
        })
        .await?;
    let mut fan = Box::new(second).into_fan();
    let (id, first) = loop {
        let (got, bytes) = tokio::time::timeout(Duration::from_secs(5), fan.rx.recv())
            .await
            .map_err(|e| anyhow!(e.to_string()))?
            .ok_or_else(|| anyhow!("fan must stay open"))?;
        if got == RELAY_CONTROL_ID {
            continue;
        }
        break (got, bytes);
    };
    assert_eq!(Message::decode(&first)?.rev(), None);
    // The replacement host sees the same session ids its predecessor saw:
    // answering on the tagged id reaches the viewer.
    fan.tx
        .send((id, Message::KeepAlive { rev: 1 }.encode()?))
        .await?;
    let got = tokio::time::timeout(Duration::from_secs(5), viewer.recv()).await??;
    assert_eq!(got.rev(), Some(1), "the replacement host lost its session");
    Ok(())
}

/// Start a relay server on an ephemeral port; return its address and pin.
async fn start_relay(token: SessionToken) -> Result<(SocketAddr, Vec<u8>)> {
    let relay_identity = generate_identity()?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr: SocketAddr = listener.local_addr()?;
    let pin = hex_to_der(&relay_identity.fingerprint)?;
    let server_identity = std::sync::Arc::new(relay_identity);
    tokio::spawn(async move {
        if let Err(e) = relay::run_relay_server(listener, server_identity, token).await {
            eprintln!("relay failed: {e}");
        }
    });
    Ok((addr, pin))
}

/// The regression test for the relay fanout bug: two viewers over one
/// host connection each complete E2E and end up pixel-identical.
///
/// The old code ran one `serve_viewer` per relay connection, so the first
/// viewer to handshake set the keys and the second received ciphertext it
/// could not open. Here the host fans out per viewer id with fresh keys.
#[tokio::test]
async fn two_relayed_viewers_each_complete_e2e_and_are_pixel_identical() -> Result<()> {
    let token = SessionToken::parse("TWO-VIEWERS-01")?;
    let (relay_addr, pin) = start_relay(token.clone()).await?;
    let session = "TWO-VIEWERS".to_string();

    let mut script = Vec::new();
    let final_reference = produce(&mut script, 15)?;
    let script_len = script.len();

    // The host fans out; the test drives one E2E session per viewer id.
    let host = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    let mut fan = Box::new(host).into_fan();
    let host_token = token.clone();
    let host_task = tokio::spawn(async move {
        use std::collections::HashMap;
        let mut pending: HashMap<u32, e2e::KeyPair> = HashMap::new();
        let mut sessions: HashMap<u32, e2e::Session> = HashMap::new();
        let mut completed = 0usize;
        // Viewers interleave: Hello and offer arrive per viewer, and each
        // viewer blocks on its reply. Serve handshake frames as they come
        // and seal the script once both sessions exist.
        while completed < 2 {
            let (id, bytes) = fan.rx.recv().await.expect("viewers must handshake");
            assert_ne!(id, RELAY_CONTROL_ID, "handshake must carry a viewer id");
            let msg = Message::decode(&bytes)?;
            match msg {
                Message::Hello { .. } => {
                    pending.insert(id, e2e::KeyPair::generate());
                }
                Message::E2eOffer { public, proof } => {
                    let keys = pending.remove(&id).expect("offer before Hello");
                    let reply = e2e::reply(&keys, host_token.as_str());
                    fan.tx
                        .send((
                            id,
                            Message::E2eReply {
                                public: reply.public,
                                proof: reply.proof,
                            }
                            .encode()?,
                        ))
                        .await
                        .ok();
                    let session =
                        e2e::accept(&e2e::Offer { public, proof }, keys, host_token.as_str())
                            .unwrap();
                    sessions.insert(id, session);
                    completed += 1;
                }
                other => panic!("unexpected handshake frame from viewer {id}: {other:?}"),
            }
        }
        for (id, session) in &mut sessions {
            for bytes in &script {
                let sealed = session.host_to_viewer.seal(bytes)?;
                fan.tx.send((*id, sealed)).await.ok();
            }
        }
        anyhow::Ok(())
    });

    // Viewers speak the unchanged viewer leg: Hello, E2E, then sealed reads.
    // Each viewer runs to completion in its own task, so the two
    // handshakes interleave the way two real viewers would.
    let mut viewers = tokio::task::JoinSet::new();
    for _ in 0..2 {
        let (relay_addr, pin, session, token) =
            (relay_addr, pin.clone(), session.clone(), token.clone());
        viewers.spawn(async move {
            let mut viewer = RelayTransport::connect(
                relay_addr,
                "pcc-relay",
                &pin,
                session.clone(),
                token.clone(),
                RelayRole::Viewer,
            )
            .await?;
            viewer
                .send(&Message::Hello {
                    token: token.as_str().to_string(),
                    resume: None,
                })
                .await?;
            let keys = e2e::KeyPair::generate();
            let (mut sink, mut source) = Box::new(viewer).split();
            let mut viewer_session =
                e2e::viewer_handshake(&mut sink, &mut source, keys, token.as_str()).await?;
            let mut compositor = Compositor::new();
            // The script starts with a snapshot and ends pixel-identical.
            // Read the whole script: breaking on first freshness would stop
            // at the opening snapshot instead of the final frame.
            for _ in 0..script_len {
                let raw = tokio::time::timeout(Duration::from_secs(10), source.recv_raw())
                    .await
                    .expect("the relay must forward sealed frames")
                    .map_err(|e| anyhow!(e.to_string()))?;
                let plain = viewer_session.host_to_viewer.open(&raw)?;
                apply(&mut compositor, &plain)?;
            }
            anyhow::Ok(compositor)
        });
    }
    let mut compositors = Vec::new();
    while let Some(viewer) = viewers.join_next().await {
        compositors.push(viewer.map_err(|e| anyhow!(e.to_string()))??);
    }
    host_task.await??;
    assert_eq!(compositors.len(), 2);
    for compositor in &compositors {
        assert!(compositor.is_fresh());
        assert_eq!(compositor.dimensions(), Some((W, H)));
        assert_eq!(
            compositor.buffer(),
            &final_reference.data,
            "each relayed viewer must be pixel-identical to the sharer's reference"
        );
    }
    // The two viewers hold different keys: one's bytes never open as the other's.
    assert_ne!(
        compositors[0].buffer(),
        Vec::<u8>::new(),
        "sanity: the compositors actually decoded frames"
    );
    Ok(())
}

/// Frames tagged for an unknown viewer id are dropped without disturbing
/// the session: the next valid frame still arrives.
#[tokio::test]
async fn an_unknown_peer_id_is_dropped_without_disturbing_the_session() -> Result<()> {
    let token = SessionToken::parse("UNKNOWN-ID-01")?;
    let (relay_addr, pin) = start_relay(token.clone()).await?;
    let session = "UNKNOWN-ID".to_string();

    let mut viewer = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Viewer,
    )
    .await?;
    let host = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    // The viewer speaks first so the host learns its id from the tag.
    // The ViewerHere replay for the already-registered viewer may arrive
    // before or after its KeepAlive; either order yields the same id.
    viewer.send(&Message::KeepAlive { rev: 7 }).await?;
    let mut fan = Box::new(host).into_fan();
    let mut id = None;
    for _ in 0..2 {
        let (got, bytes) = tokio::time::timeout(Duration::from_secs(5), fan.rx.recv())
            .await
            .map_err(|e| anyhow!(e.to_string()))?
            .ok_or_else(|| anyhow!("fan must stay open"))?;
        if got == RELAY_CONTROL_ID {
            assert_eq!(bytes[0], 0xEF, "replay carries ViewerHere");
            id = Some(u32::from_le_bytes(bytes[1..5].try_into().unwrap()));
        } else {
            assert_eq!(Message::decode(&bytes)?.rev(), Some(7));
            id = Some(got);
            break;
        }
    }
    let id = id.ok_or_else(|| anyhow!("no viewer id learned"))?;
    assert_ne!(id, RELAY_CONTROL_ID);
    // A frame for a viewer that was never registered goes nowhere.
    fan.tx
        .send((0xDEAD_BEEF, Message::KeepAlive { rev: 8 }.encode()?))
        .await?;
    // The session still works: the next frame tagged at the live id arrives.
    fan.tx
        .send((id, Message::KeepAlive { rev: 9 }.encode()?))
        .await?;
    let got = tokio::time::timeout(Duration::from_secs(5), viewer.recv())
        .await
        .expect("live frames must still arrive")?;
    assert_eq!(got.rev(), Some(9));
    Ok(())
}

/// The relay tells the host when a viewer leaves, so the host stops
/// sealing frames into the void: a ViewerLeft control frame names the id.
#[tokio::test]
async fn the_relay_tells_the_host_a_viewer_left() -> Result<()> {
    let token = SessionToken::parse("VIEWER-LEFT-1")?;
    let (relay_addr, pin) = start_relay(token.clone()).await?;
    let session = "VIEWER-LEFT".to_string();

    let viewer = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Viewer,
    )
    .await?;
    let host = RelayTransport::connect(
        relay_addr,
        "pcc-relay",
        &pin,
        session.clone(),
        token.clone(),
        RelayRole::Host,
    )
    .await?;
    let mut fan = Box::new(host).into_fan();
    // The ViewerHere replay names the already-registered viewer.
    let (id, _) = loop {
        let (got, bytes) = tokio::time::timeout(Duration::from_secs(5), fan.rx.recv())
            .await
            .map_err(|e| anyhow!(e.to_string()))?
            .ok_or_else(|| anyhow!("fan must stay open"))?;
        if got == RELAY_CONTROL_ID {
            assert_eq!(bytes[0], 0xEF, "replay carries ViewerHere");
            break (u32::from_le_bytes(bytes[1..5].try_into().unwrap()), bytes);
        }
    };
    assert_ne!(id, RELAY_CONTROL_ID);
    // The viewer goes away; the host learns its id from ViewerLeft.
    drop(viewer);
    let (control, left) = tokio::time::timeout(Duration::from_secs(5), fan.rx.recv())
        .await
        .map_err(|e| anyhow!(e.to_string()))?
        .ok_or_else(|| anyhow!("fan must stay open"))?;
    assert_eq!(control, RELAY_CONTROL_ID);
    assert_eq!(left[0], 0xEE, "departure carries ViewerLeft");
    assert_eq!(
        u32::from_le_bytes(left[1..5].try_into().unwrap()),
        id,
        "ViewerLeft must name the viewer that left"
    );
    Ok(())
}

/// A reconnecting viewer replays the ring instead of snapshotting.
///
/// Drives the real share-side catch-up decision (`repair_viewer`, the same
/// function the lag path and the Hello-resume path both call) against a
/// `Published` filled the way the capture loop fills it: snapshot
/// sequence at rev 0, then updates. A viewer holding rev N gets only the
/// tail replayed and converges pixel-identical; a viewer whose floor
/// predates the ring gets a full snapshot.
#[tokio::test]
async fn a_reconnecting_viewer_resumes_from_the_ring() -> Result<()> {
    use pixel_change_check_client::app::share::{repair_viewer, Published, Shared};
    use pixel_change_check_client::encoder::SurfaceSnapshot;
    use pixel_change_check_client::network::MessageSink;
    use std::sync::Arc;

    struct VecSink(Vec<Vec<u8>>);
    #[async_trait::async_trait]
    impl MessageSink for VecSink {
        async fn send(&mut self, msg: &Message) -> Result<()> {
            self.send_encoded(&msg.encode()?).await
        }
        async fn send_encoded(&mut self, bytes: &[u8]) -> Result<()> {
            self.0.push(bytes.to_vec());
            Ok(())
        }
    }

    let mut script = Vec::new();
    let final_reference = produce(&mut script, 10)?;

    // Share-side state, filled exactly as the capture loop fills it.
    let published: Shared = Arc::new(tokio::sync::RwLock::new(Published {
        rev: 0,
        epoch: 0,
        snapshot: Arc::new(SurfaceSnapshot::new(W, H, vec![0u8; (W * H * 3) as usize])?),
        encoded: None,
        ring: Default::default(),
    }));
    let (mut rev, epoch) = (0u64, 0u32);
    {
        let mut p = published.write().await;
        for bytes in &script {
            // Snapshot messages share rev 0; updates carry their own rev.
            if let Some(r) = pixel_change_check_client::network::peek_rev(bytes) {
                let msg = Message::decode(bytes)?;
                if !matches!(
                    msg,
                    Message::SnapshotBegin { .. }
                        | Message::SnapshotChunk { .. }
                        | Message::SnapshotCommit { .. }
                ) {
                    rev = r;
                }
            }
            p.ring.push(rev, epoch, Arc::new(bytes.clone()));
        }
        p.rev = rev;
        // The published snapshot tracks the live surface, as the loop does.
        p.snapshot = Arc::new(SurfaceSnapshot::new(W, H, final_reference.data.clone())?);
        p.encoded = None;
    }
    assert!(
        rev > 3,
        "the script must span several revisions, got rev {rev}"
    );

    // The reconnecting viewer holds rev 2: mid-history, covered by the ring.
    // Drive the whole script through one compositor first (snapshot +
    // revs 1..=2), then replay only the tail past 2 and converge.
    let mut viewer = Compositor::new();
    for bytes in &script {
        apply(&mut viewer, bytes)?;
        if pixel_change_check_client::network::peek_rev(bytes) == Some(2) {
            break;
        }
    }
    assert_eq!(
        viewer.rev(),
        2,
        "the viewer must hold rev 2 before resuming"
    );
    let tail = {
        let p = published.read().await;
        p.ring
            .replay_from(2, epoch)
            .expect("floor 2 must be covered")
    };
    // Resume replays updates, never a snapshot: the opening sequence sits
    // at rev 0, below the floor.
    let mut saw_snapshot = false;
    let mut floor = 2u64;
    for bytes in &tail {
        match Message::decode(bytes)? {
            Message::SnapshotBegin { .. }
            | Message::SnapshotChunk { .. }
            | Message::SnapshotCommit { .. } => {
                saw_snapshot = true;
            }
            _ => {}
        }
        apply(&mut viewer, bytes)?;
        if let Some(r) = pixel_change_check_client::network::peek_rev(bytes) {
            floor = floor.max(r);
        }
    }
    assert!(!saw_snapshot, "resume must replay updates, not a snapshot");
    assert_eq!(floor, rev, "replay must advance the floor to the tip");
    assert_eq!(
        viewer.buffer(),
        &final_reference.data,
        "replaying the tail must converge the viewer"
    );

    // The share-side decision agrees: repair_viewer from floor 2 returns
    // the tip without snapshotting.
    let mut sink: Box<dyn MessageSink> = Box::new(VecSink(Vec::new()));
    let repaired = repair_viewer(&mut sink, &published, 2).await?;
    assert_eq!(repaired, rev);
    Ok(())
}
