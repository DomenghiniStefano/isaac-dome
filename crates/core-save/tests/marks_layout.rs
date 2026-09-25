//! Where a completion mark lives in section 2, and where the four located tallies live.
//!
//! This is the file's shape, so it is this crate's business: a cell's index is the same
//! kind of knowledge as a section header's offset, and the screen that draws the matrix
//! should not be the only place that knows it.
//!
//! Row numbers are positions in the 34-row order — Magdalene 1, Cain 2, Keeper 12, The
//! Forgotten 14, Bethany 15, T. Jacob 33.

use core_save::{cell_index, counter_index_of, Column, CounterKey, ROWS};

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

/// Measured on 2026-09-20, and it is the window B20 had been asking for since 2026-09-12:
/// Mother beaten with T. Eden, on `20260919-pre-tainted-mother` →
/// `20260920-post-tainted-eden-mother`. Three facts agree, the way 423 and 457 were pinned:
/// achievement `[567]`, whose requirement in `graph`'s rules is *Mother* + *Tainted Eden*;
/// the Mother kills tally `[491] 6→7`, up by exactly the one new mark; and index 188 at
/// `1 << 30`, naming character 30, which is what `requirements.json` calls T. Eden.
///
/// `[449]` is the **only** cell that moved in all of 423..=490. T. Eden is row 26, so
/// `449 = base + (26 - 15)` puts the 19-block at **438**, and the one cell left over in
/// 437..=456 is The Forgotten's, at **437**. Values, not a formula, like the bases above.
#[test]
fn mothers_cells_for_the_forgotten_and_the_nineteen_are_where_the_window_put_them() {
    assert_eq!(
        cell_index(26, Column::Mother),
        Some(449),
        "the cell that moved: T. Eden"
    );
    assert_eq!(
        cell_index(15, Column::Mother),
        Some(438),
        "the 19-block's base: Bethany"
    );
    assert_eq!(
        cell_index(33, Column::Mother),
        Some(456),
        "T. Jacob & Esau closes the block"
    );
    assert_eq!(
        cell_index(14, Column::Mother),
        Some(437),
        "The Forgotten: the cell left over, by difference"
    );
}

/// Mother's group of 34 now tiles end to end, and this is the half of the arithmetic that
/// needs no sample. 423..=436 are the 14 originals, 437 is The Forgotten, 438..=456 are the
/// 19, and 457 is where The Beast's own 14-block starts. A base off by one anywhere in
/// there either opens a hole or lands two rows on one cell; the first is what this test
/// sees, the second is `no_two_cells_of_the_matrix_share_an_index`.
#[test]
fn mothers_group_of_34_tiles_from_its_own_base_to_the_beasts() {
    assert_eq!(
        cell_index(13, Column::Mother).map(|i| i + 1),
        cell_index(14, Column::Mother),
        "the 14 originals end, The Forgotten's single cell begins"
    );
    assert_eq!(
        cell_index(14, Column::Mother).map(|i| i + 1),
        cell_index(15, Column::Mother),
        "The Forgotten's one cell, then the 19-block"
    );
    assert_eq!(
        cell_index(33, Column::Mother).map(|i| i + 1),
        cell_index(0, Column::TheBeast),
        "438..=456 then 457: the 19-block ends where The Beast's block was measured"
    );
}

