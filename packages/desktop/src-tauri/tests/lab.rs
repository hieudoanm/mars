use mars_tauri_lib::lab::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::OnceLock;

fn lab_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("mars-tauri-lab-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("creates lab temp dir");
        unsafe {
            std::env::set_var("MARS_LAB_DATA_DIR", &dir);
            std::env::set_var("DATABASE_URL", format!("sqlite://{}/lab.db", dir.display()));
        }
        dir
    })
}

fn write_dataset_csv() -> String {
    // Unique content per call: datasets are deduplicated by content hash.
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nonce = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = lab_dir().join(format!("train-{nonce}.csv"));
    std::fs::write(
        &path,
        format!(
            "age,income,savings,target\n\
             25,30000,5000,1\n\
             30,45000,12000,1\n\
             35,60000,25000,1\n\
             40,80000,40000,1\n\
             45,100000,60000,1\n\
             50,120000,80000,1\n\
             55,140000,100000,1\n\
             60,160000,{}\n",
            120000 + nonce
        ),
    )
    .expect("writes csv");
    path.to_string_lossy().into_owned()
}

#[test]
fn request_payloads_deserialize() {
    let request: LabTrainRequest = serde_json::from_value(json!({
        "datasetId": uuid::Uuid::new_v4().to_string(),
        "taskType": "regression",
        "algorithmId": "linear_regression",
        "featureColumns": ["age"],
        "labelColumn": "target",
        "hyperparameters": {},
        "splitRatio": 0.8,
        "randomSeed": 1,
        "name": null
    }))
    .expect("deserializes train request");
    assert_eq!(request.algorithm_id, "linear_regression");

    let filters: LabJobFilters = serde_json::from_value(json!({
        "statuses": ["completed"],
        "taskTypes": [],
        "algorithmIds": [],
        "q": "test",
        "sortField": "status",
        "sortDirection": "asc"
    }))
    .expect("deserializes job filters");
    assert_eq!(filters.sort_direction.as_deref(), Some("asc"));
}

#[test]
fn payloads_serialize() {
    // Every command payload must round-trip through serde for the Tauri
    // boundary; exercise the Serialize impls directly.
    let algorithm = LabAlgorithm {
        id: "a".to_owned(),
        name: "n".to_owned(),
        task_type: "regression".to_owned(),
        family: "f".to_owned(),
        description: "d".to_owned(),
        default_hyperparameters: json!({}),
        exports: vec!["onnx".to_owned()],
    };
    assert!(serde_json::to_value(&algorithm).is_ok());
    let _ = format!("{algorithm:?}");
    assert!(serde_json::to_value(LabJobFilters::default()).is_ok());
    let _ = format!("{:?}", LabJobFilters::default());
    let page = LabJobPage {
        items: vec![],
        next_cursor: None,
    };
    assert!(serde_json::to_value(&page).is_ok());
    let _ = format!("{page:?}");
    let preview = LabDatasetPreview {
        columns: vec![],
        rows: vec![],
    };
    assert!(serde_json::to_value(&preview).is_ok());
    let _ = format!("{preview:?}");
    let request = train_request(uuid::Uuid::new_v4().to_string());
    assert!(serde_json::to_value(&request).is_ok());
    let _ = format!("{request:?}");
}

#[tokio::test]
async fn algorithms_listing_and_filtering() {
    lab_dir();
    let all = lab_algorithms(None).await.expect("lists algorithms");
    assert!(all.len() > 50);
    let classification = lab_algorithms(Some("classification".to_owned()))
        .await
        .expect("filters algorithms");
    assert!(!classification.is_empty());
    assert!(
        classification
            .iter()
            .all(|a| a.task_type == "classification")
    );
    assert!(lab_algorithms(Some("bogus".to_owned())).await.is_err());
}

