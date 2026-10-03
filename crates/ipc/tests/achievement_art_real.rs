//! The achievement backing against the installed game: the popup's anm2 reads, its paper is a
//! real sheet in the archives, and every achievement's drawing lands on it whole. The
//! synthetic `achievement_art.rs` pins the rules; this says whether the game still agrees.

use catalog::Catalog;
use ipc::{achievement_backing, crop_png, decode_rgba, overlay, ACHIEVEMENT_ANM2};

fn real() -> Option<(unpack::ResourceSet, ipc::AchievementBacking)> {
    let dir = test_support::packed_dir()?;
    let rs = unpack::ResourceSet::open(&dir);
    let frames = rs
        .read(ACHIEVEMENT_ANM2)
        .and_then(|b| catalog::anm2_frames(&b))
        .expect("the popup's anm2 is in the archives");
    let backing = achievement_backing(&frames).expect("the popup has a paper and a drawing");
    Some((rs, backing))
}

fn paper_png(rs: &unpack::ResourceSet, backing: &ipc::AchievementBacking) -> Vec<u8> {
    let sheet = rs.read(&backing.paper.path).expect("the paper's sheet");
    let r = backing.paper.rect.expect("the paper is a crop");
    crop_png(&sheet, r.x, r.y, r.w, r.h).expect("inside the sheet")
}

#[test]
fn every_drawing_lands_whole_on_the_paper() {
    // Measured on the installed game on 2026-10-03: the anm2 declares a 272×176 crop on a
    // 263×176 sheet — the crop clamps to the sheet — and every drawing is 263×176, placed at
    // the paper's own origin.
    let Some((rs, backing)) = real() else {
        return;
    };
    let (pw, ph, _) = decode_rgba(&paper_png(&rs, &backing)).expect("a PNG");
    let c = Catalog::build(|p| rs.read(p));
    let drawings: Vec<_> = c.achievements().collect();
    assert!(drawings.len() > 600, "the catalog read the achievements");
    let (x, y) = backing.drawing_at;
    for a in drawings {
        let Some(png) = rs.read(&a.sprite.path) else {
            continue;
        };
        let (w, h, _) = decode_rgba(&png).expect("a drawing is a PNG");
        assert!(
            x >= 0 && y >= 0 && (x as u32) + w <= pw && (y as u32) + h <= ph,
            "{}: a {w}×{h} drawing at {x},{y} hangs over a {pw}×{ph} paper",
            a.sprite.path
        );
    }
}

#[test]
fn the_composed_picture_is_the_paper_with_the_drawing_on_it() {
    // Silence is not a result: before reading "263×176" as success, the same composition has
    // to tell a drawing on the paper from the bare paper.
    let Some((rs, backing)) = real() else {
        return;
    };
    let paper = paper_png(&rs, &backing);
    let c = Catalog::build(|p| rs.read(p));
    let drawing = c
        .achievements()
        .find_map(|a| rs.read(&a.sprite.path))
        .expect("one drawing");
    let (x, y) = backing.drawing_at;
    let composed = overlay(&paper, &[(drawing.as_slice(), x, y)]).expect("a composition");
    assert_eq!(
        decode_rgba(&composed).map(|(w, h, _)| (w, h)),
        decode_rgba(&paper).map(|(w, h, _)| (w, h)),
        "the composition keeps the paper's size"
    );
    assert_ne!(
        decode_rgba(&composed).map(|(_, _, p)| p),
        decode_rgba(&paper).map(|(_, _, p)| p),
        "a drawing was laid on the paper and nothing changed"
    );
}
