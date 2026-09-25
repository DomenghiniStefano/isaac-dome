//! The walk against a profile. These build graphs straight from edges, with no catalog:
//! this task is the arithmetic, and mixing XML fixtures in would test the previous one
//! again.

use graph::build::{Graph, GraphDiagnostic};
use graph::evaluate::NodeInfo;

use graph::AchievementId;

fn a(n: u32) -> AchievementId {
    AchievementId(n)
}

fn aa(ns: &[u32]) -> Vec<AchievementId> {
    ns.iter().copied().map(AchievementId).collect()
}

fn graph(edges: &[(u32, &[u32])], unknown: &[(u32, &[&str])]) -> Graph {
    graph::for_tests::from_edges(edges, unknown)
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
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 2))));
    assert_eq!(
        e.node(a(1)),
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
    let blocked = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 3))));
    assert_eq!(
        blocked.node(a(2)),
        Some(&NodeInfo::Computed {
            available_now: false,
            blocked_by: 1,
            fan_out: 0,
            steps_missing: 1
        })
    );
    let freed = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 3))));
    assert_eq!(
        freed.node(a(2)),
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
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 5))));
    let Some(NodeInfo::Computed { steps_missing, .. }) = e.node(a(4)) else {
        panic!("node 4 should be Computed: {:?}", e.node(a(4)));
    };
    assert_eq!(
        *steps_missing, 3,
        "1, 2 and 3 — the shared ancestor counts once"
    );
}

#[test]
fn steps_missing_skips_what_is_already_done() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[2])], &[]);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 4))));
    let Some(NodeInfo::Computed { steps_missing, .. }) = e.node(a(3)) else {
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
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 4))));
    let Some(NodeInfo::Computed { fan_out, .. }) = e.node(a(1)) else {
        panic!("node 1 should be Computed");
    };
    assert_eq!(*fan_out, 2);
}

#[test]
fn an_unknown_requirement_makes_the_node_partial() {
    let g = graph(&[(1, &[])], &[(1, &["Bestiary", "Collect"])]);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 2))));
    assert_eq!(
        e.node(a(1)),
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
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 3))));
    assert!(
        e.diagnostics()
            .iter()
            .any(|d| matches!(d, GraphDiagnostic::Cycle { .. })),
        "a cycle must be named, got {:?}",
        e.diagnostics()
    );
    assert!(
        matches!(e.node(a(1)), Some(NodeInfo::Partial { .. })),
        "a node inside a cycle cannot claim a transitive count, got {:?}",
        e.node(a(1))
    );
}

#[test]
fn a_node_already_done_is_not_available_now() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 2))));
    let Some(NodeInfo::Computed {
        available_now,
        steps_missing,
        ..
    }) = e.node(a(1))
    else {
        panic!("node 1 should be Computed");
    };
    assert!(!available_now, "it is done: there is nothing to unlock");
    assert_eq!(*steps_missing, 0);
}

#[test]
fn without_section_one_there_are_no_nodes_and_no_invented_zeros() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(None));
    assert_eq!(e.node(a(1)), None, "unread is not the same as not done");
}

#[test]
fn a_slot_the_save_does_not_reach_is_treated_as_not_done() {
    // The save declares fewer slots than the catalog has achievements: an older edition.
    // The missing slot is "not done", never "done" — the optimistic reading would claim
    // progress the file doesn't contain.
    let g = graph(&[(1, &[]), (9, &[])], &[]);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 3))));
    let Some(NodeInfo::Computed { available_now, .. }) = e.node(a(9)) else {
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
    let g = graph::for_tests::from_edges(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"])],
    );
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 3))));
    assert_eq!(
        e.node(a(2)),
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
    let g = graph::for_tests::from_edges(
        &[(1, &[]), (2, &[])],
        &[(1, &["Bestiary"]), (2, &["Bestiary"])],
    );
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 3))));
    assert!(
        matches!(e.node(a(2)), Some(NodeInfo::Partial { unknown: 1, .. })),
        "nothing done behind it: we genuinely don't know, got {:?}",
        e.node(a(2))
    );
}

