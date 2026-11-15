---
title: "matlab_kernel: MATLAB Kernel for Engineering Applications"
description: "Explore matlab_kernel, the MATLAB kernel for Jupyter, and how it enables interactive MATLAB development in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 50
tags: ["kernels", "matlab_kernel", "matlab", "jupyter", "notebook", "mars"]
---

# matlab_kernel: MATLAB Kernel for Jupyter

**matlab_kernel** is a MATLAB kernel for Jupyter that enables interactive MATLAB development in notebook environments. It provides a way to write and execute MATLAB code in Jupyter notebooks, bringing MATLAB's computational capabilities to the interactive paradigm.

## What matlab_kernel Offers

### Interactive MATLAB Execution
- **Live MATLAB code**: Write and execute MATLAB code interactively
- **Persistent state**: Variables persist across cells
- **Toolbox support**: Access to MATLAB toolboxes and functions
- **REPL-like experience**: Similar to MATLAB's command window but in notebook format

### Jupyter Integration
- **Standard Jupyter protocol**: Works with JupyterLab, Jupyter Notebook, VS Code
- **Rich outputs**: Display text, images, plots, and other MIME types
- **Code completion**: Integration with MATLAB's completion system
- **Debugging**: Support for MATLAB debugging tools

### MATLAB-Specific Features
- **Matrix operations**: First-class support for matrix computations
- **Plotting**: Integration with MATLAB's plotting capabilities
- **Toolboxes**: Access to specialized toolboxes (Signal Processing, Image Processing, etc.)
- **Simulink integration**: Potential for Simulink model interaction

## matlab_kernel vs Project Mars MATLAB Runtime

| Feature | matlab_kernel | Project Mars MATLAB Runtime |
|---------|---------------|---------------------|
| Platform | Jupyter kernel | Native MATLAB runtime |
| Execution model | REPL-based | Subprocess driver |
| Performance | Good | Excellent (native MATLAB) |
| Toolbox support | Full MATLAB toolboxes | Full MATLAB toolboxes |
| Multi-language | MATLAB only | Rust, Python, R, Julia, C++ |
| Document format | .ipynb | .mars (native format) |
| Desktop app | No | Yes (Tauri-based) |
| CLI | Limited | Full CLI with automation |

## When to Use matlab_kernel

- **Jupyter ecosystem**: Already using Jupyter for other languages
- **MATLAB experimentation**: Quick MATLAB prototyping in notebooks
- **Educational purposes**: Teaching MATLAB with notebook interface
- **Jupyter extensions**: Leverage existing Jupyter extensions
- **Signal processing**: Interactive signal processing workflows
- **Image processing**: Interactive image processing tasks

## When Project Mars Might Be Better

- **Multi-language workflows** needing MATLAB + Rust + Python + R + Julia + C++
- **Desktop application** with native performance
- **CLI automation** for reproducible workflows
- **Independent platform** not tied to Jupyter
- **Enhanced MATLAB integration** with better tooling

## Project Mars's MATLAB Runtime Advantages

Project Mars provides a more integrated MATLAB runtime:

- **Subprocess driver**: Direct MATLAB subprocess execution
- **Variable inspection**: View and manipulate MATLAB variables
- **Multi-language cells**: Mix MATLAB with other languages
- **Native performance**: Direct MATLAB execution without Jupyter overhead
- **Consistent UX**: Same editor and execution model across languages
- **Rich outputs**: Unified output types across all languages

## The matlab_kernel Experience

Using matlab_kernel in Jupyter:

```matlab
% Cell 1: Define a function
function result = fibonacci(n)
    if n <= 1
        result = n;
    else
        result = fibonacci(n-1) + fibonacci(n-2);
    end
end

% Cell 2: Use the function
fibonacci(10)
```

The kernel maintains state between cells, allowing you to build up complex MATLAB programs interactively.

## Project Mars's MATLAB Experience

Using Project Mars's MATLAB runtime:

```matlab
% Cell 1: Same function
function result = fibonacci(n)
    if n <= 1
        result = n;
    else
        result = fibonacci(n-1) + fibonacci(n-2);
    end
end

% Cell 2: Same usage, but with multi-language support
fibonacci(10)
```

Project Mars provides the same interactive experience but with additional features like multi-language support, variable inspection, and desktop application integration.

## MATLAB's Unique Advantages

MATLAB offers several unique advantages for technical computing:

- **Matrix operations**: First-class support for matrix and array computations
- **Toolboxes**: Extensive collection of specialized toolboxes
- **Plotting**: Powerful plotting and visualization capabilities
- **Simulink**: Model-based design and simulation
- **Industry standard**: Widely used in engineering and scientific computing
- **Real-time**: Support for real-time applications and hardware interfacing

## The Future of MATLAB Notebooks

Both matlab_kernel and Project Mars represent different approaches:

- **matlab_kernel**: Integrates MATLAB into the Jupyter ecosystem
- **Project Mars**: Builds a multi-language notebook platform with MATLAB support

As MATLAB continues to be important for engineering and scientific computing, both approaches will evolve to meet different user needs.

## Get Started

- [Project Mars MATLAB Runtime](docs/06-runtime.md) - MATLAB execution documentation
- [Project Mars Desktop App](packages/desktop) - Native application
- [Project Mars CLI](docs/12-cli.md) - Command-line interface
- [Getting Started](docs/01-getting-started.md) - First notebook tutorial
