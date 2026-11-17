---
title: 'evcxr_jupyter: Rust Kernel for Interactive Computing'
description: 'Explore evcxr_jupyter, the Rust kernel for Jupyter, and how it enables interactive Rust development in notebook environments.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 46
tags: ['kernels', 'evcxr', 'rust', 'jupyter', 'notebook', 'mars']
---

# Why evcxr_jupyter? Rust Kernel for Jupyter

**evcxr_jupyter** is a Rust kernel for Jupyter that enables interactive Rust development in notebook environments. It provides a way to write and execute Rust code in Jupyter notebooks, bringing Rust's performance and safety to the interactive computing paradigm.

## What evcxr_jupyter Offers

### Interactive Rust Execution

- **Live Rust code**: Write and execute Rust code interactively
- **Persistent state**: Variables persist across cells
- **Cargo integration**: Manage dependencies with Cargo.toml
- **REPL-like experience**: Similar to Rust's REPL but in notebook format

### Jupyter Integration

- **Standard Jupyter protocol**: Works with JupyterLab, Jupyter Notebook, VS Code
- **Rich outputs**: Display text, images, HTML, and other MIME types
- **Code completion**: Integration with rust-analyzer for completions
- **Debugging**: Support for debugging Rust code

### Development Features

- **Syntax highlighting**: Rust syntax highlighting in notebooks
- **Error messages**: Detailed Rust compiler error messages
- **Documentation**: Access to Rust documentation and examples
- **Testing**: Write and run Rust tests in notebooks

## evcxr_jupyter vs Project Mars Rust Runtime

| Feature         | evcxr_jupyter  | Project Mars Rust Runtime      |
| --------------- | -------------- | ------------------------------ |
| Platform        | Jupyter kernel | Native Rust runtime            |
| Execution model | REPL-based     | Replay-based compilation       |
| Performance     | Good           | Excellent (native compilation) |
| Dependencies    | Cargo.toml     | Cargo.toml + workspace         |
| Multi-language  | Rust only      | Rust, Python, R, Julia, C++    |
| Document format | .ipynb         | .mars (native format)          |
| Desktop app     | No             | Yes (Tauri-based)              |
| CLI             | Limited        | Full CLI with automation       |

## When to Use evcxr_jupyter

- **Jupyter ecosystem**: Already using Jupyter for other languages
- **Rust experimentation**: Quick Rust prototyping in notebooks
- **Educational purposes**: Teaching Rust with notebook interface
- **Jupyter extensions**: Leverage existing Jupyter extensions

## When Project Mars Might Be Better

- **Performance-critical** Rust code requiring native compilation
- **Multi-language workflows** needing Rust + Python + R + Julia
- **Desktop application** with native performance
- **CLI automation** for reproducible workflows
- **Independent platform** not tied to Jupyter

## Project Mars's Rust Runtime Advantages

Project Mars provides a more sophisticated Rust runtime:

- **Replay-based compilation**: Full Rust compilation for each cell
- **Workspace support**: Cargo workspace integration
- **LSP integration**: rust-analyzer for completions, hover, diagnostics
- **Variable inspector**: View and manipulate Rust variables
- **Multi-language cells**: Mix Rust with other languages
- **Native performance**: No Jupyter overhead, direct execution

## The evcxr_jupyter Experience

Using evcxr_jupyter in Jupyter:

```rust
// Cell 1: Define a function
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Cell 2: Use the function
fibonacci(10)
```

The kernel maintains state between cells, allowing you to build up complex Rust programs interactively.

## Project Mars's Rust Experience

Using Project Mars's Rust runtime:

```rust
// Cell 1: Same function
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Cell 2: Same usage, but with LSP support
fibonacci(10)
```

Project Mars provides the same interactive experience but with additional features like LSP integration, variable inspection, and multi-language support.

## The Future of Rust Notebooks

Both evcxr_jupyter and Project Mars represent different approaches:

- **evcxr_jupyter**: Integrates Rust into the Jupyter ecosystem
- **Project Mars**: Builds a Rust-first notebook platform from the ground up

As Rust continues to grow in data science and systems programming, both approaches will evolve to meet different user needs.

## Get Started

- [Project Mars Rust Runtime](docs/02-platform/runtime-rust.md) - Rust execution documentation
- [Project Mars Desktop App](packages/desktop) - Native application
- [Project Mars CLI](docs/02-platform/cli.md) - Command-line interface
- [Getting Started](docs/01-foundations/getting-started.md) - First notebook tutorial
