//! The archive: sources, the events that are the archive, and the runs that are a cache of a
//! fold over them.

use ipc::{RunSource, RunsDiagnostic};
use run::{Event, Floor, Generated, Outcome, Run, SeedKind, SourceKey};
use store::{ArchivedRuns, SourceKind, Store, StoredSource, SCHEMA_VERSION};

mod common;
use common::temp_store;

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
        character_id: None,
        starting_items: vec![],
        collected: vec![],
        passives: vec![],
        familiars: vec![],
        held_active: None,
        floors: vec![Floor {
            stage: 1,
            stage_type: 0,
            seed: 7,
            generated: Generated::NotSaid,
        }],
        achievements: vec![],
        outcome: Outcome::Open,
    }
}

#[test]
fn the_schema_is_at_version_six() {
    let (_d, store) = temp_store();
    assert_eq!(SCHEMA_VERSION, 6);
    assert_eq!(store.schema_version().unwrap(), 6);
}

#[test]
fn a_session_is_found_again_by_its_folder_name() {
    // The name is the whole identity of an online session: re-reading the folder must find the
    // same source and import nothing twice.
    let (_d, store) = temp_store();
    assert!(store
        .session_source("09_12_2026__13_34_26")
        .unwrap()
        .is_none());
    let id = store
        .import_session("09_12_2026__13_34_26", &key(0), &[])
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
    let (_d, store) = temp_store();
    let first = store.insert_log_source(&key(0)).unwrap();
    store
        .append_to_log(first, &key(0), &[started("AAA AAA")])
        .unwrap();
    let second = store.insert_log_source(&key(0)).unwrap();
    store
        .append_to_log(second, &key(0), &[started("BBB BBB")])
        .unwrap();

    assert_ne!(first, second);
    assert_eq!(
        store.events(first).unwrap().events,
        vec![started("AAA AAA")]
    );
    assert_eq!(store.latest_log_source().unwrap().unwrap().id, second);
}

#[test]
fn appending_twice_continues_the_sequence_instead_of_starting_over() {
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    assert_eq!(
        store
            .append_to_log(id, &key(0), &[started("AAA AAA")])
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .append_to_log(id, &key(0), &[Event::RoomTransition, started("BBB BBB")])
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
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.append_to_log(id, &key(40_000), &[]).unwrap();
    assert_eq!(
        store.latest_log_source().unwrap().unwrap().source_key,
        key(40_000)
    );
}

#[test]
fn an_event_row_that_does_not_parse_is_counted_and_the_others_still_read() {
    // `goals()`'s rule, applied to the archive: one bad row must not wipe the run around it.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store
        .append_to_log(id, &key(0), &[started("AAA AAA"), Event::RoomTransition])
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
    let (_d, store) = temp_store();
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
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[run_of("AAA AAA")]).unwrap();
    store
        .cache_runs(id, 1, &[run_of("AAA AAA"), run_of("BBB BBB")])
        .unwrap();
    assert_eq!(store.cached_runs(id, 1).unwrap().unwrap().len(), 2);
}

#[test]
fn a_source_nobody_has_folded_has_no_cache() {
    // The first of the three states the one `Option` has to carry, and the only one it was ever
    // right about: nothing has been folded here, so there is nothing to answer with.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    assert_eq!(store.cached_runs(id, 1).unwrap(), None);
}

#[test]
fn a_source_folded_into_no_run_reads_back_as_an_empty_fold() {
    // A launch that holds the intro and nothing else folds to zero runs. That is an answer, and
    // it used to be indistinguishable from never having been read — which is not a missing cache
    // but a loop: fold again, produce nothing again, cache nothing again, for ever.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[]).unwrap();
    assert_eq!(store.cached_runs(id, 1).unwrap(), Some(vec![]));
}

#[test]
fn an_empty_fold_under_one_rules_version_is_not_returned_for_another() {
    // The third state. `runs` carries the rules version per row and zero rows have nowhere to
    // put one, so the version of the fold lives on the source: a newer rules file still finds
    // nothing here and folds again.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[]).unwrap();
    assert_eq!(store.cached_runs(id, 2).unwrap(), None);
}

#[test]
fn a_fold_that_used_to_hold_runs_and_now_holds_none_reads_as_empty() {
    // Replacement in the direction the test above it does not cover: `DELETE` with nothing
    // written after leaves no row behind, so the fold's own version is the only thing left that
    // says this source was read at all.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(id, 1, &[run_of("AAA AAA")]).unwrap();
    store.cache_runs(id, 1, &[]).unwrap();
    assert_eq!(store.cached_runs(id, 1).unwrap(), Some(vec![]));
}

