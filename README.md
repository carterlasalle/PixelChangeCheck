# PixelChangeCheck (PCC)

An efficient screen-sharing tool: it replicates your screen **exactly** and
sends only the pixels that changed.

Rust. Shares to:

- **Direct QUIC viewers** on your LAN or over the internet (with port
  forwarding)
- **A relay server** for viewers behind NAT, with no port forwarding
  required
- **Any web browser**, including an iPhone's Safari, over a WebSocket that
  speaks the *same* change protocol the native client does

## What makes it different

A conventional screen stream re-encodes the whole picture every frame.
PixelChangeCheck keeps a **lossless authoritative surface**: the sharer
diffs each captured frame against the framebuffer viewers actually hold,
and sends only what changed. For a terminal, an IDE, or a document, that
is usually a few kilobytes per frame. When nothing changes, it sends
nothing but a keep-alive.

Three things make that claim hold up:

- **Exactness is measured, not asserted.** The authoritative stream is
  lossless end to end. After a snapshot plus its updates, a viewer's
  pixels are byte-identical to the sender's reference; the test suite
  asserts exactly that, including over a real socket and through the
  browser's JavaScript compositor.
- **Sub-threshold changes accumulate.** The sharer diffs against what
  viewers hold, not against the previous capture, so a slow fade crosses
  the threshold and gets sent instead of being silently discarded forever.
- **Scrolling is a copy, not a repaint.** A verified displacement becomes
  a `Copy` operation -- a few dozen bytes instead of a whole screen.

## Features

- **Screen capture** via the `screenshots` crate, with an automatic
  synthetic test-pattern fallback on headless machines
- **Pixel Change Check**: allocation-free block comparison, tile-hash region
  merging, and a bounded verified-displacement search for scroll reuse
- **A three-way representation choice per region**: solid fill, verified
  copy, or an exact LZ4 patch -- whichever is genuinely cheaper
- **Cost-based fallback**: when the patches for a frame would cost more
  than a fresh lossless snapshot, the sharer sends the snapshot
- **Chunked, atomic snapshots**: bounded transfer with begin/chunk/commit,
  so an interrupted snapshot can never half-replace a working surface
- **Revisions and epochs**: one sequencer, so a late joiner, a recovered
  viewer, and a viewer that fell behind all converge to the same pixels
- **Authentication**: a viewer token authorizes access; a certificate pin
  proves who you are talking to
- **TLS on every path**, including the relay
- **Browser viewer**: a WebSocket patch compositor (lossless), plus an
  explicitly lossy MJPEG fallback for anything that cannot run one
- **A relay** with per-viewer byte budgets, generational ownership, and a
  defined slow-viewer policy

## Getting Started

### Prerequisites

- Rust (see `rust-version` in `Cargo.toml`)
- System dependencies:
  - **Linux**: `libxcb1-dev`, `libxrandr-dev`, `libdbus-1-dev`
  - **macOS/Windows**: none

### Build

```sh
cargo build --release
```

### Share your screen

```sh
cargo run --release -- share
```

That prints the certificate fingerprint and a complete, copy-pasteable
viewer command:

```text
Certificate fingerprint (sha256): 9f2c...
Direct viewers on 0.0.0.0:5800
  pcc view --connect 192.168.1.20:5800 --token ABC... --pin 9f2c...
```

### View it

**Native window**, on another machine:

```sh
pcc view --connect <sharer-ip>:5800 --token <token> --pin <fingerprint>
```

**Any browser**, including a phone: open

```text
http://<sharer-ip>:8080/?token=<token>
```

That page runs the same compositor as the native client. If your browser
cannot, `/fallback` serves a lossy MJPEG preview, labelled as such.

**Behind NAT on both ends?** Run a relay on any reachable host:

```sh
pcc relay --listen 0.0.0.0:5900
```

It prints its own token and fingerprint. On the sharer:

```sh
pcc share --relay <relay-ip>:5900 --relay-pin <relay-fingerprint> --token <token>
```

On the viewer:

```sh
pcc view --relay <relay-ip>:5900 --pin <relay-fingerprint> \
         --session <code> --token <token>
```

Both sides make only *outbound* connections, so neither needs a port
forward.

**No display available?** Use the synthetic test pattern:

```sh
pcc share --synthetic
pcc view --connect 127.0.0.1:5800 --token <token> --pin <fingerprint> --no-window
```

Or run the whole pipeline -- sharer and viewer in one process, over a
real loopback QUIC connection, checking the result is pixel-exact:

```sh
cargo run --release --example simple_screen_share
```

### Browser over TLS

The web port is plaintext unless you give it a real certificate, because a
self-signed one would only produce a browser warning:

```sh
pcc share --web-cert cert.pem --web-key key.pem
```

### Commands

```sh
cargo run --release -- share --help
cargo run --release -- view --help
cargo run --release -- relay --help
cargo run --release -- diagnose
```

### Do I need a relay?

Most of the time, no. Ask the tool:

```sh
pcc diagnose
```

