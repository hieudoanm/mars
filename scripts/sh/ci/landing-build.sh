#!/usr/bin/env bash
#
# ci/landing-build.sh — Local equivalent of the `landing-build` CI job
# (minus the GitHub Pages artifact upload, which is Actions-only).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

command -v pnpm >/dev/null 2>&1 || {
  echo "error: pnpm not found; install it from https://pnpm.io" >&2
  exit 1
}

echo "==> pnpm install --frozen-lockfile --ignore-scripts"
CI=1 pnpm install --frozen-lockfile --ignore-scripts

echo "==> pnpm --filter @mars/landing typecheck"
pnpm --filter @mars/landing run typecheck

echo "==> pnpm --filter @mars/landing lint"
pnpm --filter @mars/landing run lint

echo "==> pnpm --filter @mars/landing test"
pnpm --filter @mars/landing run test

echo "==> pnpm --filter @mars/landing build"
pnpm --filter @mars/landing run build

echo "landing-build: OK — static export in packages/landing/out"
