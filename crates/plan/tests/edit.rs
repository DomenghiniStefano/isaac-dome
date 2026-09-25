//! Adding a wish and removing one. The chain arrives from the caller, already read from
//! the graph, which is what lets these tests run without a catalog.

use graph::AchievementId;
use plan::order::Dependencies;
use plan::{Queue, Row};

fn a(n: u32) -> AchievementId {
    AchievementId(n)
}

fn aa(ns: &[u32]) -> Vec<AchievementId> {
    ns.iter().copied().map(AchievementId).collect()
}

struct Deps(&'static [(u32, u32)]);

impl Dependencies for Deps {
    fn requires(&self, a: AchievementId, b: AchievementId) -> bool {
        self.0.contains(&(a.0, b.0))
    }
}

fn ids(q: &Queue) -> Vec<u32> {
    q.rows().iter().map(|r| r.achievement.0).collect()
}

fn row_of(q: &Queue, achievement: u32) -> &Row {
    q.rows()
        .iter()
        .find(|r| r.achievement.0 == achievement)
        .expect("row present")
}

#[test]
fn a_wish_goes_last_with_its_missing_steps_immediately_before_it() {
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7, 12]), &Deps(&[(41, 7), (41, 12)]));
    assert_eq!(ids(&q), vec![7, 12, 41]);
    assert!(row_of(&q, 41).wanted, "you asked for it");
    assert!(!row_of(&q, 7).wanted, "it arrived as a step");
    assert_eq!(row_of(&q, 7).origins, aa(&[41]));
}

#[test]
fn a_second_wish_appends_after_the_first() {
    let deps = Deps(&[(41, 7), (512, 9)]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7]), &deps);
    q.enqueue(a(512), &aa(&[9]), &deps);
    assert_eq!(
        ids(&q),
        vec![7, 41, 9, 512],
        "a new wish is the lowest priority until you say otherwise"
    );
}

#[test]
fn a_step_already_queued_is_not_duplicated_and_gains_an_origin() {
    let deps = Deps(&[(41, 7), (512, 7)]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7]), &deps);
    q.enqueue(a(512), &aa(&[7]), &deps);
    assert_eq!(ids(&q), vec![7, 41, 512]);
    assert_eq!(
        row_of(&q, 7).origins,
        aa(&[41, 512]),
        "one row, two reasons to be there"
    );
}

#[test]
fn enqueueing_something_already_wanted_leaves_it_where_it_is() {
    let deps = Deps(&[]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[]), &deps);
    q.enqueue(a(512), &aa(&[]), &deps);
    q.enqueue(a(41), &aa(&[]), &deps);
    assert_eq!(
        ids(&q),
        vec![41, 512],
        "asking twice is not a reason to reorder what you already arranged"
    );
}

#[test]
fn a_step_that_was_below_its_wish_is_repaired_into_place() {
    let deps = Deps(&[(512, 7)]);
    let mut q = Queue::from_rows(vec![Row {
        achievement: a(7),
        wanted: true,
        origins: Vec::new(),
    }]);
    // 512 needs 7, and 7 is already in the queue: enqueueing 512 must not leave it above.
    q.enqueue(a(512), &aa(&[7]), &deps);
    assert_eq!(ids(&q), vec![7, 512]);
    assert!(row_of(&q, 7).wanted, "it stays wanted, and gains an origin");
    assert_eq!(row_of(&q, 7).origins, aa(&[512]));
}

#[test]
fn removing_a_wish_takes_only_the_rows_left_with_no_reason_to_be_there() {
    let deps = Deps(&[(41, 7), (41, 12)]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7, 12]), &deps);
    q.remove(a(41));
    assert_eq!(ids(&q), Vec::<u32>::new(), "its steps served nothing else");
}

#[test]
fn a_step_two_wishes_need_survives_the_removal_of_one() {
    let deps = Deps(&[(41, 7), (512, 7)]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7]), &deps);
    q.enqueue(a(512), &aa(&[7]), &deps);
    q.remove(a(41));
    assert_eq!(ids(&q), vec![7, 512]);
    assert_eq!(row_of(&q, 7).origins, aa(&[512]));
}

#[test]
fn a_step_you_also_asked_for_survives_the_removal_of_its_wish() {
    let deps = Deps(&[(41, 7)]);
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[7]), &deps);
    // You decide you want the step for itself too: wanted and an origin at once.
    q.enqueue(a(7), &aa(&[]), &deps);
    q.remove(a(41));
    assert_eq!(ids(&q), vec![7]);
    assert!(row_of(&q, 7).wanted);
    assert!(row_of(&q, 7).origins.is_empty());
}

#[test]
fn removing_something_absent_is_a_no_op() {
    let mut q = Queue::default();
    q.enqueue(a(41), &aa(&[]), &Deps(&[]));
    q.remove(a(999));
    assert_eq!(ids(&q), vec![41]);
}

/// Card #80, P4: `missing_chain` promises only increasing ids, not an order the steps can be
/// played in, and a step with a lower id can need one with a higher id. The queue must come
/// out valid whatever order the chain arrives in.
#[test]
fn the_chain_is_placed_in_playable_order_whatever_order_it_arrives_in() {
    // 41 needs 7 and 12, and 7 needs 12: the playable order is 12, 7, 41.
    let deps = Deps(&[(41, 7), (41, 12), (7, 12)]);
    for chain in [[7u32, 12], [12, 7]] {
        let mut q = Queue::default();
        q.enqueue(a(41), &aa(&chain), &deps);
        assert_eq!(ids(&q), vec![12, 7, 41], "chain arrived as {chain:?}");
    }
}
