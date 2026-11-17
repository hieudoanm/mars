#!/usr/bin/env bash
#
# ci/desktop-check.sh — Local equivalent of the `desktop-check` CI job.
#
# Installs workspace dependencies and runs typecheck, lint, and tests for
# @mars/desktop.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

for tool in node npm; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "error: $tool not found; install Node.js 26.13.1 (see .nvmrc)" >&2
    exit 1
  }
done
if command -v pnpm >/dev/null 2>&1; then
  PKG_MGR=(pnpm)
else
  echo "warning: pnpm not found; CI uses pnpm — install it for parity" >&2
  exit 1
fi

echo "==> pnpm install --frozen-lockfile --ignore-scripts"
CI=1 "${PKG_MGR[@]}" install --frozen-lockfile --ignore-scripts

echo "==> pnpm --filter @mars/desktop typecheck"
"${PKG_MGR[@]}" --filter @mars/desktop run typecheck

echo "==> pnpm --filter @mars/desktop lint"
"${PKG_MGR[@]}" --filter @mars/desktop run lint

echo "==> pnpm --filter @mars/desktop test"
"${PKG_MGR[@]}" --filter @mars/desktop run test

echo "desktop-check: OK"
