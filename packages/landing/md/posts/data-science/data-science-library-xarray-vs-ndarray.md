---
title: 'xarray vs ndarray: N-Dimensional Array Libraries'
description: "Compare Python's xarray with Rust's ndarray for labeled multi-dimensional array workflows in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 22
tags: ['data-science', 'xarray', 'ndarray', 'rust', 'python', 'mars']
---

# xarray vs ndarray: Labeled Arrays Compared

xarray extends arrays with labels and dimensions for scientific datasets in Python. Rust workflows often compose **`ndarray`** with domain-specific crates to represent multi-dimensional data. In Project Mars, these array-centric workflows can live alongside Markdown, outputs, and reproducible execution.

## What xarray does

xarray provides labeled n-dimensional arrays:

- **Named dimensions** and coordinates
- **Label-based indexing** and selection
- **Grouped operations** along dimensions
- **Integration** with NumPy, pandas, and dask
- **I/O** for NetCDF, Zarr, and other scientific formats
- **Out-of-core** computation via dask

It became the standard for working with labeled, multi-dimensional scientific data in Python.

## What ndarray brings to Rust

**`ndarray`** provides the core array representation in Rust:

- **N-dimensional arrays** with generic element types
- **Efficient storage** via contiguous memory layouts
- **Slicing and views** without copying
- **Broadcasting-style operations**
- **Integration** with domain-specific crates for labeled data
- **Zero-cost abstractions** that compile to optimized native code

The key difference: ndarray provides the core array primitive, while domain-specific Rust crates add labeling and metadata. This modular approach allows flexibility while maintaining performance.

## Comparison

| Feature          | xarray (Python)                       | ndarray (Rust)                   |
| ---------------- | ------------------------------------- | -------------------------------- |
| Array creation   | `xr.DataArray(data, dims=['x', 'y'])` | `Array::from_shape((x, y))`      |
| Labeled indexing | `da.sel(x=1)`                         | Manual via domain crates         |
| Grouping         | `da.groupby('x').mean()`              | Manual implementation            |
| Type safety      | Runtime                               | Compile-time                     |
| Performance      | NumPy-backed                          | Native compilation               |
| Metadata         | Built-in coordinates                  | Custom structs via domain crates |

## Example: xarray

```python
import xarray as xr

da = xr.DataArray(
    [[1, 2], [3, 4]],
    dims=['x', 'y'],
    coords={'x': [0, 1], 'y': [0, 1]}
)
result = da.sel(x=0)
```

## Example: ndarray

```rust
use ndarray::prelude::*;

let arr = Array::from_shape((2, 2)).unwrap();
// Add labeling via custom structs or domain crates
```

## When to use each

- **xarray** in Python when you need labeled arrays, coordinate systems, or integration with scientific data formats (NetCDF, Zarr).
- **ndarray** in Rust when you need core array operations with type safety and performance, adding labels via domain-specific crates as needed.

## How Project Mars supports both

Project Mars supports both xarray and ndarray in their respective language runtimes:

- **Python cells** can use xarray with standard library imports
- **Rust cells** can use ndarray via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Arrays persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore xarray and ndarray in Project Mars notebooks.
