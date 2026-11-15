---
title: VS Code Extension
description: Project Mars VS Code extension documentation
order: 10
author: Project Mars Team
tags: [vscode, extension, notebook-editor, export, runtime-selection]
---

# VS Code companion

The Project Mars extension provides a notebook editor for `.mars` files in VS Code,
with Rust and Markdown cell editing, rich output rendering, and CLI-backed
execution and export.

## Features

- Interactive Markdown and code cell editing in the notebook interface.
- Run individual cells or the entire notebook.
- Render execution results, errors, and rich outputs.
- Add, delete, and reorder cells.
- Standard notebook keyboard shortcuts, including `Shift+Enter` to run a cell.
- Persist execution counts and outputs in the `.mars` file.
- Export a notebook to HTML or Markdown through the Project Mars CLI.
- Kernel Variables sidebar for inspecting live runtime bindings.
- Problems panel integration for compiler diagnostics.
- Checkpoint create/restore functionality.
- Template-based notebook creation.
- Cell completions suggesting live kernel variables.

## Install

Install the extension together with the CLI from the repository root:

```bash
./scripts/install.sh --cli --vscode
```

To rebuild and reinstall only the extension while keeping an existing `mars` on
`PATH`:

```bash
./scripts/install.sh --vscode
```

The install script builds the CLI from source, installs `mars` to
`~/.local/bin`, rebuilds the extension from scratch, and installs it into VS
Code. Make sure `~/.local/bin` is on your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Use the notebook editor

1. Open any `.mars` file. It automatically opens in the notebook editor.
2. Click the play button on a cell to run it.
3. Use the **+** button to add Markdown or Rust cells.
4. Reorder or delete cells with the notebook controls.

The extension runs cells through `mars kernel serve --notebook <file>` so the
kernel loads that notebook's declared Cargo dependencies and lockfile. Switching
to a different notebook restarts the kernel.

## Runtime support

The VS Code extension supports Rust, Python, R, Julia, and C++ cells through
the kernel's `Execute.language` field; Rust remains the default when no
language metadata is present. Set
`mars.runtime` to `replay` (default) or `evcxr` (experimental):

```json
{
  "mars.runtime": "replay"
}
```

Evcxr is experimental and requires a Project Mars CLI built with
`--features experimental-evcxr`.

Each code cell's `languageId` (or its `language` metadata) selects the kernel
language, falling back to the notebook's `metadata.language`. Additional
runtimes require the corresponding backend available to the `mars` CLI.

## Export

Run **Project Mars: Export Notebook to HTML** or **Project Mars: Export Notebook to Markdown**
from the Command Palette or the notebook toolbar.

For Markdown export, the output is written next to the notebook. It preserves
Markdown cells, fences Rust code, and converts tables, LaTeX, images, and
offline charts into portable Markdown. The file opens automatically when
`mars.previewAfterExport` is enabled.

## Rich output

The built-in notebook renderer displays `image/svg+xml`, PNG, JPEG, and GIF
output. SVG and HTML output are only rendered while the workspace is trusted;
otherwise VS Code falls back to the `text/plain` alternative. Use **Workspaces:
Manage Workspace Trust** to trust the folder if figures show as text.

## Keyboard shortcuts

| Shortcut           | Action                                                |
| ------------------ | ----------------------------------------------------- |
| `Shift+Enter`      | Run cell and move to the next cell                    |
| `Ctrl/Cmd+Enter`   | Run cell                                              |
| `Alt+Enter`        | Run cell and insert a new cell below                  |
| `Ctrl/Cmd+Shift+N` | Create a new notebook when not in the notebook editor |

## Configuration

```json
{
  "mars.cliPath": "mars",
  "mars.runArguments": [],
  "mars.previewAfterExport": true,
  "mars.executionTimeout": 30000,
  "mars.runtime": "replay"
}
```

### Settings

| Setting                   | Default    | Description                               |
| ------------------------- | ---------- | ----------------------------------------- |
| `mars.cliPath`            | `"mars"`   | Path to the Project Mars CLI executable           |
| `mars.runArguments`       | `[]`       | Additional arguments passed to `mars notebook run` |
| `mars.previewAfterExport` | `true`     | Automatically open the exported file      |
| `mars.executionTimeout`   | `30000`    | Execution timeout in milliseconds         |
| `mars.runtime`            | `"replay"` | Runtime backend: `"replay"` or `"evcxr"`  |

## Troubleshooting

### CLI not found

If VS Code cannot find the `mars` CLI:

1. Verify the CLI is installed: `mars --version`
2. Check that `~/.local/bin` is on your `PATH`
3. Set `mars.cliPath` to the full path to the executable

### Execution fails

If cell execution fails:

1. Check the VS Code output panel for detailed error messages
2. Ensure the notebook has valid Rust syntax
3. Try running the notebook from the CLI: `mars notebook run notebook.mars --report`
4. For dependency-backed notebooks, ensure Cargo is installed

### SVG not rendering

If SVG output appears as text:

1. Trust the workspace: **Workspaces: Manage Workspace Trust**
2. Check that the notebook runtime is using the `replay` backend
3. Verify the output contains `image/svg+xml` MIME type

### Slow execution

The replay runtime recompiles accumulated code for each cell. This is expected behavior. For faster development:

- Use smaller notebooks or split work into multiple notebooks
- Consider the experimental `evcxr` runtime for iterative work (requires `--features experimental-evcxr`)

### Language support

The VS Code extension is currently scoped to Rust execution. For Python or R
notebooks, use the desktop application which supports all three languages.
