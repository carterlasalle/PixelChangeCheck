# Design

## Context

PixelChangeCheck replicates a screen. Its premise is that a desktop is
mostly still, so re-encoding whole frames wastes almost all of the
available bandwidth, and that a screen-sharing tool is only useful if the
text is *exactly* what was on the sharer's screen.

Those two claims pull in opposite directions. Cheap transport wants to
approximate; exactness forbids it. Most of the design below is about
refusing to trade one for the other.

## The authoritative surface

One thing in this system is authoritative: the RGB buffer the sharer
believes an up-to-date viewer holds. Everything else is derived from it.

- The sharer diffs each capture against that buffer, not against the
  previous capture. A change too small to send this frame is therefore
  still there next frame, and crosses the threshold on its own.
- The buffer advances only over rectangles that actually shipped.
- Viewers reconstruct it through one `Compositor`, which is the only code
  allowed to mutate a surface.

### Why the base is lossless

A JPEG keyframe is not the frame the sender holds. Decoding it produces
different pixels, so every later patch is computed against a surface the
viewer does not have, and static JPEG artefacts can never be corrected
because source-to-source comparison never sees them. For a tool whose
claim is exact text, a lossy base is a correctness bug.

So snapshots are PNG, patches are LZ4 over exact RGB, and JPEG survives
only as the explicitly lossy browser preview. See
`docs/adr/0001-lossless-authoritative-surface.md`.

## One sequencer

Three defects in the previous design shared a root cause: nothing owned
the order of updates. A viewer could be replayed an old keyframe and then
never receive the corrections; a viewer that fell behind on a broadcast
channel would silently skip patches that later ones depended on; and a
periodic repair could be suppressed by an idle screen.

- **Revisions** are a monotonic counter on the capture loop. Every
  snapshot part and every update carries one. A viewer applies a revision
  at most once.
- **Epochs** bump when geometry changes. Everything from an older epoch
  is discarded.
- **Joining is atomic**: subscribe first, then take the current snapshot,
  then stream. Nothing produced in between is lost, and anything at or
  below the snapshot's revision is skipped rather than applied on top.
- **Loss is repair**: falling behind produces a fresh snapshot. Falling
  behind again immediately means the viewer cannot keep up, and it is
  disconnected.

See `docs/adr/0003-revision-and-epoch-sequencing.md`.

## Sharing the surface

The authoritative buffer is one `Arc`. The capture loop mutates it
through `Arc::make_mut`, which copies only when a joining viewer is
actually holding the same allocation. So publishing "the current screen"
costs a pointer swap, and a late joiner is handed the very same bytes
rather than a per-frame copy of a snapshot that is already stale.

This is the single most important ownership decision in the codebase: it
is what makes "a late joiner sees what is on screen now" true by
construction instead of by remembering to update a cache.

## Choosing what to send

Three representations, in the order the receiver would have to think
about them:

1. **Nothing** — the content is already right. Send a keep-alive.
2. **Copy** — the receiver already holds these pixels somewhere else.
   Found by comparing per-tile hashes, then *proved* byte-for-byte
   before use. This is what makes a scroll cheap.
3. **Fill or Rect** — a solid colour is 24 bytes and a general-purpose
   byte compressor cannot beat that; anything else is an LZ4 patch.

If the resulting patch set costs more than a fresh snapshot, the
snapshot ships instead and the reference adopts it, because a snapshot
*is* the current frame.

A `Copy` reads the surface as it was *before* the transaction. That is
what lets two regions swap in one update, and it is why the same rule is
implemented twice: once in Rust, once in JavaScript.

## Region grouping

Neighbouring changed tiles are merged before they are shipped, with a
guard: a merge is declined when the bounding box would grow past twice
the dirty area. Merging saves headers and compression calls; it costs
unchanged pixels. The threshold is a policy number, documented where it
lives and measurable with `cargo run --release --example benchmarks`.

## Security model

Two separate facts, deliberately not merged:

- **Transport identity** — the viewer pins the sharer's certificate by
  SHA-256 fingerprint. The fingerprint is *not* a secret: it is on the
  wire in cleartext.
- **Authorization** — the viewer presents a session token as its first
  message. This is the only secret.

The relay authenticates during registration and never inspects frame
contents, so end-to-end encryption can be added without changing it.
Every length prefix is checked before the buffer it authorises exists,
and every decoded size is bounded by geometry the peer declared.

See `docs/adr/0002-token-and-pin.md` and `SECURITY.md`.

## Deliberate non-goals

- **Inter-frame video.** Motion content is the case a pixel-diff protocol
  is worst at, and the honest answer is a negotiated video codec rather
  than a cleverer diff. The registry, identifiers and capability probing
  are in `src/codec/`; no encoder is linked, and the two pure-Rust
  candidates were rejected on evidence recorded in that module. The
  exact path is the product until an encoder is actually available.
- **Platform damage metadata.** DXGI, ScreenCaptureKit and PipeWire all
  expose dirty rectangles, and using them would skip work. Capture is
  full-frame today.
- **Input injection.** No keyboard, mouse or clipboard forwarding, so
  there is no input-injection surface to defend.
- **Custom cryptography.** TLS comes from rustls; the token is compared
  in constant time; nothing is hand-rolled.
