//! The catalog as the UI sees it. As provisional as the verification screen, but
//! already in the right shape: resolved names, no keys, no paths.

use catalog::{Catalog, Diagnostic, ItemKind, Language};
use serde::{Deserialize, Serialize};

use crate::resources::data_url;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct KindCounts {
    pub passives: usize,
    pub actives: usize,
    pub familiars: usize,
    pub trinkets: usize,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CatalogView {
    pub counts: KindCounts,
    pub total: usize,
    /// How many distinct keys (names, descriptions, character names) the stringtable
    /// doesn't know: the catalog's `Diagnostic::UnresolvedKey`, counted one per key.
    pub unresolved_names: usize,
    pub languages: Vec<String>,
}

/// A fieldless enum: on the wire it's a bare camelCase string (`"passive"`), like
/// `OriginView`. The tag exists to distinguish variants that carry different data, and
/// here there are none: `{"kind":"passive"}` would cost a key on every row and say
/// nothing more. The day a variant gains a field, the enum becomes tagged and the
/// TypeScript side follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ItemKindView {
    Passive,
    Active,
    Familiar,
    Trinket,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: u32,
    pub kind: ItemKindView,
    pub name: String,
    pub data_url: Option<String>,
}

pub fn catalog_view(c: &Catalog) -> CatalogView {
    let n = |k: ItemKind| c.items().filter(|i| i.kind == k).count();
    CatalogView {
        counts: KindCounts {
            passives: n(ItemKind::Passive),
            actives: n(ItemKind::Active),
            familiars: n(ItemKind::Familiar),
            trinkets: n(ItemKind::Trinket),
        },
        total: c.items().count(),
        unresolved_names: c
            .diagnostics()
            .iter()
            .filter(|d| matches!(d, Diagnostic::UnresolvedKey { .. }))
            .count(),
        languages: c.languages().iter().map(|l| language_label(*l)).collect(),
    }
}

/// The first `limit` items with their English name and, if the sprite can be read, its image.
pub fn item_views(
    c: &Catalog,
    mut sprite: impl FnMut(&str) -> Option<Vec<u8>>,
    limit: usize,
) -> Vec<ItemView> {
    c.items()
        .take(limit)
        .map(|i| ItemView {
            id: i.id.0,
            kind: kind_view(i.kind),
            name: c.text(&i.name, Language::English).to_string(),
            data_url: sprite(&i.sprite.path).map(|png| data_url(&png)),
        })
        .collect()
}

/// `ItemKind` doesn't cross the IPC boundary: this is its view, shared with `graph::target_of`.
pub(crate) fn kind_view(k: ItemKind) -> ItemKindView {
    match k {
        ItemKind::Passive => ItemKindView::Passive,
        ItemKind::Active => ItemKindView::Active,
        ItemKind::Familiar => ItemKindView::Familiar,
        ItemKind::Trinket => ItemKindView::Trinket,
    }
}

/// The inverse of `kind_view`: a target arriving from the frontend maps back to the
/// catalog's type. Exhaustive: an extra variant on either side breaks the build.
pub(crate) fn item_kind(v: ItemKindView) -> ItemKind {
    match v {
        ItemKindView::Passive => ItemKind::Passive,
        ItemKindView::Active => ItemKind::Active,
        ItemKindView::Familiar => ItemKind::Familiar,
        ItemKindView::Trinket => ItemKind::Trinket,
    }
}

/// The collectible numbered `id`, whichever of the three collectible kinds it is.
///
/// The wiki's `Item { id }` and a run's `Adding collectible N` both name one by number alone.
/// Passives, actives and familiars share one id space, so at most one kind matches; a trinket
/// never does, because it can carry the same number as a collectible and is not the thing
/// either of them means. One helper where there were three copies (card #82, S4).
pub(crate) fn collectible(c: &Catalog, id: u32) -> Option<&catalog::Item> {
    [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar]
        .into_iter()
        .find_map(|kind| c.item(kind, catalog::ItemId(id)))
}

fn language_label(l: Language) -> String {
    match l {
        Language::English => "english",
        Language::Japanese => "japanese",
        Language::Korean => "korean",
        Language::ChineseSimplified => "chineseSimplified",
        Language::Russian => "russian",
        Language::German => "german",
        Language::Spanish => "spanish",
        Language::French => "french",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `kind_view` and `item_kind` are each other's inverse. The two maps are hand
    /// written and are the joint that the `(kind, id)` key of `TargetKey` rests on: a
    /// swapped row would silently save a goal against the wrong item. A pure test,
    /// always run; it lives here and not in `tests/` because the two functions aren't
    /// public.
    #[test]
    fn the_view_of_an_item_kind_maps_back_to_it() {
        for k in [
            ItemKind::Passive,
            ItemKind::Active,
            ItemKind::Familiar,
            ItemKind::Trinket,
        ] {
            assert_eq!(item_kind(kind_view(k)), k, "{k:?}");
        }
        // And the other way around: no view ends up on the wrong type.
        for v in [
            ItemKindView::Passive,
            ItemKindView::Active,
            ItemKindView::Familiar,
            ItemKindView::Trinket,
        ] {
            assert_eq!(kind_view(item_kind(v)), v, "{v:?}");
        }
    }
}
