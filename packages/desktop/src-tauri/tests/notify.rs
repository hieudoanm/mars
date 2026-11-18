use mars_tauri_lib::notify::notify_unfocused;

fn app() -> tauri::App<tauri::test::MockRuntime> {
    tauri::test::mock_builder()
        .plugin(tauri_plugin_notification::init())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("builds mock app")
}

#[test]
fn notify_unfocused_is_best_effort() {
    let app = app();
    // The mock runtime has no "main" window, so the helper proceeds to
    // `show()`, whose result is ignored; the long body also exercises
    // truncation.
    notify_unfocused(app.handle(), "Mars", "done".to_owned());
    notify_unfocused(app.handle(), "Mars", "x".repeat(500));
}
