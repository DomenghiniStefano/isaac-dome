//! From `Raw` to `Dataset`: the resolver, the pages in order, the `meta`. Deterministic:
//! same `raw/`, same JSON.

use std::collections::BTreeMap;

use crate::dataset::{Dataset, Meta, Patch, Source, HOST};
use crate::infobox::{extract_infoboxes, InfoboxKind, RawInfobox};
use crate::page::{parse_page, PageKind};
use crate::raw::{Raw, RawPage};
use crate::resolver::{Corrections, Resolver, Row};
use crate::unlock_condition::add_conditions;
use crate::Diagnostics;

/// Characters have no Cargo table: title (and the infobox's `name`) → id is derived from
/// their pages, before building the resolver. A title always takes its page's id; a `name`
/// takes one only if nothing took that key before it.
fn characters(raw: &Raw) -> BTreeMap<String, u32> {
    raw.pages
        .iter()
        .filter(|p| p.index.kind == PageKind::Character)
        .filter_map(|p| Some((p, character_infobox(&p.text)?)))
        .filter_map(|(p, ib)| Some((p, leading_id(&ib)?, ib)))
        .fold(BTreeMap::new(), |mut characters, (p, id, ib)| {
            characters.insert(p.title.clone(), id);
            if let Some(alias) = ib.params.get("name") {
                characters.entry(alias.trim().to_string()).or_insert(id);
            }
            characters
        })
}

/// A character page's first character infobox.
fn character_infobox(text: &str) -> Option<RawInfobox> {
    extract_infoboxes(text)
        .into_iter()
        .find(|ib| InfoboxKind::of(&ib.name) == Some(InfoboxKind::Character))
}

/// The infobox's `id`, when it is a number and nothing else.
fn leading_id(ib: &RawInfobox) -> Option<u32> {
    ib.params.get("id")?.trim().parse().ok()
}

/// Builds the dataset. Pages are visited in (kind, title) order; a key that recurs keeps
/// the first entry.
pub fn build(raw: &Raw, corrections: &Corrections) -> Dataset {
    let r = Resolver::new(&raw.tables, &characters(raw), corrections);
    let mut ds = Dataset::empty();
    let mut diagnostics = Diagnostics::default();
    for p in pages_in_order(raw) {
        let mut d = Diagnostics::default();
        for (key, entry) in parse_page(&p.title, p.index.revid, &p.text, &r, &mut d) {
            ds.insert_first(key, entry);
        }
        diagnostics.merge(&d);
        note_revision(&mut ds.meta, p);
    }
    apply_descriptions(&mut ds, corrections, &r, &mut diagnostics);
    add_conditions(&mut ds);
    ds.meta.last_known_patch = last_known_patch(&raw.versions);
    ds.meta.source = Source {
        name: "The Binding of Isaac: Rebirth Wiki".into(),
        url: HOST.into(),
        license: "CC BY-SA 4.0".into(),
    };
    ds.meta.counts = ds.counts();
    ds.meta.diagnostics = diagnostics;
    ds
}

fn pages_in_order(raw: &Raw) -> Vec<&RawPage> {
    let mut pages: Vec<&RawPage> = raw.pages.iter().collect();
    pages.sort_by(|a, b| (a.index.kind, &a.title).cmp(&(b.index.kind, &b.title)));
    pages
}

/// The snapshot is as recent as its most recent page.
fn note_revision(meta: &mut Meta, p: &RawPage) {
    if p.index.timestamp > meta.snapshot_at {
        meta.snapshot_at = p.index.timestamp.clone();
    }
    meta.max_revid = meta.max_revid.max(p.index.revid);
}

/// The descriptions `corrections.json` carries. Written by hand, and one wins over the page: the reason
/// to write one is that the wiki's is missing or says nothing. One that names no entry is left
/// for `Corrections::unmatched_descriptions` to report, not guessed at.
fn apply_descriptions(
    ds: &mut Dataset,
    corrections: &Corrections,
    r: &Resolver,
    d: &mut Diagnostics,
) {
    for (collection, entries) in &corrections.descriptions {
        for (key, text) in entries {
            if let Some(entry) = ds.entry_by_key_mut(collection, key) {
                entry.description = crate::inline::parse_inline(text, r, d);
            }
        }
    }
}

