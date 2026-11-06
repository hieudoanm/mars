use super::NotebookApp;
use gpui::*;
use mars_core::{CellType, ExecutionStatus, Output};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

impl NotebookApp {
    pub(super) fn execute_cell(&mut self, index: usize, cx: &mut Context<Self>) {
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
        self.running_cells.insert(index);
        self.status = format!("Running cell {}", index + 1);
        cx.notify();
        self.spawn_execution(vec![(index, source)], cx);
    }

    pub(super) fn run_all(&mut self, cx: &mut Context<Self>) {
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
        for (index, _) in &jobs {
            self.running_cells.insert(*index);
        }
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
        let (tx, rx) = mpsc::channel::<(usize, KernelResponse)>();
        executor
            .spawn(async move {
                let Ok(mut kernel) = kernel.lock() else {
                    return;
                };
                for (index, source) in jobs {
                    let response = kernel.handle_with_cancellation(
                        RequestEnvelope::new(KernelRequest::Execute {
                            source,
                            timeout_millis: None,
                        }),
                        Arc::new(AtomicBool::new(false)),
                    );
                    if tx.send((index, response.response)).is_err() {
                        return;
                    }
                }
            })
            .detach();
        let timer_executor = executor.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                match rx.try_recv() {
                    Ok((index, response)) => {
                        let _ = this.update(cx, |this, cx| {
                            this.running_cells.remove(&index);
                            this.apply_execution(index, response);
                            cx.notify();
                        });
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }
                timer_executor.timer(Duration::from_millis(20)).await;
            }
            let _ = this.update(cx, |this, cx| {
                this.executing = false;
                this.running_cells.clear();
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
                    self.durations.insert(cell.id.clone(), duration_millis);
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

    pub(super) fn restart_kernel(&mut self, cx: &mut Context<Self>) {
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
}
