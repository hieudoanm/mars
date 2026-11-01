//! Minimal Language Server Protocol client for rust-analyzer.
//!
//! Mars notebooks are not Cargo crates, so each session gets a scratch
//! project (`Cargo.toml` + `src/main.rs`) that the client keeps in sync.
//! The client speaks JSON-RPC over stdio with `Content-Length` framing and
//! supports initialize/shutdown, full-document sync, hover, completion, and
//! `publishDiagnostics` notifications.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod session;

pub use session::{
    CellDiagnostic, CellInlayHint, CellSemanticToken, CellSource, CellTextEdit, DefinitionTarget,
    Dependencies, LspCodeAction, LspSession,
};

use serde_json::{Value, json};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("rust-analyzer binary not found")]
    BinaryNotFound,
    #[error("failed to spawn rust-analyzer: {0}")]
    Spawn(String),
    #[error("rust-analyzer exited during handshake")]
    Exited,
    #[error("request timed out: {0}")]
    Timeout(String),
    #[error("server returned error for {0}: {1}")]
    Rpc(String, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

struct Pending {
    sender: Sender<Result<Value, String>>,
}

struct Shared {
    next_id: AtomicI64,
    pending: Mutex<HashMap<i64, Pending>>,
    diagnostics: Mutex<Vec<Value>>,
    latest_diagnostics: Mutex<Vec<Value>>,
    stdin: Mutex<ChildStdin>,
}

/// Handle to a running rust-analyzer process bound to one scratch project.
pub struct LspClient {
    child: Child,
    shared: Arc<Shared>,
    root: PathBuf,
    document_uri: String,
    opened: bool,
}

fn document_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

impl LspClient {
    /// Spawn `rust-analyzer` rooted at `root` (must contain Cargo.toml and
    /// `src/main.rs`) and perform the LSP initialize handshake.
    pub fn spawn(root: &Path) -> Result<Self, LspError> {
        let program =
            std::env::var("MARS_RUST_ANALYZER").unwrap_or_else(|_| "rust-analyzer".to_owned());
        let mut command = Command::new(program);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn().map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                LspError::BinaryNotFound
            } else {
                LspError::Spawn(error.to_string())
            }
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::Spawn("no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::Spawn("no stdout".into()))?;
        let shared = Arc::new(Shared {
            next_id: AtomicI64::new(1),
            pending: Mutex::new(HashMap::new()),
            diagnostics: Mutex::new(Vec::new()),
            latest_diagnostics: Mutex::new(Vec::new()),
            stdin: Mutex::new(stdin),
        });

        let reader_shared = Arc::clone(&shared);
        thread::spawn(move || reader_loop(stdout, reader_shared));

        let mut client = Self {
            child,
            shared,
            root: root.to_path_buf(),
            document_uri: String::new(),
            opened: false,
        };
        client.initialize()?;
        Ok(client)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn document_uri(&self) -> &str {
        &self.document_uri
    }

    fn send(&self, message: &Value) -> Result<(), LspError> {
        let body = serde_json::to_string(message).map_err(|e| LspError::Spawn(e.to_string()))?;
        let mut stdin = self
            .shared
            .stdin
            .lock()
            .map_err(|_| LspError::Spawn("stdin lock poisoned".into()))?;
        write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        stdin.flush()?;
        Ok(())
    }

    fn request(&self, method: &str, params: Value) -> Result<Value, LspError> {
        let id = self.shared.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel();
        self.shared
            .pending
            .lock()
            .map_err(|_| LspError::Spawn("pending lock poisoned".into()))?
            .insert(id, Pending { sender: tx });
        let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(error) = self.send(&message) {
            self.shared.pending.lock().ok().map(|mut p| p.remove(&id));
            return Err(error);
        }
        match rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(message)) => Err(LspError::Rpc(method.to_owned(), message)),
            Err(_) => {
                self.shared.pending.lock().ok().map(|mut p| p.remove(&id));
                Err(LspError::Timeout(method.to_owned()))
            }
        }
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), LspError> {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    /// Legend sent to the server; token type indices in semantic token data
    /// refer to this ordered list.
    pub const SEMANTIC_TOKEN_TYPES: &[&str] = &[
        "namespace",
        "type",
        "class",
        "enum",
        "interface",
        "struct",
        "typeParameter",
        "parameter",
        "variable",
        "property",
        "enumMember",
        "event",
        "function",
        "method",
        "macro",
        "keyword",
        "modifier",
        "comment",
        "string",
        "number",
        "regexp",
        "operator",
        "decorator",
    ];

    fn initialize(&mut self) -> Result<(), LspError> {
        let root_uri = document_uri(&self.root);
        let result = self.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "synchronization": { "didSave": false },
                        "hover": { "contentFormat": ["markdown", "plaintext"] },
                        "completion": { "completionItem": { "snippetSupport": false } },
                        "semanticTokens": {
                            "requests": { "full": true },
                            "tokenTypes": Self::SEMANTIC_TOKEN_TYPES,
                            "tokenModifiers": [],
                            "formats": ["relative"]
                        },
                        "codeAction": {
                            "codeActionLiteralSupport": {
                                "codeActionKind": {
                                    "valueSet": [
                                        "quickfix",
                                        "refactor",
                                        "refactor.extract",
                                        "refactor.inline",
                                        "refactor.rewrite",
                                        "source",
                                        "source.organizeImports"
                                    ]
                                }
                            }
                        }
                    }
                },
                "initializationOptions": { "disableBuildScripts": true, "disableProcMacros": true }
            }),
        )?;
        self.document_uri = format!("{root_uri}/src/main.rs");
        let _ = result;
        self.notify("initialized", json!({}))?;
        Ok(())
    }

    /// Replace the document content (full sync). First call sends `didOpen`,
    /// subsequent calls send `didChange`.
    pub fn sync_document(&mut self, text: &str, version: i64) -> Result<(), LspError> {
        if !self.opened {
            self.notify(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": self.document_uri,
                        "languageId": "rust",
                        "version": version,
                        "text": text
                    }
                }),
            )?;
            self.opened = true;
        } else {
            self.notify(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": self.document_uri, "version": version },
                    "contentChanges": [{ "text": text }]
                }),
            )?;
        }
        Ok(())
    }

    /// Clone of the most recently published diagnostics without draining the
    /// pending queue used for cell-mapped reporting.
    pub fn peek_diagnostics(&self) -> Vec<Value> {
        self.shared
            .latest_diagnostics
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Latest diagnostics published for the document (LSP Diagnostic[]).
    pub fn take_diagnostics(&self) -> Vec<Value> {
        self.shared
            .diagnostics
            .lock()
            .map(|mut guard| std::mem::take(&mut *guard))
            .unwrap_or_default()
    }

    /// Request inlay hints for the whole document. `line_count` bounds the
    /// requested range; rust-analyzer returns hints with document positions.
    pub fn inlay_hints(&self, line_count: u32) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/inlayHint",
            json!({
                "textDocument": { "uri": self.document_uri },
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": line_count, "character": 0 }
                }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    /// Request full-document semantic tokens (LSP delta-encoded u32 array).
    pub fn semantic_tokens(&self) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": self.document_uri } }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    pub fn hover(&self, line: u32, character: u32) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": self.document_uri },
                "position": { "line": line, "character": character }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    pub fn completion(&self, line: u32, character: u32) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": self.document_uri },
                "position": { "line": line, "character": character }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    pub fn signature_help(&self, line: u32, character: u32) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/signatureHelp",
            json!({
                "textDocument": { "uri": self.document_uri },
                "position": { "line": line, "character": character }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    pub fn definition(&self, line: u32, character: u32) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": self.document_uri },
                "position": { "line": line, "character": character }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }

    /// Request code actions (quick fixes / assists) for a range. `diagnostics`
    /// should be the raw LSP Diagnostic[] overlapping the range.
    pub fn code_action(
        &self,
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
        diagnostics: &[Value],
    ) -> Result<Option<Value>, LspError> {
        let result = self.request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": self.document_uri },
                "range": {
                    "start": { "line": start_line, "character": start_character },
                    "end": { "line": end_line, "character": end_character }
                },
                "context": { "diagnostics": diagnostics }
            }),
        )?;
        Ok(if result.is_null() { None } else { Some(result) })
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.notify("exit", json!({}));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn reader_loop(stdout: std::process::ChildStdout, shared: Arc<Shared>) {
    let mut reader = BufReader::new(stdout);
    loop {
        match read_message(&mut reader) {
            Ok(Some(message)) => dispatch(message, &shared),
            Ok(None) => break,
            Err(_) => break,
        }
    }
    if let Ok(mut pending) = shared.pending.lock() {
        for (_, entry) in pending.drain() {
            let _ = entry.sender.send(Err("server closed".to_owned()));
        }
    }
}

