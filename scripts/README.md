# Mars Scripts

This directory contains installation and setup scripts for the Mars project.

## install.sh

Rebuilds and installs every Mars application:

```bash
./scripts/install.sh
```

This is equivalent to `./scripts/install.sh --all` and will:

1. Check for required dependencies (Rust/Cargo, Node.js/npm, and VS Code)
2. Install the desktop and VS Code JavaScript dependencies
3. Build the Mars CLI from source using `cargo build --release --locked`
4. Install the Mars CLI to `~/.local/bin/mars` (and `~/.cargo/bin/mars` when present)
5. Rebuild the VS Code extension from scratch and install it with `code`
6. Rebuild the native Tauri desktop application and install `Mars.app` to
   `/Applications` or `~/Applications`

Every run builds new artifacts; it does not reuse an existing VSIX or desktop
bundle.

### Installing a single target

Pass one or more target flags to install the specified targets. If no targets are
selected, all components are installed by default.

```bash
./scripts/install.sh --cli          # Mars CLI only
./scripts/install.sh --vscode       # VS Code extension only
./scripts/install.sh --desktop      # Desktop app only
./scripts/install.sh --cli --vscode # CLI plus extension (recommended for editors)
```

The VS Code extension executes cells through the `mars` CLI, so install the CLI too
unless it is already on `PATH`. The desktop app embeds the kernel and does not need
the CLI.

Use `./scripts/install.sh --help` for the full option list.

To include optional Cargo features in both the CLI and desktop build, set
`MARS_CARGO_FEATURES`, for example:

```bash
MARS_CARGO_FEATURES=experimental-evcxr ./scripts/install.sh
```

### Requirements

- Rust and Cargo (install from https://rustup.rs/) when installing the CLI or
  desktop app
- Node.js 20 or newer with npm when installing the extension or desktop app
- Visual Studio Code with the `code` command in PATH when installing the
  extension
- Tauri desktop prerequisites for the current platform (see
  [`docs/08-development.md`](../docs/08-development.md)) when installing the desktop
  app

On macOS, the desktop build produces `Mars.app` and a DMG under
`target/release/bundle/`. The script installs the `.app`; the DMG is left in
place for distribution.

### Notes

The script adds `~/.local/bin` to PATH for future shells when it is missing. To
make this permanent, add the following to your shell profile (`~/.zshrc` or
`~/.bashrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## screenshot.sh

Captures the desktop UI screenshots used in the README into
[`assets/demo/png/`](../assets/demo/png/):

```bash
./scripts/sh/screenshot.sh
```

The script starts the Vite dev server for `packages/desktop`, then delegates
the capture to [`scripts/js/screenshot.mjs`](js/screenshot.mjs), which drives
the page with Playwright (already a dev dependency of the desktop package) and
writes:

- `desktop-welcome.png` — the welcome screen (New/Open notebook, Open folder)
- `desktop-overview.png` — the notebook editor with the "Getting started with
  Mars" notebook

Screenshots are captured at a **1280x640 ratio** (viewport 1280x640 CSS pixels
at 2x device scale, producing 2560x1280 PNGs). The dev server is started on
demand and shut down when the script exits; override the port with
`MARS_SHOT_PORT` if 4173 is taken.

Requires Node.js and the desktop dependencies installed (`pnpm install` at the
repository root, or `npm install` inside `packages/desktop`). No Tauri build or
Rust toolchain is needed — the UI runs in the browser against the Vite dev
server.
