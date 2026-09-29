//! The browser viewer.
//!
//! Two paths, and the difference matters:
//!
//! * `/ws` -- the real one. A WebSocket carrying the *same* framed
//!   `Message` bytes the native viewer gets, and a JavaScript compositor
//!   that is a direct port of `pcc::compositor`. A browser therefore gets
//!   the actual PCC stream: patches, fills, verified copies, and a
//!   lossless snapshot. The only thing it needs from the browser is
//!   `createImageBitmap` to decode the PNG snapshot.
//! * `/stream` -- an explicitly *lossy* MJPEG preview for anything that
//!   cannot run a compositor, such as a bare `<img>` tag. It costs a full
//!   JPEG per frame and is labelled as lossy in the page itself.
//!
//! Access control is the session token, on every path. Supplying a real
//! certificate with `--web-cert`/`--web-key` turns both paths into
//! HTTPS/WSS. Without one only loopback serves (the sharer refuses a
//! non-loopback --web address), which is why the token is mandatory
//! rather than optional here.

use crate::network::SessionToken;
use crate::server::renderer::SharedSurface;
use anyhow::{Context, Result};
use base64::Engine;
use sha1::{Digest, Sha1};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Semaphore};
use tracing::{info, warn};

const BOUNDARY: &str = "pccframe";
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Concurrent stream handlers allowed at once.
///
/// Receipt: tiny_http-style servers spawn a task per connection, so the
/// bound is what stops an open tab from becoming an unbounded thread count.
/// 16 concurrent streams is far above any realistic viewer count, so
/// ordinary use never sees the limit.
const MAX_STREAMS: usize = 16;

/// Largest HTTP request head we will read before giving up.
const MAX_REQUEST_HEAD: usize = 8 * 1024;

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>PixelChangeCheck</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body { margin: 0; background: #111; display: flex; align-items: center; justify-content: center; height: 100vh; }
    canvas, img { max-width: 100%; max-height: 100%; }
    #status { position: fixed; top: 8px; left: 8px; color: #9aa; font: 12px monospace; }
  </style>
</head>
<body>
  <div id="status">connecting…</div>
  <canvas id="screen"></canvas>
  <img id="fallback" hidden />
  <script src="/pcc.js"></script>
  <script>
    // The secret is in the fragment. Everything after '#' is handled by
    // the browser and never sent in a request, so it stays out of server
    // logs, proxy logs, browser history and referrers.
    const fragment = new URLSearchParams(location.hash.replace(/^#/, ''));
    const token = fragment.get('token');
    if (!token) {
      document.getElementById('status').textContent =
        'no token -- open the exact link your sharer printed';
    } else {
      // Do not leave the secret sitting in the address bar.
      history.replaceState(null, '', location.pathname + location.search);
      window.pcc.connect(token);
    }
  </script>
</body>
</html>"#;

const FALLBACK_HTML: &str = r#"<!DOCTYPE html>
<html><head><title>PixelChangeCheck (lossy preview)</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>body{margin:0;background:#111;display:flex;align-items:center;justify-content:center;height:100vh}img{max-width:100%;max-height:100%}</style>
</head><body><img src="stream" alt="live screen share (lossy JPEG preview)"></body></html>"#;

const CLIENT_JS: &str = include_str!("client.js");

/// Produces the encoded message sequence that brings a receiver up to
/// date, from whatever the sharer currently believes viewers hold.
///
/// The sharer supplies this because only it owns the authoritative
/// surface. Without it a browser that connects mid-session would get
/// patches against a base it never had, which is the stale-join bug in a
/// different costume.
pub type SnapshotFn = Arc<dyn Fn() -> Vec<Vec<u8>> + Send + Sync>;

/// Serve the browser viewer. Blocks the calling thread, so run it on its
/// own OS thread.
pub fn run_web_server(
    listener: std::net::TcpListener,
    surface: Arc<SharedSurface>,
    updates: broadcast::Receiver<Arc<Vec<u8>>>,
    token: SessionToken,
    fingerprint: String,
    tls: Option<Arc<rustls::ServerConfig>>,
    snapshot: SnapshotFn,
) -> Result<()> {
    let gate = Arc::new(Semaphore::new(MAX_STREAMS));
    let live = Arc::new(AtomicUsize::new(0));
    let acceptor = tls.clone().map(tokio_rustls::TlsAcceptor::from);
    info!(
        "Browser viewer ready{} (certificate {})",
        if tls.is_some() {
            " over TLS"
        } else {
            " over plaintext HTTP"
        },
        fingerprint
    );

    // The conversion has to happen inside the runtime: registering a
    // socket with the reactor is what makes it pollable.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let listener = match TcpListener::from_std(listener) {
            Ok(l) => l,
            Err(e) => {
                warn!("Web listener could not start: {e}");
                return;
            }
        };
        loop {
            let (tcp, peer) = match listener.accept().await {
                Ok(pair) => pair,
                Err(e) => {
                    warn!("Web listener error: {e}");
                    return;
                }
            };
            let permit = match gate.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    warn!("Refusing {peer}: {MAX_STREAMS} streams already open");
                    continue;
                }
            };
            let live_count = live.clone();
            live_count.fetch_add(1, Ordering::Relaxed);
            let surface = surface.clone();
            let token = token.clone();
            let acceptor = acceptor.clone();
            let updates = updates.resubscribe();
            let snapshot = snapshot.clone();
            tokio::spawn(async move {
                let result = match acceptor {
                    Some(acceptor) => match acceptor.accept(tcp).await {
                        Ok(stream) => {
                            handle(&surface, &updates, &token, &snapshot, stream, Some(peer)).await
                        }
                        Err(e) => Err(e.into()),
                    },
                    None => handle(&surface, &updates, &token, &snapshot, tcp, Some(peer)).await,
                };
                if let Err(e) = result {
                    warn!("Web client {peer}: {e}");
                }
                live_count.fetch_sub(1, Ordering::Relaxed);
                drop(permit);
            });
        }
    });
    Ok(())
}

