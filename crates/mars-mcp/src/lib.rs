//! Mars MCP integration: expose notebook/kernel operations as MCP tools and
//! provide a client for calling MCP servers.
//!
//! Two directions are supported:
//!
//! - **Server** — [`serve_stdio`] runs an MCP server on stdin/stdout
//!   (`mars mcp serve`), letting MCP clients (Claude Desktop, IDEs, agents)
//!   inspect, edit, execute, and save `.mars` notebooks.
//! - **Client** — [`client::McpClient`] spawns an MCP server process and
//!   performs the `initialize` / `tools/list` / `tools/call` handshake
//!   (`mars tools list|call`).
//!
//! Transport is newline-delimited JSON-RPC 2.0, the same framing as
//! `mars kernel serve`. All diagnostics must go to stderr so stdout stays
//! clean for protocol traffic.

pub mod client;
pub mod exec;
pub mod handlers;
pub mod jsonrpc;
pub mod mcp;
pub mod server;
pub mod session;
pub mod tools;

pub use client::McpClient;
pub use server::{MarsMcpServer, serve, serve_stdio};
pub use session::MarsSession;
