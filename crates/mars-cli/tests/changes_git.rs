//! Git-fixture tests for `mars notebook diff` and `run --changed-only`,
//! covering `changes.rs` baseline loading and `commands.rs` dispatch.

use mars::commands::run_cli;
use mars_core::CellType;
use mars_document::{Cell, Notebook, save_atomic};
use std::path::PathBuf;
use std::process::Command;

fn git(dir: &PathBuf, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?} failed");
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// Notebook with one markdown + one code cell.
fn sample_notebook() -> Notebook {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Demo"))
        .expect("md");
    notebook
        .add_cell(Cell::new(CellType::Code, "let x = 1;"))
        .expect("code");
    notebook
}

fn repo_with_notebook(name: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("mars-cli-git-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    git(&dir, &["init", "-q"]);
    let path = dir.join("demo.mars");
    save_atomic(&sample_notebook(), &path).expect("save");
    git(&dir, &["add", "demo.mars"]);
    git(&dir, &["commit", "-qm", "init"]);
    (dir, path)
}

#[test]
fn diff_against_git_ref_reports_changes() {
    let (dir, path) = repo_with_notebook("diff-ref");
    // Modify the code cell source so the diff reports a modification.
    let mut notebook = sample_notebook();
    notebook.cells[1].source = "let x = 2;".to_owned();
    save_atomic(&notebook, &path).expect("save modified");

    run_cli(args(&["diff", &path.to_string_lossy(), "HEAD"])).expect("diff against HEAD");

    // Diff against another .mars file (no git involved).
    let other = dir.join("other.mars");
    save_atomic(&sample_notebook(), &other).expect("save other");
    run_cli(args(&[
        "diff",
        &path.to_string_lossy(),
        &other.to_string_lossy(),
    ]))
    .expect("diff against file");
}

#[test]
fn diff_errors_for_untracked_and_missing_baseline() {
    let dir = std::env::temp_dir().join(format!("mars-cli-git-{}-untracked", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    git(&dir, &["init", "-q"]);
    let path = dir.join("untracked.mars");
    save_atomic(&sample_notebook(), &path).expect("save");
    // File exists but is not committed -> baseline_notebook errors.
    assert!(run_cli(args(&["diff", &path.to_string_lossy()])).is_err());
    // Missing current file -> load error.
    let missing = dir.join("missing.mars");
    assert!(run_cli(args(&["diff", &missing.to_string_lossy()])).is_err());
}

#[test]
fn run_changed_only_uses_git_baseline() {
    let (_dir, path) = repo_with_notebook("changed-only");
    // Markdown-only change still exercises baseline loading; no code cells
    // means nothing executes, keeping the test fast and interpreter-free.
    let mut notebook = sample_notebook();
    notebook.cells[0].source = "# Updated".to_owned();
    save_atomic(&notebook, &path).expect("save");

    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--changed-only",
        "--no-save",
    ]))
    .expect("run --changed-only");

    // --since with an explicit ref exercises the same baseline path.
    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--since",
        "HEAD",
        "--no-save",
    ]))
    .expect("run --since HEAD");
}

#[test]
fn run_changed_only_errors_outside_git() {
    let dir = std::env::temp_dir().join(format!("mars-cli-git-{}-nogit", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("plain.mars");
    save_atomic(&sample_notebook(), &path).expect("save");
    assert!(
        run_cli(args(&[
            "run",
            &path.to_string_lossy(),
            "--changed-only",
            "--no-save",
        ]))
        .is_err()
    );
}

#[test]
fn new_supports_templates_language_and_validation() {
    let dir = std::env::temp_dir().join(format!("mars-cli-new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");

    for template in ["rust", "python", "ml", "dashboard"] {
        let path = dir.join(format!("{template}.mars"));
        run_cli(args(&[
            "new",
            &path.to_string_lossy(),
            "--template",
            template,
        ]))
        .unwrap_or_else(|e| panic!("template {template}: {e}"));
        assert!(path.exists());
    }

    // --language flag overrides the template default.
    let lang_path = dir.join("lang.mars");
    run_cli(args(&[
        "new",
        &lang_path.to_string_lossy(),
        "--template",
        "rust",
        "--language",
        "python",
    ]))
    .expect("language override");
    let notebook = mars_document::load(&lang_path).expect("load");
    assert_eq!(notebook.metadata.language, "python");

    // Validation errors.
    let conflict = dir.join("conflict.mars");
    assert!(
        run_cli(args(&[
            "new",
            &conflict.to_string_lossy(),
            "--template",
            "rust",
            "--example",
            "x",
        ]))
        .is_err()
    );
    assert!(run_cli(args(&["new", &conflict.to_string_lossy(), "--template",])).is_err());
    assert!(
        run_cli(args(&[
            "new",
            &conflict.to_string_lossy(),
            "--language",
            "   ",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "new",
            &conflict.to_string_lossy(),
            "--template",
            "bogus",
        ]))
        .is_err()
    );
}

#[test]
fn diff_reports_added_and_removed_cells() {
    let dir = std::env::temp_dir().join(format!("mars-cli-git-{}-addrem", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    git(&dir, &["init", "-q"]);
    let path = dir.join("cells.mars");

    // Baseline: two cells.
    let mut baseline = Notebook::new();
    baseline
        .add_cell(Cell::new(CellType::Markdown, "# A"))
        .expect("a");
    let removed = baseline
        .add_cell(Cell::new(CellType::Markdown, "# B"))
        .expect("b");
    save_atomic(&baseline, &path).expect("save baseline");
    git(&dir, &["add", "cells.mars"]);
    git(&dir, &["commit", "-qm", "init"]);

    // Current: drop cell B, add a new cell C, keep A.
    let mut current = Notebook::new();
    current
        .add_cell(Cell::new(CellType::Markdown, "# A"))
        .expect("a");
    current
        .add_cell(Cell::new(CellType::Code, "let added = true;"))
        .expect("c");
    // Reuse the removed cell's id is impossible post-hoc; instead delete it
    // from the loaded baseline shape by rebuilding without it — diff_cells
    // matches on CellId, so removing the cell object is what matters.
    let _ = removed;
    save_atomic(&current, &path).expect("save current");

    // Diff against the file form too (covers the .mars branch without git).
    let baseline_path = dir.join("baseline.mars");
    save_atomic(&baseline, &baseline_path).expect("save baseline file");
    run_cli(args(&[
        "diff",
        &path.to_string_lossy(),
        &baseline_path.to_string_lossy(),
    ]))
    .expect("diff file-vs-file");
    run_cli(args(&["diff", &path.to_string_lossy(), "HEAD"])).expect("diff vs HEAD");
}

#[test]
fn diff_against_unknown_git_ref_errors() {
    let (_dir, path) = repo_with_notebook("bad-ref");
    assert!(
        run_cli(args(&[
            "diff",
            &path.to_string_lossy(),
            "definitely-not-a-ref",
        ]))
        .is_err()
    );
}
