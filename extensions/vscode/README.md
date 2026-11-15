# Mars Notebooks for VS Code

Mars Notebooks adds native `.mars` notebook support to Visual Studio Code. It
uses VS Code's Notebook API for editing and delegates multi-language execution,
exports, and kernel lifecycle to the Project Mars CLI, so the extension does not
implement a second runtime or notebook format.

## Features

- Open `.mars` files in a notebook editor with code and Markdown cells.
- Execute Rust, Python, R, Julia, and C++ cells through `mars kernel serve`:
  each cell's `languageId` (or its `language` metadata) is sent as
  `Execute.language`, falling back to the notebook's `metadata.language`
  (default `rust`).
- Preserve Project Mars cell IDs, metadata, execution counts, notebook metadata, Cargo
  dependencies, lockfile data, and rich outputs on round trip.
- Run selected cells or a whole notebook through `mars kernel serve` with a
  persistent per-notebook kernel.
- Interrupt a running cell with **Project Mars: Interrupt Execution** or send a line to
  a cell waiting on stdin with **Project Mars: Send Input to Kernel**.
- Inspect live session variables in the **Kernel Variables** sidebar view.
- Compile-check cells without executing them via **Project Mars: Check Cell** /
  **Project Mars: Check Notebook**, with results published to the Problems panel.
- Create and restore notebook checkpoints stored in a `.checkpoints`
  directory next to the notebook (same layout as the desktop app).
- Create notebooks from built-in templates (blank, getting started, data
  analysis).
- Restart a notebook kernel and configure CLI path, timeout, runtime, and
  additional run arguments.
- Export notebooks to standalone HTML or Markdown, preview an HTML export, and
  navigate to a cell in text-editor mode.
- Provide a `.mars` language contribution, JSON/embedded Rust/Markdown grammar,
  language configuration, and Project Mars file icon theme.
- Suggest live kernel session variables and Rust keywords as completions inside
  code cells (rust-analyzer does not index notebook cell documents).

The default `replay` runtime is supplied by the Project Mars CLI. `evcxr` is optional
and requires a CLI built with the `experimental-evcxr` feature.

## Requirements

- VS Code `1.145.2` or later.
- Node.js and Corepack-managed pnpm for development.
- The `mars` CLI on `PATH`, or an explicit `mars.cliPath`, to execute or export
  notebooks.

From the repository root:

```bash
corepack enable
pnpm install
cargo install --path crates/mars-cli
```

## Using the extension

1. Open a `.mars` file in VS Code; it opens with the **Mars Notebook** editor.
2. Add Markdown or code cells and use the standard notebook run controls.
3. Run **Project Mars: Restart Kernel** if the execution state should be discarded.
4. Run **Project Mars: Export Notebook to HTML** or **Project Mars: Export Notebook to
   Markdown** from the Command Palette to create a portable export beside the
   notebook.

The extension starts the CLI with `mars kernel serve --notebook <file>`, so
declared notebook dependencies and embedded Cargo lockfiles are available to
the kernel.

### Settings

```json
{
  "mars.cliPath": "mars",
  "mars.executionTimeout": 30000,
  "mars.runArguments": [],
  "mars.previewAfterExport": true,
  "mars.runtime": "replay"
}
```

Set `mars.runtime` to `evcxr` only when using an appropriately feature-enabled
CLI. For images and rich output, trust the workspace: VS Code intentionally
falls back to text for potentially unsafe HTML or SVG in untrusted workspaces.
This is a display-layer fallback — the underlying MIME data is preserved when
the notebook is saved. Per-notebook trust granularity like the desktop app is
not exposed by the VS Code notebook API.

Cell editing uses VS Code's standard notebook commands and shortcuts
(`Shift+Enter`, `Ctrl/Cmd+Enter`, `Alt+Enter`, insert/delete/move via the cell
toolbar). Every `mars.*` command can be rebound in `keybindings.json`; Jupyter-
style single-letter command mode is not feasible because cell editors hold
text focus, so letter keys would conflict with typing.

## Commands and shortcuts

| Command                             | Purpose                                |
| ----------------------------------- | -------------------------------------- |
| `Project Mars: Create New Notebook`         | Create a notebook from a template      |
| `Project Mars: Run Notebook`                | Execute all executable cells           |
| `Project Mars: Restart Kernel`              | Reset persistent execution state       |
| `Project Mars: Interrupt Execution`         | Cancel the in-flight cell execution    |
| `Project Mars: Send Input to Kernel`        | Write a line to a running cell's stdin |
| `Project Mars: Check Cell`                  | Compile-check the selected cells       |
| `Project Mars: Check Notebook`              | Compile-check all code cells           |
| `Project Mars: Create Checkpoint`           | Snapshot the notebook file             |
| `Project Mars: Restore Checkpoint`          | Restore a previous snapshot            |
| `Project Mars: Export Notebook to HTML`     | Export a standalone HTML document      |
| `Project Mars: Export Notebook to Markdown` | Export portable Markdown               |
| `Project Mars: Preview Notebook`            | Export and open the HTML preview       |
| `Project Mars: Go to Cell`                  | Navigate to a cell in text-editor mode |

`Ctrl/Cmd+Shift+N` creates a notebook while a notebook editor is not focused.
Standard VS Code notebook shortcuts such as `Shift+Enter`, `Ctrl/Cmd+Enter`,
and `Alt+Enter` run cells.

## Development and packaging

```bash
pnpm --filter mars-vscode check
pnpm --filter mars-vscode lint
pnpm --filter mars-vscode test
pnpm --filter mars-vscode build
pnpm --filter mars-vscode package
```

Compilation writes the extension to `out/`; packaging writes
`vsix/mars-vscode.vsix`. These are generated artifacts and are not committed.
See [AGENTS.md](AGENTS.md) for notebook conversion and extension-host rules.