#[doc(hidden)]
pub fn read_message(reader: &mut impl BufRead) -> std::io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }
    let Some(length) = content_length else {
        return Ok(None);
    };
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn dispatch(message: Value, shared: &Arc<Shared>) {
    if let Some(id) = message.get("id").and_then(Value::as_i64) {
        let entry = shared.pending.lock().ok().and_then(|mut p| p.remove(&id));
        if let Some(entry) = entry {
            let result = if let Some(error) = message.get("error") {
                Err(error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
                    .to_owned())
            } else {
                Ok(message.get("result").cloned().unwrap_or(Value::Null))
            };
            let _ = entry.sender.send(result);
            return;
        }
        // Server-initiated request (has id + method but no pending entry):
        // answer with null so rust-analyzer does not stall.
        if message.get("method").is_some() {
            if let Ok(mut stdin) = shared.stdin.lock() {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": Value::Null
                });
                if let Ok(body) = serde_json::to_string(&response) {
                    let _ = write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body);
                    let _ = stdin.flush();
                }
            }
        }
        return;
    }
    if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
        if let Some(params) = message.get("params") {
            if let Some(diagnostics) = params.get("diagnostics").and_then(Value::as_array) {
                if let Ok(mut guard) = shared.diagnostics.lock() {
                    *guard = diagnostics.clone();
                }
                if let Ok(mut guard) = shared.latest_diagnostics.lock() {
                    *guard = diagnostics.clone();
                }
            }
        }
    }
}
