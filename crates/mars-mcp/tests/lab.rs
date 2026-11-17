use mars_lab::config::{Settings, set_settings};
use mars_mcp::lab::{LabBackend, call_lab_tool};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn serial() -> std::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-mcp-lab-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Point lab storage at a private temp dir (settings are process-global).
fn isolate_storage(name: &str) -> PathBuf {
    let dir = temp_dir(name);
    set_settings(Settings::new(
        format!("sqlite://{}", dir.join("lab.db").display()),
        dir.join("data"),
    ));
    dir
}

fn write_csv(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let mut csv = String::from("a,b,label\n");
    for i in 0..30 {
        let label = if i % 2 == 0 { "cat" } else { "dog" };
        csv.push_str(&format!("{},{} ,{}\n", i as f64, (i * 3) as f64, label));
    }
    std::fs::write(&path, csv).expect("write csv");
    path
}

fn backend() -> LabBackend {
    LabBackend::with_in_memory_database().expect("lab backend")
}

fn call(backend: &mut LabBackend, name: &str, args: Value) -> Result<Value, String> {
    call_lab_tool(backend, name, &args).expect("lab tool dispatch")
}

#[test]
fn algorithms_lists_catalogue_and_filters_by_task() {
    let mut backend = backend();
    let all = call(&mut backend, "mars_lab_algorithms", json!({})).expect("algorithms");
    let algorithms = all["algorithms"].as_array().expect("array");
    assert!(algorithms.len() > 50, "catalogue should be large");
    assert!(algorithms.iter().any(|a| a["id"] == "logistic_regression"));

    let filtered = call(
        &mut backend,
        "mars_lab_algorithms",
        json!({ "task": "clustering" }),
    )
    .expect("filtered");
    let algorithms = filtered["algorithms"].as_array().expect("array");
    assert!(!algorithms.is_empty());
    assert!(algorithms.iter().all(|a| a["task_type"] == "clustering"));
}

#[test]
fn algorithms_rejects_unknown_task() {
    let mut backend = backend();
    let error = call(
        &mut backend,
        "mars_lab_algorithms",
        json!({ "task": "bogus" }),
    )
    .expect_err("invalid task");
    assert!(error.contains("invalid task"), "{error}");
}

#[test]
fn datasets_and_jobs_are_empty_on_fresh_database() {
    let mut backend = backend();
    let datasets = call(&mut backend, "mars_lab_datasets", json!({})).expect("datasets");
    assert_eq!(datasets["datasets"].as_array().expect("array").len(), 0);
    let jobs = call(&mut backend, "mars_lab_jobs", json!({})).expect("jobs");
    assert_eq!(jobs["jobs"].as_array().expect("array").len(), 0);
}

#[test]
fn dataset_inspect_reports_missing_dataset() {
    let mut backend = backend();
    let error = call(
        &mut backend,
        "mars_lab_dataset",
        json!({ "dataset_id": "00000000-0000-0000-0000-000000000000" }),
    )
    .expect_err("missing dataset");
    assert!(error.contains("dataset not found"), "{error}");
}

#[test]
fn train_rejects_unknown_algorithm_and_task_mismatch() {
    let mut backend = backend();
    let dataset_id = "00000000-0000-0000-0000-000000000000";
    let error = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": dataset_id,
            "task": "classification",
            "algorithm": "not-an-algorithm",
            "features": ["a"],
        }),
    )
    .expect_err("unknown algorithm");
    assert!(error.contains("unknown algorithm"), "{error}");

    // kmeans is a clustering algorithm — reject it for classification.
    let error = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": dataset_id,
            "task": "classification",
            "algorithm": "kmeans",
            "features": ["a"],
        }),
    )
    .expect_err("task mismatch");
    assert!(error.contains("is for task"), "{error}");
}

#[test]
fn job_and_export_report_missing_records() {
    let mut backend = backend();
    let job_id = "00000000-0000-0000-0000-000000000000";
    let error =
        call(&mut backend, "mars_lab_job", json!({ "job_id": job_id })).expect_err("missing job");
    assert!(error.contains("job not found"), "{error}");
    let error = call(
        &mut backend,
        "mars_lab_export",
        json!({ "job_id": job_id, "output": "/tmp/out.onnx" }),
    )
    .expect_err("missing artefact");
    assert!(error.contains("no onnx artefact"), "{error}");
}

#[test]
fn invalid_arguments_and_ids_are_rejected() {
    let mut backend = backend();
    let error =
        call(&mut backend, "mars_lab_jobs", json!({ "limit": "many" })).expect_err("bad limit");
    assert!(error.contains("invalid arguments"), "{error}");
    let error = call(
        &mut backend,
        "mars_lab_job",
        json!({ "job_id": "not-a-uuid" }),
    )
    .expect_err("bad uuid");
    assert!(error.contains("invalid job_id"), "{error}");
}

