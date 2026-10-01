//! What the Data page says the database holds: one count per thing the user would recognise.

use ipc::{Goal, GoalId, ItemKindView, StoreContents, TargetKey};
use plan::{AchievementId, Queue, Row};
use run::{Floor, Generated, Outcome, Run, SeedKind, SourceKey};

mod common;
use common::temp_store;

fn goal(id: &str, item_id: u32) -> Goal {
    Goal {
        id: GoalId::from_str_unchecked(id),
        target: TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: item_id,
        },
        created_unix: 1_700_000_000,
        note: None,
    }
}

fn row(achievement: u32) -> Row {
    Row {
        achievement: AchievementId(achievement),
        wanted: true,
        origins: vec![],
    }
}

fn run_of(seed: &str) -> Run {
    Run {
        seed_words: seed.to_string(),
        seed_numeric: 1,
        seed_kind: SeedKind::New,
        character: None,
        character_id: None,
        starting_items: vec![],
        collected: vec![],
        passives: vec![],
        familiars: vec![],
        held_active: None,
        floors: vec![Floor {
            stage: 1,
            stage_type: 0,
            seed: 7,
            generated: Generated::NotSaid,
        }],
        achievements: vec![],
        outcome: Outcome::Open,
    }
}

#[test]
fn a_fresh_database_holds_nothing() {
    let (_dir, store) = temp_store();
    assert_eq!(
        store.contents().expect("the database answers"),
        StoreContents {
            goals: 0,
            queue_rows: Some(0),
            sessions: 0,
            runs: 0,
            roll_saved: false,
        }
    );
}

#[test]
fn each_write_moves_its_own_count() {
    let (_dir, store) = temp_store();
    store.add_goal(&goal("a", 1)).expect("first goal");
    store.add_goal(&goal("b", 2)).expect("second goal");
    store
        .set_queue(&Queue::from_rows(vec![row(1), row(2), row(3)]))
        .expect("the queue");
    let source = store
        .insert_log_source(&SourceKey::new(b"a banner", b"some bytes", 0))
        .expect("a source");
    store
        .cache_runs(source, 1, &[run_of("AAA AAA"), run_of("BBB BBB")])
        .expect("two runs");
    store
        .set_roll(&roll::Document::default())
        .expect("the roll");

    assert_eq!(
        store.contents().expect("the database answers"),
        StoreContents {
            goals: 2,
            queue_rows: Some(3),
            sessions: 1,
            runs: 2,
            roll_saved: true,
        }
    );
}

#[test]
fn an_unparseable_queue_counts_as_unknown() {
    // "Unknown" and "empty" are different sentences, and the rest of the answer still reads.
    let (_dir, store) = temp_store();
    store.add_goal(&goal("a", 1)).expect("a goal");
    store::for_tests::corrupt_queue(&store, "{ not json").expect("writes garbage");

    let contents = store.contents().expect("the database answers");
    assert_eq!(contents.queue_rows, None);
    assert_eq!(contents.goals, 1);
}
