---
title: 'Customer Churn Prediction: Classification Project'
description: 'Predict which customers will cancel their subscription using logistic regression with class weighting and threshold tuning in Rust.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 40
tags: ['machine-learning', 'rust', 'classification', 'churn', 'logistic-regression', 'business']
---

# Machine Learning Project: Customer Churn Prediction

**Churn prediction** is the canonical business ML project: given customer
usage data, predict who will cancel next month so the retention team can
intervene. The Telco Customer Churn dataset (~7,000 customers, ~20
features) is the standard benchmark.

In this project you will:

1. Frame churn as **binary classification on imbalanced data**.
2. Handle the class imbalance that breaks naive accuracy.
3. Train **logistic regression** and read its coefficients as business
   insights.
4. Tune the decision threshold for the actual business cost.

> Rust ML note: `polars` for the dataframe, `linfa-logistic` for the model,
> `linfa` evaluation helpers for metrics.

## Why churn is the perfect ML teaching project

It combines everything real ML work involves:

- **Imbalanced classes** — only ~26% of customers churn; accuracy is
  meaningless without precision/recall.
- **Mixed features** — tenure (numeric), contract type (categorical),
  monthly charges, support calls.
- **Business interpretation** — "why" matters as much as "how accurate".
  A model that says "contract_type=month-to-month → 3x churn risk" is
  actionable; a black box is not.

```rust
// :dep polars = { version = "0.44", features = ["lazy"] }
// :dep linfa = "0.7"
// :dep linfa-logistic = "0.7"
// :dep linfa-datasets = "0.7"

use polars::prelude::*;

// Telco Customer Churn (Kaggle)
let df = CsvReadOptions::default()
    .with_has_header(true)
    .try_into_reader_with_file_path(Some("WA_Fn-UseC_-Telco-Customer-Churn.csv".into()))
    .unwrap()
    .finish()
    .unwrap();

println!("shape: {:?}", df.shape());

// Target: Churn ∈ {Yes, No} → encode as 1/0
let churn_rate = df
    .column("Churn").unwrap()
    .str().unwrap()
    .into_iter()
    .filter(|s| s.map(|v| v == "Yes").unwrap_or(false))
    .count() as f64
    / df.height() as f64;

println!("churn rate: {:.1}%", churn_rate * 100.0);
```

## Feature engineering for churn

Raw columns aren't enough — churn has strong **interaction effects**:

- `IsNewCustomer = (tenure < 6)` — new customers churn at 2-3x the rate
- `HasSupportIssues = (TechSupport == "No")` — no support → higher churn
- `ContractRisk = contract_type == "month-to-month"` — the single
  strongest predictor (~40% of churners are month-to-month)

```rust
let df = df.lazy()
    .with_columns([
        (col("tenure").lt(lit(6)))
            .cast(DataType::Int8).alias("IsNewCustomer"),
        (col("Contract").eq(lit("Month-to-month")))
            .cast(DataType::Int8).alias("IsMonthToMonth"),
        (col("tenure") * col("MonthlyCharges"))
            .alias("TenureCharges"),   // total spend proxy
    ])
    .collect()
    .unwrap();

// One-hot encode categoricals (same as house-price project)
// then standardize numeric features for logistic regression
```

## Why accuracy fails on imbalanced data

With 26% churn, a model that predicts "no churn" for _everyone_ gets
**74% accuracy** — and is useless. The metrics that matter:

| Metric          | Meaning for churn                                                                     |
| --------------- | ------------------------------------------------------------------------------------- |
| **Precision**   | Of customers flagged "will churn", how many actually do? (retention spend efficiency) |
| **Recall**      | Of actual churners, how many did we catch? (revenue saved)                            |
| **F1 / PR-AUC** | Balanced view; PR-AUC is the gold standard for imbalanced data                        |

You'd rather flag 100 customers and save 60 than flag 1,000 and annoy 940
loyal ones — precision matters, but so does catching enough churners.

