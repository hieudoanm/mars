//! `mars lab` — Mars Lab: CSV → trained model → ONNX artefact.
//!
//! Thin CLI orchestration over `mars-lab` (training/ONNX + SQLite persistence). Async database calls run on a
//! current-thread tokio runtime owned by this module.

use mars_lab::config::get_settings;
use mars_lab::db::{Database, artefacts, datasets, jobs};
use mars_lab::domain::{Artefact, ArtefactFormat, Job, JobMetric, JobStatus, TaskType, utc_now};
use mars_lab::export;
use mars_lab::storage::{artefact_path, dataset_path, ensure_dirs, write_artefact, write_dataset};
use mars_lab::training::{TrainingOutput, train};
use mars_lab::{catalogue, csv, dataframe::DataFrame};
use serde_json::Value;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn run_lab(args: &[String]) -> Result<(), String> {
    mars_lab::config::setup_logging(&get_settings().log_level);
    let Some(command) = args.first().map(String::as_str) else {
        return Err(lab_usage());
    };
    let rest = &args[1..];
    match command {
        "algorithms" => {
            let task = parse_flag(rest, "--task")?;
            cmd_algorithms(task)
        }
        "dataset" => {
            let path = rest.first().ok_or_else(lab_usage)?;
            with_runtime(|rt| rt.block_on(cmd_dataset(Path::new(path))))
        }
        "datasets" => cmd_datasets(),
        "train" => {
            let opts = TrainOpts::parse(rest)?;
            with_runtime(|rt| rt.block_on(cmd_train(&opts)))
        }
        "job" => {
            let id = rest.first().ok_or_else(lab_usage)?;
            with_runtime(|rt| rt.block_on(cmd_job(id)))
        }
        "jobs" => with_runtime(|rt| rt.block_on(cmd_jobs())),
        "export" => {
            let job_id = flag_value(rest, "--job-id")
                .ok_or_else(|| "export requires --job-id <uuid>".to_owned())?;
            let output = flag_value(rest, "--output")
                .map(PathBuf::from)
                .ok_or_else(|| "export requires --output <path>".to_owned())?;
            let format = flag_value(rest, "--format").unwrap_or("onnx");
            with_runtime(|rt| rt.block_on(cmd_export(job_id, &output, format)))
        }
        _ => Err(lab_usage()),
    }
}

fn lab_usage() -> String {
    "usage: mars lab algorithms [--task <type>] | dataset <file.csv> | datasets | train --dataset-id <id> --task <type> --algorithm <id> --features a,b [--label col] [--hyperparameters json] [--split-ratio f] [--seed n] | job <id> | jobs | export --job-id <id> --output <path> [--format onnx|pkl|joblib]".to_owned()
}

fn parse_flag(args: &[String], name: &str) -> Result<Option<TaskType>, String> {
    match flag_value(args, name) {
        Some(raw) => raw
            .parse::<TaskType>()
            .map(Some)
            .map_err(|error| format!("invalid --task: {error}")),
        None => Ok(None),
    }
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn with_runtime<T>(
    f: impl FnOnce(&tokio::runtime::Runtime) -> Result<T, String>,
) -> Result<T, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to start async runtime: {error}"))?;
    f(&rt)
}

// ── algorithms ──────────────────────────────────────────────────────────────

fn cmd_algorithms(task: Option<TaskType>) -> Result<(), String> {
    for spec in catalogue::for_task(task) {
        println!("{} ({}) - {}", spec.id, spec.task_type, spec.description);
        println!("  defaults: {}", spec.default_hyperparameters);
    }
    Ok(())
}

// ── datasets ────────────────────────────────────────────────────────────────

async fn cmd_dataset(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    csv::parse_csv(&String::from_utf8_lossy(&bytes)).map_err(|e| e.to_string())?;

    let (dataset_id, storage_key, sha256, size) =
        write_dataset(&bytes).map_err(|e| e.to_string())?;

    let db = Database::connect(&get_settings())
        .await
        .map_err(|e| e.to_string())?;
    let dataset = mars_lab::domain::Dataset {
        id: dataset_id,
        original_filename: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        sha256: sha256.clone(),
        byte_size: size,
        storage_key: storage_key.clone(),
        columns: Vec::new(),
        row_count: 0,
        created_at: utc_now(),
    };
    datasets::insert_dataset(db.pool(), &dataset)
        .await
        .map_err(|e| e.to_string())?;

    println!("Dataset uploaded:");
    println!("  id: {dataset_id}");
    println!("  storage_key: {storage_key}");
    println!("  sha256: {sha256}");
    println!("  size: {size} bytes");
    Ok(())
}

