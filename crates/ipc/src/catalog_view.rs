//! The catalog as the UI sees it. As provisional as the verification screen, but
//! already in the right shape: resolved names, no keys, no paths.

use catalog::{Catalog, Diagnostic, ItemKind, Language};
use serde::Serialize;

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

/// The catalog's own kind, under the name the boundary has always used: one definition, so
/// the wire, the icon URL and the XML cannot spell a kind three ways.
pub use catalog::ItemKind as ItemKindView;

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
            kind: i.kind,
            name: c.text(&i.name, Language::English).to_string(),
            data_url: sprite(&i.sprite.path).map(|png| data_url(&png)),
        })
        .collect()
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

    /// A kind's one word is the one `serde` writes: the icon URL (`item/passive/92`), the XML
    /// element and the payload say the same thing, and `from_name` reads it back. The joint
    /// the `(kind, id)` key of `TargetKey` rests on: a swapped row would silently save a goal
    /// against the wrong item.
    #[test]
    fn a_kinds_name_is_its_wire_string_and_reads_back() {
        for k in ItemKind::ALL {
            let wire = serde_json::to_value(k).expect("serialises");
            assert_eq!(wire, serde_json::json!(k.name()), "{k:?}");
            assert_eq!(ItemKind::from_name(k.name()), Some(k), "{k:?}");
            let back: ItemKindView = serde_json::from_value(wire).expect("deserialises");
            assert_eq!(back, k, "{k:?}");
        }
        assert_eq!(ItemKind::from_name("passives"), None);
    }
}
