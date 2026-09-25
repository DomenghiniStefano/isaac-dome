//! The Collection: the save's item collection (section 4) joined with the catalog's
//! collectibles. Section 4 holds one slot per collectible id — the catalog's ids are a subset
//! of its slots, and the gaps are exactly the unused ids — so trinkets, which have none, aren't
//! listed. What a set byte means in play isn't measured: the view calls it "in the collection",
//! the section's own name.

use catalog::{AchievementId, Catalog, Item, ItemKind, Language};
use serde::Serialize;
use wiki::{Dataset, Target};

use crate::catalog_view::kind_view;
use crate::flags::{recorded, recorded_done};
use crate::graph::origin_view;
use crate::wiki_target;
use crate::{IconRef, ItemKindView, OriginView};

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub items: Vec<CollectionItem>,
    /// The pools any listed item belongs to, each once, in the catalog's order.
    pub pools: Vec<String>,
    pub totals: CollectionTotals,
    pub diagnostics: Vec<CollectionDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CollectionTotals {
    /// Section 4's length; 0 when it wasn't read.
    pub slots: u32,
    pub items: u32,
    pub in_collection: u32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItem {
    pub id: u32,
    pub kind: ItemKindView,
    pub name: String,
    pub icon_url: Option<String>,
    pub quality: Option<i8>,
    pub pools: Vec<String>,
    pub origin: Option<OriginView>,
    /// `None` when section 4 wasn't read, or the save has no slot for this id: unread is
    /// never "not in the collection".
    pub in_collection: Option<bool>,
    pub lock: LockView,
}

/// What stands between the item and a run. Tagged: three of the four variants carry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LockView {
    /// Nothing unlocks it: it is in the game from the start, so there is no page to open.
    Free,
    Unlocked {
        achievement: u32,
        text: Option<String>,
        /// The achievement's wiki page. `None` means the dataset has no page for it: the
        /// name shows and does not link. Never "no achievement".
        page: Option<Target>,
    },
    /// Its achievement isn't done: the item can't appear in a run yet.
    Locked {
        achievement: u32,
        text: Option<String>,
        page: Option<Target>,
    },
    /// Section 1 wasn't read: whether the achievement is done isn't known.
    Unknown {
        achievement: u32,
        text: Option<String>,
        page: Option<Target>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CollectionDiagnostic {
    NoCatalog,
    NoCollectionSection,
    NoAchievementSection,
    /// Collectibles the save has no slot for: a catalog newer than the save.
    ItemsBeyondSlots {
        count: u32,
    },
}

pub fn collection_view(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    items: Option<&[bool]>,
    achievements: Option<&[bool]>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> CollectionView {
    let slots = items.map_or(0, |f| f.len() as u32);
    let Some(c) = catalog else {
        // No names and no ids to list: the section still says how much of it is set. Slot 0
        // is no item.
        let set = items.map_or(0, |f| f.iter().skip(1).filter(|b| **b).count() as u32);
        return CollectionView {
            items: Vec::new(),
            pools: Vec::new(),
            totals: CollectionTotals {
                slots,
                items: 0,
                in_collection: set,
            },
            diagnostics: vec![CollectionDiagnostic::NoCatalog],
        };
    };

    let mut listed: Vec<&Item> = c.items().filter(|i| i.kind != ItemKind::Trinket).collect();
    listed.sort_by_key(|i| i.id.0);
    let rows: Vec<CollectionItem> = listed
        .into_iter()
        .map(|i| item_row(c, dataset, i, items, achievements, &mut icon))
        .collect();
    let in_collection = rows
        .iter()
        .filter(|r| r.in_collection == Some(true))
        .count() as u32;
    CollectionView {
        totals: CollectionTotals {
            slots,
            items: rows.len() as u32,
            in_collection,
        },
        pools: pools_in_use(c, &rows),
        diagnostics: diagnostics(&rows, items, achievements),
        items: rows,
    }
}

fn item_row(
    c: &Catalog,
    dataset: Option<&Dataset>,
    i: &Item,
    items: Option<&[bool]>,
    achievements: Option<&[bool]>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> CollectionItem {
    let kind = kind_view(i.kind);
    CollectionItem {
        id: i.id.0,
        kind,
        name: c.text(&i.name, Language::English).to_string(),
        icon_url: icon(&IconRef::Item { kind, id: i.id.0 }),
        quality: i.quality,
        pools: distinct_pools(i),
        origin: i.origin.map(origin_view),
        in_collection: recorded(items, i.id.0),
        lock: lock_of(c, dataset, i.unlocked_by, achievements),
    }
}

/// The catalog's pools, in its order, that at least one listed item belongs to.
fn pools_in_use(c: &Catalog, rows: &[CollectionItem]) -> Vec<String> {
    c.pools()
        .iter()
        .map(|p| p.name.clone())
        .filter(|name| rows.iter().any(|r| r.pools.contains(name)))
        .collect()
}

/// The sections that did not read, then the items the save has no slot for.
fn diagnostics(
    rows: &[CollectionItem],
    items: Option<&[bool]>,
    achievements: Option<&[bool]>,
) -> Vec<CollectionDiagnostic> {
    // Only a section that read can end before an item: with no section, every row is unknown
    // for that reason and `NoCollectionSection` already says so.
    let beyond = match items {
        Some(_) => rows.iter().filter(|r| r.in_collection.is_none()).count() as u32,
        None => 0,
    };
    [
        items
            .is_none()
            .then_some(CollectionDiagnostic::NoCollectionSection),
        achievements
            .is_none()
            .then_some(CollectionDiagnostic::NoAchievementSection),
        (beyond > 0).then_some(CollectionDiagnostic::ItemsBeyondSlots { count: beyond }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// A slot past section 1's end reads as not done: the save has no record of it.
fn lock_of(
    c: &Catalog,
    dataset: Option<&Dataset>,
    unlocked_by: Option<AchievementId>,
    achievements: Option<&[bool]>,
) -> LockView {
    let Some(a) = unlocked_by else {
        return LockView::Free;
    };
    let achievement = a.0;
    let text = c.achievement(a).map(|x| x.text.clone());
    // A page only when the dataset really has one: never a link that leads nowhere.
    let page = wiki_target::page_of(dataset, wiki_target::achievement(a));
    match achievements.map(|f| recorded_done(f, achievement)) {
        None => LockView::Unknown {
            achievement,
            text,
            page,
        },
        Some(true) => LockView::Unlocked {
            achievement,
            text,
            page,
        },
        Some(false) => LockView::Locked {
            achievement,
            text,
            page,
        },
    }
}

/// The pools an item is found in, once each, in the order the item lists them: the catalog
/// can list one pool more than once for an item, and the screen filters by name.
fn distinct_pools(i: &Item) -> Vec<String> {
    i.pools
        .iter()
        .enumerate()
        .filter(|(n, m)| !i.pools[..*n].iter().any(|seen| seen.pool == m.pool))
        .map(|(_, m)| m.pool.clone())
        .collect()
}
