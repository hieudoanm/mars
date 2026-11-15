# mars-core

Shared domain types for the Project Mars notebook platform.

## Purpose

`mars-core` defines the foundational data types used across all Project Mars crates. It is dependency-light (only `serde`, `serde_json`, `thiserror`, and `uuid`) and contains no execution or rendering logic.

## Types

- **`CellId`** — Stable unique identifier for notebook cells, backed by a UUID.
- **`CellType`** — Markdown, Code, or Raw cell classification.
- **`Output`** — Serialized cell output: Text, Error, Html, Image, Table, Latex, Chart, MimeBundle.
- **`ExecutionStatus`** — Idle, Running, Succeeded, Failed, Interrupted.
- **`PaperSize`** — ISO 216 and common US paper sizes with dimensions in PostScript points.
- **`CoreError`** — Core error type (e.g. invalid cell ID).

## Usage

```rust
use mars_core::{CellId, CellType, Output, PaperSize};

let id = CellId::new();
let cell_type = CellType::Code;
let size = PaperSize::A4;
let (w, h) = size.dimensions();
```
