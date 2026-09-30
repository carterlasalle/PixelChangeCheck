# ADR 0007: the iroh transport (ticket signalling, N0 relay)

- **Status:** accepted
- **Supersedes:** the iroh half of ADR 0006.

## Context

Some viewers cannot reach the sharer directly and nobody wants to run a
relay for them. Iroh answers exactly that: peer-to-peer QUIC with relay
fallback through iroh's hosted N0 relays, plus discovery, in one
endpoint. The cost is a 369-crate dependency tree and a session setup
that looks nothing like `--connect host:port`.

## Decision

`--transport iroh`. The sharer binds one endpoint (N0 preset: relay +
discovery, our `pcc/v7` ALPN) and prints a **ticket**: the endpoint id
plus discovered addresses as JSON base64. The viewer dials the ticket
and opens the session stream. Everything after that — Hello, E2E,
snapshot, updates — is the same `Message` stream as every other
transport, via `IrohTransport: MessageTransport`.

Signalling is the ticket and nothing else. No `pair` URL integration
yet: the ticket is printed as a `pcc view` command line, which is
copy-pasteable today and machine-readable tomorrow.

## Constraints

- The ALPN is `pcc/v7`. A wrong ALPN fails at accept, before any
  `Message` flows — the transport-level equivalent of the version gate.
- The ticket carries endpoint-discovered addresses plus the N0 relay,
  so a viewer behind NAT still connects. A 500 ms settle before
  printing keeps direct addresses in the ticket when they exist.
- Ownership discipline, learned the hard way in live testing:
  `IrohTransport` holds the connection AND the endpoint, and the split
  sink keeps both. Dropping an iroh `Endpoint` aborts every connection
  on it ungracefully — twice this produced a session that connected
  and then died right after Hello. The QUIC path holds its connection
  for the same reason; the rule is now general, not per-transport.
- No datagram audio on this path: iroh exposes no connection handle to
  the serve loop, same as the relay fan. Viewers get video only until
  that path is built.

## Consequences

- New dependencies: `iroh 1.x` and its tree (~369 crates total in the
  lockfile). This is the heaviest thing in the dependency list by far,
  and it is load-bearing: remove it and `--transport iroh` stops
  compiling, which is the correct coupling.
- Verified live: two processes on one machine, ticket in between, the
  viewer installed a 1280x720 snapshot at revision 219 and kept
  receiving frames. Same-process tests cannot prove this (shared
  runtime, shared network); the receipt is the two-process run.
