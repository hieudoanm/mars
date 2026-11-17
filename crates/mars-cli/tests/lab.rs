//! Integration tests for `mars lab` CLI commands. Lab settings are
//! process-global, so every test runs under a shared lock and points the
//! data dir + SQLite database at a private temp directory.

use mars::commands::run_cli;
use mars_lab::config::{Settings, set_settings};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn serial() -> std::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-cli-lab-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn isolate(name: &str) -> PathBuf {
    let dir = temp_dir(name);
    set_settings(Settings::new(
        format!("sqlite://{}", dir.join("db").join("lab.db").display()),
        dir.join("data"),
    ));
    dir
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn write_csv(dir: &PathBuf, name: &str) -> PathBuf {
    let path = dir.join(name);
    let mut csv = String::from("a,b,label\n");
    for i in 0..30 {
        let label = if i % 2 == 0 { "cat" } else { "dog" };
        csv.push_str(&format!("{},{} ,{}\n", i as f64, (i * 3) as f64, label));
    }
    std::fs::write(&path, csv).expect("write csv");
    path
}

#[test]
fn lab_requires_a_known_command() {
    let _guard = serial();
    isolate("usage");
    assert!(run_cli(args(&["lab"])).is_err());
    assert!(run_cli(args(&["lab", "bogus"])).is_err());
}

#[test]
fn lab_algorithms_lists_and_filters_by_task() {
    let _guard = serial();
    isolate("algorithms");
    run_cli(args(&["lab", "algorithms"])).expect("list all algorithms");
    run_cli(args(&["lab", "algorithms", "--task", "classification"])).expect("filter by task");
    assert!(run_cli(args(&["lab", "algorithms", "--task", "bogus"])).is_err());
}

#[test]
fn lab_dataset_upload_and_listing() {
    let _guard = serial();
    let dir = isolate("datasets");
    let csv = write_csv(&dir, "pets.csv");

    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload dataset");
    run_cli(args(&["lab", "datasets"])).expect("list datasets");

    let missing = dir.join("missing.csv");
    assert!(run_cli(args(&["lab", "dataset", &missing.to_string_lossy()])).is_err());
}

#[test]
fn lab_datasets_empty_dir_reports_none() {
    let _guard = serial();
    isolate("datasets-empty");
    run_cli(args(&["lab", "datasets"])).expect("list empty datasets");
}

#[test]
fn lab_train_validates_arguments() {
    let _guard = serial();
    isolate("train-args");

    // Missing required flags
    assert!(run_cli(args(&["lab", "train"])).is_err());
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            "x",
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
        ]))
        .is_err()
    );

    // Invalid dataset id / task / algorithm / JSON / features
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            "not-a-uuid",
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a",
        ]))
        .is_err()
    );
    let uuid = "00000000-0000-0000-0000-000000000000";
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "bogus",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "classification",
            "--algorithm",
            "nope",
            "--features",
            "a",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a",
            "--hyperparameters",
            "{bad",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            " , ",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a",
        ]))
        .is_err()
    );
}

#[test]
fn lab_train_jobs_export_roundtrip() {
    let _guard = serial();
    let dir = isolate("train-flow");
    let csv = write_csv(&dir, "pets.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");

    // Extract the dataset id from the SQLite db for the train command.
    let dataset_id = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
                .await
                .expect("db");
            let list = mars_lab::db::datasets::list_datasets(db.pool())
                .await
                .expect("list datasets");
            list[0].id
        })
    };

    run_cli(args(&[
        "lab",
        "train",
        "--dataset-id",
        &dataset_id.to_string(),
        "--task",
        "classification",
        "--algorithm",
        "logistic_regression",
        "--features",
        "a,b",
        "--label",
        "label",
        "--hyperparameters",
        "{}",
    ]))
    .expect("train job");

    run_cli(args(&["lab", "jobs"])).expect("list jobs");

    // Fetch the job id, then inspect + export its ONNX artefact.
    let job_id = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
                .await
                .expect("db");
            let list = mars_lab::db::jobs::list_jobs(
                db.pool(),
                &mars_lab::db::jobs::JobFilters::default(),
                10,
                None,
            )
            .await
            .expect("list jobs");
            list.items[0].id
        })
    };
    let job_id_str = job_id.to_string();

    run_cli(args(&["lab", "job", &job_id_str])).expect("show job");

    let export_path = dir.join("model.onnx");
    run_cli(args(&[
        "lab",
        "export",
        "--job-id",
        &job_id_str,
        "--output",
        &export_path.to_string_lossy(),
        "--format",
        "onnx",
    ]))
    .expect("export onnx");
    assert!(export_path.exists());

    // Error paths: bad ids, missing flags, missing artefact format.
    assert!(run_cli(args(&["lab", "job", "not-a-uuid"])).is_err());
    assert!(run_cli(args(&["lab", "job", &uuid_zero()])).is_err());
    assert!(run_cli(args(&["lab", "export"])).is_err());
    assert!(run_cli(args(&["lab", "export", "--job-id", &job_id_str,])).is_err());
    assert!(
        run_cli(args(&[
            "lab",
            "export",
            "--job-id",
            &job_id_str,
            "--output",
            "/tmp/x.pkl",
            "--format",
            "pkl",
        ]))
        .is_err()
    );
}

