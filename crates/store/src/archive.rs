//! The run archive: where a log was read from, what it said, and the fold cached over it.
//!
//! The events are the archive and a run is a fold over them. That is why `cache_runs` carries a
//! rules version and `events` does not: one of the two tables can be thrown away and rebuilt,
//! and it is not the one the game wrote.

use run::{Event, Run, SourceKey};
use rusqlite::{params, OptionalExtension};

use crate::{Store, StoreError};

/// Which kind of file a source is. A closed set: a third kind would be a third way of finding
/// runs, which is a design decision and not a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// One launch of the game, read from `log.txt`.
    Log,
    /// One online session folder under `online_logs\`.
    Session,
}

impl SourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Session => "session",
        }
    }

    /// A kind written by a version that knew more is `None`, not a panic: the row is skipped
    /// and the rest of the archive still reads.
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "log" => Some(Self::Log),
            "session" => Some(Self::Session),
            _ => None,
        }
    }
}

/// A source as the database holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSource {
    pub id: i64,
    pub kind: SourceKind,
    /// The session's folder name. `None` for a launch, which has no name.
    pub key: Option<String>,
    pub source_key: SourceKey,
}

/// Events read back. A row that does not parse is counted rather than hidden, the way an
/// unreadable goal is named rather than dropped.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventsRead {
    pub events: Vec<Event>,
    pub unreadable: u32,
}

/// A hash as it is stored: hex, so the file is readable and the sign of an i64 never enters
/// into it.
fn hex(v: u64) -> String {
    format!("{v:016x}")
}

fn unhex(raw: &str) -> Option<u64> {
    u64::from_str_radix(raw, 16).ok()
}

impl Store {
    /// The source of an online session, by the folder's name.
    pub fn session_source(&self, name: &str) -> Result<Option<StoredSource>, StoreError> {
        self.conn
            .query_row(
                "SELECT id, kind, key, prefix_hash, prefix_len, anchor_hash, read_offset
                 FROM sources WHERE kind = 'session' AND key = ?1",
                params![name],
                source_row,
            )
            .optional()
            .map_err(StoreError::from_sqlite)
            .map(|r| r.flatten())
    }

    /// The most recent launch of `log.txt`, which is the one the watcher may still be appending
    /// to. Older ones stay in the table with their events.
    pub fn latest_log_source(&self) -> Result<Option<StoredSource>, StoreError> {
        self.conn
            .query_row(
                "SELECT id, kind, key, prefix_hash, prefix_len, anchor_hash, read_offset
                 FROM sources WHERE kind = 'log' ORDER BY id DESC LIMIT 1",
                [],
                source_row,
            )
            .optional()
            .map_err(StoreError::from_sqlite)
            .map(|r| r.flatten())
    }

