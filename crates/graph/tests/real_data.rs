//! Invariants that survive any patch of the game and any snapshot of the wiki. They are
//! deliberately not pinned numbers: the graph moves with the world, the invariants don't.

use graph::evaluate::NodeInfo;

mod support;

#[test]
fn available_now_and_blocked_by_never_contradict_each_other() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    for n in g.nodes() {
        let Some(NodeInfo::Computed {
            available_now,
            blocked_by,
            ..
        }) = e.node(n.achievement)
        else {
            continue;
        };
        assert!(
            !(*available_now && *blocked_by > 0),
            "node {} claims available_now with {blocked_by} prerequisites missing",
            n.achievement
        );
    }
}

#[test]
fn steps_missing_is_zero_exactly_when_the_node_is_done_or_available() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    for n in g.nodes() {
        let Some(NodeInfo::Computed {
            available_now,
            steps_missing,
            ..
        }) = e.node(n.achievement)
        else {
            continue;
        };
        let done = flags.get(n.achievement as usize).copied().unwrap_or(false);
        assert_eq!(
            *steps_missing == 0,
            done || *available_now,
            "node {}: steps_missing={steps_missing}, done={done}, available={available_now}",
            n.achievement
        );
    }
}

#[test]
fn no_edge_points_outside_the_catalog() {
    let Some((g, _)) = support::real_graph_and_flags() else {
        return;
    };
    let known: std::collections::BTreeSet<u32> = g.nodes().iter().map(|n| n.achievement).collect();
    for n in g.nodes() {
        for p in &n.prerequisites {
            assert!(
                known.contains(p),
                "node {} points at achievement {p}, which the catalog doesn't have — this \
                 is the silent failure mode of resolution by name",
                n.achievement
            );
        }
    }
}

#[test]
fn the_graph_has_the_shape_this_era_measured() {
    let Some((g, _)) = support::real_graph_and_flags() else {
        return;
    };
    let unknown: u32 = g.nodes().iter().map(|n| n.unknown.len() as u32).sum();
    let edges: usize = g.nodes().iter().map(|n| n.prerequisites.len()).sum();
    eprintln!(
        "graph: {} nodes, {edges} edges, {unknown} unknown requirements, {} diagnostics",
        g.nodes().len(),
        g.diagnostics().len()
    );
    // The breakdown is the useful half: an edge count on its own says nothing about
    // *where* the graph thinned out.
    let mut by_kind: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    for n in g.nodes() {
        for r in &n.requirements {
            let k = match r {
                graph::model::Requirement::Character { .. } => "character",
                graph::model::Requirement::Boss { .. } => "boss",
                graph::model::Requirement::Challenge { .. } => "challenge",
                graph::model::Requirement::Item { .. } => "item",
                graph::model::Requirement::Gate { .. } => "gate",
                graph::model::Requirement::Mark { .. } => "mark",
                graph::model::Requirement::Counter { .. } => "counter",
                graph::model::Requirement::Unknown { .. } => "unknown",
                graph::model::Requirement::None => "none",
            };
            *by_kind.entry(k).or_insert(0) += 1;
        }
    }
    eprintln!("requirements by kind: {by_kind:?}");
    // What is still uninterpreted, by name: this is the curation queue, and printing it
    // is how the next person knows where the graph stops seeing.
    let mut unknown_labels: std::collections::BTreeMap<&str, u32> =
        std::collections::BTreeMap::new();
    for n in g.nodes() {
        for r in &n.requirements {
            if let graph::model::Requirement::Unknown { label } = r {
                *unknown_labels.entry(label.as_str()).or_insert(0) += 1;
            }
        }
    }
    eprintln!("uninterpreted labels: {unknown_labels:?}");
    for d in g.diagnostics().iter().take(20) {
        eprintln!("diagnostic: {d:?}");
    }
    // Not a floor on the edge count: the spec's "2,157 edges" was measured on a different
    // relation (how many achievements unlock a thing, summed over refs without dedup) and
    // says nothing about this graph.
    //
    // Not a ratio either. A ratio conflates two different things — "we identified what the
    // ref points at" and "we can state its prerequisite" — and Delirium is the case that
    // separates them: we know exactly what it is, and its gate is run progress that the
    // model has no way to say. What the design actually promises is that **nothing is
    // unknown by inattention**, so that is what gets asserted: every uninterpreted
    // requirement traces back to a target somebody judged.
    let rules = graph::rules::embedded().expect("embedded rules");
    let judged: std::collections::BTreeSet<&str> = rules
        .targets()
        .iter()
        .filter(|t| rules.verdict(&t.key).is_some())
        .map(|t| t.label.as_str())
        .collect();
    let unjudged: Vec<&str> = unknown_labels
        .keys()
        .copied()
        .filter(|l| !judged.contains(l))
        .collect();
    assert!(
        unjudged.is_empty(),
        "uninterpreted requirements that nobody judged: {unjudged:?} — an inventory row \
         and a verdict are missing for each"
    );
    let total: u32 = g
        .nodes()
        .iter()
        .map(|n| n.requirements.len() as u32)
        .sum::<u32>();
    eprintln!(
        "judged inexpressible: {unknown} of {total} requirements ({}%)",
        unknown * 100 / total.max(1)
    );
}

#[test]
fn every_resolvable_requirement_produced_its_edge() {
    let Some((c, g)) = support::real_graph() else {
        return;
    };
    for n in g.nodes() {
        for r in &n.requirements {
            // The three kinds whose edge is a single `unlocked_by` on the catalog side.
            // A boss usually has none, and a challenge can have several — those are
            // covered by their own diagnostics, not here.
            let expected = match r {
                graph::model::Requirement::Character { id } => {
                    c.character(*id).and_then(|ch| ch.unlocked_by)
                }
                graph::model::Requirement::Item { kind, id } => {
                    c.item(*kind, *id).and_then(|i| i.unlocked_by)
                }
                graph::model::Requirement::Boss { id } => c.boss(*id).and_then(|b| b.unlocked_by),
                graph::model::Requirement::Challenge { .. }
                | graph::model::Requirement::Gate { .. }
                | graph::model::Requirement::Mark { .. }
                | graph::model::Requirement::Counter { .. }
                | graph::model::Requirement::Unknown { .. }
                | graph::model::Requirement::None => continue,
            };
            let Some(a) = expected else { continue };
            // A node that unlocks the very thing it names is not its own prerequisite:
            // that edge is dropped on purpose, with a `SelfPrerequisite` diagnostic.
            if a.0 == n.achievement {
                continue;
            }
            assert!(
                n.prerequisites.contains(&a.0),
                "node {} has requirement {r:?}, unlocked by achievement {}, and no edge for it",
                n.achievement,
                a.0
            );
        }
    }
}
