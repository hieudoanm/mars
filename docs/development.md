# Development guide

This guide covers the local commands used to develop and validate Mars. The
repository has two build systems: Cargo for Rust crates and pnpm/Turborepo for
Node applications.

## Toolchain

Install the latest stable Rust toolchain, Node.js 20 or newer, and pnpm
10.12.1. From the repository root:

```bash
corepack enable
corepack prepare pnpm@10.12.1 --activate
pnpm install

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm exec turbo run check
pnpm exec turbo run build
```

The root `Makefile` provides short aliases for the Rust and desktop workflows:

```bash
make check
make build
make desktop-build
make e2e
make benchmark CELLS=16
```

## Workspace commands

The workspace package names are:

| Package | Directory | Main commands |
| --- | --- | --- |
| `@mars/desktop` | `apps/desktop` | `dev`, `build`, `check`, `test:e2e`, `tauri` |
| `@mars/vscode` | `apps/vscode` | `check`, `build`, `compile`, `package` |
| `@mars/landing` | `apps/landing` | `dev`, `build`, `check`, `serve` |

Use Turbo filters when working on one package:

```bash
pnpm turbo run check --filter=@mars/desktop
pnpm turbo run build --filter=@mars/landing
pnpm turbo run package --filter=@mars/vscode
```

Use `pnpm run dev` for parallel development servers. The landing page and
desktop Vite servers are long-running tasks; stop them with the terminal
session that started them.

Use `npm run tauri dev` from `apps/desktop` for the native application. The
Playwright smoke tests run against Vite; native Tauri launch tests require a
platform WebDriver setup. Do not treat browser-only tests as native desktop
validation.

### Platform prerequisites

- **macOS:** Xcode Command Line Tools, Rust, Node.js, and the WebKit tooling
  required by the installed Tauri version.
- **Linux:** a C compiler, `webkit2gtk` development packages, GTK/WebKit
  dependencies, Rust, and Node.js. Package names vary by distribution.
- **Windows:** Visual Studio Build Tools with the Desktop C++ workload, the
  Windows WebView2 runtime, Rust, and Node.js.

Confirm the local Tauri prerequisites with `npm run tauri info` before opening
an issue about a native build.

## Architecture rules

Keep notebook persistence in `mars-document`, execution in `mars-runtime`,
request handling in `mars-kernel` and `mars-protocol`, rendering in
`mars-renderer`, and UI-only concerns in the desktop app. Never execute
notebook code in the Tauri process.

When changing the `.mars` schema, increment `CURRENT_FORMAT_VERSION`, add a
migration, update `docs/notebook-format.md`, and add a round-trip test.

## Dependency changes

Rust dependencies belong in the relevant crate manifest or workspace manifest.
Frontend dependencies belong in the relevant package manifest. Prefer pnpm
workspace installation for the Node applications, and keep package-local
`package-lock.json` files synchronized only when a package-local npm workflow
is intentionally maintained. Run the complete validation commands after
dependency changes and document network/runtime requirements when a feature
uses external registries or browser assets.

## Documentation changes

Update the relevant reference document when changing:

- CLI syntax or exit behavior (`docs/cli.md`).
- Notebook fields, migrations, or output variants
  (`docs/notebook-format.md`).
- Runtime semantics, cancellation, or performance
  (`docs/runtime.md`).
- Public request/response messages (`docs/protocol.md`).
- Toolchain, workspace, or test commands (`docs/development.md`,
  `CONTRIBUTING.md`).
- Release artifacts or signing (`docs/release.md`).
