# ADR 0005: the browser viewer's key exchange is ECDH P-256, not X25519

- **Status:** accepted
- **Supersedes:** nothing. Refines `0004-end-to-end-encryption.md`, which
  specified the native path and left the browser open.

## Context

`0004` made the native viewer and the relay path sealed end to end. It
left the browser unsealed, on the reasoning that WebCrypto in a handshake
is a separate piece of work rather than a gap in the E2E work.

The browser is not a secondary path. A browser is how most people will
actually watch, and an unsealed browser means the token is the only
thing standing between a passive network observer and every pixel.

The obvious implementation is to copy the native handshake: X25519 plus
ChaCha20-Poly1305. The blocker is that `crypto.subtle` has no X25519 in
the browser versions most people run. Chrome shipped X25519 in 133 and
Safari has no plan for it, so a WebCrypto X25519 handshake would work for
a developer testing on a current Chrome and fail for a user on a phone.

## Decision

The browser does **ECDH on P-256 with HKDF-SHA-256 and AES-GCM**, and
the native client keeps **X25519 with HKDF-SHA-256 and
ChaCha20-Poly1305**.

The protocol shape is identical in both: an ephemeral key, a proof that
the peer holds the session token, two direction keys derived with
distinct labels, and every frame sealed with a monotonic counter that
travels in the clear and is also the AEAD additional data. Only the
primitive differs.

There is no plaintext fallback. A browser that cannot complete the
handshake is disconnected and told why. A silent fallback would mean the
encryption is present in the code and absent in the product, which is
worse than not having it, because the code reads as though it is on.

## Constraints

- `crypto.subtle` primitives must exist in the target browsers. ECDH
  P-256, HKDF and AES-GCM have been universally available since 2017;
  X25519 has not.
- The two implementations must not drift apart. They are separate
  code in separate languages, which is exactly the condition under which
  a key schedule silently diverges and every handshake fails with no
  useful error.
- The token must be proven, not merely sent, so a wrong token is
  refused at the handshake rather than producing a session that
  decrypts to garbage.

## Evidence available at the time

- The proof construction was checked against a fixed vector computed
  independently in Rust and in JavaScript; both produce
  `4b8ebd6b5e2d59bb5cd37f45dad9b5905092a37611691b56ffbdad5cad8ddc71`
  for the same public key and token.
- The full schedule was checked the same way: for a fixed ECDH secret,
  the Rust and JavaScript implementations produce byte-identical sealed
  frames, `000000003cfb7fc938a31a7b4647236f7afff8cb7c690deab9`, which
  means the HKDF labels, the counter-as-nonce rule, the additional data
  and the AEAD all agree.
- A browser that cannot prove the token receives no session, and the
  stream is not continued.

## Alternatives considered

- **WebCrypto X25519.** Uniform with the native path and cheaper per
  byte. Rejected: unavailable in Safari and in Chrome before 133, which
  makes it a desktop-developer feature rather than a product feature.
- **Ship a WebAssembly build of the native crypto.** Rejected: it turns
  a small crate into a build-pipeline dependency for a viewer that
  should work from a single script file.
- **A WebCrypto-specific wire version, with the native protocol
  unchanged.** Rejected for the same reason: two protocol versions is a
  compatibility surface this project does not otherwise have.
- **A plaintext fallback for browsers that cannot complete the
  handshake.** Rejected outright, above.

## Rejected approaches and why the practical solution may differ

The ideal would be one primitive everywhere, chosen once. That is not
available today, because the browser's primitive set is chosen by
browser vendors rather than by this project. The split is a cost of
depending on a platform, not a preference, and it should collapse on its
own once X25519 is universally available in `crypto.subtle`; the
protocol shape above is already the one that would remain.

## Consequences

- Two key schedules must be kept in agreement. `proof_of_public` exists
  solely so the JavaScript can be checked against a fixed vector rather
  than by inspection, and `tests/sealed_browser.rs` drives the real
  server path so CI exercises the handshake on every run.
- The browser uses AES-GCM, which is hardware-accelerated almost
  everywhere, so the per-frame cost should be lower than the native
  path's software ChaCha20-Poly1305. This has not been measured.
- A certificate is now optional for confidentiality. It is still worth
  supplying, because without one the key exchange itself is visible to a
  network observer even though the payload is not.

## Follow-up conditions

- When X25519 is in `crypto.subtle` across the target browsers,
  reconsider collapsing to one primitive. The protocol shape does not
  change; only the key agreement and the AEAD do.
- Measure the per-frame sealing cost on the browser path. The claim that
  AES-GCM is cheaper here is an expectation, not a measurement.
