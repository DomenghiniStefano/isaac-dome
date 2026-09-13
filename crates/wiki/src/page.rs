//! From a wiki page to dataset entries: each infobox gives an entry, of the kind the
//! infobox itself declares (not the folder the page was listed under), and level-2
//! sections with a recognized title give the text. The preamble is discarded: it's the
//! "X is a passive item…" sentence that `catalog` already covers.

use serde::{Deserialize, Serialize};

use crate::blocks::parse_blocks;
use crate::infobox::{
    entry_facts, extract_infoboxes, infobox_from, leading_number, InfoboxKind, RawInfobox,
};
use crate::resolver::Resolver;
use crate::sections::{section_kind, split_page};
use crate::{Diagnostics, Entry, Section};

/// The page kind, as `index.json` classifies it: decides the folder in `raw/` and the
/// template the page was listed from. It does not decide the entries' kind: each infobox
/// says that for itself (a page can contain infoboxes of different kinds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Collectible,
    Trinket,
    Achievement,
    Boss,
    Challenge,
    Character,
    Transformation,
}

impl PageKind {
    pub const ALL: [PageKind; 7] = [
        PageKind::Collectible,
        PageKind::Trinket,
        PageKind::Achievement,
        PageKind::Boss,
        PageKind::Challenge,
        PageKind::Character,
        PageKind::Transformation,
    ];

    /// The subfolder of `raw/pages/`.
    pub fn dir(self) -> &'static str {
        match self {
            PageKind::Collectible => "collectible",
            PageKind::Trinket => "trinket",
            PageKind::Achievement => "achievement",
            PageKind::Boss => "boss",
            PageKind::Challenge => "challenge",
            PageKind::Character => "character",
            PageKind::Transformation => "transformation",
        }
    }

    /// The wiki template whose transclusions list the pages of this kind.
    pub fn template(self) -> &'static str {
        match self {
            PageKind::Collectible => "Template:Infobox collectible",
            PageKind::Trinket => "Template:Infobox trinket",
            PageKind::Achievement => "Template:Infobox achievement",
            PageKind::Boss => "Template:Infobox boss",
            PageKind::Challenge => "Template:Infobox challenge",
            PageKind::Character => "Template:Infobox character",
            PageKind::Transformation => "Template:Infobox transformation",
        }
    }
}

/// The key an entry enters the dataset with. A boss carries the bestiary triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKey {
    Item(u32),
    Trinket(u32),
    Achievement(u32),
    Boss(u32, u32, u32),
    Challenge(u32),
    Character(u32),
}

/// A numeric infobox parameter: the leading digits, because the wiki can follow the
/// number with a note (`349<br>{{plat|PS4}}&nbsp;340`: the PC id, then the PS4 one).
fn number(ib: &RawInfobox, name: &str) -> Option<u32> {
    leading_number(ib.params.get(name)?)
}

/// The entry key for an infobox; `None` if the id is missing, or if an item or trinket
/// isn't in the current edition's table for this page (Tonsil's collectible 474, which in
/// Repentance+ is Broken Glass Cannon).
fn entry_key(kind: InfoboxKind, title: &str, ib: &RawInfobox, r: &Resolver) -> Option<EntryKey> {
    Some(match kind {
        InfoboxKind::Passive | InfoboxKind::Activated => {
            EntryKey::Item(r.item_id_of_page(title, number(ib, "id")?)?)
        }
        InfoboxKind::Trinket => EntryKey::Trinket(r.trinket_id_of_page(title, number(ib, "id")?)?),
        InfoboxKind::Achievement => EntryKey::Achievement(number(ib, "id")?),
        InfoboxKind::Challenge => EntryKey::Challenge(number(ib, "number")?),
        // Our own map first (by `name`, then by title: Lazarus's page also contains
        // Lazarus Risen), the infobox id as a fallback.
        InfoboxKind::Character => EntryKey::Character(
            character(title, ib, r)
                .map(|(id, _)| id)
                .or_else(|| number(ib, "id"))?,
        ),
        // The entity table knows variant and subtype; the infobox only the id.
        InfoboxKind::Boss => match r.boss_key(title) {
            Some((id, variant, subtype)) => EntryKey::Boss(id, variant, subtype),
            None => EntryKey::Boss(number(ib, "id")?, 0, 0),
        },
    })
}

