---
title: 'NumPy vs ndarray: Choosing the Right Array Library'
description: "Compare Python's NumPy with Rust's ndarray for n-dimensional arrays and numerical computing in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 13
tags: ['data-science', 'numpy', 'ndarray', 'rust', 'python', 'mars']
---

# NumPy vs ndarray: Numerical Computing Compared

NumPy is the foundation of numerical computing in Python. Rust's **`ndarray`** provides n-dimensional arrays, slicing, broadcasting-style operations, and efficient numerical routines—compiled to native code with no interpreter overhead.

## What NumPy does

NumPy introduced the concept of n-dimensional arrays to Python, enabling:

- Efficient array storage and memory management
- Vectorized operations without explicit loops
- Broadcasting for element-wise operations across arrays of different shapes
- Linear algebra primitives
- Integration with the broader scientific Python ecosystem

It became the default data structure for numerical work, and nearly every Python data science library builds on it.

## What ndarray brings to Rust

**`ndarray`** is Rust's answer to NumPy's array model:

- **N-dimensional arrays** with generic element types
- **Efficient storage** via contiguous memory layouts
- **Slicing and views** without copying data
- **Broadcasting-style operations** through explicit APIs
- **Linear algebra** via integration with BLAS/LAPACK
- **Zero-cost abstractions** that compile to optimized native code

The key difference: ndarray compiles array operations to native machine code, so tight numerical loops run at C-like speeds without leaving Rust.

## Comparison

| Feature           | NumPy (Python)                  | ndarray (Rust)                      |
| ----------------- | ------------------------------- | ----------------------------------- |
| Array creation    | `np.array([1, 2, 3])`           | `array![1, 2, 3]`                   |
| Slicing           | `arr[1:5, ::2]`                 | `arr.slice(s![1..5, ..;2])`         |
| Broadcasting      | Automatic                       | Explicit APIs, compile-time checked |
| Type safety       | Runtime                         | Compile-time                        |
| Performance       | C-backed via NumPy              | Native compilation, no interpreter  |
| Memory management | Reference counting              | RAII, no garbage collector          |
| Integration       | Pandas, SciPy, Matplotlib, etc. | Polars, faer, plotters, etc.        |

## Example: NumPy

```python
import numpy as np

arr = np.array([[1, 2, 3], [4, 5, 6]])
result = arr * 2  # Element-wise multiplication
```

## Example: ndarray

```rust
use ndarray::prelude::*;

let arr = array![[1, 2, 3], [4, 5, 6]];
let result = &arr * 2; // Element-wise multiplication
```

## When to use each

- **NumPy** in Python when you need maximum ecosystem compatibility, rapid prototyping, or integration with pandas, scikit-learn, and Matplotlib.
- **ndarray** in Rust when you need compile-time safety, predictable performance, or want to avoid the Python interpreter overhead for array-heavy computations.

## Try it in Project Mars

Both ecosystems share one `.mars` document: Python cells run in a persistent interpreter, Rust cells on the replay runtime with `// :dep` Cargo dependencies — variables persist across cells within each language's own session.

The canonical pair — same broadcasting operation, once per language:

```python
# NumPy side: broadcasting (3,) against (2, 3).
import numpy as np

a = np.array([[1, 2, 3], [4, 5, 6]])
b = np.array([10, 20, 30])
a + b
```

```rust
// :dep ndarray = "0.16"

use ndarray::array;

// ndarray side: identical broadcast semantics.
let a = array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
let b = array![10.0, 20.0, 30.0];
let sum = &a + &b;
println!("{sum:?}");
```

## How Project Mars supports both

Project Mars runs both Python and Rust in the same notebook environment:

- **Python cells** can use NumPy with standard library imports
- **Rust cells** can use ndarray via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Variables persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore NumPy and ndarray in Project Mars notebooks.
