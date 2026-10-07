# Changelog

## 0.1.10 — a viewer through a relay is no longer dropped

### Fixed

- **The relay lost every viewer a second after its first snapshot.** The
  relay's forwarding loop read the socket inside a `tokio::select!`, and
  `read_exact` is not cancel-safe: when the write arm won the race — which
  it does exactly while the relay is streaming a snapshot back — the
  in-flight read was dropped after consuming the 4-byte length but before
  the payload. The next frame was then decoded from the middle of the
  previous one, so the relay saw a nonsense length and disconnected the
  peer with `Framed message too large`. Loopback rarely lost the race and
  hid it; a viewer over the tailnet lost it every time. The read now runs
  in its own task and the loop awaits a channel, which is cancel-safe.
  Verified by the scenario that reproduced it: a Mac sharing into a relay
  on a VPS with a second process watching through it.
- **`Framed message too large` now names the bytes it read**, not only the
  decoded length. A misaligned stream is the usual cause and the raw
  prefix is what distinguishes it from a peer genuinely sending something
  absurd.

### Changed

- **The menu no longer reads the relay store itself.** It is passed in, so
  a test on a machine that has a real `~/.config/pcc/relays.json` behaves
  the same as one that does not. Adding the saved-relay picker broke four
  menu tests that way, on a machine where a relay happened to be saved.

## 0.1.9 — pick a relay, paste nothing

### Added

- **Remembered relays.** `pcc relay --remember <name>` saves the relay's
  address, certificate pin and token; `pcc share --use <name>` (and
  `pcc view --use <name>`) then need none of them pasted. The store lives at
  `$XDG_CONFIG_HOME/pcc/relays.json` (`PCC_RELAYS` overrides it), is written
  `0600` because it holds a token, and is the `~/.ssh/config` +
  `known_hosts` pattern rather than a certificate authority: the pin is
  recorded at save time and still re-checked against the live certificate,
  so a relay that changes identity is refused. The menu lists saved relays
  as a numbered pick before offering to paste anything.
- **One paste instead of four values.** `reach::parse_relay_hint` accepts
  whatever the operator copied — the relay's full printed line, bare
  `--relay/--relay-pin/--token` flags, or a `pcc://relay?...` URL — and
  names exactly which value is missing when the paste is partial. Sharing
  through a relay used to mean hunting four separate values out of another
  terminal's scrollback.

### Fixed

- **A relay on the same machine was unreachable.** Sharing to a relay
  running on the same host dialed that host's own public address, which
  clouds route out and back or drop outright: the session failed with
  "could not connect to the relay" while a healthy relay was listening.
  Addresses that name this machine are now rewritten to loopback before
  connecting, and the relay prints the loopback form alongside the public
  one. Verified on a Hetzner VPS, which is where it was hit.

### Changed

- **The menu is presentable.** A wordmark banner, framed panels around the
  command and the instructions for the other side, a `❯` marker on the
  default answer, bold labels separated from dim explanations, and colour
  for warnings and the command itself. Colour is opt-out: it appears only
  on a terminal with `NO_COLOR` unset and `TERM` not `dumb`, so piping the
  menu still yields clean greppable text (the smoke suite relies on it).

## 0.1.8 — the relay prints the command for you

### Fixed

- **The relay printed placeholders instead of values.** It knew its token and
  its pin and still told you to run
  `pcc share --relay <this-host>:5900 --relay-pin <the pin it printed> --token <its token>`,
  and its browser line said `https://0.0.0.0:5900/v/<session>/#token=<token>` —
  an address nobody can dial, with the token it had just generated hidden. It
  now prints a complete `pcc share --relay …` line with the real token, the real
  pin, and an address from the interface the default route uses (a wildcard bind
  is not an address). Copy one line to the machine being shared.
- **The menu generated a token for relay shares.** The share's `--token` must
  equal the *relay's*, so a generated one guaranteed `bad credential` — the
  exact failure the menu exists to prevent. It now asks for the relay's token
  and stops rather than starting a session that cannot connect.
