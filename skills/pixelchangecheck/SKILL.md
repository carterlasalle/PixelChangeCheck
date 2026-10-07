---
name: pixelchangecheck
description: Use when running, configuring, explaining, or troubleshooting PixelChangeCheck (`pcc`) screen sharing — the share/view/relay/pair/diagnose commands, the interactive menu, tokens vs pins vs session codes, relay and browser setup, transports, observability, and the `pcc`/Portable-C-Compiler collision.
---

# PixelChangeCheck (`pcc`)

Lossless desktop replication: the sharer diffs each captured frame against the
framebuffer viewers actually hold and sends only what changed. A static screen
costs a few hundred bytes per frame; when nothing changes it sends a keep-alive.

Sharer and viewer must be the **same build** (wire protocol v7). A mismatch is
refused with a message naming both versions, not mis-parsed.

## Start here: the interactive menu

```sh
pcc
```

Bare `pcc` on a terminal walks through every choice — what to share, how viewers
connect, audio, approval — then prints the exact command and, crucially, **what
the other person runs**. It offers to run it. Nothing is hidden behind flags.

`pcc menu` forces the menu even when stdin is not a terminal.
Bare `pcc` off a terminal prints the help instead, so scripts are unaffected.

The menu is a front end, not a second implementation: it builds an argv and
feeds it through the same parser as a typed command, so a menu-built command
fails exactly like a typed one and the two cannot drift.

**It also parses a pasted invite.** Choose *View → A link* and paste a whole
`pcc://view?...` line; both long opaque strings in it (the token, and a pin that
is the *relay's* in one form and the *sharer's* in the other) are easy to
transcribe wrongly. A pasted `https://...` browser link correctly runs nothing —
it says to open it in a browser.

## The three secrets, and which is which

This is the single most common source of confusion. There are three values, and
two of them look identical (64 hex characters).

| Value | What it is | Who must agree |
|---|---|---|
| **`--token`** | The secret. Whoever has it can see the screen. | The **relay**, the **sharer**, and **every viewer** must use the *same string*. Mismatch = `bad credential`. |
| **`--pin` / `--relay-pin`** | SHA-256 of a certificate. Public; proves *who you are talking to*. | Direct: the **sharer's** pin. Through a relay: the **relay's** pin. They are different values and look the same. |
| **`--session`** | Rendezvous code. Not secret — it only names the room. | Sharer and viewers must match. Auto-generated and printed if omitted. |

The token travels to the relay as `HMAC(token, "pcc/relay/v1")`, never in the
clear, so a relay operator cannot forge a viewer's proof. The relay never sees
the screen.

## Install, and the `pcc` name collision

```sh
cargo install cargo-binstall          # once
cargo binstall pixel-change-check-client   # prebuilt, no compile
```

**If `pcc share` answers with `clang: error: ...`, you are running the wrong
binary.** macOS ships a *Portable C Compiler* that also answers to `pcc`
(`/opt/homebrew/bin/pcc` on Homebrew systems). Diagnose and fix:

```sh
which -a pcc          # the first hit is what actually runs
pcc --version         # must print "pcc 0.1.x" — anything else is not this tool
```

Put cargo's bin first, at the **end** of `~/.zshrc` (later prepends win):

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

then `hash -r`. When PATH cannot be changed, use the absolute path:
`~/.cargo/bin/pcc share`.

Two related traps:

- **The old name lingers.** Up to 0.1.3 the binary was
  `pixel-change-check-client`; since 0.1.4 it is `pcc`. Both can be on PATH, and
  `cargo uninstall pixel-change-check-client` resolves the name from the
  *current* manifest, so it deletes `pcc` and leaves the stale old binary
  behind. Remove it by hand: `rm ~/.cargo/bin/pixel-change-check-client`.
- A stale binary reports an old version. Check `pcc --version` when something
  behaves like an older release.

There is no `pixel-change-check` command; the package is
`pixel-change-check-client` and the binary is `pcc`.

## Every command

### `pcc share` — be the host

```sh
pcc share                     # primary display, direct viewers + browser on loopback
pcc share --window "Firefox"  # one window, matched by title substring
pcc share --region 100,200,1280,720
pcc share --application "Code"
pcc share --synthetic         # animated test pattern; headless machines and tests
```

