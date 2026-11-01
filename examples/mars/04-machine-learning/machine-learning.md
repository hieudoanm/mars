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
Class 0: 488
Class 1: 512
Training set size: 800
Test set size: 200
Train - Class 0: 390, Class 1: 410
Test - Class 0: 98, Class 1: 102
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
Class 0: 529
Class 1: 471
Training set size: 800
Test set size: 200
Train - Class 0: 425, Class 1: 375
Test - Class 0: 104, Class 1: 96
Model trained successfully!
Feature weights (first 5): [0.3503154969326295, 0.3144771984593554, 0.3470431305186885, 0.3673302185281649, 0.3192175723767311]
Bias: -0.0475
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
Class 0: 492
Class 1: 508
Training set size: 800
Test set size: 200
Train - Class 0: 397, Class 1: 403
Test - Class 0: 95, Class 1: 105
Model trained successfully!
Feature weights (first 5): [0.33767192746114666, 0.3290077742630423, 0.3371535289196502, 0.30866435938008374, 0.3314390197840963]
Bias: 0.0389
Accuracy: 0.9950

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   94    |   1    |
       1 |   0    |   105    |
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
Class 0: 500
Class 1: 500
Training set size: 800
Test set size: 200
Train - Class 0: 398, Class 1: 402
Test - Class 0: 102, Class 1: 98
Model trained successfully!
Feature weights (first 5): [0.2916860150386009, 0.3053013472841396, 0.32726733605249997, 0.3215069615764379, 0.3148328473787222]
Bias: -0.0427
Accuracy: 0.9850

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   99    |   3    |
       1 |   0    |   98    |
Top 10 most important features:
  1. feature_9: 0.3453
  2. feature_10: 0.3434
  3. feature_12: 0.3341
  4. feature_16: 0.3310
  5. feature_11: 0.3300
  6. feature_18: 0.3288
  7. feature_8: 0.3282
  8. feature_2: 0.3273
  9. feature_7: 0.3272
  10. feature_13: 0.3224
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
Class 0: 499
Class 1: 501
Training set size: 800
Test set size: 200
Train - Class 0: 407, Class 1: 393
Test - Class 0: 92, Class 1: 108
Model trained successfully!
Feature weights (first 5): [0.35364389186464024, 0.36475663643416223, 0.35024529368293716, 0.3369049341517111, 0.3729532820952152]
Bias: -0.0029
Accuracy: 0.9800

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   91    |   1    |
       1 |   3    |   105    |
Top 10 most important features:
  1. feature_14: 0.3848
  2. feature_5: 0.3839
  3. feature_18: 0.3768
  4. feature_4: 0.3730
  5. feature_15: 0.3707
  6. feature_9: 0.3699
  7. feature_13: 0.3698
  8. feature_1: 0.3648
  9. feature_10: 0.3565
  10. feature_0: 0.3536
()

