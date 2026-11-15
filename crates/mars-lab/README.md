# mars-lab

Machine Learning in Rust — the core of **Mars Lab**, the Project Mars studio that
turns CSV datasets into trained models and exports them as ONNX.

Ported from `llm.rs` (`llm-core` + `llm-db`) and consolidated into a single
crate. It is pure Rust: CSV parsing, tabular dataframes (polars), an
83-algorithm catalogue across six task types, training pipelines, metrics and
chart payloads, dataset insights, filesystem storage, ONNX protobuf export,
and SQLite persistence (sqlx, WAL mode).

## Layout

- `catalogue.rs` — `AlgorithmSpec` catalogue: 83 algorithms across
  classification, regression, semi-supervised, clustering, anomaly detection,
  and offline reinforcement learning.
- `domain.rs` — `TaskType`, `JobStatus`, `ArtefactFormat`, `Job`, `Dataset`,
  `JobMetric` and related domain types.
- `config.rs` — `Settings`, resolved from `MARS_LAB_*` environment variables
  (data dir, upload limit, concurrency, timeouts, rate limiting, compression).
- `csv.rs` / `dataframe.rs` — CSV parsing and the polars-backed `DataFrame`
  abstraction used by trainers.
- `preprocess` (in `training/`) — encoding, scaling, splitting, and
  validation helpers.
- `training/` — hand-rolled trainers: linear models, trees, kernel/SVM,
  k-NN, naive Bayes, MLP, clustering, anomaly detection, and offline RL/FQI —
  plus metrics and chart payload generation.
- `insights.rs` — dataset profiling (column types, missing rates, warnings).
- `storage.rs` — dataset/artefact filesystem layout under the data dir.
- `export/onnx.rs` — hand-built ONNX protobuf graphs
  (`onnx-protobuf`, pinned `protobuf`).
- `db/` — sqlx + SQLite repositories: `datasets`, `jobs` (filters, cursor
  pagination, metrics, queue helpers), `artefacts`, `idempotency`, `schema`
  (idempotent migrations), `convert` (domain ↔ row mapping).

## Consumers

- `mars lab …` CLI commands in `mars-cli` (`src/lab.rs`).
- Tauri commands in `packages/desktop/src-tauri/src/lab.rs` powering the
  desktop Lab view.

The crate has no HTTP or Tauri dependency, so the same code backs the CLI and
the desktop app.

## Data

Defaults to `.mars-lab-data/` in the working directory: `lab.db` (SQLite,
WAL mode) plus dataset and artefact files. Override with `MARS_LAB_DATA_DIR`
and the other `MARS_LAB_*` settings.

## Testing

```bash
cargo test -p mars-lab
cargo clippy -p mars-lab --all-targets -- -D warnings
```

Tests currently live in inline `#[cfg(test)]` modules ported from llm.rs;
migration to crate-level `tests/` directories is tracked as follow-up work.
