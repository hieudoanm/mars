# Mars Notebooks for VS Code

Mars Notebooks adds native `.mars` notebook support to Visual Studio Code. It
uses VS Code's Notebook API for editing and delegates Rust execution, exports,
and kernel lifecycle to the Mars CLI, so the extension does not implement a
second runtime or notebook format.

## Features

- Open `.mars` files in a notebook editor with Rust code and Markdown cells.
- Preserve Mars cell IDs, metadata, execution counts, notebook metadata, Cargo
  dependencies, lockfile data, and rich outputs on round trip.
- Run selected cells or a whole notebook through `mars kernel serve` with a
  persistent per-notebook kernel.
- Restart a notebook kernel and configure CLI path, timeout, runtime, and
  additional run arguments.
- Export notebooks to standalone HTML or Markdown, preview an HTML export, and
  navigate to a cell in text-editor mode.
- Provide a `.mars` language contribution, JSON/embedded Rust/Markdown grammar,
  language configuration, and Mars file icon theme.

The default `replay` runtime is supplied by the Mars CLI. `evcxr` is optional
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
2. Add Markdown or Rust code cells and use the standard notebook run controls.
3. Run **Mars: Restart Kernel** if the execution state should be discarded.
4. Run **Mars: Export Notebook to HTML** or **Mars: Export Notebook to
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

## Commands and shortcuts

| Command                             | Purpose                                |
| ----------------------------------- | -------------------------------------- |
| `Mars: Create New Notebook`         | Create a new Mars notebook             |
| `Mars: Run Notebook`                | Execute all executable cells           |
| `Mars: Restart Kernel`              | Reset persistent execution state       |
| `Mars: Export Notebook to HTML`     | Export a standalone HTML document      |
| `Mars: Export Notebook to Markdown` | Export portable Markdown               |
| `Mars: Preview Notebook`            | Export and open the HTML preview       |
| `Mars: Go to Cell`                  | Navigate to a cell in text-editor mode |

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
