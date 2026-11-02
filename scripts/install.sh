#!/usr/bin/env bash
set -euo pipefail

export NODE_NO_WARNINGS=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="$REPO_ROOT/apps/desktop"
VSCODE_DIR="$REPO_ROOT/apps/vscode"
LOCAL_BIN="$HOME/.local/bin"
CARGO_BIN="$HOME/.cargo/bin"
MARS_CARGO_FEATURES="${MARS_CARGO_FEATURES:-}"

echo "Installing Mars"
echo "Repository root: $REPO_ROOT"

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

require_command cargo "Please install Rust from https://rustup.rs/"
require_command node "Please install Node.js 20 or newer."
require_command npm "Please install npm with Node.js."
require_command code "Please install Visual Studio Code and ensure the 'code' command is in PATH."

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
  rm -rf "$DESKTOP_DIR/node_modules" "$VSCODE_DIR/node_modules"
  (
    cd "$DESKTOP_DIR"
    npm install --no-audit --no-fund --package-lock=false
  )
  (
    cd "$VSCODE_DIR"
    npm install --no-audit --no-fund --package-lock=false
  )
fi

step "Building and installing the Mars CLI"
cd "$REPO_ROOT"
CARGO_ARGS=(build --release --locked --bin mars)
TAURI_ARGS=(build)
if [ -n "$MARS_CARGO_FEATURES" ]; then
  CARGO_ARGS+=(--features "$MARS_CARGO_FEATURES")
  TAURI_ARGS+=(--features "$MARS_CARGO_FEATURES")
fi
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

step "Installation complete"
echo "✓ Mars CLI:        $LOCAL_BIN/mars"
echo "✓ Desktop app:     $INSTALLED_APP"
echo "✓ VS Code extension: $VSIX"
if [ -n "$MARS_CARGO_FEATURES" ]; then
  echo "✓ Cargo features:  $MARS_CARGO_FEATURES"
fi
