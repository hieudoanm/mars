---
title: 'SciPy vs faer: Scientific Computing Libraries Compared'
description: "Compare Python's SciPy with Rust's faer for linear algebra and scientific computing primitives in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 15
tags: ['data-science', 'scipy', 'faer', 'rust', 'python', 'mars']
---

# SciPy vs faer: Scientific Computing Compared

SciPy covers a broad scientific stack in Python, from linear algebra to optimization. Rust's **`faer`** offers high-performance linear algebra primitives with a strong focus on correctness and speed.

## What SciPy does

SciPy extends NumPy with a comprehensive set of scientific computing tools:

- **Linear algebra** via `scipy.linalg`
- **Optimization** algorithms
- **Integration** and differential equation solvers
- **Signal processing** and Fourier transforms
- **Sparse matrices** and iterative solvers
- **Statistical functions** and distributions

It became the backbone of scientific computing in Python, used across physics, engineering, and biology.

## What faer brings to Rust

**`faer`** is a Rust library focused on high-performance linear algebra:

- **Dense linear algebra** with BLAS/LAPACK integration
- **Matrix operations** optimized for modern CPUs
- **Cholesky, QR, SVD** decompositions
- **Eigenvalue** problems
- **Sparse matrix** support
- **Compile-time optimization** with generics

The key difference: faer leverages Rust's type system and compile-time optimizations to produce fast, safe linear algebra code without runtime overhead.

## Comparison

| Feature               | SciPy (Python)                  | faer (Rust)                                |
| --------------------- | ------------------------------- | ------------------------------------------ |
| Matrix multiplication | `np.dot(A, B)`                  | `A * B`                                    |
| Decomposition         | `scipy.linalg.cholesky(A)`      | `A.cholesky()`                             |
| Performance           | C-backed via LAPACK             | Native BLAS/LAPACK, compile-time optimized |
| Type safety           | Runtime                         | Compile-time                               |
| Memory layout         | Row-major or column-major       | Explicit control, column-major by default  |
| Integration           | NumPy, pandas, Matplotlib, etc. | ndarray, nalgebra, etc.                    |

## Example: SciPy

```python
import numpy as np
from scipy import linalg

A = np.array([[1, 2], [3, 4]])
result = linalg.cholesky(A)
```

## Example: faer

```rust
use faer::prelude::*;

let A = Mat::from_fn(2, 2, |i, j| (i * 2 + j + 1) as f64);
let result = A.cholesky();
```

## When to use each

- **SciPy** in Python when you need the full scientific stack, optimization routines, or integration with NumPy-based workflows.
- **faer** in Rust when you need fast linear algebra, compile-time safety, or want to avoid the Python interpreter overhead for matrix-heavy computations.

## Try it in Project Mars

Both ecosystems share one `.mars` document: Python cells run in a persistent interpreter, Rust cells on the replay runtime with `// :dep` Cargo dependencies — variables persist across cells within each language's own session.

Numerical linear algebra — solve Ax = b — once per language:

```python
# SciPy side.
import numpy as np
from scipy.linalg import solve

A = np.array([[3.0, 1.0], [1.0, 2.0]])
b = np.array([9.0, 8.0])
solve(A, b)   # -> [2. 3.]
```

```rust
// :dep ndarray = "0.16"

use ndarray::array;

// faer solves this natively; gradient-descent equivalent with ndarray:
let a = array![[3.0, 1.0], [1.0, 2.0]];
let b = array![9.0, 8.0];
let mut x = array![0.0, 0.0];
for _ in 0..200 {
    let r = &a.dot(&x) - &b;
    x = &x - 0.1 * &a.t().dot(&r);
}
println!("solution ≈ {x:?}");
```

## How Project Mars supports both

Project Mars supports both SciPy and faer in their respective language runtimes:

- **Python cells** can use SciPy with standard library imports
- **Rust cells** can use faer via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Matrices persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore SciPy and faer in Project Mars notebooks.
