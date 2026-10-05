# ADR 0010: the relay hosts the browser viewer, and the host answers its handshake

- **Status:** accepted
- **Supersedes:** nothing. Resolves the open question in
  `0006-deferred-transports-iroh-webrtc.md` about where a browser viewer
  should be served, and extends `0005-browser-key-exchange-p256.md`,
  which fixed *which* primitives the browser speaks but not *who*
  answers them over a relay.

## Context

The browser viewer existed but only the **sharer** served it, on a port
that refuses to run plaintext on a non-loopback address. That made the
"just open a browser" path the hardest one to reach: the sharer needs a
certificate and an open port, and the viewer needs a URL that points at a
machine which may be behind NAT.

The relay is the component both sides can already reach — that is its
whole purpose — and it typically runs on a host with a real certificate.
So it is the natural place to serve the page.

The obvious objection is trust. `0004` and `0005` promise that the relay
cannot read the screen. A relay that terminates the browser's WebCrypto
handshake would hold the AES-GCM keys and could read every frame. Serving
a *page* is fine; answering its handshake is not.

## Decision

**The relay serves the page and bridges the socket; the host answers the
handshake.**

1. `pcc relay --web` serves the viewer page at `/v/<session>/` and its
   WebSocket at `/v/<session>/ws`, on the relay's **existing** port and
   behind the relay's **existing** certificate.
2. One port carries both protocols by **ALPN**: `pcc` selects the relay
   protocol, `http/1.1` selects the browser surface. That is why
   `NetworkConfig::relay_server_config` is a separate constructor — the
   QUIC endpoints must keep advertising only `pcc`.
3. The relay registers the browser socket as an ordinary viewer leg with
   a fresh id, tells the host `BrowserHere` for that id, and forwards
   every payload **opaque and tagged**. It never inspects one.
4. The host runs the browser handshake for that id, using the same
   `run_browser_session` the sharer's own web port uses. The relay sees
   frame sizes and timing, exactly as it already did for native viewers.
5. The viewer leg is length-framed and a WebSocket frame is not; the
   bridge strips the prefix where the two conventions meet
   (`relay_web::strip_len_prefix`).

## Consequences

- A viewer needs no binary, no terminal, and no `--web-cert` files: one
  URL. The token still travels in the fragment and is still checked
  before a byte of surface data moves.
- The relay's trust story is unchanged, and the code says so: the bridge
  is deliberately a byte pipe, and the sealing happens on the host.
- `BrowserHere` is a fifth control frame kind. A host that does not
  understand it ignores it, so an old host and a new relay degrade to
  "the browser sits idle" rather than a framing error.
- The link requires the relay to have been started with `--web`. The
  sharer cannot detect that from its side of the connection, so it prints
  the browser link with that precondition stated rather than guessing.
- A self-signed relay certificate makes browsers warn. `--cert`/
  `--cert-key` already existed for a stable identity; a real certificate
  there is now also what makes the browser surface pleasant.

## Alternatives considered

- **A second HTTP port on the relay.** Rejected: it needs its own TLS
  configuration, its own certificate story, and its own firewall opening,
  for no gain over ALPN on the port that is already open.
- **The relay terminating the browser handshake.** Rejected outright: it
  would make "the relay cannot read the screen" false against the
  operator running it, which is the specific threat `0004` exists to
  answer.
- **A `--web-url` flag so the relay can print its own public link.**
  Rejected as unnecessary: the sharer knows the session and the relay
  address, so it prints the complete link. The relay only prints the URL
  shape.
- **Making the browser handshake speak X25519 so one implementation
  covers both.** Rejected for the reason `0005` already recorded:
  `crypto.subtle` has no X25519 in the browser versions most people run.