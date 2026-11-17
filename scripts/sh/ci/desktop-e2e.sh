#!/usr/bin/env bash
#
# ci/desktop-e2e.sh — Local equivalent of the `desktop-e2e` CI job.
#
# Builds the Tauri app without bundles, installs tauri-driver, and runs the
# WebDriver e2e suite under Xvfb. Linux only — WebDriver automation of the
# webview is not supported on macOS.
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "error: desktop-e2e requires Linux (xvfb + WebKitWebDriver); CI runs it on ubuntu-latest" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

for tool in cargo node pnpm xvfb-run; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "error: $tool not found" >&2
    exit 1
  }
done
if ! command -v WebKitWebDriver >/dev/null 2>&1; then
  echo "error: WebKitWebDriver not found; install it with:" >&2
  echo "  sudo apt-get install -y webkit2gtk-driver" >&2
  exit 1
fi
if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "==> cargo install tauri-driver --locked"
  cargo install tauri-driver --locked
fi

echo "==> pnpm install --frozen-lockfile --ignore-scripts"
CI=1 pnpm install --frozen-lockfile --ignore-scripts

echo "==> pnpm --filter @mars/desktop exec tauri build --no-bundle"
pnpm --filter @mars/desktop exec tauri build --no-bundle

echo "==> xvfb-run -a pnpm --filter @mars/desktop run test:e2e:tauri"
xvfb-run -a pnpm --filter @mars/desktop run test:e2e:tauri

echo "desktop-e2e: OK"
