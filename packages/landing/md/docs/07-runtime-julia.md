---
title: Julia Runtime
description: Project Mars Julia runtime documentation
order: 7
author: Project Mars Team
tags: [runtime, julia, execution, data-science]
---

# Julia Runtime

Project Mars provides a Julia execution runtime that maintains a persistent Julia
subprocess speaking newline-delimited JSON. Variables persist in the driver's
`MarsKernel` module across cells, enabling interactive Julia development in
notebooks.

## Overview

The Julia runtime (`mars-runtime-julia`) implements the `RuntimeBackend` trait
with a long-lived driver subprocess:

- Variables persist in the driver's `MarsKernel` module across cells
- Supports interactive `readline()` prompts through the kernel input channel
- Implements `_repr_*` methods for rich output (HTML, PNG, SVG, etc.)
- Supports MIME bundle output for Julia plotting libraries
- Requires a `julia` executable on PATH or `MARS_JULIA` environment variable

## Installation

Julia must be installed and available on your system:

```bash
# Install Julia from https://julialang.org/downloads/
# Or use your system package manager
```

Project Mars discovers the Julia interpreter via:

1. The `MARS_JULIA` environment variable (takes precedence)
2. The `julia` command on PATH

Verify installation:

```bash
julia --version
```

## Creating a Julia Notebook

```bash
mars notebook new analysis.mars julia
```

Or set the language in notebook metadata:

```json
{
  "metadata": {
    "language": "julia"
  }
}
```

## Julia Driver

The runtime uses an embedded Julia driver (`driver.jl`) that:

- Implements a `MarsKernel` module for state management
- Speaks newline-delimited JSON over stdin/stdout
- Handles execution, display, inspection, and input requests
- Supports MIME bundle output for rich rendering

## Execution Model

### Persistent Session

The Julia driver runs as a long-lived subprocess:

```julia
# Cell 1
x = 42

# Cell 2
x + 10  # Returns 52, x is still available
```

Variables defined in earlier cells remain available in later cells.

### Display

The trailing expression's value is displayed automatically:

```julia
x = [1, 2, 3, 4, 5]
sum(x)  # Displays: 15
```

Julia's `_repr_*` methods are supported for rich output:
- `_repr_html_` → HTML output
- `_repr_png_` → PNG images
- `_repr_svg_` → SVG graphics
- `_repr_mime_` → Arbitrary MIME bundles

### Interactive Input

The driver supports `readline()` prompts:

```julia
name = readline()  # Prompts via kernel input channel
println("Hello, $name!")
```

Prompts are handled through the kernel's `input` request in the desktop and
`mars kernel serve`.

## Rich Output

Julia cells can produce rich output through:

### MIME Bundles

```julia
# HTML output
display("text/html", "<div>Hello</div>")

# Image output
display("image/png", base64_data)

# SVG output
display("image/svg+xml", svg_string)
```

### Plots

Julia plotting libraries that output to MIME types work naturally:

```julia
using Plots
plot(rand(10))  # Displays as image/png or image/svg+xml
```

## Known Limitations

- **Timeout/cancel**: A timed-out or cancelled cell kills the driver; the next
  cell starts a fresh Julia session and loses all state
- **Direct stdin**: User code reading `stdin` directly consumes kernel protocol
  lines; use `readline()` through the driver instead
- **Package installation**: No per-notebook Julia environment or dependency
  pinning; packages install into the selected interpreter
- **Process isolation**: The driver runs in a subprocess but shares the system
  Julia installation

## Julia Data Science Ecosystem

Julia has a rich data science ecosystem that works well in Project Mars notebooks:

| Domain | Julia Packages |
| --- | --- |
| Data manipulation | DataFrames.jl, CSV.jl, JSON.jl |
| Numerical computing | LinearAlgebra, FFTW, Optim |
| Machine learning | Flux.jl, MLJ.jl, ScikitLearn.jl |
| Plotting | Plots.jl, Gadfly.jl, Makie.jl |
| Statistics | Statistics.jl, Distributions.jl |
| Scientific computing | SciML.jl ecosystem |

## Comparison with Python/R

| Feature | Julia | Python | R |
| --- | --- | --- | --- |
| Execution model | Persistent driver | Persistent driver | Persistent driver |
| Plot capture | MIME bundles | matplotlib | graphics device |
| Interactive input | `readline()` | `input()` | Not supported |
| Dependency pinning | Not supported | `%pip` | Not supported |
| Timeout behavior | Kills driver | Kills interpreter | Kills interpreter |

## Troubleshooting

### Julia not found

```
no Julia interpreter found; install Julia or set MARS_JULIA
```

Solution: Install Julia or set the `MARS_JULIA` environment variable:

```bash
export MARS_JULIA=/path/to/julia
```

### Session lost after timeout

If a cell times out or is cancelled, the Julia driver is killed and the next
cell starts with a fresh session, losing all previous state. This is a known
limitation of the current implementation.

### Package not found

Install packages directly in Julia cells:

```julia
using Pkg
Pkg.add("DataFrames")
```

Note: Packages install into the system Julia environment, not per-notebook.

## Desktop Support

The desktop application supports Julia cells with:

- Julia syntax highlighting via CodeMirror
- Full kernel integration (execution, variables, diagnostics)
- Rich output rendering (HTML, images, plots)
- Input prompt dialogs for `readline()`

## VS Code Support

The VS Code extension currently supports Rust execution only. Julia notebooks
should use the desktop application for full functionality.

## Example Notebook

A Julia introduction example is available at `examples/mars/julia/julia-intro.mars`:

```bash
mars notebook run examples/mars/julia/julia-intro.mars --report
```
