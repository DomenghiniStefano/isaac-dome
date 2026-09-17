use catalog::Catalog;
use ipc::{
    counter_index, marks_matrix, Cell, CharacterGroup, IconRef, MarkArtView, BOSSES, CHARACTERS,
};

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
                                                 // Delirium: 212 belongs to another family.
    assert_eq!(counter_index(14, 9), Some(213));
    // Mother and The Beast for The Forgotten: derived from the spacing, never observed
    // moving, so they stay unlocated rather than pointing at a guess.
    assert_eq!(counter_index(14, 10), None);
    assert_eq!(counter_index(14, 11), None);
}

#[test]
fn later_characters_now_reach_delirium() {
    assert_eq!(counter_index(15, 0), Some(214)); // Bethany, Mom's Heart
                                                 // T. Jacob & Esau, Hush = 366 + 18.
    assert_eq!(counter_index(33, 8), Some(384));
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

/// No catalog, no URL: what a machine without the game gets.
fn no_icon(_: &IconRef) -> Option<String> {
    None
}

#[test]
fn matrix_has_the_expected_shape_and_totals() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    assert_eq!(m.characters.len(), 34);
    assert_eq!(m.bosses.len(), 12);
    assert_eq!(m.totals.cells, 408);
    assert_eq!(m.totals.unknown, 40);
    assert_eq!(m.totals.readable, 368);
    assert_eq!(m.totals.unexpected, 0);
    assert_eq!(m.totals.normal, 0);
    assert_eq!(m.totals.hard, 0);
}

#[test]
fn the_hole_is_now_mother_and_the_beast_for_the_last_twenty_rows() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
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
    let m = marks_matrix(&counters(523, &[(27, 3), (41, 7)]), None, no_icon);
    assert_eq!(m.characters[0].cells[0], Cell::Known { bits: 3 });
    assert_eq!(m.characters[0].cells[1], Cell::Known { bits: 7 });
    assert_eq!(m.totals.normal, 2, "only readable, non-zero cells count");
}

#[test]
fn a_value_outside_the_mask_range_is_flagged_not_truncated() {
    let m = marks_matrix(&counters(523, &[(27, 49)]), None, no_icon);
    assert_eq!(
        m.characters[0].cells[0],
        Cell::Unexpected { value: 49 },
        "49 isn't a mask: the table would be wrong, it must not be truncated to 1"
    );
    assert_eq!(m.totals.unexpected, 1);
    assert_eq!(m.totals.normal, 0, "a suspicious cell has no level at all");
}

#[test]
fn a_shorter_section_yields_unknown_not_a_panic() {
    // 300 counters: the 19-blocks (214..384) mostly fall outside.
    let m = marks_matrix(&counters(300, &[]), None, no_icon);
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
    let m = marks_matrix(&[], None, no_icon);
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
    let m = marks_matrix(&counters(523, &[(27, 49)]), None, no_icon);
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
            "normal": 0,
            "hard": 0
        }),
        "MarksTotals's field names must stay these, in camelCase"
    );
}

