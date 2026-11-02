# Linear regression

Compute a simple least-squares slope.

```rust
let x = vec![1.0_f64, 2.0, 3.0, 4.0];
let y = vec![2.0_f64, 4.1, 5.9, 8.2];
```

```rust
let x_mean = x.iter().sum::<f64>() / x.len() as f64;
let y_mean = y.iter().sum::<f64>() / y.len() as f64;
let slope = x.iter().zip(&y).map(|(x, y)| (x - x_mean) * (y - y_mean)).sum::<f64>() / x.iter().map(|x| (x - x_mean).powi(2)).sum::<f64>();
slope
```

```text
2.04
```
