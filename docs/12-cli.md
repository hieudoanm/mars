# CLI reference

The `mars` CLI is the headless interface to the document, runtime, renderer,
and Jupyter conversion crates. Commands return a nonzero exit code when input
cannot be loaded, validation fails, execution fails, or conversion is not
supported.

## Command summary

```text
mars notebook new <file.mars> [language] [--template <name>] [--example <name>] [--language <lang>]
mars notebook templates
mars notebook doctor [file.mars]
mars notebook open <file.mars>
mars notebook inspect <file.mars> [--json]
mars notebook run <file.mars> [options]
mars notebook import <file.ipynb> [file.mars]
mars notebook export <file.mars> [ipynb|html|md] [output]
mars kernel [list|restart|serve]
```

Create and inspect notebooks:

```bash
mars notebook new analysis.mars
mars notebook new analysis.mars --template python
mars notebook new analysis.mars --example rust/02-machine-learning/machine-learning
mars notebook inspect analysis.mars
mars notebook inspect analysis.mars --json
```

`mars notebook new` accepts an optional positional language for backwards
compatibility, plus `--language <lang>` and two content sources:

- `--template rust|python|ml|dashboard` — built-in starter notebooks with
  markdown introductions and runnable code cells.
- `--example <name>` — copy one of the example notebooks shipped in
  `examples/mars/`. Names are repository-relative paths without the `.mars`
  extension (e.g. `rust/01-getting-started/getting-started`); run
  `mars notebook templates` to list them. Examples are discovered from the
  current directory upward, or from `MARS_EXAMPLES_DIR` when set.

`--template` and `--example` are mutually exclusive; an explicit language
flag overrides the template/example default.

`mars notebook templates` lists the built-in templates and any example
notebooks found on disk.

`mars notebook doctor [file.mars]` probes the external tools each language
runtime needs (`cargo`/`rustc` for Rust, `python3` for Python, `Rscript` for
R, `julia` for Julia, `clang++`/`g++` for C++) and prints versions or
install hints. With a notebook path it checks only the languages that
notebook uses and exits nonzero when a required runtime is missing.

`mars notebook inspect --json` prints a machine-readable summary (path,
format version, language, title, cell counts, dependencies) for scripts and
CI; the default output remains human-readable.

Execute a notebook:

```bash
mars notebook run analysis.mars
mars notebook run analysis.mars --timeout-ms 5000
mars notebook run analysis.mars --locked --offline --deterministic --report
```

Convert notebooks:

```bash
mars notebook import notebook.ipynb analysis.mars
mars notebook export analysis.mars ipynb notebook.ipynb
mars notebook export analysis.mars html analysis.html
mars notebook export analysis.mars md analysis.md
```

Markdown export preserves Markdown cells, writes code cells as fenced `rust`
blocks, and converts outputs into portable Markdown: text and errors become
fenced blocks, tables become Markdown tables, LaTeX becomes `$$` blocks, and
images or offline charts become data-URI images. `markdown` is accepted as an
alias for `md`.

`mars notebook run` prints `Running cell N...` progress for each code cell as it
executes and a completion summary at the end. It saves partial outputs when
execution fails or is interrupted and
returns a nonzero exit code.

By default, a successful run writes execution counts and outputs back to the
input notebook. A failed run writes the partial state that was available before
the failing cell. `--no-save` disables this write and is appropriate for
read-only checks or CI probes.

Use `--locked` to require the notebook's stored Cargo lockfile and `--offline`
to prevent Cargo from accessing registries. These options are especially useful
for reproducible or restricted execution environments. `--deterministic`
requires an embedded lockfile for dependency-backed notebooks, `--no-save`
executes without writing outputs back to disk, and `--report` prints a final
execution summary.

`--deterministic` is a convenience policy: it implies `--locked` and rejects
dependency-backed notebooks without `metadata.cargo_lock`. `--no-save` is
useful for CI dry runs or read-only workspaces, while `--report` adds a final
summary line without changing the notebook format. The report is intended for
human-readable CI logs; it is not a versioned machine-readable protocol.

### Execution option reference

| Option | Meaning |
| --- | --- |
| `--timeout-ms <n>` | Bound compilation and execution for each cell |
| `--locked` | Require the embedded Cargo lockfile for dependency-backed runs |
| `--offline` | Prevent Cargo registry/network access |
| `--deterministic` | Require a lockfile and apply locked execution policy |
| `--no-save` | Do not write outputs or counts back to the notebook |
| `--report` | Print a final execution summary |
| `--changed-only` | Skip cells before the first code cell changed vs the git baseline |
| `--since <git-ref>` | Diff base for `--changed-only` (implies it); default `HEAD` |
| `--watch` | Re-run the notebook whenever the file changes |
| `--watch-ms <n>` | Watch poll interval in milliseconds (default `1000`) |

