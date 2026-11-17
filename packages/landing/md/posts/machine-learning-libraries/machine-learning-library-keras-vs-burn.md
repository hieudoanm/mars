---
title: 'Keras vs burn: High-Level ML APIs Compared'
description: "Compare Python's Keras with Rust's burn for ergonomic model building and training abstractions in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 27
tags: ['machine-learning', 'keras', 'burn', 'rust', 'python', 'deep-learning', 'mars']
---

# Keras vs burn: High-Level ML Compared

Keras is known for its approachable, high-level API. **`burn`** offers an ergonomic Rust framework for building and training models with a modern developer experience, bridging the gap between low-level tensor control and productive model development.

## What Keras does

Keras provides a high-level deep learning API:

- **Model building** with sequential and functional APIs
- **Layers**: convolutional, recurrent, pooling, normalization
- **Optimizers**: SGD, Adam, RMSprop, etc.
- **Training callbacks** for monitoring and control
- **Pretrained models** via Keras Applications
- **Integration** with TensorFlow, JAX, and PyTorch backends

It became the default interface for deep learning in Python, especially for beginners and rapid prototyping.

## What burn brings to Rust

**`burn`** is a Rust deep learning framework:

- **Training framework** with flexible configuration
- **Neural network modules** (linear, convolution, transformer)
- **Optimizers** (SGD, Adam, AdamW)
- **Training loop** abstractions
- **Type-safe** tensor operations
- **Backend-agnostic** design (supports multiple backends)

The key difference: burn provides a Rust-native approach to deep learning with type safety and ergonomic abstractions, while Keras offers a Python-friendly API with broad backend support.

## Comparison

| Feature         | Keras (Python)              | burn (Rust)                       |
| --------------- | --------------------------- | --------------------------------- |
| Model building  | `model = Sequential([...])` | `ModelConfig::new().build()`      |
| Training        | `model.fit(X, y)`           | `Trainer::fit(model, dataloader)` |
| Type safety     | Runtime                     | Compile-time                      |
| Performance     | Python + backend            | Rust + backend                    |
| Backend support | TensorFlow, JAX, PyTorch    | Multiple backends                 |

## Example: Keras

```python
from tensorflow import keras

model = keras.Sequential([
    keras.layers.Dense(128, activation='relu'),
    keras.layers.Dense(10)
])
model.compile(optimizer='adam', loss='mse')
```

## Example: burn

```rust
use burn::nn::loss::MseLoss;
use burn::train::TrainStep;

let model = ModelConfig::new().build();
let optimizer = AdamConfig::new().init();
```

## When to use each

- **Keras** in Python when you need a high-level API, broad backend support, or integration with the Python ML ecosystem.
- **burn** in Rust when you want a Rust-native deep learning framework with type safety and ergonomic abstractions.

## Try it in Project Mars

Both runtimes share one `.mars` document: Python cells run in a persistent interpreter, Rust cells on the replay runtime with `// :dep` Cargo dependencies — and state stays inside each language's own session.

A minimal dense model — Keras above, the tensor ops burn wraps below:

```python
# Keras side: one Dense layer, one fit call.
import numpy as np
from tensorflow import keras

X = np.random.default_rng(0).normal(size=(32, 4))
y = (X.sum(axis=1) > 0).astype(float)

model = keras.Sequential([keras.layers.Dense(1, input_shape=(4,))])
model.compile(optimizer="sgd", loss="mse")
model.fit(X, y, epochs=2, verbose=0)
```

```rust
// :dep ndarray = "0.16"

use ndarray::{array, Array2};

// burn wraps exactly this: a dense layer is a matmul + bias.
// Forward pass for y = Wx + b on one sample.
let w = Array2::from_shape_vec((1, 4), vec![0.5, -0.5, 0.25, 0.25]).unwrap();
let b = 0.1;
let x = array![1.0, 2.0, -1.0, 0.5];
let y = w.dot(&x) + b;
println!("dense output: {}", y[0]);
```

## How Project Mars supports both

Project Mars supports both Keras and burn in their respective language runtimes:

- **Python cells** can use Keras with standard library imports
- **Rust cells** can use burn via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Models and training states persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore Keras and burn in Project Mars notebooks.
