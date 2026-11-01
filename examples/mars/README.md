# Mars Examples

This directory contains example Mars notebooks demonstrating various features of the Mars notebook environment.

## Examples

Each example lives in its own directory alongside a generated Markdown export:

| File                                        | Description                                                                                                                                                                                              |
| ------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `01-getting-started/getting-started.mars`   | A single tour of basic Rust: variables, types, operators, control flow, loops, functions, collections, structs, enums, traits, closures, Option/Result, summary statistics, and least-squares regression |
| `02-machine-learning/machine-learning.mars` | Machine learning workflow using Rust libraries (polars, ndarray, linfa) similar to Python's pandas, numpy, and scikit-learn                                                                              |
| `03-plotting/plotting.mars`                 | Vector plotting with plotters, rendered as an SVG image output like matplotlib in Jupyter                                                                                                                |

## Running the Examples

### Using the run script (recommended)

```bash
./run.sh
```

This discovers every `.mars` notebook recursively, executes them in sorted
order, and displays their outputs.

### Running individually

```bash
mars run 01-getting-started/getting-started.mars --report
```

Or without the `--report` flag for default output.

The `01-getting-started` notebook merges the former `02-data-analysis` and `03-linear-regression` examples into a single tour that shows as much basic Rust as possible.

Regenerate an example's Markdown export after editing it:

```bash
mars export 01-getting-started/getting-started.mars md
```

## Requirements

- [Mars CLI](../../crates/mars-cli) installed and available in PATH
- Rust toolchain (for dependencies)

To install the Mars CLI:

```bash
cd ../..
./scripts/install.sh --cli
```

Alternatively, install it directly with Cargo:

```bash
cargo install --path crates/mars-cli
```

## Notes

- Example 04 shows a machine learning workflow using Rust ecosystem libraries (polars, ndarray, linfa) equivalent to Python libraries (pandas, numpy, scikit-learn, matplotlib)
- Some examples use only the standard library, while others demonstrate patterns for using external dependencies