It reports your local addresses, whether you have a *global* IPv6
address (a link-local one is not routable and saying otherwise sends you
down a dead end), whether there is a default route, and a STUN reflexive
address. Then:

```sh
pcc share --reach auto     # try IPv6, then STUN, then the relay
pcc share --reach direct   # refuse the relay rather than use it
pcc share --reach relay    # skip discovery
```

The relay is the **last** rung on purpose. It is TCP+TLS, so it already
traverses the corporate proxies that block UDP -- which is exactly where
a direct-only path fails. Symmetric NAT and UDP-blocking proxies are the
two cases nothing free fixes.

Running one is cheap here specifically: at roughly 570 bytes/frame, a
session is about 160 MB per viewer per hour. See `deploy/relay/` for a
free-tier setup.

### Pairing without retyping a fingerprint

```sh
pcc pair --listen 192.168.1.20:5800 --pin <fingerprint>
```

prints one URL carrying both the token and the pin, so a viewer never
retypes or mistypes a 64-character hex string.

### Watching it work

```sh
pcc share --synthetic --stats-interval 5 --metrics-listen 127.0.0.1:9100
curl -s 127.0.0.1:9100/metrics
```

`--stats-interval` prints a table; `--metrics-listen` serves Prometheus
text on loopback. `RUST_LOG=pcc=debug` narrows the log, and
`--log-format json` is what you want in a bug report.

### Testing

```sh
cargo test                 # unit, replication, and real-socket end-to-end
cargo clippy --all-targets # lint
```

### Benchmarks

```sh
cargo run --release --example benchmarks
```

`--release` is required: a debug build reports numbers that mean nothing
for a codec. The benchmark drives the real pipeline end to end
(detect -> plan -> compress -> serialise -> deserialise -> apply), counts
allocations during the scan, and compares against a full-frame baseline.

## Architecture

```text
              capture
                 |
                 v
   reference  <- detect  <- current frame
   (what          |         ^
    viewers        v         |
    hold)      planner (fill / copy / rect, cost-based)
                 |
                 v
          snapshot  |  partial update      <- one sequencer: revision + epoch
                 |
                 v
        encoded ONCE, shared with every viewer
                 |
   +-------------+--------------+-------------------+
   v                            v                   v
QUIC viewer                relay (TLS)        web (WS / MJPEG)
   |                            |                   |
   v                            v                   v
Compositor  <--------------- forwards ------------> browser compositor
                                                     (a JS port of the same
                                                      state machine)
```

- **Sharer** (`pcc share`): captures, diffs against the reference, plans,
  encodes once, and fans the same bytes out to every viewer.
- **Viewer** (`pcc view`): authenticates, reconstructs through the
  `Compositor`, and presents.
- **Relay** (`pcc relay`): pairs a host with viewers by session code and
  forwards framed bytes. It never inspects frame contents, so
  end-to-end encryption can be added without changing it.

## How PCC works

1. Capture a frame.
2. Diff it against the **reference** -- the framebuffer an up-to-date
   viewer holds. The reference advances only over rectangles that actually
   shipped, so a change too small to send this frame is still there to be
   sent later.
3. If much of the screen moved as one displacement, verify it byte for
   byte and emit a `Copy`; hash agreement alone is never trusted.
4. Represent each remaining rectangle as a solid `Fill` or an exact,
   LZ4-compressed patch, whichever is smaller.
5. If the patches would cost more than a fresh lossless snapshot, send the
   snapshot instead.
6. Send nothing but a keep-alive if nothing changed.

The viewer applies each transaction atomically against a single
sequencing authority, and a `Copy` reads the surface as it was *before*
the transaction -- which is what lets two regions swap in one update.

## Roadmap

`docs/spec/roadmap.md` specifies six workstreams — telemetry, NAT
traversal, end-to-end encryption, delivery, audio, and one-command setup
— each with its interface, its definition of done, and what it
deliberately does not cover. Telemetry, reachability, end-to-end
encryption and delivery are built; the audio *transport* is not wired
into the pixel path yet, because audio must never share the compositor's
atomic-exactness guarantees.

Decisions already taken live in `docs/adr/`.

## Known limitations

- A browser gets a sealed stream, but over a *different* primitive than
  the native client. `crypto.subtle` has no X25519 in the versions most
  people run, so the browser does ECDH on P-256 with HKDF and AES-GCM
  while the native client does X25519 with ChaCha20-Poly1305. The protocol
  shape is identical; the primitives differ. A certificate is still worth
  supplying, because it stops the key exchange from being readable on the
  wire, but the surface content is encrypted either way.
- Screen capture is full-frame; platforms that expose dirty rectangles
  (DXGI, ScreenCaptureKit, PipeWire) are not yet used to skip work.
- Motion video is not implemented. The exact-replication path is the
  product; see `docs/adr/0001-lossless-authoritative-surface.md` for why a
  lossy base is not an option, and `docs/adr/` for the rest.
- There is no input injection, audio, or clipboard sync.

## License

MIT
