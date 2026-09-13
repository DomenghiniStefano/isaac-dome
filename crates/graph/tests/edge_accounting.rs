//! Where the edges come from, and where the ones that don't exist go. An edge count on
//! its own is unfalsifiable: this reconciles it against the requirements that produced it,
//! so "459" can be checked rather than believed.

mod support;

#[test]
fn every_edge_is_accounted_for() {
    let Some((c, g)) = support::real_graph() else {
        return;
    };
    let (mut character, mut boss, mut item, mut challenge) = (0u32, 0u32, 0u32, 0u32);
    let (mut no_unlocker, mut disjunction, mut gate, mut unknown, mut judged_none) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let (mut mark, mut counter, mut threshold) = (0u32, 0u32, 0u32);
    let mut produced = 0u32;
    for n in g.nodes() {
        for r in &n.requirements {
            let unlocker = match r {
                graph::model::Requirement::Character { id } => {
                    character += 1;
                    c.character(*id).and_then(|x| x.unlocked_by).is_some()
                }
                graph::model::Requirement::Boss { id } => {
                    boss += 1;
                    c.boss(*id).and_then(|x| x.unlocked_by).is_some()
                }
                graph::model::Requirement::Item { kind, id } => {
                    item += 1;
                    c.item(*kind, *id).and_then(|x| x.unlocked_by).is_some()
                }
                graph::model::Requirement::Challenge { id } => {
                    challenge += 1;
                    match c.challenge(*id).map(|x| x.unlocked_by.len()).unwrap_or(0) {
                        0 => false,
                        1 => true,
                        _ => {
                            disjunction += 1;
                            false
                        }
                    }
                }
                graph::model::Requirement::Gate { .. } => {
                    gate += 1;
                    continue;
                }
                // Answered by the profile, so it produces no edge and is not uninterpreted
                // either: it is counted on its own rather than folded into a neighbour.
                graph::model::Requirement::Mark { .. } => {
                    mark += 1;
                    continue;
                }
                graph::model::Requirement::Counter { .. } => {
                    counter += 1;
                    continue;
                }
                // Same reading, one degree sharper: a threshold produces no edge because
                // the prerequisites of "any three of these" are a disjunction, which is the
                // case `disjunction` above counts for challenges.
                graph::model::Requirement::Threshold { .. } => {
                    threshold += 1;
                    continue;
                }
                graph::model::Requirement::Unknown { .. } => {
                    unknown += 1;
                    continue;
                }
                graph::model::Requirement::None => {
                    judged_none += 1;
                    continue;
                }
            };
            if unlocker {
                produced += 1;
            } else {
                no_unlocker += 1;
            }
        }
    }
    let edges: u32 = g.nodes().iter().map(|n| n.prerequisites.len() as u32).sum();
    eprintln!(
        "requirements: character {character}, boss {boss}, item {item}, challenge {challenge}, \
         gate {gate}, mark {mark}, counter {counter}, threshold {threshold}, \
         judged-not-a-prerequisite {judged_none}, uninterpreted {unknown}"
    );
    eprintln!(
        "of those, {produced} name something the game says is unlocked by an achievement; \
         {no_unlocker} name something with no unlocker at all ({disjunction} of them \
         challenges with several)"
    );
    eprintln!(
        "edges after dedup within a node: {edges} (so {} duplicates collapsed)",
        produced.saturating_sub(edges)
    );
    assert!(
        edges <= produced,
        "more edges than requirements that could produce one: {edges} > {produced}"
    );
    assert!(
        produced.saturating_sub(edges) * 4 < produced,
        "over a quarter of the edges collapsed as duplicates ({} of {produced}): that is \
         not dedup, that is a bug in how a node's requirements are read",
        produced - edges
    );
}
