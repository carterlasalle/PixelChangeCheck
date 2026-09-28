# Security

## Threat model

PixelChangeCheck streams a live picture of the sharer's screen. An
unauthorized observer gets the keys to everything on it: passwords, mail,
private messages. The defaults below assume the network is hostile.

## What is enforced

- **Authorization**: every viewer must present the session token
  (`--token`) as the first message. The sharer compares it in constant
  time and closes the connection otherwise. The relay checks it during
  registration and rate-limits failed attempts per source address.
- **Transport identity (direct QUIC)**: the viewer pins the sharer's
  certificate by SHA-256 fingerprint. Any other certificate fails the
  handshake.
- **Transport identity (relay)**: the sharer pins the relay's certificate
  the same way, with `--relay-pin`.
- **Transport identity (browser)**: TLS only if you supply a real
  certificate with `--web-cert`/`--web-key`. Without one the web port is
  **plaintext**; the token still authenticates, but the content is readable
  by anyone on the path.
- **Framing**: every length prefix is checked against the message budget
  *before* the buffer it authorises is allocated, on every entry point
  including the relay's own reader.
- **Decoding**: decompressed sizes are bounded by the geometry the peer
  declared, which is itself bounded by a frame budget. A decompression
  bomb costs an error, not memory.

## What is not enforced

- **The rendezvous code is not a secret.** It is six unambiguous symbols
  (30 bits) and only routes a connection to a session. The token is the
  secret.
- **No end-to-end encryption between sharer and viewer.** The relay sees
  the ciphertext of the QUIC channel but not plaintext; the browser path
  is plaintext unless you configure TLS. There is no application-layer
  encryption layered on top.
- **No viewer approval prompt.** Anyone holding the token may watch.
- **Clipboard, keyboard and mouse injection are not implemented**, so
  there is no input-injection attack surface to consider.

## Reporting a vulnerability

Open a GitHub issue on this repository describing the problem, the
version, and how to reproduce it. Do not include real screen content or
tokens in a public report; a private channel is preferable for anything
that would need one.
