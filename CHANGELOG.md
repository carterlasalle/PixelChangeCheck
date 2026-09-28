# Changelog

## Unreleased

The wire protocol is version 3. Viewers and sharers must be the same
build; a version mismatch is refused with a clear message rather than
mis-parsed.

### Correctness

- The sharer now diffs against the framebuffer viewers actually hold, so
  sub-threshold changes accumulate instead of being discarded forever.
  Previously a slow fade could diverge from the viewer without limit.
- The periodic repair snapshot is no longer suppressed by an unchanged
  screen, and is driven by elapsed time rather than a frame-number
  modulo.
- A viewer that joins now receives a snapshot of the *current* surface.
  Previously it could be replayed an arbitrarily old keyframe and then
  never receive the updates that would have corrected it.
- Falling behind produces a fresh snapshot rather than silently dropping
  patches that later ones depended on.
- Revisions and epochs make out-of-order and duplicate delivery harmless,
  and a geometry change starts a new epoch instead of ending the share.
- Rectangles are validated geometrically, so one that would wrap a
  scanline is refused rather than overwriting the next row.
- A malformed update is refused atomically: the framebuffer is unchanged
  and the revision does not advance.
- The authoritative surface is lossless. JPEG is now only the explicitly
  lossy browser preview.

### Performance

- The detector allocates nothing while scanning. It previously built two
  temporary vectors per tile -- about 4,080 allocations per 1080p frame.
- Changed tiles are merged before they are shipped, with a cost guard so
  a merge never drags in more unchanged area than it saves.
- A verified displacement becomes a single `Copy` operation, so a scroll
  costs bytes proportional to the exposed strip rather than to the whole
  screen.
- Uniform regions ship as a 24-byte `Fill` instead of a compressed patch.
- The message is encoded once and shared byte-for-byte with every viewer.
- The browser viewer encodes at most one JPEG per frame regardless of how
  many tabs are open.

### Security

- Viewers must present a session token; the token is the only secret.
- Direct QUIC and the relay pin the peer's certificate by fingerprint, so
  the connection is authenticated rather than merely encrypted.
- The relay speaks TLS, rate-limits failed registrations per address, and
  expires idle sessions.
- Every length prefix is checked before the allocation it authorises,
  including the relay's own reader.
- Decoded sizes are bounded by the declared geometry on every path.

### Protocol and interfaces

- Explicit little-endian wire format shared by the native client, the
  relay, and the browser, replacing `bincode`.
- Snapshots transfer as bounded begin/chunk/commit, so a 4K noise frame
  does not need a single oversized message.
- The browser gets the real compositor over WebSocket, not only MJPEG.
- `--connect`/`--relay` accept hostnames as well as literal addresses, and
  conflicting modes are refused instead of silently preferring one.

### Telemetry

`--log-level`, `--log-format text|json`, `--log-file` (hourly rotation),
`--stats-interval`, and `--metrics-listen`. `RUST_LOG` overrides
`--log-level`, because a live bug needs per-module filtering without a
rebuild. The capture loop records counters and never logs per frame.

Measured: detect, plan, encode, serialize and apply as separate
histograms; wire bytes split by what produced them; per-viewer lag,
repairs and queue depth; and the changed-area fraction, which is the
number that predicts which strategy ran.

### Reachability

`pcc diagnose` reports local addresses with each one classified, whether
a *global* IPv6 address exists, whether there is a default route, and a
STUN reflexive address. `share --reach auto|direct|relay` turns that into
a decision, resolved once at startup rather than per viewer.

The relay stays as the last rung deliberately: it is TCP+TLS, so it
already traverses the corporate proxies that block UDP, which is
exactly where a direct-only path fails.

`pcc pair` prints one URL carrying the token and the pin, so neither is
retyped. `pcc doctor` adds an audio capture probe.

### Delivery

Tag-triggered release across Linux, macOS (arm64 and x86) and Windows
with per-platform archives and checksums, a hardened multi-stage
Dockerfile, dependabot, and `deploy/relay/` with an Oracle Always Free
provisioning script, a hardened systemd unit and firewall notes.

### End-to-end encryption

X25519 plus ChaCha20-Poly1305 between sharer and viewer, keyed from the
session token so there is no second credential. Independent keys per
direction, monotonic frame counters, and the revision as AAD so a relay
can neither replay, reorder nor substitute.

### Audio

Opus at 48 kHz stereo in 20 ms frames, a cpal capture source with a
bounded queue, the audio-master playout that slaves video to the audio
clock, and an offset estimator that returns nothing until it converges.

Audio is never derived from the video surface, and the authoritative
pixels are never modified to make audio line up.

### Investigated, not shipped

- **A video path for sustained motion** (H.264/HEVC/AV1/AV2). The codec
  registry, its wire identifiers, and runtime capability probing are in
  place, and `probe_local()` honestly reports that this build can produce
  nothing. Both pure-Rust AV1 routes were tried and rejected on evidence:
  rav1e 0.6.3 trips Rust's `unsafe` precondition checks with an
  out-of-bounds access in its CDF context, at any frame size, and rav1e
  0.6.6 pins `clap =4.0.32`, which cannot co-exist with this project's
  `clap 4.5`. Shipping either would mean a crash or a native build
  system on the capture path. The pipeline falls back to a lossless
  snapshot, which is always correct. `src/codec/mod.rs` records this in
  full.

### Removed

- `Renderer`/`FrameBuffer`'s unused buffer and render loop, and the tests
  that only exercised that dead path.
- `QualityConfig::compression_level`, which no codec read.
- `ResilienceConfig::jitter_buffer_size` and `error_correction_enabled`,
  which claimed features nothing implemented.
- `NetworkResilience::monitor_connection`, which spawned an unsupervised
  task; the module is now actually used for reconnect backoff.
