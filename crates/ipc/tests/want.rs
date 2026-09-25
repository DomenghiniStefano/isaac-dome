//! B37: naming a thing and being told what to play for it. What these tests keep apart is
//! the set of ways the answer can be "nothing": a target nothing unlocks, a thing that is
//! not unlockable at all, and a game that isn't installed are three different sentences.

use ipc::{AchievementRef, GraphInfo, Target, UnlockTarget, WantDiagnostic, WantState, WantedView};

mod support;
use support::{catalog_with_achievements, empty_view};

#[test]
fn a_stage_is_not_a_thing_you_unlock() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &empty_view(),
        None,
        None,
        &Target::Stage {
            name: "Basement".into(),
        },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NotUnlockable]);
}

#[test]
fn without_a_catalog_nothing_resolves_and_the_view_says_so() {
    let v = ipc::want_view(
        None,
        ipc::BossKeys::NONE,
        &empty_view(),
        None,
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NoCatalog]);
}

/// One node per achievement 1..=3, none done, all playable right now. These four tests are
/// about *which* nodes a name reaches; they read a profile so that the states they make no
/// claim about don't become the `NoProfile` diagnostic and drown the ones they do.
fn view_of(c: &catalog::Catalog) -> ipc::UnlockView {
    view_with(c, &[], COMPUTED_NOW)
}

/// The profile those four tests read: four slots, nothing done.
const READ: [bool; 4] = [false; 4];

#[test]
fn an_item_names_the_achievement_that_grants_it() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert!(matches!(
        &v.wanted,
        WantedView::Target {
            target: UnlockTarget::Item { id: 2, .. }
        }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(matches!(
        &v.routes[0].node.achievement,
        AchievementRef::Known { id: 1, .. }
    ));
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_challenge_named_by_two_achievements_shows_two_routes() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Challenge { number: 4 },
        |_| None,
    );
    let ids: Vec<u32> = v
        .routes
        .iter()
        .filter_map(|r| match r.node.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1, 2], "both ways in, neither hidden");
}

#[test]
fn an_achievement_named_directly_is_its_own_route() {
    // *Greedier!* is the case: the catalog models no target for it, so the only way to reach
    // that node is to name the achievement itself.
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Achievement { id: 2 },
        |_| None,
    );
    assert!(matches!(
        &v.wanted,
        WantedView::Achievement {
            achievement: AchievementRef::Known { id: 2, .. }
        }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_thing_no_achievement_grants_says_which_empty_it_is() {
    let c = catalog_with_achievements();
    // Character 7 is granted by achievement 2; character 9 is in no file at all.
    let v = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Character { id: 9 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NothingUnlocks]);
}

/// The same view, with the achievements in `done` marked and the graph's verdict forced.
fn view_with(c: &catalog::Catalog, done: &[u32], info: GraphInfo) -> ipc::UnlockView {
    let mut flags = [false; 4];
    for id in done {
        flags[*id as usize] = true;
    }
    let mut v = ipc::unlock_view(
        Some(c),
        &ipc::for_tests::bosses(c),
        None,
        Some(&flags),
        None,
        None,
        None,
        |_| None,
    );
    for n in v.nodes.iter_mut() {
        n.graph = info;
    }
    v
}

const COMPUTED_NOW: GraphInfo = GraphInfo::Computed {
    available_now: true,
    blocked_by: 0,
    fan_out: 0,
    steps_missing: 0,
};

#[test]
fn a_want_you_already_have_says_so() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[1], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::Done);
}

#[test]
fn a_want_with_nothing_in_the_way_is_available_now() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::AvailableNow);
}

#[test]
fn without_the_section_the_view_names_the_route_and_claims_nothing() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        None,
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::NoProfile);
    assert_eq!(w.diagnostics, vec![WantDiagnostic::NoProfile]);
}

#[test]
fn the_no_profile_diagnostic_and_the_rows_cannot_disagree() {
    // The banner and the rows are two readings of one fact: the diagnostic is emitted if and
    // only if every route is NoProfile. A screen that trusted the banner while a row said
    // something else would draw a lie either way round.
    let c = catalog_with_achievements();
    for flags in [None, Some(&[false; 4][..])] {
        let v = view_with(&c, &[], COMPUTED_NOW);
        let w = ipc::want_view(
            Some(&c),
            &ipc::for_tests::bosses(&c),
            &v,
            flags,
            None,
            &Target::Item { id: 2 },
            |_| None,
        );
        let all_rows = w.routes.iter().all(|r| r.state == WantState::NoProfile);
        let banner = w.diagnostics.contains(&WantDiagnostic::NoProfile);
        assert_eq!(all_rows, banner);
    }
}

/// 3 needs 2, 2 needs 1. The ids ascend here, so a test that only checked membership would
/// pass on `missing_chain`'s own order; the point is that the order comes from the edges.
fn chained_graph() -> graph::build::Graph {
    graph::for_tests::from_edges(&[(1, &[]), (2, &[1]), (3, &[2])], &[])
}

const BLOCKED: GraphInfo = GraphInfo::Computed {
    available_now: false,
    blocked_by: 1,
    fan_out: 0,
    steps_missing: 2,
};

