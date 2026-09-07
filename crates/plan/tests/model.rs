//! The queue document: what it looks like on disk, and the difference between a queue
//! that is empty and one that couldn't be read.

use plan::{Queue, Row};

fn row(achievement: u32, wanted: bool, origins: &[u32]) -> Row {
    Row {
        achievement,
        wanted,
        origins: origins.to_vec(),
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
    assert_eq!(back.position(41), Some(1));
    assert_eq!(back.position(999), None);
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
