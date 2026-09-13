//! Profile selection and settings: the two commands that write, and the two that read them back.

use tauri::AppHandle;

use discovery::{discover, Options};
use ipc::{IpcError, ProfileId, Settings, SetupState};

use crate::events::{announce, PROFILE_CHANGED, SETTINGS_CHANGED};
use crate::settings_file;
use crate::state::StoreState;

#[tauri::command]
pub fn setup_state(app: AppHandle) -> Result<SetupState, IpcError> {
    let settings = settings_file::load(&app);
    let d = discover(&Options::default());
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

#[tauri::command]
pub fn select_profile(app: AppHandle, id: ProfileId) -> Result<SetupState, IpcError> {
    let d = discover(&Options::default());
    let views = ipc::candidates(&d.saves);
    if !views.iter().any(|c| c.id == id) {
        return Err(IpcError::UnknownProfile {
            id: id.as_str().to_string(),
        });
    }
    // Load the existing settings and update only the field that changed: since the
    // interface's size joined them, switching profile would otherwise put it back to 100.
    let settings = Settings {
        active_profile_id: Some(id),
        ..settings_file::load(&app)
    };
    settings_file::save(&app, &settings)?;
    // The active profile is the app's, not this window's, now that there can be more than one.
    announce(&app, PROFILE_CHANGED);
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

/// The persisted settings, as the app will act on them: the scale comes back snapped to the
/// ladder, so a hand-edited file never puts the interface at a size nothing was drawn at.
#[tauri::command]
pub fn settings(app: AppHandle) -> Result<Settings, IpcError> {
    let stored = settings_file::load(&app);
    Ok(stored.with_scale(stored.scale()))
}

/// Changes the interface's size and answers the settings as they now are — the same shape as
/// `select_profile`, which also writes and answers. The value is snapped before it reaches
/// the file: what we write is always a size we drew.
#[tauri::command]
pub fn set_scale(app: AppHandle, percent: u16) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_scale(percent);
    settings_file::save(&app, &settings)?;
    // One interface, one size: the other windows resize with this one.
    announce(&app, SETTINGS_CHANGED);
    Ok(settings)
}

/// Whether the app stays in the notification area when the last window closes.
#[tauri::command]
pub fn set_stay_in_background(app: AppHandle, stay: bool) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_stay_in_background(stay);
    settings_file::save(&app, &settings)?;
    announce(&app, SETTINGS_CHANGED);
    Ok(settings)
}

/// Whether a window born with nothing owed to it opens on the last session's tabs.
///
/// Turning it off clears what was stored: the app should not keep a record the user has just
/// said they don't want. A store that won't open is not a reason to refuse the setting —
/// nothing will be read back either way.
#[tauri::command]
pub fn set_resume_tabs(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    resume: bool,
) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_resume_tabs(resume);
    settings_file::save(&app, &settings)?;
    if !resume {
        if let Ok(guard) = store.lock(&app) {
            let _ = guard.set_session(None);
        }
    }
    announce(&app, SETTINGS_CHANGED);
    Ok(settings)
}
