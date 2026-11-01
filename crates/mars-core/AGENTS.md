# mars-core — Shared Domain Types

Foundational crate with no internal dependencies. All other crates depend on it.

## Responsibilities

- `CellId` — stable unique cell identifier (UUID v4 string, validated non-empty on construction).
- `CellType` — `Markdown` | `Code` | `Raw` (serde lowercase).
- `ExecutionStatus` — `Idle | Running | Succeeded | Failed | Interrupted` (serde snake_case).
- `Output` — tagged enum (`type` field, snake_case) covering `Text`, `Error`, `Html`, `Image`, `Table`, `Latex`, `Chart`, `MimeBundle`. `MimeBundle` carries arbitrary MIME data so new output kinds don't require schema changes.
- `PaperSize` — standard paper sizes (A0–A6, B4/B5, Letter, Legal, Tabloid) in PostScript points, used for PDF export.
- `CoreError` — minimal error type (`InvalidCellId`).

## Rules

- Keep this crate dependency-light and free of execution, document, or I/O concerns.
- All types must be `Serialize`/`Deserialize` with stable wire formats — they cross the kernel protocol and the `.mars` JSON format.
- Add new output kinds to `Output` only when they are genuinely distinct; prefer `MimeBundle` for extensibility.

## Testing

```bash
cargo test -p mars-core
cargo clippy -p mars-core --all-targets -- -D warnings
```