- **The menu asked for values without saying where they come from.** The relay
  address prompt now says it is the address the relay printed, offers
  `127.0.0.1:5900` for a relay on this machine, and says what to do if you have
  no relay yet. The relay branch no longer prints `<this-host>` placeholders;
  it explains that the relay prints the real line, and that it holds the
  terminal.

## 0.1.7 — run `pcc` and answer questions

### Added

- **An interactive menu.** Bare `pcc` (and `pcc menu`) walks through what to
  share, how viewers connect, audio, and approval, then prints the exact command
  plus the command the other side runs, and offers to start it. It builds argv
  and feeds it through the same parser as a typed command, so it cannot drift
  from the flags and its errors are identical. Empty input quits rather than
  falling through to the first option; bare `pcc` off a terminal still prints
  help, so scripts are unaffected.
- **`pcc://` invites can be pasted whole.** `reach::parse_pair_url` is the
  inverse of `pair_url`/`pair_url_relay`, used by the menu's *View → A link*.
  Both long values in an invite are easy to mistype and impossible to tell apart
  once you have, and the relay form puts the *relay's* pin where the direct form
  puts the *sharer's*. A pasted browser link is correctly treated as "open this",
  not as a command.
- **A skill for coding agents** at `.omp/skills/pixelchangecheck/SKILL.md`:
  every flow, the three secrets (token vs pin vs session), command reference,
  troubleshooting, and the observability recipes.

### Changed

- `pcc` no longer requires a subcommand. It was an error to run it bare; now it
  is the entry point.

## 0.1.6 — one-command releases, and the easy path up front

### Changed

- **A portable release script.** `scripts/release.sh <version>` runs every
  gate, bumps the manifest, tags, pushes, and watches the release run. It
  no longer assumes this development machine's toolchain path: it uses
  whatever `cargo` is on `PATH` and falls back to rustup's `stable`
  toolchain, reads the repository slug from the remote, and picks the run
  for the tag rather than the latest one.
- **The README leads with the easy path.** A "30-second version" and a
  "one-link setup" now come first, followed by an everyday-tasks table
  (share a display/region/window/app, audio, approval, iroh, reconnect).
  The relay walkthrough no longer appears twice.

### Fixed

- **Changelog order.** `0.1.5` had been appended below `0.1.4`/`0.1.3`,
  and a leftover `Unreleased` heading sat below `0.1.1` while describing
  work that had already shipped. The releases are now newest-first and
  the leftover material is labelled as background.

## 0.1.5 — the relay-hosted viewer release

### Added

- **The relay can host the browser viewer.** `pcc relay --web` serves the
  viewer page and its WebSocket on the relay's own port, behind the
  relay's own certificate: a viewer needs no binary, no terminal, and no
  `--web-cert` files, just a URL. The page lives at
  `/v/<session>/#token=...` and the socket at `/v/<session>/ws`. The host
  answers the browser's WebCrypto handshake, not the relay, so the relay
  still sees only frame sizes and timing. ALPN decides what a connection
  is (`pcc` for the relay protocol, `http/1.1` for a browser), so one
  port serves both.
- **`pcc pair --relay` fetches the relay's pin.** Omit `--pin` and the
  relay's fingerprint is read from its certificate over the network, so
  nobody copies 64 hex characters out of a journal. `pcc share --relay`
  now also prints a one-line `pcc://` viewer link and the equivalent
  browser link.

### Fixed

- **Bare `pcc share` works.** `--web` defaults to loopback, so the TLS
  guard no longer fires on the default invocation; the README sample
  shows the loopback browser URL the build actually prints.
- **Pins are labelled.** Direct lines say "pin the sharer", relay lines
  say "pin the relay" and print the exact relay viewer command with the
  relay pin substituted; the direct line prints the LAN address instead
  of `0.0.0.0`. `pcc pair --relay/--session` emits the relay form.