`--changed-only` compares the notebook against `git show <ref>:<path>`
matching cells by their stable IDs. Execution resumes at the first code
cell that is new or whose source changed; earlier cells are skipped and
everything from that point runs, because the runtime starts with empty
state. Requires the notebook to be tracked by git; `--since` accepts any
revision (`HEAD~3`, a branch, a tag). When nothing changed, the command
reports it and exits successfully.

`--watch` runs the notebook once, then polls the file every `--watch-ms`
milliseconds and re-runs on change — useful for live demos and iterating on
outputs. Execution errors are printed but do not stop the watch loop;
Ctrl+C exits.

Open a notebook with the platform default application:

```bash
mars notebook open analysis.mars
```

Kernel lifecycle and integration commands:

```bash
mars kernel list
mars kernel restart
mars kernel serve [--runtime replay|evcxr] [--notebook <file.mars>] [--language <lang>] [--tcp <port>] [--tcp-host <addr>]
```

`mars kernel serve` runs a persistent newline-delimited JSON transport. Each
input line is a serialized `RequestEnvelope`; responses are serialized
`ResponseEnvelope` lines. While an `execute` request runs, the kernel also
emits `EventEnvelope` lines (`{"id", "event": {"type": "phase"|"chunk"|
"prompt", ...}}`) for compiling/running phases and stdout/stderr chunks
before the final response — see `docs/12-kernel-protocol.md`. Interactive
prompts (`mars_prompt`, Python `input()`) are answered with `input` requests,
which the server routes through the running backend's stdin writer without
waiting for the in-flight `execute` to finish. `Execute` requests may carry a
`language` field, and the server keeps one kernel per language; `reset`
clears all of them and `status` aggregates across languages. With `--tcp
<port>` the same protocol is served over TCP (loopback by default; add
`--tcp-host` to bind another interface) — one kernel session per
connection — for remote or headless clients. Pass `--notebook`
to load the notebook's declared Cargo dependencies and lockfile into the
kernel; without it the kernel starts with an empty dependency set.

## Examples

Run a notebook without changing it:

```bash
mars notebook run examples/getting-started.mars --no-save --report
```

Run a dependency-backed notebook using only its stored resolution:

```bash
mars notebook run analysis.mars --deterministic --offline --report
```

Generate default output paths:

```bash
mars notebook export analysis.mars html
mars notebook export analysis.mars ipynb
mars notebook export analysis.mars md
```

## Cell-aware diff

```bash
mars notebook diff analysis.mars          # vs HEAD
mars notebook diff analysis.mars HEAD~3   # vs any git ref
mars notebook diff analysis.mars other.mars
```

`mars notebook diff` compares cells by their stable IDs and prints
`+`/`-`/`~` markers with a line-level diff for modified cells — a
nbformat-native diff without external tools.

## Machine-readable run reports

```bash
mars notebook run analysis.mars --report --json-report
```

`--json-report` (implies `--report`) prints a JSON summary
(`path`, `saved`, `code_cells`, `executed_cells`, `skipped_prefix`) suitable
for CI log parsing.

## SQL cells

Any code cell can run SQL by setting `cell.metadata.language` to `"sql"`
(the desktop language picker offers it directly). SQL cells execute in a
persistent in-process SQLite session — no external database required — and
support `SELECT`, DDL, and DML. Results render as notebook tables.

Tables are pre-loaded from CSV files declared in notebook metadata:

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

Paths resolve relative to the `.mars` file. Each named CSV becomes a SQL
table, so `SELECT * FROM sales` works immediately. Sessions persist across
cells (create tables, insert rows, query later); `mars notebook run` picks
up `metadata.sql.tables` automatically, as do `mars kernel serve` and the
MCP server.

## Scheduled runs

Headless execution plus git-aware change detection covers the scheduling use
case without a daemon:

```cron
# .mars notebooks: run changed cells nightly, fail loudly on errors
0 3 * * * cd /srv/notebooks && mars notebook run analysis.mars --changed-only --locked --report --json-report >> run.log 2>&1
```

`--changed-only` exits successfully without executing when nothing changed,
so cron noise stays low; execution failures exit nonzero.

## Publishing

`mars notebook export analysis.mars html` produces a single self-contained
HTML file — inline CSS, embedded output images, and markdown `attachment:`
image references resolved to data URIs — safe to host anywhere or attach to
an email. `slides` produces a self-contained presentation deck (arrow keys,
print-friendly). For interactive publishing, keep the `.mars` file and let
readers run it in the desktop app or `mars kernel serve`.
