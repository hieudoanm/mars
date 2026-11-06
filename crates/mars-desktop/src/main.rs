mod app;
mod output_text;

use app::NotebookApp;
use gpui::*;

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
    Application::new().run(|cx| {
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
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| NotebookApp::new(window, cx, notebook_path)),
        )
        .expect("failed to open Mars window");
        cx.activate(true);
    });
}
