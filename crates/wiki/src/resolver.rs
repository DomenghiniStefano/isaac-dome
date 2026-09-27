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
    /// Descriptions written by hand, as wikitext: collection, named the way `dataset/wiki/`
    /// names its files (`achievements`, `bosses`…), then the entry's key in that collection. One
    /// wins over the page's, on any kind — the reason to write one is that the wiki's is
    /// missing (Dead God) or says nothing.
    #[serde(default)]
    pub descriptions: BTreeMap<String, BTreeMap<String, String>>,
    /// What Decision 10 excludes from "every template is read into structure": a template the
    /// parser deliberately does not model, with the reason written next to it. A completeness
    /// check in `crates/wiki/tests/` fails on a template that is neither modelled nor listed
    /// here, and on a listed name the corpus no longer uses.
    #[serde(default)]
    pub excluded: Excluded,
    /// The dead-link residue (design decision 6, `2026-09-26-wiki-complete-design.md`): every
    /// destination [`crate::dead_links::dead_links`] still reports once resolution has run,
    /// named the way [`crate::dead_links::DeadLinks`] keys it (`concept_pages`'s canonical
    /// title, or `unopenable_refs`' `"<kind> <id>"`), with the reason it has no page. A test
    /// in `tests/real.rs` fails on a destination that is dead and unlisted, and on a listed
    /// one that no longer is — the residue is meant to shrink to nothing, not to grow quietly.
    #[serde(default)]
    pub dead_links: BTreeMap<String, String>,
}

/// The `excluded` key of `corrections.json`. A struct of one field today, kept apart from
/// `Corrections`'s other maps because Decision 10 may grow more than one closed list under
/// `excluded` (section headings, infobox parameters), each with its own reason column.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Excluded {
    /// Template name → why the parser does not read it into structure: it occurs only inside
    /// a section already discarded whole (Trivia, Gallery, Audio…), or it is a one-time typo
    /// on the wiki's own side that was not worth silently correcting.
    #[serde(default)]
    pub templates: BTreeMap<String, String>,
    /// Section title (as a page writes it, not normalized) → why the whole heading is
    /// dropped rather than kept as `SectionKind::Other`: images and video the constraints
    /// forbid shipping, citations off the wiki, sound listings (a game asset like the
    /// others), and the owner's call on Trivia. The single source `sections::is_excluded_section`
    /// reads at runtime; `sections.rs`'s completeness test is what keeps this list honest
    /// against the corpus.
    #[serde(default)]
    pub sections: BTreeMap<String, String>,
}

/// The tables [`Corrections::apply`] is ever called with. A `page_id` entry filed under
/// any other name is a correction that can never fire, and `corrections.json` is written
/// by hand: a typo there produces no error, no warning and no effect. Kept next to
/// `apply` because it is the list of its call sites.
pub const CORRECTED_TABLES: [&str; 2] = ["collectible", "trinket"];

