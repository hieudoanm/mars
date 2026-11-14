---
title: "XGBoost vs xgboost-rs: Gradient Boosting Libraries"
description: "Compare Python's XGBoost with Rust's xgboost-rs for gradient-boosted decision trees in notebook environments."
author: "Mars Team"
date: "2026-11-14"
order: 28
tags: ["machine-learning", "xgboost", "rust", "python", "gradient-boosting", "mars"]
---

# XGBoost vs xgboost-rs: Gradient Boosting Compared

XGBoost is a widely used gradient boosting library for tabular data. **`xgboost-rs`** brings that power to Rust, making it possible to train and evaluate strong tree-based models where latency, throughput, and deployment simplicity matter.

## What XGBoost does

XGBoost provides an optimized gradient boosting framework:

- **Gradient boosting** with decision trees
- **Regularization** to prevent overfitting
- **Handling missing values** automatically
- **Cross-validation** and early stopping
- **GPU acceleration** for training
- **Deployment** to production environments

It became the go-to library for tabular ML competitions and production workloads.

## What xgboost-rs brings to Rust

**`xgboost-rs`** provides Rust bindings to XGBoost:

- **Gradient boosting** via XGBoost C API
- **Training and prediction** APIs
- **Cross-validation** support
- **GPU support** via CUDA
- **Type-safe** data handling
- **Performance** with native Rust data structures

The key difference: xgboost-rs allows you to use XGBoost's proven gradient boosting from Rust, combining XGBoost's performance with Rust's type safety and deployment simplicity.

## Comparison

| Feature               | XGBoost (Python)                         | xgboost-rs (Rust)                          |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| Train model           | `model.fit(X, y)`                       | `booster.train(&dtrain)`                  |
| Predict               | `model.predict(X)`                      | `booster.predict(&dmatrix)`               |
| Cross-validation      | `xgb.cv(dtrain, params)`                | Native support                            |
| Type safety           | Runtime                                  | Compile-time                               |
| Performance           | Python + C++ backend                      | Rust + C++ backend                         |
| GPU support           | CUDA                                     | CUDA                                       |

## Example: XGBoost

```python
import xgboost as xgb

dtrain = xgb.DMatrix(X, label=y)
params = {'max_depth': 3, 'eta': 0.1}
model = xgb.train(params, dtrain)
```

## Example: xgboost-rs

```rust
use xgboost::{parameters, Booster, DMatrix};

let dtrain = DMatrix::from_dense(&features)?;
let booster = Booster::train(&params, &dtrain)?;
```

## When to use each

- **XGBoost** in Python when you need the full XGBoost ecosystem, rapid prototyping, or integration with pandas and scikit-learn.
- **xgboost-rs** in Rust when you want to use XGBoost from Rust, combine it with Rust's type system, or deploy models in Rust applications.

## How Mars supports both

Mars supports both XGBoost versions in their respective language runtimes:

- **Python cells** can use XGBoost with standard library imports
- **Rust cells** can use xgboost-rs via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Models and datasets persist across cells within each language runtime

Start with the [getting started guide](/docs/01-getting-started) to explore XGBoost in Mars notebooks.