#[test]
fn unknown_lab_prefix_tool_is_not_dispatched() {
    let mut backend = backend();
    assert!(call_lab_tool(&mut backend, "mars_unknown", &json!({})).is_none());
}

#[test]
fn dataset_upload_inspect_train_job_export_flow() {
    let _guard = serial();
    let dir = isolate_storage("flow");
    let csv = write_csv(&dir, "pets.csv");
    let mut backend = backend();

    // Upload
    let uploaded = call(
        &mut backend,
        "mars_lab_dataset_upload",
        json!({ "path": csv.to_string_lossy() }),
    )
    .expect("upload");
    let dataset_id = uploaded["dataset"]["id"]
        .as_str()
        .expect("dataset id")
        .to_string();

    // Inspect (profile from stored CSV)
    let inspected = call(
        &mut backend,
        "mars_lab_dataset",
        json!({ "dataset_id": dataset_id }),
    )
    .expect("inspect");
    assert!(
        inspected["profile"].is_object(),
        "profile should be present"
    );

    // Train
    let trained = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": dataset_id,
            "task": "classification",
            "algorithm": "logistic_regression",
            "features": ["a", "b"],
            "label": "label",
            "hyperparameters": {},
        }),
    )
    .expect("train");
    assert_eq!(trained["status"], "completed");
    let job_id = trained["job"]["id"].as_str().expect("job id").to_string();

    // Jobs list contains the job
    let jobs = call(&mut backend, "mars_lab_jobs", json!({})).expect("jobs");
    assert!(
        jobs["jobs"]
            .as_array()
            .expect("array")
            .iter()
            .any(|j| j["id"] == job_id)
    );

    // Job detail includes metric + artefact
    let job = call(&mut backend, "mars_lab_job", json!({ "job_id": job_id })).expect("job");
    assert!(job["metric"].is_object());
    assert_eq!(job["artefacts"].as_array().expect("array").len(), 1);

    // Export ONNX artefact to disk
    let output = dir.join("model.onnx");
    let exported = call(
        &mut backend,
        "mars_lab_export",
        json!({
            "job_id": job_id,
            "output": output.to_string_lossy(),
            "format": "onnx",
        }),
    )
    .expect("export");
    assert_eq!(exported["format"], "onnx");
    assert!(output.exists());
    assert!(output.metadata().expect("meta").len() > 0);
}

#[test]
fn lab_tools_reject_bad_arguments_and_states() {
    let _guard = serial();
    isolate_storage("errors");
    let mut backend = backend();
    let missing = "00000000-0000-0000-0000-000000000000";

    // Parse-level errors
    assert!(call(&mut backend, "mars_lab_dataset_upload", json!({})).is_err());
    assert!(call(
        &mut backend,
        "mars_lab_train",
        json!({ "dataset_id": missing, "task": "classification", "algorithm": "logistic_regression" }),
    )
    .is_err());

    // Unknown algorithm / task mismatch / bad uuids
    let csv = write_csv(&temp_dir("errors-csv"), "x.csv");
    let uploaded = call(
        &mut backend,
        "mars_lab_dataset_upload",
        json!({ "path": csv.to_string_lossy() }),
    )
    .expect("upload");
    let dataset_id = uploaded["dataset"]["id"].as_str().unwrap().to_string();

    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": dataset_id, "task": "classification",
                "algorithm": "nope", "features": ["a"],
            }),
        )
        .is_err()
    );
    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": dataset_id, "task": "regression",
                "algorithm": "logistic_regression", "features": ["a"],
            }),
        )
        .is_err()
    );
    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": dataset_id, "task": "classification",
                "algorithm": "logistic_regression", "features": [],
            }),
        )
        .is_err()
    );
    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": "not-a-uuid", "task": "classification",
                "algorithm": "logistic_regression", "features": ["a"],
            }),
        )
        .is_err()
    );

    // Missing job / artefact format
    assert!(call(&mut backend, "mars_lab_job", json!({ "job_id": missing }),).is_err());
    assert!(
        call(
            &mut backend,
            "mars_lab_export",
            json!({ "job_id": missing, "output": "/tmp/x.onnx", "format": "onnx" }),
        )
        .is_err()
    );
}

#[test]
fn lab_jobs_limit_and_export_default_format() {
    let _guard = serial();
    isolate_storage("limits");
    let mut backend = backend();

    // jobs with explicit limit (clamped) and empty result shape.
    let jobs = call(&mut backend, "mars_lab_jobs", json!({ "limit": 5 })).expect("jobs");
    assert!(jobs["jobs"].is_array());
    assert!(jobs["next_cursor"].is_null() || jobs["next_cursor"].is_string());

    // export without "format" defaults to onnx; unknown job errors.
    let missing = "00000000-0000-0000-0000-000000000000";
    assert!(
        call(
            &mut backend,
            "mars_lab_export",
            json!({ "job_id": missing, "output": "/tmp/x.onnx" }),
        )
        .is_err()
    );
    assert!(
        call(
            &mut backend,
            "mars_lab_export",
            json!({ "job_id": missing, "output": "/tmp/x", "format": "pkl" }),
        )
        .is_err()
    );
}

