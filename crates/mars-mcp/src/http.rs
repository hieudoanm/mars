//! Minimal Streamable-HTTP transport for the Mars MCP server.
//!
//! `POST /mcp` carries one JSON-RPC message and gets an `application/json`
//! reply; notifications get `202 Accepted`. `GET`/`DELETE` return `405` —
//! this server never initiates messages, which the spec permits. Connections
//! are handled sequentially on the calling thread so the server can hold
//! non-`Send` state (kernel sessions, lab backend).
//!
//! Auth: set `MCP_AUTH_TOKEN` (or pass `--token`) to require
//! `Authorization: Bearer <token>` on every request.

use crate::jsonrpc::{PARSE_ERROR, Request, Response};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;

const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Serve `server` over HTTP/1.1 on `addr` until the process exits.
pub fn serve_http(
    addr: &str,
    token: Option<String>,
    server: &mut crate::server::MarsMcpServer,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    eprintln!("[mcp] streamable-http listening on http://{addr}/mcp");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = handle_connection(&mut stream, token.as_deref(), server);
            }
            Err(err) => eprintln!("[mcp] accept failed: {err}"),
        }
    }
    Ok(())
}

fn dispatch(server: &mut crate::server::MarsMcpServer, body: &str) -> Option<String> {
    let request: Request = match serde_json::from_str(body) {
        Ok(request) => request,
        Err(error) => {
            let response = Response::err(Value::Null, PARSE_ERROR, error.to_string());
            return serde_json::to_string(&response).ok();
        }
    };
    server
        .handle(request)
        .and_then(|response| serde_json::to_string(&response).ok())
}

fn handle_connection(
    stream: &mut std::net::TcpStream,
    token: Option<&str>,
    server: &mut crate::server::MarsMcpServer,
) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 8192];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() > MAX_HEADER_BYTES {
            return respond(
                stream,
                431,
                "text/plain",
                b"request headers too large",
                None,
            );
        }
        if let Some(pos) = find(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let mut lines = head.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    let mut content_length = 0usize;
    let mut authorized = token.is_none();
    let mut protocol_version = crate::mcp::PROTOCOL_VERSION.to_string();
    for line in lines {
        let line = line.trim_end_matches('\r');
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim();
            match name.as_str() {
                "content-length" => content_length = value.parse().unwrap_or(0),
                "authorization" => {
                    if let Some(expected) = token {
                        authorized = value == format!("Bearer {expected}");
                    }
                }
                "mcp-protocol-version" => protocol_version = value.to_string(),
                _ => {}
            }
        }
    }

    if !authorized {
        drain_body(stream, &mut buf, header_end, content_length)?;
        return respond(
            stream,
            401,
            "application/json",
            br#"{"error":"unauthorized"}"#,
            None,
        );
    }
    if path != "/mcp" {
        drain_body(stream, &mut buf, header_end, content_length)?;
        return respond(
            stream,
            404,
            "application/json",
            br#"{"error":"not found"}"#,
            None,
        );
    }
    if method != "POST" {
        drain_body(stream, &mut buf, header_end, content_length)?;
        return respond(
            stream,
            405,
            "application/json",
            br#"{"error":"method not allowed"}"#,
            None,
        );
    }
    if content_length > MAX_BODY_BYTES {
        return respond(
            stream,
            413,
            "application/json",
            br#"{"error":"body too large"}"#,
            None,
        );
    }

    drain_body(stream, &mut buf, header_end, content_length)?;
    let end = (header_end + content_length).min(buf.len());
    let body_str = String::from_utf8_lossy(&buf[header_end..end]).into_owned();

    // JSON-RPC batching was removed from the spec; reject arrays.
    if body_str.trim_start().starts_with('[') {
        let msg = br#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"batch requests are not supported"}}"#;
        return respond(
            stream,
            400,
            "application/json",
            msg,
            Some(&protocol_version),
        );
    }

    match dispatch(server, &body_str) {
        Some(body) => respond(
            stream,
            200,
            "application/json",
            body.as_bytes(),
            Some(&protocol_version),
        ),
        None => respond(
            stream,
            202,
            "application/json",
            b"",
            Some(&protocol_version),
        ),
    }
}

/// Read and discard the request body (bounded by `MAX_BODY_BYTES`) so the
/// socket has no unread bytes when it closes early — otherwise the kernel
/// sends RST and the client can lose the response.
fn drain_body(
    stream: &mut std::net::TcpStream,
    buf: &mut Vec<u8>,
    header_end: usize,
    content_length: usize,
) -> std::io::Result<()> {
    let want = header_end + content_length.min(MAX_BODY_BYTES);
    while buf.len() < want {
        let mut chunk = [0u8; 8192];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Ok(())
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn respond(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    protocol_version: Option<&str>,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    };
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n",
        body.len()
    );
    if let Some(version) = protocol_version {
        head.push_str(&format!("mcp-protocol-version: {version}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}
