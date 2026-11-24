---
title: Overview
description: Project Mars documentation index
author: Project Mars Team
order: 0
---

# Project Mars documentation

Project Mars is a Rust-powered interactive computing platform for exploring ideas,
executing code, and discovering possibilities — **Mars Notebook** for
multi-language notebooks, plus **Mars Lab**, a machine learning studio that turns
CSV datasets into trained ONNX models. Mars Notebook combines executable cells,
Markdown, mathematics, tables, charts, and rich output in a native `.mars`
document.

## Documentation chapters

### Foundations

- [Getting started](/docs/getting-started/) — install Project Mars and run your first notebook.
- [Notebook format](/docs/notebook-format/) — understand the `.mars` document model.

### Platform

- [Runtime architecture](/docs/runtime/) — how runtimes provide persistent, process-isolated sessions.
- [CLI reference](/docs/cli/) — create, run, inspect, and export notebooks.
- [Rust runtime](/docs/runtime-rust/) — Rust execution (replay and Evcxr backends).
- [Python runtime](/docs/runtime-python/) — Python execution and data science.
- [R runtime](/docs/runtime-r/) — R execution and data science.
- [Julia runtime](/docs/runtime-julia/) — Julia execution and data science.
- [C++ runtime](/docs/runtime-cpp/) — C++ execution with Conan/vcpkg dependencies.
- [SQL runtime](/docs/runtime-sql/) — in-process SQLite cells and CSV-backed tables.
- [MCP integration](/docs/mcp/) — Model Context Protocol for AI assistants and automation.

### Installation guides

- [CLI on macOS (Apple Silicon)](/docs/how-to-install-cli-macos-silicon/) — installation guide.
- [CLI on macOS (Intel)](/docs/how-to-install-cli-macos-intel/) — installation guide.
- [CLI on Linux](/docs/how-to-install-cli-linux/) — installation guide.
- [CLI on Windows](/docs/how-to-install-cli-windows/) — installation guide.
- [Desktop on macOS (Apple Silicon)](/docs/how-to-install-desktop-macos-silicon/) — installation guide.
- [Desktop on macOS (Intel)](/docs/how-to-install-desktop-macos-intel/) — installation guide.
- [Desktop on Linux](/docs/how-to-install-desktop-linux/) — installation guide.
- [Desktop on Windows](/docs/how-to-install-desktop-windows/) — installation guide.
- [VS Code extension](/docs/how-to-install-vscode/) — installation guide.
- [Zed extension](/docs/how-to-install-zed/) — installation guide.
- [JetBrains plugin](/docs/how-to-install-jetbrains/) — installation guide.

### Mars Lab

- [Lab overview](/docs/lab-overview/) — the machine learning studio: features and architecture.
- [Lab CLI](/docs/lab-cli/) — command-line training, datasets, jobs, and ONNX export.
- [Lab algorithms](/docs/lab-algorithms/) — complete algorithm catalogue and hyperparameters.
- [Lab workflows](/docs/lab-workflows/) — common training workflows and examples.

### Extensions

- [VS Code extension](/docs/extension-vscode/) — use Project Mars from the VS Code notebook editor.
- [Zed extension](/docs/extension-zed/) — Zed grammar support and usage.
- [JetBrains extension](/docs/extension-jetbrains/) — JetBrains plugin status and future plans.

## Design principles

- **Rust first.** Execution, conversion, and rendering share one Rust core.
- **Multi-language.** Rust, Python, R, Julia, C++, and SQL runtimes with a protocol boundary for future languages.
- **Clear boundaries.** The document model, kernel, runtime, renderer, and CLI are separate components.
- **Isolation by default.** Notebook code does not run inside the desktop process.
- **Open documents.** `.mars` is versioned JSON, and Jupyter notebooks can be imported and exported.
- **Portable output.** Notebooks can be exported to HTML, Markdown, and `.ipynb`.
- **LSP-powered.** rust-analyzer integration provides IDE-quality editing in the notebook.

## Repository

The source code, examples, and full project documentation live at
[hieudoanm/mars](https://github.com/hieudoanm/mars).