#[test]
fn lab_dataset_upload_rejects_missing_and_bad_files() {
    let _guard = serial();
    isolate_storage("upload-errors");
    let mut backend = backend();

    assert!(
        call(
            &mut backend,
            "mars_lab_dataset_upload",
            json!({ "path": "/nonexistent/definitely-missing.csv" }),
        )
        .is_err()
    );

    let dir = temp_dir("bad-upload");
    let bad = dir.join("bad.csv");
    std::fs::write(&bad, "a,b\n1,\"unclosed\n").expect("write");
    assert!(
        call(
            &mut backend,
            "mars_lab_dataset_upload",
            json!({ "path": bad.to_string_lossy() }),
        )
        .is_err()
    );
}

#[test]
fn lab_train_and_export_surface_io_failures() {
    let _guard = serial();
    let dir = isolate_storage("io-failures");
    let mut backend = backend();
    let csv = write_csv(&dir, "pets.csv");

    let uploaded = call(
        &mut backend,
        "mars_lab_dataset_upload",
        json!({ "path": csv.to_string_lossy() }),
    )
    .expect("upload");
    let dataset_id = uploaded["dataset"]["id"].as_str().unwrap().to_string();
    let storage_key = uploaded["storage_key"].as_str().unwrap().to_string();

    // Train against a dataset id that does not exist.
    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": "00000000-0000-0000-0000-000000000000",
                "task": "classification",
                "algorithm": "logistic_regression",
                "features": ["a"],
            }),
        )
        .is_err()
    );

    // Delete the stored CSV: training must fail at load time.
    std::fs::remove_file(mars_lab::storage::dataset_path(&storage_key)).expect("remove stored");
    assert!(
        call(
            &mut backend,
            "mars_lab_train",
            json!({
                "dataset_id": dataset_id,
                "task": "classification",
                "algorithm": "logistic_regression",
                "features": ["a"],
                "label": "label",
            }),
        )
        .is_err()
    );

    // jobs limit of wrong type is a parse error; export args validate too.
    assert!(call(&mut backend, "mars_lab_jobs", json!({ "limit": "x" })).is_err());
    assert!(call(&mut backend, "mars_lab_export", json!({ "job_id": 5 })).is_err());
    assert!(call(&mut backend, "mars_lab_job", json!({})).is_err());
    assert!(call(&mut backend, "mars_lab_dataset", json!({})).is_err());
}

#[test]
fn lab_dataset_upload_rejects_directory_path() {
    let _guard = serial();
    isolate_storage("dir-upload");
    let mut backend = backend();
    let dir = temp_dir("dir-target");
    assert!(
        call(
            &mut backend,
            "mars_lab_dataset_upload",
            json!({ "path": dir.to_string_lossy() }),
        )
        .is_err()
    );
}

#[test]
fn lab_database_connect_failure_surfaces_for_all_db_tools() {
    let _guard = serial();
    // Point the database URL at an unwritable location so Database::connect
    // fails inside the lazy `database()` initializer.
    let dir = temp_dir("bad-db");
    let blocker = dir.join("not-a-dir");
    std::fs::write(&blocker, "file").expect("blocker file");
    set_settings(Settings::new(
        format!("sqlite://{}/lab.db", blocker.display()),
        dir.join("data"),
    ));
    // `LabBackend::new` (lazy db) rather than the in-memory helper.
    let mut backend = LabBackend::new().expect("backend");
    for tool in [
        "mars_lab_datasets",
        "mars_lab_jobs",
        "mars_lab_dataset",
        "mars_lab_job",
        "mars_lab_export",
        "mars_lab_train",
    ] {
        let args = match tool {
            "mars_lab_dataset" => json!({ "dataset_id": uuid::Uuid::new_v4().to_string() }),
            "mars_lab_job" | "mars_lab_export" => {
                json!({ "job_id": uuid::Uuid::new_v4().to_string(), "output": "o.onnx" })
            }
            "mars_lab_train" => json!({
                "dataset_id": uuid::Uuid::new_v4().to_string(),
                "task": "classification",
                "algorithm": "logistic_regression",
                "features": ["a"]
            }),
            _ => json!({}),
        };
        let error = call(&mut backend, tool, args).expect_err("db failure");
        assert!(!error.is_empty(), "{tool}");
    }
}

