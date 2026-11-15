---
title: JetBrains Extension
description: Project Mars JetBrains extension documentation
order: 18
author: Project Mars Team
tags: [jetbrains, extension, kotlin, kernel]
---

# JetBrains Extension

Project Mars ships a Kotlin plugin for JetBrains IDEs at `extensions/jetbrains`, built
with the IntelliJ Platform Gradle plugin. It provides:

- `.mars` file type with JSON-aware syntax highlighting, folding, and
  structure view
- Cell execution via `mars kernel serve` — **Project Mars → Execute Cell at Caret**
  runs the code cell under the caret over the same JSON-lines kernel protocol
  used by the VS Code extension, honoring per-cell `language` metadata
  (`Execute.language`)
- Cell navigation (**Next Cell** / **Previous Cell**)
- A **Project Mars Kernel** tool window showing kernel status, live session variables,
  execution output, and stderr, with Refresh and Interrupt buttons
- Notebook actions: Execute All Cells, Check Notebook (compile diagnostics),
  Send Input to Kernel, Create/Restore Checkpoint, Run Notebook, Export to
  HTML / Markdown / `.ipynb`, Inspect, Restart Kernel, Interrupt Execution
- A configurable `mars` CLI path in **Settings → Tools → Project Mars**

## Install

```bash
cd extensions/jetbrains
./gradlew buildPlugin
```

Then install the resulting zip via **Settings → Plugins → ⚙ → Install Plugin
from Disk…**, or run `./gradlew runIde` for a sandbox IDE.
`scripts/install.sh --jetbrains` builds and installs the plugin into detected
JetBrains IDEs automatically.

## Current limitations

JetBrains IDEs have no notebook-cell editor API (unlike VS Code's Notebook
API). Cells are edited as raw JSON inside the `.mars` document; there is no
per-cell gutter, output pane, or markdown rendering. Execution results appear
in the Project Mars Kernel tool window and are **not** written back into the file's
`outputs` arrays — use `mars notebook run` or the desktop app for full output
persistence.

## Supported IDEs

The plugin targets the 2024.3 IntelliJ Platform and works with:

- IntelliJ IDEA Community and Ultimate
- PyCharm
- CLion
- RustRover
- Other JetBrains IDEs on the 2024.3 platform or newer

## Future plans

Full JetBrains notebook integration is planned but not yet implemented. The Project Mars
roadmap includes a JetBrains plugin that will provide:

- Interactive Markdown and code cell editing in the notebook interface
- Run individual cells or the entire notebook
- Render execution results, errors, and rich outputs
- Add, delete, and reorder cells
- Standard notebook keyboard shortcuts
- Persist execution counts and outputs in the `.mars` file
- Export a notebook to HTML or Markdown through the Project Mars CLI

In the meantime, use the VS Code extension or the desktop application for
the full notebook experience. See the [VS Code extension](/docs/extension-vscode/)
and [Getting started](/docs/getting-started/) documentation for more
information.
