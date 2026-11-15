# Julia Runtime

## Overview

`mars-runtime-julia` spawns a persistent `julia` driver subprocess that parses cells with `Meta.parseall` and evaluates them in a dedicated `MarsKernel` module, so bindings persist across cells. Stdout/stderr stream live, the trailing visible expression is displayed via MIME reprs, `display()` publishes mime bundles, `readline()` is answered through the kernel input channel, and `ans`/`ans_prev` are maintained.

## Discovery

Julia runtime discovers the interpreter via:

1. The `MARS_JULIA` environment variable (takes precedence)
2. The `julia` command on PATH

Verify installation:

```bash
julia --version
```

## Execution model

The Julia runtime uses a persistent driver process that:

- Parses cells with `Meta.parseall`
- Evaluates cells in a dedicated `MarsKernel` module
- Preserves bindings across cells through the module
- Streams stdout/stderr live
- Displays the trailing visible expression via MIME reprs
- Supports `text/html`/`image/*`/`application/pdf`/`text/latex` plus `text/plain` for rich display
- Supports `display()` for publishing MIME bundles
- Answers `readline()` through the kernel input channel
- Maintains `ans` and `ans_prev` for last results
- Embeds a minimal JSON parser/writer (no Julia packages required)
- Supports `inspect`/`check` requests for variable inspection and diagnostics

Timeout or cancellation kills the Julia process; the next cell starts a fresh session.

## Display support

### Trailing expression

The trailing visible expression is displayed automatically via MIME reprs:

```julia
# Cell 1
x = [1, 2, 3, 4, 5]

# Cell 2
x  # Displays as array
```

### Rich display

Objects with MIME reprs render appropriately:

```julia
using Markdown
Markdown.parse("# Hello from **Project Mars**")  # Renders as HTML
```

### MIME bundles

Use `display()` to publish MIME bundles:

```julia
display("text/html", "<b>Bold text</b>")
display(MIME("text/latex"), L"E = mc^2")
```

### Plotting

Plotting libraries that support MIME output work automatically:

```julia
using Plots
plot(1:10, (1:10).^2)
# Plot captured via MIME
```

## Interactive input

`readline()` is answered through the kernel input channel:

```julia
name = readline(stdin)
println("Hello, $name!")
```

## Result variables

The runtime maintains:

- `ans` — Last result
- `ans_prev` — Previous result

```julia
1 + 2  # ans = 3
3 + 4  # ans = 7, ans_prev = 3
```

## Module isolation

Cells are evaluated in the `MarsKernel` module:

```julia
# Cell 1
MarsKernel.x = 42

# Cell 2
MarsKernel.x  # Accessible via module
```

Variables can be accessed without the module prefix from within cells.

## Dependency management

Julia dependencies are not managed through notebook metadata. Use `Pkg` to install packages:

```julia
using Pkg
Pkg.add("DataFrames")
Pkg.add("Plots")
```

Or install packages in your Julia environment before running Project Mars.

## JSON support

The driver embeds a minimal JSON parser/writer, so no Julia packages are required for kernel communication.

## Limitations

- Timeout/cancellation kills the Julia process; the next cell starts a fresh session
- No automatic dependency management via notebook metadata
- Module-based isolation may require understanding of module scoping

## Configuration

### Interpreter override

```bash
export MARS_JULIA=/path/to/julia
```

### Julia packages

Install required packages in your Julia environment:

```julia
using Pkg
Pkg.add(["DataFrames", "Plots", "JSON"])
```

## Example notebook

```julia
# Cell 1: Basic operations
x = 1:10
y = x.^2

# Cell 2: Statistics
using Statistics
mean(y)
std(y)

# Cell 3: Plotting
using Plots
plot(x, y, label="x²")
```

## Architecture rules

See `crates/mars-runtime-julia/AGENTS.md` for architecture rules and implementation details.
