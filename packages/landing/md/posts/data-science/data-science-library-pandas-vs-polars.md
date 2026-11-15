---
title: "pandas vs Polars: Modern DataFrame Libraries Compared"
description: "Compare Python's pandas with Rust's Polars for DataFrames, tabular data manipulation, and analytics in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 14
tags: ["data-science", "pandas", "polars", "rust", "python", "mars"]
---

# pandas vs Polars: Tabular Data Processing Compared

pandas made DataFrames the default interface for tabular data in Python. Rust's **`polars`** brings a modern DataFrame engine with a focus on performance, parallel execution, and a clear expression API.

## What pandas does

pandas revolutionized data analysis in Python by introducing:

- **DataFrame** as a core data structure for tabular data
- **I/O** for CSV, JSON, Excel, SQL, and other formats
- **Data manipulation** via filtering, grouping, joining, and aggregating
- **Time series** support with datetime indexing
- **Integration** with NumPy, Matplotlib, and the broader ecosystem

It became the de facto standard for data wrangling and exploratory analysis.

## What Polars brings to Rust

**`polars`** is a modern DataFrame library designed for speed and ergonomics:

- **Apache Arrow-based** columnar memory format for efficient operations
- **Lazy evaluation** for query optimization
- **Parallel execution** by default
- **Expression API** for building complex queries in a readable way
- **Multi-threaded** I/O for large datasets
- **Zero-copy** operations where possible

The key difference: Polars compiles DataFrame operations to optimized native code and executes them in parallel, making it significantly faster for large-scale data processing.

## Comparison

| Feature               | pandas (Python)                          | Polars (Rust)                              |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| DataFrame creation    | `pd.DataFrame({'a': [1, 2, 3]})`         | `df! {"a" => [1, 2, 3]}`                  |
| Filtering             | `df[df['a'] > 1]`                       | `df.filter(col("a").gt(1))`               |
| Grouping              | `df.groupby('a').sum()`                  | `df.groupby(["a"]).sum()`                  |
| Performance           | Single-threaded, C-backed               | Multi-threaded, Arrow-native               |
| Lazy evaluation       | No                                       | Yes, with query optimization               |
| Type safety           | Runtime                                  | Compile-time                               |
| Integration          | NumPy, SciPy, Matplotlib, etc.           | ndarray, plotters, serde, etc.            |

## Example: pandas

```python
import pandas as pd

df = pd.DataFrame({'a': [1, 2, 3], 'b': [4, 5, 6]})
result = df[df['a'] > 1].groupby('a').sum()
```

## Example: Polars

```rust
use polars::prelude::*;

let df = df! {
    "a" => &[1, 2, 3],
    "b" => &[4, 5, 6]
};
let result = df.filter(col("a").gt(1)).groupby(["a"]).sum();
```

## When to use each

- **pandas** in Python when you need the vast ecosystem, familiar syntax, or compatibility with existing Python workflows.
- **polars** in Rust when you need performance for large datasets, parallel execution, or want to avoid the Python interpreter overhead.

## How Project Mars supports both

Project Mars supports both pandas and Polars in their respective language runtimes:

- **Python cells** can use pandas with standard library imports
- **Rust cells** can use Polars via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- DataFrames persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore pandas and Polars in Project Mars notebooks.