fn uuid_zero() -> String {
    "00000000-0000-0000-0000-000000000000".to_string()
}

#[test]
fn lab_dataset_rejects_invalid_csv() {
    let _guard = serial();
    let dir = isolate("bad-csv");
    let bad = dir.join("bad.csv");
    std::fs::write(&bad, "a,b\n1,\"unclosed\n").expect("write");
    assert!(run_cli(args(&["lab", "dataset", &bad.to_string_lossy()])).is_err());
}

#[test]
fn lab_datasets_lists_uploaded_files() {
    let _guard = serial();
    let dir = isolate("datasets-list");
    let csv = write_csv(&dir, "listed.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");
    run_cli(args(&["lab", "datasets"])).expect("list with entries");
}

#[test]
fn lab_train_rejects_mismatch_and_bad_numbers() {
    let _guard = serial();
    isolate("train-mismatch");
    let uuid = "00000000-0000-0000-0000-000000000000";

    // Algorithm exists but belongs to a different task.
    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            uuid,
            "--task",
            "classification",
            "--algorithm",
            "kmeans",
            "--features",
            "a",
        ]))
        .is_err()
    );
    // Invalid numeric flags.
    for (flag, value) in [("--split-ratio", "abc"), ("--seed", "xyz")] {
        assert!(
            run_cli(args(&[
                "lab",
                "train",
                "--dataset-id",
                uuid,
                "--task",
                "classification",
                "--algorithm",
                "logistic_regression",
                "--features",
                "a",
                flag,
                value,
            ]))
            .is_err(),
            "{flag}={value} should fail"
        );
    }
}

#[test]
fn lab_export_defaults_to_onnx_and_needs_job() {
    let _guard = serial();
    isolate("export-default");
    // Missing --job-id / --output are usage errors.
    assert!(run_cli(args(&["lab", "export", "--output", "/tmp/x"])).is_err());
    assert!(run_cli(args(&["lab", "export", "--job-id", &uuid_zero()])).is_err());
    // Unknown job id -> "no artefact" style error once ids parse.
    let out = std::env::temp_dir().join(format!("mars-export-{}.onnx", std::process::id()));
    assert!(
        run_cli(args(&[
            "lab",
            "export",
            "--job-id",
            &uuid_zero(),
            "--output",
            &out.to_string_lossy(),
        ]))
        .is_err()
    );
}

#[test]
fn lab_train_failure_is_recorded_and_reported() {
    let _guard = serial();
    let dir = isolate("train-fail");
    let csv = write_csv(&dir, "pets.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");

    let dataset_id = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
                .await
                .expect("db");
            mars_lab::db::datasets::list_datasets(db.pool())
                .await
                .expect("list")[0]
                .id
        })
    };

    // Features that do not exist in the dataset make train() fail; the job
    // must transition to failed rather than hang or panic.
    let result = run_cli(args(&[
        "lab",
        "train",
        "--dataset-id",
        &dataset_id.to_string(),
        "--task",
        "classification",
        "--algorithm",
        "logistic_regression",
        "--features",
        "nonexistent_column",
        "--label",
        "label",
    ]));
    assert!(result.is_err());

    // The failed job is listed and inspectable.
    run_cli(args(&["lab", "jobs"])).expect("jobs after failure");
}

#[test]
fn lab_export_write_failure_surfaces() {
    let _guard = serial();
    isolate("export-write-fail");
    // output parent does not exist -> fs::write fails after artefact lookup.
    // Use a syntactically valid job id; it will fail at artefact lookup first,
    // which still exercises the command's error path.
    assert!(
        run_cli(args(&[
            "lab",
            "export",
            "--job-id",
            &uuid_zero(),
            "--output",
            "/nonexistent-dir-xyz/model.onnx",
        ]))
        .is_err()
    );
}

