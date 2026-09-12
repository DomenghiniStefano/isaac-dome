//! Catalog record → wiki `Target`: the one place that knows how the game's ids map onto the
//! dataset's page keys. None of it is obvious — a boss is identified by its portrait's file
//! name, a trinket lives on a different page kind from a collectible, and the wiki calls a
//! challenge's number what the catalog calls its id. Search keys its documents by this, and a
//! requirement links by it; a second copy would be wrong within a release.

use catalog::{AchievementId, Boss, Challenge, Character, Item, ItemKind};
use wiki::Target;

use crate::target_sprite::entity_key;

/// `items.xml` keeps collectibles and trinkets in one file; the dataset gives them two page
/// kinds.
pub(crate) fn item(i: &Item) -> Target {
    match i.kind {
        ItemKind::Trinket => Target::Trinket { id: i.id.0 },
        ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => Target::Item { id: i.id.0 },
    }
}

/// The catalog's id already tells the base and Tainted forms apart (B28): the page is the
/// id's, and the name never enters into it.
pub(crate) fn character(c: &Character) -> Target {
    Target::Character { id: c.id.0 }
}

/// The wiki names the field `number`, the catalog names it `id`. Same number.
pub(crate) fn challenge(c: &Challenge) -> Target {
    Target::Challenge { number: c.id.0 }
}

/// The entity key lives in the portrait's file name (`Portrait_20.0_Monstro.png`), exactly as
/// `target_sprite` reads it. A portrait that declares none names no page: `None`, never a
/// guessed variant.
pub(crate) fn boss(b: &Boss) -> Option<Target> {
    entity_key(&b.portrait.path).map(|(id, variant)| Target::Entity {
        id,
        variant,
        subtype: 0,
    })
}

pub(crate) fn achievement(id: AchievementId) -> Target {
    Target::Achievement { id: id.0 }
}
