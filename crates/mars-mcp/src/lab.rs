//! `tools/call` handlers for Mars Lab: datasets, training jobs, and artefacts.
//!
//! Lab state is independent of the notebook [`crate::session::MarsSession`]:
//! datasets/jobs live in the Lab SQLite database configured by `mars-lab`
//! settings, so a [`LabBackend`] owns a current-thread tokio runtime and a
//! lazily opened [`Database`]. Synchronous handlers block on that runtime,
//! mirroring `mars lab` in `mars-cli`.

use mars_lab::catalogue;
use mars_lab::config::get_settings;
use mars_lab::db::{Database, artefacts, datasets, jobs};
use mars_lab::domain::{
    Artefact, ArtefactFormat, Dataset, Job, JobMetric, JobStatus, TaskType, utc_now,
};
use mars_lab::export;
use mars_lab::insights;
use mars_lab::storage::{artefact_path, dataset_path, ensure_dirs, write_artefact, write_dataset};
use mars_lab::training::{TrainingOutput, train};
use mars_lab::{csv, dataframe::DataFrame};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Lab tool dispatch + async runtime/database for one MCP server process.
pub struct LabBackend {
    rt: Arc<tokio::runtime::Runtime>,
    db: Option<Database>,
}

impl LabBackend {
    pub fn new() -> Result<Self, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("failed to start lab runtime: {error}"))?;
        Ok(Self {
            rt: Arc::new(rt),
            db: None,
        })
    }

    /// Backend backed by an in-memory database (tests and ephemeral use).
    #[doc(hidden)]
    pub fn with_in_memory_database() -> Result<Self, String> {
        let mut backend = Self::new()?;
        let rt = backend.rt.clone();
        let db = rt
            .block_on(Database::in_memory())
            .map_err(|error| error.to_string())?;
        backend.db = Some(db);
        Ok(backend)
    }

    async fn database(&mut self) -> Result<Database, String> {
        if self.db.is_none() {
            let db = Database::connect(&get_settings())
                .await
                .map_err(|error| error.to_string())?;
            self.db = Some(db);
        }
        self.db
            .clone()
            .ok_or_else(|| "lab database unavailable".to_owned())
    }

    /// Clone the runtime handle so futures can mutably borrow `self`.
    fn rt(&self) -> Arc<tokio::runtime::Runtime> {
        self.rt.clone()
    }
}

#[derive(Deserialize)]
struct AlgorithmsArgs {
    task: Option<String>,
}

#[derive(Deserialize)]
struct DatasetIdArgs {
    dataset_id: String,
}

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

#[derive(Deserialize)]
struct TrainArgs {
    dataset_id: String,
    task: String,
    algorithm: String,
    features: Vec<String>,
    label: Option<String>,
    hyperparameters: Option<Value>,
    split_ratio: Option<f64>,
    seed: Option<i64>,
}

#[derive(Deserialize)]
struct JobsArgs {
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct JobIdArgs {
    job_id: String,
}

#[derive(Deserialize)]
struct ExportArgs {
    job_id: String,
    output: String,
    format: Option<String>,
}

fn parse<T: for<'de> Deserialize<'de>>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|error| format!("invalid arguments: {error}"))
}

fn parse_uuid(field: &str, raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|error| format!("invalid {field}: {error}"))
}

pub fn call_lab_tool(
    backend: &mut LabBackend,
    name: &str,
    args: &Value,
) -> Option<Result<Value, String>> {
    let result = match name {
        "mars_lab_algorithms" => parse::<AlgorithmsArgs>(args).and_then(lab_algorithms),
        "mars_lab_dataset_upload" => {
            parse::<PathArgs>(args).and_then(|a| lab_dataset_upload(backend, a))
        }
        "mars_lab_datasets" => lab_datasets(backend),
        "mars_lab_dataset" => parse::<DatasetIdArgs>(args).and_then(|a| lab_dataset(backend, a)),
        "mars_lab_train" => parse::<TrainArgs>(args).and_then(|a| lab_train(backend, a)),
        "mars_lab_jobs" => parse::<JobsArgs>(args).and_then(|a| lab_jobs(backend, a)),
        "mars_lab_job" => parse::<JobIdArgs>(args).and_then(|a| lab_job(backend, a)),
        "mars_lab_export" => parse::<ExportArgs>(args).and_then(|a| lab_export(backend, a)),
        _ => return None,
    };
    Some(result)
}

// ── algorithms ──────────────────────────────────────────────────────────────

