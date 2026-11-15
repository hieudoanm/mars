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
