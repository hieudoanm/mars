#!/usr/bin/env bash
#
# ci/cli-build.sh — Local equivalent of the `cli-build` CI job.
#
# Builds the release `mars` binary and renames it to the Rust target triple,
# matching the artifact names produced by CI for the release job.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found; install the Rust toolchain from https://rustup.rs" >&2
  exit 1
}

echo "==> cargo build --release -p mars --locked"
cargo build --release -p mars --locked

# Rename to the Rust target triple so artifacts don't collide, same as CI.
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)   TARGET="x86_64-unknown-linux-gnu" ;;
  Darwin-arm64)   TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*) TARGET="x86_64-pc-windows-msvc" ;;
  *)              TARGET="$(rustc -vV | sed -n 's/^host: //p')" ;;
esac

if [ -f target/release/mars ]; then
  mv target/release/mars "target/release/mars-$TARGET"
  echo "Binary: target/release/mars-$TARGET"
elif [ -f target/release/mars.exe ]; then
  mv target/release/mars.exe "target/release/mars-$TARGET.exe"
  echo "Binary: target/release/mars-$TARGET.exe"
else
  echo "error: expected target/release/mars but it does not exist" >&2
  exit 1
fi
