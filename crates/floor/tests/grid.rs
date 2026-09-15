//! The grid, before any rule. A flat array of 169 cells makes two mistakes silently — the row
//! that wraps and the edge that does not — so both are properties here, not review comments.

use floor::{neighbours, CELLS, HEIGHT, START, WIDTH};

fn sorted(cell: u16) -> Vec<u16> {
    let mut v = neighbours(cell);
    v.sort_unstable();
    v
}

#[test]
fn the_grid_is_thirteen_by_thirteen_and_the_start_room_is_its_centre() {
    assert_eq!(WIDTH, 13);
    assert_eq!(HEIGHT, 13);
    assert_eq!(CELLS, 169);
    // 84 is what the game prints (CURRENT ROOM INDEX 84, 42 times across the five real logs
    // in samples/logs/ and never another value). That it is the centre is 6 * 13 + 6.
    assert_eq!(START, 84);
    assert_eq!(START, 6 * WIDTH + 6);
}

#[test]
fn a_cell_in_the_middle_has_four_neighbours() {
    assert_eq!(sorted(START), vec![71, 83, 85, 97]);
}

#[test]
fn a_cell_on_the_left_edge_has_no_neighbour_on_the_row_above() {
    // index 13 is (x = 0, y = 1). 12 is (x = 12, y = 0): the wrap this test exists for.
    assert_eq!(sorted(13), vec![0, 14, 26]);
}

#[test]
fn a_cell_on_the_right_edge_has_no_neighbour_on_the_row_below() {
    // index 25 is (x = 12, y = 1). 26 is (x = 0, y = 2).
    assert_eq!(sorted(25), vec![12, 24, 38]);
}

#[test]
fn the_four_corners_have_two_neighbours_each() {
    assert_eq!(sorted(0), vec![1, 13]);
    assert_eq!(sorted(12), vec![11, 25]);
    assert_eq!(sorted(156), vec![143, 157]);
    assert_eq!(sorted(168), vec![155, 167]);
}

#[test]
fn every_neighbour_is_one_step_away_and_never_across_a_row() {
    for cell in 0..CELLS as u16 {
        for n in neighbours(cell) {
            let (x, y) = (cell % WIDTH, cell / WIDTH);
            let (nx, ny) = (n % WIDTH, n / WIDTH);
            let dx = x.abs_diff(nx);
            let dy = y.abs_diff(ny);
            assert_eq!(dx + dy, 1, "{cell} and {n} are not adjacent");
        }
    }
}

#[test]
fn a_cell_outside_the_grid_has_no_neighbours() {
    assert!(neighbours(CELLS as u16).is_empty());
    assert!(neighbours(u16::MAX).is_empty());
}