fn cmd_datasets() -> Result<(), String> {
    let dir = mars_lab::storage::datasets_dir();
    if !dir.exists() {
        println!("No datasets found.");
        return Ok(());
    }
    println!("Datasets:");
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().join(format!("{name}.csv")).exists() {
            println!("  {name}");
        }
    }
    Ok(())
}

// ── train ───────────────────────────────────────────────────────────────────

struct TrainOpts {
    dataset_id: String,
    task: TaskType,
    algorithm: String,
    features: String,
    label: Option<String>,
    hyperparameters: String,
    split_ratio: f64,
    seed: i64,
}

impl TrainOpts {
    fn parse(args: &[String]) -> Result<Self, String> {
        let required = |name: &str| {
            flag_value(args, name)
                .map(str::to_owned)
                .ok_or_else(|| format!("train requires {name} <value>"))
        };
        let task_raw = required("--task")?;
        let task = task_raw
            .parse::<TaskType>()
            .map_err(|error| format!("invalid --task: {error}"))?;
        let split_ratio = flag_value(args, "--split-ratio")
            .map(|v| v.parse::<f64>())
            .transpose()
            .map_err(|_| "invalid --split-ratio".to_owned())?
            .unwrap_or(0.2);
        let seed = flag_value(args, "--seed")
            .map(|v| v.parse::<i64>())
            .transpose()
            .map_err(|_| "invalid --seed".to_owned())?
            .unwrap_or(42);
        Ok(Self {
            dataset_id: required("--dataset-id")?,
            task,
            algorithm: required("--algorithm")?,
            features: required("--features")?,
            label: flag_value(args, "--label").map(str::to_owned),
            hyperparameters: flag_value(args, "--hyperparameters")
                .unwrap_or("{}")
                .to_owned(),
            split_ratio,
            seed,
        })
    }
}

async fn cmd_train(opts: &TrainOpts) -> Result<(), String> {
    let dataset_id =
        Uuid::parse_str(&opts.dataset_id).map_err(|e| format!("invalid dataset ID: {e}"))?;
    let spec = catalogue::find(&opts.algorithm)
        .ok_or_else(|| format!("unknown algorithm: {}", opts.algorithm))?;
    if spec.task_type != opts.task {
        return Err(format!(
            "algorithm {} is for task {}, not {}",
            opts.algorithm, spec.task_type, opts.task
        ));
    }
    let hyperparameters: Value = serde_json::from_str(&opts.hyperparameters)
        .map_err(|e| format!("invalid hyperparameters JSON: {e}"))?;
    let feature_columns: Vec<String> = opts
        .features
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if feature_columns.is_empty() {
        return Err("at least one feature column is required".to_owned());
    }

    let db = Database::connect(&get_settings())
        .await
        .map_err(|e| e.to_string())?;
    let dataset = datasets::get_dataset(db.pool(), dataset_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("dataset not found: {dataset_id}"))?;
    let df = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .map_err(|e| format!("failed to load dataset: {e}"))?;

    let job = Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: opts.task.as_str().to_string(),
        algorithm_id: opts.algorithm.clone(),
        feature_columns,
        label_column: opts.label.clone(),
        rl_columns: None,
        hyperparameters,
        split_ratio: opts.split_ratio,
        random_seed: opts.seed,
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
        name: None,
        tags: vec![],
        attempts: 0,
        max_attempts: 1,
        next_attempt_at: None,
        created_at: utc_now(),
        updated_at: utc_now(),
    };

    jobs::insert_job(db.pool(), &job)
        .await
        .map_err(|e| e.to_string())?;
    jobs::set_status(db.pool(), job.id, JobStatus::Training.as_str())
        .await
        .map_err(|e| e.to_string())?;

    let result = train_job(&db, &job, &df).await;
    if let Err(err) = &result {
        let _ = jobs::record_failure(
            db.pool(),
            job.id,
            Some("TRAINING_FAILED".to_string()),
            Some(err.clone()),
        )
        .await;
    }
    result
}

