<p align="center">
  <img src="assets/svg/mars-logo.svg" alt="Mars logo" width="128">
</p>

<h1 align="center">Mars</h1>

<p align="center">
  Explore ideas. Execute code. Discover possibilities.
</p>

Mars is a Rust-first interactive notebook platform for executable code,
Markdown, mathematics, and rich output. It uses an independent `.mars`
document format and supports Jupyter import/export without being a Jupyter
frontend.

## Status

The project is in early execution development. The current workspace provides
typed notebook documents, validation, atomic persistence, cell operations,
isolated replay-based Rust execution, Cargo-backed notebook dependencies,
Jupyter conversion with common rich MIME outputs, sanitized HTML rendering,
and a functional Tauri desktop editor. The React landing page and optional VS
Code companion are included in the pnpm/Turborepo workspace.

## Quick start

### CLI

Install Rust 1.85 or newer, then run Mars directly from the workspace:

```bash
cargo run -p mars -- new analysis.mars
cargo run -p mars -- inspect analysis.mars
cargo run -p mars -- run analysis.mars
cargo run -p mars -- import notebook.ipynb analysis.mars
cargo run -p mars -- export analysis.mars ipynb notebook.ipynb
cargo run -p mars -- export analysis.mars html analysis.html
cargo test --workspace
```

### Node workspace

The Node applications use pnpm and Turborepo:

```bash
corepack enable
corepack prepare pnpm@10.12.1 --activate
pnpm install
pnpm run check
pnpm run build
```

The workspace packages are `@mars/desktop`, `@mars/vscode`, and
`@mars/landing`. Use `pnpm run dev` to start the development tasks in
parallel, or filter a task with
`pnpm turbo run build --filter=@mars/landing`.

For an installed CLI binary:

```bash
cargo install --path crates/mars-cli
mars new analysis.mars
```

### Desktop application

Install the stable Rust toolchain, Node.js 20 or newer, and the platform
prerequisites documented in [`docs/development.md`](docs/development.md).

```bash
cd apps/desktop
npm install
npm run tauri dev
```

When dependencies are installed from the root workspace, the equivalent
frontend command is:

```bash
pnpm --filter @mars/desktop dev
```

The desktop application is the canonical interactive notebook experience.
Native release bundles are produced with:

```bash
npm run tauri build
```

See [`docs/release.md`](docs/release.md) for signing, platform targets, and
artifact verification.

`mars run` reports each executable cell's position, execution count, duration,
output count, and captured output. If a cell fails, Mars persists the partial
results and exits unsuccessfully after reporting the failing cell. Long-running
cells can be bounded with `mars run analysis.mars --timeout-ms 5000`; a timed
out cell is marked interrupted and is not committed to the persistent runtime
session.

See `AGENTS.md` for the product architecture and development strategy.
Contributor and subsystem documentation is available in
`CONTRIBUTING.md`, `docs/architecture.md`, `docs/notebook-format.md`,
`docs/runtime.md`, `docs/cli.md`, `docs/protocol.md`, `docs/development.md`,
`docs/release.md`, and `docs/branding.md`.

The execution runtime recompiles the accumulated cells in an isolated temporary
program. This preserves Rust bindings between cells without executing notebook
code in the CLI process, but earlier side effects are replayed. A conservative
final expression is rendered automatically with `Debug`, with a `Display`
fallback for values that do not implement `Debug`, so common cells such as
`numbers.iter().sum::<i32>()` display a result without an explicit `println!`.
Multiline expressions and trailing line comments are supported; item
declarations and statement-only cells still require explicit output. Runtime
timeouts cover both compilation and execution, terminating the active child
process when the total cell limit expires.

HTML export renders Markdown, code, text, errors, tables, images, sanitized
HTML outputs, common LaTeX expressions, and supported line/point/bar chart
specifications into a standalone dark-themed document. Unknown MIME bundles are
represented safely as fallback blocks.

Rust notebooks may declare registry dependencies in `metadata.dependencies`,
for example `{ "serde": "1" }`. Mars creates a temporary Cargo project for
those notebooks; dependency resolution therefore uses Cargo's configured cache
and registry and may require network access.

The desktop shell is under `apps/desktop`. Run `npm install` there for the
package-local workflow, or install the root pnpm workspace as described above.
Then run `npm run tauri dev` to launch the Tauri application. The current editor keeps
notebook cells in the frontend and delegates Rust execution, reset, and status
to typed Tauri commands backed by `mars-kernel`.

The desktop editor supports running one cell, all cells, cells above or below
the current cell, clearing outputs, and the core notebook keyboard shortcuts.
The notebook toolbar also accepts an optional execution limit in milliseconds;
leaving it blank preserves unlimited execution. While a cell is running, Stop
requests cancellation of the active compiler or runtime process.
Opened notebooks show unsaved state and autosave after a short idle period.
Untitled notebooks still require an explicit Save destination. Opening another
notebook while the current one is dirty offers Cancel, Open without saving, and
Save and open. Closing the desktop window uses the same protection.

The desktop browser smoke tests use Playwright and cover the editor shell and
basic cell workflows:

```bash
cd apps/desktop
npm run test:e2e
```

These tests run against Vite and do not yet launch the native Tauri shell.

## Screenshots

The desktop application uses a dark, notebook-oriented editor with CodeMirror
cells, execution controls, rich output panels, and native open/save dialogs.
![Mars desktop notebook overview](docs/screenshots/desktop-overview.png)

Release screenshots belong in [`docs/screenshots/`](docs/screenshots/README.md);
the capture checklist and naming convention are documented there.

## VS Code extension

The optional companion extension under [`apps/vscode`](apps/vscode) provides
`.mars` syntax highlighting, cell navigation, and CLI-backed HTML export. The
Tauri desktop app remains the canonical notebook experience; the extension
reuses the Mars CLI rather than duplicating the runtime or persistence layer.

## Documentation map

| Document | Purpose |
| --- | --- |
| [`docs/architecture.md`](docs/architecture.md) | Crate boundaries, application boundaries, and data flow |
| [`docs/notebook-format.md`](docs/notebook-format.md) | `.mars` schema, validation, migrations, and Jupyter mapping |
| [`docs/runtime.md`](docs/runtime.md) | Replay execution, dependencies, output extraction, diagnostics, and cancellation |
| [`docs/cli.md`](docs/cli.md) | CLI commands, options, exit behavior, and examples |
| [`docs/protocol.md`](docs/protocol.md) | Typed kernel messages and JSONL transport |
| [`docs/development.md`](docs/development.md) | Toolchains, local development, tests, and architecture rules |
| [`docs/release.md`](docs/release.md) | Native artifacts, verification, signing, and release CI |
| [`docs/branding.md`](docs/branding.md) | Canonical logo and social-preview assets |
