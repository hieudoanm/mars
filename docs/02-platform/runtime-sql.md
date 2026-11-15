# SQL Runtime

## Overview

Project Mars ships an in-process SQL backend (`mars-runtime-sql`) so any code
cell can run SQL without an external database server. The backend owns a
persistent SQLite session per notebook session: `SELECT`, DDL, and DML are all
supported, and state created in one cell is visible to later cells.

There is nothing to install — SQLite is embedded.

## Running SQL cells

Set `cell.metadata.language` to `"sql"`. In the desktop app the language
picker offers SQL directly; in `.mars` files the metadata looks like:

```json
{
  "cell_type": "code",
  "metadata": { "language": "sql" },
  "source": "SELECT name, total FROM sales ORDER BY total DESC LIMIT 10"
}
```

`SELECT` (and `WITH`/`EXPLAIN`-style row-returning statements) render their
final result as a notebook table. Non-`SELECT` statements return a short text
confirmation instead.

## Pre-loading tables from CSV

Tables can be declared in notebook metadata. Paths resolve relative to the
`.mars` file:

```json
"metadata": {
  "sql": {
    "tables": {
      "sales": "data/sales.csv",
      "customers": "data/customers.csv"
    }
  }
}
```

Each named CSV becomes a SQL table at session start, so
`SELECT * FROM sales` works immediately. Sessions are picked up
automatically by `mars notebook run`, `mars kernel serve`, the MCP server,
and the desktop app.

## Session semantics

- One SQLite session per notebook session; DDL and DML persist across cells.
- `reset` drops all session state, including created tables.
- Table names from `metadata.sql.tables` are validated (identifier characters
  only) before being loaded.
- Errors surface as ordinary cell errors; cancellation and timeouts follow
  the shared `RuntimeBackend` contract.

## When to use it

SQL cells are the fastest way to explore tabular data inside a notebook —
join, aggregate, and filter without leaving the document. For heavier
analytics or production pipelines, pair them with the Mars Lab product
(`mars lab …`), which trains models from the same CSV files.