- **Viewer stops hanging.** Relay Hello has a 10s deadline naming the
  session, and the apply loop ends the attempt after 30s of silence;
  `--reconnect` keeps retrying, otherwise the viewer exits non-zero
  with the cause instead of waiting forever.
- **Old clients get the real reason.** Registrations carry the protocol
  version; a pre-0.1.5 client is refused with "upgrade the client",
  naming both versions, instead of `bad credential`. The role in the
  message is the actual role.
- **Relay identity persists.** `pcc relay --cert/--cert-key` loads a
  stable PEM identity, so restarts keep the fingerprint and clients
  keep their pin. Generated identities print that the pin changes on
  restart.
- **Oversized updates prefer snapshots.** The planner reserves AEAD +
  framing headroom against the send budget, so a Retina-sized patch set
  becomes a snapshot instead of an oversized sealed write that desyncs
  the stream. Sealed-frame auth failures request a fresh snapshot once
  instead of killing the session.
- **Relay token stays out of logs.** The relay prints its token only
  when it generated it; bare TCP connects log at debug instead of WARN.
- **HiDPI windows fit.** The native viewer opens `FitScreen` with
  aspect-ratio stretch instead of 1:1 physical pixels, and is
  resizable.

### Added

- **Revision replay ring.** The sharer retains the last 512 broadcast
  payloads (16 MiB cap, epoch-cleared), and a viewer that lags a few
  updates behind gets those exact bytes replayed instead of a whole
  snapshot. Far-behind, evicted, or cross-epoch viewers still get a
  snapshot. Idle keep-alives dedup so they cannot churn real history.
- **Per-viewer ack watermark.** The sharer records each viewer's newest
  applied revision and feeds the gap into the fps pressure loop: a
  viewer acking every revision is merely slow, one whose watermark has
  stalled weighs double.
- **Adaptive audio jitter hold.** Arrival variance drives a 20–100 ms
  hold before a missing frame is concealed; stable links stay near one
  frame, jittery links grow enough to stop glitching.
- **Opus loss concealment and in-band FEC.** The encoder emits FEC, and
  the viewer fills each pts gap in order — FEC for the first missing
  frame, concealment for the rest — instead of leaving a hole.
- **Congestion control feeds the planner.** Each direct viewer samples
  QUIC RTT, lost packets, and congestion window per send; fps backs off
  on high RTT or new loss before queues fill, quality degrades on a
  collapsing window, fps recovers only on a healthy path, and the worst
  RTT / deepest ack gap / oldest pending age are exported to metrics.
- **Remote browser mode requires HTTPS.** Serving the viewer JavaScript
  over plaintext HTTP lets a network attacker replace it and steal the
  session secret or the screen after decryption, which application-layer
  sealing cannot fix. A non-loopback `--web` address without
  `--web-cert`/`--web-key` now fails fast with the fix; loopback stays
  plaintext for development.
- **Capture picker: display and region.** `pcc share` takes `--display N`
  (see `pcc diagnose --displays`), `--region x,y,w,h`, `--window`, or
  `--application`. Display and region capture at the layer via the OS
  enumeration (`Screen::all`, `capture_area`); the region is verified
  against what the platform returns and clamped so geometry is stable.
  Window/application are the seam for the platform pickers and fall back
  to the display with a warning rather than failing the share.
- **Remote cursor plane (protocol v7).** The sharer samples the pointer
  once per frame and sends `CursorMove`/`CursorHide` only on change (2px
  dead band); viewers draw the arrow as a presentation-only overlay the
  compositor never sees. Real shares have no sampler linked (no
  cross-platform cursor API exists) so they hide the cursor;
  `--synthetic` sweeps a scripted pointer end to end. Native and browser
  viewers both render it.
- **Motion previews (protocol v7).** When the changed-area fraction hits
  40%, the sharer also emits a `MotionPreview` interim rectangle so a
  viewer on a slow path shows something instead of a frozen fling. Never
  authoritative: the exact update still decides, stale previews are
  dropped, and bytes are counted under `pcc_bytes_preview_total`.
