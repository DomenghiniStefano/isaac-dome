//! Collectible pool membership (design decision 4, `2026-09-26-wiki-complete-design.md`).
//!
//! The wiki's own `pool` parameter never carried a weighted pool: on the 45 items and 7
//! trinkets that write it, it names a guaranteed *source* (a boss, a machine, another item),
//! and it is empty on the rest (674 of 719 items, 181 of 188 trinkets) — `wiki::Infobox::Item`'s
//! doc comment has the measurement, and that text is kept, under its own name,
//! `Infobox::Item::obtained_from` / `Infobox::Trinket::obtained_from`, drawn as its own row.
//! The game's `itempools.xml` is the source that actually knows every weighted-pool
//! membership, and `catalog` already reads it (`Catalog::collectible`, `Item::pools`).
//! Neither `catalog` nor `wiki` can join the two on its own, so the game's half happens here,
//! at the page view: the one place that sees both the installed game and the dataset. The
//! two are two answers to two different questions, shown as two rows, neither a stand-in for
//! the other.
//!
//! Trinkets are not part of this join at all: `itempools.xml`'s pools are collectibles only,
//! by the game's own construction (see [`item_pools`]'s doc comment), so there is nothing to
//! join for a trinket page — its infobox draws no game-sourced pools row, only its own
//! `obtained_from`, when the wiki wrote one.

use catalog::{Catalog, ItemId, PoolMembership};
use serde::Serialize;
use wiki::{Dataset, Target};

use crate::wiki_target::page_of;

/// One pool the installed game lists a collectible in.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PoolMembershipView {
    /// The pool's own name: the wiki article's title when the dataset covers it (every one
    /// of the 31 real pools does, per `POOL_ARTICLES`), the game's own id otherwise — never
    /// blank, since a player still needs to know which pool this is.
    pub label: String,
    /// The wiki's own page about this pool, when the dataset has one to link.
    pub target: Option<Target>,
    /// The pool's odds for this collectible relative to the rest of the pool: real data
    /// `catalog::PoolMembership` already keeps. `DecreaseBy` and `RemoveOn` (how the weight
    /// falls after a pull) never reach the catalog, so neither reaches here.
    pub weight: f32,
}

/// The game's pool id (`itempools.xml`'s `Pool Name=`) to the wiki article that covers it:
/// the one table for the one fact, both sides measured together on 2026-09-27 — a text
/// search of the installed `itempools.xml` for every `Pool Name=`, against `articles.json`'s
/// titles. All 31 pools the game has today resolve; an id this table doesn't know (a pool a
/// future patch adds) still shows, under the game's own name, with no link.
const POOL_ARTICLES: &[(&str, &str)] = &[
    ("treasure", "Treasure Room (Item Pool)"),
    ("shop", "Shop (Item Pool)"),
    ("boss", "Boss (Item Pool)"),
    ("devil", "Devil Room (Item Pool)"),
    ("angel", "Angel Room (Item Pool)"),
    ("secret", "Secret Room (Item Pool)"),
    ("library", "Library (Item Pool)"),
    ("shellGame", "Shell Game (Item Pool)"),
    ("goldenChest", "Golden Chest (Item Pool)"),
    ("redChest", "Red Chest (Item Pool)"),
    ("beggar", "Beggar (Item Pool)"),
    ("demonBeggar", "Devil Beggar (Item Pool)"),
    ("curse", "Curse Room (Item Pool)"),
    ("keyMaster", "Key Master (Item Pool)"),
    ("batteryBum", "Battery Bum (Item Pool)"),
    ("momsChest", "Mom's Chest (Item Pool)"),
    ("greedTreasure", "Treasure Room (Greed Mode Item Pool)"),
    ("greedBoss", "Boss (Greed Mode Item Pool)"),
    ("greedShop", "Shop (Greed Mode Item Pool)"),
    ("greedCurse", "Curse Room (Greed Mode Item Pool)"),
    ("greedDevil", "Devil Room (Greed Mode Item Pool)"),
    ("greedAngel", "Angel Room (Greed Mode Item Pool)"),
    ("greedSecret", "Secret Room (Greed Mode Item Pool)"),
    ("craneGame", "Crane Game (Item Pool)"),
    ("ultraSecret", "Ultra Secret Room (Item Pool)"),
    ("bombBum", "Bomb Bum (Item Pool)"),
    ("planetarium", "Planetarium (Item Pool)"),
    ("oldChest", "Old Chest (Item Pool)"),
    ("babyShop", "Baby Shop (Item Pool)"),
    ("woodenChest", "Wooden Chest (Item Pool)"),
    ("rottenBeggar", "Rotten Beggar (Item Pool)"),
];

