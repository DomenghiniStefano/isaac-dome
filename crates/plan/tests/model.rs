//! The queue document: what it looks like on disk, and the difference between a queue
//! that is empty and one that couldn't be read.

use graph::AchievementId;
use plan::{Queue, Row};

fn a(n: u32) -> AchievementId {
    AchievementId(n)
}

fn aa(ns: &[u32]) -> Vec<AchievementId> {
    ns.iter().copied().map(AchievementId).collect()
}

fn row(achievement: u32, wanted: bool, origins: &[u32]) -> Row {
    Row {
        achievement: a(achievement),
        wanted,
        origins: aa(origins),
    }
}

#[test]
fn the_document_round_trips_and_the_order_is_the_position() {
    let q = Queue::from_rows(vec![
        row(89, false, &[41]),
        row(41, true, &[]),
        row(512, true, &[]),
    ]);
    let json = q.to_json();
    assert!(
        json.starts_with('['),
        "the document is an array: the order is the position, not a column — got {json}"
    );
    let back = Queue::from_json(&json).expect("round trip");
    assert_eq!(back, q);
    assert_eq!(back.position(a(41)), Some(1));
    assert_eq!(back.position(a(999)), None);
}

#[test]
fn the_json_field_names_are_the_ones_written_in_the_spec() {
    let q = Queue::from_rows(vec![row(89, true, &[41, 512])]);
    let v: serde_json::Value = serde_json::from_str(&q.to_json()).expect("parses");
    assert_eq!(
        v[0],
        serde_json::json!({ "achievement": 89, "wanted": true, "origins": [41, 512] })
    );
}

#[test]
fn a_document_that_does_not_parse_is_an_error_not_an_empty_queue() {
    let err = Queue::from_json("{ not json").expect_err("must not read as empty");
    assert!(
        format!("{err:?}").contains("Unreadable"),
        "an unreadable queue and an empty queue are different things, got {err:?}"
    );
}

#[test]
fn an_empty_document_is_an_empty_queue() {
    assert_eq!(Queue::from_json("[]").expect("parses").rows().len(), 0);
}

#[test]
fn a_row_with_no_reason_to_exist_knows_it() {
    assert!(row(7, false, &[]).is_orphan());
    assert!(!row(7, true, &[]).is_orphan(), "you asked for it");
    assert!(!row(7, false, &[41]).is_orphan(), "it serves a wish");
}

/// Card #81, V12: the row's ids become `AchievementId`, and the document on disk must not move.
/// Written before the type changed, from the shape the spec above fixes: a plan saved today
/// reads back after the change, with the same numbers.
#[test]
fn a_document_saved_before_the_ids_were_typed_reads_back_the_same() {
    let saved = r#"[{"achievement":89,"wanted":true,"origins":[41,512]}]"#;
    let q = Queue::from_json(saved).expect("a document from before reads");
    let row = &q.rows()[0];
    assert_eq!(row.achievement, graph::AchievementId(89));
    assert_eq!(
        row.origins,
        vec![graph::AchievementId(41), graph::AchievementId(512)]
    );
    assert_eq!(q.to_json(), saved, "and writes back byte for byte");
}
