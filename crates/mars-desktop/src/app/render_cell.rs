use super::NotebookApp;
use super::render_output::render_output;
use super::widgets::{cell_icon_button, cell_icon_button_accent};
use super::{ACCENT, BORDER, FAINT, HEADER, MUTED, PANEL, TEXT};
use crate::markdown_view::markdown_preview;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::Sizable as _;
use gpui_component::input::{Editor, Textarea};
use gpui_kit_phosphor::PhosphorIcon;
use mars_core::CellType;
use mars_document::Cell;

impl NotebookApp {
    pub(super) fn render_cell(
        &self,
        index: usize,
        cell: &Cell,
        entity: &Entity<Self>,
    ) -> AnyElement {
        let is_code = cell.cell_type == CellType::Code;
        let type_label = match cell.cell_type {
            CellType::Markdown => "Markdown",
            CellType::Code => "Rust",
            CellType::Raw => "Raw",
        };
        let accent = if is_code { ACCENT } else { FAINT };
        let execution_label = cell
            .execution_count
            .map(|count| {
                self.durations
                    .get(&cell.id)
                    .map(|ms| format!("[{count}] {ms} ms"))
                    .unwrap_or_else(|| format!("[{count}]"))
            })
            .unwrap_or_default();

        let is_running = self.running_cells.contains(&index);
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
                    )
                    .when(is_running, |this| {
                        this.child(div().text_xs().text_color(rgb(ACCENT)).child("running…"))
                    }),
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
        if !is_code {
            controls = controls.child(cell_icon_button(
                if self.preview_cells.contains(&cell.id) {
                    PhosphorIcon::EyeSlash
                } else {
                    PhosphorIcon::Eye
                },
                format!("preview-{index}").into(),
                entity,
                index,
                |this, index, cx| this.toggle_preview(index, cx),
            ));
        }
        header = header.child(controls);

        let body = if !is_code && self.preview_cells.contains(&cell.id) {
            div()
                .px_3()
                .py_2()
                .text_sm()
                .text_color(rgb(TEXT))
                .child(markdown_preview(&cell.source))
        } else if is_code && let Some(state) = self.code_editors.get(&cell.id) {
            div()
                .px_3()
                .py_2()
                .child(Editor::new(state).appearance(false).bordered(false))
        } else if !is_code && let Some(state) = self.editors.get(&cell.id) {
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

        let is_selected = self.command_mode && index == self.selected_cell;
        let card_entity = entity.clone();
        let mut card = div()
            .id(SharedString::from(format!("cell-card-{}", cell.id)))
            .flex()
            .flex_col()
            .rounded_lg()
            .border_1()
            .border_color(rgb(if is_selected { ACCENT } else { BORDER }))
            .bg(rgb(PANEL))
            .overflow_hidden()
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                card_entity.update(cx, |this, cx| this.select_cell(index, cx));
            })
            .anchor_scroll(self.scroll_anchors.get(&cell.id).cloned())
            .child(header)
            .child(body);

        for output in &cell.outputs {
            card = card.child(render_output(output));
        }
        card.into_any_element()
    }
}
