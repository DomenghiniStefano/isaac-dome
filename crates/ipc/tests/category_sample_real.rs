//! `category_sample` against the installed game: every deliberately chosen id really is a
//! row the archives have. A hardcoded id that stops existing (a patch renumbers a boss, an
//! item is cut) is exactly the kind of drift `Don't name from a guess` warns about — this is
//! the test that would catch it.

// A test that extracts one variant panics on every other, the ones added later included: here
// the wildcard *is* the assertion, and it fails loudly on a new variant instead of hiding it.
#![allow(clippy::wildcard_enum_match_arm)]

mod support;

use ipc::{category_sample, IconSource, WikiPageCategory, WIKI_PAGE_CATEGORIES};
use support::real_catalog;

#[test]
fn every_sample_with_a_reference_resolves_on_the_installed_game() {
    let Some(c) = real_catalog() else { return };
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("wiki dataset not embedded");
        return;
    };
    let bosses = ipc::for_tests::bosses(&c);
    let mut checked = 0;
    for category in WIKI_PAGE_CATEGORIES {
        let Some(r) = category_sample(category) else {
            // Transformations, Stages, Versions: declared to have no picture at all.
            continue;
        };
        let source = ipc::icon_source(&c, &bosses, Some(ds), &r);
        assert!(
            source.is_some(),
            "{category:?}'s sample {r:?} does not resolve on the installed game"
        );
        checked += 1;
    }
    eprintln!("sample: {checked} category samples resolved on the installed game");
    assert!(checked >= 9, "too few categories carry a sample: {checked}");
}

#[test]
fn the_three_kinds_with_no_picture_at_all_declare_no_sample() {
    for category in [
        WikiPageCategory::Transformations,
        WikiPageCategory::Stages,
        WikiPageCategory::Versions,
    ] {
        assert_eq!(category_sample(category), None, "{category:?}");
    }
}

#[test]
fn cards_and_pickups_compose_their_own_picture_not_a_page() {
    // The two categories with no page to draw through (measured: no data file maps an
    // individual card face to a sprite) still get an honest, composed picture of *a* card
    // or *a* pickup — `IconSource::Entity`, never `IconSource::Sprite`.
    let Some(c) = real_catalog() else { return };
    let bosses = ipc::for_tests::bosses(&c);
    for category in [WikiPageCategory::CardsAndRunes, WikiPageCategory::Pickups] {
        let r = category_sample(category).expect("declared");
        match ipc::icon_source(&c, &bosses, None, &r) {
            Some(IconSource::Entity { .. }) => {}
            other => panic!("{category:?}: expected composed entity art, got {other:?}"),
        }
    }
}
