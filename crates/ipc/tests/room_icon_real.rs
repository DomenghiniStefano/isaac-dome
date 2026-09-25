//! The room icons against the installed game: every name we map a kind onto is really in
//! `minimap_icons.anm2`, and every crop is a real 16x16 piece of a sheet the archives hold.
//!
//! `room_icon.rs` pins the rules — the round trip, which kinds go bare, the one-to-one — and
//! cannot see the only mistake that matters here: a name that is not in the file. That one
//! raises nothing. The reference resolves to `None`, the cell falls back to our drawing, and
//! the screen looks exactly like a machine without the game.

use ipc::{crop_png, decode_rgba, icon_source, trim_opaque, IconRef, RoomKindView};

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

/// The two the game itself has no icon for, checked here rather than assumed: if a later patch
/// adds one, this list is what stops the screen from quietly keeping our drawing.
const BARE: [RoomKindView; 2] = [RoomKindView::Start, RoomKindView::Normal];

#[test]
fn every_mapped_kind_crops_a_real_icon() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let catalog = catalog::Catalog::build(|p| rs.read(p));
    for kind in KINDS {
        let sprite = icon_source(
            &catalog,
            &ipc::for_tests::bosses(&catalog),
            &IconRef::Room { kind },
        );
        if BARE.contains(&kind) {
            assert!(sprite.is_none(), "{kind:?}: the game has no icon for it");
            continue;
        }
        let sprite = sprite.unwrap_or_else(|| {
            panic!("{kind:?}: the name we map it onto is not in minimap_icons.anm2")
        });
        let sheet = rs
            .read(&sprite.path)
            .unwrap_or_else(|| panic!("{kind:?}: sheet {} not in the archives", sprite.path));
        let r = sprite.rect.expect("an icon is a piece of a sheet");
        let png = crop_png(&sheet, r.x, r.y, r.w, r.h)
            .unwrap_or_else(|| panic!("{kind:?}: the rectangle is outside the sheet"));
        let (w, h, _) = decode_rgba(&png).expect("the crop is a PNG");
        assert_eq!((w, h), (16, 16), "{kind:?}");
    }
}

/// The room icons are served trimmed to their drawing, and this says what that means on the
/// real sheet — as a property, because the sizes are a fixture of today's `minimap_icons.png`
/// and the next patch is allowed to change every one of them.
///
/// Three claims: the trim keeps a drawing, it fits inside the declared square, and it has no
/// transparent border left — which is what "centring it centres the drawing" rests on.
#[test]
fn a_room_icon_is_served_with_no_transparent_margin_around_it() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let catalog = catalog::Catalog::build(|p| rs.read(p));
    // The vacuity guard: if every icon already filled its square the three claims below would
    // hold on an untrimmed picture, and the test would report a check it never made. Measured
    // on 2026-09-20, all twelve are smaller than their square.
    let mut trimmed_smaller = 0;
    for kind in KINDS {
        let Some(sprite) = icon_source(
            &catalog,
            &ipc::for_tests::bosses(&catalog),
            &IconRef::Room { kind },
        ) else {
            continue;
        };
        let sheet = rs.read(&sprite.path).expect("the sheet is in the archives");
        let r = sprite.rect.expect("an icon is a piece of a sheet");
        let png = crop_png(&sheet, r.x, r.y, r.w, r.h).expect("the rectangle is on the sheet");
        let trimmed = trim_opaque(&png)
            .unwrap_or_else(|| panic!("{kind:?}: the icon's square holds no drawing at all"));
        let (w, h, pixel) = decode_rgba(&trimmed).expect("the trim is a PNG");
        assert!(
            w <= r.w && h <= r.h,
            "{kind:?}: {w}x{h} out of {}x{}",
            r.w,
            r.h
        );
        if w < r.w || h < r.h {
            trimmed_smaller += 1;
        }
        let opaque = |x: u32, y: u32| pixel[((y * w + x) as usize) * 4 + 3] != 0;
        assert!((0..w).any(|x| opaque(x, 0)), "{kind:?}: empty first row");
        assert!((0..w).any(|x| opaque(x, h - 1)), "{kind:?}: empty last row");
        assert!((0..h).any(|y| opaque(0, y)), "{kind:?}: empty first column");
        assert!(
            (0..h).any(|y| opaque(w - 1, y)),
            "{kind:?}: empty last column"
        );
    }
    assert!(
        trimmed_smaller > 0,
        "no icon was trimmed at all: the property above proved nothing"
    );
}

#[test]
fn no_two_kinds_crop_the_same_piece_of_the_sheet() {
    // Twelve names, twelve pictures. Two kinds landing on one crop would look like a drawing
    // choice and be a mapping mistake.
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let catalog = catalog::Catalog::build(|p| rs.read(p));
    let mut seen: Vec<(String, Option<catalog::Rect>)> = Vec::new();
    for kind in KINDS {
        if let Some(sprite) = icon_source(
            &catalog,
            &ipc::for_tests::bosses(&catalog),
            &IconRef::Room { kind },
        ) {
            let key = (sprite.path.clone(), sprite.rect);
            assert!(!seen.contains(&key), "{kind:?}: already taken");
            seen.push(key);
        }
    }
    assert_eq!(seen.len(), KINDS.len() - BARE.len());
}
