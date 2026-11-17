---
title: 'Credit Card Fraud Detection: Anomaly Detection'
description: 'Detect fraudulent credit card transactions using autoencoders, isolation forests, and threshold tuning under extreme class imbalance in Rust.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 43
tags: ['machine-learning', 'rust', 'fraud-detection', 'anomaly-detection', 'imbalanced-data']
---

# Machine Learning Project: Credit Card Fraud Detection

Credit card fraud detection is the hardest classical ML problem in
production: **0.172% of transactions are fraud** — roughly 1 in 580.
At this imbalance, accuracy is completely meaningless; a model that
predicts "legitimate" for everything gets 99.83% accuracy and catches
zero fraud.

The Kaggle Credit Card Fraud dataset has 284,807 transactions with
anonymized PCA features (V1–V28) plus `Amount` and `Time`.

In this project you will:

1. Understand **extreme class imbalance** and why accuracy fails.
2. Use **anomaly detection** — train on legitimate transactions only.
3. Build an **autoencoder** reconstruction-error detector in Rust.
4. Tune thresholds for the precision/recall trade-off real fraud teams face.

> Rust ML note: `burn` for the autoencoder neural net, `ndarray` for
> preprocessing. The PCA features are already anonymized — no raw card data.

## Why fraud detection breaks standard ML

| Problem                 | Why it matters                                               |
| ----------------------- | ------------------------------------------------------------ |
| **Extreme imbalance**   | 99.83% legitimate — accuracy is useless                      |
| **Concept drift**       | Fraud patterns evolve; yesterday's model fails today         |
| **Adversarial**         | Fraudsters adapt to evade detection                          |
| **Cost asymmetry**      | False positive = angry customer; false negative = fraud loss |
| **Anonymized features** | V1–V28 are PCA components — no interpretable fields          |

This is why production fraud systems use **anomaly detection** (learn
what "normal" looks like) rather than supervised classification.

```rust
// :dep polars = { version = "0.44", features = ["lazy"] }
// :dep ndarray = "0.16"

use polars::prelude::*;

let df = CsvReadOptions::default()
    .with_has_header(true)
    .try_into_reader_with_file_path(Some("creditcard.csv".into()))
    .unwrap()
    .finish()
    .unwrap();

println!("shape: {:?}", df.shape());

// Class distribution — the imbalance problem in numbers
let fraud_count = df.column("Class").unwrap()
    .u8().unwrap().into_iter()
    .filter(|v| v.map(|x| *x == 1).unwrap_or(false))
    .count();
let legit_count = df.height() - fraud_count;

println!("legitimate: {} ({:.2}%)", legit_count,
         legit_count as f64 / df.height() as f64 * 100.0);
println!("fraud:      {} ({:.4}%)", fraud_count,
         fraud_count as f64 / df.height() as f64 * 100.0);
```

## Anomaly detection — train on "normal" only

Instead of learning fraud patterns (which are rare and evolving), learn
what **legitimate** transactions look like. Anything that deviates is
suspicious.

Two approaches:

1. **Isolation Forest**: random splits isolate anomalies quickly —
   fraud takes fewer splits to isolate than normal points.
2. **Autoencoder**: compress transactions to a bottleneck, then
   reconstruct. Legitimate transactions reconstruct well; fraud
   reconstructs poorly → high reconstruction error = anomaly.

```rust
// Isolation Forest — simpler, faster, good baseline
// sklearn: IsolationForest(contamination=0.01, random_state=42)
//
// linfa-anomaly or hand-rolled:
//   for each tree: randomly pick feature + split value
//   anomaly score = avg path length to isolate the point
//   shorter path = more anomalous

// Autoencoder approach — more powerful, captures non-linear patterns
use burn::module::Module;
use burn::nn::{Linear, LinearConfig, Relu};
use burn::tensor::{Tensor, backend::Backend};

type B = NdArray<f32>;

#[derive(Module, Debug)]
struct Autoencoder<B: Backend> {
    encoder: Vec<Linear<B>>,
    decoder: Vec<Linear<B>>,
}

impl<B: Backend> Autoencoder<B> {
    fn new(input_dim: usize, bottleneck: usize, device: &B::Device) -> Self {
        Self {
            encoder: vec![
                LinearConfig::new(input_dim, 64).init(device),
                LinearConfig::new(64, bottleneck).init(device),
            ],
            decoder: vec![
                LinearConfig::new(bottleneck, 64).init(device),
                LinearConfig::new(64, input_dim).init(device),
            ],
        }
    }

    fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let mut h = x;
        for (i, layer) in self.encoder.iter().enumerate() {
            h = layer.forward(h);
            if i < self.encoder.len() - 1 { h = h.relu(); }
        }
        for (i, layer) in self.decoder.iter().enumerate() {
            h = layer.forward(h);
            if i < self.decoder.len() - 1 { h = h.relu(); }
        }
        h
    }
}
```

