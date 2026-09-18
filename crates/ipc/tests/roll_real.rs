//! The deck's accounting over the real series.
//!
//! Properties, not pinned numbers: "every target is either drawable or named as excluded"
//! holds whatever the profile does next and whatever era the file is from, which a fixed 442
//! would not. The totals themselves come from the tables that measured them.

use core_save::{Kind, Save};
use test_support::dated_series;

/// The two editions walked separately: they are two profiles, and a jump between them would
/// read as progress. Same table as `marks_real.rs`, same reason.
const SERIES: [&str; 2] = ["rep_persistentgamedata1.dat", "rep+persistentgamedata1.dat"];

fn counters(path: &std::path::Path) -> Option<Vec<u32>> {
    Save::open(path).ok()?.u32s(Kind::Counters)
}

fn deck_of(counters: &[u32], include_taken: bool) -> ipc::DeckView {
    let doc = roll::Document {
        preset: roll::Preset {
            include_taken,
            ..roll::Preset::default()
        },
        ..roll::Document::default()
    };
    ipc::roll_view(
        ipc::RollInputs {
            counters: Some(counters),
            flags: None,
            catalog: None,
            document: Ok(&doc),
            store_reason: None,
        },
        |_| None,
    )
    .deck
}

fn every_target() -> usize {
    ipc::CHARACTERS.len() * ipc::BOSSES.len() + ipc::CHARACTERS.len()
}

#[test]
fn every_target_of_every_real_save_lands_in_exactly_one_bucket() {
    for name in SERIES {
        for path in dated_series(name) {
            let Some(c) = counters(&path) else { continue };
            for include_taken in [false, true] {
                let d = deck_of(&c, include_taken);
                assert_eq!(
                    d.size + d.taken + d.unreadable + d.locked + d.filtered,
                    every_target(),
                    "{} loses a target with include_taken = {include_taken}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn a_real_save_shows_both_a_taken_target_and_a_drawable_one() {
    // The vacuity guard. The property above holds trivially on a profile whose every cell is
    // unreadable — which is exactly what a broken `Space` would produce — so the series has to
    // be shown to contain the thing the property is about.
    let mut any_sample = false;
    let mut seen_taken = false;
    let mut seen_drawable = false;
    for name in SERIES {
        for path in dated_series(name) {
            let Some(c) = counters(&path) else { continue };
            any_sample = true;
            let d = deck_of(&c, false);
            seen_taken |= d.taken > 0;
            seen_drawable |= d.size > 0;
        }
    }
    if !any_sample {
        // `dated_series` has already declared the skip on stderr.
        return;
    }
    assert!(
        seen_taken,
        "no real save shows a taken mark: the status derivation is dead"
    );
    assert!(seen_drawable, "no real save leaves anything to draw");
}

#[test]
fn the_deck_only_ever_shrinks_as_a_profile_progresses() {
    // Marks are taken and never given back, so with the default preset the drawable deck
    // cannot grow across the dated series. The progression property, applied to this screen.
    for name in SERIES {
        let sizes: Vec<usize> = dated_series(name)
            .iter()
            .filter_map(|p| counters(p))
            .map(|c| deck_of(&c, false).size)
            .collect();
        for pair in sizes.windows(2) {
            assert!(
                pair[1] <= pair[0],
                "the deck grew across {name}: {} then {}",
                pair[0],
                pair[1]
            );
        }
    }
}
