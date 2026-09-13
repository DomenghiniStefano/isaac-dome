//! How a `store` failure reaches the UI. These were `crates/app`'s only tests, which was
//! the proof written in-house that the rule was broken: if a return value is worth checking
//! it lives in a pure crate, and the Tauri crate is not tested. They check the same things
//! here, next to the code they are about.

use ipc::{IpcError, StoreReason};
use store::{plan_parts, store_error, GoalsRead, StoreError};

/// A recognizable input: if any of it came through, the boundary leaked. Since N2 the
/// reason is a **variant**, so there is no string for it to hide in — these tests assert on
/// the variant rather than hunting for a word, which is what makes them structural.
const SECRET_PATH: &str = r"C:\secret\isaacdome.db";

fn reason_of(e: IpcError) -> StoreReason {
    match e {
        IpcError::StoreUnavailable { reason } => reason,
        other => panic!("expected StoreUnavailable, got {other:?}"),
    }
}

#[test]
fn an_unreadable_database_is_a_variant_and_not_sqlites_message() {
    assert_eq!(
        reason_of(store_error(StoreError::Unreadable {
            reason: SECRET_PATH.to_string(),
        })),
        StoreReason::Unreadable
    );
}

#[test]
fn a_newer_schema_carries_both_versions_as_numbers() {
    assert_eq!(
        reason_of(store_error(StoreError::NewerSchema {
            found: 7,
            supported: 1,
        })),
        StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        }
    );
}

fn goal(id: &str) -> ipc::Goal {
    ipc::Goal {
        id: ipc::GoalId::from_str_unchecked(id),
        target: ipc::TargetKey::Boss { id: 1 },
        created_unix: 0,
        note: None,
    }
}

#[test]
fn a_successful_read_passes_goals_and_unreadable_rows_through() {
    let read = Ok(GoalsRead {
        goals: vec![goal("a")],
        unreadable: vec![ipc::GoalId::from_str_unchecked("b")],
    });
    let (goals, unreadable, unavailable) = plan_parts(read);
    assert_eq!(goals, vec![goal("a")]);
    assert_eq!(unreadable, vec![ipc::GoalId::from_str_unchecked("b")]);
    assert_eq!(unavailable, None);
}

/// A failed query is not a plan that disappears: the Plan degrades and says why, just like
/// when the database doesn't open at all.
#[test]
fn a_failed_query_degrades_into_the_reason_not_into_an_error() {
    let (goals, unreadable, unavailable) = plan_parts(Err(StoreError::NewerSchema {
        found: 7,
        supported: 1,
    }));
    assert!(goals.is_empty());
    assert!(unreadable.is_empty());
    assert_eq!(
        unavailable,
        Some(StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        })
    );
}

/// And SQLite's message doesn't get through: `plan_parts` maps the same way the write
/// commands do, so the Plan can never end up with a reason the rest of the app can't.
#[test]
fn a_failed_query_reports_the_variant_not_sqlites_message() {
    let (_, _, unavailable) = plan_parts(Err(StoreError::Unreadable {
        reason: SECRET_PATH.to_string(),
    }));
    assert_eq!(unavailable, Some(StoreReason::Unreadable));
}
