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
Class 0: 508
Class 1: 492
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
Class 0: 508
Class 1: 492
Training set size: 800
Test set size: 200
Train - Class 0: 417, Class 1: 383
Test - Class 0: 91, Class 1: 109
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
Class 0: 516
Class 1: 484
Training set size: 800
Test set size: 200
Train - Class 0: 410, Class 1: 390
Test - Class 0: 106, Class 1: 94
Model trained successfully!
Feature weights (first 5): [0.34310180395686596, 0.31557243039275223, 0.3083120956753754, 0.336561182963987, 0.31176212028048683]
Bias: -0.0123
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
Class 0: 494
Class 1: 506
Training set size: 800
Test set size: 200
Train - Class 0: 406, Class 1: 394
Test - Class 0: 88, Class 1: 112
Model trained successfully!
Feature weights (first 5): [0.3315101054881368, 0.30729059640792283, 0.2980240133131514, 0.33311573834200325, 0.3296594926206026]
Bias: 0.0250
Accuracy: 0.9650

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   83    |   5    |
       1 |   2    |   110    |
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
Class 0: 516
Class 1: 484
Training set size: 800
Test set size: 200
Train - Class 0: 408, Class 1: 392
Test - Class 0: 108, Class 1: 92
Model trained successfully!
Feature weights (first 5): [0.33175237065517416, 0.3407315524505727, 0.3159951299946694, 0.3673904995562657, 0.35480226234869494]
Bias: -0.0628
Accuracy: 0.9800

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   106    |   2    |
       1 |   2    |   90    |
Top 10 most important features:
  1. feature_3: 0.3674
  2. feature_13: 0.3613
  3. feature_15: 0.3567
  4. feature_18: 0.3561
  5. feature_4: 0.3548
  6. feature_11: 0.3480
  7. feature_7: 0.3473
  8. feature_19: 0.3465
  9. feature_12: 0.3438
  10. feature_6: 0.3421
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
Class 0: 466
Class 1: 534
Training set size: 800
Test set size: 200
Train - Class 0: 368, Class 1: 432
Test - Class 0: 98, Class 1: 102
Model trained successfully!
Feature weights (first 5): [0.3353012717380643, 0.3095716689879196, 0.3117243438103644, 0.3260772427361615, 0.33711855895728926]
Bias: 0.0527
Accuracy: 0.9850

Confusion Matrix:
            Predicted
       |    0    |    1    |
Actual 0 |   96    |   2    |
       1 |   1    |   101    |
Top 10 most important features:
  1. feature_7: 0.3588
  2. feature_10: 0.3587
  3. feature_9: 0.3533
  4. feature_4: 0.3371
  5. feature_14: 0.3358
  6. feature_0: 0.3353
  7. feature_18: 0.3335
  8. feature_19: 0.3331
  9. feature_17: 0.3317
  10. feature_11: 0.3297
()

