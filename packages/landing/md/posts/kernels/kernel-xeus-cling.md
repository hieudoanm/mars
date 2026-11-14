---
title: "xeus-cling: C++ Kernel for High-Performance Computing"
description: "Explore xeus-cling, the C++ kernel for Jupyter, and how it enables interactive C++ development in notebook environments."
author: "Mars Team"
date: "2026-11-14"
order: 48
tags: ["kernels", "xeus-cling", "cpp", "jupyter", "notebook", "mars"]
---

# Why xeus-cling? C++ Kernel for Jupyter

**xeus-cling** is a C++ kernel for Jupyter that enables interactive C++ development in notebook environments. It provides a way to write and execute C++ code in Jupyter notebooks, bringing C++'s performance and control to the interactive computing paradigm.

## What xeus-cling Offers

### Interactive C++ Execution
- **Live C++ code**: Write and execute C++ code interactively
- **Persistent state**: Variables persist across cells
- **Cling integration**: Built on top of Cling, the C++ interpreter
- **REPL-like experience**: Similar to C++ REPL but in notebook format

### Jupyter Integration
- **Standard Jupyter protocol**: Works with JupyterLab, Jupyter Notebook, VS Code
- **Rich outputs**: Display text, images, HTML, and other MIME types
- **Code completion**: Integration with Cling for completions
- **Debugging**: Support for C++ debugging tools

### C++-Specific Features
- **Modern C++**: Support for C++11, C++14, C++17, C++20
- **Template metaprogramming**: Leverage C++'s powerful template system
- **STL integration**: Access to Standard Template Library
- **Performance**: Native C++ performance without interpretation overhead

## xeus-cling vs Mars C++ Runtime

| Feature | xeus-cling | Mars C++ Runtime |
|---------|------------|------------------|
| Platform | Jupyter kernel | Native C++ runtime |
| Execution model | Interpreter-based (Cling) | Replay-based compilation |
| Performance | Good (interpreted) | Excellent (compiled) |
| Dependencies | Cling + Jupyter | Conan/vcpkg + clang++/g++ |
| Multi-language | C++ only | Rust, Python, R, Julia, C++ |
| Document format | .ipynb | .mars (native format) |
| Desktop app | No | Yes (Tauri-based) |
| CLI | Limited | Full CLI with automation |

## When to Use xeus-cling

- **Jupyter ecosystem**: Already using Jupyter for other languages
- **C++ experimentation**: Quick C++ prototyping in notebooks
- **Educational purposes**: Teaching C++ with notebook interface
- **Jupyter extensions**: Leverage existing Jupyter extensions

## When Mars Might Be Better

- **Performance-critical** C++ code requiring full compilation
- **Multi-language workflows** needing C++ + Rust + Python + R + Julia
- **Desktop application** with native performance
- **CLI automation** for reproducible workflows
- **Dependency management** with Conan/vcpkg

## Mars's C++ Runtime Advantages

Mars provides a more sophisticated C++ runtime:

- **Replay-based compilation**: Full C++ compilation for each cell
- **Compiler support**: clang++ (preferred) and g++ (fallback)
- **Dependency management**: Conan and vcpkg support
- **Header accumulation**: Accumulate headers across cells
- **Variable inspection**: View and manipulate C++ variables
- **Multi-language cells**: Mix C++ with other languages
- **Native performance**: Direct compilation without interpreter overhead

## The xeus-cling Experience

Using xeus-cling in Jupyter:

```cpp
// Cell 1: Define a function
#include <iostream>
#include <vector>

std::vector<int> fibonacci(int n) {
    std::vector<int> seq;
    int a = 0, b = 1;
    for (int i = 0; i < n; ++i) {
        seq.push_back(a);
        int temp = a + b;
        a = b;
        b = temp;
    }
    return seq;
}

// Cell 2: Use the function
auto result = fibonacci(10);
for (int num : result) {
    std::cout << num << " ";
}
```

The kernel maintains state between cells, allowing you to build up complex C++ programs interactively.

## Mars's C++ Experience

Using Mars's C++ runtime:

```cpp
// Cell 1: Same function
#include <iostream>
#include <vector>

std::vector<int> fibonacci(int n) {
    std::vector<int> seq;
    int a = 0, b = 1;
    for (int i = 0; i < n; ++i) {
        seq.push_back(a);
        int temp = a + b;
        a = b;
        b = temp;
    }
    return seq;
}

// Cell 2: Same usage, but with full compilation
auto result = fibonacci(10);
for (int num : result) {
    std::cout << num << " ";
}
```

Mars provides the same interactive experience but with full compilation, dependency management, and multi-language support.

## C++'s Unique Advantages

C++ offers several unique advantages for high-performance computing:

- **Performance**: Zero-cost abstractions and explicit control
- **Systems programming**: Foundation for operating systems and embedded systems
- **Library ecosystem**: Extensive libraries for scientific computing and ML
- **Memory control**: Explicit memory management for optimal performance
- **Backward compatibility**: Decades of legacy code and libraries

## The Future of C++ Notebooks

Both xeus-cling and Mars represent different approaches:

- **xeus-cling**: Integrates C++ into the Jupyter ecosystem via Cling interpreter
- **Mars**: Builds a multi-language notebook platform with C++ compilation

As C++ continues to be important for high-performance computing and systems programming, both approaches will evolve to meet different user needs.

## Get Started

- [Mars C++ Runtime](docs/11-cpp-runtime.md) - C++ execution documentation
- [Mars Desktop App](packages/desktop) - Native application
- [Mars CLI](docs/12-cli.md) - Command-line interface
- [Getting Started](docs/01-getting-started.md) - First notebook tutorial
