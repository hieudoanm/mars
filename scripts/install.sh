#!/usr/bin/env bash
set -euo pipefail

export NODE_NO_WARNINGS=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="$REPO_ROOT/apps/desktop"
VSCODE_DIR="$REPO_ROOT/apps/vscode"
LOCAL_BIN="$HOME/.local/bin"
CARGO_BIN="$HOME/.cargo/bin"
MARS_CARGO_FEATURES="${MARS_CARGO_FEATURES:-}"

INSTALL_CLI=0
INSTALL_VSCODE=0
INSTALL_DESKTOP=0
SELECTED_TARGET=0

usage() {
  cat <<'USAGE'
Usage: ./scripts/install.sh [options]

Options:
  --all                Install CLI, VS Code extension, and desktop app (default)
  --cli                Install only the Mars CLI
  --vscode             Install only the VS Code extension
  --vscode-only        Alias for --vscode
  --desktop            Install only the desktop app
  --no-cli             Skip CLI build/install
  --no-vscode          Skip VS Code extension build/install
  --no-desktop         Skip desktop app build/install
  -h, --help           Show this help

Environment:
  MARS_CARGO_FEATURES  Cargo features to pass to CLI and desktop builds (e.g. experimental-evcxr)

Note: the VS Code extension needs a `mars` CLI on PATH to execute cells.
Install both with `./scripts/install.sh --cli --vscode` or the default `--all`.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      INSTALL_CLI=1
      INSTALL_VSCODE=1
      INSTALL_DESKTOP=1
      SELECTED_TARGET=1
      shift
      ;;
    --cli)
      INSTALL_CLI=1
      SELECTED_TARGET=1
      shift
      ;;
    --vscode|--vscode-only)
      INSTALL_VSCODE=1
      SELECTED_TARGET=1
      shift
      ;;
    --desktop)
      INSTALL_DESKTOP=1
      SELECTED_TARGET=1
      shift
      ;;
    --no-cli)
      INSTALL_CLI=0
      SELECTED_TARGET=1
      shift
      ;;
    --no-vscode)
      INSTALL_VSCODE=0
      SELECTED_TARGET=1
      shift
      ;;
    --no-desktop)
      INSTALL_DESKTOP=0
      SELECTED_TARGET=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "$SELECTED_TARGET" -eq 0 ]]; then
  INSTALL_CLI=1
  INSTALL_VSCODE=1
  INSTALL_DESKTOP=1
fi

if [[ "$INSTALL_CLI" -eq 0 && "$INSTALL_VSCODE" -eq 0 && "$INSTALL_DESKTOP" -eq 0 ]]; then
  echo "Nothing to install."
  usage
  exit 1
fi

echo "Installing Mars"
echo "Repository root: $REPO_ROOT"
echo "Targets: CLI=${INSTALL_CLI} VSCode=${INSTALL_VSCODE} Desktop=${INSTALL_DESKTOP}"

step() {
  echo
  echo "==> $1"
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: '$1' not found"
    echo "$2"
    exit 1
  fi
}

if [[ "$INSTALL_CLI" -eq 1 || "$INSTALL_DESKTOP" -eq 1 ]]; then
  require_command cargo "Please install Rust from https://rustup.rs/"
fi
if [[ "$INSTALL_VSCODE" -eq 1 || "$INSTALL_DESKTOP" -eq 1 ]]; then
  require_command node "Please install Node.js 20 or newer."
  require_command npm "Please install npm with Node.js."
fi
if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
  require_command code "Please install Visual Studio Code and ensure the 'code' command is in PATH."
fi

CARGO_ARGS=(build --release --locked --bin mars)
TAURI_ARGS=(build)
if [ -n "$MARS_CARGO_FEATURES" ]; then
  CARGO_ARGS+=(--features "$MARS_CARGO_FEATURES")
  TAURI_ARGS+=(--features "$MARS_CARGO_FEATURES")
fi

step "Installing JavaScript dependencies"
if command -v pnpm >/dev/null 2>&1; then
  # The repository pins a pnpm version through packageManager. Disable pnpm's
  # self-managed version switch so an interpreter without an executable shim
  # (or an offline machine) can still use the installed pnpm.
  cd "$REPO_ROOT"
  CI=1 npm_config_manage_package_manager_versions=false pnpm install --frozen-lockfile
