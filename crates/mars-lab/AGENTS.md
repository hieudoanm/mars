# mars-lab — Machine Learning in Rust

Core of Mars Lab: CSV → trained model → ONNX, in pure Rust. Ported from
llm.rs (`llm-core` + `llm-db`, consolidated into one crate). Consumed by the
`mars lab` CLI (`mars-cli/src/lab.rs`) and the desktop Tauri commands
(`packages/desktop/src-tauri/src/lab.rs`). Keep it free of HTTP, Tauri, and
CLI concerns — callers own transport and argument parsing.

## Responsibilities

- `catalogue` — the 83 `AlgorithmSpec`s across six `TaskType`s; the single
  source of truth for algorithm IDs, hyperparameters, and task compatibility.
- `domain` — `TaskType`, `JobStatus`, `ArtefactFormat`, `Job`, `Dataset`,
  `JobMetric`; serde-stable shapes shared with the CLI, Tauri, and SQLite.
- `config` — `Settings` from `MARS_LAB_*` env vars; data dir defaults to
  `.mars-lab-data/`.
- `csv` / `dataframe` — CSV ingestion and the polars `DataFrame` wrapper all
  trainers consume.
- `training` — pure-Rust trainers (linear, trees, kernel/SVM, k-NN, naive
  Bayes, MLP, cluster, anomaly, offline RL/FQI), preprocessing, metrics, and
  the `charts` payload shapes the desktop renders.
- `insights` — dataset profiling and warnings.
- `storage` — dataset/artefact paths and atomic writes under the data dir.
- `export::onnx` — hand-built ONNX protobuf graphs.
- `db` — sqlx + SQLite repositories (datasets, jobs, artefacts, metrics,
  idempotency) with idempotent schema migrations and WAL mode.

## Rules

- No `unwrap()`/`expect()` in production code; return `LabError`.
- Trainers must stay deterministic where seeded and must not do I/O —
  persistence goes through `db`/`storage`, callers own file selection.
- Job status transitions (`queued → training → exporting → completed`,
  `record_failure`) are part of the contract; don't bypass them.
- ONNX export pins: `onnx-protobuf =0.2.3`; the workspace patches it to a
  vendored copy (`vendor/onnx-protobuf`) because upstream rejects
  `protobuf 3.7.x` — keep the patch and the `protobuf` version in sync.
- SQL: build dynamic queries with `sqlx::QueryBuilder`; `push_bind` supplies
  its own placeholders — never write a literal `?` next to a bound value
  (this previously produced `LIMIT ??`).
- Tests currently live in inline `#[cfg(test)]` modules ported from llm.rs;
  repo policy prefers crate-level `tests/` dirs — extract gradually, widening
  visibility only where needed.

## Testing

```bash
cargo test -p mars-lab
cargo clippy -p mars-lab --all-targets -- -D warnings
cargo fmt --all --check
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-lab --tests` (workspace gate: `make coverage`).