/// A byte stream, TLS or not, handled identically.
trait ByteStream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> ByteStream for T {}

async fn handle<S: ByteStream>(
    surface: &SharedSurface,
    _updates: &broadcast::Receiver<Arc<Vec<u8>>>,
    token: &SessionToken,
    snapshot: &SnapshotFn,
    mut stream: S,
    peer: Option<SocketAddr>,
) -> Result<()> {
    let head = read_head(&mut stream).await?;
    let path = head.path.clone();
    let presented = head.query.get("token").map(|s| s.as_str()).unwrap_or("");
    let authorized = crate::network::verify_token(token, presented);

    // The page and the script carry no secret and no screen data, so they
    // are served to anyone who asks. The token now travels in the URL
    // fragment, which the browser never puts in a request, so the page
    // load cannot authenticate -- and must not pretend to. The WebSocket
    // handshake is where the token is checked, before a single byte of
    // surface data can move.
    if path == "/pcc.js" {
        return respond(
            &mut stream,
            200,
            "application/javascript; charset=utf-8",
            CLIENT_JS.as_bytes(),
        )
        .await;
    }
    if matches!(path.as_str(), "/" | "/index.html") {
        return respond(
            &mut stream,
            200,
            "text/html; charset=utf-8",
            INDEX_HTML.as_bytes(),
        )
        .await;
    }
    if !authorized {
        return respond(
            &mut stream,
            401,
            "text/plain; charset=utf-8",
            b"Unauthorized: open the exact link your sharer printed, including #token=...",
        )
        .await;
    }
    match path.as_str() {
        // "/" is handled above, before the token check.
        "/fallback" => {
            respond(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                FALLBACK_HTML.as_bytes(),
            )
            .await
        }
        "/stream" => serve_mjpeg(&mut stream, surface).await,
        "/ws" => {
            let key = head
                .ws_key
                .clone()
                .context("missing Sec-WebSocket-Key: this is not a WebSocket upgrade")?;
            info!("WebSocket viewer connected from {peer:?}");
            serve_websocket(
                &mut stream,
                &key,
                _updates,
                snapshot.clone(),
                token.as_str().to_string(),
            )
            .await
        }
        _ => respond(&mut stream, 404, "text/plain; charset=utf-8", b"not found").await,
    }
}

struct Head {
    path: String,
    query: std::collections::HashMap<String, String>,
    /// `Sec-WebSocket-Key`, present only for an upgrade request.
    ws_key: Option<String>,
}

async fn read_head<S: ByteStream>(stream: &mut S) -> Result<Head> {
    let mut buf = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        let n = stream.read(&mut byte).await?;
        if n == 0 {
            anyhow::bail!("client closed before sending a request");
        }
        buf.push(byte[0]);
        if buf.len() > MAX_REQUEST_HEAD {
            anyhow::bail!("request head exceeded {MAX_REQUEST_HEAD} bytes");
        }
        if buf.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    parse_head(&String::from_utf8_lossy(&buf))
}

fn parse_head(raw: &str) -> Result<Head> {
    let mut lines = raw.lines();
    let request = lines.next().context("empty request")?;
    let mut parts = request.split_whitespace();
    let method = parts.next().context("no method")?;
    let target = parts.next().context("no target")?;
    anyhow::ensure!(
        method.eq_ignore_ascii_case("GET"),
        "only GET is supported, got {method}"
    );

    let mut query = std::collections::HashMap::new();
    let mut ws_key = None;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("sec-websocket-key") {
                ws_key = Some(value.trim().to_string());
            }
        }
    }
    let path = match target.split_once('?') {
        Some((p, q)) => {
            for pair in q.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    query.insert(percent_decode(k), percent_decode(v));
                }
            }
            p.to_string()
        }
        None => target.to_string(),
    };
    Ok(Head {
        path,
        query,
        ws_key,
    })
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

