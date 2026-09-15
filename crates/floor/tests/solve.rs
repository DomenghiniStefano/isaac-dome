//! Every expected value here comes from a sentence in
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`, named in the test.

use floor::{distance_from_start, solve, Cell, Grid, RoomKind, Rules, Shape, Target, WIDTH};

/// `.` empty, `S` start, `n` normal, `B` boss, `T` treasure. Rows are laid out from cell 0.
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
fn the_ultra_secret_rule_is_reported_as_unresolved_rather_than_answered() {
    // rule ultra-secret-connections is Unmodelled: red rooms are not on this grid.
    let s = solve(&Grid::empty(), rules(), Target::UltraSecret);
    assert!(
        s.unresolved
            .iter()
            .any(|u| u.rule == "ultra-secret-connections"),
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
