use super::NotebookApp;
use super::{ACCENT, MUTED, TEXT};
use gpui::*;
use gpui_kit_phosphor::PhosphorIcon;

pub(super) fn toolbar_button(
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

pub(super) fn cell_icon_button(
    icon: PhosphorIcon,
    id: SharedString,
    entity: &Entity<NotebookApp>,
    index: usize,
    action: impl Fn(&mut NotebookApp, usize, &mut Context<NotebookApp>) + 'static,
) -> Stateful<Div> {
    icon_button(icon, id, entity, index, action, rgb(MUTED))
}

pub(super) fn cell_icon_button_accent(
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
