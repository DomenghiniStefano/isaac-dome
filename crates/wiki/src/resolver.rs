//! From a name (alias or page title) to a `Target`, using the same Cargo tables the wiki's
//! templates use to resolve links. `corrections.json` wins over the table.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::editions::Editions;
use crate::{Dlc, Target};

/// True if the row is valid in the current edition, Repentance+. The `dlc` field of the Cargo
/// tables is a bitmask, read by [`Editions`] like every other statement of an edition. A row
/// with no `dlc`, or with a `dlc` that isn't an integer, is kept: the filter only excludes
/// what the wiki declares to belong to another edition (Afterbirth+'s collectible 474
/// "Tonsil", `dlc = 4`).
pub fn in_current_edition(row: &Row) -> bool {
    match row.get("dlc").and_then(|s| s.trim().parse::<u32>().ok()) {
        Some(mask) => Editions::of_cargo_bits(mask).contains(Dlc::RepentancePlus),
        None => true,
    }
}

/// A cargoquery row, with the keys as they come from the wiki (`_pageName`, `id`,
/// `alias`, `name`, `number`, `variant`, `subtype`, `type`).
pub type Row = BTreeMap<String, String>;

/// The downloaded Cargo tables, one per reference kind.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tables {
    pub collectible: Vec<Row>,
    pub trinket: Vec<Row>,
    pub achievement: Vec<Row>,
    pub entity: Vec<Row>,
    pub challenge: Vec<Row>,
    /// Not used to build the character map (`Resolver.characters` reads pages and
    /// `corrections.json`, because this table's own `id` column is exactly as unreliable as
    /// the infoboxes' — B42, measured 2026-09-14). Read for one field only: `parent`, a
    /// second, independent statement of the relation `Infobox::Character.parent` reads from
    /// the page — see `parent_check`.
    pub player: Vec<Row>,
    pub transformation: Vec<Row>,
    pub pickup: Vec<Row>,
}

/// `corrections.json`: per table, page title → the right id when the wiki gets it wrong.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Corrections {
    #[serde(default)]
    pub page_id: BTreeMap<String, BTreeMap<String, u32>>,
    /// Name → id from `players.xml`: the character map is our own, because the wiki
    /// infoboxes' ids are unreliable (Isaac 14, Magdalene 2) and four pages use a
    /// different template. Wins over the ids derived from the pages.
    #[serde(default)]
    pub characters: BTreeMap<String, u32>,
    /// Descriptions written by hand, as wikitext: collection, named the way `wiki.json`
    /// names it (`achievements`, `bosses`…), then the entry's key in that collection. One
    /// wins over the page's, on any kind — the reason to write one is that the wiki's is
    /// missing (Dead God) or says nothing.
    #[serde(default)]
    pub descriptions: BTreeMap<String, BTreeMap<String, String>>,
}

/// The tables [`Corrections::apply`] is ever called with. A `page_id` entry filed under
/// any other name is a correction that can never fire, and `corrections.json` is written
/// by hand: a typo there produces no error, no warning and no effect. Kept next to
/// `apply` because it is the list of its call sites.
pub const CORRECTED_TABLES: [&str; 2] = ["collectible", "trinket"];

impl Corrections {
    /// The `page_id` tables no lookup will ever ask about. Empty is the healthy answer;
    /// anything else is a correction sitting in the file doing nothing.
    pub fn unknown_tables(&self) -> Vec<&str> {
        self.page_id
            .keys()
            .map(String::as_str)
            .filter(|t| !CORRECTED_TABLES.contains(t))
            .collect()
    }

    /// The hand-written descriptions that name no entry of `ds` — an unknown collection or
    /// a key it does not hold — in file order. Empty is the healthy answer: anything else
    /// is a line in `corrections.json` that is doing nothing, and saying nothing about it.
    pub fn unmatched_descriptions(&self, ds: &crate::Dataset) -> Vec<(String, String)> {
        self.descriptions
            .iter()
            .flat_map(|(collection, entries)| {
                entries
                    .keys()
                    .filter(|key| !ds.has_key(collection, key))
                    .map(|key| (collection.clone(), key.clone()))
            })
            .collect()
    }

