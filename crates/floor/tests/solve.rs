//! Every expected value here comes from a sentence in
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`, named in the test.

use floor::{distance_from_start, solve, Cell, Grid, RoomKind, Rules, Shape, Target, WIDTH};

/// `.` empty, `S` start, `n` normal, `B` boss, `T` treasure, `C` curse, `x` secret,
/// `X` super secret. Rows are laid out from cell 0.
fn parse_grid(rows: &[&str]) -> Grid {
    let mut g = Grid::empty();
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let kind = match c {
                '.' => continue,
                'S' => RoomKind::Start,
                'n' => RoomKind::Normal,
                'B' => RoomKind::Boss,
                'T' => RoomKind::Treasure,
                'C' => RoomKind::Curse,
                'x' => RoomKind::Secret,
                'X' => RoomKind::SuperSecret,
                other => panic!("unknown cell {other}"),
            };
            let cell = (y as u16) * WIDTH + x as u16;
            g.set(
                cell,
                Cell::Room {
                    kind,
                    shape: Shape::Single,
                },
            );
        }
    }
    g
}

fn rules() -> &'static Rules {
    Rules::embedded().expect("the embedded rules parse")
}

fn cells(s: &floor::Solution) -> Vec<u16> {
    s.candidates.iter().map(|c| c.cell).collect()
}

#[test]
fn a_cell_with_four_neighbours_outranks_one_with_two() {
    // rule secret-neighbours (rank 0, "3 neighbors … as it is with 4") against
    // secret-neighbours-two (rank 1, "2 neighbor locations are rare but possible").
    // Cell 14 is the hole in the middle of the ring: it touches 1, 13, 15 and 27.
    let g = parse_grid(&[".n.", "n.n", ".n."]);
    let s = solve(&g, rules(), Target::Secret);
    let first = s.candidates.first().expect("at least one candidate");
    assert_eq!(first.cell, 14);
    assert_eq!(first.neighbours, 4);
    assert_eq!(first.rank, 0);
    assert!(
        s.candidates.windows(2).all(|w| w[0].rank <= w[1].rank),
        "candidates come out in rank order"
    );
    assert!(
        s.candidates
            .iter()
            .any(|c| c.rank == 1 && c.neighbours == 2),
        "a two-neighbour cell is still a candidate, ranked below"
    );
}

#[test]
fn a_cell_touching_the_boss_room_is_not_a_secret_room_candidate() {
    // rule secret-forbidden-neighbours: "except Boss Rooms, Super Secret Rooms, and other
    // Secret Rooms". Cell 14 touches three painted rooms and would rank first without it.
    let g = parse_grid(&["nnn", "n.B"]);
    let s = solve(&g, rules(), Target::Secret);
    assert!(!cells(&s).contains(&14), "cell 14 touches the boss room");

    // The same floor with a normal room where the boss was: the instrument has to be shown
    // able to speak before its silence counts as evidence.
    let ok = parse_grid(&["nnn", "n.n"]);
    let s = solve(&ok, rules(), Target::Secret);
    assert_eq!(s.candidates.first().map(|c| c.cell), Some(14));
    assert_eq!(s.candidates.first().map(|c| c.neighbours), Some(3));
}

#[test]
fn a_candidate_names_the_rules_that_elected_it() {
    let g = parse_grid(&[".n.", "nnn", ".n."]);
    let s = solve(&g, rules(), Target::Secret);
    let c = s.candidates.first().expect("at least one candidate");
    assert!(
        c.applied
            .iter()
            .any(|id| id == "secret-neighbours" || id == "secret-neighbours-two"),
        "a lit cell says which rule lit it"
    );
}

#[test]
fn the_super_secret_room_is_a_dead_end_and_not_a_crossroads() {
    // rule super-secret-dead-end: "only located next to one other room".
    let g = parse_grid(&["Snn", "..."]);
    let s = solve(&g, rules(), Target::SuperSecret);
    for c in &s.candidates {
        assert_eq!(c.neighbours, 1, "cell {} is not a dead end", c.cell);
    }
}

#[test]
fn a_dead_end_hanging_off_a_special_room_is_not_a_super_secret_candidate() {
    // rule super-secret-neighbour-not-special: "this room can't be a Special Room".
    let g = parse_grid(&["SnT"]);
    let s = solve(&g, rules(), Target::SuperSecret);
    // cell 3 (row 0, x = 3) touches only the treasure room at cell 2.
    assert!(
        !cells(&s).contains(&3),
        "its only neighbour is a Special Room"
    );
}

#[test]
fn the_ultra_secret_rule_about_room_shapes_is_reported_as_unresolved_rather_than_answered() {
    // rule ultra-secret-shapes is Unmodelled: a side that cannot open depends on the shape of
    // the room behind it, and every room on this grid is one square. The rest of the Ultra
    // Secret paragraph *is* evaluable — see the block at the end of this file.
    let s = solve(&Grid::empty(), rules(), Target::UltraSecret);
    assert!(
        s.unresolved.iter().any(|u| u.rule == "ultra-secret-shapes"),
        "an unmodelled rule says so"
    );
    assert!(
        !s.unresolved.is_empty() && s.candidates.is_empty(),
        "nothing painted, nothing claimed"
    );
}

#[test]
fn an_empty_grid_produces_no_candidate_for_any_target() {
    for target in [Target::Secret, Target::SuperSecret, Target::UltraSecret] {
        assert!(solve(&Grid::empty(), rules(), target).candidates.is_empty());
    }
}

#[test]
fn distance_from_start_counts_rooms_walked_and_stops_at_the_paint() {
    let g = parse_grid(&["Snn", "..n"]);
    let d = distance_from_start(&g);
    assert_eq!(d[0], Some(0), "the start room is zero rooms away");
    assert_eq!(d[1], Some(1));
    assert_eq!(d[2], Some(2));
    assert_eq!(d[15], Some(3), "cell 15 is reached through cell 2");
    assert_eq!(d[13], None, "an empty cell has no distance");
}

#[test]
fn without_a_start_room_the_distance_rule_is_unresolved_and_the_dead_ends_still_show() {
    // The grid has no Start: super-secret-second-longest cannot be evaluated, which is a
    // sentence the screen has to say — not a reason to light nothing.
    let g = parse_grid(&["nnn"]);
    let s = solve(&g, rules(), Target::SuperSecret);
    assert!(s
        .unresolved
        .iter()
        .any(|u| u.rule == "super-secret-second-longest"));
    assert!(!s.candidates.is_empty(), "the dead-end rule still answers");
}

// --- secret-neighbours-one, the fallback the research pass added -------------------------
//
// "1 neighbor locations can only happen if there are no valid 3+ neighbor locations, and are
// very rare." The three tests below are the three halves of that sentence: it is switched off
// by a 3+ location, it fires without one, and the word that decides between them is **valid**.

#[test]
fn a_one_neighbour_cell_is_no_candidate_while_a_three_plus_cell_stands() {
    // Cell 14 touches four rooms, so the fallback is switched off everywhere on this floor.
    let g = parse_grid(&[".n.", "n.n", ".n."]);
    let s = solve(&g, rules(), Target::Secret);
    assert!(
        s.candidates.iter().all(|c| c.neighbours >= 2),
        "a 1-neighbour cell cannot be a candidate while a 3+ location exists"
    );
    assert!(
        s.candidates.iter().any(|c| c.neighbours == 4),
        "and the 3+ location that switched it off is itself a candidate"
    );
}

#[test]
fn a_one_neighbour_cell_is_a_candidate_when_the_floor_has_no_three_plus_location() {
    // Two rooms side by side: nothing on this grid touches more than one of them.
    let g = parse_grid(&["nn"]);
    let s = solve(&g, rules(), Target::Secret);
    assert!(
        s.candidates.iter().all(|c| c.neighbours == 1),
        "no cell here touches two rooms"
    );
    assert!(
        s.candidates.iter().any(|c| c.rank == 2),
        "the fallback fires, and says so with the rank its rule carries"
    );
    assert!(
        s.candidates
            .iter()
            .all(|c| c.applied.iter().any(|id| id == "secret-neighbours-one")),
        "and every lit cell names the rule that lit it"
    );
}

#[test]
fn a_three_plus_cell_the_boss_room_rules_out_does_not_switch_the_fallback_off() {
    // The sentence says "no **valid** 3+ neighbor locations". Cell 14 touches four rooms and
    // one of them is the Boss Room, so secret-forbidden-neighbours rejects it and it was never
    // a valid location. This is the test that pins *when* the fallback is resolved: judging it
    // before the narrowing runs would read cell 14 as a 3+ location and go quiet.
    let g = parse_grid(&[".B.", "n.n", ".n."]);
    let s = solve(&g, rules(), Target::Secret);
    assert!(!cells(&s).contains(&14), "cell 14 touches the boss room");
    assert!(
        s.candidates.iter().any(|c| c.neighbours == 1),
        "with no valid 3+ location left, the 1-neighbour cells are candidates"
    );
}

// --- the Ultra Secret Room, which used to answer nothing at all (B68) --------------------
//
// Every grid below is the same shape: three rooms, and one empty cell in the middle of them
// that touches none. `.` is both an empty cell and a side where a red room could open, which
// is the whole reason this target is evaluable — a red room is not a room somebody painted.
//
//   y0  . . n . .
//   y1  . . . . .
//   y2  n . * . n     * is cell 28, and it is empty
//   y3  . . . . .
//
// Cell 28 reaches 2, 26 and 30, each through one red room; 14 reaches 2 and 26; 42 reaches
// only 30.

const REACH: &[&str] = &["..n", ".....", "n...n"];

fn ultra(g: &Grid) -> floor::Solution {
    solve(g, rules(), Target::UltraSecret)
}

fn candidate(s: &floor::Solution, cell: u16) -> Option<&floor::Candidate> {
    s.candidates.iter().find(|c| c.cell == cell)
}

#[test]
fn a_spot_reaching_three_rooms_through_its_red_rooms_is_the_first_place() {
    // rule ultra-secret-connections: "most likely generated in spots that connect to 3+
    // non-red rooms through its adjacent red rooms".
    let s = ultra(&parse_grid(REACH));
    let first = s.candidates.first().expect("at least one candidate");
    assert_eq!(first.cell, 28);
    assert_eq!(first.rank, 0);
    assert_eq!(
        s.candidates.iter().filter(|c| c.rank == 0).count(),
        1,
        "no other cell on this floor reaches three rooms"
    );
}

#[test]
fn two_rooms_reached_rank_below_three_and_one_room_below_that() {
    // rules ultra-secret-connections-two and -one: "a specific 3+ room location is 11.5x more
    // likely than a specific 2 room location", and 2 in turn than 1.
    let s = ultra(&parse_grid(REACH));
    assert_eq!(
        candidate(&s, 14).map(|c| c.rank),
        Some(1),
        "reaches 2 and 26"
    );
    assert_eq!(
        candidate(&s, 42).map(|c| c.rank),
        Some(2),
        "reaches only 30"
    );
}

#[test]
fn a_one_room_spot_still_stands_while_a_three_room_spot_does() {
    // The sentence is "1 room locations being virtually impossible if there is a 3+ location
    // available" — *virtually*, and the page's own hidden comment says why: "sometimes the
    // last dead end created cannot connect to the Ultra Secret Room, which can create a
    // scenario where a 1 room location is chosen when a 3+ location is available". So this is
    // **not** the fallback its Secret Room sibling is, where the wiki says "can only happen
    // if". It ranks last; it is not switched off.
    let s = ultra(&parse_grid(REACH));
    assert!(candidate(&s, 28).is_some_and(|c| c.rank == 0));
    assert!(
        candidate(&s, 42).is_some(),
        "a 1-room spot is not dropped by the existence of a 3-room one"
    );
}

#[test]
fn a_spot_against_the_edge_of_the_grid_is_a_candidate() {
    // "locations on the 13x13 border where a red room would normally open to an I AM ERROR
    // room are allowed". Cell 0 is the corner: two of its four sides are off the grid, and
    // that is not two sides that failed.
    let s = ultra(&parse_grid(REACH));
    assert_eq!(
        candidate(&s, 0).map(|c| c.rank),
        Some(1),
        "reaches 2 and 26"
    );
}

#[test]
fn a_cell_touching_a_painted_room_is_never_an_ultra_candidate() {
    // rule ultra-secret-not-connected: "not connected to any other room on the map directly".
    //
    //   y0  n . n
    //   y1  . . n
    //
    // Cell 1 reaches room 15 through the red room at 14, so the count rule proposes it — and
    // it touches rooms 0 and 2, so it must not survive. Cell 27 reaches the same single room,
    // touches nothing, and does.
    let s = ultra(&parse_grid(&["n.n", "..n"]));
    assert!(
        candidate(&s, 1).is_none(),
        "cell 1 has the Normal Rooms at 0 and 2 against it"
    );
    assert_eq!(
        candidate(&s, 27).map(|c| c.rank),
        Some(2),
        "and a cell that touches nothing, reaching the same one room, stands"
    );
}

#[test]
fn a_room_beside_one_of_the_red_rooms_rules_the_whole_location_out() {
    // rule ultra-secret-red-room-invalid: "can't be connected to red rooms that connect to
    // Secret Rooms, Super Secret Rooms, or Curse Rooms, and can't be in a location where any
    // of its adjacent red rooms are invalid, such as next to a Boss Room".
    //
    // Cell 2 is two cells north of 28, so it is a room beside 28's northern red room. As a
    // Normal Room it is the third thing 28 reaches; as any of these four it takes 28 off the
    // map entirely — not down a rank, and not merely one side fewer.
    for kind in ['B', 'C', 'x', 'X'] {
        let top: String = format!("..{kind}");
        let s = ultra(&parse_grid(&[&top, ".....", "n...n"]));
        assert!(
            candidate(&s, 28).is_none(),
            "a {kind} beside the red room at 15 invalidates the location"
        );
    }
}

#[test]
fn an_ultra_candidate_names_every_rule_behind_it() {
    // The quotations are the attribution: a lit cell that cannot cite its sentence is a claim
    // this screen does not get to make.
    let s = ultra(&parse_grid(REACH));
    let c = candidate(&s, 28).expect("the three-room spot");
    assert_eq!(
        c.applied,
        vec![
            "ultra-secret-connections",
            "ultra-secret-not-connected",
            "ultra-secret-red-room-invalid",
        ]
    );
}
