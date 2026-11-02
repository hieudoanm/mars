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
Class 0: 504
Class 1: 496
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
Class 0: 501
Class 1: 499
Training set size: 800
Test set size: 200
Train - Class 0: 405, Class 1: 395
Test - Class 0: 96, Class 1: 104
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
Class 0: 487
Class 1: 513
Training set size: 800
Test set size: 200
Train - Class 0: 392, Class 1: 408
Test - Class 0: 95, Class 1: 105
Model trained successfully!
Feature weights (first 5): [0.34887512258967696, 0.40101871817193996, 0.3745192026308998, 0.3776219879313013, 0.3437739101689766]
Bias: 0.0276
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
Class 0: 499
Class 1: 501
Training set size: 800
Test set size: 200
Train - Class 0: 402, Class 1: 398
Test - Class 0: 97, Class 1: 103
Model trained successfully!
Feature weights (first 5): [0.3322888011791027, 0.33820918899830504, 0.32367394445396624, 0.30227418852399474, 0.32733238846037005]
Bias: 0.0127
Accuracy: 0.9800

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   95    |   2    |
       1 |   2    |   101    |
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
Class 0: 479
Class 1: 521
Training set size: 800
Test set size: 200
Train - Class 0: 371, Class 1: 429
Test - Class 0: 108, Class 1: 92
Model trained successfully!
Feature weights (first 5): [0.3310303522683049, 0.3434504392668493, 0.3564071911274419, 0.34715217952759364, 0.3385521216926566]
Bias: 0.0727
Accuracy: 0.9900

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   107    |   1    |
       1 |   1    |   91    |
Top 10 most important features:
  1. feature_13: 0.3779
  2. feature_14: 0.3711
  3. feature_18: 0.3670
  4. feature_10: 0.3606
  5. feature_7: 0.3599
  6. feature_2: 0.3564
  7. feature_19: 0.3524
  8. feature_8: 0.3494
  9. feature_12: 0.3480
  10. feature_3: 0.3472
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
Class 0: 507
Class 1: 493
Training set size: 800
Test set size: 200
Train - Class 0: 401, Class 1: 399
Test - Class 0: 106, Class 1: 94
Model trained successfully!
Feature weights (first 5): [0.3519708958034722, 0.33509858824114, 0.3408134664844405, 0.33622031654524653, 0.33169724766881226]
Bias: 0.0383
Accuracy: 0.9900

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   104    |   2    |
       1 |   0    |   94    |
Top 10 most important features:
  1. feature_0: 0.3520
  2. feature_7: 0.3460
  3. feature_6: 0.3444
  4. feature_19: 0.3429
  5. feature_9: 0.3419
  6. feature_2: 0.3408
  7. feature_3: 0.3362
  8. feature_5: 0.3355
  9. feature_1: 0.3351
  10. feature_10: 0.3341
()

