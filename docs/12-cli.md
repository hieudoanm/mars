# CLI reference

The `mars` CLI is the headless interface to the document, runtime, renderer,
and Jupyter conversion crates. Commands return a nonzero exit code when input
cannot be loaded, validation fails, execution fails, or conversion is not
supported.

## Command summary

```text
mars notebook new <file.mars> [language]
mars notebook open <file.mars>
mars notebook inspect <file.mars>
mars notebook run <file.mars> [options]
mars notebook import <file.ipynb> [file.mars]
mars notebook export <file.mars> [ipynb|html|md] [output]
mars kernel [list|restart|serve]
```

Create and inspect notebooks:

```bash
mars notebook new analysis.mars
mars notebook inspect analysis.mars
```

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
