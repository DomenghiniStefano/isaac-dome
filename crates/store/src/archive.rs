//! The run archive: where a log was read from, what it said, and the fold cached over it.
//!
//! The events are the archive and a run is a fold over them. That is why `cache_runs` carries a
//! rules version and `events` does not: one of the two tables can be thrown away and rebuilt,
//! and it is not the one the game wrote.

use ipc::{RunSource, RunsDiagnostic};
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

impl StoredSource {
    /// The source as the Runs screen names it. A session row with no name cannot be told from
    /// a launch, and the launch is the honest reading: it is the source with no name.
    pub fn run_source(&self) -> RunSource {
        match (self.kind, &self.key) {
            (SourceKind::Session, Some(name)) => RunSource::Session { name: name.clone() },
            (SourceKind::Session, None) | (SourceKind::Log, _) => RunSource::Live,
        }
    }
}

/// Every source's runs as a fold under one rules version produced them, named, and how many
/// sources had a cache that could not be read. What the Runs screen is built from.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchivedRuns {
    pub sources: Vec<(RunSource, Vec<Run>)>,
    pub unreadable: u32,
}

impl ArchivedRuns {
    /// What the Runs screen says about this read: a count of unreadable caches, and nothing
    /// when there were none.
    pub fn diagnostics(&self) -> Vec<RunsDiagnostic> {
        (self.unreadable > 0)
            .then_some(RunsDiagnostic::UnreadableEvents {
                count: self.unreadable,
            })
            .into_iter()
            .collect()
    }
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

    /// A launch of `log.txt` nobody has read yet.
    pub fn insert_log_source(&self, key: &SourceKey) -> Result<i64, StoreError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::from_sqlite)?;
        let id = self.insert_source(SourceKind::Log, None, key)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
        Ok(id)
    }

    /// A session, with its source row and its events written **together**.
    ///
    /// One write, because a source row with no events is a session marked imported for ever
    /// while holding nothing — and nothing would ever read that folder again.
    pub fn import_session(
        &self,
        name: &str,
        key: &SourceKey,
        events: &[Event],
    ) -> Result<i64, StoreError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::from_sqlite)?;
        let id = self.insert_source(SourceKind::Session, Some(name), key)?;
        self.write_events(id, events)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
        Ok(id)
    }

    /// Appends to a launch and moves its offset, **in one write**, answering how many rows it
    /// wrote.
    ///
    /// The two halves cannot be separate statements. Events written with the offset left behind
    /// are events the next read finds again and files a second time — the duplicate-runs failure
    /// the anchor exists to prevent, reached through a crash instead of through a bad guess.
    pub fn append_to_log(
        &self,
        source_id: i64,
        key: &SourceKey,
        events: &[Event],
    ) -> Result<u32, StoreError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::from_sqlite)?;
        let written = self.write_events(source_id, events)?;
        self.conn
            .execute(
                "UPDATE sources SET prefix_hash = ?2, prefix_len = ?3, anchor_hash = ?4,
                 read_offset = ?5 WHERE id = ?1",
                params![
                    source_id,
                    hex(key.prefix),
                    key.prefix_len as i64,
                    hex(key.anchor),
                    key.offset as i64
                ],
            )
            .map_err(StoreError::from_sqlite)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
        Ok(written)
    }

    /// Appends after whatever is already there. Inside a transaction opened by the caller: on
    /// its own, one `INSERT` per event is one commit per event, which is both slow and a way to
    /// leave half a read behind.
    fn write_events(&self, source_id: i64, events: &[Event]) -> Result<u32, StoreError> {
        let next: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(max(seq), -1) + 1 FROM events WHERE source_id = ?1",
                params![source_id],
                |r| r.get(0),
            )
            .map_err(StoreError::from_sqlite)?;
        let mut stmt = self
            .conn
            .prepare_cached("INSERT INTO events (source_id, seq, event_json) VALUES (?1, ?2, ?3)")
            .map_err(StoreError::from_sqlite)?;
        let mut written = 0;
        for (i, event) in events.iter().enumerate() {
            // An event that will not serialize cannot happen (no maps, no floats), and if it
            // ever did it would cost one row and not the whole read.
            let Ok(json) = serde_json::to_string(event) else {
                continue;
            };
            stmt.execute(params![source_id, next + i as i64, json])
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
    ///
    /// One write like the others: a cache half replaced is a source that reads as having fewer
    /// runs than it has, which is worse than one that reads as having none.
    ///
    /// The rules version is written **twice**: on each row, where it invalidates that row, and
    /// on the source, where it is the only record that the fold ran at all. A fold that produces
    /// no run writes no row, and a source with no rows is otherwise a source nobody has read.
    pub fn cache_runs(
        &self,
        source_id: i64,
        rules_version: u32,
        runs: &[Run],
    ) -> Result<(), StoreError> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::from_sqlite)?;
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
        self.conn
            .execute(
                "UPDATE sources SET folded_rules_version = ?2 WHERE id = ?1",
                params![source_id, rules_version],
            )
            .map_err(StoreError::from_sqlite)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
        Ok(())
    }

    /// The cached fold, **only** if it was produced by these rules. A newer rules file finds
    /// nothing here, which is how the cache invalidates itself.
    ///
    /// Three states in one `Option`, and the empty fold is the one that used to be lost:
    /// `None` is *nobody has folded this source under these rules*, and `Some(vec![])` is
    /// *these rules read it and it holds no run* — a launch that played the intro and shut
    /// down. The first asks the caller to fold; the second answers.
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
        if !runs.is_empty() {
            return Ok(Some(runs));
        }
        // No rows, which `runs` alone cannot explain: a fold that produced nothing and a source
        // nobody has folded leave the same absence there. The source says which.
        let folded: Option<u32> = self
            .conn
            .query_row(
                "SELECT folded_rules_version FROM sources WHERE id = ?1",
                params![source_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(StoreError::from_sqlite)?
            .flatten();
        Ok((folded == Some(rules_version)).then(Vec::new))
    }

    /// The cached runs of the launch the watcher follows — the latest `log.txt` — under
    /// `rules_version`, and no other source's (card #80, R10). What Live reads on every line the
    /// watcher reports: the whole archive would be every session ever played, read to keep one
    /// run. `None` when there is no launch, or when these rules have not folded it.
    pub fn live_runs(&self, rules_version: u32) -> Result<Option<Vec<Run>>, StoreError> {
        match self.latest_log_source()? {
            Some(source) => self.cached_runs(source.id, rules_version),
            None => Ok(None),
        }
    }

    /// Every source's cached runs under `rules_version`, named, oldest source first (card #81,
    /// V1: this was the body of the `runs` command). A source nobody folded under these rules
    /// contributes no row — it is folded again the next time its log is read — and an empty
    /// fold contributes a row with no run in it, which adds nothing to any total.
    pub fn archived_runs(&self, rules_version: u32) -> Result<ArchivedRuns, StoreError> {
        let mut archived = ArchivedRuns::default();
        for row in self.sources()? {
            match self.cached_runs(row.id, rules_version) {
                Ok(Some(runs)) => archived.sources.push((row.run_source(), runs)),
                Ok(None) => {}
                Err(_) => archived.unreadable += 1,
            }
        }
        Ok(archived)
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

    #[cfg(feature = "test-api")]
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
