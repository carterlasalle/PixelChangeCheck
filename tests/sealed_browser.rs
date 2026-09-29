//! The sealed browser path, end to end, over a real socket.
//!
//! This drives what a browser does: the WebSocket upgrade, the
//! WebCrypto-shaped handshake, then sealed frames carrying real surface
//! updates. Doing it in Rust as well as in a browser is deliberate -- CI is
//! headless, and a check that only runs when someone has a display is a
//! check that quietly stops running.
//!
//! The JavaScript side is checked against the same construction by
//! `proof_of_public` and the fixed vector in the smoke script.

use anyhow::{Context, Result};
use pixel_change_check_client::network::web_e2e::{
    complete, offer, BrowserKeyPair, BrowserOffer, BrowserReply, K_BROWSER_OFFER, K_BROWSER_REPLY,
};
use pixel_change_check_client::network::SessionToken;
use pixel_change_check_client::server::renderer::web::run_web_server;
use pixel_change_check_client::server::renderer::SharedSurface;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

const TOKEN: &str = "SEALEDTEST1234";

/// One masked client frame, as a browser sends it.
fn client_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0x80 | opcode];
    let mask = [0xa1u8, 0xb2, 0xc3, 0xd4];
    let n = payload.len();
    if n < 126 {
        out.push(0x80 | n as u8);
    } else if n <= u16::MAX as usize {
        out.push(0x80 | 126);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else {
        out.push(0x80 | 127);
        out.extend_from_slice(&(n as u64).to_be_bytes());
    }
    out.extend_from_slice(&mask);
    for (i, b) in payload.iter().enumerate() {
        out.push(b ^ mask[i % 4]);
    }
    out
}

/// Read one unmasked server frame.
async fn server_frame(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut head = [0u8; 2];
    stream.read_exact(&mut head).await?;
    anyhow::ensure!(head[0] & 0x0F != 0x8, "server closed the WebSocket");
    let mut len = (head[1] & 0x7F) as usize;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).await?;
        len = u16::from_be_bytes(ext) as usize;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).await?;
        len = u64::from_be_bytes(ext) as usize;
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(payload)
}

async fn read_http_head(stream: &mut TcpStream) -> Result<String> {
    let mut seen = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut byte))
            .await
            .context("the server never answered the upgrade")??;
        seen.push(byte[0]);
        if seen.ends_with(b"\r\n\r\n") {
            break;
        }
        anyhow::ensure!(seen.len() < 4096, "HTTP head did not terminate");
    }
    Ok(String::from_utf8_lossy(&seen).into_owned())
}

/// A real update, so the sealed frame has something to carry.
fn an_update() -> Vec<u8> {
    use pixel_change_check_client::network::{Message, WireOp};
    Message::PartialUpdate {
        rev: 7,
        pts_us: 1_234_567,
        epoch: 3,
        ops: vec![WireOp::Rect {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
            compressed: vec![1, 2, 3, 4, 5, 6],
        }],
    }
    .encode()
    .expect("the update encodes")
}

/// Start a real web server on an ephemeral port and return its address.
fn start_sharer(update: Vec<u8>) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("binds an ephemeral port");
    let addr = listener.local_addr().expect("has an address").to_string();
    // The server hands this to `TcpListener::from_std`, which needs it
    // non-blocking; `run_share` sets this before it gets here.
    listener
        .set_nonblocking(true)
        .expect("the listener can go non-blocking");
    let surface = Arc::new(SharedSurface::new(64, 64, 0.8));
    let (_tx, updates) = broadcast::channel(16);
    let token = SessionToken::parse(TOKEN).expect("a valid token");
    let snapshot = Arc::new(move || vec![update.clone()])
        as pixel_change_check_client::server::renderer::web::SnapshotFn;
    std::thread::spawn(move || {
        if let Err(e) = run_web_server(
            listener,
            surface,
            updates,
            token,
            "00".repeat(32),
            None,
            snapshot,
        ) {
            eprintln!("test server stopped: {e:?}");
        }
    });
    addr
}

async fn connect(addr: &str) -> Result<TcpStream> {
    let mut stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr))
        .await
        .expect("the server accepts a connection")
        .map_err(|e| anyhow::anyhow!("could not connect to {addr}: {e}"))?;
    stream
        .write_all(
            format!(
                "GET /ws?token={TOKEN} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\n\
                 Connection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                 Sec-WebSocket-Version: 13\r\n\r\n"
            )
            .as_bytes(),
        )
        .await?;
    stream.flush().await?;
    let head = read_http_head(&mut stream).await?;
    anyhow::ensure!(head.starts_with("HTTP/1.1 101"), "upgrade refused: {head}");
    Ok(stream)
}

#[tokio::test]
async fn a_browser_completes_the_handshake_and_reads_a_sealed_update() -> Result<()> {
    let addr = start_sharer(an_update());
    let mut stream = connect(&addr).await?;

    // Offer.
    let keys = BrowserKeyPair::generate();
    let o = offer(&keys, TOKEN);
    let mut frame = vec![K_BROWSER_OFFER];
    frame.extend_from_slice(&o.public);
    frame.extend_from_slice(&o.proof);
    stream.write_all(&client_frame(0x2, &frame)).await?;
    stream.flush().await?;

    // Reply, and a session built from it -- exactly what the browser does.
    let reply = tokio::time::timeout(Duration::from_secs(5), server_frame(&mut stream))
        .await
        .expect("the sharer must answer a browser offer")?;
    anyhow::ensure!(
        reply[0] == K_BROWSER_REPLY,
        "expected a reply, got kind 0x{:02x}",
        reply[0]
    );
    let mut public = [0u8; 65];
    public.copy_from_slice(&reply[1..66]);
    let mut proof = [0u8; 32];
    proof.copy_from_slice(&reply[66..98]);
    let mut session = complete(
        &keys,
        &BrowserOffer {
            public: o.public,
            proof: o.proof,
        },
        &BrowserReply { public, proof },
        TOKEN,
    )?;

    // The snapshot arrives sealed, and opens back to the exact bytes.
    let sealed = tokio::time::timeout(Duration::from_secs(5), server_frame(&mut stream))
        .await
        .expect("the sharer must send the joining snapshot")?;
    anyhow::ensure!(sealed[0] != 3, "the update must not travel in the clear");
    let opened = session.receive.open(&sealed)?;
    assert_eq!(
        opened,
        an_update(),
        "a sealed frame must reopen to the message"
    );

    Ok(())
}

#[tokio::test]
async fn an_offer_with_a_bad_proof_is_refused() -> Result<()> {
    let addr = start_sharer(an_update());
    let mut stream = connect(&addr).await?;

    let keys = BrowserKeyPair::generate();
    let o = offer(&keys, TOKEN);
    let mut frame = vec![K_BROWSER_OFFER];
    frame.extend_from_slice(&o.public);
    // A wrong proof: the browser cannot prove it holds the token.
    frame.extend_from_slice(&[0u8; 32]);
    stream.write_all(&client_frame(0x2, &frame)).await?;
    stream.flush().await?;

    // The sharer must not answer with a usable session, and must not go on
    // serving frames in the clear.
    if let Ok(Ok(payload)) =
        tokio::time::timeout(Duration::from_secs(3), server_frame(&mut stream)).await
    {
        anyhow::ensure!(
            payload.first() != Some(&K_BROWSER_REPLY),
            "a browser that cannot prove the token must get no session"
        );
    }
    Ok(())
}
