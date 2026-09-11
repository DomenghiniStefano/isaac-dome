//! The Collection: the save's item collection (section 4) joined with the catalog's
//! collectibles. Section 4 holds one slot per collectible id — the catalog's ids are a subset
//! of its slots, and the gaps are exactly the unused ids — so trinkets, which have none, aren't
//! listed. What a set byte means in play isn't measured: the view calls it "in the collection",
//! the section's own name.

use catalog::{AchievementId, Catalog, Item, ItemKind, Language};
use serde::Serialize;

use crate::catalog_view::kind_view;
use crate::graph::origin_view;
use crate::{IconRef, ItemKindView, OriginView};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub items: Vec<CollectionItem>,
    /// The pools any listed item belongs to, each once, in the catalog's order.
    pub pools: Vec<String>,
    pub totals: CollectionTotals,
    pub diagnostics: Vec<CollectionDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionTotals {
    /// Section 4's length; 0 when it wasn't read.
    pub slots: u32,
    pub items: u32,
    pub in_collection: u32,
}

#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LockView {
    /// Nothing unlocks it: it is in the game from the start.
    Free,
    Unlocked {
        achievement: u32,
        text: Option<String>,
    },
    /// Its achievement isn't done: the item can't appear in a run yet.
    Locked {
        achievement: u32,
        text: Option<String>,
    },
    /// Section 1 wasn't read: whether the achievement is done isn't known.
    Unknown {
        achievement: u32,
        text: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    let mut rows = Vec::with_capacity(listed.len());
    let mut beyond = 0u32;
    for i in listed {
        let kind = kind_view(i.kind);
        let in_collection = items.and_then(|f| f.get(i.id.0 as usize).copied());
        if items.is_some() && in_collection.is_none() {
            beyond += 1;
        }
        let mut pools: Vec<String> = Vec::new();
        for m in &i.pools {
            if !pools.contains(&m.pool) {
                pools.push(m.pool.clone());
            }
        }
        rows.push(CollectionItem {
            id: i.id.0,
            kind,
            name: c.text(&i.name, Language::English).to_string(),
            icon_url: icon(&IconRef::Item { kind, id: i.id.0 }),
            quality: i.quality,
            pools,
            origin: i.origin.map(origin_view),
            in_collection,
            lock: lock_of(c, i.unlocked_by, achievements),
        });
    }

    let pools = c
        .pools()
        .iter()
        .map(|p| p.name.clone())
        .filter(|name| rows.iter().any(|r| r.pools.contains(name)))
        .collect();

    let mut diagnostics = Vec::new();
    if items.is_none() {
        diagnostics.push(CollectionDiagnostic::NoCollectionSection);
    }
    if achievements.is_none() {
        diagnostics.push(CollectionDiagnostic::NoAchievementSection);
    }
    if beyond > 0 {
        diagnostics.push(CollectionDiagnostic::ItemsBeyondSlots { count: beyond });
    }

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
        items: rows,
        pools,
        diagnostics,
    }
}

/// A slot past section 1's end reads as not done: the save has no record of it.
fn lock_of(
    c: &Catalog,
    unlocked_by: Option<AchievementId>,
    achievements: Option<&[bool]>,
) -> LockView {
    let Some(a) = unlocked_by else {
        return LockView::Free;
    };
    let achievement = a.0;
    let text = c.achievement(a).map(|x| x.text.clone());
    match achievements.map(|f| f.get(achievement as usize).copied().unwrap_or(false)) {
        None => LockView::Unknown { achievement, text },
        Some(true) => LockView::Unlocked { achievement, text },
        Some(false) => LockView::Locked { achievement, text },
    }
}
