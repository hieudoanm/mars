---
title: '10 Machine Learning Libraries Every Developer Should Know'
description: 'A practical map of 10 machine learning libraries in Python and their Rust equivalents, and how Project Mars supports both ML ecosystems fully.'
author: 'Project Mars Team'
date: '2026-11-06'
order: 23
tags: ['machine-learning', 'rust', 'python', 'libraries', 'mars']
---

# 10 Machine Learning Libraries that Every Developers Should Know

Machine learning in Python benefits from a rich, familiar set of libraries. Rust now provides capable equivalents that emphasize performance, safety, and predictable deployment. Project Mars brings both Python and Rust libraries into an interactive notebook environment so developers can experiment, iterate, and share results in their preferred language.

## Python to Rust at a glance

| Python library | Rust equivalent | What it does                                                         |
| -------------- | --------------- | -------------------------------------------------------------------- |
| scikit-learn   | `linfa`         | Classical machine learning algorithms and pipelines                  |
| PyTorch        | `tch-rs`        | Deep learning with tensor operations and training loops              |
| TensorFlow     | `tensorflow`    | Graph-based and production-oriented ML workflows                     |
| Keras          | `burn`          | Ergonomic model building and training abstractions                   |
| XGBoost        | `xgboost-rs`    | Gradient-boosted decision trees                                      |
| LightGBM       | `lightgbm`      | Fast gradient boosting for tabular data                              |
| CatBoost       | `catboost-rs`   | Gradient boosting with strong categorical handling                   |
| JAX            | `dfdx`          | Composable array/tensor transformations and autodiff-style workflows |
| Transformers   | `rust-bert`     | Pretrained transformer models for NLP tasks                          |
| OpenCV         | `opencv`        | Computer vision, image processing, and video I/O                     |

## 1. scikit-learn → `linfa`

scikit-learn is the standard toolkit for classical ML in Python. **`linfa`** aims to provide a similar experience in Rust, with algorithms for clustering, classification, preprocessing, and model evaluation. Its Rust-native design makes it a strong fit for fast, type-safe experimentation in notebooks.

See [scikit-learn vs linfa](/blog/machine-learning-library-scikit-learn-vs-linfa) for a detailed comparison.

## 2. PyTorch → `tch-rs`

PyTorch is popular for research and production deep learning. **`tch-rs`** provides Rust bindings to the Torch ecosystem, enabling tensor computation and neural network workflows from Rust code. In Project Mars, developers can prototype model logic interactively while staying in a compiled language.

See [PyTorch vs tch-rs](/blog/machine-learning-library-pytorch-vs-tch-rs) for a detailed comparison.

## 3. TensorFlow → `tensorflow`

TensorFlow supports large-scale training and deployment scenarios. The Rust **`tensorflow`** crate exposes TensorFlow capabilities to Rust applications, allowing developers to integrate established ML graphs and inference workflows into a Rust-first environment.

See [TensorFlow Python vs Rust](/blog/machine-learning-library-tensorflow-vs-tensorflow) for a detailed comparison.

## 4. Keras → `burn`

Keras is known for its approachable, high-level API. **`burn`** offers an ergonomic Rust framework for building and training models with a modern developer experience. It helps bridge the gap between low-level tensor control and productive model development.

See [Keras vs burn](/blog/machine-learning-library-keras-vs-burn) for a detailed comparison.

## 5. XGBoost → `xgboost-rs`

XGBoost is a widely used gradient boosting library for tabular data. **`xgboost-rs`** brings that power to Rust, making it possible to train and evaluate strong tree-based models where latency, throughput, and deployment simplicity matter.

See [XGBoost vs xgboost-rs](/blog/machine-learning-library-xgboost-vs-xgboost-rs) for a detailed comparison.

## 6. LightGBM → `lightgbm`

LightGBM is valued for speed and efficiency on large tabular datasets. The **`lightgbm`** Rust bindings enable Rust developers to use this proven boosting approach in data pipelines and notebook experiments without leaving the Rust ecosystem.

See [LightGBM Python vs Rust](/blog/machine-learning-library-lightgbm-vs-lightgbm) for a detailed comparison.

## 7. CatBoost → `catboost-rs`

CatBoost handles categorical features effectively and often performs well with minimal preprocessing. **`catboost-rs`** provides Rust access to CatBoost workflows, supporting practical ML tasks where categorical data is central.

See [CatBoost vs catboost-rs](/blog/machine-learning-library-catboost-vs-catboost-rs) for a detailed comparison.

## 8. JAX → `dfdx`

JAX combines array programming with composable transformations. **`dfdx`** offers a Rust approach to tensor and differentiable programming patterns, giving developers a path to experiment with modern ML building blocks in a safe, compiled setting.

See [JAX vs dfdx](/blog/machine-learning-library-jax-vs-dfdx) for a detailed comparison.

## 9. Transformers → `rust-bert`

The Python Transformers ecosystem made pretrained NLP models widely accessible. **`rust-bert`** provides Rust implementations and bindings for transformer-based models, enabling tasks like classification, embeddings, and generation in Rust applications and notebooks.

See [Transformers vs rust-bert](/blog/machine-learning-library-transformers-vs-rust-bert) for a detailed comparison.

## 10. OpenCV → `opencv`

OpenCV is a foundational computer vision library. The Rust **`opencv`** crate exposes OpenCV functionality for image processing, feature detection, and video workflows, making it a natural companion for ML systems that operate on visual data.

See [OpenCV Python vs Rust](/blog/machine-learning-library-opencv-vs-opencv) for a detailed comparison.

## How Project Mars supports both ecosystems

Project Mars is built to make ML development in Python, Rust, Julia, and C++ interactive, reproducible, and practical:

- **Persistent sessions.** Train, evaluate, and iterate across notebook cells while keeping variables and intermediate results available in your preferred language.
- **Notebook-native workflow.** Combine code, Markdown explanations, equations, tables, and rich outputs in a single `.mars` document.
- **Rust ecosystem.** Use Rust crates such as `linfa`, `tch-rs`, `tensorflow`, `burn`, `xgboost-rs`, `lightgbm`, `catboost-rs`, `dfdx`, `rust-bert`, and `opencv` with Cargo dependency support.
- **Python ecosystem.** Use standard Python ML libraries (scikit-learn, PyTorch, TensorFlow, etc.) with matplotlib figure capture and top-level `await` support.
- **Julia ecosystem.** Use Julia ML libraries (Flux.jl, MLJ.jl, ScikitLearn.jl) with MIME bundle output and interactive input support.
- **C++ ecosystem.** Use C++ ML libraries via Conan/vcpkg with replay-based compilation.
- **Reproducibility and automation.** Save notebooks as JSON, preserve outputs, and run them headlessly with the Project Mars CLI for repeatable experiments.
- **Jupyter interoperability.** Import and export `.ipynb` files to bridge existing notebook workflows with Project Mars's multi-language runtime.

To get started with ML in Project Mars notebooks, open the [getting started guide](/docs/getting-started), explore the [runtime documentation](/docs/02-platform/runtime.md), and try the [machine learning example](https://github.com/hieudoanm/mars/tree/main/examples/mars/02-machine-learning) which demonstrates a complete ML workflow in Project Mars.
