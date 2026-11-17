---
title: 'C++: High-Performance Computing for Scientific Applications'
description: 'C++ was born for systems programming. It is still the default when performance, control, and zero-cost abstractions matter. Here is why it endures.'
author: 'Project Mars Team'
date: '2026-11-07'
order: 6
tags: ['cpp', 'performance', 'systems', 'history', 'mars']
---

# Why C++?

C++ was designed in 1979 as "C with Classes" — a pragmatic evolution of C that added object-oriented features without abandoning C's low-level control. It became the default language for systems programming, high-performance computing, and embedded systems. The reasons say a lot about what happens when you optimize a language for zero-cost abstractions and direct hardware access.

## Set up the C++ runtime

C++ cells need a C++17-capable compiler — `clang++` (preferred) or `g++`:

```bash
xcode-select --install                    # macOS
brew install llvm && brew link --force llvm

sudo apt install clang                    # Debian / Ubuntu (provides clang++)
# or: sudo apt install g++
sudo dnf install clang                    # Fedora
sudo pacman -S clang                      # Arch Linux
winget install LLVM.LLVM                  # Windows
```

Verify with `clang++ --version` (or `g++ --version`), or run `mars notebook doctor`. C++ dependencies come from Conan or vcpkg — see the [C++ runtime docs](/docs/runtime-cpp).

## Performance by design

C++'s most famous design principle is "you don't pay for what you don't use, and you don't pay for what you do use indirectly." A virtual function call costs only when you actually declare a function virtual. A template generates no runtime overhead. Memory layout is explicit and controllable:

```cpp
struct Point {
    double x, y;
    double distance() const {
        return std::sqrt(x * x + y * y);
    }
};

std::vector<Point> points = /* ... */;
std::sort(points.begin(), points.end(),
    [](const Point& a, const Point& b) {
        return a.distance() < b.distance();
    });
```

The compiler inlines the lambda, vector operations are bounds-checked only in debug builds, and the entire loop runs at native speed. For workloads where every nanosecond matters — high-frequency trading, game engines, real-time systems — that control is not optional.

## The ecosystem grew up around it

A language succeeds when its libraries meet real needs, and C++'s ecosystem reflects its domain:

- **STL** (Standard Template Library) proved generic programming was practical for containers and algorithms.
- **Boost** filled gaps before the committee could, and many Boost libraries eventually entered the standard.
- **Eigen, Armadillo, Blaze** made linear algebra competitive with Fortran.
- **mlpack, LightGBM, XGBoost** provided high-performance machine learning backends.
- **OpenCV, TensorFlow, PyTorch** ship C++ APIs under the hood.

The pattern is consistent: C++ is where performance-critical code lives, even when the public interface is Python or JavaScript. You use C++ when the abstraction layer adds measurable overhead.

## What C++ costs

The same properties that make C++ powerful create real structural costs:

- **Compilation time.** Templates are Turing-complete at compile time, so complex generic code can take minutes to compile. Header-only libraries exacerbate this — every translation unit re-parses the same headers.
- **Memory management.** You are responsible for allocation and deallocation. Modern C++ added smart pointers (`std::unique_ptr`, `std::shared_ptr`), but you still need to understand ownership semantics and avoid cycles.
- **Complexity.** The language has accumulated features over four decades. Move semantics, constexpr, concepts, coroutines — each addition solves a real problem, but the surface area is enormous. A new developer faces a steep learning curve.
- **Error handling.** Exceptions are the standard, but they impose runtime overhead and are often disabled in performance-critical code. Return codes and error enums are explicit but verbose. Resource leaks are still possible if an exception is thrown mid-construction.

None of this made C++ a bad choice — it made it a _specific_ choice, optimized for control and performance rather than approachability or rapid prototyping.

## The modern C++ renaissance

C++11, C++14, C++17, C++20, and C++23 modernized the language significantly:

- **Smart pointers** made manual `delete` rare.
- **`auto` type deduction** reduced verbosity.
- **Lambda expressions** made functional-style code readable.
- **Range-based for loops** eliminated iterator boilerplate.
- **`std::filesystem`** unified cross-platform file operations.
- **Concepts** constrained template arguments.
- **Modules** (in progress) promise to reduce compilation time.

You can write modern C++ that feels less like 1990s code and more like a contemporary language — but the complexity is still there, and the compiler is still demanding.

## Try it in Project Mars

C++ cells run on a **replay runtime**: Mars compiles each cell with your system compiler — `clang++` or `g++` — and streams diagnostics back onto the cell. Earlier cells are replayed for shared state, so bindings declared in one cell are visible in the next.

```cpp
#include <vector>

// This binding is part of the replayed prefix — later cells see it.
std::vector<double> samples = {5.1, 4.9, 4.7, 4.6, 5.0};
```

```cpp
#include <iostream>
#include <numeric>

// Replay re-evaluates earlier cells, so `samples` is in scope here.
double mean = std::accumulate(samples.begin(), samples.end(), 0.0) / samples.size();
std::cout << "mean sepal length: " << mean << " cm" << std::endl;
```

## Where Project Mars fits

Project Mars does not ask you to leave C++ behind. It provides a notebook where C++ code runs interactively with automatic compilation and isolated execution. What Project Mars offers is a notebook where you can:

- Prototype high-performance algorithms without leaving the notebook
- Use Conan or vcpkg for dependency management with reproducible lockfiles
- Get real-time compiler diagnostics mapped to notebook cells
- Combine C++ performance with Project Mars's rich output rendering

C++ earned its place as the backbone of performance-critical software. Project Mars is for the parts of your workflow where you want to write that backbone interactively — without giving up the language's control or its zero-cost abstractions.
