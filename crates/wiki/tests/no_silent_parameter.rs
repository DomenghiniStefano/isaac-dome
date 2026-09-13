//! Every parameter the wiki writes in an infobox is either kept in a type or named in
//! `IGNORED_PARAMS` with a reason.
//!
//! The bug this guards against is the one fixed on 2026-09-13: `infobox_from` parsed every
//! parameter of a collectible and then returned a unit variant, so 907 of 1727 entries lost
//! their whole box — with the suite green, because nothing asserted that what was parsed was
//! also kept. A silent loss needs a test that counts, not one that samples.

use std::collections::BTreeMap;
use std::path::PathBuf;

use wiki::{extract_infoboxes, InfoboxKind, Raw, IGNORED_PARAMS};

/// Every parameter name some field of `Infobox` or `EntryFacts` reads. Kept by hand, on
/// purpose: a field added to a type without a line here trips the test, which is how a new
/// parameter gets noticed instead of silently joining the ignored pile.
const KEPT: &[&str] = &[
    // EntryFacts, common to every kind.
    "description",
    "dlc",
    "unlocked by",
    // Item and Trinket.
    "quote",
    "quality",
    "tags",
    "recharge",
    "devil price",
    "shop price",
    "pool",
    // Achievement.
    "requirements",
    "notes",
    "unlocks",
    // Boss.
    "base hp",
    "stage hp",
    "variant",
    "environment",
    // Challenge.
    "blindfolded",
    "has shops",
    "has treasure rooms",
    "item",
    "trinket",
    "pickup",
    "health",
    "curse",
    "goal",
    "character",
    // Character.
    "damage",
    "tears",
    "range",
    "speed",
    "luck",
    "shot speed",
    "pickups",
    "collectibles",
    "parent",
    // Transformation. `requirement` is deliberately NOT here: it is in `IGNORED_PARAMS`,
    // because all sixteen rows hold the same template default and the count is read from
    // the page body instead.
    "items",
    "target",
];

#[test]
fn every_wikitext_parameter_is_kept_or_declared_ignored() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/raw");
    let raw = Raw::load(&root).expect("dataset/raw/");

    let mut unaccounted: BTreeMap<String, u32> = BTreeMap::new();
    let mut seen = 0usize;
    for page in &raw.pages {
        for ib in extract_infoboxes(&page.text) {
            // Only the infoboxes we convert: an unrecognized template is a different
            // question, and `unknownTemplates` is where that one is counted.
            if InfoboxKind::of(&ib.name).is_none() {
                continue;
            }
            for key in ib.params.keys() {
                seen += 1;
                let k = key.as_str();
                if !KEPT.contains(&k) && !IGNORED_PARAMS.contains(&k) {
                    *unaccounted.entry(k.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    // Non-vacuity: an empty snapshot, or an `extract_infoboxes` that stopped finding
    // anything, would leave `unaccounted` empty and pass while proving nothing.
    assert!(
        seen > 5000,
        "only {seen} infobox parameters read: the snapshot or the extractor is broken, \
         and an empty result here would mean nothing"
    );
    assert!(
        unaccounted.is_empty(),
        "parameters neither kept in a type nor declared ignored: {unaccounted:?}"
    );
}

#[test]
fn the_two_lists_do_not_overlap() {
    // A name in both lists reads as "kept" and hides the fact that nothing keeps it — the
    // same silence this file exists to prevent, one level up.
    let both: Vec<&&str> = KEPT.iter().filter(|k| IGNORED_PARAMS.contains(k)).collect();
    assert!(both.is_empty(), "in both KEPT and IGNORED_PARAMS: {both:?}");
}
