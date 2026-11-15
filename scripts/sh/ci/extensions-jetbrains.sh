#!/usr/bin/env bash
#
# ci/extensions-jetbrains.sh — Local equivalent of the `extension-jetbrains` CI job.
#
# Builds the JetBrains plugin with Gradle. Requires a JDK 17+ on PATH
# (Temurin recommended; CI uses JDK 25).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
JETBRAINS_DIR="$REPO_ROOT/extensions/jetbrains"
cd "$JETBRAINS_DIR"

command -v java >/dev/null 2>&1 || {
  echo "error: java not found; install a JDK 17+ (Temurin recommended)" >&2
  exit 1
}
[ -x ./gradlew ] || {
  echo "error: $JETBRAINS_DIR/gradlew is missing or not executable" >&2
  exit 1
}

echo "==> ./gradlew buildPlugin --no-daemon --console=plain"
./gradlew buildPlugin --no-daemon --console=plain

ZIP="$(find build/distributions -maxdepth 1 -type f -name '*.zip' -print -quit 2>/dev/null || true)"
if [ -z "$ZIP" ]; then
  echo "error: no plugin zip produced in build/distributions" >&2
  exit 1
fi
echo "jetbrains: OK — $PWD/$ZIP"