/// B22: one count became two, because a row says how many bosses at normal and how many at
/// hard. `started` is gone rather than kept beside them — it was `normal` under another
/// name, and two spellings of one number is how the two drift.
///
/// Hard is a subset of normal, never a second, disjoint tally: a mark taken on hard counts
/// as taken on normal too. The file corroborates the rule instead of merely allowing it —
/// a cell goes 1 → 2, so a bare 2 is the normal mark overwritten and not a hard mark taken
/// by someone who never took the normal one (B58, measured 2026-09-17 on the 638-era
/// series). What bit 1 *means* outside Greed is still unmeasured, which is why nothing here
/// is named after a mode.
#[test]
fn the_totals_count_hard_inside_normal_and_never_beside_it() {
    // 3 = both bits, 7 = both plus the unconfirmed one, 1 = the first level alone,
    // 2 = the second alone. Four cells with a level, three of them hard.
    let m = marks_matrix(
        &counters(523, &[(27, 3), (41, 7), (55, 1), (69, 2)]),
        None,
        no_icon,
    );
    assert_eq!(m.totals.normal, 4, "every cell that reached a level");
    assert_eq!(m.totals.hard, 3, "the three that reached the second");
    assert!(
        m.totals.hard <= m.totals.normal && m.totals.normal <= m.totals.readable,
        "hard {} <= normal {} <= readable {}",
        m.totals.hard,
        m.totals.normal,
        m.totals.readable
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

#[test]
fn rows_know_whether_they_are_tainted() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    assert!(!m.characters[0].tainted, "Isaac");
    assert!(!m.characters[14].tainted, "The Forgotten");
    assert!(!m.characters[16].tainted, "Jacob & Esau");
    assert!(m.characters[17].tainted, "T. Isaac");
    assert!(m.characters[33].tainted, "T. Jacob & Esau");
    assert_eq!(m.characters.iter().filter(|r| r.tainted).count(), 17);
}

#[test]
fn without_a_catalog_nothing_carries_a_url() {
    // Even an icon builder that would answer: without the game's archives there is nothing
    // to serve, and a URL that 404s would draw a broken image instead of the fallback.
    let m = marks_matrix(&counters(523, &[]), None, |r| Some(r.to_path()));
    assert_eq!(m.art.len(), m.bosses.len(), "art[i] draws bosses[i]");
    assert!(m
        .art
        .iter()
        .all(|a| a.normal_url.is_none() && a.hard_url.is_none()));
    assert!(m.characters.iter().all(|r| r.head_url.is_none()));
}

const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\">
<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" />
<player id=\"21\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac_b.png\" />
</players>";

/// `coop menu.anm2` with frame 0 uncropped (the "?" placeholder) and one cell per frame after.
fn coop_menu_anm2() -> Vec<u8> {
    let frames: String = (1..=37)
        .map(|f| {
            format!(
                r#"<Frame XCrop="{}" YCrop="0" Width="32" Height="32" Visible="true"/>"#,
                32 * f
            )
        })
        .collect();
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Path="coop menu.png" Id="0"/></Spritesheets><Layers><Layer Name="Main" Id="0" SpritesheetId="0"/></Layers></Content><Animations><Animation Name="Main"><LayerAnimations><LayerAnimation LayerId="0"><Frame Delay="1" Visible="true"/>{frames}</LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#
    )
    .into_bytes()
}

#[test]
fn with_a_catalog_urls_follow_what_it_knows() {
    let anm2 = coop_menu_anm2();
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(anm2.clone()),
        _ => None,
    });
    let m = marks_matrix(&counters(523, &[]), Some(&c), |r| Some(r.to_path()));
    assert_eq!(
        m.characters[0].head_url.as_deref(),
        Some("head/0"),
        "Isaac has a head"
    );
    assert_eq!(
        m.characters[17].head_url.as_deref(),
        Some("head/17"),
        "T. Isaac has a head"
    );
    assert_eq!(
        m.characters[1].head_url, None,
        "Magdalene isn't in this catalog"
    );
    assert_eq!(
        m.art[9],
        MarkArtView {
            normal_url: Some("mark/9/normal".to_string()),
            hard_url: Some("mark/9/hard".to_string()),
        }
    );
}

#[test]
fn a_cell_holding_only_the_unconfirmed_bit_counts_on_neither_side() {
    // 4 has never been observed; if it appears, the grid draws it empty, and so must the total.
    let m = marks_matrix(&counters(523, &[(27, 4), (41, 5)]), None, no_icon);
    assert_eq!(
        m.totals.normal, 1,
        "5 carries the first level, 4 carries nothing the grid draws"
    );
    assert_eq!(
        m.totals.hard, 0,
        "neither of them reached the second level: 5 is bits 0 and 2"
    );
}

#[test]
fn the_new_fields_are_camel_case_on_the_wire() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    let json = serde_json::to_value(&m).unwrap();
    let row = json["characters"][0].as_object().unwrap();
    assert!(
        row.contains_key("tainted") && row.contains_key("headUrl"),
        "{row:?}"
    );
    let art = json["art"][0].as_object().unwrap();
    assert!(
        art.contains_key("normalUrl") && art.contains_key("hardUrl"),
        "{art:?}"
    );
}
