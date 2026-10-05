//! The relay's browser-viewer surface.
//!
//! The relay is the one component both sides can already reach, so it is
//! the natural place to host a viewer page: a browser needs no binary, no
//! terminal, and no `--web-cert` files on the sharer. The page and its
//! WebSocket share the relay's own TLS certificate, which is what removes
//! the plaintext-versus-TLS problem the sharer's web port has on a
//! non-loopback address.
//!
//! What this does **not** change is the trust story. The relay forwards
//! opaque browser frames and never sees a key: the browser's WebCrypto
//! handshake is answered by the *host*, over the same host leg that
//! carries a native viewer's handshake. The relay learns frame sizes and
//! timing, exactly as it already did.

use crate::network::SessionToken;
use crate::relay::{self, SessionTable};
use crate::server::renderer::web;
use anyhow::{Context, Result};
use std::sync::atomic::AtomicU64;
use tokio::io::AsyncWriteExt;

/// Serve one HTTP request on the relay's TLS listener.
///
/// Reached only when ALPN negotiated `http/1.1`, so a browser talking to
/// the relay port lands here and a `pcc` client never does.
pub(crate) async fn handle_http<S: web::ByteStream>(
    mut stream: S,
    sessions: &SessionTable,
    next_gen: &AtomicU64,
    token: &SessionToken,
) -> Result<()> {
    let head = web::read_head(&mut stream).await?;
    let path = head.path.clone();
    let presented = head.query.get("token").map(String::as_str).unwrap_or("");
    let authorized = crate::network::verify_token(token, presented);

    match route(&path) {
        // The page and the script carry no secret and no screen data, so
        // they are served to anyone who asks. The token rides in the URL
        // fragment, which the browser never puts in a request; the
        // WebSocket handshake is where it is checked, before a byte of
        // surface data can move.
        Route::Script => {
            web::respond(
                &mut stream,
                200,
                "application/javascript; charset=utf-8",
                web::CLIENT_JS.as_bytes(),
            )
            .await
        }
        Route::Page => {
            web::respond(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                web::INDEX_HTML.as_bytes(),
            )
            .await
        }
        // The page derives its WebSocket path from its own directory, so
        // `/v/ABC` would make that `/v/ws`. Send the slash form.
        Route::RedirectTo(location) => redirect(&mut stream, &location).await,
        Route::Socket(session) => {
            if !authorized {
                return web::respond(
                    &mut stream,
                    401,
                    "text/plain; charset=utf-8",
                    b"Unauthorized: open the exact link your sharer printed, including #token=...",
                )
                .await;
            }
            if !relay::session_has_host(sessions, &session).await {
                return web::respond(
                    &mut stream,
                    503,
                    "text/plain; charset=utf-8",
                    b"no host in this session yet -- is the sharer still running?",
                )
                .await;
            }
            let key = head
                .ws_key
                .clone()
                .context("missing Sec-WebSocket-Key: this is not a WebSocket upgrade")?;
            bridge(&mut stream, &key, sessions, next_gen, &session).await
        }
        Route::NotFound(why) => {
            web::respond(
                &mut stream,
                404,
                "text/plain; charset=utf-8",
                why.as_bytes(),
            )
            .await
        }
    }
}

