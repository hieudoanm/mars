---
title: Python Runtime
description: Project Mars Python runtime documentation
order: 6
author: Project Mars Team
tags: [runtime, python, execution, data-science]
---

# Python Runtime

Project Mars provides a Python execution runtime that maintains a persistent Python
subprocess speaking newline-delimited JSON. Variables persist in the driver's
module namespace across cells, enabling interactive Python development in
notebooks.

## Overview

The Python runtime (`mars-runtime-python`) implements the `RuntimeBackend` trait
with a long-lived driver subprocess:

- Variables persist in the driver's module namespace across cells
- Stdout/stderr stream live during execution
- Supports top-level `await` via `PyCF_ALLOW_TOP_LEVEL_AWAIT`
- Captures matplotlib figures as PNG when available
- Implements `_repr_html_`, `_repr_png_`, `_repr_jpeg_`, `_repr_svg_` display methods
- Provides `Out` and `_` bindings for the last displayed expression
- Supports `input()` for interactive prompts

## Installation

Python 3 must be installed and available on your system:

```bash
# Python is typically pre-installed on most systems
python3 --version
```

Project Mars discovers the Python interpreter via:

1. The `MARS_PYTHON` environment variable (takes precedence)
2. The `python3` command on PATH

## Creating a Python Notebook

```bash
mars notebook new analysis.mars python
```

Or set the language in notebook metadata:

```json
{
  "metadata": {
    "language": "python"
  }
}
```

## Python Driver

The runtime uses an embedded Python driver (`driver.py`) that:

- Implements a MarsKernel module for state management
- Speaks newline-delimited JSON over stdin/stdout
- Handles execution, display, inspection, and input requests
- Supports top-level `await` through `PyCF_ALLOW_TOP_LEVEL_AWAIT`

## Execution Model

### Persistent Session

The Python driver runs as a long-lived subprocess:

```python
# Cell 1
x = 42

# Cell 2
x + 10  # Returns 52, x is still available
```

Variables defined in earlier cells remain available in later cells.

### Top-level await

The driver supports top-level `await` without explicit async functions:

```python
import asyncio
await asyncio.sleep(1)
print("Done")
```

### Display

The trailing expression's value is displayed automatically:

```python
x = [1, 2, 3, 4, 5]
sum(x)  # Displays: 15
```

Python's `_repr_*` methods are supported for rich output:
- `_repr_html_` → HTML output
- `_repr_png_` → PNG images
- `_repr_jpeg_` → JPEG images
- `_repr_svg_` → SVG graphics

### Interactive Input

The driver supports `input()` prompts:

```python
name = input("Enter your name: ")
print(f"Hello, {name}!")
```

Prompts are handled through the kernel's `input` request in the desktop and
`mars kernel serve`.

## Rich Output

Python cells can produce rich output through:

### Matplotlib

```python
import matplotlib.pyplot as plt
plt.plot([1, 2, 3, 4])
plt.show()  # Displays as image/png
```

Project Mars sets `MPLBACKEND=Agg` for matplotlib figure capture unless already set.

### IPython display

```python
from IPython.display import HTML, display
display(HTML("<div>Hello</div>"))
```

### MIME bundles

```python
from IPython.display import JSON
JSON({"key": "value"})
```

## Known Limitations

- **Timeout/cancel**: A timed-out or cancelled cell kills the interpreter; the next
  cell starts a fresh Python session and loses all state
- **Package installation**: No per-notebook Python environment or dependency
  pinning; `%pip` installs into the selected interpreter
- **Process isolation**: The driver runs in a subprocess but shares the system
  Python installation

## Python Data Science Ecosystem

Python has a rich data science ecosystem that works well in Project Mars notebooks:

| Domain | Python Packages |
| --- | --- |
| Data manipulation | pandas, numpy, polars |
| Machine learning | scikit-learn, PyTorch, TensorFlow |
| Plotting | matplotlib, seaborn, plotly |
| Scientific computing | scipy, numba |
| Web scraping | requests, beautifulsoup4 |

## Comparison with R/Julia

| Feature | Python | R | Julia |
| --- | --- | --- | --- |
| Execution model | Persistent driver | Persistent driver | Persistent driver |
| Plot capture | matplotlib | graphics device | MIME bundles |
| Interactive input | `input()` | Not supported | `readline()` |
| Dependency pinning | `%pip` | Not supported | Not supported |
| Timeout behavior | Kills interpreter | Kills interpreter | Kills driver |

## Troubleshooting

### Python not found

```
no Python interpreter found; install Python or set MARS_PYTHON
```

Solution: Install Python 3 or set the `MARS_PYTHON` environment variable:

```bash
export MARS_PYTHON=/path/to/python3
```

### Session lost after timeout

If a cell times out or is cancelled, the Python interpreter is killed and the
next cell starts with a fresh session, losing all previous state. This is a
known limitation of the current implementation.

### Package not found

Install packages directly in Python cells:

```python
%pip install pandas numpy matplotlib
```

Note: Packages install into the system Python environment, not per-notebook.

## Desktop Support

The desktop application supports Python cells with:

- Python syntax highlighting via CodeMirror
- Full kernel integration (execution, variables, diagnostics)
- Rich output rendering (HTML, images, plots)
- Input prompt dialogs for `input()`

## VS Code Support

The VS Code extension currently supports Rust execution only. Python notebooks
should use the desktop application for full functionality.
