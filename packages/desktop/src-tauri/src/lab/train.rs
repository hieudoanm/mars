//! Training command: validates the request, runs the trainer, exports ONNX.

use super::jobs::job_detail;
use super::{LabJobDetail, LabTrainRequest, db, err};
use mars_lab::catalogue;
use mars_lab::dataframe::DataFrame;
use mars_lab::db::{Database, artefacts, datasets, jobs};
use mars_lab::domain::{Artefact, ArtefactFormat, Job, JobMetric, JobStatus, TaskType, utc_now};
use mars_lab::export;
use mars_lab::storage::{dataset_path, ensure_dirs, write_artefact};
use mars_lab::training::{TrainingOutput, train};
use uuid::Uuid;

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
