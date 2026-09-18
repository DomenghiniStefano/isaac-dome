//! What a preset leaves in the deck, and what it takes out — by name.
//!
//! The accounting is why `Excluded` exists: a target that disappears without a bucket is the
//! failure this shape prevents, so the identity `targets + the four counts == target_count()`
//! is asserted on every preset in this file and not on one.

use roll::{contributions, deck, CellValue, Preset, Selection, Space, Target};

fn known(bits: u8) -> CellValue {
    CellValue::Known { bits }
}

/// 2 rows x 3 columns, Greed is column 1, every cell missing, both rows playable.
/// 2 * 3 + 2 = 8 targets.
fn empty_space() -> Space {
    Space::new(2, 3, 1, vec![known(0); 6], vec![true, true]).expect("declared shape")
}

fn accounted(space: &Space, preset: &Preset) -> bool {
    let d = deck(space, preset);
    let e = &d.excluded;
    d.targets.len() + e.taken + e.unreadable + e.locked + e.filtered == space.target_count()
}

#[test]
fn the_default_preset_offers_every_missing_target() {
    let space = empty_space();
    let d = deck(&space, &Preset::default());
    assert_eq!(d.targets.len(), 8);
    assert_eq!(d.excluded, roll::Excluded::default());
}

#[test]
fn the_default_preset_is_missing_only_playable_and_everything_selected() {
    let p = Preset::default();
    assert_eq!(p.characters, Selection::All);
    assert_eq!(p.columns, Selection::All);
    assert!(!p.include_taken, "the screen opens on what you are missing");
    assert!(
        p.only_playable,
        "a character you have not unlocked is not a run"
    );
}

#[test]
fn a_taken_cell_leaves_the_deck_and_is_counted_as_taken() {
    let mut cells = vec![known(0); 6];
    cells[0] = known(1);
    let space = Space::new(2, 3, 1, cells, vec![true, true]).expect("declared shape");
    let d = deck(&space, &Preset::default());
    assert_eq!(d.targets.len(), 7);
    assert_eq!(d.excluded.taken, 1);
    assert!(accounted(&space, &Preset::default()));
}

#[test]
fn an_unreadable_cell_is_never_offered_as_missing() {
    // We cannot claim a cell is missing when the file does not say.
    let mut cells = vec![known(0); 6];
    cells[0] = CellValue::Unreadable;
    let space = Space::new(2, 3, 1, cells, vec![true, true]).expect("declared shape");
    let d = deck(&space, &Preset::default());
    assert!(!d.targets.contains(&Target::Mark {
        character: 0,
        column: 0
    }));
    assert_eq!(d.excluded.unreadable, 1);
}

#[test]
fn with_include_taken_an_unreadable_cell_is_drawable_and_nothing_is_excluded_for_its_state() {
    let mut cells = vec![known(0); 6];
    cells[0] = CellValue::Unreadable;
    cells[2] = known(3);
    let space = Space::new(2, 3, 1, cells, vec![true, true]).expect("declared shape");
    let preset = Preset {
        include_taken: true,
        ..Preset::default()
    };
    let d = deck(&space, &preset);
    assert_eq!(d.targets.len(), 8);
    assert_eq!(d.excluded.taken, 0);
    assert_eq!(d.excluded.unreadable, 0);
    assert!(accounted(&space, &preset));
}

#[test]
fn an_unplayable_row_is_locked_out_with_every_target_it_has() {
    let space = Space::new(2, 3, 1, vec![known(0); 6], vec![true, false]).expect("declared shape");
    let preset = Preset::default();
    let d = deck(&space, &preset);
    // Row 1 has three marks and one Greedier.
    assert_eq!(d.excluded.locked, 4);
    assert_eq!(d.targets.len(), 4);
    assert!(accounted(&space, &preset));
}

#[test]
fn only_playable_off_puts_the_locked_row_back() {
    let space = Space::new(2, 3, 1, vec![known(0); 6], vec![true, false]).expect("declared shape");
    let preset = Preset {
        only_playable: false,
        ..Preset::default()
    };
    let d = deck(&space, &preset);
    assert_eq!(d.excluded.locked, 0);
    assert_eq!(d.targets.len(), 8);
}

