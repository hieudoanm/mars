---
title: 'Image Classification: Transfer Learning Project'
description: 'Classify cats vs dogs using a pretrained ResNet via transfer learning with tch-rs in Rust — the modern approach to vision ML.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 44
tags:
  ['machine-learning', 'rust', 'deep-learning', 'computer-vision', 'transfer-learning', 'pytorch']
---

# Machine Learning Project: Image Classification with Transfer Learning

The **cats vs. dogs** problem is the classic computer vision tutorial:
distinguish two animal classes from photos. The naive approach — training
a CNN from scratch — needs millions of images and days of GPU time.

**Transfer learning** flips this: take a model pretrained on ImageNet
(1.2M images, 1000 classes) and fine-tune just the last layer for your
task. You get 95%+ accuracy on cats-vs-dogs in minutes on a laptop.

In this project you will:

1. Understand **why transfer learning works** — learned features transfer.
2. Load a pretrained **ResNet-18** via `tch-rs` (Rust PyTorch bindings).
3. Fine-tune the classifier head on a small dataset.
4. Compare from-scratch vs. transfer learning — the modern ML workflow.

> Rust ML note: `tch-rs` wraps libtorch — the same PyTorch you'd use in
> Python, from Rust. `image` crate for loading/augmenting images.

## Why transfer learning changed everything

A CNN trained on ImageNet learns **generic visual features** in its early
layers:

- Layer 1: edges, colors, gradients
- Layer 2: textures, simple shapes
- Layer 3: object parts (ears, eyes, fur patterns)
- Layer 4+: object concepts (animal faces, bodies)

These features transfer to _any_ vision task. You only need to retrain
the final classification layer — which has just 2 outputs (cat/dog)
instead of 1000.

```rust
// :dep tch = "0.20"
// :dep image = "0.25"
// :dep walkdir = "2"

use tch::{nn, Device, Tensor};
use image::{DynamicImage, GenericImageView};

let device = Device::cuda_if_available();

// Load pretrained ResNet-18 — automatically downloads weights
// Equivalent: torchvision.models.resnet18(pretrained=True)
let vs = nn::VarStore::new(Device::Cpu);   // pretrained on CPU, move to GPU
let resnet = tch::vision::resnet::ResNet::new(
    &vs.root(), ResNetVariant::ResNet18, Some(1000),
).unwrap();

// Load ImageNet-pretrained weights
let weights_path = "resnet18-bc5d3300.pth";   // download once
vs.load(weights_path).unwrap();

resnet.to_device(&device);
println!("ResNet-18 loaded on {:?}", device);
```

## Image preprocessing — the unglamorous critical step

CNNs expect fixed-size, normalized inputs. The pipeline:

