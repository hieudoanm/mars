use mars_mcp::lab::{LabBackend, call_lab_tool};
use serde_json::{Value, json};

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
