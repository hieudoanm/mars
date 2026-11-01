# Jupyter with evcxr (Rust)

## Setup

```bash
uv venv
uv pip install jupyterlab
```

## Install Rust kernel (if not already installed)

```bash
cargo install --locked evcxr_jupyter
evcxr_jupyter --install
jupyter kernelspec list
```

## Launch

```bash
source .venv/bin/activate
jupyter lab
```

Or with uv:

```bash
uv run jupyter lab
```

Open `rust_demo.ipynb` and select the Rust kernel.