    /// The corrected id for page `title` of table `table`; `id` if there's no correction.
    pub fn apply(&self, table: &str, title: &str, id: u32) -> u32 {
        self.page_id
            .get(table)
            .and_then(|m| m.get(title))
            .copied()
            .unwrap_or(id)
    }
}

/// The outcome of `Resolver::resolve`: `Ignore` for layout templates, `Unknown` for a
/// template that is neither a link nor a layout one, `Unresolved` when the template is a
/// link but the name isn't found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Target(Target),
    /// A wiki page the game gives no id, and never would: machines, beggars, item pools.
    /// Distinct from `Unresolved`, which means "we looked for an id and did not find one" —
    /// a failure worth a diagnostic. This one is the expected answer, so it gets none.
    Concept,
    Unresolved,
    Ignore,
    Unknown,
}

/// The maps by normalized key. The `*_by_title` ones only hold page titles, for
/// `by_page_title` and for the infobox checks; a single page can carry more than one id
/// (Broken Shovel: 550 and 551). The other maps also take in aliases, for inline
/// templates. Rows from past editions don't enter them (`in_current_edition`).
#[derive(Debug, Default)]
pub struct Resolver {
    corrections: Corrections,
    items: BTreeMap<String, u32>,
    items_by_title: BTreeMap<String, BTreeSet<u32>>,
    trinkets: BTreeMap<String, u32>,
    trinkets_by_title: BTreeMap<String, BTreeSet<u32>>,
    achievements: BTreeMap<String, u32>,
    challenges: BTreeMap<String, u32>,
    challenges_by_title: BTreeMap<String, u32>,
    entities: BTreeMap<String, (u32, u32, u32)>,
    bosses_by_title: BTreeMap<String, (u32, u32, u32)>,
    transformations: BTreeMap<String, u32>,
    pickups: BTreeMap<String, String>,
    characters: BTreeMap<String, u32>,
    /// Key → name as written in our own map, for the entries' title.
    character_names: BTreeMap<String, String>,
    /// Key (a form's own name, `player`'s `alias` column) → its `parent` field, raw. Empty
    /// string is a row that states no parent, distinct from the key being absent (no row at
    /// all — see `player_table_parent`).
    player_parent: BTreeMap<String, String>,
}

/// A character page's title without the disambiguation suffix
/// (`??? (Character)` → `???`).
fn character_page_name(title: &str) -> &str {
    title
        .trim()
        .strip_suffix(" (Character)")
        .unwrap_or(title.trim())
}

/// A comparison key: lowercased, whitespace collapsed, `&` → `and`, trimmed.
pub fn key(s: &str) -> String {
    let lower = s.trim().to_lowercase().replace('&', " and ");
    lower.split_whitespace().collect::<Vec<_>>().join(" ")
}

// `column list` isn't here: on the wiki it can carry content in a positional argument
// (our corpus only uses the named `content=`, but the template allows it), so it must be
// treated as unknown to make it recurse instead of losing it.
const LAYOUT: &[&str] = &[
    "cit",
    "nav",
    "#ev:youtube",
    "disambig msg",
    "header characters",
    // The tainted pages' own header, and nothing but the header: it went uncounted while
    // the preamble it sits in was thrown away unread.
    "header tainted characters",
    "storage page",
    "unlockable",
    "header transformations",
    // The head of the two-part item table: `{{Collectible table/header}}` draws the row of
    // column titles and `{{collectible rows|…}}` carries the names. Layout and not unknown,
    // because an unknown template recurses into its argument and these have none to give.
    "collectible table/header",
    "trinket table/header",
    "reflist",
    "clear",
    "main",
    "hatnote",
    "see also",
    "distinguish",
    "distinguish visual",
    "toc",
    "__toc__",
];

