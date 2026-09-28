# 1. The authoritative surface is lossless

Date: 2026-09-28

## Context

The pipeline originally sent a JPEG keyframe and LZ4-compressed patches on
top of it. Two audits pointed at the same defect from different angles:

- A JPEG keyframe is not the frame the sender holds. Decoding it produces
  different pixels, so every later patch is computed against a surface the
  viewer does not have. Static JPEG artefacts can never be corrected,
  because source-to-source comparison never sees them.
- Lossless LZ4 everywhere "looks" lossless and is not: the reconstruction
  is only as exact as its base.

For a screen-sharing tool whose entire claim is *exact pixels for text and
controls*, a lossy base is a correctness bug, not a quality setting.

## Decision

The authoritative stream is lossless end to end.

- A snapshot is PNG (deflate level 1, adaptive filters). Lossless, and
  decoded natively by every browser, so the web compositor needs no image
  decoder of its own.
- Patches are LZ4 over exact RGB.
- JPEG survives only as the explicitly lossy `/stream` MJPEG preview, which
  the page labels as lossy and which no test asserts exactness against.

Snapshots are transferred as `SnapshotBegin` / `SnapshotChunk` /
`SnapshotCommit` rather than one message, because a 4K noise frame does not
losslessly fit in any single message and a protocol that cannot describe
that case has to either fail or lie about it.

## Consequences

- A viewer is byte-identical to the sender's reference after a snapshot
  plus its updates. That property is what the whole test suite asserts,
  and it is what makes "exact text" a claim rather than a hope.
- Snapshots cost more than JPEG would. They are emitted on join, on
  recovery, on an epoch change, and on a 30-second backstop that only fires
  while the screen is actually changing, so an idle screen still sends
  nothing.
- Every surface is bigger on the wire than a lossy one. Accepted.

## Alternatives rejected

- **JPEG keyframes plus lossless patches.** Rejected: the viewer drifts
  from the sender at the first keyframe and never recovers.
- **A lossy preview followed by exact refinement.** This is the design the
  audit proposed as a pragmatic option. Rejected here because it only pays
  off if the preview is materially faster to first paint, and it adds a
  whole class of "refinement arrived after newer content" bugs. If first
  paint ever becomes a measured problem, this is the ADR to revisit.
- **Keeping every rectangle's raw pixels uncompressed.** Rejected: a
  4K whole-screen update would be 25 MB on the wire.
