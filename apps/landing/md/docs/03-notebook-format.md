---
title: Notebook Format
description: Mars notebook-format documentation
order: 3
author: Mars Team
tags: [notebook-format, json, schema, jupyter, cells, outputs]
---

# Notebook format

A `.mars` notebook is a JSON document. The current format version is `2`.

```json
{
  "format_version": 2,
  "metadata": {
    "title": "My first notebook",
    "language": "rust"
  },
  "cells": [
    {
      "id": "cell-markdown",
      "cell_type": "markdown",
      "source": "# Welcome to Mars",
      "metadata": {},
      "execution_count": null,
      "outputs": []
    },
    {
      "id": "cell-code",
      "cell_type": "code",
      "source": "let answer = 42;\nanswer",
      "metadata": {},
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

## Cells

Every cell has a stable unique `id`, a `cell_type`, its `source`, and a
`metadata` object. Markdown cells use `"markdown"`, executable cells use
`"code"`, and raw cells use `"raw"`.

Code cells also store an `execution_count` and an `outputs` array. Outputs are
preserved when a notebook is saved, so the document remains useful after it
has been closed.

## Outputs

Mars supports text, errors, HTML, images, tables, LaTeX, chart specifications,
and MIME bundles. Images may be stored as base64 PNG, JPEG, GIF, WebP, or SVG
data. HTML is sanitised before rendering.

## Validation and saving

Documents are validated before they are written. Invalid cell types,
duplicate IDs, and execution data on Markdown cells are rejected. Saves are
atomic so an interrupted write cannot corrupt an existing notebook.

## Imports and exports

Mars can import and export representative Jupyter `.ipynb` notebooks. Cell
types, outputs, execution counts, and unknown metadata are preserved where
possible. Unsupported Jupyter features are documented rather than silently
discarded.