else
  # Fall back to independent npm installs. Remove pnpm-linked packages first so
  # npm does not try to rebuild another package manager's symlinked tree.
  if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
    rm -rf "$DESKTOP_DIR/node_modules"
  fi
  if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
    rm -rf "$VSCODE_DIR/node_modules"
  fi
  if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
    (
      cd "$DESKTOP_DIR"
      npm install --no-audit --no-fund --package-lock=false
    )
  fi
  if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
    (
      cd "$VSCODE_DIR"
      npm install --no-audit --no-fund --package-lock=false
    )
  fi
fi

VSIX=""
INSTALLED_APP=""

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  step "Building and installing the Mars CLI"
  cd "$REPO_ROOT"
  cargo "${CARGO_ARGS[@]}"

  mkdir -p "$LOCAL_BIN"
  install -m 755 "$REPO_ROOT/target/release/mars" "$LOCAL_BIN/mars"
  # Some environments put ~/.cargo/bin before ~/.local/bin. Keep both copies in
  # sync so `mars` always resolves to the freshly built CLI.
  if [ -d "$CARGO_BIN" ]; then
    install -m 755 "$REPO_ROOT/target/release/mars" "$CARGO_BIN/mars"
  fi
  echo "✓ Mars CLI installed to $LOCAL_BIN/mars"

  if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    echo "Add the following to your shell profile (~/.zshrc or ~/.bashrc):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  fi
else
  echo
  echo "Skipping CLI build/install"
fi

if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
  step "Building and installing the VS Code extension"
  cd "$VSCODE_DIR"
  rm -rf out vsix
  npm run package
  VSIX="$(ls "$VSCODE_DIR"/vsix/*.vsix 2>/dev/null | head -n 1)"
  if [ -z "$VSIX" ]; then
    echo "Error: no VSIX file was produced in $VSCODE_DIR/vsix"
    exit 1
  fi
  code --install-extension "$VSIX" --force
  echo "✓ Mars VS Code extension installed from $VSIX"
else
  echo
  echo "Skipping VS Code extension build/install"
fi

if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
  step "Building and installing the desktop application"
  cd "$DESKTOP_DIR"
  rm -rf dist
  npm run tauri -- "${TAURI_ARGS[@]}"

  APP_BUNDLE=""
  for bundle_dir in \
    "$REPO_ROOT/target/release/bundle/macos" \
    "$DESKTOP_DIR/src-tauri/target/release/bundle/macos"; do
    if [ -d "$bundle_dir" ]; then
      APP_BUNDLE="$(find "$bundle_dir" -maxdepth 1 -type d -name '*.app' -print -quit)"
      if [ -n "$APP_BUNDLE" ]; then
        break
      fi
    fi
  done

  if [ -z "$APP_BUNDLE" ]; then
    echo "Error: no macOS .app bundle was produced by 'npm run tauri build'."
    echo "Check the Tauri build output for the bundle location on this platform."
    exit 1
  fi

  if [ ! -w "/Applications" ]; then
    INSTALL_ROOT="$HOME/Applications"
    mkdir -p "$INSTALL_ROOT"
  else
    INSTALL_ROOT="/Applications"
  fi

  APP_NAME="$(basename "$APP_BUNDLE")"
  INSTALLED_APP="$INSTALL_ROOT/$APP_NAME"
  rm -rf "$INSTALLED_APP"
  ditto "$APP_BUNDLE" "$INSTALLED_APP"
  echo "✓ Mars desktop application installed to $INSTALLED_APP"
else
  echo
  echo "Skipping desktop app build/install"
fi

step "Installation complete"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  echo "✓ Mars CLI:        $LOCAL_BIN/mars"
fi
if [[ -n "$INSTALLED_APP" ]]; then
  echo "✓ Desktop app:     $INSTALLED_APP"
fi
if [[ -n "$VSIX" ]]; then
  echo "✓ VS Code extension: $VSIX"
fi
if [ -n "$MARS_CARGO_FEATURES" ]; then
  echo "✓ Cargo features:  $MARS_CARGO_FEATURES"
fi
