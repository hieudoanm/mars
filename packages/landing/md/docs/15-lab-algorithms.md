---
title: Lab Algorithms
description: Complete catalogue of machine learning algorithms in Mars Lab
author: Project Mars Team
tags: [lab, algorithms, machine-learning, classification, regression, clustering]
---

# Algorithm Catalogue

Mars Lab includes 60+ machine learning algorithms organized by task type. Each algorithm has configurable hyperparameters and supports ONNX export where applicable.

## Task Types

### Classification
Predict discrete class labels from input features.

**Label required:** Yes

### Regression
Predict continuous numerical values from input features.

**Label required:** Yes

### Semi-Supervised
Learn from partially labelled data using both labelled and unlabelled examples.

**Label required:** Partial

### Clustering
Group similar data points without predefined labels.

**Label required:** No

### Anomaly Detection
Identify unusual patterns or outliers in data.

**Label required:** No

### Reinforcement Learning
Learn optimal actions through rewards and penalties.

**Label required:** No

## Classification Algorithms

### Linear Models

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `logistic_regression` | Logistic regression classifier | `C`, `max_iter`, `solver` |
| `logistic_regression_cv` | Logistic regression with built-in CV | `cv`, `max_iter` |
| `linear_svc` | Linear support vector classifier | `C`, `penalty` |
| `ridge_classifier` | Ridge regression for classification | `alpha` |
| `perceptron` | Linear perceptron classifier | `max_iter`, `penalty` |
| `passive_aggressive_classifier` | Online passive-aggressive classifier | `C`, `max_iter` |
| `sgd_classifier` | Linear classifier with SGD | `alpha`, `max_iter`, `loss` |

### Tree-Based Models

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `decision_tree_classifier` | Single decision tree | `max_depth`, `min_samples_split`, `min_samples_leaf` |
| `random_forest_classifier` | Ensemble of decision trees | `n_estimators`, `max_depth`, `max_features` |
| `extra_trees_classifier` | Extremely randomized trees | `n_estimators`, `max_depth`, `max_features` |
| `extra_tree_classifier` | Single extra tree | `max_depth`, `min_samples_split` |
| `gradient_boosting_classifier` | Gradient boosting classifier | `n_estimators`, `learning_rate`, `max_depth` |
| `hist_gradient_boosting_classifier` | Histogram-based gradient boosting | `max_iter`, `learning_rate`, `max_leaf_nodes` |
| `ada_boost_classifier` | Adaptive boosting | `n_estimators`, `learning_rate` |
| `bagging_classifier` | Bootstrap aggregating | `n_estimators`, `max_samples`, `max_features` |

### Kernel Methods

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `svm_classifier` | Support vector machine with RBF kernel | `C`, `kernel`, `gamma` |
| `nu_svc` | Nu-support vector classifier | `nu`, `kernel`, `gamma` |

### Naive Bayes

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `gaussian_naive_bayes` | Gaussian Naive Bayes | `var_smoothing` |
| `bernoulli_nb` | Bernoulli Naive Bayes | `alpha` |
| `multinomial_nb` | Multinomial Naive Bayes | `alpha` |
| `complement_nb` | Complement Naive Bayes | `alpha` |
| `categorical_nb` | Categorical Naive Bayes | `alpha` |

### Instance-Based

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `k_neighbors_classifier` | k-Nearest Neighbours | `n_neighbors`, `weights`, `metric` |
| `radius_neighbors_classifier` | Radius-based neighbours | `radius`, `weights` |
| `nearest_centroid` | Nearest centroid classifier | `metric` |

### Probabilistic

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `linear_discriminant_analysis` | Fisher's LDA | `solver`, `shrinkage` |
| `quadratic_discriminant_analysis` | Quadratic discriminant | `reg_param` |
| `gaussian_process_classifier` | Gaussian process classification | `kernel`, `optimizer` |
| `calibrated_classifier_cv` | Probability calibration | `cv`, `method` |

### Neural Networks

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `mlp_classifier` | Multi-layer perceptron | `hidden_layer_sizes`, `activation`, `solver`, `alpha`, `learning_rate_init`, `max_iter` |

### Semi-Supervised

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `self_training_classifier` | Self-training with pseudo-labels | `threshold`, `max_iter` |
| `label_propagation` | Label propagation through kNN graph | `kernel`, `gamma` |
| `label_spreading` | Label spreading via harmonic functions | `kernel`, `alpha` |

