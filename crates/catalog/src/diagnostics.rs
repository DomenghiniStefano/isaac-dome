//! What went wrong, in typed form: no paths, no `Debug` strings.

use crate::ids::{AchievementId, ChallengeId};
use serde::Serialize;

/// The sources the catalog reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Items,
    Metadata,
    Strings,
    Players,
    CoopMenuAnm2,
    Achievements,
    ItemPools,
    Challenges,
    BossPortraits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkipReason {
    MissingId,
    MalformedId,
    MissingSprite,
    MissingName,
    /// A list `a,b,c` with a non-numeric element: the whole element is skipped.
    MalformedList,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Diagnostic {
    /// The file wasn't there.
    SourceMissing { source: Source },
    /// It was there, but quick-xml didn't accept it.
    SourceUnreadable { source: Source },
    /// An element was skipped; the rest of the file was read.
    ElementSkipped {
        source: Source,
        id: Option<u32>,
        reason: SkipReason,
    },
    /// A key the stringtable doesn't know: the key is shown as-is.
    UnresolvedKey { key: String },
    /// A new language in the stringtable: its strings are ignored.
    UnknownLanguage { name: String },
    /// The anm2 is missing or unmappable: heads stay `None`.
    HeadSheetUnavailable,
    /// An achievement claims to reward a challenge that `challenges.xml` doesn't have:
    /// the two files come from different eras. The achievement stays, the reward is
    /// assigned to no one.
    RewardForUnknownChallenge {
        achievement: AchievementId,
        challenge: ChallengeId,
    },
}
