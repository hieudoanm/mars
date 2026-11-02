# Jupyter Evcxr Example

This directory contains Jupyter notebooks demonstrating Rust data science workflows using evcxr.

## Notebooks

- **notebook.ipynb**: Basic machine learning workflow using Rust standard library and rand
  - Generates synthetic classification data
  - Trains a logistic regression model from scratch
  - Evaluates model performance
  - Displays confusion matrix and feature importance

## Development

This example uses the evcxr Jupyter kernel for Rust execution.

### Prerequisites

- Rust toolchain (install via rustup)
- Python 3.15+
- uv (Python package manager)

### Setup

```bash
# Install all dependencies (evcxr_jupyter, register kernel, Python packages)
make install

# Run the notebook interactively
make run
```

**Important Note:**
- `make run` - Opens Jupyter for interactive execution (recommended)
- `make execute` - **May not work reliably** with evcxr kernel (known limitation)
- `make execute-output` - **May not work reliably** with evcxr kernel (known limitation)

The evcxr Jupyter kernel is designed for interactive use and has known limitations with automated execution via `nbconvert`. For automated execution, use the Python notebooks in the `jupyter/` directory instead.

The `make install` command will:
1. Install `evcxr_jupyter` v0.21.0 via cargo
2. Register the Rust kernel with Jupyter
3. Install Python dependencies (jupyter) using uv

### Manual Setup

If you prefer to set up manually:

```bash
# Install evcxr_jupyter (version 0.21.0 required for compatibility)
cargo install evcxr_jupyter --version 0.21.0

# Register the Rust kernel with Jupyter
evcxr_jupyter --install

# Install Python dependencies
uv sync

# Run the notebook
.venv/bin/jupyter notebook notebook.ipynb
```

## Libraries Used

This notebook demonstrates ML concepts using basic Rust types:
- **rand v0.8** - Random number generation for synthetic data
- **Rust standard library** - Vec, tuples, arrays for data structures
- **Custom implementation** - Logistic regression from scratch

### Why Not Use Full ML Libraries?

Due to evcxr Jupyter kernel limitations:
- Complex dependencies (polars, ndarray, linfa) have compilation issues with evcxr
- Type inference requires explicit annotations, making complex libraries difficult to use
- State management across cells is limited in evcxr
- This notebook focuses on demonstrating ML concepts with minimal dependencies

For production ML work in Rust, consider using:
- [polars](https://github.com/pola-rs/polars) - DataFrames (when not using evcxr)
- [ndarray](https://github.com/rust-ndarray/ndarray) - N-dimensional arrays
- [linfa](https://github.com/rust-ml/linfa) - Machine learning algorithms
- [plotters](https://github.com/plotters-rs/plotters) - Charting library

## Key Differences from Python

1. **Compilation**: Rust code is compiled before execution, providing type safety and performance
2. **Static Typing**: All types must be explicitly specified or inferred (evcxr often requires explicit annotations)
3. **Memory Safety**: Rust's ownership system prevents memory leaks and data races
4. **Ecosystem**: The Rust ML ecosystem is younger than Python's, with fewer specialized libraries
5. **Execution Model**: evcxr compiles and executes each cell, but has limitations with state persistence and complex dependencies

## Future Enhancements

Potential improvements for this notebook:
- Add more comprehensive evaluation metrics (precision, recall, F1-score)
- Implement feature scaling/normalization
- Add cross-validation
- Support for more complex models
- When not using evcxr, integrate with Polars DataFrames and Linfa for production workflows
