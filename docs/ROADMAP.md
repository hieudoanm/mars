# Project Mars roadmap

Where Project Mars is today and what comes next. This is a living document —
update it when shipped work changes the picture.

## Shipped

### Mars Notebook

- `.mars` JSON notebook format with validation, migrations, and autosave
- Multi-language runtimes: Rust (replay + opt-in Evcxr), Python, R, Julia, C++,
  and in-process SQL (SQLite)
- Tauri 2 + React desktop app: multi-notebook tabs, file explorer, checkpoints
  with diff, variable inspector, problems panel, CodeMirror 6 editing
- rust-analyzer LSP integration (diagnostics, hover, completion, quick fixes)
- Rich outputs: HTML, LaTeX, images, tables, charts, MIME bundles
- `mars notebook` CLI: templates, doctor, cell-aware diff, git-aware
  `--changed-only`/`--watch` runs, JSON reports, HTML/Markdown/slides/`.ipynb`
  export
- Jupyter `.ipynb` import/export with metadata preservation
- MCP server (`mars mcp serve`) for AI assistants
- VS Code, Zed, and JetBrains extension foundations
- Runtime helpers: `mars_table`, `mars_csv`, `mars_display_*`, `mars_chart`,
  `mars_prompt`

### Mars Lab

- `mars-lab` crate: 83 algorithms across six task types (classification,
  regression, semi-supervised, clustering, anomaly detection, offline RL)
- CSV upload → profiling → column mapping → training → metrics/charts → ONNX
- SQLite persistence for datasets, jobs, metrics, artefacts (WAL, idempotent
  schema)
- `mars lab` CLI and the Lab view in the desktop app (Welcome / Lab / Notebook
  routing)
- Job history with filters, search, pagination; side-by-side job comparison;
  live training-stage progress events; dataset preview with profile warnings

## Near term

- **Training depth** — async/background training with real progress reporting
  (currently stage-level events only), hyperparameter sweeps, and model
  versioning in the artefact store.
- **Lab charts** — richer chart coverage for task types whose `charts` payload
  is still rendered as raw JSON.
- **Brand assets** — regenerate raster assets (`og.png`, `logo.png`,
  `favicon.ico`) from the updated SVG sources now that branding says
  "Project Mars" / "Mars Notebook" / "Mars Lab".
- **Coverage** — keep crates at the 80% line/function threshold enforced by
  `make coverage`; `mars-lab`'s ported inline tests are still being migrated to
  crate-level `tests/` directories.
- **JetBrains plugin** — move from planned to shipped (see
  `04-extensions/extension-jetbrains.md`).

## Longer term

- **More runtimes** — additional `RuntimeBackend` languages as demand appears;
  the protocol boundary is designed for it.
- **Collaboration** — real-time multi-user editing and shared kernel sessions.
- **Lab model hub** — publish/import trained ONNX models, dataset versioning,
  and experiment tracking beyond local SQLite.
- **Cloud sync** — optional sync for notebooks and Lab artefacts across
  machines.
- **Ecosystem** — more editor integrations, themes, and community notebook
  templates.

## Known limitations

- C++ notebooks are documentation-only until a C++ runtime backend lands.
- Evcxr Rust backend is opt-in (`experimental-evcxr`) and replays state on each
  cell.
- Lab training runs synchronously inside the Tauri command for v1 dataset
  sizes; very large datasets need the async worker path.
- `vendor/onnx-protobuf` is a temporary patch — upstream `onnx-protobuf`
  compatibility with `protobuf` 3.7.x should be resolved and the patch removed.