=== Feature Importance Plot ===
```

![Notebook output](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNzIwIiBoZWlnaHQ9IjQyMCIgdmlld0JveD0iMCAwIDcyMCA0MjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHg9IjAiIHk9IjAiIHdpZHRoPSI3MjAiIGhlaWdodD0iNDIwIiBvcGFjaXR5PSIxIiBmaWxsPSIjRkZGRkZGIiBzdHJva2U9Im5vbmUiLz4KPHRleHQgeD0iMzYwIiB5PSIxNyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTkuMzU0ODM4NzA5Njc3NDIiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpUb3AgMTAgRmVhdHVyZSBJbXBvcnRhbmNlCjwvdGV4dD4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjEyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMzciIHkxPSIzNjIiIHgyPSIxMzciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTUyIiB5MT0iMzYyIiB4Mj0iMTUyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjE2NyIgeTE9IjM2MiIgeDI9IjE2NyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxODIiIHkxPSIzNjIiIHgyPSIxODIiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTk3IiB5MT0iMzYyIiB4Mj0iMTk3IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjIxMiIgeTE9IjM2MiIgeDI9IjIxMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyMjciIHkxPSIzNjIiIHgyPSIyMjciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjQyIiB5MT0iMzYyIiB4Mj0iMjQyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI1NyIgeTE9IjM2MiIgeDI9IjI1NyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyNzMiIHkxPSIzNjIiIHgyPSIyNzMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjg4IiB5MT0iMzYyIiB4Mj0iMjg4IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjMwMyIgeTE9IjM2MiIgeDI9IjMwMyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzMTgiIHkxPSIzNjIiIHgyPSIzMTgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzMzIiB5MT0iMzYyIiB4Mj0iMzMzIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM0OCIgeTE9IjM2MiIgeDI9IjM0OCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzNjMiIHkxPSIzNjIiIHgyPSIzNjMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzc4IiB5MT0iMzYyIiB4Mj0iMzc4IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM5MyIgeTE9IjM2MiIgeDI9IjM5MyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0MDkiIHkxPSIzNjIiIHgyPSI0MDkiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDI0IiB5MT0iMzYyIiB4Mj0iNDI0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQzOSIgeTE9IjM2MiIgeDI9IjQzOSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0NTQiIHkxPSIzNjIiIHgyPSI0NTQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDY5IiB5MT0iMzYyIiB4Mj0iNDY5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQ4NCIgeTE9IjM2MiIgeDI9IjQ4NCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0OTkiIHkxPSIzNjIiIHgyPSI0OTkiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTE0IiB5MT0iMzYyIiB4Mj0iNTE0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjUyOSIgeTE9IjM2MiIgeDI9IjUyOSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1NDUiIHkxPSIzNjIiIHgyPSI1NDUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTYwIiB5MT0iMzYyIiB4Mj0iNTYwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU3NSIgeTE9IjM2MiIgeDI9IjU3NSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1OTAiIHkxPSIzNjIiIHgyPSI1OTAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjA1IiB5MT0iMzYyIiB4Mj0iNjA1IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjYyMCIgeTE9IjM2MiIgeDI9IjYyMCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2MzUiIHkxPSIzNjIiIHgyPSI2MzUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjUwIiB5MT0iMzYyIiB4Mj0iNjUwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjY2NSIgeTE9IjM2MiIgeDI9IjY2NSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2ODEiIHkxPSIzNjIiIHgyPSI2ODEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjk2IiB5MT0iMzYyIiB4Mj0iNjk2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjcwNyIgeTI9IjM2MiIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzMwIiB4Mj0iNzA3IiB5Mj0iMzMwIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyOTgiIHgyPSI3MDciIHkyPSIyOTgiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjI2NiIgeDI9IjcwNyIgeTI9IjI2NiIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjM0IiB4Mj0iNzA3IiB5Mj0iMjM0Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyMDIiIHgyPSI3MDciIHkyPSIyMDIiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjE3MCIgeDI9IjcwNyIgeTI9IjE3MCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTM4IiB4Mj0iNzA3IiB5Mj0iMTM4Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxMDYiIHgyPSI3MDciIHkyPSIxMDYiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9Ijc0IiB4Mj0iNzA3IiB5Mj0iNzQiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjQxIiB4Mj0iNzA3IiB5Mj0iNDEiLz4KPHRleHQgeD0iMTIiIHk9IjIwMiIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCIgdHJhbnNmb3JtPSJyb3RhdGUoMjcwLCAxMiwgMjAyKSI+CkZlYXR1cmUKPC90ZXh0Pgo8dGV4dCB4PSI0MTUiIHk9IjQwOCIgZHk9Ii0wLjVleCIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CkFic29sdXRlIHdlaWdodAo8L3RleHQ+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzNjIiIHgyPSIxMjIiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjczIiB5MT0iMzYyIiB4Mj0iMjczIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQyNCIgeTE9IjM2MiIgeDI9IjQyNCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1NzUiIHkxPSIzNjIiIHgyPSI1NzUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzYyIiB4Mj0iNzA3IiB5Mj0iMzYyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzMzAiIHgyPSI3MDciIHkyPSIzMzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjI5OCIgeDI9IjcwNyIgeTI9IjI5OCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjY2IiB4Mj0iNzA3IiB5Mj0iMjY2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyMzQiIHgyPSI3MDciIHkyPSIyMzQiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIwMiIgeDI9IjcwNyIgeTI9IjIwMiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTcwIiB4Mj0iNzA3IiB5Mj0iMTcwIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxMzgiIHgyPSI3MDciIHkyPSIxMzgiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEwNiIgeDI9IjcwNyIgeTI9IjEwNiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNzQiIHgyPSI3MDciIHkyPSI3NCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iNDEiIHgyPSI3MDciIHkyPSI0MSIvPgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTIxLDQxIDEyMSwzNjIgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMzYyIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzEwCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwzNjIgMTIxLDM2MiAiLz4KPHRleHQgeD0iMTEyIiB5PSIzMzAiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMzMwIDEyMSwzMzAgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjk4IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzUKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDI5OCAxMjEsMjk4ICIvPgo8dGV4dCB4PSIxMTIiIHk9IjI2NiIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8zCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwyNjYgMTIxLDI2NiAiLz4KPHRleHQgeD0iMTEyIiB5PSIyMzQiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMgo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjM0IDEyMSwyMzQgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjAyIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzkKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDIwMiAxMjEsMjAyICIvPgo8dGV4dCB4PSIxMTIiIHk9IjE3MCIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8xOQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMTcwIDEyMSwxNzAgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMTM4IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzYKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDEzOCAxMjEsMTM4ICIvPgo8dGV4dCB4PSIxMTIiIHk9IjEwNiIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV83CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwxMDYgMTIxLDEwNiAiLz4KPHRleHQgeD0iMTEyIiB5PSI3NCIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8wCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiw3NCAxMjEsNzQgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iNDEiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+Cgo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsNDEgMTIxLDQxICIvPgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTIyLDM2MyA3MDcsMzYzICIvPgo8dGV4dCB4PSIxMjIiIHk9IjM3MyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CjAuMAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMjIsMzYzIDEyMiwzNjggIi8+Cjx0ZXh0IHg9IjI3MyIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4xCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjI3MywzNjMgMjczLDM2OCAiLz4KPHRleHQgeD0iNDI0IiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjIKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iNDI0LDM2MyA0MjQsMzY4ICIvPgo8dGV4dCB4PSI1NzUiIHk9IjM3MyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CjAuMwo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSI1NzUsMzYzIDU3NSwzNjggIi8+CjxyZWN0IHg9IjEyMiIgeT0iMzMwIiB3aWR0aD0iNTA0IiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIyOTgiIHdpZHRoPSI1MDYiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjI2NiIgd2lkdGg9IjUwNiIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjM0IiB3aWR0aD0iNTA4IiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIyMDIiIHdpZHRoPSI1MTQiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjE3MCIgd2lkdGg9IjUxNiIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMTM4IiB3aWR0aD0iNTE4IiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIxMDYiIHdpZHRoPSI1MjAiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9Ijc0IiB3aWR0aD0iNTIyIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSI0MSIgd2lkdGg9IjUzMSIgaGVpZ2h0PSIzMyIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+Cjwvc3ZnPg==)

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
