//! The persisted settings, and the one value a user can put anything into.
//!
//! `settings.json` is a file the user can open and edit. The scale is a number in it, and
//! the app has exactly eleven sizes: a number that isn't one of them is a file we didn't
//! write, and it reads as the default rather than becoming a twelfth size nothing was
//! drawn at (`docs/BACKLOG.md` B26).

use ipc::{snap_percent, Settings, DEFAULT_SCALE, SCALE_PERCENTS};
use serde_json::{from_str, json, to_value};

#[test]
fn the_ladder_is_the_eleven_steps_in_order() {
    // Discord's zoom levels, which are Chromium's: the reference the owner handed over.
    assert_eq!(
        SCALE_PERCENTS,
        [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200]
    );
    assert!(SCALE_PERCENTS.contains(&DEFAULT_SCALE));
}

#[test]
fn a_step_is_kept_and_anything_else_reads_as_the_default() {
    for step in SCALE_PERCENTS {
        assert_eq!(snap_percent(step), step, "{step} is a step");
    }
    // Not the nearest step: 137 is not "about 125", it is a value we never wrote.
    for off in [0, 1, 49, 51, 137, 199, 201, u16::MAX] {
        assert_eq!(snap_percent(off), DEFAULT_SCALE, "{off} is not a step");
    }
}

#[test]
fn the_default_settings_are_the_default_scale() {
    let settings = Settings::default();
    assert_eq!(settings.scale(), DEFAULT_SCALE);
    // The whole object, not the fields one by one: this is the file's shape, and a field that
    // appears without anyone deciding to add it is what this catches.
    assert_eq!(
        to_value(&settings).unwrap(),
        json!({
            "activeProfileId": null,
            "scale": 100,
            "stayInBackground": true,
            "resumeTabs": true,
            "backgroundNoticeShown": false
        })
    );
}

#[test]
fn a_file_without_the_field_reads_as_the_default() {
    let settings: Settings = from_str("{}").expect("an empty object is settings");
    assert_eq!(settings.scale(), DEFAULT_SCALE);
    let kept: Settings = from_str(r#"{"scale":150}"#).expect("a step is settings");
    assert_eq!(kept.scale(), 150);
}

#[test]
fn a_hand_edited_size_is_read_through_the_same_door_as_every_other() {
    // The guarantee has to live in the accessor, not in the caller: a command that read the
    // field directly would put the app at a size nothing was drawn at.
    let odd: Settings = from_str(r#"{"scale":137}"#).expect("a number is settings");
    assert_eq!(odd.scale(), DEFAULT_SCALE);
}
