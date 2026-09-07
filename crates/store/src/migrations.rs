//! The schema, versioned with `PRAGMA user_version`. Each migration takes it from N-1 to N
//! and never reruns: whoever already has the app only gets the new ones. Never tables
//! "ahead of time".

use rusqlite::Connection;

use crate::StoreError;

/// The version this binary knows how to read and write.
pub const SCHEMA_VERSION: u32 = 1;

/// Index = version − 1. Append at the end, never modify a migration that's already shipped.
const MIGRATIONS: [&str; 1] = [
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