```rust
// sklearn: class_weight='balanced' — upweights minority class in loss
// linfa: sample weights via Dataset::with_weights

let pos_weight = (1.0 - churn_rate) / churn_rate;   // ≈ 2.8
let weights: Vec<f64> = y_train.iter()
    .map(|&t| if t == 1 { pos_weight } else { 1.0 })
    .collect();

// Train logistic regression on weighted samples
let model = LogisticRegression::default()
    .fit_with_weights(&train, &weights)
    .expect("fit failed");

// predict_proba gives P(churn) — the raw material for threshold tuning
```

## Threshold tuning — where the business value lives

Logistic regression outputs a probability, not a label. The default 0.5
threshold is almost never optimal for imbalanced business problems.

Sweep the threshold and plot precision/recall:

| Threshold | Precision | Recall   | Interpretation                          |
| --------- | --------- | -------- | --------------------------------------- |
| 0.3       | lower     | higher   | aggressive retention, more false alarms |
| 0.5       | balanced  | balanced | default                                 |
| 0.7       | higher    | lower    | only flag high-confidence churners      |

The right threshold depends on the cost of a retention offer vs. the cost
of losing a customer — a business decision, not a technical one.

```rust
fn sweep_thresholds(probs: &[f64], labels: &[u8],
                     thresholds: &[f64]) -> Vec<(f64,f64,f64)> {
    thresholds.iter().map(|&t| {
        let preds: Vec<bool> = probs.iter().map(|&p| p >= t).collect();
        let labels_b: Vec<bool> = labels.iter().map(|&l| l == 1).collect();
        let (p, r, f1) = precision_recall(&preds, &labels_b);
        (t, p, r)
    }).collect()
}

// Find the threshold maximizing F1 (or precision at fixed recall)
let results = sweep_thresholds(&val_probs, &val_labels,
                               &(0.1..0.9).step_by(0.05).collect::<Vec<_>>());
let best = results.iter()
    .max_by(|a, b| (a.1 * a.2).partial_cmp(&(b.1 * b.2)).unwrap())
    .unwrap();
println!("best threshold: {:.2} → precision {:.2}, recall {:.2}",
         best.0, best.1, best.2);
```

## Reading the coefficients — the "why"

Logistic regression is interpretable: each coefficient is the log-odds
contribution of that feature. Exponentiate for odds ratios:

```
contract_type = month-to-month  →  odds_ratio ≈ 3.5
                   (month-to-month customers are ~3.5x more likely to churn,
                    holding everything else constant)

total_charges (per $1000)       →  odds_ratio ≈ 0.95
                   (higher spend slightly *reduces* churn — invested customers)
```

This is why logistic regression remains the go-to for churn: the business
can act on "push annual contracts" because the model _told them why_.

```rust
// Inspect learned coefficients
for (name, coef) in model.params().feature_weights().iter()
    .zip(feature_names.iter()) {
    let odds_ratio = coef.exp();
    println!("{:<25} coef={:+.3}  odds_ratio={:.2}",
             name, coef, odds_ratio);
}

// Sort by |coef| to find the top churn drivers
```

## Results

| Model                       | ROC-AUC | PR-AUC | Best-F1 threshold |
| --------------------------- | ------- | ------ | ----------------- |
| Logistic regression         | ~0.85   | ~0.75  | ~0.45             |
| Gradient boosting (XGBoost) | ~0.88   | ~0.80  | ~0.40             |
| Random forest               | ~0.87   | ~0.78  | ~0.42             |

Logistic regression is within 3 points of boosting — and interpretable.
For a churn use-case where the retention team needs to _understand_ the
risk factors, that's the right trade-off.

## Key takeaways

1. **Imbalanced data breaks accuracy** — use precision/recall/PR-AUC.
2. **Class weighting** fixes the imbalance without throwing away data.
3. **Threshold ≠ 0.5** — tune it on validation data for the business cost.
4. **Interpretable models win in business ML** — churn teams need reasons,
   not just scores.

**Next project:** demand forecasting — where the target is a continuous
number varying over time, not a binary label.
