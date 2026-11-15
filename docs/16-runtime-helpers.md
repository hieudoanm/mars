# 11 — Runtime helpers, directives, and cell features

This page documents the user-facing features that live inside notebook cells:
injected Rust helper functions, cell directives, per-cell language metadata,
and Markdown attachments.

## Injected Rust helpers

Every code cell is compiled into a generated `fn main()` program. Project Mars injects
a small set of helper functions into that program so cells can produce rich
outputs without extra dependencies. All helpers are available in every Rust
code cell.

### `mars_prompt(question: &str) -> String`

Prints a prompt marker, flushes stdout, and reads one line from stdin. Used
for interactive input during execution.

```rust
let name = mars_prompt("Enter your name: ");
println!("Hello, {name}!");
```

### `mars_table<C, R>(columns: &[C], rows: &[Vec<R>])`

Renders a structured table output. `columns` is the header row; each element
of `rows` is one row of stringifiable cells.

```rust
mars_table(
    &["name", "score"],
    &[vec!["alice".to_string(), "10".to_string()],
         vec!["bob".to_string(), "20".to_string()]],
);
```

### `mars_csv(text: &str)`

Parses CSV text (first row = header, RFC-4180-style quoting) and renders it
as a table. Handy for pasting tabular data into a cell.

```rust
mars_csv("name,score\nalice,10\nbob,20\n");
```

### `mars_display_html(html: &str)`

Emits a sanitized HTML output. The renderer passes the HTML through the same
sanitizer used for all rich output — scripts and unsafe markup are stripped.

```rust
mars_display_html("<strong>bold</strong> and <em>italic</em>");
```

### `mars_display_latex(latex: &str)`

Emits a LaTeX output rendered by KaTeX.

```rust
mars_display_latex("E = mc^2");
```

### `mars_display_image(mime_type: &str, bytes: &[u8])`

Emits an image output from raw bytes. `mime_type` should be `image/png`,
`image/jpeg`, or `image/svg+xml`.

```rust
let png = std::fs::read("chart.png")?;
mars_display_image("image/png", &png);
```

### `mars_display_mime(mime_type: &str, data: &str)`

Emits an arbitrary MIME-bundle output — the escape hatch for output kinds
without a dedicated helper. Common types: `text/markdown`, `text/html`,
`application/json`, `image/svg+xml`.

```rust
mars_display_mime("text/markdown", "# Rendered **markdown**");
```

### `mars_display_markdown(markdown: &str)`

Convenience wrapper emitting a `text/markdown` MIME bundle, rendered by the
Markdown pipeline.

```rust
mars_display_markdown("## Results\n- item one\n- item two");
```

### `mars_chart(spec_json: &str)`

Emits a chart output. `spec_json` is a [Vega-Lite](https://vega.github.io/vega-lite/)
specification as a JSON string, rendered with `vega-embed`.

```rust
mars_chart(r#"{"mark":"bar","data":{"values":[{"x":"a","y":1},{"x":"b","y":2}]},"encoding":{"x":{"field":"x","type":"nominal"},"y":{"field":"y","type":"quantitative"}}}"#);
```

### Marker protocol

Helpers communicate with the runtime by printing single-line markers to
stdout (`__MARS_TABLE__=<json>`, `__MARS_HTML__=<json-string>`, etc.). The
runtime strips marker lines from captured stdout and converts them into typed
`Output` values. User code that prints a line starting with a marker prefix
accidentally will have it consumed — avoid printing these prefixes.

## Cell directives

Special comments at the top of a code cell configure execution:

```rust
// dep: serde@1.0
// timeout: 30s
let value: serde_json::Value = serde_json::from_str("{}").unwrap();
```

- `// dep: <crate>[@<version>]` (or the `// deps:` alias) — declares a
  Cargo dependency for the session. Multiple `dep:`/`deps:` lines
  accumulate; version defaults to `*`.
  Dependencies trigger a Cargo-based build instead of plain `rustc`.
- `// timeout: <seconds>s` (or a bare number of seconds) — overrides the
  session execution timeout for that cell's run.

Notebook-level dependencies can also be managed in the Dependencies dialog;
cell directives are merged into the session before execution.

## Per-cell language

Code cells carry `metadata.language`. The default is `rust`; setting it to
`text` (via the cell header language selector) makes the cell a plain-text
cell that is skipped by execution, diagnostics, and LSP sync. This metadata
is preserved in `.mars` files and mapped on `.ipynb` import/export where
possible.

## Markdown attachments

Pasting or dropping an image into a **Markdown** cell embeds it as a base64
data URL under the cell's `metadata.attachments` and inserts a reference:

```markdown
![screenshot](attachment:image-1.png)
```

`attachment:` references are resolved to data URIs at render time and are
subject to the same sanitization as other image sources. Attachments are
stored inside the `.mars` document, so notebooks remain self-contained.

## Scope notes

- Helpers are injected into the replay-based Rust session only; the
  experimental Evcxr backend does not provide them.
- There is deliberately no `mars_display_*` for widgets or interactive
  front-end state — the widget protocol is out of scope (see
  [12-kernel-protocol.md](./12-kernel-protocol.md)).

The desktop editor surfaces these helpers in autocomplete with snippet
placeholders, so `mars_table`, `mars_csv`, `mars_display_*`, `mars_chart`,
and `mars_prompt` complete like first-class functions.

Cell metadata (tags, custom keys) round-trips through `.mars` files and can
be edited as raw JSON from the cell context menu ("Edit metadata") or the
"Edit selected cell metadata" palette command.
