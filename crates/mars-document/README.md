# mars-document

Notebook document model for the Project Mars notebook platform.

## Purpose

`mars-document` represents the `.mars` notebook format in memory: cells, metadata, serialization, persistence, validation, and undo/redo history.

## Modules

- **`cell`** — `Cell` type with source, outputs, metadata, and execution count.
- **`notebook`** — `Notebook`, `NotebookMetadata`, format versioning.
- **`persistence`** — Atomic JSON save/load (`save_atomic`, `load`, `to_json_pretty`, `from_json`).
- **`validation`** — Document validation (`validate`).
- **`history`** — Undo/redo history (`NotebookHistory`).
- **`error`** — `DocumentError`.

## Usage

```rust
use mars_document::{Cell, CellType, Notebook, save_atomic, load};

let mut notebook = Notebook::new();
notebook.add_cell(Cell::new(CellType::Code, "1 + 1")).unwrap();
save_atomic(&notebook, "notebook.mars").unwrap();
let restored = load("notebook.mars").unwrap();
```
