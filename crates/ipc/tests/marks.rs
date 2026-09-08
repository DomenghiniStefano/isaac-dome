use ipc::{counter_index, marks_matrix, Cell, CharacterGroup, BOSSES, CHARACTERS};

#[test]
fn tables_have_the_expected_shape() {
    assert_eq!(
        BOSSES.len(),
        12,
        "the widget the game draws has twelve columns"
    );
    assert_eq!(CHARACTERS.len(), 34);
    assert_eq!(BOSSES[0], "Mom's Heart");
    assert_eq!(BOSSES[9], "Delirium");
    assert_eq!(BOSSES[10], "Mother");
    assert_eq!(BOSSES[11], "The Beast");
    assert_eq!(CHARACTERS[0], ("Isaac", CharacterGroup::Original));
    assert_eq!(CHARACTERS[14], ("The Forgotten", CharacterGroup::Forgotten));
    assert_eq!(CHARACTERS[15], ("Bethany", CharacterGroup::Later));
    assert_eq!(CHARACTERS[33], ("T. Jacob & Esau", CharacterGroup::Later));
}

#[test]
fn original_characters_use_the_verified_blocks() {
    // Mom's Heart starts at 27; Isaac is the first of the 14.
    assert_eq!(counter_index(0, 0), Some(27));
    // Apollyon is the fourteenth: 27 + 13.
    assert_eq!(counter_index(13, 0), Some(40));
    // Delirium for the 14 originals starts at 173.
    assert_eq!(counter_index(0, 9), Some(173));
    // Mother and The Beast, located on 2026-09-08 on the historical series. The base of
    // each block is pinned by two characters read off the winner mask at index 188:
    // Magdalene (+1) and Cain (+2) on the days their mark appeared.
    assert_eq!(counter_index(0, 10), Some(423)); // Isaac × Mother
    assert_eq!(counter_index(1, 10), Some(424)); // Magdalene
    assert_eq!(counter_index(2, 10), Some(425)); // Cain
    assert_eq!(counter_index(13, 10), Some(436)); // Apollyon, last of the 14
    assert_eq!(counter_index(0, 11), Some(457)); // Isaac × The Beast
    assert_eq!(counter_index(1, 11), Some(458)); // Magdalene
    assert_eq!(counter_index(2, 11), Some(459)); // Cain
    assert_eq!(counter_index(13, 11), Some(470)); // Apollyon
}

#[test]
fn the_forgotten_uses_single_cells() {
    assert_eq!(counter_index(14, 0), Some(203)); // Mom's Heart
    assert_eq!(counter_index(14, 8), Some(211)); // Hush
    assert_eq!(counter_index(14, 9), Some(213)); // Delirium: 212 belongs to another family
                                                 // Mother and The Beast for The Forgotten: derived from the spacing, never observed
                                                 // moving, so they stay unlocated rather than pointing at a guess.
    assert_eq!(counter_index(14, 10), None);
    assert_eq!(counter_index(14, 11), None);
}

#[test]
fn later_characters_now_reach_delirium() {
    assert_eq!(counter_index(15, 0), Some(214)); // Bethany, Mom's Heart
    assert_eq!(counter_index(33, 8), Some(384)); // T. Jacob & Esau, Hush = 366 + 18
                                                 // The column that used to be the hole. Four characters pin the base at 404:
                                                 // Bethany (+0), Jacob & Esau (+1), T. Cain (+4) and T. Azazel (+9), each on the day
                                                 // its cell appeared together with a Delirium kill.
    assert_eq!(counter_index(15, 9), Some(404)); // Bethany
    assert_eq!(counter_index(16, 9), Some(405)); // Jacob & Esau
    assert_eq!(counter_index(19, 9), Some(408)); // T. Cain
    assert_eq!(counter_index(24, 9), Some(413)); // T. Azazel
    assert_eq!(counter_index(33, 9), Some(422)); // T. Jacob & Esau, last of the 19
}

#[test]
fn exactly_forty_cells_are_unlocated() {
    let unlocated = (0..CHARACTERS.len())
        .flat_map(|c| (0..BOSSES.len()).map(move |b| (c, b)))
        .filter(|&(c, b)| counter_index(c, b).is_none())
        .count();
    assert_eq!(
        unlocated, 40,
        "The Forgotten and the 19 later characters, for Mother and for The Beast: \
         40 cells whose position is derived from the spacing and confirmed by nothing, \
         because they are zero in every save we have"
    );
}

/// A fake counters section, as long as a real save, all zero
/// except at the indices given.
fn counters(len: usize, set: &[(usize, u32)]) -> Vec<u32> {
    (0..len)
        .map(|i| set.iter().find(|&&(j, _)| j == i).map_or(0, |&(_, v)| v))
        .collect()
}