/// Templates that only do layout: they carry no reference.
pub fn is_layout_template(name: &str) -> bool {
    LAYOUT.contains(&name)
}

fn num(row: &Row, field: &str) -> Option<u32> {
    row.get(field)?.trim().parse().ok()
}

/// The rows of a table that are valid in the current edition.
fn current(rows: &[Row]) -> impl Iterator<Item = &Row> {
    rows.iter().filter(|row| in_current_edition(row))
}

fn get<'a>(row: &'a Row, field: &str) -> Option<&'a str> {
    row.get(field)
        .map(|s| s.as_str())
        .filter(|s| !s.trim().is_empty())
}

impl Resolver {
    pub fn new(
        tables: &Tables,
        characters: &BTreeMap<String, u32>,
        corrections: &Corrections,
    ) -> Resolver {
        let mut r = Resolver {
            corrections: corrections.clone(),
            ..Resolver::default()
        };
        for row in current(&tables.collectible) {
            let (Some(title), Some(id)) = (get(row, "_pageName"), num(row, "id")) else {
                continue;
            };
            let id = corrections.apply("collectible", title, id);
            r.items_by_title.entry(key(title)).or_default().insert(id);
            r.items.entry(key(title)).or_insert(id);
            if let Some(a) = get(row, "alias") {
                r.items.insert(key(a), id);
            }
        }
        for row in current(&tables.trinket) {
            let (Some(title), Some(id)) = (get(row, "_pageName"), num(row, "id")) else {
                continue;
            };
            let id = corrections.apply("trinket", title, id);
            r.trinkets_by_title
                .entry(key(title))
                .or_default()
                .insert(id);
            r.trinkets.entry(key(title)).or_insert(id);
            if let Some(a) = get(row, "alias") {
                r.trinkets.insert(key(a), id);
            }
        }
        for row in current(&tables.achievement) {
            let Some(id) = num(row, "id") else {
                continue;
            };
            if let Some(n) = get(row, "name") {
                r.achievements.insert(key(n), id);
            }
            if let Some(a) = get(row, "alias") {
                r.achievements.entry(key(a)).or_insert(id);
            }
        }
        for row in current(&tables.challenge) {
            let Some(n) = num(row, "number") else {
                continue;
            };
            if let Some(t) = get(row, "_pageName") {
                r.challenges_by_title.insert(key(t), n);
                r.challenges.entry(key(t)).or_insert(n);
            }
            if let Some(a) = get(row, "alias") {
                r.challenges.insert(key(a), n);
            }
            r.challenges.insert(n.to_string(), n);
        }
        for row in current(&tables.entity) {
            let (Some(id), Some(v), Some(s)) =
                (num(row, "id"), num(row, "variant"), num(row, "subtype"))
            else {
                continue;
            };
            let title = get(row, "_pageName");
            if let Some(t) = title {
                r.entities.entry(key(t)).or_insert((id, v, s));
            }
            if let Some(a) = get(row, "alias") {
                r.entities.insert(key(a), (id, v, s));
            }
            if get(row, "type") == Some("boss") {
                if let Some(t) = title {
                    r.bosses_by_title.entry(key(t)).or_insert((id, v, s));
                }
            }
        }
        for row in current(&tables.transformation) {
            let Some(id) = num(row, "id") else {
                continue;
            };
            if let Some(t) = get(row, "_pageName") {
                r.transformations.entry(key(t)).or_insert(id);
            }
            if let Some(a) = get(row, "alias") {
                r.transformations.insert(key(a), id);
            }
        }
        for row in current(&tables.pickup) {
            if let Some(a) = get(row, "alias") {
                r.pickups.insert(key(a), a.to_string());
            }
        }
        for row in current(&tables.player) {
            if let Some(a) = get(row, "alias") {
                r.player_parent
                    .entry(key(a))
                    .or_insert_with(|| get(row, "parent").unwrap_or("").to_string());
            }
        }
        for (name, id) in characters {
            r.characters.insert(key(name), *id);
        }
        for (name, id) in &corrections.characters {
            r.characters.insert(key(name), *id);
            r.character_names.insert(key(name), name.clone());
        }
        r
    }

