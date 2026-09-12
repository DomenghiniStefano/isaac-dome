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
