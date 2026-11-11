use mars::kernel::serve_connection;
use mars_kernel::RuntimeBackendKind;
use serde_json::Value;
use std::io::{BufReader, BufWriter, Cursor};

/// Run `serve_connection` in-process over in-memory pipes and return the
/// JSON-lines written to the "client".
fn serve(input: &str, notebook: Option<String>, language: Option<String>) -> Vec<Value> {
    let reader = BufReader::new(Cursor::new(input.as_bytes().to_vec()));
    let mut output = Vec::new();
    serve_connection(
        reader,
        BufWriter::new(&mut output),
        RuntimeBackendKind::Replay,
        notebook,
        language,
    )
    .expect("serve connection");
    String::from_utf8(output)
        .expect("utf8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("json line"))
        .collect()
}

fn notebook_file(name: &str, language: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "mars-kernel-inproc-{}-{}",
        std::process::id(),
        name
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("book.mars");
    let mut notebook = mars_document::Notebook::new();
    notebook.metadata.language = language.to_owned();
    mars_document::save_atomic(&notebook, &path).expect("save");
    path.to_string_lossy().into_owned()
}

#[test]
fn serves_status_variables_diagnose_input_cancel_reset() {
    let input = concat!(
        r#"{"id":"s","request":{"type":"status"}}"#,
        "\n",
        r#"{"id":"v","request":{"type":"variables"}}"#,
        "\n",
        r#"{"id":"d","request":{"type":"diagnose","source":"  "}}"#,
        "\n",
        r#"{"id":"i","request":{"type":"input","text":"x"}}"#,
        "\n",
        r#"{"id":"c","request":{"type":"cancel"}}"#,
        "\n",
        r#"{"id":"r","request":{"type":"reset"}}"#,
        "\n",
    );
    let responses = serve(input, None, None);
    assert_eq!(responses.len(), 6);
    // `cancel` is answered on the reader thread and can overtake queued
    // executor responses, so match by request id rather than position.
    let kind_for = |id: &str| {
        responses
            .iter()
            .find(|r| r["id"] == id)
            .and_then(|r| r["response"]["type"].as_str())
            .unwrap_or("")
            .to_owned()
    };
    assert_eq!(kind_for("s"), "status");
    assert_eq!(kind_for("v"), "variables");
    assert_eq!(kind_for("d"), "diagnostics");
    assert_eq!(kind_for("i"), "error");
    assert_eq!(kind_for("c"), "cancelled");
    assert_eq!(kind_for("r"), "reset");
}

#[test]
fn serves_execute_events_then_response() {
    let input = r#"{"id":"e","request":{"type":"execute","source":"println!(\"hi-inproc\");","timeout_millis":10000}}"#;
    let responses = serve(&format!("{input}\n"), None, None);
    // Expect phase events followed by the execution response.
    let mut saw_phase = false;
    let mut final_response = None;
    for envelope in &responses {
        if envelope.get("event").is_some() {
            saw_phase = true;
        } else {
            final_response = Some(envelope);
        }
    }
    assert!(saw_phase, "expected stream events");
    let response = final_response.expect("final response");
    assert_eq!(response["id"], "e");
    assert_eq!(response["response"]["type"], "execution");
    assert_eq!(response["response"]["execution_count"], 1);
}

#[test]
fn execute_errors_for_empty_source_and_unknown_language() {
    let input = concat!(
        r#"{"id":"empty","request":{"type":"execute","source":"  "}}"#,
        "\n",
        r#"{"id":"lang","request":{"type":"execute","source":"1","language":"cobol"}}"#,
        "\n",
    );
    let responses = serve(input, None, None);
    assert_eq!(responses.len(), 2);
    for response in &responses {
        assert_eq!(response["response"]["type"], "error");
        assert!(response["response"]["message"].is_string());
    }
}

#[test]
fn notebook_path_supplies_default_language_and_config() {
    let path = notebook_file("defaults", "rust");
    let input = r#"{"id":"n","request":{"type":"status"}}"#;
    let responses = serve(&format!("{input}\n"), Some(path), Some("rust".to_owned()));
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["response"]["type"], "status");
}

#[test]
fn blank_lines_and_eof_are_handled() {
    let input = "\n\n   \n{\"id\":\"x\",\"request\":{\"type\":\"status\"}}\n\n";
    let responses = serve(input, None, None);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], "x");
}

#[test]
fn invalid_json_line_terminates_connection() {
    let input = "garbage\n{\"id\":\"never\",\"request\":{\"type\":\"status\"}}\n";
    let responses = serve(input, None, None);
    assert!(responses.is_empty());
}

