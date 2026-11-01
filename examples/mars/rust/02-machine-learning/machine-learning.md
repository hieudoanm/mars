# Machine Learning with Rust Libraries

This notebook demonstrates a machine learning workflow using Rust libraries:
- polars for data manipulation
- ndarray for numerical operations
- linfa for machine learning

Similar to the Python workflow using pandas, numpy, and scikit-learn.

```rust
// Note: These dependencies would be added to Cargo.toml in a real project
// :dep polars = { version = "0.44", features = ["lazy", "ndarray"] }
// :dep ndarray = { version = "0.16", features = ["rand"] }
// :dep ndarray-rand = "0.15"
// :dep linfa = "0.7"
// :dep linfa-logistic = "0.7"
// :dep rand = "0.8"

// For now, we'll use standard library to demonstrate the concept
// since Mars runtime doesn't yet support external dependencies
```

## 1. Generate Synthetic Dataset

We'll create a binary classification dataset similar to sklearn's `make_classification`.

```rust
use rand::Rng;

let n_samples: usize = 1000;
let n_features: usize = 20;

// Generate random features using standard library
let mut rng = rand::thread_rng();
let mut x: Vec<Vec<f64>> = Vec::new();
let mut y: Vec<i32> = Vec::new();

for _ in 0..n_samples {
    let mut row: Vec<f64> = Vec::new();
    for _ in 0..n_features {
        row.push(rng.gen_range(-10.0..10.0));
    }
    let sum: f64 = row.iter().sum();
    x.push(row);
    y.push(if sum > 0.0 { 1 } else { 0 });
}

println!("Dataset shape: ({}, {})", n_samples, n_features);
println!("Class 0: {}", y.iter().filter(|&&v| v == 0).count());
println!("Class 1: {}", y.iter().filter(|&&v| v == 1).count());

// In a full implementation with polars, this would be:
// let df = DataFrame::new(x, feature_names.clone())?;
// df.with_column(Series::new("target", y)?);
```

```text
Dataset shape: (1000, 20)
Class 0: 476
Class 1: 524
```

## 2. Train/Test Split

Similar to sklearn's `train_test_split`.

```rust
let split_point: usize = (n_samples * 8) / 10;

let x_train: Vec<Vec<f64>> = x[..split_point].to_vec();
let x_test: Vec<Vec<f64>> = x[split_point..].to_vec();
let y_train: Vec<i32> = y[..split_point].to_vec();
let y_test: Vec<i32> = y[split_point..].to_vec();

println!("Training set size: {}", x_train.len());
println!("Test set size: {}", x_test.len());
println!("Train - Class 0: {}, Class 1: {}", 
    y_train.iter().filter(|&&v| v == 0).count(), 
    y_train.iter().filter(|&&v| v == 1).count());
println!("Test - Class 0: {}, Class 1: {}", 
    y_test.iter().filter(|&&v| v == 0).count(), 
    y_test.iter().filter(|&&v| v == 1).count());
```

```text
Dataset shape: (1000, 20)
Class 0: 513
Class 1: 487
Training set size: 800
Test set size: 200
Train - Class 0: 398, Class 1: 402
Test - Class 0: 115, Class 1: 85
```

## 3. Train Logistic Regression

Similar to sklearn's `LogisticRegression`. In a full implementation, this would use linfa-logistic.

```rust
// Simple logistic regression implementation
// In a full implementation, this would be:
// let model = LogisticRegression::default();
// let fitted_model = model.fit(&dataset)?;

// For now, a simplified version:
let mut w: Vec<f64> = (0..n_features).map(|_| rng.gen_range(-0.5..0.5)).collect();
let mut bias: f64 = 0.0;

let epochs: usize = 1000;
let learning_rate: f64 = 0.01;
let n: usize = x_train.len();

for _ in 0..epochs {
    let mut grad_w: Vec<f64> = vec![0.0; n_features];
    let mut grad_b: f64 = 0.0;
    
    for i in 0..n {
        let z: f64 = w.iter().zip(&x_train[i]).map(|(&w_j, &x_j)| w_j * x_j).sum::<f64>() + bias;
        let pred: f64 = 1.0 / (1.0 + (-z).exp());
        let error: f64 = pred - y_train[i] as f64;
        grad_b += error;
        for j in 0..n_features {
            grad_w[j] += error * x_train[i][j];
        }
    }
    
    for j in 0..n_features {
        w[j] -= learning_rate * grad_w[j] / n as f64;
    }
    bias -= learning_rate * grad_b / n as f64;
}

println!("Model trained successfully!");
println!("Feature weights (first 5): {:?}", &w[..5]);
println!("Bias: {:.4}", bias);
```