async fn train_job(db: &Database, job: &Job, df: &DataFrame) -> Result<(), String> {
    println!("Training job {}...", job.id);
    println!("  Task: {}", job.task_type);
    println!("  Algorithm: {}", job.algorithm_id);
    println!("  Features: {:?}", job.feature_columns);
    println!("  Label: {:?}", job.label_column);

    let output: TrainingOutput = train(job, df).map_err(|e| format!("training failed: {e}"))?;

    jobs::set_status(db.pool(), job.id, JobStatus::Exporting.as_str())
        .await
        .map_err(|e| e.to_string())?;

    println!("\nTraining completed!");
    println!("  Model kind: {:?}", output.model.kind);
    println!("\nMetrics:");
    println!(
        "  {}",
        serde_json::to_string_pretty(&output.metrics).unwrap_or_default()
    );
    if !output.charts.as_object().is_none_or(|m| m.is_empty()) {
        println!("\nCharts:");
        println!(
            "  {}",
            serde_json::to_string_pretty(&output.charts).unwrap_or_default()
        );
    }

    let onnx_bytes =
        export::onnx::export_onnx(&output.model).map_err(|e| format!("ONNX export failed: {e}"))?;

    ensure_dirs().map_err(|e| e.to_string())?;
    let (storage_key, path, byte_size) =
        write_artefact(job.id, &onnx_bytes, &format!("{}.onnx", job.id))
            .map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;

    jobs::set_status(db.pool(), job.id, JobStatus::Completed.as_str())
        .await
        .map_err(|e| e.to_string())?;

    let metric = JobMetric {
        job_id: job.id,
        metrics: output.metrics,
        charts: Some(output.charts),
        created_at: utc_now(),
    };
    jobs::record_metrics(db.pool(), &metric)
        .await
        .map_err(|e| e.to_string())?;
    jobs::record_completion(db.pool(), job.id, Some(metric.metrics))
        .await
        .map_err(|e| e.to_string())?;

    println!("\nONNX exported to: {}", path.display());
    println!("  Size: {} bytes", onnx_bytes.len());
    println!("  Job {} completed", job.id);
    Ok(())
}

// ── jobs ────────────────────────────────────────────────────────────────────

async fn cmd_job(id: &str) -> Result<(), String> {
    let job_id = Uuid::parse_str(id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = Database::connect(&get_settings())
        .await
        .map_err(|e| e.to_string())?;
    let job = jobs::get_job(db.pool(), job_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("job not found: {id}"))?;
    let metric = jobs::get_metric(db.pool(), job_id)
        .await
        .map_err(|e| e.to_string())?;
    let artefacts = artefacts::list_artefacts(db.pool(), job_id)
        .await
        .map_err(|e| e.to_string())?;

    println!("Job: {}", job.id);
    println!("  Status: {}", job.status);
    println!("  Task: {}", job.task_type);
    println!("  Algorithm: {}", job.algorithm_id);
    println!("  Features: {:?}", job.feature_columns);
    println!("  Label: {:?}", job.label_column);
    println!("  Created: {}", job.created_at);
    println!("  Updated: {}", job.updated_at);
    if let Some(metric) = metric {
        println!("\nMetrics:");
        println!(
            "  {}",
            serde_json::to_string_pretty(&metric.metrics).unwrap_or_default()
        );
    }
    if !artefacts.is_empty() {
        println!("\nArtefacts:");
        for a in &artefacts {
            println!("  {} ({}) - {} bytes", a.format, a.filename, a.byte_size);
        }
    }
    Ok(())
}

async fn cmd_jobs() -> Result<(), String> {
    let db = Database::connect(&get_settings())
        .await
        .map_err(|e| e.to_string())?;
    let list = jobs::list_jobs(db.pool(), &jobs::JobFilters::default(), 100, None)
        .await
        .map_err(|e| e.to_string())?;
    if list.items.is_empty() {
        println!("No jobs found.");
        return Ok(());
    }
    println!("Jobs:");
    for job in &list.items {
        println!(
            "  {} - {} - {} - {}",
            job.id, job.status, job.task_type, job.algorithm_id
        );
    }
    Ok(())
}

// ── export ──────────────────────────────────────────────────────────────────

async fn cmd_export(job_id: &str, output: &Path, format: &str) -> Result<(), String> {
    let job_uuid = Uuid::parse_str(job_id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = Database::connect(&get_settings())
        .await
        .map_err(|e| e.to_string())?;
    let artefacts = artefacts::list_artefacts(db.pool(), job_uuid)
        .await
        .map_err(|e| e.to_string())?;
    let artefact = artefacts
        .iter()
        .find(|a| a.format == format)
        .ok_or_else(|| format!("no {format} artefact found for job {job_id}"))?;
    let path = artefact_path(&artefact.storage_key, job_uuid);
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read artefact: {e}"))?;
    std::fs::write(output, &bytes).map_err(|e| e.to_string())?;
    println!("Exported {format} artefact to: {}", output.display());
    println!("  Size: {} bytes", bytes.len());
    Ok(())
}