/// The most recent patch in the `version` table, by date.
fn last_known_patch(versions: &[Row]) -> Option<Patch> {
    versions
        .iter()
        .filter_map(|v| Some((v.get("date")?.clone(), v.get("number")?.clone())))
        .max()
        .map(|(date, number)| Patch { number, date })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{DatasetError, SCHEMA_VERSION};
    use crate::raw::{IndexEntry, RawPage};
    use crate::resolver::{Row, Tables};
    use crate::{PageKind, Target};

    fn raw() -> Raw {
        let page = |kind, title: &str, ts: &str, revid, text: &str| RawPage {
            title: title.into(),
            index: IndexEntry {
                kind,
                pageid: 1,
                revid,
                timestamp: ts.into(),
            },
            text: text.into(),
        };
        let row = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Row>()
        };
        Raw {
            pages: vec![
                page(
                    PageKind::Collectible,
                    "Breakfast",
                    "2026-02-01T00:00:00Z",
                    9,
                    "{{infobox passive collectible|id=25}}\n== Effects ==\n* with {{c|Cain}} and {{i|Nope}}\n",
                ),
                page(
                    PageKind::Character,
                    "Cain",
                    "2026-01-01T00:00:00Z",
                    3,
                    "{{infobox character|id=2}}\n== Notes ==\nn\n",
                ),
                page(
                    PageKind::Transformation,
                    "Guppy",
                    "2026-01-15T00:00:00Z",
                    5,
                    "{{infobox transformation|id=0|items={{i|Breakfast}}}}\nPick up 3 [[item]]s from the following list.\n{{collectible table | Breakfast }}\n",
                ),
            ],
            tables: Tables {
                collectible: vec![row(&[
                    ("_pageName", "Breakfast"),
                    ("id", "25"),
                    ("alias", "Breakfast"),
                ])],
                ..Tables::default()
            },
            versions: vec![
                row(&[("number", "v1.9.7.16"), ("date", "2026-04-11")]),
                row(&[("number", "v1.9.7.17"), ("date", "2026-04-20")]),
            ],
        }
    }

    #[test]
    fn builds_maps_meta_and_resolves_characters_from_their_pages() {
        let ds = build(&raw(), &Corrections::default());
        assert_eq!(ds.meta.schema_version, SCHEMA_VERSION);
        assert_eq!(ds.meta.snapshot_at, "2026-02-01T00:00:00Z");
        assert_eq!(ds.meta.max_revid, 9);
        assert_eq!(
            ds.meta.last_known_patch.as_ref().map(|p| p.number.as_str()),
            Some("v1.9.7.17")
        );
        assert_eq!(ds.meta.counts.items, 1);
        assert_eq!(ds.meta.counts.characters, 1);
        assert_eq!(ds.meta.diagnostics.unresolved.get("i"), Some(&1));
        let e = ds.entry(&Target::Item { id: 25 }).unwrap();
        let s = serde_json::to_string(e).unwrap();
        assert!(s.contains(r#""kind":"character","id":2"#));
        assert!(ds.entry(&Target::Character { id: 2 }).is_some());
        assert!(ds.entry(&Target::Stage { name: "x".into() }).is_none());
    }

    /// The kind that used to answer `None` by construction now answers, and its count
    /// travels in `meta` beside the other six. `Stage` stays `None`: it has no page.
    #[test]
    fn a_transformation_has_an_entry_a_count_and_its_set() {
        let ds = build(&raw(), &Corrections::default());
        assert_eq!(ds.meta.counts.transformations, 1);
        let e = ds.entry(&Target::Transformation { id: 0 }).unwrap();
        assert_eq!(e.title, "Guppy");
        let crate::Infobox::Transformation {
            requires,
            contributors,
            ..
        } = &e.infobox
        else {
            panic!("a transformation page carries a transformation infobox")
        };
        assert_eq!(*requires, Some(3));
        assert_eq!(contributors, &vec![Target::Item { id: 25 }]);
        assert!(ds.entry(&Target::Stage { name: "x".into() }).is_none());
    }

    fn with_descriptions(pairs: &[(&str, &str, &str)]) -> Corrections {
        let mut c = Corrections::default();
        for (collection, key, text) in pairs {
            c.descriptions
                .entry(collection.to_string())
                .or_default()
                .insert(key.to_string(), text.to_string());
        }
        c
    }

    /// `corrections.json` carries descriptions written by hand — Dead God's, which the wiki
    /// leaves blank, or a better line than the wiki's — keyed the way `wiki.json` keys its
    /// collections. One **wins** over whatever the page gave, because the reason to write
    /// one is that the page's is missing or says nothing. It is wikitext, so a link in it is
    /// a link on screen.
    #[test]
    fn a_hand_written_description_wins_and_is_wikitext() {
        let ds = build(
            &raw(),
            &with_descriptions(&[
                ("characters", "2", "Starts with {{i|Breakfast}}."),
                ("transformations", "0", "Three flies."),
            ]),
        );
        let cain = ds.entry(&Target::Character { id: 2 }).unwrap();
        assert_eq!(crate::plain(&cain.description), "Starts with Breakfast.");
        assert!(cain.description.iter().any(|i| matches!(
            i,
            crate::Inline::Ref {
                target: Target::Item { id: 25 },
                ..
            }
        )));
        let guppy = ds.entry(&Target::Transformation { id: 0 }).unwrap();
        assert_eq!(crate::plain(&guppy.description), "Three flies.");
    }

    /// A correction naming no entry does nothing, and says nothing either: the file is
    /// written by hand, so a typo in a key is the likely failure. `unmatched_descriptions`
    /// is what makes it speak, and this is it speaking about one it knows is wrong.
    #[test]
    fn a_description_for_no_entry_is_reported() {
        let corrections = with_descriptions(&[
            ("characters", "2", "fine"),
            ("characters", "99", "no such character"),
            ("charcters", "2", "no such collection"),
        ]);
        let ds = build(&raw(), &corrections);
        assert_eq!(
            corrections.unmatched_descriptions(&ds),
            vec![
                ("characters".to_string(), "99".to_string()),
                ("charcters".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn json_roundtrip_and_schema_check() {
        let ds = build(&raw(), &Corrections::default());
        let s = ds.to_json();
        assert_eq!(Dataset::from_json(&s).unwrap(), ds);
        // Written against the constant, not against the literal it happens to hold: this
        // test used to pin `"schemaVersion": 1` and went silently no-op the day the schema
        // moved — `replacen` found nothing, the JSON stayed valid, and the assertion below
        // passed for the wrong reason.
        let current = format!(r#""schemaVersion": {SCHEMA_VERSION}"#);
        assert!(
            s.contains(&current),
            "the shape of the field changed: {current}"
        );
        let bad = s.replacen(&current, r#""schemaVersion": 99"#, 1);
        assert!(matches!(
            Dataset::from_json(&bad),
            Err(DatasetError::SchemaMismatch { found: 99, .. })
        ));
        assert!(matches!(
            Dataset::from_json("{"),
            Err(DatasetError::Malformed { .. })
        ));
    }
}
