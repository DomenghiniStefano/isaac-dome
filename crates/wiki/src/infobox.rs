//! Infoboxes: extracted as top-level templates, converted into the `Infobox` of their kind.

use std::collections::BTreeMap;

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::template::{template_segments, Segment, Template};
use crate::{CollectibleTemplate, Diagnostics, Dlc, Infobox, Inline, Target};

/// An `{{infobox …}}` template as-is: lowercase name and raw named parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInfobox {
    pub name: String,
    pub params: BTreeMap<String, String>,
}

/// The template that states **two** characters in one block (B45): Jacob & Esau, The
/// Forgotten, Tainted Forgotten, Tainted Lazarus, and no other page on the wiki.
const PLURAL_CHARACTER: &str = "infobox characters";
const CHARACTER: &str = "infobox character";

/// The second form's parameters carry this suffix: `name 2`, `health 2`, `collectibles 2`.
const SECOND_FORM: &str = " 2";

/// The parameters `Template:Infobox characters` reads **without** a ` 2` twin, measured on
/// the template's own source (2026-09-14): they are stated once and describe the page, so
/// both forms carry them. Everything else it reads — `name`, `alias`, `image`, `damage`,
/// `tears`, `shot speed`, `range`, `speed`, `luck`, `health`, `collectibles`, `pickups`
/// and `id` — exists in both, so an unsuffixed one of those belongs to the first form
/// alone. `id` is the one that matters: the four pages state `id` and never `id 2`, and a
/// second form inheriting it would take the first form's identity instead of resolving by
/// name — silently, since both would then key the same entry.
const SHARED_BY_BOTH_FORMS: &[&str] = &["dlc", "description", "unlocked by", "hidden"];

/// The plural template as the two ordinary character infoboxes it stands for, in page
/// order. Splitting here keeps `InfoboxKind` a closed set and leaves every reader —
/// `entries_of`, `parent_check` — looking at what it already understands.
fn character_forms(params: BTreeMap<String, String>) -> [RawInfobox; 2] {
    let mut second = BTreeMap::new();
    let mut first = BTreeMap::new();
    for (key, value) in params {
        match key.strip_suffix(SECOND_FORM) {
            Some(base) => {
                second.insert(base.to_string(), value);
            }
            None => {
                if SHARED_BY_BOTH_FORMS.contains(&key.as_str()) {
                    second.insert(key.clone(), value.clone());
                }
                first.insert(key, value);
            }
        }
    }
    [
        RawInfobox {
            name: CHARACTER.to_string(),
            params: first,
        },
        RawInfobox {
            name: CHARACTER.to_string(),
            params: second,
        },
    ]
}

/// Every top-level `{{infobox …}}`, in the order they appear. Other templates are
/// skipped whole, so an infobox nested inside another template does not count.
pub fn extract_infoboxes(text: &str) -> Vec<RawInfobox> {
    template_segments(text)
        .flat_map(|segment| match segment {
            Segment::Template { template, .. } => infoboxes_in(template),
            Segment::Text(_) => Vec::new(),
        })
        .collect()
}

