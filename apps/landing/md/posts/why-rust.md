---
title: "Why Rust?"
description: "Rust is fast, memory-safe, and predictable. Here is why that matters for data science, ML, and the tools we build on top of it."
author: "Mars Team"
date: "2026-11-06"
order: 1
tags: ["rust", "performance", "safety", "ml", "systems"]
---

# Why Rust?

Rust is a systems programming language that compiles to native machine code, runs without a garbage collector, and prevents a whole class of bugs at compile time. It is not a new idea — its design borrows from ML, C++, and functional programming — but it packages those ideas into something that *just works*.

## What makes Rust different

Three properties show up in every Rust program:

1. **Memory safety without a garbage collector.** The borrow checker enforces rules about who owns a value and how long references to it live. Use-after-free, double frees, and dangling pointers are compile-time errors, not runtime surprises.
2. **Zero-cost abstractions.** Iterators, closures, generics, and pattern matching compile down to tight loops. You pay for what you use, so expressive code does not mean slow code.
3. **Fearless concurrency.** The type system guarantees that mutable data is either exclusively owned or immutably shared. You cannot have a data race, because a data race is a type error.

## Rust vs Python

Python is a fantastic glue language. Its interpreter is forgiving, its standard library is warm, and its ecosystem (NumPy, pandas, scikit-learn) is the largest in data science. But Python has two structural costs:

- **Dynamic typing and the interpreter overhead** make hot loops slow. Numerical workloads get pushed into C extensions, which means you are really writing C and gluing it together with Python.
- **The GIL** serializes Python threads, so true parallelism requires multiprocessing or a different runtime.

Rust flips the trade-off. You write safe, expressive code that compiles to a single binary with no runtime, no interpreter, and no garbage collection. You can call into C, use WebAssembly, or embed Rust directly in a Python process with PyO3. The result is a language that behaves like C for performance while staying as ergonomic as modern Python.

| | Python | Rust |
| --- | --- | --- |
| Runtime | Interpreter + GIL | Compiled native binary |
| Memory | GC | Ownership + borrowing |
| Hot loops | Slow (push to C) | Fast, inlined |
| Concurrency | Threads limited by GIL | True parallelism |
| Deployment | Runtime + deps | Single static binary |
| Error model | Exceptions | `Result` |

## Why Rust should be used in ML more

Machine learning workloads are *systems workloads in disguise*. Training a model means moving gigabytes of data through matrix kernels, managing GPU memory, and coordinating distributed workers. The bottlenecks are not algorithmic elegance — they are memory bandwidth, cache behavior, and parallelism.

Rust is the only language that gives you:

- **Predictable performance** with no GC pauses during training epochs.
- **Safe parallelism** for data-parallel and model-parallel training without locks or atomics everywhere.
- **First-class FFI** to CUDA, ROCm, BLAS, and C++ libraries, so you do not have to write your own kernels from scratch.
- **Reproducible builds** with `Cargo.lock`, which matters when you want a model trained in 2026 to be retrainable in 2027.

The Rust ML ecosystem is young — `ndarray`, `linfa`, `burn`, `tch-rs`, and `tensors` are all under active development — but it is growing fast. Every Python library in ML has a Rust counterpart, and the Rust versions are faster and safer by construction.

## The catch

Rust has a steeper learning curve. Ownership and borrowing feel alien at first, and the compiler is uncompromising. But that friction is the point: the compiler is an assistant that catches your mistakes before they reach production. Once the borrow checker stops fighting you, you wonder how you ever wrote C++.

If you want to explore Rust without leaving the comfort of a notebook, that is exactly what [Mars](/) is for.