mod cells;
mod commands;
mod editors;
mod execution;
mod file_ops;
mod render;
mod render_cell;
mod render_output;
mod render_view;
mod widgets;

use gpui::*;
use gpui_component::input::{EditorState, TextareaState};
use mars_core::{CellId, ExecutionStatus};
use mars_document::Notebook;
use mars_kernel::{Kernel, RuntimeBackendKind};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) const BG: u32 = 0x141414;
pub(crate) const PANEL: u32 = 0x1c1c1c;
pub(crate) const HEADER: u32 = 0x202020;
pub(crate) const BORDER: u32 = 0x2a2a2a;
pub(crate) const TEXT: u32 = 0xe5e5e5;
pub(crate) const MUTED: u32 = 0x9ca3af;
pub(crate) const FAINT: u32 = 0x6b7280;
pub(crate) const ACCENT: u32 = 0xf97316;
pub(crate) const CODE_BG: u32 = 0x1f2937;
pub(crate) const ERROR_BG: u32 = 0x2a1518;
pub(crate) const ERROR_BORDER: u32 = 0xef4444;

pub struct NotebookApp {
    notebook: Notebook,
    path: Option<PathBuf>,
    kernel: Option<Arc<Mutex<Kernel>>>,
    backend: RuntimeBackendKind,
    executing: bool,
    status: String,
    kernel_status: ExecutionStatus,
    focus_handle: FocusHandle,
    editors: HashMap<CellId, Entity<TextareaState>>,
    code_editors: HashMap<CellId, Entity<EditorState>>,
    subscriptions: Vec<Subscription>,
    dirty: bool,
    scroll_handle: ScrollHandle,
    preview_cells: HashSet<CellId>,
    running_cells: HashSet<usize>,
    command_mode: bool,
    selected_cell: usize,
    scroll_anchors: HashMap<CellId, ScrollAnchor>,
    durations: HashMap<CellId, u128>,
}

impl NotebookApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, path: Option<String>) -> Self {
        let backend = RuntimeBackendKind::from_environment();
        let (kernel, kernel_error) = match Kernel::with_backend(Default::default(), backend, false)
        {
            Ok(kernel) => (Some(Arc::new(Mutex::new(kernel))), None),
            Err(error) => (None, Some(format!("Kernel creation failed: {error}"))),
        };
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        let mut this = Self {
            notebook: Notebook::new(),
            path: None,
            kernel,
            backend,
            executing: false,
            status: kernel_error.unwrap_or_else(|| "Ready".to_owned()),
            kernel_status: ExecutionStatus::Idle,
            focus_handle,
            editors: HashMap::new(),
            code_editors: HashMap::new(),
            subscriptions: Vec::new(),
            dirty: false,
            scroll_handle: ScrollHandle::new(),
            preview_cells: HashSet::new(),
            running_cells: HashSet::new(),
            command_mode: false,
            selected_cell: 0,
            scroll_anchors: HashMap::new(),
            durations: HashMap::new(),
        };
        if let Some(path) = path {
            this.open_path(PathBuf::from(path), cx);
        }
        this.ensure_editors(window, cx);

        let autosave_executor = cx.background_executor().clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                autosave_executor.timer(Duration::from_secs(2)).await;
                if this
                    .update(cx, |this, cx| {
                        if this.dirty {
                            this.autosave(cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        this
    }
}
