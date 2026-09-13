//! The one place a window is opened. Three callers — the first launch, the tray, a second
//! launch of the executable — and one recipe, the entry in `tauri.conf.json`, so the window
//! the tray gives back is the window the app started with.

use ipc::{tray_action, TrayAction};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};

/// Brings a window forward, or builds the main one if there is none.
///
/// Infallible on the outside: this is called from a tray click and from the event loop, and a
/// window that refuses to open is a click that did nothing, never a process that dies.
pub fn open_or_focus(app: &AppHandle) {
    let labels: Vec<String> = app.webview_windows().keys().cloned().collect();
    match tray_action(&labels) {
        TrayAction::Focus { label } => {
            if let Some(w) = app.get_webview_window(&label) {
                // Three states, three calls: a window can be minimized, hidden, or merely
                // behind another, and only the third is what `set_focus` alone fixes.
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        TrayAction::CreateMain => {
            // The recipe is the config's first window — size, title, the missing decorations,
            // the background colour. Cloned because the builder wants it by reference and the
            // handle is borrowed for the length of the call.
            let Some(config) = app.config().app.windows.first().cloned() else {
                return;
            };
            if let Ok(builder) = WebviewWindowBuilder::from_config(app, &config) {
                let _ = builder.build();
            }
        }
    }
}
