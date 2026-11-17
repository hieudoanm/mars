//! MCP server: JSON-RPC dispatch over a bidirectional byte stream.

use crate::exec::call_exec_tool;
use crate::handlers::call_document_tool;
use crate::jsonrpc::{self, INVALID_PARAMS, METHOD_NOT_FOUND, PARSE_ERROR, Request, Response};
use crate::lab::{self, LabBackend};
use crate::mcp::{self, CallToolResult};
use crate::session::MarsSession;
use crate::tools;
use mars_kernel::RuntimeBackendKind;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

pub struct MarsMcpServer {
    session: MarsSession,
    lab: Option<LabBackend>,
}

impl MarsMcpServer {
    pub fn new(
        backend: RuntimeBackendKind,
        offline: bool,
        default_language: Option<String>,
    ) -> Self {
        Self {
            session: MarsSession::new(backend, offline, default_language),
            lab: None,
        }
    }

    fn lab_backend(&mut self) -> Result<&mut LabBackend, String> {
        if self.lab.is_none() {
            self.lab = Some(LabBackend::new()?);
        }
        self.lab
            .as_mut()
            .ok_or_else(|| "lab backend unavailable".to_owned())
    }

    fn call_lab_tool(&mut self, name: &str, arguments: &Value) -> Option<Result<Value, String>> {
        if !name.starts_with("mars_lab_") {
            return None;
        }
        match self.lab_backend() {
            Ok(backend) => Some(
                lab::call_lab_tool(backend, name, arguments)
                    .unwrap_or_else(|| Err(format!("tool not implemented: {name}"))),
            ),
            Err(error) => Some(Err(error)),
        }
    }

    /// Load a notebook into the session before serving (e.g. `--notebook`).
    pub fn preload_notebook(&mut self, path: &std::path::Path) -> Result<(), String> {
        self.session.open_notebook(path).map(|_| ())
    }

    /// Dispatch one JSON-RPC request. Returns `None` for notifications.
    pub fn handle(&mut self, request: Request) -> Option<Response> {
        let id = request.id.clone().unwrap_or(Value::Null);
        match request.method.as_str() {
            "initialize" => Some(Response::ok(id, mcp::initialize_result())),
            "ping" => Some(Response::ok(id, json!({}))),
            "tools/list" => Some(Response::ok(id, json!({ "tools": tools::definitions() }))),
            "tools/call" => Some(self.call_tool(id, request.params)),
            "prompts/list" => Some(Response::ok(id, prompts_list())),
            "prompts/get" => Some(self.call_prompt(id, request.params)),
            "resources/list" => Some(Response::ok(id, self.resources_list())),
            "resources/read" => Some(self.read_resource(id, request.params)),
            "notifications/initialized" | "notifications/cancelled" => None,
            other => Some(Response::err(
                id,
                METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            )),
        }
    }

    /// Open notebooks exposed as `mars://notebook/<path>` resources.
    fn resources_list(&self) -> Value {
        let resources: Vec<Value> = self
            .session
            .open_paths()
            .iter()
            .map(|path| {
                json!({
                    "uri": format!("mars://notebook/{}", path.display()),
                    "name": path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    "description": format!("Mars notebook {}", path.display()),
                    "mimeType": "application/json",
                })
            })
            .collect();
        json!({ "resources": resources })
    }