/// The infoboxes one top-level template stands for: two for the plural character template,
/// one for any other `infobox …`, none for everything else.
fn infoboxes_in(t: Template) -> Vec<RawInfobox> {
    if t.name == PLURAL_CHARACTER {
        return character_forms(t.named).into();
    }
    if t.name.starts_with("infobox") {
        return vec![RawInfobox {
            name: t.name,
            params: t.named,
        }];
    }
    Vec::new()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoboxKind {
    Passive,
    Activated,
    Trinket,
    Achievement,
    Boss,
    Challenge,
    Character,
    Transformation,
}

impl InfoboxKind {
    /// From the template name (already lowercase) to the infobox kind.
    pub fn of(name: &str) -> Option<InfoboxKind> {
        Some(match name {
            // `infobox collectible` with no adjective is the generic form, and the game
            // has no third kind of collectible: it counts as passive.
            "infobox passive collectible" | "infobox collectible" => InfoboxKind::Passive,
            "infobox activated collectible" => InfoboxKind::Activated,
            "infobox trinket" => InfoboxKind::Trinket,
            "infobox achievement" => InfoboxKind::Achievement,
            "infobox boss" => InfoboxKind::Boss,
            "infobox challenge" => InfoboxKind::Challenge,
            "infobox character" => InfoboxKind::Character,
            "infobox transformation" => InfoboxKind::Transformation,
            _ => return None, // allowed: template name, an open-ended string
        })
    }
}

/// Parameters the infoboxes declare that no type keeps, each covered by a reason below. A
/// parameter in neither this list nor a field is a bug, and `no_silent_parameter` says so —
/// because "parsed, then dropped, with a green suite" is exactly the defect fixed on
/// 2026-09-13, and it was invisible for months.
///
/// - **Asset names** (`image name`, `costume name`, `files name`, `tear app*`, `costume*`,
///   `bomb app`, `portrait*`, `image`, `character appearance`): they name files inside the
///   user's own copy of the game. `unpack` extracts sprites from there and no wiki image is
///   ever shipped, so a file name from the wiki has nothing to open.
/// - **Identity** (`name`, `id`, `number`, `link`, `alias`): already resolved into the
///   entry's key and title before the infobox is converted. Keeping them twice invites the
///   two copies to disagree.
/// - **Editorial** (`hidden`, `appearance`, `behavior`, `is mini-boss`, `oldpool`,
///   `special goal`): presentation switches and prose the sections already carry.
/// - **`requirement`**, on a transformation: it looks like data and is not. All sixteen
///   rows of the Cargo table hold the identical string `three items from this set` — Adult,
///   whose infobox has no such parameter, included — so it is the template's default and
///   states nothing about any row. The count is read from the page body instead
///   (`transformation::requires`), where each page says its own.
pub const IGNORED_PARAMS: &[&str] = &[
    "image name",
    "costume name",
    "files name",
    "tear app name",
    "tear app",
    "tear app scale",
    "costume",
    "costume scale",
    "bomb app",
    "portrait",
    "portrait name",
    "image",
    "character appearance",
    "name",
    "id",
    "number",
    "link",
    "alias",
    "hidden",
    "appearance",
    "behavior",
    "is mini-boss",
    "oldpool",
    "special goal",
    "requirement",
];

fn param<'a>(ib: &'a RawInfobox, name: &str) -> &'a str {
    ib.params.get(name).map(String::as_str).unwrap_or("")
}

fn text(ib: &RawInfobox, name: &str) -> String {
    param(ib, name).trim().to_string()
}

fn inline(ib: &RawInfobox, name: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    parse_inline(param(ib, name), r, d)
}

/// An achievement's unlock-paper line, or nothing when the wiki wrote a placeholder in its
/// place: 136 rows read "???" and one "...", and a quote made only of those says nothing.
fn paper_line(inline: Vec<Inline>) -> Vec<Inline> {
    let placeholder = crate::plain(&inline)
        .chars()
        .all(|c| c == '?' || c == '.' || c == '…' || c.is_whitespace());
    if placeholder {
        Vec::new()
    } else {
        inline
    }
}

/// The `dlc` parameter as the editions it names. Absent is the one value that stays empty:
/// the page declares no range, which `Editions::of` then reads back as "narrows nothing".
/// An unreadable code is counted by `parse_code` and also leaves the entry declaring
/// nothing, since the wiki itself answers `0` there.
fn dlc_range(code: &str, d: &mut Diagnostics) -> Vec<Dlc> {
    if code.trim().is_empty() {
        return Vec::new();
    }
    match crate::editions::Editions::parse(code) {
        Some(e) => e.list(),
        None => {
            d.unknown_dlc_code(code.trim());
            Vec::new()
        }
    }
}

fn yes(ib: &RawInfobox, name: &str) -> bool {
    param(ib, name).trim().eq_ignore_ascii_case("yes")
}

