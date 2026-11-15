---
title: "JAX vs dfdx: Composable ML and Autodiff"
description: "Compare Python's JAX with Rust's dfdx for composable array/tensor transformations and autodiff-style workflows in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 31
tags: ["machine-learning", "jax", "dfdx", "rust", "python", "autodiff", "mars"]
---

# JAX vs dfdx: Composable ML Compared

JAX combines array programming with composable transformations. **`dfdx`** offers a Rust approach to tensor and differentiable programming patterns, giving developers a path to experiment with modern ML building blocks in a safe, compiled setting.

## What JAX does

JAX provides a composable transformation library:

- **Automatic differentiation** with forward and reverse modes
- **Just-in-time compilation** via XLA
- **Vectorized functions** (vmap, pmap)
- **Array programming** with NumPy-like API
- **Neural network libraries** (flax, optax)
- **GPU/TPU acceleration**

It became the preferred library for research requiring composable transformations and automatic differentiation.

## What dfdx brings to Rust

**`dfdx`** is a Rust library for differentiable programming:

- **Automatic differentiation** via forward and reverse modes
- **Tensor operations** with gradients
- **Neural network modules** (linear, convolution, etc.)
- **Training loops** with optimizers
- **Type-safe** tensor operations
- **Zero-cost abstractions** that compile to efficient code

The key difference: dfdx provides a Rust-native approach to differentiable programming with compile-time type safety, while JAX offers JIT compilation and a NumPy-like API with broad research adoption.

## Comparison

| Feature               | JAX (Python)                             | dfdx (Rust)                                  |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Tensor creation       | `jnp.array([1, 2, 3])`                  | `Tensor::from_array([1, 2, 3])`            |
| Gradient computation  | `grad(f)(x)`                            | `backward(&f, &x)`                        |
| Vectorization         | `vmap(f)`                               | Manual implementation                       |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | JIT via XLA                              | Native compilation                         |
| Integration          | NumPy, flax, optax, etc.                | ndarray, burn, etc.                       |

## Example: JAX

```python
import jax.numpy as jnp
from jax import grad

def f(x):
    return jnp.sum(x ** 2)

grad_f = grad(f)
```

## Example: dfdx

```rust
use dfdx::prelude::*;

fn f(x: &Tensor) -> Tensor {
    x.pow_scalar(2.0).sum()
}

let grad_f = backward(&f, &x)?;
```

## When to use each

- **JAX** in Python when you need composable transformations, JIT compilation, or integration with the JAX ecosystem (flax, optax).
- **dfdx** in Rust when you need automatic differentiation with type safety, or want to build differentiable ML components in Rust.

## How Project Mars supports both

Project Mars supports both JAX and dfdx in their respective language runtimes:

- **Python cells** can use JAX with standard library imports
- **Rust cells** can use dfdx via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Tensors and gradients persist across cells within each language runtime

Start with the [getting started guide](/docs/01-getting-started) to explore JAX and dfdx in Project Mars notebooks.
