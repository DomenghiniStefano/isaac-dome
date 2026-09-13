//! The window session: one opaque document, written by the main window as its tabs change and
//! read by the main window when it is born. Its shape is the frontend's — a route name and a
//! query are not things this side has a type for — so the only judgements made here are the
//! setting and the cap.

use tauri::AppHandle;

use ipc::{session_document_fits, IpcError};
use store::{store_error, store_unavailable};

use crate::settings_file;
use crate::state::StoreState;

/// The stored session, or `None`. **`None` when the setting is off**, whatever the database
/// holds: "off" is answered in one place rather than in every caller.
///
/// An unreadable store is `None` too, not an error: a session that can't be read is a landing
/// tab, never a dialog.
#[tauri::command]
pub fn window_session(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
) -> Result<Option<String>, IpcError> {
    if !settings_file::load(&app).resume_tabs {
        return Ok(None);
    }
    Ok(store
        .lock(&app)
        .ok()
        .and_then(|guard| guard.session().ok())
        .flatten())
}

/// Replaces the session, or clears it with `None`. Silently does nothing when the setting is
/// off — the frontend shouldn't have to check twice — and refuses a document past the cap.
#[tauri::command]
pub fn set_window_session(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    document: Option<String>,
) -> Result<(), IpcError> {
    if !settings_file::load(&app).resume_tabs {
        return Ok(());
    }
    if document
        .as_deref()
        .is_some_and(|d| !session_document_fits(d))
    {
        return Err(IpcError::SessionTooLarge);
    }
    let guard = store.lock(&app).map_err(store_unavailable)?;
    guard.set_session(document.as_deref()).map_err(store_error)
}
