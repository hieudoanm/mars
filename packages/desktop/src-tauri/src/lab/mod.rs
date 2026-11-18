//! Mars Lab Tauri commands — CSV datasets, training jobs, metrics, ONNX export.
//!
//! Thin command layer over `mars-lab` (training/ONNX + SQLite persistence). The shared [`Database`] is initialised lazily on first
//! use; migrations run inside `Database::connect`.

mod datasets;
mod jobs;
mod train;

pub use datasets::{
    lab_dataset_insights, lab_dataset_preview, lab_list_datasets, lab_upload_dataset,
    read_table_preview,
};
pub use jobs::{lab_get_job, lab_list_jobs, lab_list_jobs_page};
pub use train::{emit_progress, lab_notification_body, lab_train, lab_train_inner};

use mars_lab::catalogue;
use mars_lab::config::get_settings;
use mars_lab::db::Database;
use mars_lab::domain::Dataset;
use mars_lab::domain::{Artefact, Job, JobMetric, RLColumns, TaskType};
use mars_lab::insights::DatasetProfile;
use serde::Serialize;
use serde_json::Value;

static DB: tokio::sync::OnceCell<Database> = tokio::sync::OnceCell::const_new();

pub(super) async fn db() -> Result<&'static Database, String> {
    DB.get_or_try_init(|| async {
        Database::connect(&get_settings())
            .await
            .map_err(|error| error.to_string())
    })
    .await
}

pub(super) fn err(error: impl std::fmt::Display) -> String {
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

// ── artefacts ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn lab_export_artefact(
    job_id: String,
    format: String,
    output_path: String,
) -> Result<(), String> {
    let job_uuid = uuid::Uuid::parse_str(&job_id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = db().await?;
    let artefacts = mars_lab::db::artefacts::list_artefacts(db.pool(), job_uuid)
        .await
        .map_err(err)?;
    let artefact = artefacts
        .iter()
        .find(|a| a.format == format)
        .ok_or_else(|| format!("no {format} artefact found for job {job_id}"))?;
    let path = mars_lab::storage::artefact_path(&artefact.storage_key, job_uuid);
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read artefact: {e}"))?;
    std::fs::write(&output_path, &bytes)
        .map_err(|e| format!("failed to write {output_path}: {e}"))?;
    Ok(())
}