## Training the autoencoder

Train **only on legitimate transactions** — the model learns to compress
and reconstruct "normal" spending patterns. Fraud transactions, being
different, will have higher reconstruction error.

```
reconstruction_error(x) = mean((x - autoencoder(x))²)
```

Threshold on this error: above threshold → flag as fraud.

```rust
// Split: train on legitimate only, validate on held-out legitimate + all fraud
let legit: Vec<_> = /* filter Class == 0 */;
let fraud: Vec<_>  = /* filter Class == 1 */;

let (train_legit, val_legit) = legit.split_at(legit.len() * 9 / 10);

// Standardize features using train stats only
let mean = column_means(&train_legit);
let std  = column_stds(&train_legit);

// Training loop — MSE loss on reconstruction
for epoch in 1..=epochs {
    for batch in train_legit.chunks(batch_size) {
        let x = batch_tensor::<B>(batch, &mean, &std, &device);
        let reconstructed = model.forward(x.clone());
        let loss = mse_loss(reconstructed, x);
        // backprop + optimizer step (same as MNIST project)
    }
}

// Reconstruction error on validation set
let errors_legit: Vec<f64> = val_legit.iter()
    .map(|t| reconstruction_error(&model, t, &mean, &std))
    .collect();
let errors_fraud: Vec<f64> = fraud.iter()
    .map(|t| reconstruction_error(&model, t, &mean, &std))
    .collect();

println!("legit error: mean {:.4}, p95 {:.4}",
         mean(&errors_legit), percentile(&errors_legit, 95));
println!("fraud error: mean {:.4}", mean(&errors_fraud));
```

## Threshold selection — the business trade-off

The threshold determines the operating point:

| Threshold | Precision | Recall   | Business meaning                                          |
| --------- | --------- | -------- | --------------------------------------------------------- |
| Low       | low       | high     | Catch most fraud, but many false alarms → angry customers |
| Medium    | balanced  | balanced | Typical operating point                                   |
| High      | high      | low      | Only flag obvious fraud → miss sophisticated attacks      |

Real fraud teams start with **high precision** (don't annoy customers)
and lower the threshold as they gather feedback.

```rust
// Choose threshold at a target precision (e.g., 80%)
fn threshold_for_precision(errors_legit: &[f64], errors_fraud: &[f64],
                          target_precision: f64) -> f64 {
    let mut sorted: Vec<f64> = errors_legit.iter().cloned().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // binary search for threshold achieving target precision
    let mut lo = sorted[0];
    let mut hi = sorted[sorted.len() - 1];
    for _ in 0..50 {
        let mid = (lo + hi) / 2.0;
        let tp = errors_fraud.iter().filter(|&&e| e > mid).count();
        let fp = errors_legit.iter().filter(|&&e| e > mid).count();
        let precision = tp as f64 / (tp + fp).max(1) as f64;
        if precision < target_precision { lo = mid; } else { hi = mid; }
    }
    hi
}

let threshold = threshold_for_precision(&errors_legit, &errors_fraud, 0.80);
println!("threshold for 80% precision: {:.4}", threshold);
```

## Evaluation under extreme imbalance

| Model                | ROC-AUC | PR-AUC | Precision@80%recall |
| -------------------- | ------- | ------ | ------------------- |
| Isolation Forest     | ~0.90   | ~0.55  | ~0.30               |
| Autoencoder          | ~0.92   | ~0.60  | ~0.35               |
| Supervised (XGBoost) | ~0.95   | ~0.70  | ~0.45               |
| Ensemble + rules     | ~0.97   | ~0.80  | ~0.60               |

**Key insight**: supervised models outperform anomaly detection _when you
have labeled fraud data_. The Kaggle dataset has labels — production
systems collect them over time. Start with anomaly detection for cold
start, then switch to supervised once you have enough fraud examples.

## Key takeaways

1. **Accuracy is meaningless at 0.17% fraud rate** — use PR-AUC and
   precision/recall at fixed operating points.
2. **Anomaly detection handles cold start** — no fraud labels needed.
3. **Autoencoders capture non-linear anomalies** better than isolation
   forests, but need careful threshold tuning.
4. **Production fraud is a system, not a model** — rules + ML + human
   review + feedback loops.

**Final project:** image classification with transfer learning — the
modern approach to vision ML.