#[tokio::test]
async fn dataset_upload_list_insights_and_preview() {
    lab_dir();
    let csv = write_dataset_csv();
    let detail = lab_upload_dataset(csv.clone())
        .await
        .expect("uploads dataset");
    assert_eq!(detail.dataset.row_count, 8);
    assert!(!detail.dataset.id.is_nil());
    let dataset_id = detail.dataset.id.to_string();
    assert!(serde_json::to_value(&detail).is_ok());
    let _ = format!("{detail:?}");

    let datasets = lab_list_datasets().await.expect("lists datasets");
    assert!(datasets.iter().any(|d| d.id.to_string() == dataset_id));

    let insight = lab_dataset_insights(dataset_id.clone())
        .await
        .expect("gets insight")
        .expect("insight exists");
    assert_eq!(insight.row_count, 8);

    let preview = lab_dataset_preview(dataset_id.clone(), None, Some(3))
        .await
        .expect("previews dataset");
    assert_eq!(preview.columns.len(), 4);
    assert_eq!(preview.rows.len(), 3);

    let table = read_table_preview(csv, None).expect("previews table file");
    assert_eq!(table.rows.len(), 8);
    assert!(read_table_preview("/nonexistent/data.csv".to_owned(), None).is_err());

    assert!(lab_dataset_insights("not-a-uuid".to_owned()).await.is_err());
    assert!(
        lab_dataset_preview("not-a-uuid".to_owned(), None, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn job_listing_pagination_and_detail_errors() {
    lab_dir();
    // Other tests in this binary may have created jobs — assert shape only.
    let jobs = lab_list_jobs().await.expect("lists jobs");
    assert!(jobs.len() <= 200);

    let page = lab_list_jobs_page(LabJobFilters::default(), None, Some(10))
        .await
        .expect("pages jobs");
    assert!(page.items.len() <= 10);

    assert!(lab_get_job("not-a-uuid".to_owned()).await.is_err());
    let missing = lab_get_job(uuid::Uuid::new_v4().to_string())
        .await
        .expect("gets job");
    assert!(missing.is_none());

    // Invalid cursor and explicit sort fields exercise the filter/cursor paths.
    let filters = LabJobFilters {
        statuses: vec!["completed".to_owned()],
        task_types: vec![],
        algorithm_ids: vec![],
        q: Some("  ".to_owned()),
        sort_field: Some("status".to_owned()),
        sort_direction: Some("asc".to_owned()),
    };
    lab_list_jobs_page(filters, None, Some(5))
        .await
        .expect("pages with filters");
    assert!(
        lab_list_jobs_page(
            LabJobFilters::default(),
            Some("!!!bad-cursor".to_owned()),
            None
        )
        .await
        .is_err()
    );

    // Page through with limit=1, feeding next_cursor back in — exercises the
    // cursor decode success path.
    let mut cursor: Option<String> = None;
    for _ in 0..5 {
        let page = lab_list_jobs_page(LabJobFilters::default(), cursor, Some(1))
            .await
            .expect("pages jobs");
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
}

#[tokio::test]
async fn export_artefact_errors_without_job() {
    lab_dir();
    assert!(
        lab_export_artefact(
            "not-a-uuid".to_owned(),
            "onnx".to_owned(),
            "/tmp/x".to_owned()
        )
        .await
        .is_err()
    );
    let missing = uuid::Uuid::new_v4().to_string();
    assert!(
        lab_export_artefact(missing, "onnx".to_owned(), "/tmp/x".to_owned())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn train_runs_linear_regression_and_exports_onnx() {
    lab_dir();
    let csv = write_dataset_csv();
    let detail = lab_upload_dataset(csv).await.expect("uploads dataset");
    let dataset_id = detail.dataset.id.to_string();

    let request = LabTrainRequest {
        dataset_id,
        task_type: "regression".to_owned(),
        algorithm_id: "linear_regression".to_owned(),
        feature_columns: vec!["age".to_owned(), "income".to_owned()],
        label_column: Some("target".to_owned()),
        rl_columns: None,
        hyperparameters: json!({}),
        split_ratio: 0.8,
        random_seed: 42,
        name: Some("coverage-test".to_owned()),
    };
    let trained = lab_train_inner(request, |_, _| {})
        .await
        .expect("trains model");
    assert_eq!(trained.job.status, "completed");
    assert!(serde_json::to_value(&trained).is_ok());
    let _ = format!("{trained:?}");
    assert!(trained.metrics.is_some());
    assert!(trained.artefacts.iter().any(|a| a.format == "onnx"));

    let job_id = trained.job.id.to_string();
    let detail = lab_get_job(job_id.clone())
        .await
        .expect("gets job")
        .expect("job exists");
    assert_eq!(detail.job.id.to_string(), job_id);

    let out = lab_dir().join("model.onnx");
    lab_export_artefact(
        job_id,
        "onnx".to_owned(),
        out.to_string_lossy().into_owned(),
    )
    .await
    .expect("exports artefact");
    assert!(out.is_file());

    // Unwritable output path surfaces the write error.
    let trained_job = trained.job.id.to_string();
    assert!(
        lab_export_artefact(
            trained_job.clone(),
            "onnx".to_owned(),
            "/nonexistent-dir-xyz/model.onnx".to_owned(),
        )
        .await
        .is_err()
    );
    // A format the job does not have hits the "no artefact" branch.
    assert!(
        lab_export_artefact(
            trained_job.clone(),
            "pkl".to_owned(),
            "/tmp/x.pkl".to_owned()
        )
        .await
        .is_err()
    );

    // Deleting the artefact file surfaces the read error on export.
    let onnx = trained
        .artefacts
        .iter()
        .find(|a| a.format == "onnx")
        .expect("onnx artefact recorded");
    let artefact_file = mars_lab::storage::artefact_path(&onnx.storage_key, trained.job.id);
    std::fs::remove_file(&artefact_file).expect("removes artefact file");
    assert!(
        lab_export_artefact(trained_job, "onnx".to_owned(), "/tmp/x.onnx".to_owned())
            .await
            .is_err()
    );

    assert!(
        lab_train_inner(
            LabTrainRequest {
                dataset_id: uuid::Uuid::new_v4().to_string(),
                task_type: "regression".to_owned(),
                algorithm_id: "linear_regression".to_owned(),
                feature_columns: vec!["age".to_owned()],
                label_column: Some("target".to_owned()),
                rl_columns: None,
                hyperparameters: json!({}),
                split_ratio: 0.8,
                random_seed: 1,
                name: None,
            },
            |_, _| {},
        )
        .await
        .is_err()
    );
}

fn train_request(dataset_id: String) -> LabTrainRequest {
    LabTrainRequest {
        dataset_id,
        task_type: "regression".to_owned(),
        algorithm_id: "linear_regression".to_owned(),
        feature_columns: vec!["age".to_owned(), "income".to_owned()],
        label_column: Some("target".to_owned()),
        rl_columns: None,
        hyperparameters: json!({}),
        split_ratio: 0.8,
        random_seed: 7,
        name: None,
    }
}

#[tokio::test]
async fn train_validation_errors() {
    lab_dir();
    let csv = write_dataset_csv();
    let detail = lab_upload_dataset(csv).await.expect("uploads dataset");
    let dataset_id = detail.dataset.id.to_string();

    // unknown algorithm
    let mut req = train_request(dataset_id.clone());
    req.algorithm_id = "no_such_algorithm".to_owned();
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // task/algorithm mismatch: classifier for a regression task
    let mut req = train_request(dataset_id.clone());
    req.algorithm_id = "random_forest_classifier".to_owned();
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // empty features
    let mut req = train_request(dataset_id.clone());
    req.feature_columns = vec![];
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // missing dataset
    let req = train_request(uuid::Uuid::new_v4().to_string());
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // invalid task type string
    let mut req = train_request(dataset_id.clone());
    req.task_type = "bogus".to_owned();
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // invalid dataset uuid
    let req = train_request("not-a-uuid".to_owned());
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());

    // column missing from the dataframe -> train_job failure path
    let mut req = train_request(dataset_id);
    req.feature_columns = vec!["no_such_column".to_owned()];
    assert!(lab_train_inner(req, |_, _| {}).await.is_err());
}

#[tokio::test]
async fn dataset_error_paths() {
    lab_dir();
    assert!(
        lab_upload_dataset("/nonexistent/file.csv".to_owned())
            .await
            .is_err()
    );

    let bad = lab_dir().join("bad.csv");
    std::fs::write(&bad, "not,a,valid\ncsv,,,").expect("writes bad csv");
    assert!(
        lab_upload_dataset(bad.to_string_lossy().into_owned())
            .await
            .is_err()
    );

    let missing = uuid::Uuid::new_v4().to_string();
    assert!(
        lab_dataset_preview(missing.clone(), None, None)
            .await
            .is_err()
    );
    assert!(
        lab_dataset_insights(missing)
            .await
            .expect("insights")
            .is_none()
    );

    // Same checks for a UUID that parses but has no dataset row — the
    // "dataset not found" branch in preview.
    let ghost = uuid::Uuid::new_v4().to_string();
    assert!(
        lab_dataset_preview(ghost.clone(), None, None)
            .await
            .is_err()
    );
    assert!(
        lab_dataset_insights(ghost)
            .await
            .expect("insights")
            .is_none()
    );
}

#[tokio::test]
async fn dataset_with_text_column_and_missing_storage_file() {
    lab_dir();
    // A non-numeric column exercises the "string" dtype branch in previews.
    let csv = lab_dir().join("mixed-types.csv");
    std::fs::write(
        &csv,
        "name,age,note\nalice,30,hello\nbob,40,world\ncarol,50,\n",
    )
    .expect("writes csv");
    let detail = lab_upload_dataset(csv.to_string_lossy().into_owned())
        .await
        .expect("uploads dataset");
    let dataset_id = detail.dataset.id.to_string();

    let preview = lab_dataset_preview(dataset_id.clone(), None, None)
        .await
        .expect("previews dataset");
    assert!(preview.columns.contains(&"name".to_owned()));

    // Removing the stored file surfaces the dataset-load error in preview
    // and training.
    let storage = mars_lab::storage::dataset_path(&detail.dataset.storage_key);
    std::fs::remove_file(&storage).expect("removes stored dataset");
    assert!(
        lab_dataset_preview(dataset_id.clone(), None, None)
            .await
            .is_err()
    );

    let request = train_request(dataset_id);
    assert!(lab_train_inner(request, |_, _| {}).await.is_err());
}
