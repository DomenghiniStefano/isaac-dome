//! The domain types. They don't cross the IPC: `ipc` defines its own views, as it already
//! does for `ItemKindView` and `OriginView`.

use catalog::{BossId, ChallengeId, CharacterId, ItemId, ItemKind};

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
    /// Judged as gating nothing — `alwaysAvailable` or `notAPrerequisite`. A variant
    /// rather than a ref filtered away at the source, so that resolution is **total**:
    /// every ref maps to exactly one outcome and none disappears without a name.
    None,
}
