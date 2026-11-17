---
title: 'Iris Flower Classification: Your First ML Project'
description: 'Classify iris flowers into three species from sepal and petal measurements using k-nearest neighbors and logistic regression in Rust.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 35
tags: ['machine-learning', 'rust', 'classification', 'scikit-learn', 'linfa']
---

# Machine Learning Project: Iris Flower Classification

The **Iris dataset** is the "hello world" of supervised learning: 150 flowers
measured across four features — sepal length/width and petal length/width —
labelled into three species (_setosa_, _versicolor_, _virginica_).

In this project you will:

1. Load and inspect a tabular dataset.
2. Normalize features and split train/test data.
3. Train a **k-nearest neighbors** classifier and a **logistic regression** model.
4. Evaluate with accuracy and a confusion matrix.

> Rust ML note: we use `ndarray` for arrays and `linfa` for the classifiers.
> The code below assumes those crates; each cell notes the equivalent
> scikit-learn call for readers coming from Python.

## The dataset

| Feature      | Type | Typical range |
| ------------ | ---- | ------------- |
| sepal_length | cm   | 4.3 – 7.9     |
| sepal_width  | cm   | 2.0 – 4.4     |
| petal_length | cm   | 1.0 – 6.9     |
| petal_width  | cm   | 0.1 – 2.5     |

The three classes are linearly separable only partially — _setosa_ is easy,
but _versicolor_ and _virginica_ overlap, which is what makes this a useful
first benchmark.

```rust
// :dep ndarray = "0.16"
// :dep linfa = "0.7"
// :dep linfa-datasets = "0.7"
// :dep linfa-knn = "0.7"
// :dep linfa-logistic = "0.7"

use ndarray::Array2;
use linfa::traits::{Fit, Predict};
use linfa_datasets::Dataset;
use linfa_knn::KNearestNeighbors;
use linfa_logistic::LogisticRegression;

// sklearn.datasets.load_iris() equivalent
let dataset: Dataset<f64, usize, ndarray::Ix1> =
    linfa_datasets::iris().expect("iris dataset");

println!("samples: {}", dataset.nsamples());
println!("features: {}", dataset.nfeatures());
println!("classes: {:?}", dataset.distinct_labels());

// scikit-learn: train_test_split(test_size=0.25, random_state=42)
let (train, test) = Dataset::from(dataset.records().to_owned())
    .map_targets(|t| *t)
    .split_with_rng(0.75, 42);

println!("train: {}, test: {}", train.nsamples(), test.nsamples());
```

## Feature scaling

Features have different units and ranges, so we standardize each column to
zero mean / unit variance before distance- or gradient-based models see them.
This is `sklearn.preprocessing.StandardScaler` in Rust.

```rust
use ndarray::Axis;

fn standardize(train: &Array2<f64>, test: &Array2<f64>)
    -> (Array2<f64>, Array2<f64>)
{
    // mean and std per column, computed on TRAIN only
    let mean = train.mean_axis(Axis(0)).unwrap();
    let std = train.std_axis(Axis(0)).unwrap();

    let scale = |x: &Array2<f64>| {
        (x - &mean.insert_axis(Axis(0)))
            .mapv(|v| v)
            .map_axis(Axis(1), |mut col| {
                let mut scaled = col.to_owned();
                for (v, s) in scaled.iter_mut().zip(std.iter()) {
                    *v /= s;
                }
                scaled
            })
    };
    (scale(train), scale(test))
}

// In a full notebook you would rebuild Dataset objects from the scaled
// arrays; linfa pipelines make this one `Pipeline::new(&[&StandardScaler])` call.
```

## Model 1 — k-nearest neighbors

k-NN is the simplest possible classifier: a point takes the majority label of
its _k_ closest training samples. No training step, no assumptions — which is
why it is the perfect baseline.

```rust
// sklearn.neighbors.KNeighborsClassifier(n_neighbors=5)
let knn = KNearestNeighbors::params_with(5, linfa_knn::distance::Hamming)
    .unwrap()
    .fit(&train)
    .unwrap();

let predictions = knn.predict(&test);
let accuracy = predictions
    .iter()
    .zip(test.targets().iter())
    .filter(|(p, t)| p == t)
    .count() as f64
    / test.nsamples() as f64;

println!("k-NN accuracy: {:.3}", accuracy);
```

## Model 2 — logistic regression

Multinomial logistic regression learns one linear score per class and picks
the argmax — the same algorithm behind `sklearn.linear_model.LogisticRegression`.
It gives us **probabilities**, not just labels.

```rust
// sklearn.linear_model.LogisticRegression(max_iter=200)
let model = LogisticRegression::default()
    .fit(&train)
    .expect("logistic regression failed to converge");

let predictions = model.predict(&test);
let accuracy = predictions
    .iter()
    .zip(test.targets().iter())
    .filter(|(p, t)| p == t)
    .count() as f64
    / test.nsamples() as f64;

println!("logistic regression accuracy: {:.3}", accuracy);

// sklearn .predict_proba() — confidence per class
let probabilities = model.predict_log_proba(&test);
println!("first sample probabilities: {:?}", probabilities.row(0));
```

## Confusion matrix

Accuracy hides _which_ mistakes the model makes. A confusion matrix counts,
for each true class, how many predictions landed in each predicted class.
For iris, the interesting cell is `versicolor → virginica`.

```rust
let n_classes = 3;
let mut confusion = vec![vec![0usize; n_classes]; n_classes];

for (pred, target) in predictions.iter().zip(test.targets().iter()) {
    confusion[*target][*pred] += 1;
}

println!("confusion matrix (rows = true, cols = predicted):");
for row in &confusion {
    println!("{:?}", row);
}
```

## Results

Typical outcomes on a 75/25 split:

| Model               | Accuracy | Notes                                               |
| ------------------- | -------- | --------------------------------------------------- |
| k-NN (k=5)          | ~0.95    | No training; sensitive to feature scaling           |
| Logistic regression | ~0.97    | Probabilistic; versicolor/virginica overlap remains |

Both models struggle on the same pair of classes — a reminder that ~97% is
close to the ceiling for this dataset, not a sign of a weak model.

## Next steps

- Try `linfa-svm` (RBF kernel) and a small `burn` neural network to see if
  they beat the linear ceiling.
- Swap the 75/25 split for stratified 5-fold cross-validation
  (`linfa-evaluation`) to get a more stable estimate.
- Load the real `iris.csv` from UCI with `csv` + `serde` instead of the
  bundled dataset to practice your own data pipeline.

**Key takeaway:** a complete ML loop — data, split, baseline, evaluation —
is the same shape for every project in this series.
