//! Profile selection and settings: the two commands that write, and the two that read them back.

use tauri::AppHandle;

use discovery::{discover, Options};
use ipc::{IpcError, ProfileId, Settings, SetupState};

use crate::settings_file;

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
    Ok(settings)
}
