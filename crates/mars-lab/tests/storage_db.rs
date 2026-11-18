//! Coverage for `storage` (filesystem layout), `training::validation`
//! (hyperparameter rules), and the `db::jobs::queries` read paths that the
//! unit tests only smoke-tested.

use mars_lab::config::{Settings, reset_settings, set_settings};
use mars_lab::db::jobs::filters::JobFilters;
use mars_lab::db::jobs::queries::{
    decode_cursor, delete_job, get_job, job_exists, jobs_by_ids, list_jobs, queue_positions,
    queued_ids, worker_counts,
};
use mars_lab::db::jobs::writes::insert_job;
use mars_lab::db::{Database, datasets};
use mars_lab::domain::{ColumnPreview, Dataset, Job};
use mars_lab::storage;
use mars_lab::training::validation::validate_hyperparameters;
use serde_json::json;
use uuid::Uuid;

// ── storage ──────────────────────────────────────────────────────────────────

#[test]
fn storage_writes_reads_and_removes_files() {
    let dir = std::env::temp_dir().join(format!("mars-lab-storage-{}", Uuid::new_v4()));
    set_settings(Settings::new("sqlite::memory:", dir.clone()));

    storage::ensure_dirs().unwrap();
    assert!(storage::datasets_dir().starts_with(&dir));
    assert!(storage::artefacts_dir().starts_with(&dir));

    let key = storage::storage_key_for("datasets", Uuid::nil(), ".csv");
    assert!(key.contains(".csv"));

    let bytes = b"a,b\n1,2\n";
    let (id, ds_key, hash, size) = storage::write_dataset(bytes).unwrap();
    assert_eq!(size, bytes.len() as i64);
    assert_eq!(hash, storage::sha256_bytes(bytes));
    assert_eq!(storage::read_dataset(&ds_key).unwrap(), bytes);
    assert!(storage::dataset_path(&ds_key).exists());

    // Missing dataset → friendly 404.
    let err = storage::read_dataset("datasets/nope/nope.csv").unwrap_err();
    assert_eq!(err.code, "DATASET_NOT_FOUND");

    // sha256_of_file on a real file and on a missing one.
    let ds_path = storage::dataset_path(&ds_key);
    assert_eq!(storage::sha256_of_file(&ds_path).unwrap(), hash);
    assert!(storage::sha256_of_file(&dir.join("missing")).is_err());

    // Artefact write/read/remove roundtrip.
    let (ak, apath, asize) = storage::write_artefact(id, b"onnx-bytes", "model.onnx").unwrap();
    assert_eq!(asize, 10);
    assert!(apath.exists());
    assert!(storage::artefact_path(&ak, id).exists());
    assert_eq!(std::fs::read(&apath).unwrap(), b"onnx-bytes");
    storage::remove_job_files(id);
    assert!(!apath.exists());
    // Removing a non-existent job dir is a no-op.
    storage::remove_job_files(Uuid::new_v4());

    let _ = std::fs::remove_dir_all(&dir);
    reset_settings();
}

// ── hyperparameter validation ────────────────────────────────────────────────

#[test]
fn validation_rejects_malformed_input() {
    assert!(validate_hyperparameters("kmeans", &json!("nope"), None).is_err());
    assert!(validate_hyperparameters("kmeans", &json!({"bogus_key": 1}), None).is_err());
    // Unknown algorithm ids skip per-algorithm rules entirely.
    assert!(validate_hyperparameters("not_an_algo", &json!({"anything": 1}), None).is_err());
    assert!(validate_hyperparameters("definitely_unknown", &json!({}), None).is_ok());
}

