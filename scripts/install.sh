#!/usr/bin/env bash
set -Eeuo pipefail

export NODE_NO_WARNINGS=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="$REPO_ROOT/apps/desktop"
VSCODE_DIR="$REPO_ROOT/apps/vscode"
LOCAL_BIN="$HOME/.local/bin"
CARGO_BIN="$HOME/.cargo/bin"
MARS_CARGO_FEATURES="${MARS_CARGO_FEATURES:-}"
MARS_INSTALL_VERBOSE="${MARS_INSTALL_VERBOSE:-}"

INSTALL_CLI=0
INSTALL_VSCODE=0
INSTALL_DESKTOP=0

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  C_RESET=$'\033[0m'
  C_BOLD=$'\033[1m'
  C_DIM=$'\033[2m'
  C_RED=$'\033[31m'
  C_GREEN=$'\033[32m'
  C_YELLOW=$'\033[33m'
  C_BLUE=$'\033[34m'
  C_MAGENTA=$'\033[35m'
else
  C_RESET=''
  C_BOLD=''
  C_DIM=''
  C_RED=''
  C_GREEN=''
  C_YELLOW=''
  C_BLUE=''
  C_MAGENTA=''
fi

log() { printf '%s[mars]%s %s\n' "$C_DIM" "$C_RESET" "$*"; }
info() { printf '%s[mars]%s %s\n' "$C_BLUE" "$C_RESET" "$*"; }
warn() { printf '%s[mars]%s %s\n' "$C_YELLOW" "$C_RESET" "$*" >&2; }
error() { printf '%s[fail]%s %s\n' "$C_RED" "$C_RESET" "$*" >&2; }
success() { printf '%s✓%s %s\n' "$C_GREEN" "$C_RESET" "$*"; }
debug() {
  if [ -n "$MARS_INSTALL_VERBOSE" ]; then
    printf '%s[debug]%s %s\n' "$C_MAGENTA" "$C_RESET" "$*"
  fi
}

# Echo a command before running it so failures are easy to reproduce.
run() {
  printf '%s  $ %s%s\n' "$C_DIM" "$*" "$C_RESET"
  "$@"
}

STEP_INDEX=0
STEP_TOTAL=0
STEP_START=$SECONDS
INSTALL_START=$SECONDS

step() {
  STEP_INDEX=$((STEP_INDEX + 1))
  STEP_START=$SECONDS
  printf '\n%s==> [%d/%d] %s%s\n' "$C_BOLD" "$STEP_INDEX" "$STEP_TOTAL" "$*" "$C_RESET"
}

elapsed() { echo "$((SECONDS - STEP_START))s"; }

finish_step() { success "$1 (took $(elapsed))"; }

human_size() {
  if [ -e "$1" ]; then
    du -sh "$1" 2>/dev/null | cut -f1 || true
  else
    echo "missing"
  fi
}

tool_version() {
  local bin="$1"
  shift
  if command -v "$bin" >/dev/null 2>&1; then
    "$bin" "$@" 2>/dev/null | head -n 1 || true
  else
    echo "not installed"
  fi
}

on_error() {
  error "Installation failed (exit $1) near line $2."
  error "Fix the error above and re-run ./scripts/install.sh."
}

# -E (errtrace) makes this trap fire for failures inside functions too.
trap 'on_error $? "${BASH_LINENO[0]}"' ERR

usage() {
  cat <<'USAGE'
Usage: ./scripts/install.sh [options]

Options:
  --all                Install CLI, VS Code extension, and desktop app (default)
  --cli                Install only the Mars CLI
  --vscode             Install only the VS Code extension
  --desktop            Install only the desktop app
  -h, --help           Show this help

Environment:
  MARS_CARGO_FEATURES  Cargo features to pass to CLI and desktop builds (e.g. experimental-evcxr)
  MARS_INSTALL_VERBOSE Set to 1 to log extra diagnostic detail
  NO_COLOR             Disable coloured log output

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
      shift
      ;;
    --cli)
      INSTALL_CLI=1
      INSTALL_VSCODE=0
      INSTALL_DESKTOP=0
      shift
      ;;
    --vscode)
      INSTALL_CLI=0
      INSTALL_VSCODE=1
      INSTALL_DESKTOP=0
      shift
      ;;
    --desktop)
      INSTALL_CLI=0
      INSTALL_VSCODE=0
      INSTALL_DESKTOP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      error "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

# Default to all if nothing was specified
if [[ "$INSTALL_CLI" -eq 0 && "$INSTALL_VSCODE" -eq 0 && "$INSTALL_DESKTOP" -eq 0 ]]; then
  log "No target flags given; defaulting to --all"
  INSTALL_CLI=1
  INSTALL_VSCODE=1
  INSTALL_DESKTOP=1
