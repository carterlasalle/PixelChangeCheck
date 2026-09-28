# 4. Encrypt between sharer and viewer, not just to the relay

Date: 2026-09-28

## Status

Specified, not implemented. See `docs/spec/roadmap.md` section 3.

## Context

The relay terminates TLS on both legs and forwards the `Message` stream
in the clear between them. Whoever operates the relay can therefore read
the screen, which is a meaningfully different thing from being able to
carry the traffic.

The obvious objection is "just run your own relay." That is a real answer
for one person and a bad answer for the product: the whole point of the
relay is that it is a commodity someone else runs.

The enabling change already happened. The relay used to cache and replay
keyframes, which meant it had to parse messages. It no longer does: it is
a pure byte pipe that authenticates a registration and forwards framing
it does not understand. E2E is therefore an encryption layer, not a
redesign.

## Decision

Add a second encrypted layer between sharer and viewer, keyed from the
session token that already exists.

- Noise-style handshake over the two existing TLS channels, with the
  token as the pre-shared key. No new credential for a user to manage,
  which is the property that makes adoption likely.
- Independent keys per direction. A relay holding the host-to-viewer key
  must not be able to forge viewer-to-host control traffic, which is how
  a request for a snapshot, a refresh, or a session teardown would be
  injected.
- Rekey on epoch change and on repair, so one long-lived key does not
  accumulate the whole session's risk.

## Implementation notes

- The handshake itself travels in cleartext, necessarily: it is what
  derives the keys. Every frame after it is sealed. The relay therefore
  sees one handshake and then only sizes and timing.
- The AAD is the frame counter, which travels in the clear header.
  Tampering with it changes the AAD and fails the tag, so a relay cannot
  reorder or substitute without the receiver noticing.
- Sealing happens **per viewer**, after the shared broadcast, because each
  viewer has its own keys. The broadcast carries logical messages; the
  per-viewer task seals on the way out.
- The browser path is a separate trust domain and is not covered by this.
  It has no per-browser key exchange yet; a browser that wants the same
  guarantee needs WebCrypto in the handshake.

## Consequences

- A relay operator learns: that a session exists, its packet sizes and
  timing, and nothing else. Not the dimensions, not the content, not the
  session code.
- The relay's bandwidth accounting is unchanged.
- A viewer that cannot complete the handshake must fail. Falling back to
  plaintext because the handshake was inconvenient would be a silent
  downgrade of the security model, so the fallback is explicit, logged,
  and off unless someone passes a flag saying they want it.

## Alternatives rejected

- **Rely on operator trust.** Rejected: it makes "share my screen"
  depend on an unknown third party's discretion, which is not a
  security model.
- **A Diffie-Hellman exchange that ignores the token.** Rejected: it
  adds a key-exchange round trip and a new failure mode without adding a
  credential, and the token is already a high-entropy shared secret both
  sides already have.
- **Encrypt only the pixel messages.** Rejected: control traffic is
  forgeable in ways that matter (forcing a snapshot loop, tearing down a
  session), so the whole envelope is encrypted.
