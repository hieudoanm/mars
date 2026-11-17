#!/usr/bin/env bash
#
# ci/desktop-build.sh — Local equivalent of the `desktop-build` CI job.
#
# Builds the Tauri desktop bundles for the current OS only (CI runs a
# macOS/Linux/Windows matrix; locally you get your host platform's bundles).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DESKTOP_DIR="$REPO_ROOT/packages/desktop"
cd "$DESKTOP_DIR"

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found; install the Rust toolchain from https://rustup.rs" >&2
  exit 1
}
command -v pnpm >/dev/null 2>&1 || {
  echo "error: pnpm not found; install it from https://pnpm.io" >&2
  exit 1
}

if [ ! -d node_modules ]; then
  echo "==> pnpm install --frozen-lockfile --ignore-scripts"
  CI=1 pnpm install --frozen-lockfile --ignore-scripts
fi

# Linux needs the webkit2gtk/gtk system libraries for the Tauri build.
if [ "$(uname -s)" = "Linux" ]; then
  echo "==> Installing Linux dependencies"
  sudo apt-get update
  sudo apt-get install -y build-essential curl wget file libssl-dev \
    libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf \
    libglib2.0-dev libwebkit2gtk-4.1-dev
fi

case "$(uname -s)" in
  Darwin)
    echo "==> tauri build --bundles app,dmg"
    pnpm exec tauri build --bundles app,dmg
    ;;
  Linux)
    echo "==> tauri build --bundles deb,rpm,appimage"
    pnpm exec tauri build --bundles deb,rpm,appimage
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "==> tauri build --bundles msi,nsis"
    pnpm exec tauri build --bundles msi,nsis
    ;;
  *)
    echo "error: unsupported OS '$(uname -s)' for desktop bundles" >&2
    exit 1
    ;;
esac

echo "desktop-build: OK — bundles under target/release/bundle/"