- **Audio source selection.** `pcc share --audio-source mic|system|both|none`
  (plus `--audio` as a mic alias, `pcc diagnose --audio` to list). `system`
  uses an OS loopback tap (Pulse monitor, virtual cable) when one exists
  and falls back to the mic with a warning — the WASAPI/PipeWire/
  ScreenCaptureKit backends are still the seam, not the code. `both`
  mixes mic and loopback with a saturating add on one owner thread.
- **Host approval gate.** `pcc share --approve` prompts `Admit? [y/N]`
  per viewer after the token checks out; refusals get an Error with a
  wait, like a bad token. Closed stdin admits (tests, daemons) — the
  token stays the secret either way.
- **Relay geography.** `--relay` takes a comma-separated list; the sharer
  registers on every one and viewers probe in order, first handshake
  wins. A dead relay costs a timeout, not the session.
- **Path migration.** `pcc view --connect X --relay Y` races both and
  the first completed Hello wins; reconnects re-race with the same
  resume point. `Redirect` (protocol v7) is the handoff the other way:
  the viewer follows it to the named relay session without losing its
  place.
- **Broadcast mode.** `pcc share --broadcast-above N --relay R` flags the
  newest direct viewer past the threshold; its serve task sends one
  `Redirect` and ends the direct session. The viewer resumes through the
  relay, so fanout costs one socket instead of N direct streams.
- **Live platform cursor.** `--synthetic` sweeps a scripted pointer;
  real shares sample the OS pointer (`device_query`) at the capture
  origin, hide it outside the shared area, and viewers draw it as a
  presentation-only overlay the compositor never sees.
- **Window and application capture.** `pcc share --window <title>` and
  `--application <name>` match by substring (`xcap`) and re-resolve every
  frame, so a close is a freeze and a move is followed; a miss fails fast
  instead of sharing the display. Geometry comes from the captured image
  (physical pixels on Retina), and the epoch logic absorbs the change.
- **System-audio taps that exist.** `--audio-source system|both` uses an
  OS loopback tap when one is named (Pulse monitor, BlackHole, VB-Cable,
  Background Music, Zoom and friends); otherwise the mic with a warning.
  `pcc diagnose --audio` flags the taps it found.
- **Iroh transport (ADR 0007).** `pcc share --transport iroh` prints a
  ticket; `pcc view --transport iroh --ticket <ticket>` dials it — no
  ports, no relay to run. Verified two-process: 1280x720 snapshot at
  revision 219 over the ticket.
- **WebRTC transport (ADR 0008).** `pcc share --transport webrtc` prints
  an offer blob; the viewer answers with one blob and both sides trickle
  ICE as pasted lines. One reliable ordered `pcc` data channel carries
  the same `Message` envelopes, chunked at 16 KiB. Verified two-process:
  1280x720 snapshot over offer/answer/trickle.

### Changed

- **Installs skip the compile.** Release archives are named
  `pcc-<version>-<target>` (`.tar.gz`, `.zip` on Windows) and
  `[package.metadata.binstall]` points at them, so
  `cargo binstall pixel-change-check-client` downloads instead of
  compiling 700+ crates. The release workflow builds them with the new
  `dist` profile (full optimization, paid once per release); the
  `release` profile that `cargo install` uses is now thin-LTO with 16
  codegen units, which builds a large multiple faster for a low
  single-digit runtime cost the benches can measure.
- **Platform-heavy surfaces are opt-out features.** `audio` (cpal +
  Opus, including the vendored libopus cmake build) and `native-viewer`
  (minifb, i.e. the X11/Wayland dev headers) default on, so plain
  installs behave exactly as before; `--no-default-features` drops both
  for a headless relay or minimal viewer. Without `audio` every source
  resolves to silence, without `native-viewer` every session is
  headless — no new flags, no behavior change when the features are on.
