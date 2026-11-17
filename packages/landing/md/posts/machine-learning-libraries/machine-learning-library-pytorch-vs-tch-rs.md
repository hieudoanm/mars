---
title: 'PyTorch vs tch-rs: Deep Learning Frameworks Compared'
description: "Compare Python's PyTorch with Rust's tch-rs for deep learning with tensor operations and training loops in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 25
tags: ['machine-learning', 'pytorch', 'tch-rs', 'rust', 'python', 'deep-learning', 'mars']
---

# PyTorch vs tch-rs: Deep Learning Compared

PyTorch is popular for research and production deep learning. **`tch-rs`** provides Rust bindings to the Torch ecosystem, enabling tensor computation and neural network workflows from Rust code. In Project Mars, developers can prototype model logic interactively while staying in a compiled language.

## What PyTorch does

PyTorch provides a deep learning framework:

- **Tensor operations** with automatic differentiation
- **Neural network modules** (convolutions, RNNs, transformers)
- **Training loops** with optimizers and schedulers
- **GPU acceleration** via CUDA
- **Dynamic computation graphs**
- **Deployment** to production environments

It became the dominant framework for research and production deep learning in Python.

## What tch-rs brings to Rust

**`tch-rs`** provides Rust bindings to PyTorch:

- **Tensor operations** via LibTorch
- **Automatic differentiation** with autograd
- **Neural network modules** (linear, convolution, RNN)
- **Optimizers** (SGD, Adam, RMSprop)
- **GPU support** via CUDA
- **Type-safe** tensor operations

The key difference: tch-rs allows you to use PyTorch's capabilities from Rust, combining Torch's ecosystem with Rust's type safety and performance.

## Comparison

| Feature         | PyTorch (Python)          | tch-rs (Rust)                  |
| --------------- | ------------------------- | ------------------------------ |
| Tensor creation | `torch.tensor([1, 2, 3])` | `Tensor::of_slice(&[1, 2, 3])` |
| Neural network  | `nn.Linear(in, out)`      | `nn::linear(in, out)`          |
| Training loop   | Python for loops          | Rust iterators                 |
| Type safety     | Runtime                   | Compile-time (mostly)          |
| Performance     | Python + C++ backend      | Rust + C++ backend             |
| GPU support     | CUDA                      | CUDA                           |

## Example: PyTorch

```python
import torch

x = torch.tensor([1.0, 2.0, 3.0])
y = x * 2
```

## Example: tch-rs

```rust
use tch::Tensor;

let x = Tensor::of_slice(&[1.0, 2.0, 3.0]);
let y = x * 2;
```

## When to use each

- **PyTorch** in Python when you need the full Torch ecosystem, rapid prototyping, or integration with Python ML workflows.
- **tch-rs** in Rust when you want to use PyTorch from Rust, combine it with Rust's type system, or build Rust applications that leverage Torch.

## How Project Mars supports both

Project Mars supports both PyTorch and tch-rs in their respective language runtimes:

- **Python cells** can use PyTorch with standard library imports
- **Rust cells** can use tch-rs via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Tensors and models persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore PyTorch and tch-rs in Project Mars notebooks.
