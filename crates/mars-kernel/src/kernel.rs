use crate::error::KernelError;
use mars_core::ExecutionStatus;
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope, ResponseEnvelope};
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError, StreamEvent};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct Kernel {
    runtime: Box<dyn RuntimeBackend>,
    status: ExecutionStatus,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RuntimeBackendKind {
    #[default]
    Replay,
    Evcxr,
}

impl RuntimeBackendKind {
    pub fn from_environment() -> Self {
        match std::env::var("MARS_EXPERIMENTAL_RUNTIME").as_deref() {
            Ok("evcxr") => Self::Evcxr,
            _ => Self::Replay,
        }
    }
}

impl Kernel {
    pub fn new() -> Result<Self, KernelError> {
        Self::with_dependencies(BTreeMap::new())
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, KernelError> {
        Self::with_backend(dependencies, RuntimeBackendKind::Replay, false)
    }

    pub fn with_backend(
        dependencies: BTreeMap<String, String>,
        backend: RuntimeBackendKind,
        offline: bool,
    ) -> Result<Self, KernelError> {
        Self::for_language_with_configuration("rust", dependencies, None, offline, false, backend)
    }

    pub fn with_configuration(
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
        backend: RuntimeBackendKind,
    ) -> Result<Self, KernelError> {
        Self::for_language_with_configuration(
            "rust",
            dependencies,
            cargo_lock,
            offline,
            locked,
            backend,
        )
    }

    /// Builds a kernel for a specific notebook language (e.g. `"rust"`,
    /// `"python"`) with default execution options.
    pub fn for_language(language: &str, backend: RuntimeBackendKind) -> Result<Self, KernelError> {
        Self::for_language_with_configuration(
            language,
            BTreeMap::new(),
            None,
            false,
            false,
            backend,
        )
    }

    /// Builds a kernel for a specific notebook language. Rust-specific
    /// options (`dependencies`, `cargo_lock`, `offline`, `locked`) are only
    /// applied by the Rust backend.
    pub fn for_language_with_configuration(
        language: &str,
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
        backend: RuntimeBackendKind,
    ) -> Result<Self, KernelError> {
        Self::build(
            language,
            dependencies,
            cargo_lock,
            offline,
            locked,
            backend,
            mars_runtime_python::PythonOptions::default(),
        )
    }

    /// Builds a `sql` kernel with pre-loaded CSV table sources.
    pub fn for_language_with_sql_sources(
        language: &str,
        backend: RuntimeBackendKind,
        sql_sources: Vec<mars_runtime::SqlSource>,
    ) -> Result<Self, KernelError> {
        Self::build_with_sql(language, BTreeMap::new(), backend, sql_sources)
    }

    /// Builds a kernel with Python interpreter/version/package options
    /// (parsed from notebook `metadata.python`). Other languages ignore
    /// them; Rust-specific options stay at their defaults.
    pub fn for_language_with_python_options(
        language: &str,
        backend: RuntimeBackendKind,
        python_options: mars_runtime_python::PythonOptions,
    ) -> Result<Self, KernelError> {
        Self::build(
            language,
            BTreeMap::new(),
            None,
            false,
            false,
            backend,
            python_options,
        )
    }

    /// Builds a kernel carrying SQL table sources (used by `sql` notebooks).
    fn build_with_sql(
        language: &str,
        dependencies: BTreeMap<String, String>,
        backend: RuntimeBackendKind,
        sql_sources: Vec<mars_runtime::SqlSource>,
    ) -> Result<Self, KernelError> {
        Ok(Self {
            runtime: crate::runtime::create_runtime_with_config(
                language,
                crate::runtime::RuntimeConfig {
                    backend,
                    dependencies,
                    cargo_lock: None,
                    offline: false,
                    locked: false,
                    python_options: mars_runtime_python::PythonOptions::default(),
                    sql_sources,
                },
            )
            .map_err(KernelError::Runtime)?,
            status: ExecutionStatus::Idle,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        language: &str,
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
        backend: RuntimeBackendKind,
        python_options: mars_runtime_python::PythonOptions,
    ) -> Result<Self, KernelError> {
        Ok(Self {
            runtime: crate::runtime::create_runtime_with_config(
                language,
                crate::runtime::RuntimeConfig {
                    backend,
                    dependencies,
                    cargo_lock,
                    offline,
                    locked,
                    python_options,
                    sql_sources: Vec::new(),
                },
            )
            .map_err(KernelError::Runtime)?,
            status: ExecutionStatus::Idle,
        })
    }

    pub fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        self.runtime.cargo_lock()
    }

    pub fn inspect_variables(&mut self) -> Result<Vec<mars_runtime::VariableInfo>, RuntimeError> {
        self.runtime.inspect_variables()
    }

    pub fn diagnose(
        &mut self,
        source: &str,
    ) -> Result<Vec<mars_runtime::CellDiagnostic>, RuntimeError> {
        self.runtime.diagnose(source)
    }

    pub fn write_stdin(&mut self, text: &str) -> Result<(), RuntimeError> {
        self.runtime.write_stdin(text)
    }

    /// Forwards a Jupyter-style comm message to the runtime and returns the
    /// comm traffic it publishes in response.
    pub fn handle_comm(
        &mut self,
        comm_id: &str,
        comm_type: &str,
        data: &serde_json::Value,
    ) -> Result<Vec<mars_core::CommMessage>, RuntimeError> {
        self.runtime.handle_comm(comm_id, comm_type, data)
    }

    /// Returns a handle for answering interactive prompts while a cell is
    /// executing, without holding a mutable borrow on the kernel.
    pub fn stdin_writer(&self) -> Option<mars_runtime::StdinWriter> {
        self.runtime.stdin_writer()
    }

    pub fn execute_source(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.runtime
            .execute_with_control(source, timeout, cancellation)
    }

    pub fn handle(&mut self, envelope: RequestEnvelope) -> ResponseEnvelope {
        self.handle_with_cancellation(envelope, Arc::new(AtomicBool::new(false)))
    }

    pub fn handle_with_cancellation(
        &mut self,
        envelope: RequestEnvelope,
        cancellation: Arc<AtomicBool>,
    ) -> ResponseEnvelope {
        let response = match envelope.request {
            KernelRequest::Execute {
                source,
                timeout_millis,
                language: _,
                // Runtime options apply at kernel construction; a live
                // Kernel cannot swap interpreters mid-session.
                options: _,
            } => self.execute(
                &source,
                timeout_millis.map(Duration::from_millis),
                cancellation,
                None,
            ),
            KernelRequest::Reset => match self.runtime.reset() {
                Ok(()) => {
                    self.status = ExecutionStatus::Idle;
                    KernelResponse::Reset {
                        status: self.status.clone(),
                    }
                }
                Err(error) => {
                    self.status = ExecutionStatus::Failed;
                    KernelResponse::Error {
                        message: error.to_string(),
                    }
                }
            },
            KernelRequest::Status => KernelResponse::Status {
                status: self.status.clone(),
                execution_count: self.runtime.execution_count(),
            },
            KernelRequest::Cancel => {
                cancellation.store(true, Ordering::Relaxed);
                KernelResponse::Cancelled
            }
            KernelRequest::Variables => match self.runtime.inspect_variables() {
                Ok(variables) => KernelResponse::Variables {
                    variables: variables
                        .into_iter()
                        .map(|variable| mars_protocol::VariableInfo {
                            name: variable.name,
                            type_name: variable.type_name,
                            value: variable.value,
                        })
                        .collect(),
                },
                Err(error) => KernelResponse::Error {
                    message: error.to_string(),
                },
            },
            KernelRequest::Diagnose { source } => match self.runtime.diagnose(&source) {
                Ok(diagnostics) => KernelResponse::Diagnostics {
                    diagnostics: diagnostics
                        .into_iter()
                        .map(|diagnostic| mars_protocol::CellDiagnostic {
                            cell_index: diagnostic.cell_index,
                            line: diagnostic.line,
                            column: diagnostic.column,
                            end_line: diagnostic.end_line,
                            end_column: diagnostic.end_column,
                            severity: diagnostic.severity,
                            code: diagnostic.code,
                            message: diagnostic.message,
                        })
                        .collect(),
                },
                Err(error) => KernelResponse::Error {
                    message: error.to_string(),
                },
            },
            KernelRequest::Input { text } => match self.runtime.write_stdin(&text) {
                Ok(()) => KernelResponse::Input,
                Err(error) => KernelResponse::Error {
                    message: error.to_string(),
                },
            },
            KernelRequest::Comm {
                comm_id,
                comm_type,
                data,
                language: _,
            } => match self.runtime.handle_comm(&comm_id, &comm_type, &data) {
                Ok(comms) => KernelResponse::Comm { comms },
                Err(error) => KernelResponse::Error {
                    message: error.to_string(),
                },
            },
        };
        ResponseEnvelope {
            id: envelope.id,
            response,
        }
    }

    pub fn execute_with_events(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
        events: &mut dyn FnMut(StreamEvent),
    ) -> KernelResponse {
        self.execute(source, timeout, cancellation, Some(events))
    }

    fn execute(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
        events: Option<&mut dyn FnMut(StreamEvent)>,
    ) -> KernelResponse {
        self.status = ExecutionStatus::Running;
        let result = match events {
            Some(sink) => self
                .runtime
                .execute_with_events(source, timeout, cancellation, sink),
            None => self
                .runtime
                .execute_with_control(source, timeout, cancellation),
        };
        match result {
            Ok(result) => {
                let failed = result
                    .outputs
                    .iter()
                    .any(|output| matches!(output, mars_core::Output::Error { .. }));
                self.status = if result.interrupted {
                    ExecutionStatus::Interrupted
                } else if failed {
                    ExecutionStatus::Failed
                } else {
                    ExecutionStatus::Succeeded
                };
                KernelResponse::Execution {
                    outputs: result.outputs,
                    execution_count: result.execution_count,
                    duration_millis: u64::try_from(result.duration.as_millis()).unwrap_or(u64::MAX),
                }
            }
            Err(error) => {
                self.status = ExecutionStatus::Failed;
                KernelResponse::Error {
                    message: error.to_string(),
                }
            }
        }
    }
}
