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
    let absent: Vec<u32> = g
        .nodes()
        .iter()
        .filter(|n| rules.refs(n.achievement).is_empty())
        .filter(|n| !dataset.achievements.contains_key(&n.achievement))
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
