//! The Data page: the files the app writes, where they are, how large, and what the database
//! holds. `app` gathers the facts with `metadata`; this shapes them. The folder crosses as a
//! hint, and the file crosses inward as a `DataFile` — never a path, either way.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::hint::mask_user_dir;
use crate::StoreReason;

/// Which of the app's files. Also the argument of `reveal_data_file`: the frontend names a
/// file, the backend knows where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum DataFile {
    Database,
    Settings,
}

/// What the database holds, counted. `queue_rows` is `None` when the queue document does not
/// parse: "unknown" and "empty" are different sentences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StoreContents {
    pub goals: u32,
    pub queue_rows: Option<u32>,
    pub sessions: u32,
    pub runs: u32,
    pub roll_saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DataFileState {
    /// `contents` only for the database, and only when it opened.
    Present {
        folder_hint: String,
        size_bytes: u64,
        contents: Option<StoreContents>,
    },
    /// Nothing written yet: a fresh install.
    NotCreated { folder_hint: String },
    /// The database is there and will not open.
    Unreadable {
        folder_hint: String,
        reason: StoreReason,
    },
    /// The platform would not name the folder.
    FolderUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DataFileView {
    pub file: DataFile,
    pub state: DataFileState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DataView {
    pub files: Vec<DataFileView>,
}

/// One file as `app` found it. Paths stay on this side: nothing here derives `Serialize`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFact {
    FolderUnknown,
    Absent { folder: PathBuf },
    Present { folder: PathBuf, size_bytes: u64 },
}

/// What `app` knows. `store` is `None` when it was not asked — the file is absent, or its
/// folder unknown — because asking would create the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFacts {
    pub database: FileFact,
    pub settings: FileFact,
    pub store: Option<Result<StoreContents, StoreReason>>,
}

pub fn data_view(facts: DataFacts) -> DataView {
    DataView {
        files: vec![
            DataFileView {
                file: DataFile::Database,
                state: database_state(facts.database, facts.store),
            },
            DataFileView {
                file: DataFile::Settings,
                state: file_state(facts.settings, None),
            },
        ],
    }
}

fn hint(folder: &Path) -> String {
    mask_user_dir(&folder.display().to_string())
}

/// A database that is there and will not open is the one state the settings file cannot be
/// in; everything else reads the way any file does.
fn database_state(
    fact: FileFact,
    store: Option<Result<StoreContents, StoreReason>>,
) -> DataFileState {
    match (fact, store) {
        (FileFact::Present { folder, .. }, Some(Err(reason))) => DataFileState::Unreadable {
            folder_hint: hint(&folder),
            reason,
        },
        (fact, store) => file_state(fact, store.and_then(Result::ok)),
    }
}

fn file_state(fact: FileFact, contents: Option<StoreContents>) -> DataFileState {
    match fact {
        FileFact::FolderUnknown => DataFileState::FolderUnknown,
        FileFact::Absent { folder } => DataFileState::NotCreated {
            folder_hint: hint(&folder),
        },
        FileFact::Present { folder, size_bytes } => DataFileState::Present {
            folder_hint: hint(&folder),
            size_bytes,
            contents,
        },
    }
}