/// `"devil summonable offensive"` → three tags. The vocabulary is the game's and open, so
/// this stays a list of strings: a closed enum breaks the day the game adds a tag.
///
/// The value is **parsed before it is split**. 64 of 714 pages qualify a tag by edition
/// (`nolostbr summonable {{dlc|r+|fly}}`), and splitting the raw string on whitespace made
/// `{{dlc|r+|fly}}` a tag in its own right — wikitext, presented as a tag, in the dataset.
/// Parsing first turns it into `Inline::Edition` around the word `fly`; flattening then
/// gives the tag the game also knows. The edition qualification is dropped on purpose: the
/// game's `items_metadata.xml` already states what holds for the installed edition, and it
/// is the better source for that question.
fn tags(ib: &RawInfobox, name: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<String> {
    crate::inline::plain(&parse_inline(param(ib, name), r, d))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// The leading digits: `"250 (x2)"` → 250, `"6666"` → 6666, no digits → `None`.
pub(crate) fn leading_number(s: &str) -> Option<u32> {
    s.trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// The three facts every kind declares, read once per infobox and carried on `Entry`
/// instead of being repeated in all six variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryFacts {
    pub description: Vec<Inline>,
    pub dlc: Vec<Dlc>,
    pub unlocked_by: Option<Target>,
}

/// `unlocked by` is an achievement *name* on every kind that uses it — the 277 collectible
/// pages that carry it read "???'s Only Friend", "A Pound of Flesh", … — so one resolution
/// serves all six kinds.
pub fn entry_facts(ib: &RawInfobox, r: &Resolver, d: &mut Diagnostics) -> EntryFacts {
    EntryFacts {
        description: inline(ib, "description", r, d),
        dlc: dlc_range(param(ib, "dlc"), d),
        unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
    }
}

/// A collectible's infobox. The template it came from is passed in rather than read again,
/// because only the caller's `match` knows which of the two names the page used.
fn item_from(
    ib: &RawInfobox,
    template: CollectibleTemplate,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Infobox {
    Infobox::Item {
        quote: inline(ib, "quote", r, d),
        template,
        quality: param(ib, "quality").trim().parse().ok(),
        tags: tags(ib, "tags", r, d),
        recharge: inline(ib, "recharge", r, d),
        devil_price: inline(ib, "devil price", r, d),
        shop_price: inline(ib, "shop price", r, d),
        pools: inline(ib, "pool", r, d),
    }
}

/// Converts a raw infobox into the `Infobox` of its kind. Missing parameters count as an
/// empty string: a missing field degrades, it doesn't block the page.
///
/// `page` is the whole page text, and six of the seven arms ignore it. Only a transformation
/// needs it: its count and its item set are stated in the body, not in the box (spec §2.2).
/// The alternative — building that one variant outside this function — would cost the
/// property that one place builds every infobox, which is worth more than an unused
/// parameter.
pub fn infobox_from(
    kind: InfoboxKind,
    ib: &RawInfobox,
    page: &str,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Infobox {
    match kind {
        // Two arms rather than one with an inner `match kind`: an inner match would need a
        // `_` arm for the kinds this branch cannot see, and a `_` on a closed enum is what
        // stops a new variant from breaking the build.
        InfoboxKind::Passive => item_from(ib, CollectibleTemplate::Passive, r, d),
        InfoboxKind::Activated => item_from(ib, CollectibleTemplate::Activated, r, d),
        InfoboxKind::Trinket => Infobox::Trinket {
            quote: inline(ib, "quote", r, d),
            tags: tags(ib, "tags", r, d),
            pools: inline(ib, "pool", r, d),
        },
        InfoboxKind::Achievement => Infobox::Achievement {
            quote: paper_line(inline(ib, "description", r, d)),
            requirements: inline(ib, "requirements", r, d),
            notes: inline(ib, "notes", r, d),
            unlocks: r.by_page_title(param(ib, "link")),
        },
        InfoboxKind::Boss => Infobox::Boss {
            base_hp: leading_number(param(ib, "base hp")),
            stage_hp: inline(ib, "stage hp", r, d),
            variant: leading_number(param(ib, "variant")),
            environment: inline(ib, "environment", r, d),
            pool: inline(ib, "pool", r, d),
        },
        InfoboxKind::Challenge => Infobox::Challenge {
            blindfolded: yes(ib, "blindfolded"),
            has_shops: yes(ib, "has shops"),
            has_treasure_rooms: yes(ib, "has treasure rooms"),
            items: inline(ib, "item", r, d),
            trinkets: inline(ib, "trinket", r, d),
            pickups: inline(ib, "pickup", r, d),
            health: inline(ib, "health", r, d),
            curse: inline(ib, "curse", r, d),
            goal: inline(ib, "goal", r, d),
            character: r.by_page_title(param(ib, "character")),
            // `unlocks` is usually a page title; for achievements it's the name.
            unlocks: r
                .by_page_title(param(ib, "unlocks"))
                .or_else(|| r.achievement_by_name(param(ib, "unlocks"))),
        },
        InfoboxKind::Transformation => {
            let c = crate::transformation::contributors(ib, page, r, d);
            Infobox::Transformation {
                requires: crate::transformation::requires(page),
                contributors: c,
                target: inline(ib, "target", r, d),
            }
        }
        InfoboxKind::Character => Infobox::Character {
            health: inline(ib, "health", r, d),
            damage: text(ib, "damage"),
            tears: text(ib, "tears"),
            range: text(ib, "range"),
            speed: text(ib, "speed"),
            luck: text(ib, "luck"),
            shot_speed: text(ib, "shot speed"),
            pickups: inline(ib, "pickups", r, d),
            collectibles: inline(ib, "collectibles", r, d),
            parent: r.by_page_title(param(ib, "parent")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Diagnostics, Infobox, Inline, Target};

    fn raw(name: &str, pairs: &[(&str, &str)]) -> RawInfobox {
        RawInfobox {
            name: name.into(),
            params: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn the_transformation_template_is_a_kind() {
        assert_eq!(
            InfoboxKind::of("infobox transformation"),
            Some(InfoboxKind::Transformation)
        );
    }

    /// Super Bum's real infobox, read on 2026-09-13: `id = n/a`, no `requirement`, and its
    /// item list written as positional arguments so `items` is absent altogether. It is the
    /// degradation case and it degrades to "no count, no contributors" — not to a panic, and
    /// not to the three a default would have produced.
    #[test]
    fn a_malformed_transformation_infobox_degrades_to_unknown() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox transformation",
            &[("dlc", "a"), ("target", "Isaac's bums")],
        );
        let Infobox::Transformation {
            requires,
            contributors,
            target,
        } = infobox_from(InfoboxKind::Transformation, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(requires, None);
        assert!(contributors.is_empty());
        assert!(
            matches!(target.first(), Some(Inline::Text { text, .. }) if text == "Isaac's bums")
        );
    }

    /// The count comes from the page body, which is why `infobox_from` takes the text: the
    /// transformation is the only kind whose infobox is completed from outside itself.
    #[test]
    fn a_transformation_reads_its_count_and_its_set_from_the_page() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw("infobox transformation", &[("items", "{{i|Breakfast}}")]);
        let text =
            "Pick up 3 [[item]]s from the following list.\n{{collectible table | Breakfast }}";
        let Infobox::Transformation {
            requires,
            contributors,
            ..
        } = infobox_from(InfoboxKind::Transformation, &ib, text, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(requires, Some(3));
        assert_eq!(contributors, vec![Target::Item { id: 25 }]);
    }

    #[test]
    fn extracts_many_infoboxes_from_a_storage_page() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Epic Fetus\n | description = Unlocked a new item.\n | requirements = Complete {{chal|The Family Man}}\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}}";
        let v = extract_infoboxes(src);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "infobox achievement");
        assert_eq!(v[0].params.get("id").map(String::as_str), Some("62"));
        assert_eq!(v[1].params.get("name").map(String::as_str), Some("Cain"));
    }

    /// B45. `{{infobox characters}}`, plural, holds **two** characters in one block: the
    /// second one's parameters carry a ` 2` suffix. Four pages use it — Jacob & Esau, The
    /// Forgotten, Tainted Forgotten, Tainted Lazarus — and they are exactly the four pages
    /// that carry two playable forms. Splitting here rather than downstream keeps
    /// `InfoboxKind` a closed set and leaves every reader (`entries_of`, `parent_check`)
    /// unchanged: what comes out is two ordinary character infoboxes.
    ///
    /// Tainted Lazarus's real block, trimmed to the parameters this test is about.
    #[test]
    fn the_plural_character_template_becomes_two_infoboxes() {
        let src = "{{infobox characters\n | dlc = r\n | name 2 = Dead Tainted Lazarus\n | id = 29\n | health = {{hearts|red=3}}\n | health 2 = {{hearts|soul=2}}\n | unlocked by = The Enigma\n | collectibles = {{i|Flip}}\n | collectibles 2 = [[Flip]]\n}}";
        let v = extract_infoboxes(src);
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(|ib| ib.name == "infobox character"));

        // The first form is the page's own: no `name`, so the entry takes the page title.
        assert_eq!(v[0].params.get("name"), None);
        assert_eq!(v[0].params.get("id").map(String::as_str), Some("29"));
        assert_eq!(
            v[0].params.get("health").map(String::as_str),
            Some("{{hearts|red=3}}")
        );

        // The second: its own parameters with the suffix gone, and none of the first's.
        assert_eq!(
            v[1].params.get("name").map(String::as_str),
            Some("Dead Tainted Lazarus")
        );
        assert_eq!(
            v[1].params.get("health").map(String::as_str),
            Some("{{hearts|soul=2}}")
        );
        assert_eq!(
            v[1].params.get("collectibles").map(String::as_str),
            Some("[[Flip]]")
        );
        // `id` has an `id 2` twin in the template and this page states none, so the second
        // form must not inherit 29: it resolves by name, and a name that fails to resolve
        // has to be loud (`pages_without_id`) rather than silently keep the first's id.
        assert_eq!(v[1].params.get("id"), None);

        // No ` 2` key survives on either side: they would reach `no_silent_parameter` as
        // parameters no field keeps.
        assert!(v
            .iter()
            .all(|ib| ib.params.keys().all(|k| !k.ends_with(" 2"))));
    }

    /// The four parameters the template reads without a ` 2` twin — `dlc`, `description`,
    /// `unlocked by`, `hidden` — are stated once for the page and belong to both forms.
    #[test]
    fn the_parameters_with_no_second_form_are_shared_by_both() {
        let src = "{{infobox characters\n | dlc = r\n | description = two of them\n | unlocked by = The Enigma\n | name 2 = Dead Tainted Lazarus\n}}";
        let v = extract_infoboxes(src);
        assert_eq!(v.len(), 2);
        for ib in &v {
            assert_eq!(ib.params.get("dlc").map(String::as_str), Some("r"));
            assert_eq!(
                ib.params.get("unlocked by").map(String::as_str),
                Some("The Enigma")
            );
            assert_eq!(
                ib.params.get("description").map(String::as_str),
                Some("two of them")
            );
        }
    }

    #[test]
    fn a_collectible_infobox_keeps_its_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        // Brimstone's real infobox, trimmed to the parameters this type holds.
        let ib = raw(
            "infobox passive collectible",
            &[
                ("quote", "Blood laser barrage"),
                ("quality", "4"),
                ("tags", "devil summonable offensive"),
                ("devil price", "2"),
            ],
        );
        let Infobox::Item {
            quote,
            template,
            quality,
            tags,
            recharge,
            devil_price,
            shop_price,
            pools,
        } = infobox_from(InfoboxKind::Passive, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert!(matches!(
            quote.first(),
            Some(Inline::Text { text, .. }) if text == "Blood laser barrage"
        ));
        assert_eq!(template, CollectibleTemplate::Passive);
        assert_eq!(quality, Some(4));
        assert_eq!(tags, vec!["devil", "summonable", "offensive"]);
        assert!(recharge.is_empty());
        assert!(shop_price.is_empty());
        assert!(pools.is_empty());
        // A price is inline, not a number: 36 of the 56 real values are `{{dlcalt|…}}`.
        assert!(matches!(
            devil_price.first(),
            Some(Inline::Text { text, .. }) if text.trim() == "2"
        ));
    }

    #[test]
    fn an_activated_collectible_is_marked_activated() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw("infobox activated collectible", &[("recharge", "6")]);
        let Infobox::Item {
            template, recharge, ..
        } = infobox_from(InfoboxKind::Activated, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(template, CollectibleTemplate::Activated);
        assert!(!recharge.is_empty());
    }

    #[test]
    fn a_trinket_infobox_keeps_its_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox trinket",
            &[("quote", "Imaginary Friend"), ("tags", "offensive")],
        );
        let Infobox::Trinket { quote, tags, pools } =
            infobox_from(InfoboxKind::Trinket, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert!(matches!(
            quote.first(),
            Some(Inline::Text { text, .. }) if text == "Imaginary Friend"
        ));
        assert_eq!(tags, vec!["offensive"]);
        assert!(pools.is_empty());
    }

    #[test]
    fn kinds() {
        assert_eq!(
            InfoboxKind::of("infobox passive collectible"),
            Some(InfoboxKind::Passive)
        );
        assert_eq!(
            InfoboxKind::of("infobox activated collectible"),
            Some(InfoboxKind::Activated)
        );
        assert_eq!(
            InfoboxKind::of("infobox trinket"),
            Some(InfoboxKind::Trinket)
        );
        assert_eq!(InfoboxKind::of("infobox boss"), Some(InfoboxKind::Boss));
        assert_eq!(InfoboxKind::of("infobox"), None);
    }

    #[test]
    fn achievement_and_boss() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox achievement",
            &[
                ("description", "Unlocked a new item."),
                ("requirements", "Complete {{chal|The Family Man}}"),
                ("link", "Breakfast"),
            ],
        );
        let Infobox::Achievement {
            quote: _,
            requirements,
            notes,
            unlocks,
        } = infobox_from(InfoboxKind::Achievement, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        // `description` is no longer here: it rose to `Entry`, and `entry_facts` reads it.
        // An achievement's is plain text, so it arrives as a single `Inline::Text`.
        let facts = entry_facts(&ib, &r, &mut d);
        assert!(matches!(
            facts.description.first(),
            Some(Inline::Text { text, .. }) if text == "Unlocked a new item."
        ));
        assert!(requirements.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Challenge { number: 19 },
                ..
            }
        )));
        assert!(notes.is_empty(), "this fixture declares no notes");
        assert_eq!(unlocks, Some(Target::Item { id: 25 }));

        let ib = raw(
            "infobox boss",
            &[("base hp", "250 (x2)"), ("unlocked by", "Epic Fetus")],
        );
        let Infobox::Boss { base_hp, .. } = infobox_from(InfoboxKind::Boss, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(base_hp, Some(250));
        // Same move: the boss's `unlocked by` is now one of the three common facts.
        assert_eq!(
            entry_facts(&ib, &r, &mut d).unlocked_by,
            Some(Target::Achievement { id: 62 })
        );
    }

    /// An achievement's `description` is the line on the game's unlock paper — "Just Stop!"
    /// on 1000000%, "OMG!" on !Platinum God! — and not a description: it is the quote, the
    /// game's own voice, the way an item's pickup line is. 136 read "???", one "..." and one
    /// is blank, and those are no line at all, so they are no quote rather than a quote that
    /// says nothing.
    #[test]
    fn an_achievements_paper_line_is_its_quote_and_a_placeholder_is_none() {
        let r = test_resolver();
        let quote = |description: &str| {
            let ib = raw("infobox achievement", &[("description", description)]);
            let Infobox::Achievement { quote, .. } = infobox_from(
                InfoboxKind::Achievement,
                &ib,
                "",
                &r,
                &mut Diagnostics::default(),
            ) else {
                panic!()
            };
            crate::plain(&quote)
        };
        assert_eq!(quote("Just Stop!"), "Just Stop!");
        assert_eq!(quote("Unlocked..."), "Unlocked...");
        assert_eq!(quote("???"), "");
        assert_eq!(quote(" ... "), "");
        assert_eq!(quote(""), "");
    }

    #[test]
    fn entry_facts_reads_the_three_common_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox passive collectible",
            &[
                ("description", "Tears are replaced with {{i|Breakfast}}"),
                ("dlc", "a+nr"),
                ("unlocked by", "Epic Fetus"),
            ],
        );
        let facts = entry_facts(&ib, &r, &mut d);
        // The description is wikitext, not a string: its links have to survive the move.
        assert!(facts.description.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Item { id: 25 },
                ..
            }
        )));
        assert_eq!(facts.dlc, vec![Dlc::AfterbirthPlus]);
        assert_eq!(facts.unlocked_by, Some(Target::Achievement { id: 62 }));

        // An infobox that declares none of the three degrades to empty, never to an error.
        let bare = raw("infobox trinket", &[("id", "1")]);
        let facts = entry_facts(&bare, &r, &mut d);
        assert!(facts.description.is_empty());
        assert!(facts.dlc.is_empty());
        assert_eq!(facts.unlocked_by, None);
    }

    /// The parameter goes to `{{section dlc}}` → `{{page dlc}}` → `{{dlcset}}`, the same
    /// switch the inline `{{dlc|…}}` uses, so it names a **range** and not a set: `r` is
    /// "added in Repentance", which is Repentance and Repentance+.
    ///
    /// Until 2026-09-15 it was split one code at a time, and every value but `r+` came out
    /// too narrow — 1078 of the 1083 in the snapshot: `r` lost Repentance+ on 531 pages,
    /// `a+` lost three on 292, `a` lost four on 254. The one that was not merely narrow is
    /// Tonsil, whose `a+nr` read as Afterbirth † **plus Rebirth plus Repentance**, because
    /// the `n` in the middle was taken for a code of its own.
    #[test]
    fn the_infobox_dlc_parameter_names_a_range_of_editions() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let mut dlc =
            |code: &str| entry_facts(&raw("infobox trinket", &[("dlc", code)]), &r, &mut d).dlc;

        assert_eq!(dlc("r"), vec![Dlc::Repentance, Dlc::RepentancePlus]);
        assert_eq!(dlc("r+"), vec![Dlc::RepentancePlus]);
        assert_eq!(
            dlc("a"),
            vec![
                Dlc::Afterbirth,
                Dlc::AfterbirthPlus,
                Dlc::Repentance,
                Dlc::RepentancePlus
            ]
        );
        assert_eq!(dlc("a+nr"), vec![Dlc::AfterbirthPlus]);
        assert!(d.unknown_dlc_codes.is_empty());
    }

    /// An absent parameter declares nothing and stays empty; `n` declares **every**
    /// edition, which is a statement and not an absence, so it lists all five. Reading it
    /// as Rebirth — the one edition `n` rules nothing about — said the opposite.
    ///
    /// A parameter outside the switch is counted and the entry is left declaring nothing,
    /// because the wiki's own answer there is `0 <!-- invalid string! -->` and a guess
    /// would be worse than a gap that `meta` reports.
    #[test]
    fn an_absent_dlc_parameter_declares_nothing_and_the_bare_n_declares_everything() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let dlc = |code: &str, d: &mut Diagnostics| {
            entry_facts(&raw("infobox trinket", &[("dlc", code)]), &r, d).dlc
        };

        assert!(dlc("", &mut d).is_empty());
        assert_eq!(dlc("n", &mut d).len(), 5);
        assert!(d.unknown_dlc_codes.is_empty());

        assert!(dlc("zz", &mut d).is_empty());
        assert_eq!(d.unknown_dlc_codes.get("zz"), Some(&1));
    }

    #[test]
    fn the_four_parsed_kinds_keep_the_parameters_they_used_to_drop() {
        let r = test_resolver();
        let mut d = Diagnostics::default();

        // A boss's variant and its per-stage hp: 26 and 2 real pages carry them.
        let ib = raw(
            "infobox boss",
            &[
                ("base hp", "250 (x2)"),
                ("variant", "1"),
                ("stage hp", "300"),
            ],
        );
        let Infobox::Boss {
            variant, stage_hp, ..
        } = infobox_from(InfoboxKind::Boss, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(variant, Some(1));
        assert!(!stage_hp.is_empty());

        // The character a challenge is played as: 14 real pages say it, and until
        // 2026-09-13 the field did not exist, so none of them reached the frontend.
        let ib = raw("infobox challenge", &[("character", "Isaac")]);
        let Infobox::Challenge { character, .. } =
            infobox_from(InfoboxKind::Challenge, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(character, Some(Target::Character { id: 0 }));

        let ib = raw(
            "infobox character",
            &[("tears", "2.73"), ("parent", "Tainted Isaac")],
        );
        let Infobox::Character { tears, parent, .. } =
            infobox_from(InfoboxKind::Character, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(tears, "2.73");
        assert_eq!(parent, Some(Target::Character { id: 21 }));
    }

    #[test]
    fn challenge_flags() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox challenge",
            &[
                ("blindfolded", "yes"),
                ("has shops", "no"),
                ("unlocks", "Epic Fetus"),
            ],
        );
        let Infobox::Challenge {
            blindfolded,
            has_shops,
            has_treasure_rooms,
            unlocks,
            ..
        } = infobox_from(InfoboxKind::Challenge, &ib, "", &r, &mut d)
        else {
            panic!()
        };
        assert!(blindfolded);
        assert!(!has_shops);
        assert!(!has_treasure_rooms);
        assert_eq!(unlocks, Some(Target::Achievement { id: 62 }));
    }
}