#[test]
fn a_database_from_before_the_fold_version_keeps_its_archive_and_reads_it_as_unfolded() {
    // Migration 5 on somebody's file, not on an empty one. The column is nullable and additive:
    // every row that predates it gets NULL, and NULL reads as "never folded" — which is exactly
    // right for all of them, since nothing recorded which rules produced what they hold.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isaacdome.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE goals (id TEXT PRIMARY KEY, target_json TEXT NOT NULL,
                created_unix INTEGER NOT NULL, note TEXT, seq INTEGER NOT NULL);
             CREATE TABLE plan_queue (id INTEGER PRIMARY KEY CHECK (id = 1), rows_json TEXT NOT NULL);
             CREATE TABLE window_session (id INTEGER PRIMARY KEY CHECK (id = 1), document TEXT NOT NULL);
             CREATE TABLE sources (id INTEGER PRIMARY KEY, kind TEXT NOT NULL, key TEXT,
                prefix_hash TEXT NOT NULL, prefix_len INTEGER NOT NULL, anchor_hash TEXT NOT NULL,
                read_offset INTEGER NOT NULL, UNIQUE (kind, key));
             CREATE TABLE events (source_id INTEGER NOT NULL REFERENCES sources(id),
                seq INTEGER NOT NULL, event_json TEXT NOT NULL, PRIMARY KEY (source_id, seq));
             CREATE TABLE runs (source_id INTEGER NOT NULL REFERENCES sources(id),
                ordinal INTEGER NOT NULL, rules_version INTEGER NOT NULL, run_json TEXT NOT NULL,
                PRIMARY KEY (source_id, ordinal));
             INSERT INTO sources (id, kind, key, prefix_hash, prefix_len, anchor_hash, read_offset)
                VALUES (1, 'log', NULL, '0000000000000001', 8, '0000000000000002', 0);
             PRAGMA user_version = 4;",
        )
        .unwrap();
    }

    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    let sources = store.sources().unwrap();
    assert_eq!(sources.len(), 1, "the row survived the migration");
    assert_eq!(
        store.cached_runs(sources[0].id, 1).unwrap(),
        None,
        "NULL reads as never folded, not as folded into nothing"
    );
    // Migration 6 on the same file: this database predates the `roll` table entirely, not only
    // the fold version column. No row exists yet, which reads as the default document rather
    // than a failure — the same rule a fresh database's `roll()` follows.
    assert_eq!(
        store.roll().unwrap().unwrap(),
        roll::Document::default(),
        "an existing database must gain the roll table and read its default document"
    );
}

#[test]
fn every_source_comes_back_in_the_order_it_was_inserted() {
    let (_d, store) = temp_store();
    let first = store.insert_log_source(&key(0)).unwrap();
    let second = store
        .import_session("09_12_2026__13_34_26", &key(0), &[])
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
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(store.queue().unwrap().is_ok());
    assert!(store.latest_log_source().unwrap().is_none());
}

#[test]
fn the_events_and_the_offset_they_belong_to_move_in_the_same_write() {
    // Atomicity here is not a speed concern, it is the duplicate-runs failure by another road:
    // events written while the offset stays behind are events the next read finds again and
    // files a second time. So there is **one** call that does both, and no way through this API
    // to do either alone — which is why this test reads the pair after each write rather than
    // trying to tear them apart.
    let (_d, store) = temp_store();
    let id = store.insert_log_source(&key(0)).unwrap();

    store
        .append_to_log(id, &key(10), &[started("AAAA AAAA")])
        .unwrap();
    assert_eq!(store.events(id).unwrap().events.len(), 1);
    assert_eq!(
        store.latest_log_source().unwrap().unwrap().source_key,
        key(10)
    );

    store
        .append_to_log(id, &key(90), &[Event::RoomTransition, started("BBBB BBBB")])
        .unwrap();
    assert_eq!(store.events(id).unwrap().events.len(), 3);
    assert_eq!(
        store.latest_log_source().unwrap().unwrap().source_key,
        key(90)
    );
}

#[test]
fn importing_a_session_writes_its_source_and_its_events_together() {
    // The same rule on the other path: a source row with no events would be a session marked
    // imported for ever, holding nothing.
    let (_d, store) = temp_store();
    let id = store
        .import_session("09_12_2026__13_34_26", &key(500), &[started("AAAA AAAA")])
        .unwrap();
    assert_eq!(store.events(id).unwrap().events, vec![started("AAAA AAAA")]);
    assert_eq!(
        store
            .session_source("09_12_2026__13_34_26")
            .unwrap()
            .unwrap()
            .source_key,
        key(500)
    );
}

