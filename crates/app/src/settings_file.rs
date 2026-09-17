//! Settings I/O: the only state the `app` crate persists, the active profile
//! choice. A missing, unreadable, or malformed file is treated as "no choice
//! saved" — never a fatal error, never a silent overwrite of the user's file.

use discovery::Options;
use ipc::{Settings, SettingsReason};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use ipc::IpcError;

fn not_writable(reason: SettingsReason) -> IpcError {
    IpcError::SettingsNotWritable { reason }
}

/// The system's own message never crosses: it is not translatable, and an `io::Error` from
/// a path operation can name the path that produced it.
fn io_failed(e: std::io::Error) -> IpcError {
    not_writable(SettingsReason::Io {
        reason: e.kind().into(),
    })
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, IpcError> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("settings.json"))
        .map_err(|_| not_writable(SettingsReason::ConfigDirUnknown))
}

/// The whole file. `ipc::Settings` is the half that crosses the IPC; **the two folders the
/// user pointed at are paths, so they stay on this side of the boundary** — a `PathBuf` in a
/// type the `settings` command returns would put the Windows username on the wire.
///
/// `flatten` keeps the file's shape exactly as it was: the folders are two more keys beside
/// the settings, not a nested object, so an existing `settings.json` reads unchanged.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Stored {
    #[serde(flatten)]
    pub settings: Settings,
    /// Chosen by hand when discovery could not find the game (B14).
    pub game_dir: Option<PathBuf>,
    /// Chosen by hand when discovery could not find the saves (B14).
    pub save_dir: Option<PathBuf>,
}

fn read_stored(app: &AppHandle) -> Stored {
    settings_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// A missing, unreadable, or malformed file is treated as "no choice saved".
/// Never a fatal error, never a silent overwrite of the user's file.
pub fn load(app: &AppHandle) -> Settings {
    read_stored(app).settings
}

/// What `discover` is given: the folders the user pointed at, tried before its own search.
pub fn options(app: &AppHandle) -> Options {
    let stored = read_stored(app);
    Options {
        steam_root: None,
        game_dir: stored.game_dir,
        save_dir: stored.save_dir,
    }
}

/// Writes the settings **without dropping the folders**. Every caller holds an
/// `ipc::Settings` and knows nothing about the other half of the file; if this wrote only
/// what it was handed, choosing a folder by hand and then moving the scale slider would
/// silently forget the folder. The rule lives here, once, rather than in sixteen callers.
pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), IpcError> {
    write(
        app,
        &Stored {
            settings: settings.clone(),
            ..read_stored(app)
        },
    )
}

/// Writes a folder the user chose, keeping the settings beside it for the same reason.
pub fn save_folders(
    app: &AppHandle,
    game_dir: Option<PathBuf>,
    save_dir: Option<PathBuf>,
) -> Result<(), IpcError> {
    let stored = read_stored(app);
    write(
        app,
        &Stored {
            game_dir: game_dir.or(stored.game_dir),
            save_dir: save_dir.or(stored.save_dir),
            ..stored
        },
    )
}

fn write(app: &AppHandle, stored: &Stored) -> Result<(), IpcError> {
    let path = settings_path(app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_failed)?;
    }
    let body =
        serde_json::to_string_pretty(stored).map_err(|_| not_writable(SettingsReason::Encoding))?;
    std::fs::write(&path, body).map_err(io_failed)
}
