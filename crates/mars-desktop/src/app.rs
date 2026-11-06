use crate::output_text::output_lines;
use crate::{NewNotebook, OpenNotebook, Quit, RestartKernel, RunAllCells, SaveNotebook};
use gpui::*;
use gpui_component::Sizable as _;
use gpui_component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit_phosphor::PhosphorIcon;
use mars_core::{CellId, CellType, ExecutionStatus, Output};
use mars_document::{Cell, Notebook, load, save_atomic};
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

const BG: u32 = 0x141414;
const PANEL: u32 = 0x1c1c1c;
const HEADER: u32 = 0x202020;
const BORDER: u32 = 0x2a2a2a;
const TEXT: u32 = 0xe5e5e5;
const MUTED: u32 = 0x9ca3af;
const FAINT: u32 = 0x6b7280;
const ACCENT: u32 = 0xf97316;
const CODE_BG: u32 = 0x1f2937;
const ERROR_BG: u32 = 0x2a1518;
const ERROR_BORDER: u32 = 0xef4444;

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
    subscriptions: Vec<Subscription>,
    dirty: bool,
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
            subscriptions: Vec::new(),
            dirty: false,
        };
        if let Some(path) = path {
            this.open_path(PathBuf::from(path), cx);
        }
        this.ensure_editors(window, cx);
        this
    }

    fn ensure_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let missing = self
            .notebook
            .cells
            .iter()
            .filter(|cell| !self.editors.contains_key(&cell.id))
            .map(|cell| (cell.id.clone(), cell.source.clone()))
            .collect::<Vec<_>>();
        for (id, source) in missing {
            let state = cx.new(|cx| TextareaState::new(window, cx).auto_grow(1, 12));
            state.update(cx, |state, cx| state.set_value(source, window, cx));
            let edit_id = id.clone();
            let subscription = cx.subscribe(&state, move |this, state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let value = state.read(cx).value().to_string();
                    this.apply_source_edit(edit_id.clone(), value);
                }
            });
            self.subscriptions.push(subscription);
            self.editors.insert(id, state);
        }
    }

    fn apply_source_edit(&mut self, id: CellId, source: String) {
        if let Some(cell) = self.notebook.cells.iter_mut().find(|cell| cell.id == id)
            && cell.source != source
        {
            cell.source = source;
            self.dirty = true;
            if cell.cell_type == CellType::Code {
                cell.outputs.clear();
                cell.execution_count = None;
            }
        }
    }

    fn reset_editors(&mut self) {
        self.editors.clear();
        self.subscriptions.clear();
    }

    fn new_notebook(&mut self, cx: &mut Context<Self>) {
        let mut notebook = Notebook::new();
        let _ = notebook.add_cell(Cell::new(CellType::Markdown, "# New Mars notebook"));
        self.configure_kernel(&notebook);
        self.notebook = notebook;
        self.path = None;
        self.reset_editors();
        self.status = "New notebook".to_owned();
        cx.notify();
    }

    fn prompt_open(&mut self, cx: &mut Context<Self>) {
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

    fn open_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
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
                self.kernel = Some(Arc::new(Mutex::new(kernel)));
                self.kernel_status = ExecutionStatus::Idle;
            }
            Err(error) => {
                self.kernel = None;
                self.status = format!("Kernel configuration failed: {error}");
            }
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
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

    fn execute_cell(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.executing
            || !self
                .notebook
                .cells
                .get(index)
                .is_some_and(|cell| cell.cell_type == CellType::Code)
        {
            return;
        }
        let source = self.notebook.cells[index].source.clone();
        self.executing = true;
        self.status = format!("Running cell {}", index + 1);
        cx.notify();
        self.spawn_execution(vec![(index, source)], cx);
    }

    fn run_all(&mut self, cx: &mut Context<Self>) {
        let jobs = self
            .notebook
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.cell_type == CellType::Code)
            .map(|(index, cell)| (index, cell.source.clone()))
            .collect::<Vec<_>>();
        if jobs.is_empty() {
            self.status = "No code cells to run".to_owned();
            cx.notify();
            return;
        }
        self.executing = true;
        self.status = format!("Running {} code cells", jobs.len());
        cx.notify();
        self.spawn_execution(jobs, cx);
    }

    fn spawn_execution(&mut self, jobs: Vec<(usize, String)>, cx: &mut Context<Self>) {
        let Some(kernel) = self.kernel.clone() else {
            self.executing = false;
            self.status = "Kernel unavailable".to_owned();
            cx.notify();
            return;
        };
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let results = executor
                .spawn(async move {
                    let Ok(mut kernel) = kernel.lock() else {
                        return Vec::new();
                    };
                    jobs.into_iter()
                        .map(|(index, source)| {
                            let response = kernel.handle_with_cancellation(
                                RequestEnvelope::new(KernelRequest::Execute {
                                    source,
                                    timeout_millis: None,
                                }),
                                Arc::new(AtomicBool::new(false)),
                            );
                            (index, response.response)
                        })
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.executing = false;
                for (index, response) in results {
                    this.apply_execution(index, response);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_execution(&mut self, index: usize, response: KernelResponse) {
        match response {
            KernelResponse::Execution {
                outputs,
                execution_count,
                duration_millis,
            } => {
                if let Some(cell) = self.notebook.cells.get_mut(index) {
                    cell.execution_count = Some(execution_count);
                    cell.outputs = outputs.clone();
                }
                let failed = outputs
                    .iter()
                    .any(|output| matches!(output, Output::Error { .. }));
                self.kernel_status = if failed {
                    ExecutionStatus::Failed
                } else {
                    ExecutionStatus::Succeeded
                };
                self.status = format!(
                    "Cell {} {} in {} ms",
                    index + 1,
                    if failed { "failed" } else { "succeeded" },
                    duration_millis
                );
            }
            KernelResponse::Error { message } => {
                self.kernel_status = ExecutionStatus::Failed;
                self.status = format!("Cell {} error: {}", index + 1, message);
            }
            _ => {}
        }
    }

    fn restart_kernel(&mut self, cx: &mut Context<Self>) {
        if let Some(kernel) = &self.kernel
            && let Ok(mut kernel) = kernel.lock()
        {
            match kernel
                .handle(RequestEnvelope::new(KernelRequest::Reset))
                .response
            {
                KernelResponse::Reset { status } => {
                    self.kernel_status = status;
                    self.status = "Kernel restarted".to_owned();
                }
                KernelResponse::Error { message } => {
                    self.status = format!("Restart failed: {message}")
                }
                _ => {}
            }
        } else {
            self.status = "Kernel unavailable".to_owned();
        }
        cx.notify();
    }

    fn add_cell(&mut self, cell_type: CellType, cx: &mut Context<Self>) {
        let index = self.notebook.cells.len();
        self.insert_cell(index, cell_type, cx);
    }

    fn insert_cell(&mut self, index: usize, cell_type: CellType, cx: &mut Context<Self>) {
        match self.notebook.insert_cell(index, Cell::new(cell_type, "")) {
            Ok(()) => {
                self.dirty = true;
                self.status = format!("Inserted cell at {}", index + 1);
                cx.notify();
            }
            Err(error) => {
                self.status = format!("Insert failed: {error}");
                cx.notify();
            }
        }
    }

    fn delete_cell(&mut self, index: usize, cx: &mut Context<Self>) {
        match self.notebook.remove_cell(index) {
            Ok(_) => {
                self.dirty = true;
                self.status = format!("Deleted cell {}", index + 1);
                cx.notify();
            }
            Err(error) => {
                self.status = format!("Delete failed: {error}");
                cx.notify();
            }
        }
    }

    fn move_cell(&mut self, index: usize, delta: isize, cx: &mut Context<Self>) {
        let to = index as isize + delta;
        if to < 0 || to as usize >= self.notebook.cells.len() {
            return;
        }
        match self.notebook.move_cell(index, to as usize) {
            Ok(()) => {
                self.dirty = true;
                cx.notify();
            }
            Err(error) => self.status = format!("Move failed: {error}"),
        }
    }

    fn add_cell_below(&mut self, index: usize, cx: &mut Context<Self>) {
        let cell_type = self
            .notebook
            .cells
            .get(index)
            .map(|cell| cell.cell_type.clone())
            .unwrap_or(CellType::Code);
        self.insert_cell(index + 1, cell_type, cx);
    }

    fn duplicate_cell(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(cell) = self.notebook.cells.get(index) else {
            return;
        };
        let mut copy = Cell::new(cell.cell_type.clone(), cell.source.clone());
        copy.metadata = cell.metadata.clone();
        match self.notebook.insert_cell(index + 1, copy) {
            Ok(()) => {
                self.dirty = true;
                self.status = format!("Duplicated cell {}", index + 1);
                cx.notify();
            }
            Err(error) => self.status = format!("Duplicate failed: {error}"),
        }
    }

    fn clear_outputs(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Ok(cell) = self.notebook.cell_mut(index) {
            cell.outputs.clear();
            cell.execution_count = None;
            self.dirty = true;
        }
        cx.notify();
    }

    fn toggle_cell_type(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(cell) = self.notebook.cells.get(index) else {
            return;
        };
        let new_type = match cell.cell_type {
            CellType::Code => CellType::Markdown,
            _ => CellType::Code,
        };
        let clears_outputs = matches!(new_type, CellType::Markdown);
        match self.notebook.cell_mut(index) {
            Ok(cell) => {
                cell.cell_type = new_type;
                if clears_outputs {
                    cell.outputs.clear();
                    cell.execution_count = None;
                }
                self.dirty = true;
                cx.notify();
            }
            Err(error) => self.status = format!("Type change failed: {error}"),
        }
    }
}

impl Render for NotebookApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_editors(window, cx);
        let on_new = cx.listener(|this: &mut Self, _: &NewNotebook, _, cx| this.new_notebook(cx));
        let on_open = cx.listener(|this: &mut Self, _: &OpenNotebook, _, cx| this.prompt_open(cx));
        let on_save = cx.listener(|this: &mut Self, _: &SaveNotebook, _, cx| this.save(cx));
        let on_run_all = cx.listener(|this: &mut Self, _: &RunAllCells, _, cx| this.run_all(cx));
        let on_restart =
            cx.listener(|this: &mut Self, _: &RestartKernel, _, cx| this.restart_kernel(cx));
        let on_quit = cx.listener(|_: &mut Self, _: &Quit, _, cx| cx.quit());

        let entity = cx.entity();
        let cells = self
            .notebook
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| self.render_cell(index, cell, &entity))
            .collect::<Vec<_>>();

        div()
            .id("root")
            .track_focus(&self.focus_handle)
            .on_action(on_new)
            .on_action(on_open)
            .on_action(on_save)
            .on_action(on_run_all)
            .on_action(on_restart)
            .on_action(on_quit)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .child(self.render_toolbar(&entity))
            .child(
                div()
                    .id("cells")
                    .flex_1()
                    .overflow_y_scroll()
                    .px_4()
                    .py_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .children(cells),
            )
            .child(self.render_statusbar())
    }
}

