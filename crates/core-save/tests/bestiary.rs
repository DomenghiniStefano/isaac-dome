//! Section 10's own layout, on the real samples.
//!
//! The section is **self-describing**: a preamble, a declared total, a declared number of
//! tallies, then that many `(id, size)` blocks each holding a sorted key → count list.
//! What is asserted here are the invariants that identified the layout in the first
//! place — the declared total accounts for every block, and the section is consumed to
//! the word — because those are what a future patch would break, and they hold on saves
//! nobody has collected yet.
//!
//! What each of the tallies *counts* is not asserted anywhere, because nobody has
//! measured it. They keep the id the file gives them.

use core_save::Save;

mod helpers;
use helpers::{every_dated_save, series, SERIES};

/// A tally's declared size is in units of two bytes, and a record is eight: four units.
/// Stated once here so the arithmetic below reads as arithmetic and not as a constant
/// somebody chose.
const UNITS_PER_RECORD: usize = 4;

/// The four tallies consume the section exactly, but for **one word**. That word is not
/// slack: it is there in every save, it grows with the profile, and it is the reason this
/// test asserts one rather than zero. The expectation started at zero, taken from the
/// throwaway script that mapped the layout; the reader disagreed, and the script turned
/// out to have an off-by-one that ate it. A test whose number comes from a script is
/// only as good as the script.
#[test]
fn the_layout_accounts_for_the_section_but_for_one_word() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        assert_eq!(
            b.trailing.len(),
            1,
            "{name}: trailing words {:?}",
            b.trailing
        );
    });
}

/// Whatever the trailing word counts, it counts something a profile only accumulates.
/// Stated as a property because its value is a fixture of a day and this is not.
#[test]
fn the_trailing_word_never_goes_backwards() {
    SERIES.iter().for_each(|suffix| {
        series(suffix).windows(2).for_each(|w| {
            let (ref before, _, ref a) = w[0];
            let (ref after, _, ref b) = w[1];
            let word = |s: &Save| s.bestiary_tallies().expect("section 10").trailing[0];
            assert!(
                word(b) >= word(a),
                "{before} → {after}: the trailing word fell from {} to {}",
                word(a),
                word(b)
            );
        });
    });
}

#[test]
fn the_declared_total_accounts_for_every_tally() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        let summed: usize = b
            .tallies
            .iter()
            .map(|t| t.records.len() * UNITS_PER_RECORD)
            .sum();
        assert_eq!(summed, b.declared_units as usize, "{name}");
    });
}

#[test]
fn the_number_of_tallies_is_read_from_the_file() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        assert_eq!(b.tallies.len(), b.declared_tallies as usize, "{name}");
    });
}

/// Inside a tally the game writes the entities in key order, once each. This is the
/// property that told four concatenated lists from one unsorted one: read as a single
/// list the keys descend three times and 445 of them repeat, and both disappear the
/// moment the boundaries are read from the file instead of guessed.
#[test]
fn each_tally_is_ascending_and_holds_an_entity_once() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        b.tallies.iter().for_each(|t| {
            let keys: Vec<u32> = t.records.iter().map(|r| r.entity.packed()).collect();
            let ascending = keys.windows(2).all(|w| w[0] < w[1]);
            assert!(ascending, "{name}: tally {} is not in key order", t.id);
        });
    });
}

/// The four tallies are four counts of the *same* entities, not four different subjects:
/// an entity that appears in one appears in others, with its own count in each. Without
/// this the reader could just as well be handing back four unrelated lists.
#[test]
fn an_entity_is_counted_in_more_than_one_tally() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        let first = b.tallies.first().expect("at least one tally");
        let shared = first.records.iter().filter(|r| {
            b.tallies
                .iter()
                .skip(1)
                .any(|t| t.records.iter().any(|o| o.entity == r.entity))
        });
        assert!(
            shared.count() > 0,
            "{name}: no entity is counted twice, so these are not tallies of one space"
        );
    });
}

/// A profile only ever meets more of the game. Across one profile's snapshots no tally
/// loses a record — the property that survives a patch, where a pinned count would not.
#[test]
fn no_tally_ever_loses_a_record() {
    SERIES.iter().for_each(|suffix| {
        let saves = series(suffix);
        saves.windows(2).for_each(|w| {
            let (ref before, _, ref a) = w[0];
            let (ref after, _, ref b) = w[1];
            let (ba, bb) = (
                a.bestiary_tallies().expect("section 10"),
                b.bestiary_tallies().expect("section 10"),
            );
            ba.tallies.iter().for_each(|t| {
                let later = bb.tallies.iter().find(|o| o.id == t.id);
                let later =
                    later.unwrap_or_else(|| panic!("{before} → {after}: tally {} vanished", t.id));
                assert!(
                    later.records.len() >= t.records.len(),
                    "{before} → {after}: tally {} shrank from {} to {}",
                    t.id,
                    t.records.len(),
                    later.records.len()
                );
            });
        });
    });
}

/// An era fixture, and named as one: every save collected so far declares four tallies.
/// If a patch adds a fifth this goes red, which is the point — the reader keeps working
/// (the count comes from the file) while the fact that the world changed stays visible.
#[test]
fn every_sample_so_far_declares_four_tallies() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let b = save.bestiary_tallies().expect("section 10 is present");
        assert_eq!(b.declared_tallies, 4, "{name}");
    });
}
