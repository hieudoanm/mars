# Project Mars Scripts

Scripts for installing, building, and capturing assets for Project Mars.

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
        ├── desktop-e2e.sh
        ├── desktop-build.sh
        ├── landing-build.sh
        ├── extensions-vscode.sh
        ├── extensions-zed.sh
        └── extensions-jetbrains.sh
```

## install.sh

Rebuilds and installs every Project Mars component.

```bash
./scripts/sh/install.sh
```

Equivalent to `./scripts/sh/install.sh --all`. Each run builds fresh artifacts —
it does not reuse an existing VSIX or desktop bundle.

What it does:

1. Checks for required dependencies (Rust/Cargo, Node.js/pnpm, JDK for JetBrains)
2. Installs the workspace JavaScript dependencies
3. Builds the CLI from source with `cargo build --release --locked`
4. Installs `mars` to `~/.local/bin/mars` (and `~/.cargo/bin/mars` when present)
5. Rebuilds and installs the VS Code extension via `code`
6. Builds and stages the Zed extension (`wasm32-wasip1`)
7. Builds and installs the JetBrains plugin into JetBrains IDE config dirs
8. Builds and installs the Tauri desktop app to `/Applications` or `~/Applications`

### Installing a single target

```bash
./scripts/sh/install.sh --cli          # CLI only
./scripts/sh/install.sh --vscode       # VS Code extension only
./scripts/sh/install.sh --desktop      # Desktop app only
./scripts/sh/install.sh --cli --vscode # CLI + extension (recommended for editors)
```

- Pass one or more target flags; no flags means `--all`.
- The VS Code and JetBrains extensions execute cells through the `mars` CLI —
  install the CLI too unless it is already on `PATH`.
- The desktop app embeds the kernel and does not need the CLI.
- `./scripts/sh/install.sh --help` lists every option.

Optional Cargo features go through `MARS_CARGO_FEATURES`:

```bash
MARS_CARGO_FEATURES=experimental-evcxr ./scripts/sh/install.sh
```

### Requirements

- Rust and Cargo — needed for CLI, desktop, and Zed builds
- Node.js 20+ with pnpm — needed for extensions and the desktop app
- VS Code with `code` on `PATH` — needed for `--vscode`
- JDK 17+ (Temurin recommended) — needed for `--jetbrains`
- Tauri desktop prerequisites — needed for `--desktop`; see
  [`docs/05-development/development.md`](../docs/05-development/development.md)

On macOS, `--desktop` produces `Project Mars.app` (installed) plus a DMG under
`target/release/bundle/` (left in place for distribution).

### Notes

- The script adds `~/.local/bin` to PATH for future shells when missing.
- To make it permanent, add to `~/.zshrc` or `~/.bashrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## install-local.sh

Fast installer for local development.

```bash
./scripts/sh/install-local.sh                 # CLI only (default)
./scripts/sh/install-local.sh --cli --desktop # CLI + desktop app
./scripts/sh/install-local.sh --vscode --zed  # extensions only
./scripts/sh/install-local.sh --all           # every component
```

- Accepts the same target flags as `install.sh` (`--cli`, `--vscode`,
  `--desktop`, `--zed`, `--jetbrains`, `--all`).
- CLI builds happen from the working tree with stale-build detection
  (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or any
  `crates/**/*.rs` newer than the binary triggers a rebuild).
- Honors `MARS_CARGO_FEATURES` for the CLI build.
- `INSTALL_DIR` overrides the install prefix (default `~/.local/bin`).
- Mirrors the binary to `~/.cargo/bin/mars` when that directory exists.
- Non-CLI components are delegated to `install.sh` with the matching flags.

## screenshot.sh

Captures the desktop UI screenshots used in the README into
[`assets/demo/png/`](../assets/demo/png/):

```bash
./scripts/sh/screenshot.sh
```

- Starts the Vite dev server for `packages/desktop`, then runs
  [`scripts/js/screenshot.mjs`](js/screenshot.mjs).
- The capture is driven by Playwright (a desktop dev dependency) and writes:
  - `desktop-welcome.png` (welcome screen and notebook welcome)
  - `desktop-lab.png`
  - `desktop-lab-new-job.png`
  - `desktop-notebook.png`
  - `desktop-notebook-explorer.png`
- `terminal.png` and `demo.png` come from `assets/demo/svg-to-gif.sh` instead.
- Screenshots are captured at a **1280x640 ratio**.
- The dev server starts on demand and stops when the script exits.
- `MARS_SHOT_PORT` overrides the default port 4173.
- Requires Node.js only — the script runs `npm install` in `packages/desktop`
  if Playwright is missing. No Tauri build or Rust toolchain needed.

## ci/

Local equivalents of the jobs in
[`.github/workflows/ci.yaml`](../.github/workflows/ci.yaml) — one script per job:

| Script                       | CI job                | What it runs                                                                                                    |
| ---------------------------- | --------------------- | --------------------------------------------------------------------------------------------------------------- |
| `ci/cli-check.sh`            | `cli-check`           | `cargo fmt --check`, `clippy --workspace -D warnings`, `cargo test --workspace` (installs webkit deps on Linux) |
| `ci/cli-build.sh`            | `cli-build`           | `cargo build --release -p mars --locked`, renames binary to the target triple                                   |
| `ci/desktop-check.sh`        | `desktop-check`       | `pnpm install` + typecheck/lint/test for `@mars/desktop`                                                        |
| `ci/desktop-e2e.sh`          | `desktop-e2e`         | Tauri `--no-bundle` build + tauri-driver suite under Xvfb (Linux only; needs `webkit2gtk-driver`)               |
| `ci/desktop-build.sh`        | `desktop-build`       | Host-OS Tauri bundles (dmg / deb,rpm,appimage / msi,nsis)                                                       |
| `ci/landing-build.sh`        | `landing-build`       | `@mars/landing` typecheck/lint/test/build → `out/`                                                              |
| `ci/extensions-vscode.sh`    | `extension-vscode`    | `mars-vscode` checks + `package` → `.vsix`                                                                      |
| `ci/extensions-zed.sh`       | `extension-zed`       | `mars-zed` fmt/clippy/test + `wasm32-wasip1` build                                                              |
| `ci/extensions-jetbrains.sh` | `extension-jetbrains` | `./gradlew buildPlugin` (JDK 17+)                                                                               |

- The `release` job is GitHub-Actions-only (artifact download + `gh release`)
  and has no local script.
- `desktop-build.sh` builds only the host OS's bundles; CI runs the full
  macOS/Linux/Windows matrix.

```bash
./scripts/sh/ci/cli-check.sh        # run a single job locally
./scripts/sh/ci/extensions-zed.sh
```

## demo-sessions/

- `demo-sessions/mars.json` is the terminal-session script consumed by
  `assets/demo/svg-to-gif.sh` to produce the README demo assets under
  `assets/demo/`.