#[test]
fn serve_kernel_tcp_accepts_connections() {
    // Bind port 0 ourselves and hand the listener to the server: no
    // probe-then-release race for the port to be stolen between bind and
    // accept.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        let _ =
            mars::kernel::serve_kernel_listener(listener, RuntimeBackendKind::Replay, None, None);
    });

    // Retry the whole exchange: under load the accepted socket can be torn
    // down before the request is written, so a single attempt is flaky.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let response = loop {
        let attempt = (|| -> std::io::Result<Value> {
            let stream = std::net::TcpStream::connect(("127.0.0.1", port))?;
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("timeout");
            use std::io::{BufRead, BufReader, Write};
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut writer = stream;
            writeln!(
                writer,
                "{}",
                serde_json::json!({"id":"tcp","request":{"type":"status"}})
            )?;
            writer.flush()?;
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "empty response",
                ));
            }
            serde_json::from_str(line.trim())
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })();
        match attempt {
            Ok(response) => break response,
            Err(_) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => panic!("tcp exchange failed: {error}"),
        }
    };
    assert_eq!(response["id"], "tcp");
    assert_eq!(response["response"]["type"], "status");
    // The listener thread keeps running until the test process exits.
}

#[test]
fn language_override_selects_runtime_per_request() {
    // Python is optional; if unavailable the response is still an honest error.
    let input = r#"{"id":"py","request":{"type":"execute","source":"40 + 2","language":"python"}}"#;
    let responses = serve(&format!("{input}\n"), None, Some("rust".to_owned()));
    let finals: Vec<&Value> = responses
        .iter()
        .filter(|r| r.get("event").is_none())
        .collect();
    assert_eq!(finals.len(), 1);
    let kind = finals[0]["response"]["type"].as_str().unwrap_or("");
    assert!(kind == "execution" || kind == "error");
}

/// Interactive test: an `Input` request must reach a live Python `input()`
/// prompt through the stdin-writer fast path while `Execute` is blocked.
#[test]
fn input_request_answers_live_python_prompt() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    use std::io::{BufRead, Write};
    use std::sync::mpsc;

    let (req_read, req_write) = std::io::pipe().expect("request pipe");
    let (resp_read, resp_write) = std::io::pipe().expect("response pipe");

    std::thread::spawn(move || {
        let _ = serve_connection(
            BufReader::new(req_read),
            BufWriter::new(resp_write),
            RuntimeBackendKind::Replay,
            None,
            Some("python".to_owned()),
        );
    });

    // Collect response lines on a channel so the test can wait with a
    // timeout instead of blocking on the pipe.
    let (line_tx, line_rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(resp_read);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if line_tx.send(line.trim_end().to_owned()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut requests = BufWriter::new(req_write);
    writeln!(
        requests,
        "{}",
        serde_json::json!({
            "id": "e",
            "request": {
                "type": "execute",
                "source": "name = input('Name? ')\nprint('hi ' + name)",
                "timeout_millis": 30000,
                "language": "python"
            }
        })
    )
    .expect("write execute");
    requests.flush().expect("flush");

    // Retry Input until the Python driver has spawned and registered its
    // stdin writer; earlier attempts get "no active execution" errors.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut answered = false;
    while std::time::Instant::now() < deadline && !answered {
        writeln!(
            requests,
            "{}",
            serde_json::json!({"id": "inp", "request": {"type": "input", "text": "Bob"}})
        )
        .expect("write input");
        requests.flush().expect("flush");
        let wait = std::time::Instant::now() + std::time::Duration::from_millis(400);
        while std::time::Instant::now() < wait {
            if let Ok(line) = line_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                let value: Value = serde_json::from_str(&line).expect("json");
                if value["id"] == "inp" && value["response"]["type"] == "input" {
                    answered = true;
                    break;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    assert!(answered, "input request was never accepted");

    // Wait for the final execution response and check the echoed input.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut saw_hi_bob = false;
    while std::time::Instant::now() < deadline && !saw_hi_bob {
        if let Ok(line) = line_rx.recv_timeout(std::time::Duration::from_millis(200)) {
            let value: Value = serde_json::from_str(&line).expect("json");
            if value["id"] == "e"
                && serde_json::to_string(&value)
                    .expect("json")
                    .contains("hi Bob")
            {
                saw_hi_bob = true;
            }
        }
    }
    assert!(saw_hi_bob, "execution did not echo the prompted input");

    drop(requests);
}
