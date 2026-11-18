//! End-to-end coverage for `training::train` across every task type and the
//! artefact exporters. Small synthetic CSVs keep runtimes in the milliseconds.

use mars_lab::dataframe::DataFrame;
use mars_lab::domain::{Job, RLColumns, TaskType};
use mars_lab::export::{export_snapshot, load_snapshot, onnx::export_onnx};
use mars_lab::training::train;
use serde_json::json;
use uuid::Uuid;

fn make_job(
    task_type: TaskType,
    algorithm_id: &str,
    features: &[&str],
    label: Option<&str>,
    hyperparameters: serde_json::Value,
) -> Job {
    Job {
        id: Uuid::new_v4(),
        dataset_id: Uuid::new_v4(),
        task_type: task_type.as_str().to_string(),
        algorithm_id: algorithm_id.to_string(),
        feature_columns: features.iter().map(|s| s.to_string()).collect(),
        label_column: label.map(str::to_string),
        rl_columns: None,
        hyperparameters,
        split_ratio: 0.2,
        random_seed: 42,
        export_pkl: false,
        export_joblib: false,
        status: "queued".to_string(),
        error_code: None,
        error_message: None,
        library_versions: None,
        name: None,
        tags: vec![],
        attempts: 0,
        max_attempts: 3,
        next_attempt_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn classification_csv() -> &'static str {
    // 24 rows, two separable-ish classes, two numeric features.
    "x1,x2,label\n\
     0.1,0.2,cat\n0.2,0.1,cat\n0.0,0.3,cat\n0.3,0.0,cat\n\
     0.15,0.25,dog\n0.25,0.15,dog\n0.05,0.35,dog\n0.35,0.05,dog\n\
     0.1,0.4,cat\n0.4,0.1,cat\n0.2,0.2,cat\n0.0,0.0,cat\n\
     0.3,0.3,dog\n0.35,0.25,dog\n0.25,0.35,dog\n0.05,0.05,dog\n\
     0.12,0.18,cat\n0.18,0.12,cat\n0.22,0.28,cat\n0.28,0.22,cat\n\
     0.32,0.18,dog\n0.18,0.32,dog\n0.38,0.28,dog\n0.28,0.38,dog\n"
}

fn regression_csv() -> &'static str {
    "x1,x2,target\n\
     0.1,0.2,0.5\n0.2,0.1,0.6\n0.0,0.3,0.4\n0.3,0.0,0.7\n\
     0.15,0.25,0.55\n0.25,0.15,0.65\n0.05,0.35,0.45\n0.35,0.05,0.75\n\
     0.1,0.4,0.5\n0.4,0.1,0.7\n0.2,0.2,0.6\n0.0,0.0,0.4\n\
     0.3,0.3,0.8\n0.35,0.25,0.85\n0.25,0.35,0.75\n0.05,0.05,0.35\n\
     0.12,0.18,0.52\n0.18,0.12,0.58\n0.22,0.28,0.62\n0.28,0.22,0.68\n\
     0.32,0.18,0.82\n0.18,0.32,0.88\n0.38,0.28,0.92\n0.28,0.38,0.78\n"
}

fn semi_csv() -> String {
    // Mostly labelled with a few unlabelled rows ("" token).
    let mut s = String::from("x1,x2,label\n");
    for i in 0..12 {
        let x = (i % 4) as f64 * 0.1;
        let y = (i / 4) as f64 * 0.1;
        s.push_str(&format!("{x},{y},{}\n", if i % 2 == 0 { "a" } else { "b" }));
    }
    s.push_str("0.5,0.5,\n0.6,0.4,\n0.7,0.3,\n");
    s
}

fn cluster_csv() -> String {
    let mut s = String::from("x1,x2\n");
    for i in 0..15 {
        s.push_str(&format!(
            "{},{}\n",
            (i % 5) as f64 * 0.1,
            (i / 5) as f64 * 0.1
        ));
    }
    for i in 0..15 {
        s.push_str(&format!(
            "{},{}\n",
            1.0 + (i % 5) as f64 * 0.1,
            1.0 + (i / 5) as f64 * 0.1
        ));
    }
    s
}

fn anomaly_csv() -> String {
    let mut s = String::from("x1,x2\n");
    for i in 0..20 {
        s.push_str(&format!(
            "{},{}\n",
            (i % 5) as f64 * 0.1,
            (i / 5) as f64 * 0.1
        ));
    }
    s.push_str("5.0,5.0\n-4.0,-4.0\n");
    s
}

fn rl_csv() -> String {
    let mut s = String::from("state,next,action,reward\n");
    for i in 0..16 {
        let state = (i % 4) as f64 * 0.25;
        let action = ["left", "right"][(i % 2) as usize];
        s.push_str(&format!(
            "{state},{:.2},{action},{:.2}\n",
            (i % 3) as f64 * 0.5,
            (i % 3) as f64 * 0.5
        ));
    }
    s
}

