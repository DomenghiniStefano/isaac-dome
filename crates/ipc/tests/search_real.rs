//! Search on the real dataset. B5 asks for the scan to be **measured, not assumed**: the
//! timings are printed, never pinned — a number that depends on the machine would fail on
//! someone else's. What is pinned is the answer: the best-known query names its own page.

use std::time::Instant;

use catalog::Catalog;
use ipc::{search, IconRef, SearchIndex, Target};
use unpack::ResourceSet;
use wiki::Dataset;

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

#[test]
fn brimstone_finds_its_own_page_first() {
    let Ok(ds) = Dataset::embedded() else {
        test_support::skip("the embedded dataset didn't load");
        return;
    };
    let built = Instant::now();
    let index = SearchIndex::build(Ok(ds));
    eprintln!(
        "search: index of {} pages built in {} ms",
        index.len(),
        built.elapsed().as_millis()
    );
    // The catalog is a bonus here: without the game the titles still answer.
    let catalog = test_support::packed_dir().map(|p| {
        let rs = ResourceSet::open(&p);
        Catalog::build(|f| rs.read(f))
    });
    for query in ["brimstone", "the lost", "mom's heart"] {
        let at = Instant::now();
        let view = search(
            &index,
            catalog.as_ref(),
            &catalog
                .as_ref()
                .map(ipc::for_tests::bosses)
                .unwrap_or_default(),
            None,
            query,
            300,
            link,
        );
        eprintln!(
            "search: {query:?} → {} hits of {} in {} ms",
            view.hits.len(),
            view.total,
            at.elapsed().as_millis()
        );
    }
    let view = search(
        &index,
        catalog.as_ref(),
        &catalog
            .as_ref()
            .map(ipc::for_tests::bosses)
            .unwrap_or_default(),
        None,
        "brimstone",
        30,
        link,
    );
    assert_eq!(
        view.hits.first().map(|h| h.target.clone()),
        Some(Target::Item { id: 118 }),
        "the item's own page must rank first for its exact name"
    );
}
