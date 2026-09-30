# ADR 0009: install binaries, build from source fast, gate platform weight

- **Status:** accepted
- **Supersedes:** nothing. Refines the delivery half of
  `docs/spec/roadmap.md` §4 (which specified the release train but not
  install ergonomics) and records what was deliberately *not* done from
  the build-packaging feedback: no cargo-dist, no vendored-linker config,
  no client/server target split.

## Context

`cargo install` compiles ~730 crates on the user's machine behind the
slowest possible profile (fat LTO, one codegen unit), plus a vendored
libopus cmake build and ALSA/X11 system headers on Linux. On a small
build host that is tens of minutes and a toolchain the user should not
need. The feedback proposed four fixes and six smaller ones; the audit
against this tree accepted five, rejected three, and reshaped one.

## Decision

1. **Ship prebuilt binaries via `cargo binstall`, not cargo-dist.**
   `[package.metadata.binstall]` points at
   `pcc-{version}-{target}` release assets (tgz, zip on Windows), and
   the release workflow now names its archives with the Rust target
   triple instead of `RUNNER_OS`/`RUNNER_ARCH`. cargo-dist would also
   automate the matrix, a shell installer, a Homebrew tap and .deb —
   but it owns the release workflow end to end, and this repo's release
   job (triple mapping, tag/version gate, OIDC crates.io publish) is
   load-bearing as written. Binstall metadata is six lines and keeps
   everything else untouched.
2. **Split `release` (fast) from `dist` (full).** `release` — what
   `cargo install` and every local dev build pays — is thin LTO with 16
   codegen units at opt-level 2. `dist` inherits it and restores fat
   LTO, one codegen unit, opt-level 3; the release workflow builds with
   `--profile dist`. Full optimization is paid once per release by CI,
   not once per user.
3. **Gate `audio` and `native-viewer` as opt-out features.** Both
   default on, so plain installs behave exactly as before;
   `--no-default-features` drops cpal/Opus (ALSA headers, vendored
   cmake libopus) and minifb (X11/Wayland headers) for a headless relay
   or minimal viewer. Without `audio` every source resolves to silence
   through the same `AudioSource` enum (no new flags); without
   `native-viewer` every session is headless with a warning, exactly as
   if `--no-window` were passed. The web viewer and every transport
   work in all four combinations.
4. **Trim tokio to the named set** (`rt-multi-thread`, `macros`,
   `net`, `sync`, `time`, `io-util`, `io-std`). Measured effect today:
   one fewer `parking_lot` edge in the lockfile. `process` and `signal`
   still compile because iroh's tree requires them — the value is the
   honest list, which fails loudly the next time something new is
   actually ours.

## Rejected, with receipts

- **Link the system libopus instead of vendoring.** Rejected as
  stated: `opus 0.4` binds `opusic-sys`, whose non-`bundled` mode
  honors `OPUS_LIB_DIR`/`OPUS_LIB_STATIC` — but the default feature set
  is `bundled`, so consumers get the cmake build regardless of what is
  installed. Switching the default to system-linking would trade a slow
  build for a *fragile* one (missing system lib = build failure, and
  the failure the reporter saw was `alsa-sys`, not opus). The README
  documents `OPUS_LIB_DIR` for whoever wants it; the default keeps
  building anywhere. What *was* taken from this item: the README now
  names the package that avoids the cmake step in terms the reporter's
  own run proved (`libopus-dev` + `pkg-config`).
- **Deduplicate versions (`cargo tree -d`).** No-op: every remaining
  duplicate in the tree (base64, base16ct, bitflags, regex-*, libc,
  memchr, nom, data-encoding) arrives through iroh/webrtc and is
  required by those trees' own pins. There is nothing here that is
  ours to dedupe without forking a transitive pin.
- **Split client/server targets.** Rejected: there is one binary with
  two subcommands sharing `app`, `network`, `pcc` and `telemetry`. A
  split would duplicate the manifest for no measured saving — the
  weight is dependencies (iroh/webrtc/cpal), which the feature flags
  already gate, not targets.
- **`num_cpus`.** Dead weight, verified by search: nothing in `src/`,
  `tests/`, `examples/` or `benches/` referenced it, so the line was
  deleted (lockfile entry with it). The receipt survives as the method:
  search first, delete, watch the lockfile.

## Consequences

- `cargo binstall` works from the next tag on, with zero new
  infrastructure: the metadata rides `cargo publish` and the archives
  ride the existing release job.
- `cargo install` stays a full local build but a faster one (thin LTO,
  16 CGUs), and `--locked` is now documented rather than assumed.
- CI must cover the feature matrix it promises: default,
  `--no-default-features`, and each single feature on and off. A gate
  that compiles only the default rots silently.