## Regression Algorithms

### Linear Models

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `linear_regression` | Ordinary least squares | `fit_intercept`, `normalize` |
| `ridge_regression` | Ridge regression (L2) | `alpha`, `fit_intercept` |
| `lasso_regression` | Lasso regression (L1) | `alpha`, `max_iter` |
| `elastic_net_regression` | Elastic net (L1+L2) | `alpha`, `l1_ratio`, `max_iter` |
| `bayesian_ridge` | Bayesian ridge regression | `alpha_1`, `alpha_2`, `lambda_1`, `lambda_2` |
| `huber_regressor` | Robust Huber regression | `epsilon`, `alpha`, `max_iter` |
| `quantile_regressor` | Quantile regression | `quantile`, `alpha` |
| `theil_sen_regressor` | Theil-Sen robust regression | `max_subpopulation` |
| `ransac_regressor` | RANSAC robust regression | `min_samples`, `residual_threshold` |
| `isotonic_regression` | Isotonic regression | `out_of_bounds` |

### Tree-Based Models

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `decision_tree_regressor` | Single decision tree | `max_depth`, `min_samples_split`, `min_samples_leaf` |
| `random_forest_regressor` | Ensemble of decision trees | `n_estimators`, `max_depth`, `max_features` |
| `extra_trees_regressor` | Extremely randomized trees | `n_estimators`, `max_depth`, `max_features` |
| `extra_tree_regressor` | Single extra tree | `max_depth`, `min_samples_split` |
| `gradient_boosting_regressor` | Gradient boosting regressor | `n_estimators`, `learning_rate`, `max_depth` |
| `hist_gradient_boosting_regressor` | Histogram-based gradient boosting | `max_iter`, `learning_rate`, `max_leaf_nodes` |
| `ada_boost_regressor` | Adaptive boosting | `n_estimators`, `learning_rate` |
| `bagging_regressor` | Bootstrap aggregating | `n_estimators`, `max_samples`, `max_features` |

### Kernel Methods

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `svm_regressor` | Support vector regression | `C`, `kernel`, `gamma`, `epsilon` |
| `nu_svr` | Nu-support vector regression | `nu`, `kernel`, `gamma` |
| `kernel_ridge` | Kernel ridge regression | `alpha`, `kernel`, `gamma` |
| `gaussian_process_regressor` | Gaussian process regression | `kernel`, `alpha`, `optimizer` |

### Instance-Based

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `k_neighbors_regressor` | k-Nearest Neighbours | `n_neighbors`, `weights`, `metric` |
| `radius_neighbors_regressor` | Radius-based neighbours | `radius`, `weights` |

### Generalized Linear Models

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `poisson_regressor` | Poisson regression | `alpha`, `max_iter` |
| `gamma_regressor` | Gamma regression | `alpha`, `max_iter` |
| `tweedie_regressor` | Tweedie regression | `alpha`, `max_iter`, `power` |

### Neural Networks

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `mlp_regressor` | Multi-layer perceptron | `hidden_layer_sizes`, `activation`, `solver`, `alpha`, `learning_rate_init`, `max_iter` |

### Online Learning

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `sgd_regressor` | SGD regressor | `alpha`, `max_iter`, `loss` |
| `passive_aggressive_regressor` | Passive-aggressive regressor | `C`, `max_iter` |
| `linear_svr` | Linear support vector regression | `C`, `epsilon` |

### Feature Selection

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `lars` | Least angle regression | `n_nonzero_coefs`, `eps` |
| `lasso_cv` | Lasso with cross-validation | `cv`, `n_alphas` |

## Clustering Algorithms

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `kmeans` | K-Means clustering | `n_clusters`, `max_iter`, `n_init` |
| `mini_batch_kmeans` | Mini-batch K-Means | `n_clusters`, `batch_size`, `max_iter` |
| `gaussian_mixture` | Gaussian Mixture Models | `n_components`, `covariance_type`, `max_iter` |
| `dbscan` | Density-based spatial clustering | `eps`, `min_samples` |
| `hdbscan` | Hierarchical DBSCAN | `min_cluster_size`, `min_samples` |
| `optics` | Ordering points to identify structure | `min_samples`, `xi` |
| `agglomerative_clustering` | Hierarchical clustering | `n_clusters`, `linkage`, `metric` |
| `spectral_clustering` | Spectral clustering | `n_clusters`, `affinity` |
| `mean_shift` | Mean shift clustering | `bandwidth`, `bin_seeding` |
| `affinity_propagation` | Affinity propagation | `damping`, `max_iter`, `convergence_iter` |
| `birch` | Balanced iterative reducing | `n_clusters`, `threshold` |
| `bisecting_kmeans` | Bisecting K-Means | `n_clusters`, `max_iter` |

