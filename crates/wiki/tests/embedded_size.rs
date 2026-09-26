//! The embedded dataset ships in every package: decision 3 of
//! `docs/superpowers/specs/2026-09-26-wiki-complete-design.md` sets its ceiling at 4 MB,
//! twice what the single-file embed measured before `dataset/wiki.json` was split into
//! `dataset/wiki/` (1.9 MB).

const CEILING: usize = 4 * 1024 * 1024;

#[test]
fn embedded_deflate_stays_under_the_ceiling() {
    let len = wiki::embedded_len();
    assert!(
        len <= CEILING,
        "embedded deflate is {len} bytes, over the {CEILING} byte ceiling"
    );
}
