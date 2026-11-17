//! Job commands: list, page, detail.

use super::{LabJobDetail, LabJobFilters, LabJobPage, db, err};
use mars_lab::db::jobs::queries;
use mars_lab::db::{Database, artefacts, jobs};
use mars_lab::domain::Job;
use uuid::Uuid;

pub(super) async fn job_detail(
    db: &Database,
    job_id: Uuid,
) -> Result<Option<LabJobDetail>, String> {
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
pub async fn lab_get_job(job_id: String) -> Result<Option<LabJobDetail>, String> {
    let id = Uuid::parse_str(&job_id).map_err(|e| format!("invalid job ID: {e}"))?;
    let db = db().await?;
    job_detail(db, id).await
}
