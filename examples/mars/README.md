# Project Mars Examples

This directory contains example Project Mars notebooks demonstrating various features of the Project Mars notebook environment.

## Examples

Each example lives in its own directory alongside generated exports. The `rust/` examples produce Markdown, PDF, and HTML exports; the `r/` and `julia/` examples are single notebooks.

| File                                              | Language | Description                                                                                                                                                                                              |
| ------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust/01-getting-started/getting-started.mars`    | Rust     | A single tour of basic Rust: variables, types, operators, control flow, loops, functions, collections, structs, enums, traits, closures, Option/Result, summary statistics, and least-squares regression |
| `rust/02-machine-learning/machine-learning.mars`  | Rust     | Machine learning workflow using Rust libraries (polars, ndarray, linfa) similar to Python's pandas, numpy, and scikit-learn                                                                              |
| `rust/03-plotting/plotting.mars`                  | Rust     | Vector plotting with plotters, rendered as an SVG image output like matplotlib in Jupyter                                                                                                                |
| `r/r-intro.mars`                                  | R        | R basics: vectors, data frames, and plotting in a persistent `.GlobalEnv` (requires R + jsonlite)                                                                                                        |
| `julia/julia-intro.mars`                          | Julia    | Julia basics: vectors, functions, and MIME display output in a persistent module (requires Julia)                                                                                                         |
| `python/01-getting-started/getting-started.mars`  | Python   | Comprehensive Python tour: types, collections, control flow, functions, exceptions, classes, modules, and stdlib tools                                                                                    |
| `julia/01-getting-started/getting-started.mars`   | Julia    | Comprehensive Julia tour: values, arrays, broadcasting, tuples, dicts, dispatch, structs, exceptions, and stdlib tools                                                                                     |
| `r/01-getting-started/getting-started.mars`       | R        | Comprehensive R tour: vectors, factors, matrices, data frames, apply family, functions, environments, statistics, and packages                                                                            |
| `cpp/01-getting-started/getting-started.mars`     | C++      | C++ curriculum documentation — **not yet executable** (no C++ runtime backend); covers types, containers, classes, templates, exceptions, and STL tools                                                    |

## Running the Examples

### Using the run script (recommended)

```bash
./run.sh
```

This discovers every `.mars` notebook recursively, executes them in sorted
order, and displays their outputs. Note that the `r/` and `julia/`
notebooks require the corresponding interpreters (`Rscript` with
`jsonlite`, and `julia`) to be installed, or the script will fail on those
files. The `cpp/` notebook is documentation only and will fail until a C++
runtime backend exists.

### Running individually

```bash
mars run rust/01-getting-started/getting-started.mars --report
```

Or without the `--report` flag for default output.

The `01-getting-started` notebook merges the former `02-data-analysis` and `03-linear-regression` examples into a single tour that shows as much basic Rust as possible.

Regenerate an example's Markdown export after editing it:

```bash
mars export rust/01-getting-started/getting-started.mars md rust/01-getting-started/getting-started.md
```

## Requirements

- [Project Mars CLI](../../crates/mars-cli) installed and available in PATH
- Rust toolchain (for dependencies)
- R + `jsonlite` for the `r/` notebooks, Julia for the `julia/` notebooks
- The `cpp/` notebook is documentation only — Project Mars has no C++ runtime yet

To install the Project Mars CLI:

```bash
cd ../..
./scripts/install.sh --cli
```

Alternatively, install it directly with Cargo:

```bash
cargo install --path crates/mars-cli
```

## Notes

- The `rust/02-machine-learning` example shows a machine learning workflow using Rust ecosystem libraries (polars, ndarray, linfa) equivalent to Python libraries (pandas, numpy, scikit-learn, matplotlib)
- Some examples use only the standard library, while others demonstrate patterns for using external dependencies
