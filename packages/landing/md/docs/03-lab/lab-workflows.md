---
order: 15
title: Lab Workflows
description: Common workflows and examples for Mars Lab machine learning training
author: Project Mars Team
tags: [lab, workflows, examples, machine-learning, training]
---

# Lab Workflows

This guide covers common workflows for using Mars Lab to train machine learning models from CSV datasets to ONNX export.

## Dataset Preparation

### Uploading CSV Files

Mars Lab accepts CSV files with the following requirements:

- **Format**: UTF-8 encoded CSV with header row
- **Size**: Up to `MARS_LAB_MAX_UPLOAD_MB` (default 10MB)
- **Columns**: Mixed types supported (numeric, categorical, text)
- **Missing values**: Empty cells, `NaN`, `null` are handled automatically

**Example CSV:**

```csv
age,income,education,purchased
25,50000,Bachelor,yes
30,75000,Master,yes
35,60000,Bachelor,no
28,55000,PhD,yes
```

### Dataset Insights

After upload, Mars Lab provides:

- **Row count** and **column count**
- **Column types** (inferred from data)
- **Missing value counts** per column
- **Sample values** for each column
- **Warnings** for potential data issues

### Best Practices

1. **Clean data first**: Remove or handle missing values before upload
2. **Consistent formatting**: Ensure consistent date/number formats
3. **Descriptive headers**: Use clear column names
4. **Categorical encoding**: Mars Lab handles categorical data automatically
5. **Feature selection**: Choose relevant features for your task

## Classification Workflow

### Customer Churn Prediction

**Goal**: Predict which customers will churn based on usage data.

**Dataset**: `churn_data.csv`

```csv
customer_id,age,tenure,monthly_charges,total_charges,support_calls,churned
1,45,24,65.0,1500.0,2,no
2,32,12,49.9,600.0,5,yes
3,28,36,89.9,3200.0,0,no
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset churn_data.csv
   ```

2. **Train classifier**

   ```bash
   mars lab train --dataset-id <uuid> --task classification \
     --algorithm random_forest_classifier \
     --features age,tenure,monthly_charges,total_charges,support_calls \
     --label churned \
     --hyperparameters '{"n_estimators": 100, "max_depth": 10}' \
     --split-ratio 0.2
   ```

3. **Review metrics**
   - Accuracy, precision, recall, F1-score
   - Confusion matrix
   - Feature importance

4. **Export model**
   ```bash
   mars lab export --job-id <job-id> --output churn_model.onnx
   ```

**Algorithm choices**:

- `random_forest_classifier` — Good general purpose, handles non-linear relationships
- `gradient_boosting_classifier` — Often best accuracy, slower training
- `logistic_regression` — Fast, interpretable, good for linear relationships
- `xgboost` — State-of-the-art for tabular data (if available)

## Regression Workflow

### House Price Prediction

**Goal**: Predict house prices based on features.

**Dataset**: `housing.csv`

```csv
sqft,beds,baths,age,location_score,price
1500,3,2,10,8.5,350000
2000,4,3,5,9.2,450000
1200,2,1,15,7.8,280000
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset housing.csv
   ```

2. **Train regressor**

   ```bash
   mars lab train --dataset-id <uuid> --task regression \
     --algorithm gradient_boosting_regressor \
     --features sqft,beds,baths,age,location_score \
     --label price \
     --hyperparameters '{"n_estimators": 100, "learning_rate": 0.1}' \
     --split-ratio 0.2
   ```

3. **Review metrics**
   - RMSE, MAE, R² score
   - Residual plots
   - Prediction vs actual scatter plot

4. **Export model**
   ```bash
   mars lab export --job-id <job-id> --output price_model.onnx
   ```

**Algorithm choices**:

- `random_forest_regressor` — Robust, handles non-linear data well
- `gradient_boosting_regressor` — Often best accuracy for tabular data
- `linear_regression` — Fast, interpretable baseline
- `xgboost` — Excellent for regression tasks

## Clustering Workflow

### Customer Segmentation

**Goal**: Group customers into segments based on behavior.

**Dataset**: `customers.csv`