#[test]
fn validation_self_training_and_clusters() {
    assert!(
        validate_hyperparameters("self_training_classifier", &json!({"threshold": 0.4}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters("self_training_classifier", &json!({"threshold": 1.5}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters("self_training_classifier", &json!({"max_iter": 0}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters(
            "self_training_classifier",
            &json!({"threshold": 0.9, "max_iter": 3}),
            None
        )
        .is_ok()
    );

    assert!(validate_hyperparameters("kmeans", &json!({"n_clusters": 1}), None).is_err());
    assert!(validate_hyperparameters("kmeans", &json!({"n_clusters": 3}), Some(3)).is_err());
    assert!(validate_hyperparameters("kmeans", &json!({"n_clusters": 3}), Some(10)).is_ok());
    assert!(validate_hyperparameters("mini_batch_kmeans", &json!({"n_clusters": 2}), None).is_ok());
    assert!(
        validate_hyperparameters("gaussian_mixture", &json!({"n_components": 1}), None).is_err()
    );
    assert!(
        validate_hyperparameters("gaussian_mixture", &json!({"n_components": 2}), Some(100))
            .is_ok()
    );
}

#[test]
fn validation_isolation_forest() {
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"contamination": "auto"}), None)
            .is_ok()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"contamination": 0.2}), None).is_ok()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"contamination": "bogus"}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"contamination": 0.9}), None).is_err()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"contamination": -0.1}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"n_estimators": 0}), None).is_err()
    );
    assert!(
        validate_hyperparameters("isolation_forest", &json!({"n_estimators": 10}), None).is_ok()
    );
}

#[test]
fn validation_mlp() {
    let base = json!({});
    assert!(validate_hyperparameters("mlp_classifier", &base, None).is_ok());
    assert!(validate_hyperparameters("mlp_regressor", &base, None).is_ok());

    for bad in [
        json!({"hidden_layer_sizes": []}),
        json!({"hidden_layer_sizes": [1, 2, 3, 4, 5, 6]}),
        json!({"hidden_layer_sizes": "wide"}),
        json!({"hidden_layer_sizes": [0]}),
        json!({"hidden_layer_sizes": ["x"]}),
        json!({"activation": "selu"}),
        json!({"solver": "lbfgs2"}),
        json!({"alpha": -1.0}),
        json!({"learning_rate_init": 0.0}),
        json!({"learning_rate_init": 1.5}),
        json!({"max_iter": 0}),
        json!({"batch_size": 0}),
        json!({"batch_size": "big"}),
        json!({"validation_fraction": 0.0}),
        json!({"validation_fraction": 1.0}),
        json!({"early_stopping": "yes"}),
    ] {
        assert!(
            validate_hyperparameters("mlp_classifier", &bad, None).is_err(),
            "expected rejection: {bad}"
        );
    }

    for good in [
        json!({"hidden_layer_sizes": [10, 5], "activation": "tanh", "solver": "lbfgs", "batch_size": "auto", "early_stopping": true, "validation_fraction": 0.2}),
        json!({"batch_size": 32}),
        json!({"activation": "identity"}),
        json!({"solver": "sgd"}),
    ] {
        assert!(
            validate_hyperparameters("mlp_regressor", &good, None).is_ok(),
            "expected acceptance: {good}"
        );
    }
}

#[test]
fn validation_fitted_q_iteration() {
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"n_iterations": 0}), None).is_err()
    );
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"n_iterations": 51}), None).is_err()
    );
    assert!(validate_hyperparameters("fitted_q_iteration", &json!({"gamma": -0.1}), None).is_err());
    assert!(validate_hyperparameters("fitted_q_iteration", &json!({"gamma": 1.1}), None).is_err());
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"max_depth": 0}), None).is_err()
    );
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"max_depth": 21}), None).is_err()
    );
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"learning_rate": 0.0}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters("fitted_q_iteration", &json!({"learning_rate": 1.5}), None)
            .is_err()
    );
    assert!(
        validate_hyperparameters(
            "fitted_q_iteration",
            &json!({"n_iterations": 5, "gamma": 0.95, "max_depth": 4, "learning_rate": 0.2}),
            None
        )
        .is_ok()
    );
}

// ── db job queries ───────────────────────────────────────────────────────────

