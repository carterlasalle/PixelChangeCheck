# 2. Two secrets: a certificate pin and a viewer token

Date: 2026-09-28

## Context

The direct QUIC client deliberately accepted any server certificate, the
relay spoke plaintext TCP with no access control at all, and the browser
listener was unauthenticated plaintext HTTP. Anyone who could reach a port
could watch the screen.

The obvious fix -- one shared secret -- conflates two different questions:

1. *Am I talking to the sharer I meant to?* (transport identity)
2. *Am I allowed to watch?* (authorization)

The certificate fingerprint is transmitted in cleartext inside the TLS
handshake. It is therefore **not** a secret, and treating it as one would
leave anyone who has ever seen one handshake able to watch forever.

## Decision

Two values, printed together by the sharer as one copy-pasteable command.

- `--pin <sha256-hex>`: the viewer accepts exactly that certificate and no
  other. A self-signed LAN certificate has no meaningful DNS identity, so
  pinning is the stronger check anyway.
- `--token <26 symbols>`: presented as the first `Message::Hello`. This is
  the authorization credential and the only secret.

The relay authenticates the token during registration and acknowledges a
good one, so "connected" means "authorized" rather than "TCP accepted". It
rate-limits failed registrations per source address and expires idle
sessions.

The browser path requires the token on every route. It is plaintext unless
the operator supplies a certificate with `--web-cert`/`--web-key`: a
self-signed certificate would only produce a browser warning, so shipping
one would be worse than saying so.

## Consequences

- Two flags to copy instead of one. The sharer prints the whole viewer
  command, so in practice it is still one copy-paste.
- The rendezvous code for a relay is no longer the secret, which is what
  allowed it to stay short and typeable.
- Anyone who has sniffed a previous session still needs the token.

## Alternatives rejected

- **A single "secret" used as the pin.** Rejected: the pin is public, so
  this would have been authorization by a public value.
- **Host approval of each viewer.** Deferred: it needs a control channel
  that a shared-secret tool does not otherwise have. Recorded rather than
  half-built.
