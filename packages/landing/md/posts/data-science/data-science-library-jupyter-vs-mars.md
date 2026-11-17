---
title: 'Jupyter vs Project Mars: Notebook Platforms Compared'
description: "Compare Python's Jupyter with Project Mars for interactive notebook environments, multi-language support, and data science workflows."
author: 'Project Mars Team'
date: '2026-11-14'
order: 20
tags: ['data-science', 'jupyter', 'mars', 'rust', 'python', 'notebook', 'multi-language']
---

# Jupyter vs Project Mars: Interactive Notebooks Compared

Jupyter defines the interactive notebook experience for many Python users. Project Mars builds on that paradigm with a Rust-first platform, multi-language support, and a new document format designed for reproducible data science workflows.

## What Jupyter does

Jupyter introduced the notebook paradigm to data science:

- **Interactive code cells** with execution and output
- **Markdown cells** for documentation
- **Rich outputs** (HTML, images, LaTeX, widgets)
- **Kernel architecture** for language backends
- **File format** (`.ipynb`) for sharing notebooks
- **Extensible ecosystem** of kernels and extensions

It became the default environment for data science experimentation, education, and research.

## What Project Mars brings

Project Mars provides a Rust-first interactive notebook platform:

- **Rust-first design** with native `.mars` document format
- **Multi-language support** (Rust, Python, R, Julia, C++) in the same notebook
- **Persistent runtime sessions** for each language
- **Replay-based Rust execution** with Cargo dependencies
- **Desktop application** with CodeMirror editor and LSP integration
- **CLI** for headless execution and automation
- **Jupyter interoperability** via import/export

The key difference: Project Mars is designed as an independent notebook platform with a native document format and execution architecture, not just another Jupyter kernel or frontend.

## Comparison

| Feature              | Jupyter (Python)             | Project Mars                        |
| -------------------- | ---------------------------- | ----------------------------------- |
| Code execution       | Python kernel                | Rust, Python, R, Julia, C++ kernels |
| Cell structure       | Code, Markdown, raw          | Code, Markdown, raw                 |
| Rich outputs         | HTML, images, LaTeX, widgets | HTML, images, LaTeX, tables, charts |
| Variable persistence | Across cells (Python)        | Across cells (per language)         |
| Document format      | `.ipynb` (JSON)              | `.mars` (JSON, versioned)           |
| Type safety          | Runtime                      | Compile-time (Rust)                 |
| Dependencies         | pip, conda                   | Cargo (Rust), pip (Python), etc.    |
| Multi-language       | Via multiple kernels         | Native multi-language support       |
| Desktop editor       | Web-based (JupyterLab)       | Native Tauri application with LSP   |
| CLI                  | Limited                      | Full CLI for automation             |

## Example: Jupyter

```python
import numpy as np
import pandas as pd

arr = np.array([1, 2, 3])
df = pd.DataFrame({'a': arr})
df
```

## Example: Project Mars (Rust)

```rust
// Cell 1: Rust
let arr = vec![1, 2, 3];

// Cell 2: Python (in same notebook)
# Cell: Python
import numpy as np
arr = np.array([1, 2, 3])
```

## When to use each

- **Jupyter** in Python when you need the mature ecosystem, broad language support via kernels, or integration with Python-specific tools.
- **Project Mars** when you want a Rust-first notebook platform, multi-language notebooks in a single file, native desktop application, or CLI automation.

## How Project Mars supports Jupyter workflows

Project Mars provides Jupyter interoperability:

- **Import** `.ipynb` notebooks to `.mars` format
- **Export** `.mars` notebooks to `.ipynb` format
- **Rich output preservation** (HTML, images, LaTeX, tables)
- **Metadata preservation** for unsupported features
- **Cell structure mapping** (code, Markdown, raw)

This makes it easy to migrate existing Jupyter workflows to Project Mars while maintaining compatibility with the broader ecosystem.

## Multi-language notebooks

Project Mars allows mixing languages in a single notebook:

```mars
// Cell 1: Rust
let signal = load_signal();

// Cell 2: Python
# Use Python's pandas for analysis
import pandas as pd
df = pd.DataFrame(signal)

// Cell 3: R
# Use R for statistical tests
t.test(df.values)
```

Each language has its own persistent runtime, so variables persist across cells within that language.

## Get started with Project Mars

Explore Project Mars's multi-language notebook platform:

- [Getting started guide](/docs/getting-started)
- [Runtime documentation](/docs/02-platform/runtime.md)
- [CLI reference](/docs/02-platform/cli.md)
- [Desktop application](packages/desktop)
- [Extension guides](/docs/extension-vscode.md)
