//! Moving a row. The order is yours, and the graph is the one thing it may not
//! contradict: a move is never refused, the rows that must yield are moved.

use graph::AchievementId;
use plan::order::Dependencies;
use plan::{Queue, Row};

fn a(n: u32) -> AchievementId {
    AchievementId(n)
}

/// "a requires b" from an explicit list of pairs, already transitive: the tests state the
/// relation they mean instead of deriving it, so a bug in the graph's walk cannot hide one
/// here.
struct Deps(&'static [(u32, u32)]);

impl Dependencies for Deps {
    fn requires(&self, a: AchievementId, b: AchievementId) -> bool {
        self.0.contains(&(a.0, b.0))
    }
}

fn queue(ids: &[u32]) -> Queue {
    Queue::from_rows(
        ids.iter()
            .map(|a| Row {
                achievement: AchievementId(*a),
                wanted: true,
                origins: Vec::new(),
            })
            .collect(),
    )
}

fn ids(q: &Queue) -> Vec<u32> {
    q.rows().iter().map(|r| r.achievement.0).collect()
}

#[test]
fn with_no_dependencies_a_row_lands_exactly_where_it_was_dropped() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3, 4]);
    assert_eq!(q.move_row(a(4), 1, &deps), 1);
    assert_eq!(ids(&q), vec![1, 4, 2, 3]);
}

#[test]
fn moving_a_prerequisite_down_drags_what_needs_it() {
    // 3 requires 1. Dropping 1 at the bottom must not leave 3 above it.
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 2, 3]);
    let landed = q.move_row(a(1), 2, &deps);
    assert_eq!(
        landed, 1,
        "3 comes along, so 1 lands one above its dragged block rather than on the last row"
    );
    assert_eq!(
        ids(&q),
        vec![2, 1, 3],
        "3 needs 1, so it follows it down instead of the move being refused"
    );
}

#[test]
fn a_row_cannot_rise_above_its_prerequisites_and_they_do_not_move() {
    // 4 requires 1 and 2. Dropping 4 at the top is impossible: two rows must precede it.
    let deps = Deps(&[(4, 1), (4, 2)]);
    let mut q = queue(&[1, 2, 3, 4]);
    let landed = q.move_row(a(4), 0, &deps);
    assert_eq!(
        landed, 2,
        "clamped to the number of prerequisites in the queue"
    );
    assert_eq!(
        ids(&q),
        vec![1, 2, 4, 3],
        "1 and 2 never moved: they are a wall, not cargo, and the row stopped right below them"
    );
}

#[test]
fn the_repair_is_transitive() {
    // 3 requires 2, 2 requires 1 — the relation given here is already transitive.
    let deps = Deps(&[(2, 1), (3, 2), (3, 1)]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_row(a(1), 3, &deps);
    assert_eq!(
        ids(&q),
        vec![4, 1, 2, 3],
        "moving 1 to the end drags the whole chain that hangs off it"
    );
}

#[test]
fn rows_with_no_relation_keep_their_relative_order() {
    let deps = Deps(&[(5, 1)]);
    let mut q = queue(&[1, 2, 3, 4, 5]);
    q.move_row(a(1), 4, &deps);
    assert_eq!(
        ids(&q),
        vec![2, 3, 4, 1, 5],
        "2, 3 and 4 are untouched by a constraint they are not part of"
    );
}

#[test]
fn a_row_the_graph_cannot_compute_is_never_dragged() {
    // 9 has no relation to anything: the graph doesn't know its prerequisites.
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 9, 3]);
    q.move_row(a(1), 2, &deps);
    assert_eq!(
        ids(&q),
        vec![9, 1, 3],
        "9 stays put: an unknown row carries no constraint in either direction"
    );
}

#[test]
fn moving_a_row_to_where_it_already_is_changes_nothing() {
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 2, 3]);
    let before = ids(&q);
    assert_eq!(q.move_row(a(2), 1, &deps), 1);
    assert_eq!(ids(&q), before);
}

#[test]
fn moving_a_row_that_is_not_in_the_queue_does_nothing() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2]);
    assert_eq!(q.move_row(a(99), 0, &deps), 0);
    assert_eq!(
        ids(&q),
        vec![1, 2],
        "an absent row is not an error, it is a no-op"
    );
}

#[test]
fn an_index_past_the_end_lands_on_the_last_position() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    assert_eq!(q.move_row(a(1), 99, &deps), 2);
    assert_eq!(ids(&q), vec![2, 3, 1]);
}

#[test]
fn an_index_means_the_same_place_when_dependents_sit_before_it() {
    // 2 requires 1. Once 1 is out the queue reads [2, 3, 4]: index 2 is right after 3.
    let deps = Deps(&[(2, 1)]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_row(a(1), 2, &deps);
    assert_eq!(
        ids(&q),
        vec![3, 1, 2, 4],
        "1 lands below 3 with 2 in tow, not below 4"
    );
}

#[test]
fn move_after_lands_right_below_the_row_named() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_after(a(1), Some(a(3)), &deps);
    assert_eq!(ids(&q), vec![2, 3, 1, 4], "downwards");
    q.move_after(a(4), Some(a(2)), &deps);
    assert_eq!(ids(&q), vec![2, 4, 3, 1], "upwards");
}

