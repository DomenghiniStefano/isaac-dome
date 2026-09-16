//! Profile selection and settings: the two commands that write, and the two that read them back.

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::AutoLaunchManager;

use discovery::{discover, Options};
use ipc::{
    AutostartFailure, AutostartReason, AutostartView, IpcError, ProfileId, Settings, SetupState,
};

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

/// The switch's position, read from the registry every time it is asked for.
///
/// **The registry is the only source of truth**: `settings.json` holds nothing about this, so
/// there is nothing to go stale the first time somebody turns the entry off from Task Manager's
/// Startup tab — which is where a Windows user turns these things off, and which the plugin can
/// see them do.
#[tauri::command]
pub fn autostart(app: AppHandle) -> Result<AutostartView, IpcError> {
    Ok(read_autostart(&app))
}

/// Writes, reads back, and answers **the read**.
///
/// A write a policy or an antivirus silently undid reports as off, which is the only honest
/// position for a switch about a login that will not happen. It also makes the plugin's own
/// defect a non-case: `disable()` on a value that is not there returns an error, and the
/// read-back says `false`, which is what was asked for.
#[tauri::command]
pub fn set_autostart(app: AppHandle, on: bool) -> Result<AutostartView, IpcError> {
    // **Whether the write was accepted is kept**, and it is the whole difference between the
    // two failures. The plugin's error itself is dropped: it is a bare string built from
    // `e.to_string()` and it can carry the executable's path, the Windows username with it.
    let accepted = match app.try_state::<AutoLaunchManager>() {
        Some(manager) => {
            let wrote = if on {
                manager.enable()
            } else {
                manager.disable()
            };
            // `disable()` on a value that is not there is an error in the plugin and means
            // nothing here: the read-back below says `false`, which is what was asked for.
            wrote.is_ok() || !on
        }
        None => false,
    };
    let view = read_autostart(&app);
    match view.unavailable {
        // The switch cannot be offered, and saying so is a better answer than a write error
        // about a registry nobody could read in the first place.
        Some(_) => Ok(view),
        None if view.enabled == on => Ok(view),
        None => Err(IpcError::AutostartNotWritable {
            reason: if accepted {
                AutostartFailure::WriteIgnored
            } else {
                AutostartFailure::WriteRefused
            },
        }),
    }
}

/// `try_state`, never `app.autolaunch()`: that helper is `state::<AutoLaunchManager>()`, which
/// **panics** when the plugin is not registered — and a development build registers none. "Never
/// `panic!` outside tests" is a repo rule, and this is the line that would have broken it.
fn read_autostart(app: &AppHandle) -> AutostartView {
    let Some(manager) = app.try_state::<AutoLaunchManager>() else {
        return AutostartView {
            enabled: false,
            unavailable: Some(AutostartReason::NotSupported),
        };
    };
    match manager.is_enabled() {
        Ok(enabled) => AutostartView {
            enabled,
            unavailable: None,
        },
        Err(_) => AutostartView {
            enabled: false,
            unavailable: Some(AutostartReason::RegistryUnreadable),
        },
    }
}