/// Card #81, V1: naming a source and gathering its cached runs was logic inside the `runs`
/// command, which is wiring and untested. The expected values are that command's behaviour,
/// read from it: a session is named by its folder, a launch is the source with no name.
#[test]
fn every_archived_source_is_named_the_way_the_runs_screen_names_it() {
    let (_d, store) = temp_store();
    let session = store
        .import_session("09_12_2026__13_34_26", &key(0), &[])
        .unwrap();
    let launch = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(session, 1, &[run_of("AAA AAA")]).unwrap();
    store
        .cache_runs(launch, 1, &[run_of("BBB BBB"), run_of("CCC CCC")])
        .unwrap();

    let archived = store.archived_runs(1).unwrap();

    assert_eq!(archived.unreadable, 0);
    let named: Vec<(RunSource, usize)> = archived
        .sources
        .iter()
        .map(|(source, runs)| (source.clone(), runs.len()))
        .collect();
    assert_eq!(
        named,
        vec![
            (
                RunSource::Session {
                    name: "09_12_2026__13_34_26".to_string()
                },
                1
            ),
            (RunSource::Live, 2),
        ]
    );
}

#[test]
fn a_source_nobody_folded_under_these_rules_contributes_no_row() {
    let (_d, store) = temp_store();
    let launch = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(launch, 1, &[run_of("AAA AAA")]).unwrap();

    let archived = store.archived_runs(2).unwrap();

    assert!(archived.sources.is_empty());
    assert_eq!(archived.unreadable, 0);
}

#[test]
fn a_session_row_with_no_name_reads_as_the_launch_it_cannot_be_told_from() {
    let row = StoredSource {
        id: 1,
        kind: SourceKind::Session,
        key: None,
        source_key: key(0),
    };
    assert_eq!(row.run_source(), RunSource::Live);
}

#[test]
fn unreadable_caches_are_counted_in_one_diagnostic_and_none_says_nothing() {
    let clean = ArchivedRuns::default();
    assert!(clean.diagnostics().is_empty());
    let two = ArchivedRuns {
        sources: vec![],
        unreadable: 2,
    };
    assert_eq!(
        two.diagnostics(),
        vec![RunsDiagnostic::UnreadableEvents { count: 2 }]
    );
}

// ---------------------------------------------------------------------------
// The launch Live follows (card #80, R10)
// ---------------------------------------------------------------------------

#[test]
fn the_live_runs_are_the_latest_launchs_and_no_one_elses() {
    // `live` used to read the whole archive — every session, every launch — every time the
    // watcher said a line arrived, to keep one run of one source.
    let (_d, store) = temp_store();
    let old = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(old, 1, &[run_of("AAA AAA")]).unwrap();
    let session = store
        .import_session("09_12_2026__13_34_26", &key(0), &[started("SSS SSS")])
        .unwrap();
    store.cache_runs(session, 1, &[run_of("SSS SSS")]).unwrap();
    let latest = store.insert_log_source(&key(0)).unwrap();
    store
        .cache_runs(latest, 1, &[run_of("BBB BBB"), run_of("CCC CCC")])
        .unwrap();

    assert_eq!(
        store.live_runs(1).unwrap(),
        Some(vec![run_of("BBB BBB"), run_of("CCC CCC")])
    );
}

#[test]
fn there_are_no_live_runs_before_any_launch_was_read() {
    let (_d, store) = temp_store();
    let session = store
        .import_session("09_12_2026__13_34_26", &key(0), &[started("SSS SSS")])
        .unwrap();
    store.cache_runs(session, 1, &[run_of("SSS SSS")]).unwrap();

    assert_eq!(store.live_runs(1).unwrap(), None);
}

#[test]
fn the_latest_launch_folded_under_other_rules_has_no_live_runs() {
    // The same rule `cached_runs` keeps: a fold another rules file produced is not an answer.
    let (_d, store) = temp_store();
    let latest = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(latest, 1, &[run_of("AAA AAA")]).unwrap();

    assert_eq!(store.live_runs(2).unwrap(), None);
}

#[test]
fn a_source_is_stale_when_these_rules_did_not_fold_it() {
    // Never folded, folded by other rules, folded by these: only the last is current.
    let (_d, store) = temp_store();
    let never = store.insert_log_source(&key(0)).unwrap();
    let other = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(other, 1, &[run_of("AAA AAA")]).unwrap();
    let current = store.insert_log_source(&key(0)).unwrap();
    store.cache_runs(current, 2, &[]).unwrap();

    assert_eq!(store.stale_sources(2).unwrap(), vec![never, other]);
}
