//! `items.xml`: items, trinkets and familiars, with the sprite path already composed.

use serde::Serialize;

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, ItemId};
use crate::itempools::PoolMembership;
use crate::origin::{self, Origin};
use crate::sprite::SpriteRef;
use crate::text::Text;
use crate::xml::{elements, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Passive,
    Active,
    Familiar,
    Trinket,
}

impl ItemKind {
    fn from_tag(name: &str) -> Option<ItemKind> {
        match name {
            "passive" => Some(ItemKind::Passive),
            "active" => Some(ItemKind::Active),
            "familiar" => Some(ItemKind::Familiar),
            "trinket" => Some(ItemKind::Trinket),
            _ => None, // allowed: a tag name is an open string
        }
    }

    /// The subfolder under `gfxroot`: the game picks it from the kind, the XML doesn't say it.
    fn folder(self) -> &'static str {
        match self {
            ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => "collectibles",
            ItemKind::Trinket => "trinkets",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub name: Text,
    pub description: Text,
    pub quality: Option<i8>,
    pub tags: Vec<String>,
    pub sprite: SpriteRef,
    pub unlocked_by: Option<AchievementId>,
    pub pools: Vec<PoolMembership>,
    pub origin: Option<Origin>,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Item> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::Items,
            });
            return Vec::new();
        }
    };
    let gfxroot = els
        .iter()
        .find(|e| e.name == "items")
        .and_then(|e| e.attr("gfxroot"))
        .map(normalize_root)
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| "gfx/items".to_string());

    els.iter()
        .filter_map(|e| ItemKind::from_tag(&e.name).map(|k| (e, k)))
        .filter_map(|(e, kind)| item_from(e, kind, &gfxroot, diagnostics))
        .collect()
}

fn item_from(e: &Element, kind: ItemKind, gfxroot: &str, d: &mut Vec<Diagnostic>) -> Option<Item> {
    let skip = |id: Option<u32>, reason: SkipReason, d: &mut Vec<Diagnostic>| {
        d.push(Diagnostic::ElementSkipped {
            source: Source::Items,
            id,
            reason,
        });
        None
    };
    let Some(raw_id) = e.attr("id") else {
        return skip(None, SkipReason::MissingId, d);
    };
    let Ok(id) = raw_id.parse::<u32>() else {
        return skip(None, SkipReason::MalformedId, d);
    };
    let Some(gfx) = e.attr("gfx") else {
        return skip(Some(id), SkipReason::MissingSprite, d);
    };
    let Some(name) = e.attr("name") else {
        return skip(Some(id), SkipReason::MissingName, d);
    };

    Some(Item {
        id: ItemId(id),
        kind,
        name: Text::from_attr(name),
        description: Text::from_attr(e.attr("description").unwrap_or("")),
        // Quality and tags come from items_metadata.xml (Catalog::build fills them in later).
        quality: None,
        tags: Vec::new(),
        sprite: SpriteRef::whole(format!("{gfxroot}/{}/{gfx}", kind.folder())),
        unlocked_by: e
            .attr("achievement")
            .and_then(|a| a.parse().ok())
            .map(AchievementId),
        pools: Vec::new(),
        origin: origin::origin_of(kind, ItemId(id)),
    })
}

