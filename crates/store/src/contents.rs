//! What the database holds, counted for the Data page. A count is a `COUNT(*)`; the queue is
//! one JSON document, so its length is read the way the Plan reads it, and a document that
//! does not parse is "unknown" rather than a failure of the whole answer.

use rusqlite::OptionalExtension;

use crate::{Store, StoreError};

impl Store {
    pub fn contents(&self) -> Result<ipc::StoreContents, StoreError> {
        Ok(ipc::StoreContents {
            goals: self.count("goals")?,
            queue_rows: self.queue()?.ok().map(|q| saturating_u32(q.rows().len())),
            sessions: self.count("sources")?,
            runs: self.count("runs")?,
            roll_saved: self.roll_row_exists()?,
        })
    }

    /// `table` is one of this crate's own names, never input.
    fn count(&self, table: &'static str) -> Result<u32, StoreError> {
        self.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .map_err(StoreError::from_sqlite)
    }

    fn roll_row_exists(&self) -> Result<bool, StoreError> {
        self.conn
            .query_row("SELECT 1 FROM roll WHERE id = 1", [], |_| Ok(()))
            .optional()
            .map(|found| found.is_some())
            .map_err(StoreError::from_sqlite)
    }
}

fn saturating_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
