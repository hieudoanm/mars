---
title: Overview
description: Project Mars index documentation
order: 0
author: Project Mars Team
tags: [overview, getting-started]
---

# Project Mars documentation

Project Mars is a Rust-powered interactive notebook for exploring ideas, executing
code, and discovering possibilities. It combines executable cells, Markdown,
mathematics, tables, charts, and rich output in a native `.mars` document.

## Current capabilities

Project Mars is a mature, feature-rich notebook platform with:

- **Multi-language support**: Rust (default), Python, and R runtimes
- **Rich desktop editor**: Tauri-based with CodeMirror 6, LSP integration, and full notebook editing
- **Isolated execution**: Replay-based Rust runtime with timeout/cancellation and Cargo dependency support
- **Developer tools**: rust-analyzer LSP integration (diagnostics, hover, completion, signature help, go-to-definition, quick fixes, inlay hints, semantic tokens)
- **Workspace features**: Multi-notebook tabs, file explorer, checkpoints with diff view, variable inspector, problems panel
- **Rich outputs**: HTML, LaTeX, images, tables, charts, and MIME bundles with interactive rendering
- **Jupyter compatibility**: Import and export `.ipynb` notebooks with metadata and output preservation
- **CLI**: Full command-line interface for headless execution, import/export, and kernel serving
- **Editor extensions**: VS Code companion extension and Zed grammar support

## Start here

- [Getting started](/docs/01-getting-started/) — install Project Mars and run your first notebook.
- [CLI reference](/docs/02-cli/) — create, run, inspect, and export notebooks.
- [Notebook format](/docs/03-notebook-format/) — understand the `.mars` document model.
- [Rust runtime](/docs/04-runtime-rust/) — Rust execution (replay and Evcxr backends).
- [Python runtime](/docs/05-runtime-python/) — Python execution and data science.
- [R runtime](/docs/06-runtime-r/) — R execution and data science.
- [Julia runtime](/docs/07-runtime-julia/) — Julia execution and data science.
- [C++ runtime](/docs/08-runtime-cpp/) — C++ execution with Conan/vcpkg dependencies.
- [MCP integration](/docs/09-mcp/) — Model Context Protocol for AI assistants and automation.
- [VS Code extension](/docs/10-extension-vscode/) — use Project Mars from the VS Code notebook editor.
- [Zed extension](/docs/11-extension-zed/) — Zed grammar support and usage.
- [JetBrains extension](/docs/12-extension-jetbrains/) — JetBrains plugin status and future plans.
- [Mars Lab](/docs/13-lab-overview/) — Machine learning training studio with ONNX export.
- [Lab CLI](/docs/14-lab-cli/) — Command-line interface for Mars Lab training.
- [Lab Algorithms](/docs/15-lab-algorithms/) — Complete algorithm catalogue and hyperparameters.
- [Lab Workflows](/docs/16-lab-workflows/) — Common training workflows and examples.

## Design principles

- **Rust first.** Execution, conversion, and rendering share one Rust core.
- **Multi-language.** Rust, Python, and R runtimes with a protocol boundary for future languages.
- **Clear boundaries.** The document model, kernel, runtime, renderer, and CLI are separate components.
- **Isolation by default.** Notebook code does not run inside the desktop process.
- **Open documents.** `.mars` is versioned JSON, and Jupyter notebooks can be imported and exported.
- **Portable output.** Notebooks can be exported to HTML, Markdown, and `.ipynb`.
- **LSP-powered.** rust-analyzer integration provides IDE-quality editing in the notebook.

## Repository

The source code, examples, and full project documentation live at
[hieudoanm/mars](https://github.com/hieudoanm/mars).
