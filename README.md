# PixelChangeCheck (PCC)

A highly efficient screen sharing tool using PixelChangeCheck (PCC) for optimized data transmission, written in Rust. It shares your screen to:

- **Direct QUIC viewers** on your LAN or over the internet (with port forwarding)
- **A relay server** for viewers behind NAT, with no port forwarding required
- **Any web browser** -- including an iPhone's Safari -- via a plain MJPEG stream, no native client needed

## Features

- **Screen Capture**: Cross-platform screen capture using the `screenshots` crate, with an automatic synthetic-test-pattern fallback on headless machines
- **Pixel Change Detection (PCC)**: Block-based pixel comparison that detects only changed regions between frames (RGB-aware, byte-accurate)
- **JPEG Encoding**: Fast JPEG encoding with configurable quality for keyframes
- **LZ4 Compression**: Additional lossless compression for changed pixel regions
- **QUIC Transport**: Low-latency, reliable, length-framed messaging over QUIC for direct connections
- **Relay Mode**: A lightweight TCP relay so a sharer and viewer that can't reach each other directly (both behind NAT) can still connect
- **Browser Viewer**: An embedded MJPEG HTTP server so any device with a browser can watch, including phones
- **Native Window Viewer**: `pcc view` opens a real window and renders the stream
- **Adaptive Quality**: The sharer automatically lowers/raises JPEG quality based on whether it's keeping up with its frame budget
- **Keep-Alives**: When nothing changes, only a tiny keep-alive message is sent

## Project Structure

```
src/
├── app/               # CLI orchestration for `share` and `view`
├── capture/           # Screen capture (real + synthetic test pattern)
├── encoder/           # JPEG encoding/decoding and LZ4 compression
├── network/           # QUIC transport, message protocol, and resilience
│   ├── config.rs      # Network and TLS/QUIC transport configuration
│   ├── protocol.rs    # Length-framed Message protocol (FullFrame/PartialUpdate/...)
│   ├── resilience.rs  # Retry logic and connection health
│   └── transport.rs   # QUIC endpoints + MessageTransport trait
├── relay/             # TCP relay server + relay-backed transport (NAT traversal)
├── pcc/               # Pixel Change Check core logic
│   ├── detector.rs    # Block-based, RGB-stride-aware change detection
│   └── types.rs       # Frame, PixelChange, and trait definitions
├── server/            # Viewer-side components
│   └── renderer/      # Frame reconstruction, native window feed, MJPEG web server
├── lib.rs             # Library exports
└── main.rs            # `pcc` CLI entry point (share / view / relay)
```

## Getting Started

### Prerequisites

- Rust (1.70 or higher)
- System dependencies:
  - **Linux**: `libxcb1-dev`, `libxrandr-dev`, `libdbus-1-dev`
  - **macOS/Windows**: No extra dependencies needed

### Installation

```bash
git clone https://github.com/carterlasalle/PixelChangeCheck.git
cd PixelChangeCheck

# Install system dependencies (Linux)
# sudo apt-get install -y libxcb1-dev libxrandr-dev libdbus-1-dev

./setup.sh   # or just: cargo build
```

### Usage

**Share your screen** (defaults: listens on `0.0.0.0:5800` for direct viewers, and serves a browser viewer on `0.0.0.0:8080`):

```bash
cargo run -- share
```

Open `http://<this-machine's-ip>:8080/` from any browser -- including an iPhone's Safari on the same network -- to watch immediately, no client install required.

**View from another desktop, over the same LAN or the internet** (with the sharer's port forwarded):

```bash
cargo run -- view --connect <sharer-ip>:5800
```

**Behind NAT on both ends? Use a relay** (run this on any reachable host, e.g. a small cloud VM):

```bash
cargo run -- relay --listen 0.0.0.0:5900
```

Then, on the sharer:

```bash
cargo run -- share --relay <relay-ip>:5900 --session MYCODE
```

And on the viewer:

```bash
cargo run -- view --relay <relay-ip>:5900 --session MYCODE
```

Both sides only ever make *outbound* connections to the relay, so no port forwarding is needed on either end.

**Test everything locally, no display required** (uses a synthetic animated test pattern):

```bash
cargo run -- share --synthetic
cargo run -- view --connect 127.0.0.1:5800 --no-window   # or open http://127.0.0.1:8080/
```

Or run the fully self-contained demo, which spins up a sharer and viewer in one process over a real QUIC connection and prints the bandwidth PCC saved you:

```bash
cargo run --example simple_screen_share
```

### Testing

```bash
cargo test
```

Includes real, over-the-loopback-network end-to-end tests for both the direct QUIC path and the relay path, not just in-process unit tests.

### Benchmarks

```bash
cargo run --example benchmarks
```

## Architecture

- **Sharer** (`pcc share`): captures the screen, detects changed pixels with PCC, encodes/compresses them, and fans them out to any direct QUIC viewers, a relay connection, and/or the built-in browser (MJPEG) viewer, all at once.
- **Viewer** (`pcc view`): connects directly or through a relay, reconstructs the frame from keyframes + partial updates, and displays it in a native window (or headless status output if no display is available).
- **Relay**: a TCP server that pairs a sharer and any number of viewers by session code and forwards frames between them, including replaying the last keyframe to viewers who join mid-session.

## How PCC Works

1. The sharer captures screen frames at the target frame rate.
2. Each frame is compared against the previous frame using block-based, RGB-aware pixel comparison.
3. Only blocks where pixel values changed beyond a configurable threshold are identified, and their exact bounding box is computed.
4. Changed regions are extracted, lz4-compressed, and sent as a `PartialUpdate`; full frames are JPEG-encoded and sent periodically (or on first connect) as a `FullFrame`.
5. Viewers apply `FullFrame`s wholesale and blit `PartialUpdate` regions into their reconstructed frame.
6. If nothing changed, only a `KeepAlive` is sent.

## License

MIT