#[test]
fn a_chain_is_ordered_the_way_the_queue_orders_it() {
    let c = catalog_with_achievements();
    let g = chained_graph();
    let v = view_with(&c, &[], BLOCKED);
    // Trinket 1 is granted by achievement 3, the deepest node.
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        Some(&READ),
        Some(&g),
        &Target::Trinket { id: 1 },
        |_| None,
    );
    let WantState::Chain { steps, unknown } = &w.routes[0].state else {
        panic!("expected a chain, got {:?}", w.routes[0].state);
    };
    let ids: Vec<u32> = steps
        .iter()
        .filter_map(|n| match n.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1, 2], "prerequisites first, the want excluded");
    assert_eq!(*unknown, 0);

    // The same order the Plan produces, because it is the Plan's own computation.
    let mut q = plan::Queue::from_rows(vec![]);
    let chain = g.missing_chain(
        graph::AchievementId(3),
        &graph::evaluate::FlagsOnly(Some(&READ)),
    );
    let mut rows = chain.clone();
    rows.push(graph::AchievementId(3));
    q.enqueue(
        graph::AchievementId(3),
        &chain,
        &ipc::GraphDeps::new(&g, Some(&READ), &rows),
    );
    let queued: Vec<u32> = q.rows().iter().map(|r| r.achievement.0).collect();
    assert_eq!(
        queued,
        vec![1, 2, 3],
        "the preview is the queue's own order"
    );
}

#[test]
fn a_partly_read_chain_counts_what_it_could_not_interpret() {
    let c = catalog_with_achievements();
    let g = chained_graph();
    let mut v = view_with(&c, &[], BLOCKED);
    // One step of the chain has requirements the graph only partly interprets.
    v.nodes[0].graph = GraphInfo::Partial {
        blocked_by: 1,
        fan_out: 0,
        unknown: 2,
    };
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        Some(&READ),
        Some(&g),
        &Target::Trinket { id: 1 },
        |_| None,
    );
    let WantState::Chain { unknown, .. } = &w.routes[0].state else {
        panic!("expected a chain");
    };
    assert_eq!(*unknown, 1, "one step the app cannot fully read");
}

#[test]
fn no_route_is_an_empty_chain_that_claims_nothing_is_missing() {
    // The failure this guards: reading an empty `missing_chain` as "nothing in the way".
    // Whatever the node says, the state has to name which of the four situations it is.
    let c = catalog_with_achievements();
    let g = chained_graph();
    for info in [
        COMPUTED_NOW,
        BLOCKED,
        GraphInfo::Partial {
            blocked_by: 0,
            fan_out: 0,
            unknown: 1,
        },
    ] {
        for done in [&[][..], &[1, 2, 3][..]] {
            let v = view_with(&c, done, info);
            let w = ipc::want_view(
                Some(&c),
                &ipc::for_tests::bosses(&c),
                &v,
                Some(&READ),
                Some(&g),
                &Target::Trinket { id: 1 },
                |_| None,
            );
            if let WantState::Chain { steps, unknown } = &w.routes[0].state {
                assert!(
                    !steps.is_empty() || *unknown > 0,
                    "an empty chain with nothing unknown is `availableNow`, not a chain",
                );
            }
        }
    }
}

#[test]
fn want_view_json_shape_is_pinned() {
    // `SectionCount` once carried a `core_save::Kind` across the wire and a rename changed
    // the payload with the whole suite green (`summary_shape.rs`). A field that comes out
    // snake_case, or a struct variant whose fields were not renamed, reads as `undefined` in
    // TypeScript with no error at all.
    use serde_json::{json, to_value};
    let c = catalog_with_achievements();
    let g = chained_graph();
    let v = view_with(&c, &[], BLOCKED);
    let w = to_value(ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &v,
        Some(&READ),
        Some(&g),
        &Target::Trinket { id: 1 },
        |_| None,
    ))
    .unwrap();
    assert_eq!(w["wanted"]["kind"], "target");
    assert_eq!(w["wanted"]["target"]["kind"], "item");
    assert_eq!(w["wanted"]["target"]["itemKind"], "trinket");
    assert_eq!(w["routes"][0]["state"]["kind"], "chain");
    assert_eq!(w["routes"][0]["state"]["unknown"], 0);
    assert!(w["routes"][0]["state"]["steps"].is_array());
    assert!(w["routes"][0]["node"]["achievement"].is_object());
    assert_eq!(w["diagnostics"], json!([]));

    // The shapes that have no route, each naming which empty it is.
    let none = to_value(ipc::want_view(
        None,
        ipc::BossKeys::NONE,
        &v,
        Some(&READ),
        Some(&g),
        &Target::Item { id: 2 },
        |_| None,
    ))
    .unwrap();
    assert_eq!(none["wanted"], json!({ "kind": "unresolved" }));
    assert_eq!(none["diagnostics"], json!(["noCatalog"]));
    assert_eq!(none["routes"], json!([]));
}

#[test]
fn a_want_diagnostic_is_a_bare_string_on_the_wire() {
    use serde_json::{json, to_value};

    assert_eq!(
        to_value(WantDiagnostic::NothingUnlocks).expect("serializes"),
        json!("nothingUnlocks")
    );
}
