# Changelog

## Unreleased

The wire protocol is version 5. Viewers and sharers must be the same
build; a version mismatch is refused with a clear message rather than
mis-parsed. Version 4 added the end-to-end encryption handshake; version
5 adds `pts_us` to every visual message, so a viewer can schedule a frame
against the audio clock.

### Dependencies

- **quinn 0.10.2 -> 0.11**, with the TLS stack moved to match:
  **rustls 0.21 -> 0.23**, `tokio-rustls` 0.26, `rcgen` 0.13. This is
  what clears RUSTSEC-2026-0037 (8.7, DoS in Quinn endpoints) and
  RUSTSEC-2026-0185 (7.5, remote memory exhaustion from unbounded
  out-of-order stream reassembly). Both are reachable: this product
  listens on a UDP socket that anyone on the internet can send to.
- The rustls crypto provider is now named explicitly
  (`builder_with_provider` + the `ring` provider) rather than inferred.
  With `ring` and `aws-lc-rs` both reachable in the tree, rustls refuses
  to guess and panics at the first handshake.
- Also updated past advisories: `bytes` 1.9 -> 1.12, `crossbeam-epoch`
  0.9.18 -> 0.9.21, `ring` 0.17.8 -> 0.17.14, `time` 0.3.37 -> 0.3.55,
  `tracing-subscriber` 0.3.19 -> 0.3.23.
- `.cargo/audit.toml` ignores two quick-xml advisories, and only those
  two. Both are memory-exhaustion and quadratic-time bugs in an XML
  parser, reached here only at build time by `wayland-scanner` and
  `xcb`, parsing the protocol definitions that ship with the system. The
  fix needs `screenshots` above 0.8.10, which does not exist. The file
  says what to re-check when it does.

### Fixed

- **The minimum supported Rust version moves from 1.85 to 1.88.** Every
  `time` release carrying the RUSTSEC-2026-0009 fix requires rustc 1.88,
  and `time` arrives through rcgen (certificate validity dates) and
  tracing-appender (log rotation). Both are genuinely used, so the floor
  moves rather than the fix. `time` is now on 0.3.47, the oldest patched
  release.
- Two STUN peer tests failed on Windows and passed everywhere else. The
  responder binds the wildcard address, which is right in production and
  wrong to send *to*: Linux and macOS route a packet addressed to
  0.0.0.0 back over loopback, and Windows does not, so the probe never
  arrived and the check reported unreachable. The tests now ask for
  loopback explicitly.
- CI installed no system dependencies at all. `alsa-sys` runs
  `pkg-config` at build time and panics without the ALSA headers, so
  every Linux job was a build failure. All four now install them.
- CI watched `push: branches: [main]` but the default branch is `master`,
  so it never ran on a push. It failed silently, which is the worst way
  for CI to fail: the badge stays green because no run was ever
  requested.

### Packaging

- The crate now carries the metadata crates.io requires: a licence, a
  repository, a readme, keywords and categories. It is installable with
  `cargo install pixel-change-check-client`.
- Licensed **AGPL-3.0-only** -- version 3 only, not "or later". Say so
  precisely rather than leaving a consumer to infer which.
- Development tooling is excluded from the published package. Without it,
  `cargo publish` shipped 6 MB of agent configuration and a stale status
  file to every downloader: 149 of 228 packaged files. It is now 74.
- `spin` and `lz4_flex` were pinned to yanked releases. Both moved
  forward a patch to a non-yanked version, so an install no longer
  pulls known-bad versions through `ring` and into the TLS path.
- A tag whose version disagrees with `Cargo.toml` is refused before
  anything is published, and a version that already exists is refused
  rather than silently failing mid-upload.

### Browser encryption

- The browser viewer is sealed end to end. There is no plaintext
  fallback: a browser that cannot complete the handshake is disconnected
  and told why, because quietly serving it in the clear would defeat the
  point. Every visual message after the handshake is encrypted, and so is
  every control message the browser sends.
- A browser has its own keys. The sharer's plaintext broadcast exists
  nowhere: it is sealed per viewer, which is what makes a per-viewer
  revocation meaningful on a broadcast path.
- A wrong token is refused at the handshake rather than producing a
  session that decrypts to garbage. The proof is a hash over the peer's
  public key, the token and a domain-separating prefix, computed
  identically in Rust and in JavaScript.
- `crypto.subtle` has no X25519 in the browsers most people run, so the
  browser uses ECDH P-256 with HKDF and AES-GCM where the native client
  uses X25519 with ChaCha20-Poly1305. The protocol shape is identical.
  See `docs/adr/0005-browser-key-exchange-p256.md`.
- The two implementations are checked against a shared fixed vector, so
  a key schedule that silently diverges fails the build rather than
  failing every handshake in production with no useful error.

### Reachability

- NAT-PMP port mapping (RFC 6886), with no C dependency: a two-byte
  request over UDP 5351 to the gateway, and only external-port requests,
  since requesting a specific internal port is the classic amplifier.
- A real ICE-lite subset over STUN: the sharer answers binding
  indications, and a viewer tries to send one and reports success. It
  proves a direct path exists rather than merely proving reflexive
  address discovery, which is the actual point of the rung.
- The reachability ladder now reports which rung it used and how long it
  took, so a failure names the rung that failed instead of "unreachable".

### Audio

- Audio travels on its own unidirectional QUIC stream, opened by the
  viewer, so it never queues behind a video frame and never touches the
  message path. A session carries at most one audio stream.
- Every visual message carries `pts_us` from the sharer's capture clock,
  and the viewer holds a video frame until the audio clock releases its
  revision, so the two are played rather than merely received together.
- The sync harness measures the offset from a known tone instead of
  asserting one. Real paths are asymmetric, so the expectation is a
  bounded window that accounts for the codec's own delay, and the
  measurement is a real number rather than a guess.

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
direction, monotonic frame counters, and the counter as AAD so a relay
can neither replay, reorder nor substitute.

Sealing is per viewer, after the shared broadcast, because each viewer
has its own keys. There is no plaintext fallback: the handshake either
produces a session or the connection is refused. The browser path is a
separate trust domain and is not yet covered.

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
