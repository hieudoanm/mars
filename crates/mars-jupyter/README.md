# mars-jupyter

Jupyter `.ipynb` import and export for the Project Mars notebook platform.

## Purpose

`mars-jupyter` reads and writes Jupyter notebooks, mapping between Jupyter's cell/output model and Project Mars's native model. Information Project Mars cannot interpret is preserved where possible.

## Modules

- **`import`** — `import_ipynb`, `JupyterNotebook`, `JupyterCell`, `JupyterOutput`, language/metadata helpers.
- **`export`** — `export_ipynb`, `export_cell`, `export_output`.
- **`conversion`** — MIME bundle conversion, output conversion, HTML table escaping.
- **`error`** — `JupyterError`.

## Usage

```rust
use mars_jupyter::{import_ipynb, export_ipynb};

let notebook = import_ipynb(ipynb_json)?;
let exported = export_ipynb(&notebook)?;
```

## Notes

- Project Mars does not claim full Jupyter compatibility until tested against representative notebooks.
- Unsupported metadata is preserved where possible.