async fn respond<S: ByteStream>(
    stream: &mut S,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}

async fn serve_mjpeg<S: ByteStream>(stream: &mut S, surface: &SharedSurface) -> Result<()> {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary={BOUNDARY}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(head.as_bytes()).await?;
    let mut last: Option<Arc<Vec<u8>>> = None;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(33)).await;
        let Some(jpeg) = surface.jpeg_preview()? else {
            continue;
        };
        // The shared cache means an unchanged screen costs no encode at
        // all, so an idle tab is nearly free.
        if last.as_ref().is_some_and(|l| Arc::ptr_eq(l, &jpeg)) {
            continue;
        }
        last = Some(jpeg.clone());
        let mut chunk = Vec::with_capacity(jpeg.len() + 128);
        chunk.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        chunk.extend_from_slice(b"Content-Type: image/jpeg\r\n");
        chunk.extend_from_slice(format!("Content-Length: {}\r\n\r\n", jpeg.len()).as_bytes());
        chunk.extend_from_slice(&jpeg);
        chunk.extend_from_slice(b"\r\n");
        if stream.write_all(&chunk).await.is_err() {
            return Ok(());
        }
    }
}

/// Upgrade to a WebSocket and forward the sharer's own encoded messages.
/// The browser parses exactly the bytes the native viewer parses; nothing
/// about the stream is re-encoded for the web path.
/// Serve one WebSocket viewer.
///
/// The browser does the same handshake the native client does, with
/// WebCrypto primitives instead of native ones. There is no plaintext
/// fallback: a browser that cannot complete the handshake is disconnected,
/// because silently serving it in the clear would defeat the point.
async fn serve_websocket<S: ByteStream>(
    stream: &mut S,
    key: &str,
    updates: &broadcast::Receiver<Arc<Vec<u8>>>,
    snapshot: SnapshotFn,
    token: String,
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        ws_accept(key)
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    // The browser completes the same handshake the native client does,
    // with WebCrypto primitives. The first frame it sends is its offer;
    // nothing else flows until the session exists, so a browser that
    // cannot finish is disconnected rather than served in the clear.
    let mut session = match browser_handshake(stream, &token).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Web viewer handshake failed: {e}");
            let _ =
                ws_write_frame(stream, 0x1, format!("{{\"error\":\"{}\"}}", e).as_bytes()).await;
            return Ok(());
        }
    };
    info!("Web viewer sealed");

    // The sharer publishes encoded message envelopes; the browser reads
    // the version byte first, so the framing prefix is not part of it.
    let mut updates = updates.resubscribe();

    // Catch this viewer up before any live update, exactly as the native
    // viewer does. Subscribing first means nothing produced in between is
    // lost, and the compositor discards anything it already holds.
    for msg in snapshot() {
        if ws_write_frame(stream, 0x2, &session.send.seal(&msg)?)
            .await
            .is_err()
        {
            return Ok(());
        }
    }

    loop {
        let update = tokio::select! {
            update = updates.recv() => match update {
                Ok(bytes) => bytes,
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    // The browser's compositor tracks revisions, so tell
                    // it to re-request rather than leaving it guessing.
                    warn!("Web viewer fell behind by {missed} messages; asking for a refresh");
                    Arc::new(
                        crate::network::Message::RequestKeyframe
                            .encode()
                            .unwrap_or_default(),
                    )
                }
                Err(broadcast::error::RecvError::Closed) => return Ok(()),
            },
            // Serve the browser's control frames so a close is noticed and
            // a ping keeps intermediaries from timing the socket out.
            frame = read_ws_frame(stream) => match frame? {
                WsFrame::Close => return Ok(()),
                WsFrame::Ping(payload) => {
                    if ws_write_frame(stream, 0xA, &payload).await.is_err() {
                        return Ok(());
                    }
                    continue;
                }
                WsFrame::Other => continue,
                WsFrame::Binary(payload) => {
                    // The only thing a browser sends on the data path is
                    // a control message, and it arrives sealed.
                    match session.receive.open(&payload) {
                        Ok(plain) => {
                            // A refresh request arrives sealed like
                            // everything else on this stream.
                            if let Ok(crate::network::Message::RequestKeyframe) =
                                crate::network::Message::decode(&plain)
                            {
                                for m in snapshot() {
                                    if ws_write_frame(stream, 0x2, &session.send.seal(&m)?)
                                        .await
                                        .is_err()
                                    {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Web viewer sent a frame we could not open: {e}");
                            return Ok(());
                        }
                    }
                    continue;
                }
            },
        };
        if ws_write_frame(stream, 0x2, &session.send.seal(&update)?)
            .await
            .is_err()
        {
            return Ok(());
        }
    }
}

/// Complete the browser handshake: take its offer, answer, and return the
/// session.
async fn browser_handshake<S: ByteStream>(
    stream: &mut S,
    token: &str,
) -> Result<crate::network::web_e2e::BrowserSession> {
    use crate::network::web_e2e::{accept, BrowserKeyPair, BrowserOffer};
    const K_BROWSER_OFFER: u8 = 0x22;
    const K_BROWSER_REPLY: u8 = 0x23;
    let frame = read_ws_frame(stream).await?;
    let WsFrame::Binary(payload) = frame else {
        anyhow::bail!("the first browser frame was not a handshake offer")
    };
    let mut cur = crate::network::Cursor::new(&payload);
    if cur.u8()? != K_BROWSER_OFFER {
        anyhow::bail!(
            "expected a browser offer, got kind 0x{:02x}",
            cur.u8().unwrap_or(0)
        );
    }
    let mut public = [0u8; 65];
    public.copy_from_slice(cur.take(65)?);
    let mut proof = [0u8; 32];
    proof.copy_from_slice(cur.take(32)?);

    let offer = BrowserOffer { public, proof };
    let mine = BrowserKeyPair::generate();
    let (reply, session) = accept(&offer, &mine, token)?;
    let mut out = Vec::with_capacity(1 + 65 + 32);
    out.push(K_BROWSER_REPLY);
    out.extend_from_slice(&reply.public);
    out.extend_from_slice(&reply.proof);
    ws_write_frame(stream, 0x2, &out).await?;
    Ok(session)
}

enum WsFrame {
    Binary(Vec<u8>),
    Close,
    Ping(Vec<u8>),
    Other,
}

/// Read one client frame. Browsers only send control frames here, so the
/// payload of a data frame is skipped rather than buffered.
async fn read_ws_frame<S: ByteStream>(stream: &mut S) -> Result<WsFrame> {
    let mut head = [0u8; 2];
    if stream.read_exact(&mut head).await.is_err() {
        return Ok(WsFrame::Close);
    }
    let opcode = head[0] & 0x0F;
    let masked = head[1] & 0x80 != 0;
    let mut len = (head[1] & 0x7F) as u64;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).await?;
        len = u16::from_be_bytes(ext) as u64;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).await?;
        len = u64::from_be_bytes(ext);
    }
    anyhow::ensure!(
        len <= MAX_REQUEST_HEAD as u64,
        "client frame of {len} bytes is larger than this endpoint accepts"
    );
    let mut mask = [0u8; 4];
    if masked {
        stream.read_exact(&mut mask).await?;
    }
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).await?;
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }
    Ok(match opcode {
        0x8 => WsFrame::Close,
        0x9 => WsFrame::Ping(payload),
        0x2 => WsFrame::Binary(payload),
        _ => WsFrame::Other,
    })
}

