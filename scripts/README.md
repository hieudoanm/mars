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
