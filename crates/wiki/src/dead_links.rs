//! Which `Ref`s and `Concept`s a reader cannot follow: a `Concept` never had a page to begin
//! with, and a `Ref` names an id the dataset has no entry for — a common enemy with no boss
//! page, a stage, a room, a pickup concept. `WikiInline.vue`'s own `canOpen` draws the same
//! line (a `Ref` whose target the wiki store's index lacks reads like a `Concept`), so this
//! is the reader's own question, not a stricter one invented for the report.
//!
//! Computed once the whole [`Dataset`] exists: whether a target has a page is a fact about
//! the *whole* dataset, not about the page a link sits on, so it cannot be a [`Diagnostics`]
//! counter bumped while one page is still being parsed — it is a read-only pass afterwards.
//!
//! Not part of [`Diagnostics`], and not carried in `dataset/wiki/`: the destinations are the
//! wiki's own inconsistency, of interest to whoever maintains the parser, and no use to the
//! app at runtime. `wiki-snapshot build` reports it on stdout instead of paying embedded
//! bytes for a fact nothing at runtime reads.

use std::collections::BTreeMap;

use crate::dataset::Dataset;
use crate::model::Inline;
use crate::title::canonical_title;
use crate::Target;

/// How often the dataset links to a destination nothing opens, by destination.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeadLinks {
    /// `Inline::Concept` destinations, keyed by [`canonical_title`]: MediaWiki treats two
    /// spellings that differ only in the first character's case, or in `_` against a space,
    /// as the same page, and so does this tally.
    pub concept_pages: BTreeMap<String, u32>,
    /// `Inline::Ref` destinations whose target the dataset has no entry for, keyed by the
    /// target's own identity. A `Ref` carries no page title to key by — there is no page —
    /// so [`target_identity`] names it by kind and id instead.
    pub unopenable_refs: BTreeMap<String, u32>,
}

impl DeadLinks {
    fn bump_concept(&mut self, page: &str) {
        *self.concept_pages.entry(canonical_title(page)).or_default() += 1;
    }

    fn bump_ref(&mut self, target: &Target) {
        *self
            .unopenable_refs
            .entry(target_identity(target))
            .or_default() += 1;
    }
}

/// A `Target`'s identity as a reader would name it: the kind and the id, a stage or room by
/// the name it was written with. Not `Debug`: this becomes a report line, not a trace.
fn target_identity(t: &Target) -> String {
    match t {
        Target::Item { id } => format!("item {id}"),
        Target::Trinket { id } => format!("trinket {id}"),
        Target::Character { id } => format!("character {id}"),
        Target::Achievement { id } => format!("achievement {id}"),
        Target::Challenge { number } => format!("challenge {number}"),
        Target::Entity {
            id,
            variant,
            subtype,
        } => format!("entity {id}.{variant}.{subtype}"),
        Target::Transformation { id } => format!("transformation {id}"),
        Target::Stage { name } => format!("stage {name}"),
        Target::Room { name } => format!("room {name}"),
        Target::Concept { name } => format!("concept {name}"),
        Target::Article { title } => format!("article {title}"),
    }
}

/// Every dead-link destination in the whole dataset, from every entry's own inline leaves
/// ([`crate::model::Entry::inlines`]).
pub fn dead_links(ds: &Dataset) -> DeadLinks {
    let mut out = DeadLinks::default();
    for entry in ds.entries() {
        for leaf in entry.inlines() {
            bump(leaf, ds, &mut out);
        }
    }
    out
}