=== Feature Importance Plot ===
```

![Notebook output](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNzIwIiBoZWlnaHQ9IjQyMCIgdmlld0JveD0iMCAwIDcyMCA0MjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHg9IjAiIHk9IjAiIHdpZHRoPSI3MjAiIGhlaWdodD0iNDIwIiBvcGFjaXR5PSIxIiBmaWxsPSIjRkZGRkZGIiBzdHJva2U9Im5vbmUiLz4KPHRleHQgeD0iMzYwIiB5PSIxNyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTkuMzU0ODM4NzA5Njc3NDIiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpUb3AgMTAgRmVhdHVyZSBJbXBvcnRhbmNlCjwvdGV4dD4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjEyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMzYiIHkxPSIzNjIiIHgyPSIxMzYiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTUxIiB5MT0iMzYyIiB4Mj0iMTUxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjE2NiIgeTE9IjM2MiIgeDI9IjE2NiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxODEiIHkxPSIzNjIiIHgyPSIxODEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTk2IiB5MT0iMzYyIiB4Mj0iMTk2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjIxMCIgeTE9IjM2MiIgeDI9IjIxMCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyMjUiIHkxPSIzNjIiIHgyPSIyMjUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjQwIiB5MT0iMzYyIiB4Mj0iMjQwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI1NSIgeTE9IjM2MiIgeDI9IjI1NSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIyNzAiIHkxPSIzNjIiIHgyPSIyNzAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMjg1IiB5MT0iMzYyIiB4Mj0iMjg1IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI5OSIgeTE9IjM2MiIgeDI9IjI5OSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzMTQiIHkxPSIzNjIiIHgyPSIzMTQiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzI5IiB5MT0iMzYyIiB4Mj0iMzI5IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM0NCIgeTE9IjM2MiIgeDI9IjM0NCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIzNTkiIHkxPSIzNjIiIHgyPSIzNTkiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMzczIiB5MT0iMzYyIiB4Mj0iMzczIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjM4OCIgeTE9IjM2MiIgeDI9IjM4OCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0MDMiIHkxPSIzNjIiIHgyPSI0MDMiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDE4IiB5MT0iMzYyIiB4Mj0iNDE4IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQzMyIgeTE9IjM2MiIgeDI9IjQzMyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0NDgiIHkxPSIzNjIiIHgyPSI0NDgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNDYyIiB5MT0iMzYyIiB4Mj0iNDYyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjQ3NyIgeTE9IjM2MiIgeDI9IjQ3NyIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0OTIiIHkxPSIzNjIiIHgyPSI0OTIiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTA3IiB5MT0iMzYyIiB4Mj0iNTA3IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjUyMiIgeTE9IjM2MiIgeDI9IjUyMiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1MzYiIHkxPSIzNjIiIHgyPSI1MzYiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTUxIiB5MT0iMzYyIiB4Mj0iNTUxIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjU2NiIgeTE9IjM2MiIgeDI9IjU2NiIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI1ODEiIHkxPSIzNjIiIHgyPSI1ODEiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTk2IiB5MT0iMzYyIiB4Mj0iNTk2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjYxMSIgeTE9IjM2MiIgeDI9IjYxMSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2MjUiIHkxPSIzNjIiIHgyPSI2MjUiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjQwIiB5MT0iMzYyIiB4Mj0iNjQwIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjY1NSIgeTE9IjM2MiIgeDI9IjY1NSIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI2NzAiIHkxPSIzNjIiIHgyPSI2NzAiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNjg1IiB5MT0iMzYyIiB4Mj0iNjg1IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjcwMCIgeTE9IjM2MiIgeDI9IjcwMCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIzNjIiIHgyPSI3MDciIHkyPSIzNjIiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjMzMCIgeDI9IjcwNyIgeTI9IjMzMCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjk4IiB4Mj0iNzA3IiB5Mj0iMjk4Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyNjYiIHgyPSI3MDciIHkyPSIyNjYiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjIzNCIgeDI9IjcwNyIgeTI9IjIzNCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjAyIiB4Mj0iNzA3IiB5Mj0iMjAyIi8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxNzAiIHgyPSI3MDciIHkyPSIxNzAiLz4KPGxpbmUgb3BhY2l0eT0iMC4xIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjEzOCIgeDI9IjcwNyIgeTI9IjEzOCIvPgo8bGluZSBvcGFjaXR5PSIwLjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTA2IiB4Mj0iNzA3IiB5Mj0iMTA2Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI3NCIgeDI9IjcwNyIgeTI9Ijc0Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSI0MSIgeDI9IjcwNyIgeTI9IjQxIi8+Cjx0ZXh0IHg9IjEyIiB5PSIyMDIiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiIHRyYW5zZm9ybT0icm90YXRlKDI3MCwgMTIsIDIwMikiPgpGZWF0dXJlCjwvdGV4dD4KPHRleHQgeD0iNDE1IiB5PSI0MDgiIGR5PSItMC41ZXgiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpBYnNvbHV0ZSB3ZWlnaHQKPC90ZXh0Pgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzYyIiB4Mj0iMTIyIiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjI3MCIgeTE9IjM2MiIgeDI9IjI3MCIgeTI9IjQxIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSI0MTgiIHkxPSIzNjIiIHgyPSI0MTgiIHkyPSI0MSIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iNTY2IiB5MT0iMzYyIiB4Mj0iNTY2IiB5Mj0iNDEiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjM2MiIgeDI9IjcwNyIgeTI9IjM2MiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMzMwIiB4Mj0iNzA3IiB5Mj0iMzMwIi8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyOTgiIHgyPSI3MDciIHkyPSIyOTgiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjI2NiIgeDI9IjcwNyIgeTI9IjI2NiIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMjM0IiB4Mj0iNzA3IiB5Mj0iMjM0Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIyMDIiIHgyPSI3MDciIHkyPSIyMDIiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjE3MCIgeDI9IjcwNyIgeTI9IjE3MCIvPgo8bGluZSBvcGFjaXR5PSIwLjIiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiB4MT0iMTIyIiB5MT0iMTM4IiB4Mj0iNzA3IiB5Mj0iMTM4Ii8+CjxsaW5lIG9wYWNpdHk9IjAuMiIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHgxPSIxMjIiIHkxPSIxMDYiIHgyPSI3MDciIHkyPSIxMDYiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9Ijc0IiB4Mj0iNzA3IiB5Mj0iNzQiLz4KPGxpbmUgb3BhY2l0eT0iMC4yIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgeDE9IjEyMiIgeTE9IjQxIiB4Mj0iNzA3IiB5Mj0iNDEiLz4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjEyMSw0MSAxMjEsMzYyICIvPgo8dGV4dCB4PSIxMTIiIHk9IjM2MiIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8xMQo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMzYyIDEyMSwzNjIgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMzMwIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzE3CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwzMzAgMTIxLDMzMCAiLz4KPHRleHQgeD0iMTEyIiB5PSIyOTgiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMTkKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDI5OCAxMjEsMjk4ICIvPgo8dGV4dCB4PSIxMTIiIHk9IjI2NiIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8xOAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjY2IDEyMSwyNjYgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMjM0IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzAKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDIzNCAxMjEsMjM0ICIvPgo8dGV4dCB4PSIxMTIiIHk9IjIwMiIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV8xNAo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMTYsMjAyIDEyMSwyMDIgIi8+Cjx0ZXh0IHg9IjExMiIgeT0iMTcwIiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzQKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDE3MCAxMjEsMTcwICIvPgo8dGV4dCB4PSIxMTIiIHk9IjEzOCIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KZmVhdHVyZV85CjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiwxMzggMTIxLDEzOCAiLz4KPHRleHQgeD0iMTEyIiB5PSIxMDYiIGR5PSIwLjVleCIgdGV4dC1hbmNob3I9ImVuZCIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CmZlYXR1cmVfMTAKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDEwNiAxMjEsMTA2ICIvPgo8dGV4dCB4PSIxMTIiIHk9Ijc0IiBkeT0iMC41ZXgiIHRleHQtYW5jaG9yPSJlbmQiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgpmZWF0dXJlXzcKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMTE2LDc0IDEyMSw3NCAiLz4KPHRleHQgeD0iMTEyIiB5PSI0MSIgZHk9IjAuNWV4IiB0ZXh0LWFuY2hvcj0iZW5kIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjExNiw0MSAxMjEsNDEgIi8+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSIxMjIsMzYzIDcwNywzNjMgIi8+Cjx0ZXh0IHg9IjEyMiIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4wCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjEyMiwzNjMgMTIyLDM2OCAiLz4KPHRleHQgeD0iMjcwIiB5PSIzNzMiIGR5PSIwLjc2ZW0iIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtZmFtaWx5PSJzYW5zLXNlcmlmIiBmb250LXNpemU9IjkuNjc3NDE5MzU0ODM4NzEiIG9wYWNpdHk9IjEiIGZpbGw9IiMwMDAwMDAiPgowLjEKPC90ZXh0Pgo8cG9seWxpbmUgZmlsbD0ibm9uZSIgb3BhY2l0eT0iMSIgc3Ryb2tlPSIjMDAwMDAwIiBzdHJva2Utd2lkdGg9IjEiIHBvaW50cz0iMjcwLDM2MyAyNzAsMzY4ICIvPgo8dGV4dCB4PSI0MTgiIHk9IjM3MyIgZHk9IjAuNzZlbSIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1mYW1pbHk9InNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iOS42Nzc0MTkzNTQ4Mzg3MSIgb3BhY2l0eT0iMSIgZmlsbD0iIzAwMDAwMCI+CjAuMgo8L3RleHQ+Cjxwb2x5bGluZSBmaWxsPSJub25lIiBvcGFjaXR5PSIxIiBzdHJva2U9IiMwMDAwMDAiIHN0cm9rZS13aWR0aD0iMSIgcG9pbnRzPSI0MTgsMzYzIDQxOCwzNjggIi8+Cjx0ZXh0IHg9IjU2NiIgeT0iMzczIiBkeT0iMC43NmVtIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZm9udC1zaXplPSI5LjY3NzQxOTM1NDgzODcxIiBvcGFjaXR5PSIxIiBmaWxsPSIjMDAwMDAwIj4KMC4zCjwvdGV4dD4KPHBvbHlsaW5lIGZpbGw9Im5vbmUiIG9wYWNpdHk9IjEiIHN0cm9rZT0iIzAwMDAwMCIgc3Ryb2tlLXdpZHRoPSIxIiBwb2ludHM9IjU2NiwzNjMgNTY2LDM2OCAiLz4KPHJlY3QgeD0iMTIyIiB5PSIzMzAiIHdpZHRoPSI0ODgiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjI5OCIgd2lkdGg9IjQ5MSIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMjY2IiB3aWR0aD0iNDkzIiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIyMzQiIHdpZHRoPSI0OTQiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjIwMiIgd2lkdGg9IjQ5NiIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iMTcwIiB3aWR0aD0iNDk3IiBoZWlnaHQ9IjMyIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPHJlY3QgeD0iMTIyIiB5PSIxMzgiIHdpZHRoPSI0OTkiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjEwNiIgd2lkdGg9IjUyMyIgaGVpZ2h0PSIzMiIgb3BhY2l0eT0iMC42NSIgZmlsbD0iIzAwMDBGRiIgc3Ryb2tlPSJub25lIi8+CjxyZWN0IHg9IjEyMiIgeT0iNzQiIHdpZHRoPSI1MzEiIGhlaWdodD0iMzIiIG9wYWNpdHk9IjAuNjUiIGZpbGw9IiMwMDAwRkYiIHN0cm9rZT0ibm9uZSIvPgo8cmVjdCB4PSIxMjIiIHk9IjQxIiB3aWR0aD0iNTMxIiBoZWlnaHQ9IjMzIiBvcGFjaXR5PSIwLjY1IiBmaWxsPSIjMDAwMEZGIiBzdHJva2U9Im5vbmUiLz4KPC9zdmc+)

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