/// Write one unmasked server frame (RFC 6455 requires clients to mask,
/// servers must not).
async fn ws_write_frame<S: ByteStream>(stream: &mut S, opcode: u8, payload: &[u8]) -> Result<()> {
    let mut out = Vec::with_capacity(payload.len() + 10);
    out.push(0x80 | opcode);
    let len = payload.len();
    if len < 126 {
        out.push(len as u8);
    } else if len <= u16::MAX as usize {
        out.push(126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(payload);
    stream.write_all(&out).await?;
    stream.flush().await?;
    Ok(())
}

/// Compute the RFC 6455 accept token for a client key.
pub fn ws_accept(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WS_GUID.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_accept_matches_the_rfc_example() {
        // RFC 6455 section 1.3.
        assert_eq!(
            ws_accept("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn requests_are_parsed_into_path_and_query() {
        let head = parse_head("GET /ws?token=ABC123&x=1 HTTP/1.1\r\nHost: h\r\n\r\n").unwrap();
        assert_eq!(head.path, "/ws");
        assert_eq!(head.query.get("token").map(String::as_str), Some("ABC123"));
    }

    #[test]
    fn percent_encoded_tokens_are_decoded() {
        let head = parse_head("GET /?token=A%2DB HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(head.query.get("token").map(String::as_str), Some("A-B"));
    }

    #[test]
    fn non_get_requests_are_refused() {
        assert!(parse_head("POST /ws HTTP/1.1\r\n\r\n").is_err());
    }

    #[test]
    fn the_websocket_key_is_captured_for_upgrade() {
        let head = parse_head(
            "GET /ws?token=ABC123 HTTP/1.1\r\nUpgrade: websocket\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
        )
        .unwrap();
        assert_eq!(head.ws_key.as_deref(), Some("dGhlIHNhbXBsZSBub25jZQ=="));
    }
}
