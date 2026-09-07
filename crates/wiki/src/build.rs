//! From `Raw` to `Dataset`: the resolver, the pages in order, the `meta`. Deterministic:
//! same `raw/`, same JSON.

use std::collections::BTreeMap;

use crate::dataset::{Counts, Dataset, Patch, Source};
use crate::infobox::{extract_infoboxes, InfoboxKind};
use crate::page::{parse_page, EntryKey, PageKind};
use crate::raw::{Raw, RawPage};
use crate::resolver::{Corrections, Resolver};
use crate::Diagnostics;

/// Characters have no Cargo table: title (and the infobox's `name`) → id is derived from
/// their pages, before building the resolver.
fn characters(raw: &Raw) -> BTreeMap<String, u32> {
    let mut characters = BTreeMap::new();
    for p in raw
        .pages
        .iter()
        .filter(|p| p.index.kind == PageKind::Character)
    {
        let Some(ib) = extract_infoboxes(&p.text)
            .into_iter()
            .find(|ib| InfoboxKind::of(&ib.name) == Some(InfoboxKind::Character))
        else {
            continue;
        };
        let Some(id) = ib
            .params
            .get("id")
            .and_then(|s| s.trim().parse::<u32>().ok())
        else {
            continue;
        };
        characters.insert(p.title.clone(), id);
        if let Some(alias) = ib.params.get("name") {
            characters.entry(alias.trim().to_string()).or_insert(id);
        }
    }
    characters
}

/// Builds the dataset. Pages are visited in (kind, title) order; a key that recurs keeps
/// the first entry.
pub fn build(raw: &Raw, corrections: &Corrections) -> Dataset {
    let characters = characters(raw);
    let r = Resolver::new(&raw.tables, &characters, corrections);
    let mut ds = Dataset::empty();
    let mut diagnostics = Diagnostics::default();
    let mut pages: Vec<&RawPage> = raw.pages.iter().collect();
    pages.sort_by(|a, b| (a.index.kind, &a.title).cmp(&(b.index.kind, &b.title)));
    for p in pages {
        let mut d = Diagnostics::default();
        for (key, entry) in parse_page(&p.title, p.index.revid, &p.text, &r, &mut d) {
            match key {
                EntryKey::Item(id) => {
                    ds.items.entry(id).or_insert(entry);
                }
                EntryKey::Trinket(id) => {
                    ds.trinkets.entry(id).or_insert(entry);
                }
                EntryKey::Achievement(id) => {
                    ds.achievements.entry(id).or_insert(entry);
                }
                EntryKey::Boss(id, variant, subtype) => {
                    ds.bosses
                        .entry(Dataset::boss_key(id, variant, subtype))
                        .or_insert(entry);
                }
                EntryKey::Challenge(number) => {
                    ds.challenges.entry(number).or_insert(entry);
                }
                EntryKey::Character(id) => {
                    ds.characters.entry(id).or_insert(entry);
                }
            }
        }
        diagnostics.merge(&d);
        if p.index.timestamp > ds.meta.snapshot_at {
            ds.meta.snapshot_at = p.index.timestamp.clone();
        }
        ds.meta.max_revid = ds.meta.max_revid.max(p.index.revid);
    }
    ds.meta.last_known_patch = raw
        .versions
        .iter()
        .filter_map(|v| Some((v.get("date")?.clone(), v.get("number")?.clone())))
        .max()
        .map(|(date, number)| Patch { number, date });
    ds.meta.source = Source {
        name: "The Binding of Isaac: Rebirth Wiki".into(),
        url: "https://bindingofisaacrebirth.wiki.gg".into(),
        license: "CC BY-SA 4.0".into(),
    };
    ds.meta.counts = Counts {
        items: ds.items.len() as u32,
        trinkets: ds.trinkets.len() as u32,
        achievements: ds.achievements.len() as u32,
        bosses: ds.bosses.len() as u32,
        challenges: ds.challenges.len() as u32,
        characters: ds.characters.len() as u32,
    };
    ds.meta.diagnostics = diagnostics;
    ds
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

    #[test]
    fn json_roundtrip_and_schema_check() {
        let ds = build(&raw(), &Corrections::default());
        let s = ds.to_json();
        assert_eq!(Dataset::from_json(&s).unwrap(), ds);
        let bad = s.replacen(r#""schemaVersion": 1"#, r#""schemaVersion": 99"#, 1);
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
