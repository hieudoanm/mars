---
title: "10 Essential Data Science Tools"
description: "A side-by-side look at 10 essential data science tools in Python and their Rust equivalents, and how Mars supports the Rust stack fully."
author: "Mars Team"
date: "2026-11-06"
order: 3
tags: ["data-science", "rust", "python", "tools", "mars"]
---

# 10 Essential Data Science Tools

Data science workflows have long relied on a mature Python ecosystem. Rust now offers fast, memory-safe equivalents for many of those libraries, and Mars gives the Rust stack a first-class notebook home. This post maps 10 essential data science tools from Python to Rust and explains how Mars supports the Rust equivalents.

## Python to Rust at a glance

| Python library | Rust equivalent | What it does |
| --- | --- | --- |
| NumPy | `ndarray` | N-dimensional arrays and numerical computing |
| pandas | `polars` | DataFrames, tabular data manipulation, and analytics |
| SciPy | `faer` | Linear algebra and scientific computing primitives |
| Statsmodels | `statrs` | Statistical distributions, tests, and models |
| Matplotlib | `plotters` | 2D plotting and chart rendering |
| Seaborn | `charming` | High-level statistical visualizations |
| Plotly | `plotly` | Interactive, publication-quality charts |
| Jupyter | `evcxr_jupyter` | Interactive Rust execution in notebook environments |
| xarray | `ndarray` | Labeled multi-dimensional array workflows (via `ndarray` ecosystem) |
| NetworkX | `petgraph` | Graph data structures and network algorithms |

## 1. NumPy → `ndarray`

NumPy is the foundation of numerical computing in Python. In Rust, **`ndarray`** provides n-dimensional arrays, slicing, broadcasting-style operations, and efficient numerical routines. It compiles to native code and avoids interpreter overhead, making array-heavy notebook experiments fast and predictable.

## 2. pandas → `polars`

pandas made DataFrames the default interface for tabular data. **`polars`** brings a modern DataFrame engine to Rust with a focus on performance, parallel execution, and a clear expression API. It is well suited for filtering, grouping, joins, and exploratory analysis in Mars notebooks.

## 3. SciPy → `faer`

SciPy covers a broad scientific stack in Python. For core linear algebra and matrix computation in Rust, **`faer`** offers high-performance primitives with a strong focus on correctness and speed. It fits numerical workflows where dense linear algebra is a bottleneck.

## 4. Statsmodels → `statrs`

Statsmodels provides statistical modeling and inference tools in Python. **`statrs`** supplies Rust implementations of common statistical distributions, functions, and utilities that can underpin analysis, simulation, and modeling tasks inside a notebook.

## 5. Matplotlib → `plotters`

Matplotlib is the workhorse plotting library for Python. **`plotters`** is a flexible Rust drawing and charting library that can render line charts, bar charts, scatter plots, and more. In a notebook context, it enables rich visual output from compiled Rust code.

## 6. Seaborn → `charming`

Seaborn builds expressive statistical graphics on top of Matplotlib. **`charming`** brings a high-level charting API to Rust, making it easier to produce polished visualizations without manually assembling every plotting detail.

## 7. Plotly → `plotly`

Plotly is widely used for interactive, shareable visualizations in Python. The Rust **`plotly`** crate provides a similar path to interactive charts, helping notebook users explore data visually and communicate results clearly.

## 8. Jupyter → `evcxr_jupyter`

Jupyter defines the interactive notebook experience for many Python users. **`evcxr_jupyter`** demonstrates Rust's ability to run interactively in a notebook workflow. Mars builds on that goal with its own Rust-first notebook platform, document format (`.mars`), and execution architecture.

## 9. xarray → `ndarray`

xarray extends arrays with labels and dimensions for scientific datasets in Python. Rust workflows often compose **`ndarray`** with domain-specific crates to represent multi-dimensional data. In Mars, these array-centric workflows can live alongside Markdown, outputs, and reproducible execution.

## 10. NetworkX → `petgraph`

NetworkX is the go-to Python library for graphs and networks. **`petgraph`** provides graph data structures and algorithms in Rust, from traversals to shortest paths, with strong performance and type safety for network analysis tasks.

## How Mars supports the Rust equivalents fully

Mars is designed as a **Rust-first interactive notebook**. That means the Rust data science ecosystem is not an afterthought:

- **Native Rust execution.** Mars runs Rust code in a persistent session so variables, DataFrames, arrays, and models can be built up across cells.
- **Rich outputs.** Markdown, text, errors, tables, images, SVG, HTML, and chart-friendly outputs fit naturally into the `.mars` document model.
- **Reproducible workflows.** Notebooks are stored as versioned JSON (`.mars`), with outputs preserved and execution available through the CLI.
- **Cargo-friendly dependencies.** The runtime architecture is built to work with Rust crates like `ndarray`, `polars`, `faer`, `statrs`, `plotters`, `charming`, `plotly`, and `petgraph`.
- **Interoperability.** Mars can import and export `.ipynb` notebooks, making it easier to move between Python/Jupyter workflows and Rust-native exploration.

If you want to explore these tools interactively, start with the [getting started notebook](/docs/01-getting-started) and build from there in Rust.