- **Tokio is trimmed to what the crate uses.** `rt-multi-thread`,
  `macros`, `net`, `sync`, `time`, `io-util`, `io-std` instead of
  `full`. `process` and `signal` still compile (iroh's tree requires
  them), so the saving is one fewer `parking_lot` edge today and no
  silent growth tomorrow: the list now says what is ours.
- **Dead dependency removed.** `num_cpus` was referenced nowhere in
  `src/`, `tests/`, `examples/` or `benches/`; the line and its
  lockfile entry are gone.

## 0.1.4

### Changed

- **ARM Linux archive.** The release matrix gains `ubuntu-24.04-arm`
  (native ARM64, no cross-compiling), so `cargo binstall` serves
  `pcc-<version>-aarch64-unknown-linux-gnu` instead of falling back to
  a full local compile there. No code change; same `dist` profile and
  triple-named archive scheme as the other three legs.
- **MSRV is 1.91.** iroh 1.3 requires rustc 1.91, so the floor moves
  from 1.88 everywhere it was claimed (`rust-version`, CI, README,
  release runners). 1.88 was the `time`/RUSTSEC-2026-0009 floor; iroh
  moved it.
- **Headless and Windows test fixes.** The cursor sampler uses
  `DeviceState::checked_new` (no X display reads as `CursorHide`
  instead of panicking); window/application capture resolves the match
  before touching the framebuffer (headless still fails fast with "no
  visible window matches"); the closed-stdin test reads EOF from memory
  instead of `/dev/null`.
- **OSV-Scanner ignores with receipts.** `osv-scanner.toml` mirrors the
  `.cargo/audit.toml` policy for the seven flagged advisories, each
  entry naming what upstream release lifts it.

### Added

- **`docs/usage.md`.** Every entry point — all five commands, every
  flag, the browser routes, the three transports — each verified live
  against the binary. Also corrects the relay token story: the share's
  `--token` must equal the relay's `--token` (HMAC-checked on
  registration), and the viewer's must match too for the E2E handshake.

## 0.1.3

### Changed

- **Installs and builds.** Prebuilt `pcc-<version>-<target>` archives
  and `cargo binstall` support; the `release` profile builds fast
  (thin LTO) while CI ships a full-optimization `dist` profile; `audio`
  and `native-viewer` are opt-out features; tokio is trimmed to the
  named set; `num_cpus` removed. Protocol version 7 two releases in:
  sharers and viewers must be the same build.

## 0.1.2

### Fixed

- **The binary is `pcc`.** The README, the help text and every printed
  viewer command have always said `pcc`, but Cargo took the name from the
  package, so `cargo install pixel-change-check-client` produced
  `pixel-change-check-client` and every documented command failed. It is
  a breaking change for anyone who installed 0.1.0-0.1.2 and is used the
  old name.
- **The session secret moved to the URL fragment.** It was in the query
  string, which is sent in the request line and kept in browser history,
  and which can leak through server logs, proxy logs and referrers. The
  page and the script carry no secret, so they are now served to anyone;
  the WebSocket handshake is where the token is checked, before a byte of
  surface data can move. The page scrubs the fragment from the address
  bar once it has read it.
- **A race in the browser client dropped frames.** `onmessage` was async
  and a WebSocket does not await its handler, so a sealed frame could be
  opened while the handshake before it was still deriving keys -- failing
  with "not sealed yet" on a session that was about to work. Frames are
  now handled strictly in order. Before: 7 passes in 10. After: 15 in 15.
- `setup.sh` was missing `libasound2-dev` and `libudev-dev`, so it built
  a tree the audio path cannot link. It now installs the same list CI
  does, and prints the real commands.

The browser viewer did not work. Three separate bugs, all in the
end-to-end encryption path, and none of them visible to a test that
reimplements the protocol:

- The proof of token possession never included the token. The browser
  computed `SHA-256(prefix || "proof" || public_key)` and the sharer
  expected `SHA-256(prefix || "proof" || public_key || token)`, so
  every browser handshake was refused and the page sat at "connecting"
  retrying forever.
