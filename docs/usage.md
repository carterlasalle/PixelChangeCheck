# Usage

Every entry point to PixelChangeCheck, what it does, and how to invoke it.
The CLI help (`pcc <command> --help`) is generated from the same definitions
in `src/main.rs`; if this page and `--help` ever disagree, `--help` wins and
this page has a bug — file it.

## Install

```sh
cargo install cargo-binstall   # once: the `cargo binstall` subcommand itself
cargo binstall pixel-change-check-client   # prebuilt binary, no compile
cargo install pixel-change-check-client --locked
# headless (no audio capture, no native window, no system headers):
cargo install pixel-change-check-client --locked --no-default-features
```

One binary, `pcc`. Sharer and viewer must be the same build (protocol v7);
a mismatch is refused with a clear message, not mis-parsed.

Supported binary targets are the release matrix
(`x86_64`/`aarch64` Linux, Apple Silicon macOS, `x86_64` Windows). On a
target with no published archive, binstall silently falls back to a full
source compile — pass `--disable-strategies compile` to fail fast
instead of paying ~7 minutes unknowingly.

Upgraded from ≤0.1.3? The binary was renamed (`pixel-change-check-client`
→ `pcc`), so both can be on PATH. `cargo uninstall
pixel-change-check-client` resolves the name from the *current* manifest
and deletes the new `pcc`, leaving the stale old binary behind — remove
`~/.cargo/bin/pixel-change-check-client` by hand and check
`pcc --version`.

## Global flags (work on every command)

| Flag | Default | What it does |
|---|---|---|
| `--log-level` | `info` | Minimum level: `error warn info debug trace`. `RUST_LOG` overrides it. |
| `--log-format` | `text` | `text` or `json`. JSON is what you want in a bug report. |
| `--log-file` | — | Also write logs here, rotating hourly. |
| `--stats-interval` | `0` (off) | Print a metrics summary to stderr every N seconds. Live: per-second table with plan/encode/apply timings, wire totals by kind (patch/fill/copy/snapshot/preview + peak), viewer and capture counters. |
| `--metrics-listen` | — | Serve Prometheus text at e.g. `127.0.0.1:9100` (`curl …/metrics`). Loopback only by convention; it is unauthenticated, so never bind it publicly. Live: `pcc_frames_total`, `pcc_bytes_{total,patch,fill,copy,snapshot,preview}`, `pcc_viewers_{joined,rejected}`, `pcc_lag_events_total`, … |

## `pcc share` — capture and share your screen

```sh
pcc share
```

Prints the certificate fingerprint and a complete copy-pasteable viewer
command. Picks: primary display, 30 fps target (60 ceiling), token
auto-generated, browser on `:8080`, direct QUIC on `:5800`.

### What to share (pick at most one)

| Flag | What it does |
|---|---|
| `--display N` | Share display N. Order is `pcc diagnose --displays`. Default is 0 (primary). |
| `--region x,y,w,h` | Share e.g. `100,200,1280,720` in display pixels. Captured at the layer, not cropped after. Conflicts with `--display`. |
| `--window <text>` | Share the window whose title contains `<text>`. Fails fast on no match rather than sharing the display. Re-resolved every frame, so close/reopen and moves are followed. |
| `--application <text>` | Same contract, matched on app name or title. |
| `--synthetic` | Animated test pattern instead of the real screen. For headless machines and local tests; also sweeps a scripted cursor so the cursor plane is exercised end to end. |

### Transports (pick one with `--transport`)

| Value | Default | How it connects |
|---|---|---|
| `quic` | ✓ | Direct UDP listener (`--listen`, default `0.0.0.0:5800`) plus optional relay legs below. |
| `iroh` | | Prints a ticket; viewers dial it. No ports, no relay to run: `pcc view --transport iroh --ticket … --token …`. |
| `webrtc` | | Prints an `OFFER:` blob; paste the viewer's `ANSWER:` blob back on stdin, then trickle `CANDIDATE:` lines both ways (one JSON blob per line, empty line ends). One reliable ordered `pcc` data channel. 20 s handshake timeout. |