#[test]
fn matrix_has_the_expected_shape_and_totals() {
    let m = marks_matrix(&counters(523, &[]));
    assert_eq!(m.characters.len(), 34);
    assert_eq!(m.bosses.len(), 12);
    assert_eq!(m.totals.cells, 408);
    assert_eq!(m.totals.unknown, 40);
    assert_eq!(m.totals.readable, 368);
    assert_eq!(m.totals.unexpected, 0);
    assert_eq!(m.totals.started, 0);
}

#[test]
fn the_hole_is_now_mother_and_the_beast_for_the_last_twenty_rows() {
    let m = marks_matrix(&counters(523, &[]));
    // row 15 = Bethany, column 9 = Delirium: located since 2026-09-08.
    assert_eq!(m.characters[15].cells[9], Cell::Known { bits: 0 });
    // Columns 10 and 11 for the same row, and for The Forgotten: still unlocated.
    assert_eq!(m.characters[15].cells[10], Cell::Unknown);
    assert_eq!(m.characters[15].cells[11], Cell::Unknown);
    assert_eq!(m.characters[14].cells[10], Cell::Unknown);
    // Row 0 = Isaac: the original characters have all twelve columns.
    assert_eq!(m.characters[0].cells[10], Cell::Known { bits: 0 });
    assert_eq!(m.characters[0].cells[11], Cell::Known { bits: 0 });
}

#[test]
fn a_read_value_becomes_a_bit_mask() {
    let m = marks_matrix(&counters(523, &[(27, 3), (41, 7)]));
    assert_eq!(m.characters[0].cells[0], Cell::Known { bits: 3 });
    assert_eq!(m.characters[0].cells[1], Cell::Known { bits: 7 });
    assert_eq!(m.totals.started, 2, "only readable, non-zero cells count");
}

#[test]
fn a_value_outside_the_mask_range_is_flagged_not_truncated() {
    let m = marks_matrix(&counters(523, &[(27, 49)]));
    assert_eq!(
        m.characters[0].cells[0],
        Cell::Unexpected { value: 49 },
        "49 isn't a mask: the table would be wrong, it must not be truncated to 1"
    );
    assert_eq!(m.totals.unexpected, 1);
    assert_eq!(
        m.totals.started, 0,
        "a suspicious cell doesn't count as a started mark"
    );
}

#[test]
fn a_shorter_section_yields_unknown_not_a_panic() {
    // 300 counters: the 19-blocks (214..384) mostly fall outside.
    let m = marks_matrix(&counters(300, &[]));
    assert_eq!(
        m.characters[33].cells[8],
        Cell::Unknown,
        "index 384 is past the end of the file"
    );
    assert_eq!(
        m.characters[0].cells[0],
        Cell::Known { bits: 0 },
        "index 27 is still there"
    );
    assert!(m.totals.unknown > 40);
    assert_eq!(m.totals.readable + m.totals.unknown, 408);
}

#[test]
fn an_empty_section_is_all_unknown() {
    let m = marks_matrix(&[]);
    assert_eq!(m.totals.unknown, 408);
    assert_eq!(m.totals.readable, 0);
}

/// Pins `Cell`'s JSON tags and the (camelCase) field names of `MarksTotals`.
/// If anyone removed `rename_all` from `Cell`, the tags would become
/// "Known"/"Unknown"/"Unexpected": every frontend `switch` would fall into
/// `assertNever`, and no Rust test would notice, without this pin.
#[test]
fn cell_and_totals_json_shape_is_pinned() {
    // index 27 suspicious (outside the 0..=7 mask) → Unexpected
    // columns 10 and 11 for The Forgotten and the 19s → Unknown, not located
    // the rest → Known
    let m = marks_matrix(&counters(523, &[(27, 49)]));
    let json = serde_json::to_value(&m).unwrap();

    let flat = json.to_string();
    assert!(
        flat.contains(r#""kind":"known""#),
        "missing the known tag: {flat}"
    );
    assert!(
        flat.contains(r#""kind":"unknown""#),
        "missing the unknown tag: {flat}"
    );
    assert!(
        flat.contains(r#""kind":"unexpected""#),
        "missing the unexpected tag: {flat}"
    );

    assert_eq!(
        json["totals"],
        serde_json::json!({
            "cells": 408,
            "readable": 367,
            "unknown": 40,
            "unexpected": 1,
            "started": 0
        }),
        "MarksTotals's field names must stay these, in camelCase"
    );
}

/// Pins `CharacterGroup`'s JSON tags: used to distinguish the three cell-placement
/// schemes, it must stay in camelCase like every other tagged enum on the IPC.
#[test]
fn character_group_json_tags_are_pinned() {
    assert_eq!(
        serde_json::to_value(CharacterGroup::Original).unwrap(),
        serde_json::json!("original")
    );
    assert_eq!(
        serde_json::to_value(CharacterGroup::Forgotten).unwrap(),
        serde_json::json!("forgotten")
    );
    assert_eq!(
        serde_json::to_value(CharacterGroup::Later).unwrap(),
        serde_json::json!("later")
    );
}