impl NotebookApp {
    fn render_toolbar(&self, entity: &Entity<Self>) -> Div {
        let title = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_owned());
        let title = if self.dirty {
            format!("• {title}")
        } else {
            title
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .bg(rgb(PANEL))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Mars"),
                    )
                    .child(div().text_sm().text_color(rgb(MUTED)).child(title)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(toolbar_button(
                        PhosphorIcon::FilePlus,
                        "New",
                        "tb-new",
                        entity,
                        |this, cx| this.new_notebook(cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::FolderOpen,
                        "Open…",
                        "tb-open",
                        entity,
                        |this, cx| this.prompt_open(cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::FloppyDisk,
                        "Save",
                        "tb-save",
                        entity,
                        |this, cx| this.save(cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::FileCode,
                        "Code cell",
                        "tb-add-code",
                        entity,
                        |this, cx| this.add_cell(CellType::Code, cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::FileMd,
                        "Markdown cell",
                        "tb-add-md",
                        entity,
                        |this, cx| this.add_cell(CellType::Markdown, cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::Play,
                        "Run all",
                        "tb-run",
                        entity,
                        |this, cx| this.run_all(cx),
                    ))
                    .child(toolbar_button(
                        PhosphorIcon::ArrowCounterClockwise,
                        "Restart kernel",
                        "tb-restart",
                        entity,
                        |this, cx| this.restart_kernel(cx),
                    )),
            )
    }

    fn render_statusbar(&self) -> Div {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_1()
            .bg(rgb(PANEL))
            .child(
                div()
                    .text_xs()
                    .text_color(if self.executing {
                        rgb(ACCENT)
                    } else {
                        rgb(MUTED)
                    })
                    .child(self.status.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(FAINT))
                    .child(format!("kernel: {:?}", self.kernel_status)),
            )
    }

    fn render_cell(&self, index: usize, cell: &Cell, entity: &Entity<Self>) -> Div {
        let is_code = cell.cell_type == CellType::Code;
        let type_label = match cell.cell_type {
            CellType::Markdown => "Markdown",
            CellType::Code => "Rust",
            CellType::Raw => "Raw",
        };
        let accent = if is_code { ACCENT } else { FAINT };
        let execution_label = cell
            .execution_count
            .map(|count| format!("[{count}]"))
            .unwrap_or_default();

        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .bg(rgb(HEADER))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().text_xs().text_color(rgb(accent)).child(type_label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(execution_label),
                    ),
            );
        let type_icon = if is_code {
            PhosphorIcon::FileCode
        } else {
            PhosphorIcon::FileMd
        };

        let mut controls = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(cell_icon_button(
                type_icon,
                format!("type-{index}").into(),
                entity,
                index,
                |this, index, cx| this.toggle_cell_type(index, cx),
            ))
            .child(cell_icon_button(
                PhosphorIcon::ArrowUp,
                format!("up-{index}").into(),
                entity,
                index,
                |this, index, cx| this.move_cell(index, -1, cx),
            ))
            .child(cell_icon_button(
                PhosphorIcon::Plus,
                format!("add-{index}").into(),
                entity,
                index,
                |this, index, cx| this.add_cell_below(index, cx),
            ))
            .child(cell_icon_button(
                PhosphorIcon::Copy,
                format!("dup-{index}").into(),
                entity,
                index,
                |this, index, cx| this.duplicate_cell(index, cx),
            ))
            .child(cell_icon_button(
                PhosphorIcon::ArrowDown,
                format!("down-{index}").into(),
                entity,
                index,
                |this, index, cx| this.move_cell(index, 1, cx),
            ))
            .child(cell_icon_button(
                PhosphorIcon::Trash,
                format!("del-{index}").into(),
                entity,
                index,
                |this, index, cx| this.delete_cell(index, cx),
            ));
        if is_code {
            controls = controls
                .child(cell_icon_button(
                    PhosphorIcon::Eraser,
                    format!("clear-{index}").into(),
                    entity,
                    index,
                    |this, index, cx| this.clear_outputs(index, cx),
                ))
                .child(cell_icon_button_accent(
                    PhosphorIcon::Play,
                    format!("exec-{index}").into(),
                    entity,
                    index,
                    |this, index, cx| this.execute_cell(index, cx),
                ));
        }
        header = header.child(controls);

        let body = if let Some(state) = self.editors.get(&cell.id) {
            div().px_3().py_2().child(
                Textarea::new(state)
                    .appearance(false)
                    .bordered(false)
                    .small(),
            )
        } else {
            div()
                .px_3()
                .py_2()
                .text_sm()
                .font_family("Menlo")
                .text_color(rgb(TEXT))
                .child(cell.source.clone())
        };

        let mut card = div()
            .flex()
            .flex_col()
            .rounded_lg()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(PANEL))
            .overflow_hidden()
            .child(header)
            .child(body);

        for output in &cell.outputs {
            card = card.child(render_output(output));
        }
        card
    }
}

fn toolbar_button(
    icon: PhosphorIcon,
    label: &'static str,
    id: &'static str,
    entity: &Entity<NotebookApp>,
    action: impl Fn(&mut NotebookApp, &mut Context<NotebookApp>) + 'static,
) -> Stateful<Div> {
    let entity = entity.clone();
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .bg(rgb(0x262626))
        .text_sm()
        .text_color(rgb(TEXT))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x333333)))
        .on_click(move |_, _, cx| {
            entity.update(cx, |this, cx| action(this, cx));
        })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(icon)
                .child(label),
        )
}

