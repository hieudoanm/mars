use crate::AppState;
use mars_kernel::Kernel;
use mars_runtime::StreamEvent;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicBool};
use tauri::Emitter;

pub(super) type KernelMap = HashMap<(String, String), Kernel>;

#[derive(Clone, Serialize)]
pub(super) struct StreamMessage<'a> {
    session: &'a str,
    language: &'a str,
    #[serde(flatten)]
    event: &'a StreamEvent,
}

pub fn stream_sink<'a, R: tauri::Runtime>(
    app: &'a tauri::AppHandle<R>,
    session: &'a str,
    language: &'a str,
) -> impl FnMut(StreamEvent) + 'a {
    move |event| {
        let _ = app.emit(
            "kernel-stream",
            StreamMessage {
                session,
                language,
                event: &event,
            },
        );
    }
}

pub(super) fn kernel_key(session: &str, language: &str) -> (String, String) {
    (session.to_owned(), language.to_owned())
}

pub(super) fn kernel_for<'a>(
    state: &'a AppState,
    kernels: &'a mut KernelMap,
    session: &str,
    language: &str,
    backend: mars_kernel::RuntimeBackendKind,
) -> Result<&'a mut Kernel, String> {
    let key = kernel_key(session, language);
    if !kernels.contains_key(&key) {
        let kernel = if language == "python" {
            let options = state
                .python_options
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(session)
                .cloned()
                .unwrap_or_default();
            Kernel::for_language_with_python_options(language, backend, options)
        } else if language == "sql" {
            let sources = state
                .sql_sources
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(session)
                .cloned()
                .unwrap_or_default();
            Kernel::for_language_with_sql_sources(language, backend, sources)
        } else {
            Kernel::for_language(language, backend)
        }
        .map_err(|error| error.to_string())?;
        kernels.insert(key.clone(), kernel);
    }
    kernels
        .get_mut(&key)
        .ok_or_else(|| "kernel session unavailable".to_owned())
}

/// Registers the kernel's stdin writer so interactive prompts can be
/// answered while `execute_*` holds the kernels mutex.
pub(super) fn register_input_writer(
    state: &AppState,
    session: &str,
    language: &str,
    kernel: &Kernel,
) {
    if let Some(writer) = kernel.stdin_writer()
        && let Ok(mut writers) = state.input_writers.lock()
    {
        writers.insert(kernel_key(session, language), writer);
    }
}

pub(super) fn cancellation_for(state: &AppState, session: &str) -> Arc<AtomicBool> {
    let mut cancellations = state
        .cancellations
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(
        cancellations
            .entry(session.to_owned())
            .or_insert_with(|| Arc::new(AtomicBool::new(false))),
    )
}

pub(super) fn kernels_lock(
    state: &AppState,
) -> Result<std::sync::MutexGuard<'_, KernelMap>, String> {
    state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())
}
