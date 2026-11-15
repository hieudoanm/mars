
# Zed companion

Zed integration for Project Mars lives at `extensions/zed` and provides:

- `.mars` file association and file icon
- Tree-sitter syntax highlighting for the Project Mars notebook format, including
  embedded Rust and Markdown conventions
- An MCP **context server** that launches `mars mcp serve`, exposing Project Mars
  notebook, cell, and kernel tools to Zed's Agent Panel (requires the `mars`
  CLI on `PATH`)

## Current limitations

The Zed extension does not provide:

- Notebook execution — Zed has no notebook UI API, and `mars kernel serve`
  speaks the Project Mars kernel protocol rather than LSP
- Cell editing UI or rich output rendering
- Kernel commands

## Workaround for execution

To execute Project Mars notebooks from Zed, use the CLI:

```bash
# Run the notebook
mars notebook run notebook.mars --report

# Open in the desktop app for interactive editing
mars notebook open notebook.mars
```

Or configure a Zed task to run `mars` on the current file.

## Future plans

Full Zed notebook integration is planned but not yet implemented. The Project Mars
roadmap includes a Zed extension that will provide:

- Interactive Markdown and code cell editing in the Zed notebook interface
- Run individual cells or the entire notebook
- Render execution results, errors, and rich outputs
- Add, delete, and reorder cells
- Standard notebook keyboard shortcuts
- Persist execution counts and outputs in the `.mars` file
- Export a notebook to HTML or Markdown through the Project Mars CLI

In the meantime, use the VS Code extension or the desktop application for
full notebook functionality. See the [VS Code extension](./extension-vscode.md) and
[Getting started](../01-foundations/getting-started.md) documentation for more information.