```csv
customer_id,age,income,spending_score,purchase_frequency
1,25,45000,82,12
2,35,75000,45,8
3,45,95000,23,3
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset customers.csv
   ```

2. **Train clustering model**

   ```bash
   mars lab train --dataset-id <uuid> --task clustering \
     --algorithm kmeans \
     --features age,income,spending_score,purchase_frequency \
     --hyperparameters '{"n_clusters": 5}' \
     --split-ratio 0.0
   ```

3. **Review results**
   - Cluster assignments
   - Cluster centroids
   - Silhouette score
   - 2D/3D visualization

4. **Export model**
   ```bash
   mars lab export --job-id <job-id> --output segmentation_model.onnx
   ```

**Algorithm choices**:

- `kmeans` — Fast, works well with spherical clusters
- `gaussian_mixture` — Soft clustering, probabilistic assignments
- `dbscan` — Finds arbitrarily shaped clusters, handles noise
- `hierarchical` — Creates cluster hierarchy

## Anomaly Detection Workflow

### Fraud Detection

**Goal**: Identify fraudulent transactions.

**Dataset**: `transactions.csv`

```csv
transaction_id,amount,merchant_category,time_of_day,location_risk,velocity_score
1,150.50,grocery,afternoon,low,2
2,2500.00,electronics,night,high,15
3,45.99,restaurant,lunch,low,1
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset transactions.csv
   ```

2. **Train anomaly detector**

   ```bash
   mars lab train --dataset-id <uuid> --task anomaly_detection \
     --algorithm isolation_forest \
     --features amount,merchant_category,time_of_day,location_risk,velocity_score \
     --hyperparameters '{"n_estimators": 100, "contamination": 0.1}'
   ```

3. **Review results**
   - Anomaly scores
   - Anomaly labels
   - Feature importance for anomalies

4. **Export model**
   ```bash
   mars lab export --job-id <job-id> --output fraud_model.onnx
   ```

**Algorithm choices**:

- `isolation_forest` — Fast, effective for high-dimensional data
- `one_class_svm` — Good for novelty detection
- `local_outlier_factor` — Density-based, good for local anomalies
- `elliptic_envelope` — Assumes Gaussian distribution

## Semi-Supervised Workflow

### Text Classification with Limited Labels

**Goal**: Classify text documents with few labelled examples.

**Dataset**: `documents.csv`

```csv
document_id,text,category
1,"Great product, highly recommend",positive
2,"Terrible experience, would not buy",negative
3,"Average quality, nothing special",null
4,"Excellent service and fast delivery",positive
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset documents.csv
   ```

2. **Train semi-supervised model**

   ```bash
   mars lab train --dataset-id <uuid> --task semi_supervised \
     --algorithm self_training_classifier \
     --features text \
     --label category \
     --hyperparameters '{"threshold": 0.8, "max_iter": 10}'
   ```

3. **Review results**
   - Classification accuracy on labelled data
   - Pseudo-label confidence
   - Iteration history

**Algorithm choices**:

- `self_training_classifier` — Iteratively adds confident predictions
- `label_propagation` — Propagates labels through similarity graph
- `label_spreading` — Similar to propagation but with regularization

## Reinforcement Learning Workflow

### Game Playing Agent

**Goal**: Train an agent to play a simple game.

**Dataset**: `game_states.csv`

```csv
state,action,reward,next_state,done
0,1,1.0,1,false
1,2,-0.5,2,false
2,0,2.0,3,true
```

**Steps**:

1. **Upload dataset**

   ```bash
   mars lab dataset game_states.csv
   ```

2. **Train RL agent**

   ```bash
   mars lab train --dataset-id <uuid> --task reinforcement_learning \
     --algorithm q_learning \
     --features state \
     --hyperparameters '{"learning_rate": 0.1, "discount_factor": 0.9, "epsilon": 0.1}'
   ```

3. **Review results**
   - Episode rewards over time
   - Q-table convergence
   - Policy evaluation

**Algorithm choices**:

- `q_learning` — Tabular, good for small state spaces
- `sarsa` — On-policy, more stable
- `dqn` — Deep RL, good for large state spaces

## Model Comparison

### Comparing Algorithms

Train multiple models on the same dataset and compare metrics:

