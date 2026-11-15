#!/usr/bin/env bash
#
# ci/extensions-zed.sh — Local equivalent of the `extension-zed` CI job.
#
# Formats, lints, tests, and builds the Zed WASM extension for
# wasm32-wasip1.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found; install the Rust toolchain from https://rustup.rs" >&2
  exit 1
}
command -v rustup >/dev/null 2>&1 || {
  echo "error: rustup not found; install it from https://rustup.rs" >&2
  exit 1
}

echo "==> cargo fmt -p mars-zed -- --check"
cargo fmt -p mars-zed -- --check

echo "==> cargo clippy -p mars-zed --all-targets -- -D warnings"
cargo clippy -p mars-zed --all-targets -- -D warnings

echo "==> cargo test -p mars-zed"
cargo test -p mars-zed

# rust-toolchain.toml pins the toolchain, so the target must be added
# explicitly for the pinned channel (same note as CI).
echo "==> rustup target add wasm32-wasip1"
rustup target add wasm32-wasip1

echo "==> cargo build --release --target wasm32-wasip1 -p mars-zed"
cargo build --release --target wasm32-wasip1 -p mars-zed

WASM="$REPO_ROOT/target/wasm32-wasip1/release/mars_zed.wasm"
[ -f "$WASM" ] || {
  echo "error: expected $WASM but it does not exist" >&2
  exit 1
}
echo "zed: OK — $WASM"
