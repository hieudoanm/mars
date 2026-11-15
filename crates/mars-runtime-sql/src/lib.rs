//! `sql` runtime backend — interactive SQL cells over in-memory SQLite.
//!
//! Each session owns one in-memory SQLite database. Tables declared in the
//! notebook's `metadata.sql.tables` map (name → CSV path, resolved relative
//! to the notebook directory by callers) are ingested on first use with
//! per-column type inference. Cells may also create tables, attach database
//! files, and run arbitrary SQLite SQL; a single statement returning rows
//! renders as a table output.

mod session;

pub use session::SqlSession;