Aliases accepted: `web-rtc`, `rtc`. Anything else fails fast naming the three.

### Reaching the viewer

| Flag | What it does |
|---|---|
| `--listen <host:port>` | Direct QUIC listener. `--no-listen` disables it (relay-only share). |
| `--relay <host:port,…>` | Comma-separated relays to register on. First reachable relay wins per viewer. Requires `--relay-pin`. |
| `--relay-pin <fp>` | SHA-256 fingerprint of the relay's certificate, as the relay printed at startup. A short pin fails fast: `a SHA-256 fingerprint is 64 hex characters, got N`. |
| `--session <code>` | Rendezvous code for the relay session. Auto-generated if omitted. |
| `--token` | **Doubles as the relay credential.** The relay was started with its own `--token`, and the share's `--token` must be the *same string* — the relay checks `HMAC(relay-token)` on registration, and a mismatch closes with `bad credential` (seen live: `Relay: rejected Host … (bad credential)`). The viewer's `--token` must then match too, since the E2E handshake after registration checks it again. |
| `--reach auto\|direct\|relay` | `auto` (default): try IPv6, then STUN, then relay. `direct`: fail rather than use a relay. `relay`: skip discovery. Ask the tool first: `pcc diagnose`. |
| `--broadcast-above N` | Past N direct viewers, the newest is handed a `Redirect` to the first `--relay` and its direct session ends. `0` (default) disables. Needs `--relay`. |

### Browser viewer

Served on `--web` (default `127.0.0.1:8080`); `--no-web` disables it.
The loopback default is deliberate: a non-loopback browser address
without `--web-cert`/`--web-key` is refused, so the old `0.0.0.0`
default made bare `pcc share` a guaranteed error.

| Path | Auth | What it is |
|---|---|---|
| `/`, `/index.html` | none | The viewer page. Carries no secret and no pixels. Live: `200` with no token. |
| `/pcc.js` | none | The compositor — a JS port of the same state machine as the native client. Live: `200` with no token. |
| `/ws` | token in handshake | The sealed change stream. Token is checked before one byte of surface data moves. Live: `401` with no token. |
| `/fallback` | token query | Page for the lossy path. Live: `200` with token. |
| `/stream` | token query | Explicitly lossy MJPEG preview for anything that cannot run the compositor. Labelled as such. Live: `200` with token. |
| anything else | token query | `404`. Live: verified. |

The token travels in the URL *fragment* (`#token=…`), which the browser
never sends in a request — the page load cannot authenticate and does not
pretend to. Remote (non-loopback) `--web` requires `--web-cert` +
`--web-key` (real PEM certificate); loopback stays plaintext for dev.
Surface content is encrypted either way; the certificate stops the key
exchange itself from being readable on the wire.

## `pcc view` — watch a shared session

```sh
# direct
pcc view --connect <sharer-ip>:5800 --token <token> --pin <fingerprint>
# via relay
pcc view --relay <relay-ip>:5900 --pin <relay-pin> --session <session> --token <token>
# iroh / webrtc
pcc view --transport iroh --ticket <ticket> --token <token>
pcc view --transport webrtc --offer <offer-blob> --token <token>
# print ANSWER: + CANDIDATE: lines, paste them back to the sharer
```

| Flag | What it does |
|---|---|
| `--connect <host:port>` | Direct path to the sharer. With `--relay` too, both race and the first session wins — one command, no `--reach` guessing. |
| `--relay <host:port,…>` | Relay(s) to probe in order. Requires `--session`. |
| `--session <code>` | Relay rendezvous code (required with `--relay`). |
| `--pin <fp>` | Sharer's certificate fingerprint. Required for QUIC paths (it authenticates the connection, not the CA chain). Unused with `iroh` (endpoint id in the ticket is self-certifying) or `webrtc` (DTLS fingerprints ride in the SDP). |
| `--token` | **Required.** The secret the sharer printed. Wrong token = refused before key agreement. |
| `--no-window` | Headless: print periodic status instead of opening a window. Without the `native-viewer` build feature every session is this. |
| `--reconnect` | Reconnect after a dropped session instead of exiting. Offers the last applied (epoch, rev) in `Hello`, so the sharer replays the revision ring when it still covers the floor; otherwise a snapshot. The last frame stays on screen across the gap. |
| `--transport`, `--ticket`, `--offer` | Mirror the sharer's transport (default `quic`). |

