//! The queue as the UI sees it. The tests that matter are the ones that keep the four
//! ways a row can be absent from collapsing into one silent "not there".

use catalog::Catalog;
use ipc::{QueueDiagnostic, QueueInputs};
use serde_json::{json, to_value};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog_with_achievements() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

fn inputs<'a>(
    catalog: Option<&'a Catalog>,
    flags: Option<&'a [bool]>,
    queue: Result<&'a plan::Queue, &'a plan::QueueError>,
) -> QueueInputs<'a> {
    QueueInputs {
        catalog,
        dataset: None,
        flags,
        graph: None,
        eval: None,
        progress: None,
        queue,
        goals_pending: 0,
        store_reason: None,
    }
}

#[test]
fn the_diagnostic_shapes_are_pinned() {
    assert_eq!(
        to_value(QueueDiagnostic::Completed {
            count: 2,
            wanted: vec![41]
        })
        .expect("serializes"),
        json!({ "kind": "completed", "count": 2, "wanted": [41] })
    );
    assert_eq!(
        to_value(QueueDiagnostic::Unreadable).expect("serializes"),
        json!({ "kind": "unreadable" }),
        "unreadable and empty are different things, and the wire says which"
    );
    assert_eq!(
        to_value(QueueDiagnostic::GoalsPending { count: 3 }).expect("serializes"),
        json!({ "kind": "goalsPending", "count": 3 })
    );
    assert_eq!(
        to_value(QueueDiagnostic::Unresolved { achievement: 900 }).expect("serializes"),
        json!({ "kind": "unresolved", "achievement": 900 })
    );
}

#[test]
fn an_unreadable_queue_is_empty_and_says_so() {
    let err = plan::QueueError::Unreadable {
        reason: "expected value".into(),
    };
    let v = ipc::queue_view(inputs(None, None, Err(&err)), |_| None);
    assert!(v.rows.is_empty());
    assert!(v.diagnostics.contains(&QueueDiagnostic::Unreadable));
    assert!(
        v.store_available,
        "the database opened fine: it is the document that didn't parse"
    );
}

#[test]
fn a_store_that_will_not_open_is_a_different_case_from_an_unreadable_document() {
    let q = plan::Queue::default();
    let mut i = inputs(None, None, Ok(&q));
    i.store_reason = Some(ipc::StoreReason::NewerSchema {
        found: 7,
        supported: 2,
    });
    let v = ipc::queue_view(i, |_| None);
    assert!(!v.store_available);
    assert!(v
        .diagnostics
        .iter()
        .any(|d| matches!(d, QueueDiagnostic::StoreUnavailable { .. })));
}

#[test]
fn without_a_catalog_the_queue_is_empty_and_declares_why() {
    let q = plan::Queue::from_rows(vec![plan::Row {
        achievement: 1,
        wanted: true,
        origins: vec![],
    }]);
    let v = ipc::queue_view(inputs(None, None, Ok(&q)), |_| None);
    assert!(v.rows.is_empty());
    assert!(v.diagnostics.contains(&QueueDiagnostic::NoCatalog));
}

#[test]
fn a_completed_row_leaves_the_view_and_is_reported() {
    let c = catalog_with_achievements();
    let q = plan::Queue::from_rows(vec![
        plan::Row {
            achievement: 1,
            wanted: true,
            origins: vec![],
        },
        plan::Row {
            achievement: 2,
            wanted: false,
            origins: vec![1],
        },
    ]);
    // Slot 1 is done, slot 2 is not.
    let flags = [false, true, false];
    let v = ipc::queue_view(inputs(Some(&c), Some(&flags), Ok(&q)), |_| None);
    assert_eq!(v.rows.len(), 1, "the done row is gone from the view");
    assert!(!v.rows[0].wanted);
    assert!(
        v.diagnostics.contains(&QueueDiagnostic::Completed {
            count: 1,
            wanted: vec![1]
        }),
        "a row that vanishes without a word is a bug: {:?}",
        v.diagnostics
    );
}

#[test]
fn a_row_the_catalog_no_longer_knows_is_declared_by_id() {
    let c = catalog_with_achievements();
    let q = plan::Queue::from_rows(vec![plan::Row {
        achievement: 900,
        wanted: true,
        origins: vec![],
    }]);
    let flags = [false, false, false];
    let v = ipc::queue_view(inputs(Some(&c), Some(&flags), Ok(&q)), |_| None);
    assert!(v.rows.is_empty());
    assert!(v
        .diagnostics
        .contains(&QueueDiagnostic::Unresolved { achievement: 900 }));
}

#[test]
fn a_row_carries_its_origins_and_the_node_the_unlock_screen_would_draw() {
    let c = catalog_with_achievements();
    let q = plan::Queue::from_rows(vec![plan::Row {
        achievement: 2,
        wanted: false,
        origins: vec![1, 41],
    }]);
    let flags = [false, false, false];
    let v = ipc::queue_view(inputs(Some(&c), Some(&flags), Ok(&q)), |_| None);
    assert_eq!(v.rows.len(), 1);
    assert_eq!(v.rows[0].origins, vec![1, 41]);
    let json = to_value(&v.rows[0]).expect("serializes");
    assert_eq!(json["node"]["achievement"]["kind"], "known");
    assert_eq!(json["node"]["achievement"]["id"], 2);
    assert_eq!(
        json["stepsNotQueued"], 0,
        "rename_all is what keeps this from arriving as steps_not_queued"
    );
    assert_eq!(json["wanted"], false);
}

#[test]
fn a_saved_target_resolves_to_the_achievement_that_unlocks_it() {
    use ipc::{ItemKindView, TargetKey};
    let c = catalog_with_achievements();
    // Item 2 declares `achievement="1"`, so achievement 1 is what unlocks it.
    assert_eq!(
        ipc::achievement_unlocking(
            &c,
            &TargetKey::Item {
                item_kind: ItemKindView::Passive,
                id: 2
            }
        ),
        Some(1)
    );
    assert_eq!(
        ipc::achievement_unlocking(&c, &TargetKey::Character { id: 99 }),
        None,
        "a target nothing unlocks is skipped, not guessed at"
    );
}
