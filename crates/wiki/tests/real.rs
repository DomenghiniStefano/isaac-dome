//! A pass over the real snapshot in `dataset/raw/`: it lives in the repo, so no skipping.
//! The thresholds are the 2026-09-05 build's values with a small margin: a parser that
//! loses ground shows up here, not in production.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use wiki::{
    build, Block, Corrections, Dataset, Entry, Infobox, Inline, Raw, Resolution, Resolver,
    SectionKind, Target,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset")
}

fn raw() -> Raw {
    Raw::load(&root().join("raw")).expect("dataset/raw/ in the repo")
}

fn corrections() -> Corrections {
    let text = std::fs::read_to_string(root().join("corrections.json"))
        .expect("dataset/corrections.json in the repo");
    serde_json::from_str(&text).expect("corrections.json is the expected map")
}

/// The snapshot's build, once per process: it costs a second and every test reads it.
fn dataset() -> &'static Dataset {
    static DATASET: OnceLock<Dataset> = OnceLock::new();
    DATASET.get_or_init(|| build(&raw(), &corrections()))
}

fn refs_to_items(inline: &[Inline]) -> usize {
    inline
        .iter()
        .map(|i| match i {
            Inline::Ref {
                target: Target::Item { .. },
                ..
            } => 1,
            Inline::Edition { inline, .. } => refs_to_items(inline),
            Inline::Ref { .. } | Inline::Text { .. } | Inline::Concept { .. } => 0,
        })
        .sum()
}

#[test]
fn counts_match_the_index() {
    let ds = dataset();
    let c = &ds.meta.counts;
    // 720 collectible pages: G FUEL! (id -1) isn't an item and Tonsil is a trinket.
    assert!((714..=724).contains(&c.items), "items {}", c.items);
    assert_eq!(c.trinkets, 188);
    // One per Cargo table row.
    assert!(
        (636..=646).contains(&c.achievements),
        "achievements {}",
        c.achievements
    );
    // 102 pages, with Ultra Greedier colliding with Ultra Greed on the same key.
    assert!((97..=107).contains(&c.bosses), "bosses {}", c.bosses);
    assert_eq!(c.challenges, 45);
    // 30 pages and 32 infoboxes, with our own map replacing the wiki's wrong ids.
    assert!(
        (25..=35).contains(&c.characters),
        "characters {}",
        c.characters
    );
    assert!(ds.meta.max_revid > 0 && !ds.meta.snapshot_at.is_empty());
    assert!(ds.meta.last_known_patch.is_some());
}

#[test]
fn tonsil_is_a_trinket_and_474_is_broken_glass_cannon() {
    let ds = dataset();
    assert_eq!(
        ds.entry(&Target::Trinket { id: 97 })
            .map(|e| e.title.as_str()),
        Some("Tonsil")
    );
    assert_eq!(
        ds.entry(&Target::Item { id: 474 })
            .map(|e| e.title.as_str()),
        Some("Broken Glass Cannon")
    );
    let raw = raw();
    let r = Resolver::new(&raw.tables, &BTreeMap::new(), &corrections());
    assert_eq!(r.resolve("i", "Tonsil"), Resolution::Unresolved);
    assert_eq!(
        r.resolve("t", "Tonsil"),
        Resolution::Target(Target::Trinket { id: 97 })
    );
}

#[test]
fn characters_key_by_our_map_not_by_the_wiki_ids() {
    let ds = dataset();
    let title = |id| {
        ds.entry(&Target::Character { id })
            .map(|e| e.title.as_str())
    };
    // The wiki writes 14 for Isaac and 2 for Magdalene: without our own map, Keeper and
    // Magdalene would vanish behind Isaac and Cain.
    assert_eq!(title(0), Some("Isaac"));
    assert_eq!(title(1), Some("Magdalene"));
    assert_eq!(title(2), Some("Cain"));
    assert_eq!(title(14), Some("Keeper"));
    assert_eq!(title(4), Some("???"));
    assert_eq!(title(11), Some("Lazarus Risen"));
    assert_eq!(title(12), Some("Black Judas"));
}

#[test]
fn broken_shovel_page_yields_both_halves() {
    let ds = dataset();
    for id in [550, 551] {
        let e = ds.entry(&Target::Item { id }).expect("Broken Shovel");
        assert_eq!(e.title, "Broken Shovel");
        assert!(!e.sections.is_empty());
    }
}

