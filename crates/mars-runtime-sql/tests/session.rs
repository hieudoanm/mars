use mars_runtime::{RuntimeBackend, SqlSource};
use mars_runtime_sql::SqlSession;
use std::io::Write;

fn csv_fixture() -> (tempfile_like::TempDir, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("mars-sql-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("data.csv");
    let mut file = std::fs::File::create(&path).expect("csv");
    writeln!(file, "id,name,score").expect("header");
    writeln!(file, "1,alpha,0.5").expect("row");
    writeln!(file, "2,beta,1.5").expect("row");
    writeln!(file, "3,gamma,").expect("row");
    (tempfile_like::TempDir(dir.clone()), path)
}

// Minimal tempdir shim (no tempfile dep in this crate).
mod tempfile_like {
    pub struct TempDir(pub std::path::PathBuf);
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn sql_session_queries_csv_source() {
    let (_dir, path) = csv_fixture();
    let mut session = SqlSession::new(vec![SqlSource {
        name: "scores".to_owned(),
        path,
    }])
    .expect("session");

    let result = session
        .execute_with_control(
            "SELECT name, score FROM scores WHERE id > 1 ORDER BY id",
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .expect("execute");
    assert_eq!(result.execution_count, 1);
    assert_eq!(result.outputs.len(), 1);
    let mars_core::Output::Table { columns, rows } = &result.outputs[0] else {
        panic!("expected table output, got {:?}", result.outputs[0]);
    };
    assert_eq!(columns, &vec!["name".to_owned(), "score".to_owned()]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], "beta");

    // State persists across cells: create a table, insert, query.
    session
        .execute_with_control(
            "CREATE TABLE t (x INTEGER)",
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .expect("create");
    let result = session
        .execute_with_control(
            "INSERT INTO t VALUES (42)",
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .expect("insert");
    assert!(matches!(result.outputs[0], mars_core::Output::Text { .. }));

    // Errors surface as RuntimeError.
    assert!(
        session
            .execute_with_control(
                "SELECT * FROM missing_table",
                None,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .is_err()
    );

    // Reset clears state.
    session.reset().expect("reset");
    assert!(
        session
            .execute_with_control(
                "SELECT * FROM t",
                None,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .is_err()
    );
}

#[test]
fn invalid_table_name_rejected() {
    assert!(
        SqlSession::new(vec![SqlSource {
            name: "bad-name".to_owned(),
            path: std::path::PathBuf::from("x.csv"),
        }])
        .is_err()
    );
}

fn csv_session(name: &str, filename: &str, content: &str) -> (tempfile_like::TempDir, SqlSession) {
    let dir = std::env::temp_dir().join(format!("mars-sql-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(filename);
    std::fs::write(&path, content).expect("csv");
    let session = SqlSession::new(vec![SqlSource {
        name: name.to_owned(),
        path,
    }])
    .expect("session");
    (tempfile_like::TempDir(dir), session)
}

fn exec(session: &mut SqlSession, sql: &str) -> mars_runtime::ExecutionResult {
    session
        .execute_with_control(
            sql,
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .expect("execute")
}

fn table(result: &mars_runtime::ExecutionResult) -> (&[String], &[Vec<String>]) {
    let mars_core::Output::Table { columns, rows } = &result.outputs[0] else {
        panic!("expected table, got {:?}", result.outputs);
    };
    (columns, rows)
}

#[test]
fn csv_ingest_sanitizes_and_dedupes_headers() {
    let (_dir, mut session) = csv_session(
        "weird",
        "w.csv",
        "1st col,na me,na me,9lives,,score\n1,a,2,b,3,0.5\n",
    );
    let result = exec(&mut session, "SELECT * FROM weird");
    let (columns, rows) = table(&result);
    // "1st col"->"st_col", "na me"->"na_me" (dup gets _3), "9lives"->"lives",
    // ""->"column", "score" stays.
    assert_eq!(columns[0], "st_col");
    assert_eq!(columns[1], "na_me");
    assert_eq!(columns[2], "na_me_2");
    assert_eq!(columns[3], "lives");
    assert_eq!(columns[4], "column");
    assert_eq!(columns[5], "score");
    assert_eq!(rows.len(), 1);
}

#[test]
fn csv_missing_file_and_empty_file_error() {
    let mut session = SqlSession::new(vec![SqlSource {
        name: "gone".to_owned(),
        path: std::path::PathBuf::from("/nonexistent/definitely-missing.csv"),
    }])
    .expect("session");
    assert!(
        session
            .execute_with_control(
                "SELECT 1",
                None,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .is_err()
    );

    let dir = std::env::temp_dir().join(format!("mars-sql-{}-emptyfile", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let empty = dir.join("empty.csv");
    std::fs::write(&empty, "").expect("empty csv");
    let mut session = SqlSession::new(vec![SqlSource {
        name: "empty".to_owned(),
        path: empty,
    }])
    .expect("session");
    assert!(
        session
            .execute_with_control(
                "SELECT 1",
                None,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .is_err()
    );
}

#[test]
fn csv_empty_cells_become_null() {
    // The csv reader rejects ragged rows, so padding is defensive; empty
    // fields still exercise the NULL conversion path.
    let (_dir, mut session) = csv_session("ragged", "r.csv", "a,b,c\n1,2,\n3,4,5\n");
    let result = exec(&mut session, "SELECT * FROM ragged ORDER BY a");
    let (_columns, rows) = table(&result);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["1".to_owned(), "2".to_owned(), String::new()]);
    assert_eq!(
        rows[1],
        vec!["3".to_owned(), "4".to_owned(), "5".to_owned()]
    );
}

#[test]
fn sql_value_types_null_and_blob() {
    let (_dir, mut session) = csv_session("t", "t.csv", "x\n1\n");
    let result = exec(
        &mut session,
        "SELECT NULL AS n, 7 AS i, 2.5 AS r, 'txt' AS t, x'0102' AS b",
    );
    let (_columns, rows) = table(&result);
    assert_eq!(
        rows[0],
        vec![
            String::new(),
            "7".to_owned(),
            "2.5".to_owned(),
            "txt".to_owned(),
            "[blob: 2 bytes]".to_owned(),
        ]
    );
}

#[test]
fn semicolons_inside_string_literals_do_not_split() {
    let (_dir, mut session) = csv_session("s", "s.csv", "x\n1\n");
    let result = exec(&mut session, "SELECT 'a;b' AS s; SELECT 2 AS two");
    assert_eq!(result.outputs.len(), 2);
    let (_c1, rows1) = table_at(&result, 0);
    assert_eq!(rows1[0][0], "a;b");
    let (_c2, rows2) = table_at(&result, 1);
    assert_eq!(rows2[0][0], "2");
}

fn table_at(result: &mars_runtime::ExecutionResult, index: usize) -> (&[String], &[Vec<String>]) {
    let mars_core::Output::Table { columns, rows } = &result.outputs[index] else {
        panic!("expected table at {index}, got {:?}", result.outputs);
    };
    (columns, rows)
}

#[test]
fn mixed_statements_accumulate_affected_rows() {
    let (_dir, mut session) = csv_session("src", "m.csv", "x\n1\n");
    let result = exec(
        &mut session,
        "CREATE TABLE tally (x INTEGER); INSERT INTO tally VALUES (1); INSERT INTO tally VALUES (2); SELECT COUNT(*) AS n FROM tally",
    );
    // DDL+2 inserts report 3 affected rows; the select yields a table.
    assert!(
        result
            .outputs
            .iter()
            .any(|o| matches!(o, mars_core::Output::Text { text } if text.contains("3 row(s)")))
    );
    let mars_core::Output::Table { rows, .. } = &result.outputs[0] else {
        panic!("expected table output");
    };
    assert_eq!(rows[0][0], "2");
}

#[test]
fn select_output_truncates_at_max_rows() {
    let (_dir, mut session) = csv_session("big", "b.csv", "x\n1\n");
    let result = exec(
        &mut session,
        "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c WHERE x < 6000) SELECT x FROM c",
    );
    let (_columns, rows) = table(&result);
    assert_eq!(rows.len(), 5_000);
}

#[test]
fn pre_cancelled_token_interrupts_without_executing() {
    let (_dir, mut session) = csv_session("cancel", "c.csv", "x\n1\n");
    let token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let result = session
        .execute_with_control("SELECT 1", None, token)
        .expect("cancelled result");
    assert!(result.interrupted);
    assert!(result.outputs.is_empty());
    assert_eq!(result.execution_count, 0);
}

#[test]
fn empty_and_whitespace_source_increment_count() {
    let (_dir, mut session) = csv_session("empty", "e.csv", "x\n1\n");
    let result = exec(&mut session, "   \n  ");
    assert!(result.outputs.is_empty());
    assert_eq!(result.execution_count, 1);
}

#[test]
fn identifier_validation_edge_cases() {
    for bad in ["1abc", "a-b", "", "tëst", "a b"] {
        assert!(
            SqlSession::new(vec![SqlSource {
                name: bad.to_owned(),
                path: std::path::PathBuf::from("x.csv"),
            }])
            .is_err(),
            "{bad:?} should be rejected"
        );
    }
    for good in ["_ok", "a_b2", "select", "T9"] {
        assert!(
            SqlSession::new(vec![SqlSource {
                name: good.to_owned(),
                path: std::path::PathBuf::from("x.csv"),
            }])
            .is_ok(),
            "{good:?} should be accepted"
        );
    }
}

#[test]
fn reset_reingests_sources_and_clears_count() {
    let (_dir, mut session) = csv_session("reingest", "r.csv", "v\n9\n");
    exec(&mut session, "SELECT * FROM reingest");
    assert_eq!(session.execution_count(), 1);
    assert!(session.cargo_lock().expect("lock").is_none());
    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
    // Connection dropped: next execute rebuilds it and re-ingests CSVs.
    let result = exec(&mut session, "SELECT v FROM reingest");
    let (_c, rows) = table(&result);
    assert_eq!(rows[0][0], "9");
}

#[test]
fn second_statement_error_surfaces_after_first_runs() {
    let (_dir, mut session) = csv_session("partial", "p.csv", "x\n1\n");
    assert!(
        session
            .execute_with_control(
                "CREATE TABLE p (x INTEGER); this is not sql",
                None,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .is_err()
    );
}
