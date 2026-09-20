//! Cutting a piece out of a game sheet. Moved from `design-export` with B13: the app serves
//! the mark symbols and the character heads as crops, and a crop that takes the wrong cell is
//! a picture that looks plausible and is wrong.

use ipc::{crop_png, decode_rgba, trim_opaque};

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
