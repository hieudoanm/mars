//! Job persistence: discovery filters, keyset pagination, the worker lifecycle
//! writes, and metrics.

pub mod filters;
pub mod helpers;
pub mod metrics;
pub mod queries;
pub mod writes;

pub use filters::{JobCursor, JobFilters, WorkerCounts, parse_filters};
pub use helpers::{
    ACTIVE_STATUSES, DEFAULT_MAX_ATTEMPTS, MAX_QUERY_LENGTH, SORT_FIELDS, job_id_from_hash, job_key,
};
pub use metrics::{delete_metrics, get_metric, metrics_by_ids, record_metrics};
pub use queries::{
    JobPage, decode_cursor, delete_job, get_job, job_exists, jobs_by_ids, list_jobs,
    queue_positions, queued_ids, worker_counts,
};
pub use writes::{
    fail_stale_active, insert_job, mark_attempt, record_completion, record_failure,
    requeue_retrying, schedule_retry, set_status, update_metadata,
};