```bash
# Train model 1
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income --label target
# Job ID: job-1

# Train model 2
mars lab train --dataset-id <uuid> --task classification \
  --algorithm gradient_boosting_classifier --features age,income --label target
# Job ID: job-2

# Train model 3
mars lab train --dataset-id <uuid> --task classification \
  --algorithm logistic_regression --features age,income --label target
# Job ID: job-3

# Compare jobs
mars lab job job-1
mars lab job job-2
mars lab job job-3
```

### Evaluation Metrics

**Classification**:

- Accuracy, Precision, Recall, F1-Score
- ROC-AUC, PR-AUC
- Confusion Matrix

**Regression**:

- RMSE, MAE, MAPE
- R² Score
- Residual Analysis

**Clustering**:

- Silhouette Score
- Davies-Bouldin Index
- Calinski-Harabasz Index

**Anomaly Detection**:

- Precision, Recall, F1-Score
- ROC-AUC
- Anomaly Score Distribution

## Hyperparameter Tuning

### Manual Tuning

Try different hyperparameters and compare results:

```bash
# Low learning rate
mars lab train --dataset-id <uuid> --task regression \
  --algorithm gradient_boosting_regressor \
  --features sqft,beds,baths --label price \
  --hyperparameters '{"n_estimators": 100, "learning_rate": 0.01}'

# High learning rate
mars lab train --dataset-id <uuid> --task regression \
  --algorithm gradient_boosting_regressor \
  --features sqft,beds,baths --label price \
  --hyperparameters '{"n_estimators": 100, "learning_rate": 0.3}'
```

### Grid Search Pattern

For systematic tuning, run multiple jobs with different parameters:

```bash
# Depth 3
mars lab train --dataset-id <uuid> --task classification \
  --algorithm decision_tree_classifier --features age,income --label target \
  --hyperparameters '{"max_depth": 3}'

# Depth 5
mars lab train --dataset-id <uuid> --task classification \
  --algorithm decision_tree_classifier --features age,income --label target \
  --hyperparameters '{"max_depth": 5}'

# Depth 10
mars lab train --dataset-id <uuid> --task classification \
  --algorithm decision_tree_classifier --features age,income --label target \
  --hyperparameters '{"max_depth": 10}'
```

## Deploying Models

### ONNX Export

Mars Lab exports models to ONNX format for deployment:

```bash
mars lab export --job-id <uuid> --output model.onnx --format onnx
```

ONNX models can be used with:

- **ONNX Runtime** — Cross-platform inference engine
- **TensorFlow Serving** — Production serving
- **PyTorch** — Via ONNX import
- **Custom applications** — Any ONNX-compatible runtime

### Python Integration

Export to Python formats for use with scikit-learn:

```bash
mars lab export --job-id <uuid> --output model.pkl --format pkl
mars lab export --job-id <uuid> --output model.joblib --format joblib
```

### Model Versioning

Track model versions by job ID:

```bash
# Train version 1
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income --label target
# Job ID: v1

# Train version 2 with different features
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income,education --label target
# Job ID: v2

# Compare and choose best
mars lab job v1
mars lab job v2
```

## Troubleshooting

### Common Issues

**Dataset upload fails**:

- Check file format (must be valid CSV)
- Verify file size (under `MARS_LAB_MAX_UPLOAD_MB`)
- Ensure UTF-8 encoding

**Training fails**:

- Check algorithm compatibility with task type
- Verify feature columns exist in dataset
- Ensure label column is specified for classification/regression
- Check hyperparameter values are valid

**Poor model performance**:

- Try different algorithms
- Tune hyperparameters
- Check for data leakage
- Ensure proper train/test split
- Consider feature engineering

**Export fails**:

- Verify job completed successfully
- Check artefact format is supported
- Ensure output directory exists

### Debugging

Enable debug logging:

```bash
export MARS_LAB_LOG_LEVEL=DEBUG
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income --label target
```

Check job details for error messages:

```bash
mars lab job <job-id>
```

## Next Steps

- [Mars Lab overview](/docs/lab-overview/) — Features and architecture
- [CLI reference](/docs/lab-cli/) — Command-line usage
- [Algorithm catalogue](/docs/lab-algorithms/) — Available algorithms