The viewer reconstructs through the `Compositor` — atomic transactions
against one revision+epoch authority — and presents. A viewer that falls
behind gets a fresh snapshot, never silently dropped patches later ones
depended on. Each viewer has its own E2E keys; per-viewer revocation is
meaningful on the broadcast path.

## `pcc relay` — bridge two NATs

```sh
pcc relay --listen 0.0.0.0:5900
```

Prints its own token and fingerprint, then pairs hosts with viewers by
session code and forwards framed bytes. Default port `5900`; `--token`
auto-generates if omitted. **The share's `--token` must equal the relay's
`--token`** — the relay checks `HMAC(session-token)` on registration and
closes mismatches with `bad credential`, and the viewer's `--token` must
match too for the E2E handshake after it (all three verified live just
now: `Relay connected (session …)` then `Snapshot installed` only when
every token agreed). The relay holds the derived value, never the secret
behind the E2E handshake — it cannot forge a viewer proof or read a
stream. Per-viewer byte budgets, generational ownership, a defined
slow-viewer policy. ~570 bytes/frame ⇒ ~160 MB/viewer/hour;
`deploy/relay/` has a free-tier setup. Without the `audio` build
feature every session is video-only.

## `pcc diagnose` — what can this machine do?

Connects to nothing. Answers "do I need a relay?" and "what do I share?".

```sh
pcc diagnose            # network report: local addrs, global IPv6?, default route, STUN reflexive addr, the path it would take
pcc diagnose --displays # `display N: WxH` lines for --display/--region
pcc diagnose --audio    # input devices, loopback taps flagged [loopback]
```

Only the STUN rung is actively probed (one UDP round trip to a public
server); the rest is local inspection.

## `pcc pair` — one line instead of retyped hex

```sh
pcc pair --listen <host:port> --pin <fp> [--token <t>]
```

Prints a single `pcc://view?connect=…&pin=…&token=…` line carrying token
and pin (token generated if omitted) — open it or paste it, nobody
retypes 64 hex characters.

## Library, example, benches, checks

- **Library** (`src/lib.rs`): `app` (share/view), `capture`, `encoder`,
  `network`, `pcc` (detector/planner/compositor), `relay`, `server`
  (renderer/web), `audio`, `reach`, `telemetry`, `codec`. Re-exports:
  `CaptureSource`, cursor samplers, `ScreenCapture`, `NetworkConfig`,
  `NetworkResilience`, `ResilienceConfig`, `SessionToken`, `PCCDetector`,
  `QualityConfig`, `SharedSurface`.
- **Example**: `cargo run --release --example simple_screen_share` —
  the real pipeline in one process (planner → encoder → wire →
  transport → compositor), asserting pixel-identity and printing
  bandwidth saved. Live just now, 1280x720×90 synthetic frames:
  38,545 wire bytes vs 248,832,000 full-frame (99.98% saved, 6455×);
  then the same stream over real loopback QUIC — 90 frames in ~97 ms,
  53,825 bytes, 90 updates reconstructed exactly.
- **Benches**: `cargo run --release --example benchmarks` (must be
  `--release`; debug numbers are meaningless). Detection, planning, and
  snapshot-encode costs.
- **Smoke**: `bash scripts/smoke.sh` — the *only* check that exercises
  the CLI end to end (direct QUIC + pin, browser handshake, MJPEG
  fallback, relay path, diagnose, pair, metrics, JSON logs). Never run
  two at once: it `pkill`s its binaries by name.
- **Browser client check**: `node scripts/browser-client-check.mjs
  <pcc.js> <ws-url> <token>` — runs the *shipped* `client.js` text
  against a live sharer. Never verify the browser path with a
  reimplementation; two of them agreed with the server byte-for-byte
  while the shipped file was broken.
