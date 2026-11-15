use mars_core::ExecutionStatus;
use mars_document::{Notebook, load};
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{
    EventEnvelope, KernelChunkStream, KernelRequest, KernelResponse, KernelStreamEvent,
    KernelStreamPhase, RequestEnvelope, ResponseEnvelope,
};
use mars_runtime::{StdinWriter, StreamEvent};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
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
        StreamEvent::Comm { message } => KernelStreamEvent::Comm { message },
    }
}

fn build_kernel(
    notebook: &Option<Notebook>,
    backend: RuntimeBackendKind,
    language: &str,
    request_options: Option<&serde_json::Value>,
    sql_sources: &[mars_runtime::SqlSource],
) -> Result<Kernel, String> {
    if language == "sql" {
        return Kernel::for_language_with_sql_sources(language, backend, sql_sources.to_vec())
            .map_err(|error| error.to_string());
    }
    if language == "python" {
        let options = request_options
            .and_then(|options| options.get("python"))
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .or_else(|| {
                notebook.as_ref().map(|notebook| {
                    mars_kernel::python_options_from_metadata(&notebook.metadata.extra)
                })
            })
            .unwrap_or_default();
        return Kernel::for_language_with_python_options(language, backend, options)
            .map_err(|error| error.to_string());
    }
    if let Some(notebook) = notebook {
        Kernel::for_language_with_configuration(
            language,
            notebook.metadata.dependencies.clone(),
            notebook.metadata.cargo_lock.clone(),
            false,
            false,
            backend,
        )
        .map_err(|error| error.to_string())
    } else {
        Kernel::for_language(language, backend).map_err(|error| error.to_string())
    }
}

fn write_json<W: Write, T: serde::Serialize>(
    writer: &Mutex<BufWriter<W>>,
    value: &T,
) -> Result<(), String> {
    let mut guard = writer
        .lock()
        .map_err(|_| "kernel output writer poisoned".to_owned())?;
    serde_json::to_writer(&mut *guard, value).map_err(|error| error.to_string())?;
    guard.write_all(b"\n").map_err(|error| error.to_string())?;
    guard.flush().map_err(|error| error.to_string())
}

/// Serve one connection. Requests are read on the calling thread while an
/// executor thread owns the kernels, so `Cancel` requests are honoured while
/// an `Execute` is still blocked compiling or running. `Input` requests are
/// answered directly through each backend's `StdinWriter` so a cell blocked
/// on `input()`/`mars_prompt` does not deadlock behind the running `Execute`.
///
/// One kernel is kept per language: `Execute.language` (or the connection's
/// `--language`/notebook default) selects the runtime, and subsequent
/// non-language-scoped requests apply to the most recently executed language.
fn kernel_for_language<'a>(
    kernels: &'a mut HashMap<String, Kernel>,
    notebook: &Option<Notebook>,
    backend: RuntimeBackendKind,
    language: &str,
    options: Option<&serde_json::Value>,
    writers: &Mutex<HashMap<String, StdinWriter>>,
    sql_sources: &[mars_runtime::SqlSource],
) -> Result<&'a mut Kernel, String> {
    if !kernels.contains_key(language) {
        let kernel = build_kernel(notebook, backend, language, options, sql_sources)?;
        kernels.insert(language.to_owned(), kernel);
    }
    let kernel = kernels
        .get_mut(language)
        .ok_or_else(|| "kernel session unavailable".to_owned())?;
    if let Ok(mut guard) = writers.lock() {
        if let Some(writer) = kernel.stdin_writer() {
            guard.insert(language.to_owned(), writer);
        }
    }
    Ok(kernel)
}

fn worse_status(current: ExecutionStatus, next: ExecutionStatus) -> ExecutionStatus {
    fn rank(status: &ExecutionStatus) -> u8 {
        match status {
            ExecutionStatus::Running => 0,
            ExecutionStatus::Interrupted => 1,
            ExecutionStatus::Failed => 2,
            ExecutionStatus::Succeeded => 3,
            ExecutionStatus::Idle => 4,
        }
    }
    if rank(&next) < rank(&current) {
        next
    } else {
        current
    }
}

