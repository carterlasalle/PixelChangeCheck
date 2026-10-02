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

## Releasing

One tag does everything: GitHub Release binaries and a crates.io
publish. The tag must match the `version` in `Cargo.toml`, and the
workflow refuses to run if it does not, because a version that disagrees
with its tag is one nobody can find by release and blocks the next one.

```sh
cargo test && cargo clippy --all-targets && cargo fmt --all
cargo build --release && bash scripts/smoke.sh   # against the real binaries

# bump version in Cargo.toml, update CHANGELOG.md, commit
git tag v0.1.0 && git push origin master --tags
```

`0.1.0` was published by hand, because crates.io cannot attach a
trusted publisher to a crate that does not exist yet. Every release after
that is tokenless.

For the tokenless path, **one-time setup** on the crate's own page (not
account settings -- crates.io binds the mapping per crate):

    crates.io -> pixel-change-check-client -> Settings ->
    Trusted Publishing -> Add:
      provider            GitHub Actions
      repository owner    carterlasalle
      repository name     pixelchangecheck
      workflow filename   release.yml
      environment         blank

The filename is `release.yml`, not `.github/workflows/release.yml`:
crates.io matches it against GitHub's OIDC claims, which carry the bare
filename.

`rust-lang/crates-io-auth-action` sets the token as a step **output**, it
does not export `CARGO_REGISTRY_TOKEN` for you, so the publish step
passes it through explicitly. Its `post` step revokes the temporary token
when the job ends.

A crates.io version, once claimed, can never be reused -- not after a
yank, not after a deletion. If a release is broken, publish `0.1.1` and
yank `0.1.0`; do not try to reclaim `0.1.0`.

## Release asset names must match the binstall template

`Cargo.toml`'s `[package.metadata.binstall]` points at
`pcc-{version}-{target}{archive-suffix}`, where `{target}` is the Rust
target triple. The release workflow's `Resolve target triple` step
maintains that mapping by hand, and the archives must keep matching it:
v0.1.1/v0.1.2 shipped `*-Linux-X64`-style names that binstall cannot
resolve, so anyone pinned there gets a surprise source build. When the
matrix gains an OS, add its triple case in the same commit.

## The binary is `pcc`, the old name lingers

Up to 0.1.3 the installed binary was `pixel-change-check-client`; since
0.1.4 it is `pcc`. Both can be on PATH after an upgrade, and they report
the same program name, so check `pcc --version` (or the file dates) when
something behaves like an old build. `cargo uninstall
pixel-change-check-client` resolves the binary name from the *current*
manifest — it deletes the new `pcc` and leaves the stale old binary
behind. Remove `~/.cargo/bin/pixel-change-check-client` by hand instead.

## `cargo binstall` falls back to source

On a target with no published archive, binstall silently compiles the
whole graph (264 crates, ~7 minutes on ARM Linux before its asset
existed). That is working as designed, not a failure — but tell users:
`--disable-strategies compile` fails fast instead, and the supported
target list is the release matrix in `.github/workflows/release.yml`.
