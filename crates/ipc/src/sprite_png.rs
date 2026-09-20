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
    let row_bytes = (w as usize) * 4;
    let cut: Vec<u8> = (0..h)
        .flat_map(|row| {
            let from = (((y + row) as usize) * (sheet_w as usize) + x as usize) * 4;
            pixel[from..from + row_bytes].iter().copied()
        })
        .collect();
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(&cut).ok()?;
    }
    Some(out)
}
