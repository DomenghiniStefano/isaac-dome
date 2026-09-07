//! The rules compiled into the binary are the ones committed in `rules/`.

#[test]
fn the_embedded_rules_are_the_committed_ones() {
    let rules = graph::rules::embedded().expect("the embedded rules parse");
    assert!(
        !rules.targets().is_empty(),
        "the inventory must not be empty: the generator ran on a real snapshot"
    );
    assert!(
        rules
            .targets()
            .iter()
            .all(|t| !t.verdict_required || rules.verdict(&t.key).is_some()),
        "the embedded rules carry a verdict for every target that needs one"
    );
    assert_eq!(
        rules.generated_from().snapshot_at,
        "2026-09-04T17:33:31Z",
        "era of the numbers pinned across this suite"
    );
}

#[test]
fn the_embedded_rules_are_parsed_once() {
    // `OnceLock`: two calls give the same allocation, so building the graph twice doesn't
    // re-parse a 12k-line file.
    let a = graph::rules::embedded().expect("parses");
    let b = graph::rules::embedded().expect("parses");
    assert!(std::ptr::eq(a, b));
}
