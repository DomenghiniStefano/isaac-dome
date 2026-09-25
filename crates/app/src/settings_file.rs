//! The settings file, `settings.json` in the config folder: the `ipc::Settings` the screens read
//! and write — the active profile, the scale, the switches — plus the two folders chosen by hand,
//! which never cross the IPC. A missing or unreadable file is read as the defaults; a malformed
//! one is read as the defaults too and set aside before the next write (card #80, P2) — never a
//! fatal error, never a silent overwrite of the user's file.

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
    match read_file(app) {
        FileRead::Parsed(stored) => stored,
        FileRead::Absent | FileRead::Malformed => Stored::default(),
    }
}

/// What is on disk, told apart: a file that does not parse is **not** the same as no file,
/// because writing over it would lose the folders it holds (card #80, P2).
enum FileRead {
    Absent,
    Parsed(Stored),
    Malformed,
}

fn read_file(app: &AppHandle) -> FileRead {
    let Some(text) = settings_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
    else {
        return FileRead::Absent;
    };
    match serde_json::from_str(&text) {
        Ok(stored) => FileRead::Parsed(stored),
        Err(_) => FileRead::Malformed,
    }
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
    // A file that does not parse is put aside before anything is written, never written over
    // (card #80, P2): it may hold the folders chosen by hand, and "never a silent overwrite of
    // the user's file" is this module's promise. Named by the second it was set aside, so a
    // second bad file does not replace the first.
    if matches!(read_file(app), FileRead::Malformed) {
        let unix = crate::clock::now_unix();
        std::fs::rename(
            &path,
            path.with_file_name(format!("settings.malformed-{unix}.json")),
        )
        .map_err(io_failed)?;
    }
    // Atomic: written beside the file and renamed over it, so a crash mid-write leaves the old
    // file or the new one, never half of each.
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, body).map_err(io_failed)?;
    std::fs::rename(&temp, &path).map_err(io_failed)
}