| Flag | Notes |
|---|---|
| `--listen <host:port>` / `--no-listen` | Direct QUIC listener. Default `0.0.0.0:5800`. |
| `--relay <host:port,...>` + `--relay-pin <fp>` | Register on one or more relays. The pin is the **relay's**. |
| `--session <code>` | Relay room name. Auto-generated and printed if omitted. |
| `--web <host:port>` / `--no-web` | Browser viewer. Default `127.0.0.1:8080`. |
| `--web-cert` / `--web-key` | Required for a **non-loopback** `--web`: serving the viewer JavaScript over plaintext lets a network attacker replace it and steal the session. Loopback stays plaintext for development. |
| `--display N` / `--region x,y,w,h` / `--window <t>` / `--application <n>` | Capture source, picked at the capture layer. `--region` conflicts with `--display`. |
| `--audio-source mic\|system\|both\|none` | Default `none`. `system` uses an OS loopback tap when one exists, else falls back to the mic with a warning. `--audio` is a compatibility alias for `mic`. |
| `--approve` | Prompt `y`/`N` per viewer. Closed stdin admits (tests, daemons). |
| `--fps` / `--max-fps` / `--quality` | `--quality` is only the **lossy browser preview**; the authoritative stream is lossless. |
| `--reach auto\|direct\|relay` | `auto` tries IPv6 → STUN → relay. `direct` refuses the relay. |
| `--transport quic\|iroh\|webrtc` | See *Transports*. |
| `--broadcast-above N` | Past N direct viewers, the newest is redirected to the first `--relay`. Needs `--relay`. |
| `--repair-secs N` | Backstop convergence snapshots; 0 = the 30s default. |
| `--stats-interval N` / `--metrics-listen <addr>` | Observability; see *Watching it work*. |

`pcc share` prints the certificate fingerprint and copy-pasteable viewer lines.
**Send the printed line** — it carries the right pin, which is generated at
startup and cannot be known before then.

### `pcc view` — be the viewer

```sh
# direct
pcc view --connect <ip>:5800 --token <token> --pin <sharer-pin>
# through a relay
pcc view --relay <ip>:5900 --session <code> --token <token> --pin <relay-pin>
# iroh (no ports, no relay to run)
pcc view --transport iroh --ticket <ticket> --token <token>
```

| Flag | Notes |
|---|---|
| `--connect` + `--relay` together | Both race; the first session to complete wins, so you never guess which will work. |
| `--pin` | Required for QUIC paths. Unused with iroh (the ticket is self-certifying) or webrtc (DTLS fingerprints ride in the SDP). |
| `--token` | Required, always. Wrong token is refused before key agreement. |
| `--reconnect` | Keeps the last frame on screen, then resumes: the sharer replays its recent revision ring, or sends a fresh snapshot if the viewer fell too far behind. |
| `--no-window` | Headless status output instead of a window. |
| `--ticket` / `--offer` | iroh ticket / WebRTC offer blob from the sharer. |

### `pcc relay` — bridge two NATs

```sh
pcc relay --listen 0.0.0.0:5900 --web
```

Both sides dial *out* to the relay, so neither needs an open port. It is TCP+TLS,
which is why it also traverses proxies that block UDP.

| Flag | Notes |
|---|---|
| `--token` | Generated and printed if omitted. The sharer's `--token` must equal it. |
| `--web` | Also serve the browser viewer on this same port, behind the same certificate. Page at `/v/<session>/`, socket at `/v/<session>/ws`. One port, two protocols, split by ALPN (`pcc` vs `http/1.1`). |
| `--cert` / `--cert-key` | A **stable identity**: restarts keep the same fingerprint so clients keep their pin. Without them the relay generates per process and the pin changes on restart. |

The **host** answers the browser handshake, not the relay, so the relay still
sees only frame sizes and timing.

**The relay prints a complete share line** — its own token and pin filled in,
with an address a remote side can dial (the interface the default route uses,
never the `0.0.0.0` it bound). Copy that line verbatim to the machine being
shared; do not reassemble one from placeholders. It also prints the viewer line
and, with `--web`, the browser link.

The one value that must match across all three sides is `--token`: the relay's,
the sharer's, and every viewer's. A mismatch is refused as `bad credential`.

### `pcc pair` — one line instead of retyped hex

```sh
pcc pair --listen <ip>:5800 --pin <sharer-pin> --token <token>    # direct
pcc pair --relay <ip>:5900 --session <code> --token <token>       # relay; fetches the pin
```

With `--relay` you may omit `--pin`: it is read from the relay's certificate
over the network (trust-on-first-use — compare it against whatever the operator
published). Emits a `pcc://view?...` line that can be pasted whole into `pcc
menu` or shared with a viewer.

### `pcc diagnose` — what can this machine do

```sh
pcc diagnose              # reachability: addresses, global IPv6?, default route, STUN
pcc diagnose --displays   # display indices and sizes, for --display/--region
pcc diagnose --audio      # input devices, flagging loopback taps
```

Connects to nothing except one STUN probe. Run it first when a connection
fails: it says which rungs of the reachability ladder are available.

## Transports

| Transport | Use it when |
|---|---|
| `quic` (default) | Normal. Direct UDP, with relays as fallback. |
| `iroh` | You want no open port and no relay to run. `pcc share --transport iroh` prints a ticket; the ticket *is* the address. |
| `webrtc` | Both ends can only exchange a blob by hand. Manual signalling: the sharer prints an `OFFER:`, the viewer answers and both trickle `CANDIDATE:` lines. |