/// What a request path means, decided without touching a socket so the
/// shape of the URL is unit-testable.
#[derive(Debug, PartialEq, Eq)]
enum Route {
    /// The viewer page for one session.
    Page,
    /// The shipped browser compositor.
    Script,
    /// The WebSocket carrying sealed surface data.
    Socket(String),
    /// A permanent redirect to the slash form of a session URL.
    RedirectTo(String),
    /// Nothing here; the string is the explanation sent back.
    NotFound(&'static str),
}

const NO_SESSION: &str = "no session in this URL; the sharer prints the exact link";
const NOT_FOUND: &str = "not found: a viewer URL looks like /v/<session>/#token=...";

fn route(path: &str) -> Route {
    if path == "/pcc.js" {
        return Route::Script;
    }
    let Some(rest) = path.strip_prefix("/v/") else {
        return Route::NotFound(NOT_FOUND);
    };
    match rest.split_once('/') {
        // `/v/<session>`: no slash yet, so redirect.
        None if rest.is_empty() => Route::NotFound(NO_SESSION),
        None => Route::RedirectTo(format!("/v/{rest}/")),
        Some(("", _)) => Route::NotFound(NO_SESSION),
        Some((_, "")) => Route::Page,
        Some((session, "ws")) => Route::Socket(session.to_string()),
        Some(_) => Route::NotFound(NOT_FOUND),
    }
}

async fn redirect<S: web::ByteStream>(stream: &mut S, location: &str) -> Result<()> {
    let head = format!(
        "HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(head.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Upgrade to a WebSocket and bridge it to the session's host.
///
/// Every payload crossing this function is opaque: the browser's offer,
/// the host's reply, sealed surface updates, sealed acknowledgements. The
/// relay tags each one with the viewer id so the host can run an
/// independent session for it, and never inspects the contents.
async fn bridge<S: web::ByteStream>(
    stream: &mut S,
    key: &str,
    sessions: &SessionTable,
    next_gen: &AtomicU64,
    session_id: &str,
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        web::ws_accept(key)
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    let mut viewer = match relay::register_browser_viewer(sessions, next_gen, session_id).await {
        Ok(v) => v,
        Err(e) => {
            // The upgrade already succeeded, so the failure has to be said
            // in-band; a closed socket with no reason is the thing this
            // whole path exists to avoid.
            tracing::warn!("relay browser viewer refused: {e}");
            let _ =
                web::ws_write_frame(stream, 0x1, format!("{{\"error\":\"{e}\"}}").as_bytes()).await;
            return Ok(());
        }
    };
    let viewer_id = viewer.id;
    tracing::info!("relay: browser viewer {viewer_id} joined session '{session_id}'");

    loop {
        tokio::select! {
            frame = web::read_ws_frame(stream) => {
                match frame? {
                    web::WsFrame::Close => break,
                    // Answer pings here: a browser tab behind a proxy is
                    // timed out without them, and the host has no way to
                    // know a control frame arrived.
                    web::WsFrame::Ping(payload) => {
                        if web::ws_write_frame(stream, 0xA, &payload).await.is_err() {
                            break;
                        }
                    }
                    web::WsFrame::Other => {}
                    web::WsFrame::Binary(payload) => {
                        if relay::forward_browser_payload(sessions, session_id, viewer_id, &payload)
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
            outbound = viewer.rx.recv() => {
                let Some(framed) = outbound else { break };
                if web::ws_write_frame(stream, 0x2, strip_len_prefix(&framed)).await.is_err() {
                    break;
                }
            }
        }
    }

    relay::unregister_browser_viewer(sessions, session_id, &viewer).await;
    tracing::info!("relay: browser viewer {viewer_id} left session '{session_id}'");
    Ok(())
}

/// Drop the 4-byte length prefix the viewer leg carries.
///
/// The relay's viewer leg is length-framed, and the host leg's reader
/// re-adds that prefix on the way out — which is correct for a native
/// viewer, whose `Message` reader consumes it. A WebSocket frame has no
/// such prefix, so the two conventions are reconciled here rather than by
/// making either side aware of the other. A frame whose prefix does not
/// describe it is passed through untouched, because inventing a split is
/// worse than forwarding bytes the peer will reject loudly.
fn strip_len_prefix(framed: &[u8]) -> &[u8] {
    if framed.len() >= 4 {
        let len = u32::from_le_bytes([framed[0], framed[1], framed[2], framed[3]]) as usize;
        if len + 4 == framed.len() {
            return &framed[4..];
        }
    }
    framed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_urls_route_to_page_socket_or_redirect() {
        // The trailing slash is load-bearing: the page derives its
        // WebSocket path from its own directory, so `/v/ABC` must not be
        // served as-is or the socket would be requested at `/v/ws`.
        assert_eq!(route("/v/ABC"), Route::RedirectTo("/v/ABC/".into()));
        assert_eq!(route("/v/ABC/"), Route::Page);
        assert_eq!(route("/v/ABC/ws"), Route::Socket("ABC".into()));
        assert_eq!(route("/pcc.js"), Route::Script);
    }

    #[test]
    fn a_session_url_without_a_session_is_not_a_page() {
        // `/v/` and `/v//ws` are both "no session", not "the session
        // named empty string" — an empty session id would otherwise be
        // looked up in the relay's table.
        assert_eq!(route("/v/"), Route::NotFound(NO_SESSION));
        assert_eq!(route("/v//ws"), Route::NotFound(NO_SESSION));
        assert_eq!(route("/"), Route::NotFound(NOT_FOUND));
        assert_eq!(route("/v/ABC/nope"), Route::NotFound(NOT_FOUND));
    }

    #[test]
    fn the_viewer_leg_prefix_is_stripped_for_a_websocket() {
        // The relay's viewer leg is length-framed; a WebSocket frame is
        // not. Sending the prefix would put the length's low byte where
        // the browser expects the frame kind, which is exactly how the
        // first relay-hosted browser viewer failed.
        let payload = b"hello";
        let mut framed = (payload.len() as u32).to_le_bytes().to_vec();
        framed.extend_from_slice(payload);
        assert_eq!(strip_len_prefix(&framed), payload);

        // Anything the prefix does not describe is passed through whole:
        // inventing a split is worse than a peer rejecting it loudly.
        assert_eq!(strip_len_prefix(b"ab"), b"ab");
        assert_eq!(
            strip_len_prefix(b"\x09\x00\x00\x00ab"),
            b"\x09\x00\x00\x00ab"
        );
    }
}
