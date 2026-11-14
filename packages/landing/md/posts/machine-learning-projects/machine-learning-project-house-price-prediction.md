---
title: "House Price Prediction: Regression ML Project"
description: "Predict house sale prices from tabular features using linear regression and gradient boosting in Rust — the classic Kaggle regression project."
author: "Mars Team"
date: "2026-11-14"
order: 36
tags: ["machine-learning", "rust", "regression", "kaggle", "xgboost"]
---

# Machine Learning Project: House Price Prediction

**House Prices: Advanced Regression Techniques** is the most famous
regression competition on Kaggle: predict final sale prices for ~1,460 houses
in Ames, Iowa from ~80 mixed features — square footage, neighborhood, year
built, quality ratings, and more.

In this project you will:

1. Handle mixed numeric/categorical features — the core tabular ML skill.
2. Engineer features (ratios, binned categories, missing-value flags).
3. Train **linear regression** and a **gradient-boosted tree** model.
4. Evaluate with RMSE and learn why boosting wins on tabular data.

> Rust ML note: `polars` for dataframes, `ndarray` for arrays,
> `smartcore`/`linfa` or `xgboost-rs` for models.

## Why this project matters

Tabular data is what most real ML jobs look like — not images, not text.
House prices teach the unglamorous 80% of ML work: missing values,
categorical encodings, skewed targets, and feature engineering. Every trick
here transfers directly to churn, fraud, demand, and revenue projects.

```rust
// :dep polars = { version = "0.44", features = ["lazy", "ndarray"] }
// :dep ndarray = "0.16"

use polars::prelude::*;

// Load the Ames housing CSV (Kaggle 'House Prices' competition)
let df = CsvReadOptions::default()
    .with_has_header(true)
    .try_into_reader_with_file_path(Some("train.csv".into()))
    .unwrap()
    .finish()
    .unwrap();

println!("shape: {:?}", df.shape());
println!("columns: {}", df.width());

// Missing-value audit — the first thing you do on any tabular dataset
let missing: Vec<_> = df
    .get_column_names()
    .iter()
    .map(|name| {
        let nulls = df.column(name).unwrap().null_count();
        (*name, nulls)
    })
    .filter(|(_, n)| *n > 0)
    .collect();

println!("columns with missing values: {}", missing.len());
for (name, n) in missing.iter().take(10) {
    println!("  {}: {}", name, n);
}
```

## Target inspection

`SalePrice` is heavily right-skewed — a few mansion outliers stretch the
distribution. Models predict better on `log(SalePrice)`, so we train on the
log and exponentiate predictions back. This single trick often halves RMSE.

```rust
use polars::prelude::col;

let df = df.lazy()
    .with_column(
        (col("SalePrice").cast(DataType::Float64).log())
            .alias("log_price"),
    )
    .collect()
    .unwrap();

// sklearn: df['SalePrice'].describe()
let stats = df
    .column("log_price")
    .unwrap()
    .f64()
    .unwrap()
    .describe(None)
    .unwrap();
println!("log_price stats:\n{:?}", stats);
```

## Categorical encoding

Columns like `Neighborhood`, `Exterior1st`, and `SaleCondition` are strings.
Linear models need numbers, so we **one-hot encode** them — one binary
column per category. High-cardinality columns (e.g. `Neighborhood` with 25
values) stay manageable; truly free-text columns would need hashing instead.

```rust
// sklearn: OneHotEncoder(handle_unknown='ignore') + ColumnTransformer
let categorical = [
    "MSSubClass", "MSZoning", "Neighborhood", "Exterior1st",
    "Exterior2nd", "SaleType", "SaleCondition",
];

let df = df.lazy()
    .with_columns([
        // Fill missing categoricals with a sentinel, then one-hot
        col("GarageType").fill_null(lit("None")).alias("GarageType"),
        col("FireplaceQuarts").fill_null(lit(0)).alias("FireplaceQuarts"),
    ])
    .collect()
    .unwrap();

// polars get_dummies ~ pandas.get_dummies
let encoded = df
    .lazy()
    .select([
        all().exclude(categorical.iter().copied()),
        categorical
            .iter()
            .map(|c| col(*c).cast(DataType::String))
            .collect::<Vec<_>>(),
    ])
    .collect()
    .unwrap();

println!("encoded shape: {:?}", encoded.shape());
```

## Feature engineering

The best models on this dataset come from domain features, not algorithms:

- `TotalSF = TotalBsmtSF + 1stFlrSF + 2ndFlrSF` — total living area
- `HouseAge = YrSold - YearBuilt`
- `IsRemodeled = YearRemodAdd != YearBuilt`
- `QualCond = OverallQual * OverallCond` — interaction term

