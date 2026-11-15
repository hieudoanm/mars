#!/usr/bin/env bash
# Capture Project Mars desktop UI screenshots into assets/demo/png/.
#
# Starts the Vite dev server for packages/desktop, then delegates the actual
# capture to scripts/js/screenshot.mjs (Playwright). Screenshots are written at
# a 1280x640 ratio (2560x1280 px at 2x DPR). Requires Node.js and the desktop
# dependencies installed (pnpm install --filter @mars/desktop, or npm install
# in packages/desktop). No Tauri build or Rust toolchain is needed.
#
# Usage: ./scripts/sh/screenshot.sh
# Env:   MARS_SHOT_PORT  dev-server port (default 4173)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DESKTOP="$ROOT/packages/desktop"
OUT="$ROOT/assets/demo/png"
PORT="${MARS_SHOT_PORT:-4173}"
BASE="http://127.0.0.1:$PORT"

mkdir -p "$OUT"
cd "$DESKTOP"

if [ ! -d node_modules/@playwright/test ]; then
  echo "screenshot.sh: installing desktop dependencies..." >&2
  npm install --no-audit --no-fund
fi

npm run dev -- --host 127.0.0.1 --port "$PORT" > /tmp/mars-vite-dev.log 2>&1 &
VITE_PID=$!
cleanup() {
  kill "$VITE_PID" 2>/dev/null || true
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "$BASE/"; then break; fi
  sleep 1
done
curl -sf -o /dev/null "$BASE/" || { echo "screenshot.sh: dev server did not start; see /tmp/mars-vite-dev.log" >&2; exit 1; }

node "$ROOT/scripts/js/screenshot.mjs" "$BASE" "$OUT"
