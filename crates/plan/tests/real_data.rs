//! The queue against the real graph and a real profile. Synthetic queues prove the
//! algorithm; this proves the algorithm is fed the right thing.

mod support;

use support::GraphDeps;

#[test]
fn enqueueing_a_real_achievement_queues_exactly_its_missing_chain() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    // The node with the deepest chain on this profile: the case that exercises the walk.
    let (depth, deepest) = g
        .nodes()
        .iter()
        .map(|n| {
            (
                g.missing_chain(n.achievement, Some(&flags)).len(),
                n.achievement,
            )
        })
        .max()
        .expect("the catalog is not empty");
    eprintln!("deepest chain on this profile: {depth} steps for node {deepest}");
    assert!(
        depth > 0,
        "a finished profile would make this test vacuous: it needs something left to do"
    );

    let chain = g.missing_chain(deepest, Some(&flags));
    let mut q = plan::Queue::default();
    let ids: Vec<u32> = chain.iter().copied().chain([deepest]).collect();
    q.enqueue(deepest, &chain, &GraphDeps::new(&g, Some(&flags), &ids));

    assert_eq!(
        q.rows().len(),
        chain.len() + 1,
        "the wish and its chain, nothing else"
    );
    assert!(
        q.rows()
            .iter()
            .all(|r| !flags.get(r.achievement as usize).copied().unwrap_or(false)),
        "a chain must never contain something already done"
    );
    assert_eq!(
        q.rows().last().map(|r| r.achievement),
        Some(deepest),
        "the wish is last: its steps come before it"
    );

    // The order the queue produced has to satisfy the graph, row by row.
    let order: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    for (i, id) in order.iter().enumerate() {
        for prereq in g.missing_chain(*id, Some(&flags)) {
            if let Some(j) = order.iter().position(|x| *x == prereq) {
                assert!(j < i, "{prereq} must come before {id}, got {order:?}");
            }
        }
    }
}

#[test]
fn two_real_wishes_that_share_a_step_keep_one_row_for_it() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    // Two nodes whose chains overlap: the case `origins` exists for. Found rather than
    // hard-coded, because which achievements share a step moves with the game.
    let with_chain: Vec<(u32, Vec<u32>)> = g
        .nodes()
        .iter()
        .map(|n| (n.achievement, g.missing_chain(n.achievement, Some(&flags))))
        .filter(|(_, c)| !c.is_empty())
        .collect();
    let pair = with_chain.iter().enumerate().find_map(|(i, (a, ca))| {
        with_chain[i + 1..]
            .iter()
            .find(|(b, cb)| a != b && cb.iter().any(|s| ca.contains(s)))
            .map(|(b, cb)| (*a, ca.clone(), *b, cb.clone()))
    });
    let Some((a, ca, b, cb)) = pair else {
        test_support::skip("no two achievements on this profile share a missing step");
        return;
    };
    let shared: Vec<u32> = ca.iter().filter(|s| cb.contains(s)).copied().collect();
    eprintln!("{a} and {b} share {} step(s): {shared:?}", shared.len());

    let mut q = plan::Queue::default();
    let ids_a: Vec<u32> = ca.iter().copied().chain([a]).collect();
    q.enqueue(a, &ca, &GraphDeps::new(&g, Some(&flags), &ids_a));
    let mut ids_b: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    ids_b.extend(cb.iter().copied());
    ids_b.push(b);
    q.enqueue(b, &cb, &GraphDeps::new(&g, Some(&flags), &ids_b));

    let step = shared[0];
    let rows: Vec<&plan::Row> = q.rows().iter().filter(|r| r.achievement == step).collect();
    assert_eq!(rows.len(), 1, "one row, however many wishes need it");
    assert!(
        rows[0].origins.contains(&a) && rows[0].origins.contains(&b),
        "both wishes are on record as needing it: {:?}",
        rows[0].origins
    );

    // Removing one wish must not take a step the other still needs.
    q.remove(a);
    assert!(
        q.rows().iter().any(|r| r.achievement == step),
        "{step} still serves {b}: removing {a} does not take it"
    );
}
