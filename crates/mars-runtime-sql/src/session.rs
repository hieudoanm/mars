//! SQLite-backed `RuntimeBackend` for `sql` cells.

use mars_core::Output;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError, SqlSource};
use rusqlite::{Connection, params_from_iter, types::Value as SqlValue};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Maximum rows returned by a single query cell before truncation.
const MAX_ROWS: usize = 5_000;

pub struct SqlSession {
    sources: Vec<SqlSource>,
    connection: Option<Connection>,
    execution_count: u64,
}

impl SqlSession {
    pub fn new(sources: Vec<SqlSource>) -> Result<Self, RuntimeError> {
        for source in &sources {
            if !is_safe_identifier(&source.name) {
                return Err(RuntimeError::Backend(format!(
                    "invalid SQL table name: {}",
                    source.name
                )));
            }
        }
        Ok(Self {
            sources,
            connection: None,
            execution_count: 0,
        })
    }

    fn connection(&mut self) -> Result<&mut Connection, RuntimeError> {
        if self.connection.is_none() {
            let connection = Connection::open_in_memory()
                .map_err(|error| RuntimeError::Backend(error.to_string()))?;
            connection
                .busy_timeout(Duration::from_secs(5))
                .map_err(|error| RuntimeError::Backend(error.to_string()))?;
            self.connection = Some(connection);
            self.ingest_sources()?;
        }
        self.connection
            .as_mut()
            .ok_or_else(|| RuntimeError::Backend("sqlite connection unavailable".to_owned()))
    }

    /// Loads every declared CSV source as a table. Column types are inferred
    /// from the first rows: all-integer → INTEGER, all-numeric → REAL,
    /// otherwise TEXT.
    fn ingest_sources(&mut self) -> Result<(), RuntimeError> {
        let connection = self
            .connection
            .as_mut()
            .ok_or_else(|| RuntimeError::Backend("sqlite connection unavailable".to_owned()))?;
        for source in &self.sources {
            ingest_csv(connection, &source.name, &source.path)?;
        }
        Ok(())
    }
}

fn is_safe_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn ingest_csv(
    connection: &mut Connection,
    name: &str,
    path: &std::path::Path,
) -> Result<(), RuntimeError> {
    let mut reader = csv::Reader::from_path(path).map_err(|error| {
        RuntimeError::Backend(format!(
            "cannot read CSV source {}: {error}",
            path.display()
        ))
    })?;
    let headers = reader
        .headers()
        .map_err(|error| RuntimeError::Backend(error.to_string()))?
        .iter()
        .map(|header| {
            let cleaned: String = header
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let cleaned = cleaned.trim_start_matches(|c: char| c.is_ascii_digit());
            if cleaned.is_empty() {
                "column".to_owned()
            } else {
                cleaned.to_owned()
            }
        })
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return Err(RuntimeError::Backend(format!(
            "CSV source {} has no header row",
            path.display()
        )));
    }
    // Dedupe column names.
    let mut seen = std::collections::HashSet::new();
    let headers: Vec<String> = headers
        .into_iter()
        .enumerate()
        .map(|(index, header)| {
            if seen.insert(header.clone()) {
                header
            } else {
                format!("{header}_{index}")
            }
        })
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| RuntimeError::Backend(error.to_string()))?;
        rows.push(record.iter().map(str::to_owned).collect());
    }

    // Type inference over all rows (bounded by file size; CSVs are user data).
    let column_types: Vec<&'static str> = (0..headers.len())
        .map(|column| {
            let values = rows
                .iter()
                .filter_map(|row| row.get(column))
                .filter(|value| !value.trim().is_empty());
            let mut saw_value = false;
            let mut all_int = true;
            let mut all_numeric = true;
            for value in values {
                saw_value = true;
                let trimmed = value.trim();
                if trimmed.parse::<i64>().is_err() {
                    all_int = false;
                }
                if trimmed.parse::<f64>().is_err() {
                    all_numeric = false;
                }
                if !all_int && !all_numeric {
                    break;
                }
            }
            if !saw_value || all_int {
                if saw_value { "INTEGER" } else { "TEXT" }
            } else if all_numeric {
                "REAL"
            } else {
                "TEXT"
            }
        })
        .collect();

    let columns_sql = headers
        .iter()
        .zip(&column_types)
        .map(|(header, ty)| format!("{} {}", quote_identifier(header), ty))
        .collect::<Vec<_>>()
        .join(", ");
    connection
        .execute_batch(&format!(
            "CREATE TABLE {} ({columns_sql});",
            quote_identifier(name)
        ))
        .map_err(|error| RuntimeError::Backend(error.to_string()))?;

    if rows.is_empty() {
        return Ok(());
    }
    let placeholders = (0..headers.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({placeholders});",
        quote_identifier(name),
        headers
            .iter()
            .map(|header| quote_identifier(header))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut statement = connection
        .prepare(&insert_sql)
        .map_err(|error| RuntimeError::Backend(error.to_string()))?;
    for row in &mut rows {
        // Pad short rows; extra cells are ignored.
        let values: Vec<SqlValue> = (0..headers.len())
            .map(|index| match row.get(index) {
                Some(value) if value.trim().is_empty() => SqlValue::Null,
                Some(value) => match column_types[index] {
                    "INTEGER" => value
                        .trim()
                        .parse::<i64>()
                        .map(SqlValue::Integer)
                        .unwrap_or_else(|_| SqlValue::Text(value.clone())),
                    "REAL" => value
                        .trim()
                        .parse::<f64>()
                        .map(SqlValue::Real)
                        .unwrap_or_else(|_| SqlValue::Text(value.clone())),
                    _ => SqlValue::Text(value.clone()),
                },
                None => SqlValue::Null,
            })
            .collect();
        statement
            .execute(params_from_iter(values.iter()))
            .map_err(|error| RuntimeError::Backend(error.to_string()))?;
    }
    Ok(())
}

