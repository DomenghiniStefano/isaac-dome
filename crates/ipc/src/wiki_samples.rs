//! The landing's one picture per category: a deliberate, checked choice
//! (`category_sample`), the `CategorySample` row it becomes on the wire, and the pass over
//! `WIKI_PAGE_CATEGORIES` that builds the whole list for `wiki_index`. Split out of `wiki.rs`
//! for its own sake: this is the one slice of that module that is about the landing's tiles
//! and not about a page's own identity.

use catalog::Catalog;
use serde::Serialize;
use wiki::{Dataset, Target};

use crate::icon::IconRef;
use crate::target_sprite::BossKeys;
use crate::wiki::{WikiPageCategory, WIKI_PAGE_CATEGORIES};

/// A landing tile's own picture, by its category: not a random find, a checked one — see
/// `category_sample`.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CategorySample {
    pub category: WikiPageCategory,
    pub icon_url: Option<String>,
}

/// A deliberate, checked choice of one game thing per landing tile — never "whatever page
/// happens to come first", which would draw a different tile picture depending on the
/// dataset's own iteration order. `None` for a kind the game draws no picture for at all
/// (transformations: B50; stages: no title-art reader exists yet; versions: a patch note has
/// no picture) — the tile falls back to its plain icon, same as without the game.
///
/// The two "raw" choices (`CardsAndRunes`, `Pickups`) are `IconRef::Entity`, not
/// `IconRef::Page`: individual card faces have no data-file mapping at all (measured — see
/// `target_sprite`'s `Target::Article` arm), so a card page can never draw a picture through
/// this pipeline. The tile still can, honestly: `5.300.1` is a real `entities2.xml` row, a
/// tarot card's own face-down back — "a card", not "the right card" — and `5.10.1` is a
/// heart pickup, the same distinction the task drew.
pub fn category_sample(category: WikiPageCategory) -> Option<IconRef> {
    let entity = |id, variant, subtype| {
        Some(IconRef::Page {
            target: Target::Entity {
                id,
                variant,
                subtype,
            },
        })
    };
    match category {
        // Sad Onion: item 105, in the game since Rebirth, always drawn.
        WikiPageCategory::Items => Some(IconRef::Page {
            target: Target::Item { id: 105 },
        }),
        // Swallowed Penny: the first trinket a new run can find.
        WikiPageCategory::Trinkets => Some(IconRef::Page {
            target: Target::Trinket { id: 1 },
        }),
        WikiPageCategory::Achievements => Some(IconRef::Page {
            target: Target::Achievement { id: 1 },
        }),
        // Isaac: id 0, the character the game itself opens on.
        WikiPageCategory::Characters => Some(IconRef::Page {
            target: Target::Character { id: 0 },
        }),
        WikiPageCategory::Challenges => Some(IconRef::Page {
            target: Target::Challenge { number: 1 },
        }),
        // Monstro: the first boss's entity key.
        WikiPageCategory::Bosses => entity(20, 0, 0),
        // Gaper: the game's own first common enemy.
        WikiPageCategory::Monsters => entity(10, 0, 0),
        // A tarot card's shared back — see this function's own doc comment.
        WikiPageCategory::CardsAndRunes => Some(IconRef::Entity {
            id: 5,
            variant: 300,
            subtype: 1,
        }),
        // A heart pickup.
        WikiPageCategory::Pickups => Some(IconRef::Entity {
            id: 5,
            variant: 10,
            subtype: 1,
        }),
        WikiPageCategory::Transformations
        | WikiPageCategory::Stages
        | WikiPageCategory::Versions => None,
    }
}

/// `category_sample`'s picture, only when the catalog really has one — never a URL to a
/// picture that would 404. `icon_source` is the one function that already knows how to check
/// every kind of `IconRef` this can produce, `Page` and `Entity` alike.
fn category_icon_url(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    ds: &Dataset,
    category: WikiPageCategory,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<String> {
    let c = catalog?;
    let r = category_sample(category)?;
    crate::icon::icon_source(c, bosses, Some(ds), &r)?;
    icon(&r)
}

/// One representative picture per landing tile, in `WIKI_PAGE_CATEGORIES` order — what
/// `wiki_index` puts on `WikiIndex::samples`.
pub(crate) fn samples(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    ds: &Dataset,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> Vec<CategorySample> {
    WIKI_PAGE_CATEGORIES
        .into_iter()
        .map(|category| CategorySample {
            category,
            icon_url: category_icon_url(catalog, bosses, ds, category, &mut icon),
        })
        .collect()
}
