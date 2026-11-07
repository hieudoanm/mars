# Mars Examples

This directory contains example notebooks demonstrating various features of the Mars notebook environment across multiple languages and use cases.

## Overview

Mars examples are organized into two main categories:

- **`mars/`** — Native `.mars` format examples for Mars
- **`jupyter/`** — Jupyter `.ipynb` notebooks for compatibility testing and import/export examples

## Mars Examples (`mars/`)

Native Mars notebook examples demonstrating multi-language support and various features.

### Structure

```
mars/
├── rust/              # Rust notebook examples
│   ├── 01-getting-started/
│   ├── 02-machine-learning/
│   └── 03-plotting/
├── python/            # Python notebook examples
│   └── 01-getting-started/
├── r/                 # R notebook examples
│   └── 01-getting-started/
├── julia/             # Julia notebook examples
│   └── 01-getting-started/
├── cpp/               # C++ notebook examples
│   └── 01-getting-started/
├── run.sh             # Script to run all examples
└── README.md          # This file
```

### Available Examples

| Example                                          | Language | Description                                                                                                                                                                                  |
| ------------------------------------------------ | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust/01-getting-started/getting-started.mars`   | Rust     | Basic Rust tour: variables, types, operators, control flow, loops, functions, collections, structs, enums, traits, closures, Option/Result, summary statistics, and least-squares regression |
| `rust/02-machine-learning/machine-learning.mars` | Rust     | Machine learning workflow using Rust libraries (polars, ndarray, linfa) similar to Python's pandas, numpy, and scikit-learn                                                                  |
| `rust/03-plotting/plotting.mars`                 | Rust     | Vector plotting with plotters, rendered as SVG image output like matplotlib in Jupyter                                                                                                       |
| `python/01-getting-started/getting-started.mars` | Python   | Comprehensive Python tour: types, collections, control flow, functions, exceptions, classes, modules, and stdlib tools                                                                       |
| `r/01-getting-started/getting-started.mars`      | R        | Comprehensive R tour: vectors, factors, matrices, data frames, apply family, functions, environments, statistics, and packages                                                               |
| `julia/01-getting-started/getting-started.mars`  | Julia    | Comprehensive Julia tour: values, arrays, broadcasting, tuples, dicts, dispatch, structs, exceptions, and stdlib tools                                                                       |
| `cpp/01-getting-started/getting-started.mars`    | C++      | C++ basics: types, containers, classes, templates, exceptions, and STL tools                                                                                                                 |

### Running Mars Examples

#### Using the run script (recommended)

```bash
cd mars
./run.sh
```

This discovers every `.mars` notebook recursively, executes them in sorted order, and displays their outputs.

**Note:** The `r/` and `julia/` notebooks require the corresponding interpreters to be installed:

- R + `jsonlite` for R notebooks
- Julia for Julia notebooks
- C++ runtime is implemented for C++ notebooks

#### Running individually

```bash
cd mars
mars run rust/01-getting-started/getting-started.mars --report
```

Or without the `--report` flag for default output.

#### Regenerating exports

After editing an example, regenerate its export:

```bash
cd mars
mars export rust/01-getting-started/getting-started.mars md rust/01-getting-started/getting-started.md
```

### Requirements

- **Mars CLI** installed and available in PATH
- **Rust toolchain** (for Rust dependencies)
- **R + jsonlite** for R notebooks
- **Julia** for Julia notebooks
- **C++ compiler** (clang++ or g++) for C++ notebooks

To install the Mars CLI:

```bash
cd ../..
cargo install --path crates/mars-cli
```

Or use the installation script:

```bash
./scripts/install.sh --cli
```

### Language-Specific Requirements

#### Rust

- Rust 1.85 or later
- Cargo for dependency management

#### Python

- Python 3.8 or later
- Common packages: numpy, pandas, matplotlib (install via pip)

#### R

- R 4.0 or later
- jsonlite package: `install.packages("jsonlite")`

#### Julia

- Julia 1.6 or later
- Standard library packages

#### C++

- clang++ (preferred) or g++
- Conan or vcpkg for package management (optional)

## Jupyter Examples (`jupyter/`)

Jupyter `.ipynb` notebooks for testing Mars import/export compatibility and demonstrating how to migrate from Jupyter to Mars.

### Structure

```
jupyter/
├── python/            # Python Jupyter notebooks
│   ├── notebook.ipynb
│   ├── Makefile
│   ├── pyproject.toml
│   └── README.md
└── rust/              # Rust Jupyter notebooks (evcxr)
    ├── notebook.ipynb
    ├── Makefile
    ├── pyproject.toml
    └── README.md
