---
title: "Seaborn vs charming: Statistical Visualization Libraries"
description: "Compare Python's Seaborn with Rust's charming for high-level statistical visualizations in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 18
tags: ["data-science", "seaborn", "charming", "rust", "python", "mars"]
---

# Seaborn vs charming: Statistical Visualization Compared

Seaborn builds expressive statistical graphics on top of Matplotlib in Python. Rust's **`charming`** brings a high-level charting API to Rust, making it easier to produce polished visualizations without manually assembling every plotting detail.

## What Seaborn does

Seaborn simplifies statistical visualization by providing:

- **Statistical plots** (box plots, violin plots, distribution plots)
- **Built-in themes** and color palettes
- **Automatic estimation** of confidence intervals
- **Faceting** for multi-panel figures
- **Integration** with pandas DataFrames
- **Regression plots** with fitted lines and confidence bands

It became the preferred library for statistical graphics in Python, especially for exploratory data analysis.

## What charming brings to Rust

**`charming`** is a Rust library for high-level statistical visualization:

- **Statistical chart types** (histograms, box plots, violin plots)
- **Theme support** for consistent styling
- **Confidence interval** estimation
- **Faceting** for grouped visualizations
- **Composable** with plotters for rendering
- **Type-safe** API for statistical operations

The key difference: charming provides a high-level abstraction over plotters, making it easier to create statistical visualizations while maintaining Rust's performance and type safety.

## Comparison

| Feature               | Seaborn (Python)                         | charming (Rust)                             |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Box plot              | `sns.boxplot(data=df)`                  | `Chart::new().boxplot(data)`               |
| Theme                 | `sns.set_theme('darkgrid')`             | Theme-based styling                         |
| Confidence interval   | Automatic                                 | Configurable, compile-time checked          |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | Matplotlib backend                        | Native plotters rendering                  |
| Integration          | pandas, Matplotlib, NumPy, etc.          | plotters, ndarray, polars, etc.           |

## Example: Seaborn

```python
import seaborn as sns
import matplotlib.pyplot as plt

sns.boxplot(data=df, x='category', y='value')
plt.savefig('boxplot.png')
```

## Example: charming

```rust
use charming::prelude::*;

let chart = Chart::new().boxplot(data).theme(Theme::Dark);
chart.save("boxplot.png")?;
```

## When to use each

- **Seaborn** in Python when you need a high-level API for statistical graphics, built-in themes, or integration with pandas and Matplotlib.
- **charming** in Rust when you need high-level statistical visualization with type safety and native performance.

## How Project Mars supports both

Project Mars supports both Seaborn and charming in their respective language runtimes:

- **Python cells** can use Seaborn with automatic figure capture to PNG
- **Rust cells** can use charming via Cargo dependencies with SVG/PNG output
- Both can coexist in the same `.mars` notebook
- Statistical chart outputs render as rich MIME bundles in the notebook

Start with the [getting started guide](/docs/getting-started) to explore Seaborn and charming in Project Mars notebooks.