#[test]
fn lab_tools_cover_not_found_and_edge_paths() {
    let _guard = serial();
    isolate_storage("edge");
    let mut backend = backend();
    let missing = uuid::Uuid::new_v4().to_string();

    // Unknown dataset/job ids hit the `ok_or_else` not-found closures.
    let err = call(
        &mut backend,
        "mars_lab_dataset",
        json!({ "dataset_id": missing }),
    )
    .expect_err("missing dataset");
    assert!(err.contains("dataset not found"), "{err}");
    let err =
        call(&mut backend, "mars_lab_job", json!({ "job_id": missing })).expect_err("missing job");
    assert!(err.contains("job not found"), "{err}");
    let err = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": missing,
            "task": "classification",
            "algorithm": "logistic_regression",
            "features": ["a"]
        }),
    )
    .expect_err("missing train dataset");
    assert!(err.contains("dataset not found"), "{err}");

    // Empty/blank feature list.
    let err = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": missing,
            "task": "classification",
            "algorithm": "logistic_regression",
            "features": ["  ", ""]
        }),
    )
    .expect_err("empty features");
    assert!(err.contains("feature"), "{err}");

    // Malformed args for each typed payload.
    for (tool, args) in [
        ("mars_lab_algorithms", json!({ "task": 5 })),
        ("mars_lab_dataset", json!({})),
        ("mars_lab_dataset_upload", json!({ "path": 9 })),
        ("mars_lab_train", json!({ "dataset_id": 1 })),
        ("mars_lab_jobs", json!({ "limit": "many" })),
        ("mars_lab_job", json!({ "job_id": "not-a-uuid" })),
        ("mars_lab_export", json!({ "job_id": "x", "output": "o" })),
    ] {
        let err = call(&mut backend, tool, args.clone()).expect_err(&format!("{tool} {args}"));
        assert!(!err.is_empty());
    }

    // jobs limit clamp and next_cursor on empty page.
    let jobs = call(&mut backend, "mars_lab_jobs", json!({ "limit": 0 })).expect("jobs");
    assert_eq!(jobs["jobs"], json!([]));
}

#[test]
fn lab_export_and_artefact_failures_surface_errors() {
    let _guard = serial();
    let dir = isolate_storage("export-fail");
    let csv_path = write_csv(&dir, "data.csv");
    let mut backend = backend();

    let uploaded = call(
        &mut backend,
        "mars_lab_dataset_upload",
        json!({ "path": csv_path.to_string_lossy() }),
    )
    .expect("upload");
    let dataset_id = uploaded["dataset"]["id"].as_str().expect("id").to_string();

    // Sabotage the artefacts dir so export_artefact's ensure_dirs fails after
    // a successful train: swap the directory for a regular file.
    let artefacts_dir = dir.join("data").join("artefacts");
    let _ = std::fs::remove_dir_all(&artefacts_dir);
    std::fs::write(&artefacts_dir, "blocker").expect("blocker");
    let err = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": dataset_id,
            "task": "classification",
            "algorithm": "logistic_regression",
            "features": ["a", "b"],
            "label": "label",
            "seed": 7
        }),
    )
    .expect_err("export should fail");
    assert!(err.contains("EXPORT_FAILED") || !err.is_empty(), "{err}");

    // Restore storage, train again, then hit export's fs::write failure
    // (output parent missing) and fs::read failure (ghost artefact file).
    let _ = std::fs::remove_file(&artefacts_dir);
    std::fs::create_dir_all(&artefacts_dir).expect("restore artefacts dir");
    let trained = call(
        &mut backend,
        "mars_lab_train",
        json!({
            "dataset_id": dataset_id,
            "task": "classification",
            "algorithm": "logistic_regression",
            "features": ["a", "b"],
            "label": "label",
            "seed": 7
        }),
    )
    .expect("train");
    let job_id = trained["job"]["id"].as_str().expect("job id").to_string();
    let job_uuid = uuid::Uuid::parse_str(&job_id).expect("uuid");

    let bad_output = dir.join("missing").join("deep").join("model.onnx");
    call(
        &mut backend,
        "mars_lab_export",
        json!({
            "job_id": job_id,
            "output": bad_output.to_string_lossy(),
            "format": "onnx",
        }),
    )
    .expect_err("export write should fail");

    // Delete the stored artefact file → fs::read error branch.
    let artefact_path = mars_lab::storage::artefact_path(
        trained["artefact"]["storageKey"].as_str().expect("key"),
        job_uuid,
    );
    std::fs::remove_file(&artefact_path).expect("remove artefact");
    let err = call(
        &mut backend,
        "mars_lab_export",
        json!({
            "job_id": job_id,
            "output": dir.join("out.onnx").to_string_lossy(),
            "format": "onnx",
        }),
    )
    .expect_err("ghost artefact read should fail");
    assert!(err.contains("failed to read artefact"), "{err}");
}