/// `resources/gfx/items/` or `gfx/items/` -> `gfx/items`: no archive root or trailing slash.
pub(crate) fn normalize_root(root: &str) -> String {
    let r = root.replace('\\', "/");
    let r = r.strip_prefix("resources/").unwrap_or(&r);
    r.trim_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\" version=\"1\">
\t<passive gfx=\"Collectibles_001_TheSadOnion.png\" id=\"1\" name=\"#THE_SAD_ONION_NAME\" description=\"#THE_SAD_ONION_DESCRIPTION\" quality=\"1\" tags=\"offensive summonable\" />
\t<active\t\tid=\"555\"\tachievement=\"583\" gfx=\"Collectibles_555_GoldenRazor.png\" name=\"#GOLDEN_RAZOR_NAME\" description=\"#GOLDEN_RAZOR_DESCRIPTION\" />
\t<familiar id=\"10\" gfx=\"Collectibles_010_Halo.png\" name=\"Halo\" description=\"Legacy\" />
\t<trinket id=\"1\" gfx=\"Trinket_001_SwallowedPenny.png\" name=\"#SWALLOWED_PENNY_NAME\" description=\"#SWALLOWED_PENNY_DESCRIPTION\" />
\t<passive gfx=\"NoId.png\" name=\"#X\" description=\"#X\" />
\t<passive id=\"abc\" gfx=\"BadId.png\" name=\"#X\" description=\"#X\" />
\t<passive id=\"7\" name=\"#NO_GFX\" description=\"#X\" />
\t<passives>not a passive</passives>
</items>";

    fn parsed() -> (Vec<Item>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let items = parse(ITEMS, &mut d);
        (items, d)
    }

    #[test]
    fn every_kind_lands_in_its_folder_with_root_from_the_file() {
        let (items, _) = parsed();
        let by = |id: u32, kind: ItemKind| {
            items
                .iter()
                .find(|i| i.id.0 == id && i.kind == kind)
                .unwrap()
        };
        assert_eq!(
            by(1, ItemKind::Passive).sprite.path,
            "gfx/items/collectibles/Collectibles_001_TheSadOnion.png"
        );
        assert_eq!(
            by(555, ItemKind::Active).sprite.path,
            "gfx/items/collectibles/Collectibles_555_GoldenRazor.png"
        );
        assert_eq!(
            by(10, ItemKind::Familiar).sprite.path,
            "gfx/items/collectibles/Collectibles_010_Halo.png"
        );
        assert_eq!(
            by(1, ItemKind::Trinket).sprite.path,
            "gfx/items/trinkets/Trinket_001_SwallowedPenny.png"
        );
    }

    #[test]
    fn tabs_and_achievement_link_are_read() {
        let (items, _) = parsed();
        let razor = items.iter().find(|i| i.id.0 == 555).unwrap();
        assert_eq!(razor.unlocked_by, Some(AchievementId(583)));
        assert_eq!(razor.quality, None);
        let onion = items
            .iter()
            .find(|i| i.id.0 == 1 && i.kind == ItemKind::Passive)
            .unwrap();
        // quality="1" tags="offensive summonable" are in the fixture but items.xml
        // doesn't have them in the real file: the parser ignores them, quality and tags
        // come from items_metadata.xml.
        assert_eq!(onion.quality, None);
        assert!(onion.tags.is_empty());
        assert_eq!(onion.name, Text::from_attr("#THE_SAD_ONION_NAME"));
    }

    #[test]
    fn literal_names_from_the_base_game_are_kept_as_literals() {
        let (items, _) = parsed();
        let halo = items.iter().find(|i| i.id.0 == 10).unwrap();
        assert_eq!(
            halo.name,
            Text::Literal {
                text: "Halo".to_string()
            }
        );
    }

    #[test]
    fn malformed_elements_are_skipped_with_a_reason_and_the_rest_is_read() {
        let (items, d) = parsed();
        assert_eq!(items.len(), 4);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Items,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Items,
            id: None,
            reason: SkipReason::MalformedId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Items,
            id: Some(7),
            reason: SkipReason::MissingSprite
        }));
    }

    #[test]
    fn the_skips_come_in_file_order_with_the_sprite_checked_before_the_name() {
        let (_, d) = parsed();
        let skipped = |id, reason| Diagnostic::ElementSkipped {
            source: Source::Items,
            id,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(None, SkipReason::MissingId),
                skipped(None, SkipReason::MalformedId),
                skipped(Some(7), SkipReason::MissingSprite),
            ]
        );
        let mut d = Vec::new();
        let none = parse(
            b"<items><trinket id=\"3\" /><trinket id=\"4\" gfx=\"t.png\" /></items>",
            &mut d,
        );
        assert!(none.is_empty());
        assert_eq!(
            d,
            vec![
                skipped(Some(3), SkipReason::MissingSprite),
                skipped(Some(4), SkipReason::MissingName),
            ]
        );
    }

    #[test]
    fn without_a_gfxroot_or_a_description_the_game_defaults_apply() {
        let mut d = Vec::new();
        let items = parse(
            b"<items><trinket id=\"2\" gfx=\"t.png\" name=\"T\" achievement=\"x\" /></items>",
            &mut d,
        );
        assert_eq!(items[0].sprite.path, "gfx/items/trinkets/t.png");
        assert_eq!(items[0].description, Text::from_attr(""));
        assert_eq!(items[0].unlocked_by, None, "a malformed link is no link");
        assert!(d.is_empty());
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<items><passive", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Items
            }]
        );
    }

    #[test]
    fn normalize_root_strips_the_archive_prefix_backslashes_and_trailing_slashes() {
        assert_eq!(normalize_root("resources/gfx/items/"), "gfx/items");
        assert_eq!(normalize_root("gfx/items"), "gfx/items");
        assert_eq!(normalize_root("gfx\\items\\"), "gfx/items");
        assert_eq!(normalize_root(""), "");
    }

    #[test]
    fn a_gfxroot_that_normalizes_to_empty_falls_back_to_the_default_like_a_missing_one() {
        let mut d = Vec::new();
        let empty = parse(
            b"<items gfxroot=\"\"><passive id=\"1\" gfx=\"a.png\" name=\"#X\" description=\"#X\" /></items>",
            &mut d,
        );
        assert_eq!(
            empty[0].sprite.path, "gfx/items/collectibles/a.png",
            "empty gfxroot: no leading slash, default as if it were missing"
        );

        let mut d = Vec::new();
        let resources_only = parse(
            b"<items gfxroot=\"resources/\"><passive id=\"1\" gfx=\"a.png\" name=\"#X\" description=\"#X\" /></items>",
            &mut d,
        );
        assert_eq!(
            resources_only[0].sprite.path, "gfx/items/collectibles/a.png",
            "gfxroot that normalizes to empty: same default, no double slash"
        );
    }
}