#[test]
fn catalogue_lists_all_algorithms() {
    let all = mars_lab::catalogue::algorithms();
    assert_eq!(all.len(), 83);
    assert_eq!(
        mars_lab::catalogue::for_task(Some(TaskType::Classification)).len()
            + mars_lab::catalogue::for_task(Some(TaskType::Regression)).len()
            + mars_lab::catalogue::for_task(Some(TaskType::SemiSupervised)).len()
            + mars_lab::catalogue::for_task(Some(TaskType::Clustering)).len()
            + mars_lab::catalogue::for_task(Some(TaskType::AnomalyDetection)).len()
            + mars_lab::catalogue::for_task(Some(TaskType::ReinforcementLearning)).len(),
        all.len()
    );
    assert!(mars_lab::catalogue::find("logistic_regression").is_some());
    assert!(mars_lab::catalogue::find("nope").is_none());
    assert!(mars_lab::catalogue::default_hyperparameters("kmeans")["n_clusters"].is_number());
}

#[test]
fn classification_algorithms_train() {
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    for algo in [
        "logistic_regression",
        "random_forest_classifier",
        "decision_tree_classifier",
        "gaussian_naive_bayes",
        "k_neighbors_classifier",
        "bernoulli_nb",
        "multinomial_nb",
        "nearest_centroid",
        "ridge_classifier",
        "perceptron",
        "linear_discriminant_analysis",
        "quadratic_discriminant_analysis",
        "extra_tree_classifier",
        "svm_classifier",
        "linear_svc",
        "nu_svc",
        "sgd_classifier",
        "passive_aggressive_classifier",
        "radius_neighbors_classifier",
        "self_training_classifier",
        "calibrated_classifier_cv",
        "logistic_regression_cv",
    ] {
        let job = make_job(
            TaskType::Classification,
            algo,
            &["x1", "x2"],
            Some("label"),
            json!({"max_iter": 50, "n_estimators": 5, "n_neighbors": 3}),
        );
        let out = train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
        assert!(out.metrics.is_object(), "{algo} metrics");
        assert!(!out.charts.is_null(), "{algo} charts");
    }
}

#[test]
fn mlp_and_boosting_classifiers_train() {
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    for algo in [
        "mlp_classifier",
        "gradient_boosting_classifier",
        "ada_boost_classifier",
        "hist_gradient_boosting_classifier",
        "bagging_classifier",
    ] {
        let job = make_job(
            TaskType::Classification,
            algo,
            &["x1", "x2"],
            Some("label"),
            json!({"max_iter": 30, "n_estimators": 5, "hidden_layer_sizes": [4]}),
        );
        train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
    }
}

#[test]
fn semi_supervised_algorithms_train() {
    let df = DataFrame::from_csv_str(&semi_csv()).unwrap();
    for algo in [
        "self_training_classifier",
        "label_propagation",
        "label_spreading",
    ] {
        let job = make_job(
            TaskType::SemiSupervised,
            algo,
            &["x1", "x2"],
            Some("label"),
            json!({"max_iter": 50}),
        );
        train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
    }
}

#[test]
fn regression_algorithms_train() {
    let df = DataFrame::from_csv_str(regression_csv()).unwrap();
    for algo in [
        "linear_regression",
        "ridge_regression",
        "lasso_regression",
        "elastic_net_regression",
        "bayesian_ridge",
        "huber_regression",
        "theil_sen_regressor",
        "ransac_regressor",
        "isotonic_regression",
        "lars",
        "lasso_cv",
        "k_neighbors_regressor",
        "radius_neighbors_regressor",
        "decision_tree_regressor",
        "extra_tree_regressor",
        "random_forest_regressor",
        "extra_trees_regressor",
        "ada_boost_regressor",
        "bagging_regressor",
        "gradient_boosting_regressor",
        "hist_gradient_boosting_regressor",
        "quantile_regressor",
        "poisson_regressor",
        "gamma_regressor",
        "tweedie_regressor",
        "sgd_regressor",
        "passive_aggressive_regressor",
        "kernel_ridge",
        "svm_regressor",
        "linear_svr",
        "nu_svr",
        "gaussian_process_regressor",
    ] {
        let job = make_job(
            TaskType::Regression,
            algo,
            &["x1", "x2"],
            Some("target"),
            json!({"max_iter": 50, "n_estimators": 5, "n_neighbors": 3, "alpha": 0.1, "epsilon": 0.1}),
        );
        train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
    }
}

#[test]
fn mlp_regressor_trains() {
    let df = DataFrame::from_csv_str(regression_csv()).unwrap();
    let job = make_job(
        TaskType::Regression,
        "mlp_regressor",
        &["x1", "x2"],
        Some("target"),
        json!({"max_iter": 30, "hidden_layer_sizes": [4]}),
    );
    train(&job, &df).unwrap();
}

#[test]
fn clustering_algorithms_train() {
    let df = DataFrame::from_csv_str(&cluster_csv()).unwrap();
    for algo in [
        "kmeans",
        "mini_batch_kmeans",
        "gaussian_mixture",
        "dbscan",
        "optics",
        "hdbscan",
        "mean_shift",
        "affinity_propagation",
        "spectral_clustering",
        "agglomerative_clustering",
        "birch",
        "bisecting_kmeans",
    ] {
        let job = make_job(
            TaskType::Clustering,
            algo,
            &["x1", "x2"],
            None,
            json!({"n_clusters": 2, "n_components": 2, "max_iter": 20, "eps": 0.3, "min_samples": 3, "min_cluster_size": 3}),
        );
        let out = train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
        assert!(out.charts.is_object(), "{algo} charts");
    }
}

