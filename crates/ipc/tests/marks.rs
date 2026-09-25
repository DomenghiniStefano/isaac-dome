use catalog::Catalog;
use core_save::{cell_index, Column};
use ipc::{
    counter_index, marks_matrix, Cell, CellLevel, CharacterGroup, IconRef, MarkArtView,
    SecondLevelView, BOSSES, ROSTER,
};

#[test]
fn tables_have_the_expected_shape() {
    assert_eq!(
        BOSSES.len(),
        12,
        "the widget the game draws has twelve columns"
    );
    assert_eq!(ROSTER.len(), 34);
    assert_eq!(BOSSES[0], "Mom's Heart");
    assert_eq!(BOSSES[9], "Delirium");
    assert_eq!(BOSSES[10], "Mother");
    assert_eq!(BOSSES[11], "The Beast");
    assert_eq!(
        (ROSTER[0].name, ROSTER[0].group),
        ("Isaac", CharacterGroup::Original)
    );
    assert_eq!(
        (ROSTER[14].name, ROSTER[14].group),
        ("The Forgotten", CharacterGroup::Forgotten)
    );
    assert_eq!(
        (ROSTER[15].name, ROSTER[15].group),
        ("Bethany", CharacterGroup::Later)
    );
    assert_eq!(
        (ROSTER[33].name, ROSTER[33].group),
        ("T. Jacob & Esau", CharacterGroup::Later)
    );
}

/// A row's name and its `players.xml` key are one table since card #82 (S2); these anchors pin
/// the pair at the rows where the two used to be easiest to shift apart — the Jacob & Esau row
/// that takes Jacob's key, the first Tainted row, and The Forgotten in both forms. Verified
/// against the file on 2026-09-03.
#[test]
fn the_roster_carries_each_rows_key_and_form() {
    let key_of = |row: usize| (ROSTER[row].name, ROSTER[row].key, ROSTER[row].tainted);
    assert_eq!(key_of(0), ("Isaac", "ISAAC", false));
    assert_eq!(key_of(14), ("The Forgotten", "THE_FORGOTTEN", false));
    assert_eq!(key_of(16), ("Jacob & Esau", "JACOB", false));
    assert_eq!(key_of(17), ("T. Isaac", "ISAAC", true));
    assert_eq!(key_of(31), ("T. Forgotten", "THE_FORGOTTEN", true));
    assert_eq!(key_of(33), ("T. Jacob & Esau", "JACOB", true));
    assert_eq!(
        ROSTER.iter().filter(|r| r.tainted).count(),
        17,
        "the seventeen Tainted"
    );
}

/// The layout's indices are pinned in `core-save`'s own tests (`marks_layout.rs`), where the
/// layout is. What this crate adds is the screen's translation, a column's index in `BOSSES`
/// onto the layout's `Column`, and that is what is held here: every cell, and nothing past the
/// twelfth column.
#[test]
fn counter_index_is_the_layouts_cell_at_the_columns_position() {
    for row in 0..ROSTER.len() {
        for (b, column) in Column::ALL.into_iter().enumerate() {
            assert_eq!(
                counter_index(row, b),
                cell_index(row, column),
                "{row} x {column:?}"
            );
        }
    }
    assert_eq!(counter_index(0, BOSSES.len()), None, "a thirteenth column");
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

/// A readable cell with no mark at all: the value every cell holds on a fresh profile.
/// Written out rather than derived from the bits — a helper that computed the level here
/// would be the production rule stated a second time, and every assertion below would
/// agree with it by construction.
const NEVER: Cell = Cell::Known {
    bits: 0,
    level: CellLevel::Empty,
    online: false,
};

#[test]
fn matrix_has_the_expected_shape_and_totals() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    assert_eq!(m.characters.len(), 34);
    assert_eq!(m.bosses.len(), 12);
    assert_eq!(m.totals.cells, 408);
    assert_eq!(m.totals.unknown, 20);
    assert_eq!(m.totals.readable, 388);
    assert_eq!(m.totals.unexpected, 0);
    assert_eq!(m.totals.normal, 0);
    assert_eq!(m.totals.hard, 0);
}

