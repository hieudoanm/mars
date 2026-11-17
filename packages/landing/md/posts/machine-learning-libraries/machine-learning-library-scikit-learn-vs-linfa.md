---
title: 'scikit-learn vs linfa: Classical ML in Python vs Rust'
description: "Compare Python's scikit-learn with Rust's linfa for classical machine learning algorithms and pipelines in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 24
tags: ['machine-learning', 'scikit-learn', 'linfa', 'rust', 'python', 'mars']
---

# scikit-learn vs linfa: Classical ML Compared

scikit-learn is the standard toolkit for classical ML in Python. Rust's **`linfa`** aims to provide a similar experience with algorithms for clustering, classification, preprocessing, and model evaluation—designed for fast, type-safe experimentation in notebooks.

## What scikit-learn does

scikit-learn provides a unified interface for classical ML:

- **Classification**: SVM, random forest, logistic regression, k-NN
- **Regression**: linear regression, ridge, lasso, decision trees
- **Clustering**: k-means, DBSCAN, hierarchical clustering
- **Preprocessing**: scaling, encoding, dimensionality reduction
- **Model selection**: cross-validation, grid search
- **Metrics**: accuracy, precision, recall, F1-score

It became the default library for classical ML in Python, used across industry and academia.

## What linfa brings to Rust

**`linfa`** is a Rust machine learning library:

- **Algorithms**: k-means, hierarchical clustering, PCA, logistic regression
- **Preprocessing**: scaling, encoding, feature extraction
- **Model evaluation**: cross-validation, metrics
- **Type-safe APIs** that catch errors at compile time
- **Modular design** with trait-based abstractions
- **Performance** via native compilation

The key difference: linfa provides a Rust-native approach to classical ML with compile-time type safety and performance benefits, while scikit-learn offers a broader algorithm set with mature ecosystem integration.

## Comparison

| Feature            | scikit-learn (Python)           | linfa (Rust)                   |
| ------------------ | ------------------------------- | ------------------------------ |
| Train model        | `model.fit(X, y)`               | `model.fit(&dataset)`          |
| Predict            | `model.predict(X)`              | `model.predict(&features)`     |
| K-means clustering | `KMeans(n_clusters=3)`          | `KMeans::new_with_params(3)`   |
| Type safety        | Runtime                         | Compile-time                   |
| Performance        | C-backed via NumPy/scipy        | Native compilation             |
| Integration        | NumPy, pandas, Matplotlib, etc. | ndarray, plotters, serde, etc. |

## Example: scikit-learn

```python
from sklearn.cluster import KMeans

kmeans = KMeans(n_clusters=3)
kmeans.fit(X)
labels = kmeans.predict(X)
```

## Example: linfa

```rust
use linfa_clustering::KMeans;

let kmeans = KMeans::params(3).fit(&dataset)?;
let labels = kmeans.predict(&dataset);
```

## When to use each

- **scikit-learn** in Python when you need the broadest algorithm set, mature ecosystem, or integration with pandas and NumPy.
- **linfa** in Rust when you need type-safe ML operations, performance for large datasets, or want to avoid the Python interpreter overhead.

## How Project Mars supports both

Project Mars supports both scikit-learn and linfa in their respective language runtimes:

- **Python cells** can use scikit-learn with standard library imports
- **Rust cells** can use linfa via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Models and datasets persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) and the [machine learning example](https://github.com/hieudoanm/mars/tree/main/examples/mars/02-machine-learning) to explore scikit-learn and linfa in Project Mars notebooks.
