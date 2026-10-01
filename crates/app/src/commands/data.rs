//! The Data page: where the app writes, and a way to see it in Explorer. The facts are read
//! with `metadata` — the files are never opened for this — and `ipc::data_view` shapes them.

use std::path::{Path, PathBuf};

use ipc::{DataFacts, DataFile, DataView, FileFact, IpcError, StoreContents, StoreReason};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::settings_file;
use crate::state::{data_dir, StoreState, DATABASE_FILE};

#[tauri::command]
pub fn data_location(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
) -> Result<DataView, IpcError> {
    let database = fact(path_of(&app, DataFile::Database));
    let settings = fact(path_of(&app, DataFile::Settings));
    let store = asked_store(&database, &app, &store);
    Ok(ipc::data_view(DataFacts {
        database,
        settings,
        store,
    }))
}

#[tauri::command]
pub fn reveal_data_file(app: AppHandle, file: DataFile) -> Result<(), IpcError> {
    let path = path_of(&app, file).ok_or(IpcError::FolderNotOpenable)?;
    // The plugin's error can name the path: it stops here.
    reveal(&app, &path).map_err(|_| IpcError::FolderNotOpenable)
}

fn path_of(app: &AppHandle, file: DataFile) -> Option<PathBuf> {
    match file {
        DataFile::Database => data_dir(app).ok().map(|d| d.join(DATABASE_FILE)),
        DataFile::Settings => settings_file::settings_path(app).ok(),
    }
}

/// The store is asked only when its file is there: opening creates it, and a fresh install
/// would then never read "not created yet".
fn asked_store(
    database: &FileFact,
    app: &AppHandle,
    store: &StoreState,
) -> Option<Result<StoreContents, StoreReason>> {
    match database {
        FileFact::Present { .. } => Some(store_contents(app, store)),
        FileFact::Absent { .. } | FileFact::FolderUnknown => None,
    }
}

fn store_contents(app: &AppHandle, store: &StoreState) -> Result<StoreContents, StoreReason> {
    store
        .lock(app)?
        .contents()
        .map_err(|e| StoreReason::from(&e))
}

fn fact(file: Option<PathBuf>) -> FileFact {
    match file {
        None => FileFact::FolderUnknown,
        Some(path) => fact_of(&path),
    }
}

fn fact_of(path: &Path) -> FileFact {
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    match std::fs::metadata(path) {
        Ok(meta) => FileFact::Present {
            folder,
            size_bytes: meta.len(),
        },
        Err(_) => FileFact::Absent { folder },
    }
}

/// The file selected in its folder; the folder alone when the file is not there (a fresh
/// install, or deleted since the page was drawn), because `reveal_item_in_dir` wants an item
/// that exists.
fn reveal(app: &AppHandle, path: &Path) -> Result<(), tauri_plugin_opener::Error> {
    match (path.exists(), path.parent()) {
        (true, _) | (false, None) => app.opener().reveal_item_in_dir(path),
        (false, Some(folder)) => app
            .opener()
            .open_path(folder.display().to_string(), None::<&str>),
    }
}
