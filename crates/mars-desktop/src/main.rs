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
        Quit
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
