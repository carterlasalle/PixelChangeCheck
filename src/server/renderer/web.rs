//! A tiny MJPEG (`multipart/x-mixed-replace`) HTTP server so any device
//! with a browser -- an iPhone, an iPad, a Chromebook, whatever -- can
//! watch the stream at `http://<host>:<port>/` without needing the native
//! client at all.

use crate::encoder::encode_jpeg;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tiny_http::{Header, Response, Server};
use tracing::{error, info, warn};

const BOUNDARY: &str = "pccframe";
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>PixelChangeCheck Viewer</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body { margin: 0; background: #111; display: flex; align-items: center; justify-content: center; height: 100vh; }
    img { max-width: 100%; max-height: 100%; }
  </style>
</head>
<body>
  <img src="/stream" alt="live screen share">
</body>
</html>"#;

/// Serves the live frame in `snapshot` as an MJPEG stream. Blocks the
/// calling thread forever, so run it via `spawn_blocking` / its own thread.
pub fn run_web_server(
    bind_addr: SocketAddr,
    snapshot: Arc<Mutex<Vec<u8>>>,
    width: u32,
    height: u32,
    quality: f32,
    target_fps: u32,
) -> anyhow::Result<()> {
    let server = Server::http(bind_addr)
        .map_err(|e| anyhow::anyhow!("Failed to start web viewer on {bind_addr}: {e}"))?;
    info!("Web viewer available at http://{bind_addr}/");

    let frame_interval = Duration::from_secs(1) / target_fps.max(1);

    for request in server.incoming_requests() {
        let snapshot = snapshot.clone();
        let url = request.url().to_string();
        std::thread::spawn(move || {
            let result = if url == "/stream" {
                serve_mjpeg_stream(request, snapshot, width, height, quality, frame_interval)
            } else {
                serve_index(request)
            };
            if let Err(e) = result {
                warn!("Web viewer client error: {e}");
            }
        });
    }

    Ok(())
}

fn serve_index(request: tiny_http::Request) -> anyhow::Result<()> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .expect("valid header");
    let response = Response::from_string(INDEX_HTML).with_header(header);
    request.respond(response)?;
    Ok(())
}

fn serve_mjpeg_stream(
    request: tiny_http::Request,
    snapshot: Arc<Mutex<Vec<u8>>>,
    width: u32,
    height: u32,
    quality: f32,
    frame_interval: Duration,
) -> anyhow::Result<()> {
    let content_type = format!("multipart/x-mixed-replace; boundary={BOUNDARY}");
    let header = Header::from_bytes(b"Content-Type".as_slice(), content_type.as_bytes())
        .expect("valid header");

    // tiny_http doesn't have a "keep writing chunks forever" response
    // helper, so we drive the underlying writer directly via a chunked
    // response body implemented as a `Read` that blocks between frames.
    let body = MjpegBody {
        snapshot,
        width,
        height,
        quality,
        frame_interval,
        pending: Vec::new(),
        pos: 0,
    };

    let response = Response::new(
        tiny_http::StatusCode(200),
        vec![header],
        body,
        None,
        None,
    );

    match request.respond(response) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("MJPEG stream ended: {e}");
            Ok(())
        }
    }
}

/// A `Read` impl that lazily produces the next multipart-JPEG chunk each
/// time the previous one has been fully consumed, pacing itself to
/// `frame_interval`. This lets tiny_http stream indefinitely to a browser
/// `<img>` tag without us needing async plumbing on this thread.
struct MjpegBody {
    snapshot: Arc<Mutex<Vec<u8>>>,
    width: u32,
    height: u32,
    quality: f32,
    frame_interval: Duration,
    pending: Vec<u8>,
    pos: usize,
}

impl std::io::Read for MjpegBody {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.pending.len() {
            std::thread::sleep(self.frame_interval);

            let rgb = self
                .snapshot
                .lock()
                .map_err(|_| std::io::Error::other("snapshot lock poisoned"))?
                .clone();

            let jpeg = if rgb.len() == (self.width * self.height * 3) as usize {
                encode_jpeg(self.width, self.height, self.quality, &rgb)
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let mut chunk = Vec::with_capacity(jpeg.len() + 128);
            chunk.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
            chunk.extend_from_slice(b"Content-Type: image/jpeg\r\n");
            chunk.extend_from_slice(format!("Content-Length: {}\r\n\r\n", jpeg.len()).as_bytes());
            chunk.extend_from_slice(&jpeg);
            chunk.extend_from_slice(b"\r\n");

            self.pending = chunk;
            self.pos = 0;
        }

        let mut cursor = Cursor::new(&self.pending[self.pos..]);
        let n = std::io::Read::read(&mut cursor, buf)?;
        self.pos += n;
        Ok(n)
    }
}
