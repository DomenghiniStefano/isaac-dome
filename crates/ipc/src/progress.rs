//! What the save says about progress, in the shape the graph asks for.
//!
//! This is the only place that knows a column is a block base and a tally is an index:
//! `graph` names them and never learns where they live. The direction matters — it is why
//! `graph` gains no dependency on `core-save` and no knowledge of the file.
//!
//! Not to be confused with [`crate::profile`], which is about *which save file* is the
//! active one. This module is about what is inside it.

use std::collections::BTreeMap;

use catalog::{Catalog, CharacterId};
use core_save::marks::{cell_index, counter_index_of, Column, CounterKey};
use graph::rules::{CounterName, MarkColumn, MarkLevel};

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
        let mut rows = BTreeMap::new();
        if let Some(c) = catalog {
            for row in 0..core_save::marks::ROWS {
                if let Some(ch) = crate::marks::character_for(row, c) {
                    rows.insert(ch.id.0, row);
                }
            }
        }
        SaveProgress {
            flags,
            counters,
            rows,
        }
    }

    /// `None` — cannot say: the cell isn't located, section 2 wasn't read or doesn't reach
    /// that far, or the value isn't a mask. `Some(None)` — read, nothing reached.
    /// `Some(Some(level))` — the highest level its bits show.
    ///
    /// Bit 2 is masked away first: it is "won online" (measured 2026-09-12), not a level,
    /// and reading it as one would show a second level nobody reached.
    pub fn level_at(&self, row: usize, column: MarkColumn) -> Option<Option<MarkLevel>> {
        let i = cell_index(row, column_of(column))?;
        let v = *self.counters?.get(i)?;
        if v > 7 {
            // Outside the mask's range: the index points at something else, and a level
            // read out of it would be invented.
            return None;
        }
        Some(match v & 0b11 {
            0b00 => None,
            0b01 => Some(MarkLevel::Base),
            _ => Some(MarkLevel::Second),
        })
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
        self.counters?.get(counter_index_of(key_of(name))).copied()
    }
}

/// The graph's column, as the layout's. Written out with no `_` arm: the two are the same
/// twelve in the same order, and the day one of them gains a thirteenth the build breaks.
fn column_of(c: MarkColumn) -> Column {
    match c {
        MarkColumn::MomsHeart => Column::MomsHeart,
        MarkColumn::Isaac => Column::Isaac,
        MarkColumn::Satan => Column::Satan,
        MarkColumn::BossRush => Column::BossRush,
        MarkColumn::BlueBaby => Column::BlueBaby,
        MarkColumn::TheLamb => Column::TheLamb,
        MarkColumn::MegaSatan => Column::MegaSatan,
        MarkColumn::Greed => Column::Greed,
        MarkColumn::Hush => Column::Hush,
        MarkColumn::Delirium => Column::Delirium,
        MarkColumn::Mother => Column::Mother,
        MarkColumn::TheBeast => Column::TheBeast,
    }
}

/// Same for the tallies: the rules file spells a name, the layout holds the number, and an
/// exhaustive match is what keeps them joined.
fn key_of(n: CounterName) -> CounterKey {
    match n {
        CounterName::HushKills => CounterKey::HushKills,
        CounterName::DeliriumKills => CounterKey::DeliriumKills,
        CounterName::MotherKills => CounterKey::MotherKills,
        CounterName::BeastKills => CounterKey::BeastKills,
    }
}
