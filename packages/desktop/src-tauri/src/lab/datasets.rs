//! Dataset commands: upload, list, insights, previews.

use super::{LabDatasetDetail, LabDatasetPreview, db, err};
use mars_lab::csv;
use mars_lab::dataframe::DataFrame;
use mars_lab::db::datasets;
use mars_lab::domain::{ColumnPreview, Dataset, DatasetInsight, utc_now};
use mars_lab::insights::{self, DatasetProfile};
use mars_lab::storage::{dataset_path, write_dataset};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

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
