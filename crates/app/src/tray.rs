//! The icon in the notification area. Always present, whether or not a window is open: the
//! app is running either way, and an icon that appears and disappears is an icon nobody
//! recognises.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use ipc::{tray_locale, tray_text, TrayText};

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// The strings the tray shows, in the system's language.
pub fn text() -> TrayText {
    tray_text(tray_locale(
        sys_locale::get_locale().unwrap_or_default().as_str(),
    ))
}

/// Builds the icon and its menu. A failure here is an app without a tray, not an app that
/// won't start: the windows still work.
pub fn build(app: &AppHandle) {
    let t = text();
    let (Ok(open), Ok(quit)) = (
        MenuItem::with_id(app, OPEN_ID, t.open, true, None::<&str>),
        MenuItem::with_id(app, QUIT_ID, t.quit, true, None::<&str>),
    ) else {
        return;
    };
    let Ok(menu) = Menu::with_items(app, &[&open, &quit]) else {
        return;
    };
    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        // The left button is the shortcut, the right one is the menu. With the default, a
        // left click would open the menu and there would be no one-click way back.
        .show_menu_on_left_click(false)
        .tooltip(t.tooltip)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => crate::window::open_or_focus(app),
            QUIT_ID => app.exit(0),
            // The menu's ids are this file's two constants; `MenuId` is a string and cannot be
            // matched exhaustively.
            _ => (),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::window::open_or_focus(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let _ = builder.build(app);
}

/// Says, once ever, that the app is still running.
///
/// **The result is ignored on purpose**: a Windows toast wants a registered AppUserModelID,
/// which an installed build has and a development run may not, and an app that refused to stay
/// in the tray because a toast didn't appear would be worse than a silent one. The tooltip
/// carries the same sentence either way.
pub fn notice_once(app: &AppHandle) {
    let settings = crate::settings_file::load(app);
    if settings.background_notice_shown {
        return;
    }
    let t = text();
    let _ = app
        .notification()
        .builder()
        .title(t.notice_title)
        .body(t.notice_body)
        .show();
    // Written whether or not it appeared: a notice that failed twice is not worth a third try.
    let _ = crate::settings_file::save(app, &settings.with_notice_shown(true));
}