#[test]
fn binge_eater_notes_reference_eight_food_items() {
    let ds = dataset();
    let e = ds.entry(&Target::Item { id: 664 }).expect("Binge Eater");
    let notes = e
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Notes)
        .expect("Notes");
    let Block::List { items, .. } = &notes.blocks[0] else {
        panic!("the first note is a list")
    };
    let Block::List { items: foods, .. } = &items[0].children[0] else {
        panic!("nested list of foods")
    };
    assert_eq!(foods.len(), 8);
    assert_eq!(
        foods
            .iter()
            .map(|f| refs_to_items(&f.inline))
            .sum::<usize>(),
        8
    );
}

#[test]
fn false_phd_has_the_pill_table() {
    let ds = dataset();
    let e = ds.entry(&Target::Item { id: 654 }).expect("False PHD");
    let effects = e
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Effects)
        .expect("Effects");
    let (header, rows) = effects
        .blocks
        .iter()
        .find_map(|b| match b {
            Block::Table { header, rows } => Some((header, rows)),
            Block::Paragraph { .. } | Block::List { .. } | Block::Heading { .. } => None,
        })
        .expect("table");
    assert_eq!(header.len(), 2);
    // From the wikitext: the `! Pill !! Changes Into` line is the header; then 22 `|-`,
    // that is the "Stat up pill" row, 17 pill pairs, the "Neutral pills" title row, and 3
    // neutral pills.
    assert_eq!(rows.len(), 22);
}

#[test]
fn hush_has_phase_headings() {
    let ds = dataset();
    let hush = ds
        .bosses
        .values()
        .find(|e| e.title == "Hush")
        .expect("Hush");
    let behavior = hush
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Behavior)
        .expect("Behavior");
    let levels: Vec<u8> = behavior
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Heading { level, .. } => Some(*level),
            Block::Paragraph { .. } | Block::List { .. } | Block::Table { .. } => None,
        })
        .collect();
    assert!(levels.contains(&3) && levels.contains(&4));
}

#[test]
fn achievement_62_unlocks_epic_fetus_by_beating_challenge_19() {
    let ds = dataset();
    let e = ds
        .entry(&Target::Achievement { id: 62 })
        .expect("achievement 62");
    assert_eq!(e.title, "Epic Fetus");
    let Infobox::Achievement {
        requirements,
        unlocks,
        ..
    } = &e.infobox
    else {
        panic!("infobox achievement")
    };
    // The wiki writes the requirement as a wikilink, `[[The Family Man|… (challenge #19)]]`:
    // the page is a challenge's, so it's a reference just like `{{chal|…}}`.
    assert!(
        requirements.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Challenge { number: 19 },
                ..
            }
        )),
        "{requirements:?}"
    );
    assert_eq!(*unlocks, Some(Target::Item { id: 168 }));
}

#[test]
fn achievement_349_reads_its_pc_id_before_the_ps4_note() {
    let ds = dataset();
    let e = ds
        .entry(&Target::Achievement { id: 349 })
        .expect("achievement 349");
    assert_eq!(e.title, "Black Hole");
}

#[test]
fn diagnostics_are_bounded() {
    let ds = dataset();
    let d = &ds.meta.diagnostics;
    // 19 as of 2026-09-08 (18 `{{e|…}}`, 1 `{{i|…}}`), down from 22, and what remains is
    // a **floor**, not a backlog — which is why the bounds below are per template and
    // tight, instead of one loose sum:
    //
    // - the 18 `{{e|…}}` are three keys, `Killswitch`, `Pressure Plate` and
    //   `Reward Plate`. All three *are* in `entity.json`, as aliases of the page
    //   `Buttons`, and all three carry `id: ""` — they're grid entities, which the game
    //   gives no `EntityType`. `Target::Entity` needs an id and we can't invent one, so
    //   resolving them would take a `Target` variant for grid entities: a contract
    //   change, not a resolver fix.
    // - the 1 `{{i|…}}` is `Tonsil`, unresolved **on purpose** — it's a trinket, not an
    //   item, and `tonsil_is_a_trinket_and_474_is_broken_glass_cannon` pins exactly that.
    //   If this one ever reaches zero, the bug is there and not here.
    //
    // The three that went away were `{{i|1=Name}}`, MediaWiki's explicit positional
    // syntax, which `assemble` used to file under `named` leaving `args` empty.
    let unresolved: u32 = d.unresolved.values().sum();
    assert!(
        unresolved <= 20,
        "unresolved {unresolved}: {:?}",
        d.unresolved
    );
    assert!(
        d.unresolved.get("i").copied().unwrap_or(0) <= 1,
        "only Tonsil may stay an unresolved item: {:?}",
        d.unresolved
    );
    assert!(
        d.unresolved.get("e").copied().unwrap_or(0) <= 18,
        "the unresolved entities are the three id-less buttons: {:?}",
        d.unresolved
    );
    assert_eq!(d.unresolved.get("c"), None, "{:?}", d.unresolved);
    // G FUEL! (id -1) and Tonsil's collectible 474, from another edition.
    assert!(
        d.pages_without_id <= 10,
        "without id {}",
        d.pages_without_id
    );
}

