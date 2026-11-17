---
title: 'Jupyter Kernels: Language Support Beyond Python'
description: 'Explore the unique strengths and trade-offs of evcxr_jupyter, IJulia, xeus-cling, IRkernel, and matlab_kernel as language-specific kernels for Jupyter notebooks.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 45
tags: ['kernels', 'jupyter', 'rust', 'julia', 'cpp', 'r', 'matlab', 'comparison', 'mars']
---

# These Jupyter Kernels

Jupyter kernels enable interactive computing in multiple languages within the same notebook interface. This post explores the major language-specific kernels and how they compare to Project Mars's native multi-language approach.

## Kernel Overview

| Kernel        | Language       | Execution Model     | Key Strengths                          | Project Mars Differentiator             |
| ------------- | -------------- | ------------------- | -------------------------------------- | --------------------------------------- |
| evcxr_jupyter | Rust           | REPL-based          | Rust performance, Cargo integration    | Native compilation, multi-language      |
| IJulia        | Julia          | REPL-based          | Julia performance, multiple dispatch   | MarsKernel module, MIME bundles         |
| xeus-cling    | C++            | Interpreter (Cling) | C++ performance, modern C++            | Full compilation, dependency management |
| IRkernel      | R              | REPL-based          | Statistical computing, tidyverse       | Plot capture, warnings streaming        |
| matlab_kernel | MATLAB         | REPL-based          | Matrix operations, toolboxes, plotting | Multi-language, desktop app             |
| Project Mars  | Multi-language | Native runtimes     | Rust-first, multi-language, CLI        | Independent platform, desktop app       |

## Why evcxr_jupyter?

evcxr_jupyter brings Rust's performance and safety to Jupyter:

- **Rust integration**: Write Rust code in Jupyter notebooks
- **Cargo support**: Manage dependencies with Cargo.toml
- **REPL experience**: Interactive Rust development
- **Jupyter ecosystem**: Leverage existing Jupyter extensions

**When evcxr_jupyter excels**: Rust experimentation, Jupyter ecosystem integration

**Project Mars's alternative**: Native Rust compilation, multi-language support, desktop application

## Why IJulia?

IJulia enables Julia's high-performance computing in Jupyter:

- **Julia integration**: Write Julia code in Jupyter notebooks
- **Multiple dispatch**: Leverage Julia's unique paradigm
- **Scientific computing**: First-class support for numerical computing
- **Parallel computing**: Built-in parallel and distributed support

**When IJulia excels**: Julia scientific computing, Jupyter ecosystem integration

**Project Mars's alternative**: MarsKernel module, MIME bundles, multi-language support

## Why xeus-cling?

xeus-cling brings C++'s performance to Jupyter:

- **C++ integration**: Write C++ code in Jupyter notebooks
- **Modern C++**: Support for C++11 through C++20
- **Cling interpreter**: Interactive C++ development
- **Performance**: Near-native C++ performance

**When xeus-cling excels**: C++ experimentation, Jupyter ecosystem integration

**Project Mars's alternative**: Full compilation, Conan/vcpkg dependencies, multi-language support

## Why IRkernel?

IRkernel enables R's statistical computing in Jupyter:

- **R integration**: Write R code in Jupyter notebooks
- **Statistical computing**: First-class support for statistical analysis
- **Tidyverse**: Modern data manipulation and visualization
- **Reproducible research**: Support for R Markdown workflows

**When IRkernel excels**: R statistical computing, Jupyter ecosystem integration

**Project Mars's alternative**: Plot capture, warnings streaming, multi-language support

## Why matlab_kernel?

matlab_kernel brings MATLAB's computational capabilities to Jupyter:

- **MATLAB integration**: Write MATLAB code in Jupyter notebooks
- **Matrix operations**: First-class support for matrix computations
- **Toolboxes**: Access to specialized MATLAB toolboxes
- **Plotting**: Powerful plotting and visualization capabilities

**When matlab_kernel excels**: MATLAB engineering, Jupyter ecosystem integration

**Project Mars's alternative**: Multi-language support, desktop application, CLI automation

## The Kernel Landscape

The Jupyter kernel ecosystem has evolved to support multiple languages:

### Language-Specific Kernels

- **evcxr_jupyter**: Rust
- **IJulia**: Julia
- **xeus-cling**: C++
- **IRkernel**: R
- **matlab_kernel**: MATLAB
- **Python**: Built-in Jupyter kernel

### Multi-Language Platforms

- **Project Mars**: Rust, Python, R, Julia, C++ in one notebook
- **JupyterLab**: Multiple kernels in one interface
- **VS Code**: Jupyter extension with multiple kernels

### Platform-Specific

- **Project Mars**: Independent platform, not Jupyter-based
- **Jupyter**: Kernel-based architecture
- **Observable**: JavaScript-only reactive notebooks

## Project Mars's Unique Approach

Project Mars takes a different approach to multi-language notebooks:

### Native Runtimes

- **Rust**: Replay-based compilation with Cargo
- **Python**: Persistent subprocess with matplotlib capture
- **R**: Rscript subprocess with plot capture
- **Julia**: Julia subprocess with MarsKernel module
- **C++**: Replay-based compilation with Conan/vcpkg

### Unified Experience

- **Single document format**: `.mars` files for all languages
- **Consistent UI**: Same editor and execution model across languages
- **Variable persistence**: Each language maintains its own state
- **Rich outputs**: Unified output types across all languages

### Platform Features

- **Desktop application**: Native Tauri-based app
- **CLI automation**: Headless execution for reproducibility
- **MCP integration**: AI assistant support
- **Open-source**: Community-driven development

## When to Choose Each Approach

### Choose Jupyter Kernels if:

- Already using Jupyter ecosystem
- Need specific language kernel
- Want to leverage Jupyter extensions
- Prefer kernel-based architecture

### Choose Project Mars if:

- Want multi-language notebooks in one platform
- Need Rust-first performance and type safety
- Require desktop application with offline capability
- Want CLI automation for reproducible workflows
- Prefer independent platform not tied to Jupyter

## The Future of Multi-Language Notebooks

The notebook ecosystem is evolving toward multi-language support:

- **Jupyter**: Expanding kernel ecosystem
- **Project Mars**: Native multi-language platform
- **Observable**: JavaScript-only reactive approach
- **Deepnote**: Cloud-native collaboration focus

Project Mars aims to lead in the **performance** and **multi-language** aspects while providing a unified, independent platform.

## Get Started

Explore Project Mars's multi-language approach:

- [Project Mars Desktop App](packages/desktop) - Native application
- [Project Mars CLI](docs/02-platform/cli.md) - Command-line interface
- [Project Mars Runtime](docs/02-platform/runtime.md) - Multi-language execution
- [Getting Started](docs/01-foundations/getting-started.md) - First notebook tutorial
