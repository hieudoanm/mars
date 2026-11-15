---
title: R Runtime
description: Project Mars R runtime documentation
order: 6
author: Project Mars Team
tags: [runtime, r, execution, data-science]
---

# R Runtime

Project Mars provides an R execution runtime that maintains a persistent R subprocess
speaking newline-delimited JSON. Variables persist in `.GlobalEnv` across cells,
enabling interactive R development in notebooks.

## Overview

The R runtime (`mars-runtime-r`) implements the `RuntimeBackend` trait with a
long-lived driver subprocess:

- Cells evaluate in `.GlobalEnv` so bindings persist
- Captures plots as PNG (or PDF fallback) via graphics devices
- Streams warnings and messages to stderr
- Requires the `jsonlite` R package for the driver protocol
- Trailing visible expression is displayed via `print()`

## Installation

R must be installed and available on your system:

```bash
# Install R from https://cran.r-project.org/
# Or use your system package manager
```

Project Mars discovers the R interpreter via:

1. The `MARS_RSCRIPT` environment variable (takes precedence)
2. The `Rscript` command on PATH

Verify installation:

```bash
Rscript --version
```

## Creating an R Notebook

```bash
mars notebook new analysis.mars r
```

Or set the language in notebook metadata:

```json
{
  "metadata": {
    "language": "r"
  }
}
```

## R Driver

The runtime uses an embedded R driver (`driver.R`) that:

- Speaks newline-delimited JSON over stdin/stdout
- Evaluates cells in `.GlobalEnv` for binding persistence
- Captures plots via graphics devices
- Streams warnings and messages to stderr
- Supports `inspect` and `check` requests for variable inspection and diagnostics

## Execution Model

### Persistent Session

The R driver runs as a long-lived subprocess:

```r
# Cell 1
x <- 42

# Cell 2
x + 10  # Returns 52, x is still available
```

Variables defined in earlier cells remain available in later cells through
`.GlobalEnv`.

### Display

The trailing visible expression is displayed automatically:

```r
x <- c(1, 2, 3, 4, 5)
sum(x)  # Displays: 15
```

### Plot Capture

Each cell evaluates with a graphics device open. The driver detects whether
anything was drawn and emits the device file as an image output:

```r
plot(1:10, 1:10)
```

Requires the `jsonlite` R package for the driver protocol. Install it:

```r
install.packages("jsonlite")
```

## Rich Output

R cells can produce rich output through:

### Plots

```r
plot(1:10, type = "l", col = "blue")
```

Plots are captured as PNG when available, with PDF fallback.

### Tables

```r
df <- data.frame(x = 1:5, y = letters[1:5])
df
```

Tables are displayed with formatting.

### HTML output

```r
cat("<div>Hello</div>")
```

## Known Limitations

- **Interactive input**: `readline()` and `scan()` are unsupported and consume
  protocol lines
- **Stderr bypass**: `cat(..., file = stderr())` bypasses stream capture
- **Timeout/cancel**: A timed-out or cancelled cell kills the R process; the next
  cell starts fresh and loses all state
- **Package installation**: No per-notebook R environment or dependency pinning
- **Device support**: Plot capture depends on graphics device display-list support

## R Data Science Ecosystem

R has a rich data science ecosystem that works well in Project Mars notebooks:

| Domain | R Packages |
| --- | --- |
| Data manipulation | dplyr, tidyr, data.table |
| Machine learning | caret, randomForest, xgboost |
| Plotting | ggplot2, lattice, plotly |
| Statistical modeling | stats, lme4, survival |
| Web scraping | rvest, httr |

## Comparison with Python/Julia

| Feature | R | Python | Julia |
| --- | --- | --- | --- |
| Execution model | Persistent driver | Persistent driver | Persistent driver |
| Plot capture | graphics device | matplotlib | MIME bundles |
| Interactive input | Not supported | `input()` | `readline()` |
| Dependency pinning | Not supported | `%pip` | Not supported |
| Timeout behavior | Kills interpreter | Kills interpreter | Kills driver |

## Troubleshooting

### R not found

```
no R interpreter found; install R or set MARS_RSCRIPT
```

Solution: Install R or set the `MARS_RSCRIPT` environment variable:

```bash
export MARS_RSCRIPT=/path/to/Rscript
```

### jsonlite not found

```
Error in library(jsonlite) : there is no package called 'jsonlite'
```

Solution: Install the required package:

```r
install.packages("jsonlite")
```

### Session lost after timeout

If a cell times out or is cancelled, the R process is killed and the next cell
starts with a fresh session, losing all previous state. This is a known
limitation of the current implementation.

### Package not found

Install packages directly in R cells:

```r
install.packages("dplyr")
```

Note: Packages install into the system R environment, not per-notebook.

## Desktop Support

The desktop application supports R cells with:

- R syntax highlighting via CodeMirror (using `@codemirror/legacy-modes/mode/r`)
- Full kernel integration (execution, variables, diagnostics)
- Rich output rendering (images, tables)
- Bracket colorizer, sticky scroll, and indentation markers

## VS Code Support

The VS Code extension currently supports Rust execution only. R notebooks
should use the desktop application for full functionality.

## Example Notebook

An R introduction example is available at `examples/mars/r/r-intro.mars`:

```bash
mars notebook run examples/mars/r/r-intro.mars --report
```