```text
Dataset shape: (1000, 20)
Class 0: 525
Class 1: 475
Training set size: 800
Test set size: 200
Train - Class 0: 421, Class 1: 379
Test - Class 0: 104, Class 1: 96
Model trained successfully!
Feature weights (first 5): [0.36227389711215635, 0.34972863789401754, 0.31047059356264134, 0.29989677513988805, 0.3290270782846827]
Bias: -0.0518
```

## 4. Make Predictions and Evaluate

Similar to sklearn's `accuracy_score` and `classification_report`.

```rust
let mut y_pred: Vec<i32> = Vec::new();

for i in 0..x_test.len() {
    let z: f64 = w.iter().zip(&x_test[i]).map(|(&w_j, &x_j)| w_j * x_j).sum::<f64>() + bias;
    let pred: f64 = 1.0 / (1.0 + (-z).exp());
    y_pred.push(if pred > 0.5 { 1 } else { 0 });
}

let correct: usize = y_test.iter().zip(y_pred.iter()).filter(|(t, p)| *t == *p).count();
let accuracy: f64 = correct as f64 / y_test.len() as f64;

println!("Accuracy: {:.4}", accuracy);

// Confusion matrix
let mut cm: [[usize; 2]; 2] = [[0; 2]; 2];
for i in 0..y_test.len() {
    cm[y_test[i] as usize][y_pred[i] as usize] += 1;
}

println!("\nConfusion Matrix:");
println!("            Predicted");
println!("       |    0    |    1    |");
println!("Actual 0 |   {}    |   {}    |", cm[0][0], cm[0][1]);
println!("       1 |   {}    |   {}    |", cm[1][0], cm[1][1]);
```

```text
Dataset shape: (1000, 20)
Class 0: 495
Class 1: 505
Training set size: 800
Test set size: 200
Train - Class 0: 389, Class 1: 411
Test - Class 0: 106, Class 1: 94
Model trained successfully!
Feature weights (first 5): [0.3561301287492175, 0.339819297141489, 0.33628537415121074, 0.35910518961092286, 0.3803014838311296]
Bias: -0.0111
Accuracy: 0.9950

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   105    |   1    |
       1 |   0    |   94    |
```

## 5. Feature Importance

Similar to analyzing model coefficients in sklearn.

```rust
let feature_names: Vec<String> = (0..n_features)
    .map(|i| format!("feature_{}", i))
    .collect();

let mut importance: Vec<(usize, f64)> = w.iter()
    .enumerate()
    .map(|(i, &w)| (i, w.abs()))
    .collect();
importance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

println!("Top 10 most important features:");
for (i, (idx, imp)) in importance.iter().take(10).enumerate() {
    println!("  {}. {}: {:.4}", i + 1, feature_names[*idx], imp);
}
```

```text
Dataset shape: (1000, 20)
Class 0: 515
Class 1: 485
Training set size: 800
Test set size: 200
Train - Class 0: 415, Class 1: 385
Test - Class 0: 100, Class 1: 100
Model trained successfully!
Feature weights (first 5): [0.2945416340027652, 0.33037494856544586, 0.3059649906518464, 0.2840380729831545, 0.2973601882989032]
Bias: -0.0787
Accuracy: 0.9750

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   97    |   3    |
       1 |   2    |   98    |
Top 10 most important features:
  1. feature_11: 0.3453
  2. feature_15: 0.3444
  3. feature_16: 0.3439
  4. feature_13: 0.3391
  5. feature_1: 0.3304
  6. feature_17: 0.3293
  7. feature_19: 0.3285
  8. feature_12: 0.3273
  9. feature_9: 0.3202
  10. feature_10: 0.3149
()
```

