# Spec: telemetry, delivery, reach, and audio

Status: **specified, not started.** Nothing in this document is
implemented. It exists so the work can be done in any order by anyone,
and so each piece can be judged against a definition of done rather than
a feeling.

Every workstream below is scoped against what the code does *today*. Where
that is wrong, the spec says so rather than assuming a better system.

Read `DESIGN.md` and `docs/adr/` first: this document builds on decisions
already taken and does not reopen them.

---

## 0. Shape of the whole thing

Six workstreams, deliberately ordered by what unblocks the rest:

| # | Workstream | Unblocks | Needs |
|---|---|---|---|
| 1 | Telemetry (logging + stats) | audio, reach, everything measurable | — |
| 2 | NAT traversal ladder | reaching non-LAN peers cheaply | 1 |
| 3 | End-to-end encryption | trusting a third-party relay | 2 |
| 4 | Delivery (release, image, crates) | installing at all | — |
| 5 | Audio + sync | — | 1 |
| 6 | One-command setup | everything above | 1, 2, 4 |

**Telemetry first is not a preference.** Sync is the one workstream that
cannot be built by eye: without per-frame age and per-viewer latency
numbers, "is the audio in sync?" is unanswerable. The other five are all
cheaper to do correctly with measurements in hand.

---

## 1. Telemetry

### Problem

`tracing` is wired but the subscriber is hardcoded to `INFO` in
`main.rs::init_logging()`. There is no level flag, no JSON, no file sink.
Per-frame behaviour is unobservable, so the adaptive paths
(`Pressure` in `app/share.rs`, the repair deadline) cannot be told apart
from "it seems fine".

### Interface

Three flags, on `share`, `view` and `relay`:

```
--log-level <error|warn|info|debug|trace>   default: info
--log-format <text|json>                   default: text
--log-file <path>                          optional; rotating
--stats-interval <seconds>                 0 disables; default 0
--metrics-listen <host:port>               loopback only unless set
```

`RUST_LOG` continues to work and overrides `--log-level` for
per-module filtering, so `RUST_LOG=pcc=debug,app.share=trace` is the
way to debug one subsystem.

### What is measured

Fixed-bucket histograms with atomics, hand-rolled. No dependency: a
20-bucket log-scale histogram is about forty lines, and the only consumer
is a text renderer.

| Group | Series |
|---|---|
| Latency | capture→present p50/p95/p99; time to first paint; time to first *exact* image |
| Bytes | by kind: patch / fill / copy / snapshot / keep-alive; plus peak single-message size |
| Work | detect, plan, encode, serialize, apply — one histogram each |
| Staleness | longest time any region has been wrong; stall count |
| Network | per viewer: queue depth, lag events, repairs, acked revision, RTT |
| Content | changed-area fraction per frame |

The changed-area fraction is the one that predicts which strategy ran,
and therefore the one that makes the planner's decisions reviewable
after the fact.

### Outputs

- `--stats-interval 5` prints a summary table to stderr. This is what a
  human running the tool locally actually wants.
- `GET /metrics` on `--metrics-listen`, Prometheus text format. Bound to
  loopback by default. The web port is token-authenticated; metrics are
  *not* a session and must not be exposed on it.

### Rules

- **Never log in the capture loop.** It runs at 10-60 Hz. Log
  transitions — viewer joined, fell behind, repairing, epoch bumped,
  quality reduced — and let stats carry the rate.
- **Structured fields, not sentences.** `serve_viewer` already has peer,
  revision and lag count; the compositor already has a `Rejected` reason.
  Emit them as fields so filtering works.
- **Every viewer gets an id**, carried on every span, so one noisy
  viewer's history is separable from another's.
- **Never log the token.** It is the only secret. The certificate pin is
  public and safe to log; a token is not, not even truncated beyond its
  length.
- The in-process numbers must mean the same thing as
  `benches/benchmarks.rs`, or the two will drift.

### Done when

`--log-format json` produces machine-readable events for a full share →
view session including join, lag, repair and disconnect;
`--stats-interval` reports every group in the table above; no log line
is emitted from inside the per-frame path; and `cargo test` covers the
histogram's bucket boundaries and percentile calculation.

---

## 2. NAT traversal ladder

### Problem

