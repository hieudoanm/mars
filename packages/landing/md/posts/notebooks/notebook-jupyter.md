---
title: 'Jupyter: The Original Interactive Computing Platform'
description: 'Jupyter turned code into documents and documents into code. Here is why the notebook format won, and what it taught the tools that came after it.'
author: 'Project Mars Team'
date: '2026-11-07'
order: 9
tags: ['jupyter', 'notebooks', 'literate-programming', 'history', 'mars']
---

# Why Jupyter?

Before Jupyter, code and explanation lived in different places. You wrote a script, ran it, copied the output into a report, and hoped nothing changed in between. Jupyter collapsed that workflow into a single document: prose, code, and results in one file, executed top to bottom, saved, and shared. It is hard to overstate how much that simple idea changed computational work.

## The idea behind it

The notebook descends from Donald Knuth's _literate programming_ — the idea that a program should be written for humans first and machines second. Jupyter made that practical: a cell could be Markdown explaining _why_ you are doing something, followed by code that does it, followed by the output that proves it worked.

That format happens to fit how people actually think about data:

1. **Explore.** Load a dataset, look at the first rows, plot a histogram.
2. **Hypothesize.** Write down what you expect to see.
3. **Test.** Run the analysis, compare the output to the hypothesis.
4. **Iterate.** Change one thing, run it again, keep what works.

A notebook is the lab notebook for that loop — except the experiments are code.

## Why it spread so far

Jupyter's success was not just the document format. Three design decisions made it stick:

- **The kernel protocol.** Jupyter separated the _interface_ from the _runtime_. A kernel is just a process that accepts code and returns results, so the same frontend could drive Python, R, Julia, or anything else that spoke the protocol. The ecosystem of kernels became a marketplace of languages.
- **Rich outputs.** A cell could return more than text: images, HTML, LaTeX equations, interactive widgets. Outputs became first-class content, not just console logs.
- **`.ipynb` as a file.** A notebook is a single JSON file. It diffs, it versions, it attaches to an email. No server required to read it — just open it anywhere.

## Where it strains

The same design choices that made Jupyter universal also created its limits:

- **Python is the default.** Most Jupyter users are really using Python. The kernel protocol supports other languages, but the tooling, documentation, and community assume Python — which means the notebook inherits the GIL, the interpreter overhead, and dynamic typing whether you like it or not.
- **Execution is opaque.** A notebook's outputs are a snapshot. Rerun the cells in a different order and you can get a different answer with no warning. Reproducibility is a discipline the user has to bring, not a property the format enforces.
- **Deployment is awkward.** Sharing a notebook means sharing its environment. "Works on my machine" is the default failure mode, and pinning dependencies is a separate discipline from writing the analysis.
- **Scale hits a wall.** Notebooks are great for exploration and painful for production. The path from "notebook that works" to "script that runs in CI" is a rewrite.

## What Project Mars took from Jupyter

Project Mars is a notebook platform, and Jupyter is the reason notebooks exist as a category. We kept the parts that work:

- **The document model.** Markdown prose, executable code cells, rich outputs — stored as a versioned JSON file (`.mars`).
- **Kernel isolation.** User code runs in a separate process, never inside the application itself.
- **Rich outputs.** Text, errors, HTML, images, LaTeX, tables, and charts are all first-class output types.
- **Interop.** Project Mars imports and exports `.ipynb` notebooks, so existing Jupyter work is not stranded.

What Project Mars changes is the center of gravity: Rust is the primary language, not an afterthought. Where Jupyter made _any_ language possible but _Python_ inevitable, Project Mars makes _compiled, memory-safe, reproducible_ the default. See [Why Project Mars?](/blog/why-mars) for the full comparison.

The notebook format Jupyter invented was one of the best ideas in developer tooling. Project Mars exists because we think it deserves a runtime built for the workloads people actually run inside it.
