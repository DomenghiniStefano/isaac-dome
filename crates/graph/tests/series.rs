//! Properties over the historical series. These find what a pinned number cannot: they
//! compare one era against the next, and the graph has to stay coherent across both.

use graph::evaluate::NodeInfo;

mod support;

#[test]
fn an_available_node_is_never_taken_away() {
    let Some(series) = support::series_evals() else {
        return;
    };
    for pair in series.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        for (id, info) in &before.1 {
            let NodeInfo::Computed {
                available_now: true,
                ..
            } = info
            else {
                continue;
            };
            let done_after = after.2.get(*id as usize).copied().unwrap_or(false);
            let still = matches!(
                after.1.get(id),
                Some(NodeInfo::Computed {
                    available_now: true,
                    ..
                })
            );
            assert!(
                done_after || still,
                "node {id} was unlockable in {} and is neither done nor unlockable in {}: \
                 achievements are not lost in this game, so the graph is wrong",
                before.0,
                after.0
            );
        }
    }
}

#[test]
fn steps_missing_never_grows() {
    let Some(series) = support::series_evals() else {
        return;
    };
    for pair in series.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        for (id, info) in &before.1 {
            let (
                NodeInfo::Computed {
                    steps_missing: was, ..
                },
                Some(NodeInfo::Computed {
                    steps_missing: now, ..
                }),
            ) = (info, after.1.get(id))
            else {
                continue;
            };
            assert!(
                now <= was,
                "node {id}: {was} steps in {} became {now} in {} — progress does not undo",
                before.0,
                after.0
            );
        }
    }
}

#[test]
fn the_series_actually_moves() {
    // A guard on the guards: if every era evaluated identically, the two properties above
    // would pass without ever being tested. Fourteen months of play have to show up.
    let Some(series) = support::series_evals() else {
        return;
    };
    let first = &series[0];
    let last = &series[series.len() - 1];
    let done_first = first.2.iter().filter(|d| **d).count();
    let done_last = last.2.iter().filter(|d| **d).count();
    eprintln!(
        "series: {} done in {}, {} done in {}",
        done_first, first.0, done_last, last.0
    );
    assert!(
        done_last > done_first,
        "the series doesn't progress: the properties above would be vacuous"
    );
}