#[test]
fn evidence_is_per_gate_not_per_node() {
    // Node 1 proves "Delirium"; node 3 waits on "Bestiary", which nobody has passed.
    let g = graph::for_tests::from_edges(
        &[(1, &[]), (2, &[]), (3, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"]), (3, &["Bestiary"])],
    );
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 4))));
    assert!(matches!(e.node(a(2)), Some(NodeInfo::Computed { .. })));
    assert!(
        matches!(e.node(a(3)), Some(NodeInfo::Partial { unknown: 1, .. })),
        "one gate's evidence says nothing about another's"
    );
}

#[test]
fn a_node_waiting_on_two_gates_needs_evidence_for_both() {
    let g = graph::for_tests::from_edges(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium", "Bestiary"])],
    );
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 3))));
    assert!(
        matches!(e.node(a(2)), Some(NodeInfo::Partial { unknown: 1, .. })),
        "Delirium is proven, Bestiary isn't: one unknown left, got {:?}",
        e.node(a(2))
    );
}

#[test]
fn the_inference_is_declared_not_silent() {
    let g = graph::for_tests::from_edges(
        &[(1, &[]), (2, &[])],
        &[(1, &["Delirium"]), (2, &["Delirium"])],
    );
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 3))));
    assert!(
        e.diagnostics().iter().any(|d| matches!(
            d,
            GraphDiagnostic::GateSatisfiedByEvidence { label, done: 1 } if label == "Delirium"
        )),
        "the app infers something from the save: it has to say so, got {:?}",
        e.diagnostics()
    );
}

#[test]
fn the_missing_chain_is_what_still_stands_between_you_and_a_node() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[2])], &[]);
    assert_eq!(
        g.missing_chain(a(3), &graph::evaluate::FlagsOnly(Some(&flags(&[], 4)))),
        aa(&[1, 2])
    );
    assert_eq!(
        g.missing_chain(a(3), &graph::evaluate::FlagsOnly(Some(&flags(&[1], 4)))),
        aa(&[2]),
        "what is done is not owed again"
    );
    assert!(
        g.missing_chain(a(1), &graph::evaluate::FlagsOnly(Some(&flags(&[], 4))))
            .is_empty(),
        "nothing stands between you and a node with no prerequisites"
    );
    assert!(
        g.missing_chain(a(3), &graph::evaluate::FlagsOnly(None))
            .is_empty(),
        "without section 1 there is nothing to compute, and nothing is claimed"
    );
    assert!(
        g.missing_chain(a(999), &graph::evaluate::FlagsOnly(Some(&flags(&[], 4))))
            .is_empty(),
        "a node that isn't in the graph owes nothing: it is not an error"
    );
}

#[test]
fn a_node_in_a_cycle_has_no_knowable_chain() {
    let g = graph(&[(1, &[2]), (2, &[1])], &[]);
    assert!(
        g.missing_chain(a(1), &graph::evaluate::FlagsOnly(Some(&flags(&[], 3))))
            .is_empty(),
        "not knowable comes back empty, and `NodeInfo::Partial` is what says so"
    );
}

// --- requirements the profile answers (spec 2026-09-12, §4.3 and §4.6) ---------------

use catalog::CharacterId;
use graph::model::Requirement;
use graph::rules::{CounterName, MarkColumn, MarkLevel};
use std::collections::BTreeMap;

/// The tests own the numbers, so nothing here depends on a sample.
///
/// `marks` absent = the cell is not located (one of the 40). `Some(None)` = located and
/// read, nothing reached. The two must not collapse: the first makes a node `Partial`,
/// the second leaves it computed and unmet.
#[derive(Default)]
struct Fake {
    done: Vec<bool>,
    marks: BTreeMap<(u32, MarkColumn), Option<MarkLevel>>,
    counters: BTreeMap<CounterName, u32>,
}