```rust
use plotters::prelude::*;

let mut top_features: Vec<(String, f64)> = importance
    .iter()
    .take(10)
    .map(|(index, weight)| (feature_names[*index].clone(), *weight))
    .collect();
// Plotters draws the first category at the bottom, so reverse the sorted
// list to place the most important feature at the top of the chart.
top_features.reverse();
let max_importance = top_features
    .iter()
    .map(|(_, weight)| *weight)
    .fold(0.0_f64, f64::max)
    .max(1e-9);

println!("\n=== Feature Importance Plot ===");

let figure = plotters::evcxr::evcxr_figure((720, 420), |root| {
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Top 10 Feature Importance", ("sans-serif", 24))
        .margin(12)
        .x_label_area_size(45)
        .y_label_area_size(110)
        .build_cartesian_2d(0.0_f64..(max_importance * 1.1), 0..top_features.len())?;

    chart
        .configure_mesh()
        .x_desc("Absolute weight")
        .y_desc("Feature")
        .y_label_formatter(&|index| {
            top_features
                .get(*index)
                .map(|(name, _)| name.clone())
                .unwrap_or_default()
        })
        .x_labels(6)
        .y_labels(top_features.len())
        .draw()?;

    chart.draw_series(top_features.iter().enumerate().map(|(index, (_, weight))| {
        Rectangle::new(
            [(0.0, index), (*weight, index + 1)],
            BLUE.mix(0.65).filled(),
        )
    }))?;

    Ok(())
});

figure.evcxr_display();
```

```text
Dataset shape: (1000, 20)
Class 0: 496
Class 1: 504
Training set size: 800
Test set size: 200
Train - Class 0: 399, Class 1: 401
Test - Class 0: 97, Class 1: 103
Model trained successfully!
Feature weights (first 5): [0.3464847720656975, 0.36310303782469694, 0.31152062236705813, 0.31482342634365945, 0.3288481824019128]
Bias: -0.0356
Accuracy: 0.9700

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   95    |   2    |
       1 |   4    |   99    |
Top 10 most important features:
  1. feature_18: 0.3723
  2. feature_1: 0.3631
  3. feature_14: 0.3592
  4. feature_10: 0.3591
  5. feature_16: 0.3509
  6. feature_15: 0.3473
  7. feature_0: 0.3465
  8. feature_11: 0.3445
  9. feature_7: 0.3375
  10. feature_12: 0.3366
()

=== Feature Importance Plot ===
```