/// One inline leaf. `Entry::inlines` has already unwrapped every `Edition`, so a leaf is
/// always a `Text`, a `Ref` or a `Concept`.
fn bump(leaf: &Inline, ds: &Dataset, out: &mut DeadLinks) {
    match leaf {
        Inline::Concept { page, .. } => out.bump_concept(page),
        Inline::Ref { target, .. } if ds.entry(target).is_none() => out.bump_ref(target),
        Inline::Ref { .. } | Inline::Text { .. } | Inline::Edition { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::build;
    use crate::page::PageKind;
    use crate::raw::{IndexEntry, Raw, RawPage};
    use crate::resolver::{Corrections, Row, Tables};

    fn row(pairs: &[(&str, &str)]) -> Row {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn page(kind: PageKind, title: &str, text: &str) -> RawPage {
        RawPage {
            title: title.to_string(),
            index: IndexEntry {
                kind,
                pageid: 1,
                revid: 1,
                timestamp: "2026-01-01T00:00:00Z".into(),
                category: None,
            },
            text: text.to_string(),
        }
    }

    /// Breakfast has a page; "Missing Page" is written as a wikilink and has none, twice,
    /// with a different first-letter case each time; "Uriel" is a mini-boss's alias in the
    /// entity table, with no boss page behind it.
    fn raw() -> Raw {
        Raw {
            pages: vec![
                page(
                    PageKind::Collectible,
                    "Breakfast",
                    "{{infobox passive collectible|id=25}}\n",
                ),
                page(
                    PageKind::Collectible,
                    "Wooden Spoon",
                    "{{infobox passive collectible|id=99}}\n\
                     == Effects ==\n\
                     See [[Breakfast]] and [[Missing Page]]. Guarded by {{e|Uriel}}.\n",
                ),
                page(
                    PageKind::Trinket,
                    "Broken Dice",
                    "{{infobox trinket|id=1}}\n== Notes ==\nSee [[missing Page]] too.\n",
                ),
            ],
            tables: Tables {
                collectible: vec![
                    row(&[
                        ("_pageName", "Breakfast"),
                        ("id", "25"),
                        ("alias", "Breakfast"),
                    ]),
                    row(&[
                        ("_pageName", "Wooden Spoon"),
                        ("id", "99"),
                        ("alias", "Wooden Spoon"),
                    ]),
                ],
                trinket: vec![row(&[
                    ("_pageName", "Broken Dice"),
                    ("id", "1"),
                    ("alias", "Broken Dice"),
                ])],
                entity: vec![row(&[
                    ("_pageName", "Angel"),
                    ("id", "271"),
                    ("variant", "0"),
                    ("subtype", "0"),
                    ("type", "mini-boss"),
                    ("alias", "Uriel"),
                ])],
                ..Tables::default()
            },
            versions: vec![],
            redirects: BTreeMap::new(),
            template_infobox_character: None,
        }
    }

    #[test]
    fn a_wikilink_to_a_missing_page_is_counted_and_an_existing_one_is_not() {
        let ds = build(&raw(), &Corrections::default());
        let links = dead_links(&ds);
        assert_eq!(links.concept_pages.get("Missing Page"), Some(&2));
        assert_eq!(
            links.concept_pages.get("Breakfast"),
            None,
            "a page the dataset has is not a dead link: {:?}",
            links.concept_pages
        );
    }

    /// "Missing Page" and "missing Page" are the same MediaWiki title: the tally must not
    /// read them as two destinations just because one page wrote the first letter lowercase.
    #[test]
    fn two_spellings_of_the_same_title_are_one_destination() {
        let ds = build(&raw(), &Corrections::default());
        let links = dead_links(&ds);
        assert_eq!(links.concept_pages.get("missing Page"), None);
        assert_eq!(links.concept_pages.len(), 1, "{:?}", links.concept_pages);
    }

    #[test]
    fn an_entity_with_no_boss_page_is_an_unopenable_ref() {
        let ds = build(&raw(), &Corrections::default());
        let links = dead_links(&ds);
        assert_eq!(links.unopenable_refs.get("entity 271.0.0"), Some(&1));
    }

    /// A `Ref` to a target the dataset *does* have a page for — Breakfast, resolved through
    /// `[[Breakfast]]` — must not also show up as unopenable, beside Uriel's, which must: the
    /// two counters are read from the same walk, and this is the guard that it tells them apart
    /// node by node rather than by kind.
    #[test]
    fn a_ref_to_an_existing_page_is_not_an_unopenable_ref() {
        let ds = build(&raw(), &Corrections::default());
        let links = dead_links(&ds);
        assert_eq!(
            links.unopenable_refs.get("item 25"),
            None,
            "{:?}",
            links.unopenable_refs
        );
        assert_eq!(links.unopenable_refs.get("entity 271.0.0"), Some(&1));
    }
}
