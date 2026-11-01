# Python Runtime

## Overview

`mars-runtime-python` spawns a persistent `python3 -u` driver subprocess that executes cells in a shared module namespace. Stdout/stderr stream live, the trailing expression is displayed with `_repr_*` support, matplotlib figures are captured as PNG, and top-level `await` is supported.

## Discovery

Python runtime discovers the interpreter via:

1. The `MARS_PYTHON` environment variable (takes precedence)
2. The `python3` command on PATH
3. The `python` command on PATH (fallback)

Verify installation:

```bash
python3 --version
# or
python --version
```

## Execution model

The Python runtime uses a persistent driver process that:

- Executes cells in a shared module namespace
- Preserves variables across cells
- Streams stdout/stderr live
- Captures matplotlib figures as PNG (Agg backend)
- Supports `_repr_html_`/`_repr_png_`/`_repr_jpeg_`/`_repr_svg_` for rich display
- Supports `display()` for publishing MIME bundles
- Supports `!command` shell escapes
- Supports `%pip` magic for package installation
- Supports top-level `await` for async code
- Answers `input()` through the kernel input channel

Timeout or cancellation kills the interpreter; the next cell starts a fresh session.

## Display support

### Trailing expression

The trailing visible expression is displayed automatically using `_repr_*` methods:

```python
# Cell 1
import pandas as pd
df = pd.DataFrame({"A": [1, 2, 3], "B": [4, 5, 6]})

# Cell 2
df  # Displays as HTML table via _repr_html_
```

### Rich display

Objects with `_repr_*` methods render appropriately:

- `_repr_html_()` → HTML output
- `_repr_png_()` → PNG image
- `_repr_jpeg_()` → JPEG image
- `_repr_svg_()` → SVG image

```python
from IPython.display import Markdown
Markdown("# Hello from **Mars**")  # Renders as HTML
```

### Matplotlib figures

Matplotlib figures are automatically captured as PNG images:

```python
import matplotlib.pyplot as plt

plt.plot([1, 2, 3, 4], [1, 4, 2, 3])
plt.title("Sample Plot")
# Figure captured automatically
```

### MIME bundles

Use `display()` to publish MIME bundles:

```python
from IPython.display import display, HTML, Math
display(HTML("<b>Bold text</b>"))
display(Math(r"E = mc^2"))
```

## Interactive input

`input()` is answered through the kernel input channel:

```python
name = input("Enter your name: ")
print(f"Hello, {name}!")
```

Piped stdin EOF raises `EOFError`.

## Shell escapes

### Command execution

```python
!ls -la
```

### Package installation

```python
%pip install numpy pandas matplotlib
```

## Top-level await

Async code works without explicit event loop:

```python
import asyncio

async def fetch_data():
    await asyncio.sleep(1)
    return [1, 2, 3]

data = await fetch_data()
```

## Error handling

stderr is a non-fatal text stream, so warnings do not fail cells:

```python
import warnings
warnings.warn("This is a warning")  # Does not fail the cell
```

## Dependency management

Python dependencies are not managed through notebook metadata. Use `%pip` to install packages:

```python
%pip install numpy
```

Or install packages in your system Python environment before running Mars.

## Limitations

- Timeout/cancellation kills the interpreter; the next cell starts a fresh session
- No automatic dependency management via notebook metadata
- `readline()`/`scan()` consume protocol lines and are unsupported
- `cat(..., file = stderr())` bypasses stream capture

## Configuration

### Interpreter override

```bash
export MARS_PYTHON=/path/to/python3
```

### Python packages

Install required packages in your Python environment:

```bash
pip install numpy pandas matplotlib ipython
```

## Architecture rules

See `crates/mars-runtime-python/AGENTS.md` for architecture rules and implementation details.