fn lab_algorithms(args: AlgorithmsArgs) -> Result<Value, String> {
    let task = args
        .task
        .as_deref()
        .map(|raw| TaskType::from_str(raw).map_err(|error| format!("invalid task: {error}")))
        .transpose()?;
    let algorithms: Vec<Value> = catalogue::for_task(task)
        .iter()
        .map(|spec| {
            json!({
                "id": spec.id,
                "name": spec.name,
                "task_type": spec.task_type.as_str(),
                "family": spec.family,
                "description": spec.description,
                "default_hyperparameters": spec.default_hyperparameters,
                "exports": spec.exports.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(json!({ "algorithms": algorithms }))
}

// ── datasets ────────────────────────────────────────────────────────────────

fn lab_datasets(backend: &mut LabBackend) -> Result<Value, String> {
    let rt = backend.rt();
    let db = rt.block_on(backend.database())?;
    let list = backend
        .rt
        .block_on(datasets::list_datasets(db.pool()))
        .map_err(|error| error.to_string())?;
    Ok(json!({ "datasets": list }))
}

fn lab_dataset_upload(backend: &mut LabBackend, args: PathArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let path = PathBuf::from(&args.path);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    csv::parse_csv(&String::from_utf8_lossy(&bytes)).map_err(|error| error.to_string())?;
    let (dataset_id, storage_key, sha256, size) =
        write_dataset(&bytes).map_err(|error| error.to_string())?;

    let db = rt.block_on(backend.database())?;
    let dataset = Dataset {
        id: dataset_id,
        original_filename: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string(),
        sha256: sha256.clone(),
        byte_size: size,
        storage_key: storage_key.clone(),
        columns: Vec::new(),
        row_count: 0,
        created_at: utc_now(),
    };
    backend
        .rt
        .block_on(datasets::insert_dataset(db.pool(), &dataset))
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "dataset": dataset,
        "storage_key": storage_key,
        "sha256": sha256,
        "byte_size": size,
    }))
}

fn lab_dataset(backend: &mut LabBackend, args: DatasetIdArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let dataset_id = parse_uuid("dataset_id", &args.dataset_id)?;
    let db = rt.block_on(backend.database())?;
    let dataset = backend
        .rt
        .block_on(datasets::get_dataset(db.pool(), dataset_id))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("dataset not found: {dataset_id}"))?;
    let insight = backend
        .rt
        .block_on(datasets::get_insight(db.pool(), dataset_id))
        .map_err(|error| error.to_string())?;

    // Best-effort profile from the stored CSV; failures degrade to `null`.
    let profile = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .ok()
        .map(|df| insights::profile_dataframe(&df));

    Ok(json!({
        "dataset": dataset,
        "insight": insight,
        "profile": profile,
    }))
}

// ── train ───────────────────────────────────────────────────────────────────

fn lab_train(backend: &mut LabBackend, args: TrainArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let dataset_id = parse_uuid("dataset_id", &args.dataset_id)?;
    let task = TaskType::from_str(&args.task).map_err(|error| error.to_string())?;
    let spec = catalogue::find(&args.algorithm)
        .ok_or_else(|| format!("unknown algorithm: {}", args.algorithm))?;
    if spec.task_type != task {
        return Err(format!(
            "algorithm {} is for task {}, not {}",
            args.algorithm,
            spec.task_type.as_str(),
            task.as_str()
        ));
    }
    let feature_columns: Vec<String> = args
        .features
        .iter()
        .map(|column| column.trim().to_string())
        .filter(|column| !column.is_empty())
        .collect();
    if feature_columns.is_empty() {
        return Err("at least one feature column is required".to_owned());
    }

    let db = rt.block_on(backend.database())?;
    let dataset = backend
        .rt
        .block_on(datasets::get_dataset(db.pool(), dataset_id))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("dataset not found: {dataset_id}"))?;
    let df = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .map_err(|error| format!("failed to load dataset: {error}"))?;

    let now = utc_now();
    let job = Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: task.as_str().to_string(),
        algorithm_id: args.algorithm.clone(),
        feature_columns,
        label_column: args.label.clone(),
        rl_columns: None,
        hyperparameters: args.hyperparameters.unwrap_or_else(|| json!({})),
        split_ratio: args.split_ratio.unwrap_or(0.2),
        random_seed: args.seed.unwrap_or(42),
        export_pkl: true,
        export_joblib: true,
        status: JobStatus::Queued.as_str().to_string(),
        error_code: None,
        error_message: None,
        library_versions: Some(json!({
            "rust": env!("CARGO_PKG_VERSION"),
            "mars": env!("CARGO_PKG_VERSION"),
            "onnx": "1.x",
        })),
        name: None,
        tags: vec![],
        attempts: 0,
        max_attempts: 1,
        next_attempt_at: None,
        created_at: now,
        updated_at: now,
    };

    backend
        .rt
        .block_on(jobs::insert_job(db.pool(), &job))
        .map_err(|error| error.to_string())?;
    backend
        .rt
        .block_on(jobs::set_status(
            db.pool(),
            job.id,
            JobStatus::Training.as_str(),
        ))
        .map_err(|error| error.to_string())?;

    // `train` and ONNX export are synchronous; DB transitions happen between
    // the synchronous phases on the backend's current-thread runtime.
    let trained = train(&job, &df).map_err(|error| error.to_string());
    let trained = match trained {
        Ok(output) => output,
        Err(error) => {
            let _ = rt.block_on(jobs::record_failure(
                db.pool(),
                job.id,
                Some("TRAINING_FAILED".to_string()),
                Some(error.clone()),
            ));
            return Err(format!("training failed: {error}"));
        }
    };

    if let Err(error) = rt.block_on(jobs::set_status(
        db.pool(),
        job.id,
        JobStatus::Exporting.as_str(),
    )) {
        return Err(error.to_string());
    }

    let exported = export_artefact(&job, &trained);
    let (storage_key, byte_size) = match exported {
        Ok(result) => result,
        Err(error) => {
            let _ = rt.block_on(jobs::record_failure(
                db.pool(),
                job.id,
                Some("EXPORT_FAILED".to_string()),
                Some(error.clone()),
            ));
            return Err(error);
        }
    };

    let artefact = Artefact {
        id: Uuid::new_v4(),
        job_id: job.id,
        format: ArtefactFormat::Onnx.as_str().to_string(),
        filename: format!("{}.onnx", job.id),
        byte_size,
        storage_key,
        created_at: utc_now(),
    };
    if let Err(error) = backend
        .rt
        .block_on(artefacts::insert_artefact(db.pool(), &artefact))
    {
        return Err(error.to_string());
    }
    if let Err(error) = rt.block_on(jobs::set_status(
        db.pool(),
        job.id,
        JobStatus::Completed.as_str(),
    )) {
        return Err(error.to_string());
    }

    let metric = JobMetric {
        job_id: job.id,
        metrics: trained.metrics,
        charts: Some(trained.charts),
        created_at: utc_now(),
    };
    if let Err(error) = backend
        .rt
        .block_on(jobs::record_metrics(db.pool(), &metric))
    {
        return Err(error.to_string());
    }
    if let Err(error) = rt.block_on(jobs::record_completion(
        db.pool(),
        job.id,
        Some(metric.metrics.clone()),
    )) {
        return Err(error.to_string());
    }

    Ok(json!({
        "job": job,
        "status": JobStatus::Completed.as_str(),
        "metrics": metric.metrics,
        "charts": metric.charts,
        "artefact": artefact,
    }))
}

