# Project Mars Scripts

This directory contains installation, CI, and asset-generation scripts for the
Project Mars project.

## Layout

```text
scripts/
├── install.sh            # scripts/sh/install.sh — full component installer
├── README.md             # this file
├── demo-sessions/        # terminal demo session data (mars.json)
├── js/
│   └── screenshot.mjs    # Playwright capture used by screenshot.sh
└── sh/
    ├── install.sh        # install every component (CLI, desktop, extensions)
    ├── install-local.sh  # fast local-checkout installer (CLI + delegates)
    ├── screenshot.sh     # capture README demo screenshots
    └── ci/               # per-job equivalents of .github/workflows/ci.yaml
        ├── cli-check.sh
        ├── cli-build.sh
        ├── desktop-check.sh
        ├── desktop-build.sh
        ├── landing-build.sh
        ├── extensions-vscode.sh
        ├── extensions-zed.sh
        └── extensions-jetbrains.sh
```

## install.sh

Rebuilds and installs every Project Mars application:

```bash
./scripts/sh/install.sh
```

This is equivalent to `./scripts/sh/install.sh --all` and will:

1. Check for required dependencies (Rust/Cargo, Node.js/pnpm, JDK for JetBrains)
2. Install the workspace JavaScript dependencies
3. Build the Project Mars CLI from source using `cargo build --release --locked`
4. Install the Project Mars CLI to `~/.local/bin/mars` (and `~/.cargo/bin/mars` when present)
5. Rebuild the VS Code extension and install it with `code`
6. Build and stage the Zed extension (`wasm32-wasip1`)
7. Build and install the JetBrains plugin into JetBrains IDE config dirs
8. Rebuild the native Tauri desktop application and install `Project Mars.app`
   to `/Applications` or `~/Applications`

Every run builds new artifacts; it does not reuse an existing VSIX or desktop
bundle.

### Installing a single target

Pass one or more target flags to install the specified targets. If no targets
are selected, all components are installed by default.

```bash
./scripts/sh/install.sh --cli          # Project Mars CLI only
./scripts/sh/install.sh --vscode       # VS Code extension only
./scripts/sh/install.sh --desktop      # Desktop app only
./scripts/sh/install.sh --cli --vscode # CLI plus extension (recommended for editors)
```

The VS Code and JetBrains extensions execute cells through the `mars` CLI, so
install the CLI too unless it is already on `PATH`. The desktop app embeds the
kernel and does not need the CLI.

Use `./scripts/sh/install.sh --help` for the full option list.

To include optional Cargo features in both the CLI and desktop build, set
`MARS_CARGO_FEATURES`, for example:

```bash
MARS_CARGO_FEATURES=experimental-evcxr ./scripts/sh/install.sh
```

### Requirements

- Rust and Cargo (install from https://rustup.rs/) when installing the CLI,
  desktop app, or Zed extension
- Node.js 20 or newer with pnpm when installing the extension or desktop app
- Visual Studio Code with the `code` command in PATH when installing the
  extension
- JDK 17+ (Temurin recommended) when installing the JetBrains plugin
- Tauri desktop prerequisites for the current platform (see
  [`docs/05-development/development.md`](../docs/05-development/development.md))
  when installing the desktop app

On macOS, the desktop build produces `Project Mars.app` and a DMG under
`target/release/bundle/`. The script installs the `.app`; the DMG is left in
place for distribution.

### Notes

The script adds `~/.local/bin` to PATH for future shells when it is missing. To
make this permanent, add the following to your shell profile (`~/.zshrc` or
`~/.bashrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## install-local.sh

Fast installer for development: builds the `mars` CLI from the local working
tree (with stale-build detection) and installs it to `~/.local/bin`. Other
components are delegated to `install.sh`.

```bash
./scripts/sh/install-local.sh                 # CLI only (default)
./scripts/sh/install-local.sh --cli --desktop # CLI + desktop app
./scripts/sh/install-local.sh --vscode --zed  # extensions only
./scripts/sh/install-local.sh --all           # every component
```

Accepts the same target flags as `install.sh` (`--cli`, `--vscode`,
`--desktop`, `--zed`, `--jetbrains`, `--all`). The CLI build honors
`MARS_CARGO_FEATURES`; `INSTALL_DIR` overrides the install prefix (default
`~/.local/bin`).

## screenshot.sh

Captures the desktop UI screenshots used in the README into
[`assets/demo/png/`](../assets/demo/png/):

```bash
./scripts/sh/screenshot.sh
```

The script starts the Vite dev server for `packages/desktop`, then delegates
the capture to [`scripts/js/screenshot.mjs`](js/screenshot.mjs), which drives
the page with Playwright (already a dev dependency of the desktop package) and
writes `desktop-welcome.png`, `desktop-overview.png`, `desktop-notebook.png`,
`desktop-notebook-explorer.png`, `desktop-lab.png`, `desktop-lab-new-job.png`,
and `terminal.png`.

Screenshots are captured at a **1280x640 ratio**. The dev server is started on
demand and shut down when the script exits; override the port with
`MARS_SHOT_PORT` if 4173 is taken.

Requires Node.js and the desktop dependencies installed (`pnpm install` at the
repository root). No Tauri build or Rust toolchain is needed — the UI runs in
the browser against the Vite dev server.

## ci/

Local equivalents of the jobs in [`.github/workflows/ci.yaml`](../.github/workflows/ci.yaml) —
one script per job, runnable on a dev machine:

| Script                        | CI job             | What it runs                                              |
| ----------------------------- | ------------------ | --------------------------------------------------------- |
| `ci/cli-check.sh`             | `cli-check`        | `cargo fmt --check`, `clippy --workspace -D warnings`, `cargo test --workspace` (installs webkit deps on Linux) |
| `ci/cli-build.sh`             | `cli-build`        | `cargo build --release -p mars --locked`, renames binary to the target triple |
| `ci/desktop-check.sh`         | `desktop-check`    | `pnpm install` + typecheck/lint/test for `@mars/desktop`  |
| `ci/desktop-build.sh`         | `desktop-build`    | Host-OS Tauri bundles (dmg / deb,rpm,appimage / msi,nsis) |
| `ci/landing-build.sh`         | `landing-build`    | `@mars/landing` typecheck/lint/test/build → `out/`        |
| `ci/extensions-vscode.sh`     | `extension-vscode` | `mars-vscode` checks + `package` → `.vsix`                |
| `ci/extensions-zed.sh`        | `extension-zed`    | `mars-zed` fmt/clippy/test + `wasm32-wasip1` build        |
| `ci/extensions-jetbrains.sh`  | `extension-jetbrains` | `./gradlew buildPlugin` (JDK 17+)                      |

The `release` job is GitHub-Actions-only (artifact download + `gh release`)
and has no local script. `desktop-build.sh` builds only the host OS's bundles;
CI runs the full macOS/Linux/Windows matrix.

```bash
./scripts/sh/ci/cli-check.sh        # run a single job locally
./scripts/sh/ci/extensions-zed.sh
```

## demo-sessions/

`demo-sessions/mars.json` is the terminal-session script consumed by
`assets/demo/svg-to-gif.sh` (and `scripts/js/screenshot.mjs`-style demo
recording) to produce the README demo assets under `assets/demo/`.