    /// The character of a page or an infobox: id and canonical name (the one from our own
    /// map, if there is one; otherwise the title without « (Character)»).
    pub fn character_of_page(&self, title: &str) -> Option<(u32, String)> {
        let name = character_page_name(title);
        let k = key(name);
        let id = *self.characters.get(&k)?;
        let canonical = self
            .character_names
            .get(&k)
            .cloned()
            .unwrap_or_else(|| name.to_string());
        Some((id, canonical))
    }

    /// `template` already lowercase; `arg` the first raw argument.
    pub fn resolve(&self, template: &str, arg: &str) -> Resolution {
        let t = template.trim().to_lowercase();
        if is_layout_template(&t) {
            return Resolution::Ignore;
        }
        let k = key(arg);
        let found = match t.as_str() {
            "i" => self.items.get(&k).map(|id| Target::Item { id: *id }),
            "t" => self.trinkets.get(&k).map(|id| Target::Trinket { id: *id }),
            "a" | "achievement" => self
                .achievements
                .get(&k)
                .map(|id| Target::Achievement { id: *id }),
            "chal" => self
                .challenges
                .get(&k)
                .map(|n| Target::Challenge { number: *n }),
            "e" => self
                .entities
                .get(&k)
                .map(|(id, variant, subtype)| Target::Entity {
                    id: *id,
                    variant: *variant,
                    subtype: *subtype,
                }),
            // `{{transformation contribution|X}}` reads as a sentence on the page ("counts
            // toward X"), but the only part of it with an identity is the transformation,
            // which is exactly what `{{tf|X}}` names.
            "tf" | "transformation contribution" => self
                .transformations
                .get(&k)
                .map(|id| Target::Transformation { id: *id }),
            "p" => self
                .pickups
                .get(&k)
                .map(|n| Target::Concept { name: n.clone() }),
            "c" => self
                .characters
                .get(&k)
                .map(|id| Target::Character { id: *id }),
            // Machines and beggars: the game has no id for them, so they are wiki concepts
            // rather than targets. `m` is the largest single entry `unknownTemplates` had.
            "m" | "machine" => return Resolution::Concept,
            // An item pool. `itempools.xml` keys pools by name and gives them no id, so
            // there is no target to resolve to — the same shape as a machine.
            "ip" => return Resolution::Concept,
            "s" | "floor" => {
                return Resolution::Target(Target::Stage {
                    name: arg.trim().to_string(),
                })
            }
            "r" | "room" => {
                return Resolution::Target(Target::Room {
                    name: arg.trim().to_string(),
                })
            }
            _ => return Resolution::Unknown, // allowed: template name, an open-ended string
        };
        match found {
            Some(t) => Resolution::Target(t),
            None => Resolution::Unresolved,
        }
    }

    /// Page title → target, for the infoboxes' `link`/`unlocks`/`unlocked by`.
    /// Precedence: character, item, trinket, challenge, entity.
    pub fn by_page_title(&self, title: &str) -> Option<Target> {
        let k = key(title);
        if let Some(id) = self.characters.get(&k) {
            return Some(Target::Character { id: *id });
        }
        if let Some(id) = self.items_by_title.get(&k).and_then(|ids| ids.first()) {
            return Some(Target::Item { id: *id });
        }
        if let Some(id) = self.trinkets_by_title.get(&k).and_then(|ids| ids.first()) {
            return Some(Target::Trinket { id: *id });
        }
        if let Some(n) = self.challenges_by_title.get(&k) {
            return Some(Target::Challenge { number: *n });
        }
        if let Some((id, variant, subtype)) = self.entities.get(&k) {
            return Some(Target::Entity {
                id: *id,
                variant: *variant,
                subtype: *subtype,
            });
        }
        None
    }

