---
title: '10 Essential Data Science Tools for Modern Workflows'
description: 'A side-by-side look at 10 essential data science tools in Python and their Rust equivalents, and how Project Mars supports both ecosystems fully.'
author: 'Project Mars Team'
date: '2026-11-06'
order: 12
tags: ['data-science', 'rust', 'python', 'tools', 'mars']
---

# 10 Essential Data Science Tools

Data science workflows have long relied on a mature Python ecosystem. Rust now offers fast, memory-safe equivalents for many of those libraries, and Project Mars gives both Rust and Python stacks a first-class notebook home. This post maps 10 essential data science tools from Python to Rust and explains how Project Mars supports both ecosystems.

## Python to Rust at a glance

| Python library | Rust equivalent | What it does                                                        |
| -------------- | --------------- | ------------------------------------------------------------------- |
| NumPy          | `ndarray`       | N-dimensional arrays and numerical computing                        |
| pandas         | `polars`        | DataFrames, tabular data manipulation, and analytics                |
| SciPy          | `faer`          | Linear algebra and scientific computing primitives                  |
| Statsmodels    | `statrs`        | Statistical distributions, tests, and models                        |
| Matplotlib     | `plotters`      | 2D plotting and chart rendering                                     |
| Seaborn        | `charming`      | High-level statistical visualizations                               |
| Plotly         | `plotly`        | Interactive, publication-quality charts                             |
| Jupyter        | Project Mars    | Multi-language notebook platform with Rust-first design             |
| xarray         | `ndarray`       | Labeled multi-dimensional array workflows (via `ndarray` ecosystem) |
| NetworkX       | `petgraph`      | Graph data structures and network algorithms                        |

## 1. NumPy → `ndarray`

NumPy is the foundation of numerical computing in Python. In Rust, **`ndarray`** provides n-dimensional arrays, slicing, broadcasting-style operations, and efficient numerical routines. It compiles to native code and avoids interpreter overhead, making array-heavy notebook experiments fast and predictable.

See [NumPy vs ndarray](/blog/data-science-library-numpy-vs-ndarray) for a detailed comparison.

## 2. pandas → `polars`

pandas made DataFrames the default interface for tabular data. **`polars`** brings a modern DataFrame engine to Rust with a focus on performance, parallel execution, and a clear expression API. It is well suited for filtering, grouping, joins, and exploratory analysis in Project Mars notebooks.

See [pandas vs Polars](/blog/data-science-library-pandas-vs-polars) for a detailed comparison.

## 3. SciPy → `faer`

SciPy covers a broad scientific stack in Python. For core linear algebra and matrix computation in Rust, **`faer`** offers high-performance primitives with a strong focus on correctness and speed. It fits numerical workflows where dense linear algebra is a bottleneck.

See [SciPy vs faer](/blog/data-science-library-scipy-vs-faer) for a detailed comparison.

## 4. Statsmodels → `statrs`

Statsmodels provides statistical modeling and inference tools in Python. **`statrs`** supplies Rust implementations of common statistical distributions, functions, and utilities that can underpin analysis, simulation, and modeling tasks inside a notebook.

See [Statsmodels vs statrs](/blog/data-science-library-statsmodels-vs-statrs) for a detailed comparison.

## 5. Matplotlib → `plotters`

Matplotlib is the workhorse plotting library for Python. **`plotters`** is a flexible Rust drawing and charting library that can render line charts, bar charts, scatter plots, and more. In a notebook context, it enables rich visual output from compiled Rust code.

See [Matplotlib vs plotters](/blog/data-science-library-matplotlib-vs-plotters) for a detailed comparison.

## 6. Seaborn → `charming`

Seaborn builds expressive statistical graphics on top of Matplotlib. **`charming`** brings a high-level charting API to Rust, making it easier to produce polished visualizations without manually assembling every plotting detail.

See [Seaborn vs charming](/blog/data-science-library-seaborn-vs-charming) for a detailed comparison.

## 7. Plotly → `plotly`

Plotly is widely used for interactive, shareable visualizations in Python. The Rust **`plotly`** crate provides a similar path to interactive charts, helping notebook users explore data visually and communicate results clearly.

See [Plotly Python vs Rust](/blog/data-science-library-plotly-python-vs-rust) for a detailed comparison.

## 8. Jupyter → Project Mars

Jupyter defines the interactive notebook experience for many Python users. Project Mars builds on that paradigm with a Rust-first platform, multi-language support (Rust, Python, R, Julia, C++), and a new document format designed for reproducible data science workflows.

See [Jupyter vs Project Mars](/blog/data-science-library-jupyter-vs-mars) for a detailed comparison.

## 9. xarray → `ndarray`

xarray extends arrays with labels and dimensions for scientific datasets in Python. Rust workflows often compose **`ndarray`** with domain-specific crates to represent multi-dimensional data. In Project Mars, these array-centric workflows can live alongside Markdown, outputs, and reproducible execution.

See [xarray vs ndarray](/blog/data-science-library-xarray-vs-ndarray) for a detailed comparison.

## 10. NetworkX → `petgraph`

NetworkX is the go-to Python library for graphs and networks. **`petgraph`** provides graph data structures and algorithms in Rust, from traversals to shortest paths, with strong performance and type safety for network analysis tasks.

See [NetworkX vs petgraph](/blog/data-science-library-networkx-vs-petgraph) for a detailed comparison.

## How Project Mars supports both ecosystems

Project Mars is designed as a **multi-language interactive notebook**. That means both Python and Rust data science ecosystems are first-class citizens:

- **Multi-language execution.** Project Mars runs Rust, Python, R, Julia, and C++ code in persistent sessions so variables, DataFrames, arrays, and models can be built up across cells in your preferred language.
- **Native Rust execution.** The Rust runtime supports Cargo dependencies for crates like `ndarray`, `polars`, `faer`, `statrs`, `plotters`, `charming`, `plotly`, and `petgraph`.
- **Python execution.** The Python runtime supports top-level `await`, matplotlib figure capture, and standard Python data science libraries (pandas, numpy, scikit-learn, etc.).
- **Julia execution.** The Julia runtime supports MIME bundle output, interactive input, and Julia's rich data science ecosystem (DataFrames.jl, Plots.jl, Flux.jl, etc.).
- **C++ execution.** The C++ runtime supports replay-based compilation with Conan/vcpkg dependencies.
- **Rich outputs.** Markdown, text, errors, tables, images, SVG, HTML, and chart-friendly outputs fit naturally into the `.mars` document model across all languages.
- **Reproducible workflows.** Notebooks are stored as versioned JSON (`.mars`), with outputs preserved and execution available through the CLI.
- **Interoperability.** Project Mars can import and export `.ipynb` notebooks, making it easier to move between Python/Jupyter workflows and multi-language Project Mars exploration.

If you want to explore these tools interactively, start with the [getting started guide](/docs/getting-started) and the [plotting example](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/03-plotting) which demonstrates vector plotting with plotters in a Project Mars notebook.
