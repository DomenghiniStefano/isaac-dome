//! Moving a row. The order is yours, and the graph is the one thing it may not
//! contradict: a move is never refused, the rows that must yield are moved.

use plan::order::Dependencies;
use plan::{Queue, Row};

/// "a requires b" from an explicit list of pairs, already transitive: the tests state the
/// relation they mean instead of deriving it, so a bug in the graph's walk cannot hide one
/// here.
struct Deps(&'static [(u32, u32)]);

impl Dependencies for Deps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.0.contains(&(a, b))
    }
}

fn queue(ids: &[u32]) -> Queue {
    Queue::from_rows(
        ids.iter()
            .map(|a| Row {
                achievement: *a,
                wanted: true,
                origins: Vec::new(),
            })
            .collect(),
    )
}

fn ids(q: &Queue) -> Vec<u32> {
    q.rows().iter().map(|r| r.achievement).collect()
}

#[test]
fn with_no_dependencies_a_row_lands_exactly_where_it_was_dropped() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3, 4]);
    assert_eq!(q.move_row(4, 1, &deps), 1);
    assert_eq!(ids(&q), vec![1, 4, 2, 3]);
}

#[test]
fn moving_a_prerequisite_down_drags_what_needs_it() {
    // 3 requires 1. Dropping 1 at the bottom must not leave 3 above it.
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 2, 3]);
    let landed = q.move_row(1, 2, &deps);
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
    let landed = q.move_row(4, 0, &deps);
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
    q.move_row(1, 3, &deps);
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
    q.move_row(1, 4, &deps);
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
    q.move_row(1, 2, &deps);
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
    assert_eq!(q.move_row(2, 1, &deps), 1);
    assert_eq!(ids(&q), before);
}

#[test]
fn moving_a_row_that_is_not_in_the_queue_does_nothing() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2]);
    assert_eq!(q.move_row(99, 0, &deps), 0);
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
    assert_eq!(q.move_row(1, 99, &deps), 2);
    assert_eq!(ids(&q), vec![2, 3, 1]);
}

/// The property the readable cases above are examples of. Deterministic pseudo-random: a
/// failure has to be reproducible from the round printed in the message.
#[test]
fn after_any_move_the_queue_never_contradicts_the_graph() {
    // 1 <- 2 <- 3, transitive closure written out; 4 and 5 unconstrained.
    const PAIRS: &[(u32, u32)] = &[(2, 1), (3, 2), (3, 1)];
    let deps = Deps(PAIRS);
    let mut seed = 12345u64;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            let to = next() % 5;
            q.move_row(who, to, &deps);
        }
        let order = ids(&q);
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
}
