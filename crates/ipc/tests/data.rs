//! The Data page's view: which files the app writes, where, how large, and what the database
//! holds — with the folder as a hint and never as a path.

#![allow(clippy::wildcard_enum_match_arm)]

use std::path::PathBuf;

use ipc::{
    data_view, DataFacts, DataFile, DataFileState, DataView, FileFact, StoreContents, StoreReason,
};

fn folder() -> PathBuf {
    PathBuf::from(r"C:\Users\stefa\AppData\Roaming\dev.isaacdome.app")
}

fn contents() -> StoreContents {
    StoreContents {
        goals: 3,
        queue_rows: Some(12),
        sessions: 40,
        runs: 95,
        roll_saved: true,
    }
}

fn present(size_bytes: u64) -> FileFact {
    FileFact::Present {
        folder: folder(),
        size_bytes,
    }
}

fn state_of(view: &DataView, file: DataFile) -> &DataFileState {
    &view
        .files
        .iter()
        .find(|f| f.file == file)
        .expect("the file has a row")
        .state
}

#[test]
fn both_files_have_a_row_database_first() {
    let view = data_view(DataFacts {
        database: present(1),
        settings: present(1),
        store: Some(Ok(contents())),
    });
    let order: Vec<DataFile> = view.files.iter().map(|f| f.file).collect();
    assert_eq!(order, vec![DataFile::Database, DataFile::Settings]);
}

#[test]
fn a_present_database_carries_its_size_and_contents() {
    let view = data_view(DataFacts {
        database: present(204_800),
        settings: present(310),
        store: Some(Ok(contents())),
    });
    assert_eq!(
        state_of(&view, DataFile::Database),
        &DataFileState::Present {
            folder_hint: r"C:\Users\<user>\AppData\Roaming\dev.isaacdome.app".to_string(),
            size_bytes: 204_800,
            contents: Some(contents()),
        }
    );
}

#[test]
fn the_settings_file_never_carries_contents() {
    let view = data_view(DataFacts {
        database: present(1),
        settings: present(310),
        store: Some(Ok(contents())),
    });
    match state_of(&view, DataFile::Settings) {
        DataFileState::Present {
            contents,
            size_bytes,
            ..
        } => {
            assert_eq!(*contents, None);
            assert_eq!(*size_bytes, 310);
        }
        other => panic!("expected Present, got {other:?}"),
    }
}

#[test]
fn absent_database_reads_not_created() {
    let view = data_view(DataFacts {
        database: FileFact::Absent { folder: folder() },
        settings: FileFact::Absent { folder: folder() },
        store: None,
    });
    assert!(matches!(
        state_of(&view, DataFile::Database),
        DataFileState::NotCreated { .. }
    ));
    assert!(matches!(
        state_of(&view, DataFile::Settings),
        DataFileState::NotCreated { .. }
    ));
}

#[test]
fn an_unreadable_store_keeps_the_settings_row() {
    let reason = StoreReason::NewerSchema {
        found: 9,
        supported: 6,
    };
    let view = data_view(DataFacts {
        database: present(4096),
        settings: present(310),
        store: Some(Err(reason)),
    });
    assert!(matches!(
        state_of(&view, DataFile::Database),
        DataFileState::Unreadable { reason: r, .. } if *r == reason
    ));
    assert!(matches!(
        state_of(&view, DataFile::Settings),
        DataFileState::Present { .. }
    ));
}

#[test]
fn an_unknown_folder_reads_folder_unknown() {
    let view = data_view(DataFacts {
        database: FileFact::FolderUnknown,
        settings: FileFact::FolderUnknown,
        store: None,
    });
    assert_eq!(
        state_of(&view, DataFile::Database),
        &DataFileState::FolderUnknown
    );
    assert_eq!(
        state_of(&view, DataFile::Settings),
        &DataFileState::FolderUnknown
    );
}

#[test]
fn the_folder_hint_masks_the_username() {
    // Both forms the platform hands back: plain, and the verbatim `\\?\` prefix, with `users`
    // in lowercase — the mask matches the segment case-insensitively.
    [
        r"C:\Users\stefa\AppData\Roaming\x",
        r"\\?\C:\users\stefa\AppData\Roaming\x",
    ]
    .into_iter()
    .for_each(|raw| {
        let view = data_view(DataFacts {
            database: FileFact::Absent {
                folder: PathBuf::from(raw),
            },
            settings: FileFact::FolderUnknown,
            store: None,
        });
        let DataFileState::NotCreated { folder_hint } = state_of(&view, DataFile::Database) else {
            panic!("expected NotCreated");
        };
        assert!(!folder_hint.contains("stefa"), "{folder_hint}");
        assert!(folder_hint.contains("<user>"), "{folder_hint}");
    });
}

#[test]
fn the_wire_shape_is_camel_case_and_tagged() {
    let view = data_view(DataFacts {
        database: present(2048),
        settings: FileFact::FolderUnknown,
        store: Some(Ok(contents())),
    });
    let json = serde_json::to_value(&view).expect("serializes");
    let db = &json["files"][0];
    assert_eq!(db["file"], "database");
    assert_eq!(db["state"]["kind"], "present");
    assert_eq!(
        db["state"]["folderHint"],
        r"C:\Users\<user>\AppData\Roaming\dev.isaacdome.app"
    );
    assert_eq!(db["state"]["sizeBytes"], 2048);
    assert_eq!(db["state"]["contents"]["queueRows"], 12);
    assert_eq!(db["state"]["contents"]["rollSaved"], true);
    assert_eq!(json["files"][1]["state"]["kind"], "folderUnknown");
}

#[test]
fn the_file_argument_reads_back_from_its_wire_name() {
    // `reveal_data_file` receives it from the frontend: the only thing that crosses inward.
    let file: DataFile = serde_json::from_str("\"settings\"").expect("deserializes");
    assert_eq!(file, DataFile::Settings);
}
