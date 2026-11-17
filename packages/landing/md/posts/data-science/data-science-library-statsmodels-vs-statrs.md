---
title: 'Statsmodels vs statrs: Statistical Modeling in Python vs Rust'
description: "Compare Python's Statsmodels with Rust's statrs for statistical distributions, tests, and models in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 16
tags: ['data-science', 'statsmodels', 'statrs', 'rust', 'python', 'mars']
---

# Statsmodels vs statrs: Statistical Modeling Compared

Statsmodels provides statistical modeling and inference tools in Python. Rust's **`statrs`** supplies implementations of common statistical distributions, functions, and utilities for analysis, simulation, and modeling.

## What Statsmodels does

Statsmodels extends Python's scientific stack with:

- **Statistical models** (linear regression, logistic regression, time series)
- **Hypothesis tests** (t-tests, ANOVA, chi-square)
- **Probability distributions** and density functions
- **Descriptive statistics** and summary functions
- **Time series analysis** (ARIMA, VAR)
- **Regression diagnostics** and model validation

It became the go-to library for statistical inference in Python, widely used in academia and industry.

## What statrs brings to Rust

**`statrs`** is a Rust library for statistical computing:

- **Probability distributions** (normal, beta, gamma, exponential, etc.)
- **Statistical functions** (mean, variance, skewness, kurtosis)
- **Hypothesis tests** (t-tests, chi-square)
- **Random sampling** from distributions
- **Regression** utilities
- **Compile-time type safety** for statistical operations

The key difference: statrs provides statistical primitives in a type-safe, compiled environment, making it suitable for simulation, analysis, and modeling tasks in Rust.

## Comparison

| Feature         | Statsmodels (Python)            | statrs (Rust)                           |
| --------------- | ------------------------------- | --------------------------------------- |
| Distribution    | `stats.norm.cdf(x)`             | `Normal::new(0.0, 1.0).unwrap().cdf(x)` |
| Random sampling | `np.random.normal()`            | `Normal::sample(&mut rng)`              |
| Hypothesis test | `stats.ttest_ind(a, b)`         | `test::t_test_independent(a, b)`        |
| Type safety     | Runtime                         | Compile-time                            |
| Performance     | C-backed via NumPy              | Native compilation                      |
| Integration     | NumPy, pandas, Matplotlib, etc. | ndarray, rand, etc.                     |

## Example: Statsmodels

```python
import numpy as np
from scipy import stats

data = np.array([1, 2, 3, 4, 5])
result = stats.ttest_1samp(data, 3.0)
```

## Example: statrs

```rust
use statrs::distribution::{Normal, Continuous};
use statrs::statistics::Statistics;

let normal = Normal::new(0.0, 1.0).unwrap();
let x = 1.96;
let result = normal.cdf(x);
```

## When to use each

- **Statsmodels** in Python when you need a comprehensive statistical modeling suite, time series analysis, or integration with pandas and NumPy.
- **statrs** in Rust when you need type-safe statistical primitives, performance for simulations, or want to avoid the Python interpreter overhead.

## Try it in Project Mars

Both ecosystems share one `.mars` document: Python cells run in a persistent interpreter, Rust cells on the replay runtime with `// :dep` Cargo dependencies — variables persist across cells within each language's own session.

Descriptive statistics — the numbers behind every OLS summary — once per language:

```python
# statsmodels side: the full regression summary.
import numpy as np
import statsmodels.api as sm

x = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
y = 2.0 * x + 1.0 + np.array([0.1, -0.2, 0.0, 0.1, -0.1])
X = sm.add_constant(x)
sm.OLS(y, X).fit().summary()
```

```rust
// :dep ndarray = "0.16"

use ndarray::array;

// statrs provides the same moments; ndarray computes them eagerly.
let y = array![3.1, 5.8, 7.0, 9.1, 10.9];
let mean = y.mean().unwrap();
let var = y.var(0.0).unwrap();
println!("mean={mean:.3} var={var:.3} n={}", y.len());
```

## How Project Mars supports both

Project Mars supports both Statsmodels and statrs in their respective language runtimes:

- **Python cells** can use Statsmodels with standard library imports
- **Rust cells** can use statrs via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Statistical results persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore Statsmodels and statrs in Project Mars notebooks.
