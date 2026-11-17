//! MCP tool definitions exposed by `mars mcp serve`.

use crate::mcp::{Annotations, Tool};
use serde_json::json;

fn object_schema(properties: serde_json::Value, required: &[&str]) -> serde_json::Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

fn path_prop() -> serde_json::Value {
    json!({ "type": "string", "description": "Path to the .mars notebook" })
}

fn str_prop(description: &str) -> serde_json::Value {
    json!({ "type": "string", "description": description })
}

fn tool(
    name: &str,
    description: &str,
    schema: serde_json::Value,
    annotations: Annotations,
) -> Tool {
    Tool {
        name: name.to_owned(),
        title: Some(crate::mcp::titleize(name)),
        description: Some(description.to_owned()),
        input_schema: schema,
        annotations,
    }
}

pub fn definitions() -> Vec<Tool> {
    vec![
        tool(
            "mars_notebook_new",
            "Create a new .mars notebook file with a starter Markdown cell.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "language": str_prop("Notebook language (default: rust)"),
                }),
                &["path"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_notebook_open",
            "Open a .mars notebook into the session and return its summary.",
            object_schema(json!({ "path": path_prop() }), &["path"]),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_notebook_read",
            "Read a notebook's metadata and cells (optionally a single cell).",
            object_schema(
                json!({
                    "path": path_prop(),
                    "cell_index": { "type": "integer", "description": "0-based cell index; omit for the whole notebook" },
                    "include_outputs": { "type": "boolean", "description": "Include execution outputs (default: true)" },
                }),
                &["path"],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_notebook_save",
            "Persist session changes to disk (atomic write).",
            object_schema(json!({ "path": path_prop() }), &["path"]),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_notebook_inspect",
            "Inspect notebook metadata: format version, language, cell count, dependencies.",
            object_schema(json!({ "path": path_prop() }), &["path"]),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_add",
            "Insert a cell into a notebook. Cell type is code, markdown, or raw.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "cell_type": { "type": "string", "enum": ["code", "markdown", "raw"] },
                    "source": str_prop("Cell source code/text"),
                    "index": { "type": "integer", "description": "0-based insertion index; defaults to the end" },
                    "language": str_prop("Per-cell language override (stored in cell metadata)"),
                }),
                &["path", "cell_type"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_update",
            "Update a cell's source and/or type.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "index": { "type": "integer", "description": "0-based cell index" },
                    "source": str_prop("New cell source"),
                    "cell_type": { "type": "string", "enum": ["code", "markdown", "raw"] },
                }),
                &["path", "index"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_delete",
            "Delete a cell by index.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "index": { "type": "integer", "description": "0-based cell index" },
                }),
                &["path", "index"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: true,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_move",
            "Move a cell to a new index.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "from": { "type": "integer", "description": "0-based source index" },
                    "to": { "type": "integer", "description": "0-based destination index" },
                }),
                &["path", "from", "to"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_execute",
            "Execute one code cell (or ad-hoc source) in the session kernel and persist results.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "index": { "type": "integer", "description": "0-based cell index; omit together with source for ad-hoc execution" },
                    "source": str_prop("Ad-hoc source to execute instead of a cell"),
                    "language": str_prop("Runtime language override"),
                    "timeout_ms": { "type": "integer", "description": "Execution timeout in milliseconds" },
                }),
                &["path"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_notebook_execute",
            "Execute all code cells sequentially, record outputs, and save the notebook.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "timeout_ms": { "type": "integer", "description": "Per-cell execution timeout in milliseconds" },
                }),
                &["path"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_cell_diagnose",
            "Compile-check a cell (or ad-hoc source) without committing it.",
            object_schema(
                json!({
                    "path": path_prop(),
                    "index": { "type": "integer", "description": "0-based cell index" },
                    "source": str_prop("Ad-hoc source to diagnose"),
                    "language": str_prop("Runtime language override"),
                }),
                &["path"],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_kernel_status",
            "Report kernel execution status for a notebook/language.",
            object_schema(
                json!({
                    "path": { "type": "string", "description": "Notebook path (optional)" },
                    "language": str_prop("Runtime language (default: rust)"),
                }),
                &[],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_kernel_reset",
            "Reset the session kernel for a notebook/language.",
            object_schema(
                json!({
                    "path": { "type": "string", "description": "Notebook path (optional)" },
                    "language": str_prop("Runtime language (default: rust)"),
                }),
                &[],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: true,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_kernel_variables",
            "List live bindings in the session kernel.",
            object_schema(
                json!({
                    "path": { "type": "string", "description": "Notebook path (optional)" },
                    "language": str_prop("Runtime language (default: rust)"),
                }),
                &[],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_algorithms",
            "List Mars Lab algorithms, optionally filtered by task type.",
            object_schema(
                json!({
                    "task": { "type": "string", "enum": ["classification", "regression", "semi_supervised", "clustering", "anomaly_detection", "reinforcement_learning"], "description": "Filter by task type" },
                }),
                &[],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_dataset_upload",
            "Upload a CSV file as a Mars Lab dataset.",
            object_schema(
                json!({ "path": str_prop("Path to the .csv file") }),
                &["path"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_datasets",
            "List datasets stored in the Mars Lab database.",
            object_schema(json!({}), &[]),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_dataset",
            "Inspect a dataset: record, stored insight, and computed profile.",
            object_schema(
                json!({ "dataset_id": str_prop("Dataset UUID") }),
                &["dataset_id"],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_train",
            "Train a model on a dataset and export an ONNX artefact.",
            object_schema(
                json!({
                    "dataset_id": str_prop("Dataset UUID"),
                    "task": { "type": "string", "enum": ["classification", "regression", "semi_supervised", "clustering", "anomaly_detection", "reinforcement_learning"] },
                    "algorithm": str_prop("Algorithm ID from mars_lab_algorithms"),
                    "features": { "type": "array", "items": { "type": "string" }, "description": "Feature column names" },
                    "label": str_prop("Label column (required for supervised tasks)"),
                    "hyperparameters": { "type": "object", "description": "Algorithm hyperparameters (defaults applied)" },
                    "split_ratio": { "type": "number", "description": "Train/validation split ratio (default: 0.2)" },
                    "seed": { "type": "integer", "description": "Random seed (default: 42)" },
                }),
                &["dataset_id", "task", "algorithm", "features"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: false,
                idempotent_hint: false,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_jobs",
            "List Mars Lab training jobs.",
            object_schema(
                json!({
                    "limit": { "type": "integer", "description": "Maximum jobs to return (default: 100, max: 500)" },
                }),
                &[],
            ),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_job",
            "Inspect a training job: record, metrics, and artefacts.",
            object_schema(json!({ "job_id": str_prop("Job UUID") }), &["job_id"]),
            Annotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
        tool(
            "mars_lab_export",
            "Export a job artefact (onnx|pkl|joblib) to a file path.",
            object_schema(
                json!({
                    "job_id": str_prop("Job UUID"),
                    "output": str_prop("Destination file path"),
                    "format": { "type": "string", "enum": ["onnx", "pkl", "joblib"], "description": "Artefact format (default: onnx)" },
                }),
                &["job_id", "output"],
            ),
            Annotations {
                read_only_hint: false,
                destructive_hint: true,
                idempotent_hint: true,
                open_world_hint: false,
            },
        ),
    ]
}