/// The pools `target` belongs to, the installed game's own truth. `None` when the catalog
/// doesn't have this collectible at all (no game, or an id the game doesn't use), or when
/// `target` isn't a collectible.
///
/// **Trinkets included, and always `None`**: `itempools.xml`'s `<Pool>` elements are, by
/// construction, collectibles only — `catalog::Catalog`'s own `attach_pools` never writes a
/// membership onto a trinket (`collectible_mut` looks it up only among the three collectible
/// kinds), verified against the real file (`pools_are_31_every_entry_is_a_known_item_and_24_items_are_in_none`).
/// So a trinket's `pools` would be `Some(&[])` on every one of the 188 trinkets, always, game
/// or no game — indistinguishable from "in no pool" when the true answer is "the file does
/// not model this for a trinket at all". `None` says the honest thing; a trinket page draws
/// no pools row at all, the same as before the wiki's own field was removed for it.
///
/// `Some(&[])` on an item is a real answer, not a missing one: 24 collectibles are in no pool
/// by construction, the same test above.
pub fn item_pools(
    c: &Catalog,
    ds: Option<&Dataset>,
    target: &Target,
) -> Option<Vec<PoolMembershipView>> {
    let memberships: &[PoolMembership] = match target {
        Target::Item { id } => &c.collectible(ItemId(*id))?.pools,
        Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. }
        | Target::Article { .. } => return None,
    };
    Some(memberships.iter().map(|m| pool_view(ds, m)).collect())
}

/// One membership, resolved against `POOL_ARTICLES` and, through it, the dataset: a pool the
/// table knows but the dataset doesn't have (should not happen against a real snapshot) shows
/// the article's title as plain text rather than a link that would 404.
fn pool_view(ds: Option<&Dataset>, m: &PoolMembership) -> PoolMembershipView {
    let article_title = POOL_ARTICLES
        .iter()
        .find(|(id, _)| *id == m.pool)
        .map(|(_, title)| *title);
    let target = article_title.and_then(|title| {
        page_of(
            ds,
            Target::Article {
                title: title.to_string(),
            },
        )
    });
    let label = article_title.unwrap_or(&m.pool).to_string();
    PoolMembershipView {
        label,
        target,
        weight: m.weight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiki::for_tests::{empty_dataset, entry};
    use wiki::Infobox;

    // Item 1 in `treasure` and `aFuturePool` (a name this table doesn't know), item 2 in no
    // pool at all. Trinket 1 shares item 1's id space and sits in `devil` too, to prove that
    // still resolves to nothing for a trinket target (real `itempools.xml` pools are
    // collectibles only).
    const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" /><passive id=\"2\" gfx=\"b.png\" name=\"B\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" /></items>";
    const POOLS: &[u8] = b"<ItemPools><Pool Name=\"treasure\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"aFuturePool\"><Item Id=\"1\" Weight=\"0.5\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"devil\"><Item Id=\"1\" Weight=\"2\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool></ItemPools>";

    fn catalog() -> Catalog {
        Catalog::build(|p| match p {
            "items.xml" => Some(ITEMS.to_vec()),
            "itempools.xml" => Some(POOLS.to_vec()),
            _ => None,
        })
    }

    #[test]
    fn a_known_pool_links_to_its_article_when_the_dataset_has_it() {
        let c = catalog();
        let mut ds = empty_dataset();
        ds.articles.insert(
            "Treasure Room (Item Pool)".into(),
            entry(
                "Treasure Room (Item Pool)",
                Infobox::Article { category: None },
            ),
        );
        let got = item_pools(&c, Some(&ds), &Target::Item { id: 1 }).unwrap();
        let treasure = got
            .iter()
            .find(|p| p.label == "Treasure Room (Item Pool)")
            .expect("the treasure membership is listed");
        assert_eq!(
            treasure.target,
            Some(Target::Article {
                title: "Treasure Room (Item Pool)".into()
            })
        );
        assert_eq!(treasure.weight, 1.0);
    }

    #[test]
    fn the_article_title_still_labels_the_pool_without_the_dataset() {
        let c = catalog();
        let got = item_pools(&c, None, &Target::Item { id: 1 }).unwrap();
        let devil = got
            .iter()
            .find(|p| p.label == "Devil Room (Item Pool)")
            .expect("the devil membership is listed");
        assert_eq!(devil.target, None, "no dataset was given to link against");
        assert_eq!(devil.weight, 2.0);
    }

    #[test]
    fn a_trinket_is_never_in_a_pool_even_when_its_id_collides_with_an_items() {
        // Trinket 1 shares its id with the item that IS in `devil`, `treasure` and
        // `aFuturePool` above; if the join keyed only on the numeric id it would find those
        // rows. `catalog::Catalog::attach_pools` never writes to a trinket, so this is `None`
        // regardless — the concept doesn't exist for a trinket, not "found in none".
        let c = catalog();
        assert_eq!(item_pools(&c, None, &Target::Trinket { id: 1 }), None);
    }

    #[test]
    fn a_pool_id_the_table_does_not_know_falls_back_to_its_own_name() {
        let c = catalog();
        let got = item_pools(&c, None, &Target::Item { id: 1 }).unwrap();
        let unknown = got
            .iter()
            .find(|p| p.label == "aFuturePool")
            .expect("the unmapped pool is still listed, under its own name");
        assert_eq!(unknown.target, None);
        assert_eq!(unknown.weight, 0.5);
    }

    #[test]
    fn an_item_in_no_pool_is_some_of_an_empty_list_not_none() {
        let c = catalog();
        assert_eq!(item_pools(&c, None, &Target::Item { id: 2 }), Some(vec![]));
    }

    #[test]
    fn an_id_the_catalog_does_not_have_is_none() {
        let c = catalog();
        assert_eq!(item_pools(&c, None, &Target::Item { id: 999 }), None);
    }

    #[test]
    fn a_target_with_no_pool_concept_is_none() {
        let c = catalog();
        assert_eq!(
            item_pools(&c, None, &Target::Character { id: 0 }),
            None,
            "pools are a collectible's concept, not a character's"
        );
    }
}