#[test]
fn the_hole_is_now_the_beast_alone_for_the_last_twenty_rows() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    // row 15 = Bethany, column 9 = Delirium: located since 2026-09-08.
    assert_eq!(m.characters[15].cells[9], NEVER);
    // Column 10 is Mother, located since 2026-09-20 for these rows too.
    assert_eq!(m.characters[15].cells[10], NEVER);
    assert_eq!(m.characters[14].cells[10], NEVER);
    // Column 11 is The Beast, and it is what is left of the hole.
    assert_eq!(m.characters[15].cells[11], Cell::Unknown);
    assert_eq!(m.characters[14].cells[11], Cell::Unknown);
    // Row 0 = Isaac: the original characters have all twelve columns.
    assert_eq!(m.characters[0].cells[10], NEVER);
    assert_eq!(m.characters[0].cells[11], NEVER);
}

#[test]
fn a_read_value_becomes_a_bit_mask() {
    let m = marks_matrix(&counters(523, &[(27, 3), (41, 7)]), None, no_icon);
    assert_eq!(
        m.characters[0].cells[0],
        Cell::Known {
            bits: 3,
            level: CellLevel::Hard,
            online: false
        }
    );
    assert_eq!(
        m.characters[0].cells[1],
        Cell::Known {
            bits: 7,
            level: CellLevel::Hard,
            online: true
        }
    );
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
    assert_eq!(m.characters[0].cells[0], NEVER, "index 27 is still there");
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
            "readable": 387,
            "unknown": 20,
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
    // 3 = both bits, 7 = both plus the online bit, 1 = the first level alone,
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
fn a_cell_holding_only_the_online_bit_counts_on_neither_side() {
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

/// B21: the cell says what it is, instead of handing the frontend a mask to decode.
///
/// The three facts a mask holds are three fields: the raw value it was read as, the level
/// it reached, and whether the mark was taken online. They used to be one number, and the
/// rules that turned it into a reading lived in two TypeScript files at once — which is
/// how bit 2 went on being called "a third level, meaning unconfirmed" in the tooltip for
/// eight days after `docs/save-format.md` had it as "won online".
///
/// `online` is a field beside the level and never a level of its own: it says *where* a
/// mark was taken, not how high it is.
#[test]
fn a_known_cell_says_the_level_and_whether_it_was_taken_online() {
    let m = marks_matrix(
        &counters(523, &[(27, 1), (41, 3), (55, 2), (69, 7), (83, 5), (97, 4)]),
        None,
        no_icon,
    );
    let cells = &m.characters[0].cells;
    assert_eq!(
        cells[0],
        Cell::Known {
            bits: 1,
            level: CellLevel::Normal,
            online: false
        },
    );
    assert_eq!(
        cells[1],
        Cell::Known {
            bits: 3,
            level: CellLevel::Hard,
            online: false
        },
    );
    assert_eq!(
        cells[2],
        Cell::Known {
            bits: 2,
            level: CellLevel::Hard,
            online: false
        },
        "a bare 2 is the normal mark overwritten, so hard wins without bit 0 (B58)"
    );
    assert_eq!(
        cells[3],
        Cell::Known {
            bits: 7,
            level: CellLevel::Hard,
            online: true
        },
    );
    assert_eq!(
        cells[4],
        Cell::Known {
            bits: 5,
            level: CellLevel::Normal,
            online: true
        },
    );
    assert_eq!(
        cells[5],
        Cell::Known {
            bits: 4,
            level: CellLevel::Empty,
            online: true
        },
        "no sample has ever held 4, but the reading is total: the bit is reported where it \
         is, and an empty level is not a mark"
    );
}

/// The reading crosses the IPC as its own fields, in camelCase, with the level a bare
/// string and not a tagged object: it carries no data, so a tag would add a key per cell
/// and hide that the field is a value (CLAUDE.md, "Enums on the IPC").
#[test]
fn the_reading_json_shape_is_pinned() {
    let m = marks_matrix(&counters(523, &[(27, 7)]), None, no_icon);
    let json = serde_json::to_value(&m).unwrap();
    assert_eq!(
        json["characters"][0]["cells"][0],
        serde_json::json!({ "kind": "known", "bits": 7, "level": "hard", "online": true }),
    );
    assert_eq!(
        json["characters"][0]["cells"][1],
        serde_json::json!({ "kind": "known", "bits": 0, "level": "empty", "online": false }),
    );
}

/// A catalog that exists. What the widget's URL needs is only that the game's archives are
/// open — the paper and the symbols come from the anm2 files, never from the catalog.
fn catalog_with_heads() -> Catalog {
    let anm2 = coop_menu_anm2();
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(anm2.clone()),
        _ => None,
    })
}

