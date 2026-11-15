//! Git-aware change detection for `mars notebook run --changed-only`.
//!
//! Cells are matched across revisions by their stable `CellId`. The baseline
//! is read with `git show <ref>:<path>`; when `--changed-only` is used
//! without `--since`, `HEAD` is the default ref. Execution resumes at the
//! first code cell that is new or whose source changed, because the runtime
//! starts with empty state and every later cell may depend on earlier ones.

use mars_core::CellType;
use mars_document::{Cell, Notebook};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Index of the first code cell that differs from the baseline notebook.
/// Returns `None` when no code cell changed. Pure function — unit-testable.
pub fn first_changed_code_index(current: &Notebook, baseline: &Notebook) -> Option<usize> {
    let baseline_sources: HashMap<String, &str> = baseline
        .cells
        .iter()
        .map(|cell| (cell.id.to_string(), cell.source.as_str()))
        .collect();
    current.cells.iter().position(|cell| {
        cell.cell_type == CellType::Code
            && baseline_sources
                .get(cell.id.as_str())
                .is_none_or(|source| *source != cell.source)
    })
}

/// Load the notebook as it existed at `git_ref` for `path`.
pub fn baseline_notebook(path: &str, git_ref: &str) -> Result<Notebook, String> {
    let absolute =
        std::fs::canonicalize(path).map_err(|error| format!("cannot resolve {path}: {error}"))?;
    let search_dir = absolute
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let repo_root = git_output(&search_dir, &["rev-parse", "--show-toplevel"])?
        .trim()
        .to_owned();
    let repo_root_path = Path::new(&repo_root);
    let relative = absolute
        .strip_prefix(repo_root_path)
        .map_err(|_| format!("{path} is outside the git repository {repo_root}"))?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    // Confirm the file is tracked so the error is actionable.
    let check = Command::new("git")
        .args(["ls-files", "--error-unmatch", "--", &relative])
        .current_dir(repo_root_path)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !check.status.success() {
        return Err(format!(
            "{path} is not tracked by git; commit it or drop --changed-only/--since"
        ));
    }
    let spec = format!("{git_ref}:{relative}");
    let content = git_output(repo_root_path, &["show", &spec])?;
    mars_document::from_json(&content).map_err(|error| error.to_string())
}

/// How a cell differs between two notebook revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellChange {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Classify every cell of `current` against `baseline`, matched by stable
/// `CellId`. Order follows `current`; removed cells are appended at the end.
pub fn diff_cells<'a>(
    current: &'a Notebook,
    baseline: &'a Notebook,
) -> Vec<(CellChange, &'a Cell)> {
    let baseline_by_id: HashMap<&str, &Cell> = baseline
        .cells
        .iter()
        .map(|cell| (cell.id.as_str(), cell))
        .collect();
    let mut seen: HashMap<&str, ()> = HashMap::new();
    let mut diff = Vec::with_capacity(current.cells.len() + baseline.cells.len());
    for cell in &current.cells {
        seen.insert(cell.id.as_str(), ());
        let change = match baseline_by_id.get(cell.id.as_str()) {
            None => CellChange::Added,
            Some(old) if old.source == cell.source => CellChange::Unchanged,
            Some(_) => CellChange::Modified,
        };
        diff.push((change, cell));
    }
    for cell in &baseline.cells {
        if !seen.contains_key(cell.id.as_str()) {
            diff.push((CellChange::Removed, cell));
        }
    }
    diff
}

/// Render a line-level unified diff for a modified cell's source.
pub fn source_diff(old: &str, new: &str) -> String {
    let mut out = String::new();
    for chunk in dissimilar::diff(old, new) {
        match chunk {
            dissimilar::Chunk::Equal(text) => {
                for line in text.lines() {
                    out.push_str(&format!("  {line}\n"));
                }
            }
            dissimilar::Chunk::Delete(text) => {
                for line in text.lines() {
                    out.push_str(&format!("- {line}\n"));
                }
            }
            dissimilar::Chunk::Insert(text) => {
                for line in text.lines() {
                    out.push_str(&format!("+ {line}\n"));
                }
            }
        }
    }
    out
}

fn git_output(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("git output is not UTF-8: {error}"))
}
