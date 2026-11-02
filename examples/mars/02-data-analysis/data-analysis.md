# Data analysis

A small dependency-free summary workflow.

```rust
let values = vec![3_i32, 7, 4, 9, 12, 6];
let total: i32 = values.iter().sum();
let mean = total as f64 / values.len() as f64;
(mean, values.iter().min(), values.iter().max())
```

```text
(6.833333333333333, Some(3), Some(12))
```
