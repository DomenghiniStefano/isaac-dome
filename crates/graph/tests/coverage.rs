//! How much of the catalog the graph actually covers. A total edge count hides the shape:
//! a graph where half the nodes were never described would have the same total as one
//! where every node is described and most genuinely gate nothing.

mod support;

#[test]
fn every_node_the_catalog_has_was_described_by_the_rules() {
    let Some((c, g)) = support::real_graph() else {
        return;
    };
    let rules = graph::rules::embedded().expect("embedded rules");
    let described = g
        .nodes()
        .iter()
        .filter(|n| !rules.refs(n.achievement).is_empty())
        .count();
    let with_prereq = g
        .nodes()
        .iter()
        .filter(|n| !n.prerequisites.is_empty())
        .count();
    let unlocking = c
        .achievements()
        .filter(|a| !c.unlocks(a.id).is_empty())
        .count();
    let mut by_count = std::collections::BTreeMap::new();
    for n in g.nodes() {
        *by_count.entry(n.prerequisites.len()).or_insert(0u32) += 1;
    }
    eprintln!(
        "nodes {}, described by the rules {described}, with at least one prerequisite \
         {with_prereq}",
        g.nodes().len()
    );
    eprintln!("achievements that unlock something: {unlocking}");
    eprintln!("prerequisites per node: {by_count:?}");

    // The rules come from the wiki, keyed by the game's achievement id; the nodes come
    // from the catalog, keyed the same way. If the two id spaces had drifted, nodes would
    // silently lose their requirements and the edge total alone would not show it.
    //
    // The check isn't a percentage — a threshold picked to fit today's number proves
    // nothing tomorrow. A node with no requirements has to be explainable: the wiki must
    // still *have* that achievement, and its requirement must be text with no typed ref
    // ("Donate 10 Coins to the Donation Machine"), which is the threshold family M2
    // deliberately leaves out. A node missing from the wiki entirely is drift.
    let dataset = wiki::Dataset::embedded().expect("embedded wiki dataset");
    let absent: Vec<graph::AchievementId> = g
        .nodes()
        .iter()
        .filter(|n| rules.refs(n.achievement).is_empty())
        .filter(|n| !dataset.achievements.contains_key(&n.achievement.0))
        .map(|n| n.achievement)
        .collect();
    assert!(
        absent.is_empty(),
        "nodes with no requirements that the wiki doesn't have either: {absent:?} — the \
         two id spaces have drifted apart"
    );
    eprintln!(
        "{} nodes carry no typed ref, all present in the wiki as text-only requirements \
         (the counter family, out of scope for M2)",
        g.nodes().len() - described
    );
}

#[test]
fn zero_prerequisites_is_not_the_same_as_available_now() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let (mut done, mut available, mut blocked, mut partial) = (0u32, 0u32, 0u32, 0u32);
    let mut zero_but_partial = 0u32;
    for n in g.nodes() {
        // Done comes first, whatever the graph says about it: an achievement already
        // earned needs no recommendation, and counting it under "can't say" would inflate
        // that number with work that is finished.
        if flags
            .get(n.achievement.0 as usize)
            .copied()
            .unwrap_or(false)
        {
            done += 1;
            continue;
        }
        match e.node(n.achievement) {
            Some(graph::evaluate::NodeInfo::Computed {
                available_now,
                blocked_by,
                ..
            }) => {
                if *available_now {
                    available += 1;
                } else {
                    blocked += 1;
                    assert!(
                        *blocked_by > 0,
                        "not done, not available, and blocked by nothing"
                    );
                }
            }
            Some(graph::evaluate::NodeInfo::Partial { .. }) => {
                partial += 1;
                if n.prerequisites.is_empty() {
                    zero_but_partial += 1;
                }
            }
            None => {}
        }
    }
    eprintln!(
        "on this profile: done {done}, unlockable now {available}, blocked {blocked}, \
         can't say {partial} (of which {zero_but_partial} have no prerequisite at all — \
         zero edges, and still not a recommendation)"
    );
    assert!(
        available > 0,
        "nothing is unlockable on a profile that isn't finished: the graph would be useless"
    );
}