/// Every `Text` with a literal `{{`/`}}` inside `inline`, recursing into `Edition`.
fn raw_brace_texts(inline: &[Inline], out: &mut Vec<String>) {
    for i in inline {
        match i {
            Inline::Text { text, .. } if text.contains("{{") || text.contains("}}") => {
                out.push(text.clone());
            }
            Inline::Edition { inline, .. } => raw_brace_texts(inline, out),
            Inline::Text { .. } | Inline::Ref { .. } | Inline::Concept { .. } => {}
        }
    }
}

/// The same search, over a section's blocks: paragraphs, headings, lists (with their
/// nested children) and table cells.
fn raw_brace_texts_in_blocks(blocks: &[Block], out: &mut Vec<String>) {
    for b in blocks {
        match b {
            Block::Paragraph { inline } | Block::Heading { inline, .. } => {
                raw_brace_texts(inline, out);
            }
            Block::List { items, .. } => {
                for item in items {
                    raw_brace_texts(&item.inline, out);
                    raw_brace_texts_in_blocks(&item.children, out);
                }
            }
            Block::Table { header, rows } => {
                for cell in header {
                    raw_brace_texts(cell, out);
                }
                for row in rows {
                    for cell in row {
                        raw_brace_texts(cell, out);
                    }
                }
            }
        }
    }
}

/// The infoboxes' `Vec<Inline>` fields: the only ones that can carry text from the wikitext.
fn raw_brace_texts_in_infobox(infobox: &Infobox, out: &mut Vec<String>) {
    match infobox {
        Infobox::Item {
            recharge,
            devil_price,
            shop_price,
            pools,
            ..
        } => {
            raw_brace_texts(recharge, out);
            raw_brace_texts(devil_price, out);
            raw_brace_texts(shop_price, out);
            raw_brace_texts(pools, out);
        }
        Infobox::Trinket { pools, .. } => raw_brace_texts(pools, out),
        Infobox::Achievement { requirements, .. } => raw_brace_texts(requirements, out),
        Infobox::Boss {
            environment, pool, ..
        } => {
            raw_brace_texts(environment, out);
            raw_brace_texts(pool, out);
        }
        Infobox::Challenge {
            items,
            trinkets,
            pickups,
            health,
            curse,
            goal,
            ..
        } => {
            for v in [items, trinkets, pickups, health, curse, goal] {
                raw_brace_texts(v, out);
            }
        }
        Infobox::Character {
            health,
            pickups,
            collectibles,
            ..
        } => {
            for v in [health, pickups, collectibles] {
                raw_brace_texts(v, out);
            }
        }
    }
}

fn raw_brace_texts_in_entry(e: &Entry, out: &mut Vec<String>) {
    raw_brace_texts_in_infobox(&e.infobox, out);
    for s in &e.sections {
        raw_brace_texts_in_blocks(&s.blocks, out);
    }
}

