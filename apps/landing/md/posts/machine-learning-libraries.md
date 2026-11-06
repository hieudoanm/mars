---
title: "10 Machine Learning Libraries that Every Developers Should Know"
description: "A practical map of 10 machine learning libraries in Python and their Rust equivalents, and how Mars supports the Rust ML stack fully."
author: "Mars Team"
date: "2026-11-06"
order: 4
tags: ["machine-learning", "rust", "python", "libraries", "mars"]
---

# 10 Machine Learning Libraries that Every Developers Should Know

Machine learning in Python benefits from a rich, familiar set of libraries. Rust now provides capable equivalents that emphasize performance, safety, and predictable deployment. Mars brings those Rust libraries into an interactive notebook environment so developers can experiment, iterate, and share results in a Rust-first workflow.

## Python to Rust at a glance

| Python library | Rust equivalent | What it does |
| --- | --- | --- |
| scikit-learn | `linfa` | Classical machine learning algorithms and pipelines |
| PyTorch | `tch-rs` | Deep learning with tensor operations and training loops |
| TensorFlow | `tensorflow` | Graph-based and production-oriented ML workflows |
| Keras | `burn` | Ergonomic model building and training abstractions |
| XGBoost | `xgboost-rs` | Gradient-boosted decision trees |
| LightGBM | `lightgbm` | Fast gradient boosting for tabular data |
| CatBoost | `catboost-rs` | Gradient boosting with strong categorical handling |
| JAX | `dfdx` | Composable array/tensor transformations and autodiff-style workflows |
| Transformers | `rust-bert` | Pretrained transformer models for NLP tasks |
| OpenCV | `opencv` | Computer vision, image processing, and video I/O |

## 1. scikit-learn → `linfa`

scikit-learn is the standard toolkit for classical ML in Python. **`linfa`** aims to provide a similar experience in Rust, with algorithms for clustering, classification, preprocessing, and model evaluation. Its Rust-native design makes it a strong fit for fast, type-safe experimentation in notebooks.

## 2. PyTorch → `tch-rs`

PyTorch is popular for research and production deep learning. **`tch-rs`** provides Rust bindings to the Torch ecosystem, enabling tensor computation and neural network workflows from Rust code. In Mars, developers can prototype model logic interactively while staying in a compiled language.

## 3. TensorFlow → `tensorflow`

TensorFlow supports large-scale training and deployment scenarios. The Rust **`tensorflow`** crate exposes TensorFlow capabilities to Rust applications, allowing developers to integrate established ML graphs and inference workflows into a Rust-first environment.

## 4. Keras → `burn`

Keras is known for its approachable, high-level API. **`burn`** offers an ergonomic Rust framework for building and training models with a modern developer experience. It helps bridge the gap between low-level tensor control and productive model development.

## 5. XGBoost → `xgboost-rs`

XGBoost is a widely used gradient boosting library for tabular data. **`xgboost-rs`** brings that power to Rust, making it possible to train and evaluate strong tree-based models where latency, throughput, and deployment simplicity matter.

## 6. LightGBM → `lightgbm`

LightGBM is valued for speed and efficiency on large tabular datasets. The **`lightgbm`** Rust bindings enable Rust developers to use this proven boosting approach in data pipelines and notebook experiments without leaving the Rust ecosystem.

## 7. CatBoost → `catboost-rs`

CatBoost handles categorical features effectively and often performs well with minimal preprocessing. **`catboost-rs`** provides Rust access to CatBoost workflows, supporting practical ML tasks where categorical data is central.

## 8. JAX → `dfdx`

JAX combines array programming with composable transformations. **`dfdx`** offers a Rust approach to tensor and differentiable programming patterns, giving developers a path to experiment with modern ML building blocks in a safe, compiled setting.

## 9. Transformers → `rust-bert`

The Python Transformers ecosystem made pretrained NLP models widely accessible. **`rust-bert`** provides Rust implementations and bindings for transformer-based models, enabling tasks like classification, embeddings, and generation in Rust applications and notebooks.

## 10. OpenCV → `opencv`

OpenCV is a foundational computer vision library. The Rust **`opencv`** crate exposes OpenCV functionality for image processing, feature detection, and video workflows, making it a natural companion for ML systems that operate on visual data.

## How Mars supports the Rust equivalents fully

Mars is built to make Rust ML development interactive, reproducible, and practical:

- **Persistent Rust sessions.** Train, evaluate, and iterate across notebook cells while keeping variables and intermediate results available.
- **Notebook-native workflow.** Combine code, Markdown explanations, equations, tables, and rich outputs in a single `.mars` document.
- **Cargo-compatible ecosystem.** Use Rust crates such as `linfa`, `tch-rs`, `tensorflow`, `burn`, `xgboost-rs`, `lightgbm`, `catboost-rs`, `dfdx`, `rust-bert`, and `opencv` as part of your workflow.
- **Reproducibility and automation.** Save notebooks as JSON, preserve outputs, and run them headlessly with the Mars CLI for repeatable experiments.
- **Jupyter interoperability.** Import and export `.ipynb` files to bridge existing notebook workflows with Mars's Rust-first runtime.

To get started with Rust ML in notebooks, open the [getting started guide](/docs/01-getting-started) and explore the [runtime documentation](/docs/04-runtime).
