#!/usr/bin/env bash
#
# install-local.sh — Build (if needed) and install Project Mars components
# from this checkout.
#
# The `mars` CLI is built and installed directly here (with stale-build
# detection). Other components — desktop app, VS Code/Zed/JetBrains
# extensions — are delegated to install.sh, which already knows how to build
# and place each of them.
#
# Unlike install.sh alone, this script always installs the CLI binary
# produced by the local working tree, making it suitable for development and
# testing local changes.
#
# Usage:
#   ./scripts/install-local.sh                  # CLI only (default)
#   ./scripts/install-local.sh --cli --desktop  # CLI + desktop app
#   ./scripts/install-local.sh --vscode --zed   # extensions only
#   INSTALL_DIR=/usr/local/bin ./scripts/install-local.sh
#
# Environment:
#   INSTALL_DIR            Install prefix for the CLI (default: ~/.local/bin)
#   MARS_CARGO_FEATURES    Cargo features to pass to builds
#                          (e.g. experimental-evcxr)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_SH="$REPO_ROOT/scripts/sh/install.sh"
cd "$REPO_ROOT"

INSTALL_CLI=0
INSTALL_VSCODE=0
INSTALL_DESKTOP=0
INSTALL_ZED=0
INSTALL_JETBRAINS=0

usage() {
  cat <<'USAGE'
Usage: ./scripts/install-local.sh [options]

Options (same targets as install.sh):
  --cli          Install only the Project Mars CLI (default when no flags given)
  --desktop      Install only the desktop app
  --vscode       Install only the VS Code extension
  --zed          Install only the Zed extension
  --jetbrains    Install only the JetBrains plugin
  --all          Install every component
  -h, --help     Show this help

The CLI is always built from this checkout. Other components are delegated
to scripts/install.sh. The VS Code and JetBrains extensions need a `mars`
CLI on PATH to execute cells — include --cli (or install it first).

Environment:
  INSTALL_DIR          Install prefix for the CLI (default: ~/.local/bin)
  MARS_CARGO_FEATURES  Cargo features for CLI and desktop builds
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cli)        INSTALL_CLI=1; shift ;;
    --vscode)     INSTALL_VSCODE=1; INSTALL_CLI=0; INSTALL_DESKTOP=0; INSTALL_ZED=0; INSTALL_JETBRAINS=0; shift ;;
    --desktop)    INSTALL_DESKTOP=1; INSTALL_CLI=0; INSTALL_VSCODE=0; INSTALL_ZED=0; INSTALL_JETBRAINS=0; shift ;;
    --zed)        INSTALL_ZED=1; INSTALL_CLI=0; INSTALL_VSCODE=0; INSTALL_DESKTOP=0; INSTALL_JETBRAINS=0; shift ;;
    --jetbrains)  INSTALL_JETBRAINS=1; INSTALL_CLI=0; INSTALL_VSCODE=0; INSTALL_DESKTOP=0; INSTALL_ZED=0; shift ;;
    --all)
      INSTALL_CLI=1; INSTALL_VSCODE=1; INSTALL_DESKTOP=1; INSTALL_ZED=1; INSTALL_JETBRAINS=1
      shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown option $1" >&2; usage >&2; exit 1 ;;
  esac
done

# Default to CLI only when nothing was selected.
if [[ "$INSTALL_CLI" -eq 0 && "$INSTALL_VSCODE" -eq 0 && "$INSTALL_DESKTOP" -eq 0 && "$INSTALL_ZED" -eq 0 && "$INSTALL_JETBRAINS" -eq 0 ]]; then
  INSTALL_CLI=1
fi

