//! `mars mcp serve` (MCP server) and `mars tools` (MCP client commands).

use mars_mcp::{MarsMcpServer, McpClient};
use serde_json::Value;
use std::path::Path;

pub fn serve_mcp(args: &[String]) -> Result<(), String> {
    let options = mars::options::parse_mcp_options(args)?;
    let mut server = MarsMcpServer::new(options.backend, options.offline, options.language);
    if let Some(notebook) = &options.notebook {
        server
            .preload_notebook(Path::new(notebook))
            .map_err(|error| format!("failed to open notebook {notebook}: {error}"))?;
    }
    if let Some(addr) = options.http {
        return mars_mcp::serve_http(
            &addr,
            options
                .token
                .or_else(|| std::env::var("MCP_AUTH_TOKEN").ok()),
            &mut server,
        )
        .map_err(crate::util::error_string);
    }
    mars_mcp::serve_stdio(&mut server).map_err(crate::util::error_string)
}

/// `mars tools list|call` — client-side MCP commands.
pub fn run_tools(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(
            "usage: mars tools list | mars tools call <name> [json-args] [--server <cmd>]"
                .to_owned(),
        );
    };
    let (server_args, rest) = split_server_arg(&args[1..])?;
    let mut client = connect(&server_args)?;
    match command {
        "list" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            let tools = client.list_tools()?;
            for tool in &tools {
                println!(
                    "{} — {}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("")
                );
            }
            Ok(())
        }
        "call" => {
            let name = rest.first().ok_or_else(|| "missing tool name".to_owned())?;
            let arguments: Value = match rest.get(1) {
                Some(raw) => serde_json::from_str(raw)
                    .map_err(|error| format!("invalid JSON arguments: {error}"))?,
                None => serde_json::json!({}),
            };
            if rest.len() > 2 {
                return Err(format!("unexpected arguments: {}", rest[2..].join(" ")));
            }
            let result = client.call_tool(name, arguments)?;
            for content in &result.content {
                println!("{}", content.text);
            }
            if result.is_error == Some(true) {
                return Err(format!("tool {name} reported an error"));
            }
            Ok(())
        }
        other => Err(format!("unknown tools command: {other}")),
    }
}

/// Extract `--server <command string>`; everything else is positional.
fn split_server_arg(args: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    let mut server: Option<String> = None;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--server" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "missing value for --server".to_owned())?;
            server = Some(value.clone());
            index += 2;
        } else {
            rest.push(args[index].clone());
            index += 1;
        }
    }
    let command = match server {
        Some(command) => command
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        None => default_server_command()?,
    };
    if command.is_empty() {
        return Err("empty --server command".to_owned());
    }
    Ok((command, rest))
}

/// Default MCP server: this same `mars` binary running `mcp serve`.
fn default_server_command() -> Result<Vec<String>, String> {
    let exe = std::env::current_exe().map_err(crate::util::error_string)?;
    Ok(vec![
        exe.to_string_lossy().into_owned(),
        "mcp".to_owned(),
        "serve".to_owned(),
    ])
}

fn connect(command: &[String]) -> Result<McpClient, String> {
    let mut client = McpClient::spawn(command)?;
    client.initialize()?;
    Ok(client)
}
