use mars_document::load;
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{
    EventEnvelope, KernelChunkStream, KernelRequest, KernelStreamEvent, KernelStreamPhase,
    RequestEnvelope, ResponseEnvelope,
};
use mars_runtime::StreamEvent;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn wire_event(event: StreamEvent) -> KernelStreamEvent {
    match event {
        StreamEvent::Phase {
            phase: mars_runtime::StreamPhase::Compiling,
        } => KernelStreamEvent::Phase {
            phase: KernelStreamPhase::Compiling,
        },
        StreamEvent::Phase {
            phase: mars_runtime::StreamPhase::Running,
        } => KernelStreamEvent::Phase {
            phase: KernelStreamPhase::Running,
        },
        StreamEvent::Chunk { stream, data } => KernelStreamEvent::Chunk {
            stream: match stream {
                mars_runtime::ChunkStream::Stdout => KernelChunkStream::Stdout,
                mars_runtime::ChunkStream::Stderr => KernelChunkStream::Stderr,
            },
            data,
        },
        StreamEvent::Prompt { message } => KernelStreamEvent::Prompt { message },
    }
}

fn build_kernel(notebook: Option<String>, backend: RuntimeBackendKind) -> Result<Kernel, String> {
    if let Some(path) = notebook {
        let notebook = load(&path).map_err(|error| error.to_string())?;
        Kernel::with_configuration(
            notebook.metadata.dependencies,
            notebook.metadata.cargo_lock,
            false,
            false,
            backend,
        )
        .map_err(|error| error.to_string())
    } else {
        Kernel::with_backend(Default::default(), backend, false).map_err(|error| error.to_string())
    }
}

fn serve_connection<R: BufRead, W: Write>(
    reader: R,
    mut writer: BufWriter<W>,
    backend: RuntimeBackendKind,
    notebook: Option<String>,
) -> Result<(), String> {
    let mut kernel = build_kernel(notebook, backend)?;
    let mut reader = reader;
    let mut line = String::new();
    loop {
        line.clear();
        if reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let request: RequestEnvelope =
            serde_json::from_str(&line).map_err(|error| format!("invalid request: {error}"))?;
        let response = match request.request {
            KernelRequest::Execute {
                source,
                timeout_millis,
            } => {
                let id = request.id.clone();
                let mut sink = |event: StreamEvent| {
                    let envelope = EventEnvelope {
                        id: id.clone(),
                        event: wire_event(event),
                    };
                    if serde_json::to_writer(&mut writer, &envelope).is_ok() {
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                };
                ResponseEnvelope {
                    id: request.id,
                    response: kernel.execute_with_events(
                        &source,
                        timeout_millis.map(Duration::from_millis),
                        Arc::new(AtomicBool::new(false)),
                        &mut sink,
                    ),
                }
            }
            _ => kernel.handle(request),
        };
        serde_json::to_writer(&mut writer, &response).map_err(|error| error.to_string())?;
        writer.write_all(b"\n").map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn serve_kernel(backend: RuntimeBackendKind, notebook: Option<String>) -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve_connection(
        stdin.lock(),
        BufWriter::new(stdout.lock()),
        backend,
        notebook,
    )
}

/// Serve the kernel protocol over TCP. Each connection gets its own kernel
/// session; connections are handled sequentially.
pub fn serve_kernel_tcp(
    port: u16,
    host: Option<String>,
    backend: RuntimeBackendKind,
    notebook: Option<String>,
) -> Result<(), String> {
    // Loopback by default: the kernel protocol has no authentication, so
    // exposing it on all interfaces would let any LAN client execute code.
    let host = host.unwrap_or_else(|| "127.0.0.1".to_owned());
    if !matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") {
        eprintln!(
            "warning: mars kernel protocol has no authentication;              {host}:{port} is reachable by any client that can connect"
        );
    }
    let listener = TcpListener::bind((host.as_str(), port))
        .map_err(|error| format!("cannot bind {host}:{port}: {error}"))?;
    eprintln!("mars kernel listening on {host}:{port}");
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let notebook = notebook.clone();
        std::thread::spawn(move || match stream.try_clone() {
            Ok(clone) => {
                let _ = serve_connection(
                    BufReader::new(clone),
                    BufWriter::new(stream),
                    backend,
                    notebook,
                );
            }
            Err(error) => eprintln!("mars kernel: cannot clone tcp stream: {error}"),
        });
    }
    Ok(())
}