impl Corrections {
    /// The `page_id` tables no lookup will ever ask about. Empty is the healthy answer;
    /// anything else is a correction sitting in the file doing nothing.
    #[cfg(feature = "test-api")]
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
    #[cfg(feature = "test-api")]
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
    /// Design decision 4: a character page's base-stat default (`damage`, `tears`, `range`,
    /// `speed`, `luck`, `shot speed`), read from `Template:Infobox character`'s own wikitext
    /// by `infobox::stat_defaults` and set once through
    /// [`Resolver::with_character_stat_defaults`]. Empty until that runs — no template
    /// fetched, or nobody called it — which degrades to today's "a stat nobody stated is
    /// empty".
    character_stat_defaults: BTreeMap<String, String>,
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
    // The page-wide edition range, read from the first infobox instead (`page_editions_of`):
    // the template itself renders no icon of its own.
    "page dlc",
    // Editorial markers: a flag for another editor, drawn as a small icon or nothing at all,
    // never a fact about the game.
    "citation needed",
    "reconfirm",
    "explain",
    // `{{dlc clear}}` clears the floating edition box `{{dlc}}` can leave open, the same way
    // `{{clear}}` clears a floated image: layout, not content.
    "dlc clear",
    // Wiki maintenance markers with no reader-facing text at all: hidden categories and the
    // shop-storage flag pages carry beside `{{storage page}}`.
    "categories",
    "storage",
    "no storage",
    // MediaWiki's own magic word for overriding how the page's title renders (`Less Than
    // Three` displays as "<3"): the title the reader sees, not a fact about the subject.
    "displaytitle:<3",
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

/// A row's title and alias into one name map. The title enters only if no earlier row took
/// that key; the alias always does, over whatever was there. That is the rule of every table
/// read this way — collectibles, trinkets, challenges, entities, transformations — and it is
/// one rule, so it is written once.
fn index_by_title_and_alias<V: Clone>(map: &mut BTreeMap<String, V>, row: &Row, value: V) {
    if let Some(title) = get(row, "_pageName") {
        map.entry(key(title)).or_insert_with(|| value.clone());
    }
    if let Some(alias) = get(row, "alias") {
        map.insert(key(alias), value);
    }
}

/// Collectibles and trinkets, which read alike: a row needs a title and an id, the id is the
/// one `corrections.json` gives for `table`, and the title also enters the by-title map that
/// keeps every id a page carries.
fn index_numbered_pages(
    rows: &[Row],
    table: &str,
    corrections: &Corrections,
    by_title: &mut BTreeMap<String, BTreeSet<u32>>,
    names: &mut BTreeMap<String, u32>,
) {
    for (row, title, id) in
        current(rows).filter_map(|row| Some((row, get(row, "_pageName")?, num(row, "id")?)))
    {
        let id = corrections.apply(table, title, id);
        by_title.entry(key(title)).or_default().insert(id);
        index_by_title_and_alias(names, row, id);
    }
}

/// An entity row's `id`, `variant` and `subtype`, all three or nothing.
fn bestiary_triple(row: &Row) -> Option<(u32, u32, u32)> {
    Some((num(row, "id")?, num(row, "variant")?, num(row, "subtype")?))
}

impl Resolver {
    /// The maps, one table at a time. Rows from past editions enter none of them.
    pub fn new(
        tables: &Tables,
        characters: &BTreeMap<String, u32>,
        corrections: &Corrections,
    ) -> Resolver {
        let mut r = Resolver {
            corrections: corrections.clone(),
            ..Resolver::default()
        };
        index_numbered_pages(
            &tables.collectible,
            "collectible",
            corrections,
            &mut r.items_by_title,
            &mut r.items,
        );
        index_numbered_pages(
            &tables.trinket,
            "trinket",
            corrections,
            &mut r.trinkets_by_title,
            &mut r.trinkets,
        );
        r.index_achievements(&tables.achievement);
        r.index_challenges(&tables.challenge);
        r.index_entities(&tables.entity);
        r.index_transformations(&tables.transformation);
        r.index_pickups(&tables.pickup);
        r.index_players(&tables.player);
        r.index_characters(characters, corrections);
        r
    }

    /// Design decision 4: sets the base-stat defaults `infobox::stat_defaults` read from
    /// `Raw::template_infobox_character`. A separate step from [`Resolver::new`] rather than
    /// one more argument on it, because every other caller in this crate's own tests builds a
    /// `Resolver` with no template text at all and would otherwise have to invent one.
    #[must_use]
    pub fn with_character_stat_defaults(mut self, defaults: BTreeMap<String, String>) -> Resolver {
        self.character_stat_defaults = defaults;
        self
    }

    /// A character base stat's default, by its infobox parameter name (`"damage"`, `"shot
    /// speed"`…). `None` when `with_character_stat_defaults` was never called, or didn't have
    /// that stat — a missing template degrades the same way a page that states nothing does.
    pub(crate) fn character_stat_default(&self, name: &str) -> Option<&str> {
        self.character_stat_defaults.get(name).map(String::as_str)
    }

    /// The reverse of the title rule: an achievement's `name` always enters, its alias only
    /// if nothing took that key.
    fn index_achievements(&mut self, rows: &[Row]) {
        for (row, id) in current(rows).filter_map(|row| Some((row, num(row, "id")?))) {
            if let Some(n) = get(row, "name") {
                self.achievements.insert(key(n), id);
            }
            if let Some(a) = get(row, "alias") {
                self.achievements.entry(key(a)).or_insert(id);
            }
        }
    }