#[doc(hidden)]
pub fn serve_connection<R: BufRead, W: Write + Send>(
    mut reader: R,
    writer: BufWriter<W>,
    backend: RuntimeBackendKind,
    notebook_path: Option<String>,
    language_override: Option<String>,
) -> Result<(), String> {
    let notebook_base = notebook_path
        .as_ref()
        .and_then(|path| Path::new(path).parent().map(Path::to_path_buf))
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    let notebook = notebook_path
        .map(|path| load(&path).map_err(|error| error.to_string()))
        .transpose()?;
    let sql_sources = notebook
        .as_ref()
        .map(|notebook| {
            mars_kernel::sql_sources_from_metadata(&notebook.metadata.extra, &notebook_base)
        })
        .unwrap_or_default();
    let default_language = language_override
        .or_else(|| notebook.as_ref().map(|n| n.metadata.language.clone()))
        .unwrap_or_else(|| "rust".to_owned());

    let writers = Arc::new(Mutex::new(HashMap::<String, StdinWriter>::new()));
    let current_language = Arc::new(Mutex::new(default_language));
    let cancellation = Arc::new(AtomicBool::new(false));
    let writer = Arc::new(Mutex::new(writer));
    let (inbox, requests) = mpsc::channel::<RequestEnvelope>();

    let exec_writer = Arc::clone(&writer);
    let exec_cancellation = Arc::clone(&cancellation);
    let exec_writers = Arc::clone(&writers);
    let exec_language = Arc::clone(&current_language);
    let exec_notebook = notebook;
    let exec_sql_sources = sql_sources;
    let reader_writers = Arc::clone(&writers);
    let reader_language = Arc::clone(&current_language);

    thread::scope(|scope| {
        scope.spawn(move || {
            let mut kernels: HashMap<String, Kernel> = HashMap::new();
            while let Ok(envelope) = requests.recv() {
                let RequestEnvelope { id, request } = envelope;
                let response = match request {
                    KernelRequest::Execute {
                        source,
                        timeout_millis,
                        language,
                        options,
                    } => {
                        exec_cancellation.store(false, Ordering::Relaxed);
                        let language = language.unwrap_or_else(|| {
                            exec_language
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .clone()
                        });
                        *exec_language
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) = language.clone();
                        match kernel_for_language(
                            &mut kernels,
                            &exec_notebook,
                            backend,
                            &language,
                            options.as_ref(),
                            &exec_writers,
                            &exec_sql_sources,
                        ) {
                            Err(message) => ResponseEnvelope {
                                id,
                                response: KernelResponse::Error { message },
                            },
                            Ok(kernel) => {
                                let event_id = id.clone();
                                let event_writer = Arc::clone(&exec_writer);
                                let mut sink = move |event: StreamEvent| {
                                    let envelope = EventEnvelope {
                                        id: event_id.clone(),
                                        event: wire_event(event),
                                    };
                                    let _ = write_json(&event_writer, &envelope);
                                };
                                ResponseEnvelope {
                                    id,
                                    response: kernel.execute_with_events(
                                        &source,
                                        timeout_millis.map(Duration::from_millis),
                                        Arc::clone(&exec_cancellation),
                                        &mut sink,
                                    ),
                                }
                            }
                        }
                    }
                    KernelRequest::Reset => {
                        for kernel in kernels.values_mut() {
                            let _ = kernel.handle_with_cancellation(
                                RequestEnvelope {
                                    id: String::new(),
                                    request: KernelRequest::Reset,
                                },
                                Arc::clone(&exec_cancellation),
                            );
                        }
                        ResponseEnvelope {
                            id,
                            response: KernelResponse::Reset {
                                status: ExecutionStatus::Idle,
                            },
                        }
                    }
                    KernelRequest::Status => {
                        let mut status = ExecutionStatus::Idle;
                        let mut count = 0_u64;
                        for kernel in kernels.values_mut() {
                            if let KernelResponse::Status {
                                status: kernel_status,
                                execution_count,
                            } = kernel
                                .handle_with_cancellation(
                                    RequestEnvelope {
                                        id: String::new(),
                                        request: KernelRequest::Status,
                                    },
                                    Arc::clone(&exec_cancellation),
                                )
                                .response
                            {
                                status = worse_status(status, kernel_status);
                                count += execution_count;
                            }
                        }
                        ResponseEnvelope {
                            id,
                            response: KernelResponse::Status {
                                status,
                                execution_count: count,
                            },
                        }
                    }
                    KernelRequest::Variables => {
                        // Read-only: never spawn a runtime just to report an
                        // empty session.
                        let language = exec_language
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone();
                        match kernels.get_mut(&language) {
                            Some(kernel) => kernel.handle_with_cancellation(
                                RequestEnvelope {
                                    id,
                                    request: KernelRequest::Variables,
                                },
                                Arc::clone(&exec_cancellation),
                            ),
                            None => ResponseEnvelope {
                                id,
                                response: KernelResponse::Variables {
                                    variables: Vec::new(),
                                },
                            },
                        }
                    }
                    KernelRequest::Input { .. } => {
                        // The reader's stdin fast path handles live executions;
                        // reaching the executor means there is no writer, so
                        // there is no active execution to write to.
                        ResponseEnvelope {
                            id,
                            response: KernelResponse::Error {
                                message: "no active execution for kernel input".to_owned(),
                            },
                        }
                    }
                    KernelRequest::Comm {
                        comm_id,
                        comm_type,
                        data,
                        language,
                    } => {
                        let language = language.unwrap_or_else(|| {
                            exec_language
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .clone()
                        });
                        *exec_language
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) = language.clone();
                        let request = KernelRequest::Comm {
                            comm_id,
                            comm_type,
                            data,
                            language: None,
                        };
                        match kernel_for_language(
                            &mut kernels,
                            &exec_notebook,
                            backend,
                            &language,
                            None,
                            &exec_writers,
                            &exec_sql_sources,
                        ) {
                            Err(message) => ResponseEnvelope {
                                id,
                                response: KernelResponse::Error { message },
                            },
                            Ok(kernel) => kernel.handle_with_cancellation(
                                RequestEnvelope { id, request },
                                Arc::clone(&exec_cancellation),
                            ),
                        }
                    }
                    other => {
                        let language = exec_language
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone();
                        match kernel_for_language(
                            &mut kernels,
                            &exec_notebook,
                            backend,
                            &language,
                            None,
                            &exec_writers,
                            &exec_sql_sources,
                        ) {
                            Err(message) => ResponseEnvelope {
                                id,
                                response: KernelResponse::Error { message },
                            },
                            Ok(kernel) => kernel.handle_with_cancellation(
                                RequestEnvelope { id, request: other },
                                Arc::clone(&exec_cancellation),
                            ),
                        }
                    }
                };
                if write_json(&exec_writer, &response).is_err() {
                    break;
                }
            }
        });

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if line.trim().is_empty() {
                continue;
            }
            let request: RequestEnvelope = match serde_json::from_str(&line) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("mars kernel: invalid request: {error}");
                    break;
                }
            };
            if matches!(request.request, KernelRequest::Cancel) {
                cancellation.store(true, Ordering::Relaxed);
                let cancelled = ResponseEnvelope {
                    id: request.id.clone(),
                    response: KernelResponse::Cancelled,
                };
                if write_json(&writer, &cancelled).is_err() {
                    break;
                }
                continue;
            }
            if let KernelRequest::Input { ref text } = request.request {
                // Fast path: answer through the backend's stdin writer so a
                // running Execute is not blocked behind this request.
                let language = reader_language
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clone();
                let handle = reader_writers
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .get(&language)
                    .cloned();
                if let Some(handle) = handle {
                    let response = ResponseEnvelope {
                        id: request.id.clone(),
                        response: match handle.write(text) {
                            Ok(()) => KernelResponse::Input,
                            Err(error) => KernelResponse::Error {
                                message: error.to_string(),
                            },
                        },
                    };
                    if write_json(&writer, &response).is_err() {
                        break;
                    }
                    continue;
                }
            }
            if inbox.send(request).is_err() {
                break;
            }
        }
        drop(inbox);
    });
    Ok(())
}

