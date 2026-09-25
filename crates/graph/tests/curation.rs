//! The curation is complete for the snapshot in the repo, and it stays complete: when a
//! new snapshot introduces a target nobody has judged, this test names it. The red is the
//! point — a warning would mean nodes dropping to `Partial` with nobody noticing.

use graph::rules::{Corrections, MarkColumn, MarkLevel, Requirements, Rules, Verdict};
use std::path::Path;

fn rules() -> Rules {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    let r: Requirements = serde_json::from_str(
        &std::fs::read_to_string(dir.join("requirements.json")).expect("read"),
    )
    .expect("requirements.json parses");
    let c: Corrections =
        serde_json::from_str(&std::fs::read_to_string(dir.join("corrections.json")).expect("read"))
            .expect("corrections.json parses");
    Rules::build(r, c).expect("rules build")
}

#[test]
fn every_target_has_a_verdict() {
    let rules = rules();
    let missing: Vec<String> = rules
        .targets()
        .iter()
        .filter(|t| rules.verdict(&t.key).is_none())
        .map(|t| format!("{} ({} uses)", t.key, t.uses))
        .collect();
    assert!(
        missing.is_empty(),
        "{} targets have no verdict in corrections.json; judge each one \
         (alwaysAvailable / behind / notAPrerequisite / unknown):\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn no_verdict_names_a_target_that_no_longer_exists() {
    let rules = rules();
    let keys: std::collections::BTreeSet<&str> =
        rules.targets().iter().map(|t| t.key.as_str()).collect();
    // A verdict left behind by a snapshot that dropped its target is dead weight, and
    // worse, it hides that the judgement was never re-made for whatever replaced it.
    let orphans: Vec<&str> = rules
        .verdict_keys()
        .filter(|k| !keys.contains(*k))
        .collect();
    assert!(
        orphans.is_empty(),
        "verdicts for targets that are no longer in the inventory: {orphans:?}"
    );
}

#[test]
fn no_alias_restates_what_the_name_key_already_matches() {
    // Names are compared by `wiki::key`, which reads `&` as `and` and collapses whitespace:
    // an alias whose two sides share a key is dead weight that reads as a needed bridge.
    // "Jacob and Esau" -> "Jacob & Esau" was one, carrying fifteen refs, until card #82 (F4)
    // gave the graph the wiki's key. An empty table holds this trivially, and that is fine:
    // the check is on what a person adds, not on a series.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    let c: Corrections =
        serde_json::from_str(&std::fs::read_to_string(dir.join("corrections.json")).expect("read"))
            .expect("corrections.json parses");
    let dead: Vec<(&String, &String)> = c
        .aliases
        .iter()
        .filter(|(from, to)| wiki::key(from) == wiki::key(to))
        .collect();
    assert!(
        dead.is_empty(),
        "aliases the name key already covers: {dead:?}"
    );
}

// --- the five targets the profile answers (spec 2026-09-12, §4.2) ---------------------

/// Each of the five with the halves it must carry. `Ultra Greedier` has no located tally,
/// so it carries the mark alone: a `counter` here would be an index nobody measured.
#[test]
fn the_five_progress_targets_are_curated() {
    let rules = rules();
    let expected: &[(&str, MarkColumn, bool)] = &[
        ("entity:Hush", MarkColumn::Hush, true),
        ("entity:Delirium", MarkColumn::Delirium, true),
        ("entity:Mother", MarkColumn::Mother, true),
        ("entity:The Beast", MarkColumn::TheBeast, true),
        ("entity:Ultra Greedier", MarkColumn::Greed, false),
    ];
    for (key, column, has_counter) in expected {
        let Some(Verdict::Progress { mark, counter }) = rules.verdict(key) else {
            panic!("{key} must carry a progress verdict");
        };
        assert_eq!(mark.map(|m| m.column), Some(*column), "{key}");
        assert_eq!(counter.is_some(), *has_counter, "{key}");
    }
}

/// Ultra Greedier is the only one of the five whose level is not the base bit, and that is
/// the measurement of 2026-09-12: in the Greed column, bit 1 is Ultra Greedier. A
/// regression here silently turns 34 answered nodes back into partial ones.
#[test]
fn ultra_greedier_is_the_second_level_of_the_greed_column() {
    let rules = rules();
    let Some(Verdict::Progress { mark: Some(m), .. }) = rules.verdict("entity:Ultra Greedier")
    else {
        panic!("entity:Ultra Greedier must carry a mark");
    };
    assert_eq!(m.column, MarkColumn::Greed);
    assert_eq!(m.level, MarkLevel::Second);
}
