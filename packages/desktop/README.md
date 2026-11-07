# Mars desktop

The Mars desktop application is the primary notebook experience for Mars. It
combines a React/CodeMirror interface with a Tauri 2 host, keeping notebook
persistence and isolated Rust execution in the Rust workspace rather than in
the browser process.

## What it provides

- Create, open, save, and rename native `.mars` notebooks.
- Add, edit, reorder, duplicate, delete, and convert Markdown, Rust code, and
  raw cells.
- Execute a selected cell, all cells, or a range of cells; reset or interrupt
  the kernel; optionally set an execution timeout.
- Preserve cell IDs, metadata, execution counts, rich outputs, Cargo
  dependencies, and embedded lockfile data when saving.
- Render Markdown, KaTeX math, sanitized HTML, tables, images, MIME bundles,
  and Vega/Vega-Lite chart specifications.
- Provide CodeMirror Rust/Markdown editing, diagnostics, completion,
  bracket-matching, folding, keyboard shortcuts, undo/redo, autosave, and an
  unsaved-change confirmation flow.

The default `replay` runtime recompiles accumulated successful code in a
separate process. This preserves bindings between cells without running
notebook code inside the desktop application process. Earlier side effects are
replayed; see the root [runtime documentation](../../docs/06-runtime.md) for
the trade-offs and experimental Evcxr option.

## Requirements

- Current stable Rust toolchain and the platform dependencies required by
  [Tauri 2](https://v2.tauri.app/start/prerequisites/).
- Node.js and Corepack-managed pnpm. The workspace pins pnpm in the root
  `package.json`.

## Development

Install the workspace dependencies once from the repository root:

```bash
corepack enable
pnpm install
```

Launch the native application:

```bash
pnpm --filter @mars/desktop tauri dev
```

For browser-only UI work, run Vite instead:

```bash
pnpm --filter @mars/desktop dev
```

The browser mode cannot perform native Tauri file or kernel operations unless
those APIs are mocked. Native development is the correct path for testing
open/save and execution end to end.

## Validation

Run targeted frontend checks:

```bash
pnpm --filter @mars/desktop check
pnpm --filter @mars/desktop lint
pnpm --filter @mars/desktop test
pnpm --filter @mars/desktop build
```

Run the browser smoke suite when UI flows change:

```bash
pnpm --filter @mars/desktop test:e2e
```

Validate the host from the repository root:

```bash
cargo fmt --all --check
cargo clippy -p mars-tauri --all-targets -- -D warnings
cargo test -p mars-tauri
```

Create a platform bundle with:

```bash
pnpm --filter @mars/desktop tauri build
```

## Project layout

```text
packages/desktop/
├── src/              React UI, hooks, typed frontend services, and unit tests
├── src-tauri/        Tauri host and typed commands backed by Mars crates
├── public/           Packaged web assets and metadata
├── tests/            Playwright browser smoke tests
└── playwright.config.ts
```

The frontend accesses native operations only through `src/lib/kernelService.ts`.
The command implementations in `src-tauri/src/lib.rs` use `mars-document` for
validated atomic persistence, `mars-kernel` for execution, and `mars-renderer`
for Markdown export. Keep that boundary intact when adding features.

## Keyboard shortcuts

| Shortcut         | Action                                          |
| ---------------- | ----------------------------------------------- |
| `Shift+Enter`    | Run the selected cell and move to the next cell |
| `Ctrl/Cmd+Enter` | Run the selected cell                           |
| `Alt+Enter`      | Run the selected cell and insert one below      |
| `A` / `B`        | Insert a cell above / below in command mode     |
| `D`, `D`         | Delete the selected cell in command mode        |
| `M` / `Y`        | Convert the selected cell to Markdown / code    |
| `X` / `C`        | Cut / copy the selected cell (command mode)     |
| `V` / `Shift+V`  | Paste the cell clipboard below / above          |
| `I`, `I`         | Interrupt the running kernel (command mode)     |
| `O`              | Toggle the selected cell's outputs              |
| `Z`              | Undo the last notebook change (command mode)    |
| `Up` / `Down`    | Move the selection between cells (command mode) |
| `1`–`6`          | Set a Markdown cell's heading level             |
| `H` or `?`       | Open the keyboard shortcut reference            |
| `Shift+L`        | Toggle line numbers (editor preference)         |
| `S`              | Save the notebook                               |
| `Escape`         | Enter command mode                              |

See [AGENTS.md](AGENTS.md) for implementation boundaries and contributor rules.
