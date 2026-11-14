---
title: "Matplotlib vs plotters: Python vs Rust Plotting Libraries"
description: "Compare Python's Matplotlib with Rust's plotters for 2D plotting and chart rendering in notebook environments."
author: "Mars Team"
date: "2026-11-14"
order: 17
tags: ["data-science", "matplotlib", "plotters", "rust", "python", "mars"]
---

# Matplotlib vs plotters: 2D Plotting Compared

Matplotlib is the workhorse plotting library for Python. Rust's **`plotters`** is a flexible drawing and charting library that can render line charts, bar charts, scatter plots, and more with native performance.

## What Matplotlib does

Matplotlib provides a comprehensive plotting ecosystem:

- **Line plots**, scatter plots, bar charts, histograms
- **Customizable axes**, labels, legends, and styling
- **Multiple backends** (PNG, SVG, PDF, interactive)
- **Integration** with NumPy and pandas
- **Subplots** and complex figure layouts
- **Publication-quality** graphics

It became the default plotting library for Python, used across scientific computing, data science, and engineering.

## What plotters brings to Rust

**`plotters`** is a Rust library for creating 2D charts:

- **Chart types**: line, bar, scatter, histogram, pie, box plot
- **Customizable styling** with fonts, colors, and themes
- **Multiple backends**: bitmap (PNG, BMP), vector (SVG, PDF)
- **Composable** chart building with a fluent API
- **Performance** via native rendering
- **Type-safe** API that catches errors at compile time

The key difference: plotters compiles chart rendering to native code and outputs directly to image files or SVG without the Python interpreter overhead.

## Comparison

| Feature               | Matplotlib (Python)                      | plotters (Rust)                             |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Line plot             | `plt.plot(x, y)`                        | `Chart::new((100, 50)).draw(&line)`       |
| Styling               | `plt.style.use('seaborn')`              | Theme-based styling                         |
| Backends              | Agg, SVG, PDF, interactive              | Bitmap, SVG, PDF                           |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | C-backed                                 | Native compilation                         |
| Integration          | NumPy, pandas, Jupyter, etc.            | ndarray, polars, etc.                     |

## Example: Matplotlib

```python
import matplotlib.pyplot as plt

x = [1, 2, 3, 4, 5]
y = [1, 4, 9, 16, 25]
plt.plot(x, y)
plt.savefig('plot.png')
```

## Example: plotters

```rust
use plotters::prelude::*;

let root = BitMapBackend::new("plot.png", (640, 480)).into_drawing_area();
let mut chart = ChartBuilder::on(&root)
    .caption("Line Plot", ("sans-serif", 20))
    .build()
    .unwrap();
```

## When to use each

- **Matplotlib** in Python when you need the vast ecosystem, familiar API, or integration with NumPy and pandas.
- **plotters** in Rust when you need native performance, type safety, or want to avoid the Python interpreter overhead for chart rendering.

## How Mars supports both

Mars supports both Matplotlib and plotters in their respective language runtimes:

- **Python cells** can use Matplotlib with automatic figure capture to PNG
- **Rust cells** can use plotters via Cargo dependencies with SVG/PNG output
- Both can coexist in the same `.mars` notebook
- Chart outputs render as rich MIME bundles in the notebook

Start with the [getting started guide](/docs/01-getting-started) and the [plotting example](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/03-plotting) to explore Matplotlib and plotters in Mars notebooks.