/// Spec §2.4, now **half true**: The Beast for The Forgotten and the 19 is still not
/// located. The spacing says those cells sit inside 471..=490, and the layout of Mother's
/// own group of 34 — measured on 2026-09-20 — says which of them is which. That is an
/// inference from one worked example, not a window on this half, so it stays `None`: a
/// derived index and a measured one must not be told apart only by reading the git log.
///
/// Mother's half left this test on 2026-09-20. What closes this one is a run of The Beast
/// with any of those 20 characters.
#[test]
fn the_unlocated_cells_answer_none() {
    assert_eq!(cell_index(14, Column::TheBeast), None, "The Forgotten");
    assert_eq!(cell_index(15, Column::TheBeast), None, "Bethany");
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

// The four below were pinned in `ipc`'s tests, through the screen's `(row, column index)`
// translation, until card #82 (D7) moved them here: they are the layout's indices, and this is
// the crate that holds the layout.

#[test]
fn original_characters_use_the_verified_blocks() {
    // Mom's Heart starts at 27; Isaac is the first of the 14.
    assert_eq!(cell_index(0, Column::MomsHeart), Some(27));
    // Apollyon is the fourteenth: 27 + 13.
    assert_eq!(cell_index(13, Column::MomsHeart), Some(40));
    // Delirium for the 14 originals starts at 173.
    assert_eq!(cell_index(0, Column::Delirium), Some(173));
    // Mother and The Beast, located on 2026-09-08 on the historical series. The base of
    // each block is pinned by two characters read off the winner mask at index 188:
    // Magdalene (+1) and Cain (+2) on the days their mark appeared.
    assert_eq!(cell_index(0, Column::Mother), Some(423)); // Isaac × Mother
    assert_eq!(cell_index(1, Column::Mother), Some(424)); // Magdalene
    assert_eq!(cell_index(2, Column::Mother), Some(425)); // Cain
    assert_eq!(cell_index(13, Column::Mother), Some(436)); // Apollyon, last of the 14
    assert_eq!(cell_index(0, Column::TheBeast), Some(457)); // Isaac × The Beast
    assert_eq!(cell_index(1, Column::TheBeast), Some(458)); // Magdalene
    assert_eq!(cell_index(2, Column::TheBeast), Some(459)); // Cain
    assert_eq!(cell_index(13, Column::TheBeast), Some(470)); // Apollyon
}

#[test]
fn the_forgotten_uses_single_cells() {
    assert_eq!(cell_index(14, Column::MomsHeart), Some(203));
    assert_eq!(cell_index(14, Column::Hush), Some(211));
    // Delirium: 212 belongs to another family.
    assert_eq!(cell_index(14, Column::Delirium), Some(213));
    // Mother closed on 2026-09-20: T. Eden's cell moved at 449, which puts the 19-block at
    // 438 and leaves 437 — the one cell over — to The Forgotten.
    assert_eq!(cell_index(14, Column::Mother), Some(437));
    // The Beast is still derived from the spacing and never observed moving, so it stays
    // unlocated rather than pointing at a guess.
    assert_eq!(cell_index(14, Column::TheBeast), None);
}

#[test]
fn later_characters_now_reach_delirium() {
    assert_eq!(cell_index(15, Column::MomsHeart), Some(214)); // Bethany
    assert_eq!(cell_index(33, Column::Hush), Some(384)); // T. Jacob & Esau = 366 + 18
                                                         // The column that used to be the hole. Four characters pin the base at 404:
                                                         // Bethany (+0), Jacob & Esau (+1), T. Cain (+4) and T. Azazel (+9), each on the day
                                                         // its cell appeared together with a Delirium kill.
    assert_eq!(cell_index(15, Column::Delirium), Some(404)); // Bethany
    assert_eq!(cell_index(16, Column::Delirium), Some(405)); // Jacob & Esau
    assert_eq!(cell_index(19, Column::Delirium), Some(408)); // T. Cain
    assert_eq!(cell_index(24, Column::Delirium), Some(413)); // T. Azazel
    assert_eq!(cell_index(33, Column::Delirium), Some(422)); // T. Jacob & Esau, last of the 19
}

#[test]
fn exactly_twenty_cells_are_unlocated() {
    let unlocated = (0..ROWS)
        .flat_map(|row| Column::ALL.map(|column| (row, column)))
        .filter(|&(row, column)| cell_index(row, column).is_none())
        .count();
    assert_eq!(
        unlocated, 20,
        "The Forgotten and the 19 later characters, for The Beast alone: 20 cells whose \
         position is derived from the spacing and confirmed by nothing. It was 40 until \
         2026-09-20, when a window on T. Eden beating Mother closed that half; what closes \
         this one is the same run against The Beast"
    );
}
