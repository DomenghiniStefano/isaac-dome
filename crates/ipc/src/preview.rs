//! What a candidate's save says about itself, read from the `.dat` alone.
//!
//! No `catalog`: counting needs the file's own layout, which lives in `core-save`; naming a
//! character needs the game's archives and is a different question. So this answers on a
//! machine with no game installed, which is where the welcome matters most.

use core_save::{Kind, Save};
use serde::Serialize;

use crate::marks::marks_totals;

/// A count the file may not let us make. **Never `Read { done: 0 }` for a section that was
/// not read**: a zero is a profile at the start, and the two are not one sentence. Same
/// choice as `Verdict::Partial` in `graph` and `Generated::NotSaid` in `run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PreviewCount {
    Read { done: u32, of: u32 },
    Unread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePreview {
    pub achievements: PreviewCount,
    pub items: PreviewCount,
    /// Cells that reached a level, out of the cells the file lets us read — the denominator
    /// Completion already uses, where what cannot be read stays outside it.
    pub marks: PreviewCount,
    /// Cells the file does not let us read. Zero when the matrix is whole, and zero as well
    /// when there is no counters section at all: that absence is `marks: Unread`, not 408
    /// unreadable cells.
    pub unreadable_cells: u32,
}

pub fn preview_of(save: &Save) -> CandidatePreview {
    let totals = save.u32s(Kind::Counters).map(|c| marks_totals(&c));
    CandidatePreview {
        achievements: slots(save.flags(Kind::Achievements).as_deref()),
        items: slots(save.flags(Kind::Items).as_deref()),
        marks: match totals {
            Some(t) => PreviewCount::Read {
                done: t.normal as u32,
                of: t.readable as u32,
            },
            None => PreviewCount::Unread,
        },
        unreadable_cells: totals.map_or(0, |t| (t.unknown + t.unexpected) as u32),
    }
}

/// Slot 0 is neither an achievement nor an item — `unlock_view` skips it and
/// `collection_view` says so out loud — so it is out of the numerator *and* the denominator.
///
/// This is the first place a **user** sees an achievements total: `UnlockTotals.slots` counts
/// the file's slots, 642 on the 2026 era, and is drawn only on the development-only
/// verification page.
fn slots(flags: Option<&[bool]>) -> PreviewCount {
    match flags {
        None => PreviewCount::Unread,
        Some(f) => PreviewCount::Read {
            done: f.iter().skip(1).filter(|&&b| b).count() as u32,
            of: f.len().saturating_sub(1) as u32,
        },
    }
}
