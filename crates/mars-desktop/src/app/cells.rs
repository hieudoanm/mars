use super::NotebookApp;
use gpui::Context;
use mars_core::CellType;
use mars_document::Cell;

impl NotebookApp {
    pub(super) fn add_cell(&mut self, cell_type: CellType, cx: &mut Context<Self>) {
        let index = self.notebook.cells.len();
        self.insert_cell(index, cell_type, cx);
    }

    pub(super) fn insert_cell(
        &mut self,
        index: usize,
        cell_type: CellType,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn delete_cell(&mut self, index: usize, cx: &mut Context<Self>) {
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

    pub(super) fn move_cell(&mut self, index: usize, delta: isize, cx: &mut Context<Self>) {
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

    pub(super) fn add_cell_below(&mut self, index: usize, cx: &mut Context<Self>) {
        let cell_type = self
            .notebook
            .cells
            .get(index)
            .map(|cell| cell.cell_type.clone())
            .unwrap_or(CellType::Code);
        self.insert_cell(index + 1, cell_type, cx);
    }

    pub(super) fn duplicate_cell(&mut self, index: usize, cx: &mut Context<Self>) {
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

    pub(super) fn clear_outputs(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Ok(cell) = self.notebook.cell_mut(index) {
            cell.outputs.clear();
            cell.execution_count = None;
            self.dirty = true;
        }
        cx.notify();
    }

    pub(super) fn select_cell(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_cell = index.min(self.notebook.cells.len().saturating_sub(1));
        cx.notify();
    }

    pub(super) fn toggle_preview(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(cell) = self.notebook.cells.get(index) {
            if self.preview_cells.contains(&cell.id) {
                self.preview_cells.remove(&cell.id);
            } else {
                self.preview_cells.insert(cell.id.clone());
            }
        }
        cx.notify();
    }

    pub(super) fn set_cell_type(
        &mut self,
        index: usize,
        new_type: CellType,
        cx: &mut Context<Self>,
    ) {
        let clears_outputs = matches!(new_type, CellType::Markdown);
        match self.notebook.cell_mut(index) {
            Ok(cell) => {
                cell.cell_type = new_type.clone();
                if clears_outputs {
                    cell.outputs.clear();
                    cell.execution_count = None;
                }
                self.dirty = true;
                self.status = format!("Cell {} set to {:?}", index + 1, new_type);
                cx.notify();
            }
            Err(error) => self.status = format!("Type change failed: {error}"),
        }
    }

    pub(super) fn toggle_cell_type(&mut self, index: usize, cx: &mut Context<Self>) {
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