#[test]
fn move_after_nothing_is_the_top() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    q.move_after(a(3), None, &deps);
    assert_eq!(ids(&q), vec![3, 1, 2]);
}

#[test]
fn moving_under_one_of_its_own_dependents_goes_as_low_as_it_can() {
    // 2 requires 1. Below 2 is not a place 1 can be: it goes right above 2, and 2 follows.
    let deps = Deps(&[(2, 1)]);
    let mut q = queue(&[1, 3, 2, 4]);
    q.move_after(a(1), Some(a(2)), &deps);
    assert_eq!(ids(&q), vec![3, 1, 2, 4]);
}

#[test]
fn rising_past_a_prerequisite_stops_right_below_it() {
    // 3 requires 2. Asked to sit below 1, 3 rises past 4 and stops under 2.
    let deps = Deps(&[(3, 2)]);
    let mut q = queue(&[1, 2, 4, 3]);
    q.move_after(a(3), Some(a(1)), &deps);
    assert_eq!(ids(&q), vec![1, 2, 3, 4]);
}

#[test]
fn an_anchor_not_queued_or_the_row_itself_changes_nothing() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    q.move_after(a(1), Some(a(99)), &deps);
    q.move_after(a(2), Some(a(2)), &deps);
    q.move_after(a(99), None, &deps);
    assert_eq!(
        ids(&q),
        vec![1, 2, 3],
        "a stale picture is answered with the truth"
    );
}

/// 1 <- 2 <- 3, transitive closure written out; 4 and 5 unconstrained.
const PAIRS: &[(u32, u32)] = &[(2, 1), (3, 2), (3, 1)];

/// The property the readable cases above are examples of.
fn assert_consistent(order: &[u32], round: usize) {
    for (a, b) in PAIRS {
        let ia = order.iter().position(|x| x == a).expect("present");
        let ib = order.iter().position(|x| x == b).expect("present");
        assert!(
            ib < ia,
            "round {round}: {a} requires {b}, and {b} ended up below it: {order:?}"
        );
    }
    assert_eq!(
        order.len(),
        5,
        "round {round}: a move lost or duplicated a row"
    );
}

/// Deterministic pseudo-random: a failure has to be reproducible from the round printed in
/// the message.
fn lcg(mut seed: u64) -> impl FnMut() -> usize {
    move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    }
}

#[test]
fn after_any_move_the_queue_never_contradicts_the_graph() {
    let deps = Deps(PAIRS);
    let mut next = lcg(12345);
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            q.move_row(a(who), next() % 5, &deps);
        }
        assert_consistent(&ids(&q), round);
    }
}

#[test]
fn after_any_move_after_the_queue_never_contradicts_the_graph() {
    let deps = Deps(PAIRS);
    let mut next = lcg(54321);
    let anchors = [
        None,
        Some(1u32),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(99),
    ];
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            q.move_after(a(who), anchors[next() % anchors.len()].map(a), &deps);
        }
        assert_consistent(&ids(&q), round);
    }
}

/// Card #80, item 03: a dragged dependent keeps its **own** prerequisites above it. Traced by
/// hand in the review: `[2, 1, 3]`, 3 requiring both 1 and 2 (and 1, 2 independent of each
/// other), `move_row(1, 0)` dragged 3 up to right below 1, above 2 — which it requires.
#[test]
fn a_dragged_dependent_does_not_rise_above_its_other_prerequisite() {
    let deps = Deps(&[(3, 1), (3, 2)]);
    let mut q = queue(&[2, 1, 3]);
    q.move_row(a(1), 0, &deps);
    let order = ids(&q);
    let at = |x: u32| order.iter().position(|y| *y == x).expect("present");
    assert!(at(2) < at(3), "3 requires 2 and sits above it: {order:?}");
    assert!(at(1) < at(3), "3 requires 1 and sits above it: {order:?}");
    assert_eq!(
        at(1),
        0,
        "the moved row lands where it was dropped: {order:?}"
    );
}

/// The same property as the randomized test above, over a relation with two independent
/// prerequisites of one row — the shape the first `PAIRS` could not produce.
const INDEPENDENT: &[(u32, u32)] = &[(3, 1), (3, 2), (5, 4)];

#[test]
fn after_any_move_two_independent_prerequisites_both_stay_above() {
    let deps = Deps(INDEPENDENT);
    let mut next = lcg(777);
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            q.move_row(a(who), next() % 5, &deps);
        }
        let order = ids(&q);
        for (x, y) in INDEPENDENT {
            let ix = order.iter().position(|v| v == x).expect("present");
            let iy = order.iter().position(|v| v == y).expect("present");
            assert!(iy < ix, "round {round}: {x} requires {y}: {order:?}");
        }
        assert_eq!(order.len(), 5, "round {round}: a row lost or duplicated");
    }
}
