//! The archive: sources, the events that are the archive, and the runs that are a cache of a
//! fold over them.

use run::{Event, Floor, Outcome, Run, SeedKind, SourceKey};
use store::{SourceKind, Store, StoreError, SCHEMA_VERSION};

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("isaacdome.db")).unwrap();
    (dir, store)
}

fn key(offset: u64) -> SourceKey {
    SourceKey::new(b"a banner", b"some bytes", offset)
}

fn started(seed: &str) -> Event {
    Event::RunStarted {
        seed_words: seed.to_string(),
        seed_numeric: 1,
        kind: SeedKind::New,
    }
}

fn run_of(seed: &str) -> Run {
    Run {
        seed_words: seed.to_string(),
        seed_numeric: 1,
        seed_kind: SeedKind::New,
        character: None,
        starting_items: vec![],
        collected: vec![],
        passives: vec![],
        familiars: vec![],
        held_active: None,
        floors: vec![Floor {
            stage: 1,
            stage_type: 0,
            seed: 7,
        }],
        achievements: vec![],
        outcome: Outcome::Open,
    }
}

#[test]
fn the_schema_is_at_version_four() {
    let (_d, store) = open();
    assert_eq!(SCHEMA_VERSION, 4);
    assert_eq!(store.schema_version().unwrap(), 4);
}

#[test]
fn a_session_is_found_again_by_its_folder_name() {
    // The name is the whole identity of an online session: re-reading the folder must find the
    // same source and import nothing twice.
    let (_d, store) = open();
    assert!(store
        .session_source("09_12_2026__13_34_26")
        .unwrap()
        .is_none());
    let id = store
        .insert_session_source("09_12_2026__13_34_26", &key(0))
        .unwrap();
    let found = store
        .session_source("09_12_2026__13_34_26")
        .unwrap()
        .expect("it is there now");
    assert_eq!(found.id, id);
    assert_eq!(found.kind, SourceKind::Session);
    assert_eq!(found.key.as_deref(), Some("09_12_2026__13_34_26"));
}

#[test]
fn two_launches_are_two_sources_and_the_first_keeps_its_events() {
    // The game rewrites log.txt on every launch. The second launch must not overwrite the
    // first: that is the archive losing exactly what it exists to keep.
    let (_d, store) = open();
    let first = store.insert_log_source(&key(0)).unwrap();
    store.append_events(first, &[started("AAA AAA")]).unwrap();
    let second = store.insert_log_source(&key(0)).unwrap();
    store.append_events(second, &[started("BBB BBB")]).unwrap();

    assert_ne!(first, second);
    assert_eq!(
        store.events(first).unwrap().events,
        vec![started("AAA AAA")]
    );
    assert_eq!(store.latest_log_source().unwrap().unwrap().id, second);
}

#[test]
fn appending_twice_continues_the_sequence_instead_of_starting_over() {
    let (_d, store) = open();
    let id = store.insert_log_source(&key(0)).unwrap();
    assert_eq!(store.append_events(id, &[started("AAA AAA")]).unwrap(), 1);
    assert_eq!(
        store
            .append_events(id, &[Event::RoomTransition, started("BBB BBB")])
            .unwrap(),
        2
    );
    assert_eq!(
        store.events(id).unwrap().events,
        vec![
            started("AAA AAA"),
            Event::RoomTransition,
            started("BBB BBB")
        ]
    );
}

#[test]
fn the_offset_a_source_reached_survives_being_written_again() {
    let (_d, store) = open();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.set_source_key(id, &key(40_000)).unwrap();
    assert_eq!(
        store.latest_log_source().unwrap().unwrap().source_key,
        key(40_000)
    );
}

#[test]
fn an_event_row_that_does_not_parse_is_counted_and_the_others_still_read() {
    // `goals()`'s rule, applied to the archive: one bad row must not wipe the run around it.
    let (_d, store) = open();
    let id = store.insert_log_source(&key(0)).unwrap();
    store
        .append_events(id, &[started("AAA AAA"), Event::RoomTransition])
        .unwrap();
    store::for_tests::corrupt_event(&store, id, 1, "{\"NotAnEvent\":{}}").unwrap();

    let read = store.events(id).unwrap();
    assert_eq!(read.events, vec![started("AAA AAA")]);
    assert_eq!(read.unreadable, 1);
}

#[test]
fn runs_cached_under_one_rules_version_are_not_returned_for_another() {
    // Decision 2 of the spec, made structural: a newer rules file invalidates the cache instead
    // of leaving two readings side by side.
    let (_d, store) = open();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[run_of("AAA AAA")]).unwrap();

    assert_eq!(
        store.cached_runs(id, 1).unwrap(),
        Some(vec![run_of("AAA AAA")])
    );
    assert_eq!(store.cached_runs(id, 2).unwrap(), None);
}

#[test]
fn caching_again_replaces_the_fold_instead_of_adding_to_it() {
    let (_d, store) = open();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[run_of("AAA AAA")]).unwrap();
    store
        .cache_runs(id, 1, &[run_of("AAA AAA"), run_of("BBB BBB")])
        .unwrap();
    assert_eq!(store.cached_runs(id, 1).unwrap().unwrap().len(), 2);
}

#[test]
fn every_source_comes_back_in_the_order_it_was_inserted() {
    let (_d, store) = open();
    let first = store.insert_log_source(&key(0)).unwrap();
    let second = store
        .insert_session_source("09_12_2026__13_34_26", &key(0))
        .unwrap();
    let ids: Vec<i64> = store.sources().unwrap().iter().map(|s| s.id).collect();
    assert_eq!(ids, vec![first, second]);
}

#[test]
fn a_database_from_before_the_archive_gains_the_tables_and_keeps_its_plan() {
    // The migration runs on someone's file, not on an empty one. What was there stays there.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isaacdome.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE goals (id TEXT PRIMARY KEY, target_json TEXT NOT NULL,
                created_unix INTEGER NOT NULL, note TEXT, seq INTEGER NOT NULL);
             CREATE TABLE plan_queue (id INTEGER PRIMARY KEY CHECK (id = 1), rows_json TEXT NOT NULL);
             CREATE TABLE window_session (id INTEGER PRIMARY KEY CHECK (id = 1), document TEXT NOT NULL);
             INSERT INTO plan_queue (id, rows_json) VALUES (1, '[]');
             PRAGMA user_version = 3;",
        )
        .unwrap();
    }

    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 4);
    assert!(store.queue().unwrap().is_ok());
    assert!(store.latest_log_source().unwrap().is_none());
}

#[test]
fn a_file_from_a_newer_app_is_still_refused_untouched() {
    // The guard that already exists, checked against the new version number rather than assumed
    // to have survived it.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isaacdome.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA user_version = 99;").unwrap();
    }
    match Store::open(&path) {
        Err(StoreError::NewerSchema { found, supported }) => {
            assert_eq!((found, supported), (99, 4));
        }
        other => panic!("expected NewerSchema, got {other:?}"),
    }
}