![Notebook output](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNzIwIiBoZWlnaHQ9IjQyMCIgdmlld0JveD0iMCAwIDcyMCA0MjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHg9IjAiIHk9IjAiIHdpZHRoPSI3MjAiIGhlaWdodD0iNDIwIiBvcGFjaXR5PSIxIiBmaWxsPSIjRkZGRkZGIiBzdHJva2U9Im5vbmUiLz4KPHRleHQgeD0iMzYwIiB5PSIxNyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTkuMzU0ODM4NzA5Njc3NDIiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpUb3AgMTAgRmVhdHVyZSBJbXBvcnRhbmNlCjwvdGV4dD4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjEyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMzYiIHkxPSIzNjIiIHgyPSIxMzYiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTUwIiB5MT0iMzYyIiB4Mj0iMTUwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjE2NCIgeTE9IjM2MiIgeDI9IjE2NCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxNzkiIHkxPSIzNjIiIHgyPSIxNzkiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTkzIiB5MT0iMzYyIiB4Mj0iMTkzIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjIwNyIgeTE9IjM2MiIgeDI9IjIwNyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyMjEiIHkxPSIzNjIiIHgyPSIyMjEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjM2IiB5MT0iMzYyIiB4Mj0iMjM2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI1MCIgeTE9IjM2MiIgeDI9IjI1MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyNjQiIHkxPSIzNjIiIHgyPSIyNjQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjc5IiB5MT0iMzYyIiB4Mj0iMjc5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI5MyIgeTE9IjM2MiIgeDI9IjI5MyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzMDciIHkxPSIzNjIiIHgyPSIzMDciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzIxIiB5MT0iMzYyIiB4Mj0iMzIxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjMzNiIgeTE9IjM2MiIgeDI9IjMzNiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzNTAiIHkxPSIzNjIiIHgyPSIzNTAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzY0IiB5MT0iMzYyIiB4Mj0iMzY0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM3OSIgeTE9IjM2MiIgeDI9IjM3OSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzOTMiIHkxPSIzNjIiIHgyPSIzOTMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDA3IiB5MT0iMzYyIiB4Mj0iNDA3IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQyMSIgeTE9IjM2MiIgeDI9IjQyMSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0MzYiIHkxPSIzNjIiIHgyPSI0MzYiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDUwIiB5MT0iMzYyIiB4Mj0iNDUwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQ2NCIgeTE9IjM2MiIgeDI9IjQ2NCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0NzkiIHkxPSIzNjIiIHgyPSI0NzkiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDkzIiB5MT0iMzYyIiB4Mj0iNDkzIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjUwNyIgeTE9IjM2MiIgeDI9IjUwNyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1MjEiIHkxPSIzNjIiIHgyPSI1MjEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTM2IiB5MT0iMzYyIiB4Mj0iNTM2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU1MCIgeTE9IjM2MiIgeDI9IjU1MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1NjQiIHkxPSIzNjIiIHgyPSI1NjQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTc5IiB5MT0iMzYyIiB4Mj0iNTc5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU5MyIgeTE9IjM2MiIgeDI9IjU5MyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2MDciIHkxPSIzNjIiIHgyPSI2MDciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjIxIiB5MT0iMzYyIiB4Mj0iNjIxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjYzNiIgeTE9IjM2MiIgeDI9IjYzNiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2NTAiIHkxPSIzNjIiIHgyPSI2NTAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjY0IiB5MT0iMzYyIiB4Mj0iNjY0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjY3OSIgeTE9IjM2MiIgeDI9IjY3OSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2OTMiIHkxPSIzNjIiIHgyPSI2OTMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzYyIiB4Mj0iNzA3IiB5Mj0iMzYyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzMzAiIHgyPSI3MDciIHkyPSIzMzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjI5OCIgeDI9IjcwNyIgeTI9IjI5OCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjY2IiB4Mj0iNzA3IiB5Mj0iMjY2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyMzQiIHgyPSI3MDciIHkyPSIyMzQiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIwMiIgeDI9IjcwNyIgeTI9IjIwMiIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTcwIiB4Mj0iNzA3IiB5Mj0iMTcwIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxMzgiIHgyPSI3MDciIHkyPSIxMzgiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEwNiIgeDI9IjcwNyIgeTI9IjEwNiIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNzQiIHgyPSI3MDciIHkyPSI3NCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNDEiIHgyPSI3MDciIHkyPSI0MSIvPgo8dGV4dCB4PSIxMiIgeT0iMjAyIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIiB0cmFuc2Zvcm09InJvdGF0ZSgyNzAsIDEyLCAyMDIpIj4KRmVhdHVyZQo8L3RleHQ+Cjx0ZXh0IHg9IjQxNSIgeT0iNDA4IiBkeT0iLTAuNWV4IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KQWJzb2x1dGUgd2VpZ2h0CjwvdGV4dD4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjEyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyNjQiIHkxPSIzNjIiIHgyPSIyNjQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDA3IiB5MT0iMzYyIiB4Mj0iNDA3IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU1MCIgeTE9IjM2MiIgeDI9IjU1MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2OTMiIHkxPSIzNjIiIHgyPSI2OTMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzYyIiB4Mj0iNzA3IiB5Mj0iMzYyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzMzAiIHgyPSI3MDciIHkyPSIzMzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjI5OCIgeDI9IjcwNyIgeTI9IjI5OCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjY2IiB4Mj0iNzA3IiB5Mj0iMjY2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyMzQiIHgyPSI3MDciIHkyPSIyMzQiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIwMiIgeDI9IjcwNyIgeTI9IjIwMiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTcwIiB4Mj0iNzA3IiB5Mj0iMTcwIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxMzgiIHgyPSI3MDciIHkyPSIxMzgiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEwNiIgeDI9IjcwNyIgeTI9IjEwNiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNzQiIHgyPSI3MDciIHkyPSI3NCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNDEiIHgyPSI3MDciIHkyPSI0MSIvPgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTIxLDQxIDEyMSwzNjIgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMzYyIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzEyCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwzNjIgMTIxLDM2MiAiLz4KPHRleHQgeD0iMTEyIiB5PSIzMzAiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfNwo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMzMwIDEyMSwzMzAgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjk4IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzExCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwyOTggMTIxLDI5OCAiLz4KPHRleHQgeD0iMTEyIiB5PSIyNjYiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjY2IDEyMSwyNjYgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjM0IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzE1CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwyMzQgMTIxLDIzNCAiLz4KPHRleHQgeD0iMTEyIiB5PSIyMDIiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMTYKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDIwMiAxMjEsMjAyICIvPgo8dGV4dCB4PSIxMTIiIHk9IjE3MCIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8xMAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMTcwIDEyMSwxNzAgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMTM4IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzE0CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwxMzggMTIxLDEzOCAiLz4KPHRleHQgeD0iMTEyIiB5PSIxMDYiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMTA2IDEyMSwxMDYgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iNzQiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMTgKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDc0IDEyMSw3NCAiLz4KPHRleHQgeD0iMTEyIiB5PSI0MSIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiw0MSAxMjEsNDEgIi8+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMjIsMzYzIDcwNywzNjMgIi8+Cjx0ZXh0IHg9IjEyMiIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4wCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjEyMiwzNjMgMTIyLDM2OCAiLz4KPHRleHQgeD0iMjY0IiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjEKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMjY0LDM2MyAyNjQsMzY4ICIvPgo8dGV4dCB4PSI0MDciIHk9IjM3MyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CjAuMgo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSI0MDcsMzYzIDQwNywzNjggIi8+Cjx0ZXh0IHg9IjU1MCIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4zCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjU1MCwzNjMgNTUwLDM2OCAiLz4KPHRleHQgeD0iNjkzIiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjQKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iNjkzLDM2MyA2OTMsMzY4ICIvPgo8cmVjdCB4PSIxMjIiIHk9IjMzMCIgd2lkdGg9IjQ4MCIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjk4IiB3aWR0aD0iNDgyIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIyNjYiIHdpZHRoPSI0OTIiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjIzNCIgd2lkdGg9IjQ5NCIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjAyIiB3aWR0aD0iNDk2IiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIxNzAiIHdpZHRoPSI1MDEiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjEzOCIgd2lkdGg9IjUxMiIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMTA2IiB3aWR0aD0iNTEzIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSI3NCIgd2lkdGg9IjUxOCIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iNDEiIHdpZHRoPSI1MzEiIGhlaWdodD0iMzMiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8L3N2Zz4=)

## Summary

This notebook demonstrated:
1. Synthetic data generation (like `make_classification`)
2. Train/test split (like `train_test_split`)
3. Logistic regression (like `LogisticRegression`)
4. Model evaluation with accuracy and confusion matrix
5. Feature importance analysis (like model coefficients)

**Note:** This is a simplified implementation using the standard library. A full implementation would use:
- **polars** for DataFrame operations (similar to pandas)
- **ndarray** for numerical arrays (similar to numpy)
- **linfa** for ML algorithms (similar to scikit-learn)
- **plotters** for visualization (similar to matplotlib)

These libraries would be added as dependencies in a real Mars project's Cargo.toml.