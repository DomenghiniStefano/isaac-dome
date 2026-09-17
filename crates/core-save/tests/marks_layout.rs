//! Where a completion mark lives in section 2, and where the four located tallies live.
//!
//! This is the file's shape, so it is this crate's business: a cell's index is the same
//! kind of knowledge as a section header's offset, and the screen that draws the matrix
//! should not be the only place that knows it.
//!
//! Row numbers are positions in the 34-row order — Magdalene 1, Cain 2, Keeper 12, The
//! Forgotten 14, Bethany 15, T. Jacob 33.

use core_save::marks::{cell_index, counter_index_of, Column, CounterKey};

/// The bases pinned on 2026-09-08 and re-derived independently on 2026-09-12 by walking
/// the days an achievement flipped (spec 2026-09-12, §2.3). Values, not a formula: a
/// formula would reproduce whatever mistake generated the tables.
#[test]
fn the_located_cells_are_where_the_series_put_them() {
    assert_eq!(
        cell_index(1, Column::Mother),
        Some(424),
        "Mother base 423 + Magdalene"
    );
    assert_eq!(
        cell_index(2, Column::Mother),
        Some(425),
        "Mother base 423 + Cain"
    );
    assert_eq!(
        cell_index(1, Column::TheBeast),
        Some(458),
        "The Beast base 457 + Magdalene"
    );
    assert_eq!(
        cell_index(12, Column::Greed),
        Some(142),
        "Greed base 130 + Keeper"
    );
    assert_eq!(
        cell_index(14, Column::Delirium),
        Some(213),
        "The Forgotten has single cells, not a block"
    );
    assert_eq!(
        cell_index(19, Column::Delirium),
        Some(408),
        "19-block Delirium base 404 + T. Cain (+4)"
    );
}

/// Spec §2.4: Mother and The Beast for The Forgotten and the 19 are **not located**. The
/// spacing says they sit inside 423..=490, but every candidate cell is zero in every save
/// collected, and on 2026-09-12 a walk of the whole series found no completion of either
/// boss by any of those 20 characters. The layout says "I can't tell you" rather than
/// returning a plausible index, which would show a mark nobody earned.
#[test]
fn the_unlocated_cells_answer_none() {
    assert_eq!(cell_index(14, Column::Mother), None, "The Forgotten");
    assert_eq!(cell_index(14, Column::TheBeast), None, "The Forgotten");
    assert_eq!(cell_index(15, Column::Mother), None, "Bethany");
    assert_eq!(cell_index(33, Column::TheBeast), None, "T. Jacob");
}

#[test]
fn a_row_outside_the_matrix_answers_none() {
    assert_eq!(
        cell_index(34, Column::Greed),
        None,
        "there are 34 rows, 0..=33"
    );
}

/// The four tallies whose index is located. Named on this side too: the name is the whole
/// point, because it is what a rules file is allowed to carry instead of a number.
#[test]
fn the_four_named_tallies_have_their_indices() {
    assert_eq!(counter_index_of(CounterKey::HushKills), 158);
    assert_eq!(counter_index_of(CounterKey::DeliriumKills), 187);
    assert_eq!(counter_index_of(CounterKey::MotherKills), 491);
    assert_eq!(counter_index_of(CounterKey::BeastKills), 492);
}

/// No two cells of the matrix may share an index, and no cell may land on a located
/// tally. Added 2026-09-17 with B58, and it is the guard that does **not** need a sample.
///
/// The properties in `crates/ipc/tests/marks_real.rs` compare counts, and the file says
/// so: a base off by one lights the neighbour's cell on the same day and the arithmetic
/// still works. Only the identity check separates them, and it needs a window where one
/// character won and one mark appeared — the 638-era series offers exactly one, and it is
/// an Azazel window that says nothing about Mother. Measured while closing B58: moving
/// Mother's base from 423 to **422** leaves every real-data property green on this
/// machine.
///
/// What catches it is arithmetic on the tables themselves. The blocks tile: Delirium's
/// 19-block runs 404..=422, Mother's 14-block starts at 423. A base off by one downwards
/// makes two different cells answer the same index, which is a contradiction no sample is
/// needed to see. This test costs nothing and holds on every machine, which is the point —
/// `samples/` is per-machine and this is not.
#[test]
fn no_two_cells_of_the_matrix_share_an_index() {
    use std::collections::BTreeMap;

    let mut owner: BTreeMap<usize, String> = BTreeMap::new();
    for row in 0..34 {
        for column in Column::ALL {
            let Some(index) = cell_index(row, column) else {
                continue;
            };
            let who = format!("row {row} × {column:?}");
            if let Some(other) = owner.insert(index, who.clone()) {
                panic!("index {index} is claimed by both {other} and {who}");
            }
        }
    }
    for key in [
        CounterKey::HushKills,
        CounterKey::DeliriumKills,
        CounterKey::MotherKills,
        CounterKey::BeastKills,
    ] {
        let index = counter_index_of(key);
        if let Some(other) = owner.get(&index) {
            panic!("the tally {key:?} sits at {index}, which is also {other}");
        }
    }
}

/// The blocks tile without a hole where the file says they do. Same day, same reason: this
/// is the other half of the arithmetic that pins a base without a sample.
///
/// Only the three runs the series actually established are asserted. The gaps elsewhere
/// are real and documented — 111..=115, 158..=172 (158 is Hush's kills), 385..=403 — and
/// asserting a tiling across them would be inventing a rule the file does not follow.
#[test]
fn the_three_derived_blocks_tile_against_their_neighbours() {
    // Delirium's 19-block ends where Mother's 14-block begins.
    assert_eq!(
        cell_index(33, Column::Delirium).map(|i| i + 1),
        cell_index(0, Column::Mother),
        "404..=422 then 423: a Mother base one lower would overlap Delirium's block"
    );
    // Mother's 14-block, the 20 unlocated cells, then The Beast's: 14 + 1 + 19 = 34.
    assert_eq!(
        cell_index(0, Column::Mother).map(|i| i + 34),
        cell_index(0, Column::TheBeast),
        "the spacing that located The Beast in the first place"
    );
    // The Beast's own 34 run up to its kills tally, which is the cell after it.
    assert_eq!(
        cell_index(0, Column::TheBeast).map(|i| i + 34),
        Some(counter_index_of(CounterKey::MotherKills)),
        "457..=490 then 491: Mother's kills are the cell after the last mark"
    );
    assert_eq!(
        counter_index_of(CounterKey::MotherKills) + 1,
        counter_index_of(CounterKey::BeastKills),
        "the two tallies are adjacent, which is how they were found"
    );
}
