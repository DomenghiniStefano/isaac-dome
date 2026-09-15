use floor::{Cell, Grid, RoomKind, Shape, CELLS, START};

fn one_room(cell: u16, kind: RoomKind) -> Grid {
    let mut g = Grid::empty();
    g.set(
        cell,
        Cell::Room {
            kind,
            shape: Shape::Single,
        },
    );
    g
}

#[test]
fn an_empty_grid_has_no_painted_cell() {
    let g = Grid::empty();
    assert_eq!(g.painted(), 0);
    assert_eq!(g.at(START), Cell::Empty);
}

#[test]
fn a_painted_cell_reads_back_as_what_was_painted() {
    let g = one_room(START, RoomKind::Start);
    assert_eq!(
        g.at(START),
        Cell::Room {
            kind: RoomKind::Start,
            shape: Shape::Single
        }
    );
    assert_eq!(g.painted(), 1);
}

#[test]
fn painting_outside_the_grid_changes_nothing() {
    let mut g = Grid::empty();
    g.set(
        CELLS as u16,
        Cell::Room {
            kind: RoomKind::Boss,
            shape: Shape::Single,
        },
    );
    assert_eq!(g.painted(), 0);
    assert_eq!(g.at(CELLS as u16), Cell::Empty);
}

#[test]
fn a_grid_of_the_wrong_length_is_refused_rather_than_padded() {
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS]).is_some());
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS - 1]).is_none());
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS + 1]).is_none());
}

#[test]
fn the_kinds_next_to_a_cell_are_the_painted_ones_only() {
    let mut g = one_room(83, RoomKind::Normal);
    g.set(
        85,
        Cell::Room {
            kind: RoomKind::Boss,
            shape: Shape::Single,
        },
    );
    // 71 and 97 stay empty and contribute nothing.
    let mut kinds = g.neighbour_kinds(START);
    kinds.sort_by_key(|k| format!("{k:?}"));
    assert_eq!(kinds, vec![RoomKind::Boss, RoomKind::Normal]);
}
