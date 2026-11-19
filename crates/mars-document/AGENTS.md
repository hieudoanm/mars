# mars-document — Notebook Model & Persistence

Owns the `.mars` notebook representation. Depends only on `mars-core`.

## Responsibilities

- `Notebook` / `NotebookMetadata` — format version (`CURRENT_FORMAT_VERSION`, currently 2), language, dependencies (`BTreeMap`), `cargo_lock` snapshot, `page_size`, and `extra` metadata preserved via `#[serde(flatten)]`.
- `Cell` — `id`, `cell_type`, `source`, free-form `metadata`, optional `execution_count`, `outputs`.
- `Notebook::add_cell / insert_cell / remove_cell / move_cell / cell / cell_mut` — all return `DocumentError` on duplicate IDs or out-of-bounds indexes; never panic.
- `validate()` — enforces format version, non-empty unique cell IDs, and rejects execution data on Markdown/Raw cells.
- `persistence` — `to_json_pretty` / `from_json` / `load` / `save_atomic`. Serialization always validates first; loading migrates old `format_version` values and validates after. `save_atomic` writes a sibling `.tmp` file then renames — never write notebooks non-atomically.
- `NotebookHistory` — bounded undo/redo stack of whole `Notebook` snapshots (capacity-based, clears redo on record).

## Rules

- The document model must stay language-agnostic — do not couple cell sources or metadata to Rust specifically.
- New serialization fields must be `#[serde(default)]`/`skip_serializing_if` so old files keep loading.
- Format changes require bumping `CURRENT_FORMAT_VERSION` and adding a migration in `migrate_value`.

## Testing

```bash
cargo test -p mars-document
cargo clippy -p mars-document --all-targets -- -D warnings
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-document --tests` (workspace gate: `make coverage`).
