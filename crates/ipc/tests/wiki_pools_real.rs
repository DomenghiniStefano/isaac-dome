//! `item_pools` against the installed game (design decision 4 of
//! `2026-09-26-wiki-complete-design.md`): The Sad Onion (item 1) really is in the `treasure`
//! pool.
//!
//! The expected fact is read **independently** of `catalog::itempools::parse` — a text
//! search of `itempools.xml`'s own bytes, extracted the same way
//! `crates/unpack/examples/extract.rs` does (`unpack::ResourceSet::read`, no XML parser), for
//! the `<Pool Name="treasure">` block and an `Id="1"` inside it — so the test does not just
//! check the reader against itself.

mod support;

#[test]
fn the_sad_onion_is_in_the_treasure_pool_on_the_installed_game() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("wiki dataset not embedded");
        return;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let c = catalog::Catalog::build(|p| rs.read(p));

    let target = wiki::Target::Item { id: 1 };
    let pools = ipc::item_pools(&c, Some(ds), &target)
        .expect("The Sad Onion (item 1) is a real collectible on every known snapshot");
    assert!(
        pools.iter().any(|p| p.label.starts_with("Treasure Room")),
        "The Sad Onion should be in the treasure pool, via `POOL_ARTICLES`: {pools:?}"
    );

    // Independent check, bypassing `catalog::itempools::parse` entirely: read the raw file
    // and confirm `Id="1"` occurs inside the `treasure` pool's own block.
    let bytes = rs
        .read("itempools.xml")
        .expect("itempools.xml is in the archives");
    let text = String::from_utf8_lossy(&bytes);
    let start = text
        .find("Pool Name=\"treasure\"")
        .expect("the installed game has a treasure pool");
    let end = text[start..]
        .find("</Pool>")
        .map(|i| start + i)
        .unwrap_or(text.len());
    assert!(
        text[start..end].contains("Id=\"1\" "),
        "the treasure pool's own block does not mention item 1"
    );
}
