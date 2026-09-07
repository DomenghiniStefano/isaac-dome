//! Settings I/O: the only state the `app` crate persists, the active profile
//! choice. A missing, unreadable, or malformed file is treated as "no choice
//! saved" — never a fatal error, never a silent overwrite of the user's file.

use ipc::Settings;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::error::IpcError;

fn settings_path(app: &AppHandle) -> Result<PathBuf, IpcError> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("settings.json"))
        .map_err(|e| IpcError::SettingsNotWritable {
            reason: e.to_string(),
        })
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
    let fail = |e: std::io::Error| IpcError::SettingsNotWritable {
        reason: e.to_string(),
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let body =
        serde_json::to_string_pretty(settings).map_err(|e| IpcError::SettingsNotWritable {
            reason: e.to_string(),
        })?;
    std::fs::write(&path, body).map_err(fail)
}
