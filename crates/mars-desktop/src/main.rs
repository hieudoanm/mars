mod app;
mod markdown_view;
mod output_text;

use app::NotebookApp;
use gpui::*;
use gpui_kit_phosphor::PhosphorAssets;
use gpui_platform::application;

actions!(
    mars_desktop,
    [
        NewNotebook,
        OpenNotebook,
        SaveNotebook,
        RunAllCells,
        RestartKernel,
        Quit,
        ToggleCommandMode,
        EditMode,
        InsertAbove,
        InsertBelow,
        ToMarkdown,
        ToCode,
        DeleteCell,
        RunCell,
        RunCellAdvance,
        RunInsertBelow,
        SelectPrev,
        SelectNext
    ]
);

fn main() {
    mars_kernel::runtime_hook();
    application().with_assets(PhosphorAssets).run(|cx| {
        gpui_component::init(cx);
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
        cx.bind_keys([
            KeyBinding::new("cmd-n", NewNotebook, None),
            KeyBinding::new("cmd-o", OpenNotebook, None),
            KeyBinding::new("cmd-s", SaveNotebook, None),
            KeyBinding::new("cmd-shift-enter", RunAllCells, None),
            KeyBinding::new("cmd-r", RestartKernel, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("shift-enter", RunCellAdvance, None),
            KeyBinding::new("cmd-enter", RunCell, None),
            KeyBinding::new("alt-enter", RunInsertBelow, None),
            KeyBinding::new("escape", ToggleCommandMode, None),
            KeyBinding::new("a", InsertAbove, Some("MarsCommand")),
            KeyBinding::new("b", InsertBelow, Some("MarsCommand")),
            KeyBinding::new("m", ToMarkdown, Some("MarsCommand")),
            KeyBinding::new("y", ToCode, Some("MarsCommand")),
            KeyBinding::new("d d", DeleteCell, Some("MarsCommand")),
            KeyBinding::new("r", RunCell, Some("MarsCommand")),
            KeyBinding::new("s", SaveNotebook, Some("MarsCommand")),
            KeyBinding::new("enter", EditMode, Some("MarsCommand")),
            KeyBinding::new("up", SelectPrev, Some("MarsCommand")),
            KeyBinding::new("down", SelectNext, Some("MarsCommand")),
        ]);
        let bounds = Bounds::centered(None, size(px(1120.), px(760.)), cx);
        let notebook_path = std::env::args().nth(1);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| NotebookApp::new(window, cx, notebook_path)),
        ) {
            eprintln!("failed to open Mars window: {error:?}");
            std::process::exit(1);
        }
        cx.activate(true);
    });
}
