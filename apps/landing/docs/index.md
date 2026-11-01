---
title: Overview
description: Mars index documentation
order: 0
author: Mars Team
tags: [overview, getting-started]
---

# Mars documentation

Mars is a Rust-powered interactive notebook for exploring ideas, executing
code, and discovering possibilities. It combines executable cells, Markdown,
mathematics, tables, charts, and rich output in a native `.mars` document.

## Start here

- [Getting started](/docs/01-getting-started/) — install Mars and run your first notebook.
- [Notebook format](/docs/03-notebook-format/) — understand the `.mars` document model.
- [Execution runtime](/docs/04-runtime/) — learn how Rust cells run and persist state.
- [CLI reference](/docs/02-cli/) — create, run, inspect, and export notebooks.
- [VS Code companion](/docs/05-vscode/) — use Mars from the VS Code notebook editor.

## Design principles

- **Rust first.** Execution, conversion, and rendering share one Rust core.
- **Clear boundaries.** The document model, kernel, runtime, renderer, and CLI
  are separate components.
- **Isolation by default.** Notebook code does not run inside the desktop
  process.
- **Open documents.** `.mars` is versioned JSON, and Jupyter notebooks can be
  imported and exported.
- **Portable output.** Notebooks can be exported to HTML, Markdown, and
  `.ipynb`.

## Repository

The source code, examples, and full project documentation live at
[hieudoanm/mars](https://github.com/hieudoanm/mars).
