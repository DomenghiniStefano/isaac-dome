//! The bridge between the graph's chains and the queue's ordering rule.
//!
//! It lived in the Tauri crate, which by convention isn't tested, while the two things it
//! decides are exactly the ones a wrong answer reorders someone's plan by: a row the graph
//! can't compute carries no constraint, and a row nobody asked about isn't one either.

use ipc::GraphDeps;
use plan::Dependencies;

#[test]
fn a_row_requires_every_step_of_its_chain_and_nothing_else() {
    let deps = GraphDeps::from_chains([(10, vec![7, 8])]);
    assert!(deps.requires(10, 7));
    assert!(deps.requires(10, 8));
    assert!(!deps.requires(10, 9));
}

#[test]
fn a_row_the_graph_cannot_compute_carries_no_constraint() {
    // An empty chain is how `missing_chain` answers for a node it can't compute. Such a
    // row must never be dragged and never wall: `requires` is false in both directions.
    let deps = GraphDeps::from_chains([(10, Vec::new()), (7, Vec::new())]);
    assert!(!deps.requires(10, 7));
    assert!(!deps.requires(7, 10));
}

#[test]
fn a_row_that_was_never_asked_about_is_not_a_constraint() {
    // Chains are precomputed only for the rows a move involves. Anything outside that set
    // answers false, rather than silently falling back to a fresh walk of the graph.
    let deps = GraphDeps::from_chains([(10, vec![7])]);
    assert!(!deps.requires(99, 7));
}
