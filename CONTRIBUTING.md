# Contributing to Project Mars

Thank you for contributing to Project Mars. The project is intentionally split between
Rust crates and a small set of Node applications. Keep domain behavior in the
Rust workspace, keep UI behavior in the relevant application, and update the
documentation whenever a supported workflow or limitation changes.

## Development setup

Install the latest stable Rust toolchain, Node.js 20 or newer, and pnpm
12.8.0. From the repository root:

```bash
corepack enable
corepack prepare pnpm@12.8.0 --activate
pnpm install
```

The Node workspace contains:

- `@mars/desktop` — React/Vite frontend and Tauri shell.
- `@mars/vscode` — VS Code language and notebook companion.
- `@mars/landing` — Next.js static landing page with Markdown docs.

Turborepo coordinates package tasks and caches successful work locally:

```bash
pnpm run check
pnpm run build
pnpm run dev
```

Run a specific package task with a Turbo filter:

```bash
pnpm turbo run build --filter=@mars/landing
pnpm turbo run check --filter=@mars/desktop
pnpm turbo run check --filter=@mars/vscode
```

The Rust workspace is validated independently:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Rust crates under `crates/` must maintain at least **80% line and function
coverage**. `make coverage` runs `cargo llvm-cov` over those crates with
`--fail-under-lines 80` and `--fail-under-functions 80`, so it exits non-zero
when coverage drops below either threshold. CI runs the same `make coverage`
gate; install `cargo-llvm-cov` before running it locally.

## Running applications

Run the desktop frontend in a browser:

```bash
pnpm --filter @mars/desktop dev
```

Run the native Tauri application:

```bash
cd packages/desktop
npm run tauri dev
```

Run the landing page:

```bash
pnpm --filter @mars/landing dev
```

Run the VS Code extension compiler:

```bash
pnpm run vscode:compile
```

## Pre-commit hooks

Install `pre-commit` with your preferred Python environment, then enable and
run the hooks:

```bash
pre-commit install
pre-commit run --all-files
```

The hooks cover file hygiene, YAML/JSON/TOML validation, Rust formatting,
Clippy, workspace tests, the desktop build, VS Code packaging, and landing
page checks. They are intentionally comprehensive and may take longer than a
typical commit.

## Tests and native validation

The desktop Playwright tests run against Vite and validate the browser editor
surface:

```bash
pnpm run desktop:e2e
```

They do not launch the native Tauri shell. Native launch, save/reopen, dialog,
and platform integration tests require a configured WebDriver/`tauri-driver`
environment and should be run on the target operating system.

## Architecture

Keep document management, execution, protocol, rendering, and UI logic in
their respective crates or application layers. Do not put persistence or
runtime behavior in React components.

Before submitting a change, update `README.md`, `PROGRESS.md`, or the relevant
document when behavior or limitations change. Add focused tests for new
runtime, format, protocol, or conversion behavior.

When changing a public format or command:

1. Update the implementation and focused tests.
2. Update the relevant reference documentation.
3. Update an example or fixture when it improves discoverability.
4. Run the smallest relevant validation, followed by the baseline checks when
   the change crosses crate/application boundaries.

## Code standards

Project Mars uses explicit error handling, strong Rust types, and strict TypeScript.
Avoid `unwrap()` and `expect()` in production Rust code, and do not introduce
TypeScript `any`. User code must execute outside the desktop application
process.
