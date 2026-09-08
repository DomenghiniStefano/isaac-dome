//! Real coverage of `target_sprite`: how many references in the wiki dataset turn into
//! an image from the installed game.
//!
//! Runs against the real catalog (`samples/packed`) and the embedded dataset. It's a
//! **threshold property**, not a pinned value: the thresholds come from the structure
//! of the data — every item has an icon, every boss with `<type>.<variant>` in its
//! portrait's name is reachable — and they hold even as the game adds things.
//!
//! It also serves another purpose: the numbers it prints are the ones `DESIGN-BRIEF.md`
//! uses to tell the design side how much it will be able to illustrate. If they change,
//! it shows up here.

use std::collections::BTreeMap;

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
        Target::Pickup { .. } => "pickup",
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
        match target_sprite(c, &t) {
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
    assert!(
        n.found > 50,
        "boss portraits must stay reachable from the entity key ({n:?})"
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
