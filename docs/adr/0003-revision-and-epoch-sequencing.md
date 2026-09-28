# 3. One sequencer: revisions, epochs, and an atomic join

Date: 2026-09-28

## Context

Three separate defects had the same root cause: nothing owned the order of
updates.

- The periodic snapshot was computed as `frame.id % (fps * 5) == 0` *after*
  the "nothing changed" branch, so an idle screen never sent one.
- A late joiner was replayed whatever keyframe the host happened to be
  caching, which could be arbitrarily old, and then never received the
  patches that would have fixed it.
- A viewer that fell behind on the broadcast channel did `continue`,
  silently dropping patches that later patches depended on.

## Decision

- **Revisions.** A single monotonic `u64` on the capture loop. Every
  snapshot and every update carries one. A viewer applies a revision at
  most once and refuses anything at or below the revision it holds.
- **Epochs.** A `u32` bumped whenever the geometry changes (resize,
  rotation, scale, display switch). A viewer discards everything from an
  older epoch and refuses work from a newer one until it has that epoch's
  snapshot.
- **Atomic join.** A viewer subscribes to the broadcast *first*, then takes
  the current snapshot, then streams. Nothing produced between those two
  points is lost, and anything at or below the snapshot's revision is
  skipped rather than applied on top of it.
- **Loss is repair.** Falling behind produces a fresh snapshot, not a
  `continue`. Falling behind again immediately after means the viewer is
  not keeping up at all, and it is disconnected.

## Consequences

- The capture loop is the only writer of the authoritative surface. That is
  a real constraint, and it is what makes "the reference is what viewers
  hold" true by construction.
- The relay no longer caches or replays keyframes. A stale snapshot is a
  bug, and the only correct place to produce a fresh one is the host.
- Updates that arrive out of order cannot corrupt a viewer, because a
  revision is applied at most once.

## Alternatives rejected

- **A larger broadcast buffer to stop lag.** Rejected: it postpones the
  problem while adding latency and memory, and it does not fix the case
  where a viewer has genuinely stopped reading.
- **The relay repairing viewers.** Rejected: the relay never inspects frame
  contents, so it cannot build a valid snapshot. Repair belongs to the
  host, which is the only party that holds the reference.