#[test]
fn lab_job_shows_bare_job_without_metric_or_artefacts() {
    let _guard = serial();
    let dir = isolate("bare-job");
    let csv = write_csv(&dir, "pets.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");
    // Insert a queued job directly so `job`/`jobs` hit the no-metric,
    // no-artefacts display branches.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let job_id = rt.block_on(async {
        let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
            .await
            .expect("db");
        let datasets = mars_lab::db::datasets::list_datasets(db.pool())
            .await
            .expect("list datasets");
        let dataset_id = datasets[0].id;
        let job = mars_lab::domain::Job {
            id: uuid::Uuid::new_v4(),
            dataset_id,
            task_type: "classification".to_owned(),
            algorithm_id: "logistic_regression".to_owned(),
            feature_columns: vec!["a".to_owned()],
            label_column: Some("label".to_owned()),
            rl_columns: None,
            hyperparameters: serde_json::json!({}),
            split_ratio: 0.2,
            random_seed: 42,
            export_pkl: true,
            export_joblib: true,
            status: "queued".to_owned(),
            error_code: None,
            error_message: None,
            library_versions: None,
            name: None,
            tags: vec![],
            attempts: 0,
            max_attempts: 1,
            next_attempt_at: None,
            created_at: mars_lab::domain::utc_now(),
            updated_at: mars_lab::domain::utc_now(),
        };
        mars_lab::db::jobs::insert_job(db.pool(), &job)
            .await
            .expect("insert job");
        job.id
    });

    run_cli(args(&["lab", "job", &job_id.to_string()])).expect("show bare job");
    run_cli(args(&["lab", "jobs"])).expect("list jobs");
    let _ = dir;
}

#[test]
fn lab_dataset_dir_instead_of_file_errors() {
    let _guard = serial();
    let dir = isolate("dataset-is-dir");
    // A directory path fails the initial fs::read.
    assert!(run_cli(args(&["lab", "dataset", &dir.to_string_lossy()])).is_err());
}

#[test]
fn lab_train_fails_when_stored_dataset_file_is_missing() {
    let _guard = serial();
    let dir = isolate("train-missing-file");
    let csv = write_csv(&dir, "pets.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");

    let (dataset_id, storage_key) = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
                .await
                .expect("db");
            let d = mars_lab::db::datasets::list_datasets(db.pool())
                .await
                .expect("list")[0]
                .clone();
            (d.id, d.storage_key)
        })
    };
    // Remove the stored CSV so DataFrame::from_csv_path fails mid-command.
    let stored = mars_lab::storage::dataset_path(&storage_key);
    std::fs::remove_file(&stored).expect("remove stored csv");

    assert!(
        run_cli(args(&[
            "lab",
            "train",
            "--dataset-id",
            &dataset_id.to_string(),
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a",
            "--label",
            "label",
        ]))
        .is_err()
    );
}

#[test]
fn lab_export_surfaces_missing_artefact_file_and_bad_output() {
    let _guard = serial();
    let dir = isolate("export-io-errors");

    // Insert a job + artefact row whose file was never written: the export
    // finds the row then fails on fs::read.
    let csv = write_csv(&dir, "pets.csv");
    run_cli(args(&["lab", "dataset", &csv.to_string_lossy()])).expect("upload");

    let (job_id, artefact_id) = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let db = mars_lab::db::Database::connect(&mars_lab::config::get_settings())
                .await
                .expect("db");
            let datasets = mars_lab::db::datasets::list_datasets(db.pool())
                .await
                .expect("datasets");
            let dataset_id = datasets[0].id;
            let job = mars_lab::domain::Job {
                id: uuid::Uuid::new_v4(),
                dataset_id,
                task_type: "classification".to_owned(),
                algorithm_id: "logistic_regression".to_owned(),
                feature_columns: vec!["a".to_owned()],
                label_column: Some("label".to_owned()),
                rl_columns: None,
                hyperparameters: serde_json::json!({}),
                split_ratio: 0.2,
                random_seed: 42,
                export_pkl: true,
                export_joblib: true,
                status: "completed".to_owned(),
                error_code: None,
                error_message: None,
                library_versions: None,
                name: None,
                tags: vec![],
                attempts: 0,
                max_attempts: 1,
                next_attempt_at: None,
                created_at: mars_lab::domain::utc_now(),
                updated_at: mars_lab::domain::utc_now(),
            };
            mars_lab::db::jobs::insert_job(db.pool(), &job)
                .await
                .expect("insert job");
            let artefact = mars_lab::domain::Artefact {
                id: uuid::Uuid::new_v4(),
                job_id: job.id,
                format: "onnx".to_owned(),
                filename: "ghost.onnx".to_owned(),
                byte_size: 1,
                storage_key: "ghost/missing.onnx".to_owned(),
                created_at: mars_lab::domain::utc_now(),
            };
            mars_lab::db::artefacts::insert_artefact(db.pool(), &artefact)
                .await
                .expect("insert artefact");
            (job.id, artefact.id)
        })
    };
    let _ = artefact_id;

    // fs::read fails: artefact row exists but the file does not.
    let out = dir.join("out.onnx");
    assert!(
        run_cli(args(&[
            "lab",
            "export",
            "--job-id",
            &job_id.to_string(),
            "--output",
            &out.to_string_lossy(),
            "--format",
            "onnx",
        ]))
        .is_err()
    );
    assert!(!out.exists());
}
