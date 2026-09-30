# ADR 0006: Iroh and WebRTC transports are named but not implemented

- **Status:** partially superseded by ADR 0007 (iroh) and ADR 0008
  (webrtc). The WebRTC half below is historical; the iroh half is
  replaced.
- **Supersedes:** nothing.

## Context

The session runs on QUIC directly or through the relay fan. Two more
transports keep coming up: **iroh** (peer-to-peer with relay fallback,
no user-run relay needed) and **WebRTC** (browser-native peer
connection, no WebSocket bridge). Both are real answers to real gaps —
NAT traversal without operating a relay, and a browser viewer without
the WebSocket shim.

Both are also heavy. Iroh is a new dependency tree (iroh, iroh-net,
iroh-relay) with its own relay infrastructure to choose or run; WebRTC
is a new dependency tree (webrtc-rs) plus signalling, which this
project currently has no channel for. Either one lands as thousands of
lines before the first pixel flows, and neither is needed for the
sessions the tool serves today.

## Decision

Name both transports in `TransportKind` (`quic | iroh | webrtc`),
parse them in the CLI, and fail loudly at session setup with a pointer
here. Do **not** add stub crates, stub modules, feature flags, or
half-wired code paths for them.

The seam that matters already exists: `MessageTransport` is the only
contract a session needs, and both future transports land as new
implementors behind an existing enum variant. No flag day, no protocol
change — the `Message` envelope does not care which socket carries it.

What would reopen this:

- **Iroh:** a deployment where neither side can run or reach a relay.
  Needs: iroh endpoint + ticket signalling out of band (the `pair` URL
  is the natural carrier) + a relay-hosting decision.
- **WebRTC:** a browser viewer that must not go through the sharer's
  WebSocket bridge. Needs: signalling (same `pair` question) + codec
  decision (the lossless surface is not video; a data channel carries
  `Message` envelopes, not an encoded stream).

## Constraints

- `--transport iroh|webrtc` fails at startup with the ADR number and
  the working alternative. A mid-handshake failure would read as a
  network problem; a missing-dependency panic would read as a bug.
- No `iroh`/`webrtc` crates in `Cargo.toml` or `Cargo.lock` until an
  implementation lands. A dependency that compiles but does nothing is
  attack surface with no feature behind it.
- The `TransportKind::require_implemented` test pins the gate: new
  variants must fail loudly by default, not silently ride the QUIC path.

## Consequences

- CLI help advertises transports that do not work yet. That is
  deliberate: discovering the answer at `--help` time with a pointer to
  the reasoning beats discovering it at handshake time with a panic.
- When either lands, this ADR is superseded by the implementation ADR,
  and the corresponding `require_implemented` arm flips to `Ok`.
