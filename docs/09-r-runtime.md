# R Runtime

## Overview

`mars-runtime-r` spawns a persistent `Rscript --vanilla` driver subprocess that parses and evaluates cells in `.GlobalEnv`, so bindings persist across cells. Stdout is captured per cell, warnings and `message()` calls stream as stderr, the trailing visible expression is displayed via `print()`, and plots drawn on the graphics device are captured as PNG (or PDF fallback) images.

## Discovery

R runtime discovers the interpreter via:

1. The `MARS_RSCRIPT` environment variable (takes precedence)
2. The `Rscript` command on PATH
3. The `R` command on PATH (fallback)

Verify installation:

```bash
Rscript --version
# or
R --version
```

## Execution model

The R runtime uses a persistent driver process that:

- Parses and evaluates cells in `.GlobalEnv`
- Preserves bindings across cells through `.GlobalEnv`
- Captures stdout per cell
- Streams warnings and `message()` calls as stderr
- Displays the trailing visible expression via `print()`
- Captures plots drawn on the graphics device as PNG (or PDF fallback)
- Requires the `jsonlite` R package for data interchange

Timeout or cancellation kills the R process; the next cell starts a fresh session.

## Display support

### Trailing expression

The trailing visible expression is displayed automatically via `print()`:

```r
# Cell 1
x <- c(1, 2, 3, 4, 5)

# Cell 2
mean(x)  # Displays: 3
```

### Plot capture

Plots drawn on the graphics device are automatically captured as PNG images:

```r
library(ggplot2)
df <- data.frame(x = 1:10, y = (1:10)^2)
ggplot(df, aes(x, y)) + geom_point()
# Plot captured automatically
```

PDF fallback is used if PNG capture fails.

### Tabular data

Data frames display as structured output:

```r
df <- data.frame(
  name = c("Alice", "Bob", "Charlie"),
  age = c(25, 30, 35)
)
df  # Displays as table
```

## Global environment

Variables defined in cells persist in `.GlobalEnv`:

```r
# Cell 1
x <- 42

# Cell 2
y <- x + 10  # x is still available
```

## Error handling

Warnings and `message()` calls stream as stderr and do not fail cells:

```r
warning("This is a warning")  # Streams to stderr, does not fail
message("This is a message")  # Streams to stderr, does not fail
```

## Package requirements

The R runtime requires the `jsonlite` package for data interchange:

```r
install.packages("jsonlite")
```

Or install via system package manager on Linux:

```bash
# Ubuntu/Debian
sudo apt install r-cran-jsonlite

# Fedora
sudo dnf install R-jsonlite
```

## Limitations

- Timeout/cancellation kills the R process; the next cell starts a fresh session
- `readline()`/`scan()` are unsupported and consume protocol lines
- `cat(..., file = stderr())` bypasses stream capture
- Requires `jsonlite` package to be installed

## Configuration

### Interpreter override

```bash
export MARS_RSCRIPT=/path/to/Rscript
```

### R packages

Install required packages in your R environment:

```r
install.packages(c("jsonlite", "ggplot2", "dplyr"))
```

## Example notebook

```r
# Cell 1: Basic operations
x <- c(1, 2, 3, 4, 5)
y <- x^2

# Cell 2: Statistics
mean(y)
sd(y)

# Cell 3: Plotting
library(ggplot2)
df <- data.frame(x = x, y = y)
ggplot(df, aes(x, y)) + geom_line() + geom_point()
```

## Architecture rules

See `crates/mars-runtime-r/AGENTS.md` for architecture rules and implementation details.
