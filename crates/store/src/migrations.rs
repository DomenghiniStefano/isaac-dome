//! The schema, versioned with `PRAGMA user_version`. Each migration takes it from N-1 to N
//! and never reruns: whoever already has the app only gets the new ones. Never tables
//! "ahead of time".

use rusqlite::Connection;

use crate::StoreError;

/// The version this binary knows how to read and write.
pub const SCHEMA_VERSION: u32 = 4;

/// Index = version − 1. Append at the end, never modify a migration that's already shipped.
const MIGRATIONS: [&str; 4] = [
    // 1: the user's goals. `target_json` is the serialized `ipc::TargetKey` -- identity
    // alone, never name or icon: a column per variant would be a schema that changes
    // with every new kind of unlock.
    "CREATE TABLE goals (
        id TEXT PRIMARY KEY,
        target_json TEXT NOT NULL,
        created_unix INTEGER NOT NULL,
        note TEXT,
        seq INTEGER NOT NULL
    );",
    // 2: the plan queue, as one JSON document rather than a row per achievement. The
    // order is the position in the array, so there is no sequence column that could
    // contradict itself. One row, pinned by the CHECK: a second document would be a
    // second answer to "what is the order". It seeds nothing — resolving a saved target
    // to the achievement that unlocks it needs the catalog, which this crate doesn't
    // have and doesn't depend on.
    "CREATE TABLE plan_queue (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        rows_json TEXT NOT NULL
    );",
    // 3: the window session, one JSON document written by the main window. An object with a
    // version, not a bare array of tabs: 3.7 adds the sidebar's width and each table's size
    // (B27) as named parts of the same document, and a named part costs no migration. What
    // the parts mean is the frontend's business -- this crate stores the string.
    "CREATE TABLE window_session (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        document TEXT NOT NULL
    );",
    // 4: the run archive. `events` are the archive and a run is a fold over them (M4's second
    // decision), so `runs` is a cache and says so by carrying the rules version that produced
    // it: a newer rules file finds nothing and the fold runs again.
    //
    // `key` is NULL for a launch of log.txt and the folder's name for an online session. A
    // launch has no name — and SQLite counts NULLs as distinct in a UNIQUE, so two launches are
    // two rows and the older one keeps its events when the game rewrites the file. A schema
    // that keyed a launch by its prefix hash would have the second launch overwrite the first,
    // which is the archive losing what it exists to keep.
    //
    // `read_offset`, not `offset`: OFFSET is a keyword.
    "CREATE TABLE sources (
        id INTEGER PRIMARY KEY,
        kind TEXT NOT NULL,
        key TEXT,
        prefix_hash TEXT NOT NULL,
        prefix_len INTEGER NOT NULL,
        anchor_hash TEXT NOT NULL,
        read_offset INTEGER NOT NULL,
        UNIQUE (kind, key)
    );
    CREATE TABLE events (
        source_id INTEGER NOT NULL REFERENCES sources(id),
        seq INTEGER NOT NULL,
        event_json TEXT NOT NULL,
        PRIMARY KEY (source_id, seq)
    );
    CREATE TABLE runs (
        source_id INTEGER NOT NULL REFERENCES sources(id),
        ordinal INTEGER NOT NULL,
        rules_version INTEGER NOT NULL,
        run_json TEXT NOT NULL,
        PRIMARY KEY (source_id, ordinal)
    );",
];

pub fn current_version(conn: &Connection) -> Result<u32, StoreError> {
    conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
        .map(|v| v as u32)
        .map_err(StoreError::from_sqlite)
}

/// Brings the file to the current version. `found` is the version the caller already
/// read: it avoids reading `user_version` a second time. A file newer than us is
/// refused without being touched: an old binary must never corrupt a newer one's data.
pub fn apply(conn: &Connection, found: u32) -> Result<(), StoreError> {
    if found > SCHEMA_VERSION {
        return Err(StoreError::NewerSchema {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let target = i as u32 + 1;
        if target <= found {
            continue;
        }
        let tx = conn
            .unchecked_transaction()
            .map_err(StoreError::from_sqlite)?;
        tx.execute_batch(sql).map_err(StoreError::from_sqlite)?;
        tx.pragma_update(None, "user_version", target)
            .map_err(StoreError::from_sqlite)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
    }
    Ok(())
}
