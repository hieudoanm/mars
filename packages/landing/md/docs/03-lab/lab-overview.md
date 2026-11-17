---
order: 12
title: Mars Lab
description: Machine learning training studio for CSV datasets with ONNX export
author: Project Mars Team
tags: [lab, machine-learning, training, onnx, csv, algorithms]
---

# Mars Lab

Mars Lab is a machine learning training studio built into Project Mars. It turns CSV datasets into trained models with ONNX export, all powered by pure Rust.

## What is Mars Lab?

Mars Lab is the ML training component of Project Mars. It provides:

- **CSV dataset management** — Upload, preview, and analyze datasets
- **Algorithm catalogue** — 60+ ML algorithms across 6 task types
- **Training pipeline** — Preprocessing, training, metrics, and charts
- **ONNX export** — Export trained models to the industry-standard ONNX format
- **Job management** — Track training jobs with status, metrics, and artefacts
- **Desktop UI** — Full lab interface in the Mars desktop application
- **CLI access** — Headless training via `mars lab` commands

## Task Types

Mars Lab supports six machine learning task types:

| Task Type                | Description                        | Label Required |
| ------------------------ | ---------------------------------- | -------------- |
| `classification`         | Predict discrete class labels      | Yes            |
| `regression`             | Predict continuous values          | Yes            |
| `semi_supervised`        | Learn from partially labelled data | Partial        |
| `clustering`             | Group similar data points          | No             |
| `anomaly_detection`      | Identify unusual patterns          | No             |
| `reinforcement_learning` | Learn from rewards and actions     | No             |

## Algorithm Catalogue

Mars Lab includes 60+ algorithms organized by task type:

### Classification Algorithms

- Logistic Regression, Random Forest, Gradient Boosting
- Decision Trees, Extra Trees, k-Nearest Neighbours
- Naive Bayes variants (Gaussian, Bernoulli, Multinomial, Complement, Categorical)
- Support Vector Machines (SVC, NuSVC)
- MLP Classifier, Linear Discriminant Analysis
- And 20+ more algorithms

### Regression Algorithms

- Linear Regression, Ridge, Lasso, Elastic Net
- Random Forest, Gradient Boosting, Decision Trees
- Support Vector Regression, Kernel Ridge
- MLP Regressor, Huber, Quantile Regression
- And 20+ more algorithms

### Clustering Algorithms

- K-Means, Mini-Batch K-Means
- Gaussian Mixture Models
- DBSCAN, HDBSCAN, OPTICS
- Agglomerative Clustering, Spectral Clustering
- And more clustering algorithms

### Anomaly Detection Algorithms

- Isolation Forest
- One-Class SVM
- Local Outlier Factor
- Elliptic Envelope

### Semi-Supervised Algorithms

- Self-Training Classifier
- Label Propagation
- Label Spreading

### Reinforcement Learning Algorithms

- Q-Learning
- SARSA
- Deep Q-Networks (DQN)

## Getting Started

### Upload a Dataset

In the desktop app, click "Upload CSV" to add a dataset. Mars Lab will:

- Parse the CSV file
- Infer column types
- Detect missing values
- Generate dataset insights
- Store the dataset with SHA256 deduplication

### Create a Training Job

1. Select a dataset
2. Choose a task type (classification, regression, etc.)
3. Select feature columns and optional label column
4. Choose an algorithm from the catalogue
5. Configure hyperparameters
6. Set train/test split ratio and random seed
7. Click "Train" to start the job

### Monitor Training

Mars Lab tracks job status through stages:

- `queued` → `validating` → `training` → `evaluating` → `exporting` → `completed`

View metrics, charts, and download ONNX artefacts when training completes.

### Export Models

Trained models can be exported to:

- **ONNX** — Industry-standard format for ML model deployment
- **PKL** — Python pickle format
- **Joblib** — Python joblib format

## CLI Usage

Mars Lab is also accessible via the CLI:

```bash
# List available algorithms
mars lab algorithms --task classification

# Upload a dataset
mars lab dataset data.csv

# List datasets
mars lab datasets

# Train a model
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income \
  --label target --hyperparameters '{"n_estimators": 100}'

# Check job status
mars lab job <job-id>

# List all jobs
mars lab jobs

# Export artefact
mars lab export --job-id <job-id> --output model.onnx --format onnx
```

## Configuration

Mars Lab uses environment variables for configuration:

| Variable                       | Default                          | Description                               |
| ------------------------------ | -------------------------------- | ----------------------------------------- |
| `DATABASE_URL`                 | `sqlite://.mars-lab-data/lab.db` | SQLite database path                      |
| `MARS_LAB_DATA_DIR`            | `.mars-lab-data`                 | Data directory for datasets and artefacts |
| `MARS_LAB_MAX_UPLOAD_MB`       | `10`                             | Maximum upload size in MB                 |
| `MARS_LAB_MAX_CONCURRENT_JOBS` | `1`                              | Maximum concurrent training jobs          |
| `MARS_LAB_JOB_TIMEOUT_SECONDS` | `1800`                           | Job timeout in seconds                    |
| `MARS_LAB_JOB_MAX_ATTEMPTS`    | `2`                              | Maximum retry attempts per job            |

## Architecture

Mars Lab is built on pure Rust with these components:

- **DataFrame** — CSV parsing and data manipulation
- **Catalogue** — Algorithm registry and metadata
- **Training** — Preprocessing, training, metrics, and charts
- **Storage** — Dataset and artefact file management
- **Database** — SQLite persistence for jobs, datasets, and artefacts
- **Export** — ONNX model export
- **Insights** — Dataset analysis and statistics

The same core library backs both the CLI and the desktop application.

## Next Steps

- [CLI reference](/docs/cli/) — Full `mars lab` command documentation
- [Algorithm catalogue](/docs/lab-algorithms/) — Detailed algorithm descriptions
- [Workflows](/docs/lab-workflows/) — Common training workflows and examples
