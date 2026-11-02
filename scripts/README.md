# Mars Scripts

This directory contains installation and setup scripts for the Mars project.

## install.sh

Rebuilds and installs every Mars application:

```bash
./scripts/install.sh
```

This script will:

1. Check for required dependencies (Rust/Cargo, Node.js/npm, and VS Code)
2. Install the desktop and VS Code JavaScript dependencies
3. Build the Mars CLI from source using `cargo build --release --locked`
4. Install the Mars CLI to `~/.local/bin/mars` (and `~/.cargo/bin/mars` when present)
5. Rebuild the VS Code extension from scratch and install it with `code`
6. Rebuild the native Tauri desktop application and install `Mars.app` to
   `/Applications` or `~/Applications`

Every run builds new artifacts; it does not reuse an existing VSIX or desktop
bundle.

To include optional Cargo features in both the CLI and desktop build, set
`MARS_CARGO_FEATURES`, for example:

```bash
MARS_CARGO_FEATURES=experimental-evcxr ./scripts/install.sh
```

### Requirements

- Rust and Cargo (install from https://rustup.rs/)
- Node.js 20 or newer with npm
- Visual Studio Code with the `code` command in PATH
- Tauri desktop prerequisites for the current platform (see
  [`docs/development.md`](../docs/development.md))

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
