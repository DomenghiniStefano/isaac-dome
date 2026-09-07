use ipc::{counter_index, marks_matrix, Cell, CharacterGroup, BOSSES, CHARACTERS};

#[test]
fn tables_have_the_expected_shape() {
    assert_eq!(BOSSES.len(), 10);
    assert_eq!(CHARACTERS.len(), 34);
    assert_eq!(BOSSES[0], "Mom's Heart");
    assert_eq!(BOSSES[9], "Delirium");
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
}

#[test]
fn the_forgotten_uses_single_cells() {
    assert_eq!(counter_index(14, 0), Some(203)); // Mom's Heart
    assert_eq!(counter_index(14, 8), Some(211)); // Hush
    assert_eq!(counter_index(14, 9), Some(213)); // Delirium: 212 belongs to another family
}

#[test]
fn later_characters_stop_at_hush() {
    assert_eq!(counter_index(15, 0), Some(214)); // Bethany, Mom's Heart
    assert_eq!(counter_index(33, 8), Some(384)); // T. Jacob & Esau, Hush = 366 + 18
    assert_eq!(
        counter_index(15, 9),
        None,
        "the Delirium column isn't located for the 19s: this is where the unknown starts"
    );
}

#[test]
fn exactly_nineteen_cells_are_unlocated() {
    let unlocated = (0..CHARACTERS.len())
        .flat_map(|c| (0..BOSSES.len()).map(move |b| (c, b)))
        .filter(|&(c, b)| counter_index(c, b).is_none())
        .count();
    assert_eq!(unlocated, 19);
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
    assert_eq!(m.bosses.len(), 10);
    assert_eq!(m.totals.cells, 340);
    assert_eq!(m.totals.unknown, 19);
    assert_eq!(m.totals.readable, 321);
    assert_eq!(m.totals.unexpected, 0);
    assert_eq!(m.totals.started, 0);
}

#[test]
fn the_delirium_column_is_unknown_for_later_characters() {
    let m = marks_matrix(&counters(523, &[]));
    // row 15 = Bethany, column 9 = Delirium
    assert_eq!(m.characters[15].cells[9], Cell::Unknown);
    // row 0 = Isaac, same column: located
    assert_eq!(m.characters[0].cells[9], Cell::Known { bits: 0 });
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
    assert!(m.totals.unknown > 19);
    assert_eq!(m.totals.readable + m.totals.unknown, 340);
}

#[test]
fn an_empty_section_is_all_unknown() {
    let m = marks_matrix(&[]);
    assert_eq!(m.totals.unknown, 340);
    assert_eq!(m.totals.readable, 0);
}

/// Pins `Cell`'s JSON tags and the (camelCase) field names of `MarksTotals`.
/// If anyone removed `rename_all` from `Cell`, the tags would become
/// "Known"/"Unknown"/"Unexpected": every frontend `switch` would fall into
/// `assertNever`, and no Rust test would notice, without this pin.
#[test]
fn cell_and_totals_json_shape_is_pinned() {
    // index 27 suspicious (outside the 0..=7 mask) → Unexpected
    // index 15/9 (Bethany × Delirium) → Unknown, not located for the 19s
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
            "cells": 340,
            "readable": 320,
            "unknown": 19,
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
