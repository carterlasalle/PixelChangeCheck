# PixelChangeCheck (PCC) Project Status

## Project Overview
A highly efficient screen sharing tool using PixelChangeCheck (PCC) for optimized data transmission. It now actually runs end-to-end: a real CLI (`pcc share` / `pcc view` / `pcc relay`) wires capture, detection, encoding, and networking together, with direct QUIC, NAT-friendly relay, and browser (MJPEG, works from iPhones) transports all sharing the same pipeline.

## Status Legend
- 🔴 Not Started
- 🟡 In Progress
- 🟢 Completed

## What was broken (pre-overhaul)

The project had all the right pieces (capture, PCC detector, JPEG/LZ4, QUIC transport, frame buffer) but they were never wired together, and several pieces were internally broken:

- `main()` just constructed a capture + detector + encoder and exited; there was no actual capture/detect/encode/send loop, and no way to run as a sharer or viewer.
- The PCC detector treated RGB frame data (3 bytes/pixel) as if it were 1 byte/pixel, corrupting both the block comparison and the extracted change coordinates/data for any real (color) frame.
- There were three separate, inconsistent, dead/duplicate networking implementations (`network::NetworkManager`/`Connection`, `network::transport::QUICTransport`, `server::network::ServerNetwork`), none of which used length-prefixed framing consistently, so reads over a QUIC stream could tear a message in half.
- No CLI, no relay, no browser viewer, no native window viewer -- there was genuinely no way to watch a shared screen at all, from any device.
- Tests exercised individual functions in isolation; nothing verified frames actually flowing over a real network connection.

## Core Components

### 1. Capture 🟢
- [x] Real screen capture via the `screenshots` crate
- [x] Synthetic animated test-pattern fallback (`CaptureSource`) for headless machines / CI / local testing without a display

### 2. PCC Detection 🟢
- [x] Block-based frame comparison, **fixed to be RGB-stride-accurate** (was silently wrong before)
- [x] Regression test (`test_pcc_detector_respects_rgb_stride`) pinning exact changed-pixel geometry

### 3. Encoding 🟢
- [x] JPEG encoding (keyframes) + decoding (viewer-side reconstruction)
- [x] LZ4 compression/decompression for partial-update regions

### 4. Networking 🟢
- [x] Single, consistent length-framed `Message` protocol (`FullFrame` / `PartialUpdate` / `KeepAlive` / `QualityConfig` / `Error` / `Bye`)
- [x] `MessageTransport` trait implemented for both direct QUIC and relayed TCP, so upper layers don't care which path a viewer is using
- [x] Fixed a real QUIC bug: a viewer that never writes to its stream is invisible to the peer's `accept_bi()`; viewers now send a handshake `KeepAlive` immediately after connecting
- [x] QUIC transport config now actually wires up `connection_timeout` (max idle timeout) and `keepalive_interval`
- [x] Removed the three duplicate/dead networking implementations

### 5. Relay (NAT / internet traversal) 🟢
- [x] TCP relay server: sharer and viewer both dial out with a session code, no port-forwarding required on either end
- [x] Late-joining viewers are replayed the last cached keyframe instead of waiting for the next periodic one

### 6. Browser Viewer (iPhone / any device) 🟢
- [x] Embedded MJPEG (`multipart/x-mixed-replace`) HTTP server, works from any browser with zero native client
- [x] Verified by hand: fetched `/stream`, extracted a JPEG frame, confirmed it visually matches the captured content

### 7. Native Window Viewer 🟢
- [x] `pcc view` opens a `minifb` window on the main thread and renders reconstructed frames
- [x] Falls back to a headless status-printing loop if no display is available

### 8. Adaptive Quality 🟢
- [x] Sharer measures per-frame loop overrun and lowers/raises JPEG quality accordingly, notifying viewers via `QualityConfig` messages

### 9. Testing 🟢
- [x] Unit tests: capture (real + synthetic), detector (incl. RGB-stride regression), renderer/buffer, resilience, compression
- [x] **Real network end-to-end tests**: direct QUIC (`test_direct_quic_end_to_end`) and relay (`test_relay_end_to_end`, incl. late-join keyframe replay) -- not just in-process calls
- [x] `cargo run --example simple_screen_share` runs a full sharer+viewer demo over a real loopback QUIC connection and reports bytes saved by PCC
- [x] `cargo build`, `cargo test`, and `cargo clippy --all-targets` all clean

## Manually verified in this environment (headless container, no display)
- `pcc share --synthetic --web-port 8091` served a valid MJPEG stream; fetched and rendered a frame -- showed the expected animated gradient + bouncing box test pattern.
- `pcc share --listen ... ` + `pcc view --connect ...` transferred real frames end-to-end over QUIC (correct byte counts, quality-adaptation messages).
- `pcc relay` + `pcc share --relay ...` + `pcc view --relay ...` transferred frames through the relay, including a late-joining second viewer getting the cached keyframe immediately.

## Known limitations / next steps
- Relay path only forwards `Message` traffic; it does not implement UDP hole-punching, so it's a full byte-relay (higher latency than a direct connection) rather than making QUIC itself NAT-transparent.
- No authentication/encryption beyond QUIC's own TLS (with a self-signed, unverified cert) -- fine for personal use on a trusted network, not hardened for hostile networks.
- Adaptive quality reacts to local CPU/encode overrun, not measured network throughput; a real bandwidth probe would be a further improvement.