/// The page's kept sections, in the order they appear; ones with an unrecognized title
/// count among the discarded.
fn sections(text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Section> {
    let (_preamble, raw) = split_page(text);
    raw.iter()
        .filter_map(|s| match section_kind(&s.title) {
            Some(kind) => Some(Section {
                kind,
                blocks: parse_blocks(&s.body, r, d),
            }),
            None => {
                d.discarded_section(&s.title);
                None
            }
        })
        .collect()
}

/// The infobox's `name`, if it's present and not empty.
fn name_param(ib: &RawInfobox) -> Option<&str> {
    ib.params
        .get("name")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

/// The character of an infobox according to our own map: by `name`, otherwise by title.
fn character(title: &str, ib: &RawInfobox, r: &Resolver) -> Option<(u32, String)> {
    name_param(ib)
        .and_then(|n| r.character_of_page(n))
        .or_else(|| r.character_of_page(title))
}

/// The entry's title. An achievement takes the infobox's `name` (a page holds dozens of
/// them); a character its `name`, then the name from our own map ("Lazarus Risen" lives
/// on Lazarus's page, `??? (Character)` is called `???`); everything else the page title,
/// because there `name` carries edition markup.
fn entry_title(kind: InfoboxKind, title: &str, ib: &RawInfobox, r: &Resolver) -> String {
    let name = name_param(ib);
    match kind {
        InfoboxKind::Achievement => name.unwrap_or(title).to_string(),
        InfoboxKind::Character => name
            .map(str::to_string)
            .or_else(|| character(title, ib, r).map(|(_, n)| n))
            .unwrap_or_else(|| title.to_string()),
        InfoboxKind::Passive
        | InfoboxKind::Activated
        | InfoboxKind::Trinket
        | InfoboxKind::Boss
        | InfoboxKind::Challenge => title.to_string(),
    }
}

/// The entries of a page, one per recognized infobox, each of its own infobox's kind.
/// The page's sections go to every entry except achievements, which are containers with
/// no text of their own; they're parsed once. An infobox with no key is counted in
/// `pages_without_id` and skipped.
pub fn parse_page(
    title: &str,
    revid: u64,
    text: &str,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Vec<(EntryKey, Entry)> {
    let mut out = Vec::new();
    let mut page_sections: Option<Vec<Section>> = None;
    for ib in extract_infoboxes(text) {
        let Some(kind) = InfoboxKind::of(&ib.name) else {
            continue;
        };
        let Some(key) = entry_key(kind, title, &ib, r) else {
            d.pages_without_id += 1;
            continue;
        };
        let sections = match kind {
            InfoboxKind::Achievement => Vec::new(),
            InfoboxKind::Passive
            | InfoboxKind::Activated
            | InfoboxKind::Trinket
            | InfoboxKind::Boss
            | InfoboxKind::Challenge
            | InfoboxKind::Character => page_sections
                .get_or_insert_with(|| sections(text, r, d))
                .clone(),
        };
        let facts = entry_facts(&ib, r, d);
        out.push((
            key,
            Entry {
                title: entry_title(kind, title, &ib, r),
                revid,
                description: facts.description,
                dlc: facts.dlc,
                unlocked_by: facts.unlocked_by,
                infobox: infobox_from(kind, &ib, r, d),
                sections,
            },
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, SectionKind};

    /// Measured on 2026-09-13: `Template:Infobox transformation` exists and is transcluded by
    /// exactly the sixteen pages the Cargo table has rows for. The kind is declared like the
    /// other six so that `fetch`, which walks `PageKind::ALL`, picks the pages up without a
    /// special case — until it exists, no transformation page is downloaded at all.
    #[test]
    fn the_transformation_kind_names_its_template_and_its_folder() {
        assert_eq!(PageKind::Transformation.dir(), "transformation");
        assert_eq!(
            PageKind::Transformation.template(),
            "Template:Infobox transformation"
        );
        assert!(PageKind::ALL.contains(&PageKind::Transformation));
    }

    #[test]
    fn the_three_common_facts_land_on_the_entry_not_the_infobox() {
        let src = "{{infobox passive collectible\n | id = 25\n | dlc = r\n | description = Tears up\n | unlocked by = Epic Fetus\n}}\n== Effects ==\n* a\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        let e = &v[0].1;
        assert_eq!(e.dlc, vec![crate::Dlc::Repentance]);
        assert_eq!(e.unlocked_by, Some(crate::Target::Achievement { id: 62 }));
        assert!(matches!(
            e.description.first(),
            Some(crate::Inline::Text { text, .. }) if text.contains("Tears up")
        ));
    }

    #[test]
    fn collectible_page_yields_one_entry_with_kept_sections() {
        let src = "{{infobox passive collectible\n | id = 25\n | quote = x\n}}\n{{cit|p|r}}\n\n== Effects ==\n* a\n== Trivia ==\n* t\n== Notes ==\nn\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, EntryKey::Item(25));
        let e = &v[0].1;
        assert_eq!(e.title, "Breakfast");
        assert_eq!(e.revid, 7);
        assert_eq!(
            e.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
            vec![SectionKind::Effects, SectionKind::Notes]
        );
        assert!(matches!(e.sections[0].blocks[0], Block::List { .. }));
        assert_eq!(d.discarded_sections.get("Trivia"), Some(&1));
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn achievement_storage_page_yields_many() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Breakfast\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}} {{infobox achievement\n | name = NoId\n}}";
        let mut d = Diagnostics::default();
        let v = parse_page("Achievements/Rebirth 1", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Achievement(62), EntryKey::Achievement(2)]
        );
        assert_eq!(v[0].1.title, "Epic Fetus");
        assert!(v[0].1.sections.is_empty());
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn an_id_with_a_platform_note_is_its_leading_number() {
        // Achievements/Afterbirth † 2: the PC id followed by the PS4 one.
        let src = "{{infobox achievement\n | name = Black Hole\n | id = 349<br>{{plat|PS4}}&nbsp;340\n}} {{infobox achievement\n | name = Negative\n | id = -1\n}}";
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Achievements/Afterbirth † 2",
            1,
            src,
            &test_resolver(),
            &mut d,
        );
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Achievement(349)]
        );
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn boss_key_from_entity_table_or_infobox() {
        let src = "{{infobox boss\n | id = 45\n | base hp = 1\n}}\n== Behavior ==\nx\n";
        let v = parse_page("Mom", 1, src, &test_resolver(), &mut Diagnostics::default());
        assert_eq!(v[0].0, EntryKey::Boss(45, 0, 0));
        let v = parse_page(
            "Unknown Boss",
            1,
            "{{infobox boss\n | id = 999\n}}",
            &test_resolver(),
            &mut Diagnostics::default(),
        );
        assert_eq!(v[0].0, EntryKey::Boss(999, 0, 0));
    }

    #[test]
    fn each_infobox_is_dispatched_by_its_own_kind() {
        // Tonsil: the page is listed among collectibles, but trinket 97 is the only entry
        // for the current edition; collectible 474 (Afterbirth+) is discarded and counted.
        let src = "{{infobox trinket\n | id = 97\n}}\nintro\n{{infobox passive collectible\n | id = 474\n}}\n== Effects ==\ne\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Tonsil", 3, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Trinket(97)]
        );
        assert_eq!(v[0].1.title, "Tonsil");
        assert_eq!(v[0].1.sections.len(), 1);
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn a_page_with_two_entries_gives_the_sections_to_both_and_parses_them_once() {
        let src = "{{infobox activated collectible\n | id = 550\n}}\n== Effects ==\na\n{{infobox passive collectible\n | id = 551\n}}\n== Trivia ==\nt\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Broken Shovel", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Item(550), EntryKey::Item(551)]
        );
        assert_eq!(v[0].1.sections, v[1].1.sections);
        assert_eq!(v[0].1.sections[0].kind, SectionKind::Effects);
        assert_eq!(d.discarded_sections.get("Trivia"), Some(&1));
    }

    #[test]
    fn an_alternate_character_form_takes_its_own_name() {
        let src = "{{infobox character\n | name = Lazarus\n | id = 8\n}}\n{{infobox character\n | name = Lazarus Risen\n | id = 11\n}}\n{{infobox monster\n | name = Dark Esau\n | id = 866\n}}\n== Notes ==\nn\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Lazarus", 1, src, &test_resolver(), &mut d);
        assert_eq!(
            v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
            vec![EntryKey::Character(8), EntryKey::Character(11)]
        );
        assert_eq!(v[0].1.title, "Lazarus");
        assert_eq!(v[1].1.title, "Lazarus Risen");
        assert_eq!(v[1].1.sections.len(), 1);
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn a_character_page_keys_from_our_map_before_the_infobox_id() {
        let mut d = Diagnostics::default();
        // The wiki writes 14 for Isaac: the key is the one from our own map.
        let v = parse_page(
            "Isaac",
            1,
            "{{infobox character\n | name = Isaac\n | id = 14\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(0));
        assert_eq!(v[0].1.title, "Isaac");
        // Without `name`: the page title, without « (Character)», and the map's name.
        let v = parse_page(
            "??? (Character)",
            1,
            "{{infobox character\n | id = 4\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(4));
        assert_eq!(v[0].1.title, "???");
        // A character the map doesn't know: the infobox id, as before.
        let v = parse_page(
            "Nobody",
            1,
            "{{infobox character\n | id = 99\n}}",
            &test_resolver(),
            &mut d,
        );
        assert_eq!(v[0].0, EntryKey::Character(99));
        assert_eq!(v[0].1.title, "Nobody");
        assert_eq!(d.pages_without_id, 0);
    }

    #[test]
    fn an_item_page_the_table_does_not_know_is_dropped() {
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Nope",
            1,
            "{{infobox passive collectible\n | id = 5\n}}",
            &test_resolver(),
            &mut d,
        );
        assert!(v.is_empty());
        assert_eq!(d.pages_without_id, 1);
        // Same page, an id different from the table's: it's discarded all the same.
        let mut d = Diagnostics::default();
        let v = parse_page(
            "Breakfast",
            1,
            "{{infobox passive collectible\n | id = 26\n}}",
            &test_resolver(),
            &mut d,
        );
        assert!(v.is_empty());
        assert_eq!(d.pages_without_id, 1);
    }
}
