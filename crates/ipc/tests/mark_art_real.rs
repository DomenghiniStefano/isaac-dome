//! The marks map against the installed game: both anm2 files read, every column resolves
//! to two tiers, and each crop is a real 16×16 piece of a sheet that exists in the archives.
//! The synthetic `mark_art.rs` pins the rules; this says whether the game still agrees.

use ipc::{
    crop_png, decode_rgba, mark_source, overlay, paper_source, widget_source, MarkFill, MarkFrames,
    MarkTier, BOSSES, LOBBY_ANM2, WIDGET_ANM2,
};

#[test]
fn every_column_crops_two_real_symbols() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let read = |p: &str| {
        rs.read(p)
            .and_then(|b| catalog::anm2_frames(&b))
            .unwrap_or_default()
    };
    let frames = MarkFrames {
        widget: read(WIDGET_ANM2),
        lobby: read(LOBBY_ANM2),
    };
    for (column, boss) in core_save::Column::ALL.into_iter().zip(BOSSES) {
        for tier in [MarkTier::Normal, MarkTier::Hard] {
            let sprite = mark_source(column, tier, &frames)
                .unwrap_or_else(|| panic!("{boss} {tier:?}: no frame"));
            let sheet = rs
                .read(&sprite.path)
                .unwrap_or_else(|| panic!("{boss}: sheet {} not in the archives", sprite.path));
            let r = sprite.rect.expect("a mark is a piece of a sheet");
            let png = crop_png(&sheet, r.x, r.y, r.w, r.h)
                .unwrap_or_else(|| panic!("{boss} {tier:?}: the rectangle is outside the sheet"));
            let (w, h, _) = decode_rgba(&png).expect("the crop is a PNG");
            assert_eq!((w, h), (16, 16), "{boss} {tier:?}");
        }
    }
}

/// The two anm2 files as the installed game ships them, or `None` when `samples/packed`
/// isn't there — the skip is `test_support`'s to declare.
fn real_frames() -> Option<(unpack::ResourceSet, MarkFrames)> {
    let dir = test_support::packed_dir()?;
    let rs = unpack::ResourceSet::open(&dir);
    let read = |p: &str| {
        rs.read(p)
            .and_then(|b| catalog::anm2_frames(&b))
            .unwrap_or_default()
    };
    let frames = MarkFrames {
        widget: read(WIDGET_ANM2),
        lobby: read(LOBBY_ANM2),
    };
    Some((rs, frames))
}

#[test]
fn the_paper_is_ninety_six_square_at_both_tiers() {
    let Some((rs, frames)) = real_frames() else {
        return;
    };
    for tier in [MarkTier::Normal, MarkTier::Hard] {
        let sprite = paper_source(tier, &frames).unwrap_or_else(|| panic!("{tier:?}: no paper"));
        let sheet = rs.read(&sprite.path).expect("the widget's sheet");
        let r = sprite.rect.expect("the paper is a piece of a sheet");
        let png = crop_png(&sheet, r.x, r.y, r.w, r.h).expect("inside the sheet");
        assert_eq!(decode_rgba(&png).map(|(w, h, _)| (w, h)), Some((96, 96)));
    }
    // The two tiers are two different sheets of paper, not the same one twice — the whole
    // reward the emblem carries.
    assert_ne!(
        paper_source(MarkTier::Normal, &frames).and_then(|s| s.rect),
        paper_source(MarkTier::Hard, &frames).and_then(|s| s.rect)
    );
}

#[test]
fn every_mark_the_widget_places_lands_on_its_paper() {
    // The offsets are the anm2's, so nothing here is checking our arithmetic against
    // itself: what it checks is that the game still places all eleven inside the sheet it
    // draws them on. A patch that moved one would show up as a symbol hanging off the edge.
    let Some((_, frames)) = real_frames() else {
        return;
    };
    let art = widget_source(&[MarkFill::Hard; BOSSES.len()], &frames).expect("a full widget");
    assert_eq!(
        art.marks.len(),
        11,
        "the widget the game draws has eleven marks; Delirium is the lobby's"
    );
    let paper = art.paper.rect.expect("the paper is a crop");
    for (sprite, x, y) in &art.marks {
        let r = sprite.rect.expect("a mark is a crop");
        assert!(
            *x >= 0 && *y >= 0,
            "a mark placed off the top-left: {x},{y}"
        );
        assert!(
            (*x as u32) + r.w <= paper.w && (*y as u32) + r.h <= paper.h,
            "a mark at {x},{y} hangs over a {}×{} paper",
            paper.w,
            paper.h
        );
    }
}

#[test]
fn the_composed_widget_is_the_paper_with_something_on_it() {
    // A negative result is a result, and silence is not one: before reading "the picture is
    // 96×96" as success, the same instrument has to show it can tell the composed picture
    // from the bare paper it started with.
    let Some((rs, frames)) = real_frames() else {
        return;
    };
    let compose = |fills: [MarkFill; BOSSES.len()]| {
        let art = widget_source(&fills, &frames).expect("a widget");
        let piece = |s: &catalog::SpriteRef| {
            let sheet = rs.read(&s.path)?;
            let r = s.rect?;
            crop_png(&sheet, r.x, r.y, r.w, r.h)
        };
        let paper = piece(&art.paper).expect("the paper");
        let marks: Vec<(Vec<u8>, i32, i32)> = art
            .marks
            .iter()
            .map(|(s, x, y)| (piece(s).expect("a mark"), *x, *y))
            .collect();
        let pieces: Vec<(&[u8], i32, i32)> = marks
            .iter()
            .map(|(p, x, y)| (p.as_slice(), *x, *y))
            .collect();
        overlay(&paper, &pieces).expect("a composition")
    };
    let bare = compose([MarkFill::None; BOSSES.len()]);
    let full = compose([MarkFill::Hard; BOSSES.len()]);
    assert_eq!(
        decode_rgba(&bare).map(|(w, h, _)| (w, h)),
        Some((96, 96)),
        "the composition keeps the paper's size"
    );
    assert_eq!(decode_rgba(&full).map(|(w, h, _)| (w, h)), Some((96, 96)));
    assert_ne!(
        decode_rgba(&bare).map(|(_, _, p)| p),
        decode_rgba(&full).map(|(_, _, p)| p),
        "eleven marks were laid on the paper and nothing changed"
    );
}
