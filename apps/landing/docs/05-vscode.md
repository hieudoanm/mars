---
title: VSCode
description: Mars vscode documentation
order: 5
author: Mars Team
tags: [vscode, extension, notebook-editor, export, runtime-selection]
---

# VS Code companion

The Mars extension provides a notebook editor for `.mars` files in VS Code,
with Rust and Markdown cell editing, rich output rendering, and CLI-backed
execution and export.

## Features

- Interactive Markdown and code cell editing in the notebook interface.
- Run individual cells or the entire notebook.
- Render execution results, errors, and rich outputs.
- Add, delete, and reorder cells.
- Standard notebook keyboard shortcuts, including `Shift+Enter` to run a cell.
- Persist execution counts and outputs in the `.mars` file.
- Export a notebook to HTML or Markdown through the Mars CLI.

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

## Export

Run **Mars: Export Notebook to HTML** or **Mars: Export Notebook to Markdown**
from the Command Palette or the notebook toolbar.

For Markdown export, the output is written next to the notebook. It preserves
Markdown cells, fences Rust code, and converts tables, LaTeX, images, and
offline charts into portable Markdown. The file opens automatically when
`mars.previewAfterExport` is enabled.

## Runtime selection

Set `mars.runtime` to `evcxr` to run cells through the Evcxr backend. The
default `replay` backend re-executes prior cells and converts
`EVCXR_BEGIN_CONTENT` display markers, used by `plotters`, into
`image/svg+xml` MIME bundles.

Evcxr is experimental and requires a Mars CLI built with
`--features experimental-evcxr`.

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
  "mars.executionTimeout": 30000
}
```
