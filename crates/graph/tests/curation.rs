//! The curation is complete for the snapshot in the repo, and it stays complete: when a
//! new snapshot introduces a target nobody has judged, this test names it. The red is the
//! point — a warning would mean nodes dropping to `Partial` with nobody noticing.

use graph::rules::{Corrections, Requirements, Rules};
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
fn every_target_that_needs_a_verdict_has_one() {
    let rules = rules();
    let missing: Vec<String> = rules
        .targets()
        .iter()
        .filter(|t| t.verdict_required && rules.verdict(&t.key).is_none())
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
fn the_alias_that_carries_fifteen_refs_is_there() {
    // The wiki writes "Jacob and Esau", the game writes "Jacob & Esau": 15 refs hang on
    // this one line, measured on 2026-09-07 against snapshot 2026-09-04T17:33:31Z.
    assert_eq!(rules().alias("Jacob and Esau"), "Jacob & Esau");
}
