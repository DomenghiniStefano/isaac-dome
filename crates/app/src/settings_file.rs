//! Settings I/O: the only state the `app` crate persists, the active profile
//! choice. A missing, unreadable, or malformed file is treated as "no choice
//! saved" — never a fatal error, never a silent overwrite of the user's file.

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

/// A missing, unreadable, or malformed file is treated as "no choice saved".
/// Never a fatal error, never a silent overwrite of the user's file.
pub fn load(app: &AppHandle) -> Settings {
    settings_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), IpcError> {
    let path = settings_path(app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_failed)?;
    }
    let body = serde_json::to_string_pretty(settings)
        .map_err(|_| not_writable(SettingsReason::Encoding))?;
    std::fs::write(&path, body).map_err(io_failed)
}
