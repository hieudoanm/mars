---
title: 'MNIST Digit Recognition: Computer Vision Project'
description: 'Recognize handwritten digits from 28x28 pixel images using a convolutional neural network trained with burn in Rust.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 37
tags: ['machine-learning', 'rust', 'deep-learning', 'cnn', 'mnist', 'burn']
---

# Machine Learning Project: MNIST Handwritten Digit Recognition

**MNIST** is the "Hello World" of deep learning: 70,000 grayscale images of
handwritten digits (28×28 pixels), labelled 0–9. It is small enough to train
on a laptop, hard enough to teach real concepts, and the dataset every
neural network tutorial starts with.

In this project you will:

1. Understand what a pixel matrix actually is.
2. Build a **convolutional neural network (CNN)** — the architecture that
   made image recognition practical.
3. Train it with the **burn** deep learning framework in Rust.
4. Evaluate accuracy and visualize predictions.

> Rust ML note: burn is Rust's PyTorch equivalent — tensor operations,
> autograd, and a module API. The code mirrors `torchvision`/PyTorch idioms.

## What the data looks like

Each image is a 28×28 matrix of pixel intensities, 0 (black) to 255 (white).
After normalization they live in roughly `[0, 1]`. A digit "7" might look
like:

```
0 0 0 0 1 1 1 0 0
0 0 0 1 1 1 1 0 0
0 0 1 1 0 0 0 0 0
...
```

Fully-connected networks can solve MNIST, but CNNs are better because they
exploit **spatial structure**: a digit is made of local strokes, and a
convolution kernel detects the same stroke anywhere in the image.

```rust
// :dep burn = { version = "0.19", features = ["ndarray"] }
// :dep mnist = "0.5"   // pure-Rust MNIST loader

use burn::backend::NdArray;
use burn::module::Module;
use burn::nn::{Conv2d, Conv2dConfig, Linear, LinearConfig, Relu};
use burn::tensor::{Tensor, backend::Backend};

type B = NdArray<f32>;

// A tiny CNN: conv → relu → conv → relu → flatten → linear
#[derive(Module, Debug)]
struct DigitNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    fc1:   Linear<B>,
}

impl<B: Backend> DigitNet<B> {
    fn new(device: &B::Device) -> Self {
        Self {
            conv1: Conv2dConfig::new([1, 16], [3, 3]).init(device),
            conv2: Conv2dConfig::new([16, 32], [3, 3]).init(device),
            fc1:   LinearConfig::new(32 * 12 * 12, 10).init(device),
        }
    }

    fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        // x: [batch, 1, 28, 28]
        let x = self.conv1.forward(x).relu();
        let x = self.conv2.forward(x).relu();
        let x = x.flatten(1, 3);            // [batch, 32*12*12]
        self.fc1.forward(x)
    }
}
```

## Data loading

`mnist` is a pure-Rust crate that downloads and decodes the IDX file
format. Images come out as `[u8; 784]` (28×28), labels as `u8`.

```rust
use mnist::{Mnist, Nx28};

let dataset = Mnist::new();
let train_images: Vec<Nx28> = dataset.train_images.iter().cloned().collect();
let train_labels: Vec<u8> = dataset.train_labels.iter().cloned().collect();
let test_images  = &dataset.test_images[..];
let test_labels  = &dataset.test_labels[..];

println!("train: {} images, {} labels", train_images.len(), train_labels.len());
println!("test:  {} images", test_images.len());

// One image as a flat [u8; 784] — reshape to [1, 1, 28, 28] for the CNN
let img = &train_images[0];
let tensor = Tensor::<B, 4>::from_data(
    img.as_slice()
       .iter()
       .map(|&p| p as f32 / 255.0)   // normalize to [0, 1]
       .collect::<Vec<_>>()
       .into_data()
       .reshape([1, 1, 28, 28]),
    &Default::default(),
);
```

## Training loop

The training loop is the same in every deep learning framework:

```
for epoch in 1..=epochs {
    for (images, labels) in batches {
        logits  = model.forward(images)
        loss    = cross_entropy(logits, labels)
        grads   = loss.backward()
        optimizer.step(grads)
    }
    evaluate on test set
}
```

**Cross-entropy** is the standard classification loss: it heavily penalizes
confident wrong predictions and barely penalizes correct ones.

```rust
use burn::optim::{AdamConfig, GradientsParams};
use burn::tensor::loss::cross_entropy_with_logits;

let device = Default::default();
let mut model = DigitNet::<B>::new(&device);
let mut optim = AdamConfig::new().init();

let epochs = 5;
let batch_size = 128;

for epoch in 1..=epochs {
    let mut total_loss = 0.0f32;
    let mut n_batches = 0usize;

    for chunk in train_images.chunks(batch_size) {
        // build batch tensors [batch, 1, 28, 28] and [batch]
        let images_t = batch_images::<B>(chunk, &device);
        let labels_t = batch_labels::<B>(&train_labels[..chunk.len()], &device);

        let logits = model.forward(images_t);
        let loss   = cross_entropy_with_logits(logits, labels_t);

        let grads = loss.backward();
        let grads = optim.step(0.001, model.parameters(), grads);
        model = model.update(grads, &device);   // apply gradients

        total_loss += loss.into_scalar().elem::<f32>();
        n_batches += 1;
    }

    println!("epoch {}: avg loss {:.4}", epoch, total_loss / n_batches as f32);
}
```

## Evaluation

After training, run the model on the 10,000-image test set and compute
**accuracy** — fraction of images where `argmax(logits) == label`. State-of-
the-art on MNIST is ~99.8%; our small CNN should land around 98–99%.

```rust
fn evaluate<B: Backend>(model: &DigitNet<B>, images: &[Nx28], labels: &[u8],
                 device: &B::Device) -> f64 {
    let mut correct = 0usize;
    for (img, label) in images.iter().zip(labels.iter()) {
        let x = image_tensor::<B>(img, device);
        let logits = model.forward(x);
        let pred = logits.argmax(1).into_data().to_vec::<i64>().unwrap()[0];
        if pred as u8 == *label { correct += 1; }
    }
    correct as f64 / labels.len() as f64
}

let acc = evaluate(&model, test_images, test_labels, &device);
println!("test accuracy: {:.2}%", acc * 100.0);

// Confusion matrix: which digits get confused?
// Typical confusions: 4↔9, 3↔5, 7↔9 — visually similar shapes.
```

## Why CNNs beat fully-connected nets here

A dense network treats pixel (0,0) and pixel (27,27) as unrelated inputs —
it must learn every position of every stroke separately. A convolutional
layer slides a small kernel (e.g. 3×3) across the image, so the same edge-
detector fires anywhere. This **translation invariance** is why CNNs need
far fewer parameters and generalize far better.

## Next steps

- Add **max-pooling** layers and a second fully-connected layer — accuracy
  jumps to ~99%.
- Try **data augmentation**: random rotations/shifts of training images.
- Swap `NdArray` for `Wgpu` backend to train on GPU — burn makes this a
  one-line type change.
- For real projects, start from a pretrained model (ResNet/EfficientNet via
  `tch-rs`) instead of training from scratch — see the cats-vs-dogs project.

**Key takeaway:** CNNs are the right tool whenever your data has spatial
structure. The same conv → pool → flatten → dense pattern works for CIFAR-10,
medical images, and satellite imagery.
