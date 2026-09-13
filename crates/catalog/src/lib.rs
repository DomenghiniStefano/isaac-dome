//! catalog — the game's XML files as a model queryable by id. Pure: no I/O.

mod achievements;
mod anm2;
mod bossportraits;
mod catalog;
mod challenges;
mod diagnostics;
pub mod for_tests;
mod heads;
mod ids;
mod itempools;
mod items;
mod metadata;
mod origin;
mod players;
mod reward;
mod sprite;
mod strings;
mod text;
mod unlock;
mod xml;

pub use achievements::Achievement;
pub use anm2::{frames as anm2_frames, spritesheets as anm2_spritesheets, Anm2Frame};
pub use bossportraits::Boss;
pub use catalog::Catalog;
pub use catalog::SOURCES;
pub use challenges::Challenge;
pub use diagnostics::{Diagnostic, SkipReason, Source};
pub use ids::{AchievementId, BossId, ChallengeId, CharacterId, ItemId};
pub use itempools::{Pool, PoolEntry, PoolMembership};
pub use items::{Item, ItemKind};
// `origin_of` alongside `Origin`: it is a pure lookup over id boundaries verified against
// `items.xml`, so a caller can ask which edition introduced an id without a game installed.
pub use origin::{origin_of, Origin};
pub use players::Character;
pub use sprite::{Rect, SpriteRef};
pub use text::{Language, Text};
pub use unlock::Unlock;
