# Project Mars

Project Mars is an independent, Rust-powered interactive computing platform. It is
not a Jupyter frontend — it has its own notebook format, execution engine,
protocol, CLI, desktop application, and rendering pipeline.

**Tagline:** Explore ideas. Execute code. Discover possibilities.

## Products

Project Mars is an umbrella project with two products:

- **Mars Notebook** — a Jupyter Notebook alternative: an interactive environment
  combining executable cells, Markdown, mathematics, tables, charts, and rich
  output in a native `.mars` document, with `.ipynb` import/export.
- **Mars Lab** — _Machine Learning in Rust_: a studio that turns CSV datasets
  into trained ONNX models (83 algorithms across six task types).

## Mars Notebook

- **Native format:** `.mars` — versioned JSON document (cells, metadata, outputs).
- **Languages:** Rust (primary, replay + opt-in Evcxr backends), Python, R, Julia,
  C++, and SQL — all behind the `RuntimeBackend` trait, each running in an
  isolated child process (SQL is in-process SQLite).
- **Execution:** `mars-kernel` owns sessions and dispatches cells to language
  runtimes; `mars-protocol` defines the typed messages; `mars-renderer` turns
  outputs into HTML, Markdown, slides, and `.ipynb`.
- **Editing:** `mars-lsp` provides rust-analyzer diagnostics, hover, completion,
  and quick fixes inside the desktop editor (Tauri 2 + React + TypeScript).
- **CLI:** `mars notebook …` works fully headless — create, run, inspect, diff,
  export — plus `mars kernel serve` for external clients.
- **AI integration:** `mars-mcp` exposes notebook operations over the Model
  Context Protocol; VS Code, Zed, and JetBrains extensions round out the
  ecosystem.

## Mars Lab

- **Pipeline:** CSV upload → profiling → column mapping → algorithm selection →
  training → metrics/charts → ONNX export.
- **Catalogue:** 83 algorithms across classification, regression, semi-supervised
  learning, clustering, anomaly detection, and offline reinforcement learning.
- **Storage:** SQLite (sqlx, WAL) for datasets, jobs, metrics, and artefacts;
  dataset files under `.mars-lab-data/`.
- **Surfaces:** `mars lab …` CLI and the Lab view inside the desktop app
  (Welcome / Lab / Notebook product routing via React Router).

## Repository layout

```text
mars/
├── crates/                  # Rust workspace — all core logic lives here
│   ├── mars-core/           # Shared domain types, IDs, errors
│   ├── mars-document/       # .mars notebook model, persistence, validation
│   ├── mars-runtime/        # Rust replay execution engine
│   ├── mars-runtime-{python,r,julia,cpp,sql,evcxr}/
│   ├── mars-kernel/         # Kernel lifecycle, sessions, language dispatch
│   ├── mars-protocol/       # Typed frontend↔kernel messages
│   ├── mars-renderer/       # HTML/Markdown/slides/.ipynb export
│   ├── mars-jupyter/        # .ipynb import/export
│   ├── mars-lsp/            # rust-analyzer LSP client
│   ├── mars-cli/            # `mars` binary (notebook, lab, kernel, mcp)
│   ├── mars-mcp/            # MCP server (JSON-RPC over stdio)
│   └── mars-lab/            # Mars Lab core + SQLite repositories
├── packages/
│   ├── desktop/             # Tauri 2 + React desktop app (Notebook + Lab)
│   └── landing/             # Next.js static site + blog + docs
├── extensions/              # VS Code, Zed, JetBrains integrations
├── examples/                # Sample .mars notebooks per language
├── vendor/onnx-protobuf/    # Vendored patch (protobuf 3.7.x compat)
└── docs/                    # Canonical documentation (numbered chapters)
```

## Git hooks

- `.husky/pre-commit` runs `pnpm exec lint-staged` on every commit (auto-installed
  by `pnpm install` via the `prepare` script).
- Config lives in the root `package.json` `lint-staged` key: staged
  JS/TS/JSON/Markdown/CSS/HTML/YAML files get `prettier --write` (per-package
  `.prettierrc.json` applies, including the Tailwind plugin); staged `.rs`
  files outside `vendor/` get `rustfmt --edition 2024`.
- Root `.prettierignore` excludes `vendor/`, `pnpm-lock.yaml`, and generated
  directories — do not remove those entries.
- `.husky/commit-msg` runs `commitlint` (`commitlint.config.mjs`): commits
  must use Conventional Commits — `type(scope): subject`, lower-case
  imperative subject.
- `.pre-commit-config.yaml` is a separate, optional Python `pre-commit` suite
  (fmt/clippy/tests/build checks); it is not installed by default.

## Documentation

- [`README.md`](README.md) — project overview, features, quick start.
- [`docs/README.md`](docs/README.md) — full documentation index, organised into
  numbered chapter folders (`01-foundations` … `06-appendix`).
- [`packages/landing/md/docs/`](packages/landing/md/docs/) — the curated docs
  subset published on the landing site (same chapter layout, flat `/docs/…`
  routes).
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — product roadmap and outstanding work.
- Per-crate `README.md`/`AGENTS.md` files document module responsibilities and
  local conventions.
