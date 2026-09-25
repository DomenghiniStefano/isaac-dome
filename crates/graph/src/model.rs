//! The domain types. They don't cross the IPC: `ipc` builds its own views of them, and only
//! the catalog's own enums (`ItemKind`, `Origin`) travel as they are.

use catalog::{AchievementId, BossId, ChallengeId, CharacterId, ItemId, ItemKind};

/// One item of a `Threshold`'s set, resolved against the catalog. A struct and not a tuple
/// because the two halves are read together at four call sites, and `(kind, id)` reversed
/// is a bug the compiler cannot see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdItem {
    pub kind: ItemKind,
    pub id: ItemId,
    /// The achievement that gates this item, if the game gates it at all. Resolved when the
    /// graph is built, because `Graph::evaluate` has a profile and **no catalog**: every
    /// other requirement answers that question by turning it into a prerequisite edge, and
    /// a threshold is precisely the one that must not draw edges.
    pub unlocked_by: Option<AchievementId>,
}

/// What an achievement demands. The type is the point: it says whether the prerequisite
/// has an achievement of its own behind it (character, challenge, item) or is content
/// available from the start (Satan, Mom).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    Character {
        id: CharacterId,
    },
    Boss {
        id: BossId,
    },
    Challenge {
        id: ChallengeId,
    },
    Item {
        kind: ItemKind,
        id: ItemId,
    },
    /// Stage, room, mode: outside the catalog by nature. `gate` is the target's key in the
    /// curated table — `kind:label` — and not an id of the game's: these targets have none
    /// on our side.
    Gate {
        gate: String,
    },
    /// Declared, never dropped: the node knows what it doesn't know.
    Unknown {
        label: String,
    },
    /// One cell of the completion matrix: "beat `column` as `character`, at `level`".
    /// Symbolic on purpose — the cell's index into the save is not this crate's business,
    /// and a rules file carrying an offset would be the same mistake as an offset crossing
    /// the IPC.
    Mark {
        character: CharacterId,
        column: crate::rules::MarkColumn,
        level: crate::rules::MarkLevel,
    },
    /// A tally of section 2, at or above a threshold. Named, never indexed.
    ///
    /// Separate from `Mark` and not folded into one `Progress` because the two are
    /// different things to the player: a cell is "you, with this character", a tally is
    /// "anyone, ever". Fusing them would erase the distinction the screen draws.
    Counter {
        name: crate::rules::CounterName,
        at_least: u32,
    },
    /// N of a set of items, in any combination: a transformation.
    ///
    /// Not a wall like the others, and it never draws a prerequisite edge: the
    /// prerequisites of *any three of these eight* are a disjunction of subsets, which this
    /// model cannot say and must not fake. `Graph::evaluate` answers it against the profile,
    /// the way `Mark` and `Counter` are answered.
    Threshold {
        transformation: u32,
        label: String,
        at_least: u32,
        of: Vec<ThresholdItem>,
        /// Contributors the wiki names and this catalog does not have. Carried as a number
        /// rather than dropped, because evaluation has to know the tally is incomplete —
        /// an unresolved item can only ever *add* to it.
        unresolved: u32,
    },
    /// Judged as gating nothing — `alwaysAvailable` or `notAPrerequisite`. A variant
    /// rather than a ref filtered away at the source, so that resolution is **total**:
    /// every ref maps to exactly one outcome and none disappears without a name.
    None,
}
