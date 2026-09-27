//! From `Raw` to `Dataset`: the resolver, the pages in order, the `meta`. Deterministic:
//! same `raw/`, same JSON.

use std::collections::{BTreeMap, BTreeSet};

use crate::dataset::{resolve_redirect, Dataset, Meta, Patch, Source, HOST};
use crate::infobox::{extract_infoboxes, InfoboxKind, RawInfobox};
use crate::page::{parse_article_page, parse_page, EntryKey, PageKind};
use crate::raw::{Raw, RawPage};
use crate::resolver::{Corrections, Resolver, Row};
use crate::unlock_condition::add_conditions;
use crate::{Block, Diagnostics, Entry, Inline, Target};

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
    // Carried through as-is: `Dataset::entry` chases a redirect to resolve a `Stage`, `Room`
    // or `Concept` target to its article (design decision 3), and `resolve_concepts_to_articles`
    // below chases the same map to turn a `Concept` link into a `Ref`.
    ds.meta.redirects = raw.redirects.clone();
    let mut diagnostics = Diagnostics::default();
    for p in pages_in_order(raw) {
        note_revision(&mut ds.meta, p);
        let mut d = Diagnostics::default();
        for (key, entry) in entries_of(p, &r, &mut d) {
            ds.insert_first(key, entry);
        }
        diagnostics.merge(&d);
    }
    apply_descriptions(&mut ds, corrections, &r, &mut diagnostics);
    add_conditions(&mut ds);
    resolve_concepts_to_articles(&mut ds, &r);
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

/// The entries one page gives. Every other kind is dispatched by its infoboxes
/// (`parse_page`, one entry per recognized infobox — `Entity` reads the same way, through
/// `InfoboxKind::Entity`). `Article` has none: it is one entry per page, built from the page
/// itself and the category `index.json` already decided at fetch time (design decision 2).
/// Written as an explicit match rather than a wildcard so a ninth kind has to say which arm
/// it takes.
fn entries_of(p: &RawPage, r: &Resolver, d: &mut Diagnostics) -> Vec<(EntryKey, Entry)> {
    match p.index.kind {
        PageKind::Article => vec![parse_article_page(
            &p.title,
            p.index.revid,
            &p.text,
            p.index.category,
            r,
            d,
        )],
        PageKind::Collectible
        | PageKind::Trinket
        | PageKind::Achievement
        | PageKind::Boss
        | PageKind::Challenge
        | PageKind::Character
        | PageKind::Transformation
        | PageKind::Entity => parse_page(&p.title, p.index.revid, &p.text, r, d),
    }
}

/// Turns a remaining `Inline::Concept` into a `Ref`, once the whole dataset exists: whether a
/// page is an article, or a redirect leads to one, is a fact about the *whole* snapshot, not
/// about the entry currently being read, so this cannot run inside `parse_page` itself
/// (design decision 3). `r.by_page_title` is tried first — a redirect can lead onto a page of
/// any other kind (an item, a boss…), not only an article — an article match is the fallback,
/// since `by_page_title` does not know that collection.
fn resolve_concepts_to_articles(ds: &mut Dataset, r: &Resolver) {
    let articles: BTreeSet<String> = ds.articles.keys().cloned().collect();
    let redirects = ds.meta.redirects.clone();
    for entry in ds.entries_mut() {
        resolve_inline(&mut entry.description, r, &articles, &redirects);
        for field in entry.infobox.inlines_mut() {
            resolve_inline(field, r, &articles, &redirects);
        }
        for section in &mut entry.sections {
            for block in &mut section.blocks {
                resolve_block(block, r, &articles, &redirects);
            }
        }
    }
}

fn resolve_block(
    block: &mut Block,
    r: &Resolver,
    articles: &BTreeSet<String>,
    redirects: &BTreeMap<String, String>,
) {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, level: _ } => {
            resolve_inline(inline, r, articles, redirects)
        }
        Block::List { ordered: _, items } => {
            for item in items {
                resolve_inline(&mut item.inline, r, articles, redirects);
                for child in &mut item.children {
                    resolve_block(child, r, articles, redirects);
                }
            }
        }
        Block::Table { header, rows } => {
            for cell in header {
                resolve_inline(cell, r, articles, redirects);
            }
            for row in rows {
                for cell in row {
                    resolve_inline(cell, r, articles, redirects);
                }
            }
        }
    }
}

fn resolve_inline(
    inline: &mut [Inline],
    r: &Resolver,
    articles: &BTreeSet<String>,
    redirects: &BTreeMap<String, String>,
) {
    for node in inline {
        match node {
            Inline::Edition { inline, .. } => resolve_inline(inline, r, articles, redirects),
            Inline::Concept { page, label } => {
                if let Some(target) = article_target_of(page, r, articles, redirects) {
                    let label = std::mem::take(label);
                    *node = Inline::Ref { target, label };
                }
            }
            Inline::Text { .. } | Inline::Ref { .. } => {}
        }
    }
}

