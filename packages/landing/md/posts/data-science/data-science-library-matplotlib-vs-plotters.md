---
title: 'Matplotlib vs plotters: Python vs Rust Plotting Libraries'
description: "Compare Python's Matplotlib with Rust's plotters for 2D plotting and chart rendering in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 17
tags: ['data-science', 'matplotlib', 'plotters', 'rust', 'python', 'mars']
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

| Feature     | Matplotlib (Python)          | plotters (Rust)                     |
| ----------- | ---------------------------- | ----------------------------------- |
| Line plot   | `plt.plot(x, y)`             | `Chart::new((100, 50)).draw(&line)` |
| Styling     | `plt.style.use('seaborn')`   | Theme-based styling                 |
| Backends    | Agg, SVG, PDF, interactive   | Bitmap, SVG, PDF                    |
| Type safety | Runtime                      | Compile-time                        |
| Performance | C-backed                     | Native compilation                  |
| Integration | NumPy, pandas, Jupyter, etc. | ndarray, polars, etc.               |

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

## Try it in Project Mars

Both ecosystems share one `.mars` document: Python cells run in a persistent interpreter, Rust cells on the replay runtime with `// :dep` Cargo dependencies — variables persist across cells within each language's own session.

A line chart's data pipeline — compute the series, then hand it to the renderer — once per language:

```python
# Matplotlib side: plot is captured as cell output.
import matplotlib.pyplot as plt
import numpy as np

x = np.linspace(0, 2 * np.pi, 50)
plt.plot(x, np.sin(x))
plt.title("sin(x)")
plt.show()
```

```rust
// :dep ndarray = "0.16"

use ndarray::Array2;

// plotters renders charts to SVG/PNG; the data side is ndarray.
let x = Array2::from_shape_fn((50, 1), |(i, _)| {
    (i as f64) * 2.0 * std::f64::consts::PI / 49.0
});
let y: Vec<f64> = x.column(0).iter().map(|v| v.sin()).collect();
println!("series ready: {} points, first={:.3}", y.len(), y[0]);
```

## How Project Mars supports both

Project Mars supports both Matplotlib and plotters in their respective language runtimes:

- **Python cells** can use Matplotlib with automatic figure capture to PNG
- **Rust cells** can use plotters via Cargo dependencies with SVG/PNG output
- Both can coexist in the same `.mars` notebook
- Chart outputs render as rich MIME bundles in the notebook

Start with the [getting started guide](/docs/getting-started) and the [plotting example](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/03-plotting) to explore Matplotlib and plotters in Project Mars notebooks.
