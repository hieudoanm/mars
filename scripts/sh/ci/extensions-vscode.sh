#!/usr/bin/env bash
#
# ci/extensions-vscode.sh — Local equivalent of the `extension-vscode` CI job.
#
# Typechecks, lints, tests, builds, and packages the VS Code extension,
# leaving the .vsix under extensions/vscode/vsix/.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

command -v pnpm >/dev/null 2>&1 || {
  echo "error: pnpm not found; install it from https://pnpm.io" >&2
  exit 1
}

echo "==> pnpm install --frozen-lockfile --ignore-scripts"
CI=1 pnpm install --frozen-lockfile --ignore-scripts

echo "==> pnpm --filter mars-vscode typecheck"
pnpm --filter mars-vscode run typecheck

echo "==> pnpm --filter mars-vscode lint"
pnpm --filter mars-vscode run lint

echo "==> pnpm --filter mars-vscode test"
pnpm --filter mars-vscode run test

echo "==> pnpm --filter mars-vscode build"
pnpm --filter mars-vscode run build

echo "==> pnpm --filter mars-vscode package"
pnpm --filter mars-vscode run package

echo "vscode: OK — VSIX under extensions/vscode/vsix/"