The relay solves exactly one thing: both peers are behind NAT with no
direct path. That is rarer than it looks. A relay is also a bandwidth
bill and a third party to trust, so it should be the last resort rather
than the first.

The order below is fixed. It is not a preference list; it is roughly
ascending in cost and descending in success rate.

```
1. IPv6 direct        free, best latency, no third party
2. UPnP / NAT-PMP     free, opens a port on the sharer's own router
3. STUN + ICE         free, resolves the large majority of NAT pairs
4. relay              guaranteed, costs money, needs trust
```

### Why the relay stays

Two cases that nothing free fixes:

- **Symmetric NAT on both sides.** No candidate pair works. ICE cannot
  save you.
- **Corporate proxies that block UDP.** This is the important one and it
  is easy to get backwards: the relay is **TCP + TLS**, so it already
  traverses firewalls that would kill QUIC outright. A "free" direct-only
  path is *less* reliable than a cheap relay behind a corporate proxy.

So the goal is not to remove the relay. It is to make the relay
unnecessary most of the time.

### Interface

```
pcc diagnose [--target <host:port>]   report reachability, connect to nothing
```

Output is a short report: IPv6 present and routable? UPnP mapping
obtained? STUN reflexive address? Which ladder rung a real connection
would have used, and roughly what latency?

```
pcc share --reach auto                default: try 1,2,3 then fall back
pcc share --reach direct              fail rather than use a relay
pcc share --reach relay               skip discovery, go straight to 4
```

`--reach direct` is a real feature: someone who knows they have a port
forwarded should not pay for STUN round trips every session.

### Done when

`pcc diagnose` correctly reports the rung for at least IPv6, UPnP
success, UPnP-absent, and no-IPv6 on a real network; `share --reach
auto` reaches a viewer over direct IPv6 with the relay argument omitted
and no relay traffic; and `--reach direct` fails loudly rather than
silently falling back.

---

## 3. End-to-end encryption

### Problem

The relay terminates TLS and forwards plaintext `Message` bytes to the
far side. The relay operator can therefore read the screen. The relay is
now a pure byte pipe — it caches nothing and inspects nothing — so this is
a small change with a large payoff.

This changes the security model, so it needs an ADR (`docs/adr/0004`).

### Interface

No new flags. The token already exists and is already the shared secret;
it simply gains a second job. Bump the protocol to v4 and have `Hello`
carry a key-agreement public value.

### Design

- A Noise-style handshake over the existing TLS channels, using the
  session token as the pre-shared key.
- Both directions get independent keys. Viewer→host control traffic must
  not be forgeable by a relay that holds the host→viewer key.
- The relay forwards ciphertext and learns nothing: no session code, no
  dimensions, no frame timing beyond packet sizes.
- Rekey on epoch change and on repair, so a long session does not
  accumulate one key's worth of risk.

### Done when

A relay whose process memory is dumped reveals no `Message` content, no
dimensions and no session code; the round-trip cost is measured and
acceptable; and a token mismatch fails the handshake with the same error
as today.

### Risks

Losing E2E would be a real regression, so the fallback must be explicit
and logged, never silent.

---

## 4. Delivery

### Problem

CI proves the code builds. It does not produce anything installable.

### Scope

- **Build matrix** — Linux, macOS, Windows, `--release`, tar/zip with the
  binary, uploaded as a workflow artifact.
- **Tag release** — on `v*`, build the matrix, create a GitHub Release,
  attach binaries plus checksums.
- **crates.io** — `cargo publish --dry-run` must pass. Prefer *trusted
  publishing* (OIDC from Actions, no long-lived secret in the repo).
- **Container** — multi-stage, distroless, non-root. Worth stating
  plainly: the sharer needs a display or `--synthetic`; the image is
  really for the relay and the viewer.
- **Supply chain** — `cargo audit` exists; add `cargo-deny` for
  licences and advisories, and an SBOM step.

### Decisions to make first

- Does the relay ship from the same release train?
- Are binaries signed, and is the macOS one notarised?

### Done when

A tag produces downloadable artefacts for three platforms, a container
image, and a crates.io release, and a clean checkout can install the tool
in one command on each platform.

---

## 5. Audio and audio sync

The largest workstream and the only one with a genuinely hard problem in
it. Three decisions, in order.

### 5.1 Transport

