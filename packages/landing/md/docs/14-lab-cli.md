---
title: Mars Lab CLI
description: Command-line interface for Mars Lab machine learning training
author: Project Mars Team
tags: [lab, cli, machine-learning, training, onnx]
---

# Mars Lab CLI

The `mars lab` command provides headless access to Mars Lab's machine learning training capabilities. It allows you to upload datasets, train models, monitor jobs, and export artefacts from the command line.

## Command Summary

```text
mars lab algorithms [--task <type>]
mars lab dataset <file.csv>
mars lab datasets
mars lab train --dataset-id <id> --task <type> --algorithm <id> --features a,b [options]
mars lab job <id>
mars lab jobs
mars lab export --job-id <id> --output <path> [--format onnx|pkl|joblib]
```

## List Algorithms

```bash
mars lab algorithms
mars lab algorithms --task classification
mars lab algorithms --task regression
```

Lists all available algorithms in the catalogue. Use `--task` to filter by task type.

**Task types:** `classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection`, `reinforcement_learning`

## Upload Dataset

```bash
mars lab dataset data.csv
```

Uploads a CSV file to Mars Lab. The command will:
- Parse the CSV and infer column types
- Detect missing values
- Generate dataset insights
- Store the dataset with SHA256 deduplication
- Return a dataset ID for use in training

**Output:**
```text
Dataset uploaded:
  id: <uuid>
  storage_key: <path>
  sha256: <hash>
  size: <bytes> bytes
```

## List Datasets

```bash
mars lab datasets
```

Lists all uploaded datasets in the Mars Lab data directory.

## Train a Model

```bash
mars lab train --dataset-id <uuid> --task <type> --algorithm <id> \
  --features col1,col2,col3 [--label target] [--hyperparameters json] \
  [--split-ratio 0.2] [--seed 42]
```

Creates and executes a training job. Required parameters:

- `--dataset-id` — Dataset UUID from upload
- `--task` — Task type (classification, regression, etc.)
- `--algorithm` — Algorithm ID from the catalogue
- `--features` — Comma-separated feature column names

Optional parameters:

- `--label` — Label/target column name (required for classification/regression)
- `--hyperparameters` — JSON object of algorithm hyperparameters
- `--split-ratio` — Train/test split ratio (default: 0.2)
- `--seed` — Random seed for reproducibility (default: 42)

**Example:**
```bash
mars lab train --dataset-id abc123 --task classification \
  --algorithm random_forest_classifier \
  --features age,income,education \
  --label purchased \
  --hyperparameters '{"n_estimators": 100, "max_depth": 10}' \
  --split-ratio 0.2 \
  --seed 42
```

**Output:**
```text
Training job <uuid>...
  Task: classification
  Algorithm: random_forest_classifier
  Features: ["age", "income", "education"]
  Label: Some("purchased")

Training completed!
  Model kind: Trees

Metrics:
  {
    "accuracy": 0.95,
    "precision": 0.94,
    "recall": 0.93,
    "f1": 0.94
  }

ONNX exported to: .mars-lab-data/artefacts/<job-id>.onnx
  Size: 12345 bytes
  Job <uuid> completed
```

## Check Job Status

```bash
mars lab job <job-id>
```

Shows detailed information about a training job, including status, metrics, and artefacts.

**Output:**
```text
Job: <uuid>
  Status: completed
  Task: classification
  Algorithm: random_forest_classifier
  Features: ["age", "income", "education"]
  Label: Some("purchased")
  Created: 2026-11-15T10:30:00Z
  Updated: 2026-11-15T10:35:00Z

Metrics:
  {
    "accuracy": 0.95,
    "precision": 0.94,
    "recall": 0.93,
    "f1": 0.94
  }

Artefacts:
  onnx (model.onnx) - 12345 bytes
```

## List Jobs

```bash
mars lab jobs
```

Lists all training jobs with their status, task type, and algorithm.

**Output:**
```text
Jobs:
  <uuid> - completed - classification - random_forest_classifier
  <uuid> - training - regression - linear_regression
  <uuid> - failed - clustering - kmeans
```

## Export Artefact

```bash
mars lab export --job-id <uuid> --output model.onnx --format onnx
mars lab export --job-id <uuid> --output model.pkl --format pkl
mars lab export --job-id <uuid> --output model.joblib --format joblib
```

Exports a trained model artefact to a specified file path.

**Formats:**
- `onnx` — ONNX model format (default)
- `pkl` — Python pickle format
- `joblib` — Python joblib format

## Job Status Lifecycle

Jobs progress through these statuses:

| Status | Description |
|--------|-------------|
| `queued` | Job created, waiting to start |
| `validating` | Validating dataset and parameters |
| `training` | Model training in progress |
| `evaluating` | Computing metrics and charts |
| `exporting` | Exporting artefacts |
| `completed` | Job finished successfully |
| `failed` | Job failed with error |
| `retrying` | Job will be retried after failure |

## Examples

### Classification Workflow

```bash
# Upload dataset
mars lab dataset iris.csv
# Output: id: abc-123

# Train classifier
mars lab train --dataset-id abc-123 --task classification \
  --algorithm random_forest_classifier \
  --features sepal_length,sepal_width,petal_length,petal_width \
  --label species \
  --hyperparameters '{"n_estimators": 100}'
# Output: job id: def-456

# Check status
mars lab job def-456

# Export model
mars lab export --job-id def-456 --output iris_model.onnx
```

### Regression Workflow

```bash
# Upload housing data
mars lab dataset housing.csv

# Train regressor
mars lab train --dataset-id xyz-789 --task regression \
  --algorithm linear_regression \
  --features sqft,beds,baths,age \
  --label price \
  --split-ratio 0.2
```

### Clustering Workflow

```bash
# Upload customer data
mars lab dataset customers.csv

# Train clustering model
mars lab train --dataset-id cust-001 --task clustering \
  --algorithm kmeans \
  --features age,income,spending \
  --hyperparameters '{"n_clusters": 5}'
```

## Configuration

Mars Lab CLI uses the same configuration as the desktop app. Set environment variables to customize:

```bash
export DATABASE_URL="sqlite:///path/to/lab.db"
export MARS_LAB_DATA_DIR="/path/to/data"
export MARS_LAB_MAX_UPLOAD_MB=50
export MARS_LAB_JOB_TIMEOUT_SECONDS=3600
```

See [Mars Lab overview](/docs/13-lab-overview/) for configuration details.

## Error Handling

CLI commands return nonzero exit codes on errors:

- `1` — General error (invalid input, training failure, etc.)

Error messages are printed to stderr with the format:
```text
mars: <error message>
```

## Next Steps

- [Mars Lab overview](/docs/13-lab-overview/) — Features and architecture
- [Algorithm catalogue](/docs/15-lab-algorithms/) — Available algorithms
- [Workflows](/docs/16-lab-workflows/) — Common use cases
