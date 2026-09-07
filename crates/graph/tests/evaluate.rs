//! The walk against a profile. These build graphs straight from edges, with no catalog:
//! this task is the arithmetic, and mixing XML fixtures in would test the previous one
//! again.

use graph::build::{Graph, GraphDiagnostic};
use graph::evaluate::NodeInfo;

fn graph(edges: &[(u32, &[u32])], unknown: &[(u32, &[&str])]) -> Graph {
    Graph::from_edges_for_tests(edges, unknown)
}

fn flags(done: &[u32], slots: usize) -> Vec<bool> {
    let mut f = vec![false; slots];
    for &d in done {
        f[d as usize] = true;
    }
    f
}

#[test]
fn a_node_with_no_prerequisites_is_available_now() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(Some(&flags(&[], 2)));
    assert_eq!(
        e.node(1),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        })
    );
}

#[test]
fn a_done_prerequisite_stops_blocking() {
    let g = graph(&[(1, &[]), (2, &[1])], &[]);
    let blocked = g.evaluate(Some(&flags(&[], 3)));
    assert_eq!(
        blocked.node(2),
        Some(&NodeInfo::Computed {
            available_now: false,
            blocked_by: 1,
            fan_out: 0,
            steps_missing: 1
        })
    );
    let freed = g.evaluate(Some(&flags(&[1], 3)));
    assert_eq!(
        freed.node(2),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        })
    );
}

#[test]
fn steps_missing_counts_a_shared_ancestor_once() {
    // 4 needs 2 and 3; both need 1. Three runs, not four.
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[1]), (4, &[2, 3])], &[]);
    let e = g.evaluate(Some(&flags(&[], 5)));
    let Some(NodeInfo::Computed { steps_missing, .. }) = e.node(4) else {
        panic!("node 4 should be Computed: {:?}", e.node(4));
    };
    assert_eq!(
        *steps_missing, 3,
        "1, 2 and 3 — the shared ancestor counts once"
    );
}

#[test]
fn steps_missing_skips_what_is_already_done() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[2])], &[]);
    let e = g.evaluate(Some(&flags(&[1], 4)));
    let Some(NodeInfo::Computed { steps_missing, .. }) = e.node(3) else {
        panic!("node 3 should be Computed");
    };
    assert_eq!(
        *steps_missing, 1,
        "only 2 is left between the profile and 3"
    );
}

#[test]
fn fan_out_counts_the_nodes_this_one_opens() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[1])], &[]);
    let e = g.evaluate(Some(&flags(&[], 4)));
    let Some(NodeInfo::Computed { fan_out, .. }) = e.node(1) else {
        panic!("node 1 should be Computed");
    };
    assert_eq!(*fan_out, 2);
}

#[test]
fn an_unknown_requirement_makes_the_node_partial() {
    let g = graph(&[(1, &[])], &[(1, &["Bestiary", "Collect"])]);
    let e = g.evaluate(Some(&flags(&[], 2)));
    assert_eq!(
        e.node(1),
        Some(&NodeInfo::Partial {
            blocked_by: 0,
            fan_out: 0,
            unknown: 2
        }),
        "zero prerequisites is not 'available now' when something wasn't understood"
    );
}

#[test]
fn a_cycle_is_declared_and_never_walked_twice() {
    let g = graph(&[(1, &[2]), (2, &[1])], &[]);
    let e = g.evaluate(Some(&flags(&[], 3)));
    assert!(
        e.diagnostics()
            .iter()
            .any(|d| matches!(d, GraphDiagnostic::Cycle { .. })),
        "a cycle must be named, got {:?}",
        e.diagnostics()
    );
    assert!(
        matches!(e.node(1), Some(NodeInfo::Partial { .. })),
        "a node inside a cycle cannot claim a transitive count, got {:?}",
        e.node(1)
    );
}

#[test]
fn a_node_already_done_is_not_available_now() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(Some(&flags(&[1], 2)));
    let Some(NodeInfo::Computed {
        available_now,
        steps_missing,
        ..
    }) = e.node(1)
    else {
        panic!("node 1 should be Computed");
    };
    assert!(!available_now, "it is done: there is nothing to unlock");
    assert_eq!(*steps_missing, 0);
}

#[test]
fn without_section_one_there_are_no_nodes_and_no_invented_zeros() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(None);
    assert_eq!(e.node(1), None, "unread is not the same as not done");
}

#[test]
fn a_slot_the_save_does_not_reach_is_treated_as_not_done() {
    // The save declares fewer slots than the catalog has achievements: an older edition.
    // The missing slot is "not done", never "done" — the optimistic reading would claim
    // progress the file doesn't contain.
    let g = graph(&[(1, &[]), (9, &[])], &[]);
    let e = g.evaluate(Some(&flags(&[], 3)));
    let Some(NodeInfo::Computed { available_now, .. }) = e.node(9) else {
        panic!("node 9 should be Computed");
    };
    assert!(available_now);
}

// --- a gate is satisfied when the profile has already passed it ---

#[test]
fn an_uninterpreted_gate_stops_blocking_once_something_behind_it_is_done() {
    // Two nodes need the same thing the graph can't express — "Delirium". One of them is
    // already done, which is proof the player can reach Delirium: the other is no longer
    // waiting on an unknown.
    let g = Graph::from_edges_for_tests(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"])],
    );
    let e = g.evaluate(Some(&flags(&[1], 3)));
    assert_eq!(
        e.node(2),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        }),
        "the gate is passed, and the save is the proof"
    );
}

#[test]
fn a_gate_with_no_evidence_behind_it_still_blocks() {
    let g = Graph::from_edges_for_tests(
        &[(1, &[]), (2, &[])],
        &[(1, &["Bestiary"]), (2, &["Bestiary"])],
    );
    let e = g.evaluate(Some(&flags(&[], 3)));
    assert!(
        matches!(e.node(2), Some(NodeInfo::Partial { unknown: 1, .. })),
        "nothing done behind it: we genuinely don't know, got {:?}",
        e.node(2)
    );
}

#[test]
fn evidence_is_per_gate_not_per_node() {
    // Node 1 proves "Delirium"; node 3 waits on "Bestiary", which nobody has passed.
    let g = Graph::from_edges_for_tests(
        &[(1, &[]), (2, &[]), (3, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"]), (3, &["Bestiary"])],
    );
    let e = g.evaluate(Some(&flags(&[1], 4)));
    assert!(matches!(e.node(2), Some(NodeInfo::Computed { .. })));
    assert!(
        matches!(e.node(3), Some(NodeInfo::Partial { unknown: 1, .. })),
        "one gate's evidence says nothing about another's"
    );
}

#[test]
fn a_node_waiting_on_two_gates_needs_evidence_for_both() {
    let g = Graph::from_edges_for_tests(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium", "Bestiary"])],
    );
    let e = g.evaluate(Some(&flags(&[1], 3)));
    assert!(
        matches!(e.node(2), Some(NodeInfo::Partial { unknown: 1, .. })),
        "Delirium is proven, Bestiary isn't: one unknown left, got {:?}",
        e.node(2)
    );
}

#[test]
fn the_inference_is_declared_not_silent() {
    let g = Graph::from_edges_for_tests(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"])],
    );
    let e = g.evaluate(Some(&flags(&[1], 3)));
    assert!(
        e.diagnostics().iter().any(|d| matches!(
            d,
            GraphDiagnostic::GateSatisfiedByEvidence { label, done: 1 } if label == "Delirium"
        )),
        "the app infers something from the save: it has to say so, got {:?}",
        e.diagnostics()
    );
}
