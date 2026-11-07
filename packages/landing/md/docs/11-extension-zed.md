---
title: Zed Extension
description: Mars Zed extension documentation
order: 11
author: Mars Team
tags: [zed, extension, grammar, mcp]
---

# Zed companion

Zed integration for Mars lives at `extensions/zed` and provides:

- `.mars` file association and file icon
- Tree-sitter syntax highlighting for the Mars notebook format, including
  embedded Rust and Markdown conventions
- An MCP **context server** that launches `mars mcp serve`, exposing Mars
  notebook, cell, and kernel tools to Zed's Agent Panel (requires the `mars`
  CLI on `PATH`)

## Current limitations

The Zed extension does not provide:

- Notebook execution — Zed has no notebook UI API, and `mars kernel serve`
  speaks the Mars kernel protocol rather than LSP
- Cell editing UI or rich output rendering
- Kernel commands

## Workaround for execution

To execute Mars notebooks from Zed, use the CLI:

```bash
# Run the notebook
mars run notebook.mars --report

# Open in the desktop app for interactive editing
mars open notebook.mars
```

Or configure a Zed task to run `mars` on the current file.

## Future plans

Full Zed notebook integration is planned but not yet implemented. The Mars
roadmap includes a Zed extension that will provide:

- Interactive Markdown and code cell editing in the Zed notebook interface
- Run individual cells or the entire notebook
- Render execution results, errors, and rich outputs
- Add, delete, and reorder cells
- Standard notebook keyboard shortcuts
- Persist execution counts and outputs in the `.mars` file
- Export a notebook to HTML or Markdown through the Mars CLI

In the meantime, use the VS Code extension or the desktop application for
full notebook functionality. See the [VS Code extension](/docs/10-extension-vscode/) and
[Getting started](/docs/01-getting-started/) documentation for more information.
