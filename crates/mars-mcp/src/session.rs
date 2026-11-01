//! Server-side session state: open notebooks and per-language kernels.
//!
//! Mirrors the multi-language kernel model of `mars kernel serve`: each
//! notebook gets one `Kernel` per language, created lazily with the
//! notebook's declared dependencies. Kernels also exist for ad-hoc execution
//! without a notebook (`path == None`).

use mars_document::{Notebook, load, save_atomic};
use mars_kernel::{Kernel, RuntimeBackendKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct KernelKey {
    path: Option<PathBuf>,
    language: String,
}

struct NotebookEntry {
    notebook: Notebook,
    dirty: bool,
}

pub struct MarsSession {
    backend: RuntimeBackendKind,
    offline: bool,
    default_language: Option<String>,
    notebooks: HashMap<PathBuf, NotebookEntry>,
    kernels: HashMap<KernelKey, Kernel>,
}

impl MarsSession {
    pub fn new(
        backend: RuntimeBackendKind,
        offline: bool,
        default_language: Option<String>,
    ) -> Self {
        Self {
            backend,
            offline,
            default_language,
            notebooks: HashMap::new(),
            kernels: HashMap::new(),
        }
    }

    pub fn default_language(&self) -> &str {
        self.default_language.as_deref().unwrap_or("rust")
    }

    /// Load a notebook into the session (or return the cached copy).
    pub fn open_notebook(&mut self, path: &Path) -> Result<&Notebook, String> {
        let key = path.to_path_buf();
        if !self.notebooks.contains_key(&key) {
            let notebook = load(path).map_err(|error| error.to_string())?;
            self.notebooks.insert(
                key.clone(),
                NotebookEntry {
                    notebook,
                    dirty: false,
                },
            );
        }
        self.notebooks
            .get(&key)
            .map(|entry| &entry.notebook)
            .ok_or_else(|| "notebook session unavailable".to_owned())
    }

    /// Insert a notebook that does not exist on disk yet (`mars_notebook_new`).
    pub fn create_notebook(&mut self, path: &Path, notebook: Notebook) {
        self.notebooks.insert(
            path.to_path_buf(),
            NotebookEntry {
                notebook,
                dirty: true,
            },
        );
    }

    pub fn notebook(&self, path: &Path) -> Option<&Notebook> {
        self.notebooks.get(path).map(|entry| &entry.notebook)
    }

    /// Mutable access for cell mutations. Loads the notebook from disk first
    /// if it is not already in the session, so one-shot clients (each
    /// `mars tools call` is a fresh server process) can edit any `.mars` file.
    pub fn notebook_mut(&mut self, path: &Path) -> Result<&mut Notebook, String> {
        self.open_notebook(path)?;
        self.notebooks
            .get_mut(path)
            .map(|entry| {
                entry.dirty = true;
                &mut entry.notebook
            })
            .ok_or_else(|| format!("notebook is not open: {}", path.display()))
    }

    pub fn is_dirty(&self, path: &Path) -> bool {
        self.notebooks.get(path).is_some_and(|entry| entry.dirty)
    }

    /// Persist a notebook if it was modified through the session.
    pub fn save_notebook(&mut self, path: &Path) -> Result<bool, String> {
        self.open_notebook(path)?;
        let Some(entry) = self.notebooks.get(path) else {
            return Err(format!("notebook is not open: {}", path.display()));
        };
        if !entry.dirty {
            return Ok(false);
        }
        let notebook = entry.notebook.clone();
        save_atomic(&notebook, path).map_err(|error| error.to_string())?;
        if let Some(entry) = self.notebooks.get_mut(path) {
            entry.dirty = false;
        }
        Ok(true)
    }

    /// The kernel for `(notebook, language)`, created lazily. Notebook
    /// metadata supplies Rust dependencies and the Cargo.lock snapshot.
    pub fn kernel_for(
        &mut self,
        path: Option<&Path>,
        language: &str,
    ) -> Result<&mut Kernel, String> {
        let key = KernelKey {
            path: path.map(Path::to_path_buf),
            language: language.to_owned(),
        };
        if !self.kernels.contains_key(&key) {
            let kernel = match path {
                Some(path) => {
                    let notebook = self.open_notebook(path)?;
                    Kernel::for_language_with_configuration(
                        language,
                        notebook.metadata.dependencies.clone(),
                        notebook.metadata.cargo_lock.clone(),
                        self.offline,
                        false,
                        self.backend,
                    )
                }
                None => Kernel::for_language(language, self.backend),
            }
            .map_err(|error| error.to_string())?;
            self.kernels.insert(key.clone(), kernel);
        }
        self.kernels
            .get_mut(&key)
            .ok_or_else(|| "kernel session unavailable".to_owned())
    }

    /// Kernel status without creating a kernel (read-only).
    pub fn kernel_status(
        &mut self,
        path: Option<&Path>,
        language: &str,
    ) -> Option<mars_core::ExecutionStatus> {
        let key = KernelKey {
            path: path.map(Path::to_path_buf),
            language: language.to_owned(),
        };
        self.kernels.get_mut(&key).map(|kernel| {
            match kernel
                .handle(mars_protocol::RequestEnvelope::new(
                    mars_protocol::KernelRequest::Status,
                ))
                .response
            {
                mars_protocol::KernelResponse::Status { status, .. } => status,
                _ => mars_core::ExecutionStatus::Idle,
            }
        })
    }
}