    /// A challenge is also found by its number, written as text.
    fn index_challenges(&mut self, rows: &[Row]) {
        for (row, n) in current(rows).filter_map(|row| Some((row, num(row, "number")?))) {
            if let Some(t) = get(row, "_pageName") {
                self.challenges_by_title.insert(key(t), n);
            }
            index_by_title_and_alias(&mut self.challenges, row, n);
            self.challenges.insert(n.to_string(), n);
        }
    }

    /// Every entity by name; the bosses, by title alone, into the bestiary map as well.
    fn index_entities(&mut self, rows: &[Row]) {
        for (row, triple) in current(rows).filter_map(|row| Some((row, bestiary_triple(row)?))) {
            index_by_title_and_alias(&mut self.entities, row, triple);
            if let (Some("boss"), Some(t)) = (get(row, "type"), get(row, "_pageName")) {
                self.bosses_by_title.entry(key(t)).or_insert(triple);
            }
        }
    }

    fn index_transformations(&mut self, rows: &[Row]) {
        for (row, id) in current(rows).filter_map(|row| Some((row, num(row, "id")?))) {
            index_by_title_and_alias(&mut self.transformations, row, id);
        }
    }

    fn index_pickups(&mut self, rows: &[Row]) {
        for a in current(rows).filter_map(|row| get(row, "alias")) {
            self.pickups.insert(key(a), a.to_string());
        }
    }

    /// Each form's `parent`, raw, by its alias; the first row for an alias wins.
    fn index_players(&mut self, rows: &[Row]) {
        for row in current(rows) {
            if let Some(a) = get(row, "alias") {
                self.player_parent
                    .entry(key(a))
                    .or_insert_with(|| get(row, "parent").unwrap_or("").to_string());
            }
        }
    }