    /// The `player` Cargo table's own `parent` for `name` (its `alias` column), resolved
    /// the same way an infobox's `parent` parameter is. `None` means the table has no row
    /// for this name at all — not a disagreement, since there is nothing to compare against
    /// (see `parent_check`) — distinct from `Some(None)`, a row that states no parent.
    #[cfg(feature = "test-api")]
    pub(crate) fn player_table_parent(&self, name: &str) -> Option<Option<Target>> {
        let raw = self.player_parent.get(&key(name))?;
        Some(self.by_page_title(raw))
    }

    /// The id page `title` enters the items with, if the table (already filtered by
    /// edition) knows it with the id the infobox declares, corrected like the table.
    /// `None` for an infobox from another edition (Tonsil's collectible 474).
    pub fn item_id_of_page(&self, title: &str, declared: u32) -> Option<u32> {
        let id = self.corrections.apply("collectible", title, declared);
        self.items_by_title
            .get(&key(title))
            .filter(|ids| ids.contains(&id))
            .map(|_| id)
    }

    /// Like `item_id_of_page`, for trinkets.
    pub fn trinket_id_of_page(&self, title: &str, declared: u32) -> Option<u32> {
        let id = self.corrections.apply("trinket", title, declared);
        self.trinkets_by_title
            .get(&key(title))
            .filter(|ids| ids.contains(&id))
            .map(|_| id)
    }

    pub fn achievement_by_name(&self, name: &str) -> Option<Target> {
        self.achievements
            .get(&key(name))
            .map(|id| Target::Achievement { id: *id })
    }

    /// Only entities with `type = boss`: the bestiary key for a boss's page.
    pub fn boss_key(&self, page_title: &str) -> Option<(u32, u32, u32)> {
        self.bosses_by_title.get(&key(page_title)).copied()
    }

