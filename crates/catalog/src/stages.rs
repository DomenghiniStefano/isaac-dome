//! `stages.xml`: one row per stage *file*, keyed by the game's own stage id, its `name` a string
//! key (`#BASEMENT_NAME`) and not a name. Measured on the installed game on 2026-10-04: ids 1–17
//! are the original floors, 26 The Void, 27–33 the Repentance alternates, 35 Home, and the greed
//! stages 18–25 sit inside an XML comment, so they are not rows at all.
//!
//! The log does not use these ids: it writes `Level::Init m_Stage 2, m_StageType 1`. The link from
//! that pair to a row is [`file_of`], which no game file holds.

use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, Source};
use crate::text::Text;
use crate::xml;

const SOURCE: Source = Source::Stages;

/// Every `<stage>` row with an id and a name, by id. A row without either is not a floor the
/// table can point at, and is left out rather than diagnosed: the file's first row is the
/// special rooms, which no floor is.
pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> BTreeMap<u32, Text> {
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return BTreeMap::new();
    };
    els.iter()
        .filter(|e| e.name == "stage")
        .filter_map(|e| {
            let id = e.attr("id")?.parse().ok()?;
            Some((id, Text::from_attr(e.attr("name")?)))
        })
        .collect()
}

/// The `stages.xml` id the log's `(m_Stage, m_StageType)` names, and whether the floor is one of
/// a chapter's two (Basement I and II) rather than a floor of its own (the Blue Womb).
///
/// **Not in any game file.** It is the game's own enumeration as the modding API documents it:
/// `LevelStage` 1–13, two stages per chapter for the first four, and `StageType` 0 the original
/// floor, 1 Wrath of the Lamb's, 2 Afterbirth's, 4 Repentance's and 5 Repentance's second (3 is
/// Greed's, which this table leaves out). Held against the one pair this repo has measured on a
/// real log: `m_Stage 4, m_StageType 4` is Mines II (`docs/log-format.md`).
pub fn file_of(stage: u32, stage_type: u32) -> Option<(u32, bool)> {
    let id = match stage {
        1 | 2 => chapter(stage_type, [1, 2, 3, 27, 28])?,
        3 | 4 => chapter(stage_type, [4, 5, 6, 29, 30])?,
        5 | 6 => chapter(stage_type, [7, 8, 9, 31, 32])?,
        // The womb has no second Repentance floor: Corpse is its only alternate.
        7 | 8 if stage_type == 5 => return None,
        7 | 8 => chapter(stage_type, [10, 11, 12, 33, 33])?,
        9 => 13,
        10 => match stage_type {
            0 => 14,
            1 => 15,
            _ => return None,
        },
        11 => match stage_type {
            0 => 16,
            1 => 17,
            _ => return None,
        },
        12 => 26,
        13 => 35,
        _ => return None,
    };
    Some((id, (1..=8).contains(&stage)))
}

/// One chapter's five files, in `StageType` order: original, Wrath of the Lamb, Afterbirth,
/// Repentance, Repentance B. `StageType` 3 is Greed's and any other value is not a floor here.
fn chapter(stage_type: u32, files: [u32; 5]) -> Option<u32> {
    match stage_type {
        0 => Some(files[0]),
        1 => Some(files[1]),
        2 => Some(files[2]),
        4 => Some(files[3]),
        5 => Some(files[4]),
        _ => None,
    }
}