=== Feature Importance Plot ===
```

![Notebook output](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNzIwIiBoZWlnaHQ9IjQyMCIgdmlld0JveD0iMCAwIDcyMCA0MjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHg9IjAiIHk9IjAiIHdpZHRoPSI3MjAiIGhlaWdodD0iNDIwIiBvcGFjaXR5PSIxIiBmaWxsPSIjRkZGRkZGIiBzdHJva2U9Im5vbmUiLz4KPHRleHQgeD0iMzYwIiB5PSIxNyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTkuMzU0ODM4NzA5Njc3NDIiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpUb3AgMTAgRmVhdHVyZSBJbXBvcnRhbmNlCjwvdGV4dD4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjEyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMzUiIHkxPSIzNjIiIHgyPSIxMzUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTQ5IiB5MT0iMzYyIiB4Mj0iMTQ5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjE2MyIgeTE9IjM2MiIgeDI9IjE2MyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxNzciIHkxPSIzNjIiIHgyPSIxNzciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTkxIiB5MT0iMzYyIiB4Mj0iMTkxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjIwNCIgeTE9IjM2MiIgeDI9IjIwNCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyMTgiIHkxPSIzNjIiIHgyPSIyMTgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjMyIiB5MT0iMzYyIiB4Mj0iMjMyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI0NiIgeTE9IjM2MiIgeDI9IjI0NiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyNjAiIHkxPSIzNjIiIHgyPSIyNjAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjc0IiB5MT0iMzYyIiB4Mj0iMjc0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI4NyIgeTE9IjM2MiIgeDI9IjI4NyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzMDEiIHkxPSIzNjIiIHgyPSIzMDEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzE1IiB5MT0iMzYyIiB4Mj0iMzE1IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjMyOSIgeTE9IjM2MiIgeDI9IjMyOSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzNDMiIHkxPSIzNjIiIHgyPSIzNDMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzU2IiB5MT0iMzYyIiB4Mj0iMzU2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM3MCIgeTE9IjM2MiIgeDI9IjM3MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzODQiIHkxPSIzNjIiIHgyPSIzODQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzk4IiB5MT0iMzYyIiB4Mj0iMzk4IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQxMiIgeTE9IjM2MiIgeDI9IjQxMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0MjYiIHkxPSIzNjIiIHgyPSI0MjYiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDM5IiB5MT0iMzYyIiB4Mj0iNDM5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQ1MyIgeTE9IjM2MiIgeDI9IjQ1MyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0NjciIHkxPSIzNjIiIHgyPSI0NjciIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDgxIiB5MT0iMzYyIiB4Mj0iNDgxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQ5NSIgeTE9IjM2MiIgeDI9IjQ5NSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1MDgiIHkxPSIzNjIiIHgyPSI1MDgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTIyIiB5MT0iMzYyIiB4Mj0iNTIyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjUzNiIgeTE9IjM2MiIgeDI9IjUzNiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1NTAiIHkxPSIzNjIiIHgyPSI1NTAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTY0IiB5MT0iMzYyIiB4Mj0iNTY0IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU3OCIgeTE9IjM2MiIgeDI9IjU3OCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1OTEiIHkxPSIzNjIiIHgyPSI1OTEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjA1IiB5MT0iMzYyIiB4Mj0iNjA1IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjYxOSIgeTE9IjM2MiIgeDI9IjYxOSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2MzMiIHkxPSIzNjIiIHgyPSI2MzMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjQ3IiB5MT0iMzYyIiB4Mj0iNjQ3IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjY2MCIgeTE9IjM2MiIgeDI9IjY2MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2NzQiIHkxPSIzNjIiIHgyPSI2NzQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjg4IiB5MT0iMzYyIiB4Mj0iNjg4IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjcwMiIgeTE9IjM2MiIgeDI9IjcwMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzNjIiIHgyPSI3MDciIHkyPSIzNjIiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjMzMCIgeDI9IjcwNyIgeTI9IjMzMCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjk4IiB4Mj0iNzA3IiB5Mj0iMjk4Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyNjYiIHgyPSI3MDciIHkyPSIyNjYiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIzNCIgeDI9IjcwNyIgeTI9IjIzNCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjAyIiB4Mj0iNzA3IiB5Mj0iMjAyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxNzAiIHgyPSI3MDciIHkyPSIxNzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEzOCIgeDI9IjcwNyIgeTI9IjEzOCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTA2IiB4Mj0iNzA3IiB5Mj0iMTA2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI3NCIgeDI9IjcwNyIgeTI9Ijc0Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI0MSIgeDI9IjcwNyIgeTI9IjQxIi8+Cjx0ZXh0IHg9IjEyIiB5PSIyMDIiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiIHRyYW5zZm9ybT0icm90YXRlKDI3MCwgMTIsIDIwMikiPgpGZWF0dXJlCjwvdGV4dD4KPHRleHQgeD0iNDE1IiB5PSI0MDgiIGR5PSItMC41ZXgiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpBYnNvbHV0ZSB3ZWlnaHQKPC90ZXh0Pgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzYyIiB4Mj0iMTIyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI2MCIgeTE9IjM2MiIgeDI9IjI2MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzOTgiIHkxPSIzNjIiIHgyPSIzOTgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTM2IiB5MT0iMzYyIiB4Mj0iNTM2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjY3NCIgeTE9IjM2MiIgeDI9IjY3NCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzNjIiIHgyPSI3MDciIHkyPSIzNjIiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjMzMCIgeDI9IjcwNyIgeTI9IjMzMCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjk4IiB4Mj0iNzA3IiB5Mj0iMjk4Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyNjYiIHgyPSI3MDciIHkyPSIyNjYiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIzNCIgeDI9IjcwNyIgeTI9IjIzNCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjAyIiB4Mj0iNzA3IiB5Mj0iMjAyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxNzAiIHgyPSI3MDciIHkyPSIxNzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEzOCIgeDI9IjcwNyIgeTI9IjEzOCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTA2IiB4Mj0iNzA3IiB5Mj0iMTA2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI3NCIgeDI9IjcwNyIgeTI9Ijc0Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI0MSIgeDI9IjcwNyIgeTI9IjQxIi8+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMjEsNDEgMTIxLDM2MiAiLz4KPHRleHQgeD0iMTEyIiB5PSIzNjIiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMzYyIDEyMSwzNjIgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMzMwIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzEwCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwzMzAgMTIxLDMzMCAiLz4KPHRleHQgeD0iMTEyIiB5PSIyOTgiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjk4IDEyMSwyOTggIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjY2IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzEzCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwyNjYgMTIxLDI2NiAiLz4KPHRleHQgeD0iMTEyIiB5PSIyMzQiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfOQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjM0IDEyMSwyMzQgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjAyIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzE1CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwyMDIgMTIxLDIwMiAiLz4KPHRleHQgeD0iMTEyIiB5PSIxNzAiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfNAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMTcwIDEyMSwxNzAgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMTM4IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzE4CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwxMzggMTIxLDEzOCAiLz4KPHRleHQgeD0iMTEyIiB5PSIxMDYiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfNQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMTA2IDEyMSwxMDYgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iNzQiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMTQKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDc0IDEyMSw3NCAiLz4KPHRleHQgeD0iMTEyIiB5PSI0MSIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiw0MSAxMjEsNDEgIi8+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMjIsMzYzIDcwNywzNjMgIi8+Cjx0ZXh0IHg9IjEyMiIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4wCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjEyMiwzNjMgMTIyLDM2OCAiLz4KPHRleHQgeD0iMjYwIiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjEKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMjYwLDM2MyAyNjAsMzY4ICIvPgo8dGV4dCB4PSIzOTgiIHk9IjM3MyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CjAuMgo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIzOTgsMzYzIDM5OCwzNjggIi8+Cjx0ZXh0IHg9IjUzNiIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4zCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjUzNiwzNjMgNTM2LDM2OCAiLz4KPHRleHQgeD0iNjc0IiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjQKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iNjc0LDM2MyA2NzQsMzY4ICIvPgo8cmVjdCB4PSIxMjIiIHk9IjMzMCIgd2lkdGg9IjQ4OCIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjk4IiB3aWR0aD0iNDkyIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIyNjYiIHdpZHRoPSI1MDQiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjIzNCIgd2lkdGg9IjUxMSIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjAyIiB3aWR0aD0iNTExIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIxNzAiIHdpZHRoPSI1MTIiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjEzOCIgd2lkdGg9IjUxNSIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMTA2IiB3aWR0aD0iNTIwIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSI3NCIgd2lkdGg9IjUzMCIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iNDEiIHdpZHRoPSI1MzEiIGhlaWdodD0iMzMiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8L3N2Zz4=)

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