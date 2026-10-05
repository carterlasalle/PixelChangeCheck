#!/usr/bin/env bash
# Cut a release: gates, version bump, tag, push, and watch the release run.
#
# Usage: scripts/release.sh <version>
#   version is the bare number, e.g. 0.1.5 (the tag becomes v0.1.5).
#
# What it does, in order, stopping on the first failure:
#   1. tree must be clean (so the tag points at exactly what was tested)
#   2. fmt, clippy (both feature configs), full test suite
#   3. release-binary smoke.sh (the only check that exercises the CLI)
#   4. Cargo.toml version bumped and Cargo.lock refreshed in one commit
#   5. annotated tag created (annotated: `git cat-file -t` says "tag";
#      the release notes render the tagger and date, lightweights do not)
#   6. commit + tag pushed; the tag push starts the release workflow,
#      which builds the four archives, creates the GitHub Release, and
#      publishes to crates.io (tokenless, via the trusted publisher)
#   7. watches the release run until it succeeds or fails
#
# The release workflow refuses a tag that disagrees with Cargo.toml or a
# version already on crates.io, so a version can never be reused -- if the
# release breaks after the tag lands, fix forward with a new version.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Find a working cargo without assuming this machine's layout. Prefer
# whatever is already on PATH; fall back to rustup's toolchain directory,
# which some setups (this repo's dev machine included) keep off PATH.
# A rustup "stable" toolchain is preferred over other installed ones so
# the fallback cannot pick something below the MSRV.
if ! command -v cargo >/dev/null 2>&1; then
  RUSTUP_TOOLCHAINS="${RUSTUP_HOME:-$HOME/.rustup}/toolchains"
  for candidate in "$RUSTUP_TOOLCHAINS"/stable-*/bin "$RUSTUP_TOOLCHAINS"/*/bin "$HOME/.cargo/bin"; do
    if [ -x "$candidate/cargo" ]; then PATH="$candidate:$PATH"; break; fi
  done
fi
command -v cargo >/dev/null 2>&1 || {
  echo "cargo not found; install rustup (https://rustup.rs) or put cargo on PATH"; exit 1
}
command -v gh >/dev/null 2>&1 || {
  echo "the GitHub CLI (gh) is required to watch the release run; install it or drop the watch"; exit 1
}

# owner/repo for `gh`, read from the remote so forks work without edits.
REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null || true)"
[ -n "$REPO" ] || { echo "could not determine the GitHub repository; run from the checkout"; exit 1; }

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "usage: scripts/release.sh <version>   (e.g. scripts/release.sh 0.1.5)"
  exit 1
fi
case "$VERSION" in
  *[!0-9.]*) echo "version must be a bare number like 0.1.5, got: $VERSION"; exit 1;;
esac
TAG="v$VERSION"

if [ -n "$(git status --porcelain -- . ':!.omp' ':!.scc' ':!.bughunt' ':!.tmp')" ]; then
  echo "tree is not clean; commit or stash first:"
  git status --short -- . ':!.omp' ':!.scc' ':!.bughunt' ':!.tmp'
  exit 1
fi
if [ "$(git rev-parse --abbrev-ref HEAD)" != "master" ]; then
  echo "releases cut from master; currently on $(git rev-parse --abbrev-ref HEAD)"
  exit 1
fi
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "tag $TAG already exists; pick a new version"
  exit 1
fi
if ! grep -q "^## $VERSION" CHANGELOG.md; then
  echo "CHANGELOG.md has no '## $VERSION' section; write it first"
  exit 1
fi

echo "--- fmt"; cargo fmt --all -- --check
echo "--- clippy (default features)"
cargo clippy --all-targets --locked -- -D warnings 2>&1 | tail -1
echo "--- clippy (no default features)"
cargo clippy --no-default-features --all-targets --locked -- -D warnings 2>&1 | tail -1
echo "--- tests"
cargo test --locked 2>&1 | tail -15
echo "--- release build + smoke"
cargo build --release 2>&1 | tail -1
pkill -f 'pcc (relay|share|view)' 2>/dev/null || true
sleep 1
bash scripts/smoke.sh 2>&1 | tail -3

sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml && rm Cargo.toml.bak
# A plain check rewrites the workspace member's version in Cargo.lock
# without touching any dependency pins (unlike `cargo update`). Run it
# without `--locked`, since it is the step that writes the lock.
cargo check --quiet
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Cut $VERSION" -m "CHANGELOG section $VERSION already describes this tree; this commit only pins the manifest version the release tag must match."
git tag -a "$TAG" -m "$TAG"
git push origin master "$TAG"

echo "--- watching the release run"
# The run for this tag may take a moment to register, and other runs may
# exist, so match on the tag explicitly rather than taking the latest.
RUN_ID=""
for _ in $(seq 1 30); do
  RUN_ID="$(gh run list --repo "$REPO" --workflow release.yml --limit 20 \
    --json databaseId,headBranch \
    --jq ".[] | select(.headBranch == \"$TAG\") | .databaseId" | head -1)"
  [ -n "$RUN_ID" ] && break
  sleep 5
done
if [ -z "$RUN_ID" ]; then
  echo "no release run appeared for $TAG; check https://github.com/$REPO/actions"
  exit 1
fi
gh run watch --repo "$REPO" "$RUN_ID" --exit-status
echo "released $TAG"
