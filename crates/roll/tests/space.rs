//! The space's shape and the one judgment it makes: what a target's status is.
//!
//! The expected values come from `docs/superpowers/specs/2026-09-17-roll-design.md` §1, whose
//! status table comes from `docs/save-format.md`: a cell's bits **replace** one another
//! rather than accumulating, so any non-zero value is taken and there is no "cleared bit".

use roll::{deck, CellValue, Preset, Space, SpaceError, Status, Target};

/// A 2x3 space whose Greed column is 1. Small on purpose: every cell can be named in a
/// comment, which a 34x12 one cannot.
fn small(cells: Vec<CellValue>, playable: Vec<bool>) -> Space {
    Space::new(2, 3, 1, cells, playable).expect("the shape is the one declared")
}

fn known(bits: u8) -> CellValue {
    CellValue::Known { bits }
}

#[test]
fn a_space_holds_one_target_per_cell_plus_one_greedier_per_row() {
    let space = small(vec![known(0); 6], vec![true, true]);
    assert_eq!(space.target_count(), 8);
}

#[test]
fn a_cell_at_zero_is_missing() {
    let space = small(vec![known(0); 6], vec![true, true]);
    assert_eq!(
        space.status(&Target::Mark {
            character: 0,
            column: 0
        }),
        Some(Status::Missing)
    );
}

#[test]
fn any_non_zero_cell_is_taken() {
    // 1 -> 2 is measured across the 638-era series: a bare 2 is a cell that was taken, not one
    // that lost bit 0. Every non-zero value reads the same way.
    for bits in 1u8..=7 {
        let mut cells = vec![known(0); 6];
        cells[0] = known(bits);
        let space = small(cells, vec![true, true]);
        assert_eq!(
            space.status(&Target::Mark {
                character: 0,
                column: 0
            }),
            Some(Status::Taken),
            "bits = {bits}"
        );
    }
}

#[test]
fn an_unreadable_cell_is_unreadable_and_never_missing() {
    let mut cells = vec![known(0); 6];
    cells[0] = CellValue::Unreadable;
    let space = small(cells, vec![true, true]);
    assert_eq!(
        space.status(&Target::Mark {
            character: 0,
            column: 0
        }),
        Some(Status::Unreadable)
    );
}

#[test]
fn greedier_reads_bit_one_of_the_greed_column_and_nothing_else() {
    // Row 0's greed cell is 1: the mark is taken, the second level is not.
    // Row 1's greed cell is 2: the second level is taken.
    let cells = vec![known(0), known(1), known(0), known(0), known(2), known(0)];
    let space = small(cells, vec![true, true]);
    assert_eq!(
        space.status(&Target::Greedier { character: 0 }),
        Some(Status::Missing)
    );
    assert_eq!(
        space.status(&Target::Greedier { character: 1 }),
        Some(Status::Taken)
    );
}

#[test]
fn bit_two_alone_is_not_the_second_level() {
    // Bit 2 is "won online" (measured on a matched window), not a third level: it must not
    // read as Greedier's bit. The mark itself is still taken — the cell is non-zero.
    let mut cells = vec![known(0); 6];
    cells[1] = known(4);
    let space = small(cells, vec![true, true]);
    assert_eq!(
        space.status(&Target::Greedier { character: 0 }),
        Some(Status::Missing)
    );
    assert_eq!(
        space.status(&Target::Mark {
            character: 0,
            column: 1
        }),
        Some(Status::Taken)
    );
}

#[test]
fn a_greedier_on_an_unreadable_greed_cell_is_unreadable() {
    let mut cells = vec![known(0); 6];
    cells[1] = CellValue::Unreadable;
    let space = small(cells, vec![true, true]);
    assert_eq!(
        space.status(&Target::Greedier { character: 0 }),
        Some(Status::Unreadable)
    );
}

#[test]
fn a_target_outside_the_space_has_no_status() {
    let space = small(vec![known(0); 6], vec![true, true]);
    assert_eq!(
        space.status(&Target::Mark {
            character: 9,
            column: 0
        }),
        None
    );
    assert_eq!(
        space.status(&Target::Mark {
            character: 0,
            column: 9
        }),
        None
    );
    assert_eq!(space.status(&Target::Greedier { character: 9 }), None);
}

#[test]
fn a_short_cell_vector_is_refused_rather_than_silently_shortening_the_deck() {
    assert_eq!(
        Space::new(2, 3, 1, vec![known(0); 5], vec![true, true]),
        Err(SpaceError::CellCount {
            expected: 6,
            found: 5
        })
    );
}

#[test]
fn a_playable_vector_that_is_not_one_per_row_is_refused() {
    assert_eq!(
        Space::new(2, 3, 1, vec![known(0); 6], vec![true]),
        Err(SpaceError::PlayableCount {
            expected: 2,
            found: 1
        })
    );
}

#[test]
fn a_greed_column_outside_the_space_is_refused() {
    assert_eq!(
        Space::new(2, 3, 3, vec![known(0); 6], vec![true, true]),
        Err(SpaceError::GreedColumn {
            columns: 3,
            found: 3
        })
    );
}

#[test]
fn empty_holds_nothing_to_draw() {
    // The one shape `Space::new` can never reject (0 rows, 1 column, nothing to draw), built
    // directly rather than through a `Result` nobody can fail to unwrap. It stands in for the
    // fallback path `ipc::unreadable_space` is documented to never reach.
    let space = Space::empty();
    assert_eq!(space.target_count(), 0);

    let d = deck(&space, &Preset::default());
    assert!(d.targets.is_empty());
    assert_eq!(d.excluded.taken, 0);
    assert_eq!(d.excluded.unreadable, 0);
    assert_eq!(d.excluded.locked, 0);
    assert_eq!(d.excluded.filtered, 0);
}

/// A target names its row and column in a `u8`, so a space past 256 of either cannot be
/// addressed: row 256 would be drawn as row 0 (card #80, P11c). Refused, not wrapped.
#[test]
fn a_space_wider_than_a_target_can_name_is_refused() {
    assert_eq!(
        Space::new(257, 1, 0, vec![known(0); 257], vec![true; 257]),
        Err(SpaceError::TooLarge {
            rows: 257,
            columns: 1
        })
    );
    assert_eq!(
        Space::new(1, 257, 0, vec![known(0); 257], vec![true]),
        Err(SpaceError::TooLarge {
            rows: 1,
            columns: 257
        })
    );
    assert!(Space::new(256, 1, 0, vec![known(0); 256], vec![true; 256]).is_ok());
}
