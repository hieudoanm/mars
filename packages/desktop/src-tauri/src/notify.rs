//! Desktop notifications via `tauri-plugin-notification`.
//!
//! Notifications are only sent while the main window is unfocused — if the
//! user is already looking at the app, the in-UI progress events carry the
//! same information. All calls are best-effort: notification failures
//! (missing D-Bus session, unsigned-app quirks) are ignored.

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Show a desktop notification with the given title/body, unless the main
/// window is focused. Bodies are truncated to keep notifications readable.
pub fn notify_unfocused<R: tauri::Runtime>(app: &AppHandle<R>, title: &str, body: String) {
    if window_focused(app) {
        return;
    }
    let body: String = body.chars().take(200).collect();
    let _ = app.notification().builder().title(title).body(body).show();
}

fn window_focused<R: tauri::Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.is_focused().ok())
        .unwrap_or(false)
}
