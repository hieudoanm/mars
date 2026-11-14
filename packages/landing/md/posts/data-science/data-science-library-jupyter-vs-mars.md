---
title: "Jupyter vs Mars: Notebook Platforms Compared"
description: "Compare Python's Jupyter with Mars for interactive notebook environments, multi-language support, and data science workflows."
author: "Mars Team"
date: "2026-11-14"
order: 20
tags: ["data-science", "jupyter", "mars", "rust", "python", "notebook", "multi-language"]
---

# Jupyter vs Mars: Interactive Notebooks Compared

Jupyter defines the interactive notebook experience for many Python users. Mars builds on that paradigm with a Rust-first platform, multi-language support, and a new document format designed for reproducible data science workflows.

## What Jupyter does

Jupyter introduced the notebook paradigm to data science:

- **Interactive code cells** with execution and output
- **Markdown cells** for documentation
- **Rich outputs** (HTML, images, LaTeX, widgets)
- **Kernel architecture** for language backends
- **File format** (`.ipynb`) for sharing notebooks
- **Extensible ecosystem** of kernels and extensions

It became the default environment for data science experimentation, education, and research.

## What Mars brings

Mars provides a Rust-first interactive notebook platform:

- **Rust-first design** with native `.mars` document format
- **Multi-language support** (Rust, Python, R, Julia, C++) in the same notebook
- **Persistent runtime sessions** for each language
- **Replay-based Rust execution** with Cargo dependencies
- **Desktop application** with CodeMirror editor and LSP integration
- **CLI** for headless execution and automation
- **Jupyter interoperability** via import/export

The key difference: Mars is designed as an independent notebook platform with a native document format and execution architecture, not just another Jupyter kernel or frontend.

## Comparison

| Feature               | Jupyter (Python)                          | Mars                                        |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Code execution       | Python kernel                             | Rust, Python, R, Julia, C++ kernels        |
| Cell structure        | Code, Markdown, raw                      | Code, Markdown, raw                         |
| Rich outputs          | HTML, images, LaTeX, widgets             | HTML, images, LaTeX, tables, charts         |
| Variable persistence  | Across cells (Python)                    | Across cells (per language)                |
| Document format       | `.ipynb` (JSON)                          | `.mars` (JSON, versioned)                  |
| Type safety           | Runtime                                  | Compile-time (Rust)                         |
| Dependencies          | pip, conda                                | Cargo (Rust), pip (Python), etc.           |
| Multi-language        | Via multiple kernels                    | Native multi-language support             |
| Desktop editor        | Web-based (JupyterLab)                   | Native Tauri application with LSP          |
| CLI                   | Limited                                  | Full CLI for automation                    |

## Example: Jupyter

```python
import numpy as np
import pandas as pd

arr = np.array([1, 2, 3])
df = pd.DataFrame({'a': arr})
df
```

## Example: Mars (Rust)

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
- **Mars** when you want a Rust-first notebook platform, multi-language notebooks in a single file, native desktop application, or CLI automation.

## How Mars supports Jupyter workflows

Mars provides Jupyter interoperability:

- **Import** `.ipynb` notebooks to `.mars` format
- **Export** `.mars` notebooks to `.ipynb` format
- **Rich output preservation** (HTML, images, LaTeX, tables)
- **Metadata preservation** for unsupported features
- **Cell structure mapping** (code, Markdown, raw)

This makes it easy to migrate existing Jupyter workflows to Mars while maintaining compatibility with the broader ecosystem.

## Multi-language notebooks

Mars allows mixing languages in a single notebook:

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

## Get started with Mars

Explore Mars's multi-language notebook platform:

- [Getting started guide](/docs/01-getting-started)
- [Runtime documentation](/docs/06-runtime.md)
- [CLI reference](/docs/12-cli.md)
- [Desktop application](packages/desktop)
- [Extension guides](/docs/10-extension-vscode.md)