## Anomaly Detection Algorithms

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `isolation_forest` | Isolation Forest | `n_estimators`, `contamination`, `max_samples` |
| `one_class_svm` | One-Class SVM | `kernel`, `gamma`, `nu` |
| `local_outlier_factor` | Local Outlier Factor | `n_neighbors`, `contamination`, `novelty` |
| `elliptic_envelope` | Elliptic Envelope | `contamination`, `support_fraction` |

## Reinforcement Learning Algorithms

| Algorithm | Description | Key Hyperparameters |
|-----------|-------------|---------------------|
| `q_learning` | Tabular Q-Learning | `learning_rate`, `discount_factor`, `epsilon` |
| `sarsa` | SARSA algorithm | `learning_rate`, `discount_factor`, `epsilon` |
| `dqn` | Deep Q-Network | `learning_rate`, `discount_factor`, `epsilon`, `batch_size`, `target_update_freq` |

## Hyperparameter Guidelines

### Common Hyperparameters

- **`n_estimators`**: Number of trees/iterations (100-500 for forests)
- **`max_depth`**: Maximum tree depth (None for unlimited, 5-20 typical)
- **`learning_rate`**: Step size for gradient descent (0.01-0.3)
- **`alpha`**: Regularization strength (0.0001-1.0)
- **`max_iter`**: Maximum iterations (100-1000)
- **`random_state`/`seed`**: Random seed for reproducibility

### Classification-Specific

- **`C`**: Regularization parameter for SVM (0.1-10)
- **`kernel`**: Kernel function for SVM (`rbf`, `linear`, `poly`, `sigmoid`)
- **`gamma`**: Kernel coefficient (`scale`, `auto`, or float)
- **`class_weight`**: Class weights for imbalanced data (`balanced`, `None`)

### Regression-Specific

- **`epsilon`**: Epsilon for SVR and Huber (0.01-1.0)
- **`quantile`**: Quantile for quantile regression (0.0-1.0)
- **`fit_intercept`**: Whether to fit intercept (true/false)

### Clustering-Specific

- **`n_clusters`/`n_components`**: Number of clusters
- **`eps`**: Epsilon for DBSCAN (distance threshold)
- **`min_samples`**: Minimum samples for DBSCAN
- **`linkage`**: Linkage criterion for hierarchical clustering

## ONNX Export Support

Most algorithms support ONNX export. Exceptions:

- Algorithms marked `NO_ONNX` in the catalogue
- Semi-supervised algorithms (label propagation, label spreading)
- Some specialized algorithms (calibrated classifier, Gaussian process)

Check the algorithm's `exports` field in the catalogue for supported formats.

## Choosing Algorithms

### For Classification
- **Small datasets**: Logistic Regression, Naive Bayes, SVM
- **Large datasets**: Random Forest, Gradient Boosting, MLP
- **Imbalanced data**: Class-weighted algorithms, SMOTE preprocessing
- **High-dimensional**: Linear models, Naive Bayes
- **Non-linear**: Random Forest, Gradient Boosting, Kernel methods

### For Regression
- **Linear relationships**: Linear Regression, Ridge, Lasso
- **Non-linear**: Random Forest, Gradient Boosting, Kernel methods
- **Robust to outliers**: Huber, RANSAC, Theil-Sen
- **Quantile estimation**: Quantile Regression
- **Count data**: Poisson, Gamma, Tweedie regression

### For Clustering
- **Known k**: K-Means, Gaussian Mixture
- **Unknown k**: DBSCAN, HDBSCAN, OPTICS
- **Hierarchical**: Agglomerative, Spectral
- **Large datasets**: Mini-Batch K-Means, BIRCH

### For Anomaly Detection
- **Tabular data**: Isolation Forest, One-Class SVM, LOF
- **High-dimensional**: Isolation Forest
- **Density-based**: Local Outlier Factor

## Next Steps

- [Mars Lab overview](/docs/13-lab-overview/) — Features and architecture
- [CLI reference](/docs/14-lab-cli/) — Command-line usage
- [Workflows](/docs/16-lab-workflows/) — Common use cases
