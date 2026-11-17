//! Integration tests for the Streamable-HTTP MCP transport (`serve_http`).
//! Each test spawns a real server on an ephemeral port and speaks raw
//! HTTP/1.1 over a TcpStream.

use mars_kernel::RuntimeBackendKind;
use mars_mcp::{MarsMcpServer, serve_http};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// Spawn `serve_http` on a free port and return the port once it accepts.
fn serve(token: Option<String>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind probe");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);

    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    thread::spawn(move || {
        let addr = format!("127.0.0.1:{port}");
        let _ = serve_http(&addr, token, &mut server);
    });

    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return port;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("mcp http server did not start on port {port}");
}

/// Send a raw request and read the whole response (server closes the
/// connection after each reply).
fn request(port: u16, raw: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let _ = stream.write_all(raw.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    stream.read_to_string(&mut out).expect("read response");
    out
}

fn post(port: u16, body: &str) -> String {
    post_with_auth(port, "Bearer t", body)
}

fn post_with_auth(port: u16, auth: &str, body: &str) -> String {
    request(
        port,
        &format!(
            "POST /mcp HTTP/1.1\r\nauthorization: {auth}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            body.len(),
            body
        ),
    )
}

fn status_of(response: &str) -> u16 {
    response
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("status line")
}

#[test]
fn serves_jsonrpc_and_accepts_notifications() {
    let port = serve(None);

    let init = post(
        port,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );
    assert_eq!(status_of(&init), 200);
    assert!(init.contains("application/json"));
    assert!(init.contains("\"result\""));

    let notify = post(
        port,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(status_of(&notify), 202);
}

#[test]
fn rejects_wrong_path_method_and_batch() {
    let port = serve(None);

    let get = request(port, "GET /mcp HTTP/1.1\r\n\r\n");
    assert_eq!(status_of(&get), 405);

    let wrong = request(port, "POST /other HTTP/1.1\r\ncontent-length: 2\r\n\r\n{}");
    assert_eq!(status_of(&wrong), 404);

    let batch = post(port, r#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#);
    assert_eq!(status_of(&batch), 400);
    assert!(batch.contains("batch requests are not supported"));

    // Malformed JSON still gets a JSON-RPC parse error response.
    let bad = post(port, "{not json");
    assert_eq!(status_of(&bad), 200);
    assert!(bad.contains("-32700"));
}

#[test]
fn enforces_bearer_token_when_configured() {
    let port = serve(Some("s3cret".to_owned()));

    let anonymous = request(port, "POST /mcp HTTP/1.1\r\ncontent-length: 2\r\n\r\n{}");
    assert_eq!(status_of(&anonymous), 401);

    let wrong = request(
        port,
        "POST /mcp HTTP/1.1\r\nauthorization: Bearer nope\r\ncontent-length: 2\r\n\r\n{}",
    );
    assert_eq!(status_of(&wrong), 401);

    // `post` sends `authorization: Bearer t`, which must be rejected too.
    let bad_token = post(port, r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
    assert_eq!(status_of(&bad_token), 401);

    let good = post_with_auth(
        port,
        "Bearer s3cret",
        r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
    );
    assert_eq!(status_of(&good), 200);
    assert!(good.contains("\"result\""));
}

#[test]
fn rejects_oversized_headers_and_body() {
    let port = serve(None);

    // Headers beyond MAX_HEADER_BYTES (64 KiB) get a 431.
    let big_header = format!(
        "POST /mcp HTTP/1.1\r\nx-pad: {}\r\ncontent-length: 2\r\n\r\n{{}}",
        "a".repeat(70 * 1024)
    );
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let _ = stream.write_all(big_header.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    stream.read_to_string(&mut out).expect("read 431");
    assert_eq!(status_of(&out), 431);

    // Content-Length beyond MAX_BODY_BYTES (8 MiB) gets a 413.
    let big_body = request(
        port,
        "POST /mcp HTTP/1.1\r\ncontent-length: 9000000\r\n\r\n{}",
    );
    assert_eq!(status_of(&big_body), 413);
}
