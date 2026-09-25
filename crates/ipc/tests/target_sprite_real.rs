//! Real coverage of `target_sprite`: how many references in the wiki dataset turn into
//! an image from the installed game.
//!
//! Runs against the real catalog (`samples/packed`) and the embedded dataset. It's a
//! **threshold property**, not a pinned value: the thresholds come from the structure
//! of the data — every item has an icon, every boss `bossportraits.xml` has a row for is
//! reachable — and they hold even as the game adds things.
//!
//! It also serves another purpose: the numbers it prints are the ones `DESIGN-BRIEF.md`
//! uses to tell the design side how much it will be able to illustrate. If they change,
//! it shows up here.

// A test that extracts one variant panics on every other, the ones added later included: here
// the wildcard *is* the assertion, and it fails loudly on a new variant instead of hiding it.
#![allow(clippy::wildcard_enum_match_arm)]

use std::collections::{BTreeMap, BTreeSet};

use catalog::Catalog;
use ipc::{target_sprite, Target, TargetSprite};
use unpack::ResourceSet;

/// The family of a target, to group the count.
fn family(t: &Target) -> &'static str {
    match t {
        Target::Item { .. } => "item",
        Target::Trinket { .. } => "trinket",
        Target::Character { .. } => "character",
        Target::Achievement { .. } => "achievement",
        Target::Challenge { .. } => "challenge",
        Target::Entity { .. } => "entity",
        Target::Transformation { .. } => "transformation",
        Target::Stage { .. } => "stage",
        Target::Room { .. } => "room",
        Target::Concept { .. } => "concept",
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Counts {
    found: usize,
    no_art: usize,
    unknown: usize,
}

impl Counts {
    fn total(&self) -> usize {
        self.found + self.no_art + self.unknown
    }
}

/// The target of every page in the dataset. Boss pages are indexed with the game's
/// `type.variant.subtype` notation: it's read back from there.
fn pages() -> Vec<Target> {
    let Ok(ds) = wiki::Dataset::embedded() else {
        return Vec::new();
    };
    let mut out: Vec<Target> = Vec::new();
    out.extend(ds.items.keys().map(|id| Target::Item { id: *id }));
    out.extend(ds.trinkets.keys().map(|id| Target::Trinket { id: *id }));
    out.extend(
        ds.achievements
            .keys()
            .map(|id| Target::Achievement { id: *id }),
    );
    out.extend(
        ds.challenges
            .keys()
            .map(|n| Target::Challenge { number: *n }),
    );
    out.extend(ds.characters.keys().map(|id| Target::Character { id: *id }));
    out.extend(ds.bosses.keys().filter_map(|k| {
        let mut p = k.split('.').map(|n| n.parse::<u32>());
        match (p.next(), p.next(), p.next()) {
            (Some(Ok(id)), Some(Ok(variant)), Some(Ok(subtype))) => Some(Target::Entity {
                id,
                variant,
                subtype,
            }),
            _ => None,
        }
    }));
    out
}

/// Walks the dataset's pages and counts how **the page itself** (its target) resolves,
/// per family.
fn page_coverage(c: &Catalog) -> BTreeMap<&'static str, Counts> {
    let mut per_family: BTreeMap<&'static str, Counts> = BTreeMap::new();
    for t in pages() {
        let entry = per_family.entry(family(&t)).or_default();
        match target_sprite(c, &ipc::for_tests::bosses(c), &t) {
            TargetSprite::Found(_) => entry.found += 1,
            TargetSprite::NoArt => entry.no_art += 1,
            TargetSprite::Unknown => entry.unknown += 1,
        }
    }
    per_family
}

fn real_catalog() -> Option<Catalog> {
    let packed = test_support::packed_dir()?;
    let rs = ResourceSet::open(&packed);
    Some(Catalog::build(|p| rs.read(p)))
}

