//! MCP client: spawns an MCP server process and speaks JSON-RPC over its
//! stdin/stdout. Notifications and server-initiated requests are ignored.

use crate::jsonrpc::{self, Request, Response};
use crate::mcp::{CallToolResult, Tool};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpClient {
    /// Spawn `command` (program plus arguments) as an MCP stdio server.
    pub fn spawn(command: &[String]) -> Result<Self, String> {
        let (program, args) = command
            .split_first()
            .ok_or_else(|| "empty server command".to_owned())?;
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("failed to spawn {program}: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "server stdin unavailable".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "server stdout unavailable".to_owned())?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    /// Perform the MCP `initialize` handshake.
    pub fn initialize(&mut self) -> Result<(), String> {
        self.request(
            "initialize",
            json!({
                "protocolVersion": crate::mcp::PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": { "name": "mars-cli", "version": env!("CARGO_PKG_VERSION") },
            }),
        )?;
        self.notify("notifications/initialized", json!({}))
    }

    pub fn list_tools(&mut self) -> Result<Vec<Tool>, String> {
        let result = self.request("tools/list", json!({}))?;
        serde_json::from_value(result["tools"].clone())
            .map_err(|error| format!("invalid tools/list result: {error}"))
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<CallToolResult, String> {
        let result = self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )?;
        serde_json::from_value(result)
            .map_err(|error| format!("invalid tools/call result: {error}"))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        jsonrpc::write_message(&mut self.stdin, &message)
            .map_err(|error| format!("failed to write notification: {error}"))
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let message = Request {
            id: Some(json!(id)),
            method: method.to_owned(),
            params: Some(params),
        };
        jsonrpc::write_message(&mut self.stdin, &message)
            .map_err(|error| format!("failed to write request: {error}"))?;
        loop {
            let line = {
                let mut line = String::new();
                self.stdout
                    .read_line(&mut line)
                    .map_err(|error| format!("failed to read response: {error}"))?;
                line
            };
            if line.trim().is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line.trim())
                .map_err(|error| format!("invalid JSON from server: {error}"))?;
            // Skip notifications and responses for other ids.
            let response_id = value.get("id").cloned().unwrap_or(Value::Null);
            if response_id != json!(id) {
                continue;
            }
            let response: Response = serde_json::from_value(value)
                .map_err(|error| format!("invalid response envelope: {error}"))?;
            if let Some(error) = response.error {
                return Err(format!("{} (code {})", error.message, error.code));
            }
            return Ok(response.result.unwrap_or(Value::Null));
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_handshake_against_test_server() {
        // In-process smoke test: run the real server on pipes via threads is
        // covered by server tests; here we only verify spawn plumbing fails
        // gracefully for a missing binary.
        let client = McpClient::spawn(&["definitely-not-a-real-binary-xyz".to_owned()]);
        assert!(client.is_err());
    }
}