// The emblem's address. The band over the matrix draws the game's own completion widget,
// and what it draws is read from the same cells the footer's column totals are: a column is
// at hard when *every* readable cell in it is hard, at normal when at least one has a level,
// and absent when none has. So the picture and the numbers under it can only ever agree.

/// The value a cell holds at each level, as `cell_at` reads it: bit 0 is the first level,
/// bit 1 the second. Written out rather than derived, like `NEVER` above.
const NORMAL: u32 = 1;
const HARD: u32 = 3;

/// Every cell of `column` set to `value`, for the rows that have one.
fn whole_column(column: usize, value: u32) -> Vec<(usize, u32)> {
    (0..ROSTER.len())
        .filter_map(|row| counter_index(row, column).map(|i| (i, value)))
        .collect()
}

/// The `widget/…` segment the matrix asks for, with an icon builder that answers.
fn widget_fills(counters: &[u32], catalog: Option<&Catalog>) -> String {
    let m = marks_matrix(counters, catalog, |r| Some(r.to_path()));
    m.widget_url
        .expect("a catalog means a widget")
        .strip_prefix("widget/")
        .expect("the widget's own address")
        .to_string()
}

#[test]
fn an_untouched_profile_asks_for_a_bare_paper() {
    let c = catalog_with_heads();
    assert_eq!(
        widget_fills(&counters(523, &[]), Some(&c)),
        "------------",
        "nothing done draws no symbol at all, not twelve faint ones"
    );
}

#[test]
fn a_column_reaches_hard_only_when_every_readable_cell_does() {
    let c = catalog_with_heads();
    // One character on hard is not the column: thirty-three others have not done it.
    let one = marks_matrix(
        &counters(523, &[(counter_index(0, 0).unwrap(), HARD)]),
        Some(&c),
        |r| Some(r.to_path()),
    );
    assert_eq!(
        one.widget_url.unwrap(),
        "widget/n-----------",
        "some progress is the normal symbol"
    );
    // The whole column on hard is.
    assert_eq!(
        widget_fills(&counters(523, &whole_column(0, HARD)), Some(&c)),
        "h-----------"
    );
    // The whole column on normal is not: `hard` is what "done" means for a column (B22).
    assert_eq!(
        widget_fills(&counters(523, &whole_column(0, NORMAL)), Some(&c)),
        "n-----------"
    );
}

#[test]
fn a_column_nobody_can_read_is_not_counted_against_it() {
    // The Beast is unlocated for twenty rows, so its readable cells are the fourteen
    // originals'. Filling those is the whole of what can be read, and the column is done —
    // the same rule `TallyTone::Full` uses over `readable`, not over every row.
    let c = catalog_with_heads();
    let fills = widget_fills(&counters(523, &whole_column(11, HARD)), Some(&c));
    assert_eq!(fills.chars().nth(11), Some('h'));
}

#[test]
fn without_the_game_there_is_no_widget_to_ask_for() {
    // Same rule as the symbols and the heads: a URL nothing can serve draws a broken image
    // where the band should simply have no picture.
    let m = marks_matrix(&counters(523, &[]), None, |r| Some(r.to_path()));
    assert!(m.widget_url.is_none());
}

/// What the second level of each column is called, one per boss (B66). In Greed bit 1 is
/// Ultra Greedier, measured on 2026-09-12 on three characters, and a mode is not a
/// difficulty; in the other eleven it is hard, on the one observation of 2026-09-20 and the
/// owner's wording for the screen. The list is read from Rust so the screen never decides
/// which column is Greed.
#[test]
fn greed_names_its_second_level_ultra_greedier_and_the_other_columns_hard() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    assert_eq!(m.second_levels.len(), m.bosses.len(), "one per column");
    for (boss, level) in m.bosses.iter().zip(&m.second_levels) {
        let expected = if boss == "Greed" {
            SecondLevelView::UltraGreedier
        } else {
            SecondLevelView::Hard
        };
        assert_eq!(*level, expected, "{boss}");
    }
    assert!(
        m.second_levels.contains(&SecondLevelView::UltraGreedier),
        "the one column the rule is about is in the list"
    );
}

#[test]
fn a_second_level_crosses_as_a_bare_camel_case_string() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    let json = serde_json::to_value(&m).expect("serializes");
    let levels = json["secondLevels"].as_array().expect("an array");
    assert_eq!(levels[0], "hard");
    assert_eq!(levels[7], "ultraGreedier");
}
