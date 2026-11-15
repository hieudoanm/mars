#!/usr/bin/env bash
#
# ci/cli-check.sh — Local equivalent of the `cli-check` CI job.
#
# Runs cargo fmt --check, clippy, and the workspace test suite. On Linux it
# installs the Tauri system dependencies first (src-tauri is a workspace
# member, so workspace-wide cargo commands compile the desktop backend).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found; install the Rust toolchain from https://rustup.rs" >&2
  exit 1
}

# src-tauri is a workspace member, so workspace-wide cargo commands compile
# the Tauri backend and need its system libraries on Linux.
if [ "$(uname -s)" = "Linux" ]; then
  echo "==> Installing Linux dependencies (webkit2gtk, gtk, libsoup)"
  sudo apt-get update
  sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
    patchelf libglib2.0-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
fi

echo "==> cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "==> cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo test --workspace"
cargo test --workspace

echo "cli-check: OK"
