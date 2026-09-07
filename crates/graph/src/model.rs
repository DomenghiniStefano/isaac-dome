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
    /// Judged as gating nothing — `alwaysAvailable` or `notAPrerequisite`. A variant
    /// rather than a ref filtered away at the source, so that resolution is **total**:
    /// every ref maps to exactly one outcome and none disappears without a name.
    None,
}