#[test]
fn every_wiki_page_that_is_an_item_or_a_trinket_has_its_icon() {
    let Some(c) = real_catalog() else { return };
    let coverage = page_coverage(&c);
    for family in ["item", "trinket"] {
        let n = coverage.get(family).copied().unwrap_or_default();
        eprintln!("coverage {family}: {n:?}");
        assert!(n.total() > 100, "{family}: is the dataset not loaded?");
        // Every collectible and every trinket in the game has a `gfx` in `items.xml`: if
        // any ends up without an icon, either the wiki cites an id the game doesn't
        // have, or the catalog failed to read `items.xml`. Both must be seen, not
        // tolerated.
        assert_eq!(
            n.no_art, 0,
            "{family}: none can be 'without art', the icon is mandatory in the file"
        );
        assert!(
            n.unknown * 100 < n.total(),
            "{family}: more than 1% of ids the catalog doesn't know ({n:?})"
        );
    }
}

#[test]
fn most_boss_pages_reach_their_portrait_through_the_entity_key() {
    let Some(c) = real_catalog() else { return };
    let coverage = page_coverage(&c);
    let n = coverage.get("entity").copied().unwrap_or_default();
    eprintln!("coverage entity: {n:?}");
    if n.total() == 0 {
        test_support::skip("the dataset has no entity pages");
        return;
    }
    // The wiki's `entity` pages aren't just bosses: they include common enemies, which
    // by definition don't have a portrait. The property that holds is that the bosses
    // the game illustrates are reachable — not that every entity has a picture.
    //
    // The threshold was `> 50` while the key was read from the portrait's file name alone
    // and 75 of 102 resolved. Since the page's own title decides the key (2026-09-21) the
    // only pages left without a picture are the ones `bossportraits.xml` has no row for at
    // all — four, the Ultra Harbingers — so the structural statement is "all but a
    // handful", which nine tenths expresses without pinning a number that a patch moves.
    assert!(
        n.found * 10 > n.total() * 9,
        "boss portraits must stay reachable from the entity key ({n:?})"
    );
}

/// Case, spaces and punctuation dropped, and a leading `the` with them: `normalized` in
/// `target_sprite`, written again here so the test compares against the rule rather than
/// against the implementation of it.
fn normalized(s: &str) -> String {
    let n: String = s
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    n.strip_prefix("the").map_or(n.clone(), str::to_string)
}

/// `…/Portrait_<type>.<variant>_<Name>.png` → `(type, variant)`.
fn declared_key(path: &str) -> Option<(u32, u32)> {
    let file = path.rsplit(['/', '\\']).next()?;
    let (kind, variant) = file
        .strip_prefix("Portrait_")?
        .split('_')
        .next()?
        .split_once('.')?;
    Some((kind.parse().ok()?, variant.parse().ok()?))
}

#[test]
fn a_contested_key_draws_the_boss_whose_page_it_is_and_not_its_neighbour() {
    let Some(c) = real_catalog() else { return };
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("wiki dataset not embedded");
        return;
    };
    // Some keys are written into the file name of **two** portraits: `45.0` by *Mom* and
    // by *Mom (Mausoleum)*, and the same for `78.0`, `406.0` and `902.0`. Collecting the
    // portraits into a map by that key let the last row win, in silence, and a boss was
    // drawn with another's face — a defect that never looks like one, because the picture
    // it shows is a real picture of a real boss.
    //
    // The contested set is read from the catalog, never listed here: a patch that contests
    // a fifth key has to be covered the day it ships.
    let mut claims: BTreeMap<(u32, u32), Vec<&catalog::Boss>> = BTreeMap::new();
    for b in c.bosses() {
        if let Some(k) = declared_key(&b.portrait.path) {
            claims.entry(k).or_default().push(b);
        }
    }
    let contested: Vec<_> = claims.into_iter().filter(|(_, v)| v.len() > 1).collect();
    // Vacuity guard: on a roster where no key is contested this test proves nothing, and
    // must say so rather than pass.
    assert!(
        !contested.is_empty(),
        "no key is claimed twice: this test is checking nothing"
    );

    let mut checked = 0usize;
    for (key, rows) in &contested {
        let (id, variant) = *key;
        let page = ds.bosses.get(&wiki::Dataset::boss_key(id, variant, 0));
        let Some(page) = page else { continue };
        // The row the page is about: the one whose own name is the page's title.
        let owner = rows
            .iter()
            .find(|b| normalized(&b.name) == normalized(&page.title));
        let Some(owner) = owner else { continue };
        checked += 1;
        let t = Target::Entity {
            id,
            variant,
            subtype: 0,
        };
        let path = match target_sprite(&c, &ipc::for_tests::bosses(&c), &t) {
            TargetSprite::Found(s) => s.path.clone(),
            other => panic!("{}.{} ({}) resolved to {other:?}", id, variant, page.title),
        };
        assert_eq!(
            path,
            owner.portrait.path,
            "{}.{} is {}'s page: {:?} also claims the key and must not be drawn for it",
            id,
            variant,
            page.title,
            rows.iter().map(|b| &b.name).collect::<Vec<_>>()
        );
    }
    eprintln!(
        "sample: {checked} contested keys checked out of {} ",
        contested.len()
    );
    assert!(
        checked >= 4,
        "only {checked} contested keys had a page to compare against"
    );
}

