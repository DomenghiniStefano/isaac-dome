//! The two rules files: they read, they join, and a file from another schema is refused
//! whole rather than read half-broken.

use graph::rules::{Corrections, Requirements, Rules, Verdict, SCHEMA_VERSION};

const REQS: &str = r#"{
  "schemaVersion": 1,
  "generatedFrom": { "snapshotAt": "2026-09-04T17:33:31Z", "maxRevid": 269057 },
  "achievements": {
    "1": { "refs": [{ "target": { "kind": "entity", "id": 5, "variant": 10, "subtype": 1 },
                      "label": "Red Heart" }] }
  },
  "targets": [{ "key": "entity:Red Heart", "label": "Red Heart", "uses": 3,
               "verdictRequired": false }]
}"#;

const CORR: &str = r#"{
  "schemaVersion": 1,
  "aliases": { "Jacob and Esau": "Jacob & Esau" },
  "verdicts": { "entity:Red Heart": { "notAPrerequisite": true } }
}"#;

#[test]
fn reads_the_two_files_and_joins_them() {
    let r: Requirements = serde_json::from_str(REQS).expect("requirements parse");
    let c: Corrections = serde_json::from_str(CORR).expect("corrections parse");
    assert_eq!(r.schema_version, SCHEMA_VERSION);
    let rules = Rules::build(r, c).expect("rules build");
    assert_eq!(rules.alias("Jacob and Esau"), "Jacob & Esau");
    assert_eq!(
        rules.alias("Mom"),
        "Mom",
        "an unaliased label passes through unchanged"
    );
    assert_eq!(
        rules.verdict("entity:Red Heart"),
        Some(&Verdict::NotAPrerequisite(true))
    );
    assert_eq!(
        rules.verdict("stage:Nowhere"),
        None,
        "no verdict is not a verdict of 'none'"
    );
}

#[test]
fn a_file_from_another_schema_is_refused_whole() {
    let bumped = REQS.replace("\"schemaVersion\": 1", "\"schemaVersion\": 2");
    let r: Requirements = serde_json::from_str(&bumped).expect("parses");
    let c: Corrections = serde_json::from_str(CORR).expect("parses");
    let err = Rules::build(r, c).expect_err("a newer schema must not be read half-broken");
    assert!(
        format!("{err:?}").contains("SchemaMismatch"),
        "expected SchemaMismatch, got {err:?}"
    );
}

#[test]
fn every_target_row_is_addressable_by_its_key() {
    let r: Requirements = serde_json::from_str(REQS).expect("parses");
    let c: Corrections = serde_json::from_str(CORR).expect("parses");
    let rules = Rules::build(r, c).expect("rules build");
    assert_eq!(rules.targets().len(), 1);
    assert_eq!(rules.targets()[0].key, "entity:Red Heart");
}

#[test]
fn refs_are_read_for_the_achievement_that_owns_them() {
    let r: Requirements = serde_json::from_str(REQS).expect("parses");
    let c: Corrections = serde_json::from_str(CORR).expect("parses");
    let rules = Rules::build(r, c).expect("rules build");
    assert_eq!(rules.refs(1).len(), 1);
    assert_eq!(rules.refs(1)[0].label, "Red Heart");
    assert!(
        rules.refs(999).is_empty(),
        "an achievement with no row has no requirements, and that is not an error"
    );
}
