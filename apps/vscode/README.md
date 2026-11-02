# Mars Notebooks for VS Code

![Mars](assets/mars-og.png)

This companion extension provides a full-featured notebook editor for `.mars` files using VSCode's Notebook API, along with syntax recognition, Rust/Markdown embedded highlighting, CLI-backed HTML export, and headless notebook execution.

## Features

### Notebook Editor
- **Interactive cell editing**: Edit Markdown and code cells in a true notebook interface
- **Cell execution**: Run individual cells or entire notebooks
- **Output rendering**: View execution results, errors, and rich outputs
- **Cell management**: Add, delete, and reorder cells
- **Keyboard shortcuts**: Standard notebook shortcuts (Shift+Enter to run, etc.)
- **Persistent state**: Execution counts and outputs are preserved in the `.mars` file

### Legacy Commands
The extension also provides CLI-backed commands for compatibility:
- **Mars: Export Notebook to HTML**: Export to HTML via Mars CLI
- **Mars: Preview Notebook**: Export and preview HTML
- **Mars: Run Notebook**: Execute entire notebook via Mars CLI
- **Mars: Go to Cell**: Navigate to a specific cell in text editor mode

## Development

The extension is the `@mars/vscode` package in the root pnpm workspace. The
package-local npm commands remain supported for contributors who work only on
the extension.

```bash
pnpm install
pnpm run vscode:compile
pnpm run vscode:package
```

Equivalent package-local commands are:

```bash
cd apps/vscode
npm install
npm run compile
npm run package
```

## Installation

To install the extension together with the Mars CLI:

```bash
# From the repository root
./scripts/install.sh --cli --vscode
```

To rebuild and reinstall only the extension (keeping an existing `mars` on
`PATH`):

```bash
./scripts/install.sh --vscode
```

The script will:
1. Build the Mars CLI from source (`--cli` or `--all` only)
2. Install the Mars CLI to `~/.local/bin/mars`
3. Build the VSCode extension from scratch (`out/` and `vsix/` are removed first)
4. Install the extension in VSCode

Make sure to add `~/.local/bin` to your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Usage

### Using the Notebook Editor
1. Open any `.mars` file in VSCode
2. The file will automatically open in the notebook editor
3. Click the play button on a cell to execute it
4. Use the "+" button to add new cells
5. Cells can be code (Rust) or Markdown

### Runtime Selection
Set `mars.runtime` to `evcxr` to run cells through the Evcxr backend. The
default `replay` backend re-executes prior cells and now also converts
`EVCXR_BEGIN_CONTENT` display markers (used by `plotters`) into
`image/svg+xml` MIME bundles. Evcxr is experimental and requires a Mars CLI
built with `--features experimental-evcxr`.

The extension passes the open notebook to `mars kernel serve --notebook <file>`
so the kernel loads that notebook's declared Cargo dependencies and lockfile,
then restarts the kernel when you switch to a different notebook.

### Images and rich output

The built-in VS Code notebook renderer displays `image/svg+xml`, PNG, JPEG, and
GIF output. SVG and HTML output are only rendered while the workspace is
trusted; otherwise VS Code falls back to the `text/plain` alternative. Use
**Workspaces: Manage Workspace Trust** to trust the folder if figures show as
text.

### Keyboard Shortcuts
- `Shift+Enter`: Run cell and move to next
- `Ctrl/Cmd+Enter`: Run cell
- `Alt+Enter`: Run cell and insert new cell below
- `Ctrl/Cmd+Shift+N`: Create new notebook (when not in notebook editor)

### Configuration
Configure the Mars CLI path and execution arguments:

```json
{
  "mars.cliPath": "mars",
  "mars.runArguments": [],
  "mars.previewAfterExport": true,
  "mars.executionTimeout": 30000
}
```

### Cell Execution

The extension uses the Mars CLI's `kernel serve` command to execute Rust code cells with persistent state. The Mars CLI must be installed and available on your PATH. Use the installation script at the repository root (`./scripts/install.sh`) to build and install both the CLI and extension.

The extension package includes the Mars project mark in
[`assets/mars-logo.png`](assets/mars-logo.png) and the notebook-compatible
file mark in [`assets/mars-file-logo.png`](assets/mars-file-logo.png).
The package also includes the Open Graph artwork at
[`assets/mars-og.png`](assets/mars-og.png) for project documentation.
