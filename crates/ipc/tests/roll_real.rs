//! The deck's accounting over the real series.
//!
//! Properties, not pinned numbers: "every target is either drawable or named as excluded"
//! holds whatever the profile does next and whatever era the file is from, which a fixed 442
//! would not. The totals themselves come from the tables that measured them.

mod support;

use support::counters;
use test_support::{dated_series, SERIES};

/// The two editions walked separately: they are two profiles, and a jump between them would
/// read as progress. Same table as `marks_real.rs`, same reason.
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
    ipc::ROSTER.len() * ipc::BOSSES.len() + ipc::ROSTER.len()
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
    // Not an unconditional property: a save that *locates* a cell this crate previously read as
    // `Unreadable` can legitimately grow the drawable deck, since an unreadable cell excludes a
    // target the same way a taken one does. It holds on this series because nothing in it is
    // known to cross such a boundary — measured, not assumed — which is exactly why the vacuity
    // guard below asks the series to show at least one pair, not merely at least one sample.
    // `samples/` holds 15 `rep_` files and 1 `rep+`: the `rep_` half genuinely walks 14 pairs,
    // the `rep+` half is a no-op on its own.
    let mut any_sample = false;
    let mut any_pair = false;
    for name in SERIES {
        let sizes: Vec<usize> = dated_series(name)
            .iter()
            .filter_map(|p| counters(p))
            .map(|c| deck_of(&c, false).size)
            .collect();
        any_sample |= !sizes.is_empty();
        any_pair |= sizes.windows(2).next().is_some();
        for pair in sizes.windows(2) {
            assert!(
                pair[1] <= pair[0],
                "the deck grew across {name}: {} then {}",
                pair[0],
                pair[1]
            );
        }
    }
    if !any_sample {
        // `dated_series` has already declared the skip on stderr.
        return;
    }
    assert!(
        any_pair,
        "every name in the series had fewer than two samples: the progression property was \
         never actually exercised"
    );
}
