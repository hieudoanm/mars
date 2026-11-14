---
title: "IRkernel: R Kernel for Statistical Computing"
description: "Explore IRkernel, the R kernel for Jupyter, and how it enables interactive R development in notebook environments."
author: "Mars Team"
date: "2026-11-14"
order: 49
tags: ["kernels", "irkernel", "r", "jupyter", "notebook", "mars"]
---

# Why IRkernel? R Kernel for Jupyter

**IRkernel** is the R kernel for Jupyter, enabling interactive R development in notebook environments. It provides a way to write and execute R code in Jupyter notebooks, bringing R's statistical computing capabilities to the interactive paradigm.

## What IRkernel Offers

### Interactive R Execution
- **Live R code**: Write and execute R code interactively
- **Persistent state**: Variables persist across cells in `.GlobalEnv`
- **Package management**: Use R's package manager directly in notebooks
- **REPL-like experience**: Similar to R's REPL but in notebook format

### Jupyter Integration
- **Standard Jupyter protocol**: Works with JupyterLab, Jupyter Notebook, VS Code
- **Rich outputs**: Display text, images, HTML, LaTeX, and other MIME types
- **Code completion**: Integration with R's completion system
- **Debugging**: Support for R's debugging tools

### R-Specific Features
- **Statistical computing**: First-class support for statistical analysis
- **Data visualization**: Integration with ggplot2 and other plotting libraries
- **Tidyverse**: Modern data manipulation and visualization ecosystem
- **Reproducible research**: Support for R Markdown and reproducible workflows

## IRkernel vs Mars R Runtime

| Feature | IRkernel | Mars R Runtime |
|---------|----------|----------------|
| Platform | Jupyter kernel | Native R runtime |
| Execution model | REPL-based | Subprocess driver |
| Performance | Good | Good |
| Package management | R packages | R packages |
| Multi-language | R only | Rust, Python, R, Julia, C++ |
| Document format | .ipynb | .mars (native format) |
| Desktop app | No | Yes (Tauri-based) |
| CLI | Limited | Full CLI with automation |

## When to Use IRkernel

- **Jupyter ecosystem**: Already using Jupyter for other languages
- **R experimentation**: Quick R prototyping in notebooks
- **Educational purposes**: Teaching R with notebook interface
- **Jupyter extensions**: Leverage existing Jupyter extensions
- **Statistical analysis**: Interactive statistical computing

## When Mars Might Be Better

- **Multi-language workflows** needing R + Rust + Python + Julia + C++
- **Desktop application** with native performance
- **CLI automation** for reproducible workflows
- **Independent platform** not tied to Jupyter
- **Enhanced R integration** with plot capture and warnings

## Mars's R Runtime Advantages

Mars provides a more integrated R runtime:

- **Subprocess driver**: Direct Rscript subprocess execution
- **`.GlobalEnv` persistence**: Variables persist across cells
- **Plot capture**: Automatic capture of R plots (PNG/PDF)
- **Warnings streaming**: Stream R warnings and messages
- **jsonlite integration**: JSON serialization for data exchange
- **Multi-language cells**: Mix R with other languages
- **Native performance**: Direct R execution without Jupyter overhead

## The IRkernel Experience

Using IRkernel in Jupyter:

```r
# Cell 1: Define a function
fibonacci <- function(n) {
  if (n <= 1) return(n)
  fibonacci(n - 1) + fibonacci(n - 2)
}

# Cell 2: Use the function
fibonacci(10)
```

The kernel maintains state between cells, allowing you to build up complex R programs interactively.

## Mars's R Experience

Using Mars's R runtime:

```r
# Cell 1: Same function
fibonacci <- function(n) {
  if (n <= 1) return(n)
  fibonacci(n - 1) + fibonacci(n - 2)
}

# Cell 2: Same usage, but with plot capture and warnings
fibonacci(10)
```

Mars provides the same interactive experience but with additional features like plot capture, warnings streaming, and multi-language support.

## R's Unique Advantages

R offers several unique advantages for statistical computing:

- **Statistical foundation**: Built specifically for statistical analysis
- **Tidyverse ecosystem**: Modern, consistent data manipulation and visualization
- **Research standard**: Widely used in academia and research institutions
- **Domain expertise**: Deep domain knowledge in statistics and biostatistics
- **Visualization**: Excellent plotting capabilities with ggplot2

## The Future of R Notebooks

Both IRkernel and Mars represent different approaches:

- **IRkernel**: Integrates R into the Jupyter ecosystem
- **Mars**: Builds a multi-language notebook platform with R support

As R continues to be important for statistical computing and data science, both approaches will evolve to meet different user needs.

## Get Started

- [Mars R Runtime](docs/09-r-runtime.md) - R execution documentation
- [Mars Desktop App](packages/desktop) - Native application
- [Mars CLI](docs/12-cli.md) - Command-line interface
- [Getting Started](docs/01-getting-started.md) - First notebook tutorial
