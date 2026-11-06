# mars-jupyter — `.ipynb` Import & Export

Bidirectional conversion between Mars notebooks and Jupyter nbformat 4. Depends on `mars-core` and `mars-document`.

## Responsibilities

- `import.rs` — `import_ipynb(&str) -> Notebook`: parses nbformat JSON (`JupyterNotebook`, `JupyterCell`, `JupyterOutput`, `Source` which accepts both string and line-array forms), maps cell types, execution counts, and metadata. `language_from_metadata` / `metadata_string` extract kernel/language info.
- `export.rs` — `export_ipynb(&Notebook) -> String`: writes nbformat 4 JSON, `export_cell`/`export_output` map Mars `Output` back to Jupyter output types.
- `conversion.rs` — `convert_output`, `convert_mime_bundle`, `table_to_html` (escaped), `escape_html`.

## Rules

- Unknown MIME types and unrecognised metadata must be preserved via `MimeBundle`/`metadata.extra` — never silently drop data.
- All HTML emitted during export (e.g. tables) must be escaped.
- Unsupported Jupyter features should degrade gracefully with preserved raw data, not errors.
- Keep round-trip fidelity: import → export should retain cells, outputs, and metadata for the supported subset (covered by `tests/fixtures/*.ipynb`).

## Testing

```bash
cargo test -p mars-jupyter
cargo clippy -p mars-jupyter --all-targets -- -D warnings
```
Fixtures live in `tests/fixtures/` at the workspace root.