1. **Resize** to 224×224 (ResNet's input size)
2. **Center-crop** if aspect ratio differs
3. **Normalize** with ImageNet mean/std: `mean=[0.485,0.456,0.406]`,
   `std=[0.229,0.224,0.225]`
4. **Convert** to tensor [C,H,W] → batch [N,C,H,W]

```rust
fn load_image(path: &str) -> Tensor {
    let img = image::open(path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();

    // Resize to 256×256, center-crop 224×224
    let resized = image::imageops::resize(&img, 256, 256,
        image::imageops::FilterType::Triangle);
    let x0 = (256 - 224) / 2;
    let y0 = (256 - 224) / 2;
    let cropped = image::imageops::crop_imm(&resized, x0, y0, 224, 224)
        .to_image();

    // HWC u8 → CHW f32, normalized
    let mut data = Vec::with_capacity(3 * 224 * 224);
    for c in 0..3 {
        for y in 0..224 {
            for x in 0..224 {
                let px = cropped.get_pixel(x, y)[c] as f32 / 255.0;
                let mean = [0.485, 0.456, 0.406][c];
                let std  = [0.229, 0.224, 0.225][c];
                data.push((px - mean) / std);
            }
        }
    }

    Tensor::from_slice(&data)
        .reshape([1, 3, 224, 224])
        .to_device(device)
}
```

## Replace the classifier head

ResNet-18's final layer outputs 1000 ImageNet classes. For cats-vs-dogs,
replace it with a 2-class linear layer. Everything else stays frozen —
we're only learning "cat vs dog", not relearning vision.

```rust
use tch::nn::{LinearConfig, Module};

// Access ResNet's penultimate features (before final FC)
// ResNet-18: avgpool → [512] → fc(512→1000)
// We replace fc with fc(512→2)

let mut vs_head = nn::VarStore::new(Device::Cpu);
let head = nn::linear(&vs_head.root(), 512, 2, Default::default());

// Forward pass: features → head
fn forward(resnet: &tch::vision::resnet::ResNet, head: &nn::Linear,
           img: &Tensor) -> Tensor {
    let features = resnet.forward_features(img);   // [N, 512, 7, 7]
    let pooled = tch::Tensor::avg_pool2d(&features,
        [7, 7], [1, 1], [0, 0], false);            // [N, 512, 1, 1]
    let flat = pooled.flatten(1, -1);               // [N, 512]
    head.forward(&flat)                             // [N, 2]
}
```

## Data augmentation — the secret sauce

With only ~25k cats-vs-dogs images, overfitting is the enemy. **Data
augmentation** artificially expands the dataset by applying random
transformations:

- Random horizontal flip (cats/dogs face both ways)
- Random crop + resize (simulates different framing)
- Color jitter (lighting variation)
- Random rotation ±10°

```rust
// image crate has basic ops; for serious augmentation use
// the `imageproc` crate or `fast_image_resize`

fn augment(img: &DynamicImage, rng: &mut impl Rng) -> DynamicImage {
    let mut img = img.clone();

    // random horizontal flip (50% chance)
    if rng.gen_bool(0.5) {
        img = image::imageops::flip_horizontal(&img);
    }

    // random rotation ±10°
    let angle = rng.gen_range(-10.0..10.0);
    img = image::imageops::rotate(&img, angle);

    // random brightness ±10%
    let factor = rng.gen_range(0.9..1.1);
    img = adjust_brightness(&img, factor);

    img
}

// Apply augmentation during training, not validation
// sklearn equivalent: ImageDataGenerator(rotation_range=10, ...)
```

## Fine-tuning strategy

Two approaches:

1. **Feature extraction**: freeze ResNet, train only the head.
   Fast, works with tiny datasets, but limited accuracy.

2. **Fine-tuning**: unfreeze last few ResNet layers + train head.
   Better accuracy, needs more data, slower.

For cats-vs-dogs (~25k images), **fine-tuning the last block** is the
sweet spot — you adapt the features to animals without catastrophic
forgetting of general vision.

```rust
// Fine-tune: unfreeze last ResNet block, lower LR for backbone
let mut params_head = head.params();
let mut params_backbone = resnet.layer4().params();   // last block only

let optimizer = nn::Adam::default()
    .build(&[&params_head, &params_backbone], 1e-4);

// Training loop
for epoch in 1..=epochs {
    for (imgs, labels) in train_batches {
        let logits = forward(&resnet, &head, &imgs);
        let loss = logits.cross_entropy_for_logits(&labels);
        optimizer.set_learning_rate(if epoch < 5 { 1e-3 } else { 1e-4 });
        // backward + step
    }
    evaluate(&resnet, &head, &val_loader);
}
```

## Evaluation

| Approach                      | Accuracy | Training time | Data needed  |
| ----------------------------- | -------- | ------------- | ------------ |
| From scratch CNN              | ~85%     | hours (GPU)   | 100k+ images |
| Transfer learning (head only) | ~95%     | minutes       | 1k+ images   |
| Fine-tuned ResNet             | ~97%     | ~30 min       | 10k+ images  |
| Pretrained + linear probe     | ~96%     | seconds       | 100+ images  |

The last row is the modern extreme: **frozen features + logistic
regression on top** — sometimes called "linear probing" — matches
fine-tuning with a fraction of the compute.

```rust
// Linear probe: freeze ResNet entirely, train only head
// This is shockingly effective — ResNet features are that good

for param in resnet.parameters() {
    param.set_requires_grad(false);   // freeze backbone
}

// Train only head — converges in seconds, not hours
// Use sklearn-style LogisticRegression on extracted features:
// features = resnet.forward_features(all_images).flatten()
// labels = [0,1,0,1,...]

let train_acc = evaluate_linear_probe(&features_train, &labels_train);
let test_acc  = evaluate_linear_probe(&features_test,  &labels_test);
println!("linear probe: train {:.2}%, test {:.2}%", train_acc*100.0, test_acc*100.0);
```

## Key takeaways

1. **Transfer learning is the default for vision ML** — never train
   from scratch unless you have massive data and compute.
2. **Pretrained features are remarkably general** — ResNet trained on
   ImageNet transfers to medical imaging, satellite photos, and yes,
   cats vs dogs.
3. **Fine-tune progressively**: head first, then last block, then
   deeper — with decreasing learning rates.
4. **Linear probing is the new baseline** — frozen features + simple
   classifier often matches full fine-tuning at a fraction of cost.

**This completes the 10-project series.** You've now covered:
classification (iris, spam, sentiment, fraud, image), regression
(house prices), time series (demand), and recommender systems
(movies) — the canonical ML project taxonomy in Rust.