#[test]
fn text_nodes_carry_no_raw_template_syntax() {
    let ds = dataset();
    let mut offenders = Vec::new();
    for e in ds
        .items
        .values()
        .chain(ds.trinkets.values())
        .chain(ds.achievements.values())
        .chain(ds.bosses.values())
        .chain(ds.challenges.values())
        .chain(ds.characters.values())
    {
        raw_brace_texts_in_entry(e, &mut offenders);
    }
    // 83 as of 2026-09-08 (125 → 123 → 83), and the remainder is now two things only:
    //
    // 75 belong to **one open defect**: a template whose content is block-level, which
    // opens on a list line and closes lines below. `blocks.rs` walks the wikitext line by
    // line, so `parse_template_at` never sees it as a single template. What's left of it
    // in the tree is 62 openers — `column list` 51, `Book of Virtues synergy` 6, `bug` 4,
    // `Book of Belial synergy` 1 — plus 13 tails, a `}}` riding at the end of the last
    // list item rather than on a line of its own.
    // Closing it properly needs a way to say "a template wrapping blocks", and that is a
    // `Block` variant, which crosses the IPC. `column list` is pure layout and could be
    // dropped; `{{bug|…}}` is not, and the crate already models it specially in the
    // single-line case. Choosing between transparent and modelled is a decision for the
    // wiki screen's design, not a parser detail, so it stays open on purpose.
    //
    // 8 are **genuine text and will never go to zero**: the `<math>…</math>` formulas on
    // Rosary and Mom's Contacts stay as text deliberately (no LaTeX parser here) and
    // their nested curly braces produce `}}` by coincidence. Keeper's page also carries a
    // wiki typo, `and}}` with nothing opening it.
    //
    // Closed on 2026-09-08, two families:
    // - `build_table` split a cell on `||` without counting `{{…}}` depth, so Mystery
    //   Egg's `{{e|Mask + Heart||Heart}}` became two half templates (2 entries).
    // - a line of nothing but `}}` became a `}}` paragraph *and* cut the list around it
    //   in two, because any non-list line flushes the list (40 entries). It's dropped and
    //   counted in `Diagnostics::orphan_closers`, at 50 on this snapshot.
    //
    // Threshold pinned on purpose: don't loosen it silently, and if it grows, understand
    // where it comes from before raising it. Each drop so far matched the size its family
    // predicted, which is the evidence the change hit that family and nothing else.
    assert!(
        offenders.len() <= 83,
        "{} nodes with raw template syntax: {offenders:?}",
        offenders.len()
    );
}

/// `corrections.json` is written by hand and every one of its entries is a claim about a
/// page that exists in `dataset/raw/`. Both files are committed together, so that claim
/// is checkable — and it needs to be, because a correction that matches nothing produces
/// no error, no warning and no effect: it just quietly doesn't happen.
///
/// Vacuous today: `pageId` is empty. That's the point of writing it now — it costs
/// nothing while there's nothing to break, and it's the first edit to that file that it
/// exists to catch. `the_correction_check_detects_a_correction_that_matches_nothing`
/// below is what proves the check itself works.
#[test]
fn every_page_correction_names_a_table_and_a_page_that_exist() {
    let corr = corrections();
    let raw = raw();
    let titles: BTreeMap<&str, ()> = raw.pages.iter().map(|p| (p.title.as_str(), ())).collect();

    assert!(
        corr.unknown_tables().is_empty(),
        "corrections for tables nothing consults, so they can never fire: {:?} (known: {:?})",
        corr.unknown_tables(),
        wiki::CORRECTED_TABLES
    );
    let orphans: Vec<(&str, &str)> = corr
        .page_id
        .iter()
        .flat_map(|(table, pages)| {
            pages
                .keys()
                .map(move |title| (table.as_str(), title.as_str()))
        })
        .filter(|(_, title)| !titles.contains_key(title))
        .collect();
    assert!(
        orphans.is_empty(),
        "corrections for pages that aren't in dataset/raw/: {orphans:?}"
    );
}

/// The check above is vacuous while `pageId` is empty, so this one runs it against a file
/// that is deliberately wrong in both ways. Without it, the guard could be broken and
/// nobody would know until it was needed.
#[test]
fn the_correction_check_detects_a_correction_that_matches_nothing() {
    let mut corr = Corrections::default();
    corr.page_id.insert(
        "collectibles".into(), // the real table is singular
        [("Breakfast".to_string(), 25u32)].into_iter().collect(),
    );
    corr.page_id.insert(
        "collectible".into(),
        [("No Such Page At All".to_string(), 1u32)]
            .into_iter()
            .collect(),
    );

    assert_eq!(corr.unknown_tables(), vec!["collectibles"]);

    let raw = raw();
    let titles: BTreeMap<&str, ()> = raw.pages.iter().map(|p| (p.title.as_str(), ())).collect();
    let orphans: Vec<&str> = corr
        .page_id
        .values()
        .flat_map(|pages| pages.keys().map(String::as_str))
        .filter(|title| !titles.contains_key(title))
        .collect();
    assert!(
        orphans.contains(&"No Such Page At All"),
        "the page check has to see it: {orphans:?}"
    );
}
