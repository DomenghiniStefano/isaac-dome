//! The draw is a pure function of `(deck, seed)`. The seed comes from `app`; this crate never
//! reads a clock and never calls an RNG of its own, which is what makes these tests say
//! anything at all.

use roll::{deck, draw, CellValue, Deck, Excluded, Preset, Space, Target};

fn space(rows: usize, columns: usize) -> Space {
    Space::new(
        rows,
        columns,
        0,
        vec![CellValue::Known { bits: 0 }; rows * columns],
        vec![true; rows],
    )
    .expect("declared shape")
}

#[test]
fn the_same_seed_draws_the_same_target() {
    let d = deck(&space(4, 5), &Preset::default());
    assert_eq!(draw(&d, 12_345), draw(&d, 12_345));
}

#[test]
fn a_drawn_target_is_one_the_deck_holds() {
    let d = deck(&space(4, 5), &Preset::default());
    for seed in 0..500u64 {
        let target = draw(&d, seed).expect("a deck with targets draws one");
        assert!(d.targets.contains(&target), "seed {seed} drew a stranger");
    }
}

#[test]
fn an_empty_deck_draws_nothing() {
    let d = Deck {
        targets: Vec::new(),
        excluded: Excluded::default(),
    };
    assert_eq!(draw(&d, 1), None);
    assert_eq!(draw(&d, 0), None);
}

#[test]
fn a_deck_of_one_draws_that_one_whatever_the_seed() {
    let d = Deck {
        targets: vec![Target::Greedier { character: 3 }],
        excluded: Excluded::default(),
    };
    for seed in [0, 1, u64::MAX, 7_919] {
        assert_eq!(draw(&d, seed), Some(Target::Greedier { character: 3 }));
    }
}

#[test]
fn every_target_of_a_small_deck_is_reachable() {
    // A target nobody can draw is a dead entry in the deck. Eight targets and 2000 seeds is
    // far more than enough to see all eight.
    let d = deck(&space(2, 3), &Preset::default());
    let seen: std::collections::BTreeSet<Target> =
        (0..2000u64).filter_map(|seed| draw(&d, seed)).collect();
    assert_eq!(seen.len(), d.targets.len());
}

#[test]
fn consecutive_seeds_do_not_walk_the_deck_in_order() {
    // The clock moves by one tick between two draws, so a mixer that maps n -> n % len would
    // deal the deck in order and never feel like a draw. This is what splitmix64 is for.
    let d = deck(&space(4, 5), &Preset::default());
    let first: Vec<Target> = (0..6u64).filter_map(|seed| draw(&d, seed)).collect();
    let in_order: Vec<Target> = d.targets.iter().take(6).copied().collect();
    assert_ne!(first, in_order);
}
