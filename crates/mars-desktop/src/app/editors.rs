use super::NotebookApp;
use gpui::*;
use gpui_component::input::{EditorState, InputEvent, TextareaState};
use mars_core::{CellId, CellType};

impl NotebookApp {
    pub(super) fn ensure_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for cell in &self.notebook.cells {
            self.scroll_anchors
                .entry(cell.id.clone())
                .or_insert_with(|| ScrollAnchor::for_handle(self.scroll_handle.clone()));
        }
        self.scroll_anchors
            .retain(|id, _| self.notebook.cells.iter().any(|cell| &cell.id == id));
        let missing_text = self
            .notebook
            .cells
            .iter()
            .filter(|cell| cell.cell_type != CellType::Code)
            .filter(|cell| !self.editors.contains_key(&cell.id))
            .map(|cell| (cell.id.clone(), cell.source.clone()))
            .collect::<Vec<_>>();
        for (id, source) in missing_text {
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

        let missing_code = self
            .notebook
            .cells
            .iter()
            .filter(|cell| cell.cell_type == CellType::Code)
            .filter(|cell| !self.code_editors.contains_key(&cell.id))
            .map(|cell| (cell.id.clone(), cell.source.clone()))
            .collect::<Vec<_>>();
        for (id, source) in missing_code {
            let state = cx.new(|cx| {
                EditorState::new(window, cx)
                    .language("rust")
                    .line_number(false)
            });
            state.update(cx, |state, cx| state.set_value(source, window, cx));
            let edit_id = id.clone();
            let subscription = cx.subscribe(&state, move |this, state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let value = state.read(cx).value().to_string();
                    this.apply_source_edit(edit_id.clone(), value);
                }
            });
            self.subscriptions.push(subscription);
            self.code_editors.insert(id, state);
        }
    }

    pub(super) fn apply_source_edit(&mut self, id: CellId, source: String) {
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

    pub(super) fn reset_editors(&mut self) {
        self.editors.clear();
        self.code_editors.clear();
        self.subscriptions.clear();
    }

    pub(super) fn focus_selected_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(cell) = self.notebook.cells.get(self.selected_cell) else {
            return;
        };
        let handle = if cell.cell_type == CellType::Code {
            self.code_editors
                .get(&cell.id)
                .map(|state| state.read(cx).focus_handle(cx))
        } else {
            self.editors
                .get(&cell.id)
                .map(|state| state.read(cx).focus_handle(cx))
        };
        if let Some(handle) = handle {
            window.focus(&handle, cx);
        }
    }
}
