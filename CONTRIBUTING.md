# Contributing

## Setup

```sh
cargo build --release
cargo test
```

Linux needs `libxcb1-dev libxrandr-dev libdbus-1-dev`.

## Before you push

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo test` runs three suites:

- unit tests, next to the code they cover;
- `tests/replication.rs`: the state machine's invariants, including a
  property test that drives random fill/replace/copy transactions against
  an independently written model;
- `tests/transport.rs`: real QUIC and real TLS relay connections.

## Conventions

- **Traceability.** Behavioural boundaries are marked with a
  `trace:v1` comment (`id=`, `work=`, `satisfies=`) or an explicit
  `trace:exempt` with a reason. Run `trace verify --changed` before
  committing.
- **Errors say what was rejected.** A limit failure names the budget, the
  value observed, and the limit, e.g.
  `frame budget exceeded: max_frame_bytes=100000000, requested=1200000000`.
  "Operation failed" is not acceptable.
- **Numbers need receipts.** A threshold, a limit, or a default gets a
  comment explaining where the number came from, and ideally a test or a
  benchmark that would notice if it were wrong.
- **Tests assert observable behaviour.** A test that restates the
  implementation, asserts internal call order, or mirrors a conditional
  has negative value; delete it rather than maintain it.
- **Architecture decisions go in `docs/adr/`.** If a change alters a
  boundary, a protocol, a security model, or a persistence format, write
  the ADR in the same change.

## Architecture, briefly

- `pcc/` is the core: `detector` finds changed rectangles,
  `planner` decides how to send them, `compositor` is the only code
  allowed to mutate a viewer's surface.
- `network/` owns the wire format, framing, and TLS. `relay.rs` is a byte
  relay that never inspects frame contents.
- `app/share.rs` is the only writer of the authoritative surface.
- `server/renderer/client.js` is a port of `pcc/compositor.rs`. The two
  must agree; if you change one, change the other in the same commit.
