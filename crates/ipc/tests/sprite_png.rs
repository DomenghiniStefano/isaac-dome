//! Cutting a piece out of a game sheet. Moved from `design-export` with B13: the app serves
//! the mark symbols and the character heads as crops, and a crop that takes the wrong cell is
//! a picture that looks plausible and is wrong.

use ipc::{crop_png, decode_rgba, overlay, trim_opaque};

fn encode(w: u32, h: u32, pixel: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(pixel).unwrap();
    }
    out
}

/// A `w`×`h` sheet whose left half is `left` and whose right half is `right`.
fn two_halves(w: u32, h: u32, left: [u8; 4], right: [u8; 4]) -> Vec<u8> {
    let pixel: Vec<u8> = (0..h)
        .flat_map(|_| (0..w).flat_map(move |x| if x < w / 2 { left } else { right }))
        .collect();
    encode(w, h, &pixel)
}

#[test]
fn the_crop_takes_exactly_the_requested_rectangle() {
    let sheet = two_halves(8, 4, [255, 0, 0, 255], [0, 255, 0, 255]);
    let green = crop_png(&sheet, 4, 0, 4, 4).expect("the right half");
    let (w, h, pixel) = decode_rgba(&green).expect("the crop is a PNG");
    assert_eq!((w, h), (4, 4));
    assert!(
        pixel.chunks_exact(4).all(|p| p == [0, 255, 0, 255]),
        "the crop took the wrong cell"
    );
}

#[test]
fn a_rect_past_the_edge_is_clipped_not_dropped() {
    let sheet = two_halves(4, 4, [1, 2, 3, 255], [1, 2, 3, 255]);
    let corner = crop_png(&sheet, 2, 2, 8, 8).expect("the bottom-right corner remains");
    let (w, h, _) = decode_rgba(&corner).unwrap();
    assert_eq!((w, h), (2, 2));
}

#[test]
fn past_the_sheet_edge_no_empty_png_is_invented() {
    let sheet = two_halves(4, 4, [1, 2, 3, 255], [1, 2, 3, 255]);
    assert!(
        crop_png(&sheet, 4, 0, 4, 4).is_none(),
        "x right on the edge"
    );
    assert!(crop_png(&sheet, 0, 9, 4, 4).is_none(), "y past the sheet");
    assert!(crop_png(&sheet, 0, 0, 0, 4).is_none(), "zero width");
}

#[test]
fn something_that_is_not_a_png_is_not_cropped() {
    assert!(crop_png(b"nothing", 0, 0, 1, 1).is_none());
    assert!(decode_rgba(b"nothing").is_none());
}

/// A `w`×`h` picture, fully transparent except for an opaque `bw`×`bh` block at `(bx, by)`.
fn block(w: u32, h: u32, bx: u32, by: u32, bw: u32, bh: u32) -> Vec<u8> {
    let pixel: Vec<u8> = (0..h)
        .flat_map(|y| {
            (0..w).flat_map(move |x| {
                let inside = x >= bx && x < bx + bw && y >= by && y < by + bh;
                if inside {
                    [9, 8, 7, 255]
                } else {
                    [0, 0, 0, 0]
                }
            })
        })
        .collect();
    encode(w, h, &pixel)
}

// `trim_opaque` exists because the game's own crops are not centred on their drawing.
// `minimap_icons.anm2` declares 16x16 for every icon with `XPivot="0" YPivot="0"`, and the
// drawing sits in the upper-left of it: measured 2026-09-20 on the installed game,
// `IconTreasureRoom` covers x 3-10, y 3-8 and `IconBoss` x 2-10, y 2-9. Centring the tile
// therefore puts the drawing 3px left and 5px high of where the eye expects it. Trimming to
// the opaque pixels makes "centred" mean the drawing, whatever the sheet does later.

#[test]
fn the_transparent_margin_goes_and_the_drawing_stays() {
    let picture = block(16, 16, 3, 2, 8, 6);
    let trimmed = trim_opaque(&picture).expect("there is a drawing in there");
    let (w, h, pixel) = decode_rgba(&trimmed).expect("the trim is a PNG");
    assert_eq!((w, h), (8, 6), "the box is the drawing's, not the tile's");
    assert!(
        pixel.chunks_exact(4).all(|p| p == [9, 8, 7, 255]),
        "the trim kept a transparent row or column"
    );
}

#[test]
fn a_drawing_that_touches_every_edge_comes_back_whole() {
    let picture = block(4, 4, 0, 0, 4, 4);
    let trimmed = trim_opaque(&picture).expect("nothing to trim, but still a picture");
    let (w, h, _) = decode_rgba(&trimmed).expect("the trim is a PNG");
    assert_eq!((w, h), (4, 4));
}

#[test]
fn a_picture_with_nothing_in_it_trims_to_nothing() {
    // Not an empty PNG: the caller keeps the untrimmed crop, and the cell falls back to our
    // own drawing rather than showing a zero-sized image.
    let empty = block(8, 8, 0, 0, 0, 0);
    assert!(trim_opaque(&empty).is_none());
}

#[test]
fn something_that_is_not_a_png_is_not_trimmed() {
    assert!(trim_opaque(b"nothing").is_none());
}

// `overlay` exists for one picture: the completion widget, which the game draws as a single
// paper with eleven marks laid on it, each at its own place (measured 2026-09-21 on the
// installed game). Eleven sprites positioned in CSS would put the game's coordinates in a
// stylesheet; one composed picture keeps them where the rest of the game knowledge is.

