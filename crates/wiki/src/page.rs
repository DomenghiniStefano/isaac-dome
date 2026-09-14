//! From a wiki page to dataset entries: each infobox gives an entry, of the kind the
//! infobox itself declares (not the folder the page was listed under), and level-2
//! sections with a recognized title give the text. The preamble is discarded: it's the
//! "X is a passive item…" sentence that `catalog` already covers — **except on a
//! transformation**, where it is the only place the wiki says how you become one (B51).

use serde::{Deserialize, Serialize};

use crate::blocks::parse_blocks;
use crate::infobox::{
    entry_facts, extract_infoboxes, infobox_from, leading_number, InfoboxKind, RawInfobox,
};
use crate::resolver::{is_layout_template, Resolver};
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

    /// The wiki templates whose transclusions list the pages of this kind. Plural because
    /// the characters need two: `Infobox characters` is a different template, not a
    /// spelling of the first, and it holds the four pages that state two playable forms
    /// (B45). A kind with one template is the ordinary case and reads the same.
    pub fn templates(self) -> &'static [&'static str] {
        match self {
            PageKind::Collectible => &["Template:Infobox collectible"],
            PageKind::Trinket => &["Template:Infobox trinket"],
            PageKind::Achievement => &["Template:Infobox achievement"],
            PageKind::Boss => &["Template:Infobox boss"],
            PageKind::Challenge => &["Template:Infobox challenge"],
            PageKind::Character => &["Template:Infobox character", "Template:Infobox characters"],
            PageKind::Transformation => &["Template:Infobox transformation"],
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
    Transformation(u32),
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
        // The Cargo table first, because it is the only source with an id for every page:
        // Super Bum's infobox says `id = n/a` and the table maps that onto 1000. By the
        // infobox alone that page would be dropped as having no id.
        InfoboxKind::Transformation => EntryKey::Transformation(
            r.transformation_of_page(title)
                .or_else(|| number(ib, "id"))?,
        ),
    })
}

