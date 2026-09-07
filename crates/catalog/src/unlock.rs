//! What an achievement unlocks: the inverse of the `unlocked_by` links scattered
//! across the files. Built once in `Catalog::build`; the graph (M2) and Unlock read it.

use crate::ids::{AchievementId, BossId, ChallengeId, CharacterId, ItemId};
use crate::items::ItemKind;

/// An edge achievement -> what it unlocks. The derived order (variant, then id) makes
/// the index deterministic without a hand-written comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unlock {
    Item { kind: ItemKind, id: ItemId },
    Character { id: CharacterId },
    Boss { id: BossId },
    Challenge { id: ChallengeId },
}

/// Marks the type for readers; the actual work is done by `Catalog::build`.
pub(crate) type Index = std::collections::BTreeMap<AchievementId, Vec<Unlock>>;
