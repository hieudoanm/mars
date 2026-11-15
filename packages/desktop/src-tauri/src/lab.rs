//! Mars Lab Tauri commands — CSV datasets, training jobs, metrics, ONNX export.
//!
//! Thin command layer over `mars-lab` (training/ONNX + SQLite persistence). The shared [`Database`] is initialised lazily on first
//! use; migrations run inside `Database::connect`.

use mars_lab::catalogue;
use mars_lab::config::get_settings;
use mars_lab::csv;
use mars_lab::dataframe::DataFrame;
use mars_lab::db::jobs::queries;
use mars_lab::db::{Database, artefacts, datasets, jobs};
use mars_lab::domain::{
    Artefact, ArtefactFormat, ColumnPreview, Dataset, DatasetInsight, Job, JobMetric, JobStatus,
    RLColumns, TaskType, utc_now,
};
use mars_lab::export;
use mars_lab::insights::{self, DatasetProfile};
use mars_lab::storage::{artefact_path, dataset_path, ensure_dirs, write_artefact, write_dataset};
use mars_lab::training::{TrainingOutput, train};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

static DB: tokio::sync::OnceCell<Database> = tokio::sync::OnceCell::const_new();

async fn db() -> Result<&'static Database, String> {
    DB.get_or_try_init(|| async {
        Database::connect(&get_settings())
            .await
            .map_err(|error| error.to_string())
    })
    .await
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

