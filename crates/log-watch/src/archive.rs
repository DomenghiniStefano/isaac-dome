//! Ingesting a log into the archive. **Backfill and live are this one function**: a session
//! folder is a stream that is finished and `log.txt` is a stream that is still growing, which
//! is the whole difference between them.

use std::path::Path;

use run::{
    resume, Event, ItemKinds, Resume, Rules, Run, SourceKey, Tail, ANCHOR_BYTES, PREFIX_BYTES,
};
use store::Store;

use crate::read::{chunk, head, len, window_ending_at, CHUNK};
use crate::WatchError;

/// What one ingest did. Numbers, not a sentence: the caller logs them and the tests read them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ingested {
    pub source_id: i64,
    /// Events appended by this pass, not the total in the archive.
    pub events: u32,
    /// Runs the source folds to now, which is a total.
    pub runs: u32,
}

/// The three things ingesting needs: where to put it, how to read a line, and what an item is.
pub struct Ingest<'a> {
    pub store: &'a Store,
    pub rules: &'a Rules,
    pub kinds: &'a dyn ItemKinds,
}

impl Ingest<'_> {
    /// One online session. `None` when it was already imported: the folder's name is its
    /// identity, and a folder read twice must not become two archives.
    pub fn session(&self, folder: &Path) -> Result<Option<Ingested>, WatchError> {
        let Some(name) = folder.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };
        if self.store.session_source(name)?.is_some() {
            return Ok(None);
        }
        let log = folder.join("log.txt");
        let (events, offset) = self.read_events(&log, 0)?;
        let written = events.len() as u32;
        let id = self
            .store
            .import_session(name, &self.key_at(&log, offset)?, &events)?;
        let runs = self.refold(id)?;
        Ok(Some(Ingested {
            source_id: id,
            events: written,
            runs,
        }))
    }

    /// `log.txt`, from wherever we stopped. The decision of *where* that is belongs to
    /// `run::resume`; this supplies the bytes it asks for.
    pub fn live_log(&self, log: &Path) -> Result<Ingested, WatchError> {
        let length = len(log)?;
        let (id, from) = match self.store.latest_log_source()? {
            Some(stored) => {
                // The prefix is read over **the window the stored hash covers**, not over
                // `PREFIX_BYTES`: a log is short for its first instants, and re-hashing a window
                // that has grown makes a file stop matching itself — which imports every run in
                // it a second time.
                let prefix = head(log, stored.source_key.prefix_len as usize)?;
                let anchor = window_ending_at(log, stored.source_key.offset, ANCHOR_BYTES)?;
                match resume(&stored.source_key, &prefix, length, &anchor) {
                    Resume::Continue { offset } => (stored.id, offset),
                    Resume::Fresh => (self.new_log_source(log)?, 0),
                }
            }
            None => (self.new_log_source(log)?, 0),
        };
        let (events, offset) = self.read_events(log, from)?;
        let written = self
            .store
            .append_to_log(id, &self.key_at(log, offset)?, &events)?;
        let runs = self.refold(id)?;
        Ok(Ingested {
            source_id: id,
            events: written,
            runs,
        })
    }

    /// Every session on the disk that is not in the archive yet. Errors are collected, never
    /// raised: one unreadable folder must not cost the other twenty-seven.
    ///
    /// Only the tests call it (card #81, C8): the app walks `sessions()` itself and ingests each
    /// folder under a lock of its own, so the database is never held for the whole backfill.
    #[cfg(feature = "test-api")]
    pub fn backfill(&self, online_logs: &Path) -> (Vec<Ingested>, Vec<WatchError>) {
        let (read, failed): (Vec<_>, Vec<_>) = crate::sessions(online_logs)
            .iter()
            .map(|folder| self.session(folder))
            .partition(Result::is_ok);
        (
            read.into_iter().filter_map(Result::ok).flatten().collect(),
            failed.into_iter().filter_map(Result::err).collect(),
        )
    }

    /// Folds again every source these rules have not folded, from the events already in the
    /// archive — no file is read. How many were folded, and what failed, collected like
    /// `backfill`'s: one source that will not fold must not cost the others.
    pub fn refold_stale(&self) -> (u32, Vec<WatchError>) {
        let stale = match self.store.stale_sources(self.rules.version()) {
            Ok(ids) => ids,
            Err(e) => return (0, vec![e.into()]),
        };
        let (folded, failed): (Vec<_>, Vec<_>) = stale
            .into_iter()
            .map(|id| self.refold(id))
            .partition(Result::is_ok);
        let errors = failed.into_iter().filter_map(Result::err).collect();
        (folded.len() as u32, errors)
    }

    fn new_log_source(&self, log: &Path) -> Result<i64, WatchError> {
        let prefix = head(log, PREFIX_BYTES)?;
        Ok(self
            .store
            .insert_log_source(&SourceKey::new(&prefix, &[], 0))?)
    }

    /// Reads from `from` to the end, a chunk at a time, and answers with the events and the
    /// offset of the **last complete line** — never the end of the last chunk, which may be the
    /// middle of a line the game is still writing.
    fn read_events(&self, log: &Path, from: u64) -> Result<(Vec<Event>, u64), WatchError> {
        let mut tail = Tail::default();
        let mut events = Vec::new();
        let mut offset = from;
        loop {
            let bytes = chunk(log, offset, CHUNK)?;
            if bytes.is_empty() {
                break;
            }
            offset += bytes.len() as u64;
            for line in tail.advance(&bytes) {
                if let Some(event) = self.rules.event(&line) {
                    events.push(event);
                }
            }
        }
        Ok((events, offset - tail.pending() as u64))
    }

    fn key_at(&self, log: &Path, offset: u64) -> Result<SourceKey, WatchError> {
        let prefix = head(log, PREFIX_BYTES)?;
        let anchor = window_ending_at(log, offset, ANCHOR_BYTES)?;
        Ok(SourceKey::new(&prefix, &anchor, offset))
    }

    /// The runs of a source, folded from **all** of its events and cached under the rules that
    /// produced them. A run spans chunks, so a partial fold would be a different answer.
    fn refold(&self, source_id: i64) -> Result<u32, WatchError> {
        let read = self.store.events(source_id)?;
        let runs = Run::fold(read.events.into_iter(), self.kinds);
        self.store
            .cache_runs(source_id, self.rules.version(), &runs)?;
        Ok(runs.len() as u32)
    }
}
