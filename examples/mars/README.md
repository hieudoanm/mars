# Mars Examples

This directory contains example Mars notebooks demonstrating various features of the Mars notebook environment.

## Examples

Each example lives in its own directory alongside a generated Markdown export:

| File                                          | Description                                                                                                                 |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `01-getting-started/getting-started.mars`     | Basic introduction to Mars - variables and expressions                                                                      |
| `02-data-analysis/data-analysis.mars`         | Simple data analysis with vectors, sums, and statistics                                                                     |
| `03-linear-regression/linear-regression.mars` | Computing least-squares linear regression from scratch                                                                      |
| `04-machine-learning/machine-learning.mars`   | Machine learning workflow using Rust libraries (polars, ndarray, linfa) similar to Python's pandas, numpy, and scikit-learn |
| `05-plotting/plotting.mars`                   | Vector plotting with plotters, rendered as an SVG image output like matplotlib in Jupyter                                    |

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
