---
title: "CatBoost vs catboost-rs: Categorical Gradient Boosting"
description: "Compare Python's CatBoost with Rust's catboost-rs for gradient boosting with strong categorical handling in notebook environments."
author: "Project Mars Team"
date: "2026-11-14"
order: 30
tags: ["machine-learning", "catboost", "rust", "python", "gradient-boosting", "mars"]
---

# CatBoost vs catboost-rs: Categorical Gradient Boosting Compared

CatBoost handles categorical features effectively and often performs well with minimal preprocessing. **`catboost-rs`** provides Rust access to CatBoost workflows, supporting practical ML tasks where categorical data is central.

## What CatBoost does

CatBoost provides a gradient boosting framework optimized for categorical features:

- **Automatic handling** of categorical features
- **Ordered boosting** to reduce overfitting
- **Fast training** on GPU
- **Cross-validation** and early stopping
- **Model interpretation** via feature importance
- **Production deployment** support

It became the preferred library for datasets with many categorical features, often outperforming other boosting libraries with minimal preprocessing.

## What catboost-rs brings to Rust

**`catboost-rs`** provides Rust bindings to CatBoost:

- **Gradient boosting** via CatBoost C API
- **Training and prediction** APIs
- **Categorical feature** handling
- **GPU support** for faster training
- **Type-safe** data interfaces
- **Performance** with native Rust data structures

The key difference: catboost-rs allows you to use CatBoost's categorical handling from Rust, combining CatBoost's strengths with Rust's type safety and deployment simplicity.

## Comparison

| Feature               | CatBoost (Python)                        | catboost-rs (Rust)                         |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Train model           | `model.fit(X, y, cat_features)`        | `train(pool, params)`                      |
| Predict               | `model.predict(X)`                       | `predict(&model, &pool)`                  |
| Categorical features  | Automatic detection                     | Specified via pool                       |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | Python + C++ backend                      | Rust + C++ backend                         |
| GPU support           | CUDA                                     | CUDA                                       |

## Example: CatBoost

```python
from catboost import CatBoostClassifier

model = CatBoostClassifier(cat_features=['cat1', 'cat2'])
model.fit(X, y)
```

## Example: catboost-rs

```rust
use catboost::{Pool, TrainParams};

let pool = Pool::from_features(features, cat_features)?;
let model = train(&params, &pool)?;
```

## When to use each

- **CatBoost** in Python when you need automatic categorical handling, GPU training, or integration with pandas and scikit-learn.
- **catboost-rs** in Rust when you want to use CatBoost from Rust, combine it with Rust's type system, or deploy models in Rust applications.

## How Project Mars supports both

Project Mars supports both CatBoost versions in their respective language runtimes:

- **Python cells** can use CatBoost with standard library imports
- **Rust cells** can use catboost-rs via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Models and datasets persist across cells within each language runtime

Start with the [getting started guide](/docs/01-getting-started) to explore CatBoost in Project Mars notebooks.
