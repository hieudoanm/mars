use super::NotebookApp;
use crate::{
    DeleteCell, EditMode, InsertAbove, InsertBelow, NewNotebook, OpenNotebook, Quit, RestartKernel,
    RunAllCells, RunCell, RunCellAdvance, RunInsertBelow, SaveNotebook, SelectNext, SelectPrev,
    ToCode, ToMarkdown, ToggleCommandMode,
};
use gpui::*;

use super::{BG, MUTED, TEXT};

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
        let on_toggle_cmd =
            cx.listener(|this: &mut Self, action: &ToggleCommandMode, window, cx| {
                this.toggle_command_mode(action, window, cx)
            });
        let on_edit_mode = cx.listener(|this: &mut Self, action: &EditMode, window, cx| {
            this.edit_mode(action, window, cx)
        });
        let on_insert_above = cx.listener(|this: &mut Self, action: &InsertAbove, window, cx| {
            this.command_insert_above(action, window, cx)
        });
        let on_insert_below = cx.listener(|this: &mut Self, action: &InsertBelow, window, cx| {
            this.command_insert_below(action, window, cx)
        });
        let on_to_md = cx.listener(|this: &mut Self, action: &ToMarkdown, window, cx| {
            this.command_to_markdown(action, window, cx)
        });
        let on_to_code = cx.listener(|this: &mut Self, action: &ToCode, window, cx| {
            this.command_to_code(action, window, cx)
        });
        let on_delete = cx.listener(|this: &mut Self, action: &DeleteCell, window, cx| {
            this.command_delete(action, window, cx)
        });
        let on_run = cx.listener(|this: &mut Self, action: &RunCell, window, cx| {
            this.command_run(action, window, cx)
        });
        let on_run_adv = cx.listener(|this: &mut Self, action: &RunCellAdvance, window, cx| {
            this.command_run_advance(action, window, cx)
        });
        let on_run_ins = cx.listener(|this: &mut Self, action: &RunInsertBelow, window, cx| {
            this.command_run_insert(action, window, cx)
        });
        let on_prev = cx.listener(|this: &mut Self, action: &SelectPrev, window, cx| {
            this.command_select_prev(action, window, cx)
        });
        let on_next = cx.listener(|this: &mut Self, action: &SelectNext, window, cx| {
            this.command_select_next(action, window, cx)
        });

        let entity = cx.entity();
        let cells = self
            .notebook
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| self.render_cell(index, cell, &entity))
            .collect::<Vec<_>>();
        let body = if cells.is_empty() {
            div()
                .id("empty-state")
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .child("No cells yet — add a Markdown or Code cell to get started."),
                )
        } else {
            // gpui-pre 0.3.9 fails to paint children of a scrollable element when
            // they are direct siblings; wrap them in a single content div (the
            // same structure gpui-component's Scrollable uses).
            let content = div()
                .id("cells-content")
                .flex_none()
                .h_auto()
                .min_h_full()
                .flex()
                .flex_col()
                .gap_4()
                .px_4()
                .py_4()
                .children(cells);
            div()
                .id("cells")
                .flex_1()
                .overflow_y_scroll()
                .track_scroll(&self.scroll_handle)
                .child(content)
        };

        div()
            .id("root")
            .track_focus(&self.focus_handle)
            .key_context(if self.command_mode {
                "MarsCommand"
            } else {
                "MarsNotebook"
            })
            .on_action(on_new)
            .on_action(on_open)
            .on_action(on_save)
            .on_action(on_run_all)
            .on_action(on_restart)
            .on_action(on_quit)
            .on_action(on_toggle_cmd)
            .on_action(on_edit_mode)
            .on_action(on_insert_above)
            .on_action(on_insert_below)
            .on_action(on_to_md)
            .on_action(on_to_code)
            .on_action(on_delete)
            .on_action(on_run)
            .on_action(on_run_adv)
            .on_action(on_run_ins)
            .on_action(on_prev)
            .on_action(on_next)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .child(self.render_toolbar(&entity))
            .child(body)
            .child(self.render_statusbar())
    }
}
