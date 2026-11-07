# Jupyter Examples

This directory contains Jupyter `.ipynb` notebooks for testing Mars import/export compatibility and demonstrating how to migrate from Jupyter to Mars.

## Overview

These examples serve two purposes:

1. **Compatibility testing** — Verify that Mars can correctly import and export Jupyter notebooks
2. **Migration reference** — Show how typical Jupyter workflows translate to Mars

## Available Examples

### Python Example

**Location:** `python/notebook.ipynb`

A comprehensive Python data science workflow demonstrating:

- Synthetic data generation for classification
- Data manipulation with pandas
- Numerical computing with numpy
- Machine learning with scikit-learn (logistic regression)
- Model evaluation (accuracy, precision, recall, F1-score)
- Visualization with matplotlib (confusion matrix, feature importance)

**Libraries used:**
- pandas — Data frames and manipulation
- numpy — Numerical computing
- matplotlib — Visualization
- scikit-learn — Machine learning

### Rust Example

**Location:** `rust/notebook.ipynb`

A Rust Jupyter notebook using the evcxr kernel demonstrating:

- Synthetic data generation with rand
- Logistic regression implementation from scratch
- Model training and evaluation
- Confusion matrix and feature importance

**Note:** This example uses the evcxr Jupyter kernel, which is experimental and has known limitations with automated execution.

## Running Jupyter Examples

### Python Example

```bash
cd python
make install
make run
```

Or execute and save outputs:

```bash
cd python
make execute
```

See [python/README.md](./python/README.md) for detailed instructions.

### Rust Example

```bash
cd rust
make install
make run
```

See [rust/README.md](./rust/README.md) for detailed instructions.

## Importing to Mars

Convert Jupyter notebooks to Mars format:

```bash
# Import Python notebook
mars import jupyter/python/notebook.ipynb mars-python.mars
mars run mars-python.mars

# Import Rust notebook
mars import jupyter/rust/notebook.ipynb mars-rust.mars
mars run mars-rust.mars
```

## Exporting from Mars

Convert Mars notebooks to Jupyter format:

```bash
# Export to Jupyter
mars export mars/python/01-getting-started/getting-started.mars ipynb notebook.ipynb
```

## Import/Export Compatibility

Mars supports Jupyter import/export with the following mappings:

| Jupyter Feature | Mars Equivalent |
| --- | --- |
| Code cells | Code cells |
| Markdown cells | Markdown cells |
| Raw cells | Raw cells |
| Outputs (text) | Text outputs |
| Outputs (error) | Error outputs |
| Outputs (HTML) | HTML outputs |
| Outputs (images) | Image outputs (PNG, JPEG, SVG) |
| Outputs (LaTeX) | LaTeX outputs |
| Execution counts | Execution counts |
| Cell metadata | Cell metadata (preserved) |
| Notebook metadata | Notebook metadata (preserved) |

## Limitations

- **Interactive widgets** — Not yet supported
- **Custom MIME types** — Some may not render correctly
- **Cell attachments** — Basic support only
- **Complex notebook metadata** — May be partially preserved

## Migration Guide

### From Jupyter to Mars

1. **Install Mars CLI**
   ```bash
   cargo install --path crates/mars-cli
   ```

2. **Import your notebook**
   ```bash
   mars import my-notebook.ipynb my-notebook.mars
   ```

3. **Run in Mars**
   ```bash
   mars run my-notebook.mars
   ```

4. **Open in desktop app** (recommended)
   ```bash
   mars open my-notebook.mars
   ```

### Key Differences

| Feature | Jupyter | Mars |
| --- | --- | --- |
| Native format | `.ipynb` | `.mars` |
| Primary language | Python | Rust (with multi-language support) |
| Dependency management | pip / conda | Cargo (Rust), pip (Python), etc. |
| Kernel persistence | Per-language | Per-language persistent sessions |
| VS Code support | Native | Via Mars extension |
| Desktop app | None | Native Tauri app |

## Python → Mars Translation

### Cell structure

**Jupyter:**
```python
# In [1]
import pandas as pd
df = pd.DataFrame({"A": [1, 2, 3]})
```

**Mars:**
```python
# Cell 1
import pandas as pd
df = pd.DataFrame({"A": [1, 2, 3]})
```

### Magic commands

**Jupyter:**
```python
%pip install numpy
!ls -la
```

**Mars:**
```python
%pip install numpy  # Supported
!ls -la              # Supported
```

### Display

**Jupyter:**
```python
from IPython.display import display, HTML
display(HTML("<b>Bold</b>"))
```

**Mars:**
```python
from IPython.display import display, HTML
display(HTML("<b>Bold</b>"))  # Same syntax
```

## Rust Evcxr Limitations

The Rust Jupyter example uses evcxr, which has known limitations:

- No automated execution via `nbconvert`
- State persistence issues in some scenarios
- Experimental and not production-ready

For production Rust notebooks, use the Mars desktop app or CLI with the native Rust runtime.

## Requirements

### For Jupyter notebooks

- Python 3.8 or later
- Jupyter: `pip install jupyter`
- Dependencies per example (see individual READMEs)

### For Mars import/export

- Mars CLI installed
- Runtime dependencies (Python, R, Julia, C++ as needed)

## Troubleshooting

### Import fails

```bash
# Check notebook is valid
jupyter nbconvert --to notebook --stdout notebook.ipynb > /dev/null

# Check Mars CLI is installed
mars --version
```

### Export fails

```bash
# Check notebook is valid Mars format
mars inspect my-notebook.mars
```

### Runtime not found

```bash
# Check Python runtime
python3 --version

# Check Rust toolchain
rustc --version
```

## Related Documentation

- [Mars CLI documentation](../../docs/12-cli.md)
- [Jupyter compatibility](../../docs/03-notebook-format.md)
- [Python runtime documentation](../../docs/08-python-runtime.md)
- [Rust runtime documentation](../../docs/07-rust-runtime.md)
- [Mars examples README](../README.md)

## Contributing

When adding new Jupyter examples:

1. Create a new directory for the language
2. Include a `.ipynb` notebook
3. Add a `Makefile` with install/run targets
4. Add a `README.md` with usage instructions
5. Update this README with the new example
6. Test import/export with Mars

Example structure:

```
jupyter/
├── python/
│   ├── notebook.ipynb
│   ├── Makefile
│   ├── pyproject.toml
│   └── README.md
└── new-language/
    ├── notebook.ipynb
    ├── Makefile
    └── README.md
```

## Desktop Application

For the best notebook experience, use the Mars desktop application:

```bash
cd ../../packages/desktop
npm install
npm run tauri dev
```

The desktop app provides features not available in Jupyter:
- Multi-language notebooks in a single file
- Rust-first architecture with performance benefits
- Native multi-notebook workspace
- LSP integration
- Checkpoints and diff
- Rich UI features