pub fn serve_kernel(
    backend: RuntimeBackendKind,
    notebook: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve_connection(
        stdin.lock(),
        BufWriter::new(stdout),
        backend,
        notebook,
        language,
    )
}

/// Serve the kernel protocol over TCP. Each connection gets its own kernel
/// session; connections are handled sequentially.
pub fn serve_kernel_tcp(
    port: u16,
    host: Option<String>,
    backend: RuntimeBackendKind,
    notebook: Option<String>,
    language: Option<String>,
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
    serve_kernel_listener(listener, backend, notebook, language)
}

/// Serve the kernel protocol on an already-bound TCP listener. Split out so
/// callers (and tests) can bind port 0 and learn the assigned port without a
/// probe-then-release race.
pub fn serve_kernel_listener(
    listener: TcpListener,
    backend: RuntimeBackendKind,
    notebook: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let notebook = notebook.clone();
        let language = language.clone();
        std::thread::spawn(move || match stream.try_clone() {
            Ok(clone) => {
                let _ = serve_connection(
                    BufReader::new(clone),
                    BufWriter::new(stream),
                    backend,
                    notebook,
                    language,
                );
            }
            Err(error) => eprintln!("mars kernel: cannot clone tcp stream: {error}"),
        });
    }
    Ok(())
}