/// Export the trained model to ONNX and persist the artefact file.
/// Returns `(storage_key, byte_size)`.
fn export_artefact(job: &Job, output: &TrainingOutput) -> Result<(String, i64), String> {
    let onnx_bytes = export::onnx::export_onnx(&output.model)
        .map_err(|error| format!("ONNX export failed: {error}"))?;
    ensure_dirs().map_err(|error| error.to_string())?;
    let (storage_key, _path, byte_size) =
        write_artefact(job.id, &onnx_bytes, &format!("{}.onnx", job.id))
            .map_err(|error| error.to_string())?;
    Ok((storage_key, byte_size))
}

// ── jobs ────────────────────────────────────────────────────────────────────

fn lab_jobs(backend: &mut LabBackend, args: JobsArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let db = rt.block_on(backend.database())?;
    let limit = args.limit.unwrap_or(100).clamp(1, 500);
    let page = backend
        .rt
        .block_on(jobs::list_jobs(
            db.pool(),
            &jobs::JobFilters::default(),
            limit,
            None,
        ))
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "jobs": page.items,
        "next_cursor": page.next_cursor,
    }))
}

fn lab_job(backend: &mut LabBackend, args: JobIdArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let job_id = parse_uuid("job_id", &args.job_id)?;
    let db = rt.block_on(backend.database())?;
    let job = backend
        .rt
        .block_on(jobs::get_job(db.pool(), job_id))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("job not found: {job_id}"))?;
    let metric = backend
        .rt
        .block_on(jobs::get_metric(db.pool(), job_id))
        .map_err(|error| error.to_string())?;
    let artefacts = backend
        .rt
        .block_on(artefacts::list_artefacts(db.pool(), job_id))
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "job": job,
        "metric": metric,
        "artefacts": artefacts,
    }))
}

// ── export ──────────────────────────────────────────────────────────────────

fn lab_export(backend: &mut LabBackend, args: ExportArgs) -> Result<Value, String> {
    let rt = backend.rt();
    let job_id = parse_uuid("job_id", &args.job_id)?;
    let format = args.format.unwrap_or_else(|| "onnx".to_string());
    let output = PathBuf::from(&args.output);

    let db = rt.block_on(backend.database())?;
    let artefacts = backend
        .rt
        .block_on(artefacts::list_artefacts(db.pool(), job_id))
        .map_err(|error| error.to_string())?;
    let artefact = artefacts
        .iter()
        .find(|a| a.format == format)
        .ok_or_else(|| format!("no {format} artefact found for job {job_id}"))?;
    let path = artefact_path(&artefact.storage_key, job_id);
    let bytes =
        std::fs::read(&path).map_err(|error| format!("failed to read artefact: {error}"))?;
    std::fs::write(&output, &bytes).map_err(|error| error.to_string())?;
    Ok(json!({
        "job_id": job_id,
        "format": format,
        "output": output,
        "byte_size": bytes.len(),
    }))
}