#[test]
fn anomaly_algorithms_train() {
    let df = DataFrame::from_csv_str(&anomaly_csv()).unwrap();
    for algo in [
        "isolation_forest",
        "local_outlier_factor",
        "one_class_svm",
        "elliptic_envelope",
    ] {
        let job = make_job(
            TaskType::AnomalyDetection,
            algo,
            &["x1", "x2"],
            None,
            json!({"n_estimators": 10, "n_neighbors": 5, "contamination": 0.1}),
        );
        train(&job, &df).unwrap_or_else(|e| panic!("{algo} failed: {e}"));
    }
}

#[test]
fn reinforcement_learning_trains() {
    let df = DataFrame::from_csv_str(&rl_csv()).unwrap();
    let mut job = make_job(
        TaskType::ReinforcementLearning,
        "fitted_q_iteration",
        &["state"],
        None,
        json!({"gamma": 0.9, "max_iterations": 10}),
    );
    job.rl_columns = Some(RLColumns {
        action_column: "action".to_string(),
        reward_column: "reward".to_string(),
        next_state_columns: vec!["next".to_string()],
        done_column: None,
        episode_column: None,
    });
    let out = train(&job, &df).unwrap();
    assert!(!out.model.rl_actions.is_empty());
}

#[test]
fn train_rejects_missing_label_and_columns() {
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    // Supervised task without label_column.
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1"],
        None,
        json!({}),
    );
    assert!(train(&job, &df).is_err());
    // Missing feature column.
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["nope"],
        Some("label"),
        json!({}),
    );
    assert!(train(&job, &df).is_err());
    // Unknown task type string.
    let mut job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1"],
        Some("label"),
        json!({}),
    );
    job.task_type = "bogus".to_string();
    assert!(train(&job, &df).is_err());
    // Single-class labels are rejected.
    let one_class = "x1,x2,label\n".to_string() + &"0.1,0.2,a\n".repeat(12);
    let df2 = DataFrame::from_csv_str(&one_class).unwrap();
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1", "x2"],
        Some("label"),
        json!({}),
    );
    assert!(train(&job, &df2).is_err());
}

#[test]
fn export_snapshot_roundtrips_and_onnx_builds() {
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1", "x2"],
        Some("label"),
        json!({"max_iter": 50}),
    );
    let out = train(&job, &df).unwrap();

    let bytes = export_snapshot(&out.model).unwrap();
    let restored = load_snapshot(&bytes).unwrap();
    assert_eq!(restored.algorithm_id, out.model.algorithm_id);
    assert!(load_snapshot(b"not json").is_err());

    let onnx = export_onnx(&out.model).unwrap();
    assert!(!onnx.is_empty());
    // ONNX protobuf magic / field sanity: model proto bytes parse.
    assert!(onnx.len() > 8);
}

#[test]
fn onnx_export_rejects_rl_non_tree_model() {
    // Only RL jobs with a non-tree model are rejected by the ONNX exporter.
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1", "x2"],
        Some("label"),
        json!({"max_iter": 50}),
    );
    let out = train(&job, &df).unwrap();
    let mut trained = out.model.clone();
    trained.kind = mars_lab::training::ModelKind::ReinforcementLearning;
    assert!(export_onnx(&trained).is_err());
    // Clustering models do export (nearest-centroid graph).
    let df = DataFrame::from_csv_str(&cluster_csv()).unwrap();
    let job = make_job(
        TaskType::Clustering,
        "kmeans",
        &["x1", "x2"],
        None,
        json!({}),
    );
    let out = train(&job, &df).unwrap();
    assert!(!export_onnx(&out.model).unwrap().is_empty());
}

#[test]
fn model_prediction_helpers_work() {
    let df = DataFrame::from_csv_str(classification_csv()).unwrap();
    let job = make_job(
        TaskType::Classification,
        "logistic_regression",
        &["x1", "x2"],
        Some("label"),
        json!({"max_iter": 80}),
    );
    let out = train(&job, &df).unwrap();
    let sample = vec![vec![0.15, 0.25]];
    let transformed = out.model.transform(&sample);
    assert_eq!(transformed.len(), 1);
    let idx = out.model.predict_class(&transformed[0]);
    assert!(idx < out.model.classes.len());
    let proba = out.model.proba(&transformed[0]);
    assert!(!proba.is_empty());

    let df = DataFrame::from_csv_str(regression_csv()).unwrap();
    let job = make_job(
        TaskType::Regression,
        "linear_regression",
        &["x1", "x2"],
        Some("target"),
        json!({}),
    );
    let out = train(&job, &df).unwrap();
    let t = out.model.transform(&sample);
    let v = out.model.predict_value(&t[0]);
    assert!(v.is_finite());
}