## The flows

**Same machine or LAN (simplest).** `pcc share` → the viewer runs the printed
`pcc view --connect ... --token ... --pin ...`. The printed browser URL is
**loopback**, so it only opens on the sharing machine.

**Across the internet, viewer installs nothing.** On a reachable host:
`pcc relay --listen 0.0.0.0:5900 --web`. Note its token and fingerprint. On the
sharer: `pcc share --relay <host>:5900 --relay-pin <relay-pin> --token <relay-token>`.
Send the printed **browser link** (`https://<relay>/v/<session>/#token=...`).
The token rides in the URL fragment, which the browser never sends as part of a
request.

**Viewer with the binary but not the details.** Send them a `pcc://view?...`
line; they paste it into `pcc menu`.

**Through a disconnect.** Add `--reconnect` on the viewer.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `clang: error: no input files` | You are running Homebrew's Portable C Compiler. Fix PATH (above). |
| `bad credential` | The sharer's `--token` does not equal the **relay's** `--token`. It is one value across all three sides. |
| `token too short: N chars (min 8)` | `--token` has a minimum length. |
| Certificate/pin error | You used the **sharer's** pin where the **relay's** belongs (or vice versa). They are both 64 hex characters. |
| Browser warns about the certificate | The relay generated a self-signed identity. Pass `--cert`/`--cert-key`. |
| Browser cannot connect to `--web` on a non-loopback address | Refused on purpose: supply `--web-cert`/`--web-key`, or use the relay's `--web`. |
| Viewer hangs with no error | It has deadlines (10s relay Hello, 30s silence watchdog) and names the session when it gives up. Use `--reconnect` to keep retrying. |
| `no visible window matches` | `--window`/`--application` fail fast rather than silently sharing the display. Check the title substring. |
| `a SHA-256 fingerprint is 64 hex characters, got N` | The pin was truncated in a copy-paste. |

## Watching it work

```sh
pcc share --synthetic --stats-interval 5 --metrics-listen 127.0.0.1:9100
curl -s 127.0.0.1:9100/metrics | grep pcc_
```

Prometheus families worth knowing: `pcc_bytes_total` (and
`pcc_bytes_{patch,fill,copy,snapshot,preview}_total`), `pcc_frames_total`,
`pcc_changed_area_fraction`, `pcc_viewer_rtt_ms`, `pcc_oldest_pending_ms`,
`pcc_effective_fps`, `pcc_viewer_queue_depth`, `pcc_lag_events_total`,
`pcc_repairs_total`, `pcc_detect_seconds_*`, `pcc_plan_seconds_*`,
`pcc_encode_seconds_*`, `pcc_apply_seconds_*`, `pcc_first_exact_image_seconds_*`.

`--stats-interval` prints a table; `--log-format json` is what to attach to a bug
report. The metrics endpoint is unauthenticated — bind it to loopback.

Note `pcc_frames_total` and `pcc_bytes_total` are **sharer-side**;
`pcc_apply_seconds_*` and `pcc_first_exact_image_seconds_*` are **viewer-side**,
so ask the right process for them.

## Benchmarks

```sh
cargo run --release --example benchmarks   # --release is not optional
```

Measured on an M-series Mac at 1920x1080 with a sparse (cursor-sized) change:
full pipeline ~0.85 ms/frame, detect ~0.5 ms, plan+encode ~0.7 ms, **wire cost
~578 bytes/frame against a 6,220,800-byte raw frame (~0.01%)**, and an idle scan
allocates nothing.

A live 1280x720 synthetic session at 30 fps showed ~450 bytes/frame, 0 lag
events, `pcc_viewer_rtt_ms` ~0, and ~13 ms from connect to first exact image on
loopback.

These are the **mostly-static screen** numbers, which is the design point. A
full-page scroll changes nearly every pixel and is the documented weak spot; see
`docs/adr/0001-lossless-authoritative-surface.md`.

## Repository notes (for changing the code)

- `pcc/compositor.rs` is the only code allowed to mutate a viewer's surface;
  `server/renderer/client.js` is a port of it — change both in one commit.
- `app/share.rs` is the only writer of the authoritative surface.
- `relay.rs` never inspects frame contents, which is what keeps end-to-end
  encryption meaningful.
- `network/protocol.rs` owns the wire format; the browser parses the same bytes.
- `scripts/smoke.sh` is the only check that exercises the CLI end to end
  (`cargo test` passes while the printed pin does not work). Never run two at
  once — each begins by `pkill`ing the binaries.
- Releasing: `scripts/release.sh <version>` after writing the `## <version>`
  section at the **top** of `CHANGELOG.md`.
