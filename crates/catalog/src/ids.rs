//! Ids as newtypes: never bare `u32` in public signatures.

use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u32);

        /// The number, as the game writes it: an id in a message reads the way it did as a
        /// bare `u32`.
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(
    ItemId,
    "Item id, as in `items.xml` and in section 4 of the save."
);
id_type!(CharacterId, "Character id, as in `players.xml`.");
id_type!(
    AchievementId,
    "Achievement id, as in `achievements.xml` and in section 1."
);
id_type!(
    ChallengeId,
    "Challenge id, as in `challenges.xml` and in section 7 of the save."
);
id_type!(BossId, "Boss id, as in `bossportraits.xml`.");