/// A `w`×`h` picture filled with one colour.
fn flat(w: u32, h: u32, colour: [u8; 4]) -> Vec<u8> {
    encode(w, h, &(0..w * h).flat_map(|_| colour).collect::<Vec<u8>>())
}

fn pixel_at(png: &[u8], x: u32, y: u32) -> [u8; 4] {
    let (w, _, p) = decode_rgba(png).expect("a PNG");
    let i = ((y * w + x) as usize) * 4;
    [p[i], p[i + 1], p[i + 2], p[i + 3]]
}

#[test]
fn a_piece_lands_where_it_is_placed_and_nowhere_else() {
    let base = flat(8, 8, [10, 10, 10, 255]);
    let mark = flat(2, 2, [200, 0, 0, 255]);
    let out = overlay(&base, &[(mark.as_slice(), 3, 5)]).expect("a composition");
    assert_eq!(
        (decode_rgba(&out).unwrap().0, decode_rgba(&out).unwrap().1),
        (8, 8),
        "the composition keeps the base's size, never the union of the pieces"
    );
    assert_eq!(pixel_at(&out, 3, 5), [200, 0, 0, 255]);
    assert_eq!(pixel_at(&out, 4, 6), [200, 0, 0, 255]);
    assert_eq!(
        pixel_at(&out, 2, 5),
        [10, 10, 10, 255],
        "one column left of it"
    );
    assert_eq!(pixel_at(&out, 3, 4), [10, 10, 10, 255], "one row above it");
}

#[test]
fn a_transparent_pixel_of_a_piece_leaves_the_base_showing() {
    // The marks are pixel art on a transparent tile: a piece that replaced its whole
    // rectangle would punch eleven holes in the paper.
    let base = flat(4, 4, [10, 20, 30, 255]);
    let hole = block(2, 2, 0, 0, 1, 1);
    let out = overlay(&base, &[(hole.as_slice(), 1, 1)]).expect("a composition");
    assert_eq!(pixel_at(&out, 1, 1), [9, 8, 7, 255], "the opaque pixel");
    assert_eq!(
        pixel_at(&out, 2, 2),
        [10, 20, 30, 255],
        "the transparent one"
    );
}

#[test]
fn a_half_transparent_pixel_is_blended_not_chosen() {
    // The paper's own shadow carries partial alpha, and so will anything laid over it.
    let base = flat(2, 2, [0, 0, 0, 255]);
    let half = flat(1, 1, [255, 255, 255, 128]);
    let out = overlay(&base, &[(half.as_slice(), 0, 0)]).expect("a composition");
    let [r, g, b, a] = pixel_at(&out, 0, 0);
    assert_eq!(a, 255, "the base was opaque, so the result is");
    for c in [r, g, b] {
        assert!(
            (127..=129).contains(&c),
            "half of white over black is grey, got {c}"
        );
    }
}

#[test]
fn pieces_are_laid_in_the_order_they_are_given() {
    let base = flat(2, 2, [0, 0, 0, 255]);
    let red = flat(2, 2, [255, 0, 0, 255]);
    let blue = flat(2, 2, [0, 0, 255, 255]);
    let out =
        overlay(&base, &[(red.as_slice(), 0, 0), (blue.as_slice(), 0, 0)]).expect("a composition");
    assert_eq!(
        pixel_at(&out, 0, 0),
        [0, 0, 255, 255],
        "the last one is on top"
    );
}

#[test]
fn a_piece_that_hangs_over_an_edge_is_clipped_not_dropped() {
    // Same rule as `crop_png`: the anm2 files occasionally place a layer past the actor's
    // own box, and losing the mark over that would be worse than drawing part of it.
    let base = flat(4, 4, [0, 0, 0, 255]);
    let mark = flat(4, 4, [1, 2, 3, 255]);
    let out = overlay(&base, &[(mark.as_slice(), -2, 3)]).expect("a composition");
    assert_eq!(
        pixel_at(&out, 0, 3),
        [1, 2, 3, 255],
        "the part that is on the base"
    );
    assert_eq!(
        pixel_at(&out, 2, 3),
        [0, 0, 0, 255],
        "past the piece's own width"
    );
    assert_eq!(
        pixel_at(&out, 0, 2),
        [0, 0, 0, 255],
        "above where it starts"
    );
}

#[test]
fn a_piece_entirely_off_the_base_changes_nothing_and_fails_nothing() {
    let base = flat(4, 4, [7, 7, 7, 255]);
    let mark = flat(2, 2, [1, 2, 3, 255]);
    let out = overlay(&base, &[(mark.as_slice(), 9, 9)]).expect("still a composition");
    let (_, _, p) = decode_rgba(&out).unwrap();
    assert!(p.chunks_exact(4).all(|q| q == [7, 7, 7, 255]));
}

#[test]
fn no_pieces_is_the_base_itself() {
    let base = flat(3, 3, [4, 5, 6, 255]);
    let out = overlay(&base, &[]).expect("a composition");
    assert_eq!(decode_rgba(&out).unwrap().2, decode_rgba(&base).unwrap().2);
}

#[test]
fn a_base_that_is_not_a_png_composes_nothing_and_a_broken_piece_is_skipped() {
    let base = flat(2, 2, [1, 1, 1, 255]);
    assert!(overlay(b"nothing", &[]).is_none(), "no base, no picture");
    // A piece is one of many: the paper without one mark still says most of the truth,
    // while refusing the whole widget over it says nothing at all.
    let out = overlay(&base, &[(b"nothing".as_slice(), 0, 0)]).expect("the base survives");
    assert_eq!(pixel_at(&out, 0, 0), [1, 1, 1, 255]);
}
