---
title: "Project Mars: The Multi-Language Notebook Platform"
description: "Project Mars is a Rust-first interactive notebook. Here is how it compares to Jupyter, and why we built it."
author: "Project Mars Team"
date: "2026-11-06"
order: 8
tags: ["mars", "notebooks", "jupyter", "rust", "product"]
---

# Why Project Mars?

Jupyter notebooks are the standard way to explore code interactively. They are also built on Python, which means they inherit Python's strengths and its limits. **Project Mars** is a notebook platform where the primary language is Rust. It keeps the things people love about Jupyter — Markdown, executable cells, rich outputs, shared state — and rebuilds the whole stack in a language designed for the workloads notebooks actually run.

## What Jupyter does well

Jupyter is not a failure. It popularized a format that has become the default way to write, share, and reproduce data science code. Its strengths:

- A document model that mixes prose, code, and outputs in one file.
- A huge ecosystem of kernels: Python, R, Julia, and more.
- Rich output types: plots, tables, HTML, LaTeX, images.
- A mature community and tooling (nbconvert, papermill, JupyterLab).

## Where Jupyter hits a wall

Jupyter is built for interpreted, dynamic languages. That choice carries real costs:

- **Performance.** A Python notebook that trains a model or processes a large dataset is really a Python wrapper around C extensions. The notebook itself cannot express the fast path.
- **Deployment.** Sharing a notebook means sharing a Python environment, a runtime, and a set of pinned dependencies. There is no single binary you can hand someone.
- **Concurrency.** The GIL means a notebook cell cannot use threads for real parallelism without multiprocessing hacks.
- **Type safety.** Notebook code is dynamically typed, so errors surface at runtime, during execution, after you have already run the cells above.

## Project Mars vs Jupyter

Project Mars keeps the notebook format but makes Rust the first-class language. Every cell you write is real Rust, compiled and executed in a persistent session, with variables shared across cells just like in Jupyter.

|                  | Jupyter                   | Project Mars                                   |
| ---------------- | ------------------------- | -------------------------------------- |
| Primary language | Python (interpreted)      | Rust (compiled)                        |
| Execution model  | Kernel process            | Persistent Rust session                |
| Performance      | GC + interpreter overhead | Native binary, no GC                   |
| Memory safety    | Runtime errors            | Compile-time guarantees                |
| Concurrency      | GIL-limited               | True parallelism                       |
| Deployment       | Runtime + environment     | Single binary via CLI                  |
| Format           | `.ipynb`                  | `.mars` (JSON)                         |
| Interop          | —                         | Import/export `.ipynb`                 |
| Outputs          | Rich (HTML, plots, LaTeX) | Rich (HTML, plots, LaTeX, SVG, images) |

Project Mars is not a replacement for Python. It is a replacement for the *notebook runtime*. If you want to keep using Python, you can still import `.ipynb` files and export them. But if you want to write the notebook itself in a language that is fast, safe, and reproducible, Rust is the answer.

## Why we built Project Mars

Three motivations drove the project:

1. **Teaching.** A Rust notebook can show students the language in a runnable, explorable format. Variables persist across cells, so you can experiment with ownership, iterators, and generics the way you would in a REPL, but with a document that stays saved and shareable.
2. **Data science and ML.** The Rust ecosystem — `polars`, `ndarray`, `linfa`, `plotters` — is maturing fast. Project Mars gives those libraries a notebook home where they can be explored interactively, without the overhead of a Python interpreter.
3. **Reproducibility.** A `.mars` notebook is a JSON document with stored outputs. Execute it with `mars notebook run`, and you get deterministic results. No environment drift, no missing dependencies, no "it worked on my machine."

Project Mars is open source, MIT licensed, and built as a modular Cargo workspace. The execution engine runs Rust code in an isolated process, so untrusted notebooks cannot touch the desktop application. The CLI works headless, so notebooks can be part of CI pipelines.

If you are curious, the [getting-started notebook](/docs/getting-started) is a good place to begin.
