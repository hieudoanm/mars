# mars-runtime-sql — SQL Execution Backend

`RuntimeBackend` implementation backed by an in-process SQLite session
(sqlite via `rusqlite` or equivalent — see Cargo.toml). Language id: `sql`.

## Responsibilities

- `SqlSession` — owns a SQLite connection; `execute`/`execute_with_control`
  run cell source and map the final statement's rows to `Output::Table`.
- `SqlSource { name, path }` — CSV tables pre-loaded at session creation;
  declared in notebook `metadata.sql.tables` and resolved by
  `mars_kernel::sql_sources_from_metadata`.
- DDL/DML persist across cells within a session; `reset()` drops state.
- Non-SELECT statements return a short text confirmation, not a table.

## Rules

- Table names from `SqlSource` are validated (identifier characters only)
  before being used in `CREATE TABLE ... AS` / attach statements.
- Errors surface as `RuntimeError`; cancelled/timeout semantics follow the
  shared `RuntimeBackend` contract.

## Testing

```bash
cargo test -p mars-runtime-sql
cargo clippy -p mars-runtime-sql --all-targets -- -D warnings
```
