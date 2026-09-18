//! Migration 6: one preset, always saved, and the current draw beside it.
//!
//! The nested `Result` mirrors `queue()`, for its reason: "the database will not open" and
//! "the document will not parse" are different sentences on the screen.

use roll::{Document, DocumentError, Drawn, Preset, Selection, Target};
use store::Store;

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let store = Store::open(&dir.path().join("isaacdome.db")).expect("a fresh database opens");
    (dir, store)
}

#[test]
fn the_schema_reaches_version_six() {
    assert_eq!(store::SCHEMA_VERSION, 6);
}

#[test]
fn no_row_reads_as_the_default_document() {
    // A fresh database has no preset, and that is not a failure: it is every character, every
    // column, only what's missing, only playable, nothing drawn.
    let (_dir, store) = temp_store();
    assert_eq!(
        store.roll().expect("the database opens"),
        Ok(Document::default())
    );
}

#[test]
fn a_document_survives_a_write_and_a_read() {
    let (_dir, store) = temp_store();
    let doc = Document {
        preset: Preset {
            characters: Selection::Only { ids: vec![2, 3] },
            columns: Selection::All,
            include_taken: true,
            only_playable: false,
        },
        current: Some(Drawn {
            target: Target::Mark {
                character: 2,
                column: 7,
            },
            deck_size: 44,
            drawn_unix: 1_789_100_000,
        }),
        ..Document::default()
    };
    store.set_roll(&doc).expect("the write lands");
    assert_eq!(store.roll().expect("the database opens"), Ok(doc));
}

#[test]
fn a_second_write_replaces_the_first_rather_than_adding_a_row() {
    // One row, pinned by the CHECK: a second document would be a second answer to "what is the
    // preset".
    let (_dir, store) = temp_store();
    store
        .set_roll(&Document::default())
        .expect("the first write lands");
    let second = Document {
        preset: Preset {
            include_taken: true,
            ..Preset::default()
        },
        ..Document::default()
    };
    store.set_roll(&second).expect("the second write lands");
    assert_eq!(store.roll().expect("the database opens"), Ok(second));
}

#[test]
fn a_corrupt_document_reaches_the_caller_as_the_inner_error_and_never_panics() {
    let (dir, store) = temp_store();
    drop(store);
    let path = dir.path().join("isaacdome.db");
    let conn = rusqlite::Connection::open(&path).expect("reopen");
    conn.execute(
        "INSERT INTO roll (id, document) VALUES (1, '{ not json')",
        [],
    )
    .expect("the row goes in");
    drop(conn);
    let store = Store::open(&path).expect("it opens again");
    let read = store.roll().expect("the database itself is fine");
    assert!(matches!(read, Err(DocumentError::Unreadable { .. })));
}