impl graph::evaluate::Profile for Fake {
    fn done(&self) -> Option<&[bool]> {
        Some(&self.done)
    }
    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<Option<MarkLevel>> {
        self.marks.get(&(character.0, column)).copied()
    }
    fn counter(&self, name: CounterName) -> Option<u32> {
        self.counters.get(&name).copied()
    }
}

impl Fake {
    fn node_1(&self, g: &Graph) -> Option<NodeInfo> {
        g.evaluate(self).node(a(1)).cloned()
    }
}

fn one_node(r: Requirement) -> Graph {
    graph::for_tests::from_requirements(&[(1, vec![r])])
}

fn mother_of(id: u32) -> Requirement {
    Requirement::Mark {
        character: CharacterId(id),
        column: MarkColumn::Mother,
        level: MarkLevel::Base,
    }
}

/// Spec §4.6: nothing is locked, the content is reachable, it only has to be played. So
/// the node is available now and `blocked_by` stays a count of achievements.
#[test]
fn a_node_held_only_by_an_unmet_mark_is_available_now() {
    let g = one_node(mother_of(0));
    let p = Fake {
        done: vec![false, false],
        marks: [((0, MarkColumn::Mother), None)].into_iter().collect(),
        ..Default::default()
    };
    let Some(NodeInfo::Computed {
        available_now,
        blocked_by,
        ..
    }) = p.node_1(&g)
    else {
        panic!("a cell the profile can answer is not a reason to be Partial");
    };
    assert!(available_now, "nothing is locked: it only has to be played");
    assert_eq!(
        blocked_by, 0,
        "blocked_by counts achievements, and a mark is not one"
    );
}

/// One of the 40 cells nobody has located. "I can't tell you" must not become "not done".
#[test]
fn a_cell_the_layout_cannot_answer_keeps_the_node_partial() {
    let g = one_node(mother_of(99));
    let p = Fake {
        done: vec![false, false],
        ..Default::default()
    };
    assert!(
        matches!(p.node_1(&g), Some(NodeInfo::Partial { .. })),
        "an unlocated cell is 'we can't say', never 'not satisfied'"
    );
}

#[test]
fn the_second_level_satisfies_a_base_requirement() {
    let g = one_node(Requirement::Mark {
        character: CharacterId(0),
        column: MarkColumn::Greed,
        level: MarkLevel::Base,
    });
    let p = Fake {
        done: vec![false, false],
        marks: [((0, MarkColumn::Greed), Some(MarkLevel::Second))]
            .into_iter()
            .collect(),
        ..Default::default()
    };
    assert!(
        matches!(p.node_1(&g), Some(NodeInfo::Computed { .. })),
        "reached at the higher level: the base requirement is met, and answerable"
    );
}

#[test]
fn a_tally_below_its_threshold_does_not_block_and_an_unread_one_is_partial() {
    let g = one_node(Requirement::Counter {
        name: CounterName::HushKills,
        at_least: 1,
    });
    let below = Fake {
        done: vec![false, false],
        counters: [(CounterName::HushKills, 0)].into_iter().collect(),
        ..Default::default()
    };
    assert!(
        matches!(
            below.node_1(&g),
            Some(NodeInfo::Computed {
                available_now: true,
                ..
            })
        ),
        "read and zero: unmet, and nothing is locked"
    );

    let unread = Fake {
        done: vec![false, false],
        ..Default::default()
    };
    assert!(
        matches!(unread.node_1(&g), Some(NodeInfo::Partial { .. })),
        "section 2 unread must not read as 'the tally is zero'"
    );
}

/// A caller that only has the achievement flags answers `None` to both other questions,
/// so every node holding one of these requirements stays `Partial`. That is the honest
/// outcome, and it is what keeps a half-wired caller from claiming progress.
#[test]
fn a_flags_only_profile_cannot_answer_and_says_so() {
    let g = one_node(mother_of(0));
    let f = vec![false, false];
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&f)));
    assert!(matches!(e.node(a(1)), Some(NodeInfo::Partial { .. })));
}