```rust
let df = df.lazy()
    .with_columns([
        (col("TotalBsmtSF") + col("1stFlrSF") + col("2ndFlrSF"))
            .alias("TotalSF"),
        (col("YrSold") - col("YearBuilt")).alias("HouseAge"),
        (col("YearRemodAdd") - col("YearBuilt"))
            .gt_eq(lit(1))
            .cast(DataType::Int32)
            .alias("IsRemodeled"),
    ])
    .collect()
    .unwrap();

// sklearn: PolynomialFeatures / manual interactions
// xgboost does NOT need these — trees find interactions themselves.
// That is the key practical difference between linear and GBM models.
```

## Model 1 — ridge regression

Ridge regression is linear regression with L2 regularization: it shrinks
coefficients toward zero, which stabilizes predictions when features are
correlated (square footage and room counts usually are).

```rust
// smartcore::linear::ridge_regression::RidgeRegressor
// or linfa-linear with L2 penalty
use ndarray::Array2;

// X: (n_samples, n_features) standardized features
// y: log_price target
// Solve (X'X + lambda*I) w = X'y  — the ridge normal equation
fn ridge_fit(x: &Array2<f64>, y: &ndarray::Array1<f64>, lambda: f64)
    -> ndarray::Array1<f64>
{
    let n_features = x.ncols();
    let xt_x = x.t().dot(x);
    let mut a = xt_x;
    for i in 0..n_features {
        a[[i, i]] += lambda;
    }
    let xt_y = x.t().dot(y);
    // In production: a.solve(xt_y) via ndarray-linalg
    ndarray_linalg::solve::Inverse::inv(&a).unwrap().dot(&xt_y)
}

// RMSE on log scale, then convert back:
// rmse_price = sqrt(mean((exp(pred_log) - exp(y_log))^2))
```

## Model 2 — gradient boosting

**XGBoost/LightGBM-style** gradient boosting builds trees sequentially,
each correcting the residuals of the ensemble. It dominates tabular
benchmarks because it:

- handles non-linear relationships automatically,
- needs no feature scaling or one-hot encoding for categoricals,
- is robust to outliers and missing values.

```rust
// xgboost-rs or lightgbm crate
// DMatrix is XGBoost's internal columnar format
//
// let dtrain = DMatrix::from_dense_matrix(x_train)?;
// let params = [
//     ("objective", "reg:squarederror"),
//     ("max_depth", "6"),
//     ("eta", "0.05"),       // learning rate
//     ("subsample", "0.8"),
// ];
// let booster = TrainParams::new(params).fit(&dtrain, &dvalid)?;
//
// Typical Kaggle scores (log-RMSE):
//   ridge regression ......... ~0.13
//   XGBoost default .......... ~0.12
//   tuned XGBoost ............ ~0.11
//
// The winning margin comes from hyperparameter search
// (max_depth, eta, subsample, colsample) via cross-validation —
// not from a cleverer algorithm.
```

## Evaluation — RMSE

Kaggle scores this competition on **RMSE of log-price**. We report both
scales because exp-space RMSE is what dollars actually cost you.

| Model | Log-RMSE | Approx. $ RMSE |
| ----- | -------- | -------------- |
| Mean predictor (baseline) | ~0.40 | ~$55k |
| Ridge regression | ~0.13 | ~$18k |
| Tuned XGBoost | ~0.11 | ~$15k |

A good sanity check: your model should beat the mean baseline by 3x or more.
If it doesn't, suspect a data-leakage bug or an encoding mistake — not the
algorithm.

```rust
// Baseline: predict the training mean for every house
let mean_log = y_train.mean().unwrap();
let baseline_rmse = {
    let se: f64 = y_test
        .iter()
        .map(|t| (mean_log - t).powi(2))
        .sum::<f64>()
        / y_test.len() as f64;
    se.sqrt()
};
println!("baseline log-RMSE: {:.4}", baseline_rmse);

// Model: predict log_price, exponentiate, compute dollar RMSE
let preds_log = ridge_predict(&weights, &x_test);
let dollar_rmse = preds_log
    .iter()
    .zip(y_test.iter())
    .map(|(p, t)| (p.exp() - t.exp()).powi(2))
    .sum::<f64>()
    .sqrt()
    / (y_test.len() as f64).sqrt();
println!("model $ RMSE: {:.0}", dollar_rmse);
```

## Key takeaways

1. **Tabular ML is 80% data work.** Encoding, missing values, and feature
   engineering matter more than model choice.
2. **Log-transform skewed targets** — cheap, huge win.
3. **Linear models need feature engineering; trees don't.** That is why
   gradient boosting dominates Kaggle tabular competitions.
4. **Always build the mean-predictor baseline first.** It tells you whether
   your pipeline works at all.

**Next project:** the same pipeline — load, clean, encode, split, baseline,
model, evaluate — applied to customer churn, where the target is a class
instead of a price.