    /// Every source, oldest first. The order the archive is shown in.
    pub fn sources(&self) -> Result<Vec<StoredSource>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, key, prefix_hash, prefix_len, anchor_hash, read_offset
                 FROM sources ORDER BY id",
            )
            .map_err(StoreError::from_sqlite)?;
        let rows = stmt
            .query_map([], source_row)
            .map_err(StoreError::from_sqlite)?;
        let mut out = Vec::new();
        for row in rows {
            // A row this binary cannot read is skipped, not fatal.
            if let Some(source) = row.map_err(StoreError::from_sqlite)? {
                out.push(source);
            }
        }
        Ok(out)
    }

    pub fn insert_session_source(&self, name: &str, key: &SourceKey) -> Result<i64, StoreError> {
        self.insert_source(SourceKind::Session, Some(name), key)
    }

    pub fn insert_log_source(&self, key: &SourceKey) -> Result<i64, StoreError> {
        self.insert_source(SourceKind::Log, None, key)
    }

    /// Where this source has been read to, after a read advanced it.
    pub fn set_source_key(&self, id: i64, key: &SourceKey) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE sources SET prefix_hash = ?2, prefix_len = ?3, anchor_hash = ?4,
                 read_offset = ?5 WHERE id = ?1",
                params![
                    id,
                    hex(key.prefix),
                    key.prefix_len as i64,
                    hex(key.anchor),
                    key.offset as i64
                ],
            )
            .map(|_| ())
            .map_err(StoreError::from_sqlite)
    }

    /// Appends after whatever is already there, and answers how many rows it wrote.
    pub fn append_events(&self, source_id: i64, events: &[Event]) -> Result<u32, StoreError> {
        let next: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(max(seq), -1) + 1 FROM events WHERE source_id = ?1",
                params![source_id],
                |r| r.get(0),
            )
            .map_err(StoreError::from_sqlite)?;
        let mut written = 0;
        for (i, event) in events.iter().enumerate() {
            // An event that will not serialize cannot happen (no maps, no floats), and if it
            // ever did it would cost one row and not the whole read.
            let Ok(json) = serde_json::to_string(event) else {
                continue;
            };
            self.conn
                .execute(
                    "INSERT INTO events (source_id, seq, event_json) VALUES (?1, ?2, ?3)",
                    params![source_id, next + i as i64, json],
                )
                .map_err(StoreError::from_sqlite)?;
            written += 1;
        }
        Ok(written)
    }

    /// Every event of a source, in the order the log wrote them.
    pub fn events(&self, source_id: i64) -> Result<EventsRead, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT event_json FROM events WHERE source_id = ?1 ORDER BY seq")
            .map_err(StoreError::from_sqlite)?;
        let rows = stmt
            .query_map(params![source_id], |r| r.get::<_, String>(0))
            .map_err(StoreError::from_sqlite)?;
        let mut out = EventsRead::default();
        for row in rows {
            let json = row.map_err(StoreError::from_sqlite)?;
            match serde_json::from_str::<Event>(&json) {
                Ok(e) => out.events.push(e),
                Err(_) => out.unreadable += 1,
            }
        }
        Ok(out)
    }

    /// Replaces the fold of this source. The ordinal is the position in the log.
    pub fn cache_runs(
        &self,
        source_id: i64,
        rules_version: u32,
        runs: &[Run],
    ) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM runs WHERE source_id = ?1", params![source_id])
            .map_err(StoreError::from_sqlite)?;
        for (ordinal, run) in runs.iter().enumerate() {
            let Ok(json) = serde_json::to_string(run) else {
                continue;
            };
            self.conn
                .execute(
                    "INSERT INTO runs (source_id, ordinal, rules_version, run_json)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![source_id, ordinal as i64, rules_version, json],
                )
                .map_err(StoreError::from_sqlite)?;
        }
        Ok(())
    }

    /// The cached fold, **only** if it was produced by these rules. A newer rules file finds
    /// nothing here, which is how the cache invalidates itself.
    pub fn cached_runs(
        &self,
        source_id: i64,
        rules_version: u32,
    ) -> Result<Option<Vec<Run>>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT run_json FROM runs WHERE source_id = ?1 AND rules_version = ?2
                 ORDER BY ordinal",
            )
            .map_err(StoreError::from_sqlite)?;
        let rows = stmt
            .query_map(params![source_id, rules_version], |r| r.get::<_, String>(0))
            .map_err(StoreError::from_sqlite)?;
        let mut runs = Vec::new();
        for row in rows {
            let json = row.map_err(StoreError::from_sqlite)?;
            match serde_json::from_str::<Run>(&json) {
                Ok(r) => runs.push(r),
                // A cache row that will not parse is a cache row: throw the cache away and let
                // the caller fold again.
                Err(_) => return Ok(None),
            }
        }
        Ok((!runs.is_empty()).then_some(runs))
    }

    fn insert_source(
        &self,
        kind: SourceKind,
        key: Option<&str>,
        source_key: &SourceKey,
    ) -> Result<i64, StoreError> {
        self.conn
            .execute(
                "INSERT INTO sources (kind, key, prefix_hash, prefix_len, anchor_hash, read_offset)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    kind.as_str(),
                    key,
                    hex(source_key.prefix),
                    source_key.prefix_len as i64,
                    hex(source_key.anchor),
                    source_key.offset as i64
                ],
            )
            .map_err(StoreError::from_sqlite)?;
        Ok(self.conn.last_insert_rowid())
    }

    pub(crate) fn replace_event_json(
        &self,
        source_id: i64,
        seq: i64,
        raw: &str,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE events SET event_json = ?3 WHERE source_id = ?1 AND seq = ?2",
                params![source_id, seq, raw],
            )
            .map(|_| ())
            .map_err(StoreError::from_sqlite)
    }
}

/// `None` when the row names a kind this binary does not know: written by a newer app, and
/// skipped rather than guessed at.
fn source_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Option<StoredSource>> {
    let id: i64 = r.get(0)?;
    let kind: String = r.get(1)?;
    let key: Option<String> = r.get(2)?;
    let prefix: String = r.get(3)?;
    let prefix_len: i64 = r.get(4)?;
    let anchor: String = r.get(5)?;
    let offset: i64 = r.get(6)?;
    Ok(
        match (SourceKind::parse(&kind), unhex(&prefix), unhex(&anchor)) {
            (Some(kind), Some(prefix), Some(anchor)) => Some(StoredSource {
                id,
                kind,
                key,
                source_key: SourceKey {
                    prefix,
                    prefix_len: prefix_len as u64,
                    anchor,
                    offset: offset as u64,
                },
            }),
            _ => None,
        },
    )
}