    /// A transformation's id from its page title. The Cargo table is the only source that
    /// has one for every page: Super Bum's infobox says `id = n/a`, and the table maps that
    /// onto 1000 — a sentinel for "no id in the game", not a `PlayerForm` beside 0…15.
    /// Without this the page would be dropped as having no id.
    pub fn transformation_of_page(&self, page_title: &str) -> Option<u32> {
        self.transformations.get(&key(page_title)).copied()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    fn row(pairs: &[(&str, &str)]) -> Row {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// A resolver over minimal tables, shared by the crate's tests.
    pub(crate) fn test_resolver() -> Resolver {
        let tables = Tables {
            collectible: vec![
                row(&[
                    ("_pageName", "Breakfast"),
                    ("id", "25"),
                    ("alias", "Breakfast"),
                ]),
                // Named by `{{Book of Virtues synergy}}`, which resolves it by hand.
                row(&[
                    ("_pageName", "Book of Virtues"),
                    ("id", "584"),
                    ("alias", "Book of Virtues"),
                ]),
                // Its twin, and the page's name carries the article: the arm that resolved
                // `Book of Belial` found nothing for 33 uses.
                row(&[
                    ("_pageName", "The Book of Belial"),
                    ("id", "34"),
                    ("alias", "The Book of Belial"),
                ]),
                // Afterbirth+'s collectible 474: no longer exists in Repentance+.
                row(&[
                    ("_pageName", "Tonsil"),
                    ("id", "474"),
                    ("dlc", "4"),
                    ("alias", "Tonsil"),
                ]),
                row(&[
                    ("_pageName", "Broken Glass Cannon"),
                    ("id", "474"),
                    ("dlc", "24"),
                    ("alias", "Broken Glass Cannon"),
                ]),
                // Two entries on the same page, like Broken Shovel (550 and 551).
                row(&[
                    ("_pageName", "Broken Shovel"),
                    ("id", "550"),
                    ("dlc", "28"),
                    ("alias", "Broken Shovel"),
                ]),
                row(&[
                    ("_pageName", "Broken Shovel"),
                    ("id", "551"),
                    ("dlc", "28"),
                    ("alias", "Broken Shovel 2"),
                ]),
                // A page with the wrong id, corrected by `corrections.json`.
                row(&[
                    ("_pageName", "Misfiled"),
                    ("id", "900"),
                    ("alias", "Misfiled"),
                ]),
                row(&[
                    ("_pageName", "Jacob & Esau"),
                    ("id", "0"),
                    ("alias", "Jacob and Esau"),
                ]),
            ],
            trinket: vec![
                row(&[
                    ("_pageName", "Swallowed Penny"),
                    ("id", "1"),
                    ("alias", "Swallowed Penny"),
                ]),
                row(&[
                    ("_pageName", "Tonsil"),
                    ("id", "97"),
                    ("dlc", "28"),
                    ("alias", "Tonsil"),
                ]),
            ],
            achievement: vec![
                row(&[
                    ("_pageName", "Achievements/Rebirth 1"),
                    ("id", "62"),
                    ("name", "Epic Fetus"),
                    ("alias", "Epic Fetus"),
                ]),
                // A second one so a test about a *list* of achievements can have a list.
                row(&[
                    ("_pageName", "Achievements/Rebirth 1"),
                    ("id", "2"),
                    ("name", "Cain"),
                    ("alias", "Cain"),
                ]),
            ],
            entity: vec![
                row(&[
                    ("_pageName", "Mom"),
                    ("id", "45"),
                    ("variant", "0"),
                    ("subtype", "0"),
                    ("type", "boss"),
                    ("alias", "Mom"),
                ]),
                row(&[
                    ("_pageName", "Angel"),
                    ("id", "271"),
                    ("variant", "0"),
                    ("subtype", "0"),
                    ("type", "mini-boss"),
                    ("alias", "Uriel"),
                ]),
            ],
            challenge: vec![row(&[
                ("_pageName", "The Family Man"),
                ("number", "19"),
                ("alias", "The Family Man"),
            ])],
            player: vec![],
            transformation: vec![row(&[
                ("_pageName", "Beelzebub"),
                ("id", "1"),
                ("alias", "Beelzebub"),
            ])],
            pickup: vec![row(&[("_pageName", "Cards"), ("alias", "The Fool")])],
        };
        let mut chars = BTreeMap::new();
        chars.insert("Tainted Isaac".to_string(), 21);
        chars.insert("Jacob & Esau".to_string(), 19);
        // The wiki gives Isaac the id 14 (Keeper's own): our own map wins.
        chars.insert("Isaac".to_string(), 14);
        let corrections: Corrections = serde_json::from_str(
            r#"{"pageId":{"collectible":{"Misfiled":901}},
                "characters":{"Isaac":0,"Jacob & Esau":19,"???":4}}"#,
        )
        .unwrap();
        Resolver::new(&tables, &chars, &corrections)
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::test_resolver;
    use super::*;
    use crate::Target;

    #[test]
    fn key_normalizes() {
        assert_eq!(key("  Jacob   &  Esau "), "jacob and esau");
        assert_eq!(key("Tainted ???"), "tainted ???");
    }

    #[test]
    fn items_by_alias_and_page_name_with_correction() {
        let r = test_resolver();
        assert_eq!(
            r.resolve("i", "Breakfast"),
            Resolution::Target(Target::Item { id: 25 })
        );
        assert_eq!(
            r.resolve("I", "breakfast"),
            Resolution::Target(Target::Item { id: 25 })
        );
        assert_eq!(
            r.resolve("i", "Misfiled"),
            Resolution::Target(Target::Item { id: 901 })
        );
        assert_eq!(
            r.resolve("i", "Jacob and Esau"),
            Resolution::Target(Target::Item { id: 0 })
        );
        assert_eq!(r.resolve("i", "Nope"), Resolution::Unresolved);
    }

    #[test]
    fn edition_filter_reads_the_dlc_bitmask() {
        let row = |dlc: Option<&str>| -> Row {
            let mut r = Row::new();
            r.insert("id".into(), "1".into());
            if let Some(d) = dlc {
                r.insert("dlc".into(), d.into());
            }
            r
        };
        assert!(!in_current_edition(&row(Some("4"))));
        assert!(!in_current_edition(&row(Some("15"))));
        assert!(in_current_edition(&row(Some("16"))));
        assert!(in_current_edition(&row(Some("24"))));
        assert!(in_current_edition(&row(Some("31"))));
        assert!(in_current_edition(&row(Some(" 28 "))));
        assert!(in_current_edition(&row(None)));
        assert!(in_current_edition(&row(Some(""))));
        assert!(in_current_edition(&row(Some("x"))));
    }

    #[test]
    fn rows_of_past_editions_are_not_in_the_resolver() {
        let r = test_resolver();
        // Tonsil's collectible row (Afterbirth+) is excluded: the name doesn't resolve as
        // an item, and for items the page doesn't exist.
        assert_eq!(r.resolve("i", "Tonsil"), Resolution::Unresolved);
        assert_eq!(
            r.resolve("t", "Tonsil"),
            Resolution::Target(Target::Trinket { id: 97 })
        );
        assert_eq!(
            r.resolve("i", "Broken Glass Cannon"),
            Resolution::Target(Target::Item { id: 474 })
        );
        assert_eq!(r.by_page_title("Tonsil"), Some(Target::Trinket { id: 97 }));
    }

    #[test]
    fn page_ids_are_checked_per_type_with_corrections_applied() {
        let r = test_resolver();
        assert_eq!(r.item_id_of_page("Breakfast", 25), Some(25));
        assert_eq!(r.item_id_of_page("Breakfast", 26), None);
        assert_eq!(r.item_id_of_page("Tonsil", 474), None);
        assert_eq!(r.trinket_id_of_page("Tonsil", 97), Some(97));
        assert_eq!(r.trinket_id_of_page("Breakfast", 25), None);
        // A page with two entries knows both of them.
        assert_eq!(r.item_id_of_page("Broken Shovel", 550), Some(550));
        assert_eq!(r.item_id_of_page("Broken Shovel", 551), Some(551));
        assert_eq!(
            r.by_page_title("Broken Shovel"),
            Some(Target::Item { id: 550 })
        );
        // The infobox declares the wrong id: the correction, by title, brings it to the
        // table's id no matter what id was declared.
        assert_eq!(r.item_id_of_page("Misfiled", 900), Some(901));
        assert_eq!(r.item_id_of_page("Misfiled", 12345), Some(901));
    }

    #[test]
    fn our_character_map_wins_over_the_infobox_ids() {
        let r = test_resolver();
        assert_eq!(
            r.resolve("c", "Isaac"),
            Resolution::Target(Target::Character { id: 0 })
        );
        assert_eq!(
            r.resolve("c", "jacob and esau"),
            Resolution::Target(Target::Character { id: 19 })
        );
        assert_eq!(
            r.resolve("c", "???"),
            Resolution::Target(Target::Character { id: 4 })
        );
        // From the page only for the infobox: it's still valid as a fallback.
        assert_eq!(
            r.resolve("c", "Tainted Isaac"),
            Resolution::Target(Target::Character { id: 21 })
        );
        assert_eq!(r.by_page_title("Isaac"), Some(Target::Character { id: 0 }));
        // Page title: the « (Character)» suffix doesn't count; the name is the one from the map.
        assert_eq!(
            r.character_of_page("??? (Character)"),
            Some((4, "???".to_string()))
        );
        assert_eq!(r.character_of_page("Isaac"), Some((0, "Isaac".to_string())));
        assert_eq!(
            r.character_of_page("Tainted Isaac"),
            Some((21, "Tainted Isaac".to_string()))
        );
        assert_eq!(r.character_of_page("Nope"), None);
    }

    #[test]
    fn other_link_templates() {
        let r = test_resolver();
        assert_eq!(
            r.resolve("t", "Swallowed Penny"),
            Resolution::Target(Target::Trinket { id: 1 })
        );
        assert_eq!(
            r.resolve("a", "Epic Fetus"),
            Resolution::Target(Target::Achievement { id: 62 })
        );
        assert_eq!(
            r.resolve("chal", "The Family Man"),
            Resolution::Target(Target::Challenge { number: 19 })
        );
        assert_eq!(
            r.resolve("chal", "19"),
            Resolution::Target(Target::Challenge { number: 19 })
        );
        assert_eq!(
            r.resolve("e", "Mom"),
            Resolution::Target(Target::Entity {
                id: 45,
                variant: 0,
                subtype: 0
            })
        );
        assert_eq!(
            r.resolve("e", "Uriel"),
            Resolution::Target(Target::Entity {
                id: 271,
                variant: 0,
                subtype: 0
            })
        );
        assert_eq!(
            r.resolve("tf", "Beelzebub"),
            Resolution::Target(Target::Transformation { id: 1 })
        );
        assert_eq!(
            r.resolve("p", "The Fool"),
            Resolution::Target(Target::Concept {
                name: "The Fool".into()
            })
        );
        assert_eq!(
            r.resolve("c", "Tainted Isaac"),
            Resolution::Target(Target::Character { id: 21 })
        );
        assert_eq!(
            r.resolve("c", "Jacob and Esau"),
            Resolution::Target(Target::Character { id: 19 })
        );
        assert_eq!(
            r.resolve("s", "Depths"),
            Resolution::Target(Target::Stage {
                name: "Depths".into()
            })
        );
        assert_eq!(
            r.resolve("floor", "Depths"),
            Resolution::Target(Target::Stage {
                name: "Depths".into()
            })
        );
        assert_eq!(
            r.resolve("r", "Shop"),
            Resolution::Target(Target::Room {
                name: "Shop".into()
            })
        );
        assert_eq!(
            r.resolve("room", "boss rush"),
            Resolution::Target(Target::Room {
                name: "boss rush".into()
            })
        );
    }

    #[test]
    fn layout_and_unknown() {
        let r = test_resolver();
        assert_eq!(r.resolve("cit", "p"), Resolution::Ignore);
        assert_eq!(r.resolve("nav", ""), Resolution::Ignore);
        assert_eq!(r.resolve("#ev:youtube", "x"), Resolution::Ignore);
        // A name no template will ever have: see `unknown_and_layout_templates`.
        assert_eq!(
            r.resolve("notatemplate", "Donation Machine"),
            Resolution::Unknown
        );
        // Machines are concepts by construction, not failed lookups.
        assert_eq!(r.resolve("m", "Donation Machine"), Resolution::Concept);
        assert_eq!(r.resolve("machine", "Beggar"), Resolution::Concept);
    }

    #[test]
    fn page_titles_and_boss_keys() {
        let r = test_resolver();
        assert_eq!(r.by_page_title("Breakfast"), Some(Target::Item { id: 25 }));
        assert_eq!(
            r.by_page_title("Swallowed Penny"),
            Some(Target::Trinket { id: 1 })
        );
        assert_eq!(
            r.by_page_title("Jacob & Esau"),
            Some(Target::Character { id: 19 })
        );
        assert_eq!(
            r.by_page_title("The Family Man"),
            Some(Target::Challenge { number: 19 })
        );
        assert_eq!(
            r.by_page_title("Mom"),
            Some(Target::Entity {
                id: 45,
                variant: 0,
                subtype: 0
            })
        );
        assert_eq!(r.by_page_title("Nope"), None);
        assert_eq!(
            r.achievement_by_name("Epic Fetus"),
            Some(Target::Achievement { id: 62 })
        );
        assert_eq!(r.boss_key("Mom"), Some((45, 0, 0)));
        assert_eq!(r.boss_key("Angel"), None); // mini-boss, not a boss
    }
}