fi

# One dependency step plus one step per selected target plus the summary.
STEP_TOTAL=$((1 + INSTALL_CLI + INSTALL_VSCODE + INSTALL_DESKTOP + 1))

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------

log "${C_BOLD}Installing Mars${C_RESET}"
log "Repository: $REPO_ROOT"
log "Platform:   $(uname -s) $(uname -m)"
log "Shell:      bash $BASH_VERSION"
log "Time:       $(date '+%Y-%m-%d %H:%M:%S %Z')"
log "Targets:    CLI=$INSTALL_CLI VS Code=$INSTALL_VSCODE Desktop=$INSTALL_DESKTOP"
if [ -n "$MARS_CARGO_FEATURES" ]; then
  log "Cargo features: $MARS_CARGO_FEATURES"
else
  debug "Cargo features: <none> (set MARS_CARGO_FEATURES to override)"
fi

# ---------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    error "'$1' was not found on PATH."
    error "$2"
    exit 1
  fi
  debug "Found $1 at $(command -v "$1")"
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

if [[ "$INSTALL_CLI" -eq 1 || "$INSTALL_DESKTOP" -eq 1 ]]; then
  info "cargo: $(tool_version cargo --version)"
fi
if [[ "$INSTALL_VSCODE" -eq 1 || "$INSTALL_DESKTOP" -eq 1 ]]; then
  info "node:  $(tool_version node --version)"
  info "npm:   $(tool_version npm --version)"
fi
if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
  info "code:  $(tool_version code --version)"
fi
if command -v pnpm >/dev/null 2>&1; then
  info "pnpm:  $(tool_version pnpm --version) ($(command -v pnpm))"
else
  debug "pnpm not found; will fall back to npm"
fi

if [[ "$(uname -s)" == "Darwin" && "$INSTALL_DESKTOP" -eq 1 ]]; then
  if [ -w "/Applications" ]; then
    debug "/Applications is writable; desktop app will install there"
  else
    debug "/Applications is not writable; desktop app will install to ~/Applications"
  fi
fi

CARGO_ARGS=(build --release --locked --bin mars)
TAURI_ARGS=(build)
if [ -n "$MARS_CARGO_FEATURES" ]; then
  CARGO_ARGS+=(--features "$MARS_CARGO_FEATURES")
  TAURI_ARGS+=(--features "$MARS_CARGO_FEATURES")
fi

debug "cargo args: ${CARGO_ARGS[*]}"
debug "tauri args: ${TAURI_ARGS[*]}"

if [ -f "$REPO_ROOT/pnpm-lock.yaml" ]; then
  debug "Found pnpm-lock.yaml"
else
  warn "No pnpm-lock.yaml found; pnpm install may fail with --frozen-lockfile"
fi

# ---------------------------------------------------------------------------
# JavaScript dependencies
# ---------------------------------------------------------------------------

step "Installing JavaScript dependencies"

if command -v pnpm >/dev/null 2>&1; then
  # The repository pins a pnpm version through packageManager. Disable pnpm's
  # self-managed version switch so an interpreter without an executable shim
  # (or an offline machine) can still use the installed pnpm.
  cd "$REPO_ROOT"
  info "Using pnpm with the repository lockfile"
  run env CI=1 npm_config_manage_package_manager_versions=false pnpm install --frozen-lockfile
else
  info "pnpm not available; installing workspace packages with npm instead"
  # Fall back to independent npm installs. Remove pnpm-linked packages first so
  # npm does not try to rebuild another package manager's symlinked tree.
  if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
    info "Removing $DESKTOP_DIR/node_modules"
    rm -rf "$DESKTOP_DIR/node_modules"
  fi
  if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
    info "Removing $VSCODE_DIR/node_modules"
    rm -rf "$VSCODE_DIR/node_modules"
  fi
  if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
    info "Installing desktop dependencies"
    (
      cd "$DESKTOP_DIR"
      run npm install --no-audit --no-fund --package-lock=false
    )
  fi
  if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
    info "Installing VS Code extension dependencies"
    (
      cd "$VSCODE_DIR"
      run npm install --no-audit --no-fund --package-lock=false
    )
  fi
fi
finish_step "JavaScript dependencies installed"

VSIX=""
INSTALLED_APP=""

# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  step "Building and installing the Mars CLI"
  cd "$REPO_ROOT"
  info "Running: cargo ${CARGO_ARGS[*]}"
  run cargo "${CARGO_ARGS[@]}"

  CLI_BINARY="$REPO_ROOT/target/release/mars"
  if [ ! -f "$CLI_BINARY" ]; then
    error "Expected CLI binary at $CLI_BINARY but it does not exist."
    exit 1
  fi
  info "Built CLI: $CLI_BINARY ($(human_size "$CLI_BINARY"))"

  mkdir -p "$LOCAL_BIN"
  install -m 755 "$CLI_BINARY" "$LOCAL_BIN/mars"
  success "Mars CLI installed to $LOCAL_BIN/mars"

  # Some environments put ~/.cargo/bin before ~/.local/bin. Keep both copies in
  # sync so `mars` always resolves to the freshly built CLI.
  if [ -d "$CARGO_BIN" ]; then
    install -m 755 "$CLI_BINARY" "$CARGO_BIN/mars"
    success "Mars CLI mirrored to $CARGO_BIN/mars"
  else
    debug "Skipping $CARGO_BIN mirror: directory does not exist"
  fi

  if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    warn "$LOCAL_BIN is not on PATH in this shell"
    warn "Add the following to your shell profile (~/.zshrc or ~/.bashrc):"
    warn "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  else
    debug "$LOCAL_BIN is already on PATH"
  fi

  finish_step "Mars CLI installed"
else
  log "Skipping CLI build/install (--cli not selected)"
fi

# ---------------------------------------------------------------------------
# VS Code extension
# ---------------------------------------------------------------------------

if [[ "$INSTALL_VSCODE" -eq 1 ]]; then
  step "Building and installing the VS Code extension"
  cd "$VSCODE_DIR"
  info "Removing old build output (out/, vsix/)"
  rm -rf out vsix
  info "Running: npm run package"
  run npm run package

  VSIX="$(find "$VSCODE_DIR/vsix" -maxdepth 1 -type f -name '*.vsix' -print -quit 2>/dev/null || true)"
  if [ -z "$VSIX" ]; then
    error "No VSIX file was produced in $VSCODE_DIR/vsix"
    exit 1
  fi
  info "Built extension: $VSIX ($(human_size "$VSIX"))"

  info "Installing extension with: code --install-extension"
  run code --install-extension "$VSIX" --force
  finish_step "Mars VS Code extension installed from $VSIX"
else
  log "Skipping VS Code extension build/install (--vscode not selected)"
fi

# ---------------------------------------------------------------------------
# Desktop app
# ---------------------------------------------------------------------------

if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
  step "Building and installing the desktop application"
  cd "$DESKTOP_DIR"
  info "Removing old frontend build output (dist/)"
  rm -rf dist
  info "Running: npm run tauri -- ${TAURI_ARGS[*]}"
  run npm run tauri -- "${TAURI_ARGS[@]}"

  APP_BUNDLE=""
  for bundle_dir in \
    "$REPO_ROOT/target/release/bundle/macos" \
    "$DESKTOP_DIR/src-tauri/target/release/bundle/macos"; do
    debug "Looking for .app bundles in $bundle_dir"
    if [ -d "$bundle_dir" ]; then
      APP_BUNDLE="$(find "$bundle_dir" -maxdepth 1 -type d -name '*.app' -print -quit)"
      if [ -n "$APP_BUNDLE" ]; then
        break
      fi
    else
      debug "Bundle directory does not exist: $bundle_dir"
    fi
  done

  if [ -z "$APP_BUNDLE" ]; then
    error "No macOS .app bundle was produced by 'npm run tauri build'."
    error "Check the Tauri build output for the bundle location on this platform."
    exit 1
  fi
  info "Built app bundle: $APP_BUNDLE ($(human_size "$APP_BUNDLE"))"

  if [ ! -w "/Applications" ]; then
    INSTALL_ROOT="$HOME/Applications"
    warn "/Applications is not writable; installing to $INSTALL_ROOT"
    mkdir -p "$INSTALL_ROOT"
  else
    INSTALL_ROOT="/Applications"
  fi

  APP_NAME="$(basename "$APP_BUNDLE")"
  INSTALLED_APP="$INSTALL_ROOT/$APP_NAME"
  info "Copying app bundle to $INSTALLED_APP"
  rm -rf "$INSTALLED_APP"
  run ditto "$APP_BUNDLE" "$INSTALLED_APP"
  finish_step "Mars desktop application installed to $INSTALLED_APP"
else
  log "Skipping desktop app build/install (--desktop not selected)"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

step "Installation complete"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  success "Mars CLI:          $LOCAL_BIN/mars"
fi
if [[ -n "$INSTALLED_APP" ]]; then
  success "Desktop app:       $INSTALLED_APP"
fi
if [[ -n "$VSIX" ]]; then
  success "VS Code extension: $VSIX"
fi
if [ -n "$MARS_CARGO_FEATURES" ]; then
  success "Cargo features:    $MARS_CARGO_FEATURES"
fi
log "Total time: $((SECONDS - INSTALL_START))s"