#[test]
fn the_pages_a_portrait_file_name_cannot_reach_are_reached_by_the_title() {
    let Some(c) = real_catalog() else { return };
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("wiki dataset not embedded");
        return;
    };
    // The pages whose key **no** portrait writes into its file name. Reading the file name
    // alone left every one of them without a picture — 20 of the 27 that had none on
    // 2026-09-21, Dogma and Mother and Delirium among them. They are reachable only
    // because the page's own title decides the key, so they are the property to hold.
    let declared: BTreeSet<(u32, u32)> = c
        .bosses()
        .filter_map(|b| declared_key(&b.portrait.path))
        .collect();
    let mut recovered = 0usize;
    let mut still_missing: Vec<&str> = Vec::new();
    for (key, entry) in &ds.bosses {
        let mut p = key.split('.').map(|n| n.parse::<u32>());
        let (Some(Ok(id)), Some(Ok(variant)), Some(Ok(subtype))) = (p.next(), p.next(), p.next())
        else {
            continue;
        };
        if declared.contains(&(id, variant)) {
            continue;
        }
        match target_sprite(
            &c,
            &ipc::for_tests::bosses(&c),
            &Target::Entity {
                id,
                variant,
                subtype,
            },
        ) {
            TargetSprite::Found(_) => recovered += 1,
            TargetSprite::NoArt | TargetSprite::Unknown => still_missing.push(&entry.title),
        }
    }
    eprintln!("sample: {recovered} boss pages no file name reaches, recovered by title");
    assert!(
        recovered >= 20,
        "only {recovered} recovered by title: the name join is not doing its work"
    );
    // What is left is the pages `bossportraits.xml` has no row for at all — the Ultra
    // Harbingers. No name join reaches those; `entities2.xml` is the card for them.
    still_missing.sort_unstable();
    assert_eq!(
        still_missing,
        [
            "Ultra Death",
            "Ultra Famine",
            "Ultra Pestilence",
            "Ultra War"
        ],
        "the only boss pages with no portrait row of any kind"
    );
}

#[test]
fn the_coverage_of_every_family_is_declared_not_guessed() {
    let Some(c) = real_catalog() else { return };
    let coverage = page_coverage(&c);
    if coverage.is_empty() {
        test_support::skip("wiki dataset not embedded");
        return;
    }
    // This test imposes no thresholds: it **declares**. This is the number that ends
    // up in the brief, and printing it every run is how a change gets noticed.
    let mut total = Counts::default();
    for (family, n) in &coverage {
        eprintln!(
            "sample: wiki/{family} — {} with image, {} without art, {} unknown ids (out of {})",
            n.found,
            n.no_art,
            n.unknown,
            n.total()
        );
        total.found += n.found;
        total.no_art += n.no_art;
        total.unknown += n.unknown;
    }
    eprintln!(
        "sample: wiki/all — {} pages out of {} have an image",
        total.found,
        total.total()
    );
    // The one property: the majority of pages are illustrable. If it drops below that,
    // the design's Wiki section needs rethinking, and that must be known before it's
    // designed.
    assert!(
        total.found * 2 > total.total(),
        "less than half of wiki pages have an image: {total:?}"
    );
}
