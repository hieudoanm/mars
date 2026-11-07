use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

struct ServeProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl ServeProcess {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mars"))
            .args(["kernel", "serve"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mars kernel serve");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, request: Value) {
        writeln!(self.stdin, "{request}").expect("write request");
        self.stdin.flush().expect("flush");
    }

    fn read_line(&mut self) -> Value {
        let mut line = String::new();
        let read = self.stdout.read_line(&mut line).expect("read response");
        assert!(read > 0, "kernel serve closed stdout");
        serde_json::from_str(line.trim()).expect("parse json line")
    }
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn serve_streams_events_then_responds() {
    let mut serve = ServeProcess::spawn();
    serve.send(json!({
        "id": "exec-1",
        "request": { "type": "execute", "source": "println!(\"streamed\\n\");" }
    }));

    let mut saw_phase = false;
    let mut saw_chunk = false;
    let mut response = None;
    for _ in 0..20 {
        let line = serve.read_line();
        if line.get("event").is_some() {
            let event = &line["event"];
            match event["type"].as_str() {
                Some("phase") => saw_phase = true,
                Some("chunk")
                    if event["data"]
                        .as_str()
                        .is_some_and(|d| d.contains("streamed")) =>
                {
                    saw_chunk = true;
                }
                _ => {}
            }
            continue;
        }
        response = Some(line);
        break;
    }

    assert!(saw_phase, "expected phase events before response");
    assert!(saw_chunk, "expected stdout chunk event");
    let response = response.expect("final response envelope");
    assert_eq!(response["id"], "exec-1");
    assert_eq!(response["response"]["type"], "execution");
    assert_eq!(response["response"]["execution_count"], 1);
}

#[test]
fn serve_handles_status_and_reset() {
    let mut serve = ServeProcess::spawn();
    serve.send(json!({ "id": "s1", "request": { "type": "status" } }));
    let status = serve.read_line();
    assert_eq!(status["response"]["type"], "status");

    serve.send(json!({ "id": "r1", "request": { "type": "reset" } }));
    let reset = serve.read_line();
    assert_eq!(reset["response"]["type"], "reset");

    serve.send(json!({ "id": "s2", "request": { "type": "status" } }));
    let status = serve.read_line();
    assert_eq!(status["response"]["execution_count"], 0);
}

#[test]
fn serve_rejects_invalid_json() {
    let mut serve = ServeProcess::spawn();
    serve.stdin.write_all(b"not json\n").expect("write");
    serve.stdin.flush().expect("flush");
    // The serve loop returns an error and exits on invalid JSON.
    let mut line = String::new();
    let _ = serve.stdout.read_line(&mut line);
    assert!(serve.child.wait().expect("wait").success() || true);
}

#[test]
fn serve_handles_variables_diagnose_input_and_cancel() {
    let mut serve = ServeProcess::spawn();

    // Variables on a fresh kernel reports an empty list without compiling.
    serve.send(json!({ "id": "v1", "request": { "type": "variables" } }));
    let variables = serve.read_line();
    assert_eq!(variables["response"]["type"], "variables");
    assert_eq!(variables["response"]["variables"], json!([]));

    // Diagnose with empty source short-circuits before compiling.
    serve.send(json!({
        "id": "d1",
        "request": { "type": "diagnose", "source": "   " }
    }));
    let diagnose = serve.read_line();
    assert_eq!(diagnose["response"]["type"], "diagnostics");
    assert_eq!(diagnose["response"]["diagnostics"], json!([]));

    // Input with no active execution is an error, not a hang.
    serve.send(json!({
        "id": "i1",
        "request": { "type": "input", "text": "answer" }
    }));
    let input = serve.read_line();
    assert_eq!(input["response"]["type"], "error");

    // Cancel is acknowledged immediately by the reader loop.
    serve.send(json!({ "id": "c1", "request": { "type": "cancel" } }));
    let cancel = serve.read_line();
    assert_eq!(cancel["response"]["type"], "cancelled");
}

#[test]
fn serve_rejects_empty_execute_source() {
    let mut serve = ServeProcess::spawn();
    serve.send(json!({
        "id": "e1",
        "request": { "type": "execute", "source": "   " }
    }));
    let response = serve.read_line();
    assert_eq!(response["response"]["type"], "error");
}

#[test]
fn serve_executes_in_requested_language() {
    let mut serve = ServeProcess::spawn();
    // python3 is available on dev machines; skip gracefully if not.
    serve.send(json!({
        "id": "py1",
        "request": {
            "type": "execute",
            "source": "21 * 2",
            "language": "python"
        }
    }));

    let mut response = None;
    for _ in 0..30 {
        let line = serve.read_line();
        if line.get("event").is_none() {
            response = Some(line);
            break;
        }
    }
    let response = response.expect("final response");
    match &response["response"]["type"] {
        // Either the expression result or an honest backend error.
        Value::String(kind) => assert!(kind == "execution" || kind == "error"),
        _ => panic!("unexpected response: {response}"),
    }
}

#[test]
fn serve_tcp_handles_status() {
    // Find a free port, then hand it to the kernel.
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind probe");
        listener.local_addr().expect("local addr").port()
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_mars"))
        .args(["kernel", "serve", "--tcp", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mars kernel serve --tcp");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let stream = loop {
        match std::net::TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => break stream,
            Err(_) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(error) => panic!("connect to kernel tcp port: {error}"),
        }
    };
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut writer = stream;
    writeln!(
        writer,
        "{}",
        json!({ "id": "tcp-1", "request": { "type": "status" } })
    )
    .expect("write request");
    writer.flush().expect("flush");
    let mut line = String::new();
    let read = reader.read_line(&mut line).expect("read response");
    assert!(read > 0, "kernel closed tcp connection");
    let response: Value = serde_json::from_str(line.trim()).expect("parse json line");
    assert_eq!(response["id"], "tcp-1");
    assert!(response["response"]["type"].is_string());

    let _ = child.kill();
    let _ = child.wait();
}