- The handshake reply was delivered to two handlers. A WebSocket
  delivers every frame to every listener, so the sealed-data handler also
  saw the plaintext reply, tried to open it, and killed the session.
- Acknowledgements and keyframe requests were sent unencrypted. The
  sharer could not open them and dropped the connection -- which is what
  the retry loop was.

- The browser surface was RGB in the compositor and RGBA after a PNG
  snapshot, while the paint path handed it to `ImageData`, which is
  RGBA and nothing else. Every patch applied to a PNG-seeded surface
  landed on the wrong pixels. The surface is now RGBA throughout and the
  wire format is still RGB; the widening happens on the way in.

- `pcc_detect_seconds` and `pcc_plan_seconds` recorded the same span, so
  neither said which phase was expensive. `plan` runs detection
  internally and now reports how long that took.

### Verification

- `scripts/browser-client-check.mjs` fetches the *served* `pcc.js`, runs
  that exact text under Node's WebCrypto against a live sharer, and
  completes a sealed session. Reintroducing either proof bug makes it
  fail. This exists because two reimplementations agreed with the server
  while the shipped file did not, and a check that is quietly weaker
  than it looks is worse than none.
- A real browser was driven end to end: `1280x720 rev 200`, 921,600
  pixels painted, all fully opaque.

### Security

- **The relay no longer holds the end-to-end secret.** The session token
  authenticates the encryption handshake, and it was also sent to the
  relay during registration, so a relay that learned it could compute a
  valid viewer proof and sit in the middle of a stream it claims not to
  be able to read. Peers now present `HMAC(token, "pcc/relay/v1")` to the
  relay and the token itself never leaves the two endpoints. The claim
  holds against the operator running the relay, which is the only
  adversary it was ever really about.

### Known: the relay serves one viewer

The host calls `serve_viewer` once for the whole relay connection, so
whichever viewer completes the handshake first establishes the keys and
every other viewer receives ciphertext it cannot open. The host also
cannot tell the viewers' traffic apart, because the relay forwards with
no peer identity. The relay path is correct for one viewer and wrong for
two. It is left as it is rather than half-fixed, and the reasoning is
recorded in `AGENTS.md` so the next attempt starts from the mechanism
rather than the symptom.

### Packaging and CI

- Every GitHub Action is pinned to a commit SHA rather than a moving tag,
  so whoever controls a tag cannot change what runs with the repository's
  credentials. Dependabot still updates the pins.
- `rand` moves 0.8.5 -> 0.8.6, clearing RUSTSEC-2026-0097 (unsound with
  a custom logger using `rand::rng()`). A semver-compatible patch, so it
  is fixed rather than ignored.
- `ci.yml` declares `permissions: contents: read`. It had no permissions
  block at all, so every job inherited the repository default, which is
  broader than any of them need. The crates.io publish job keeps
  `id-token: write`; nothing else needs it.

## 0.1.1

A security release. `0.1.0` shipped quinn 0.10.2 and rustls 0.21, and
carries two remote denial-of-service advisories against the QUIC
endpoint this product exposes to the internet. Upgrade to `0.1.1`.

## Earlier releases (0.1.0–0.1.4) — details

This material was written while those releases were being prepared and
was never folded into per-version headings. It is kept for provenance:
the protocol-version history, the dependency and advisory work, the
packaging metadata, and the browser-encryption design notes that
produced the current tree.

The wire protocol is version 6. Viewers and sharers must be the same
build; a version mismatch is refused with a clear message rather than
mis-parsed. Version 4 added the end-to-end encryption handshake; version
5 adds `pts_us` to every visual message, so a viewer can schedule a frame
against the audio clock; version 6 carries the viewer's last applied
(epoch, rev) in `Hello`, so a reconnect can replay the revision ring
instead of snapshotting. A reconnecting viewer keeps its last frame on
screen across the gap and catches up from the ring when it still covers
its floor; anything older, evicted, or cross-epoch snapshots as before.
Fresh session keys are derived on every attempt, so no counter or nonce
crosses the reconnect.

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