# ---------------------------------------------------------------------------
# CLI (local working-tree build)
# ---------------------------------------------------------------------------

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  BINARY="$REPO_ROOT/target/release/mars"
  INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
  CARGO_BIN="$HOME/.cargo/bin"

  # --- Detect platform -------------------------------------------------------
  OS="$(uname -s)"
  ARCH="$(uname -m)"
  case "$ARCH" in
      x86_64 | amd64) ARCH="x86_64" ;;
      arm64 | aarch64) ARCH="aarch64" ;;
  esac
  OS_LOWER="$(echo "$OS" | tr '[:upper:]' '[:lower:]')"
  echo "Detected platform: ${OS_LOWER}-${ARCH}"

  case "$OS_LOWER" in
      darwin | linux) ;;
      *)
          echo "warning: untested platform '${OS}'; continuing anyway" >&2
          ;;
  esac

  # --- Ensure a fresh release build exists -----------------------------------
  needs_build=0
  if [[ ! -x "$BINARY" ]]; then
      needs_build=1
      echo "No release build found at target/release/mars"
  elif [[ Cargo.toml -nt "$BINARY" || Cargo.lock -nt "$BINARY" || rust-toolchain.toml -nt "$BINARY" ]] \
      || [[ -n "$(find crates -type f -name '*.rs' -newer "$BINARY" -print -quit 2>/dev/null)" ]]; then
      needs_build=1
      echo "Release build is stale (sources changed since last build)"
  fi

  if [[ "$needs_build" -eq 1 ]]; then
      if ! command -v cargo >/dev/null 2>&1; then
          echo "error: cargo not found; install the Rust toolchain from https://rustup.rs" >&2
          exit 1
      fi
      CARGO_ARGS=(build --release --locked --bin mars)
      if [ -n "${MARS_CARGO_FEATURES:-}" ]; then
          CARGO_ARGS+=(--features "$MARS_CARGO_FEATURES")
          echo "Cargo features: $MARS_CARGO_FEATURES"
      fi
      echo "Building release binary (this may take a while)..."
      echo "Running: cargo ${CARGO_ARGS[*]}"
      cargo "${CARGO_ARGS[@]}"
  fi

  if [[ ! -x "$BINARY" ]]; then
      echo "error: build finished but $BINARY is missing" >&2
      exit 1
  fi

  # --- Install ---------------------------------------------------------------
  mkdir -p "$INSTALL_DIR"
  install -m 755 "$BINARY" "$INSTALL_DIR/mars"
  echo "Installed Project Mars CLI to $INSTALL_DIR/mars"

  # Some environments put ~/.cargo/bin before ~/.local/bin. Keep both copies in
  # sync so `mars` always resolves to the freshly built CLI.
  if [ -d "$CARGO_BIN" ]; then
      install -m 755 "$BINARY" "$CARGO_BIN/mars"
      echo "Mirrored Project Mars CLI to $CARGO_BIN/mars"
  fi

  if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
      echo "warning: $INSTALL_DIR is not on PATH in this shell" >&2
      echo "  Add the following to your shell profile (~/.zshrc or ~/.bashrc):" >&2
      echo "    export PATH=\"\$HOME/.local/bin:\$PATH\"" >&2
  fi
fi

# ---------------------------------------------------------------------------
# Other components — delegate to install.sh
# ---------------------------------------------------------------------------

OTHER_ARGS=()
[[ "$INSTALL_VSCODE" -eq 1 ]] && OTHER_ARGS+=(--vscode)
[[ "$INSTALL_DESKTOP" -eq 1 ]] && OTHER_ARGS+=(--desktop)
[[ "$INSTALL_ZED" -eq 1 ]] && OTHER_ARGS+=(--zed)
[[ "$INSTALL_JETBRAINS" -eq 1 ]] && OTHER_ARGS+=(--jetbrains)

if [[ ${#OTHER_ARGS[@]} -gt 0 ]]; then
  if [[ ! -x "$INSTALL_SH" ]]; then
      echo "error: expected $INSTALL_SH but it is missing or not executable" >&2
      exit 1
  fi
  if [[ "$INSTALL_CLI" -eq 0 ]] && { [[ "$INSTALL_VSCODE" -eq 1 ]] || [[ "$INSTALL_JETBRAINS" -eq 1 ]]; }; then
      echo "warning: the VS Code and JetBrains extensions need a 'mars' CLI on PATH to execute cells" >&2
      echo "  Re-run with --cli (or --all) to install the CLI too" >&2
  fi
  echo "==> Delegating to scripts/install.sh ${OTHER_ARGS[*]}"
  exec "$INSTALL_SH" "${OTHER_ARGS[@]}"
fi

echo "Done. Try: mars notebook templates"
