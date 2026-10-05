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

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

echo "--- fmt"; cargo fmt --all -- --check
echo "--- clippy (default features)"
cargo clippy --all-targets --locked --offline -- -D warnings 2>&1 | tail -1
echo "--- clippy (no default features)"
cargo clippy --no-default-features --all-targets --locked --offline -- -D warnings 2>&1 | tail -1
echo "--- tests"
cargo test --locked --offline 2>&1 | tail -15
echo "--- release build + smoke"
cargo build --release --offline 2>&1 | tail -1
pkill -f 'pcc (relay|share|view)' 2>/dev/null || true
sleep 1
bash scripts/smoke.sh 2>&1 | tail -3

sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml && rm Cargo.toml.bak
# A plain check rewrites the workspace member's version in Cargo.lock
# without touching any dependency pins (unlike `cargo update`).
cargo check --offline >/dev/null 2>&1
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Cut $VERSION" -m "CHANGELOG section $VERSION already describes this tree; this commit only pins the manifest version the release tag must match."
git tag -a "$TAG" -m "$TAG"
git push origin master "$TAG"

echo "--- watching the release run"
gh run watch --repo carterlasalle/pixelchangecheck "$(gh run list --repo carterlasalle/pixelchangecheck --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
echo "released $TAG"
