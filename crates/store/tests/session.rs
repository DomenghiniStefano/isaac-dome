//! The window session on disk: one document, replaced whole, and the difference between
//! "there isn't one" and "there is an empty one".

use store::{Store, SCHEMA_VERSION};
use tempfile::tempdir;

mod common;
use common::temp_store;

#[test]
fn the_schema_knows_the_session() {
    // The session arrived with migration 3, so every version from there on knows it. The number
    // of the day is pinned in `tests/archive.rs`; what this one is about is that the table is
    // not behind us.
    // A `const` block: the claim is about the constant and nothing about a running database, so
    // it is checked when the test is compiled rather than when it runs.
    const { assert!(SCHEMA_VERSION >= 3) };
}

#[test]
fn a_fresh_database_has_no_session() {
    let (_dir, s) = temp_store();
    assert_eq!(s.session().expect("reads"), None);
}

#[test]
fn the_document_written_is_the_document_read() {
    let (_dir, s) = temp_store();
    let document = r#"{"version":1,"tabs":[],"activeIndex":0}"#;
    s.set_session(Some(document)).expect("writes");
    assert_eq!(s.session().expect("reads").as_deref(), Some(document));
}

#[test]
fn a_session_survives_reopening() {
    // The point of a file rather than an in-memory database: a session is for the next launch.
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    {
        let s = Store::open(&path).expect("opens");
        s.set_session(Some("one")).expect("writes");
    }
    let s = Store::open(&path).expect("reopens");
    assert_eq!(s.session().expect("reads").as_deref(), Some("one"));
}

#[test]
fn a_version_two_database_gains_the_session_without_losing_its_queue() {
    // The upgrade path an existing install takes. Built by hand at version 2 — the shape
    // migration 2 shipped — and then opened by this binary.
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    {
        let conn = rusqlite::Connection::open(&path).expect("opens");
        conn.execute_batch(
            "CREATE TABLE goals (
                id TEXT PRIMARY KEY,
                target_json TEXT NOT NULL,
                created_unix INTEGER NOT NULL,
                note TEXT,
                seq INTEGER NOT NULL
             );
             CREATE TABLE plan_queue (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                rows_json TEXT NOT NULL
             );
             INSERT INTO plan_queue (id, rows_json)
             VALUES (1, '[{\"achievement\":7,\"wanted\":true,\"origins\":[]}]');
             PRAGMA user_version = 2;",
        )
        .expect("builds a version 2 file");
    }
    let s = Store::open(&path).expect("upgrades");
    // The current version, not a number of the day: this test is about the chain arriving from
    // version 2, and it has to keep holding when a fifth migration is added.
    assert_eq!(s.schema_version().expect("reads"), SCHEMA_VERSION);
    assert_eq!(
        s.queue().expect("query").expect("parses").rows().len(),
        1,
        "migration 3 must not touch what migration 2 wrote"
    );
    assert_eq!(s.session().expect("reads"), None);
}

#[test]
fn a_second_write_replaces_the_first_rather_than_adding_a_second_answer() {
    let (_dir, s) = temp_store();
    s.set_session(Some("one")).expect("writes");
    s.set_session(Some("two")).expect("writes again");
    assert_eq!(s.session().expect("reads").as_deref(), Some("two"));
}

#[test]
fn clearing_leaves_no_session_behind() {
    let (_dir, s) = temp_store();
    s.set_session(Some("one")).expect("writes");
    s.set_session(None).expect("clears");
    assert_eq!(s.session().expect("reads"), None);
}