/// The target `page` names, once redirects are followed: any kind's page title first, an
/// article by its canonical name otherwise, `None` if neither — the same dead link
/// [`crate::dead_links::dead_links`] already counted, now confirmed after redirects.
fn article_target_of(
    page: &str,
    r: &Resolver,
    articles: &BTreeSet<String>,
    redirects: &BTreeMap<String, String>,
) -> Option<Target> {
    let resolved = resolve_redirect(page, redirects);
    r.by_page_title(&resolved).or_else(|| {
        articles.contains(&resolved).then(|| Target::Article {
            title: resolved.clone(),
        })
    })
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
                category: None,
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
            redirects: BTreeMap::new(),
            template_infobox_character: None,
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

    /// `Entity` and `Article` pages build into entries like every other kind (design
    /// decision 2): both advance `meta`'s snapshot clock, and each files under its own
    /// collection with its own count.
    #[test]
    fn entity_and_article_pages_build_into_entries() {
        let mut with_new_kinds = raw();
        with_new_kinds.pages.push(RawPage {
            title: "Gaper".into(),
            index: IndexEntry {
                kind: PageKind::Entity,
                pageid: 2,
                revid: 99,
                timestamp: "2026-03-01T00:00:00Z".into(),
                category: None,
            },
            text: "{{infobox monster|id=1|variant=0|subtype=0}}\n== Behavior ==\nx\n".into(),
        });
        with_new_kinds.pages.push(RawPage {
            title: "Damage".into(),
            index: IndexEntry {
                kind: PageKind::Article,
                pageid: 3,
                revid: 100,
                timestamp: "2026-03-02T00:00:00Z".into(),
                category: None,
            },
            text: "'''Damage''' is a stat.\n== Notes ==\nx\n".into(),
        });
        let ds = build(&with_new_kinds, &Corrections::default());
        // Neither page added an item, a character or a transformation.
        assert_eq!(ds.meta.counts.items, 1);
        assert_eq!(ds.meta.counts.characters, 1);
        assert_eq!(ds.meta.counts.transformations, 1);
        assert_eq!(ds.meta.counts.entities, 1);
        assert_eq!(ds.meta.counts.articles, 1);
        // Both still moved the snapshot's own clock.
        assert_eq!(ds.meta.snapshot_at, "2026-03-02T00:00:00Z");
        assert_eq!(ds.meta.max_revid, 100);
        let gaper = ds
            .entry(&Target::Entity {
                id: 1,
                variant: 0,
                subtype: 0,
            })
            .unwrap();
        assert_eq!(gaper.title, "Gaper");
        let damage = ds
            .entry(&Target::Article {
                title: "Damage".into(),
            })
            .unwrap();
        assert_eq!(damage.title, "Damage");
    }

    /// Decision 3: a `[[link]]` that stays a dead `Concept` after parsing — the page it names
    /// was not fetched with an id in mind — is resolved once the whole dataset exists,
    /// through any redirect the wiki wrote.
    #[test]
    fn a_concept_through_a_redirect_becomes_a_ref_to_the_article() {
        let mut with_article = raw();
        with_article.pages[0].text =
            "{{infobox passive collectible|id=25}}\n== Effects ==\n[[Soul Hearts]] heal.\n".into();
        with_article.pages.push(RawPage {
            title: "Health".into(),
            index: IndexEntry {
                kind: PageKind::Article,
                pageid: 4,
                revid: 1,
                timestamp: "2026-01-01T00:00:00Z".into(),
                category: None,
            },
            text: "'''Health''' is how much damage Isaac can take.\n".into(),
        });
        with_article
            .redirects
            .insert("Soul Hearts".to_string(), "Health".to_string());
        let ds = build(&with_article, &Corrections::default());
        let breakfast = ds.entry(&Target::Item { id: 25 }).unwrap();
        assert!(
            breakfast.inlines().iter().any(|i| matches!(
                i,
                crate::Inline::Ref {
                    target: Target::Article { title },
                    label
                } if title == "Health" && label == "Soul Hearts"
            )),
            "{:?}",
            breakfast.inlines()
        );
    }

    /// The same resolution, but the redirect leads onto a page of another kind entirely — the
    /// `by_page_title` half of decision 3, tried before the article fallback.
    #[test]
    fn a_concept_through_a_redirect_onto_another_kind_resolves_to_that_kind() {
        let mut with_redirect = raw();
        with_redirect.pages[0].text =
            "{{infobox passive collectible|id=25}}\n== Effects ==\n[[Brekkie]] restores.\n".into();
        with_redirect
            .redirects
            .insert("Brekkie".to_string(), "Breakfast".to_string());
        let ds = build(&with_redirect, &Corrections::default());
        let breakfast = ds.entry(&Target::Item { id: 25 }).unwrap();
        assert!(
            breakfast.inlines().iter().any(|i| matches!(
                i,
                crate::Inline::Ref {
                    target: Target::Item { id: 25 },
                    ..
                }
            )),
            "{:?}",
            breakfast.inlines()
        );
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
        // `from_json` reads whatever pretty JSON serde produces for the whole `Dataset`; no
        // production code serializes it that way any more (`write_dir` writes one file per
        // collection), so the test builds its own input instead of a removed `to_json`.
        let s = serde_json::to_string_pretty(&ds).unwrap();
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