fn cell_icon_button(
    icon: PhosphorIcon,
    id: SharedString,
    entity: &Entity<NotebookApp>,
    index: usize,
    action: impl Fn(&mut NotebookApp, usize, &mut Context<NotebookApp>) + 'static,
) -> Stateful<Div> {
    icon_button(icon, id, entity, index, action, rgb(MUTED))
}

fn cell_icon_button_accent(
    icon: PhosphorIcon,
    id: SharedString,
    entity: &Entity<NotebookApp>,
    index: usize,
    action: impl Fn(&mut NotebookApp, usize, &mut Context<NotebookApp>) + 'static,
) -> Stateful<Div> {
    icon_button(icon, id, entity, index, action, rgb(ACCENT))
}

fn icon_button(
    icon: PhosphorIcon,
    id: SharedString,
    entity: &Entity<NotebookApp>,
    index: usize,
    action: impl Fn(&mut NotebookApp, usize, &mut Context<NotebookApp>) + 'static,
    color: Rgba,
) -> Stateful<Div> {
    let entity = entity.clone();
    div()
        .id(id)
        .p_1()
        .rounded_md()
        .text_xs()
        .text_color(color)
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x333333)))
        .on_click(move |_, _, cx| {
            entity.update(cx, |this, cx| action(this, index, cx));
        })
        .child(icon)
}

fn render_output(output: &Output) -> Div {
    let lines = output_lines(output);
    let (bg, border, color) = match output {
        Output::Error { .. } => (ERROR_BG, Some(ERROR_BORDER), 0xfecaca),
        Output::Text { .. } => (CODE_BG, None, 0xd1d5db),
        _ => (CODE_BG, None, MUTED),
    };
    let mut block = div()
        .mx_3()
        .mb_2()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(bg))
        .text_xs()
        .font_family("Menlo")
        .text_color(rgb(color))
        .child(lines.join("\n"));
    if let Some(border) = border {
        block = block.border_1().border_color(rgb(border));
    }
    block
}
