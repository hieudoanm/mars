#!/usr/bin/env bash
set -euo pipefail

export NODE_NO_WARNINGS=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VSCODE_DIR="$REPO_ROOT/apps/vscode"
echo "Installing Mars VSCode extension"
echo "Repository root: $REPO_ROOT"
echo "VSCode extension directory: $VSCODE_DIR"

# Check if code command is available
if ! command -v code &> /dev/null; then
  echo "Error: 'code' command not found"
  echo "Please install Visual Studio Code and ensure it's in your PATH"
  exit 1
fi

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
  echo "Error: Rust/Cargo not found"
  echo "Please install Rust from https://rustup.rs/"
  exit 1
fi

# Build Mars CLI
echo "Building Mars CLI..."
cd "$REPO_ROOT"
cargo build --release --bin mars

if [ $? -ne 0 ]; then
  echo "✗ Mars CLI build failed"
  exit 1
fi

echo "✓ Mars CLI built successfully"

# Install Mars CLI to local directory
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"
cp "$REPO_ROOT/target/release/mars" "$LOCAL_BIN/mars"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
  echo "Adding $LOCAL_BIN to PATH..."
  export PATH="$LOCAL_BIN:$PATH"
  echo "Add the following to your shell profile (~/.zshrc or ~/.bashrc):"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo "✓ Mars CLI installed to $LOCAL_BIN/mars"

# Check if VSIX exists, build if needed
if [ ! -d "$VSCODE_DIR/vsix" ] || [ -z "$(ls -A "$VSCODE_DIR/vsix"/*.vsix 2>/dev/null)" ]; then
  echo "No VSIX file found in $VSCODE_DIR/vsix/"
  echo "Building extension first..."
  cd "$VSCODE_DIR"
  npm run package
  echo "Build complete"
fi

VSIX="$(ls "$VSCODE_DIR"/vsix/*.vsix 2>/dev/null | head -n 1)"

if [ -z "$VSIX" ]; then
  echo "Error: No VSIX file found after build attempt"
  echo "Please run 'npm run package' manually first"
  exit 1
fi

echo "Found VSIX: $VSIX"
echo "Installing extension (force)..."
code --install-extension "$VSIX" --force

if [ $? -eq 0 ]; then
  echo "✓ Mars VSCode extension installed successfully"
  echo "You may need to reload VSCode to activate the extension"
else
  echo "✗ Installation failed"
  exit 1
fi
