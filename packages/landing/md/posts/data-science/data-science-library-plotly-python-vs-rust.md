---
title: "Plotly vs plotly: Interactive Visualization Libraries"
description: "Compare Python's Plotly with Rust's plotly for interactive, publication-quality charts in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 19
tags: ["data-science", "plotly", "rust", "python", "interactive", "mars"]
---

# Plotly vs plotly: Interactive Charts Compared

Plotly is widely used for interactive, shareable visualizations in Python. The Rust **`plotly`** crate provides a similar path to interactive charts, helping notebook users explore data visually and communicate results clearly.

## What Plotly does

Plotly provides an interactive charting ecosystem:

- **Interactive charts** (line, scatter, bar, pie, 3D plots)
- **Hover tooltips** and zoom/pan capabilities
- **Web-based rendering** via HTML/JavaScript
- **Export** to PNG, SVG, and PDF
- **Dash integration** for building dashboards
- **Collaborative sharing** via Plotly Cloud

It became the default library for interactive visualization in Python, especially for dashboards and exploratory analysis.

## What plotly brings to Rust

**`plotly`** is a Rust library for creating interactive charts:

- **Chart types**: line, scatter, bar, pie, heatmap, 3D
- **Interactive features**: zoom, pan, hover tooltips
- **HTML/JS rendering** for web deployment
- **Export** to static images
- **Type-safe** chart construction
- **Native performance** without Python overhead

The key difference: plotly generates interactive charts from Rust code and renders them in web browsers, combining Rust's performance with Plotly's interactivity.

## Comparison

| Feature               | Plotly (Python)                          | plotly (Rust)                               |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Line plot             | `fig = px.line(df, x='x', y='y')`       | `Plot::new().line(x, y)`                   |
| Interactivity         | Zoom, pan, hover tooltips                | Zoom, pan, hover tooltips                   |
| Rendering             | HTML/JavaScript                          | HTML/JavaScript                            |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | Python + JavaScript                        | Rust + JavaScript                          |
| Integration          | pandas, NumPy, Dash, etc.                | ndarray, polars, etc.                      |

## Example: Plotly

```python
import plotly.express as px

df = px.data.iris()
fig = px.scatter(df, x='sepal_width', y='sepal_length')
fig.show()
```

## Example: plotly

```rust
use plotly::Plot;

let mut plot = Plot::new();
plot.add_trace(plotly::Scatter::new(x, y));
plot.show();
```

## When to use each

- **Plotly** in Python when you need interactive charts, Dash integration, or want to share visualizations via Plotly Cloud.
- **plotly** in Rust when you need interactive charts with type safety and want to avoid the Python interpreter overhead.

## How Project Mars supports both

Project Mars supports both Plotly versions in their respective language runtimes:

- **Python cells** can use Plotly with HTML output rendering
- **Rust cells** can use plotly via Cargo dependencies with HTML output
- Both can coexist in the same `.mars` notebook
- Interactive chart outputs render as HTML MIME bundles in the notebook

Start with the [getting started guide](/docs/01-getting-started) to explore Plotly in Project Mars notebooks.
