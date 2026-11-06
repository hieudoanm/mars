use super::NotebookApp;
use crate::{
    DeleteCell, EditMode, InsertAbove, InsertBelow, RunCell, RunCellAdvance, RunInsertBelow,
    SelectNext, SelectPrev, ToCode, ToMarkdown, ToggleCommandMode,
};
use gpui::*;
use mars_core::CellType;

impl NotebookApp {
    pub(super) fn toggle_command_mode(
        &mut self,
        _: &ToggleCommandMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_mode = !self.command_mode;
        if self.command_mode {
            window.focus(&self.focus_handle, cx);
        } else {
            self.focus_selected_editor(window, cx);
        }
        cx.notify();
    }

    pub(super) fn edit_mode(&mut self, _: &EditMode, window: &mut Window, cx: &mut Context<Self>) {
        self.command_mode = false;
        self.focus_selected_editor(window, cx);
        cx.notify();
    }

    fn select_adjacent(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let len = self.notebook.cells.len();
        if len == 0 {
            return;
        }
        let next = (self.selected_cell as isize + delta).clamp(0, len as isize - 1) as usize;
        self.selected_cell = next;
        self.scroll_to_selected(window, cx);
        cx.notify();
    }

    fn scroll_to_selected(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(cell) = self.notebook.cells.get(self.selected_cell)
            && let Some(anchor) = self.scroll_anchors.get(&cell.id)
        {
            anchor.scroll_to(window, cx);
        }
    }

    pub(super) fn command_insert_above(
        &mut self,
        _: &InsertAbove,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self.selected_cell;
        self.insert_cell(index, CellType::Code, cx);
        self.selected_cell = index;
    }

    pub(super) fn command_insert_below(
        &mut self,
        _: &InsertBelow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self.selected_cell;
        let cell_type = self
            .notebook
            .cells
            .get(index)
            .map(|cell| cell.cell_type.clone())
            .unwrap_or(CellType::Code);
        self.insert_cell(index + 1, cell_type, cx);
        self.selected_cell = index + 1;
    }

    pub(super) fn command_to_markdown(
        &mut self,
        _: &ToMarkdown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_cell_type(self.selected_cell, CellType::Markdown, cx);
    }

    pub(super) fn command_to_code(&mut self, _: &ToCode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_cell_type(self.selected_cell, CellType::Code, cx);
    }

    pub(super) fn command_delete(
        &mut self,
        _: &DeleteCell,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self
            .selected_cell
            .min(self.notebook.cells.len().saturating_sub(1));
        self.delete_cell(index, cx);
        self.selected_cell = index.min(self.notebook.cells.len().saturating_sub(1));
    }

    pub(super) fn command_run(&mut self, _: &RunCell, _: &mut Window, cx: &mut Context<Self>) {
        self.execute_cell(self.selected_cell, cx);
    }

    pub(super) fn command_run_advance(
        &mut self,
        _: &RunCellAdvance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.execute_cell(self.selected_cell, cx);
        if self.selected_cell + 1 < self.notebook.cells.len() {
            self.selected_cell += 1;
            self.scroll_to_selected(window, cx);
        }
        cx.notify();
    }

    pub(super) fn command_run_insert(
        &mut self,
        _: &RunInsertBelow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.execute_cell(self.selected_cell, cx);
        let index = self.selected_cell;
        let cell_type = self
            .notebook
            .cells
            .get(index)
            .map(|cell| cell.cell_type.clone())
            .unwrap_or(CellType::Code);
        self.insert_cell(index + 1, cell_type, cx);
        self.selected_cell = index + 1;
        self.scroll_to_selected(window, cx);
        cx.notify();
    }

    pub(super) fn command_select_prev(
        &mut self,
        _: &SelectPrev,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_adjacent(-1, window, cx);
    }

    pub(super) fn command_select_next(
        &mut self,
        _: &SelectNext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_adjacent(1, window, cx);
    }
}
