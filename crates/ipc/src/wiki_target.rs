//! Catalog record → wiki `Target`: the one place that knows how the game's ids map onto the
//! dataset's page keys. None of it is obvious — a boss is identified by its portrait's file
//! name, a trinket lives on a different page kind from a collectible, and the wiki calls a
//! challenge's number what the catalog calls its id. Search keys its documents by this, and a
//! requirement links by it; a second copy would be wrong within a release.

use catalog::{AchievementId, Boss, Catalog, Challenge, Character, Item, ItemKind};
use wiki::{Dataset, Target};

use crate::target_sprite::boss_keys;

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

/// The entity key is whatever `boss_keys` settles for the row — the page's own key where a
/// page names the row, the portrait's file name otherwise — so the page a boss links to and
/// the picture it is drawn with are the same decision, taken once.
///
/// It takes the catalog because the rules are about the roster, not the row: a key two rows
/// declare belongs to neither, and that cannot be seen from one of them. A row left without a
/// key names no page: `None`, never a guessed variant.
pub(crate) fn boss(c: &Catalog, b: &Boss) -> Option<Target> {
    boss_keys(c)
        .get(b.name.as_str())
        .map(|(id, variant)| Target::Entity {
            id: *id,
            variant: *variant,
            subtype: 0,
        })
}

pub(crate) fn achievement(id: AchievementId) -> Target {
    Target::Achievement { id: id.0 }
}

/// A page, only when the dataset really has one. `Some(target)` is a link the screen can
/// follow; `None` is a name it draws without one — never a link that leads nowhere. One rule
/// for every screen that links a page (card #82, S4): the graph, the challenges and the
/// collection each had their own copy.
pub(crate) fn page_of(dataset: Option<&Dataset>, target: Target) -> Option<Target> {
    dataset?.entry(&target).is_some().then_some(target)
}
