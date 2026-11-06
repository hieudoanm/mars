use super::NotebookApp;
use super::widgets::toolbar_button;
use super::{ACCENT, FAINT, MUTED, PANEL};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_kit_phosphor::PhosphorIcon;
use mars_core::CellType;

impl NotebookApp {
    pub(super) fn render_toolbar(&self, entity: &Entity<Self>) -> Div {
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

    pub(super) fn render_statusbar(&self) -> Div {
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
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .when(self.command_mode, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x141414))
                                .bg(rgb(ACCENT))
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .child("COMMAND"),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(if self.executing {
                                rgb(ACCENT)
                            } else {
                                rgb(MUTED)
                            })
                            .child(self.status.clone()),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(FAINT))
                    .child(format!("kernel: {:?}", self.kernel_status)),
            )
    }
}