fn test_dataset() -> Dataset {
    Dataset {
        id: Uuid::new_v4(),
        original_filename: "q.csv".to_string(),
        sha256: "deadbeef".to_string(),
        byte_size: 42,
        storage_key: format!("datasets/{}/x.csv", Uuid::new_v4()),
        columns: vec![ColumnPreview {
            name: "x".to_string(),
            inferred_dtype: "f64".to_string(),
            null_count: 0,
            sample_values: vec![],
        }],
        row_count: 3,
        created_at: chrono::Utc::now(),
    }
}

fn test_job(dataset_id: Uuid, status: &str, task: &str, algo: &str, tags: &[&str]) -> Job {
    Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: task.to_string(),
        algorithm_id: algo.to_string(),
        feature_columns: vec!["x".to_string()],
        label_column: match task {
            "clustering" | "anomaly_detection" | "reinforcement_learning" => None,
            _ => Some("y".to_string()),
        },
        rl_columns: None,
        hyperparameters: json!({}),
        split_ratio: 0.8,
        random_seed: 1,
        export_pkl: false,
        export_joblib: false,
        status: status.to_string(),
        error_code: None,
        error_message: None,
        library_versions: None,
        name: None,
        tags: tags.iter().map(|t| t.to_string()).collect(),
        attempts: 0,
        max_attempts: 3,
        next_attempt_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

async fn seeded_db() -> (Database, Vec<Job>) {
    let db = Database::in_memory().await.unwrap();
    db.ping().await.unwrap();
    let ds = test_dataset();
    datasets::insert_dataset(db.pool(), &ds).await.unwrap();

    let jobs = vec![
        test_job(
            ds.id,
            "queued",
            "classification",
            "logistic_regression",
            &["cv"],
        ),
        test_job(
            ds.id,
            "completed",
            "regression",
            "linear_regression",
            &["reg"],
        ),
        test_job(ds.id, "training", "clustering", "kmeans", &["cv", "unsup"]),
    ];
    for job in &jobs {
        insert_job(db.pool(), job).await.unwrap();
    }
    (db, jobs)
}

#[tokio::test]
async fn list_jobs_filters_and_paginates() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .try_init();
    let (db, jobs) = seeded_db().await;
    let pool = db.pool();

    // Unfiltered list returns everything.
    let page = list_jobs(pool, &JobFilters::default(), 50, None)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 3);

    // Status / task / algorithm / tag / text filters.
    let f = JobFilters {
        statuses: vec!["queued".to_string()],
        ..Default::default()
    };
    assert_eq!(list_jobs(pool, &f, 50, None).await.unwrap().items.len(), 1);

    let f = JobFilters {
        task_types: vec!["regression".to_string()],
        ..Default::default()
    };
    assert_eq!(list_jobs(pool, &f, 50, None).await.unwrap().items.len(), 1);

    let f = JobFilters {
        algorithm_ids: vec!["kmeans".to_string()],
        ..Default::default()
    };
    assert_eq!(list_jobs(pool, &f, 50, None).await.unwrap().items.len(), 1);

    let f = JobFilters {
        tags: vec!["cv".to_string()],
        ..Default::default()
    };
    assert_eq!(list_jobs(pool, &f, 50, None).await.unwrap().items.len(), 2);

    let f = JobFilters {
        q: Some("logistic".to_string()),
        ..Default::default()
    };
    let items = list_jobs(pool, &f, 50, None).await.unwrap().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, jobs[0].id);

    // Combined filters + created range.
    let f = JobFilters {
        statuses: vec!["queued".to_string(), "training".to_string()],
        created_from: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        created_to: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        ..Default::default()
    };
    assert_eq!(list_jobs(pool, &f, 50, None).await.unwrap().items.len(), 2);

    // Sorting variants.
    for (field, dir) in [
        ("createdAt", "asc"),
        ("createdAt", "desc"),
        ("updatedAt", "asc"),
        ("status", "asc"),
        ("algorithmId", "desc"),
    ] {
        let f = JobFilters {
            sort_field: field.to_string(),
            sort_direction: dir.to_string(),
            ..Default::default()
        };
        assert_eq!(
            list_jobs(pool, &f, 50, None).await.unwrap().items.len(),
            3,
            "{field} {dir}"
        );
    }

    // Cursor pagination: limit 2 → next_cursor → second page.
    let page1 = list_jobs(pool, &JobFilters::default(), 2, None)
        .await
        .unwrap();
    assert_eq!(page1.items.len(), 2);
    let cursor = page1.next_cursor.expect("expected next cursor");
    let decoded = decode_cursor(&cursor).unwrap();
    let page2 = list_jobs(
        pool,
        &JobFilters::default(),
        2,
        Some(&mars_lab::db::jobs::filters::JobCursor {
            value: decoded.0,
            id: decoded.1,
        }),
    )
    .await
    .unwrap();
    assert_eq!(page2.items.len(), 1);
    assert!(page2.next_cursor.is_none());
    let all: std::collections::HashSet<_> = page1
        .items
        .iter()
        .chain(page2.items.iter())
        .map(|j| j.id)
        .collect();
    assert_eq!(all.len(), 3);

    // Ascending sort + text cursor path.
    let f = JobFilters {
        sort_direction: "asc".to_string(),
        ..Default::default()
    };
    let p1 = list_jobs(pool, &f, 2, None).await.unwrap();
    if let Some(cursor) = &p1.next_cursor {
        let d = decode_cursor(cursor).unwrap();
        let _ = list_jobs(
            pool,
            &f,
            2,
            Some(&mars_lab::db::jobs::filters::JobCursor {
                value: d.0,
                id: d.1,
            }),
        )
        .await
        .unwrap();
    }

    // Point lookups.
    assert!(get_job(pool, jobs[0].id).await.unwrap().is_some());
    assert!(get_job(pool, Uuid::new_v4()).await.unwrap().is_none());
    assert!(job_exists(pool, jobs[1].id).await.unwrap());
    assert!(!job_exists(pool, Uuid::new_v4()).await.unwrap());
    let by_ids = jobs_by_ids(pool, &[jobs[0].id, jobs[2].id, Uuid::new_v4()])
        .await
        .unwrap();
    assert_eq!(by_ids.len(), 2);
    assert!(jobs_by_ids(pool, &[]).await.unwrap().is_empty());

    // Queue ordering helpers.
    let positions = queue_positions(pool, &[jobs[0].id, jobs[1].id])
        .await
        .unwrap();
    assert_eq!(positions.get(&jobs[0].id), Some(&1));
    assert!(!positions.contains_key(&jobs[1].id)); // not queued
    let qids = queued_ids(pool).await.unwrap();
    assert_eq!(qids, vec![jobs[0].id]);

    // Worker counts group active statuses.
    let counts = worker_counts(pool).await.unwrap();
    assert_eq!(counts.training, 1);
    assert_eq!(counts.validating + counts.evaluating + counts.exporting, 0);

    // Delete.
    assert!(delete_job(pool, jobs[2].id).await.unwrap());
    assert!(!delete_job(pool, jobs[2].id).await.unwrap());
    assert!(!job_exists(pool, jobs[2].id).await.unwrap());
}

#[tokio::test]
async fn database_connect_and_migrate_on_file() {
    let dir = std::env::temp_dir().join(format!("mars-lab-db-{}", Uuid::new_v4()));
    let settings = Settings::new("sqlite::memory:", dir.clone());
    // sqlite_path() only resolves file-backed URLs; connect via in-memory is
    // covered above, so exercise connect() with a file URL instead.
    let file_settings = Settings::new(
        format!("sqlite://{}", dir.join("lab.db").display()),
        dir.clone(),
    );
    let _ = settings;
    let db = Database::connect(&file_settings).await.unwrap();
    db.ping().await.unwrap();
    // Re-connect: schema must already exist (idempotent migrate).
    let db2 = Database::connect(&file_settings).await.unwrap();
    db2.ping().await.unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn convert_error_helpers() {
    // convert::database_error / LabError conversions are exercised implicitly
    // by every query above; hit the invalid-cursor edge directly.
    assert!(decode_cursor("").is_err());
    assert!(decode_cursor("!!!").is_err());
}
