use super::NotebookApp;
use gpui::*;
use mars_core::{CellType, ExecutionStatus};
use mars_document::{Cell, Notebook, load, save_atomic};
use mars_kernel::Kernel;
use std::path::{Path, PathBuf};

impl NotebookApp {
    pub(super) fn new_notebook(&mut self, cx: &mut Context<Self>) {
        let mut notebook = Notebook::new();
        let _ = notebook.add_cell(Cell::new(CellType::Markdown, "# New Mars notebook"));
        self.configure_kernel(&notebook);
        self.notebook = notebook;
        self.path = None;
        self.reset_editors();
        self.status = "New notebook".to_owned();
        cx.notify();
    }

    pub(super) fn prompt_open(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open Mars notebook".into()),
        });
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            if let Ok(Ok(Some(paths))) = receiver.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update(cx, |this, cx| this.open_path(path, cx));
            }
        })
        .detach();
    }

    pub(super) fn open_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match load(&path) {
            Ok(notebook) => {
                self.configure_kernel(&notebook);
                self.notebook = notebook;
                self.path = Some(path.clone());
                self.reset_editors();
                self.status = format!("Opened {}", path.display());
            }
            Err(error) => self.status = format!("Open failed: {error}"),
        }
        cx.notify();
    }

    fn configure_kernel(&mut self, notebook: &Notebook) {
        match Kernel::with_backend(notebook.metadata.dependencies.clone(), self.backend, false) {
            Ok(kernel) => {
                self.kernel = Some(std::sync::Arc::new(std::sync::Mutex::new(kernel)));
                self.kernel_status = ExecutionStatus::Idle;
            }
            Err(error) => {
                self.kernel = None;
                self.status = format!("Kernel configuration failed: {error}");
            }
        }
    }

    pub(super) fn save(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = self.path.clone() {
            self.write_to(&path, cx);
            return;
        }
        let directory = self
            .path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let receiver = cx.prompt_for_new_path(&directory, Some("untitled.mars"));
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            if let Ok(Ok(Some(path))) = receiver.await {
                let _ = this.update(cx, |this, cx| {
                    this.path = Some(path.clone());
                    this.write_to(&path, cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn autosave(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };
        match save_atomic(&self.notebook, &path) {
            Ok(()) => self.dirty = false,
            Err(error) => self.status = format!("Autosave failed: {error}"),
        }
        cx.notify();
    }

    fn write_to(&mut self, path: &Path, cx: &mut Context<Self>) {
        match save_atomic(&self.notebook, path) {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Saved {}", path.display());
            }
            Err(error) => self.status = format!("Save failed: {error}"),
        }
        cx.notify();
    }
}