// --- Thresholds: a transformation, answered against the profile and never as an edge. ---

use catalog::{ItemId, ItemKind};
use graph::model::ThresholdItem;

/// One contributor: `gate` is the achievement that unlocks it, `None` when nothing does.
fn contributor(id: u32, gate: Option<u32>) -> ThresholdItem {
    ThresholdItem {
        kind: ItemKind::Passive,
        id: ItemId(id),
        unlocked_by: gate.map(AchievementId),
    }
}

/// Node 65 held by nothing but Guppy's threshold: three of the items listed, each behind an
/// achievement of its own, plus however many the catalog could not resolve.
fn guppy_node(gates: &[Option<u32>], unresolved: u32) -> graph::build::Graph {
    graph::for_tests::from_requirements(&[(
        65,
        vec![Requirement::Threshold {
            transformation: 0,
            label: "Guppy".into(),
            at_least: 3,
            of: gates
                .iter()
                .enumerate()
                .map(|(n, g)| contributor(200 + n as u32, *g))
                .collect(),
            unresolved,
        }],
    )])
}

/// Above the line the threshold costs nothing: no edge, nothing unknown. A node held by
/// nothing else is available now — the reading `Mark` and `Counter` already have.
#[test]
fn a_met_threshold_leaves_the_node_available_now() {
    let g = guppy_node(&[Some(1), Some(2), Some(3)], 0);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1, 2, 3], 66))));
    assert_eq!(
        e.node(a(65)),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        })
    );
}

/// An item nothing gates counts without any achievement being done.
#[test]
fn an_ungated_contributor_counts_on_its_own() {
    let g = guppy_node(&[None, None, None], 0);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[], 66))));
    assert!(matches!(
        e.node(a(65)),
        Some(NodeInfo::Computed {
            available_now: true,
            ..
        })
    ));
}

/// Below the line the node cannot be done, and the graph still draws no edge: `blocked_by`
/// stays zero because the prerequisites of "any three of these" are a disjunction. What it
/// does instead is say why.
#[test]
fn an_unmet_threshold_makes_the_node_partial_and_says_why() {
    let g = guppy_node(&[Some(1), Some(2), Some(3)], 0);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1, 2], 66))));
    assert!(
        matches!(e.node(a(65)), Some(NodeInfo::Partial { blocked_by: 0, .. })),
        "{:?}",
        e.node(a(65))
    );
    assert!(
        e.diagnostics().iter().any(|d| matches!(
            d,
            GraphDiagnostic::ThresholdUnmet {
                node: AchievementId(65),
                current: 2,
                at_least: 3,
                ..
            }
        )),
        "{:?}",
        e.diagnostics()
    );
}

/// Monotonicity, and the property the ordering exists for: an unresolved contributor can
/// only ever add to the tally, so a threshold already met stays met when one of the items it
/// did not need turns out to be outside this catalog.
#[test]
fn an_unresolved_contributor_never_unmeets_a_met_threshold() {
    let g = guppy_node(&[Some(1), Some(2), Some(3)], 1);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1, 2, 3], 66))));
    assert!(matches!(
        e.node(a(65)),
        Some(NodeInfo::Computed {
            available_now: true,
            ..
        })
    ));
}

/// Below the line **and** with something unresolved, the honest answer is "we cannot say"
/// rather than "you are one short": the missing item might have been the third.
#[test]
fn an_unresolved_contributor_below_the_line_is_unanswerable_not_unmet() {
    let g = guppy_node(&[Some(1), Some(2), Some(3)], 1);
    let e = g.evaluate(&graph::evaluate::FlagsOnly(Some(&flags(&[1], 66))));
    assert!(matches!(e.node(a(65)), Some(NodeInfo::Partial { .. })));
    assert!(
        !e.diagnostics()
            .iter()
            .any(|d| matches!(d, GraphDiagnostic::ThresholdUnmet { .. })),
        "an unanswerable threshold must not claim a count it does not have"
    );
}
