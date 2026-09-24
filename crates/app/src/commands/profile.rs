//! Profile selection and settings: the two commands that write, and the two that read them back.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::AutoLaunchManager;
use tauri_plugin_dialog::DialogExt;

use ipc::{AutostartReason, AutostartView, IpcError, ProfileId, Settings, SetupState};

use crate::events::{announce, PROFILE_CHANGED, SETTINGS_CHANGED};
use crate::settings_file;
use crate::state::StoreState;

/// The I/O the pure crate does not do. A save is 11–12 KB and there is a handful of
/// candidates, so this is one small read per candidate per answer and nothing worth caching:
/// `SaveCache` holds one slot, for the active profile, and a failed read is never remembered.
fn read_save(path: &std::path::Path) -> Option<core_save::Save> {
    core_save::Save::open(path).ok()
}

/// The state as it is now: discovery run with the folders the user pointed at, and the saved
/// choice. Shared by the three commands that answer it, so they cannot drift apart.
fn state_now(app: &AppHandle) -> SetupState {
    let settings = settings_file::load(app);
    let d = crate::state::discovery_now(app);
    ipc::setup_state(
        &d,
        settings.active_profile_id.as_ref(),
        read_save,
        crate::icons::icon_url,
    )
}

#[tauri::command]
pub fn setup_state(app: AppHandle) -> Result<SetupState, IpcError> {
    Ok(state_now(&app))
}

/// Asks for a folder and answers the discovery that comes out of it. **The path travels
/// inward only**: what goes back is a `SetupState`, whose `pathHint` is redacted.
///
/// `async` plus the **callback** form is the documented-safe pair (checked 2026-09-17 against
/// `v2.tauri.app/plugin/dialog/`): `blocking_pick_folder` *"should NOT be used when running on
/// the main thread"*, and which thread a synchronous command runs on is not documented, so
/// this takes the form that needs no inference. `try_send` and never `blocking_send`: the
/// channel has room for the one message, and `blocking_send` panics if the callback happens
/// to run inside a runtime thread.
async fn ask_for_folder(app: &AppHandle) -> Option<PathBuf> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    app.dialog().file().pick_folder(move |picked| {
        let _ = tx.try_send(picked);
    });
    rx.recv().await.flatten().and_then(|p| p.into_path().ok())
}

#[tauri::command]
pub async fn choose_game_folder(app: AppHandle) -> Result<SetupState, IpcError> {
    // Cancelled is not an error and not a change: the state as it already was.
    let Some(dir) = ask_for_folder(&app).await else {
        return Ok(state_now(&app));
    };
    settings_file::save_folders(&app, Some(dir), None)?;
    announce(&app, PROFILE_CHANGED);
    Ok(state_now(&app))
}

#[tauri::command]
pub async fn choose_saves_folder(app: AppHandle) -> Result<SetupState, IpcError> {
    let Some(dir) = ask_for_folder(&app).await else {
        return Ok(state_now(&app));
    };
    settings_file::save_folders(&app, None, Some(dir))?;
    announce(&app, PROFILE_CHANGED);
    Ok(state_now(&app))
}

#[tauri::command]
pub fn select_profile(app: AppHandle, id: ProfileId) -> Result<SetupState, IpcError> {
    // The same options `setup_state` answered with: a candidate found in a folder chosen by
    // hand must be choosable, and searching without them here would refuse it as unknown.
    let d = crate::state::discovery_now(&app);
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
    Ok(ipc::setup_state(
        &d,
        settings.active_profile_id.as_ref(),
        read_save,
        crate::icons::icon_url,
    ))
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

/// Whether the app asks GitHub for a newer version when it starts.
///
/// **Turning it off stops the request, not a notice**: the launch reads this before it spawns
/// anything, so with it off nothing leaves the machine unless somebody presses the button on
/// the Updates screen. Turning it on does not check now — the button is there for that, and a
/// switch that also acted would be two things on one control.
#[tauri::command]
pub fn set_auto_update(app: AppHandle, on: bool) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_auto_update(on);
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
    let wrote = app.try_state::<AutoLaunchManager>().map(|manager| {
        if on {
            manager.enable().is_ok()
        } else {
            manager.disable().is_ok()
        }
    });
    ipc::autostart_answer(on, wrote, read_autostart(&app))
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
