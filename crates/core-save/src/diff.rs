use serde::Serialize;

use crate::parse::Save;
use crate::section::Kind;

/// What changed going from save `a` to save `b`.
///
/// No screen reads it: it is kept for the measuring examples (`live_probe`, `matched_window`),
/// the instruments the save format is learned with.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SaveDiff {
    /// Indices that flipped from off to on in `b`.
    pub achievements: Vec<usize>,
    pub items: Vec<usize>,
    pub challenges: Vec<usize>,
    /// Bosses met for the first time. Named `cards_pills` until B9: 104 cells against the
    /// catalog's 103 bosses, and on a beginner profile the 48 that are off are exactly the
    /// late and alt-path roster.
    pub bosses: Vec<usize>,
    /// (index, value in `a`, value in `b`) where the counters differ.
    pub counters: Vec<(usize, u32, u32)>,
}

/// Structural diff. No semantic knowledge: only indices and values.
pub fn diff(a: &Save, b: &Save) -> SaveDiff {
    let newly_set = |kind: Kind| -> Vec<usize> {
        let fa = a.flags(kind).unwrap_or_default();
        let fb = b.flags(kind).unwrap_or_default();
        fb.iter()
            .enumerate()
            .filter(|&(i, &on)| on && !fa.get(i).copied().unwrap_or(false))
            .map(|(i, _)| i)
            .collect()
    };

    let ca = a.u32s(Kind::Counters).unwrap_or_default();
    let cb = b.u32s(Kind::Counters).unwrap_or_default();
    let counters = cb
        .iter()
        .enumerate()
        .filter_map(|(i, &y)| {
            let x = ca.get(i).copied().unwrap_or(0);
            (x != y).then_some((i, x, y))
        })
        .collect();

    SaveDiff {
        achievements: newly_set(Kind::Achievements),
        items: newly_set(Kind::Items),
        challenges: newly_set(Kind::Challenges),
        bosses: newly_set(Kind::Bosses),
        counters,
    }
}
