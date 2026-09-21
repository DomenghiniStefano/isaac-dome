//! The boss portraits of the installed game, against the crop the catalog declares for them.
//!
//! The defect this guards (B70): six portraits are 384x192 — the boss on the left, the
//! rubble it climbs out of on the right — and `Portrait_Mother.png` is 480x440, her body
//! above and her hands below. Served whole, those rows draw a creature *and a second thing
//! beside it*. The crop comes from the game's own versus screens, and what is checked here
//! is that it agrees with the files: never bigger than the picture it cuts, smaller exactly
//! where the file holds more than one drawing.
//!
//! Sizes are read straight out of each PNG's IHDR rather than through our own decoder, so
//! the expected value comes from the file and not from the code under test.

use catalog::Catalog;
use unpack::ResourceSet;

/// Width and height out of the IHDR: the eight bytes after the signature and the length and
/// type fields. `None` for anything that is not a PNG.
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let signature = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes.len() < 24 || bytes[..8] != signature || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let n =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    Some((n(16), n(20)))
}

/// Every boss whose portrait the archives actually hold: its file name, the file's size and
/// the rectangle the catalog says to cut.
fn portraits(c: &Catalog, rs: &ResourceSet) -> Vec<(String, (u32, u32), Option<catalog::Rect>)> {
    c.bosses()
        .filter_map(|b| {
            let bytes = rs.read(&b.portrait.path)?;
            let size = png_size(&bytes)?;
            let name = b.portrait.path.rsplit('/').next()?.to_string();
            Some((name, size, b.portrait.rect))
        })
        .collect()
}

/// The crop as the picture actually comes out: the rectangle stops at the edge of the file,
/// in `crop_png` and in the game alike, so that is what "cut" has to be measured against.
fn effective(r: catalog::Rect, (w, h): (u32, u32)) -> (u32, u32, u32, u32) {
    (r.x, r.y, r.w.min(w - r.x), r.h.min(h - r.y))
}

fn real() -> Option<(Catalog, ResourceSet)> {
    let packed = test_support::packed_dir()?;
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    Some((c, rs))
}

#[test]
fn every_portrait_is_cut_from_inside_its_own_file() {
    let Some((c, rs)) = real() else { return };
    let rows = portraits(&c, &rs);
    if rows.is_empty() {
        test_support::skip("no boss portrait could be read from the archives");
        return;
    }
    eprintln!("boss portraits read: {}", rows.len());
    for (name, (w, h), rect) in &rows {
        let Some(r) = rect else {
            panic!("{name}: no crop at all, the versus screen was not read");
        };
        // The corner, not the far edge. A rectangle that runs past the end of the file is
        // the game's own doing and costs nothing — `versusscreen_dogma.anm2` asks for
        // 208x192 of a 192x192 file (measured 2026-09-22), and both the game and
        // `crop_png` stop at the edge. A corner outside the file is the other thing
        // entirely: it would serve no picture at all.
        assert!(
            r.x < *w && r.y < *h,
            "{name}: the crop {r:?} starts outside a {w}x{h} file"
        );
    }
}

#[test]
fn a_crop_smaller_than_its_file_leaves_a_whole_second_drawing_behind() {
    let Some((c, rs)) = real() else { return };
    let rows = portraits(&c, &rs);
    if rows.is_empty() {
        test_support::skip("no boss portrait could be read from the archives");
        return;
    }
    // Why a sheet is cut and a picture is not: the leftover is another drawing of the same
    // size, never a margin. Pin's file is two 192 squares, Mother's is two 220 bands. A
    // crop that took *part* of one drawing would land here, and that is the failure this
    // test exists for — a boss shown as a fragment of himself.
    for (name, (w, h), rect) in &rows {
        let Some(r) = rect else { continue };
        let (_, _, cw, ch) = effective(*r, (*w, *h));
        if (r.x, r.y, cw, ch) == (0, 0, *w, *h) {
            continue;
        }
        assert!(
            *w >= cw * 2 || *h >= ch * 2,
            "{name}: {w}x{h} cut to {cw}x{ch} — that is a piece of one drawing, not one of two"
        );
    }
}

#[test]
fn the_sheets_are_cut_down_and_the_single_pictures_are_left_alone() {
    let Some((c, rs)) = real() else { return };
    let rows = portraits(&c, &rs);
    if rows.is_empty() {
        test_support::skip("no boss portrait could be read from the archives");
        return;
    }
    let mut cut: Vec<&str> = Vec::new();
    for (name, (w, h), rect) in &rows {
        let Some(r) = rect else { continue };
        let (_, _, cw, ch) = effective(*r, (*w, *h));
        if (r.x, r.y, cw, ch) != (0, 0, *w, *h) {
            cut.push(name);
        }
    }
    cut.sort_unstable();
    eprintln!("portraits cut down to a piece of themselves: {cut:?}");
    // The vacuity guard: on a build where every portrait happened to be a single square,
    // the property above would hold with nothing to say. The seven are the point of the
    // entry, so at least one of them has to be here for the check to mean anything.
    assert!(
        !cut.is_empty(),
        "no portrait is cut: either the scenes were not read, or the sheets are gone"
    );
    // Measured on the 2026-09 build: Pin is the one the defect was reported on, 384x192
    // with the rubble on the right half.
    if let Some((_, size, rect)) = rows.iter().find(|(n, _, _)| n == "Portrait_62.0_Pin.png") {
        assert_eq!(
            *size,
            (384, 192),
            "Pin's sheet changed shape: re-measure B70"
        );
        let r = rect.expect("Pin has a crop");
        assert_eq!(
            (r.x, r.y, r.w, r.h),
            (0, 0, 192, 192),
            "Pin is the left square of his sheet, the right one is his hole"
        );
    }
}