```

### Available Examples

| Example                 | Language | Description                                                            |
| ----------------------- | -------- | ---------------------------------------------------------------------- |
| `python/notebook.ipynb` | Python   | Data science workflow with pandas, numpy, matplotlib, and scikit-learn |
| `rust/notebook.ipynb`   | Rust     | Rust Jupyter notebook using evcxr kernel (experimental)                |

### Running Jupyter Examples

See individual README files in each directory:

- [Python Jupyter example](./jupyter/python/README.md)
- [Rust Jupyter example](./jupyter/rust/README.md)

### Importing to Mars

Convert Jupyter notebooks to Mars format:

```bash
mars import jupyter/python/notebook.ipynb mars-python.mars
mars run mars-python.mars
```

### Exporting from Mars

Convert Mars notebooks to Jupyter format:

```bash
mars export mars/python/01-getting-started/getting-started.mars ipynb notebook.ipynb
```

## Common Features Across Examples

### Notebook Format

All Mars examples use the `.mars` format with:

- Versioned JSON structure
- Stable cell IDs
- Execution counts and outputs
- Rich MIME bundle support
- Notebook metadata (language, dependencies)

### Runtime Features

Examples demonstrate:

- Persistent variable state across cells
- Rich output rendering (HTML, images, LaTeX, tables, charts)
- Dependency management (Cargo for Rust, pip for Python, etc.)
- Error handling and diagnostics
- Execution timeouts and cancellation

### Export Formats

Many examples include generated exports:

- **HTML** — Standalone HTML document with KaTeX math and Vega-Lite charts
- **Markdown** — Portable Markdown with code fences and data-URI images
- **PDF** — (R examples) PDF via graphics device capture

## Contributing Examples

When adding new examples:

1. Create a new directory under the appropriate language folder
2. Use descriptive names (e.g., `04-data-visualization`)
3. Include a `.mars` notebook file
4. Generate HTML and Markdown exports
5. Update the run script if needed
6. Update this README with the new example

Example structure:

```
mars/rust/04-data-visualization/
├── data-visualization.mars
├── data-visualization.html
└── data-visualization.md
```

## Troubleshooting

### Mars CLI not found

```bash
# Install Mars CLI
cargo install --path crates/mars-cli

# Or add to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### Runtime not found

```bash
# Check Rust
rustc --version

# Check Python
python3 --version

# Check R
Rscript --version

# Check Julia
julia --version

# Check C++
clang++ --version  # or g++ --version
```

### Dependency errors

For Rust notebooks, Mars will automatically resolve dependencies. For other languages, install required packages:

```bash
# Python
pip install numpy pandas matplotlib

# R
R -e "install.packages('jsonlite')"

# Julia
julia -e "using Pkg; Pkg.add('DataFrames')"
```

## Related Documentation

- [Mars CLI documentation](../docs/12-cli.md)
- [Runtime documentation](../docs/06-runtime.md)
- [Notebook format specification](../docs/03-notebook-format.md)
- [Landing site examples](../packages/landing/md/docs/01-getting-started.md)
- [Development documentation](../docs/13-development.md)

## Desktop Application

For the best interactive notebook experience, use the Mars desktop application:

```bash
cd ../packages/desktop
npm install
npm run tauri dev
```

The desktop app provides features not available in the CLI:

- Interactive cell editing with syntax highlighting
- Multi-notebook workspace
- File explorer
- Variable inspector
- Problems panel
- Checkpoints and diff
- Autosave
- LSP integration (rust-analyzer)
