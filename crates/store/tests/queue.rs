//! The plan queue on disk: one document, replaced whole. The tests that matter are the
//! two that separate an empty queue from one that couldn't be read.

use plan::{Queue, Row};
use store::Store;
use tempfile::tempdir;

fn row(achievement: u32, wanted: bool, origins: &[u32]) -> Row {
    Row {
        achievement,
        wanted,
        origins: origins.to_vec(),
    }
}

#[test]
fn a_queue_survives_reopening() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    {
        let s = Store::open(&path).expect("opens");
        s.set_queue(&Queue::from_rows(vec![
            row(7, false, &[41]),
            row(41, true, &[]),
        ]))
        .expect("writes");
    }
    let s = Store::open(&path).expect("reopens");
    let q = s.queue().expect("query").expect("document parses");
    assert_eq!(q.rows().len(), 2);
    assert_eq!(q.position(41), Some(1), "the order is what was written");
    assert_eq!(q.rows()[0].origins, vec![41], "and so is everything else");
}

#[test]
fn a_fresh_database_has_an_empty_queue_and_that_is_not_an_error() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    let q = s.queue().expect("query").expect("parses");
    assert_eq!(q.rows().len(), 0);
}

#[test]
fn a_document_that_does_not_parse_is_declared_not_flattened_to_empty() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    store::for_tests::corrupt_queue(&s, "{ not json").expect("writes garbage");
    let inner = s.queue().expect("the query itself works");
    assert!(
        inner.is_err(),
        "an unreadable queue must not read as an empty one"
    );
}

#[test]
fn writing_twice_replaces_the_document_instead_of_keeping_two() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    s.set_queue(&Queue::from_rows(vec![row(1, true, &[])]))
        .expect("writes");
    s.set_queue(&Queue::from_rows(vec![row(2, true, &[])]))
        .expect("writes again");
    let q = s.queue().expect("query").expect("parses");
    assert_eq!(q.rows().len(), 1);
    assert_eq!(q.rows()[0].achievement, 2);
}

#[test]
fn the_schema_version_moved_to_three() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    assert_eq!(s.schema_version().expect("reads"), 3);
}

#[test]
fn migration_two_leaves_the_goals_table_alone() {
    // The goals are not destroyed to make room for the queue: the import is a separate,
    // explicit step, and it has to still find them.
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    let read = s
        .goals()
        .expect("the goals table is still there and still readable");
    assert!(read.goals.is_empty());
    assert!(read.unreadable.is_empty());
}

#[test]
fn a_version_one_database_gains_the_queue_without_losing_its_goals() {
    // The upgrade path, which is the only one an existing install takes. Built by hand at
    // version 1 — the shape migration 1 shipped — and then opened by this binary.
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
             INSERT INTO goals (id, target_json, created_unix, note, seq)
             VALUES ('g1', '{\"kind\":\"character\",\"id\":1}', 0, NULL, 1);
             PRAGMA user_version = 1;",
        )
        .expect("builds a version 1 file");
    }
    let s = Store::open(&path).expect("upgrades");
    assert_eq!(s.schema_version().expect("reads"), 3);
    let goals = s.goals().expect("goals still readable");
    assert_eq!(
        goals.goals.len(),
        1,
        "migration 2 must not touch what migration 1 wrote"
    );
    // The chain runs the whole way, not just one step: a query against a table that migration
    // 3 never created would be an error, not a `None`.
    assert_eq!(
        s.session()
            .expect("the session table exists at the end of the chain"),
        None
    );
    assert_eq!(
        s.queue().expect("query").expect("parses").rows().len(),
        0,
        "the queue starts empty: the import is an explicit step, not a side effect"
    );
}
