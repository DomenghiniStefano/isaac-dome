//! A piece of a game sheet, as a PNG of its own.
//!
//! The game ships some pictures joined in one sheet — `completion_widget.png`, the co-op
//! menu's heads — and the anm2 files say where to cut. The app serves those pieces through
//! its icon protocol, so the cut lives in this pure crate: bytes in, bytes out. It moved
//! here from `crates/design-export`, which imported it back until that crate was removed
//! on 2026-09-20.

/// Decodes a PNG to 8-bit RGBA, whatever its internal format (palette, grayscale, no alpha
/// channel): the game's sprites aren't all the same type. `(width, height, pixels)`.
pub fn decode_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut d = png::Decoder::new(bytes);
    d.set_transformations(
        png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
    );
    let mut reader = d.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);
    let expected = (w as usize) * (h as usize);
    let pixel = match info.color_type {
        png::ColorType::Rgba => {
            buf.truncate(expected * 4);
            buf
        }
        // Grayscale with alpha: two channels, expanded to four.
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .take(expected)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        // The transformations above never hand these back; if one does, it isn't a sprite
        // we know how to read.
        png::ColorType::Grayscale | png::ColorType::Rgb | png::ColorType::Indexed => return None,
    };
    (pixel.len() >= expected * 4).then_some((w, h, pixel))
}

/// Crops a rectangle out of a sheet and turns it into a standalone PNG.
///
/// A rectangle that runs past the sheet's edge is **clipped to what's there**, not rejected:
/// the anm2 files occasionally declare cells bigger than the sheet, and losing the icon over
/// that would be worse than giving it a bit smaller. If nothing is left, `None`.
pub fn crop_png(sheet: &[u8], x: u32, y: u32, w: u32, h: u32) -> Option<Vec<u8>> {
    let (sheet_w, sheet_h, pixel) = decode_rgba(sheet)?;
    if x >= sheet_w || y >= sheet_h {
        return None;
    }
    let w = w.min(sheet_w - x);
    let h = h.min(sheet_h - y);
    if w == 0 || h == 0 {
        return None;
    }
    encode_rgba(w, h, &cut(&pixel, sheet_w, x, y, w, h))
}

/// Shrinks a picture to the box its opaque pixels occupy. `None` when the bytes aren't a PNG,
/// and when there is nothing in them — a caller keeps the untrimmed picture rather than
/// serving one of zero size.
///
/// **A crop the game declares is not centred on the drawing inside it.** Every animation in
/// `minimap_icons.anm2` declares 16x16 with `XPivot="0" YPivot="0"`, and the drawing sits in
/// the upper-left corner of that square: measured on the installed game on 2026-09-20,
/// `IconTreasureRoom` covers x 3-10 and y 3-8 of its sixteen, `IconBoss` x 2-10 and y 2-9. A
/// cell that centres the square therefore puts the drawing three pixels left and five high of
/// where it is looked for, which is what it did.
///
/// The alternative was an offset measured on today's sheet and written into a stylesheet — a
/// constant that is right until a patch moves an icon, and then wrong with nothing to say so.
/// The box here is read from the picture each time, so it survives the sheet changing.
pub fn trim_opaque(png: &[u8]) -> Option<Vec<u8>> {
    let (w, h, pixel) = decode_rgba(png)?;
    let (mut left, mut top) = (w, h);
    let (mut right, mut bottom) = (0, 0);
    for y in 0..h {
        for x in 0..w {
            // Any alpha at all is drawing. A threshold would decide that the faintest edge of
            // an anti-aliased sprite is margin, and pixel art's edges are the sprite.
            if pixel[((y * w + x) as usize) * 4 + 3] == 0 {
                continue;
            }
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
        }
    }
    if left > right || top > bottom {
        return None;
    }
    let (tw, th) = (right - left + 1, bottom - top + 1);
    encode_rgba(tw, th, &cut(&pixel, w, left, top, tw, th))
}

/// The pixels of one rectangle of `pixel`, which is `sheet_w` wide. The rectangle is the
/// caller's to bound: both callers above have already clipped it to the picture.
fn cut(pixel: &[u8], sheet_w: u32, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
    let row_bytes = (w as usize) * 4;
    (0..h)
        .flat_map(|row| {
            let from = (((y + row) as usize) * (sheet_w as usize) + x as usize) * 4;
            pixel[from..from + row_bytes].iter().copied()
        })
        .collect()
}

fn encode_rgba(w: u32, h: u32, pixel: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(pixel).ok()?;
    }
    Some(out)
}

/// Lays `pieces` over `base` and encodes the result, which keeps **the base's size**: a
/// piece is placed on the picture, it never enlarges it.
///
/// Each piece is `(png, x, y)`, where `x, y` is its top-left in the base's pixels and may be
/// negative — the game places a layer around a pivot, so an offset that runs off the edge is
/// ordinary. Anything outside the base is clipped, the way `crop_png` clips: losing a mark
/// over a rectangle that hangs over the corner would be worse than drawing the part of it
/// that fits.
///
/// **A piece that isn't a PNG is skipped, while a base that isn't one is `None`.** They are
/// not the same failure: the widget without one of its eleven marks still says most of the
/// truth, and the widget without its paper is not a picture at all.
///
/// The blend is source-over on straight (un-premultiplied) alpha. Replacing the rectangle
/// instead would punch a hole in the paper for every mark, because a mark is pixel art on a
/// transparent tile.
pub fn overlay(base: &[u8], pieces: &[(&[u8], i32, i32)]) -> Option<Vec<u8>> {
    let (w, h, mut pixel) = decode_rgba(base)?;
    for &(png, at_x, at_y) in pieces {
        let Some((pw, ph, src)) = decode_rgba(png) else {
            continue;
        };
        for y in 0..ph {
            let Some(dy) = offset(at_y, y, h) else {
                continue;
            };
            for x in 0..pw {
                let Some(dx) = offset(at_x, x, w) else {
                    continue;
                };
                let s = ((y * pw + x) as usize) * 4;
                let d = ((dy * w + dx) as usize) * 4;
                blend(&mut pixel[d..d + 4], &src[s..s + 4]);
            }
        }
    }
    encode_rgba(w, h, &pixel)
}

/// Where pixel `i` of a piece placed at `at` lands, or `None` when that is off the picture.
fn offset(at: i32, i: u32, limit: u32) -> Option<u32> {
    let p = at.checked_add(i32::try_from(i).ok()?)?;
    u32::try_from(p).ok().filter(|&p| p < limit)
}

/// Source-over, rounded: `out = src + dst * (1 - a)`, on straight alpha.
fn blend(dst: &mut [u8], src: &[u8]) {
    let a = u32::from(src[3]);
    if a == 0 {
        return;
    }
    if a == 255 {
        dst.copy_from_slice(src);
        return;
    }
    let keep = 255 - a;
    for c in 0..3 {
        let over = u32::from(src[c]) * a + u32::from(dst[c]) * keep;
        dst[c] = ((over + 127) / 255) as u8;
    }
    let out_a = a * 255 + u32::from(dst[3]) * keep;
    dst[3] = ((out_a + 127) / 255) as u8;
}