/// Splits cell source into statements on `;` outside quoted strings.
fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in sql.chars() {
        match quote {
            Some(q) => {
                current.push(ch);
                if ch == q {
                    quote = None;
                }
            }
            None if ch == '\'' || ch == '"' || ch == '`' => {
                quote = Some(ch);
                current.push(ch);
            }
            None if ch == ';' => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_owned());
                }
                current.clear();
            }
            None => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_owned());
    }
    statements
}

fn value_to_string(value: SqlValue) -> String {
    match value {
        SqlValue::Null => String::new(),
        SqlValue::Integer(value) => value.to_string(),
        SqlValue::Real(value) => value.to_string(),
        SqlValue::Text(value) => value,
        SqlValue::Blob(blob) => format!("[blob: {} bytes]", blob.len()),
    }
}

impl RuntimeBackend for SqlSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        _timeout: Option<Duration>,
        cancellation: std::sync::Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        let started = Instant::now();
        if cancellation.load(Ordering::Relaxed) {
            return Ok(ExecutionResult {
                outputs: vec![],
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: true,
            });
        }
        let sql = source.trim();
        let mut outputs = Vec::new();
        if !sql.is_empty() {
            let connection = self.connection()?;
            let mut affected_rows = 0usize;
            for statement_sql in split_statements(sql) {
                let mut statement = connection
                    .prepare(&statement_sql)
                    .map_err(|error| RuntimeError::Backend(error.to_string()))?;
                let column_count = statement.column_count();
                if column_count > 0 {
                    let columns = statement
                        .column_names()
                        .iter()
                        .map(|name| (*name).to_owned())
                        .collect::<Vec<_>>();
                    let mut rows = Vec::new();
                    let mut query = statement
                        .query([])
                        .map_err(|error| RuntimeError::Backend(error.to_string()))?;
                    while let Some(row) = query
                        .next()
                        .map_err(|error| RuntimeError::Backend(error.to_string()))?
                    {
                        if rows.len() >= MAX_ROWS {
                            break;
                        }
                        let mut values = Vec::with_capacity(column_count);
                        for index in 0..column_count {
                            values.push(value_to_string(
                                row.get::<_, SqlValue>(index)
                                    .map_err(|error| RuntimeError::Backend(error.to_string()))?,
                            ));
                        }
                        rows.push(values);
                    }
                    drop(query);
                    outputs.push(Output::Table { columns, rows });
                } else {
                    affected_rows += statement
                        .execute([])
                        .map_err(|error| RuntimeError::Backend(error.to_string()))?;
                }
            }
            if affected_rows > 0 {
                outputs.push(Output::Text {
                    text: format!("{affected_rows} row(s) affected"),
                });
            }
            if outputs.is_empty() {
                outputs.push(Output::Text {
                    text: "ok".to_owned(),
                });
            }
        }
        self.execution_count += 1;
        Ok(ExecutionResult {
            outputs,
            duration: started.elapsed(),
            execution_count: self.execution_count,
            interrupted: false,
        })
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.connection = None;
        self.execution_count = 0;
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        self.execution_count
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        Ok(None)
    }
}
