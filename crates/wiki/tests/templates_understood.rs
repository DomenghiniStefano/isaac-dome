//! The closing criterion of the 2026-09-13 spec, as a property instead of a note: no
//! template appears more than fifty times without the parser understanding it.
//!
//! Below that line the remaining ones are listed in the spec, each with the reason it stays
//! out. The line is not sacred — what it buys is that a template growing into a common one,
//! on a wiki that keeps being edited, cannot do it quietly.

use std::path::PathBuf;

use wiki::{build, Corrections, Raw};

const LIMIT: u32 = 50;

#[test]
fn no_frequent_template_is_left_unknown() {
    // The same three lines as `derived.rs`: one way to build the dataset, not two.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(
        &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
    )
    .expect("json");
    let dataset = build(&raw, &corrections);
    let meta = &dataset.meta;

    // Non-vacuity: a build that parsed nothing would have an empty map and pass while
    // proving nothing. 719 items today; anything under 700 means the snapshot is broken.
    assert!(
        meta.counts.items > 700,
        "only {} items built: too few to trust an empty result",
        meta.counts.items
    );

    let frequent: Vec<(&String, &u32)> = meta
        .diagnostics
        .unknown_templates
        .iter()
        .filter(|(_, n)| **n > LIMIT)
        .collect();
    assert!(
        frequent.is_empty(),
        "templates above {LIMIT} occurrences are still unknown: {frequent:?}"
    );
}

/// The other half, and the one that actually measures progress: the total. On 2026-09-13 the
/// parser did not understand 25 templates over 1290 occurrences; teaching it seven of them
/// brought that to 17 over 200. A ceiling here means a regression — or a fetch that pulled in
/// a wiki that has started using something new — has to be looked at rather than absorbed.
#[test]
fn the_unknown_templates_stay_few() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(
        &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
    )
    .expect("json");
    let dataset = build(&raw, &corrections);

    let total: u32 = dataset
        .meta
        .diagnostics
        .unknown_templates
        .values()
        .sum::<u32>();
    assert!(
        dataset.meta.counts.items > 700,
        "the build produced too few items to trust this count"
    );
    // 200 today. The headroom is for a snapshot that adds pages, not for letting the number
    // drift back up: if this fails, read what grew before raising it.
    assert!(
        total < 400,
        "{total} unknown-template occurrences, up from the 200 this was left at: \
         {:?}",
        dataset.meta.diagnostics.unknown_templates
    );
}
