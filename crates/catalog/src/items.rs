//! `items.xml`: items, trinkets and familiars, with the sprite path already composed.

use serde::{Deserialize, Serialize};

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, ItemId};
use crate::itempools::PoolMembership;
use crate::origin::{self, Origin};
use crate::sprite::SpriteRef;
use crate::text::Text;
use crate::xml::{self, Element};

const SOURCE: Source = Source::Items;

/// The four kinds `items.xml` files an item under, and the one definition of them: it crosses
/// the IPC as `ItemKindView`, and an icon URL spells it with the same word
/// ([`ItemKind::name`]).
///
/// A fieldless enum: on the wire it's a bare camelCase string (`"passive"`), like
/// `OriginView`. The tag exists to distinguish variants that carry different data, and here
/// there are none: `{"kind":"passive"}` would cost a key on every row and say nothing more.
/// The day a variant gains a field, the enum becomes tagged and the TypeScript side follows.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(rename = "ItemKindView")]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Passive,
    Active,
    Familiar,
    Trinket,
}

impl ItemKind {
    /// Every kind, in the order `Ord` sorts them.
    pub const ALL: [ItemKind; 4] = [
        ItemKind::Passive,
        ItemKind::Active,
        ItemKind::Familiar,
        ItemKind::Trinket,
    ];

    /// The three kinds that share one id space — `items.xml`'s collectibles, the only kinds a
    /// pool names — in the order a lookup by bare id searches them. Trinkets are numbered
    /// apart: passive 46 and trinket 46 are two things.
    pub const COLLECTIBLES: [ItemKind; 3] =
        [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar];

    /// The kind's one word: the element `items.xml` files it under, the segment of an icon URL
    /// (`item/passive/92`), and the string `serde` writes for it on the IPC. Three uses of one
    /// spelling, held together by a test on the serialized form in `ipc`.
    pub const fn name(self) -> &'static str {
        match self {
            ItemKind::Passive => "passive",
            ItemKind::Active => "active",
            ItemKind::Familiar => "familiar",
            ItemKind::Trinket => "trinket",
        }
    }

    /// The inverse of [`ItemKind::name`]. `None` for any other word: an element name and a
    /// URL segment are both open strings.
    pub fn from_name(name: &str) -> Option<ItemKind> {
        ItemKind::ALL.into_iter().find(|k| k.name() == name)
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
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return Vec::new();
    };
    let gfxroot = xml::root_attr(&els, "items", "gfxroot", "gfx/items");
    els.iter()
        .filter_map(|e| ItemKind::from_name(&e.name).map(|k| (e, k)))
        .filter_map(|(e, kind)| item_from(e, kind, &gfxroot, diagnostics))
        .collect()
}

fn item_from(e: &Element, kind: ItemKind, gfxroot: &str, d: &mut Vec<Diagnostic>) -> Option<Item> {
    let id = xml::required_id(e, "id", SOURCE, d)?;
    let gfx = xml::required_attr(e, "gfx", id, SkipReason::MissingSprite, SOURCE, d)?;
    let name = xml::required_attr(e, "name", id, SkipReason::MissingName, SOURCE, d)?;
    Some(Item {
        id: ItemId(id),
        kind,
        name: Text::from_attr(name),
        description: Text::from_attr(e.attr("description").unwrap_or("")),
        // Quality and tags come from items_metadata.xml (Catalog::build fills them in later).
        quality: None,
        tags: Vec::new(),
        sprite: SpriteRef::whole(format!("{gfxroot}/{}/{gfx}", kind.folder())),
        unlocked_by: xml::unlocked_by(e),
        pools: Vec::new(),
        origin: origin::origin_of(kind, ItemId(id)),
    })
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

    /// The position each kind must hold in `ALL`. An exhaustive match, so a new kind does not
    /// compile until it is given one here, next to the list it has to join.
    fn position_in_all(kind: ItemKind) -> usize {
        match kind {
            ItemKind::Passive => 0,
            ItemKind::Active => 1,
            ItemKind::Familiar => 2,
            ItemKind::Trinket => 3,
        }
    }

    #[test]
    fn all_holds_every_kind_once_in_sort_order() {
        let positions: Vec<usize> = ItemKind::ALL.iter().map(|&k| position_in_all(k)).collect();
        assert_eq!(positions, (0..ItemKind::ALL.len()).collect::<Vec<_>>());
        assert!(ItemKind::ALL.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn the_collectibles_are_every_kind_that_lands_in_the_collectibles_folder() {
        let by_folder: Vec<ItemKind> = ItemKind::ALL
            .into_iter()
            .filter(|k| k.folder() == "collectibles")
            .collect();
        assert_eq!(by_folder, ItemKind::COLLECTIBLES.to_vec());
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
