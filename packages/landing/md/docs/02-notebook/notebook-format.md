---
title: Notebook Format
description: Project Mars notebook-format documentation
order: 2
author: Project Mars Team
tags: [notebook-format, json, schema, jupyter, cells, outputs]
---

# Notebook format

A `.mars` notebook is a JSON document. The current format version is `2`.

```json
{
  "format_version": 2,
  "metadata": {
    "title": "My first notebook",
    "language": "rust",
    "dependencies": {
      "serde": "1.0"
    },
    "cargo_lock": "..."
  },
  "cells": [
    {
      "id": "cell-markdown",
      "cell_type": "markdown",
      "source": "# Welcome to Project Mars",
      "metadata": {},
      "execution_count": null,
      "outputs": []
    },
    {
      "id": "cell-code",
      "cell_type": "code",
      "source": "let answer = 42;\nanswer",
      "metadata": {
        "language": "rust"
      },
      "execution_count": 1,
      "outputs": [
        {
          "type": "text",
          "text": "42"
        }
      ]
    }
  ]
}
```

## Metadata

Notebook-level metadata includes:

- `title`: Notebook title
- `language`: Default execution language (`rust`, `python`, `r`)
- `dependencies`: Cargo dependency specifications for Rust notebooks
- `cargo_lock`: Embedded Cargo.lock for deterministic execution
- `custom`: Arbitrary user-defined metadata

## Cells

Every cell has a stable unique `id`, a `cell_type`, its `source`, and a
`metadata` object. Cell types:

- `"markdown"`: Markdown content
- `"code"`: Executable code
- `"raw"`: Raw content for future compatibility

Code cells also store an `execution_count` and an `outputs` array. Outputs are
preserved when a notebook is saved, so the document remains useful after it
has been closed.

### Cell metadata

Code cells can override the notebook language:

```json
{
  "metadata": {
    "language": "python"
  }
}
```

Other supported cell metadata:

- `language`: Execution language for this cell
- `attachments`: Image attachments for Markdown cells (data URIs)
- `collapsed`: Whether outputs are collapsed
- `tags`: Cell tags for organization
- `custom`: Arbitrary user-defined metadata

## Outputs

Project Mars supports text, errors, HTML, images, tables, LaTeX, chart specifications,
and MIME bundles. Images may be stored as base64 PNG, JPEG, GIF, WebP, or SVG
data. HTML is sanitised before rendering.

### Output types

| Type                           | Example                                                 |
| ------------------------------ | ------------------------------------------------------- |
| `text`                         | Plain text output from `println!` or expression results |
| `error`                        | Compilation or runtime error messages with source spans |
| `html`                         | Sanitized HTML content for rich formatting              |
| `image/png`                    | Base64-encoded PNG image data                           |
| `image/jpeg`                   | Base64-encoded JPEG image data                          |
| `image/svg+xml`                | SVG markup for vector graphics                          |
| `text/latex`                   | LaTeX equations for mathematical notation               |
| `application/vnd.dataframe`    | Tabular data with column types and values               |
| `application/vnd.vega.vl+json` | Vega-Lite chart specification                           |

### MIME bundles

Multiple representations can be provided together as a MIME bundle:

```json
{
  "type": "mime_bundle",
  "data": {
    "text/plain": "A simple chart",
    "image/svg+xml": "<svg>...</svg>",
    "text/html": "<div class=\"chart\">...</div>"
  }
}
```

Renderers choose the best available representation based on their capabilities.

### Error outputs

Error outputs include diagnostic information:

```json
{
  "type": "error",
  "ename": "error name",
  "evalue": "error message",
  "traceback": ["line 1", "line 2"]
}
```

## Validation and saving

Documents are validated before they are written. Invalid cell types,
duplicate IDs, and execution data on Markdown cells are rejected. Saves are
atomic so an interrupted write cannot corrupt an existing notebook.

## Format migrations

Project Mars supports format migrations. Version 1 → Version 2 migration is handled
automatically when loading older notebooks. The migration includes:

- Updated output schema
- Added MIME bundle support
- Enhanced metadata structure

## Imports and exports

Project Mars can import and export representative Jupyter `.ipynb` notebooks. Cell
types, outputs, execution counts, and unknown metadata are preserved where
possible. Unsupported Jupyter features are documented rather than silently
discarded.

### Jupyter compatibility

Supported Jupyter features:

- Code, Markdown, and raw cells
- Text, error, HTML, and image outputs
- PNG, JPEG, and SVG images
- Display metadata
- Widget preservation (as raw metadata)
- MIME bundles
- Execution counts

Known limitations:

- Widget interactive behavior is not executed
- Structured table semantics are simplified
- Custom notebook metadata is preserved but not interpreted

### Language mapping

Jupyter kernelspecs map to Project Mars languages:

| Jupyter kernelspec | Project Mars language |
| ------------------ | --------------------- |
| `ir`               | `r`                   |
| `python3`          | `python`              |
| `rust`             | `rust`                |
| `ijulia`           | `julia`               |
| _other_            | Preserved in metadata |

## Optional metadata conventions

The format preserves arbitrary metadata; these keys have defined meanings:

- `cell.metadata.language` — per-cell language override.
- `cell.metadata.attachments` — attachment name → `data:` URL, referenced
  from Markdown as `![](attachment:name)`; resolved on export.
- `cell.metadata.preview` — CSV path rendered as a lazy table preview in
  the desktop app.
- `cell.metadata.comments` — `{ text, created_at }` review comments.
- `cell.metadata.tags` — free-form cell labels.
- `metadata.sql.tables` — table name → CSV path pre-loaded into `sql`
  cells' in-memory SQLite database.