#[test]
fn an_unpicked_character_is_filtered_and_says_so() {
    let space = empty_space();
    let preset = Preset {
        characters: Selection::Only { ids: vec![0] },
        ..Preset::default()
    };
    let d = deck(&space, &preset);
    assert_eq!(d.excluded.filtered, 4);
    assert!(d.targets.iter().all(|t| match t {
        Target::Mark { character, .. } => *character == 0,
        Target::Greedier { character } => *character == 0,
    }));
    assert!(accounted(&space, &preset));
}

#[test]
fn an_unpicked_column_takes_its_marks_and_leaves_the_others() {
    let space = empty_space();
    let preset = Preset {
        columns: Selection::Only { ids: vec![0, 2] },
        ..Preset::default()
    };
    let d = deck(&space, &preset);
    // Column 1 is Greed: dropping it takes two marks and both Greedier targets.
    assert_eq!(d.excluded.filtered, 4);
    assert_eq!(d.targets.len(), 4);
    assert!(accounted(&space, &preset));
}

#[test]
fn greedier_follows_the_greed_column_and_not_the_others() {
    let space = empty_space();
    let preset = Preset {
        columns: Selection::Only { ids: vec![1] },
        ..Preset::default()
    };
    let d = deck(&space, &preset);
    // Two marks on the Greed column, plus one Greedier per row.
    assert_eq!(d.targets.len(), 4);
    assert_eq!(
        d.targets
            .iter()
            .filter(|t| matches!(t, Target::Greedier { .. }))
            .count(),
        2
    );
}

#[test]
fn a_target_is_never_in_two_buckets_at_once() {
    // Taken, unreadable, unplayable and unpicked all at once on the same space: the identity
    // holds only if the buckets are exclusive.
    let cells = vec![
        known(1),
        CellValue::Unreadable,
        known(0),
        known(7),
        known(0),
        CellValue::Unreadable,
    ];
    let space = Space::new(2, 3, 1, cells, vec![true, false]).expect("declared shape");
    for characters in [Selection::All, Selection::Only { ids: vec![1] }] {
        for columns in [Selection::All, Selection::Only { ids: vec![0] }] {
            for include_taken in [false, true] {
                for only_playable in [false, true] {
                    let preset = Preset {
                        characters: characters.clone(),
                        columns: columns.clone(),
                        include_taken,
                        only_playable,
                    };
                    assert!(
                        accounted(&space, &preset),
                        "preset {preset:?} loses a target"
                    );
                }
            }
        }
    }
}

#[test]
fn a_rows_count_is_what_ticking_it_would_be_worth_not_what_it_gives_now() {
    // Character 1 is unpicked. Its count must still say what it holds, otherwise every
    // unticked row reads zero and the number cannot help anyone decide.
    let space = empty_space();
    let preset = Preset {
        characters: Selection::Only { ids: vec![0] },
        ..Preset::default()
    };
    assert_eq!(contributions(&space, &preset).characters, vec![4, 4]);
}

#[test]
fn a_rows_count_still_obeys_the_other_axes() {
    // Only column 0 is picked, so each character is worth one mark and no Greedier.
    let space = empty_space();
    let preset = Preset {
        columns: Selection::Only { ids: vec![0] },
        ..Preset::default()
    };
    let c = contributions(&space, &preset);
    assert_eq!(c.characters, vec![1, 1]);
    // And each column is worth what it holds with every character allowed: two marks, plus the
    // two Greedier targets on the Greed column.
    assert_eq!(c.columns, vec![2, 4, 2]);
}

#[test]
fn a_rows_count_drops_what_the_other_axes_exclude() {
    let space = Space::new(2, 3, 1, vec![known(0); 6], vec![true, false]).expect("declared shape");
    let c = contributions(&space, &Preset::default());
    // Row 1 is not playable: ticking it would be worth nothing, and that is the honest number.
    assert_eq!(c.characters, vec![4, 0]);
    assert_eq!(c.columns, vec![1, 2, 1]);
}