    fn read_resource(&mut self, id: Value, params: Option<Value>) -> Response {
        let uri = params
            .as_ref()
            .and_then(|p| p.get("uri"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(path_str) = uri.strip_prefix("mars://notebook/") else {
            return Response::err(id, INVALID_PARAMS, format!("unknown resource: {uri}"));
        };
        let path = std::path::PathBuf::from(path_str);
        let Some(notebook) = self.session.notebook(&path) else {
            return Response::err(
                id,
                INVALID_PARAMS,
                format!("notebook is not open: {}", path.display()),
            );
        };
        match mars_document::persistence::to_json_pretty(notebook) {
            Ok(text) => Response::ok(
                id,
                json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": text,
                    }]
                }),
            ),
            Err(error) => Response::err(
                id,
                INVALID_PARAMS,
                format!("could not serialize notebook: {error}"),
            ),
        }
    }

    fn call_prompt(&mut self, id: Value, params: Option<Value>) -> Response {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let text = match name {
            "notebook_walkthrough" => {
                "Walk me through the notebook currently open in this session.                  For each code cell, explain what it does, what it produces,                  and how the cells connect. Use the mars_* tools to read cell                  contents and outputs before answering."
            }
            "debug_cell" => {
                "A cell in this notebook failed or produced unexpected output.                  Inspect the notebook and kernel state with the mars_* tools,                  identify the failing cell, explain the likely cause, and                  propose a fix."
            }
            _ => return Response::err(id, INVALID_PARAMS, format!("unknown prompt: {name}")),
        };
        Response::ok(
            id,
            json!({
                "description": format!("Mars prompt: {name}"),
                "messages": [
                    { "role": "user", "content": { "type": "text", "text": text } }
                ]
            }),
        )
    }

    fn call_tool(&mut self, id: Value, params: Option<Value>) -> Response {
        let Some(params) = params else {
            return Response::err(id, INVALID_PARAMS, "tools/call requires params");
        };
        let name = params.get("name").and_then(Value::as_str);
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let Some(name) = name else {
            return Response::err(id, INVALID_PARAMS, "tools/call requires a tool name");
        };
        if !tools::definitions().iter().any(|tool| tool.name == name) {
            let result = CallToolResult::error(format!("unknown tool: {name}"));
            return Response::ok(id, serde_json::to_value(result).unwrap_or_default());
        }
        let outcome = call_document_tool(&mut self.session, name, &arguments)
            .or_else(|| call_exec_tool(&mut self.session, name, &arguments))
            .or_else(|| self.call_lab_tool(name, &arguments));
        let result = match outcome {
            Some(Ok(value)) => CallToolResult::ok_json(&value),
            Some(Err(message)) => CallToolResult::error(message),
            None => CallToolResult::error(format!("tool not implemented: {name}")),
        };
        Response::ok(id, serde_json::to_value(result).unwrap_or_default())
    }
}

/// Serve MCP over arbitrary streams until EOF. Used by tests.
pub fn serve<R: BufRead, W: Write>(
    reader: R,
    writer: W,
    server: &mut MarsMcpServer,
) -> io::Result<()> {
    let mut reader = reader;
    let mut writer = writer;
    loop {
        let message = match jsonrpc::read_message(&mut reader) {
            Ok(Some(message)) => message,
            Ok(None) => return Ok(()),
            Err(error) => {
                let response = Response::err(Value::Null, PARSE_ERROR, error.to_string());
                jsonrpc::write_message(&mut writer, &response)?;
                continue;
            }
        };
        let request: Request = match serde_json::from_value(message) {
            Ok(request) => request,
            Err(error) => {
                let response = Response::err(Value::Null, PARSE_ERROR, error.to_string());
                jsonrpc::write_message(&mut writer, &response)?;
                continue;
            }
        };
        if let Some(response) = server.handle(request) {
            jsonrpc::write_message(&mut writer, &response)?;
        }
    }
}

fn prompts_list() -> Value {
    json!({
        "prompts": [
            {
                "name": "notebook_walkthrough",
                "title": "Notebook Walkthrough",
                "description": "Explain every cell in the open notebook and how they connect",
                "arguments": []
            },
            {
                "name": "debug_cell",
                "title": "Debug Cell",
                "description": "Diagnose a failing or surprising cell using kernel state",
                "arguments": []
            }
        ]
    })
}

/// Serve MCP on the process's stdin/stdout. All diagnostics must go to stderr.
pub fn serve_stdio(server: &mut MarsMcpServer) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(stdin.lock(), stdout.lock(), server)
}
