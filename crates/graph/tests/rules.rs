//! The two rules files: they read, they join, and a file from another schema is refused
//! whole rather than read half-broken.

use graph::rules::{
    Corrections, CounterName, CounterRule, MarkColumn, MarkLevel, MarkRule, Requirements, Rules,
    RulesError, Verdict, SCHEMA_VERSION,
};

use graph::AchievementId;

fn a(n: u32) -> AchievementId {
    AchievementId(n)
}

const REQS: &str = r#"{
  "schemaVersion": 2,
  "generatedFrom": { "snapshotAt": "2026-09-04T17:33:31Z", "maxRevid": 269057 },
  "achievements": {
    "1": { "refs": [{ "target": { "kind": "entity", "id": 5, "variant": 10, "subtype": 1 },
                      "label": "Red Heart" }] }
  },
  "targets": [{ "key": "entity:Red Heart", "label": "Red Heart", "uses": 3,
               "verdictRequired": false }]
}"#;

const CORR: &str = r#"{
  "schemaVersion": 2,
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

/// Written against the constant, not against the literal it happens to hold: pinning
/// `"schemaVersion": 1` made this test go silently no-op the day the schema moved — the
/// replace found nothing, the fixture stayed current, and `Rules::build` succeeded while the
/// assertion below claimed to have seen it refuse.
#[test]
fn a_file_from_another_schema_is_refused_whole() {
    let current = format!("\"schemaVersion\": {SCHEMA_VERSION}");
    assert!(
        REQS.contains(&current),
        "the fixture is not on the current schema: {current}"
    );
    let bumped = REQS.replace(&current, "\"schemaVersion\": 99");
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
    assert_eq!(rules.refs(a(1)).len(), 1);
    assert_eq!(rules.refs(a(1))[0].label, "Red Heart");
    assert!(
        rules.refs(a(999)).is_empty(),
        "an achievement with no row has no requirements, and that is not an error"
    );
}

// --- the profile-answered verdict (spec 2026-09-12, §4.2) -----------------------------

const CORR_PROGRESS: &str = r#"{
  "schemaVersion": 2,
  "verdicts": {
    "entity:Hush": { "progress": {
      "mark": { "column": "hush", "level": "base" },
      "counter": { "name": "hushKills", "atLeast": 1 }
    } },
    "entity:Ultra Greedier": { "progress": {
      "mark": { "column": "greed", "level": "second" }
    } }
  }
}"#;

#[test]
fn a_progress_verdict_parses_both_halves() {
    let c: Corrections = serde_json::from_str(CORR_PROGRESS).expect("parses");
    assert_eq!(
        c.verdicts.get("entity:Hush"),
        Some(&Verdict::Progress {
            mark: Some(MarkRule {
                column: MarkColumn::Hush,
                level: MarkLevel::Base,
            }),
            counter: Some(CounterRule {
                name: CounterName::HushKills,
                at_least: 1,
            }),
        })
    );
}

#[test]
fn a_progress_verdict_may_carry_only_the_mark() {
    let c: Corrections = serde_json::from_str(CORR_PROGRESS).expect("parses");
    let Some(Verdict::Progress { mark, counter }) = c.verdicts.get("entity:Ultra Greedier") else {
        panic!("expected a progress verdict");
    };
    assert_eq!(mark.map(|m| m.level), Some(MarkLevel::Second));
    assert!(
        counter.is_none(),
        "an absent half stays absent: it is not defaulted into existence"
    );
}

/// Ordered so that a cell reached at the second level satisfies a base requirement.
#[test]
fn the_base_level_sorts_below_the_second() {
    assert!(MarkLevel::Base < MarkLevel::Second);
}

/// A verdict that answers nothing is not the same as no verdict, and it must not behave
/// like one: it is a broken rules file, and the file is refused whole.
#[test]
fn a_progress_verdict_with_neither_half_is_malformed() {
    let empty = r#"{
      "schemaVersion": 2,
      "verdicts": { "entity:Nothing": { "progress": {} } }
    }"#;
    let r: Requirements = serde_json::from_str(REQS).expect("parses");
    let c: Corrections = serde_json::from_str(empty).expect("parses");
    let err = Rules::build(r, c).expect_err("an empty progress answers nothing");
    assert!(
        matches!(err, RulesError::Malformed { .. }),
        "expected Malformed, got {err:?}"
    );
}
