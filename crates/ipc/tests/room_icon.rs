//! The room kinds' minimap icons: which name each kind wears, and the round trip that decides
//! whether the picture ever reaches the screen.
//!
//! The reference is rendered into a URL by the Tauri crate and parsed back by its protocol
//! handler, so a kind that renders to something `parse` refuses is a cell that quietly keeps
//! drawing our own symbol with the game installed — the failure looks exactly like success.

use std::collections::BTreeSet;

use ipc::{room_icons, IconRef, RoomKindView};

const KINDS: [RoomKindView; 14] = [
    RoomKindView::Start,
    RoomKindView::Normal,
    RoomKindView::Boss,
    RoomKindView::Treasure,
    RoomKindView::Shop,
    RoomKindView::Curse,
    RoomKindView::Challenge,
    RoomKindView::Sacrifice,
    RoomKindView::Arcade,
    RoomKindView::Library,
    RoomKindView::Miniboss,
    RoomKindView::Secret,
    RoomKindView::SuperSecret,
    RoomKindView::UltraSecret,
];

#[test]
fn every_room_reference_survives_the_round_trip() {
    for kind in KINDS {
        let r = IconRef::Room { kind };
        let path = r.to_path();
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
}

#[test]
fn a_room_path_we_did_not_write_is_refused_rather_than_guessed() {
    assert_eq!(IconRef::parse("room/"), None);
    assert_eq!(IconRef::parse("room/boudoir"), None);
    assert_eq!(IconRef::parse("room/boss/extra"), None);
    // The payload's spelling, not a prettified one: a kind travels as `superSecret`.
    assert_eq!(IconRef::parse("room/super_secret"), None);
}

#[test]
fn the_token_is_the_word_the_payload_uses() {
    // Same strings `serde` writes for `RoomKindView`, so a URL and a view-model never
    // disagree about what a kind is called.
    let path = IconRef::Room {
        kind: RoomKindView::SuperSecret,
    }
    .to_path();
    assert_eq!(path, "room/superSecret");
}

#[test]
fn the_fourteen_are_answered_once_each_and_in_the_palette_order() {
    let rows = room_icons(|r| Some(r.to_path()));
    let kinds: Vec<RoomKindView> = rows.iter().map(|r| r.kind).collect();
    assert_eq!(kinds, KINDS.to_vec());
}

#[test]
fn two_kinds_carry_no_picture_and_the_rest_carry_one() {
    // Normal, because the game draws nothing on a normal room; Start, because
    // `minimap_icons.anm2` has no icon for it. Every other kind has a name in that file, and
    // a third blank row would mean a mapping was dropped.
    let rows = room_icons(|r| Some(r.to_path()));
    let blank: Vec<RoomKindView> = rows
        .iter()
        .filter(|r| r.icon_url.is_none())
        .map(|r| r.kind)
        .collect();
    assert_eq!(blank, vec![RoomKindView::Start, RoomKindView::Normal]);
}

#[test]
fn no_two_kinds_share_a_picture() {
    // The whole point of the icon is telling two cells apart. Two kinds resolving to the same
    // crop would be a mapping mistake that looks like a drawing choice.
    let rows = room_icons(|r| Some(r.to_path()));
    let urls: Vec<&String> = rows.iter().filter_map(|r| r.icon_url.as_ref()).collect();
    let unique: BTreeSet<&&String> = urls.iter().collect();
    assert_eq!(unique.len(), urls.len());
}

#[test]
fn a_kind_the_game_has_no_icon_for_never_asks_for_one() {
    // Not merely "answers None": it must not build a reference the handler would then go and
    // look for. `room_icons` is called once per screen, but a lookup that cannot succeed is a
    // request that cannot succeed.
    let mut asked: Vec<String> = Vec::new();
    let rows = room_icons(|r| {
        asked.push(r.to_path());
        Some(r.to_path())
    });
    assert_eq!(rows.len(), 14);
    assert!(!asked.contains(&"room/normal".to_string()));
    assert!(!asked.contains(&"room/start".to_string()));
}