    /// The ids derived from the pages, then our own map over them, which also gives the name.
    fn index_characters(&mut self, characters: &BTreeMap<String, u32>, corrections: &Corrections) {
        for (name, id) in characters {
            self.characters.insert(key(name), *id);
        }
        for (name, id) in &corrections.characters {
            self.characters.insert(key(name), *id);
            self.character_names.insert(key(name), name.clone());
        }
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

    /// `template` in any case and with any spacing: it is trimmed and lowercased here, since
    /// the name comes from the wikitext. `arg` is the first raw argument.
    ///
    /// Long, and flat on purpose: one arm per template the wiki uses for a link, each saying
    /// which map answers it. Split up, the list of link templates would stop being one list.
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

    /// Page title → target, for the infoboxes' `link`/`unlocks`/`unlocked by`, and for a
    /// plain `[[wikilink]]` (`inline::link`).
    /// Precedence: character, item, trinket, challenge, entity, transformation.
    ///
    /// Transformations joined this list on 2026-09-26 (design decision 3): a wikilink to a
    /// transformation's own page (`[[Beelzebub]]`, `[[Guppy]]`) used to name nothing here —
    /// only `{{tf|…}}` resolved one — so every such link fell through to `Inline::Concept`
    /// and stayed a dead link even though the page exists and has an id. Measured on the
    /// snapshot: seven transformation names accounted for 30 of the 161 dead-concept
    /// occurrences before this was added.
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
        if let Some(id) = self.transformations.get(&k) {
            return Some(Target::Transformation { id: *id });
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

    /// Whether a level-2 heading is on the closed exclusion list, read from
    /// `corrections.json`'s `excluded.sections` the resolver already carries: the one place
    /// that map reaches the parser, so `page.rs`'s own section split never reads the file
    /// itself. See `sections::is_excluded_section` for the normalization and the reason.
    pub fn is_section_excluded(&self, title: &str) -> bool {
        crate::sections::is_excluded_section(title, &self.corrections.excluded.sections)
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

    /// An entity's bestiary triple by name — its alias in the Cargo table, or the page title
    /// for the entity a page's own first infobox names. Every type (`monster`, `mini-boss`,
    /// `boss`, or none), unlike `boss_key`: an entity infobox's own key resolution needs the
    /// whole table, not only the bosses.
    pub fn entity_of_name(&self, name: &str) -> Option<(u32, u32, u32)> {
        self.entities.get(&key(name)).copied()
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
        // `excluded.sections` mirrors `dataset/corrections.json`'s own set (not read from
        // disk: this fixture stays a pure unit-test double), because page/block tests that
        // exercise a page's sections — `Trivia` chief among them — expect the same headings
        // production drops, now that `is_section_excluded` reads this map instead of a
        // hardcoded list.
        let corrections: Corrections = serde_json::from_str(
            r#"{"pageId":{"collectible":{"Misfiled":901}},
                "characters":{"Isaac":0,"Jacob & Esau":19,"???":4},
                "excluded":{"sections":{
                    "Gallery":"images, a game asset",
                    "In-game Footage":"video, a game asset",
                    "In-Game Footage":"video, a game asset",
                    "Ingame Footage":"video, a game asset",
                    "References":"external citations, off the wiki",
                    "Trivia":"the owner's call",
                    "Audio":"sound listings, a game asset",
                    "Sounds":"sound listings, a game asset"
                }}}"#,
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

    /// `corrections.json`'s `excluded.templates` key, read the way the file actually writes
    /// it: camelCase at the outer level, the template's own name (lowercase, spaces and all)
    /// unchanged as a map key.
    #[test]
    fn excluded_templates_round_trip_from_json() {
        let c: Corrections = serde_json::from_str(
            r#"{"excluded":{"templates":{"sound table row":"only inside a discarded section"}}}"#,
        )
        .unwrap();
        assert_eq!(
            c.excluded
                .templates
                .get("sound table row")
                .map(String::as_str),
            Some("only inside a discarded section")
        );
        // Absent entirely: reads as empty, the same degrade every other `Corrections` map has.
        let empty: Corrections = serde_json::from_str("{}").unwrap();
        assert!(empty.excluded.templates.is_empty());
    }

    /// `corrections.json`'s `excluded.sections` key, the single source `is_section_excluded`
    /// reads — `sections.rs` no longer carries its own copy of this list.
    #[test]
    fn excluded_sections_round_trip_from_json() {
        let c: Corrections =
            serde_json::from_str(r#"{"excluded":{"sections":{"Gallery":"images, a game asset"}}}"#)
                .unwrap();
        assert_eq!(
            c.excluded.sections.get("Gallery").map(String::as_str),
            Some("images, a game asset")
        );
        let empty: Corrections = serde_json::from_str("{}").unwrap();
        assert!(empty.excluded.sections.is_empty());
    }

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
        // A transformation's own page, which `{{tf|…}}` already resolved but a plain
        // `[[Beelzebub]]` wikilink did not, until 2026-09-26.
        assert_eq!(
            r.by_page_title("Beelzebub"),
            Some(Target::Transformation { id: 1 })
        );
        assert_eq!(r.by_page_title("Nope"), None);
        assert_eq!(
            r.achievement_by_name("Epic Fetus"),
            Some(Target::Achievement { id: 62 })
        );
        assert_eq!(r.boss_key("Mom"), Some((45, 0, 0)));
        assert_eq!(r.boss_key("Angel"), None); // mini-boss, not a boss
    }

    /// `Resolver::is_section_excluded` reaches the same `excluded.sections` map the JSON
    /// round-trip test above reads, through the resolver rather than a second file read —
    /// the way `page.rs`'s section split is meant to consult it.
    #[test]
    fn is_section_excluded_reads_the_resolver_s_own_corrections() {
        let corrections: Corrections =
            serde_json::from_str(r#"{"excluded":{"sections":{"Gallery":"a game asset"}}}"#)
                .unwrap();
        let r = Resolver::new(&Tables::default(), &BTreeMap::new(), &corrections);
        assert!(r.is_section_excluded("Gallery"));
        assert!(r.is_section_excluded("{{dlc|nr}} Gallery"));
        assert!(!r.is_section_excluded("Blood Clots"));
    }
}