/// The page's opening sentence, as inline text: what `split_page` puts before the first
/// level-2 heading, with the infobox and the header template gone — `parse_inline` resolves
/// the links and skips the layout templates, and an empty preamble gives an empty list.
///
/// Discarded for every other kind, where it is the "X is a passive item…" sentence that
/// `catalog` already covers. B51: on a transformation it carries the requirement.
fn preamble(text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<crate::Inline> {
    let (pre, _) = split_page(text);
    crate::inline::parse_inline(without_the_boxes(&pre).trim(), r, d)
}

/// The preamble text with the infobox and the page header taken out, and **nothing else**:
/// a `{{i|Flip}}` in the same sentence is a reference the prose needs. `parse_template_at`
/// is what says where a template ends, which a line-by-line pass cannot — an infobox spans
/// a dozen lines and a header one.
fn without_the_boxes(text: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while let Some(pos) = text.get(i..).and_then(|rest| rest.find("{{")) {
        let at = i + pos;
        out.push_str(&text[i..at]);
        match crate::template::parse_template_at(text, at) {
            Some((t, end)) if t.name.starts_with("infobox") || is_layout_template(&t.name) => {
                i = end
            }
            // Any other template is prose: leave it for `parse_inline` to resolve.
            Some((_, end)) => {
                out.push_str(&text[at..end]);
                i = end;
            }
            None => {
                out.push_str("{{");
                i = at + 2;
            }
        }
    }
    out.push_str(&text[i..]);
    out
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
        | InfoboxKind::Challenge
        | InfoboxKind::Transformation => title.to_string(),
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
            d.unknown_infobox(&ib.name);
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
            | InfoboxKind::Character
            | InfoboxKind::Transformation => page_sections
                .get_or_insert_with(|| sections(text, r, d))
                .clone(),
        };
        let facts = entry_facts(&ib, r, d);
        let description = match kind {
            // B51: a transformation's preamble is the only place the wiki says **how you
            // become one** — Adult's "upon taking three Puberty pills" is there and nowhere
            // else on the page — so for this kind alone the discarded sentence is kept, in
            // front of the infobox's own `description`. That parameter restates the Effects
            // section on all sixteen pages (Guppy's is empty) and on none of them says how
            // the transformation happens, so nothing is dropped by putting it second.
            InfoboxKind::Transformation => {
                let mut out = preamble(text, r, d);
                // Two sentences written in two places meet in one field, and nothing in
                // either carries the space between them.
                if !out.is_empty() && !facts.description.is_empty() {
                    out.push(crate::Inline::Text {
                        text: " ".to_string(),
                        style: crate::Style::Plain,
                    });
                }
                out.extend(facts.description);
                out
            }
            InfoboxKind::Passive
            | InfoboxKind::Activated
            | InfoboxKind::Trinket
            | InfoboxKind::Achievement
            | InfoboxKind::Boss
            | InfoboxKind::Challenge
            | InfoboxKind::Character => facts.description,
        };
        out.push((
            key,
            Entry {
                title: entry_title(kind, title, &ib, r),
                revid,
                description,
                dlc: facts.dlc,
                unlocked_by: facts.unlocked_by,
                infobox: infobox_from(kind, &ib, text, r, d),
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
    /// B51: a transformation's preamble is the only place the wiki says **how you become
    /// one** — Adult states "upon taking three Puberty pills" there and nowhere else — and the
    /// parser drops preambles by a rule written for the "X is a passive item…" sentence that
    /// `catalog` already covers. `transformation::requires` even reads that same line for its
    /// digit and throws the sentence away.
    ///
    /// It is kept ahead of the infobox's own `description`, which on all sixteen pages
    /// restates the Effects section (Guppy's is empty) and on none of them says how the
    /// transformation happens.
    #[test]
    fn a_transformations_preamble_is_kept_ahead_of_its_description() {
        let src = "{{header transformations}}
{{infobox transformation
 | id = 1
 | description = Grants flight.
}}

'''Beelzebub''' is a [[transformation]], turning Isaac into a fly after picking up 3 fly items.

== Effects ==
* a
";
        let mut d = Diagnostics::default();
        let v = parse_page("Beelzebub", 7, src, &test_resolver(), &mut d);
        let text = crate::plain(&v[0].1.description);
        assert!(
            text.starts_with("Beelzebub is a transformation, turning Isaac into a fly after picking up 3 fly items."),
            "{text}"
        );
        // The two sentences are written in two places and meet in one field: the space
        // between them belongs to neither, so it is put here.
        assert!(text.ends_with("3 fly items. Grants flight."), "{text}");
    }

    #[test]
    fn the_transformation_kind_names_its_template_and_its_folder() {
        assert_eq!(PageKind::Transformation.dir(), "transformation");
        assert_eq!(
            PageKind::Transformation.templates(),
            ["Template:Infobox transformation"]
        );
        assert!(PageKind::ALL.contains(&PageKind::Transformation));
    }

    /// B45: a kind's pages are the transclusions of **its templates**, plural, because the
    /// characters are listed by two — `Infobox character` for the thirty pages that hold
    /// one form, and `Infobox characters` for the four that hold two. Enumerating only the
    /// singular is not a query that *misses* those four, it is a query they are not in,
    /// which is why the fetch reported no error while losing eight playable characters.
    ///
    /// Measured 2026-09-14 against the wiki: `Infobox characters` has exactly 4
    /// transclusions in namespace 0, and they are those four pages.
    #[test]
    fn the_characters_are_listed_by_two_templates_not_one() {
        assert_eq!(
            PageKind::Character.templates(),
            ["Template:Infobox character", "Template:Infobox characters"]
        );
        // Every other kind still has exactly one, so nothing else changed shape.
        for kind in PageKind::ALL {
            let n = kind.templates().len();
            assert_eq!(
                n,
                if kind == PageKind::Character { 2 } else { 1 },
                "{kind:?}"
            );
        }
    }

    /// B45's mechanism, not its symptom. `extract_infoboxes` takes any template whose name
    /// starts with `infobox`, and `InfoboxKind::of` answers `None` for the ones we do not
    /// know — after which the page simply produces nothing, with no error and no counter.
    /// That is how four character pages could have been downloaded and parsed into zero
    /// entries without a word. Counted now, the way `unknown_entities` counts the entities
    /// that shipped undecoded for months.
    #[test]
    fn an_infobox_whose_template_we_do_not_know_is_counted() {
        let src = "{{infobox rune\n | id = 1\n}}\n== Effects ==\n* a\n";
        let mut d = Diagnostics::default();
        let v = parse_page("Rune of Hagalaz", 7, src, &test_resolver(), &mut d);
        assert!(v.is_empty());
        assert_eq!(d.unknown_infoboxes.get("infobox rune"), Some(&1));
    }

    /// The counter has to be able to stay quiet, or it says nothing by saying nothing.
    #[test]
    fn a_template_we_know_is_not_counted_as_unknown() {
        let src = "{{infobox passive collectible\n | id = 25\n}}\n";
        let mut d = Diagnostics::default();
        parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
        assert!(d.unknown_infoboxes.is_empty());
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
