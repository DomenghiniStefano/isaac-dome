//! What the save says about progress, in the shape the graph asks for.
//!
//! This is the only place that knows a column is a block base and a tally is an index:
//! `graph` names them and never learns where they live. It shares the layout's `Column` so
//! a requirement names a column with the same twelve values the matrix draws, and nothing
//! more of the file: the index behind a cell is read here, through the same decoder the
//! matrix uses.
//!
//! Not to be confused with [`crate::profile`], which is about *which save file* is the
//! active one. This module is about what is inside it.

use std::collections::BTreeMap;

use catalog::{Catalog, CharacterId};
use core_save::counter_index_of;
use graph::rules::{CounterName, MarkColumn, MarkLevel};

use crate::marks::{cell_at, character_for, Cell, CellLevel, ROSTER};

pub struct SaveProgress<'a> {
    flags: Option<&'a [bool]>,
    counters: Option<&'a [u32]>,
    /// `CharacterId` → matrix row. Built once: resolution happens per requirement, and a
    /// scan of the catalog per lookup would show on 170 of them.
    rows: BTreeMap<u32, usize>,
}

impl<'a> SaveProgress<'a> {
    pub fn new(
        flags: Option<&'a [bool]>,
        counters: Option<&'a [u32]>,
        catalog: Option<&Catalog>,
    ) -> SaveProgress<'a> {
        SaveProgress {
            flags,
            counters,
            rows: catalog.map(rows_by_character).unwrap_or_default(),
        }
    }

    /// `None` — cannot say: the cell isn't located, section 2 wasn't read or doesn't reach
    /// that far, or the value isn't a mask. `Some(None)` — read, nothing reached.
    /// `Some(Some(level))` — the highest level its bits show.
    ///
    /// Read through [`cell_at`], the decoder the matrix draws with (card #82, S3), so bit 2 —
    /// "won online", measured 2026-09-12, not a level — is left out here for the same reason
    /// it is left out of a drawn cell's level.
    pub fn level_at(&self, row: usize, column: MarkColumn) -> Option<Option<MarkLevel>> {
        match cell_at(self.counters?, row, column) {
            Cell::Known { level, .. } => Some(reached_level(level)),
            // Not located, past the end of the section, or outside the mask's range: a level
            // read out of any of them would be invented.
            Cell::Unknown | Cell::Unexpected { .. } => None,
        }
    }
}

/// Every matrix row the catalog names, by the id of the character it names.
fn rows_by_character(c: &Catalog) -> BTreeMap<u32, usize> {
    (0..ROSTER.len())
        .filter_map(|row| character_for(row, c).map(|ch| (ch.id.0, row)))
        .collect()
}

/// The matrix's level as the graph names it: the same two bits, `normal`/`hard` on the
/// screen and `base`/`second` in a rule.
fn reached_level(level: CellLevel) -> Option<MarkLevel> {
    match level {
        CellLevel::Empty => None,
        CellLevel::Normal => Some(MarkLevel::Base),
        CellLevel::Hard => Some(MarkLevel::Second),
    }
}

impl graph::evaluate::Profile for SaveProgress<'_> {
    fn done(&self) -> Option<&[bool]> {
        self.flags
    }

    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<Option<MarkLevel>> {
        // No catalog, no way from an id to a row — so the question cannot be answered at
        // all. Answering about row 0 instead would be a different character's cell.
        self.level_at(*self.rows.get(&character.0)?, column)
    }

    fn counter(&self, name: CounterName) -> Option<u32> {
        self.counters?.get(counter_index_of(name)).copied()
    }
}