**Separate transport. Not a `Message` variant.**

Audio is lossy-tolerant, latency-critical, and must never queue behind a
video snapshot. Putting it on the compositor's path would give it exactly
the wrong semantics: the compositor is about atomic exactness, audio is
the opposite.

Use QUIC datagrams (RFC 9221) or a second stream. Datagrams are
unreliable, unordered and congestion-controlled, which is what audio
wants, and they cannot head-of-line-block the reliable pixel stream.

### 5.2 Codec

Opus. 20 ms frames, built-in FEC and packet-loss concealment, built for
exactly this. Bind via `audiopus`. No separate signalling needed.

### 5.3 Capture

The `screenshots` crate yields pixels only, and system-audio capture is
genuinely per-platform:

| Platform | Mechanism |
|---|---|
| macOS 14.2+ | ScreenCaptureKit, application audio |
| Windows | WASAPI loopback |
| Linux | PipeWire or PulseAudio monitor source |

Put it behind a `SystemAudio` trait with three implementations. **For the
first cut, make the source pluggable and default to a microphone or
virtual device** — that is testable on every machine including CI, and it
separates the capture problem from the sync problem.

### 5.4 Sync — the actual work

- Both streams are timestamped **on the same monotonic clock at capture**,
  and the viewer holds a per-viewer offset estimate. The protocol has a
  revision and no time, so this is a **protocol change: v4 adds `pts_us`**
  to snapshots and updates.
- **Audio is the master; video is slaved to it.** The audio device has a
  buffer and a latency nobody controls, so its clock defines "now". The
  renderer keeps a small jitter buffer of decoded frames and releases each
  when its presentation time matches the audio clock. This is WebRTC's
  `AudioMaster`/`VideoPlayout`; read it rather than reinvent it.
- **The invariant that protects everything else:** audio is never derived
  from the video surface, and the video surface is never altered to make
  audio line up. A late audio glitch is a presentation problem; corrupting
  the authoritative lossless surface to fix sync would undo every guarantee
  in `DESIGN.md`. Synchronise **at presentation only**.
- **Independent failure.** An audio stall must not stall video, or the
  reverse. Two supervisors, two clocks, one shared presentation decision.

### 5.5 Testing it

Not by ear. Emit a known tone alongside the video, then measure the
offset between the audio peak and the video flash in the recording. That
is automatable, and it is the only test that means anything.

### Done when

Measured audio-video offset is within an agreed bound (to be set from
telemetry, not guessed) for at least 5 minutes of continuous playback on
each platform; a dropped audio packet produces a glitch and not a video
stall; and the round trip degrades audio before it degrades pixels.

---

## 6. One-command setup

The user's actual question: *how do I just run this?* Today the answer is
copy a token and a 64-character fingerprint between two terminals. That
is the single largest usability gap, and it is also what makes the
reachability work above feel unnecessary — if setup is painful, people
just use the relay.

### Scope

- **`pcc pair`** — the sharer runs it and prints one self-contained URL.
  The viewer scans or pastes it. Carries the token and the certificate
  pin, so the fingerprint is never retyped.
- **`pcc doctor`** — one command that answers: does capture work, is
  audio available, is a display present, which reachability rung applies,
  and what the measured frame cost is.
- **A relay you can run for free.** An Oracle Cloud Always Free tier
  guide plus a `deploy/relay` script that provisions a 1 GB VM and
  installs the relay as a systemd unit. Oracle's free tier is genuinely
  permanent; the signup is painful and capacity is often unavailable, so
  the script should verify the instance actually came up and say so if
  it did not.

### Done when

A new user goes from clone to a working share, on a machine behind NAT,
without typing a fingerprint, and without needing to know what a
certificate pin is.

---

## Explicitly not in scope

- **Inter-frame video.** Blocked upstream: no encoder can be linked (see
  `src/codec/mod.rs` for what was tried). The codec registry and its
  identifiers are stable so adding one later is a single module.
- **Input injection.** No keyboard, mouse or clipboard forwarding, so
  there is no input-injection surface to defend.
- **Platform damage metadata** for capture (DXGI, ScreenCaptureKit,
  PipeWire). A real saving on capture cost, independent of everything
  above.
- **A hosted relay service.** The relay is something you run, not
  something we sell.