// ── serialisable payloads ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabAlgorithm {
    pub id: String,
    pub name: String,
    pub task_type: String,
    pub family: String,
    pub description: String,
    pub default_hyperparameters: Value,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabDatasetDetail {
    pub dataset: Dataset,
    pub profile: DatasetProfile,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabJobDetail {
    pub job: Job,
    pub metrics: Option<JobMetric>,
    pub artefacts: Vec<Artefact>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabTrainRequest {
    pub dataset_id: String,
    pub task_type: String,
    pub algorithm_id: String,
    pub feature_columns: Vec<String>,
    pub label_column: Option<String>,
    pub rl_columns: Option<RLColumns>,
    pub hyperparameters: Value,
    pub split_ratio: f64,
    pub random_seed: i64,
    pub name: Option<String>,
}

// ── algorithms ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn lab_algorithms(task: Option<String>) -> Result<Vec<LabAlgorithm>, String> {
    let task = task
        .map(|raw| {
            raw.parse::<TaskType>()
                .map_err(|e| format!("invalid task: {e}"))
        })
        .transpose()?;
    Ok(catalogue::for_task(task)
        .into_iter()
        .map(|spec| LabAlgorithm {
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            task_type: spec.task_type.as_str().to_string(),
            family: spec.family.to_string(),
            description: spec.description.to_string(),
            default_hyperparameters: spec.default_hyperparameters.clone(),
            exports: spec
                .exports
                .iter()
                .map(|format| format.as_str().to_string())
                .collect(),
        })
        .collect())
}

// ── datasets ────────────────────────────────────────────────────────────────

fn column_previews(df: &DataFrame) -> Vec<ColumnPreview> {
    df.column_names()
        .iter()
        .map(|name| {
            let values = df.column(name).unwrap_or_default();
            let null_count = values.iter().filter(|v| v.is_none()).count() as i64;
            let samples = values
                .iter()
                .flatten()
                .take(5)
                .map(|v| Value::String(v.clone()))
                .collect();
            ColumnPreview {
                name: name.clone(),
                inferred_dtype: if df.is_numeric_column(name) {
                    "number".to_string()
                } else {
                    "string".to_string()
                },
                null_count,
                sample_values: samples,
            }
        })
        .collect()
}

fn dataset_insight(dataset_id: Uuid, profile: &DatasetProfile) -> Result<DatasetInsight, String> {
    Ok(DatasetInsight {
        dataset_id,
        row_count: profile.row_count as i64,
        column_count: profile.column_count as i64,
        total_missing: profile.total_missing as i64,
        columns: serde_json::to_value(&profile.columns).map_err(err)?,
        warnings: serde_json::to_value(&profile.warnings).map_err(err)?,
        created_at: utc_now(),
    })
}

#[tauri::command]
pub async fn lab_upload_dataset(path: String) -> Result<LabDatasetDetail, String> {
    let path = Path::new(&path);
    let bytes =
        std::fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    csv::parse_csv(&String::from_utf8_lossy(&bytes)).map_err(err)?;

    let (dataset_id, storage_key, sha256, size) = write_dataset(&bytes).map_err(err)?;
    let df = DataFrame::from_csv_path(&dataset_path(&storage_key))
        .map_err(|e| format!("failed to parse dataset: {e}"))?;
    let profile = insights::profile_dataframe(&df);
    let previews = column_previews(&df);

    let dataset = Dataset {
        id: dataset_id,
        original_filename: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        sha256,
        byte_size: size,
        storage_key,
        columns: previews,
        row_count: profile.row_count as i64,
        created_at: utc_now(),
    };

    let db = db().await?;
    datasets::insert_dataset(db.pool(), &dataset)
        .await
        .map_err(err)?;
    let insight = dataset_insight(dataset_id, &profile)?;
    datasets::insert_insight(db.pool(), &insight)
        .await
        .map_err(err)?;
    Ok(LabDatasetDetail { dataset, profile })
}

#[tauri::command]
pub async fn lab_list_datasets() -> Result<Vec<Dataset>, String> {
    let db = db().await?;
    datasets::list_datasets(db.pool()).await.map_err(err)
}

#[tauri::command]
pub async fn lab_dataset_insights(dataset_id: String) -> Result<Option<DatasetInsight>, String> {
    let id = Uuid::parse_str(&dataset_id).map_err(|e| format!("invalid dataset ID: {e}"))?;
    let db = db().await?;
    datasets::get_insight(db.pool(), id).await.map_err(err)
}

// ── jobs ────────────────────────────────────────────────────────────────────

async fn job_detail(db: &Database, job_id: Uuid) -> Result<Option<LabJobDetail>, String> {
    let Some(job) = jobs::get_job(db.pool(), job_id).await.map_err(err)? else {
        return Ok(None);
    };
    let metrics = jobs::get_metric(db.pool(), job_id).await.map_err(err)?;
    let artefact_list = artefacts::list_artefacts(db.pool(), job_id)
        .await
        .map_err(err)?;
    Ok(Some(LabJobDetail {
        job,
        metrics,
        artefacts: artefact_list,
    }))
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LabJobFilters {
    pub statuses: Vec<String>,
    pub task_types: Vec<String>,
    pub algorithm_ids: Vec<String>,
    pub q: Option<String>,
    pub sort_field: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabJobPage {
    pub items: Vec<Job>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabDatasetPreview {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
}

#[tauri::command]
pub async fn lab_list_jobs() -> Result<Vec<Job>, String> {
    let db = db().await?;
    let list = jobs::list_jobs(db.pool(), &jobs::JobFilters::default(), 200, None)
        .await
        .map_err(err)?;
    Ok(list.items)
}

#[tauri::command]
pub async fn lab_list_jobs_page(
    filters: LabJobFilters,
    cursor: Option<String>,
    limit: Option<usize>,
) -> Result<LabJobPage, String> {
    let db = db().await?;
    let job_filters = jobs::JobFilters {
        statuses: filters.statuses,
        task_types: filters.task_types,
        algorithm_ids: filters.algorithm_ids,
        tags: vec![],
        q: filters.q.filter(|q| !q.trim().is_empty()),
        created_from: None,
        created_to: None,
        sort_field: filters
            .sort_field
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "createdAt".to_string()),
        sort_direction: filters
            .sort_direction
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "desc".to_string()),
    };
    let cursor = cursor
        .map(|raw| {
            let (value, id) = queries::decode_cursor(&raw).map_err(|e| e.to_string())?;
            Ok::<jobs::JobCursor, String>(jobs::JobCursor { value, id })
        })
        .transpose()?;
    let page = jobs::list_jobs(
        db.pool(),
        &job_filters,
        limit.unwrap_or(25),
        cursor.as_ref(),
    )
    .await
    .map_err(err)?;
    Ok(LabJobPage {
        items: page.items,
        next_cursor: page.next_cursor,
    })
}

#[tauri::command]
pub async fn lab_dataset_preview(
    dataset_id: String,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<LabDatasetPreview, String> {
    let id = Uuid::parse_str(&dataset_id).map_err(|e| format!("invalid dataset ID: {e}"))?;
    let db = db().await?;
    let dataset = datasets::get_dataset(db.pool(), id)
        .await
        .map_err(err)?
        .ok_or_else(|| format!("dataset not found: {dataset_id}"))?;
    let df = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .map_err(|e| format!("failed to load dataset: {e}"))?;
    Ok(LabDatasetPreview {
        columns: df.column_names(),
        rows: insights::preview_rows(&df, offset.unwrap_or(0), limit.unwrap_or(20)),
    })
}

/// Preview arbitrary CSV/Parquet-adjacent tabular files for notebook
/// `metadata.preview` cells — returns up to `limit` rows (default 50, max 500).
#[tauri::command]
pub fn read_table_preview(path: String, limit: Option<usize>) -> Result<LabDatasetPreview, String> {
    let df = DataFrame::from_csv_path(Path::new(&path))
        .map_err(|e| format!("failed to read {path}: {e}"))?;
    Ok(LabDatasetPreview {
        columns: df.column_names(),
        rows: insights::preview_rows(&df, 0, limit.unwrap_or(50).min(500)),
    })
}

#[tauri::command]
pub async fn lab_get_job(job_id: String) -> Result<Option<LabJobDetail>, String> {
    let id = Uuid::parse_str(&job_id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = db().await?;
    job_detail(db, id).await
}

fn emit_progress(app: &tauri::AppHandle, job_id: Uuid, stage: &str) {
    use tauri::Emitter;
    let _ = app.emit(
        "lab-job-progress",
        serde_json::json!({ "jobId": job_id, "stage": stage }),
    );
}

#[tauri::command]
pub async fn lab_train(
    app: tauri::AppHandle,
    request: LabTrainRequest,
) -> Result<LabJobDetail, String> {
    let dataset_id =
        Uuid::parse_str(&request.dataset_id).map_err(|e| format!("invalid dataset ID: {e}"))?;
    let task = request
        .task_type
        .parse::<TaskType>()
        .map_err(|e| format!("invalid task type: {e}"))?;
    let spec = catalogue::find(&request.algorithm_id)
        .ok_or_else(|| format!("unknown algorithm: {}", request.algorithm_id))?;
    if spec.task_type != task {
        return Err(format!(
            "algorithm {} is for task {}, not {}",
            request.algorithm_id,
            spec.task_type.as_str(),
            task.as_str()
        ));
    }
    if request.feature_columns.is_empty() {
        return Err("at least one feature column is required".to_owned());
    }

    let db = db().await?;
    let dataset = datasets::get_dataset(db.pool(), dataset_id)
        .await
        .map_err(err)?
        .ok_or_else(|| format!("dataset not found: {dataset_id}"))?;
    let df = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .map_err(|e| format!("failed to load dataset: {e}"))?;

    let job = Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: task.as_str().to_string(),
        algorithm_id: request.algorithm_id.clone(),
        feature_columns: request.feature_columns,
        label_column: request.label_column,
        rl_columns: request.rl_columns,
        hyperparameters: request.hyperparameters,
        split_ratio: request.split_ratio,
        random_seed: request.random_seed,
        export_pkl: true,
        export_joblib: true,
        status: JobStatus::Queued.as_str().to_string(),
        error_code: None,
        error_message: None,
        library_versions: Some(serde_json::json!({
            "rust": env!("CARGO_PKG_VERSION"),
            "mars": env!("CARGO_PKG_VERSION"),
            "onnx": "1.x",
        })),
        name: request.name,
        tags: vec![],
        attempts: 0,
        max_attempts: 1,
        next_attempt_at: None,
        created_at: utc_now(),
        updated_at: utc_now(),
    };

    jobs::insert_job(db.pool(), &job).await.map_err(err)?;
    jobs::set_status(db.pool(), job.id, JobStatus::Training.as_str())
        .await
        .map_err(err)?;
    emit_progress(&app, job.id, "training");

    match train_job(&app, db, &job, &df).await {
        Ok(()) => {}
        Err(message) => {
            emit_progress(&app, job.id, "failed");
            let _ = jobs::record_failure(
                db.pool(),
                job.id,
                Some("TRAINING_FAILED".to_string()),
                Some(message.clone()),
            )
            .await;
            return Err(message);
        }
    }

    emit_progress(&app, job.id, "completed");
    job_detail(db, job.id)
        .await?
        .ok_or_else(|| "job disappeared after training".to_owned())
}

async fn train_job(
    app: &tauri::AppHandle,
    db: &Database,
    job: &Job,
    df: &DataFrame,
) -> Result<(), String> {
    let output: TrainingOutput = train(job, df).map_err(|e| format!("training failed: {e}"))?;

    jobs::set_status(db.pool(), job.id, JobStatus::Exporting.as_str())
        .await
        .map_err(err)?;
    emit_progress(app, job.id, "exporting");

    let onnx_bytes =
        export::onnx::export_onnx(&output.model).map_err(|e| format!("ONNX export failed: {e}"))?;

    ensure_dirs().map_err(err)?;
    let (storage_key, _path, byte_size) =
        write_artefact(job.id, &onnx_bytes, &format!("{}.onnx", job.id)).map_err(err)?;

    let artefact = Artefact {
        id: Uuid::new_v4(),
        job_id: job.id,
        format: ArtefactFormat::Onnx.as_str().to_string(),
        filename: format!("{}.onnx", job.id),
        byte_size,
        storage_key,
        created_at: utc_now(),
    };
    artefacts::insert_artefact(db.pool(), &artefact)
        .await
        .map_err(err)?;

    jobs::set_status(db.pool(), job.id, JobStatus::Completed.as_str())
        .await
        .map_err(err)?;

    let metric = JobMetric {
        job_id: job.id,
        metrics: output.metrics,
        charts: Some(output.charts),
        created_at: utc_now(),
    };
    jobs::record_metrics(db.pool(), &metric)
        .await
        .map_err(err)?;
    jobs::record_completion(db.pool(), job.id, Some(metric.metrics))
        .await
        .map_err(err)?;
    Ok(())
}

// ── artefacts ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn lab_export_artefact(
    job_id: String,
    format: String,
    output_path: String,
) -> Result<(), String> {
    let job_uuid = Uuid::parse_str(&job_id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = db().await?;
    let artefacts = artefacts::list_artefacts(db.pool(), job_uuid)
        .await
        .map_err(err)?;
    let artefact = artefacts
        .iter()
        .find(|a| a.format == format)
        .ok_or_else(|| format!("no {format} artefact found for job {job_id}"))?;
    let path = artefact_path(&artefact.storage_key, job_uuid);
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read artefact: {e}"))?;
    std::fs::write(&output_path, &bytes)
        .map_err(|e| format!("failed to write {output_path}: {e}"))?;
    Ok(())
}
