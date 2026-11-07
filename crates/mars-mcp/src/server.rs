//! MCP server: JSON-RPC dispatch over a bidirectional byte stream.

use crate::exec::call_exec_tool;
use crate::handlers::call_document_tool;
use crate::jsonrpc::{self, INVALID_PARAMS, METHOD_NOT_FOUND, PARSE_ERROR, Request, Response};
use crate::mcp::{self, CallToolResult};
use crate::session::MarsSession;
use crate::tools;
use mars_kernel::RuntimeBackendKind;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

pub struct MarsMcpServer {
    session: MarsSession,
}

impl MarsMcpServer {
    pub fn new(
        backend: RuntimeBackendKind,
        offline: bool,
        default_language: Option<String>,
    ) -> Self {
        Self {
            session: MarsSession::new(backend, offline, default_language),
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
            "notifications/initialized" | "notifications/cancelled" => None,
            other => Some(Response::err(
                id,
                METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            )),
        }
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
            .or_else(|| call_exec_tool(&mut self.session, name, &arguments));
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

/// Serve MCP on the process's stdin/stdout. All diagnostics must go to stderr.
pub fn serve_stdio(server: &mut MarsMcpServer) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(stdin.lock(), stdout.lock(), server)
}
