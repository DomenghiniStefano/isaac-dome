//! Probe: composes the completion widget from the installed game and writes the PNGs, so
//! the emblem can be looked at without opening the app.
//!
//! The icon protocol only answers inside the Tauri window, so the browser the rest of the
//! UI is checked in cannot draw this picture at all. This is the way to see it — and the way
//! to see it again after a patch moves a layer, which is the failure this whole file exists
//! to make visible: a mark off its place still composes, still serves, and looks almost
//! right.
//!
//! `cargo run -p ipc --example widget_png -- <out-dir>` (default: `samples/sprites`).

use ipc::{
    crop_png, overlay, widget_source, MarkFill, MarkFrames, BOSSES, LOBBY_ANM2, WIDGET_ANM2,
};

fn main() {
    let packed = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/packed");
    let out = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/sprites")
        });
    let rs = unpack::ResourceSet::open(&packed);
    let read = |p: &str| {
        rs.read(p)
            .and_then(|b| catalog::anm2_frames(&b))
            .unwrap_or_default()
    };
    let frames = MarkFrames {
        widget: read(WIDGET_ANM2),
        lobby: read(LOBBY_ANM2),
    };
    if frames.widget.is_empty() {
        println!("skip: no {WIDGET_ANM2} — needs samples/packed");
        return;
    }
    if std::fs::create_dir_all(&out).is_err() {
        println!("skip: cannot write to {}", out.display());
        return;
    }
    // Three states worth looking at: nothing, a scatter, and everything — the last one is
    // the only one that asks for the bloodied paper.
    let mut some = [MarkFill::None; BOSSES.len()];
    for (i, slot) in some.iter_mut().enumerate() {
        *slot = match i % 3 {
            0 => MarkFill::Hard,
            1 => MarkFill::Normal,
            _ => MarkFill::None,
        };
    }
    for (name, fills) in [
        ("widget_none", [MarkFill::None; BOSSES.len()]),
        ("widget_some", some),
        ("widget_all", [MarkFill::Hard; BOSSES.len()]),
    ] {
        let Some(art) = widget_source(&fills, &frames) else {
            println!("{name}: no paper");
            continue;
        };
        let piece = |s: &catalog::SpriteRef| {
            let sheet = rs.read(&s.path)?;
            let r = s.rect?;
            crop_png(&sheet, r.x, r.y, r.w, r.h)
        };
        let Some(paper) = piece(&art.paper) else {
            println!("{name}: the paper did not crop");
            continue;
        };
        let marks: Vec<(Vec<u8>, i32, i32)> = art
            .marks
            .iter()
            .filter_map(|(s, x, y)| Some((piece(s)?, *x, *y)))
            .collect();
        let pieces: Vec<(&[u8], i32, i32)> = marks
            .iter()
            .map(|(p, x, y)| (p.as_slice(), *x, *y))
            .collect();
        match overlay(&paper, &pieces) {
            Some(png) => {
                let path = out.join(format!("{name}.png"));
                match std::fs::write(&path, &png) {
                    Ok(()) => println!(
                        "{name}: {} marks, {} bytes -> {}",
                        pieces.len(),
                        png.len(),
                        path.display()
                    ),
                    Err(e) => println!("{name}: {e}"),
                }
            }
            None => println!("{name}: the composition failed"),
        }
    }
}
